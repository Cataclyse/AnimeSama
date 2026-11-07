use crate::catalogue::CardItem;
use super::home::parse_cards_from_container;

pub fn execute_search(query: &str) -> Result<Vec<CardItem>, String> {
    let url = format!(
        "https://anime-sama.to/catalogue/?search={}",
        urlencode(query)
    );

    let resp = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| format!("Erreur réseau recherche: {}", e))?;

    let html = resp.into_string().map_err(|e| format!("Erreur lecture réponse: {}", e))?;

    Ok(parse_cards_from_container(&html, "list_catalog"))
}

pub fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn unescape_html(s: &str) -> String {
    s.replace("&#039;", "'")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}
