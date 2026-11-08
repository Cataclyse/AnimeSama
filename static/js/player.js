let playerState = {
    currentTime: 0,
    duration: 0,
    bufferedEnd: 0,
    paused: true,
    volume: 1,
    muted: false,
    playbackRate: 1,
    filter: 'normal',
    autoFilterInterval: null,
    autoFilterLastLum: null,
    timeDisplayMode: 'elapsed',
    idleTimer: null,
    isDraggingScrubber: false,
    nextPromptActive: false,
    nextCountdownTimer: null,
    pollInterval: null,
    clickTimeout: null
};

const VIDEO_FILTERS = {
    normal: 'none',
    auto: 'contrast(1.15) saturate(1.4) brightness(1.04)',
    hdr: 'contrast(1.18) saturate(1.45) brightness(1.06)',
    vivid: 'saturate(1.75) contrast(1.12) brightness(1.04)',
    cinema: 'contrast(1.15) brightness(0.96) saturate(1.1) sepia(0.08)',
    night: 'brightness(0.82) contrast(1.05) saturate(0.9)'
};

function formatPlayerTime(seconds) {
    if (!seconds || isNaN(seconds) || seconds < 0) return '00:00';
    const s = Math.floor(seconds);
    const m = Math.floor(s / 60);
    const h = Math.floor(m / 60);
    const remM = m % 60;
    const remS = s % 60;
    if (h > 0) {
        return `${h}:${remM < 10 ? '0' : ''}${remM}:${remS < 10 ? '0' : ''}${remS}`;
    }
    return `${remM < 10 ? '0' : ''}${remM}:${remS < 10 ? '0' : ''}${remS}`;
}

function getIframeVideo() {
    try {
        const frame = document.getElementById('videoIframe');
        if (!frame) return null;
        const doc = frame.contentDocument || (frame.contentWindow ? frame.contentWindow.document : null);
        if (!doc) return null;
        return doc.querySelector('video');
    } catch (e) {
        return null;
    }
}

function sendFrameMessage(msg) {
    try {
        const frame = document.getElementById('videoIframe');
        if (frame && frame.contentWindow) {
            frame.contentWindow.postMessage(msg, '*');
        }
    } catch (e) {}
}

function updateAppTitle(isPlaying) {
    if (isPlaying && State.currentAnime) {
        const title = State.currentAnime.title || State.currentAnime.slug;
        const ep = (State.currentEpIndex !== undefined) ? (State.currentEpIndex + 1) : 1;
        const lang = (State.selectedLanguage || 'vostfr').toUpperCase();
        document.title = `Anime-Sama — ${title} — Ep ${ep} (${lang})`;
    } else {
        document.title = 'Anime-Sama';
    }
}

async function openEpisodePlayer(slug, seasonPath, episodeIndex, resumeSeconds) {
    try {
        if (!State.currentAnime || State.currentAnime.slug !== slug) {
            try {
                const aRes = await fetch(`/api/anime?slug=${encodeURIComponent(slug)}`);
                if (aRes.ok) {
                    State.currentAnime = await aRes.json();
                }
            } catch (e) {}
        }

        let normalizedSeason = seasonPath;
        if (!normalizedSeason || typeof normalizedSeason !== 'string') {
            normalizedSeason = 'saison1/vostfr';
        } else if (!normalizedSeason.includes('/')) {
            normalizedSeason = `${normalizedSeason}/vostfr`;
        }

        if (!State.currentEpisodes || State.currentEpisodes.slug !== slug || State.currentEpisodes.season !== normalizedSeason) {
            let res = await fetch(`/api/episodes?slug=${slug}&season=${encodeURIComponent(normalizedSeason)}`);
            if (!res.ok) {
                const animeRes = await fetch(`/api/anime?slug=${slug}`);
                if (animeRes.ok) {
                    const animeData = await animeRes.json();
                    if (animeData.seasons && animeData.seasons.length > 0) {
                        let altSeason = animeData.seasons[0].path;
                        if (!altSeason.includes('/')) altSeason = `${altSeason}/vostfr`;
                        normalizedSeason = altSeason;
                        res = await fetch(`/api/episodes?slug=${slug}&season=${encodeURIComponent(normalizedSeason)}`);
                    }
                }
            }
            if (!res.ok) throw new Error('Erreur récupération épisodes');
            State.currentEpisodes = await res.json();
            const players = Object.keys(State.currentEpisodes.players || {});
            State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);
        }

        const players = Object.keys(State.currentEpisodes.players || {});
        if (!State.selectedPlayer || !players.includes(State.selectedPlayer)) {
            State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);
        }

        let urls = State.currentEpisodes.players[State.selectedPlayer] || [];
        let safeIdx = parseInt(episodeIndex, 10);
        if (isNaN(safeIdx) || safeIdx < 0) safeIdx = 0;
        if (urls.length > 0 && safeIdx >= urls.length) {
            safeIdx = urls.length - 1;
        }
        episodeIndex = safeIdx;

        let videoUrl = (State.currentEpisodes.players[State.selectedPlayer] || [])[episodeIndex];
        if (!videoUrl) {
            const fallbackPlayer = players.find(p => (State.currentEpisodes.players[p] || [])[episodeIndex]);
            if (fallbackPlayer) {
                State.selectedPlayer = fallbackPlayer;
                videoUrl = (State.currentEpisodes.players[State.selectedPlayer] || [])[episodeIndex];
            }
        }

        if (!videoUrl) {
            showToast(t('player.link_unavailable', "Lien d'épisode indisponible"));
            return;
        }

        let estimatedDuration = 1440;
        if (resumeSeconds === undefined || resumeSeconds === null) {
            const epProg = (typeof getEpisodeProgress === 'function') ? getEpisodeProgress(slug, episodeIndex + 1, normalizedSeason) : null;
            const animeProg = (typeof getPlaybackProgress === 'function') ? getPlaybackProgress(slug) : null;
            const prog = epProg || ((animeProg && animeProg.episode === episodeIndex + 1) ? animeProg : null);
            if (prog && prog.duration && prog.duration > 60) {
                estimatedDuration = prog.duration;
            }
            if (prog && prog.currentTime > 5) {
                if (prog.duration && prog.duration > 60 && prog.currentTime >= prog.duration - 20) {
                    resumeSeconds = 0;
                } else {
                    resumeSeconds = prog.currentTime;
                }
            } else {
                resumeSeconds = 0;
            }
        } else {
            resumeSeconds = parseFloat(resumeSeconds) || 0;
            const epProg = (typeof getEpisodeProgress === 'function') ? getEpisodeProgress(slug, episodeIndex + 1, normalizedSeason) : null;
            const animeProg = (typeof getPlaybackProgress === 'function') ? getPlaybackProgress(slug) : null;
            const prog = epProg || ((animeProg && animeProg.episode === episodeIndex + 1) ? animeProg : null);
            if (prog && prog.duration && prog.duration > 60) {
                estimatedDuration = prog.duration;
            }
        }

        State.targetResumeTime = resumeSeconds > 5 ? resumeSeconds : 0;
        State.currentEpIndex = episodeIndex;
        State.lastCurrentTime = resumeSeconds || 0;
        playerState.currentTime = resumeSeconds || 0;
        playerState.duration = estimatedDuration;
        playerState.paused = false;
        playerState.nextPromptActive = false;
        playerState.skipData = null;
        playerState.fallbackSkippedAt = null;

        const animeTitle = State.currentAnime ? State.currentAnime.title : slug;
        loadSkipTimes(animeTitle, episodeIndex + 1);

        const titleEl = document.getElementById('playerTitle');
        const epTitleEl = document.getElementById('playerEpisodeTitle');
        const currentLangEl = document.getElementById('netflixCurrentLang');
        const currentServerEl = document.getElementById('netflixCurrentServer');

        if (titleEl) titleEl.textContent = State.currentAnime ? State.currentAnime.title : slug;
        const currentSeasonBase = (State.currentEpisodes.season || 'saison1').split('/')[0];
        const formattedSeason = formatSeasonLabel(currentSeasonBase);
        if (epTitleEl) epTitleEl.textContent = `${formattedSeason} • ${t('modal.episode_single', 'Épisode')} ${episodeIndex + 1}`;

        const currentLang = (State.currentEpisodes.season.split('/')[1] || 'vostfr').toUpperCase();
        if (currentLangEl) currentLangEl.textContent = currentLang;
        if (currentServerEl) currentServerEl.textContent = State.selectedPlayer;
        State.selectedLanguage = currentLang.toLowerCase();
        try {
            localStorage.setItem(`as_lang_${slug}`, State.selectedLanguage);
            localStorage.setItem('as_last_selected_lang', State.selectedLanguage);
        } catch (e) {}

        let proxyUrl = toProxyUrl(videoUrl);
        if (!proxyUrl) {
            showToast(t('player.launch_error', 'Impossible de lancer le lecteur'));
            return;
        }
        const sep = proxyUrl.includes('?') ? '&' : '?';
        const qParams = [];
        if (resumeSeconds > 0) {
            qParams.push(`as_resume=${Math.floor(resumeSeconds)}`);
        }
        qParams.push(`as_ep=${episodeIndex + 1}`);
        proxyUrl += `${sep}${qParams.join('&')}`;

        const iframe = document.getElementById('videoIframe');
        if (iframe) {
            iframe.onload = () => {
                syncPlayerMediaMetadata();
                startPlayerPolling();
                sendFrameMessage({ type: 'AS_PLAY' });
                if (resumeSeconds > 5) {
                    sendFrameMessage({ type: 'AS_SEEK_TO', time: resumeSeconds });
                }
                setTimeout(() => {
                    sendFrameMessage({ type: 'AS_PLAY' });
                    if (State.targetResumeTime > 5) {
                        sendFrameMessage({ type: 'AS_SEEK_TO', time: State.targetResumeTime });
                    }
                }, 800);
                setTimeout(() => {
                    sendFrameMessage({ type: 'AS_PLAY' });
                    if (State.targetResumeTime > 5) {
                        sendFrameMessage({ type: 'AS_SEEK_TO', time: State.targetResumeTime });
                    }
                }, 2000);
            };
            iframe.src = proxyUrl;
        }

        const modal = document.getElementById('playerModal');
        if (modal) modal.classList.add('open');
        document.body.style.overflow = 'hidden';

        let savedFilter = 'normal';
        try {
            savedFilter = localStorage.getItem('as_video_filter') || 'normal';
        } catch (e) {}
        applyVideoFilter(savedFilter);

        try {
            const savedVolStr = localStorage.getItem('as_video_volume');
            if (savedVolStr !== null) {
                const parsedVol = parseFloat(savedVolStr);
                if (!isNaN(parsedVol)) {
                    playerState.volume = Math.max(0, Math.min(1, parsedVol));
                }
            }
            const savedMuted = localStorage.getItem('as_video_muted');
            if (savedMuted !== null) {
                playerState.muted = savedMuted === '1';
            }
            sendFrameMessage({ type: 'AS_SET_VOLUME', volume: playerState.volume });
            sendFrameMessage({ type: 'AS_SET_MUTED', muted: playerState.muted });
            updateVolumeUI(playerState.volume, playerState.muted);
        } catch (e) {}

        const prevBtn = document.getElementById('playerPrevBtn');
        const nextBtn = document.getElementById('playerNextBtn');
        if (prevBtn) prevBtn.disabled = episodeIndex <= 0;
        if (nextBtn) nextBtn.disabled = episodeIndex >= urls.length - 1;

        updateAppTitle(true);
        recordHistory(slug, episodeIndex + 1);
        resetIdleTimer();
        startPlayerPolling();
        updatePlayPauseButton(true);
        syncPlayerMediaMetadata();

        renderScrubber(resumeSeconds, estimatedDuration, 0);
        renderTimeDisplay(resumeSeconds, estimatedDuration);

        const promptEl = document.getElementById('netflixNextPrompt');
        if (promptEl) promptEl.style.display = 'none';

        const skipBtn = document.getElementById('netflixSkipIntroBtn');
        if (skipBtn) skipBtn.style.display = 'none';
    } catch (e) {
        console.error('Erreur lecture vidéo:', e);
        showToast('Impossible de lancer le lecteur');
    }
}

