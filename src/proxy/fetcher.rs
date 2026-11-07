use super::rewriter::absolute_url;
use regex::Regex;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub enum FetchError {
    Status(u16, ureq::Response),
    Other(String),
}

pub fn fetch_following_redirects(
    url: &str,
    fallback_host: &str,
    max_js_hops: u8,
) -> Result<(String, String, Vec<u8>), FetchError> {
    let agent = ureq::AgentBuilder::new()
        .redirects(30)
        .timeout(std::time::Duration::from_secs(20))
        .build();

    let resp = agent
        .get(url)
        .set("User-Agent", USER_AGENT)
        .set("Referer", "https://anime-sama.to/")
        .set("Accept", "*/*")
        .call()
        .map_err(|e| match e {
            ureq::Error::Status(code, r) => FetchError::Status(code, r),
            ureq::Error::Transport(t) => FetchError::Other(t.to_string()),
        })?;

    let final_host = final_host_of(&resp).unwrap_or_else(|| fallback_host.to_string());
    let content_type = resp.header("Content-Type").unwrap_or("application/octet-stream").to_string();
    let is_html = content_type.contains("text/html");

    let mut body = Vec::new();
    if std::io::Read::read_to_end(&mut resp.into_reader(), &mut body).is_err() {
        return Err(FetchError::Other("Erreur lecture réponse".to_string()));
    }

    if max_js_hops == 0 || !is_html {
        return Ok((final_host, content_type, body));
    }

    let text = String::from_utf8_lossy(&body);
    let js_redirect_re = Regex::new(r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#).unwrap();
    match js_redirect_re.captures(&text) {
        Some(cap) => {
            let next_url = absolute_url(&cap[1], &final_host);
            fetch_following_redirects(&next_url, &final_host, max_js_hops - 1)
        }
        None => Ok((final_host, content_type, body)),
    }
}

fn final_host_of(resp: &ureq::Response) -> Option<String> {
    let url = resp.get_url();
    let after_scheme = url.split("://").nth(1)?;
    Some(after_scheme.split('/').next()?.to_string())
}
