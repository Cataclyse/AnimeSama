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
                        let state_str = if state_parts.is_empty() {
                            if payload.status == "playing" {
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

                        let mut assets = activity::Assets::new()
                            .large_image("animesama_logo")
                            .large_text("Anime-Sama");

                        if payload.status == "playing" {
                            assets = assets.small_image("play").small_text("En lecture");
                        } else if payload.status == "paused" {
                            assets = assets.small_image("pause").small_text("En pause");
                        }
                        act = act.assets(assets);

                        if payload.status == "playing" {
                            if let (Some(dur), Some(cur)) = (payload.duration, payload.current_time) {
                                if dur > cur {
                                    let now_secs = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs();
                                    let start_ts = now_secs.saturating_sub(cur);
                                    act = act.timestamps(
                                        activity::Timestamps::new().start(start_ts as i64),
                                    );
                                }
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
