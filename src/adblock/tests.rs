use super::*;

#[test]
fn test_blocked_domains() {
    assert!(is_blocked_host("popcash.net"));
    assert!(is_blocked_host("sub.popcash.net"));
    assert!(is_blocked_host("deep.sub.popcash.net"));
    assert!(is_blocked_host("doubleclick.net"));
    assert!(is_blocked_host("ad.doubleclick.net"));
    assert!(is_blocked_host("google-analytics.com"));
    assert!(is_blocked_host("adsterra.com"));
    assert!(is_blocked_host("operagx.com"));
    assert!(is_blocked_host("download.operasoftware.com"));
}

#[test]
fn test_whitelist_protection() {
    assert!(!is_blocked_host("voe.sx"));
    assert!(!is_blocked_host("uqload.to"));
    assert!(!is_blocked_host("uqload.vc"));
    assert!(!is_blocked_host("vidmoly.to"));
    assert!(!is_blocked_host("ansembed.net"));
    assert!(!is_blocked_host("staticmoly.me"));
    assert!(!is_blocked_host("sibnet.ru"));
    assert!(!is_blocked_host("anime-sama.to"));
    assert!(!is_blocked_host("kitsu.io"));
    assert!(!is_blocked_host("jwplayer.com"));
    assert!(!is_blocked_host("cdn.jwplayer.com"));
    assert!(!is_blocked_host("fontawesome.com"));
}

#[test]
fn test_ad_url_detection() {
    assert!(is_blocked_url("https://popcash.net/pop.js"));
    assert!(is_blocked_url("https://example.com/popunder.js"));
    assert!(!is_blocked_url("https://voe.sx/hls/master.m3u8"));
    assert!(!is_blocked_url("https://ansembed.net/static/jquery.min.js"));
}

#[test]
fn test_sanitize_html() {
    let raw = r#"<html><head><title>Test</title><script src="https://popcash.net/pop.js"></script><script src="https://ansembed.net/static/player.js"></script></head><body><h1>Video</h1><iframe src="https://popunder.com/banner"></iframe></body></html>"#;
    let sanitized = sanitize_embed_html(raw, "ansembed.net", 120.0, 2);
    assert!(!sanitized.contains("popcash.net"));
    assert!(!sanitized.contains("<iframe"));
    assert!(sanitized.contains("player.js"));
    assert!(sanitized.contains("resumeTarget = 120.00"));
    assert!(sanitized.contains("asEpNum = 2"));
    assert!(sanitized.contains("killFakePopupsAndBanners"));
    assert!(sanitized.contains("opera"));

    let resume_only = inject_resume_only_html(raw, 45.0, 3);
    assert!(resume_only.contains("resumeTarget = 45.00"));
    assert!(resume_only.contains("asEpNum = 3"));
}
