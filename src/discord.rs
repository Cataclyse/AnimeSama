use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

const CLIENT_ID: &str = "1569018746784456867";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresencePayload {
    pub title: String,
    pub episode: Option<u32>,
    pub season: Option<String>,
    pub status: String,
    pub duration: Option<u64>,
    pub current_time: Option<u64>,
    #[serde(default)]
    pub poster: Option<String>,
}

enum RpcCommand {
    Update(PresencePayload),
    Clear,
}

static RPC_SENDER: Mutex<Option<Sender<RpcCommand>>> = Mutex::new(None);

pub fn init_discord_rpc() {
    let (tx, rx): (Sender<RpcCommand>, Receiver<RpcCommand>) = channel();
    {
        let mut sender_guard = RPC_SENDER.lock().unwrap();
        *sender_guard = Some(tx);
    }

    thread::spawn(move || {
        let mut client: Option<DiscordIpcClient> = None;
        let mut last_connect_attempt = Instant::now() - Duration::from_secs(60);
        let mut cached_start_ts: Option<u64> = None;
        let mut home_start_ts: Option<u64> = None;

        while let Ok(cmd) = rx.recv() {
            match cmd {
                RpcCommand::Update(payload) => {
                    ensure_connected(&mut client, &mut last_connect_attempt);
                    if let Some(c) = client.as_mut() {
                        let mut act = activity::Activity::new();
                        let details_text = if payload.title.is_empty() {
                            "Page d'accueil".to_string()
                        } else {
                            payload.title.clone()
                        };
                        act = act.details(&details_text);

                        let mut state_parts = Vec::new();
                        let is_scan = payload
                            .season
                            .as_deref()
                            .map(|s| s.to_lowercase().contains("scan"))
                            .unwrap_or(false)
                            || payload.status == "reading";

                        if is_scan {
                            if let Some(ep) = payload.episode {
                                state_parts.push(format!("Scan • Chapitre {}", ep));
                            } else {
                                state_parts.push("Lecture de scans".to_string());
                            }
                        } else {
                            if let Some(s) = &payload.season {
                                let s_clean = s.split('/').next().unwrap_or(s);
                                let s_num = s_clean.replace(|c: char| !c.is_numeric(), "");
                                if !s_num.is_empty() {
                                    state_parts.push(format!("Saison {}", s_num));
                                }
                            }
                            if let Some(ep) = payload.episode {
                                state_parts.push(format!("Épisode {}", ep));
                            }
                        }

                        let state_str = if state_parts.is_empty() {
                            if payload.status == "playing" || payload.status == "reading" {
                                "En cours de lecture".to_string()
                            } else {
                                "Parcourt le catalogue".to_string()
                            }
                        } else if payload.status == "paused" {
                            format!("{} (En pause)", state_parts.join(" • "))
                        } else {
                            state_parts.join(" • ")
                        };
                        act = act.state(&state_str);
                        act = act.activity_type(activity::ActivityType::Watching);

                        let large_image = payload
                            .poster
                            .as_deref()
                            .filter(|p| !p.is_empty())
                            .unwrap_or("animesama_logo");
                        let mut assets = activity::Assets::new()
                            .large_image(large_image)
                            .large_text(details_text.clone());

                        if payload.status == "playing" {
                            assets = assets.small_image("play").small_text("En lecture");
                        } else if payload.status == "paused" {
                            assets = assets.small_image("pause").small_text("En pause");
                        } else if is_scan || payload.status == "reading" {
                            assets = assets.small_image("play").small_text("Lecture de scans");
                        }
                        act = act.assets(assets);

                        if payload.status == "playing" {
                            home_start_ts = None;
                            if let (Some(dur), Some(cur)) = (payload.duration, payload.current_time) {
                                if dur > cur {
                                    let now_secs = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    let computed_start = now_secs.saturating_sub(cur);
                                    let start_ts = match cached_start_ts {
                                        Some(prev)
                                            if (prev as i64 - computed_start as i64).abs() < 5 =>
                                        {
                                            prev
                                        }
                                        _ => computed_start,
                                    };
                                    cached_start_ts = Some(start_ts);
                                    let end_ts = start_ts + dur;
                                    act = act.timestamps(
                                        activity::Timestamps::new()
                                            .start(start_ts as i64)
                                            .end(end_ts as i64),
                                    );
                                }
                            }
                        } else if is_scan || payload.status == "reading" {
                            home_start_ts = None;
                            let now_secs = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let start_ts = match cached_start_ts {
                                Some(prev) => prev,
                                None => {
                                    cached_start_ts = Some(now_secs);
                                    now_secs
                                }
                            };
                            act = act.timestamps(
                                activity::Timestamps::new().start(start_ts as i64),
                            );
                        } else {
                            cached_start_ts = None;
                            if payload.status == "idle" || payload.title == "Page d'accueil" {
                                let now_secs = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let start_ts = match home_start_ts {
                                    Some(prev) => prev,
                                    None => {
                                        home_start_ts = Some(now_secs);
                                        now_secs
                                    }
                                };
                                act = act.timestamps(
                                    activity::Timestamps::new().start(start_ts as i64),
                                );
                            } else {
                                home_start_ts = None;
                            }
                        }

                        let btn = activity::Button::new("Regarder sur Anime-Sama", "https://anime-sama.to");
                        act = act.buttons(vec![btn]);

                        if c.set_activity(act).is_err() {
                            let _ = c.close();
                            client = None;
                        }
                    }
                }
                RpcCommand::Clear => {
                    cached_start_ts = None;
                    home_start_ts = None;
                    if let Some(c) = client.as_mut() {
                        let _ = c.clear_activity();
                    }
                }
            }
        }
    });
}

fn ensure_connected(client: &mut Option<DiscordIpcClient>, last_attempt: &mut Instant) {
    if client.is_none() && last_attempt.elapsed() > Duration::from_secs(5) {
        *last_attempt = Instant::now();
        let mut new_client = DiscordIpcClient::new(CLIENT_ID);
        if new_client.connect().is_ok() {
            *client = Some(new_client);
        }
    }
}

pub fn update_presence(payload: PresencePayload) {
    let sender_guard = RPC_SENDER.lock().unwrap();
    if let Some(tx) = sender_guard.as_ref() {
        let _ = tx.send(RpcCommand::Update(payload));
    }
}

pub fn clear_presence() {
    let sender_guard = RPC_SENDER.lock().unwrap();
    if let Some(tx) = sender_guard.as_ref() {
        let _ = tx.send(RpcCommand::Clear);
    }
}
