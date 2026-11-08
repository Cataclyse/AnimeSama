use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageMeta {
    pub id: String,
    pub name: String,
    pub code: String,
    pub is_custom: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag_svg: Option<String>,
}

pub fn app_data_dir() -> PathBuf {
    let dir = if let Ok(local) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local).join("AnimeSama")
    } else if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("AnimeSama")
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        PathBuf::from(home).join("AppData").join("Local").join("AnimeSama")
    } else if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg).join("AnimeSama")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local").join("share").join("AnimeSama")
    } else {
        PathBuf::from("AnimeSama")
    };

    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn userdata_path() -> PathBuf {
    let target = app_data_dir().join("userdata.json");
    if !target.exists() && Path::new("userdata.json").exists() {
        let _ = fs::copy("userdata.json", &target);
    }
    target
}

pub fn session_path() -> PathBuf {
    let target = app_data_dir().join("session.json");
    if !target.exists() && Path::new("session.json").exists() {
        let _ = fs::copy("session.json", &target);
    }
    target
}

pub fn lang_dir() -> PathBuf {
    let dir = app_data_dir().join("lang");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub const EMBEDDED_LANGS: &[(&str, &str)] = &[
    ("fr.lang", include_str!("../../static/lang/fr.lang")),
    ("en.lang", include_str!("../../static/lang/en.lang")),
    ("ja.lang", include_str!("../../static/lang/ja.lang")),
    ("es.lang", include_str!("../../static/lang/es.lang")),
    ("de.lang", include_str!("../../static/lang/de.lang")),
    ("it.lang", include_str!("../../static/lang/it.lang")),
    ("pt.lang", include_str!("../../static/lang/pt.lang")),
    ("ar.lang", include_str!("../../static/lang/ar.lang")),
    ("ru.lang", include_str!("../../static/lang/ru.lang")),
    ("ko.lang", include_str!("../../static/lang/ko.lang")),
];

pub fn init_app_dirs() {
    let _ = app_data_dir();
    let _ = userdata_path();
    let _ = session_path();
    let dir = lang_dir();

    for (name, content) in EMBEDDED_LANGS {
        let dest = dir.join(name);
        if !dest.exists() {
            let _ = fs::write(&dest, content);
        } else if let Ok(existing) = fs::read_to_string(&dest) {
            let mut merged = existing;
            let mut changed = false;
            for line in content.lines() {
                if let Some(eq) = line.find('=') {
                    let key = &line[..eq];
                    if !merged.contains(key) {
                        if !merged.ends_with('\n') {
                            merged.push('\n');
                        }
                        merged.push_str(line);
                        merged.push('\n');
                        changed = true;
                    }
                }
            }
            if changed {
                let _ = fs::write(&dest, &merged);
            }
        }
    }

    if let Ok(entries) = fs::read_dir("static/lang") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("lang") {
                if let Some(file_name) = p.file_name() {
                    let dest = dir.join(file_name);
                    if !dest.exists() {
                        let _ = fs::copy(&p, &dest);
                    }
                }
            }
        }
    }
}

pub fn list_available_languages() -> Vec<LanguageMeta> {
    let mut map = std::collections::BTreeMap::new();

    let builtins = [
        ("fr", "Français", "FR"),
        ("en", "English", "EN"),
        ("ja", "日本語", "JA"),
        ("es", "Español", "ES"),
        ("de", "Deutsch", "DE"),
        ("it", "Italiano", "IT"),
        ("pt", "Português", "PT"),
        ("ar", "العربية", "AR"),
        ("ru", "Русский", "RU"),
        ("ko", "한국어", "KO"),
    ];

    for (id, name, code) in builtins {
        map.insert(
            id.to_string(),
            LanguageMeta {
                id: id.to_string(),
                name: name.to_string(),
                code: code.to_string(),
                is_custom: false,
                flag_svg: None,
            },
        );
    }

    let dir = lang_dir();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("lang") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let stem_lower = stem.to_lowercase();
                    let is_builtin = builtins.iter().any(|(b_id, _, _)| *b_id == stem_lower);

                    let mut name = stem.to_string();
                    let mut code = stem.to_uppercase();
                    if code.len() > 3 {
                        code = code.chars().take(3).collect();
                    }
                    let mut flag_svg = None;

                    if let Ok(content) = fs::read_to_string(&path) {
                        for line in content.lines() {
                            let line = line.trim();
                            if let Some((k, v)) = line.split_once('=') {
                                let k = k.trim();
                                let v = v.trim();
                                match k {
                                    "meta.name" => name = v.to_string(),
                                    "meta.code" => code = v.to_string(),
                                    "meta.flag" => flag_svg = Some(v.to_string()),
                                    _ => (),
                                }
                            }
                        }
                    }

                    if is_builtin {
                        if let Some(existing) = map.get_mut(&stem_lower) {
                            if name != stem {
                                existing.name = name;
                            }
                            if flag_svg.is_some() {
                                existing.flag_svg = flag_svg;
                            }
                        }
                    } else {
                        map.insert(
                            stem_lower.clone(),
                            LanguageMeta {
                                id: stem_lower,
                                name,
                                code,
                                is_custom: true,
                                flag_svg,
                            },
                        );
                    }
                }
            }
        }
    }

    map.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_paths() {
        init_app_dirs();
        let app_dir = app_data_dir();
        assert!(app_dir.exists());
        let lang = lang_dir();
        assert!(lang.exists());
        let langs = list_available_languages();
        assert!(langs.len() >= 10);
    }
}