function syncPlayerMediaMetadata() {
    try {
        const animeTitle = State.currentAnime ? State.currentAnime.title : '';
        const epNum = State.currentEpIndex !== undefined ? (State.currentEpIndex + 1) : 1;
        const epTitle = `${t('modal.episode_single', 'Épisode')} ${epNum}`;
        const poster = State.currentAnime && State.currentAnime.poster ? State.currentAnime.poster : '';
        const fullTitle = animeTitle ? `${animeTitle} - ${epTitle}` : epTitle;

        document.title = fullTitle;

        const v = getIframeVideo();
        if (v) {
            try {
                const doc = v.ownerDocument || document;
                if (doc) doc.title = fullTitle;
                v.title = fullTitle;
            } catch (err) {}
        }

        if ('mediaSession' in navigator) {
            navigator.mediaSession.metadata = new MediaMetadata({
                title: epTitle,
                artist: animeTitle || 'Anime-Sama',
                album: 'Anime-Sama',
                artwork: poster ? [{ src: poster, sizes: '512x512', type: 'image/jpeg' }] : []
            });
        }

        sendFrameMessage({
            type: 'AS_SET_METADATA',
            animeTitle,
            epTitle,
            poster
        });
    } catch (e) {}
}

function startPlayerPolling() {
    stopPlayerPolling();
    const bindNativeVideoEvents = () => {
        const v = getIframeVideo();
        if (v && !v._asParentBound) {
            v._asParentBound = true;
            const handler = () => updateFromVideoElement(v);
            v.addEventListener('timeupdate', handler, { passive: true });
            v.addEventListener('progress', handler, { passive: true });
            v.addEventListener('seeking', handler, { passive: true });
            v.addEventListener('seeked', handler, { passive: true });
        }
    };
    bindNativeVideoEvents();
    playerState.pollInterval = setInterval(() => {
        bindNativeVideoEvents();
        const v = getIframeVideo();
        if (v) {
            updateFromVideoElement(v);
        }
    }, 1000);
}

function stopPlayerPolling() {
    if (playerState.pollInterval) {
        clearInterval(playerState.pollInterval);
        playerState.pollInterval = null;
    }
}

function updateFromVideoElement(v) {
    if (!v || isNaN(v.currentTime)) return;
    const dur = v.duration || 0;
    const paused = v.paused;

    if (State.targetResumeTime && State.targetResumeTime > 5) {
        if (v.currentTime < State.targetResumeTime - 2) {
            return;
        } else {
            State.targetResumeTime = 0;
        }
    }

    const ct = v.currentTime;
    let bEnd = 0;
    try {
        if (v.buffered && v.buffered.length > 0) {
            bEnd = v.buffered.end(v.buffered.length - 1);
        }
    } catch (e) {}

    syncPlaybackState(ct, dur, bEnd, paused, v.volume, v.muted);
}

function syncPlaybackState(currentTime, duration, bufferedEnd, paused, volume, muted) {
    playerState.currentTime = currentTime;
    if (duration > 0) playerState.duration = duration;
    playerState.bufferedEnd = bufferedEnd;
    playerState.paused = paused;
    if (volume !== undefined) playerState.volume = volume;
    if (muted !== undefined) playerState.muted = muted;

    State.lastCurrentTime = currentTime;
    State.lastDuration = playerState.duration;

    if (!playerState.isDraggingScrubber) {
        renderScrubber(currentTime, playerState.duration, bufferedEnd);
    }
    renderTimeDisplay(currentTime, playerState.duration);
    updatePlayPauseButton(!paused);
    updateVolumeUI(playerState.volume, playerState.muted);
    checkSkipIntro(currentTime);
    checkNextEpisodePrompt(currentTime, playerState.duration);
}

