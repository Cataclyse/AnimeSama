use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipInterval {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipResponse {
    pub found: bool,
    pub mal_id: Option<u64>,
    pub op: Option<SkipInterval>,
    pub ed: Option<SkipInterval>,
}

static MAL_CACHE: OnceLock<Mutex<HashMap<String, Option<u64>>>> = OnceLock::new();
static SKIPS_CACHE: OnceLock<Mutex<HashMap<String, SkipResponse>>> = OnceLock::new();

fn get_mal_cache() -> &'static Mutex<HashMap<String, Option<u64>>> {
    MAL_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_skips_cache() -> &'static Mutex<HashMap<String, SkipResponse>> {
    SKIPS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn clean_anime_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let cleaned = lower
        .replace("(tv)", "")
        .replace("(vf)", "")
        .replace("(vostfr)", "");
    let mut parts: Vec<&str> = Vec::new();
    for token in cleaned.split_whitespace() {
        if token.starts_with("saison") || token.starts_with("season") {
            break;
        }
        parts.push(token);
    }
    let res = parts.join(" ").trim().to_string();
    if res.is_empty() {
        title.trim().to_string()
    } else {
        res
    }
}

pub fn resolve_mal_id(title: &str) -> Option<u64> {
    let clean = clean_anime_title(title);
    if clean.is_empty() {
        return None;
    }
    let key = clean.to_lowercase();
    {
        let cache = get_mal_cache().lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return *cached;
        }
    }

    let payload = serde_json::json!({
        "query": "query ($s: String) { Media(search: $s, type: ANIME) { idMal } }",
        "variables": { "s": clean }
    });

    let res = ureq::post("https://graphql.anilist.co")
        .set("Content-Type", "application/json")
        .set("Accept", "application/json")
        .set("User-Agent", "Mozilla/5.0 AnimeSama/1.0")
        .timeout(std::time::Duration::from_secs(4))
        .send_json(payload);

    let mal_id = if let Ok(resp) = res {
        if let Ok(json) = resp.into_json::<serde_json::Value>() {
            json.get("data")
                .and_then(|d| d.get("Media"))
                .and_then(|m| m.get("idMal"))
                .and_then(|id| id.as_u64())
        } else {
            None
        }
    } else {
        None
    };

    let mut cache = get_mal_cache().lock().unwrap();
    cache.insert(key, mal_id);
    mal_id
}

pub fn get_skip_times(title: &str, episode: u32) -> SkipResponse {
    let clean_title = title.trim();
    if clean_title.is_empty() || episode == 0 {
        return SkipResponse {
            found: false,
            mal_id: None,
            op: None,
            ed: None,
        };
    }

    let mal_id = match resolve_mal_id(clean_title) {
        Some(id) => id,
        None => {
            return SkipResponse {
                found: false,
                mal_id: None,
                op: None,
                ed: None,
            };
        }
    };

    let cache_key = format!("{}:{}", mal_id, episode);
    {
        let cache = get_skips_cache().lock().unwrap();
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
    }

    let url = format!(
        "https://api.aniskip.com/v2/skip-times/{}/{}?types[]=op&types[]=ed&types[]=mixed-op&types[]=recap&episodeLength=0",
        mal_id, episode
    );

    let res = ureq::get(&url)
        .set("Accept", "application/json")
        .set("User-Agent", "Mozilla/5.0 AnimeSama/1.0")
        .timeout(std::time::Duration::from_secs(4))
        .call();

    let mut op_interval: Option<SkipInterval> = None;
    let mut ed_interval: Option<SkipInterval> = None;
    let mut is_found = false;

    if let Ok(resp) = res {
        if let Ok(json) = resp.into_json::<serde_json::Value>() {
            if json.get("found").and_then(|f| f.as_bool()).unwrap_or(false) {
                if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
                    for item in results {
                        let skip_type = item.get("skipType").and_then(|s| s.as_str()).unwrap_or("");
                        let interval = item.get("interval");
                        let start = interval.and_then(|i| i.get("startTime")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let end = interval.and_then(|i| i.get("endTime")).and_then(|v| v.as_f64()).unwrap_or(0.0);

                        if end > start && end > 0.0 {
                            if (skip_type == "op" || skip_type == "mixed-op") && op_interval.is_none() {
                                op_interval = Some(SkipInterval { start, end });
                                is_found = true;
                            } else if skip_type == "ed" && ed_interval.is_none() {
                                ed_interval = Some(SkipInterval { start, end });
                                is_found = true;
                            }
                        }
                    }
                }
            }
        }
    }

    let result = SkipResponse {
        found: is_found,
        mal_id: Some(mal_id),
        op: op_interval,
        ed: ed_interval,
    };

    let mut cache = get_skips_cache().lock().unwrap();
    cache.insert(cache_key, result.clone());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_times_empty_title() {
        let resp = get_skip_times("", 1);
        assert!(!resp.found);
        assert_eq!(resp.mal_id, None);
        assert!(resp.op.is_none());
        assert!(resp.ed.is_none());
    }

    #[test]
    fn test_skip_times_zero_episode() {
        let resp = get_skip_times("Naruto", 0);
        assert!(!resp.found);
    }
}
