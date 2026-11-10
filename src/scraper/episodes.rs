use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanChapter {
    pub number: usize,
    pub name: String,
    pub pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanData {
    pub oeuvre: String,
    pub total_chapters: usize,
    pub chapters: Vec<ScanChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonEpisodes {
    pub slug: String,
    pub season: String,
    pub total_episodes: usize,
    pub players: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub available_languages: Vec<String>,
    #[serde(default)]
    pub scan_data: Option<ScanData>,
}

pub fn url_encode(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

fn unescape_html(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&#039;", "'")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
}

pub fn parse_scan_custom_names(html: &str, total_keys: usize) -> Vec<String> {
    let mut custom_names: Vec<String> = Vec::new();
    if html.contains("resetListe") {
        let cmd_re = Regex::new(r#"(creerListe\s*\(\s*(\d+)\s*,\s*(\d+)\s*\)|newSP\s*\(\s*([^)]+)\s*\)|newSPF\s*\(\s*([^)]+)\s*\)|finirListe\s*\(\s*(\d+)\s*\))"#).unwrap();
        let mut chap_retards: usize = 0;
        for cap in cmd_re.captures_iter(html) {
            let full_cmd = &cap[1];
            if full_cmd.starts_with("creerListe") {
                if let (Some(s_m), Some(e_m)) = (cap.get(2), cap.get(3)) {
                    if let (Ok(s), Ok(e)) = (s_m.as_str().parse::<usize>(), e_m.as_str().parse::<usize>()) {
                        for n in s..=e {
                            custom_names.push(format!("Chapitre {}", n));
                        }
                    }
                }
            } else if full_cmd.starts_with("newSP(") {
                if let Some(spe_m) = cap.get(4) {
                    let spe = spe_m.as_str().trim().trim_matches(|c| c == '\'' || c == '"');
                    custom_names.push(format!("Chapitre {}", spe));
                    chap_retards += 1;
                }
            } else if full_cmd.starts_with("newSPF") {
                if let Some(spe_m) = cap.get(5) {
                    let spe = spe_m.as_str().trim().trim_matches(|c| c == '\'' || c == '"');
                    custom_names.push(spe.to_string());
                    chap_retards += 1;
                }
            } else if full_cmd.starts_with("finirListe") {
                if let Some(s_m) = cap.get(6) {
                    if let Ok(s) = s_m.as_str().parse::<usize>() {
                        let end_val = total_keys.saturating_sub(chap_retards);
                        if s <= end_val {
                            for n in s..=end_val {
                                custom_names.push(format!("Chapitre {}", n));
                            }
                        }
                    }
                }
            }
        }
    }
    custom_names
}

pub fn fetch_scan_episodes(slug: &str, season_path: &str) -> Result<SeasonEpisodes, String> {
    let clean_slug = slug.trim().trim_matches('/');
    let clean_season = season_path.trim().trim_matches('/');

    if !super::http::valid_slug(clean_slug)
        || !super::http::valid_season_path(clean_season)
    {
        return Err("Chemin de scan invalide".to_string());
    }

    let scan_page_url = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, clean_season);
    let scan_referer =
        format!("https://anime-sama.to/catalogue/{}/", clean_slug);

    let html = super::http::fetch_site_text(
        &scan_page_url,
        8,
        4 * 1024 * 1024,
        Some(&scan_referer),
    )?;

    let titre_re = Regex::new(r#"<h3[^>]*id=["']titreOeuvre["'][^>]*>(.*?)</h3>"#).unwrap();
    let raw_titre = titre_re
        .captures(&html)
        .map(|c| c[1].trim().to_string())
        .unwrap_or_else(|| clean_slug.replace('-', " "));
    let titre_oeuvre = unescape_html(&raw_titre);

    let mut langs = Vec::new();
    let switch_re = Regex::new(r#"id=["']switch([A-Za-z0-9]+)["']"#).unwrap();
    for cap in switch_re.captures_iter(&html) {
        let l = cap[1].trim().to_lowercase();
        if !l.is_empty() && !langs.contains(&l) {
            langs.push(l);
        }
    }
    if langs.is_empty() {
        let parts: Vec<&str> = clean_season.split('/').collect();
        if parts.len() > 1 {
            langs.push(parts[1].to_lowercase());
        } else {
            langs.push("vf".to_string());
        }
    }

    let encoded_oeuvre = url_encode(&titre_oeuvre);
    let api_url = format!("https://anime-sama.to/s2/scans/get_nb_chap_et_img.php?oeuvre={}", encoded_oeuvre);
    let json_text = super::http::fetch_site_text(
        &api_url,
        8,
        2 * 1024 * 1024,
        Some(&scan_page_url),
    )?;

    let data_oeuvre: HashMap<String, usize> = serde_json::from_str(&json_text)
        .map_err(|e| format!("Erreur parsing json scans: {}", e))?;

    let total_keys = data_oeuvre.len();
    if total_keys == 0 {
        return Err("Aucun chapitre disponible pour cette œuvre".to_string());
    }

    let custom_names = parse_scan_custom_names(&html, total_keys);

    let total_chapters = if custom_names.is_empty() {
        total_keys
    } else {
        custom_names.len()
    };

    let mut chapters = Vec::with_capacity(total_chapters);
    let mut scan_urls = Vec::with_capacity(total_chapters);

    for idx in 0..total_chapters {
        let chap_num = idx + 1;
        let pages = data_oeuvre.get(&chap_num.to_string()).copied().unwrap_or(0);
        let name = if idx < custom_names.len() {
            custom_names[idx].clone()
        } else {
            format!("Chapitre {}", chap_num)
        };

        scan_urls.push(format!(
            "https://anime-sama.to/s2/scans/{}/{}/?pages={}",
            encoded_oeuvre, chap_num, pages
        ));

        chapters.push(ScanChapter {
            number: chap_num,
            name,
            pages,
        });
    }

    let mut players = HashMap::new();
    players.insert("Scans".to_string(), scan_urls);

    Ok(SeasonEpisodes {
        slug: clean_slug.to_string(),
        season: clean_season.to_string(),
        total_episodes: total_chapters,
        players,
        available_languages: langs,
        scan_data: Some(ScanData {
            oeuvre: titre_oeuvre,
            total_chapters,
            chapters,
        }),
    })
}

pub fn fetch_episodes(slug: &str, season_path: &str) -> Result<SeasonEpisodes, String> {
    let clean_slug = slug.trim().trim_matches('/');
    let mut clean_season = season_path.trim().trim_matches('/').to_string();

    if !super::http::valid_slug(clean_slug)
        || !super::http::valid_season_path(&clean_season)
    {
        return Err("Slug ou chemin de saison invalide".to_string());
    }

    if clean_season.to_lowercase().contains("scan") {
        return fetch_scan_episodes(clean_slug, &clean_season);
    }
    if !clean_season.contains('/') {
        clean_season = format!("{}/vostfr", clean_season);
    }

    let base_url = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, clean_season);
    let ep_url = format!("https://anime-sama.to/catalogue/{}/{}/episodes.js", clean_slug, clean_season);

    let base_url_for_lang = base_url.clone();
    let lang_handle = std::thread::spawn(move || {
        let mut langs = Vec::new();
        if let Ok(html) = super::http::fetch_site_text(
            &base_url_for_lang,
            6,
            4 * 1024 * 1024,
            Some("https://anime-sama.to/"),
        ) {
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
        langs
    });

    let resp = super::http::fetch_site_text(
        &ep_url,
        8,
        4 * 1024 * 1024,
        Some(&base_url),
    );

    let mut actual_season = clean_season.clone();
    let js = match resp {
        Ok(js) => js,
        Err(_) => {
            let season_base = clean_season.split('/').next().unwrap_or("saison1");
            let fallback_season = format!("{}/vostfr", season_base);
            let fallback_url = format!("https://anime-sama.to/catalogue/{}/{}/episodes.js", clean_slug, fallback_season);
            let fallback_base = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, fallback_season);

            let fallback_res = if fallback_season != clean_season {
                super::http::fetch_site_text(
                    &fallback_url,
                    8,
                    4 * 1024 * 1024,
                    Some(&fallback_base),
                )
                .ok()
            } else {
                None
            };

            if let Some(r) = fallback_res {
                actual_season = fallback_season;
                r
            } else {
                let first_path = fetch_first_season_path(clean_slug);
                if let Some(first_season) = first_path {
                    let first_url = format!("https://anime-sama.to/catalogue/{}/{}/episodes.js", clean_slug, first_season);
                    let first_base = format!("https://anime-sama.to/catalogue/{}/{}/", clean_slug, first_season);
                    let first_js = super::http::fetch_site_text(
                        &first_url,
                        8,
                        4 * 1024 * 1024,
                        Some(&first_base),
                    )?;

                    actual_season = first_season;
                    first_js
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
        scan_data: None,
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

    // Contrôler la destination avant toute connexion réseau.
    let parsed = url::Url::parse(url).ok()?;
    crate::proxy::security::validate_proxy_url(
        parsed.as_str(),
    ).ok()?;

    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .https_only(true)
        .try_proxy_from_env(false)
        .resolver(
            crate::proxy::fetcher::resolve_public_addresses,
        )
        .timeout(std::time::Duration::from_secs(3))
        .build();

    let resp = agent
        .get(parsed.as_str())
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36",
        )
        .set("Referer", "https://anime-sama.to/")
        .call()
        .ok()?;

    // Le HTML n'est utilisé que pour identifier l'hébergeur.
    const MAX_EMBED_HTML_BYTES: usize = 1024 * 1024;

    let bytes = crate::proxy::fetcher::read_limited(
        resp.into_reader(),
        MAX_EMBED_HTML_BYTES,
    ).ok()?;

    let text = String::from_utf8_lossy(&bytes).into_owned();

    let js_redirect_re = Regex::new(
        r#"window\.location\.href\s*=\s*['"]([^'"]+)['"]"#
    ).ok()?;

    if let Some(cap) = js_redirect_re.captures(&text) {
        let next_url = &cap[1];

        if next_url.starts_with("http://")
            || next_url.starts_with("https://")
        {
            // Le prochain appel revalide la destination et le DNS.
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

fn normalize_first_season_path(path: &str) -> Option<String> {
    let clean = path.trim().trim_matches('/');

    let normalized = if clean.contains('/') {
        clean.to_string()
    } else {
        format!("{}/vostfr", clean)
    };

    if super::http::valid_season_path(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

fn fetch_first_season_path(slug: &str) -> Option<String> {
    if !super::http::valid_slug(slug) {
        return None;
    }

    let url = format!("https://anime-sama.to/catalogue/{}/", slug);

    let html = super::http::fetch_site_text(
        &url,
        6,
        4 * 1024 * 1024,
        None,
    ).ok()?;

    let season_re = Regex::new(r#"panneauAnime\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).ok()?;
    for cap in season_re.captures_iter(&html) {
        let name = cap[1].trim();
        let path = cap[2].trim().trim_matches('/');
        if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") || name.starts_with("/*") {
            continue;
        }
        if let Some(normalized) = normalize_first_season_path(path) {
            return Some(normalized);
        }
    }
    None
}


#[cfg(test)]
mod safe_embed_detection_tests {
    use super::fetch_embed_html;

    #[test]
    fn rejects_untrusted_embed_destinations_before_network() {
        for url in [
            "http://127.0.0.1/",
            "https://127.0.0.1/",
            "https://[::1]/",
            "https://169.254.169.254/latest/meta-data/",
            "https://example.invalid/",
            "https://voe.sx.evil.invalid/",
            "https://user:pass@voe.sx/e/abc",
        ] {
            assert!(
                fetch_embed_html(url, 2).is_none(),
                "Destination non autorisée : {url}"
            );
        }
    }

    #[test]
    fn refuses_fetch_when_no_hops_remain() {
        assert!(
            fetch_embed_html("https://voe.sx/e/test", 0).is_none()
        );
    }
}


#[cfg(test)]
mod safe_first_season_tests {
    use super::normalize_first_season_path;

    #[test]
    fn preserves_valid_first_season_paths() {
        for (input, expected) in [
            ("saison1", "saison1/vostfr"),
            ("saison2/vf", "saison2/vf"),
            ("hors-serie/vostfr", "hors-serie/vostfr"),
            ("saison3/vostfr", "saison3/vostfr"),
        ] {
            assert_eq!(
                normalize_first_season_path(input).as_deref(),
                Some(expected),
            );
        }
    }

    #[test]
    fn rejects_invalid_first_season_paths() {
        for input in [
            "",
            "..",
            "../api",
            "saison1/../admin",
            "saison1/%2e%2e",
            "saison1?x=1",
            "https://example.com/",
            "saison1//vostfr",
        ] {
            assert!(
                normalize_first_season_path(input).is_none(),
                "Chemin accepté : {input}",
            );
        }
    }
}
