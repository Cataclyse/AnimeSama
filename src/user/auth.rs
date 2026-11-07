use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::{Arc, Mutex};

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SessionData {
    #[serde(skip_serializing_if = "Option::is_none")]
    cookie: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pseudo: Option<String>,
}

#[derive(Clone)]
pub struct AuthClient {
    file_path: std::path::PathBuf,
    state: Arc<Mutex<SessionData>>,
}

impl AuthClient {
    pub fn new() -> Self {
        let session_file = super::paths::session_path();
        let loaded = if session_file.exists() {
            fs::read_to_string(&session_file)
                .ok()
                .and_then(|c| serde_json::from_str::<SessionData>(&c).ok())
                .unwrap_or_default()
        } else {
            SessionData::default()
        };

        Self {
            file_path: session_file,
            state: Arc::new(Mutex::new(loaded)),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        self.state.lock().unwrap().cookie.is_some()
    }

    pub fn pseudo(&self) -> Option<String> {
        self.state.lock().unwrap().pseudo.clone()
    }

    fn save(&self, data: &SessionData) {
        if let Ok(json) = serde_json::to_string_pretty(data) {
            let _ = fs::write(&self.file_path, json);
        }
    }

    fn cookie(&self) -> Option<String> {
        self.state.lock().unwrap().cookie.clone()
    }

    fn set_session(&self, cookie: String, pseudo: String) {
        let data = SessionData {
            cookie: Some(cookie),
            pseudo: Some(pseudo),
        };
        *self.state.lock().unwrap() = data.clone();
        self.save(&data);
    }

    pub fn clear_session(&self) {
        let data = SessionData::default();
        *self.state.lock().unwrap() = data.clone();
        self.save(&data);
    }

    pub fn login(&self, pseudo: &str, password: &str) -> Result<String, String> {
        let body = serde_json::json!({ "pseudo": pseudo, "password": password }).to_string();

        let resp = ureq::post("https://anime-sama.to/api/auth/login.php")
            .set("User-Agent", USER_AGENT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&body);

        self.handle_auth_response(resp, pseudo)
    }

    pub fn register(&self, pseudo: &str, password: &str) -> Result<String, String> {
        let body = serde_json::json!({ "pseudo": pseudo, "password": password }).to_string();

        let resp = ureq::post("https://anime-sama.to/api/auth/register.php")
            .set("User-Agent", USER_AGENT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&body);

        self.handle_auth_response(resp, pseudo)
    }

    fn handle_auth_response(
        &self,
        resp: Result<ureq::Response, ureq::Error>,
        pseudo: &str,
    ) -> Result<String, String> {
        match resp {
            Ok(r) => {
                let session_cookie = extract_session_cookie(&r);
                let body: serde_json::Value = r.into_json().unwrap_or_default();

                match session_cookie {
                    Some(cookie) => {
                        self.set_session(cookie, pseudo.to_string());
                        Ok(pseudo.to_string())
                    }
                    None => Err(body
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Réponse inattendue du serveur")
                        .to_string()),
                }
            }
            Err(ureq::Error::Status(429, _)) => {
                Err("Trop de tentatives, réessayez dans 1 heure".to_string())
            }
            Err(ureq::Error::Status(_, r)) => {
                let body: serde_json::Value = r.into_json().unwrap_or_default();
                Err(body
                    .get("error")
                    .and_then(|e| e.as_str())
                    .unwrap_or("Pseudo ou mot de passe incorrect")
                    .to_string())
            }
            Err(e) => Err(format!("Erreur réseau: {}", e)),
        }
    }

    pub fn logout(&self) {
        if let Some(cookie) = self.cookie() {
            let _ = ureq::get("https://anime-sama.to/api/auth/logout.php")
                .set("User-Agent", USER_AGENT)
                .set("Cookie", &cookie)
                .timeout(std::time::Duration::from_secs(10))
                .call();
        }
        self.clear_session();
    }

    pub fn verify_session(&self) -> bool {
        let cookie = match self.cookie() {
            Some(c) => c,
            None => return false,
        };

        let resp = ureq::get("https://anime-sama.to/api/get-data.php")
            .set("User-Agent", USER_AGENT)
            .set("Cookie", &cookie)
            .timeout(std::time::Duration::from_secs(10))
            .call();

        match resp {
            Ok(r) => {
                let body: serde_json::Value = r.into_json().unwrap_or_default();
                let still_valid = body.get("logged_in").and_then(|v| v.as_bool()).unwrap_or(false);
                if !still_valid {
                    self.clear_session();
                }
                still_valid
            }
            Err(_) => true,
        }
    }

    pub fn fetch_cloud_data(&self) -> Result<serde_json::Value, String> {
        let cookie = self.cookie().ok_or("Non connecté")?;

        let resp = ureq::get("https://anime-sama.to/api/get-data.php")
            .set("User-Agent", USER_AGENT)
            .set("Cookie", &cookie)
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .map_err(|e| format!("Erreur réseau: {}", e))?;

        let body: serde_json::Value = resp.into_json().map_err(|e| format!("Erreur lecture réponse: {}", e))?;

        if !body.get("logged_in").and_then(|v| v.as_bool()).unwrap_or(false) {
            self.clear_session();
            return Err("Session expirée, merci de vous reconnecter".to_string());
        }

        Ok(body)
    }

    pub fn push_cloud_data(&self, endpoint: &str, payload: serde_json::Value) -> Result<(), String> {
        let cookie = self.cookie().ok_or("Non connecté")?;
        let url = format!("https://anime-sama.to/api/{}", endpoint);

        let resp = ureq::post(&url)
            .set("User-Agent", USER_AGENT)
            .set("Cookie", &cookie)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&payload.to_string());

        match resp {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(401, _)) => {
                self.clear_session();
                Err("Session expirée, merci de vous reconnecter".to_string())
            }
            Err(e) => Err(format!("Erreur réseau: {}", e)),
        }
    }
}

fn extract_session_cookie(resp: &ureq::Response) -> Option<String> {
    let raw = resp.header("Set-Cookie")?;
    let first_part = raw.split(';').next()?.trim();
    if first_part.starts_with("PHPSESSID=") {
        Some(first_part.to_string())
    } else {
        None
    }
}
