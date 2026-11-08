use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::sync::{Mutex, OnceLock};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub active: bool,
    pub step: String,
    pub step_index: u8,
    pub total_steps: u8,
    pub percent: u8,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub message: String,
    pub version: String,
}

impl Default for UpdateProgress {
    fn default() -> Self {
        Self {
            active: false,
            step: "idle".to_string(),
            step_index: 0,
            total_steps: 4,
            percent: 0,
            bytes_downloaded: 0,
            total_bytes: 0,
            message: String::new(),
            version: String::new(),
        }
    }
}

fn progress_storage() -> &'static Mutex<UpdateProgress> {
    static STORAGE: OnceLock<Mutex<UpdateProgress>> = OnceLock::new();
    STORAGE.get_or_init(|| Mutex::new(UpdateProgress::default()))
}

pub fn get_update_progress() -> UpdateProgress {
    progress_storage().lock().unwrap().clone()
}

pub fn set_update_progress(step: &str, step_index: u8, percent: u8, downloaded: u64, total: u64, message: &str, version: &str) {
    if let Ok(mut p) = progress_storage().lock() {
        p.active = true;
        p.step = step.to_string();
        p.step_index = step_index;
        p.total_steps = 4;
        p.percent = percent;
        p.bytes_downloaded = downloaded;
        p.total_bytes = total;
        p.message = message.to_string();
        p.version = version.to_string();
    }
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
            InstallKind::WindowsInstaller | InstallKind::WindowsStandalone => {
                if name == "animesama-windows-x86_64.exe" || name == "animesama.exe" {
                    return Some((name, download_url));
                }
                if name.ends_with(".exe") && name.to_lowercase().contains("setup") && candidate.is_none() {
                    candidate = Some((name, download_url));
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
        install_kind: kind.as_str().to_string(),
    })
}

pub fn download_file_with_progress(url: &str, version: &str) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(10))
        .timeout_read(std::time::Duration::from_secs(120))
        .build();

    let resp = agent
        .get(url)
        .set("User-Agent", "AnimeSama-Desktop-Updater")
        .call()
        .map_err(|e| format!("Erreur lors du téléchargement: {}", e))?;

    let total_bytes: u64 = resp
        .header("Content-Length")
        .and_then(|h| h.parse::<u64>().ok())
        .unwrap_or(0);

    set_update_progress(
        "downloading",
        2,
        0,
        0,
        total_bytes,
        "Téléchargement en cours...",
        version,
    );

    let mut reader = resp.into_reader();
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 65536];
    let mut downloaded: u64 = 0;
    let mut last_percent: u8 = 0;

    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|e| format!("Erreur de lecture du flux: {}", e))?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..n]);
        downloaded += n as u64;

        let percent = if total_bytes > 0 {
            let p = ((downloaded as f64 / total_bytes as f64) * 100.0) as u8;
            p.min(100)
        } else {
            50
        };

        if percent != last_percent || downloaded == total_bytes {
            last_percent = percent;
            set_update_progress(
                "downloading",
                2,
                percent,
                downloaded,
                total_bytes,
                "Téléchargement en cours...",
                version,
            );
        }
    }

    set_update_progress(
        "downloading",
        2,
        100,
        downloaded,
        if total_bytes > 0 { total_bytes } else { downloaded },
        "Téléchargement terminé.",
        version,
    );

    Ok(bytes)
}

pub fn cleanup_old_executable() {
    if let Ok(exe) = std::env::current_exe() {
        let clean_exe = if let Some(stripped) = exe.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
            std::path::PathBuf::from(stripped)
        } else {
            exe
        };
        let old = clean_exe.with_extension("exe.old");
        if old.exists() {
            let _ = fs::remove_file(old);
        }
    }
}

fn schedule_delayed_exit() {
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(1200));
        std::process::exit(0);
    });
}

