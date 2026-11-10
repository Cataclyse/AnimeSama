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


// Transport réservé aux appels d'authentification.
// Ne jamais transmettre un cookie vers une destination de redirection.
fn auth_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .https_only(true)
        .try_proxy_from_env(false)
        .resolver(crate::proxy::fetcher::resolve_public_addresses)
        .timeout(std::time::Duration::from_secs(10))
        .build()
}


const MAX_AUTH_JSON_BYTES: usize = 4 * 1024 * 1024;

fn read_auth_json_limited<R: std::io::Read>(
    reader: R,
) -> Result<serde_json::Value, String> {
    let bytes = crate::proxy::fetcher::read_limited(
        reader,
        MAX_AUTH_JSON_BYTES,
    )
    .map_err(|e| format!("Réponse distante trop volumineuse ou illisible: {}", e))?;

    serde_json::from_slice(&bytes)
        .map_err(|_| "Réponse JSON distante invalide".to_string())
}


// Conserve la compatibilité des réponses de connexion sans JSON,
// tout en limitant strictement leur taille.
fn read_login_json_limited<R: std::io::Read>(
    reader: R,
) -> Result<serde_json::Value, String> {
    let bytes = crate::proxy::fetcher::read_limited(
        reader,
        MAX_AUTH_JSON_BYTES,
    )
    .map_err(|e| format!("Réponse de connexion illisible: {}", e))?;

    Ok(serde_json::from_slice(&bytes).unwrap_or_default())
}

fn cloud_endpoint_allowed(endpoint: &str) -> bool {
    matches!(
        endpoint,
        "sync-watchlist.php"
            | "sync-favorites.php"
            | "sync-history.php"
            | "sync-progress.php"
    )
}


