pub fn get_blocker_script() -> &'static str {
    r#"    try {
        var origCreate = document.createElement;
        document.createElement = function(tag) {
            var el = origCreate.apply(this, arguments);
            if (tag && typeof tag === 'string' && tag.toLowerCase() === 'iframe') {
                try {
                    el.src = 'about:blank';
                    el.style.display = 'none';
                    el.style.width = el.style.height = '0px';
                    el.style.pointerEvents = 'none';
                } catch(e) {}
            }
            return el;
        };
        var origAppend = Node.prototype.appendChild;
        Node.prototype.appendChild = function(c) {
            if (c && c.tagName && c.tagName.toUpperCase() === 'IFRAME') return c;
            return origAppend.apply(this, arguments);
        };
        var origInsert = Node.prototype.insertBefore;
        Node.prototype.insertBefore = function(n, r) {
            if (n && n.tagName && n.tagName.toUpperCase() === 'IFRAME') return n;
            return origInsert.apply(this, arguments);
        };

        var BLOCKED_HOSTS_RE = /(opera\.com|operagx|operasoftware|popunder|popcash|adsterra|monetag|clickadu|propellerads|exoclick|hilltopads|recaptcha|turnstile|propu\.sh|trafficjunky|syndication)/i;
        var scriptDesc = Object.getOwnPropertyDescriptor(HTMLScriptElement.prototype, 'src');
        if (scriptDesc && scriptDesc.set) {
            Object.defineProperty(HTMLScriptElement.prototype, 'src', {
                set: function(val) {
                    if (val && BLOCKED_HOSTS_RE.test(String(val))) {
                        this.type = 'text/plain';
                        return '';
                    }
                    return scriptDesc.set.call(this, val);
                },
                get: function() { return scriptDesc.get.call(this); },
                configurable: true
            });
        }
    } catch(e) {}

    try {
        if (window.frameElement) {
            var origHasAttr = window.frameElement.hasAttribute;
            window.frameElement.hasAttribute = function(name) {
                if (name === 'sandbox') return false;
                return origHasAttr ? origHasAttr.apply(this, arguments) : false;
            };
        }
    } catch(e) {}

    var dummyWindow = {
        closed: false, name: '', opener: null, parent: window, top: window,
        close: function() { this.closed = true; }, focus: function() {}, blur: function() {}, postMessage: function() {},
        document: { write: function() {}, writeln: function() {}, open: function() { return this; }, close: function() {}, createElement: function() { return {}; } },
        location: { href: '', replace: function() {}, assign: function() {} }
    };
    window.open = function() { return dummyWindow; };

    try {
        var origClick = HTMLAnchorElement.prototype.click;
        var BAD_CLICK_RE = /(popunder|adsterra|monetag|opera\.com|update|installer)/i;
        HTMLAnchorElement.prototype.click = function() {
            var h = (this.href || '').toLowerCase();
            var t = (this.target || '').toLowerCase();
            if (t === '_blank' || t === '_new' || BAD_CLICK_RE.test(h)) return;
            return origClick.apply(this, arguments);
        };
    } catch(e) {}

    function Fab() {}
    var noop = function() {};
    Fab.prototype.check = Fab.prototype.clearEvent = Fab.prototype.emitEvent = Fab.prototype.setOption = noop;
    Fab.prototype.on = function(d, fn) { if (!d && typeof fn === 'function') { try { fn(); } catch(e) {} } return this; };
    Fab.prototype.onDetected = function() { return this; };
    Fab.prototype.onNotDetected = function(fn) { if (typeof fn === 'function') { try { fn(); } catch(e) {} } return this; };
    Fab.prototype.options = { set: noop, get: noop };
    var fabInst = new Fab();

    function defProp(prop, val) {
        try { Object.defineProperty(window, prop, { get: function() { return val; }, set: function() {}, configurable: true }); }
        catch(e) { window[prop] = val; }
    }
    ['FuckAdBlock', 'BlockAdBlock', 'SniffAdBlock'].forEach(function(k) { defProp(k, Fab); });
    ['fuckAdBlock', 'blockAdBlock', 'sniffAdBlock'].forEach(function(k) { defProp(k, fabInst); });

    window.canRunAds = true;
    window.isAdBlockActive = false;
    window.adBlockDetected = false;
    window.google_ad_status = 1;
    window.nH7eXzOsG = 858;
    window.adsbygoogle = window.adsbygoogle || [];
    window.adsbygoogle.loaded = true;
    window.adsbygoogle.push = function() {};
    window.adblockDetector = { init: function(o) { if (o && typeof o.notfound === 'function') setTimeout(o.notfound, 10); } };

    window.google = window.google || {};
    if (!window.google.ima) {
        window.google.ima = {
            AdDisplayContainer: function() { this.initialize = this.destroy = noop; },
            AdsLoader: function() {
                var l = {};
                this.addEventListener = function(t, fn) { l[t] = fn; };
                this.removeEventListener = noop;
                this.requestAds = function() {
                    setTimeout(function() {
                        if (l['adsManagerLoaded']) {
                            l['adsManagerLoaded']({
                                getAdsManager: function() {
                                    return { init: noop, start: noop, stop: noop, destroy: noop,
                                        addEventListener: function(t, fn) {
                                            if (t === 'allAdsCompleted' || t === 'contentResumeRequested') setTimeout(fn, 10);
                                        }
                                    };
                                }
                            });
                        }
                    }, 10);
                };
                this.contentComplete = this.destroy = noop;
            },
            AdsRequest: function() {},
            ViewMode: { FULLSCREEN: 'fullscreen', NORMAL: 'normal' }
        };
    }

    window.ga = window.gtag = noop;
    window.dataLayer = window.dataLayer || [];
    window.COMSCORE = { purge: noop, beacon: noop };
    window.apstag = { init: noop, fetchBids: function(a, b) { if (b) b(); } };
    window.alert = window.confirm = window.prompt = noop;

    try {
        if (window.Notification) {
            window.Notification.requestPermission = function() { return Promise.resolve('denied'); };
            Object.defineProperty(window.Notification, 'permission', { get: function() { return 'denied'; }, configurable: true });
        }
    } catch(e) {}

    var BAD_TEXT_RE = /(robot|opera|recommandé|recaptcha|installer|continuer à regarder|software update|mise à jour)/i;
    var PLAYER_CLASSES = ['jwplayer', 'video-js', 'plyr', 'plyr__video-wrapper', 'jw-display', 'jw-overlays', 'jw-media', 'jw-video', 'plyr__control--overlaid', 'jw-preview', 'jw-controls', 'jw-controlbar', 'jw-slider', 'jw-button-color', 'jw-settings-menu', 'jw-captions', 'jw-text-track-cue', 'jw-display-icon-container', 'jw-display-icon-display', 'jw-icon-display', 'jw-svg-icon', 'jw-title', 'vjs-control-bar', 'vjs-tech'];

    function isProtectedPlayerElement(el) {
        if (!el || el === document.body || el === document.documentElement) return true;
        var tag = (el.tagName || '').toUpperCase();
        if (tag === 'VIDEO' || tag === 'AUDIO' || tag === 'SOURCE' || tag === 'TRACK') return true;
        if (BAD_TEXT_RE.test(el.textContent || '')) return false;
        if (el.id === 'vplayer' || el.id === 'player') return true;
        if (el.classList) {
            for (var i = 0; i < PLAYER_CLASSES.length; i++) {
                if (el.classList.contains(PLAYER_CLASSES[i])) return true;
            }
        }
        return false;
    }

    function runOverlayBuster() {
        try {
            var doc = document.documentElement;
            var body = document.body;
            if (!doc || !body) return;
            var vw = Math.min(doc.clientWidth, window.innerWidth);
            var vh = Math.min(doc.clientHeight, window.innerHeight);
            if (vw <= 0 || vh <= 0) return;

            var pts = [[vw / 2, vh / 2], [vw / 2, vh * 0.35], [vw / 2, vh * 0.65], [vw * 0.35, vh / 2], [vw * 0.65, vh / 2]];
            for (var p = 0; p < pts.length; p++) {
                var topEl = document.elementFromPoint(pts[p][0], pts[p][1]);
                while (topEl && topEl !== body && topEl !== doc) {
                    if (isProtectedPlayerElement(topEl)) break;
                    var st = window.getComputedStyle(topEl);
                    var z = parseInt(st.zIndex, 10) || 0;
                    var rect = topEl.getBoundingClientRect();
                    var covers = (rect.width > vw * 0.25 && rect.height > vh * 0.2);
                    var susp = (st.position === 'fixed' || (st.position === 'absolute' && z >= 5)) && covers;
                    if (susp && !topEl.querySelector('video, .jw-controlbar, .plyr') && !topEl.classList.contains('jw-media') && !topEl.classList.contains('plyr')) {
                        topEl.remove();
                        break;
                    }
                    topEl = topEl.parentElement;
                }
            }
        } catch(e) {}
    }

    var FAKE_PROMPT_RE = /(pas un robot|not a robot|recaptcha|confirmez que vous|cliquez sur|click allow|press allow|verify you are human|prouvez que vous|navigateur|software update|out of date|obsolète|opera|installer|continuer à regarder)/i;
    var AD_SELECTORS = [
        '.jw-ad', '.jw-ads', '.jw-ad-container', '.jw-ad-ui', '.jw-plugin-ad', '.jw-vast', '.jw-vpaid', '.ima-ad-container',
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
        'div[class*="push-prompt" i]', 'div[class*="notify-prompt" i]', 'div[id*="cf-turnstile" i]', 'div[class*="cf-turnstile" i]',
        'div[id*="cloudflare" i]', 'div[class*="cloudflare" i]', 'div[id*="browser-update" i]', 'div[class*="browser-update" i]',
        'div[id*="software-update" i]', 'div[class*="software-update" i]', 'div[class*="download-dialog" i]', 'div[id*="download-dialog" i]',
        'div[class*="browser-upgrade" i]'
    ].join(',');

    var RESIDUES_SELECTORS = [
        '.jw-pause-overlay', '.jw-related', '.jw-dock', '.jw-dock-btn', '.jw-nextup',
        'a.jw-download', '.jw-svg-icon-download', 'a[download]', 'a[href*="download" i]', 'a[href*="/dl" i]',
        'button[class*="download" i]', 'button[id*="download" i]', '.vidmoly-download',
        '[class*="watermark" i]', '[id*="watermark" i]', '.voe-title', '.voe-watermark'
    ].join(',');

    function killFakePopupsAndBanners() {
        try {
            var overlays = document.querySelectorAll('[class*="popup" i], [class*="modal" i], [class*="overlay" i], [class*="banner" i], [class*="alert" i], [id*="popup" i], [id*="modal" i], [id*="overlay" i], [id*="banner" i], [id*="alert" i], [role="dialog"], [role="alertdialog"], .sweet-alert, .sweet-overlay');
            for (var j = 0; j < overlays.length; j++) {
                var item = overlays[j];
                var txt = (item.textContent || '').trim();
                if (txt.length >= 3 && txt.length <= 2000 && FAKE_PROMPT_RE.test(txt)) {
                    var box = item;
                    while (box.parentElement && box.parentElement !== document.body && box.parentElement !== document.documentElement && box.parentElement.id !== 'vplayer' && box.parentElement.id !== 'player' && !box.parentElement.classList.contains('jwplayer') && !box.parentElement.classList.contains('video-js')) {
                        box = box.parentElement;
                    }
                    var pNode = box.parentElement;
                    try { box.remove(); } catch(e) {}
                    if (pNode && pNode !== document.body && pNode !== document.documentElement && pNode.id !== 'vplayer' && pNode.id !== 'player') {
                        var ps = window.getComputedStyle(pNode);
                        if ((ps.position === 'fixed' || ps.position === 'absolute') && !pNode.querySelector('video, .jw-controlbar')) {
                            try { pNode.remove(); } catch(e) {}
                        }
                    }
                }
            }

            var cands = document.querySelectorAll(AD_SELECTORS);
            for (var c = 0; c < cands.length; c++) {
                if (!isProtectedPlayerElement(cands[c])) try { cands[c].remove(); } catch(e) {}
            }

            var iframes = document.querySelectorAll('iframe');
            for (var f = 0; f < iframes.length; f++) {
                try { iframes[f].remove(); } catch(e) {}
            }

            var res = document.querySelectorAll(RESIDUES_SELECTORS);
            for (var r = 0; r < res.length; r++) {
                if (!isProtectedPlayerElement(res[r])) try { res[r].remove(); } catch(e) {}
            }
        } catch(e) {}
    }

    try {
        var mutTimer = null;
        var obs = new MutationObserver(function() {
            if (mutTimer) return;
            mutTimer = setTimeout(function() { mutTimer = null; killFakePopupsAndBanners(); }, 500);
        });
        if (document.documentElement) obs.observe(document.documentElement, { childList: true, subtree: true });
    } catch(e) {}

    var cleanCount = 0;
    var cleanInterval = setInterval(function() {
        killFakePopupsAndBanners();
        cleanCount++;
        if (cleanCount === 1) runOverlayBuster();
        if (cleanCount > 25) {
            clearInterval(cleanInterval);
            cleanInterval = setInterval(killFakePopupsAndBanners, 3000);
        }
    }, 400);

    document.addEventListener('DOMContentLoaded', function() {
        killFakePopupsAndBanners();
        runOverlayBuster();
    });
"#
}
