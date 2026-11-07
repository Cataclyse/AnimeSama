pub mod fetcher;
pub mod rewriter;

use fetcher::{fetch_following_redirects, FetchError};
use rewriter::rewrite_body;
use tiny_http::{Header, Response, StatusCode};

pub fn handle_proxy_request(req: tiny_http::Request, raw_path: &str, query: &str, ad_shield: bool) {
    let (host, rest) = match raw_path.split_once('/') {
        Some((h, r)) => (h, r),
        None => (raw_path, ""),
    };

    if host.is_empty() {
        respond_text(req, 400, "Hôte manquant");
        return;
    }

    let mut resume_seconds: f64 = 0.0;
    let mut episode_num: u32 = 0;
    let clean_query = if !query.is_empty() {
        let mut clean_parts = Vec::new();
        for pair in query.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                if k == "as_resume" {
                    resume_seconds = v.parse::<f64>().unwrap_or(0.0);
                    continue;
                }
                if k == "as_ep" {
                    episode_num = v.parse::<u32>().unwrap_or(0);
                    continue;
                }
            }
            clean_parts.push(pair);
        }
        clean_parts.join("&")
    } else {
        String::new()
    };

    let mut target = format!("https://{}/{}", host, rest);
    if !clean_query.is_empty() {
        target.push('?');
        target.push_str(&clean_query);
    }

    if ad_shield && (crate::adblock::is_blocked_host(host) || crate::adblock::is_blocked_url(&target)) {
        if target.ends_with(".js") || rest.contains("script") {
            let mut resp = Response::from_string(crate::adblock::DUMMY_SCRIPT.to_string())
                .with_status_code(StatusCode(200));
            resp = resp.with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/javascript; charset=utf-8"[..]).unwrap());
            resp = resp.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
            let _ = req.respond(resp);
            return;
        } else if target.ends_with(".png") || target.ends_with(".gif") || target.ends_with(".jpg") {
            let mut resp = Response::from_data(crate::adblock::TRANSPARENT_1X1_GIF.to_vec())
                .with_status_code(StatusCode(200));
            resp = resp.with_header(Header::from_bytes(&b"Content-Type"[..], &b"image/gif"[..]).unwrap());
            resp = resp.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
            let _ = req.respond(resp);
            return;
        } else {
            let resp = Response::empty(StatusCode(204))
                .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
            let _ = req.respond(resp);
            return;
        }
    }

    match fetch_following_redirects(&target, host, 6) {
        Ok((final_host, content_type, body)) => {
            forward_body(req, &body, &content_type, &final_host, 200, resume_seconds, episode_num, ad_shield);
        }
        Err(FetchError::Status(code, r)) => forward_response_status(req, r, host, code, resume_seconds, episode_num, ad_shield),
        Err(FetchError::Other(e)) => respond_error_html(req, 502, &e),
    }
}

fn forward_body(req: tiny_http::Request, bytes: &[u8], content_type: &str, host: &str, status: u16, resume_seconds: f64, episode_num: u32, ad_shield: bool) {
    let is_text = content_type.contains("text/")
        || content_type.contains("javascript")
        || content_type.contains("json")
        || content_type.contains("mpegurl")
        || content_type.contains("xml");

    if is_text {
        let body = String::from_utf8_lossy(bytes).into_owned();
        let rewritten = rewrite_body(&body, host, content_type, resume_seconds, episode_num, ad_shield);

        let header_ct = if content_type.contains("text/html") {
            "text/html; charset=utf-8"
        } else if content_type.contains("css") {
            "text/css; charset=utf-8"
        } else if content_type.contains("mpegurl") {
            "application/vnd.apple.mpegurl"
        } else if content_type.contains("javascript") {
            "application/javascript; charset=utf-8"
        } else {
            "text/plain; charset=utf-8"
        };

        let mut response = Response::from_string(rewritten).with_status_code(StatusCode(status));
        response = response.with_header(Header::from_bytes(&b"Content-Type"[..], header_ct.as_bytes()).unwrap());
        response = response.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
        response = response.with_header(Header::from_bytes(&b"Referrer-Policy"[..], &b"no-referrer"[..]).unwrap());
        let _ = req.respond(response);
    } else {
        let mut response = Response::from_data(bytes.to_vec()).with_status_code(StatusCode(status));
        response = response.with_header(Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
        response = response.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
        response = response.with_header(Header::from_bytes(&b"Cache-Control"[..], &b"public, max-age=3600"[..]).unwrap());
        let _ = req.respond(response);
    }
}

fn forward_response_status(req: tiny_http::Request, resp: ureq::Response, host: &str, status: u16, resume_seconds: f64, episode_num: u32, ad_shield: bool) {
    let content_type = resp
        .header("Content-Type")
        .unwrap_or("application/octet-stream")
        .to_string();

    let mut bytes = Vec::new();
    if std::io::Read::read_to_end(&mut resp.into_reader(), &mut bytes).is_err() {
        respond_text(req, 502, "Erreur lecture réponse proxy");
        return;
    }

    forward_body(req, &bytes, &content_type, host, status, resume_seconds, episode_num, ad_shield);
}

fn respond_text(req: tiny_http::Request, status: u16, msg: &str) {
    let resp = Response::from_string(msg.to_string())
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

fn respond_error_html(req: tiny_http::Request, status: u16, error_msg: &str) {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{
            margin: 0; padding: 24px;
            background: #141414; color: #fff;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex; flex-direction: column; align-items: center; justify-content: center;
            height: 100vh; box-sizing: border-box; text-align: center;
        }}
        .error-card {{
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(255, 255, 255, 0.15);
            border-radius: 8px; padding: 32px 24px; max-width: 540px;
        }}
        h2 {{ margin: 0 0 12px; font-size: 20px; color: #E50914; }}
        p {{ margin: 0 0 16px; color: #aaa; font-size: 14px; line-height: 1.5; }}
        .hint {{
            background: rgba(229, 9, 20, 0.1); border: 1px solid rgba(229, 9, 20, 0.3);
            border-radius: 6px; padding: 12px 16px; color: #ff8888; font-size: 13px; margin-top: 14px; font-weight: 500;
        }}
        .tech {{ font-family: monospace; font-size: 11px; color: #666; margin-top: 16px; word-break: break-all; }}
    </style>
</head>
<body>
    <div class="error-card">
        <h2>Lecteur indisponible</h2>
        <p>Ce lecteur vidéo semble actuellement inaccessible ou hors-ligne (hébergeur en panne ou domaine miroir expiré).</p>
        <div class="hint">Changez simplement de lecteur via le menu déroulant <strong>« Lecteur »</strong> dans la barre en haut à droite.</div>
        <div class="tech">Détail technique : {}</div>
    </div>
</body>
</html>"#,
        html_escape(error_msg)
    );

    let resp = Response::from_string(html)
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    let _ = req.respond(resp);
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
