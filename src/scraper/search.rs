use crate::catalogue::CardItem;
use super::home::parse_cards_from_container;

pub fn execute_search(query: &str) -> Result<Vec<CardItem>, String> {
    let url = format!(
        "https://anime-sama.to/catalogue/?search={}",
        urlencode(query)
    );

    let html = super::http::fetch_site_text(
    &url,
    10,
    4 * 1024 * 1024,
    None,
)?;

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
