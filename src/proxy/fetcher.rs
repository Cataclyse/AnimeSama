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

fn checked_next_url(base: &str, destination: &str) -> Result<String, FetchError> {
    let base_url = Url::parse(base).map_err(|e| FetchError::Other(e.to_string()))?;

    let next = base_url
        .join(destination)
        .map_err(|e| FetchError::Other(e.to_string()))?;

    validate_proxy_url(next.as_str()).map_err(|e| FetchError::Other(e.to_string()))?;

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

pub fn fetch_following_redirects(
    url: &str,
    _fallback_host: &str,
    max_js_hops: u8,
) -> Result<(String, String, Vec<u8>), FetchError> {
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .https_only(true)
        .try_proxy_from_env(false)
        .timeout(Duration::from_secs(20))
        .resolver(|addr: &str| -> io::Result<Vec<SocketAddr>> {
            let absolute_addr = absolute_dns_addr(addr)?;
            let addresses: Vec<SocketAddr> = absolute_addr.to_socket_addrs()?.collect();

            if !addresses_allowed(&addresses) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Destination DNS non autorisée",
                ));
            }

            Ok(addresses)
        })
        .build();

    let mut current = url.to_string();
    let mut hops = 0u8;

    let js_redirect_re = Regex::new(r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#).unwrap();

    loop {
        validate_proxy_url(&current).map_err(|e| FetchError::Other(e.to_string()))?;

        let result = agent
            .get(&current)
            .set("User-Agent", USER_AGENT)
            .set("Referer", "https://anime-sama.to/")
            .set("Accept", "*/*")
            .call();

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
            .to_string();

        let is_html = content_type.contains("text/html");

        let mut body = Vec::new();
        resp.into_reader()
            .read_to_end(&mut body)
            .map_err(|e| FetchError::Other(e.to_string()))?;

        if is_html {
            let html = String::from_utf8_lossy(&body);

            if let Some(capture) = js_redirect_re.captures(&html) {
                if hops >= max_js_hops {
                    return Err(FetchError::Other(
                        "Nombre maximal de redirections atteint".into(),
                    ));
                }

                current = checked_next_url(&current, &capture[1])?;

                hops += 1;
                continue;
            }
        }

        return Ok((final_host, content_type, body));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
