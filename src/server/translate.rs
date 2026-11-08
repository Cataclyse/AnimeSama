use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type TranslationCache = Arc<Mutex<HashMap<(String, String), String>>>;

pub fn new_translation_cache() -> TranslationCache {
    Arc::new(Mutex::new(HashMap::new()))
}

fn url_encode(input: &str) -> String {
    let mut encoded = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

pub fn translate_text(cache: &TranslationCache, text: &str, target_lang: &str) -> String {
    let clean_text = text.trim();
    let target = target_lang.trim().to_lowercase();
    if clean_text.is_empty() || target.is_empty() || target == "fr" {
        return clean_text.to_string();
    }

    let key = (target.clone(), clean_text.to_string());
    {
        if let Ok(c) = cache.lock() {
            if let Some(cached) = c.get(&key) {
                return cached.clone();
            }
        }
    }

    let encoded = url_encode(clean_text);
    let google_url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target, encoded
    );

    let mut result: Option<String> = None;

    if let Ok(resp) = ureq::get(&google_url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .timeout(std::time::Duration::from_secs(5))
        .call()
    {
        if let Ok(body) = resp.into_string() {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(arr) = parsed.get(0).and_then(|v| v.as_array()) {
                    let mut combined = String::new();
                    for item in arr {
                        if let Some(chunk) = item.get(0).and_then(|v| v.as_str()) {
                            combined.push_str(chunk);
                        }
                    }
                    if !combined.is_empty() {
                        result = Some(combined);
                    }
                }
            }
        }
    }

    if result.is_none() {
        let mymemory_url = format!(
            "https://api.mymemory.translated.net/get?q={}&langpair=fr|{}",
            encoded, target
        );
        if let Ok(resp) = ureq::get(&mymemory_url)
            .set("User-Agent", "Mozilla/5.0")
            .timeout(std::time::Duration::from_secs(4))
            .call()
        {
            if let Ok(body) = resp.into_string() {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(t) = parsed
                        .get("responseData")
                        .and_then(|d| d.get("translatedText"))
                        .and_then(|v| v.as_str())
                    {
                        if !t.is_empty() && !t.starts_with("MYMEMORY WARNING") {
                            result = Some(t.to_string());
                        }
                    }
                }
            }
        }
    }

    let final_translation = result.unwrap_or_else(|| clean_text.to_string());

    if let Ok(mut c) = cache.lock() {
        if c.len() > 3000 {
            c.clear();
        }
        c.insert(key, final_translation.clone());
    }

    final_translation
}

pub fn translate_batch(cache: &TranslationCache, texts: &[String], target_lang: &str) -> Vec<String> {
    texts
        .iter()
        .map(|t| translate_text(cache, t, target_lang))
        .collect()
}
