use super::episodes::{detect_hoster_brand, parse_scan_custom_names, url_encode};
use regex::Regex;

#[test]
fn test_detect_hoster_brand_known_domains() {
    assert_eq!(detect_hoster_brand("https://uqload.vc/embed-3pp9id76kssc.html"), Some("Uqload".to_string()));
    assert_eq!(detect_hoster_brand("https://ansembed.net/embed-ejbsz9lxez75.html"), Some("VidMoly".to_string()));
    assert_eq!(detect_hoster_brand("https://vidmoly.to/embed-xyz.html"), Some("VidMoly".to_string()));
    assert_eq!(detect_hoster_brand("https://derricknationtop.com/e/c4gbyeojdqdo"), Some("VOE".to_string()));
    assert_eq!(detect_hoster_brand("https://jeremyparticipantanything.com/e/asbgooheeuyf"), Some("VOE".to_string()));
    assert_eq!(detect_hoster_brand("https://sendvid.com/embed/4b1f6g3r"), Some("Sendvid".to_string()));
    assert_eq!(detect_hoster_brand("https://minochinos.com/embed/49f4l7vv6pmm"), Some("Earnvids".to_string()));
    assert_eq!(detect_hoster_brand("https://smoothpre.com/embed/wa805bkcrb02"), Some("Earnvids".to_string()));
    assert_eq!(detect_hoster_brand("https://video.sibnet.ru/shell.php?videoid=4826196"), Some("Sibnet".to_string()));
}

#[test]
fn test_extract_languages_regex() {
    let html1 = r#"<script>var languesDispo = ["vostfr","vf","va"];</script>"#;
    let lang_re = Regex::new(r#"(?:var|let|const)\s+languesDispo\s*=\s*\[([^\]]+)\]"#).unwrap();
    let str_re = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();
    let mut langs = Vec::new();
    if let Some(cap) = lang_re.captures(html1) {
        for s in str_re.captures_iter(&cap[1]) {
            langs.push(s[1].trim().to_lowercase());
        }
    }
    assert_eq!(langs, vec!["vostfr", "vf", "va"]);

    let html2 = r#"<a href="/cat/s1/vostfr" id="switchVOSTFR">VOSTFR</a><a href="/cat/s1/vf" id="switchVF">VF</a>"#;
    let switch_re = Regex::new(r#"id=["']switch([A-Za-z0-9]+)["']"#).unwrap();
    let mut langs2 = Vec::new();
    for cap in switch_re.captures_iter(html2) {
        langs2.push(cap[1].trim().to_lowercase());
    }
    assert_eq!(langs2, vec!["vostfr", "vf"]);
}

#[test]
fn test_parse_scan_custom_names_and_encode() {
    let html = r#"
        resetListe();
        creerListe(1, 2);newSP(2.5);
        finirListe(3);
    "#;
    let names = parse_scan_custom_names(html, 4);
    assert_eq!(names, vec![
        "Chapitre 1".to_string(),
        "Chapitre 2".to_string(),
        "Chapitre 2.5".to_string(),
        "Chapitre 3".to_string()
    ]);

    assert_eq!(url_encode("Kagura Bachi"), "Kagura%20Bachi");
    assert_eq!(url_encode("Solo Leveling Side Story"), "Solo%20Leveling%20Side%20Story");
}

#[test]
fn test_fetch_scan_episodes_kagura_bachi() {
    if let Ok(ep) = super::episodes::fetch_scan_episodes("kagura-bachi", "scan/vf") {
        assert!(ep.total_episodes >= 100);
        assert_eq!(ep.scan_data.as_ref().unwrap().oeuvre, "Kagura Bachi");
        assert!(ep.players.contains_key("Scans"));
    }
}
