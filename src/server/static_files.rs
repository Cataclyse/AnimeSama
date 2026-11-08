use std::fs;
use std::io::Read;
use std::path::Path;
use tiny_http::{Header, Response, StatusCode};

pub fn serve_image(req: tiny_http::Request, slug: &str) {
    let clean = slug.trim_start_matches('/').trim_end_matches(".jpg");
    let cdn_url = format!("https://cdn.jsdelivr.net/gh/Anime-Sama/IMG@img/contenu/{}.jpg", clean);
    let resp = Response::empty(StatusCode(302))
        .with_header(Header::from_bytes(&b"Location"[..], cdn_url.as_bytes()).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

pub fn serve_scan_image(req: tiny_http::Request, oeuvre: &str, chapter: &str, page: &str) {
    let clean_page = if page.ends_with(".jpg") {
        page.to_string()
    } else {
        format!("{}.jpg", page)
    };
    let mut encoded_oeuvre = String::with_capacity(oeuvre.len() * 3);
    for b in oeuvre.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded_oeuvre.push(b as char);
            }
            b' ' => encoded_oeuvre.push_str("%20"),
            _ => encoded_oeuvre.push_str(&format!("%{:02X}", b)),
        }
    }
    let url = format!("https://anime-sama.to/s2/scans/{}/{}/{}", encoded_oeuvre, chapter, clean_page);
    let resp = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .set("Referer", "https://anime-sama.to/")
        .timeout(std::time::Duration::from_secs(10))
        .call();

    match resp {
        Ok(r) => {
            let mut bytes = Vec::new();
            if r.into_reader().read_to_end(&mut bytes).is_ok() {
                let tiny_resp = Response::from_data(bytes)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"image/jpeg"[..]).unwrap())
                    .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"public, max-age=86400"[..]).unwrap())
                    .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = req.respond(tiny_resp);
            } else {
                let _ = req.respond(Response::empty(StatusCode(502)));
            }
        }
        Err(_) => {
            let _ = req.respond(Response::empty(StatusCode(502)));
        }
    }
}

pub fn read_static_or(path: &str, fallback: &str) -> String {
    if Path::new(path).exists() {
        if let Ok(s) = fs::read_to_string(path) {
            return s;
        }
    }
    fallback.to_string()
}

