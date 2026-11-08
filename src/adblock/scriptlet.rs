pub fn generate_shield_script(resume_seconds: f64, episode_num: u32) -> String {
    format!(
        r#"
<meta name="referrer" content="no-referrer">
<style>

iframe {{
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
}}

.jw-ad, .jw-ads, .jw-ad-container, .jw-ad-ui, .jw-plugin-ad,
.jw-vast, .jw-vpaid, .ima-ad-container, [class*="ima-ad" i], [id*="ima-ad" i],
[class*="vast-ad" i], [id*="vast-ad" i],
[class*="outstream" i], [id*="outstream" i] {{
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    pointer-events: none !important;
    width: 0 !important;
    height: 0 !important;
}}

.jw-controlbar, .jw-controls, .jw-controls-backdrop, .jw-display-icon-container,
.jw-display-icon-display, .jw-icon-display, .jw-svg-icon, .jw-preview,
.jw-pause-overlay, .jw-overlays, .jw-overlay, .jw-display, .jw-dock, .jw-dock-btn,
.jw-related, .jw-nextup, .jw-card, .jw-rightclick, .jw-flag-paused .jw-overlays,
.jw-flag-paused .jw-display, .jw-flag-paused [class*="download" i],
.vjs-control-bar, .vjs-big-play-button, .vjs-loading-spinner, .vjs-poster,
.plyr__controls, .plyr__control--overlaid, .plyr__poster, .plyr__captions, .plyr__badge,
.plyr--paused .plyr__control--overlaid, .voe-title, .voe-watermark,
[class*="watermark" i], div[class*="play-wrapper" i], div[id*="play-wrapper" i],
div[class*="loading-overlay" i], div[class*="center-icon" i],
.btn-play, .play-button, .overlay-play, .clappr-controls, .fp-ui, .control-bar, .mejs__controls,
#controls, .controls, [class*="controlbar" i], [class*="player-controls" i],
.vjs-text-track-display, .jw-title, [class*="download" i], [id*="download" i],
a[href*="download" i], a[download], button[class*="download" i], .vidmoly-download, div[class*="moly" i] {{
    display: none !important;
    opacity: 0 !important;
    visibility: hidden !important;
    pointer-events: none !important;
    width: 0 !important;
    height: 0 !important;
}}

video, .jw-media, .jw-video {{
    display: block !important;
    width: 100% !important;
    height: 100% !important;
    object-fit: contain !important;
    background: #000 !important;
    opacity: 1 !important;
    visibility: visible !important;
    pointer-events: auto !important;
}}

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
div[class*="cf-turnstile" i] {{
    display: none !important;
    visibility: hidden !important;
    opacity: 0 !important;
    pointer-events: none !important;
}}
</style>
<script>
(function() {{
    'use strict';

    
    
    
    try {{
        var origCreateElement = document.createElement;
        document.createElement = function(tagName) {{
            var el = origCreateElement.apply(this, arguments);
            if (tagName && typeof tagName === 'string') {{
                var t = tagName.toLowerCase();
                if (t === 'iframe') {{
                    try {{
                        el.src = 'about:blank';
                        el.style.display = 'none';
                        el.style.width = '0px';
                        el.style.height = '0px';
                        el.style.pointerEvents = 'none';
                    }} catch(err) {{}}
                }}
            }}
            return el;
        }};
        var origAppendChild = Node.prototype.appendChild;
        Node.prototype.appendChild = function(child) {{
            if (child && child.tagName && child.tagName.toUpperCase() === 'IFRAME') {{
                return child;
            }}
            return origAppendChild.apply(this, arguments);
        }};
        var origInsertBefore = Node.prototype.insertBefore;
        Node.prototype.insertBefore = function(newNode, referenceNode) {{
            if (newNode && newNode.tagName && newNode.tagName.toUpperCase() === 'IFRAME') {{
                return newNode;
            }}
            return origInsertBefore.apply(this, arguments);
        }};

        var isBlockedScriptUrl = function(u) {{
            if (!u || typeof u !== 'string') return false;
            var s = u.toLowerCase();
            return (
                s.indexOf('opera.com') !== -1 ||
                s.indexOf('operagx') !== -1 ||
                s.indexOf('operasoftware') !== -1 ||
                s.indexOf('popunder') !== -1 ||
                s.indexOf('popcash') !== -1 ||
                s.indexOf('adsterra') !== -1 ||
                s.indexOf('monetag') !== -1 ||
                s.indexOf('clickadu') !== -1 ||
                s.indexOf('propellerads') !== -1 ||
                s.indexOf('exoclick') !== -1 ||
                s.indexOf('hilltopads') !== -1 ||
                s.indexOf('recaptcha') !== -1 ||
                s.indexOf('turnstile') !== -1 ||
                s.indexOf('propu.sh') !== -1 ||
                s.indexOf('trafficjunky') !== -1 ||
                s.indexOf('syndication') !== -1
            );
        }};

        var origScriptSrcDesc = Object.getOwnPropertyDescriptor(HTMLScriptElement.prototype, 'src');
        if (origScriptSrcDesc && origScriptSrcDesc.set) {{
            Object.defineProperty(HTMLScriptElement.prototype, 'src', {{
                set: function(val) {{
                    if (isBlockedScriptUrl(val)) {{
                        this.type = 'text/plain';
                        return '';
                    }}
                    return origScriptSrcDesc.set.call(this, val);
                }},
                get: function() {{
                    return origScriptSrcDesc.get.call(this);
                }},
                configurable: true
            }});
        }}
    }} catch(e) {{}}

    
    
    
    try {{
        if (window.frameElement) {{
            var origHasAttr = window.frameElement.hasAttribute;
            window.frameElement.hasAttribute = function(name) {{
                if (name === 'sandbox') return false;
                return origHasAttr ? origHasAttr.apply(this, arguments) : false;
            }};
        }}
    }} catch(e) {{}}

    
    
    
    var dummyWindow = {{
        closed: false,
        name: '',
        opener: null,
        parent: window,
        top: window,
        close: function() {{ this.closed = true; }},
        focus: function() {{}},
        blur: function() {{}},
        postMessage: function() {{}},
        document: {{
            write: function() {{}},
            writeln: function() {{}},
            open: function() {{ return this; }},
            close: function() {{}},
            createElement: function() {{ return {{}}; }}
        }},
        location: {{
            href: '',
            replace: function() {{}},
            assign: function() {{}}
        }}
    }};

    var origOpen = window.open;
    window.open = function(url, target, features) {{
        console.warn('[AdShield] Bloqué popup/popunder uBlock:', url);
        return dummyWindow;
    }};

    try {{
        var origAnchorClick = HTMLAnchorElement.prototype.click;
        HTMLAnchorElement.prototype.click = function() {{
            var href = (this.href || '').toLowerCase();
            var target = (this.target || '').toLowerCase();
            if (target === '_blank' || target === '_new' ||
                href.indexOf('popunder') !== -1 ||
                href.indexOf('adsterra') !== -1 ||
                href.indexOf('monetag') !== -1 ||
                href.indexOf('opera.com') !== -1 ||
                href.indexOf('update') !== -1 ||
                href.indexOf('installer') !== -1) {{
                console.warn('[AdShield] Bloqué clic programmatique sur lien externe/pub:', href);
                return;
            }}
            return origAnchorClick.apply(this, arguments);
        }};
    }} catch(e) {{}}

    
    
    
    function makeFabClass() {{
        function Fab() {{}}
        var noop = function() {{}};
        Fab.prototype.check = noop;
        Fab.prototype.clearEvent = noop;
        Fab.prototype.emitEvent = noop;
        Fab.prototype.on = function(detected, fn) {{
            if (!detected && typeof fn === 'function') {{
                try {{ fn(); }} catch(err) {{}}
            }}
            return this;
        }};
        Fab.prototype.onDetected = function() {{ return this; }};
        Fab.prototype.onNotDetected = function(fn) {{
            if (typeof fn === 'function') {{
                try {{ fn(); }} catch(err) {{}}
            }}
            return this;
        }};
        Fab.prototype.setOption = noop;
        Fab.prototype.options = {{ set: noop, get: noop }};
        return Fab;
    }}

    var FabClass = makeFabClass();
    var fabInstance = new FabClass();

    function defProp(prop, val) {{
        try {{
            Object.defineProperty(window, prop, {{
                get: function() {{ return val; }},
                set: function() {{}},
                configurable: true
            }});
        }} catch(e) {{
            window[prop] = val;
        }}
    }}

    ['FuckAdBlock', 'BlockAdBlock', 'SniffAdBlock'].forEach(function(k) {{ defProp(k, FabClass); }});
    ['fuckAdBlock', 'blockAdBlock', 'sniffAdBlock'].forEach(function(k) {{ defProp(k, fabInstance); }});

    
    window.canRunAds = true;
    window.isAdBlockActive = false;
    window.adBlockDetected = false;
    window.google_ad_status = 1;
    window.nH7eXzOsG = 858; 

    
    
    
    
    window.adsbygoogle = window.adsbygoogle || [];
    window.adsbygoogle.loaded = true;
    window.adsbygoogle.push = function() {{}};

    
    window.adblockDetector = {{
        init: function(opts) {{
            if (opts && typeof opts.notfound === 'function') {{
                setTimeout(opts.notfound, 10);
            }}
        }}
    }};

    window.google = window.google || {{}};
    if (!window.google.ima) {{
        window.google.ima = {{
            AdDisplayContainer: function() {{ this.initialize = function() {{}}; this.destroy = function() {{}}; }},
            AdsLoader: function() {{
                var listeners = {{}};
                this.addEventListener = function(t, fn) {{ listeners[t] = fn; }};
                this.removeEventListener = function() {{}};
                this.requestAds = function() {{
                    setTimeout(function() {{
                        if (listeners['adsManagerLoaded']) {{
                            listeners['adsManagerLoaded']({{
                                getAdsManager: function() {{
                                    return {{
                                        init: function() {{}},
                                        start: function() {{}},
                                        stop: function() {{}},
                                        destroy: function() {{}},
                                        addEventListener: function(t, fn) {{
                                            if (t === 'allAdsCompleted' || t === 'contentResumeRequested') {{
                                                setTimeout(fn, 10);
                                            }}
                                        }}
                                    }};
                                }}
                            }});
                        }}
                    }}, 10);
                }};
                this.contentComplete = function() {{}};
                this.destroy = function() {{}};
            }},
            AdsRequest: function() {{}},
            ViewMode: {{ FULLSCREEN: 'fullscreen', NORMAL: 'normal' }}
        }};
    }}

    
    window.ga = window.gtag = function() {{}};
    window.dataLayer = window.dataLayer || [];
    window.COMSCORE = {{ purge: function() {{}}, beacon: function() {{}} }};
    window.apstag = {{ init: function() {{}}, fetchBids: function(a, b) {{ if (b) b(); }} }};

    
    
    
    try {{
        if (window.Notification) {{
            window.Notification.requestPermission = function() {{
                console.warn('[AdShield] Bloqué faux prompt de notification');
                return Promise.resolve('denied');
            }};
            Object.defineProperty(window.Notification, 'permission', {{
                get: function() {{ return 'denied'; }},
                set: function() {{}},
                configurable: true
            }});
        }}
    }} catch(e) {{}}

    window.alert = function() {{}};
    window.confirm = function() {{ return false; }};
    window.prompt = function() {{ return null; }};

    function isProtectedPlayerElement(el) {{
        if (!el || el === document.body || el === document.documentElement) return true;
        var tag = (el.tagName || '').toUpperCase();
        if (tag === 'VIDEO' || tag === 'AUDIO' || tag === 'SOURCE' || tag === 'TRACK') return true;

        
        var txt = (el.textContent || '').toLowerCase();
        if (
            txt.indexOf('robot') !== -1 ||
            txt.indexOf('opera') !== -1 ||
            txt.indexOf('recommandé') !== -1 ||
            txt.indexOf('recaptcha') !== -1 ||
            txt.indexOf('installer') !== -1 ||
            txt.indexOf('continuer à regarder') !== -1 ||
            txt.indexOf('software update') !== -1 ||
            txt.indexOf('mise à jour') !== -1
        ) {{
            return false;
        }}

        
        if (el.id === 'vplayer' || el.id === 'player') return true;

        if (el.classList) {{
            
            if (
                (el.classList.contains('jwplayer') || el.classList.contains('video-js')) &&
                (el.parentElement === document.body || (el.parentElement && (el.parentElement.id === 'vplayer' || el.parentElement.id === 'player')))
            ) {{
                return true;
            }}
            
            if (
                el.classList.contains('jw-media') ||
                el.classList.contains('jw-video') ||
                el.classList.contains('jw-preview') ||
                el.classList.contains('jw-controls') ||
                el.classList.contains('jw-controlbar') ||
                el.classList.contains('jw-slider') ||
                el.classList.contains('jw-button-color') ||
                el.classList.contains('jw-settings-menu') ||
                el.classList.contains('jw-captions') ||
                el.classList.contains('jw-text-track-cue') ||
                el.classList.contains('jw-display-icon-container') ||
                el.classList.contains('jw-display-icon-display') ||
                el.classList.contains('jw-icon-display') ||
                el.classList.contains('jw-svg-icon') ||
                el.classList.contains('jw-title') ||
                el.classList.contains('vjs-control-bar') ||
                el.classList.contains('vjs-tech')
            ) {{
                return true;
            }}
        }}
        return false;
    }}

    
    
    
    function runOverlayBuster() {{
        try {{
            var docEl = document.documentElement;
            var bodyEl = document.body;
            if (!docEl || !bodyEl) return;
            var vw = Math.min(docEl.clientWidth, window.innerWidth);
            var vh = Math.min(docEl.clientHeight, window.innerHeight);
            if (vw <= 0 || vh <= 0) return;

            var testPoints = [
                [vw / 2, vh / 2],
                [vw / 2, vh * 0.35],
                [vw / 2, vh * 0.65],
                [vw * 0.35, vh / 2],
                [vw * 0.65, vh / 2]
            ];

            for (var p = 0; p < testPoints.length; p++) {{
                var pt = testPoints[p];
                var topEl = document.elementFromPoint(pt[0], pt[1]);
                while (topEl && topEl !== bodyEl && topEl !== docEl) {{
                    if (isProtectedPlayerElement(topEl)) {{
                        break;
                    }}
                    var style = window.getComputedStyle(topEl);
                    var pos = style.position;
                    var z = parseInt(style.zIndex, 10) || 0;
                    var rect = topEl.getBoundingClientRect();

                    var coversArea = (rect.width > vw * 0.25 && rect.height > vh * 0.2);
                    var isSuspicious = (pos === 'fixed' || (pos === 'absolute' && z >= 5)) && coversArea;

                    if (isSuspicious) {{
                        if (!topEl.querySelector('video') && !topEl.querySelector('.jw-controlbar') && !topEl.classList.contains('jw-media')) {{
                            topEl.remove();
                            break;
                        }}
                    }}
                    topEl = topEl.parentElement;
                }}
            }}
        }} catch(err) {{}}
    }}

    function killFakePopupsAndBanners() {{
        try {{
            
            var overlays = document.querySelectorAll('div, section, aside, form, p, dialog, span, h1, h2, h3, h4, a, button, [role="dialog"], [role="alertdialog"]');
            for (var j = 0; j < overlays.length; j++) {{
                var item = overlays[j];
                var txt = (item.textContent || '').toLowerCase().trim();
                if (txt.length < 3 || txt.length > 2000) continue;

                var isFakeRobot = (
                    txt.indexOf('pas un robot') !== -1 ||
                    txt.indexOf('not a robot') !== -1 ||
                    txt.indexOf('recaptcha') !== -1 ||
                    txt.indexOf('confirmez que vous') !== -1 ||
                    txt.indexOf('cliquez sur le bouton') !== -1 ||
                    txt.indexOf('cliquez sur autoriser') !== -1 ||
                    txt.indexOf('click allow') !== -1 ||
                    txt.indexOf('press allow') !== -1 ||
                    txt.indexOf('verify you are human') !== -1 ||
                    txt.indexOf('prouvez que vous') !== -1
                );
                var isFakeOperaOrUpdate = (
                    txt.indexOf('navigateur recommandé') !== -1 ||
                    txt.indexOf('mise à jour logicielle') !== -1 ||
                    txt.indexOf('nouveau navigateur') !== -1 ||
                    txt.indexOf('tester maintenant') !== -1 ||
                    txt.indexOf('software update') !== -1 ||
                    txt.indexOf('browser is out of date') !== -1 ||
                    txt.indexOf('navigateur est obsolète') !== -1 ||
                    (txt.indexOf('opera') !== -1 && (
                        txt.indexOf('navigateur') !== -1 ||
                        txt.indexOf('installer') !== -1 ||
                        txt.indexOf('vpn') !== -1 ||
                        txt.indexOf('bloqueur') !== -1 ||
                        txt.indexOf('mise à jour') !== -1 ||
                        txt.indexOf('recommandé') !== -1 ||
                        txt.indexOf('continuer à regarder') !== -1 ||
                        txt.indexOf('performances') !== -1 ||
                        txt.indexOf('gx') !== -1
                    )) ||
                    (txt.indexOf('continuer à regarder') !== -1 && (
                        txt.indexOf('installer') !== -1 ||
                        txt.indexOf('navigateur') !== -1
                    ))
                );

                if (isFakeRobot || isFakeOperaOrUpdate) {{
                    var topBox = item;
                    while (
                        topBox.parentElement &&
                        topBox.parentElement !== document.body &&
                        topBox.parentElement !== document.documentElement &&
                        topBox.parentElement.id !== 'vplayer' &&
                        topBox.parentElement.id !== 'player' &&
                        !topBox.parentElement.classList.contains('jwplayer') &&
                        !topBox.parentElement.classList.contains('video-js')
                    ) {{
                        topBox = topBox.parentElement;
                    }}
                    var parentNode = topBox.parentElement;
                    try {{ topBox.remove(); }} catch(e) {{}}
                    if (parentNode && parentNode !== document.body && parentNode !== document.documentElement && parentNode.id !== 'vplayer' && parentNode.id !== 'player') {{
                        var pStyle = window.getComputedStyle(parentNode);
                        if (pStyle.position === 'fixed' || pStyle.position === 'absolute') {{
                            if (!parentNode.querySelector('video') && !parentNode.querySelector('.jw-controlbar')) {{
                                try {{ parentNode.remove(); }} catch(e) {{}}
                            }}
                        }}
                    }}
                    var masks = document.querySelectorAll('[class*="backdrop" i], [class*="modal-mask" i], [class*="overlay-dark" i]');
                    for (var m = 0; m < masks.length; m++) {{
                        if (!isProtectedPlayerElement(masks[m])) {{
                            try {{ masks[m].remove(); }} catch(e) {{}}
                        }}
                    }}
                }}
            }}

            
            var selectors = [
                '.jw-ad', '.jw-ads', '.jw-ad-container', '.jw-ad-ui', '.jw-plugin-ad',
                '.jw-vast', '.jw-vpaid', '.ima-ad-container',
                '[class*="opera" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(.jw-display-icon-container):not(video)',
                '[id*="opera" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(.jw-display-icon-container):not(video)',
                'a[href*="opera" i]', 'a[href*="operagx" i]', 'button[onclick*="opera" i]',
                '[id*="captcha" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[class*="captcha" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[id*="robot" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[class*="robot" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[id*="recaptcha" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[class*="recaptcha" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[id*="verify" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                '[class*="verify" i]:not(.jwplayer):not(#vplayer):not(#player):not(.jw-controlbar):not(.jw-controls):not(.jw-media):not(video)',
                'div[class*="push-prompt" i]', 'div[class*="notify-prompt" i]',
                'div[id*="cf-turnstile" i]', 'div[class*="cf-turnstile" i]',
                'div[id*="cloudflare" i]', 'div[class*="cloudflare" i]',
                'div[id*="browser-update" i]', 'div[class*="browser-update" i]',
                'div[id*="software-update" i]', 'div[class*="software-update" i]',
                'div[class*="download-dialog" i]', 'div[id*="download-dialog" i]',
                'div[class*="browser-upgrade" i]'
            ];
            var candidates = document.querySelectorAll(selectors.join(','));
            for (var i = 0; i < candidates.length; i++) {{
                var cand = candidates[i];
                if (isProtectedPlayerElement(cand)) continue;
                try {{ cand.remove(); }} catch(e) {{}}
            }}

            
            var badLinks = document.querySelectorAll('a[href*="opera" i], a[href*="operagx" i], a[href*="update" i], a[href*="install" i], button[onclick*="opera" i]');
            for (var k = 0; k < badLinks.length; k++) {{
                var bad = badLinks[k];
                if (isProtectedPlayerElement(bad)) continue;
                try {{ bad.remove(); }} catch(e) {{}}
            }}

            
            var adIframes = document.querySelectorAll('iframe');
            for (var f = 0; f < adIframes.length; f++) {{
                try {{
                    adIframes[f].remove();
                }} catch(e) {{}}
            }}

            var residues = document.querySelectorAll(
                '.jw-pause-overlay, .jw-overlays, .jw-related, .jw-dock, .jw-nextup, ' +
                '[class*="download" i], [id*="download" i], a[href*="download" i], a[download], button[class*="download" i], ' +
                '.plyr__control--overlaid, .plyr__controls, .plyr__poster, [class*="watermark" i], .voe-title, .voe-watermark'
            );
            for (var r = 0; r < residues.length; r++) {{
                if (isProtectedPlayerElement(residues[r])) continue;
                try {{ residues[r].remove(); }} catch(e) {{}}
            }}

            
            runOverlayBuster();
        }} catch(e) {{}}
    }}

    
    try {{
        var observer = new MutationObserver(function() {{
            killFakePopupsAndBanners();
        }});
        if (document.documentElement) {{
            observer.observe(document.documentElement, {{ childList: true, subtree: true }});
        }}
    }} catch(e) {{}}

    setInterval(killFakePopupsAndBanners, 200);
    document.addEventListener('DOMContentLoaded', killFakePopupsAndBanners);

    
    
    
    function setupOneClickPlay() {{
        var userStarted = false;
        function tryStart(e) {{
            if (userStarted) return;
            var t = e.target;
            if (t && t.closest && t.closest('.jw-controlbar, .jw-settings-menu, .jw-button-color, button, input')) {{
                return;
            }}
            try {{
                var v = document.querySelector('video');
                if (v && v.paused) {{
                    var p = v.play();
                    if (p && typeof p.catch === 'function') p.catch(function(){{}});
                    userStarted = true;
                }}
            }} catch(err) {{}}
        }}
        document.addEventListener('click', tryStart, {{ capture: true }});
        window.addEventListener('playing', function() {{ userStarted = true; }}, true);
    }}
    setupOneClickPlay();

    
    
    
    function cleanAdOverlays() {{
        try {{
            var badEls = document.querySelectorAll(
                'div[id*="popunder"], div[class*="popunder"], ' +
                'div[class*="ad-overlay"], div[id*="ad-overlay"], ' +
                'a[target="_blank"][style*="position: absolute"], ' +
                'a[target="_blank"][style*="position: fixed"]'
            );
            for (var i = 0; i < badEls.length; i++) {{
                var el = badEls[i];
                if (isProtectedPlayerElement(el)) continue;
                if (el.id === 'vidmolyadblocktest') continue;
                el.remove();
            }}
        }} catch(e) {{}}
    }}

    setInterval(cleanAdOverlays, 800);
    document.addEventListener('DOMContentLoaded', cleanAdOverlays);

    
    
    
    var resumeTarget = {:.2};
    var asEpNum = {};
    try {{
        if (!resumeTarget || resumeTarget <= 0) {{
            var p = new URLSearchParams(window.location.search);
            resumeTarget = parseFloat(p.get('as_resume') || '0');
        }}
        if (!asEpNum || asEpNum <= 0) {{
            var p2 = new URLSearchParams(window.location.search);
            asEpNum = parseInt(p2.get('as_ep') || '0', 10);
        }}
    }} catch(e) {{}}

    var lastReported = 0;
    var maxTimeSeen = 0;
    function report(v, force) {{
        if (!v || isNaN(v.currentTime)) return;
        var now = Date.now();
        if (v.currentTime > maxTimeSeen) {{
            maxTimeSeen = v.currentTime;
        }}
        
        if (maxTimeSeen > 5 && v.currentTime < maxTimeSeen - 15) {{
            return;
        }}
        
        
        if (resumeTarget > 2 && !v._asSeekCompleted && v.currentTime < resumeTarget - 2) {{
            return;
        }}
        if (!force && now - lastReported < 250) return;
        lastReported = now;
        var bEnd = 0;
        try {{
            if (v.buffered && v.buffered.length > 0) {{
                bEnd = v.buffered.end(v.buffered.length - 1);
            }}
        }} catch(e) {{}}
        try {{
            window.parent.postMessage({{
                type: 'AS_PLAYBACK_PROGRESS',
                ep: asEpNum,
                currentTime: v.currentTime,
                duration: v.duration || 0,
                paused: v.paused,
                volume: v.volume,
                muted: v.muted,
                playbackRate: v.playbackRate,
                bufferedEnd: bEnd
            }}, '*');
        }} catch(e) {{}}
    }}

    function hookVideo(v) {{
        if (v._asHooked) return;
        v._asHooked = true;
        v._asSeekCompleted = (resumeTarget <= 2);

        function attemptSeek() {{
            if (v._asSeekCompleted || resumeTarget <= 2) return;
            try {{
                
                if (v.readyState >= 1 || (v.duration && v.duration > 2) || v.currentTime > 0) {{
                    v.currentTime = resumeTarget;
                    if (Math.abs(v.currentTime - resumeTarget) < 3) {{
                        v._asSeekCompleted = true;
                    }}
                }}
            }} catch(e) {{}}
        }}

        ['loadedmetadata', 'loadeddata', 'canplay', 'canplaythrough', 'play', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, attemptSeek);
        }});

        ['play', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_PLAY_STATE', paused: false }}, '*'); }} catch(e){{}}
            }});
        }});
        ['pause'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_PLAY_STATE', paused: true }}, '*'); }} catch(e){{}}
            }});
        }});
        ['waiting'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_WAITING', waiting: true }}, '*'); }} catch(e){{}}
            }});
        }});
        ['canplay', 'canplaythrough', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_WAITING', waiting: false }}, '*'); }} catch(e){{}}
            }});
        }});
        v.addEventListener('ended', function() {{
            try {{ window.parent.postMessage({{ type: 'AS_ENDED' }}, '*'); }} catch(e){{}}
        }});

        v.addEventListener('seeked', function() {{
            if (resumeTarget > 2 && Math.abs(v.currentTime - resumeTarget) < 4) {{
                v._asSeekCompleted = true;
            }}
            report(v, true);
        }});

        v.addEventListener('timeupdate', function() {{
            if (!v._asSeekCompleted && resumeTarget > 2) {{
                if (v.currentTime < resumeTarget - 2) {{
                    try {{ v.currentTime = resumeTarget; }} catch(e) {{}}
                    return; 
                }} else {{
                    v._asSeekCompleted = true;
                }}
            }}
            report(v, false);
        }});

        v.addEventListener('pause', function() {{ report(v, true); }});
        v.addEventListener('ended', function() {{ report(v, true); }});

        
        window.addEventListener('beforeunload', function() {{ report(v, true); }});
        window.addEventListener('pagehide', function() {{ report(v, true); }});
        document.addEventListener('visibilitychange', function() {{
            if (document.visibilityState === 'hidden') report(v, true);
        }});
    }}

    var videoPoll = setInterval(function() {{
        var list = document.querySelectorAll('video');
        for (var i = 0; i < list.length; i++) hookVideo(list[i]);
    }}, 300);
    setTimeout(function() {{ clearInterval(videoPoll); }}, 60000);

    function triggerHosterPlay() {{
        try {{
            if (window.player && typeof window.player.play === 'function') {{
                window.player.play();
            }}
        }} catch(e) {{}}
        try {{
            if (window.jwplayer && typeof window.jwplayer === 'function') {{
                var jwp = window.jwplayer();
                if (jwp && typeof jwp.play === 'function') jwp.play();
            }}
        }} catch(e) {{}}
        try {{
            var playBtns = document.querySelectorAll(
                '.plyr__control--overlaid, button[data-plyr="play"], .play-wrapper, .play-button, .btn-play, .overlay-play, .vjs-big-play-button, .jw-display-icon-display, [aria-label="Play"]'
            );
            for (var b = 0; b < playBtns.length; b++) {{
                playBtns[b].click();
            }}
        }} catch(e) {{}}
        try {{
            var v = document.querySelector('video');
            if (v && v.paused) {{
                var p = v.play();
                if (p && typeof p.catch === 'function') p.catch(function(){{}});
            }}
        }} catch(e) {{}}
    }}

    function triggerHosterPause() {{
        try {{
            var v = document.querySelector('video');
            if (v) v.pause();
        }} catch(e) {{}}
        try {{
            if (window.player && typeof window.player.pause === 'function') {{
                window.player.pause();
            }}
        }} catch(e) {{}}
        try {{
            if (window.jwplayer && typeof window.jwplayer === 'function') {{
                var jwp = window.jwplayer();
                if (jwp && typeof jwp.pause === 'function') jwp.pause();
            }}
        }} catch(e) {{}}
    }}

    function togglePlay() {{
        var v = document.querySelector('video');
        if (v && !v.paused) {{
            triggerHosterPause();
        }} else {{
            triggerHosterPlay();
        }}
    }}

    
    window.addEventListener('keydown', function(e) {{
        var tag = (e.target && e.target.tagName || '').toUpperCase();
        if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target && e.target.isContentEditable)) {{
            return;
        }}
        if (e.code === 'Space' || e.key === ' ' || e.key === 'k' || e.key === 'K') {{
            e.preventDefault();
            e.stopPropagation();
            togglePlay();
        }} else if (e.code === 'ArrowLeft' || e.key === 'ArrowLeft') {{
            var v = document.querySelector('video');
            if (v && !isNaN(v.currentTime)) {{
                e.preventDefault();
                try {{ v.currentTime = Math.max(0, v.currentTime - 5); }} catch(err) {{}}
            }}
        }} else if (e.code === 'ArrowRight' || e.key === 'ArrowRight') {{
            var v = document.querySelector('video');
            if (v && !isNaN(v.currentTime)) {{
                e.preventDefault();
                try {{ v.currentTime = Math.min(v.duration || Infinity, v.currentTime + 5); }} catch(err) {{}}
            }}
        }}
    }}, true);

    window.addEventListener('message', function(ev) {{
        if (!ev.data) return;
        var v = document.querySelector('video');
        if (ev.data.type === 'AS_SEEK_TO') {{
            var target = parseFloat(ev.data.time);
            if (v && !isNaN(target)) {{
                try {{ v.currentTime = target; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_TOGGLE_PLAY') {{
            togglePlay();
        }} else if (ev.data.type === 'AS_PLAY') {{
            triggerHosterPlay();
        }} else if (ev.data.type === 'AS_PAUSE') {{
            triggerHosterPause();
        }} else if (ev.data.type === 'AS_SEEK_RELATIVE') {{
            var delta = parseFloat(ev.data.delta) || 0;
            if (v && !isNaN(v.currentTime)) {{
                try {{ v.currentTime = Math.max(0, Math.min(v.duration || Infinity, v.currentTime + delta)); }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_VOLUME') {{
            var vol = parseFloat(ev.data.volume);
            if (v && !isNaN(vol)) {{
                try {{ v.volume = Math.max(0, Math.min(1, vol)); }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_MUTED') {{
            if (v) {{
                try {{ v.muted = !!ev.data.muted; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_PLAYBACK_RATE') {{
            var rate = parseFloat(ev.data.rate);
            if (v && !isNaN(rate)) {{
                try {{ v.playbackRate = rate; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_REQUEST_PIP') {{
            if (document.pictureInPictureElement) {{
                try {{ document.exitPictureInPicture().catch(function(){{}}); }} catch(err) {{}}
            }} else if (v && v.requestPictureInPicture) {{
                try {{ v.requestPictureInPicture().catch(function(){{}}); }} catch(err) {{}}
            }}
        }}
    }});
}})();
</script>"#,
        resume_seconds,
        episode_num
    )
}

pub fn generate_resume_only_script(resume_seconds: f64, episode_num: u32) -> String {
    format!(
        r#"<meta name="referrer" content="no-referrer">
<script>
(function() {{
    'use strict';
    var resumeTarget = {:.2};
    var asEpNum = {};
    try {{
        if (!resumeTarget || resumeTarget <= 0) {{
            var p = new URLSearchParams(window.location.search);
            resumeTarget = parseFloat(p.get('as_resume') || '0');
        }}
        if (!asEpNum || asEpNum <= 0) {{
            var p2 = new URLSearchParams(window.location.search);
            asEpNum = parseInt(p2.get('as_ep') || '0', 10);
        }}
    }} catch(e) {{}}

    var lastReported = 0;
    var maxTimeSeen = 0;
    function report(v, force) {{
        if (!v || isNaN(v.currentTime)) return;
        var now = Date.now();
        if (v.currentTime > maxTimeSeen) {{
            maxTimeSeen = v.currentTime;
        }}
        
        if (maxTimeSeen > 5 && v.currentTime < maxTimeSeen - 15) {{
            return;
        }}
        if (resumeTarget > 2 && !v._asSeekCompleted && v.currentTime < resumeTarget - 2) {{
            return;
        }}
        if (!force && now - lastReported < 250) return;
        lastReported = now;
        var bEnd = 0;
        try {{
            if (v.buffered && v.buffered.length > 0) {{
                bEnd = v.buffered.end(v.buffered.length - 1);
            }}
        }} catch(e) {{}}
        try {{
            window.parent.postMessage({{
                type: 'AS_PLAYBACK_PROGRESS',
                ep: asEpNum,
                currentTime: v.currentTime,
                duration: v.duration || 0,
                paused: v.paused,
                volume: v.volume,
                muted: v.muted,
                playbackRate: v.playbackRate,
                bufferedEnd: bEnd
            }}, '*');
        }} catch(e) {{}}
    }}

    function hookVideo(v) {{
        if (v._asHooked) return;
        v._asHooked = true;
        v._asSeekCompleted = (resumeTarget <= 2);

        function attemptSeek() {{
            if (v._asSeekCompleted || resumeTarget <= 2) return;
            try {{
                if (v.readyState >= 1 || (v.duration && v.duration > 2) || v.currentTime > 0) {{
                    v.currentTime = resumeTarget;
                    if (Math.abs(v.currentTime - resumeTarget) < 3) {{
                        v._asSeekCompleted = true;
                    }}
                }}
            }} catch(e) {{}}
        }}

        ['loadedmetadata', 'loadeddata', 'canplay', 'canplaythrough', 'play', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, attemptSeek);
        }});

        ['play', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_PLAY_STATE', paused: false }}, '*'); }} catch(e){{}}
            }});
        }});
        ['pause'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_PLAY_STATE', paused: true }}, '*'); }} catch(e){{}}
            }});
        }});
        ['waiting'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_WAITING', waiting: true }}, '*'); }} catch(e){{}}
            }});
        }});
        ['canplay', 'canplaythrough', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_WAITING', waiting: false }}, '*'); }} catch(e){{}}
            }});
        }});
        v.addEventListener('ended', function() {{
            try {{ window.parent.postMessage({{ type: 'AS_ENDED' }}, '*'); }} catch(e){{}}
        }});

        v.addEventListener('seeked', function() {{
            if (resumeTarget > 2 && Math.abs(v.currentTime - resumeTarget) < 4) {{
                v._asSeekCompleted = true;
            }}
            report(v, true);
        }});

        v.addEventListener('timeupdate', function() {{
            if (!v._asSeekCompleted && resumeTarget > 2) {{
                if (v.currentTime < resumeTarget - 2) {{
                    try {{ v.currentTime = resumeTarget; }} catch(e) {{}}
                    return;
                }} else {{
                    v._asSeekCompleted = true;
                }}
            }}
            report(v, false);
        }});

        v.addEventListener('pause', function() {{ report(v, true); }});
        v.addEventListener('ended', function() {{ report(v, true); }});

        window.addEventListener('beforeunload', function() {{ report(v, true); }});
        window.addEventListener('pagehide', function() {{ report(v, true); }});
        document.addEventListener('visibilitychange', function() {{
            if (document.visibilityState === 'hidden') report(v, true);
        }});
    }}

    var videoPoll = setInterval(function() {{
        var list = document.querySelectorAll('video');
        for (var i = 0; i < list.length; i++) hookVideo(list[i]);
    }}, 300);
    setTimeout(function() {{ clearInterval(videoPoll); }}, 60000);

    function triggerHosterPlay() {{
        try {{
            if (window.player && typeof window.player.play === 'function') {{
                window.player.play();
            }}
        }} catch(e) {{}}
        try {{
            if (window.jwplayer && typeof window.jwplayer === 'function') {{
                var jwp = window.jwplayer();
                if (jwp && typeof jwp.play === 'function') jwp.play();
            }}
        }} catch(e) {{}}
        try {{
            var playBtns = document.querySelectorAll(
                '.plyr__control--overlaid, button[data-plyr="play"], .play-wrapper, .play-button, .btn-play, .overlay-play, .vjs-big-play-button, .jw-display-icon-display, [aria-label="Play"]'
            );
            for (var b = 0; b < playBtns.length; b++) {{
                playBtns[b].click();
            }}
        }} catch(e) {{}}
        try {{
            var v = document.querySelector('video');
            if (v && v.paused) {{
                var p = v.play();
                if (p && typeof p.catch === 'function') p.catch(function(){{}});
            }}
        }} catch(e) {{}}
    }}

    function triggerHosterPause() {{
        try {{
            var v = document.querySelector('video');
            if (v) v.pause();
        }} catch(e) {{}}
        try {{
            if (window.player && typeof window.player.pause === 'function') {{
                window.player.pause();
            }}
        }} catch(e) {{}}
        try {{
            if (window.jwplayer && typeof window.jwplayer === 'function') {{
                var jwp = window.jwplayer();
                if (jwp && typeof jwp.pause === 'function') jwp.pause();
            }}
        }} catch(e) {{}}
    }}

    function togglePlay() {{
        var v = document.querySelector('video');
        if (v && !v.paused) {{
            triggerHosterPause();
        }} else {{
            triggerHosterPlay();
        }}
    }}

    
    window.addEventListener('keydown', function(e) {{
        var tag = (e.target && e.target.tagName || '').toUpperCase();
        if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target && e.target.isContentEditable)) {{
            return;
        }}
        if (e.code === 'Space' || e.key === ' ' || e.key === 'k' || e.key === 'K') {{
            e.preventDefault();
            e.stopPropagation();
            togglePlay();
        }} else if (e.code === 'ArrowLeft' || e.key === 'ArrowLeft') {{
            var v = document.querySelector('video');
            if (v && !isNaN(v.currentTime)) {{
                e.preventDefault();
                try {{ v.currentTime = Math.max(0, v.currentTime - 5); }} catch(err) {{}}
            }}
        }} else if (e.code === 'ArrowRight' || e.key === 'ArrowRight') {{
            var v = document.querySelector('video');
            if (v && !isNaN(v.currentTime)) {{
                e.preventDefault();
                try {{ v.currentTime = Math.min(v.duration || Infinity, v.currentTime + 5); }} catch(err) {{}}
            }}
        }}
    }}, true);

    window.addEventListener('message', function(ev) {{
        if (!ev.data) return;
        var v = document.querySelector('video');
        if (ev.data.type === 'AS_SEEK_TO') {{
            var target = parseFloat(ev.data.time);
            if (v && !isNaN(target)) {{
                try {{ v.currentTime = target; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_TOGGLE_PLAY') {{
            togglePlay();
        }} else if (ev.data.type === 'AS_PLAY') {{
            triggerHosterPlay();
        }} else if (ev.data.type === 'AS_PAUSE') {{
            triggerHosterPause();
        }} else if (ev.data.type === 'AS_SEEK_RELATIVE') {{
            var delta = parseFloat(ev.data.delta) || 0;
            if (v && !isNaN(v.currentTime)) {{
                try {{ v.currentTime = Math.max(0, Math.min(v.duration || Infinity, v.currentTime + delta)); }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_VOLUME') {{
            var vol = parseFloat(ev.data.volume);
            if (v && !isNaN(vol)) {{
                try {{ v.volume = Math.max(0, Math.min(1, vol)); }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_MUTED') {{
            if (v) {{
                try {{ v.muted = !!ev.data.muted; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_PLAYBACK_RATE') {{
            var rate = parseFloat(ev.data.rate);
            if (v && !isNaN(rate)) {{
                try {{ v.playbackRate = rate; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_REQUEST_PIP') {{
            if (document.pictureInPictureElement) {{
                try {{ document.exitPictureInPicture().catch(function(){{}}); }} catch(err) {{}}
            }} else if (v && v.requestPictureInPicture) {{
                try {{ v.requestPictureInPicture().catch(function(){{}}); }} catch(err) {{}}
            }}
        }}
    }});
}})();
</script>"#,
        resume_seconds,
        episode_num
    )
}
