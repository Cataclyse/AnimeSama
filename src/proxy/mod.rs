pub mod security;
pub mod fetcher;
pub mod rewriter;

use fetcher::{fetch_following_redirects, FetchBody, FetchError};
use rewriter::{rewrite_body, rewrite_m3u8_for_proxy};
use tiny_http::{Header, Response, StatusCode};

// Isolation de tout contenu renvoyé par le proxy externe.
// Ne jamais ajouter allow-same-origin à cette politique.
macro_rules! proxy_respond {
    ($request:expr, $response:expr) => {{
        let response = $response
            .with_header(
                Header::from_bytes(
                    b"Content-Security-Policy",
                    b"sandbox allow-scripts allow-forms allow-presentation",
                ).unwrap()
            )
            .with_header(
                Header::from_bytes(
                    b"X-Content-Type-Options",
                    b"nosniff",
                ).unwrap()
            );

        let _ = $request.respond(response);
    }};
}


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
            proxy_respond!(req, resp);
            return;
        } else if target.ends_with(".png") || target.ends_with(".gif") || target.ends_with(".jpg") {
            let mut resp = Response::from_data(crate::adblock::TRANSPARENT_1X1_GIF.to_vec())
                .with_status_code(StatusCode(200));
            resp = resp.with_header(Header::from_bytes(&b"Content-Type"[..], &b"image/gif"[..]).unwrap());
            resp = resp.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
            proxy_respond!(req, resp);
            return;
        } else {
            let resp = Response::empty(StatusCode(204))
                .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
            proxy_respond!(req, resp);
            return;
        }
    }

    if let Err(reason) = security::validate_proxy_url(&target) {
        respond_text(req, 403, reason);
        return;
    }

    let range = req.headers().get("Range")
        .map(|h| h.as_str().to_owned());

    if let Some(value) = range.as_deref() {
        if !fetcher::valid_single_range(value) {
            respond_text(req, 400, "Range HTTP invalide");
            return;
        }
    }

    match fetch_following_redirects(
        &target, host, 6, range.as_deref()
    ) {
        Ok(result) => match result.body {
            FetchBody::Buffered(body) => forward_body(
                req,
                &body,
                &result.content_type,
                &result.final_host,
                &result.final_url,
                result.status,
                resume_seconds,
                episode_num,
                ad_shield,
            ),
            FetchBody::Streaming(resp) => {
                forward_binary_stream(req, resp, &result.content_type)
            }
        },
        Err(FetchError::Status(code, r)) => {
            forward_response_status(
                req, r, host, &target, code,
                resume_seconds, episode_num, ad_shield
            )
        }
        Err(FetchError::Other(e)) => {
            respond_error_html(req, 502, &e)
        }
    }
}


fn forward_binary_stream(
    req: tiny_http::Request,
    resp: ureq::Response,
    content_type: &str,
) {
    let status = resp.status();

    if status != 200 && status != 206 {
        respond_text(req, 502, "Statut du flux distant inattendu");
        return;
    }

    let content_encoding = resp
        .header("Content-Encoding")
        .unwrap_or("identity");

    if !content_encoding.eq_ignore_ascii_case("identity") {
        respond_text(req, 502, "Encodage binaire non pris en charge");
        return;
    }

    let length = resp
        .header("Content-Length")
        .and_then(|v| v.parse::<usize>().ok());

    // tiny_http peut mettre en mémoire les corps sans longueur
    // sur une connexion HTTP/1.0.
    if length.is_none() && *req.http_version() <= (1, 0) {
        respond_text(req, 505, "HTTP/1.1 requis pour ce flux");
        return;
    }

    let content_range = resp.header("Content-Range")
        .map(str::to_owned);

    if status == 206 && content_range.is_none() {
        respond_text(req, 502, "Content-Range manquant");
        return;
    }

    let accept_ranges = resp.header("Accept-Ranges")
        .map(str::to_owned);

    let mut headers = vec![
        Header::from_bytes(
            &b"Content-Type"[..], content_type.as_bytes()
        ).unwrap(),
        Header::from_bytes(
            &b"Access-Control-Allow-Origin"[..], &b"*"[..]
        ).unwrap(),
        Header::from_bytes(
            &b"Access-Control-Expose-Headers"[..],
            &b"Content-Range, Accept-Ranges, Content-Length"[..]
        ).unwrap(),
        Header::from_bytes(
            &b"Cache-Control"[..],
            &b"public, max-age=3600"[..]
        ).unwrap(),
    ];

    if let Some(value) = content_range {
        if let Ok(header) = Header::from_bytes(
            &b"Content-Range"[..], value.as_bytes()
        ) {
            headers.push(header);
        }
    }

    if let Some(value) = accept_ranges {
        if let Ok(header) = Header::from_bytes(
            &b"Accept-Ranges"[..], value.as_bytes()
        ) {
            headers.push(header);
        }
    }

    let response = Response::new(
        StatusCode(status),
        headers.into(),
        resp.into_reader(),
        length,
        None,
    );

    proxy_respond!(req, response);
}

