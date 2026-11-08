use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonEpisodes {
    pub slug: String,
    pub season: String,
    pub total_episodes: usize,
    pub players: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub available_languages: Vec<String>,
}

pub fn fetch_episodes(slug: &str, season_path: &str) -> Result<SeasonEpisodes, String> {
    let clean_slug = slug.trim().trim_matches('/');
    let mut clean_season = season_path.trim().trim_matches('/').to_string();
    if !clean_season.contains('/') && !clean_season.to_lowercase().starts_with("scan") {
        clean_season = format!("{}/vostfr", clean_season);
    }

    let base_url = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, clean_season);
    let ep_url = format!("https://anime-sama.to/catalogue/{}/{}/episodes.js", clean_slug, clean_season);

    let base_url_for_lang = base_url.clone();
    let lang_handle = std::thread::spawn(move || {
        let mut langs = Vec::new();
        if let Ok(resp_html) = ureq::get(&base_url_for_lang)
            .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .set("Referer", "https://anime-sama.to/")
            .timeout(std::time::Duration::from_secs(6))
            .call()
        {
            if let Ok(html) = resp_html.into_string() {
                let lang_re = Regex::new(r#"(?:var|let|const)\s+languesDispo\s*=\s*\[([^\]]+)\]"#).unwrap();
                if let Some(cap) = lang_re.captures(&html) {
                    let str_re = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();
                    for s in str_re.captures_iter(&cap[1]) {
                        let l = s[1].trim().to_lowercase();
                        if !l.is_empty() && !langs.contains(&l) {
                            langs.push(l);
                        }
                    }
                }
                if langs.is_empty() {
                    let switch_re = Regex::new(r#"id=["']switch([A-Za-z0-9]+)["']"#).unwrap();
                    for cap in switch_re.captures_iter(&html) {
                        let l = cap[1].trim().to_lowercase();
                        if !l.is_empty() && !langs.contains(&l) {
                            langs.push(l);
                        }
                    }
                }
            }
        }
        langs
    });

    let resp = ureq::get(&ep_url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .set("Referer", &base_url)
        .timeout(std::time::Duration::from_secs(8))
        .call();

    let mut actual_season = clean_season.clone();
    let js = match resp {
        Ok(r) => r.into_string().map_err(|e| format!("Erreur lecture episodes.js: {}", e))?,
        Err(_) => {
            let season_base = clean_season.split('/').next().unwrap_or("saison1");
            let fallback_season = format!("{}/vostfr", season_base);
            let fallback_url = format!("https://anime-sama.to/catalogue/{}/{}/episodes.js", clean_slug, fallback_season);
            let fallback_base = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, fallback_season);

            let fallback_res = if fallback_season != clean_season {
                ureq::get(&fallback_url)
                    .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                    .set("Referer", &fallback_base)
                    .timeout(std::time::Duration::from_secs(8))
                    .call()
                    .ok()
            } else {
                None
            };

            if let Some(r) = fallback_res {
                actual_season = fallback_season;
                r.into_string().map_err(|e| format!("Erreur lecture episodes.js fallback: {}", e))?
            } else {
                let first_path = fetch_first_season_path(clean_slug);
                if let Some(first_season) = first_path {
                    let first_url = format!("https://anime-sama.to/catalogue/{}/{}/episodes.js", clean_slug, first_season);
                    let first_base = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, first_season);
                    let res_first = ureq::get(&first_url)
                        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                        .set("Referer", &first_base)
                        .timeout(std::time::Duration::from_secs(8))
                        .call()
                        .map_err(|e| format!("Erreur réseau pour {}: {}", first_url, e))?;
                    actual_season = first_season;
                    res_first.into_string().map_err(|e| format!("Erreur lecture episodes.js catalogue fallback: {}", e))?
                } else {
                    return Err(format!("Erreur réseau pour {}", ep_url));
                }
            }
        }
    };

    let array_re = Regex::new(r#"(?:var|let|const)\s+([a-zA-Z0-9_]+)\s*=\s*\[([^\]]*)\]"#).unwrap();
    let str_re = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();

    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    let mut max_eps = 0;

    for cap in array_re.captures_iter(&js) {
        let var_name = cap[1].to_string();
        let content = &cap[2];

        if !var_name.starts_with("eps") {
            continue;
        }

        let mut urls = Vec::new();
        for s in str_re.captures_iter(content) {
            let u = s[1].trim().to_string();
            if !u.is_empty() {
                urls.push(u);
            }
        }

        if !urls.is_empty() {
            if urls.len() > max_eps {
                max_eps = urls.len();
            }
            groups.push((var_name, urls));
        }
    }

    let brand_handles: Vec<_> = groups
        .iter()
        .map(|(_, urls)| {
            let first = urls.first().cloned();
            std::thread::spawn(move || first.and_then(|u| detect_hoster_brand(&u)))
        })
        .collect();

    let mut players: HashMap<String, Vec<String>> = HashMap::new();
    for ((var_name, urls), handle) in groups.into_iter().zip(brand_handles) {
        let detected_brand = handle.join().unwrap_or(None);
        let host_name = detected_brand
            .or_else(|| urls.first().and_then(|u| host_label(u)))
            .unwrap_or_else(|| format!("Lecteur {}", var_name.trim_start_matches("eps")));

        let label = if players.contains_key(&host_name) {
            format!("{} ({})", host_name, var_name)
        } else {
            host_name
        };

        players.insert(label, urls);
    }

    let mut available_languages = lang_handle.join().unwrap_or_default();
    if available_languages.is_empty() {
        let current_lang = actual_season.split('/').nth(1).unwrap_or("vostfr").to_lowercase();
        available_languages.push(current_lang);
    }

    let season_episodes = SeasonEpisodes {
        slug: clean_slug.to_string(),
        season: actual_season,
        total_episodes: max_eps,
        players,
        available_languages,
    };

    Ok(season_episodes)
}

pub fn detect_hoster_brand(url: &str) -> Option<String> {
    let lower = url.to_lowercase();
    let after_scheme = lower.split("://").nth(1).unwrap_or(&lower);
    let host = after_scheme.split('/').next().unwrap_or("").split(':').next().unwrap_or("");
    let path = after_scheme.split('/').nth(1).unwrap_or("");

    if host == "ansembed.net"
        || host.contains("vidmoly")
        || host.contains("vmpx")
        || host.contains("vmcld")
        || host.contains("staticmoly")
    {
        return Some("VidMoly".to_string());
    }

    if host.contains("uqload") {
        return Some("Uqload".to_string());
    }

    if host.contains("sibnet.ru") {
        return Some("Sibnet".to_string());
    }

    if host.contains("sendvid.com") {
        return Some("Sendvid".to_string());
    }

    if host.contains("streamtape.com") || host.contains("streamtape.to") || host.contains("streamtape.net") {
        return Some("Streamtape".to_string());
    }

    if host.contains("dood") || host.contains("ds2play") || host.contains("doodstream") {
        return Some("Doodstream".to_string());
    }

    if host.contains("streamwish") || host.contains("wishembed") || host.contains("wishfast") {
        return Some("Streamwish".to_string());
    }

    if host.contains("filemoon") {
        return Some("Filemoon".to_string());
    }

    if host.contains("mp4upload.com") {
        return Some("Mp4upload".to_string());
    }

    if host.contains("vidoza.net") {
        return Some("Vidoza".to_string());
    }

    if host.contains("oneupload.to") {
        return Some("Oneupload".to_string());
    }

    if host.contains("minochinos.com")
        || host.contains("smoothpre.com")
        || host.contains("morencius.com")
        || host.contains("movearnpre.com")
        || host.contains("earnvids.com")
    {
        return Some("Earnvids".to_string());
    }

    if host.contains("voe.sx")
        || host.contains("embed4me.com")
        || host.contains("player4me.com")
        || host.contains("4meplayer.com")
        || host.contains("teresapoliticallearn.com")
        || host.contains("megankiddifference.com")
        || host.contains("jeremyparticipantanything.com")
        || host.contains("jamesbornmain.com")
        || host.contains("claireguessthousand.com")
        || host.contains("derricknationtop.com")
        || host.contains("jamespatterncamera.com")
        || host.contains("ashleyhardat.com")
        || host.contains("shanelocaldog.com")
        || host.contains("shannoncoldalready.com")
    {
        return Some("VOE".to_string());
    }

    let brand_signatures = [
        ("VidMoly", "vidmoly"),
        ("VidMoly", "ansembed"),
        ("Uqload", "uqload"),
        ("Sibnet", "sibnet"),
        ("Sendvid", "sendvid"),
        ("Streamtape", "streamtape"),
        ("Doodstream", "dood"),
        ("Streamwish", "streamwish"),
        ("Filemoon", "filemoon"),
        ("Mp4upload", "mp4upload"),
        ("Vidoza", "vidoza"),
        ("Oneupload", "oneupload"),
        ("Earnvids", "earnvids"),
        ("Earnvids", "minochinos"),
        ("Earnvids", "smoothpre"),
        ("VOE", "voe"),
    ];

    if path == "e" || path.starts_with("e/") {
        if let Some(html) = fetch_embed_html(url, 2) {
            let lower_html = html.to_lowercase();
            for (brand, sig) in brand_signatures {
                if lower_html.contains(sig) {
                    return Some(brand.to_string());
                }
            }
        }
    }

    None
}

fn fetch_embed_html(url: &str, max_hops: u8) -> Option<String> {
    if max_hops == 0 {
        return None;
    }
    let resp = ureq::get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .set("Referer", "https://anime-sama.to/")
        .timeout(std::time::Duration::from_secs(3))
        .call()
        .ok()?;

    let text = resp.into_string().ok()?;
    let js_redirect_re = Regex::new(r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#).ok()?;
    if let Some(cap) = js_redirect_re.captures(&text) {
        let next_url = &cap[1];
        if next_url.starts_with("http://") || next_url.starts_with("https://") {
            return fetch_embed_html(next_url, max_hops - 1);
        }
    }
    Some(text)
}

fn host_label(url: &str) -> Option<String> {
    let lower = url.to_lowercase();
    let after_scheme = lower.split("://").nth(1)?;
    let host = after_scheme.split('/').next()?.split(':').next()?;
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 2 {
        let name = parts[parts.len() - 2];
        if !name.is_empty() {
            let mut c = name.chars();
            return Some(match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            });
        }
    }
    None
}

fn fetch_first_season_path(slug: &str) -> Option<String> {
    let url = format!("https://anime-sama.to/catalogue/{}/", slug);
    let resp = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(6))
        .call()
        .ok()?;
    let html = resp.into_string().ok()?;
    let season_re = Regex::new(r#"panneauAnime\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).ok()?;
    for cap in season_re.captures_iter(&html) {
        let name = cap[1].trim();
        let path = cap[2].trim().trim_matches('/');
        if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") || name.starts_with("/*") {
            continue;
        }
        if !path.is_empty() {
            let normalized = if path.contains('/') {
                path.to_string()
            } else {
                format!("{}/vostfr", path)
            };
            return Some(normalized);
        }
    }
    None
}
