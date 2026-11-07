use std::sync::OnceLock;

const AD_DOMAINS_RAW: &str = include_str!("../data/ad_domains.txt");

static AD_DOMAINS: OnceLock<Vec<&'static str>> = OnceLock::new();

pub fn get_ad_domains() -> &'static [&'static str] {
    AD_DOMAINS.get_or_init(|| {
        let mut list: Vec<&'static str> = AD_DOMAINS_RAW
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        list.sort_unstable();
        list
    })
}

pub const WHITELIST: &[&str] = &[
    "anime-sama.to",
    "voe.sx",
    "uqload.to",
    "uqload.com",
    "uqload.io",
    "uqload.ws",
    "uqload.vc",
    "vidmoly.to",
    "vidmoly.me",
    "vidmoly.net",
    "vidmoly.biz",
    "ansembed.net",
    "staticmoly.me",
    "vmpx.online",
    "vmcld.space",
    "jwpcdn.com",
    "jwplayer.com",
    "jwplatform.com",
    "jwpltx.com",
    "fontawesome.com",
    "cloudflare.com",
    "unpkg.com",
    "fonts.googleapis.com",
    "fonts.gstatic.com",
    "sibnet.ru",
    "sendvid.com",
    "minochinos.com",
    "smoothpre.com",
    "morencius.com",
    "movearnpre.com",
    "earnvids.com",
    "streamtape.com",
    "doodstream.com",
    "dood.to",
    "dood.so",
    "dood.watch",
    "dood.la",
    "streamwish.com",
    "streamwish.to",
    "filemoon.sx",
    "filemoon.to",
    "mp4upload.com",
    "vidoza.net",
    "oneupload.to",
    "embed4me.com",
    "player4me.com",
    "4meplayer.com",
    "teresapoliticallearn.com",
    "megankiddifference.com",
    "jeremyparticipantanything.com",
    "jamesbornmain.com",
    "claireguessthousand.com",
    "derricknationtop.com",
    "jamespatterncamera.com",
    "ashleyhardat.com",
    "shanelocaldog.com",
    "shannoncoldalready.com",
    "kitsu.io",
    "jsdelivr.net",
    "cdnjs.cloudflare.com",
];

pub const TRANSPARENT_1X1_GIF: &[u8] = &[
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
    0x00, 0xff, 0xff, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2c, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x01, 0x44, 0x00, 0x3b,
];

pub const DUMMY_SCRIPT: &str = " (function(){})();";

pub fn is_whitelisted(host: &str) -> bool {
    let lower = host.to_lowercase();
    let clean_host = lower.split(':').next().unwrap_or(&lower);
    for &w in WHITELIST {
        if clean_host == w || clean_host.ends_with(&format!(".{}", w)) {
            return true;
        }
    }
    false
}

pub fn is_blocked_host(host: &str) -> bool {
    let lower = host.to_lowercase();
    let clean_host = lower.split(':').next().unwrap_or(&lower);

    if is_whitelisted(clean_host) {
        return false;
    }

    let domains = get_ad_domains();
    let parts: Vec<&str> = clean_host.split('.').collect();
    if parts.len() < 2 {
        return false;
    }

    
    for i in 0..parts.len().saturating_sub(1) {
        let suffix = parts[i..].join(".");
        if domains.binary_search(&suffix.as_str()).is_ok() {
            return true;
        }
    }

    
    if clean_host.contains("popcash")
        || clean_host.contains("adsterra")
        || clean_host.contains("propellerads")
        || clean_host.contains("monetag")
        || clean_host.contains("clickadu")
        || clean_host.contains("exoclick")
        || clean_host.contains("hilltopads")
        || clean_host.contains("doubleclick")
        || clean_host.contains("google-analytics")
        || clean_host.contains("googlesyndication")
        || clean_host.contains("scorecardresearch")
        || clean_host.contains("opera.com")
        || clean_host.contains("operagx")
        || clean_host.contains("operasoftware")
        || clean_host.contains("recaptcha.net")
        || clean_host.contains("hcaptcha.com")
        || clean_host.contains("turnstile")
        || clean_host.contains("propu.sh")
        || clean_host.contains("trafficjunky")
        || clean_host.contains("adsupply")
        || clean_host.contains("syndication")
    {
        return true;
    }

    false
}

pub fn is_blocked_url(url: &str) -> bool {
    let lower = url.to_lowercase();

    
    if let Some(after_scheme) = lower.split("://").nth(1) {
        let host = after_scheme.split('/').next().unwrap_or("");
        if is_whitelisted(host) {
            return false;
        }
        if is_blocked_host(host) {
            return true;
        }
    }

    
    if lower.contains("/popunder")
        || lower.contains("popcash")
        || lower.contains("adsterra")
        || lower.contains("monetag")
        || lower.contains("clickadu")
        || lower.contains("/prebid")
        || lower.contains("doubleclick.net")
        || lower.contains("google-analytics.com")
        || lower.contains("googletagmanager.com/gtm.js")
        || lower.contains("googletagservices.com/tag/js/gpt.js")
        || lower.contains("scorecardresearch.com")
        || lower.contains("propu.sh")
        || lower.contains("opera.com")
        || lower.contains("operagx")
        || lower.contains("operasoftware")
        || lower.contains("recaptcha")
        || lower.contains("turnstile")
        || lower.contains("trafficjunky")
    {
        return true;
    }

    false
}