function renderScrubber(currentTime, duration, bufferedEnd) {
    const playedBar = document.getElementById('netflixProgressPlayed');
    const bufferedBar = document.getElementById('netflixProgressBuffered');

    if (!playedBar || duration <= 0) return;
    const pct = Math.max(0, Math.min(100, (currentTime / duration) * 100));
    playedBar.style.width = `${pct}%`;

    if (bufferedBar && bufferedEnd > 0) {
        const bufPct = Math.max(0, Math.min(100, (bufferedEnd / duration) * 100));
        bufferedBar.style.width = `${bufPct}%`;
    }
}

function renderTimeDisplay(currentTime, duration) {
    const el = document.getElementById('netflixTimeDisplay');
    if (!el) return;

    if (playerState.timeDisplayMode === 'remaining' && duration > 0) {
        const rem = Math.max(0, duration - currentTime);
        el.textContent = `-${formatPlayerTime(rem)}`;
    } else {
        el.textContent = `${formatPlayerTime(currentTime)} / ${formatPlayerTime(duration)}`;
    }
}

function toggleTimeDisplayMode() {
    playerState.timeDisplayMode = playerState.timeDisplayMode === 'elapsed' ? 'remaining' : 'elapsed';
    renderTimeDisplay(playerState.currentTime, playerState.duration);
}

function updatePlayPauseButton(isPlaying) {
    const playIcon = document.querySelector('#netflixPlayBtn .icon-play');
    const pauseIcon = document.querySelector('#netflixPlayBtn .icon-pause');
    if (playIcon && pauseIcon) {
        playIcon.style.display = isPlaying ? 'none' : 'block';
        pauseIcon.style.display = isPlaying ? 'block' : 'none';
    }
}

function togglePlayPause() {
    const v = getIframeVideo();
    let willPlay = false;
    if (v) {
        if (v.paused) {
            v.play().catch(() => {});
            willPlay = true;
        } else {
            v.pause();
            willPlay = false;
        }
    } else {
        willPlay = playerState.paused;
    }

    playerState.paused = !willPlay;
    updatePlayPauseButton(willPlay);

    if (willPlay) {
        sendFrameMessage({ type: 'AS_PLAY' });
    } else {
        sendFrameMessage({ type: 'AS_PAUSE' });
    }
    showCenterPulse(willPlay ? 'play' : 'pause');
    resetIdleTimer();
}

function showCenterPulse(type) {
    const pulse = document.getElementById('netflixCenterPulse');
    const icon = document.getElementById('netflixPulseIcon');
    if (!pulse || !icon) return;

    if (type === 'play') {
        icon.innerHTML = '<svg viewBox="0 0 24 24" width="36" height="36" fill="white"><polygon points="6 4 20 12 6 20 6 4"></polygon></svg>';
    } else {
        icon.innerHTML = '<svg viewBox="0 0 24 24" width="36" height="36" fill="white"><rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect></svg>';
    }

    pulse.classList.remove('show');
    void pulse.offsetWidth;
    pulse.classList.add('show');
    setTimeout(() => {
        pulse.classList.remove('show');
    }, 450);
}

function seekRelative(delta) {
    const v = getIframeVideo();
    const newTime = Math.max(0, Math.min(playerState.duration || Infinity, playerState.currentTime + delta));
    if (v) {
        v.currentTime = newTime;
    } else {
        sendFrameMessage({ type: 'AS_SEEK_RELATIVE', delta });
    }
    playerState.currentTime = newTime;
    renderScrubber(newTime, playerState.duration, playerState.bufferedEnd);
    renderTimeDisplay(newTime, playerState.duration);
    showSeekRipple(delta > 0 ? 'right' : 'left');
    resetIdleTimer();
}

function seekTo(targetTime) {
    const t = Math.max(0, Math.min(playerState.duration || Infinity, targetTime));
    const v = getIframeVideo();
    if (v) {
        v.currentTime = t;
    } else {
        sendFrameMessage({ type: 'AS_SEEK_TO', time: t });
    }
    playerState.currentTime = t;
    renderScrubber(t, playerState.duration, playerState.bufferedEnd);
    renderTimeDisplay(t, playerState.duration);
}

function showSeekRipple(side) {
    const ripple = document.getElementById(side === 'right' ? 'netflixRippleRight' : 'netflixRippleLeft');
    if (!ripple) return;
    if (ripple._hideTimeout) clearTimeout(ripple._hideTimeout);
    ripple.classList.remove('show');
    void ripple.offsetWidth;
    ripple.classList.add('show');
    ripple._hideTimeout = setTimeout(() => {
        ripple.classList.remove('show');
    }, 450);
}

function toggleMute() {
    const v = getIframeVideo();
    const newMuted = !playerState.muted;
    if (v) {
        v.muted = newMuted;
    } else {
        sendFrameMessage({ type: 'AS_SET_MUTED', muted: newMuted });
    }
    playerState.muted = newMuted;
    try {
        localStorage.setItem('as_video_muted', newMuted ? '1' : '0');
    } catch (e) {}
    updateVolumeUI(playerState.volume, newMuted);
    resetIdleTimer();
}

function setVolume(val) {
    const v = getIframeVideo();
    const parsed = parseFloat(val);
    const clamped = isNaN(parsed) ? 1 : Math.max(0, Math.min(1, Math.round(parsed * 100) / 100));
    if (v) {
        v.volume = clamped;
        v.muted = clamped === 0;
    } else {
        sendFrameMessage({ type: 'AS_SET_VOLUME', volume: clamped });
        if (clamped === 0) sendFrameMessage({ type: 'AS_SET_MUTED', muted: true });
    }
    playerState.volume = clamped;
    playerState.muted = clamped === 0;
    try {
        localStorage.setItem('as_video_volume', clamped.toString());
        localStorage.setItem('as_video_muted', playerState.muted ? '1' : '0');
    } catch (e) {}
    updateVolumeUI(clamped, playerState.muted);
    resetIdleTimer();
}

function updateVolumeUI(vol, muted) {
    const highIcon = document.querySelector('#netflixVolBtn .icon-vol-high');
    const muteIcon = document.querySelector('#netflixVolBtn .icon-vol-mute');
    const slider = document.getElementById('netflixVolSlider');

    const isMuted = muted || vol === 0;
    if (highIcon && muteIcon) {
        highIcon.style.display = isMuted ? 'none' : 'block';
        muteIcon.style.display = isMuted ? 'block' : 'none';
    }
    if (slider) {
        const displayVal = isMuted ? 0 : vol;
        slider.value = displayVal;
        slider.style.setProperty('--vol-pct', `${Math.round(displayVal * 100)}%`);
    }
}

function setSpeed(rate) {
    const v = getIframeVideo();
    if (v) {
        v.playbackRate = rate;
    } else {
        sendFrameMessage({ type: 'AS_SET_PLAYBACK_RATE', rate });
    }
    playerState.playbackRate = rate;
    const btn = document.getElementById('netflixSpeedBtn');
    if (btn) btn.textContent = `${rate}x`;

    const menu = document.getElementById('netflixSpeedMenu');
    if (menu) {
        const opts = menu.querySelectorAll('.netflix-speed-opt');
        opts.forEach(o => {
            const num = parseFloat(o.textContent);
            o.classList.toggle('active', num === rate);
        });
        menu.style.display = 'none';
    }
    resetIdleTimer();
}

