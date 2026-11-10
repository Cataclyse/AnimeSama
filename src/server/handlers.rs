use super::AppState;
use crate::catalogue::CatalogData;
use crate::user::HistoryEntry;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

pub fn build_list_payload(catalog: &CatalogData, slugs: &[String]) -> serde_json::Value {
    let mut noms = Vec::new();
    let mut urls = Vec::new();
    let mut imgs = Vec::new();

    for slug in slugs {
        if let Some(item) = catalog.get_by_slug(slug) {
            noms.push(item.nom);
            urls.push(item.url);
            imgs.push(item.img);
        }
    }

    json!({ "nom": noms, "url": urls, "img": imgs })
}

pub fn run_auto_sync(state: &Arc<AppState>) -> Result<serde_json::Value, String> {
    if state
        .is_syncing
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return Ok(json!({ "synced": false, "in_progress": true }));
    }

    struct SyncGuard<'a>(&'a std::sync::atomic::AtomicBool);
    impl<'a> Drop for SyncGuard<'a> {
        fn drop(&mut self) {
            self.0.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let _guard = SyncGuard(&state.is_syncing);

    let cloud = state.auth.fetch_cloud_data()?;
    let local = state.user_mgr.get_data();

    let cloud_entries = extract_cloud_history(&cloud);

    let mut local_by_slug: HashMap<String, HistoryEntry> = local
        .history
        .iter()
        .map(|h| (h.slug.clone(), h.clone()))
        .collect();

    let mut updated_local = 0u32;
    let mut updated_cloud = 0u32;
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    
    
    for cloud_entry in &cloud_entries {
        if local.dismissed_continue.iter().any(|s| s == &cloud_entry.slug) {
            continue;
        }
        let local_num = local_by_slug.get(&cloud_entry.slug).map(|h| h.episode);
        if local_num.is_none() || cloud_entry.episode > local_num.unwrap() {
            let entry = HistoryEntry {
                slug: cloud_entry.slug.clone(),
                title: cloud_entry.title.clone(),
                season: cloud_entry.season.clone(),
                episode: cloud_entry.episode,
                timestamp: if cloud_entry.timestamp > 0 { cloud_entry.timestamp } else { now_ts },
            };
            state.user_mgr.add_history(entry.clone());
            local_by_slug.insert(cloud_entry.slug.clone(), entry);
            updated_local += 1;
        }
    }

    
    
    let cloud_by_slug: HashMap<String, usize> = cloud_entries
        .iter()
        .map(|c| (c.slug.clone(), c.episode))
        .collect();

    let needs_cloud_push = local_by_slug.values().any(|h| {
        cloud_by_slug
            .get(&h.slug)
            .map(|cloud_num| h.episode > *cloud_num)
            .unwrap_or(true)
    });

    if needs_cloud_push {
        let history_payload = build_history_payload(&state.catalog, local_by_slug.values());
        state.auth
            .push_cloud_data("sync-history.php", json!({ "history": history_payload }))
            .map_err(|e| format!("Échec sync-history : {e}"))?;

        let progress_payload = build_progress_payload(local_by_slug.values());
        state.auth
            .push_cloud_data("sync-progress.php", json!({ "progress": progress_payload }))
            .map_err(|e| format!("Échec sync-progress : {e}"))?;

        updated_cloud = local_by_slug.len() as u32;
    }

    let final_data = state.user_mgr.get_data();
    if updated_local > 0 || updated_cloud > 0 {
        println!(
            "[AutoSync] Synchronisation terminée : {} mis à jour en local, {} envoyés au cloud",
            updated_local, updated_cloud
        );
    }

    Ok(json!({
        "synced": true,
        "updated_local": updated_local,
        "updated_cloud": updated_cloud,
        "userdata": final_data
    }))
}

#[derive(Debug, Clone)]
pub struct CloudHistoryEntry {
    pub slug: String,
    pub title: String,
    pub season: String,
    pub episode: usize,
    pub timestamp: u64,
}

pub fn extract_cloud_history(cloud: &serde_json::Value) -> Vec<CloudHistoryEntry> {
    let mut by_slug: HashMap<String, CloudHistoryEntry> = HashMap::new();

    
    if let Some(history) = cloud.get("history") {
        let noms = history.get("nom").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let urls = history.get("url").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let nums = history.get("num").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let eps = history.get("ep").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        for i in 0..urls.len() {
            let url = match urls.get(i).and_then(|v| v.as_str()) {
                Some(u) => u,
                None => continue,
            };
            let slug = url
                .trim_matches('/')
                .trim_start_matches("catalogue/")
                .split('/')
                .next()
                .unwrap_or_default()
                .to_string();
            if slug.is_empty() {
                continue;
            }

            
            let episode = match nums.get(i) {
                Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
                Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
                _ => eps.get(i)
                    .and_then(|v| v.as_str())
                    .and_then(|s| {
                        s.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<usize>()
                            .ok()
                    })
                    .unwrap_or(0),
            };

            let season = url
                .trim_matches('/')
                .trim_start_matches("catalogue/")
                .splitn(2, '/')
                .nth(1)
                .unwrap_or("saison1/vostfr")
                .to_string();

            let title = noms.get(i).and_then(|v| v.as_str()).unwrap_or(&slug).to_string();

            if let Some(existing) = by_slug.get_mut(&slug) {
                if episode > existing.episode {
                    existing.episode = episode;
                    existing.season = season;
                }
            } else {
                by_slug.insert(
                    slug.clone(),
                    CloudHistoryEntry {
                        slug,
                        title,
                        season,
                        episode,
                        timestamp: 0,
                    },
                );
            }
        }
    }

    
    if let Some(progress_map) = cloud.get("progress").and_then(|v| v.as_object()) {
        for (url, prog) in progress_map {
            let slug = url
                .trim_matches('/')
                .trim_start_matches("catalogue/")
                .split('/')
                .next()
                .unwrap_or_default()
                .to_string();
            if slug.is_empty() {
                continue;
            }

            let episode = match prog.get("num") {
                Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
                Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
                _ => prog.get("name")
                    .and_then(|v| v.as_str())
                    .and_then(|s| {
                        s.chars()
                            .filter(|c| c.is_ascii_digit())
                            .collect::<String>()
                            .parse::<usize>()
                            .ok()
                    })
                    .unwrap_or(0),
            };

            if episode == 0 {
                continue;
            }

            let season = url
                .trim_matches('/')
                .trim_start_matches("catalogue/")
                .splitn(2, '/')
                .nth(1)
                .unwrap_or("saison1/vostfr")
                .to_string();

            let title = prog
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&slug)
                .to_string();

            if let Some(existing) = by_slug.get_mut(&slug) {
                if episode > existing.episode {
                    existing.episode = episode;
                    existing.season = season;
                }
            } else {
                by_slug.insert(
                    slug.clone(),
                    CloudHistoryEntry {
                        slug,
                        title,
                        season,
                        episode,
                        timestamp: 0,
                    },
                );
            }
        }
    }

    by_slug.into_values().collect()
}

