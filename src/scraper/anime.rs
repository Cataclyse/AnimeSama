use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeSeason {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeDetails {
    pub slug: String,
    pub title: String,
    pub banner: String,
    pub synopsis: String,
    pub genres: Vec<String>,
    pub seasons: Vec<AnimeSeason>,
}

pub fn fetch_anime_details(slug: &str) -> Result<AnimeDetails, String> {
    let clean_slug = slug.trim().trim_matches('/');

    let url = format!("https://anime-sama.to/catalogue/{}/", clean_slug);
    let resp = ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| format!("Erreur réseau pour {}: {}", url, e))?;

    let html = resp.into_string().map_err(|e| format!("Erreur lecture réponse: {}", e))?;

    let title_re = Regex::new(r#"<h1[^>]*id=["']titreOeuvre["'][^>]*>(.*?)</h1>"#).unwrap();
    let fallback_title_re = Regex::new(r#"<h1[^>]*>(.*?)</h1>"#).unwrap();
    let title = title_re
        .captures(&html)
        .or_else(|| fallback_title_re.captures(&html))
        .map(|c| c[1].trim().to_string())
        .unwrap_or_else(|| clean_slug.replace('-', " "));

    let syn_re = Regex::new(r#"<p[^>]*id=["']synopsisText["'][^>]*>(.*?)</p>"#).unwrap();
    let syn_re2 = Regex::new(r#"<p[^>]*id=["']synopsis["'][^>]*>(.*?)</p>"#).unwrap();
    let syn_fallback = Regex::new(r#"class=["'][^"']*synopsis[^"']*["'][^>]*>(.*?)</p>"#).unwrap();
    let synopsis = syn_re
        .captures(&html)
        .or_else(|| syn_re2.captures(&html))
        .or_else(|| syn_fallback.captures(&html))
        .map(|c| {
            let tag_strip = Regex::new(r#"<[^>]+>"#).unwrap();
            tag_strip.replace_all(&c[1], "").trim().to_string()
        })
        .unwrap_or_default();

    let genre_re = Regex::new(r#"<a[^>]*href=["']/catalogue/\?[^"']*genre[^"']*["'][^>]*>(.*?)</a>"#).unwrap();
    let mut genres: Vec<String> = genre_re
        .captures_iter(&html)
        .map(|c| c[1].trim().to_string())
        .filter(|g| !g.is_empty())
        .collect();
    if genres.is_empty() {
        let alt_genre_re = Regex::new(r#"class=["'][^"']*genre[^"']*["'][^>]*>(.*?)<"#).unwrap();
        genres = alt_genre_re
            .captures_iter(&html)
            .map(|c| c[1].trim().to_string())
            .filter(|g| !g.is_empty() && g.len() < 30)
            .collect();
    }

    let season_re = Regex::new(r#"panneauAnime\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).unwrap();
    let scan_re = Regex::new(r#"panneauScan\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).unwrap();

    let mut seasons: Vec<AnimeSeason> = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for cap in season_re.captures_iter(&html) {
        let name = cap[1].trim().to_string();
        let path = cap[2].trim().trim_matches('/').to_string();

        if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") || name.starts_with("/*") {
            continue;
        }

        if seen_paths.insert(path.clone()) {
            seasons.push(AnimeSeason { name, path });
        }
    }

    for cap in scan_re.captures_iter(&html) {
        let name = cap[1].trim().to_string();
        let path = cap[2].trim().trim_matches('/').to_string();

        if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") || name.starts_with("/*") {
            continue;
        }

        if seen_paths.insert(path.clone()) {
            seasons.push(AnimeSeason {
                name: format!("Scan: {}", name),
                path,
            });
        }
    }

    if seasons.is_empty() {
        seasons.push(AnimeSeason {
            name: "Saison 1".to_string(),
            path: "saison1/vostfr".to_string(),
        });
    }

    let banner = format!("https://cdn.jsdelivr.net/gh/Anime-Sama/IMG@img/contenu/{}.jpg", clean_slug);

    let details = AnimeDetails {
        slug: clean_slug.to_string(),
        title,
        banner,
        synopsis,
        genres,
        seasons,
    };

    Ok(details)
}
