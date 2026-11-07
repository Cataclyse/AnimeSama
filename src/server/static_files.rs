use std::fs;
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
        if Path::new(&disk_path).exists() {
            if let Ok(c) = fs::read_to_string(&disk_path) {
                respond_css(req, &c);
                return true;
            }
        }
    }
    if let Some(sub) = clean.strip_prefix("js/") {
        let disk_path = format!("static/js/{}", sub);
        if Path::new(&disk_path).exists() {
            if let Ok(c) = fs::read_to_string(&disk_path) {
                respond_js(req, &c);
                return true;
            }
        }
    }
    if let Some(sub) = clean.strip_prefix("lang/") {
        let custom_path = crate::user::paths::lang_dir().join(sub);
        if custom_path.exists() {
            if let Ok(c) = fs::read_to_string(&custom_path) {
                let resp = Response::from_string(c)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap())
                    .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = req.respond(resp);
                return true;
            }
        }
        let disk_path = format!("static/lang/{}", sub);
        if Path::new(&disk_path).exists() {
            if let Ok(c) = fs::read_to_string(&disk_path) {
                let resp = Response::from_string(c)
                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap())
                    .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
                let _ = req.respond(resp);
                return true;
            }
        }
    }
    false
}
