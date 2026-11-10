pub mod anime;
pub mod episodes;
pub mod home;
pub mod search;
mod http;

#[allow(unused_imports)]
pub use anime::{AnimeDetails, AnimeSeason};
#[allow(unused_imports)]
pub use episodes::{detect_hoster_brand, SeasonEpisodes};

use crate::catalogue::{CardItem, CatalogData};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Scraper {
    details_cache: Arc<Mutex<HashMap<String, AnimeDetails>>>,
    episodes_cache: Arc<Mutex<HashMap<String, SeasonEpisodes>>>,
}

pub(crate) fn valid_episode_request(
    slug: &str,
    season_path: &str,
) -> bool {
    let clean_slug = slug.trim().trim_matches('/');
    let clean_season = season_path.trim().trim_matches('/');

    http::valid_slug(clean_slug)
        && http::valid_season_path(clean_season)
}

impl Scraper {
    pub fn new() -> Self {
        Scraper {
            details_cache: Arc::new(Mutex::new(HashMap::new())),
            episodes_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_details(&self, slug: &str) -> Result<AnimeDetails, String> {
        let clean_slug = slug.trim().trim_matches('/');
        {
            let cache = self.details_cache.lock().unwrap();
            if let Some(details) = cache.get(clean_slug) {
                return Ok(details.clone());
            }
        }
        let details = anime::fetch_anime_details(clean_slug)?;
        let mut cache = self.details_cache.lock().unwrap();
        cache.insert(clean_slug.to_string(), details.clone());
        Ok(details)
    }

    pub fn get_episodes(&self, slug: &str, season_path: &str) -> Result<SeasonEpisodes, String> {
        let clean_slug = slug.trim().trim_matches('/');
        let clean_season = season_path.trim().trim_matches('/');

        if !valid_episode_request(clean_slug, clean_season) {
            return Err("Slug ou chemin de saison invalide".to_string());
        }

        let cache_key = format!("{}:{}", clean_slug, clean_season);
        {
            let cache = self.episodes_cache.lock().unwrap();
            if let Some(episodes) = cache.get(&cache_key) {
                return Ok(episodes.clone());
            }
        }
        let episodes = episodes::fetch_episodes(clean_slug, clean_season)?;
        let mut cache = self.episodes_cache.lock().unwrap();
        cache.insert(cache_key, episodes.clone());
        Ok(episodes)
    }

    pub fn get_home(&self) -> Result<CatalogData, String> {
        home::fetch_home()
    }

    pub fn search_live(&self, query: &str) -> Result<Vec<CardItem>, String> {
        search::execute_search(query)
    }
}

#[cfg(test)]
mod tests;


#[cfg(test)]
mod episode_request_validation_tests {
    use super::valid_episode_request;

    #[test]
    fn accepts_valid_episode_requests() {
        for (slug, season) in [
            ("frieren", "saison1/vostfr"),
            ("frieren", "saison1/vf"),
            ("one-piece", "saison2/vf"),
            ("kagura-bachi", "scan"),
            ("kagura-bachi", "scan/vf"),
        ] {
            assert!(
                valid_episode_request(slug, season),
                "Requête rejetée : {slug} / {season}"
            );
        }
    }

    #[test]
    fn rejects_invalid_episode_requests() {
        for (slug, season) in [
            ("", "saison1/vostfr"),
            ("../admin", "saison1/vostfr"),
            ("frieren", "../admin"),
            ("frieren", "saison1/../admin"),
            ("frieren", "saison1/%2e%2e"),
            ("frieren", "saison1//vostfr"),
            ("frieren?x=1", "saison1/vostfr"),
        ] {
            assert!(
                !valid_episode_request(slug, season),
                "Requête acceptée : {slug} / {season}"
            );
        }
    }
}
