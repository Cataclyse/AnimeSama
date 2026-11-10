use super::handlers::*;
use super::static_files::*;
use super::AppState;
use crate::user::{HistoryEntry, PlaybackPosition};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};


const LOCAL_COOKIE: &str = "cataclyse_session";
const LOCAL_COOKIE_AGE: u64 = 30 * 24 * 60 * 60;

fn parse_local_cookie(header: &str) -> Option<&str> {
    let mut found = None;

    for item in header.split(';') {
        let Some((name, value)) = item.trim().split_once('=') else {
            continue;
        };

        if name != LOCAL_COOKIE {
            continue;
        }

        if found.is_some()
            || value.len() != 64
            || !value.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return None;
        }

        found = Some(value);
    }

    found
}

fn request_local_token(req: &tiny_http::Request) -> Option<&str> {
    req.headers()
        .get("Cookie")
        .and_then(|h| parse_local_cookie(h.as_str()))
}

fn local_session_cookie(token: &str) -> String {
    format!(
        "{}={}; Path=/api; HttpOnly; SameSite=Strict; Max-Age={}",
        LOCAL_COOKIE, token, LOCAL_COOKIE_AGE
    )
}

fn clear_local_session_cookie() -> String {
    format!(
        "{}=; Path=/api; HttpOnly; SameSite=Strict; Max-Age=0",
        LOCAL_COOKIE
    )
}

fn respond_local_json(
    req: tiny_http::Request,
    status: u16,
    body: serde_json::Value,
    cookie: Option<String>,
) {
    let mut response = tiny_http::Response::from_string(
        body.to_string()
    ).with_status_code(tiny_http::StatusCode(status));

    response.add_header(
        tiny_http::Header::from_bytes(
            &b"Content-Type"[..],
            &b"application/json; charset=utf-8"[..],
        ).expect("En-tête Content-Type")
    );

    response.add_header(
        tiny_http::Header::from_bytes(
            &b"Cache-Control"[..],
            &b"no-store"[..],
        ).expect("En-tête Cache-Control")
    );

    response.add_header(
        tiny_http::Header::from_bytes(
            &b"X-Content-Type-Options"[..],
            &b"nosniff"[..],
        ).expect("En-tête de sécurité")
    );

    if let Some(value) = cookie {
        response.add_header(
            tiny_http::Header::from_bytes(
                &b"Set-Cookie"[..],
                value.as_bytes(),
            ).expect("En-tête Set-Cookie")
        );
    }

    let _ = req.respond(response);
}

#[derive(Default)]
struct LocalAuthBudget {
    started: Option<Instant>,
    signups: u32,
    logins: u32,
    by_name: HashMap<String, u32>,
}

impl LocalAuthBudget {
    fn allow(&mut self, username: &str, signup: bool) -> bool {
        let current = Instant::now();

        if self.started
            .map(|t| current.duration_since(t) >= Duration::from_secs(60))
            .unwrap_or(true)
        {
            *self = Self {
                started: Some(current),
                ..Self::default()
            };
        }

        let key = username.trim().to_ascii_lowercase();
        let total = if signup { self.signups } else { self.logins };
        let global_limit = if signup { 10 } else { 30 };
        let per_user = *self.by_name.get(&key).unwrap_or(&0);

        if total >= global_limit || per_user >= 5 {
            return false;
        }

        if signup {
            self.signups += 1;
        } else {
            self.logins += 1;
        }

        *self.by_name.entry(key).or_insert(0) += 1;
        true
    }
}

static LOCAL_AUTH_BUDGET:
    OnceLock<Mutex<LocalAuthBudget>> = OnceLock::new();

fn allow_local_auth(username: &str, signup: bool) -> bool {
    LOCAL_AUTH_BUDGET
        .get_or_init(|| Mutex::new(LocalAuthBudget::default()))
        .lock()
        .map(|mut budget| budget.allow(username, signup))
        .unwrap_or(false)
}

fn allowed_dev_request(
    req: &tiny_http::Request,
    method: &str,
) -> bool {
    let host = req.headers().get("Host").map(|v| v.as_str())
        .unwrap_or("");

    if !matches!(host, "127.0.0.1:48621" | "localhost:48621") {
        return false;
    }

    if method != "POST" {
        return true;
    }

    let origin = req.headers().get("Origin").map(|v| v.as_str());

    let expected = format!("http://{}", host);

    if origin != Some(expected.as_str()) {
        return false;
    }

    if let Some(site) = req.headers().get("Sec-Fetch-Site").map(|v| v.as_str())
    {
        if site != "same-origin" {
            return false;
        }
    }

    true
}