pub fn respond_html(req: tiny_http::Request, content: &str) {
    let resp = Response::from_string(content)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

pub fn respond_css(req: tiny_http::Request, content: &str) {
    let resp = Response::from_string(content)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/css; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

pub fn respond_js(req: tiny_http::Request, content: &str) {
    let resp = Response::from_string(content)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/javascript; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

pub fn respond_json(req: tiny_http::Request, content: &str) {
    let resp = Response::from_string(content)
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

pub fn respond_error(req: tiny_http::Request, status: u16, msg: &str) {
    let body = serde_json::json!({ "error": msg }).to_string();
    let resp = Response::from_string(body)
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

fn get_embedded_css(name: &str) -> Option<&'static str> {
    match name {
        "base.css" => Some(include_str!("../../static/css/base.css")),
        "cards.css" => Some(include_str!("../../static/css/cards.css")),
        "hero.css" => Some(include_str!("../../static/css/hero.css")),
        "modal.css" => Some(include_str!("../../static/css/modal.css")),
        "navbar.css" => Some(include_str!("../../static/css/navbar.css")),
        "player.css" => Some(include_str!("../../static/css/player.css")),
        "responsive.css" => Some(include_str!("../../static/css/responsive.css")),
        "settings.css" => Some(include_str!("../../static/css/settings.css")),
        "variables.css" => Some(include_str!("../../static/css/variables.css")),
        _ => None,
    }
}

fn get_embedded_js(name: &str) -> Option<&'static str> {
    match name {
        "api.js" => Some(include_str!("../../static/js/api.js")),
        "app.js" => Some(include_str!("../../static/js/app.js")),
        "hero.js" => Some(include_str!("../../static/js/hero.js")),
        "i18n.js" => Some(include_str!("../../static/js/i18n.js")),
        "modal.js" => Some(include_str!("../../static/js/modal.js")),
        "navigation.js" => Some(include_str!("../../static/js/navigation.js")),
        "planning.js" => Some(include_str!("../../static/js/planning.js")),
        "player.js" => Some(include_str!("../../static/js/player.js")),
        "rows.js" => Some(include_str!("../../static/js/rows.js")),
        "search.js" => Some(include_str!("../../static/js/search.js")),
        "settings.js" => Some(include_str!("../../static/js/settings.js")),
        "state.js" => Some(include_str!("../../static/js/state.js")),
        "userdata.js" => Some(include_str!("../../static/js/userdata.js")),
        "utils.js" => Some(include_str!("../../static/js/utils.js")),
        _ => None,
    }
}

fn get_embedded_lang(name: &str) -> Option<&'static str> {
    match name {
        "ar.lang" => Some(include_str!("../../static/lang/ar.lang")),
        "de.lang" => Some(include_str!("../../static/lang/de.lang")),
        "en.lang" => Some(include_str!("../../static/lang/en.lang")),
        "es.lang" => Some(include_str!("../../static/lang/es.lang")),
        "fr.lang" => Some(include_str!("../../static/lang/fr.lang")),
        "it.lang" => Some(include_str!("../../static/lang/it.lang")),
        "ja.lang" => Some(include_str!("../../static/lang/ja.lang")),
        "ko.lang" => Some(include_str!("../../static/lang/ko.lang")),
        "pt.lang" => Some(include_str!("../../static/lang/pt.lang")),
        "ru.lang" => Some(include_str!("../../static/lang/ru.lang")),
        _ => None,
    }
}

pub fn serve_static(req: tiny_http::Request, path: &str) -> bool {
    let clean = path.trim_start_matches('/');
    if clean.is_empty() || clean == "index.html" {
        let html = read_static_or("static/index.html", include_str!("../../static/index.html"));
        respond_html(req, &html);
        return true;
    }
    if clean == "app.css" {
        let css = read_static_or("static/app.css", include_str!("../../static/app.css"));
        respond_css(req, &css);
        return true;
    }
    if clean == "app.js" {
        let js = read_static_or("static/app.js", include_str!("../../static/app.js"));
        respond_js(req, &js);
        return true;
    }
    if let Some(sub) = clean.strip_prefix("css/") {
        let disk_path = format!("static/css/{}", sub);
        let content = if Path::new(&disk_path).exists() {
            fs::read_to_string(&disk_path).ok()
        } else {
            get_embedded_css(sub).map(|s| s.to_string())
        };
        if let Some(c) = content {
            respond_css(req, &c);
            return true;
        }
    }
    if let Some(sub) = clean.strip_prefix("js/") {
        let disk_path = format!("static/js/{}", sub);
        let content = if Path::new(&disk_path).exists() {
            fs::read_to_string(&disk_path).ok()
        } else {
            get_embedded_js(sub).map(|s| s.to_string())
        };
        if let Some(c) = content {
            respond_js(req, &c);
            return true;
        }
    }
    if let Some(sub) = clean.strip_prefix("lang/") {
        let disk_path = format!("static/lang/{}", sub);
        let content = if Path::new(&disk_path).exists() {
            fs::read_to_string(&disk_path).ok()
        } else {
            let custom_path = crate::user::paths::lang_dir().join(sub);
            if custom_path.exists() {
                fs::read_to_string(&custom_path).ok()
            } else {
                get_embedded_lang(sub).map(|s| s.to_string())
            }
        };
        if let Some(c) = content {
            let resp = Response::from_string(c)
                .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap())
                .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
            let _ = req.respond(resp);
            return true;
        }
    }
    false
}
