use crate::catalogue::{CardItem, CatalogData, HeroSlide, Section};
use super::search::unescape_html;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub fn fetch_home() -> Result<CatalogData, String> {
        let resp = ureq::get("https://anime-sama.to/")
            .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .map_err(|e| format!("Erreur réseau page d'accueil: {}", e))?;

        let html = resp.into_string().map_err(|e| format!("Erreur lecture réponse: {}", e))?;

        let slides = parse_slides(&html);
        let mut seen_slugs: HashSet<String> = HashSet::new();
        let mut catalogue: Vec<CardItem> = Vec::new();

        let sections_defs = [
            ("containerAjoutsAnimes", "Derniers épisodes ajoutés"),
            ("containerAjoutsScans", "Derniers scans ajoutés"),
            ("containerSorties", "Derniers contenus sortis"),
            ("containerClassiques", "les classiques"),
            ("containerPepites", "découvrez des pépites"),
        ];

        let mut sections = Vec::new();
        for (container_id, title) in sections_defs {
            let cards = parse_cards_from_container(&html, container_id);
            for c in &cards {
                if seen_slugs.insert(c.slug.clone()) {
                    catalogue.push(c.clone());
                }
            }
            if !cards.is_empty() {
                sections.push(Section {
                    title: title.to_string(),
                    cards,
                });
            }
        }

        let days = [
            "Dimanche", "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi",
        ];
        let mut planning = HashMap::new();
        for day in days {
            let container_id = format!("container{}", day);
            let cards = parse_cards_from_container(&html, &container_id);
            for c in &cards {
                if seen_slugs.insert(c.slug.clone()) {
                    catalogue.push(c.clone());
                }
            }
            if !cards.is_empty() {
                let label = find_day_heading(&html, day).unwrap_or_else(|| format!("Sorties du {}", day));
                planning.insert(label, cards);
            }
        }

        if catalogue.is_empty() {
            return Err("Aucune carte trouvée sur la page d'accueil".to_string());
        }

        Ok(CatalogData {
            slides,
            planning,
            sections,
            catalogue,
        })
    }

