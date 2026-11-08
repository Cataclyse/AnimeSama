pub fn get_blocker_script() -> &'static str {
    r#"    try {
        var origCreateElement = document.createElement;
        document.createElement = function(tagName) {
            var el = origCreateElement.apply(this, arguments);
            if (tagName && typeof tagName === 'string') {
                var t = tagName.toLowerCase();
                if (t === 'iframe') {
                    try {
                        el.src = 'about:blank';
                        el.style.display = 'none';
                        el.style.width = '0px';
                        el.style.height = '0px';
                        el.style.pointerEvents = 'none';
                    } catch(err) {}
                }
            }
            return el;
        };
        var origAppendChild = Node.prototype.appendChild;
        Node.prototype.appendChild = function(child) {
            if (child && child.tagName && child.tagName.toUpperCase() === 'IFRAME') {
                return child;
            }
            return origAppendChild.apply(this, arguments);
        };
        var origInsertBefore = Node.prototype.insertBefore;
        Node.prototype.insertBefore = function(newNode, referenceNode) {
            if (newNode && newNode.tagName && newNode.tagName.toUpperCase() === 'IFRAME') {
                return newNode;
            }
            return origInsertBefore.apply(this, arguments);
        };

        var isBlockedScriptUrl = function(u) {
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
        };

        var origScriptSrcDesc = Object.getOwnPropertyDescriptor(HTMLScriptElement.prototype, 'src');
        if (origScriptSrcDesc && origScriptSrcDesc.set) {
            Object.defineProperty(HTMLScriptElement.prototype, 'src', {
                set: function(val) {
                    if (isBlockedScriptUrl(val)) {
                        this.type = 'text/plain';
                        return '';
                    }
                    return origScriptSrcDesc.set.call(this, val);
                },
                get: function() {
                    return origScriptSrcDesc.get.call(this);
                },
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
        closed: false,
        name: '',
        opener: null,
        parent: window,
        top: window,
        close: function() { this.closed = true; },
        focus: function() {},
        blur: function() {},
        postMessage: function() {},
        document: {
            write: function() {},
            writeln: function() {},
            open: function() { return this; },
            close: function() {},
            createElement: function() { return {}; }
        },
        location: {
            href: '',
            replace: function() {},
            assign: function() {}
        }
    };

    var origOpen = window.open;
    window.open = function(url, target, features) {
        console.warn('[AdShield] Bloqué popup/popunder uBlock:', url);
        return dummyWindow;
    };

    try {
        var origAnchorClick = HTMLAnchorElement.prototype.click;
        HTMLAnchorElement.prototype.click = function() {
            var href = (this.href || '').toLowerCase();
            var target = (this.target || '').toLowerCase();
            if (target === '_blank' || target === '_new' ||
                href.indexOf('popunder') !== -1 ||
                href.indexOf('adsterra') !== -1 ||
                href.indexOf('monetag') !== -1 ||
                href.indexOf('opera.com') !== -1 ||
                href.indexOf('update') !== -1 ||
                href.indexOf('installer') !== -1) {
                console.warn('[AdShield] Bloqué clic programmatique sur lien externe/pub:', href);
                return;
            }
            return origAnchorClick.apply(this, arguments);
        };
    } catch(e) {}

    function makeFabClass() {
        function Fab() {}
        var noop = function() {};
        Fab.prototype.check = noop;
        Fab.prototype.clearEvent = noop;
        Fab.prototype.emitEvent = noop;
        Fab.prototype.on = function(detected, fn) {
            if (!detected && typeof fn === 'function') {
                try { fn(); } catch(err) {}
            }
            return this;
        };
        Fab.prototype.onDetected = function() { return this; };
        Fab.prototype.onNotDetected = function(fn) {
            if (typeof fn === 'function') {
                try { fn(); } catch(err) {}
            }
            return this;
        };
        Fab.prototype.setOption = noop;
        Fab.prototype.options = { set: noop, get: noop };
        return Fab;
    }

    var FabClass = makeFabClass();
    var fabInstance = new FabClass();

    function defProp(prop, val) {
        try {
            Object.defineProperty(window, prop, {
                get: function() { return val; },
                set: function() {},
                configurable: true
            });
        } catch(e) {
            window[prop] = val;
        }
    }

    ['FuckAdBlock', 'BlockAdBlock', 'SniffAdBlock'].forEach(function(k) { defProp(k, FabClass); });
    ['fuckAdBlock', 'blockAdBlock', 'sniffAdBlock'].forEach(function(k) { defProp(k, fabInstance); });

    window.canRunAds = true;
    window.isAdBlockActive = false;
    window.adBlockDetected = false;
    window.google_ad_status = 1;
    window.nH7eXzOsG = 858;

    window.adsbygoogle = window.adsbygoogle || [];
    window.adsbygoogle.loaded = true;
    window.adsbygoogle.push = function() {};

    window.adblockDetector = {
        init: function(opts) {
            if (opts && typeof opts.notfound === 'function') {
                setTimeout(opts.notfound, 10);
            }
        }
    };

    window.google = window.google || {};
    if (!window.google.ima) {
        window.google.ima = {
            AdDisplayContainer: function() { this.initialize = function() {}; this.destroy = function() {}; },
            AdsLoader: function() {
                var listeners = {};
                this.addEventListener = function(t, fn) { listeners[t] = fn; };
                this.removeEventListener = function() {};
                this.requestAds = function() {
                    setTimeout(function() {
                        if (listeners['adsManagerLoaded']) {
                            listeners['adsManagerLoaded']({
                                getAdsManager: function() {
                                    return {
                                        init: function() {},
                                        start: function() {},
                                        stop: function() {},
                                        destroy: function() {},
                                        addEventListener: function(t, fn) {
                                            if (t === 'allAdsCompleted' || t === 'contentResumeRequested') {
                                                setTimeout(fn, 10);
                                            }
                                        }
                                    };
                                }
                            });
                        }
                    }, 10);
                };
                this.contentComplete = function() {};
                this.destroy = function() {};
            },
            AdsRequest: function() {},
            ViewMode: { FULLSCREEN: 'fullscreen', NORMAL: 'normal' }
        };
    }

    window.ga = window.gtag = function() {};
    window.dataLayer = window.dataLayer || [];
    window.COMSCORE = { purge: function() {}, beacon: function() {} };
    window.apstag = { init: function() {}, fetchBids: function(a, b) { if (b) b(); } };

    try {
        if (window.Notification) {
            window.Notification.requestPermission = function() {
                console.warn('[AdShield] Bloqué faux prompt de notification');
                return Promise.resolve('denied');
            };
            Object.defineProperty(window.Notification, 'permission', {
                get: function() { return 'denied'; },
                set: function() {},
                configurable: true
            });
        }
    } catch(e) {}

    window.alert = function() {};
    window.confirm = function() { return false; };
    window.prompt = function() { return null; };

    function isProtectedPlayerElement(el) {
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
        ) {
            return false;
        }

        if (el.id === 'vplayer' || el.id === 'player') return true;

        if (el.classList) {
            if (
                el.classList.contains('jwplayer') ||
                el.classList.contains('video-js') ||
                el.classList.contains('plyr') ||
                el.classList.contains('plyr__video-wrapper') ||
                el.classList.contains('jw-display') ||
                el.classList.contains('jw-overlays') ||
                el.classList.contains('jw-media') ||
                el.classList.contains('jw-video') ||
                el.classList.contains('plyr__control--overlaid')
            ) {
                return true;
            }

            if (
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
            ) {
                return true;
            }
        }
        return false;
    }

    function runOverlayBuster() {
        try {
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

            for (var p = 0; p < testPoints.length; p++) {
                var pt = testPoints[p];
                var topEl = document.elementFromPoint(pt[0], pt[1]);
                while (topEl && topEl !== bodyEl && topEl !== docEl) {
                    if (isProtectedPlayerElement(topEl)) {
                        break;
                    }
                    var style = window.getComputedStyle(topEl);
                    var pos = style.position;
                    var z = parseInt(style.zIndex, 10) || 0;
                    var rect = topEl.getBoundingClientRect();

                    var coversArea = (rect.width > vw * 0.25 && rect.height > vh * 0.2);
                    var isSuspicious = (pos === 'fixed' || (pos === 'absolute' && z >= 5)) && coversArea;

                    if (isSuspicious) {
                        if (!topEl.querySelector('video') && !topEl.querySelector('.jw-controlbar') && !topEl.classList.contains('jw-media') && !topEl.querySelector('.plyr') && !topEl.classList.contains('plyr')) {
                            topEl.remove();
                            break;
                        }
                    }
                    topEl = topEl.parentElement;
                }
            }
        } catch(err) {}
    }

    function killFakePopupsAndBanners() {
        try {
            var overlays = document.querySelectorAll(
                '[class*="popup" i], [class*="modal" i], [class*="overlay" i], [class*="banner" i], [class*="alert" i], ' +
                '[id*="popup" i], [id*="modal" i], [id*="overlay" i], [id*="banner" i], [id*="alert" i], ' +
                '[role="dialog"], [role="alertdialog"], .sweet-alert, .sweet-overlay'
            );
            for (var j = 0; j < overlays.length; j++) {
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

                if (isFakeRobot || isFakeOperaOrUpdate) {
                    var topBox = item;
                    while (
                        topBox.parentElement &&
                        topBox.parentElement !== document.body &&
                        topBox.parentElement !== document.documentElement &&
                        topBox.parentElement.id !== 'vplayer' &&
                        topBox.parentElement.id !== 'player' &&
                        !topBox.parentElement.classList.contains('jwplayer') &&
                        !topBox.parentElement.classList.contains('video-js')
                    ) {
                        topBox = topBox.parentElement;
                    }
                    var parentNode = topBox.parentElement;
                    try { topBox.remove(); } catch(e) {}
                    if (parentNode && parentNode !== document.body && parentNode !== document.documentElement && parentNode.id !== 'vplayer' && parentNode.id !== 'player') {
                        var pStyle = window.getComputedStyle(parentNode);
                        if (pStyle.position === 'fixed' || pStyle.position === 'absolute') {
                            if (!parentNode.querySelector('video') && !parentNode.querySelector('.jw-controlbar')) {
                                try { parentNode.remove(); } catch(e) {}
                            }
                        }
                    }
                    var masks = document.querySelectorAll('[class*="backdrop" i], [class*="modal-mask" i], [class*="overlay-dark" i]');
                    for (var m = 0; m < masks.length; m++) {
                        if (!isProtectedPlayerElement(masks[m])) {
                            try { masks[m].remove(); } catch(e) {}
                        }
                    }
                }
            }

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
            for (var i = 0; i < candidates.length; i++) {
                var cand = candidates[i];
                if (isProtectedPlayerElement(cand)) continue;
                try { cand.remove(); } catch(e) {}
            }

            var badLinks = document.querySelectorAll('a[href*="opera" i], a[href*="operagx" i], a[href*="update" i], a[href*="install" i], button[onclick*="opera" i]');
            for (var k = 0; k < badLinks.length; k++) {
                var bad = badLinks[k];
                if (isProtectedPlayerElement(bad)) continue;
                try { bad.remove(); } catch(e) {}
            }

            var adIframes = document.querySelectorAll('iframe');
            for (var f = 0; f < adIframes.length; f++) {
                try {
                    adIframes[f].remove();
                } catch(e) {}
            }

            var residues = document.querySelectorAll(
                '.jw-pause-overlay, .jw-related, .jw-dock, .jw-dock-btn, .jw-nextup, ' +
                'a.jw-download, .jw-svg-icon-download, a[download], a[href*="download" i], a[href*="/dl" i], ' +
                'button[class*="download" i], button[id*="download" i], .vidmoly-download, ' +
                '[class*="watermark" i], [id*="watermark" i], .voe-title, .voe-watermark'
            );
            for (var r = 0; r < residues.length; r++) {
                if (isProtectedPlayerElement(residues[r])) continue;
                try { residues[r].remove(); } catch(e) {}
            }
        } catch(e) {}
    }

    try {
        var mutTimer = null;
        var observer = new MutationObserver(function() {
            if (mutTimer) return;
            mutTimer = setTimeout(function() {
                mutTimer = null;
                killFakePopupsAndBanners();
            }, 500);
        });
        if (document.documentElement) {
            observer.observe(document.documentElement, { childList: true, subtree: true });
        }
    } catch(e) {}

    var cleanupCount = 0;
    var cleanupInterval = setInterval(function() {
        killFakePopupsAndBanners();
        cleanupCount++;
        if (cleanupCount === 1) {
            runOverlayBuster();
        }
        if (cleanupCount > 25) {
            clearInterval(cleanupInterval);
            cleanupInterval = setInterval(killFakePopupsAndBanners, 3000);
        }
    }, 400);

    document.addEventListener('DOMContentLoaded', function() {
        killFakePopupsAndBanners();
        runOverlayBuster();
    });
"#
}