type PostBodyError = (u16, &'static str);

fn max_post_body_bytes(path: &str) -> usize {
    match path {
        "/api/auth/login" | "/api/auth/register"
        | "/api/local-auth/login" | "/api/local-auth/register" => 16 * 1024,
        "/api/translate" => 1024 * 1024,
        _ => 256 * 1024,
    }
}

fn read_bounded_post_body<R: std::io::Read>(
    reader: R,
    max_bytes: usize,
) -> Result<String, PostBodyError> {
    use std::io::Read;

    let mut body = Vec::new();

    reader
        .take(max_bytes as u64 + 1)
        .read_to_end(&mut body)
        .map_err(|_| (400, "Lecture du corps HTTP impossible"))?;

    if body.len() > max_bytes {
        return Err((413, "Corps HTTP trop volumineux"));
    }

    String::from_utf8(body)
        .map_err(|_| (400, "Corps HTTP UTF-8 invalide"))
}

fn parse_post_json(
    content: &str,
) -> Result<serde_json::Value, PostBodyError> {
    serde_json::from_str(content)
        .map_err(|_| (400, "Corps JSON invalide"))
}

macro_rules! read_post_body_or_reply {
    ($request:ident, $limit:expr $(,)?) => {{
        match read_bounded_post_body($request.as_reader(), $limit) {
            Ok(body) => body,
            Err((status, message)) => {
                respond_error($request, status, message);
                return;
            }
        }
    }};
}

macro_rules! parse_post_json_or_reply {
    ($request:ident, $content:expr) => {{
        match parse_post_json($content) {
            Ok(value) => value,
            Err((status, message)) => {
                respond_error($request, status, message);
                return;
            }
        }
    }};
}

pub fn handle_request(mut req: tiny_http::Request, state: Arc<AppState>) {
    let url = req.url().to_string();
    let method = req.method().to_string();

    if !allowed_dev_request(&req, &method) {
        respond_error(req, 403, "Origine HTTP non autorisée");
        return;
    }


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

        ("GET", "/api/video/resolve") => {
            match super::video_bridge::resolve(&query_params) {
                Ok(data) => respond_json(req, &data.to_string()),
                Err((status, message)) => {
                    respond_error(req, status, message)
                }
            }
        }

        ("GET", "/api/video/sources") => {
            match super::video_bridge::sources(&query_params) {
                Ok(data) => respond_json(req, &data.to_string()),
                Err((status, message)) => {
                    respond_error(req, status, message)
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

            if !crate::scraper::valid_episode_request(&slug, &season) {
                respond_error(
                    req,
                    400,
                    "Slug ou chemin de saison invalide",
                );
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
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
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

        ("GET", "/api/playback/position") => {
            let slug = query_params.get("slug");
            let season = query_params.get("season");
            let episode = query_params
                .get("episode")
                .and_then(|v| v.parse::<usize>().ok());

            match (slug, season, episode) {
                (Some(slug), Some(season), Some(episode))
                    if !slug.is_empty()
                        && !season.is_empty()
                        && episode > 0 =>
                {
                    let position = state.user_mgr
                        .get_playback_position(slug, season, episode);

                    respond_json(
                        req,
                        &json!({"position": position}).to_string()
                    );
                }
                _ => respond_error(
                    req, 400, "Paramètres de progression invalides"
                ),
            }
        }

        ("POST", "/api/playback/position") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );

            match serde_json::from_str::<PlaybackPosition>(&content) {
                Ok(position)
                    if !position.slug.is_empty()
                        && position.slug.len() <= 200
                        && !position.season.is_empty()
                        && position.season.len() <= 100
                        && position.episode > 0
                        && position.episode <= 10000
                        && position.current_time.is_finite()
                        && position.duration.is_finite()
                        && position.current_time >= 0.0
                        && position.duration >= 0.0
                        && position.current_time <= 864000.0
                        && position.duration <= 864000.0
                        && position.timestamp > 0 =>
                {
                    match state.user_mgr
                        .put_playback_position(position)
                    {
                        Ok(saved) => respond_json(
                            req,
                            &json!({"position": saved}).to_string(),
                        ),
                        Err(()) => respond_error(
                            req,
                            409,
                            "Position modifiée par un autre appareil",
                        ),
                    }
                }
                _ => respond_error(
                    req, 400, "Position de lecture invalide"
                ),
            }
        }

        ("POST", "/api/history") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
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

        ("POST", "/api/history/remove") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
            if let Ok(val) = parsed {
                if let Some(slug) = val.get("slug").and_then(|s| s.as_str()) {
                    let data = state.user_mgr.remove_history(slug);
                    respond_json(req, &json!(data).to_string());
                    return;
                }
            }
            respond_error(req, 400, "Paramètre 'slug' manquant");
        }

        ("GET", "/api/img") => {
            let slug = query_params.get("slug").cloned().unwrap_or_default();
            serve_image(req, &slug);
        }

        ("GET", "/api/scan/img") => {
            let oeuvre = query_params.get("oeuvre").cloned().unwrap_or_default();
            let chapter = query_params.get("chapter").cloned().unwrap_or_default();
            let page = query_params.get("page").cloned().unwrap_or_default();
            serve_scan_image(req, &oeuvre, &chapter, &page);
        }


        ("GET", "/api/local-auth/status") => {
            let result = match request_local_token(&req) {
                Some(token) => state.accounts.resolve(token),
                None => Ok(None),
            };

            match result {
                Ok(user) => respond_local_json(
                    req, 200,
                    json!({
                        "logged_in": user.is_some(),
                        "user": user.map(|u| json!({
                            "id": u.id,
                            "username": u.username
                        }))
                    }),
                    None,
                ),
                Err(_) => respond_error(
                    req, 500, "Session locale indisponible"
                ),
            }
        }

        ("POST", "/api/local-auth/register") => {
            let content = read_post_body_or_reply!(
                req, max_post_body_bytes(path),
            );
            let parsed: serde_json::Value =
                parse_post_json_or_reply!(req, &content);

            let username = parsed.get("username")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let password = parsed.get("password")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if username.len() > 32 || password.len() > 1024 {
                respond_error(req, 400, "Paramètres invalides");
                return;
            }

            if !allow_local_auth(username, true) {
                respond_error(req, 429, "Trop de tentatives");
                return;
            }

            match state.accounts.register(username, password) {
                Ok(user) => respond_local_json(
                    req, 201,
                    json!({
                        "created": true,
                        "user": {
                            "id": user.id,
                            "username": user.username
                        }
                    }),
                    None,
                ),
                Err(_) => respond_error(
                    req, 400, "Création du compte impossible"
                ),
            }
        }

        ("POST", "/api/local-auth/login") => {
            let content = read_post_body_or_reply!(
                req, max_post_body_bytes(path),
            );
            let parsed: serde_json::Value =
                parse_post_json_or_reply!(req, &content);

            let username = parsed.get("username")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let password = parsed.get("password")
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            if username.len() > 32 || password.len() > 1024 {
                respond_error(req, 401, "Identifiants invalides");
                return;
            }

            if !allow_local_auth(username, false) {
                respond_error(req, 429, "Trop de tentatives");
                return;
            }

            match state.accounts.login(username, password) {
                Ok(Some((user, token))) => respond_local_json(
                    req, 200,
                    json!({
                        "logged_in": true,
                        "user": {
                            "id": user.id,
                            "username": user.username
                        }
                    }),
                    Some(local_session_cookie(&token)),
                ),
                Ok(None) => respond_error(
                    req, 401, "Identifiants invalides"
                ),
                Err(_) => respond_error(
                    req, 500, "Connexion locale indisponible"
                ),
            }
        }

        ("POST", "/api/local-auth/logout") => {
            if let Some(token) = request_local_token(&req) {
                if state.accounts.logout(token).is_err() {
                    respond_error(
                        req, 500, "Déconnexion impossible"
                    );
                    return;
                }
            }

            respond_local_json(
                req, 200,
                json!({"logged_in": false}),
                Some(clear_local_session_cookie()),
            );
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
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
            let pseudo = parsed.get("pseudo").and_then(|v| v.as_str()).unwrap_or_default();
            let password = parsed.get("password").and_then(|v| v.as_str()).unwrap_or_default();

            match state.auth.login(pseudo, password) {
                Ok(p) => respond_json(req, &json!({ "logged_in": true, "pseudo": p }).to_string()),
                Err(e) => respond_error(req, 401, &e),
            }
        }

        ("POST", "/api/auth/register") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
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
                "discord_rpc": data.discord_rpc,
                "version": crate::update::CURRENT_VERSION
            }).to_string());
        }

        ("POST", "/api/settings/auto-sync") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);

            let data = state.user_mgr.set_auto_sync(enabled);
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update, "discord_rpc": data.discord_rpc }).to_string());
        }

        ("POST", "/api/settings/ad-shield") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let data = state.user_mgr.set_ad_shield(enabled);
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update, "discord_rpc": data.discord_rpc }).to_string());
        }

        ("POST", "/api/settings/auto-update") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let data = state.user_mgr.set_auto_update(enabled);
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update, "discord_rpc": data.discord_rpc }).to_string());
        }

        ("POST", "/api/settings/discord-rpc") => {
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
            let enabled = parsed.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);

            let data = state.user_mgr.set_discord_rpc(enabled);
            if !enabled {
                crate::discord::clear_presence();
            }
            respond_json(req, &json!({ "auto_sync": data.auto_sync, "ad_shield": data.ad_shield, "auto_update": data.auto_update, "discord_rpc": data.discord_rpc }).to_string());
        }

        ("POST", "/api/rpc/presence") => {
            if !state.user_mgr.get_data().discord_rpc {
                respond_json(req, &json!({ "status": "disabled" }).to_string());
                return;
            }
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: Result<crate::discord::PresencePayload, _> = serde_json::from_str(&content);
            match parsed {
                Ok(payload) => {
                    crate::discord::update_presence(payload);
                    respond_json(req, &json!({ "status": "ok" }).to_string());
                }
                Err(_) => {
                    respond_error(req, 400, "Payload RPC invalide");
                }
            }
        }

        ("POST", "/api/rpc/clear") => {
            crate::discord::clear_presence();
            respond_json(req, &json!({ "status": "cleared" }).to_string());
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
            let content = read_post_body_or_reply!(
                req,
                max_post_body_bytes(path),
            );
            let parsed: serde_json::Value = parse_post_json_or_reply!(req, &content);
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
            // Instance DEV : installation interdite.
            respond_error(
                req,
                403,
                "Mises a jour desactivees sur Cataclyse DEV",
            );
        }

        ("GET", "/api/update/status") => {
            let status = crate::update::get_update_progress();
            respond_json(req, &json!(status).to_string());
        }

        ("POST", "/api/update/open-release") => {
            // Instance DEV : ouverture d'URL interdite.
            respond_error(
                req,
                403,
                "Ouverture externe desactivee sur Cataclyse DEV",
            );
        }

        _ => {
            respond_error(req, 404, "Page non trouvée");
        }
    }
}