fn parse_slides(html: &str) -> Vec<HeroSlide> {
    let slide_re = Regex::new(r#"(?s)<div class="ak-slide"[^>]*>(.*?)</div>\s*</div>\s*</div>"#).unwrap();
    let bg_re = Regex::new(r#"<div class="ak-slide-bg">\s*<img[^>]*src="([^"]+)""#).unwrap();
    let badge_re = Regex::new(r#"<span class="ak-badge[^"]*">(?:\s*<svg.*?</svg>\s*)?([^<]+)</span>"#).unwrap();
    let title_re = Regex::new(r#"<h2 class="ak-slide-title">(.*?)</h2>"#).unwrap();
    let genre_re = Regex::new(r#"<span class="ak-genre-tag">([^<]*)</span>"#).unwrap();
    let synopsis_re = Regex::new(r#"<p class="ak-slide-synopsis">(.*?)</p>"#).unwrap();
    let cta_re = Regex::new(r#"<a href="([^"]+)" class="ak-slide-cta">(?:\s*<svg.*?</svg>\s*)?([^<]+)"#).unwrap();
    let tag_strip = Regex::new(r#"<[^>]+>"#).unwrap();

    let mut slides = Vec::new();

    for cap in slide_re.captures_iter(html) {
        let block = &cap[1];

        let bg = bg_re.captures(block).map(|c| c[1].trim().to_string()).unwrap_or_default();
        let title = title_re
            .captures(block)
            .map(|c| unescape_html(tag_strip.replace_all(&c[1], "").trim()))
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }

        let badge = badge_re.captures(block).map(|c| unescape_html(c[1].trim())).unwrap_or_default();
        let synopsis = synopsis_re
            .captures(block)
            .map(|c| unescape_html(tag_strip.replace_all(&c[1], "").trim()))
            .unwrap_or_default();
        let genres: Vec<String> = genre_re
            .captures_iter(block)
            .map(|c| unescape_html(c[1].trim()))
            .filter(|g| !g.is_empty())
            .collect();

        let (cta_link, cta_text) = cta_re
            .captures(block)
            .map(|c| (c[1].trim().to_string(), c[2].trim().to_string()))
            .unwrap_or_default();

        let slug = cta_link
            .trim_matches('/')
            .trim_start_matches("catalogue/")
            .split('/')
            .next()
            .unwrap_or_default()
            .to_string();

        slides.push(HeroSlide {
            title,
            slug,
            badge,
            bg,
            genres,
            synopsis,
            cta_link,
            cta_text,
        });
    }

    slides
}

fn extract_container_block<'a>(html: &'a str, container_id: &str) -> Option<&'a str> {
    let needle = format!(r#"id="{}""#, container_id);
    let start = html.find(&needle)?;
    let open_div = html[..start].rfind("<div")?;
    let tag_end = html[start..].find('>')? + start + 1;

    
    let mut depth = 1;
    let mut pos = tag_end;
    let bytes = html.as_bytes();
    while depth > 0 && pos < bytes.len() {
        let rest = &html[pos..];
        let next_open = rest.find("<div");
        let next_close = rest.find("</div>");
        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                depth += 1;
                pos += o + 4;
            }
            (_, Some(c)) => {
                depth -= 1;
                pos += c + 6;
            }
            _ => break,
        }
    }

    Some(&html[open_div..pos])
}

pub fn parse_cards_from_container(html: &str, container_id: &str) -> Vec<CardItem> {
    let block = match extract_container_block(html, container_id) {
        Some(b) => b,
        None => return Vec::new(),
    };

    let header_re = Regex::new(
        r#"<div class="(as-carte[^"]*)"\s+data-url="([^"]*)"\s+data-cat="[^"]*"\s+data-nom="([^"]*)"\s+data-img="([^"]*)">"#,
    )
    .unwrap();
    let href_re = Regex::new(r#"<a class="as-carte-lien" href="([^"]+)""#).unwrap();
    let thumb_re = Regex::new(r#"<div class="as-carte-image">\s*<img[^>]*src="([^"]+)""#).unwrap();
    let badge_re = Regex::new(r#"<span class="as-badge[^"]*">([^<]+)</span>"#).unwrap();
    let flag_re = Regex::new(r#"<span class="as-drapeau">\s*<img[^>]*alt="([^"]*)""#).unwrap();
    let etat_re = Regex::new(r#"<span class="as-etat">([^<]+)</span>"#).unwrap();
    let info_re = Regex::new(r#"<div class="as-carte-info">(.*?)</div>"#).unwrap();
    let tag_strip = Regex::new(r#"<[^>]+>"#).unwrap();

    
    
    let headers: Vec<_> = header_re.captures_iter(block).collect();

    headers
        .iter()
        .enumerate()
        .filter_map(|(idx, cap)| {
            let classes = cap[1].to_string();
            let url = cap[2].trim().to_string();
            let nom = unescape_html(cap[3].trim());
            let img = cap[4].trim().to_string();

            let header_end = cap.get(0).unwrap().end();
            let segment_end = headers
                .get(idx + 1)
                .map(|next| next.get(0).unwrap().start())
                .unwrap_or(block.len());
            let inner = &block[header_end..segment_end];

            let slug = url
                .trim_matches('/')
                .trim_start_matches("catalogue/")
                .split('/')
                .next()
                .unwrap_or_default()
                .to_string();
            if slug.is_empty() || nom.is_empty() {
                return None;
            }

            let link = href_re.captures(inner).map(|c| c[1].trim().to_string()).unwrap_or_else(|| url.clone());
            let thumb_src = thumb_re.captures(inner).map(|c| c[1].trim().to_string());

            let badges: Vec<String> = badge_re
                .captures_iter(inner)
                .map(|c| unescape_html(c[1].trim()))
                .filter(|b| !b.is_empty())
                .collect();

            let flags: Vec<String> = flag_re
                .captures_iter(inner)
                .map(|c| unescape_html(c[1].trim()))
                .filter(|f| !f.is_empty())
                .collect();

            let etat = etat_re.captures(inner).map(|c| unescape_html(c[1].trim())).unwrap_or_default();

            let info = info_re
                .captures(inner)
                .map(|c| unescape_html(&tag_strip.replace_all(&c[1], " ").split_whitespace().collect::<Vec<_>>().join(" ")))
                .unwrap_or_default();

            let is_scan = classes.contains("as-carte-scan");

            let thumb = thumb_src.unwrap_or_else(|| img.clone());

            Some(CardItem {
                slug,
                nom,
                url,
                img: img.clone(),
                thumb,
                link,
                badges: if badges.is_empty() {
                    vec![if is_scan { "Scan".to_string() } else { "Anime".to_string() }]
                } else {
                    badges
                },
                flags,
                etat,
                info,
                genres: Vec::new(),
                synopsis: String::new(),
            })
        })
        .collect()
}

fn find_day_heading(html: &str, day: &str) -> Option<String> {
    let re = Regex::new(&format!(
        r#"(?s)<h2 class="titreJours[^"]*">.*?</a>\s*(Sorties du {}[^<]*)<a"#,
        regex::escape(day)
    ))
    .ok()?;
    re.captures(html).map(|c| c[1].trim().to_string())
}