function toggleSpeedMenu() {
    const filterMenu = document.getElementById('netflixFilterMenu');
    if (filterMenu) filterMenu.style.display = 'none';
    const menu = document.getElementById('netflixSpeedMenu');
    if (!menu) return;
    menu.style.display = menu.style.display === 'none' ? 'flex' : 'none';
    resetIdleTimer();
}

function startDynamicFilter() {
    stopDynamicFilter();
    let sampleCanvas = null;
    let sampleCtx = null;

    playerState.autoFilterInterval = setInterval(() => {
        if (playerState.filter !== 'auto' || playerState.paused) return;
        const v = getIframeVideo();
        const iframe = document.getElementById('videoIframe');
        if (!iframe) return;

        if (v && v.readyState >= 2 && v.videoWidth > 0 && v.videoHeight > 0) {
            try {
                if (!sampleCanvas) {
                    sampleCanvas = document.createElement('canvas');
                    sampleCanvas.width = 16;
                    sampleCanvas.height = 9;
                    sampleCtx = sampleCanvas.getContext('2d', { willReadFrequently: true });
                }
                sampleCtx.drawImage(v, 0, 0, 16, 9);
                const frameData = sampleCtx.getImageData(0, 0, 16, 9).data;
                let totalLum = 0;
                const pixelCount = 16 * 9;
                for (let i = 0; i < frameData.length; i += 4) {
                    totalLum += (frameData[i] * 0.299 + frameData[i + 1] * 0.587 + frameData[i + 2] * 0.114);
                }
                const avgLum = totalLum / pixelCount;
                const prev = playerState.autoFilterLastLum !== null ? playerState.autoFilterLastLum : avgLum;
                const smoothedLum = prev * 0.65 + avgLum * 0.35;
                playerState.autoFilterLastLum = smoothedLum;

                let b = 1.05;
                let c = 1.15;
                let s = 1.35;

                if (smoothedLum < 60) {
                    b = 1.14;
                    c = 1.22;
                    s = 1.25;
                } else if (smoothedLum > 170) {
                    b = 0.96;
                    c = 1.12;
                    s = 1.45;
                } else {
                    b = 1.06;
                    c = 1.18;
                    s = 1.42;
                }
                iframe.style.filter = `contrast(${c.toFixed(2)}) saturate(${s.toFixed(2)}) brightness(${b.toFixed(2)})`;
                return;
            } catch (err) {}
        }

        const t = (playerState.currentTime || 0) * 0.2;
        const b = (1.04 + 0.04 * Math.sin(t)).toFixed(3);
        const c = (1.16 + 0.03 * Math.cos(t * 0.7)).toFixed(3);
        const s = (1.42 + 0.05 * Math.sin(t * 0.5)).toFixed(3);
        iframe.style.filter = `contrast(${c}) saturate(${s}) brightness(${b})`;
    }, 450);
}

function stopDynamicFilter() {
    if (playerState.autoFilterInterval) {
        clearInterval(playerState.autoFilterInterval);
        playerState.autoFilterInterval = null;
    }
    playerState.autoFilterLastLum = null;
}

function applyVideoFilter(filterKey) {
    const key = VIDEO_FILTERS[filterKey] ? filterKey : 'normal';
    playerState.filter = key;
    try {
        localStorage.setItem('as_video_filter', key);
    } catch (e) {}

    const iframe = document.getElementById('videoIframe');

    if (key === 'auto') {
        startDynamicFilter();
    } else {
        stopDynamicFilter();
        if (iframe) {
            iframe.style.filter = VIDEO_FILTERS[key];
        }
    }

    const filterBtn = document.getElementById('netflixFilterBtn');
    if (filterBtn) {
        filterBtn.classList.toggle('active-filter', key !== 'normal');
    }

    const menu = document.getElementById('netflixFilterMenu');
    if (menu) {
        const opts = menu.querySelectorAll('.netflix-filter-opt');
        opts.forEach(o => {
            o.classList.toggle('active', o.getAttribute('data-filter') === key);
        });
    }
}

function setVideoFilter(filterKey) {
    applyVideoFilter(filterKey);
    const menu = document.getElementById('netflixFilterMenu');
    if (menu) menu.style.display = 'none';
    resetIdleTimer();
}

function toggleFilterMenu() {
    const speedMenu = document.getElementById('netflixSpeedMenu');
    if (speedMenu) speedMenu.style.display = 'none';
    const menu = document.getElementById('netflixFilterMenu');
    if (!menu) return;
    menu.style.display = menu.style.display === 'none' ? 'flex' : 'none';
    resetIdleTimer();
}

async function loadSkipTimes(title, episode) {
    try {
        playerState.skipData = null;
        const res = await fetch(`/api/skips?title=${encodeURIComponent(title)}&ep=${episode}`);
        if (res.ok) {
            const data = await res.json();
            if (data && data.found) {
                playerState.skipData = data;
            }
        }
    } catch (e) {
        playerState.skipData = null;
    }
}

function checkSkipIntro(currentTime) {
    const skipBtn = document.getElementById('netflixSkipIntroBtn');
    if (!skipBtn) return;

    if (playerState.skipData && playerState.skipData.found) {
        const op = playerState.skipData.op;
        const ed = playerState.skipData.ed;

        if (op && currentTime >= (op.start - 1.0) && currentTime < (op.end - 1.0)) {
            skipBtn.style.display = 'flex';
            skipBtn.dataset.skipTarget = op.end;
            skipBtn.dataset.skipType = 'op';
            const labelEl = skipBtn.querySelector('[data-i18n]');
            if (labelEl) labelEl.textContent = "Passer l'intro";
            const badgeEl = skipBtn.querySelector('.netflix-skip-intro-badge');
            if (badgeEl) badgeEl.textContent = `${formatPlayerTime(op.start)} → ${formatPlayerTime(op.end)}`;
            return;
        }

        if (ed && currentTime >= (ed.start - 1.0) && currentTime < (ed.end - 1.0)) {
            skipBtn.style.display = 'flex';
            skipBtn.dataset.skipTarget = ed.end;
            skipBtn.dataset.skipType = 'ed';
            const labelEl = skipBtn.querySelector('[data-i18n]');
            if (labelEl) labelEl.textContent = "Passer l'ending";
            const badgeEl = skipBtn.querySelector('.netflix-skip-intro-badge');
            if (badgeEl) badgeEl.textContent = `${formatPlayerTime(ed.start)} → ${formatPlayerTime(ed.end)}`;
            return;
        }

        skipBtn.style.display = 'none';
        return;
    }

    if (playerState.fallbackSkippedAt !== null) {
        if (currentTime < playerState.fallbackSkippedAt - 2) {
            playerState.fallbackSkippedAt = null;
        }
    }

    if (playerState.fallbackSkippedAt === null && currentTime >= 45 && currentTime <= 210) {
        skipBtn.style.display = 'flex';
        skipBtn.dataset.skipTarget = 'relative_85';
        skipBtn.dataset.skipType = 'fallback';
        const labelEl = skipBtn.querySelector('[data-i18n]');
        if (labelEl) labelEl.textContent = "Passer l'intro";
        const badgeEl = skipBtn.querySelector('.netflix-skip-intro-badge');
        if (badgeEl) badgeEl.textContent = '+85s';
    } else {
        skipBtn.style.display = 'none';
    }
}

function skipIntro() {
    const skipBtn = document.getElementById('netflixSkipIntroBtn');
    const target = skipBtn ? skipBtn.dataset.skipTarget : null;
    const type = skipBtn ? skipBtn.dataset.skipType : null;

    if (type === 'fallback') {
        playerState.fallbackSkippedAt = playerState.currentTime;
    }

    if (target && target !== 'relative_85') {
        const num = parseFloat(target);
        if (!isNaN(num) && num > 0) {
            seekTo(num + 0.2);
            showToast(type === 'ed' ? "Générique de fin passé (précision API)" : "Intro passée (précision API)");
        } else {
            seekRelative(85);
            showToast("Intro passée (+85s)");
        }
    } else {
        seekRelative(85);
        showToast("Intro passée (+85s)");
    }

    if (skipBtn) skipBtn.style.display = 'none';
    resetIdleTimer();
}

