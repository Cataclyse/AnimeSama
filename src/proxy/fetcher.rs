use super::security::{addresses_allowed, validate_proxy_url};
use regex::Regex;
use std::io::{self, Read};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use url::Url;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
     AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36";

pub enum FetchError {
    Status(u16, ureq::Response),
    Other(String),
}

pub(crate) const MAX_TEXT_BODY_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn read_limited<R: Read>(reader: R, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let limit = (max_bytes as u64).saturating_add(1);

    reader.take(limit).read_to_end(&mut bytes)?;

    if bytes.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Réponse distante trop volumineuse",
        ));
    }

    Ok(bytes)
}

fn trace_proxy_hop(kind: &str, base: &str, destination: &str) {
    let parsed = Url::parse(base).ok();

    let from = parsed
        .as_ref()
        .and_then(|u| u.host_str())
        .unwrap_or("?");

    let target = parsed
        .as_ref()
        .and_then(|u| u.join(destination).ok());

    let to = target
        .as_ref()
        .and_then(|u| u.host_str())
        .unwrap_or("?");

    eprintln!(
        "[proxy-hop] kind={} from={} to={}",
        kind, from, to
    );
}

fn checked_next_url(base: &str, destination: &str) -> Result<String, FetchError> {
    let base_url = Url::parse(base).map_err(|e| FetchError::Other(e.to_string()))?;

    let next = base_url
        .join(destination)
        .map_err(|e| FetchError::Other(e.to_string()))?;

    validate_proxy_url(next.as_str()).map_err(|e| {
        eprintln!(
            "[proxy-redirect-denied] from={} to={} reason={}",
            base_url.host_str().unwrap_or("<absent>"),
            next.host_str().unwrap_or("<absent>"),
            e
        );
        FetchError::Other(e.to_string())
    })?;

    Ok(next.into())
}

fn absolute_dns_addr(addr: &str) -> io::Result<String> {
    let (host, port) = addr
        .rsplit_once(':')
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Adresse DNS invalide"))?;

    if host.is_empty()
        || !host.is_ascii()
        || host.contains(':')
        || host.contains('/')
        || host.contains('[')
        || host.contains(']')
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Nom DNS invalide",
        ));
    }

    let port: u16 = port
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Port DNS invalide"))?;

    if port == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Port DNS nul"));
    }

    Ok(format!("{}.:{}", host.trim_end_matches('.'), port))
}

pub(crate) fn resolve_public_addresses(
    addr: &str,
) -> io::Result<Vec<SocketAddr>> {
    let absolute_addr = absolute_dns_addr(addr)?;
    let addresses: Vec<SocketAddr> =
        absolute_addr.to_socket_addrs()?.collect();

    if !addresses_allowed(&addresses) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Destination DNS non autorisée",
        ));
    }

    Ok(addresses)
}

pub enum FetchBody {
    Buffered(Vec<u8>),
    Streaming(ureq::Response),
}

pub struct FetchResult {
    pub final_url: String,
    pub final_host: String,
    pub content_type: String,
    pub status: u16,
    pub body: FetchBody,
}

pub fn is_text_content_type(content_type: &str) -> bool {
    let ct = content_type.to_ascii_lowercase();
    ct.contains("text/")
        || ct.contains("javascript")
        || ct.contains("json")
        || ct.contains("mpegurl")
        || ct.contains("xml")
}

pub fn valid_single_range(value: &str) -> bool {
    let Some(spec) = value.strip_prefix("bytes=") else {
        return false;
    };

    if spec.len() > 64 || spec.contains(',') {
        return false;
    }

    let Some((start, end)) = spec.split_once('-') else {
        return false;
    };

    if start.is_empty() {
        return end.parse::<u64>().map(|n| n > 0).unwrap_or(false);
    }

    let Ok(start) = start.parse::<u64>() else {
        return false;
    };

    end.is_empty() || end.parse::<u64>().map(|n| n >= start).unwrap_or(false)
}

