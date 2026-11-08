pub fn get_shield_styles() -> &'static str {
    r#"<style>
div[id*="ad" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-media):not(.plyr),
div[class*="ad" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-media):not(.plyr):not(.plyr__video-wrapper),
iframe[src*="ad" i], iframe[id*="ad" i], iframe[class*="ad" i],
a[href*="bet" i], a[href*="casino" i], a[href*="1xbet" i],
div[class*="banner" i]:not(video):not(#vplayer):not(#player),
div[id*="banner" i]:not(video):not(#vplayer):not(#player),
div[class*="pop" i]:not(video):not(#vplayer):not(#player),
div[id*="pop" i]:not(video):not(#vplayer):not(#player),
div[class*="sponsor" i], div[id*="sponsor" i],
div[id*="overlay" i]:not(#netflixOverlay):not(.netflix-video-viewport):not(.jw-media):not(.jw-display):not(.jw-overlays):not(.plyr):not(.plyr__video-wrapper),
div[class*="overlay" i]:not(#netflixOverlay):not(.netflix-video-viewport):not(.jw-media):not(.jw-display):not(.jw-overlays):not(.plyr):not(.plyr__video-wrapper) {
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    pointer-events: none !important;
    width: 0 !important;
    height: 0 !important;
    max-width: 0 !important;
    max-height: 0 !important;
    position: absolute !important;
    top: -9999px !important;
    left: -9999px !important;
}

.jw-ad, .jw-ads, .jw-ad-container, .jw-ad-ui, .jw-plugin-ad,
.jw-vast, .jw-vpaid, .ima-ad-container, [class*="ima-ad" i], [id*="ima-ad" i],
[class*="vast-ad" i], [id*="vast-ad" i],
[class*="outstream" i], [id*="outstream" i] {
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    pointer-events: none !important;
    width: 0 !important;
    height: 0 !important;
}

.jw-controlbar, .jw-controls-backdrop, .jw-display-icon-container,
.jw-display-icon-display, .jw-icon-display, .jw-svg-icon, .jw-preview,
.jw-pause-overlay, .jw-dock, .jw-dock-btn,
.jw-related, .jw-nextup, .jw-card, .jw-rightclick,
a.jw-download, .jw-svg-icon-download, a[download], a[href*="download" i], a[href*="/dl" i],
button[class*="download" i], button[id*="download" i], .vidmoly-download,
.vjs-control-bar, .vjs-big-play-button, .vjs-loading-spinner, .vjs-poster,
.plyr__controls, .plyr__poster, .plyr__captions, .plyr__badge,
.voe-title, .voe-watermark, [class*="watermark" i], [id*="watermark" i],
.clappr-controls, .fp-ui, .control-bar, .mejs__controls,
#controls, .controls, [class*="controlbar" i], [class*="player-controls" i],
.vjs-text-track-display, .jw-title {
    display: none !important;
    opacity: 0 !important;
    visibility: hidden !important;
    pointer-events: none !important;
}

.plyr__control--overlaid {
    opacity: 0 !important;
    pointer-events: none !important;
}

.jw-display, .jw-overlays, .plyr, .plyr__video-wrapper {
    display: block !important;
    width: 100% !important;
    height: 100% !important;
    background: transparent !important;
}

video, .jw-media, .jw-video {
    display: block !important;
    width: 100% !important;
    height: 100% !important;
    object-fit: contain !important;
    background: #000 !important;
    opacity: 1 !important;
    visibility: visible !important;
    pointer-events: auto !important;
}

[class*="opera" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(.jw-display-icon-container),
[id*="opera" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(.jw-display-icon-container),
a[href*="opera" i], a[href*="operagx" i], button[onclick*="opera" i],
div:has(> a[href*="opera" i]),
div:has(a[href*="operagx" i]),
div:has(img[src*="opera" i]),
div:has(svg[class*="opera" i]),
div:has([class*="recaptcha" i]),
div:has([id*="recaptcha" i]),
div:has([class*="robot" i]),
[id*="captcha" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[class*="captcha" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[id*="robot" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[class*="robot" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[id*="recaptcha" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[class*="recaptcha" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[id*="verify" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[class*="verify" i]:not(video):not(.jw-video):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media),
[class*="swal" i],
[id*="swal" i],
.sweet-alert,
.sweet-overlay,
div[class*="browser-upgrade" i],
div[class*="browser-update" i],
div[id*="browser-update" i],
div[class*="software-update" i],
div[id*="software-update" i],
div[class*="download-dialog" i],
div[id*="download-dialog" i],
div[class*="push-prompt" i],
div[class*="notify-prompt" i],
div[id*="cf-turnstile" i],
div[class*="cf-turnstile" i] {
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    pointer-events: none !important;
}
</style>"#
}