fn forward_body(req: tiny_http::Request, bytes: &[u8], content_type: &str, host: &str, source_url: &str, status: u16, resume_seconds: f64, episode_num: u32, ad_shield: bool) {
    let is_text = content_type.contains("text/")
        || content_type.contains("javascript")
        || content_type.contains("json")
        || content_type.contains("mpegurl")
        || content_type.contains("xml");

    if is_text {
        let body = String::from_utf8_lossy(bytes).into_owned();
        let rewritten = if content_type.contains("mpegurl") {
            match rewrite_m3u8_for_proxy(&body, source_url) {
                Ok(manifest) => manifest,
                Err(_) => {
                    respond_text(req, 502, "Reference HLS refusee");
                    return;
                }
            }
        } else {
            rewrite_body(
                &body, host, content_type,
                resume_seconds, episode_num, ad_shield
            )
        };

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
        proxy_respond!(req, response);
    } else {
        let mut response = Response::from_data(bytes.to_vec()).with_status_code(StatusCode(status));
        response = response.with_header(Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());
        response = response.with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
        response = response.with_header(Header::from_bytes(&b"Cache-Control"[..], &b"public, max-age=3600"[..]).unwrap());
        proxy_respond!(req, response);
    }
}

fn forward_response_status(req: tiny_http::Request, resp: ureq::Response, host: &str, source_url: &str, status: u16, resume_seconds: f64, episode_num: u32, ad_shield: bool) {
    // Réponse HTTP 416 : conserver Content-Range.
    if status == 416 {
        let content_type = resp
            .header("Content-Type")
            .unwrap_or("text/plain; charset=utf-8")
            .to_string();

        let content_range = resp
            .header("Content-Range")
            .map(str::to_owned);

        let bytes = match fetcher::read_limited(
            resp.into_reader(),
            fetcher::MAX_TEXT_BODY_BYTES,
        ) {
            Ok(bytes) => bytes,
            Err(_) => {
                respond_text(req, 502, "Erreur lecture réponse 416");
                return;
            }
        };

        let mut response = Response::from_data(bytes)
            .with_status_code(StatusCode(416))
            .with_header(
                Header::from_bytes(
                    b"Content-Type",
                    content_type.as_bytes(),
                ).unwrap()
            )
            .with_header(
                Header::from_bytes(
                    b"Access-Control-Allow-Origin",
                    b"*",
                ).unwrap()
            )
            .with_header(
                Header::from_bytes(
                    b"Access-Control-Expose-Headers",
                    b"Content-Range",
                ).unwrap()
            );

        if let Some(value) = content_range {
            if let Ok(header) = Header::from_bytes(
                b"Content-Range",
                value.as_bytes(),
            ) {
                response = response.with_header(header);
            }
        }

        proxy_respond!(req, response);
        return;
    }

    let content_type = resp
        .header("Content-Type")
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = match fetcher::read_limited(
        resp.into_reader(),
        fetcher::MAX_TEXT_BODY_BYTES,
    ) {
        Ok(bytes) => bytes,
        Err(_) => {
            respond_text(req, 502, "Réponse distante trop volumineuse ou illisible");
            return;
        }
    };

    forward_body(req, &bytes, &content_type, host, source_url, status, resume_seconds, episode_num, ad_shield);
}

fn respond_text(req: tiny_http::Request, status: u16, msg: &str) {
    let resp = Response::from_string(msg.to_string())
        .with_status_code(StatusCode(status))
        .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..]).unwrap())
        .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap());
    proxy_respond!(req, resp);
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
    proxy_respond!(req, resp);
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