#[cfg(test)]
mod bounded_post_body_tests {
    use super::*;

    #[test]
    fn accepts_body_at_exact_limit() {
        let data = b"12345678";
        let parsed = read_bounded_post_body(&data[..], 8).unwrap();
        assert_eq!(parsed, "12345678");
    }

    #[test]
    fn rejects_body_over_limit() {
        let data = b"123456789";
        let error = read_bounded_post_body(&data[..], 8).unwrap_err();
        assert_eq!(error.0, 413);
    }

    #[test]
    fn rejects_invalid_utf8() {
        let data = [0xff, 0xfe];
        let error = read_bounded_post_body(&data[..], 8).unwrap_err();
        assert_eq!(error.0, 400);
    }

    #[test]
    fn parses_valid_json() {
        let data = parse_post_json(r#"{"enabled":true}"#).unwrap();
        assert_eq!(data["enabled"], true);
    }

    #[test]
    fn rejects_malformed_json() {
        for body in ["", "{invalid", "not-json"] {
            assert_eq!(parse_post_json(body).unwrap_err().0, 400);
        }
    }

    #[test]
    fn uses_route_specific_limits() {
        assert_eq!(max_post_body_bytes("/api/auth/login"), 16384);
        assert_eq!(max_post_body_bytes("/api/auth/register"), 16384);
        assert_eq!(max_post_body_bytes("/api/history"), 262144);
        assert_eq!(max_post_body_bytes("/api/translate"), 1048576);
    }
}


#[cfg(test)]
mod local_auth_route_tests {
    use super::*;