pub fn build_history_payload<'a>(
    catalog: &CatalogData,
    entries: impl Iterator<Item = &'a HistoryEntry>,
) -> serde_json::Value {
    let mut noms = Vec::new();
    let mut urls = Vec::new();
    let mut imgs = Vec::new();
    let mut types = Vec::new();
    let mut langs = Vec::new();
    let mut eps = Vec::new();
    let mut nums = Vec::new();

    for entry in entries {
        let item = catalog.get_by_slug(&entry.slug);
        let nom = item.as_ref().map(|i| i.nom.clone()).unwrap_or_else(|| entry.title.clone());
        let img = item.as_ref().map(|i| i.img.clone()).unwrap_or_default();
        let is_scan = item
            .as_ref()
            .map(|i| i.badges.iter().any(|b| b.eq_ignore_ascii_case("scan")))
            .unwrap_or(false);
        let lang = if entry.season.to_lowercase().contains("vf") { "VF" } else { "VOSTFR" };

        noms.push(nom);
        urls.push(format!("/catalogue/{}/{}/", entry.slug, entry.season));
        imgs.push(img);
        types.push(if is_scan { "Scans" } else { "Anime" }.to_string());
        langs.push(lang.to_string());
        eps.push(format!("Episode {}", entry.episode));
        nums.push(entry.episode);
    }

    json!({ "nom": noms, "url": urls, "img": imgs, "type": types, "lang": langs, "ep": eps, "num": nums })
}

pub fn build_progress_payload<'a>(
    entries: impl Iterator<Item = &'a HistoryEntry>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for entry in entries {
        let key = format!("/catalogue/{}/{}/", entry.slug, entry.season);
        map.insert(
            key,
            json!({
                "name": format!("Episode {}", entry.episode),
                "num": entry.episode
            }),
        );
    }
    serde_json::Value::Object(map)
}

pub fn extract_slugs(cloud: &serde_json::Value, key: &str) -> Vec<String> {
    cloud
        .get(key)
        .and_then(|v| v.get("url"))
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|u| u.as_str())
                .filter_map(|u| {
                    u.trim_matches('/')
                        .trim_start_matches("catalogue/")
                        .split('/')
                        .next()
                        .map(|s| s.to_string())
                })
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn url_decode(s: &str) -> String {
    fn hex_value(value: u8) -> Option<u8> {
        match value {
            b'0'..=b'9' => Some(value - b'0'),
            b'a'..=b'f' => Some(value - b'a' + 10),
            b'A'..=b'F' => Some(value - b'A' + 10),
            _ => None,
        }
    }

    let input = s.as_bytes();
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        match input[index] {
            b'+' => {
                output.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < input.len() => {
                if let (Some(hi), Some(lo)) = (
                    hex_value(input[index + 1]),
                    hex_value(input[index + 2]),
                ) {
                    output.push((hi << 4) | lo);
                    index += 3;
                } else {
                    output.push(b'%');
                    index += 1;
                }
            }
            byte => {
                output.push(byte);
                index += 1;
            }
        }
    }

    String::from_utf8(output).unwrap_or_else(|_| s.to_owned())
}


#[cfg(test)]
mod url_decode_utf8_tests {
    use super::url_decode;

    #[test]
    fn decodes_utf8_accents() {
        assert_eq!(url_decode("Pok%C3%A9mon"), "Pokémon");
        assert_eq!(url_decode("Fran%C3%A7ais"), "Français");
        assert_eq!(url_decode("D%C3%A9j%C3%A0"), "Déjà");
    }

    #[test]
    fn preserves_standard_query_semantics() {
        assert_eq!(url_decode("Kagura+Bachi"), "Kagura Bachi");
        assert_eq!(url_decode("Solo%20Leveling"), "Solo Leveling");
        assert_eq!(url_decode("scan%2Fvf"), "scan/vf");
        assert_eq!(url_decode("%252F"), "%2F");
    }

    #[test]
    fn handles_malformed_sequences_without_dropping_text() {
        assert_eq!(url_decode("test%ZZ"), "test%ZZ");
        assert_eq!(url_decode("test%2"), "test%2");
        assert_eq!(url_decode("test%C3"), "test%C3");
    }
}
