use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub slug: String,
    pub title: String,
    pub season: String,
    pub episode: usize,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackPosition {
    pub slug: String,
    pub season: String,
    pub episode: usize,
    pub current_time: f64,
    pub duration: f64,
    pub timestamp: u64,
    #[serde(default)]
    pub revision: u64,
}

fn resolve_position_update(
    current: Option<&PlaybackPosition>,
    mut incoming: PlaybackPosition,
) -> Result<PlaybackPosition, ()> {
    match current {
        None if incoming.revision == 0 => {
            incoming.revision = 1;
            Ok(incoming)
        }
        Some(saved) if saved.revision == incoming.revision => {
            incoming.revision =
                saved.revision.checked_add(1).ok_or(())?;
            Ok(incoming)
        }
        _ => Err(()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    pub watchlist: Vec<String>,
    pub favorites: Vec<String>,
    pub history: Vec<HistoryEntry>,
    #[serde(default)]
    pub playback_positions: Vec<PlaybackPosition>,
    #[serde(default)]
    pub dismissed_continue: Vec<String>,
    #[serde(default)]
    pub auto_sync: bool,
    #[serde(default = "default_true")]
    pub ad_shield: bool,
    #[serde(default = "default_true")]
    pub auto_update: bool,
    #[serde(default = "default_true")]
    pub discord_rpc: bool,
}

fn default_true() -> bool {
    true
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            watchlist: Vec::new(),
            favorites: Vec::new(),
            history: Vec::new(),
            playback_positions: Vec::new(),
            dismissed_continue: Vec::new(),
            auto_sync: false,
            ad_shield: true,
            auto_update: true,
            discord_rpc: true,
        }
    }
}

#[derive(Clone)]
pub struct UserManager {
    file_path: String,
    data: Arc<Mutex<UserData>>,
}

impl UserManager {
    pub fn new(path: &str) -> Self {
        let loaded = if Path::new(path).exists() {
            fs::read_to_string(path)
                .ok()
                .and_then(|c| serde_json::from_str::<UserData>(&c).ok())
                .unwrap_or_default()
        } else {
            UserData::default()
        };

        Self {
            file_path: path.to_string(),
            data: Arc::new(Mutex::new(loaded)),
        }
    }

    pub fn get_data(&self) -> UserData {
        self.data.lock().unwrap().clone()
    }

    pub fn toggle_watchlist(&self, slug: &str) -> (bool, UserData) {
        let mut d = self.data.lock().unwrap();
        let in_list = if let Some(pos) = d.watchlist.iter().position(|s| s == slug) {
            d.watchlist.remove(pos);
            false
        } else {
            d.watchlist.push(slug.to_string());
            true
        };
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        (in_list, snapshot)
    }

    pub fn set_watchlist(&self, slugs: Vec<String>) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.watchlist = slugs;
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn set_favorites(&self, slugs: Vec<String>) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.favorites = slugs;
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn set_auto_sync(&self, enabled: bool) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.auto_sync = enabled;
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn set_ad_shield(&self, enabled: bool) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.ad_shield = enabled;
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn set_auto_update(&self, enabled: bool) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.auto_update = enabled;
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn set_discord_rpc(&self, enabled: bool) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.discord_rpc = enabled;
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn get_playback_position(
        &self,
        slug: &str,
        season: &str,
        episode: usize,
    ) -> Option<PlaybackPosition> {
        let data = self.data.lock().unwrap();
        data.playback_positions
            .iter()
            .find(|p| {
                p.slug == slug
                    && p.season == season
                    && p.episode == episode
            })
            .cloned()
    }

    pub fn put_playback_position(
        &self,
        incoming: PlaybackPosition,
    ) -> Result<PlaybackPosition, ()> {
        let mut data = self.data.lock().unwrap();

        let index = data.playback_positions.iter().position(|p| {
            p.slug == incoming.slug
                && p.season == incoming.season
                && p.episode == incoming.episode
        });

        let next = resolve_position_update(
            index.map(|i| &data.playback_positions[i]),
            incoming,
        )?;

        match index {
            Some(i) => data.playback_positions[i] = next.clone(),
            None => data.playback_positions.push(next.clone()),
        }

        if data.playback_positions.len() > 1000 {
            data.playback_positions.sort_by_key(|p| {
                std::cmp::Reverse(p.timestamp)
            });
            data.playback_positions.truncate(1000);
        }

        let snapshot = data.clone();
        drop(data);
        self.save(&snapshot);

        Ok(next)
    }

    pub fn add_history(&self, entry: HistoryEntry) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.dismissed_continue.retain(|s| s != &entry.slug);
        d.history.retain(|h| h.slug != entry.slug);
        d.history.insert(0, entry);
        if d.history.len() > 50 {
            d.history.truncate(50);
        }
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    pub fn remove_history(&self, slug: &str) -> UserData {
        let mut d = self.data.lock().unwrap();
        d.history.retain(|h| h.slug != slug);
        if !d.dismissed_continue.iter().any(|s| s == slug) {
            d.dismissed_continue.push(slug.to_string());
        }
        let snapshot = d.clone();
        drop(d);
        self.save(&snapshot);
        snapshot
    }

    fn save(&self, data: &UserData) {
        if let Ok(json) = serde_json::to_string_pretty(data) {
            let _ = fs::write(&self.file_path, &json);
            if self.file_path != "userdata.json" && Path::new("userdata.json").exists() {
                let _ = fs::write("userdata.json", &json);
            }
        }
    }
}

#[cfg(test)]
mod playback_position_tests {
    use super::*;

    #[test]
    fn legacy_data_without_positions_is_supported() {
        let mut legacy =
            serde_json::to_value(UserData::default()).unwrap();
        legacy.as_object_mut()
            .unwrap()
            .remove("playback_positions");

        let parsed: UserData =
            serde_json::from_value(legacy).unwrap();

        assert!(parsed.playback_positions.is_empty());
    }

    fn sample(revision: u64, time: f64) -> PlaybackPosition {
        PlaybackPosition {
            slug: "frieren".into(),
            season: "saison1/vostfr".into(),
            episode: 2,
            current_time: time,
            duration: 1560.0,
            timestamp: 123456,
            revision,
        }
    }

    #[test]
    fn accepts_first_position_with_revision_zero() {
        let result =
            resolve_position_update(None, sample(0, 300.0)).unwrap();

        assert_eq!(result.revision, 1);
        assert_eq!(result.current_time, 300.0);
    }

    #[test]
    fn rejects_stale_browser_write() {
        let saved = sample(5, 600.0);
        let stale = sample(4, 330.0);

        assert!(resolve_position_update(Some(&saved), stale).is_err());
        assert_eq!(saved.current_time, 600.0);
    }

    #[test]
    fn accepts_intentional_rewind_with_current_revision() {
        let saved = sample(5, 600.0);
        let rewind = sample(5, 330.0);

        let result =
            resolve_position_update(Some(&saved), rewind).unwrap();

        assert_eq!(result.revision, 6);
        assert_eq!(result.current_time, 330.0);
    }

    #[test]
    fn legacy_position_without_revision_is_supported() {
        let value = serde_json::json!({
            "slug": "frieren",
            "season": "saison1/vostfr",
            "episode": 2,
            "currentTime": 300.0,
            "duration": 1560.0,
            "timestamp": 123456
        });

        let parsed: PlaybackPosition =
            serde_json::from_value(value).unwrap();

        assert_eq!(parsed.revision, 0);
    }
}
