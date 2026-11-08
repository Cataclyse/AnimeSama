use serde::{Deserialize, Serialize};
use std::fs;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_name: String,
    pub release_notes: String,
    pub release_url: String,
    pub download_url: Option<String>,
    pub asset_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplyResult {
    pub success: bool,
    pub message: String,
    pub needs_restart: bool,
    pub downloaded_file: Option<String>,
}

pub fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let clean = v.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    let parts: Vec<&str> = clean.split('.').collect();
    if parts.is_empty() {
        return None;
    }
    let major = parts.first()?.parse::<u64>().ok()?;
    let minor = parts.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
    Some((major, minor, patch))
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(curr), Some(lat)) => lat > curr,
        _ => false,
    }
}

pub fn select_asset<'a>(assets: &'a [serde_json::Value]) -> Option<(&'a str, &'a str)> {
    let is_windows = cfg!(target_os = "windows");
    let is_linux = cfg!(target_os = "linux");

    let mut candidate = None;

    for asset in assets {
        let name = match asset.get("name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };
        let download_url = match asset.get("browser_download_url").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => continue,
        };

        if is_windows {
            if name == "animesama.exe" {
                return Some((name, download_url));
            }
            if name.ends_with(".zip") && name.contains("windows") && candidate.is_none() {
                candidate = Some((name, download_url));
            }
        } else if is_linux {
            if name.ends_with(".deb") {
                return Some((name, download_url));
            }
            if name == "animesama-linux-gui" {
                return Some((name, download_url));
            }
            if name == "animesama-linux-server" && candidate.is_none() {
                candidate = Some((name, download_url));
            }
            if name.ends_with(".tar.gz") && candidate.is_none() {
                candidate = Some((name, download_url));
            }
        }
    }

    candidate
}

pub fn check_for_updates() -> Result<UpdateInfo, String> {
    let api_url = "https://api.github.com/repos/Cataclyse/AnimeSama/releases/latest";

    let resp = match ureq::get(api_url)
        .set("User-Agent", "AnimeSama-Desktop-Updater")
        .set("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(404, _)) => {
            return Ok(UpdateInfo {
                current_version: CURRENT_VERSION.to_string(),
                latest_version: CURRENT_VERSION.to_string(),
                update_available: false,
                release_name: "Aucune mise à jour publiée".to_string(),
                release_notes: "Vous utilisez la version actuelle.".to_string(),
                release_url: "https://github.com/Cataclyse/AnimeSama/releases".to_string(),
                download_url: None,
                asset_name: None,
            });
        }
        Err(e) => return Err(format!("Erreur réseau lors de la vérification: {}", e)),
    };

    let body: serde_json::Value = resp
        .into_json()
        .map_err(|e| format!("Erreur de lecture de la réponse: {}", e))?;

    let tag_name = body
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let release_name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&tag_name)
        .to_string();

    let release_notes = body
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let release_url = body
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/Cataclyse/AnimeSama/releases")
        .to_string();

    let assets = body
        .get("assets")
        .and_then(|v| v.as_array())
        .map(|a| a.as_slice())
        .unwrap_or(&[]);

    let selected = select_asset(assets);
    let (asset_name, download_url) = match selected {
        Some((name, url)) => (Some(name.to_string()), Some(url.to_string())),
        None => (None, None),
    };

    let update_available = is_newer_version(CURRENT_VERSION, &tag_name);

    Ok(UpdateInfo {
        current_version: CURRENT_VERSION.to_string(),
        latest_version: tag_name,
        update_available,
        release_name,
        release_notes,
        release_url,
        download_url,
        asset_name,
    })
}

pub fn download_file(url: &str) -> Result<Vec<u8>, String> {
    let resp = ureq::get(url)
        .set("User-Agent", "AnimeSama-Desktop-Updater")
        .call()
        .map_err(|e| format!("Erreur lors du téléchargement: {}", e))?;

    let mut bytes = Vec::new();
    resp.into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Erreur de lecture du flux: {}", e))?;

    Ok(bytes)
}

