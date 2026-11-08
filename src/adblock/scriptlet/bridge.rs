pub fn get_bridge_script(resume_seconds: f64, episode_num: u32) -> String {
    format!(
        r#"    var resumeTarget = {:.2};
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
            v.addEventListener(evt, attemptSeek, {{ passive: true }});
        }});

        ['play', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_PLAY_STATE', paused: false }}, '*'); }} catch(e){{}}
            }}, {{ passive: true }});
        }});
        ['pause'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_PLAY_STATE', paused: true }}, '*'); }} catch(e){{}}
            }}, {{ passive: true }});
        }});
        ['waiting'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_WAITING', waiting: true }}, '*'); }} catch(e){{}}
            }}, {{ passive: true }});
        }});
        ['canplay', 'canplaythrough', 'playing'].forEach(function(evt) {{
            v.addEventListener(evt, function() {{
                try {{ window.parent.postMessage({{ type: 'AS_WAITING', waiting: false }}, '*'); }} catch(e){{}}
            }}, {{ passive: true }});
        }});
        v.addEventListener('ended', function() {{
            try {{ window.parent.postMessage({{ type: 'AS_ENDED' }}, '*'); }} catch(e){{}}
        }}, {{ passive: true }});

        v.addEventListener('seeked', function() {{
            if (resumeTarget > 2 && Math.abs(v.currentTime - resumeTarget) < 4) {{
                v._asSeekCompleted = true;
            }}
            report(v, true);
        }}, {{ passive: true }});

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
        }}, {{ passive: true }});

        v.addEventListener('pause', function() {{ report(v, true); }}, {{ passive: true }});
        v.addEventListener('ended', function() {{ report(v, true); }}, {{ passive: true }});

        window.addEventListener('beforeunload', function() {{ report(v, true); }});
        window.addEventListener('pagehide', function() {{ report(v, true); }});
        document.addEventListener('visibilitychange', function() {{
            if (document.visibilityState === 'hidden') report(v, true);
        }});
    }}

    var videoPollCount = 0;
    var videoPoll = setInterval(function() {{
        var list = document.querySelectorAll('video');
        if (list.length > 0) {{
            for (var i = 0; i < list.length; i++) hookVideo(list[i]);
            videoPollCount++;
            if (videoPollCount >= 10) clearInterval(videoPoll);
        }}
    }}, 300);
    setTimeout(function() {{ clearInterval(videoPoll); }}, 15000);

    function triggerHosterPlay() {{
        try {{
            if (window.player && typeof window.player.play === 'function') {{
                var p1 = window.player.play();
                if (p1 && typeof p1.catch === 'function') p1.catch(function(){{}});
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
                'button.plyr__control--overlaid, button[data-plyr="play"], button.vjs-big-play-button, button.play-button, button.btn-play, .jw-display-icon-display'
            );
            for (var b = 0; b < playBtns.length; b++) {{
                try {{ playBtns[b].click(); }} catch(err) {{}}
            }}
        }} catch(e) {{}}
        try {{
            var v = document.querySelector('video');
            if (v && v.paused) {{
                var p2 = v.play();
                if (p2 && typeof p2.catch === 'function') p2.catch(function(){{}});
            }}
        }} catch(e) {{}}
    }}

    function triggerHosterPause() {{
        try {{
            var v = document.querySelector('video');
            if (v && !v.paused) v.pause();
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

    function setupOneClickPlay() {{
        var userStarted = false;
        function tryStart(e) {{
            if (userStarted) return;
            var t = e.target;
            if (t && t.closest && t.closest('.jw-controlbar, .jw-settings-menu, .jw-button-color, button, input')) {{
                return;
            }}
            try {{
                triggerHosterPlay();
                userStarted = true;
            }} catch(err) {{}}
        }}
        document.addEventListener('click', tryStart, {{ capture: true }});
        window.addEventListener('playing', function() {{ userStarted = true; }}, true);
    }}
    setupOneClickPlay();

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
            if (!isNaN(target)) {{
                try {{
                    if (window.jwplayer && typeof window.jwplayer === 'function') {{
                        var jwp = window.jwplayer();
                        if (jwp && typeof jwp.seek === 'function') jwp.seek(target);
                    }}
                }} catch(e) {{}}
                try {{
                    if (window.player && typeof window.player.currentTime !== 'undefined') {{
                        window.player.currentTime = target;
                    }}
                }} catch(e) {{}}
                if (v) {{
                    try {{ v.currentTime = target; }} catch(err) {{}}
                }}
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
                var newT = Math.max(0, Math.min(v.duration || Infinity, v.currentTime + delta));
                try {{
                    if (window.jwplayer && typeof window.jwplayer === 'function') {{
                        var jwp2 = window.jwplayer();
                        if (jwp2 && typeof jwp2.seek === 'function') jwp2.seek(newT);
                    }}
                }} catch(e) {{}}
                try {{
                    if (window.player && typeof window.player.currentTime !== 'undefined') {{
                        window.player.currentTime = newT;
                    }}
                }} catch(e) {{}}
                try {{ v.currentTime = newT; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_VOLUME') {{
            var vol = parseFloat(ev.data.volume);
            if (!isNaN(vol)) {{
                try {{
                    if (window.jwplayer && typeof window.jwplayer === 'function') {{
                        var jwpV = window.jwplayer();
                        if (jwpV && typeof jwpV.setVolume === 'function') jwpV.setVolume(Math.round(vol * 100));
                    }}
                }} catch(e) {{}}
                try {{
                    if (window.player && typeof window.player.volume !== 'undefined') {{
                        window.player.volume = vol;
                    }}
                }} catch(e) {{}}
                if (v) {{
                    try {{ v.volume = Math.max(0, Math.min(1, vol)); }} catch(err) {{}}
                }}
            }}
        }} else if (ev.data.type === 'AS_SET_MUTED') {{
            var isMuted = !!ev.data.muted;
            try {{
                if (window.jwplayer && typeof window.jwplayer === 'function') {{
                    var jwpM = window.jwplayer();
                    if (jwpM && typeof jwpM.setMute === 'function') jwpM.setMute(isMuted);
                }}
            }} catch(e) {{}}
            try {{
                if (window.player && typeof window.player.muted !== 'undefined') {{
                    window.player.muted = isMuted;
                }}
            }} catch(e) {{}}
            if (v) {{
                try {{ v.muted = isMuted; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_PLAYBACK_RATE') {{
            var rate = parseFloat(ev.data.rate);
            if (v && !isNaN(rate)) {{
                try {{ v.playbackRate = rate; }} catch(err) {{}}
            }}
        }} else if (ev.data.type === 'AS_SET_METADATA') {{
            applyMediaMetadata(ev.data.epTitle, ev.data.animeTitle, ev.data.poster);
        }} else if (ev.data.type === 'AS_REQUEST_PIP') {{
            if (document.pictureInPictureElement) {{
                try {{ document.exitPictureInPicture().catch(function(){{}}); }} catch(err) {{}}
            }} else if (v && v.requestPictureInPicture) {{
                try {{
                    applyMediaMetadata(ev.data.epTitle, ev.data.animeTitle, ev.data.poster);
                    v.requestPictureInPicture().catch(function(){{}});
                }} catch(err) {{}}
            }}
        }}
    }});"#,
        resume_seconds,
        episode_num
    )
}