pub fn fetch_following_redirects(
    url: &str,
    _fallback_host: &str,
    max_js_hops: u8,
    range: Option<&str>,
) -> Result<FetchResult, FetchError> {
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .https_only(true)
        .try_proxy_from_env(false)
        .timeout(Duration::from_secs(20))
        .resolver(resolve_public_addresses)
        .build();

    let js_redirect_re = Regex::new(r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#).unwrap();

    let mut current = url.to_owned();
    let mut hops = 0u8;

    loop {
        validate_proxy_url(&current).map_err(|e| FetchError::Other(e.to_string()))?;

        let mut request = agent
            .get(&current)
            .set("User-Agent", USER_AGENT)
            .set("Referer", "https://anime-sama.to/")
            .set("Accept", "*/*")
            .set("Accept-Encoding", "identity");

        if let Some(value) = range {
            request = request.set("Range", value);
        }

        let result = request.call();

        let resp = match result {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) if (300..400).contains(&code) => resp,
            Err(ureq::Error::Status(code, resp)) => return Err(FetchError::Status(code, resp)),
            Err(e) => return Err(FetchError::Other(e.to_string())),
        };

        if (300..400).contains(&resp.status()) {
            let location = resp
                .header("Location")
                .ok_or_else(|| FetchError::Other("Redirection sans Location".into()))?;

            if hops >= max_js_hops {
                return Err(FetchError::Other(
                    "Nombre maximal de redirections atteint".into(),
                ));
            }

            trace_proxy_hop("http", &current, location);
            current = checked_next_url(&current, location)?;
            hops += 1;
            continue;
        }

        let final_host = Url::parse(&current)
            .ok()
            .and_then(|u| u.host_str().map(str::to_owned))
            .ok_or_else(|| FetchError::Other("Hôte final invalide".into()))?;

        let content_type = resp
            .header("Content-Type")
            .unwrap_or("application/octet-stream")
            .to_owned();

        let status = resp.status();

        if !is_text_content_type(&content_type) {
            return Ok(FetchResult {
                final_url: current.clone(),
                final_host,
                content_type,
                status,
                body: FetchBody::Streaming(resp),
            });
        }

        if range.is_some() && status == 206 {
            return Err(FetchError::Other(
                "Range partiel sur contenu réécrit non pris en charge".into(),
            ));
        }

        let body = read_limited(resp.into_reader(), MAX_TEXT_BODY_BYTES)
            .map_err(|e| FetchError::Other(e.to_string()))?;

        if content_type.to_ascii_lowercase().contains("text/html") {
            let html = String::from_utf8_lossy(&body);

            if let Some(capture) = js_redirect_re.captures(&html) {
                if hops >= max_js_hops {
                    return Err(FetchError::Other(
                        "Nombre maximal de redirections atteint".into(),
                    ));
                }

                trace_proxy_hop("javascript", &current, &capture[1]);
                current = checked_next_url(&current, &capture[1])?;

                hops += 1;
                continue;
            }
        }

        return Ok(FetchResult {
            final_url: current.clone(),
            final_host,
            content_type,
            status,
            body: FetchBody::Buffered(body),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_streaming_content() {
        assert!(is_text_content_type("text/html"));
        assert!(is_text_content_type("application/vnd.apple.mpegurl"));
        assert!(!is_text_content_type("video/mp4"));
        assert!(!is_text_content_type("video/mp2t"));
        assert!(!is_text_content_type("image/jpeg"));
        assert!(!is_text_content_type("application/octet-stream"));
    }

    #[test]
    fn validates_single_byte_ranges() {
        for range in ["bytes=0-", "bytes=100-200", "bytes=-500"] {
            assert!(valid_single_range(range), "{range}");
        }

        for range in [
            "bytes=200-100",
            "bytes=0-1,5-10",
            "bytes=-0",
            "items=0-10",
            "bytes=abc-",
        ] {
            assert!(!valid_single_range(range), "{range}");
        }
    }

    #[test]
    fn accepts_body_at_limit() {
        let data = vec![42u8; 8];
        assert_eq!(read_limited(&data[..], 8).unwrap(), data);
    }

    #[test]
    fn rejects_oversized_body() {
        let data = vec![42u8; 9];
        assert!(read_limited(&data[..], 8).is_err());

        // Le lecteur infini doit s'arrêter dès la limite.
        assert!(read_limited(std::io::repeat(42), 8).is_err());
    }

    #[test]
    fn uses_absolute_dns_names() {
        assert_eq!(
            absolute_dns_addr("uqload.to:443").unwrap(),
            "uqload.to.:443"
        );

        assert_eq!(absolute_dns_addr("voe.sx:443").unwrap(), "voe.sx.:443");

        assert!(absolute_dns_addr("voe.sx:0").is_err());
        assert!(absolute_dns_addr("voe.sx:notaport").is_err());
        assert!(absolute_dns_addr("[::1]:443").is_err());
    }

    #[test]
    fn rejects_unsafe_redirects() {
        let base = "https://voe.sx/e/123";

        assert!(checked_next_url(base, "https://127.0.0.1/admin").is_err());

        assert!(checked_next_url(base, "http://voe.sx/video").is_err());

        assert!(checked_next_url(base, "https://evil.example/").is_err());

        assert!(checked_next_url(base, "/player").is_ok());
    }
}
