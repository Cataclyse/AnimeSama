use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallKind {
    WindowsInstaller,
    WindowsStandalone,
    LinuxDeb,
    LinuxGuiStandalone,
    LinuxServerStandalone,
}

impl InstallKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallKind::WindowsInstaller => "windows_installer",
            InstallKind::WindowsStandalone => "windows_standalone",
            InstallKind::LinuxDeb => "linux_deb",
            InstallKind::LinuxGuiStandalone => "linux_gui_standalone",
            InstallKind::LinuxServerStandalone => "linux_server_standalone",
        }
    }
}

pub fn detect_install_kind() -> InstallKind {
    let is_windows = cfg!(target_os = "windows");
    let is_linux = cfg!(target_os = "linux");

    let current_exe = std::env::current_exe().ok();

    if is_windows {
        if let Some(exe) = &current_exe {
            if let Some(parent) = exe.parent() {
                if parent.join("unins000.exe").exists() || parent.join("unins001.exe").exists() {
                    return InstallKind::WindowsInstaller;
                }
            }
            let p_lower = exe.to_string_lossy().to_lowercase();
            if p_lower.contains("program files") || p_lower.contains("programs\\anime-sama") || p_lower.contains("programs/anime-sama") {
                return InstallKind::WindowsInstaller;
            }
        }
        InstallKind::WindowsStandalone
    } else if is_linux {
        if !cfg!(feature = "gui") {
            return InstallKind::LinuxServerStandalone;
        }
        if let Some(exe) = &current_exe {
            let p = exe.to_string_lossy();
            if p.starts_with("/usr/bin") || p.starts_with("/usr/local/bin") || p.starts_with("/opt") {
                return InstallKind::LinuxDeb;
            }
            let file_name = exe.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.contains("server") {
                return InstallKind::LinuxServerStandalone;
            }
        }
        InstallKind::LinuxGuiStandalone
    } else {
        InstallKind::WindowsStandalone
    }
}

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
    pub install_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplyResult {
    pub success: bool,
    pub message: String,
    pub needs_restart: bool,
    pub downloaded_file: Option<String>,
}

pub fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let clean = v.trim().trim_start_matches(|c| c == 'v' || c == 'V' || c == ' ');
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

pub fn select_asset_for_kind<'a>(assets: &'a [serde_json::Value], kind: InstallKind) -> Option<(&'a str, &'a str)> {
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

        match kind {
            InstallKind::WindowsInstaller => {
                if name.ends_with(".exe") && name.to_lowercase().contains("setup") {
                    return Some((name, download_url));
                }
                if (name == "animesama.exe" || name == "animesama-windows-x86_64.exe") && candidate.is_none() {
                    candidate = Some((name, download_url));
                }
            }
            InstallKind::WindowsStandalone => {
                if name == "animesama-windows-x86_64.exe" || name == "animesama.exe" {
                    return Some((name, download_url));
                }
                if name.ends_with(".zip") && name.contains("windows") && candidate.is_none() {
                    candidate = Some((name, download_url));
                }
                if name.ends_with(".exe") && candidate.is_none() {
                    candidate = Some((name, download_url));
                }
            }
            InstallKind::LinuxDeb => {
                if name.ends_with(".deb") {
                    return Some((name, download_url));
                }
                if (name == "animesama-linux-gui-x86_64" || name == "animesama-linux-gui") && candidate.is_none() {
                    candidate = Some((name, download_url));
                }
            }
            InstallKind::LinuxGuiStandalone => {
                if name == "animesama-linux-gui-x86_64" || name == "animesama-linux-gui" {
                    return Some((name, download_url));
                }
                if name.ends_with(".tar.gz") && candidate.is_none() {
                    candidate = Some((name, download_url));
                }
            }
            InstallKind::LinuxServerStandalone => {
                if name == "animesama-linux-server-x86_64" || name == "animesama-linux-server" {
                    return Some((name, download_url));
                }
                if name.ends_with(".tar.gz") && candidate.is_none() {
                    candidate = Some((name, download_url));
                }
            }
        }
    }

    candidate
}

pub fn select_asset<'a>(assets: &'a [serde_json::Value]) -> Option<(&'a str, &'a str)> {
    select_asset_for_kind(assets, detect_install_kind())
}

