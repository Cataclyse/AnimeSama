use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

static TRAILER_CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, Option<String>>> {
    TRAILER_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_anime_trailer(title: &str) -> Option<String> {
    let clean = title.trim();
    if clean.is_empty() {
        return None;
    }
    let key = clean.to_lowercase();
    {
        let cache = get_cache().lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let encoded: String = clean
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "%20".to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect();

    let url = format!("https://kitsu.io/api/edge/anime?filter[text]={}&page[limit]=1", encoded);

    let res = ureq::get(&url)
        .set("Accept", "application/vnd.api+json")
        .set("User-Agent", "Mozilla/5.0 AnimeSama/1.0")
        .timeout(std::time::Duration::from_secs(4))
        .call();

    let youtube_id = if let Ok(resp) = res {
        if let Ok(json) = resp.into_json::<serde_json::Value>() {
            json.get("data")
                .and_then(|d| d.get(0))
                .and_then(|first| first.get("attributes"))
                .and_then(|attr| attr.get("youtubeVideoId"))
                .and_then(|y| y.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let mut cache = get_cache().lock().unwrap();
    cache.insert(key, youtube_id.clone());
    youtube_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailer_empty_title() {
        assert_eq!(get_anime_trailer(""), None);
        assert_eq!(get_anime_trailer("   "), None);
    }
}