pub fn apply_update() -> Result<UpdateApplyResult, String> {
    if cfg!(debug_assertions) {
        let update_info = check_for_updates().unwrap_or(UpdateInfo {
            current_version: CURRENT_VERSION.to_string(),
            latest_version: "v0.1.29".to_string(),
            update_available: true,
            release_name: "Anime-Sama Desktop (Simulation)".to_string(),
            release_notes: String::new(),
            release_url: String::new(),
            download_url: None,
            asset_name: None,
            install_kind: "windows_standalone".to_string(),
        });
        let version = update_info.latest_version.clone();

        std::thread::spawn(move || {
            set_update_progress("checking", 1, 100, 0, 0, "Vérification terminée", &version);
            std::thread::sleep(std::time::Duration::from_millis(350));

            let total: u64 = 42_500_000;
            for pct in (0..=100).step_by(5) {
                let downloaded = (total * pct as u64) / 100;
                set_update_progress("downloading", 2, pct as u8, downloaded, total, "Téléchargement des fichiers en cours...", &version);
                std::thread::sleep(std::time::Duration::from_millis(80));
            }

            set_update_progress("installing", 3, 100, total, total, "Installation et remplacement des fichiers...", &version);
            std::thread::sleep(std::time::Duration::from_millis(1200));

            set_update_progress("restart", 4, 100, total, total, "Redémarrage de l'application...", &version);
            std::thread::sleep(std::time::Duration::from_millis(1200));

            set_update_progress("done", 4, 100, total, total, "Mise à jour terminée avec succès.", &version);
        });

        return Ok(UpdateApplyResult {
            success: true,
            message: "Simulation de mise à jour lancée (mode dev).".to_string(),
            needs_restart: false,
            downloaded_file: None,
        });
    }

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
        Some(u) => u.clone(),
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

    let latest_version = update_info.latest_version.clone();
    let asset_name = update_info.asset_name.clone().unwrap_or_else(|| "update.bin".to_string());

    std::thread::spawn(move || {
        set_update_progress("checking", 1, 100, 0, 0, "Vérification terminée", &latest_version);
        std::thread::sleep(std::time::Duration::from_millis(200));

        let bytes = match download_file_with_progress(&download_url, &latest_version) {
            Ok(b) => b,
            Err(e) => {
                set_update_progress("error", 2, 0, 0, 0, &format!("Échec du téléchargement: {}", e), &latest_version);
                return;
            }
        };

        set_update_progress("installing", 3, 100, bytes.len() as u64, bytes.len() as u64, "Installation des fichiers...", &latest_version);

        if asset_name.ends_with(".deb") {
            let temp_dir = std::env::temp_dir();
            let target = temp_dir.join(&asset_name);
            if let Err(e) = fs::write(&target, &bytes) {
                set_update_progress("error", 3, 0, 0, 0, &format!("Échec d'écriture: {}", e), &latest_version);
                return;
            }
            let _ = opener::open(&target);
            set_update_progress("restart", 4, 100, bytes.len() as u64, bytes.len() as u64, "Paquet prêt. Installation lancée.", &latest_version);
            return;
        }

        if cfg!(target_os = "windows") && asset_name.to_lowercase().contains("setup") {
            let temp_dir = std::env::temp_dir();
            let target = temp_dir.join(&asset_name);
            if let Err(e) = fs::write(&target, &bytes) {
                set_update_progress("error", 3, 0, 0, 0, &format!("Échec d'écriture: {}", e), &latest_version);
                return;
            }

            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let _ = std::process::Command::new(&target)
                    .args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/FORCECLOSEAPPLICATIONS"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
            }

            set_update_progress("restart", 4, 100, bytes.len() as u64, bytes.len() as u64, "Redémarrage de l'application...", &latest_version);
            schedule_delayed_exit();
            return;
        }

        let current_exe = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                set_update_progress("error", 3, 0, 0, 0, &format!("Impossible de localiser l'exécutable: {}", e), &latest_version);
                return;
            }
        };

        if cfg!(target_os = "windows") {
            let clean_exe = if let Some(stripped) = current_exe.to_str().and_then(|s| s.strip_prefix(r"\\?\")) {
                std::path::PathBuf::from(stripped)
            } else {
                current_exe.clone()
            };
            let old_exe = clean_exe.with_extension("exe.old");
            let _ = fs::remove_file(&old_exe);
            if let Err(_e) = fs::rename(&clean_exe, &old_exe) {
                let temp_dir = std::env::temp_dir();
                let target = temp_dir.join(&asset_name);
                if let Err(err) = fs::write(&target, &bytes) {
                    set_update_progress("error", 3, 0, 0, 0, &format!("Échec d'écriture: {}", err), &latest_version);
                    return;
                }
                let _ = opener::open(&target);
                set_update_progress("restart", 4, 100, bytes.len() as u64, bytes.len() as u64, "Mise à jour prête. Relancez l'application.", &latest_version);
                return;
            }

            if let Err(e) = fs::write(&clean_exe, &bytes) {
                let _ = fs::rename(&old_exe, &clean_exe);
                set_update_progress("error", 3, 0, 0, 0, &format!("Échec de remplacement: {}", e), &latest_version);
                return;
            }

            set_update_progress("restart", 4, 100, bytes.len() as u64, bytes.len() as u64, "Redémarrage de l'application...", &latest_version);

            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let _ = std::process::Command::new(&clean_exe)
                    .creation_flags(CREATE_NO_WINDOW)
                    .spawn();
            }

            schedule_delayed_exit();
        } else {
            let temp_exe = current_exe.with_extension("new");
            if let Err(e) = fs::write(&temp_exe, &bytes) {
                set_update_progress("error", 3, 0, 0, 0, &format!("Échec d'écriture: {}", e), &latest_version);
                return;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&temp_exe, fs::Permissions::from_mode(0o755));
            }

            if let Err(e) = fs::rename(&temp_exe, &current_exe) {
                set_update_progress("error", 3, 0, 0, 0, &format!("Échec de renommage: {}", e), &latest_version);
                return;
            }

            set_update_progress("restart", 4, 100, bytes.len() as u64, bytes.len() as u64, "Redémarrage de l'application...", &latest_version);

            #[cfg(target_os = "linux")]
            {
                let exe_clone = current_exe.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    std::process::Command::new(&exe_clone).spawn().ok();
                    std::process::exit(0);
                });
            }
        }
    });

    Ok(UpdateApplyResult {
        success: true,
        message: "Mise à jour en cours d'application...".to_string(),
        needs_restart: true,
        downloaded_file: None,
    })
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
        assert_eq!(win_inst.0, "animesama-windows-x86_64.exe");

        let win_stand = select_asset_for_kind(arr, InstallKind::WindowsStandalone).unwrap();
        assert_eq!(win_stand.0, "animesama-windows-x86_64.exe");

        let setup_only = serde_json::json!([
            { "name": "AnimeSama-Setup-x86_64.exe", "browser_download_url": "https://example.com/setup.exe" }
        ]);
        let setup_arr = setup_only.as_array().unwrap();
        let fallback_inst = select_asset_for_kind(setup_arr, InstallKind::WindowsInstaller).unwrap();
        assert_eq!(fallback_inst.0, "AnimeSama-Setup-x86_64.exe");

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
