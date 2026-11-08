use super::handlers::*;
use crate::user::HistoryEntry;
use serde_json::json;

#[test]
fn test_extract_cloud_history_numbers_and_strings() {
    let cloud = json!({
        "history": {
            "nom": ["Frieren", "Solo Leveling"],
            "url": ["/catalogue/frieren/saison1/vostfr/", "/catalogue/solo-leveling/saison1/vostfr/"],
            "num": [2, "5"],
            "ep": ["Episode 2", "Episode 5"]
        }
    });

    let entries = extract_cloud_history(&cloud);
    let frieren = entries.iter().find(|e| e.slug == "frieren").unwrap();
    assert_eq!(frieren.episode, 2);

    let solo = entries.iter().find(|e| e.slug == "solo-leveling").unwrap();
    assert_eq!(solo.episode, 5);
}

#[test]
fn test_extract_cloud_history_with_progress_merge() {
    let cloud = json!({
        "history": {
            "nom": ["One Piece"],
            "url": ["/catalogue/one-piece/saison1/vostfr/"],
            "num": [10]
        },
        "progress": {
            "/catalogue/one-piece/saison1/vostfr/": {
                "num": 12,
                "name": "Episode 12"
            },
            "/catalogue/naruto/saison1/vostfr/": {
                "num": 3,
                "name": "Episode 3"
            }
        }
    });

    let entries = extract_cloud_history(&cloud);
    let op = entries.iter().find(|e| e.slug == "one-piece").unwrap();
    assert_eq!(op.episode, 12);

    let naruto = entries.iter().find(|e| e.slug == "naruto").unwrap();
    assert_eq!(naruto.episode, 3);
}

#[test]
fn test_build_progress_payload() {
    let entries = vec![
        HistoryEntry {
            slug: "frieren".to_string(),
            title: "Frieren".to_string(),
            season: "saison1/vostfr".to_string(),
            episode: 4,
            timestamp: 123456789,
        }
    ];
    let payload = build_progress_payload(entries.iter());
    assert_eq!(payload["/catalogue/frieren/saison1/vostfr/"]["num"], 4);
    assert_eq!(payload["/catalogue/frieren/saison1/vostfr/"]["name"], "Episode 4");
}

#[test]
fn test_translate_passthrough_and_cache() {
    let cache = super::translate::new_translation_cache();
    let res_fr = super::translate::translate_text(&cache, "Bonjour le monde", "fr");
    assert_eq!(res_fr, "Bonjour le monde");

    let res_empty = super::translate::translate_text(&cache, "", "en");
    assert_eq!(res_empty, "");

    {
        let mut c = cache.lock().unwrap();
        c.insert(("en".to_string(), "Test".to_string()), "Pre-cached".to_string());
    }
    let res_cached = super::translate::translate_text(&cache, "Test", "en");
    assert_eq!(res_cached, "Pre-cached");
}