function checkNextEpisodePrompt(currentTime, duration) {
    const prompt = document.getElementById('netflixNextPrompt');
    if (!prompt || duration <= 60) return;

    if (currentTime >= duration - 40 && !playerState.nextPromptActive) {
        playerState.nextPromptActive = true;
        prompt.style.display = 'flex';

        const nextIdx = State.currentEpIndex + 1;
        const urls = (State.currentEpisodes && State.selectedPlayer)
            ? (State.currentEpisodes.players[State.selectedPlayer] || [])
            : [];
        if (nextIdx >= urls.length) {
            prompt.style.display = 'none';
            return;
        }

        const titleEl = document.getElementById('netflixNextPromptTitle');
        if (titleEl) titleEl.textContent = `${t('modal.episode_single', 'Épisode')} ${nextIdx + 1}`;

        let countdown = 15;
        const timerEl = document.getElementById('netflixNextPromptTimer');
        if (timerEl) timerEl.textContent = `${countdown}s`;

        clearInterval(playerState.nextCountdownTimer);
        playerState.nextCountdownTimer = setInterval(() => {
            countdown -= 1;
            if (timerEl) timerEl.textContent = `${countdown}s`;
            if (countdown <= 0) {
                clearInterval(playerState.nextCountdownTimer);
                prompt.style.display = 'none';
                changeEpisode(1);
            }
        }, 1000);
    }
}

function playNextEpisodeDirect() {
    clearInterval(playerState.nextCountdownTimer);
    const prompt = document.getElementById('netflixNextPrompt');
    if (prompt) prompt.style.display = 'none';
    changeEpisode(1);
}

function toggleAudioSubModal() {
    const modal = document.getElementById('netflixAudioSubModal');
    if (!modal) return;
    const isHidden = modal.style.display === 'none';
    if (isHidden) {
        renderAudioSubModal();
        modal.style.display = 'block';
    } else {
        modal.style.display = 'none';
    }
    resetIdleTimer();
}

function renderAudioSubModal() {
    if (!State.currentEpisodes) return;

    const currentSeason = State.currentEpisodes.season;
    const currentLang = (currentSeason.split('/')[1] || 'vostfr').toLowerCase();
    const availableLangs = (State.currentEpisodes.available_languages && State.currentEpisodes.available_languages.length)
        ? State.currentEpisodes.available_languages
        : [currentLang];

    const audioList = document.getElementById('netflixAudioList');
    if (audioList) {
        audioList.innerHTML = availableLangs.map(lang => {
            const isActive = lang.toLowerCase() === currentLang;
            return `
                <button class="netflix-option-row ${isActive ? 'active' : ''}" onclick="selectLanguageFromModal('${escapeHtml(lang)}')">
                    <span>${escapeHtml(formatLanguageName(lang))}</span>
                </button>
            `;
        }).join('');
    }

    const players = Object.keys(State.currentEpisodes.players || {});
    const serverList = document.getElementById('netflixServerList');
    if (serverList) {
        serverList.innerHTML = players.map(p => {
            const isActive = p === State.selectedPlayer;
            return `
                <button class="netflix-option-row ${isActive ? 'active' : ''}" onclick="selectServerFromModal('${escapeHtml(p)}')">
                    <span>${escapeHtml(p)}</span>
                </button>
            `;
        }).join('');
    }
}

function selectLanguageFromModal(newLang) {
    toggleAudioSubModal();
    switchPlayerLanguage(newLang);
}

function selectServerFromModal(newPlayer) {
    toggleAudioSubModal();
    switchLecteur(newPlayer);
}

function formatSeasonLabel(raw) {
    if (!raw) return 'Saison 1';
    const base = raw.split('/')[0].toLowerCase();
    if (base.includes('film')) return 'Film';
    if (base.includes('oav') || base.includes('ova')) return 'OAV';
    if (base.includes('special')) return 'Spécial';
    const num = base.replace(/[^0-9]/g, '');
    return num ? `Saison ${num}` : (base.charAt(0).toUpperCase() + base.slice(1));
}

function toggleEpisodeDrawer() {
    const drawer = document.getElementById('netflixEpisodesDrawer');
    if (!drawer) return;
    const audioModal = document.getElementById('netflixAudioSubModal');
    if (audioModal) audioModal.style.display = 'none';
    const speedMenu = document.getElementById('netflixSpeedMenu');
    if (speedMenu) speedMenu.style.display = 'none';

    const isHidden = drawer.style.display === 'none' || !drawer.style.display;
    if (isHidden) {
        renderEpisodeDrawer();
        drawer.style.display = 'block';
    } else {
        drawer.style.display = 'none';
    }
    resetIdleTimer();
}

