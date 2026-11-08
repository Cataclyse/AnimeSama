pub fn get_bridge_script(resume_seconds: f64, episode_num: u32) -> String {
    format!(
        r#"    var resumeTarget = {:.2};
    var asEpNum = {};
    try {{
        if (!resumeTarget || resumeTarget <= 0) {{
            resumeTarget = parseFloat(new URLSearchParams(window.location.search).get('as_resume') || '0');
        }}
        if (!asEpNum || asEpNum <= 0) {{
            asEpNum = parseInt(new URLSearchParams(window.location.search).get('as_ep') || '0', 10);
        }}
    }} catch(e) {{}}

    function applyMediaMetadata(title, artist, artworkUrl) {{
        try {{
            if (title) {{
                document.title = artist ? (artist + ' - ' + title) : title;
                var v = document.querySelector('video');
                if (v) v.title = document.title;
            }}
            if ('mediaSession' in navigator) {{
                navigator.mediaSession.metadata = new MediaMetadata({{
                    title: title || ('Épisode ' + asEpNum),
                    artist: artist || 'Anime-Sama',
                    album: 'Anime-Sama',
                    artwork: artworkUrl ? [{{ src: artworkUrl, sizes: '512x512', type: 'image/jpeg' }}] : []
                }});
            }}
        }} catch(e) {{}}
    }}

    var lastReported = 0;
    function report(v, force) {{
        if (!v || isNaN(v.currentTime)) return;
        var now = Date.now();
        if (resumeTarget > 2 && !v._asSeekCompleted && v.currentTime < resumeTarget - 2) return;
        if (!force && now - lastReported < 250) return;
        lastReported = now;
        var bEnd = 0;
        try {{
            if (v.buffered && v.buffered.length > 0) bEnd = v.buffered.end(v.buffered.length - 1);
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

    function notify(type, extra) {{
        try {{
            var msg = Object.assign({{ type: type }}, extra || {{}});
            window.parent.postMessage(msg, '*');
        }} catch(e) {{}}
    }}

    function withHoster(fn) {{
        try {{
            if (typeof window.jwplayer === 'function') {{
                var j = window.jwplayer();
                if (j) fn(j, 'jw');
            }}
        }} catch(e) {{}}
        try {{
            if (window.player) fn(window.player, 'player');
        }} catch(e) {{}}
    }}

    function triggerHosterPlay() {{
        withHoster(function(p) {{
            if (typeof p.play === 'function') {{
                var res = p.play();
                if (res && typeof res.catch === 'function') res.catch(function() {{}});
            }}
        }});
        try {{
            var btns = document.querySelectorAll('button.plyr__control--overlaid, button[data-plyr="play"], button.vjs-big-play-button, button.play-button, button.btn-play, .jw-display-icon-display, .play_button, #play, .play-wrapper, .play-btn, .vjs-play-control');
            for (var b = 0; b < btns.length; b++) {{
                try {{ btns[b].click(); }} catch(e) {{}}
            }}
        }} catch(e) {{}}
        try {{
            var v = document.querySelector('video');
            if (v && v.paused) {{
                var p = v.play();
                if (p && typeof p.catch === 'function') {{
                    p.catch(function() {{
                        v.muted = true;
                        var p2 = v.play();
                        if (p2 && typeof p2.catch === 'function') p2.catch(function() {{}});
                        setTimeout(function() {{ v.muted = false; }}, 300);
                    }});
                }}
            }}
        }} catch(e) {{}}
    }}

    function triggerHosterPause() {{
        try {{
            var v = document.querySelector('video');
            if (v && !v.paused) v.pause();
        }} catch(e) {{}}
        withHoster(function(p) {{
            if (typeof p.pause === 'function') p.pause();
        }});
    }}

    function togglePlay() {{
        var v = document.querySelector('video');
        if (v && !v.paused) triggerHosterPause();
        else triggerHosterPlay();
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
                    if (Math.abs(v.currentTime - resumeTarget) < 3) v._asSeekCompleted = true;
                }}
            }} catch(e) {{}}
        }}

        ['loadedmetadata', 'loadeddata', 'canplay', 'canplaythrough', 'play', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, attemptSeek, {{ passive: true }});
        }});
        ['loadedmetadata', 'canplay', 'loadeddata'].forEach(function(evt) {{
            v.addEventListener(evt, triggerHosterPlay, {{ passive: true }});
        }});

        v.addEventListener('play', function() {{ notify('AS_PLAY_STATE', {{ paused: false }}); }}, {{ passive: true }});
        v.addEventListener('pause', function() {{ notify('AS_PLAY_STATE', {{ paused: true }}); report(v, true); }}, {{ passive: true }});
        v.addEventListener('waiting', function() {{ notify('AS_WAITING', {{ waiting: true }}); }}, {{ passive: true }});
        ['canplay', 'canplaythrough', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{ notify('AS_WAITING', {{ waiting: false }}); }}, {{ passive: true }});
        }});
        v.addEventListener('ended', function() {{ notify('AS_ENDED'); report(v, true); }}, {{ passive: true }});

        v.addEventListener('seeked', function() {{
            if (resumeTarget > 2 && Math.abs(v.currentTime - resumeTarget) < 4) v._asSeekCompleted = true;
            report(v, true);
        }}, {{ passive: true }});

        v.addEventListener('timeupdate', function() {{
            if (!v._asSeekCompleted && resumeTarget > 2) {{
                if (v.currentTime < resumeTarget - 2) {{
                    try {{ v.currentTime = resumeTarget; }} catch(e) {{}}
                    return;
                }}
                v._asSeekCompleted = true;
            }}
            report(v, false);
        }}, {{ passive: true }});

        window.addEventListener('beforeunload', function() {{ report(v, true); }});
        window.addEventListener('pagehide', function() {{ report(v, true); }});
        document.addEventListener('visibilitychange', function() {{
            if (document.visibilityState === 'hidden') report(v, true);
        }});
    }}

    var videoPoll = setInterval(function() {{
        var list = document.querySelectorAll('video');
        if (list.length > 0) {{
            for (var i = 0; i < list.length; i++) hookVideo(list[i]);
        }}
    }}, 300);
    setTimeout(function() {{ clearInterval(videoPoll); }}, 15000);

    var autoPlayTries = 0;
    var autoPlayTimer = setInterval(function() {{
        autoPlayTries++;
        var v = document.querySelector('video');
        if (v) {{
            hookVideo(v);
            if (!v.paused || autoPlayTries > 30) {{
                clearInterval(autoPlayTimer);
                return;
            }}
        }}
        triggerHosterPlay();
    }}, 200);
    setTimeout(function() {{ clearInterval(autoPlayTimer); }}, 10000);

    function setupOneClickPlay() {{
        var userStarted = false;
        document.addEventListener('click', function(e) {{
            if (userStarted) return;
            var t = e.target;
            if (t && t.closest && t.closest('.jw-controlbar, .jw-settings-menu, .jw-button-color, button, input')) return;
            try {{ triggerHosterPlay(); userStarted = true; }} catch(e) {{}}
        }}, {{ capture: true }});
        window.addEventListener('playing', function() {{ userStarted = true; }}, true);
    }}
    setupOneClickPlay();

    window.addEventListener('keydown', function(e) {{
        var tag = (e.target && e.target.tagName || '').toUpperCase();
        if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target && e.target.isContentEditable)) return;
        var v = document.querySelector('video');
        if (e.code === 'Space' || e.key === ' ' || e.key === 'k' || e.key === 'K') {{
            e.preventDefault(); e.stopPropagation(); togglePlay();
        }} else if (v && !isNaN(v.currentTime)) {{
            if (e.code === 'ArrowLeft' || e.key === 'ArrowLeft') {{
                e.preventDefault();
                try {{ v.currentTime = Math.max(0, v.currentTime - 5); }} catch(e) {{}}
            }} else if (e.code === 'ArrowRight' || e.key === 'ArrowRight') {{
                e.preventDefault();
                try {{ v.currentTime = Math.min(v.duration || Infinity, v.currentTime + 5); }} catch(e) {{}}
            }}
        }}
    }}, true);

    window.addEventListener('message', function(ev) {{
        if (!ev.data) return;
        var v = document.querySelector('video');
        var d = ev.data;
        if (d.type === 'AS_SEEK_TO') {{
            var target = parseFloat(d.time);
            if (!isNaN(target)) {{
                withHoster(function(p) {{
                    if (typeof p.seek === 'function') p.seek(target);
                    else if (typeof p.currentTime !== 'undefined') p.currentTime = target;
                }});
                if (v) try {{ v.currentTime = target; }} catch(e) {{}}
            }}
        }} else if (d.type === 'AS_TOGGLE_PLAY') {{
            togglePlay();
        }} else if (d.type === 'AS_PLAY') {{
            triggerHosterPlay();
        }} else if (d.type === 'AS_PAUSE') {{
            triggerHosterPause();
        }} else if (d.type === 'AS_SEEK_RELATIVE') {{
            var delta = parseFloat(d.delta) || 0;
            if (v && !isNaN(v.currentTime)) {{
                var newT = Math.max(0, Math.min(v.duration || Infinity, v.currentTime + delta));
                withHoster(function(p) {{
                    if (typeof p.seek === 'function') p.seek(newT);
                    else if (typeof p.currentTime !== 'undefined') p.currentTime = newT;
                }});
                try {{ v.currentTime = newT; }} catch(e) {{}}
            }}
        }} else if (d.type === 'AS_SET_VOLUME') {{
            var vol = parseFloat(d.volume);
            if (!isNaN(vol)) {{
                withHoster(function(p) {{
                    if (typeof p.setVolume === 'function') p.setVolume(Math.round(vol * 100));
                    else if (typeof p.volume !== 'undefined') p.volume = vol;
                }});
                if (v) try {{ v.volume = Math.max(0, Math.min(1, vol)); }} catch(e) {{}}
            }}
        }} else if (d.type === 'AS_SET_MUTED') {{
            var isMuted = !!d.muted;
            withHoster(function(p) {{
                if (typeof p.setMute === 'function') p.setMute(isMuted);
                else if (typeof p.muted !== 'undefined') p.muted = isMuted;
            }});
            if (v) try {{ v.muted = isMuted; }} catch(e) {{}}
        }} else if (d.type === 'AS_SET_PLAYBACK_RATE') {{
            var rate = parseFloat(d.rate);
            if (v && !isNaN(rate)) try {{ v.playbackRate = rate; }} catch(e) {{}}
        }} else if (d.type === 'AS_SET_METADATA') {{
            applyMediaMetadata(d.epTitle, d.animeTitle, d.poster);
        }} else if (d.type === 'AS_REQUEST_PIP') {{
            if (document.pictureInPictureElement) {{
                try {{ document.exitPictureInPicture().catch(function() {{}}); }} catch(e) {{}}
            }} else if (v && v.requestPictureInPicture) {{
                try {{
                    applyMediaMetadata(d.epTitle, d.animeTitle, d.poster);
                    v.requestPictureInPicture().catch(function() {{}});
                }} catch(e) {{}}
            }}
        }}
    }});"#,
        resume_seconds,
        episode_num
    )
}
