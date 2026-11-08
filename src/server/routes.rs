use super::handlers::*;
use super::static_files::*;
use super::AppState;
use crate::user::HistoryEntry;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

pub fn handle_request(mut req: tiny_http::Request, state: Arc<AppState>) {
    let url = req.url().to_string();
    let method = req.method().to_string();

    let (path, query_str) = match url.find('?') {
        Some(idx) => (&url[..idx], &url[idx + 1..]),
        None => (url.as_str(), ""),
    };

    let query_params: HashMap<String, String> = query_str
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let k = parts.next()?;
            let v = parts.next().unwrap_or("");
            Some((url_decode(k), url_decode(v)))
        })
        .collect();

    if method == "GET" {
        if let Some(raw_path) = path.strip_prefix("/proxy/") {
            let ad_shield = state.user_mgr.get_data().ad_shield;
            crate::proxy::handle_proxy_request(req, raw_path, query_str, ad_shield);
            return;
        }
    }

    match (method.as_str(), path) {
        ("GET", p)
            if p == "/"
                || p == "/app.css"
                || p == "/app.js"
                || p.starts_with("/css/")
                || p.starts_with("/js/")
                || p.starts_with("/lang/") =>
        {
            serve_static(req, p);
        }

        
        ("GET", "/api/home") => {
            let body = json!({
                "slides": state.catalog.slides,
                "planning": state.catalog.planning,
                "sections": state.catalog.sections,
                "total_catalog": state.catalog.catalogue.len(),
                "error": state.catalog_error
            });
            respond_json(req, &body.to_string());
        }

        ("GET", "/api/catalogue") => {
            let body = json!(state.catalog.catalogue);
            respond_json(req, &body.to_string());
        }

        ("GET", "/api/trailer") => {
            let title = query_params
                .get("title")
                .or_else(|| query_params.get("q"))
                .cloned()
                .unwrap_or_default();
            let ytid = crate::catalogue::trailer::get_anime_trailer(&title);
            let resp = json!({
                "title": title,
                "youtube_id": ytid
            });
            respond_json(req, &resp.to_string());
        }

        ("GET", "/api/skips") => {
            let title = query_params
                .get("title")
                .or_else(|| query_params.get("q"))
                .cloned()
                .unwrap_or_default();
            let episode: u32 = query_params
                .get("ep")
                .or_else(|| query_params.get("episode"))
                .and_then(|e| e.parse().ok())
                .unwrap_or(1);

            let skips = crate::catalogue::skips::get_skip_times(&title, episode);
            let resp = serde_json::to_string(&skips).unwrap_or_else(|_| "{}".to_string());
            respond_json(req, &resp);
        }

        ("GET", "/api/search") => {
            let q = query_params.get("q").cloned().unwrap_or_default();

            if q.trim().is_empty() {
                let body = json!({
                    "query": q,
                    "count": state.catalog.catalogue.len(),
                    "results": state.catalog.catalogue
                });
                respond_json(req, &body.to_string());
                return;
            }

            match state.scraper.search_live(&q) {
                Ok(results) => {
                    let body = json!({
                        "query": q,
                        "count": results.len(),
                        "results": results
                    });
                    respond_json(req, &body.to_string());
                }
                Err(e) => {
                    respond_error(req, 502, &format!("Recherche indisponible: {}", e));
                }
            }
        }

        ("GET", "/api/anime") => {
            let slug = query_params.get("slug").cloned().unwrap_or_default();
            if slug.is_empty() {
                respond_error(req, 400, "Paramètre 'slug' manquant");
                return;
            }

            
            match state.scraper.get_details(&slug) {
                Ok(details) => {
                    respond_json(req, &json!(details).to_string());
                }
                Err(err) => {
                    
                    if let Some(item) = state.catalog.get_by_slug(&slug) {
                        let fallback = json!({
                            "slug": item.slug,
                            "title": item.nom,
                            "banner": item.img,
                            "synopsis": item.synopsis,
                            "genres": item.genres,
                            "seasons": [
                                { "name": "Saison 1", "path": "saison1/vostfr" }
                            ]
                        });
                        respond_json(req, &fallback.to_string());
                    } else {
                        respond_error(req, 404, &format!("Anime non trouvé: {}", err));
                    }
                }
            }
        }

        ("GET", "/api/episodes") => {
            let slug = query_params.get("slug").cloned().unwrap_or_default();
            let season = query_params.get("season").cloned().unwrap_or_else(|| "saison1/vostfr".to_string());

            if slug.is_empty() {
                respond_error(req, 400, "Paramètre 'slug' manquant");
                return;
            }

            match state.scraper.get_episodes(&slug, &season) {
                Ok(episodes) => {
                    respond_json(req, &json!(episodes).to_string());
                }
                Err(e) => {
                    respond_error(req, 500, &format!("Erreur épisodes: {}", e));
                }
            }
        }

        ("GET", "/api/user") => {
            let data = state.user_mgr.get_data();
            respond_json(req, &json!(data).to_string());
        }

        ("POST", "/api/watchlist/toggle") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
            let slug = parsed
                .ok()
                .and_then(|v| v.get("slug").and_then(|s| s.as_str()).map(|s| s.to_string()))
                .unwrap_or_default();

            if slug.is_empty() {
                respond_error(req, 400, "Paramètre 'slug' requis");
                return;
            }

            let (in_list, data) = state.user_mgr.toggle_watchlist(&slug);
            let resp = json!({
                "slug": slug,
                "in_watchlist": in_list,
                "userdata": data
            });
            respond_json(req, &resp.to_string());
        }

        ("POST", "/api/history") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&content) {
                let data = state.user_mgr.add_history(entry);
                respond_json(req, &json!(data).to_string());

                
                
                if data.auto_sync && state.auth.is_logged_in() {
                    let state = Arc::clone(&state);
                    thread::spawn(move || {
                        let _ = run_auto_sync(&state);
                    });
                }
            } else {
                respond_error(req, 400, "Données d'historique invalides");
            }
        }

        ("GET", "/api/img") => {
            let slug = query_params.get("slug").cloned().unwrap_or_default();
            serve_image(req, &slug);
        }

        ("GET", "/api/auth/status") => {
            let verify = query_params.get("verify").map(|v| v == "1").unwrap_or(false);
            let logged_in = if verify {
                state.auth.verify_session()
            } else {
                state.auth.is_logged_in()
            };

            let body = json!({
                "logged_in": logged_in,
                "pseudo": if logged_in { state.auth.pseudo() } else { None }
            });
            respond_json(req, &body.to_string());
        }

        ("POST", "/api/auth/login") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let pseudo = parsed.get("pseudo").and_then(|v| v.as_str()).unwrap_or_default();
            let password = parsed.get("password").and_then(|v| v.as_str()).unwrap_or_default();

            match state.auth.login(pseudo, password) {
                Ok(p) => respond_json(req, &json!({ "logged_in": true, "pseudo": p }).to_string()),
                Err(e) => respond_error(req, 401, &e),
            }
        }

        ("POST", "/api/auth/register") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let pseudo = parsed.get("pseudo").and_then(|v| v.as_str()).unwrap_or_default();
            let password = parsed.get("password").and_then(|v| v.as_str()).unwrap_or_default();

            match state.auth.register(pseudo, password) {
                Ok(p) => respond_json(req, &json!({ "logged_in": true, "pseudo": p }).to_string()),
                Err(e) => respond_error(req, 400, &e),
            }
        }

        ("POST", "/api/auth/logout") => {
            state.auth.logout();
            respond_json(req, &json!({ "logged_in": false }).to_string());
        }

        ("POST", "/api/sync/to-cloud") => {
            if !state.auth.is_logged_in() {
                respond_error(req, 401, "Non connecté");
                return;
            }

            let local = state.user_mgr.get_data();

            let watchlist_payload = build_list_payload(&state.catalog, &local.watchlist);
            let favorites_payload = build_list_payload(&state.catalog, &local.favorites);

            let mut errors = Vec::new();
            if let Err(e) = state.auth.push_cloud_data(
                "sync-watchlist.php",
                json!({ "watchlist": watchlist_payload }),
            ) {
                errors.push(format!("watchlist: {}", e));
            }
            if let Err(e) = state.auth.push_cloud_data(
                "sync-favorites.php",
                json!({ "favorites": favorites_payload }),
            ) {
                errors.push(format!("favoris: {}", e));
            }

            if errors.is_empty() {
                respond_json(req, &json!({ "synced": true }).to_string());
            } else {
                respond_error(req, 502, &errors.join("; "));
            }
        }

        ("POST", "/api/sync/from-cloud") => {
            if !state.auth.is_logged_in() {
                respond_error(req, 401, "Non connecté");
                return;
            }

            match state.auth.fetch_cloud_data() {
                Ok(cloud) => {
                    let watchlist_slugs = extract_slugs(&cloud, "watchlist");
                    let favorites_slugs = extract_slugs(&cloud, "favorites");

                    state.user_mgr.set_watchlist(watchlist_slugs);
                    let data = state.user_mgr.set_favorites(favorites_slugs);

                    respond_json(req, &json!({ "synced": true, "userdata": data }).to_string());
                }
                Err(e) => respond_error(req, 502, &e),
            }
        }

        ("GET", "/api/settings") => {
            let data = state.user_mgr.get_data();
            respond_json(req, &json!({
                "auto_sync": data.auto_sync,
                "ad_shield": data.ad_shield,
                "auto_update": data.auto_update,
                "version": crate::update::CURRENT_VERSION
            }).to_string());
        }

        ("POST", "/api/settings/auto-sync") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

            let data = state.user_mgr.set_auto_sync(enabled);
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update }).to_string());
        }

        ("POST", "/api/settings/ad-shield") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let data = state.user_mgr.set_ad_shield(enabled);
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update }).to_string());
        }

        ("POST", "/api/settings/auto-update") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let data = state.user_mgr.set_auto_update(enabled);
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update }).to_string());
        }

        ("POST", "/api/sync/auto") => {
            if !state.auth.is_logged_in() {
                respond_error(req, 401, "Non connecté");
                return;
            }

            match run_auto_sync(&state) {
                Ok(summary) => respond_json(req, &summary.to_string()),
                Err(e) => respond_error(req, 502, &e),
            }
        }

        ("GET", "/api/languages") => {
            let list = crate::user::paths::list_available_languages();
            respond_json(req, &json!(list).to_string());
        }

        ("POST", "/api/open-lang-folder") => {
            let dir = crate::user::paths::lang_dir();
            let _ = opener::open(&dir);
            respond_json(req, &json!({ "opened": true, "path": dir.to_string_lossy() }).to_string());
        }

        ("POST", "/api/open-appdata") => {
            let dir = crate::user::paths::app_data_dir();
            let _ = opener::open(&dir);
            respond_json(req, &json!({ "opened": true, "path": dir.to_string_lossy() }).to_string());
        }

        ("GET", "/api/paths") => {
            let app_data = crate::user::paths::app_data_dir();
            let lang = crate::user::paths::lang_dir();
            let userdata = crate::user::paths::userdata_path();
            respond_json(req, &json!({
                "app_data": app_data.to_string_lossy(),
                "lang_dir": lang.to_string_lossy(),
                "userdata_file": userdata.to_string_lossy()
            }).to_string());
        }

        ("GET", "/api/version") => {
            respond_json(req, &json!({
                "version": crate::update::CURRENT_VERSION
            }).to_string());
        }

        ("GET", "/api/translate") => {
            let text = query_params.get("text").cloned().unwrap_or_default();
            let target = query_params.get("target").cloned().unwrap_or_else(|| "fr".to_string());
            let translated = super::translate::translate_text(&state.translation_cache, &text, &target);
            respond_json(req, &json!({
                "original": text,
                "translated": translated,
                "target": target
            }).to_string());
        }

        ("POST", "/api/translate") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let target = parsed.get("target").and_then(|v| v.as_str()).unwrap_or("fr");

            if let Some(arr) = parsed.get("texts").and_then(|v| v.as_array()) {
                let texts: Vec<String> = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                let translated = super::translate::translate_batch(&state.translation_cache, &texts, target);
                respond_json(req, &json!({ "translated": translated, "target": target }).to_string());
            } else {
                let text = parsed.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let translated = super::translate::translate_text(&state.translation_cache, text, target);
                respond_json(req, &json!({ "original": text, "translated": translated, "target": target }).to_string());
            }
        }

        ("GET", "/api/update/check") => {
            match crate::update::check_for_updates() {
                Ok(info) => respond_json(req, &json!(info).to_string()),
                Err(e) => respond_error(req, 500, &e),
            }
        }

        ("POST", "/api/update/apply") => {
            match crate::update::apply_update() {
                Ok(res) => respond_json(req, &json!(res).to_string()),
                Err(e) => respond_error(req, 500, &e),
            }
        }

        ("GET", "/api/update/status") => {
            let status = crate::update::get_update_progress();
            respond_json(req, &json!(status).to_string());
        }

        ("POST", "/api/update/open-release") => {
            let mut content = String::new();
            let _ = req.as_reader().read_to_string(&mut content);
            let parsed: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            let url = parsed.get("url").and_then(|v| v.as_str()).unwrap_or("https://github.com/Cataclyse/AnimeSama/releases");
            let _ = opener::open(url);
            respond_json(req, &json!({ "opened": true }).to_string());
        }

        _ => {
            respond_error(req, 404, "Page non trouvée");
        }
    }
}
