use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;
use url::Url;

type ApiError = (u16, &'static str);

fn media_type(url: &Url) -> &'static str {
    let path = url.path().to_ascii_lowercase();

    if path.ends_with(".m3u8") {
        "hls"
    } else if path.ends_with(".mp4") {
        "mp4"
    } else if path.ends_with(".webm") {
        "webm"
    } else {
        "unknown"
    }
}

fn valid_source(raw: &str) -> Option<Url> {
    let url = Url::parse(raw).ok()?;
    let host = url.host_str()?;

    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || host.parse::<std::net::IpAddr>().is_ok()
        || !host.contains('.')
        || host.ends_with(".local")
    {
        return None;
    }

    Some(url)
}

pub fn resolve(
    params: &HashMap<String, String>,
) -> Result<Value, ApiError> {
    let title = params.get("title")
        .map(|v| v.trim())
        .filter(|v| !v.is_empty() && v.len() <= 160)
        .ok_or((400, "Titre invalide"))?;

    if title.chars().any(char::is_control) {
        return Err((400, "Titre invalide"));
    }

    let season = params.get("season")
        .map(String::as_str)
        .unwrap_or("saison1");

    if season.is_empty()
        || season.len() > 32
        || !season.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-'
        })
    {
        return Err((400, "Saison invalide"));
    }

    let version = params.get("version")
        .map(String::as_str)
        .unwrap_or("vostfr");

    if version != "vf" && version != "vostfr" {
        return Err((400, "Langue invalide"));
    }

    let episode: u32 = params.get("episode")
        .ok_or((400, "Épisode manquant"))?
        .parse()
        .map_err(|_| (400, "Épisode invalide"))?;

    if !(1..=1000).contains(&episode) {
        return Err((400, "Épisode invalide"));
    }

    // Destination fixe : aucune URL fournie par le client
    // n'est utilisée comme destination de requête HTTP.
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(3))
        .timeout_read(Duration::from_secs(65))
        .build();

    let response = agent
        .get("http://127.0.0.1:5087/api/getAnimeLink")
        .query("n", title)
        .query("s", season)
        .query("v", version)
        .call()
        .map_err(|_| (502, "AnimeSamaApi indisponible"))?;

    if response.status() != 200 {
        return Err((502, "Réponse Python inattendue"));
    }

    const MAX_RESPONSE: usize = 512 * 1024;

    let mut body = Vec::new();

    response.into_reader()
        .take((MAX_RESPONSE + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| (502, "Lecture API impossible"))?;

    if body.len() > MAX_RESPONSE {
        return Err((502, "Réponse API trop volumineuse"));
    }

    let entries: Vec<Value> = serde_json::from_slice(&body)
        .map_err(|_| (502, "JSON Python invalide"))?;

    let entry = entries.iter().find(|item| {
        let value = &item["episode"];

        value.as_u64() == Some(episode as u64)
            || value.as_str()
                .and_then(|s| s.parse::<u32>().ok())
                == Some(episode)
    }).ok_or((404, "Épisode absent des sources"))?;

    let raw_url = entry["url"].as_str()
        .ok_or((404, "URL vidéo absente"))?;

    let url = valid_source(raw_url)
        .ok_or((502, "Adresse vidéo non autorisée"))?;

    Ok(json!({
        "episode": episode,
        "source": "AnimeSamaApi",
        "type": media_type(&url),
        "host": url.host_str(),
        "url": url.as_str()
    }))
}


