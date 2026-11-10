use regex::Regex;

pub fn rewrite_body(body: &str, default_host: &str, content_type: &str, resume_seconds: f64, episode_num: u32, ad_shield: bool) -> String {
    if content_type.contains("mpegurl") {
        return rewrite_m3u8(body, default_host);
    }

    let mut html_body = body.to_string();
    if content_type.contains("text/html") {
        if ad_shield {
            html_body = crate::adblock::sanitize_embed_html(&html_body, default_host, resume_seconds, episode_num);
        } else {
            html_body = crate::adblock::inject_resume_only_html(&html_body, resume_seconds, episode_num);
        }
    }

    let attr_re = Regex::new(r#"(src|href)=(["'])([^"']+)(["'])"#).unwrap();
    let css_url_re = Regex::new(r#"url\((['"]?)([^)'"]+)(['"]?)\)"#).unwrap();

    let step1 = attr_re.replace_all(&html_body, |caps: &regex::Captures| {
        let attr = &caps[1];
        let quote_open = &caps[2];
        let url = &caps[3];
        let quote_close = &caps[4];
        let is_same_host = is_relative_or_same_host(url, default_host);
        if ad_shield && !is_same_host && crate::adblock::is_blocked_url(url) {
            format!("{}=\"about:blank\"", attr)
        } else {
            format!("{}={}{}{}", attr, quote_open, absolute_url(url, default_host), quote_close)
        }
    });

    let step2 = css_url_re.replace_all(&step1, |caps: &regex::Captures| {
        let quote_open = &caps[1];
        let url = &caps[2];
        let quote_close = &caps[3];
        let is_same_host = is_relative_or_same_host(url, default_host);
        if ad_shield && !is_same_host && crate::adblock::is_blocked_url(url) {
            String::new()
        } else {
            format!("url({}{}{})", quote_open, absolute_url(url, default_host), quote_close)
        }
    });

    step2.into_owned()
}

pub fn is_relative_or_same_host(url: &str, default_host: &str) -> bool {
    if !url.contains("://") && !url.starts_with("//") {
        return true;
    }
    let lower = url.to_lowercase();
    if let Some(after_scheme) = lower.split("://").nth(1) {
        let host = after_scheme.split('/').next().unwrap_or("").split(':').next().unwrap_or("");
        if host == default_host || host.ends_with(&format!(".{}", default_host)) {
            return true;
        }
    }
    false
}

pub fn rewrite_m3u8(body: &str, default_host: &str) -> String {
    body.lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                line.to_string()
            } else {
                absolute_url(trimmed, default_host)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn absolute_url(url: &str, default_host: &str) -> String {
    if url.starts_with("data:")
        || url.starts_with("javascript:")
        || url.starts_with('#')
        || url.starts_with("http://")
        || url.starts_with("https://")
    {
        return url.to_string();
    }

    if let Some(rest) = url.strip_prefix("//") {
        return format!("https://{}", rest);
    }
    if url.starts_with('/') {
        return format!("https://{}{}", default_host, url);
    }

    format!("https://{}/{}", default_host, url)
}


// Réécriture HLS utilisant l'URL complète du manifeste.
// Chaque destination passe par la validation existante du proxy.
fn hls_proxy_reference(
    base: &url::Url,
    reference: &str,
) -> Result<String, &'static str> {
    if reference.is_empty() || reference.len() > 8192 {
        return Err("Référence HLS invalide");
    }

    let mut target = base
        .join(reference)
        .map_err(|_| "Référence HLS malformée")?;

    target.set_fragment(None);

    crate::proxy::security::validate_proxy_url(target.as_str())?;

    let host = target.host_str().ok_or("Hôte HLS manquant")?;

    let mut path = format!("/proxy/{}{}", host, target.path());

    if let Some(query) = target.query() {
        path.push('?');
        path.push_str(query);
    }

    Ok(path)
}

pub fn rewrite_m3u8_for_proxy(
    body: &str,
    base_url: &str,
) -> Result<String, &'static str> {
    let base = url::Url::parse(base_url)
        .map_err(|_| "URL du manifeste invalide")?;

    crate::proxy::security::validate_proxy_url(base.as_str())?;

    let uri_re = Regex::new(r#"URI="([^"]+)""#)
        .map_err(|_| "Expression HLS invalide")?;

    let mut output = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            output.push(line.to_string());
            continue;
        }

        if !trimmed.starts_with('#') {
            output.push(hls_proxy_reference(&base, trimmed)?);
            continue;
        }

        let mut rewritten = String::new();
        let mut previous_end = 0;

        for captures in uri_re.captures_iter(line) {
            let matched = captures.get(0).unwrap();
            let reference = captures.get(1).unwrap().as_str();

            rewritten.push_str(&line[previous_end..matched.start()]);
            rewritten.push_str("URI=\"");
            rewritten.push_str(&hls_proxy_reference(&base, reference)?);
            rewritten.push('"');

            previous_end = matched.end();
        }

        rewritten.push_str(&line[previous_end..]);
        output.push(rewritten);
    }

    Ok(output.join("\n"))
}

#[cfg(test)]
mod hls_proxy_tests {
    use super::rewrite_m3u8_for_proxy;

    const BASE: &str =
        "https://prx-1353-ant-23.vmpx.online/hls/episode/master.m3u8";

    #[test]
    fn rewrites_relative_variant_with_query() {
        let body = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nlow/index.m3u8?token=abc&x=2";

        let result = rewrite_m3u8_for_proxy(body, BASE).unwrap();

        assert!(result.contains(
            "/proxy/prx-1353-ant-23.vmpx.online/hls/episode/low/index.m3u8?token=abc&x=2"
        ));
    }

    #[test]
    fn rewrites_hls_uri_attributes() {
        let body = "#EXTM3U\n#EXT-X-MEDIA:TYPE=AUDIO,URI=\"audio/track.m3u8?token=xyz\"";

        let result = rewrite_m3u8_for_proxy(body, BASE).unwrap();

        assert!(result.contains(
            "URI=\"/proxy/prx-1353-ant-23.vmpx.online/hls/episode/audio/track.m3u8?token=xyz\""
        ));
    }

    #[test]
    fn rejects_untrusted_destinations() {
        let body = "#EXTM3U\nhttps://untrusted.example/video.ts";

        assert!(rewrite_m3u8_for_proxy(body, BASE).is_err());
    }
}