function renderEpisodeDrawer() {
    const grid = document.getElementById('netflixDrawerGrid');
    if (!grid) return;

    if (!State.currentEpisodes || !State.selectedPlayer) {
        grid.innerHTML = '<div class="netflix-drawer-empty">Aucun épisode disponible</div>';
        return;
    }

    const urls = State.currentEpisodes.players[State.selectedPlayer] || [];
    const seasonName = formatSeasonLabel(State.currentEpisodes.season);

    const seasonTitle = document.getElementById('netflixDrawerSeasonTitle');
    const countEl = document.getElementById('netflixDrawerCount');
    if (seasonTitle) seasonTitle.textContent = seasonName;
    if (countEl) countEl.textContent = `${urls.length} épisode${urls.length > 1 ? 's' : ''}`;

    const slug = State.currentEpisodes.slug;
    grid.innerHTML = urls.map((url, i) => {
        const isCurrent = i === State.currentEpIndex;
        const prog = getEpisodeProgress(slug, i + 1);
        const isSeen = prog && prog.duration > 0 && prog.currentTime >= prog.duration - 25;
        const pct = (prog && prog.duration > 0) ? Math.min(100, Math.round((prog.currentTime / prog.duration) * 100)) : (isSeen ? 100 : 0);

        return `
            <button class="netflix-drawer-chip ${isCurrent ? 'active' : ''} ${isSeen ? 'seen' : ''}" onclick="selectEpisodeFromDrawer(${i})" title="Épisode ${i + 1}${isCurrent ? ' (En cours)' : (isSeen ? ' (Vu)' : '')}">
                <span class="netflix-drawer-chip-num">${i + 1}</span>
                ${pct > 0 && !isCurrent ? `<span class="netflix-drawer-chip-bar" style="width: ${pct}%"></span>` : ''}
            </button>
        `;
    }).join('');

    setTimeout(() => {
        const activeChip = grid.querySelector('.netflix-drawer-chip.active');
        if (activeChip) {
            activeChip.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
    }, 50);
}

function selectEpisodeFromDrawer(index) {
    toggleEpisodeDrawer();
    openEpisodePlayer(State.currentEpisodes.slug, State.currentEpisodes.season, index, null);
}

async function togglePictureInPicture() {
    try {
        if (window.documentPictureInPicture && window.documentPictureInPicture.window) {
            window.documentPictureInPicture.window.close();
            resetIdleTimer();
            return;
        }
        if (document.pictureInPictureElement) {
            await document.exitPictureInPicture();
            resetIdleTimer();
            return;
        }
        const v = getIframeVideo();
        if (v) {
            const doc = v.ownerDocument || document;
            if (doc.pictureInPictureElement) {
                await doc.exitPictureInPicture();
                resetIdleTimer();
                return;
            }
        }

        const animeTitle = State.currentAnime ? State.currentAnime.title : '';
        const epNum = State.currentEpIndex !== undefined ? (State.currentEpIndex + 1) : 1;
        const epTitle = `${t('modal.episode_single', 'Épisode')} ${epNum}`;
        const fullTitle = animeTitle ? `${animeTitle} - ${epTitle}` : epTitle;

        syncPlayerMediaMetadata();

        if ('documentPictureInPicture' in window) {
            try {
                const pipWindow = await window.documentPictureInPicture.requestWindow({
                    width: 720,
                    height: 405
                });
                pipWindow.document.title = fullTitle;

                const style = pipWindow.document.createElement('style');
                style.textContent = `
                    * { box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
                    body { background: #000; overflow: hidden; width: 100vw; height: 100vh; position: relative; user-select: none; }
                    .pip-viewport-wrapper { width: 100%; height: 100%; position: relative; }
                    .pip-viewport-wrapper > * { width: 100% !important; height: 100% !important; }
                    .pip-overlay { position: absolute; inset: 0; display: flex; flex-direction: column; justify-content: space-between; padding: 12px 14px; background: linear-gradient(to bottom, rgba(0,0,0,0.7) 0%, transparent 35%, transparent 65%, rgba(0,0,0,0.85) 100%); opacity: 0; transition: opacity 0.25s ease; z-index: 100; pointer-events: none; }
                    body:hover .pip-overlay { opacity: 1; pointer-events: auto; }
                    .pip-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
                    .pip-title-box { display: flex; flex-direction: column; overflow: hidden; }
                    .pip-main-title { font-size: 13px; font-weight: 700; color: #fff; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; text-shadow: 0 1px 3px rgba(0,0,0,0.9); }
                    .pip-sub-title { font-size: 11px; color: #e50914; font-weight: 600; text-shadow: 0 1px 2px rgba(0,0,0,0.9); }
                    .pip-controls { display: flex; align-items: center; justify-content: space-between; width: 100%; gap: 8px; }
                    .pip-btn { background: rgba(255,255,255,0.15); border: none; color: #fff; width: 32px; height: 32px; border-radius: 50%; display: flex; align-items: center; justify-content: center; cursor: pointer; transition: transform 0.15s, background 0.15s; }
                    .pip-btn:hover { background: #e50914; transform: scale(1.1); }
                    .pip-btn svg { width: 16px; height: 16px; fill: currentColor; }
                    .pip-progress-wrap { flex: 1; height: 4px; background: rgba(255,255,255,0.3); border-radius: 2px; position: relative; cursor: pointer; }
                    .pip-progress-bar { height: 100%; width: 0%; background: #e50914; border-radius: 2px; pointer-events: none; }
                    .pip-time { font-size: 11px; color: #eee; font-weight: 600; min-width: 65px; text-align: right; }
                `;
                pipWindow.document.head.appendChild(style);

                const viewport = document.getElementById('netflixVideoViewport');
                if (viewport) {
                    let placeholder = document.getElementById('pipPlaceholder');
                    if (!placeholder) {
                        placeholder = document.createElement('div');
                        placeholder.id = 'pipPlaceholder';
                        placeholder.style.width = '100%';
                        placeholder.style.height = '100%';
                        placeholder.style.background = '#000';
                        viewport.parentNode.insertBefore(placeholder, viewport);
                    }

                    const wrap = pipWindow.document.createElement('div');
                    wrap.className = 'pip-viewport-wrapper';
                    wrap.appendChild(viewport);

                    const overlay = pipWindow.document.createElement('div');
                    overlay.className = 'pip-overlay';
                    overlay.innerHTML = `
                        <div class="pip-header">
                            <div class="pip-title-box">
                                <span class="pip-main-title">${escapeHtml(animeTitle || 'Anime-Sama')}</span>
                                <span class="pip-sub-title">${escapeHtml(epTitle)}</span>
                            </div>
                        </div>
                        <div class="pip-controls">
                            <button class="pip-btn" id="pipBackBtn" title="-10s">
                                <svg viewBox="0 0 24 24"><path d="M12.5 3a9.5 9.5 0 0 0-8.8 6H1l3.5 4.5L8 9H5.5A7.5 7.5 0 1 1 5 13.5H3a9.5 9.5 0 1 0 9.5-10.5z"/></svg>
                            </button>
                            <button class="pip-btn" id="pipPlayBtn" title="Play/Pause">
                                <svg class="pip-icon-play" viewBox="0 0 24 24"><polygon points="6 4 20 12 6 20 6 4"></polygon></svg>
                                <svg class="pip-icon-pause" viewBox="0 0 24 24" style="display:none;"><rect x="6" y="4" width="4" height="16"></rect><rect x="14" y="4" width="4" height="16"></rect></svg>
                            </button>
                            <button class="pip-btn" id="pipFwdBtn" title="+10s">
                                <svg viewBox="0 0 24 24"><path d="M11.5 3a9.5 9.5 0 0 1 8.8 6H23l-3.5 4.5L16 9h2.5A7.5 7.5 0 1 0 19 13.5h2a9.5 9.5 0 1 1-9.5-10.5z"/></svg>
                            </button>
                            <div class="pip-progress-wrap" id="pipProgressTrack">
                                <div class="pip-progress-bar" id="pipProgressBar"></div>
                            </div>
                            <span class="pip-time" id="pipTime">00:00</span>
                        </div>
                    `;
                    wrap.appendChild(overlay);
                    pipWindow.document.body.appendChild(wrap);

                    const pBtn = overlay.querySelector('#pipPlayBtn');
                    const bBtn = overlay.querySelector('#pipBackBtn');
                    const fBtn = overlay.querySelector('#pipFwdBtn');
                    const pTrack = overlay.querySelector('#pipProgressTrack');
                    const pBar = overlay.querySelector('#pipProgressBar');
                    const pTime = overlay.querySelector('#pipTime');

                    if (bBtn) bBtn.onclick = () => seekRelative(-10);
                    if (fBtn) fBtn.onclick = () => seekRelative(10);
                    if (pBtn) pBtn.onclick = () => togglePlayPause();

                    if (pTrack) {
                        pTrack.onclick = (e) => {
                            const rect = pTrack.getBoundingClientRect();
                            const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
                            seekTo(pos * (playerState.duration || 0));
                        };
                    }

                    const pipUpdateTimer = setInterval(() => {
                        if (pipWindow.closed) {
                            clearInterval(pipUpdateTimer);
                            return;
                        }
                        const playIcon = overlay.querySelector('.pip-icon-play');
                        const pauseIcon = overlay.querySelector('.pip-icon-pause');
                        if (playIcon && pauseIcon) {
                            playIcon.style.display = playerState.paused ? 'block' : 'none';
                            pauseIcon.style.display = playerState.paused ? 'none' : 'block';
                        }
                        if (pBar && playerState.duration > 0) {
                            const pct = Math.min(100, (playerState.currentTime / playerState.duration) * 100);
                            pBar.style.width = `${pct}%`;
                        }
                        if (pTime) {
                            pTime.textContent = formatPlayerTime(playerState.currentTime);
                        }
                    }, 300);

                    pipWindow.addEventListener('pagehide', () => {
                        clearInterval(pipUpdateTimer);
                        const ph = document.getElementById('pipPlaceholder');
                        if (ph && ph.parentNode) {
                            ph.parentNode.insertBefore(viewport, ph);
                            ph.remove();
                        }
                        resetIdleTimer();
                    });
                    resetIdleTimer();
                    return;
                }
            } catch (err) {}
        }

        if (v && v.requestPictureInPicture) {
            try {
                const doc = v.ownerDocument || document;
                if (doc) doc.title = fullTitle;
                v.title = fullTitle;
            } catch (e) {}
            await v.requestPictureInPicture();
            resetIdleTimer();
            return;
        }

        sendFrameMessage({
            type: 'AS_REQUEST_PIP',
            animeTitle,
            epTitle,
            poster: State.currentAnime ? State.currentAnime.poster : ''
        });
    } catch (e) {
        sendFrameMessage({
            type: 'AS_REQUEST_PIP',
            animeTitle: State.currentAnime ? State.currentAnime.title : '',
            epTitle: `${t('modal.episode_single', 'Épisode')} ${(State.currentEpIndex || 0) + 1}`,
            poster: State.currentAnime ? State.currentAnime.poster : ''
        });
    }
    resetIdleTimer();
}

function updateFullscreenIcons() {
    const isFs = !!(document.fullscreenElement || document.webkitFullscreenElement || document.body.classList.contains('native-fullscreen'));
    const expandIcon = document.querySelector('#netflixFullscreenBtn .icon-expand');
    const compressIcon = document.querySelector('#netflixFullscreenBtn .icon-compress');
    if (expandIcon && compressIcon) {
        expandIcon.style.display = isFs ? 'none' : 'block';
        compressIcon.style.display = isFs ? 'block' : 'none';
    }
}

function toggleFullscreen() {
    const isWry = window.ipc && typeof window.ipc.postMessage === 'function';
    if (isWry) {
        window.ipc.postMessage('toggle_fullscreen');
        document.body.classList.toggle('native-fullscreen');
    }

    const container = document.getElementById('playerModal') || document.getElementById('netflixPlayerContainer') || document.documentElement;
    const isFs = !!(document.fullscreenElement || document.webkitFullscreenElement);

    if (!isFs) {
        if (container.requestFullscreen) {
            container.requestFullscreen().catch(() => {
                document.documentElement.requestFullscreen().catch(() => {});
            });
        } else if (container.webkitRequestFullscreen) {
            container.webkitRequestFullscreen();
        } else if (document.documentElement.requestFullscreen) {
            document.documentElement.requestFullscreen().catch(() => {});
        }
    } else {
        if (document.exitFullscreen) {
            document.exitFullscreen().catch(() => {});
        } else if (document.webkitExitFullscreen) {
            document.webkitExitFullscreen();
        }
        document.body.classList.remove('native-fullscreen');
    }
    updateFullscreenIcons();
    resetIdleTimer();
}

function resetIdleTimer() {
    const container = document.getElementById('netflixPlayerContainer');
    if (!container) return;
    container.classList.remove('idle');

    clearTimeout(playerState.idleTimer);
    if (!playerState.paused) {
        playerState.idleTimer = setTimeout(() => {
            const audioModal = document.getElementById('netflixAudioSubModal');
            const drawer = document.getElementById('netflixEpisodesDrawer');
            const speedMenu = document.getElementById('netflixSpeedMenu');

            const isAnyOpen = (audioModal && audioModal.style.display !== 'none') ||
                (drawer && drawer.style.display !== 'none') ||
                (speedMenu && speedMenu.style.display !== 'none');

            if (!isAnyOpen && !playerState.paused && !playerState.isDraggingScrubber) {
                container.classList.add('idle');
            }
        }, 2500);
    }
}

function switchLecteur(newPlayer) {
    if (!State.currentEpisodes || !newPlayer) return;
    State.selectedPlayer = newPlayer;

    const currentServerEl = document.getElementById('netflixCurrentServer');
    if (currentServerEl) currentServerEl.textContent = newPlayer;

    if (DOM.modalPlayerTabs) {
        const tabs = DOM.modalPlayerTabs.querySelectorAll('.player-tab-btn');
        tabs.forEach(t => t.classList.toggle('active', t.textContent.trim() === newPlayer));
    }

    openEpisodePlayer(State.currentEpisodes.slug, State.currentEpisodes.season, State.currentEpIndex, State.lastCurrentTime || 0);
}

function switchPlayerLanguage(newLang) {
    if (!State.currentEpisodes) return;
    const currentSeason = State.currentEpisodes.season;
    const seasonBase = currentSeason.split('/')[0] || 'saison1';
    const newPath = `${seasonBase}/${newLang.toLowerCase()}`;
    State.selectedLanguage = newLang.toLowerCase();
    try {
        localStorage.setItem(`as_lang_${State.currentEpisodes.slug}`, State.selectedLanguage);
        localStorage.setItem('as_last_selected_lang', State.selectedLanguage);
    } catch (e) {}

    const currentLangEl = document.getElementById('netflixCurrentLang');
    if (currentLangEl) currentLangEl.textContent = newLang.toUpperCase();

    openEpisodePlayer(State.currentEpisodes.slug, newPath, State.currentEpIndex, State.lastCurrentTime || 0);
}

function changeEpisode(delta) {
    if (playerState.isChangingEpisode) return;
    playerState.isChangingEpisode = true;
    setTimeout(() => {
        playerState.isChangingEpisode = false;
    }, 400);

    flushCurrentPlayback();
    clearInterval(playerState.nextCountdownTimer);
    const newIdx = State.currentEpIndex + delta;
    if (State.currentEpisodes) {
        const players = Object.keys(State.currentEpisodes.players || {});
        const p = State.selectedPlayer || players[0];
        const urls = (State.currentEpisodes.players && p) ? (State.currentEpisodes.players[p] || []) : [];
        if (newIdx >= 0 && newIdx < urls.length) {
            openEpisodePlayer(State.currentEpisodes.slug, State.currentEpisodes.season, newIdx, null);
        } else if (newIdx >= urls.length && delta > 0 && State.currentAnime && State.currentAnime.seasons) {
            const currentSeasonBase = (State.currentEpisodes.season || '').split('/')[0];
            const seasonIdx = State.currentAnime.seasons.findIndex(s => s.path.split('/')[0] === currentSeasonBase);
            if (seasonIdx !== -1 && seasonIdx + 1 < State.currentAnime.seasons.length) {
                const nextSeasonPath = getSeasonPathForLang(State.currentAnime.seasons[seasonIdx + 1].path, State.selectedLanguage);
                openEpisodePlayer(State.currentEpisodes.slug, nextSeasonPath, 0, null);
            }
        }
    }
}

function closePlayerModal() {
    stopDynamicFilter();
    flushCurrentPlayback();
    stopPlayerPolling();
    clearInterval(playerState.nextCountdownTimer);

    if (window.ipc && typeof window.ipc.postMessage === 'function') {
        window.ipc.postMessage('exit_fullscreen');
    }
    document.body.classList.remove('native-fullscreen');

    if (document.fullscreenElement || document.webkitFullscreenElement) {
        try {
            if (document.exitFullscreen) document.exitFullscreen().catch(() => {});
            else if (document.webkitExitFullscreen) document.webkitExitFullscreen();
        } catch (e) {}
    }
    updateFullscreenIcons();

    const modal = document.getElementById('playerModal');
    if (modal) modal.classList.remove('open');

    const iframe = document.getElementById('videoIframe');
    if (iframe) iframe.src = '';

    document.body.style.overflow = 'auto';
    updateAppTitle(false);
    renderEpisodesGrid();
    updateModalPlayButton(State.currentAnime);
    if (typeof renderHomeRows === 'function' && document.getElementById('viewHome') && document.getElementById('viewHome').classList.contains('active')) {
        renderHomeRows();
    }

    const audioModal = document.getElementById('netflixAudioSubModal');
    if (audioModal) audioModal.style.display = 'none';

    const drawer = document.getElementById('netflixEpisodesDrawer');
    if (drawer) drawer.style.display = 'none';

    const prompt = document.getElementById('netflixNextPrompt');
    if (prompt) prompt.style.display = 'none';
}

function initNetflixPlayerEvents() {
    const container = document.getElementById('netflixPlayerContainer');
    if (container) {
        container.addEventListener('mousemove', resetIdleTimer);
        container.addEventListener('mousedown', resetIdleTimer);
    }

    const shield = document.getElementById('netflixClickShield');
    if (shield) {
        shield.addEventListener('click', (e) => {
            if (playerState.clickTimeout) {
                clearTimeout(playerState.clickTimeout);
                playerState.clickTimeout = null;
                const rect = shield.getBoundingClientRect();
                const clickX = e.clientX - rect.left;
                if (clickX < rect.width * 0.35) {
                    seekRelative(-10);
                } else if (clickX > rect.width * 0.65) {
                    seekRelative(10);
                } else {
                    togglePlayPause();
                }
            } else {
                playerState.clickTimeout = setTimeout(() => {
                    playerState.clickTimeout = null;
                    togglePlayPause();
                }, 220);
            }
        });
    }

    const scrubber = document.getElementById('netflixScrubberContainer');
    const tooltip = document.getElementById('netflixScrubberTooltip');

    if (scrubber) {
        const handleScrub = (e) => {
            const rect = scrubber.getBoundingClientRect();
            const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
            const targetTime = pos * (playerState.duration || 0);

            if (tooltip) {
                tooltip.style.left = `${pos * 100}%`;
                tooltip.style.display = 'block';
                tooltip.textContent = formatPlayerTime(targetTime);
            }

            if (playerState.isDraggingScrubber) {
                const playedBar = document.getElementById('netflixProgressPlayed');
                if (playedBar) playedBar.style.width = `${pos * 100}%`;
                renderTimeDisplay(targetTime, playerState.duration);
            }
        };

        scrubber.addEventListener('mousemove', handleScrub);
        scrubber.addEventListener('mouseenter', () => {
            if (tooltip) tooltip.style.display = 'block';
        });
        scrubber.addEventListener('mouseleave', () => {
            if (!playerState.isDraggingScrubber && tooltip) tooltip.style.display = 'none';
        });

        scrubber.addEventListener('mousedown', (e) => {
            playerState.isDraggingScrubber = true;
            handleScrub(e);
            resetIdleTimer();

            const onMouseMove = (ev) => {
                if (playerState.isDraggingScrubber) handleScrub(ev);
            };

            const onMouseUp = (ev) => {
                if (playerState.isDraggingScrubber) {
                    playerState.isDraggingScrubber = false;
                    const rect = scrubber.getBoundingClientRect();
                    const pos = Math.max(0, Math.min(1, (ev.clientX - rect.left) / rect.width));
                    seekTo(pos * (playerState.duration || 0));
                    if (tooltip) tooltip.style.display = 'none';
                }
                window.removeEventListener('mousemove', onMouseMove);
                window.removeEventListener('mouseup', onMouseUp);
            };

            window.addEventListener('mousemove', onMouseMove);
            window.addEventListener('mouseup', onMouseUp);
        });
    }

    const volWrap = document.querySelector('.netflix-volume-wrap');
    const volSlider = document.getElementById('netflixVolSlider');
    if (volSlider) {
        volSlider.addEventListener('input', (e) => {
            setVolume(e.target.value);
        });
        volSlider.addEventListener('change', (e) => {
            setVolume(e.target.value);
        });
        volSlider.addEventListener('mousedown', () => {
            if (volWrap) volWrap.classList.add('active');
        });
    }
    if (volWrap) {
        volWrap.addEventListener('wheel', (e) => {
            e.preventDefault();
            const delta = e.deltaY < 0 ? 0.05 : -0.05;
            setVolume(playerState.volume + delta);
        }, { passive: false });
    }
    window.addEventListener('mouseup', () => {
        if (volWrap) volWrap.classList.remove('active');
    });

    document.addEventListener('fullscreenchange', updateFullscreenIcons);
    document.addEventListener('webkitfullscreenchange', updateFullscreenIcons);

    window.addEventListener('message', (ev) => {
        if (!ev.data) return;

        if (ev.data.type === 'AS_PLAYBACK_PROGRESS') {
            const ct = parseFloat(ev.data.currentTime) || 0;
            const dur = parseFloat(ev.data.duration) || 0;
            const bEnd = parseFloat(ev.data.bufferedEnd) || 0;
            const paused = ev.data.paused !== undefined ? !!ev.data.paused : playerState.paused;

            if (State.targetResumeTime && State.targetResumeTime > 5) {
                if (ct < State.targetResumeTime - 2) {
                    sendFrameMessage({ type: 'AS_SEEK_TO', time: State.targetResumeTime });
                    return;
                } else {
                    State.targetResumeTime = 0;
                }
            }

            syncPlaybackState(ct, dur, bEnd, paused, ev.data.volume, ev.data.muted);

            const epSlug = (State.currentAnime && State.currentAnime.slug)
                ? State.currentAnime.slug
                : (State.currentEpisodes ? State.currentEpisodes.slug : null);
            if (ev.data.ep && epSlug) {
                recordEpisodeProgress(epSlug, ev.data.ep, ct, dur);
            }
        } else if (ev.data.type === 'AS_PLAY_STATE') {
            playerState.paused = !!ev.data.paused;
            updatePlayPauseButton(!playerState.paused);
            if (!playerState.paused) resetIdleTimer();
        } else if (ev.data.type === 'AS_WAITING') {
            const spinner = document.getElementById('netflixSpinner');
            if (spinner) spinner.style.display = ev.data.waiting ? 'block' : 'none';
        } else if (ev.data.type === 'AS_ENDED') {
            changeEpisode(1);
        }
    });

    window.addEventListener('keydown', (e) => {
        const modal = document.getElementById('playerModal');
        if (!modal || !modal.classList.contains('open')) return;

        const tag = (e.target && e.target.tagName || '').toUpperCase();
        if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target && e.target.isContentEditable)) return;

        if (e.code === 'Space' || e.key === ' ' || e.key === 'k' || e.key === 'K') {
            e.preventDefault();
            togglePlayPause();
        } else if (e.code === 'ArrowLeft' || e.key === 'j' || e.key === 'J') {
            e.preventDefault();
            seekRelative(-10);
        } else if (e.code === 'ArrowRight' || e.key === 'l' || e.key === 'L') {
            e.preventDefault();
            seekRelative(10);
        } else if (e.code === 'ArrowUp') {
            e.preventDefault();
            setVolume(Math.min(1, playerState.volume + 0.05));
        } else if (e.code === 'ArrowDown') {
            e.preventDefault();
            setVolume(Math.max(0, playerState.volume - 0.05));
        } else if (e.key === 'm' || e.key === 'M') {
            e.preventDefault();
            toggleMute();
        } else if (e.key === 'f' || e.key === 'F') {
            e.preventDefault();
            toggleFullscreen();
        } else if (e.key === 'p' || e.key === 'P') {
            e.preventDefault();
            togglePictureInPicture();
        } else if (e.key === 's' || e.key === 'S') {
            const skipBtn = document.getElementById('netflixSkipIntroBtn');
            if (skipBtn && skipBtn.style.display !== 'none') {
                e.preventDefault();
                skipIntro();
            }
        } else if (e.key === 'n' || e.key === 'N') {
            e.preventDefault();
            changeEpisode(1);
        } else if (e.key === 'Escape') {
            const filterMenu = document.getElementById('netflixFilterMenu');
            const speedMenu = document.getElementById('netflixSpeedMenu');
            const audioModal = document.getElementById('netflixAudioSubModal');
            const drawer = document.getElementById('netflixEpisodesDrawer');
            if (filterMenu && filterMenu.style.display !== 'none') {
                filterMenu.style.display = 'none';
            } else if (speedMenu && speedMenu.style.display !== 'none') {
                speedMenu.style.display = 'none';
            } else if (audioModal && audioModal.style.display !== 'none') {
                audioModal.style.display = 'none';
            } else if (drawer && drawer.style.display !== 'none') {
                drawer.style.display = 'none';
            } else {
                closePlayerModal();
            }
        }
    });
}

document.addEventListener('DOMContentLoaded', () => {
    initNetflixPlayerEvents();
});