fn filter_sources(payload: &Value, episode: u32) -> Result<Value, ApiError> {
    use std::collections::HashSet;

    if payload["episode"].as_u64() != Some(episode as u64) {
        return Err((502, "Épisode Python incohérent"));
    }

    let candidates = payload["sources"]
        .as_array()
        .ok_or((502, "Liste Python invalide"))?;

    let mut output = Vec::new();
    let mut seen = HashSet::new();

    for item in candidates.iter().take(20) {
        let Some(raw) = item["url"].as_str() else {
            continue;
        };

        if raw.len() > 8192 {
            continue;
        }

        let Some(url) = valid_source(raw) else {
            continue;
        };

        if crate::proxy::security::validate_proxy_url(url.as_str())
            .is_err()
        {
            continue;
        }

        let kind = item["type"].as_str().unwrap_or("");
        if !matches!(kind, "hls" | "mp4")
            || media_type(&url) != kind
        {
            continue;
        }

        let health = item["health"].as_str().unwrap_or("");
        if !matches!(health, "verified" | "uncertain") {
            continue;
        }

        if !seen.insert(url.as_str().to_owned()) {
            continue;
        }

        let reader = item["reader"]
            .as_str()
            .filter(|s| s.len() <= 64)
            .unwrap_or("");

        let provider = item["provider"]
            .as_str()
            .filter(|s| s.len() <= 128)
            .unwrap_or("");

        output.push(json!({
            "reader": reader,
            "provider": provider,
            "host": url.host_str(),
            "type": kind,
            "health": health,
            "url": url.as_str()
        }));

        if output.len() >= 10 {
            break;
        }
    }

    Ok(json!({
        "episode": episode,
        "sources": output
    }))
}

pub fn sources(
    params: &HashMap<String, String>,
) -> Result<Value, ApiError> {
    let title = params.get("title")
        .map(|v| v.trim())
        .filter(|v| !v.is_empty() && v.len() <= 160)
        .ok_or((400, "Titre invalide"))?;

    if title.chars().any(char::is_control) {
        return Err((400, "Titre invalide"));
    }

    let season = params.get("season")
        .map(String::as_str)
        .unwrap_or("saison1");

    if season.is_empty()
        || season.len() > 32
        || !season.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-'
        })
    {
        return Err((400, "Saison invalide"));
    }

    let version = params.get("version")
        .map(String::as_str)
        .unwrap_or("vostfr");

    if version != "vf" && version != "vostfr" {
        return Err((400, "Langue invalide"));
    }

    let episode: u32 = params.get("episode")
        .ok_or((400, "Épisode manquant"))?
        .parse()
        .map_err(|_| (400, "Épisode invalide"))?;

    if !(1..=1000).contains(&episode) {
        return Err((400, "Épisode invalide"));
    }

    let mode = params.get("mode")
        .map(String::as_str)
        .unwrap_or("primary");

    if !matches!(mode, "primary" | "all") {
        return Err((400, "Mode invalide"));
    }

    // Destination Python fixe, locale et réservée au DEV.
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(Duration::from_secs(3))
        .timeout_read(Duration::from_secs(65))
        .build();

    let response = agent
        .get("http://127.0.0.1:5090/api/getEpisodeSources")
        .query("n", title)
        .query("s", season)
        .query("v", version)
        .query("e", &episode.to_string())
        .query("mode", mode)
        .call()
        .map_err(|_| (502, "AnimeSamaApi DEV indisponible"))?;

    const MAX_RESPONSE: usize = 128 * 1024;
    let mut body = Vec::new();

    response.into_reader()
        .take((MAX_RESPONSE + 1) as u64)
        .read_to_end(&mut body)
        .map_err(|_| (502, "Lecture Python impossible"))?;

    if body.len() > MAX_RESPONSE {
        return Err((502, "Réponse Python trop volumineuse"));
    }

    let payload: Value = serde_json::from_slice(&body)
        .map_err(|_| (502, "JSON Python invalide"))?;

    filter_sources(&payload, episode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_source_mode() {
        let params = HashMap::from([
            ("title".to_string(), "Frieren".to_string()),
            ("episode".to_string(), "1".to_string()),
            ("mode".to_string(), "invalid".to_string()),
        ]);

        assert_eq!(
            sources(&params).unwrap_err(),
            (400, "Mode invalide")
        );
    }

    #[test]
    fn identifies_hls_with_query() {
        let url = valid_source(
            "https://video.example.com/master.m3u8?token=abc"
        ).unwrap();

        assert_eq!(media_type(&url), "hls");
    }

    #[test]
    fn rejects_local_destinations() {
        assert!(valid_source(
            "http://127.0.0.1/video.mp4"
        ).is_none());

        assert!(valid_source(
            "https://192.168.1.1/video.mp4"
        ).is_none());

        assert!(valid_source(
            "https://localhost/video.mp4"
        ).is_none());
    }
}