#[cfg(unix)]
fn write_session_atomically(
    path: &std::path::Path,
    content: &[u8],
) -> std::io::Result<()> {
    use std::io::{self, Write};
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Dossier de session absent",
        )
    })?;

    fs::set_permissions(
        parent,
        fs::Permissions::from_mode(0o700),
    )?;

    let name = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Nom de session invalide",
            )
        })?;

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    for attempt in 0..16 {
        let temp = parent.join(format!(
            ".{}.tmp-{}-{}-{}",
            name,
            std::process::id(),
            stamp,
            attempt,
        ));

        let mut file = match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)
        {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                continue;
            }
            Err(e) => return Err(e),
        };

        let result = (|| -> io::Result<()> {
            file.write_all(content)?;
            file.sync_all()?;
            drop(file);

            fs::rename(&temp, path)?;
            fs::File::open(parent)?.sync_all()?;

            Ok(())
        })();

        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }

        return result;
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "Impossible de créer un fichier temporaire unique",
    ))
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
        let Ok(json) = serde_json::to_string_pretty(data) else {
            return;
        };

        #[cfg(unix)]
        if let Err(e) = write_session_atomically(
            &self.file_path,
            json.as_bytes(),
        ) {
            eprintln!("[auth] Echec de sauvegarde de session: {}", e);
        }

        #[cfg(not(unix))]
        if let Err(e) = fs::write(&self.file_path, json) {
            eprintln!("[auth] Echec de sauvegarde de session: {}", e);
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

        let resp = auth_agent().post("https://anime-sama.to/api/auth/login.php")
            .set("User-Agent", USER_AGENT)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&body);

        self.handle_auth_response(resp, pseudo)
    }

    pub fn register(&self, pseudo: &str, password: &str) -> Result<String, String> {
        let body = serde_json::json!({ "pseudo": pseudo, "password": password }).to_string();

        let resp = auth_agent().post("https://anime-sama.to/api/auth/register.php")
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
                if r.status() != 200 {
                    return Err("Statut d'authentification inattendu".to_string());
                }
                let session_cookie = extract_session_cookie(&r);
                let body = read_login_json_limited(r.into_reader())?;

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
                let body = read_auth_json_limited(r.into_reader())
                    .unwrap_or_default();
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
            let _ = auth_agent().get("https://anime-sama.to/api/auth/logout.php")
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

        let resp = auth_agent().get("https://anime-sama.to/api/get-data.php")
            .set("User-Agent", USER_AGENT)
            .set("Cookie", &cookie)
            .timeout(std::time::Duration::from_secs(10))
            .call();

        match resp {
            Ok(r) => {
                let body = match read_auth_json_limited(r.into_reader()) {
                    Ok(value) => value,
                    Err(_) => return true,
                };

                let Some(still_valid) =
                    body.get("logged_in").and_then(|v| v.as_bool())
                else {
                    return true;
                };
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

        let resp = auth_agent().get("https://anime-sama.to/api/get-data.php")
            .set("User-Agent", USER_AGENT)
            .set("Cookie", &cookie)
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .map_err(|e| format!("Erreur réseau: {}", e))?;

        let body = read_auth_json_limited(resp.into_reader())?;

        if !body.get("logged_in").and_then(|v| v.as_bool()).unwrap_or(false) {
            self.clear_session();
            return Err("Session expirée, merci de vous reconnecter".to_string());
        }

        Ok(body)
    }

    pub fn push_cloud_data(&self, endpoint: &str, payload: serde_json::Value) -> Result<(), String> {
        if !cloud_endpoint_allowed(endpoint) {
            return Err("Endpoint cloud non autorisé".to_string());
        }

        let cookie = self.cookie().ok_or("Non connecté")?;
        let url = format!("https://anime-sama.to/api/{}", endpoint);

        let resp = auth_agent().post(&url)
            .set("User-Agent", USER_AGENT)
            .set("Cookie", &cookie)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(10))
            .send_string(&payload.to_string());

        match resp {
            Ok(r) if (200..300).contains(&r.status()) => Ok(()),
            Ok(_) => Err("Statut de synchronisation inattendu".to_string()),
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


#[cfg(test)]
mod auth_response_hardening_tests {
    use super::*;

    #[test]
    fn login_accepts_empty_or_invalid_json_body() {
        let empty = read_login_json_limited(&b""[..]).unwrap();
        assert!(empty.is_null());

        let invalid = read_login_json_limited(&b"not-json"[..]).unwrap();
        assert!(invalid.is_null());
    }

    #[test]
    fn login_rejects_oversized_response() {
        let data = vec![b' '; MAX_AUTH_JSON_BYTES + 1];
        assert!(read_login_json_limited(&data[..]).is_err());
    }

    #[test]
    fn accepts_valid_bounded_json() {
        let data = br#"{"logged_in":true}"#;
        let parsed = read_auth_json_limited(&data[..]).unwrap();
        assert_eq!(parsed["logged_in"], true);
    }

    #[test]
    fn rejects_oversized_json() {
        let data = vec![b' '; MAX_AUTH_JSON_BYTES + 1];
        assert!(read_auth_json_limited(&data[..]).is_err());
    }

    #[test]
    fn restricts_cloud_endpoints() {
        for name in [
            "sync-watchlist.php",
            "sync-favorites.php",
            "sync-history.php",
            "sync-progress.php",
        ] {
            assert!(cloud_endpoint_allowed(name));
        }

        for name in [
            "../admin.php",
            "auth/login.php",
            "sync-history.php?x=1",
            "https://example.com/",
            "",
        ] {
            assert!(!cloud_endpoint_allowed(name));
        }
    }
}


#[cfg(all(test, unix))]
mod atomic_session_tests {
    use super::write_session_atomically;
    use std::fs;
    use std::os::unix::fs::{symlink, PermissionsExt};

    fn fixture() -> std::path::PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let dir = std::env::temp_dir().join(format!(
            "cataclyse-session-test-{}-{}",
            std::process::id(),
            stamp,
        ));

        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn atomic_write_replaces_file_with_private_permissions() {
        let dir = fixture();
        let path = dir.join("session.json");

        fs::write(&path, b"ancien").unwrap();

        write_session_atomically(&path, b"nouveau").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"nouveau");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600,
        );

        assert_eq!(
            fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700,
        );

        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn atomic_write_does_not_follow_existing_symlink() {
        let dir = fixture();
        let target = dir.join("target.txt");
        let session = dir.join("session.json");

        fs::write(&target, b"intact").unwrap();
        symlink(&target, &session).unwrap();

        write_session_atomically(&session, b"session").unwrap();

        assert_eq!(fs::read(&target).unwrap(), b"intact");
        assert_eq!(fs::read(&session).unwrap(), b"session");
        assert!(!fs::symlink_metadata(&session)
            .unwrap()
            .file_type()
            .is_symlink());

        fs::remove_dir_all(&dir).unwrap();
    }
}
