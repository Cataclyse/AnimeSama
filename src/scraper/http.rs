
use std::time::Duration;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub(super) fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 256
        && slug != "."
        && !slug.contains("..")
        && slug.chars().all(|c| {
            c.is_alphanumeric() || matches!(c, '-' | '_' | '.')
        })
}


pub(super) fn valid_season_path(path: &str) -> bool {
    if path.is_empty() || path.len() > 256 {
        return false;
    }

    let segments: Vec<&str> = path.split('/').collect();

    !segments.is_empty()
        && segments.len() <= 8
        && segments.iter().all(|segment| {
            !segment.is_empty()
                && segment.len() <= 96
                && *segment != "."
                && *segment != ".."
                && segment.chars().all(|c| {
                    c.is_alphanumeric()
                        || matches!(c, '-' | '_' | '.')
                })
        })
}

fn validate_site_url(input: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(input)
        .map_err(|_| "URL du catalogue invalide".to_string())?;

    if url.scheme() != "https"
        || url.host_str() != Some("anime-sama.to")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.fragment().is_some()
    {
        return Err("Destination du catalogue interdite".to_string());
    }

    if url.path() != "/"
        && !url.path().starts_with("/catalogue/")
        && url.path() != "/s2/scans/get_nb_chap_et_img.php"
    {
        return Err("Chemin distant non autorisé".to_string());
    }

    Ok(url)
}

pub(super) fn fetch_site_text(
    input: &str,
    timeout_seconds: u64,
    max_bytes: usize,
    referer: Option<&str>,
) -> Result<String, String> {
    let url = validate_site_url(input)?;

    // Les réponses ne peuvent pas rediriger la connexion ailleurs.
    // Le résolveur refuse les adresses privées ou réservées.
    let agent = ureq::AgentBuilder::new()
        .https_only(true)
        .redirects(0)
        .try_proxy_from_env(false)
        .resolver(crate::proxy::fetcher::resolve_public_addresses)
        .timeout(Duration::from_secs(timeout_seconds))
        .build();

    let mut request = agent
        .get(url.as_str())
        .set("User-Agent", USER_AGENT);

    if let Some(referer) = referer {
        let validated = validate_site_url(referer)?;
        request = request.set("Referer", validated.as_str());
    }

    let response = request
        .call()
        .map_err(|e| format!("Requête catalogue échouée: {}", e))?;

    let bytes = crate::proxy::fetcher::read_limited(
        response.into_reader(),
        max_bytes,
    )
    .map_err(|e| format!("Réponse catalogue trop volumineuse ou illisible: {}", e))?;

    // Le contenu d'une page peut comporter des caractères mal encodés.
    // Le décodage tolérant évite de rendre la page entière inaccessible.
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn accepts_expected_season_paths() {
        for path in [
            "saison1",
            "saison1/vostfr",
            "saison1/vf",
            "saison2/vostfr",
            "scan",
            "scans",
            "scan/vf",
            "hors-serie/vostfr",
        ] {
            assert!(valid_season_path(path), "{path}");
        }
    }

    #[test]
    fn rejects_unsafe_season_paths() {
        for path in [
            "",
            ".",
            "..",
            "/saison1/vostfr",
            "saison1//vostfr",
            "saison1/../vostfr",
            "saison1/%2e%2e",
            "saison1?x=1",
            "saison1#fragment",
            "saison1\\vostfr",
            "saison1/vostfr/../../api",
        ] {
            assert!(!valid_season_path(path), "{path}");
        }

        assert!(!valid_season_path(&"a".repeat(257)));
    }

    #[test]
    fn accepts_expected_site_urls() {
        for address in [
            "https://anime-sama.to/",
            "https://anime-sama.to/catalogue/",
            "https://anime-sama.to/catalogue/frieren/",
            "https://anime-sama.to/catalogue/?search=Frieren",
            "https://anime-sama.to/catalogue/frieren/saison1/vostfr/episodes.js",
            "https://anime-sama.to/s2/scans/get_nb_chap_et_img.php?oeuvre=Kagura%20Bachi",
        ] {
            assert!(validate_site_url(address).is_ok(), "{address}");
        }
    }

    #[test]
    fn rejects_external_or_unsafe_destinations() {
        for address in [
            "http://anime-sama.to/",
            "https://anime-sama.to.evil.invalid/",
            "https://127.0.0.1/",
            "https://[::1]/",
            "https://user:pass@anime-sama.to/catalogue/",
            "https://anime-sama.to:444/catalogue/",
            "https://anime-sama.to/api/auth/logout.php",
            "https://anime-sama.to/catalogue/#fragment",
            "not-a-url",
        ] {
            assert!(validate_site_url(address).is_err(), "{address}");
        }
    }

    #[test]
    fn validates_catalogue_slugs() {
        for slug in ["frieren", "kagura-bachi", "one-piece", "anime_2"] {
            assert!(valid_slug(slug), "{slug}");
        }

        for slug in [
            "",
            ".",
            "..",
            "../admin",
            "anime/autre",
            "anime?x=1",
            "anime#fragment",
            "%2e%2e",
        ] {
            assert!(!valid_slug(slug), "{slug}");
        }
    }
}