pub fn apply_update() -> Result<UpdateApplyResult, String> {
    let update_info = check_for_updates()?;
    if !update_info.update_available {
        return Ok(UpdateApplyResult {
            success: true,
            message: "L'application est déjà à jour.".to_string(),
            needs_restart: false,
            downloaded_file: None,
        });
    }

    let download_url = match &update_info.download_url {
        Some(u) => u,
        None => {
            let _ = opener::open(&update_info.release_url);
            return Ok(UpdateApplyResult {
                success: true,
                message: "Page de téléchargement ouverte dans le navigateur.".to_string(),
                needs_restart: false,
                downloaded_file: None,
            });
        }
    };

    let asset_name = update_info.asset_name.clone().unwrap_or_else(|| "update.bin".to_string());
    let bytes = download_file(download_url)?;

    if asset_name.ends_with(".deb") {
        let temp_dir = std::env::temp_dir();
        let target = temp_dir.join(&asset_name);
        fs::write(&target, &bytes).map_err(|e| format!("Échec d'écriture du paquet: {}", e))?;
        let _ = opener::open(&target);
        return Ok(UpdateApplyResult {
            success: true,
            message: format!("Paquet {} téléchargé et ouvert dans l'installateur.", asset_name),
            needs_restart: false,
            downloaded_file: Some(target.to_string_lossy().to_string()),
        });
    }

    let current_exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => {
            let temp_dir = std::env::temp_dir();
            let target = temp_dir.join(&asset_name);
            fs::write(&target, &bytes).map_err(|e| format!("Échec d'écriture: {}", e))?;
            let _ = opener::open(&target);
            return Ok(UpdateApplyResult {
                success: true,
                message: format!("Fichier téléchargé dans {}. Veuillez l'exécuter.", target.display()),
                needs_restart: false,
                downloaded_file: Some(target.to_string_lossy().to_string()),
            });
        }
    };

    if cfg!(target_os = "windows") {
        let old_exe = current_exe.with_extension("exe.old");
        let _ = fs::remove_file(&old_exe);
        if let Err(_e) = fs::rename(&current_exe, &old_exe) {
            let temp_dir = std::env::temp_dir();
            let target = temp_dir.join(&asset_name);
            fs::write(&target, &bytes).map_err(|err| format!("Échec d'écriture: {}", err))?;
            let _ = opener::open(&target);
            return Ok(UpdateApplyResult {
                success: true,
                message: format!("Mise à jour téléchargée: {}. Veuillez relancer.", target.display()),
                needs_restart: true,
                downloaded_file: Some(target.to_string_lossy().to_string()),
            });
        }
        if let Err(e) = fs::write(&current_exe, &bytes) {
            let _ = fs::rename(&old_exe, &current_exe);
            return Err(format!("Échec du remplacement de l'exécutable: {}", e));
        }
        Ok(UpdateApplyResult {
            success: true,
            message: format!("Mise à jour vers {} réussie ! Veuillez redémarrer l'application.", update_info.latest_version),
            needs_restart: true,
            downloaded_file: Some(current_exe.to_string_lossy().to_string()),
        })
    } else {
        let temp_exe = current_exe.with_extension("new");
        fs::write(&temp_exe, &bytes).map_err(|e| format!("Échec d'écriture: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&temp_exe, fs::Permissions::from_mode(0o755));
        }

        fs::rename(&temp_exe, &current_exe).map_err(|e| format!("Échec du remplacement: {}", e))?;

        Ok(UpdateApplyResult {
            success: true,
            message: format!("Mise à jour vers {} appliquée avec succès ! Veuillez redémarrer.", update_info.latest_version),
            needs_restart: true,
            downloaded_file: Some(current_exe.to_string_lossy().to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing_and_comparison() {
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("v0.2.1"), Some((0, 2, 1)));
        assert_eq!(parse_version("V1.0.0"), Some((1, 0, 0)));
        assert!(is_newer_version("0.1.0", "0.2.0"));
        assert!(is_newer_version("0.1.0", "v0.1.1"));
        assert!(is_newer_version("0.1.0", "1.0.0"));
        assert!(!is_newer_version("0.1.0", "0.1.0"));
        assert!(!is_newer_version("0.2.0", "0.1.9"));
    }

    #[test]
    fn test_asset_selection_windows_and_linux() {
        let assets = serde_json::json!([
            { "name": "animesama.exe", "browser_download_url": "https://example.com/animesama.exe" },
            { "name": "animesama_0.1.0_amd64.deb", "browser_download_url": "https://example.com/deb" },
            { "name": "animesama-linux-gui", "browser_download_url": "https://example.com/gui" }
        ]);
        let selected = select_asset(assets.as_array().unwrap());
        assert!(selected.is_some());
    }
}