    #[test]
    fn parses_only_exact_session_cookie() {
        let token = "a".repeat(64);
        let header = format!(
            "other=123; {}={}; preference=dark",
            LOCAL_COOKIE, token
        );

        assert_eq!(
            parse_local_cookie(&header),
            Some(token.as_str())
        );
        assert_eq!(
            parse_local_cookie(&format!(
                "other_{}={}", LOCAL_COOKIE, token
            )),
            None
        );
    }

    #[test]
    fn refuses_ambiguous_or_invalid_cookies() {
        let token = "b".repeat(64);
        let duplicate = format!(
            "{}={}; {}={}",
            LOCAL_COOKIE, token, LOCAL_COOKIE, token
        );

        assert!(parse_local_cookie(&duplicate).is_none());
        assert!(parse_local_cookie(
            "cataclyse_session=invalid"
        ).is_none());
    }

    #[test]
    fn limits_authentication_attempts() {
        let mut budget = LocalAuthBudget::default();

        for _ in 0..5 {
            assert!(budget.allow("alice", false));
        }

        assert!(!budget.allow("alice", false));
        assert!(budget.allow("bob", false));
    }

    #[test]
    fn resets_rate_limits_after_window() {
        let mut budget = LocalAuthBudget::default();

        assert!(budget.allow("alice", true));

        budget.started = Some(
            Instant::now() - Duration::from_secs(61)
        );

        assert!(budget.allow("alice", true));
        assert_eq!(budget.signups, 1);
    }
}