fn update_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(6))
        .timeout_read(std::time::Duration::from_secs(12))
        .build()
}

pub fn check_for_updates() -> Result<UpdateInfo, String> {
    let agent = update_agent();
    let api_url = "https://api.github.com/repos/Cataclyse/AnimeSama/releases/latest";

    let mut body = match agent
        .get(api_url)
        .set("User-Agent", "AnimeSama-Desktop-Updater")
        .set("Accept", "application/vnd.github.v3+json")
        .call()
    {
        Ok(r) => r.into_json::<serde_json::Value>().ok(),
        Err(ureq::Error::Status(404, _)) => None,
        Err(e) => return Err(format!("Erreur réseau lors de la vérification: {}", e)),
    };

    if body.is_none() {
        let fallback_url = "https://api.github.com/repos/Cataclyse/AnimeSama/releases?per_page=1";
        if let Ok(r) = agent
            .get(fallback_url)
            .set("User-Agent", "AnimeSama-Desktop-Updater")
            .set("Accept", "application/vnd.github.v3+json")
            .call()
        {
            if let Ok(serde_json::Value::Array(list)) = r.into_json() {
                if let Some(first) = list.into_iter().next() {
                    body = Some(first);
                }
            }
        }
    }

    let kind = detect_install_kind();

    let body = match body {
        Some(b) => b,
        None => {
            return Ok(UpdateInfo {
                current_version: CURRENT_VERSION.to_string(),
                latest_version: CURRENT_VERSION.to_string(),
                update_available: false,
                release_name: "Aucune mise à jour publiée".to_string(),
                release_notes: "Vous utilisez la version actuelle.".to_string(),
                release_url: "https://github.com/Cataclyse/AnimeSama/releases".to_string(),
                download_url: None,
                asset_name: None,
                install_kind: kind.as_str().to_string(),
            });
        }
    };

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

    let selected = select_asset_for_kind(assets, kind);
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
        install_kind: kind.as_str().to_string(),
    })
}

pub fn download_file(url: &str) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(120))
        .build();

    let resp = agent
        .get(url)
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

    if asset_name.ends_with(".deb") || (cfg!(target_os = "windows") && asset_name.to_lowercase().contains("setup")) {
        let temp_dir = std::env::temp_dir();
        let target = temp_dir.join(&asset_name);
        fs::write(&target, &bytes).map_err(|e| format!("Échec d'écriture du fichier: {}", e))?;
        let _ = opener::open(&target);
        return Ok(UpdateApplyResult {
            success: true,
            message: format!("Fichier {} téléchargé et lancé.", asset_name),
            needs_restart: true,
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
            { "name": "AnimeSama-Setup-x86_64.exe", "browser_download_url": "https://example.com/setup.exe" },
            { "name": "animesama-windows-x86_64.exe", "browser_download_url": "https://example.com/standalone.exe" },
            { "name": "animesama_0.1.0_amd64.deb", "browser_download_url": "https://example.com/deb" },
            { "name": "animesama-linux-gui-x86_64", "browser_download_url": "https://example.com/gui" },
            { "name": "animesama-linux-server-x86_64", "browser_download_url": "https://example.com/server" }
        ]);
        let arr = assets.as_array().unwrap();

        let win_inst = select_asset_for_kind(arr, InstallKind::WindowsInstaller).unwrap();
        assert_eq!(win_inst.0, "AnimeSama-Setup-x86_64.exe");

        let win_stand = select_asset_for_kind(arr, InstallKind::WindowsStandalone).unwrap();
        assert_eq!(win_stand.0, "animesama-windows-x86_64.exe");

        let lin_deb = select_asset_for_kind(arr, InstallKind::LinuxDeb).unwrap();
        assert_eq!(lin_deb.0, "animesama_0.1.0_amd64.deb");

        let lin_gui = select_asset_for_kind(arr, InstallKind::LinuxGuiStandalone).unwrap();
        assert_eq!(lin_gui.0, "animesama-linux-gui-x86_64");

        let lin_srv = select_asset_for_kind(arr, InstallKind::LinuxServerStandalone).unwrap();
        assert_eq!(lin_srv.0, "animesama-linux-server-x86_64");

        let selected = select_asset(arr);
        assert!(selected.is_some());
    }
}
