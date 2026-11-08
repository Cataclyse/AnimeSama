let playerState = {
    currentTime: 0,
    duration: 0,
    bufferedEnd: 0,
    paused: true,
    volume: 1,
    muted: false,
    playbackRate: 1,
    timeDisplayMode: 'elapsed',
    idleTimer: null,
    isDraggingScrubber: false,
    nextPromptActive: false,
    nextCountdownTimer: null,
    pollInterval: null,
    clickTimeout: null
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

async function openEpisodePlayer(slug, seasonPath, episodeIndex, resumeSeconds) {
    try {
        if (!State.currentEpisodes || State.currentEpisodes.slug !== slug || State.currentEpisodes.season !== seasonPath) {
            const res = await fetch(`/api/episodes?slug=${slug}&season=${encodeURIComponent(seasonPath)}`);
            if (!res.ok) throw new Error('Erreur récupération épisodes');
            State.currentEpisodes = await res.json();
            const players = Object.keys(State.currentEpisodes.players || {});
            State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);
        }

        const players = Object.keys(State.currentEpisodes.players || {});
        if (!State.selectedPlayer || !players.includes(State.selectedPlayer)) {
            State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);
        }

        const urls = State.currentEpisodes.players[State.selectedPlayer] || [];
        if (!urls[episodeIndex]) {
            const fallbackPlayer = players.find(p => (State.currentEpisodes.players[p] || [])[episodeIndex]);
            if (fallbackPlayer) {
                State.selectedPlayer = fallbackPlayer;
            } else {
                showToast(t('player.link_unavailable', 'Lien d\'épisode indisponible'));
                return;
            }
        }

        if (resumeSeconds === undefined || resumeSeconds === null) {
            const epProg = getEpisodeProgress(slug, episodeIndex + 1);
            const animeProg = getPlaybackProgress(slug);
            const prog = epProg || ((animeProg && animeProg.episode === episodeIndex + 1) ? animeProg : null);
            if (prog && prog.currentTime > 5) {
                if (prog.duration && prog.duration > 60 && prog.currentTime >= prog.duration - 20) {
                    resumeSeconds = 0;
                } else {
                    resumeSeconds = prog.currentTime;
                }
            } else {
                resumeSeconds = 0;
            }
        }

        State.currentEpIndex = episodeIndex;
        State.lastCurrentTime = resumeSeconds || 0;
        playerState.currentTime = resumeSeconds || 0;
        playerState.duration = 0;
        playerState.paused = false;
        playerState.nextPromptActive = false;

        const videoUrl = (State.currentEpisodes.players[State.selectedPlayer] || [])[episodeIndex];

        const titleEl = document.getElementById('playerTitle');
        const epTitleEl = document.getElementById('playerEpisodeTitle');
        const currentLangEl = document.getElementById('netflixCurrentLang');
        const currentServerEl = document.getElementById('netflixCurrentServer');

        if (titleEl) titleEl.textContent = State.currentAnime ? State.currentAnime.title : slug;
        if (epTitleEl) epTitleEl.textContent = `${t('modal.episode_single', 'Épisode')} ${episodeIndex + 1}`;

        const currentLang = (State.currentEpisodes.season.split('/')[1] || 'vostfr').toUpperCase();
        if (currentLangEl) currentLangEl.textContent = currentLang;
        if (currentServerEl) currentServerEl.textContent = State.selectedPlayer;

        let proxyUrl = toProxyUrl(videoUrl);
        const sep = proxyUrl.includes('?') ? '&' : '?';
        const qParams = [];
        if (resumeSeconds > 0) {
            qParams.push(`as_resume=${Math.floor(resumeSeconds)}`);
        }
        qParams.push(`as_ep=${episodeIndex + 1}`);
        proxyUrl += `${sep}${qParams.join('&')}`;

        const iframe = document.getElementById('videoIframe');
        if (iframe) iframe.src = proxyUrl;

        const modal = document.getElementById('playerModal');
        if (modal) modal.classList.add('open');
        document.body.style.overflow = 'hidden';

        const prevBtn = document.getElementById('playerPrevBtn');
        const nextBtn = document.getElementById('playerNextBtn');
        if (prevBtn) prevBtn.disabled = episodeIndex <= 0;
        if (nextBtn) nextBtn.disabled = episodeIndex >= urls.length - 1;

        updateAppTitle(true);
        recordHistory(slug, episodeIndex + 1);
        resetIdleTimer();
        startPlayerPolling();
        updatePlayPauseButton(true);

        const promptEl = document.getElementById('netflixNextPrompt');
        if (promptEl) promptEl.style.display = 'none';

        const skipBtn = document.getElementById('netflixSkipIntroBtn');
        if (skipBtn) skipBtn.style.display = 'none';
    } catch (e) {
        console.error('Erreur lecture vidéo:', e);
        showToast('Impossible de lancer le lecteur');
    }
}

function startPlayerPolling() {
    stopPlayerPolling();
    playerState.pollInterval = setInterval(() => {
        const v = getIframeVideo();
        if (v) {
            updateFromVideoElement(v);
        }
    }, 250);
}

function stopPlayerPolling() {
    if (playerState.pollInterval) {
        clearInterval(playerState.pollInterval);
        playerState.pollInterval = null;
    }
}

function updateFromVideoElement(v) {
    if (!v || isNaN(v.currentTime)) return;
    const ct = v.currentTime;
    const dur = v.duration || 0;
    const paused = v.paused;

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
    if (v) {
        if (v.paused) {
            v.play().catch(() => {});
            showCenterPulse('play');
        } else {
            v.pause();
            showCenterPulse('pause');
        }
    } else {
        sendFrameMessage({ type: 'AS_TOGGLE_PLAY' });
        showCenterPulse(playerState.paused ? 'play' : 'pause');
    }
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
    ripple.classList.remove('show');
    void ripple.offsetWidth;
    ripple.classList.add('show');
    setTimeout(() => {
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
    updateVolumeUI(playerState.volume, newMuted);
    resetIdleTimer();
}

function setVolume(val) {
    const v = getIframeVideo();
    const clamped = Math.max(0, Math.min(1, parseFloat(val)));
    if (v) {
        v.volume = clamped;
        v.muted = clamped === 0;
    } else {
        sendFrameMessage({ type: 'AS_SET_VOLUME', volume: clamped });
        if (clamped === 0) sendFrameMessage({ type: 'AS_SET_MUTED', muted: true });
    }
    playerState.volume = clamped;
    playerState.muted = clamped === 0;
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
        slider.value = isMuted ? 0 : vol;
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
    const menu = document.getElementById('netflixSpeedMenu');
    if (!menu) return;
    menu.style.display = menu.style.display === 'none' ? 'flex' : 'none';
    resetIdleTimer();
}

function checkSkipIntro(currentTime) {
    const skipBtn = document.getElementById('netflixSkipIntroBtn');
    if (!skipBtn) return;
    if (currentTime >= 5 && currentTime <= 85) {
        skipBtn.style.display = 'flex';
    } else {
        skipBtn.style.display = 'none';
    }
}

function skipIntro() {
    seekTo(88);
    const skipBtn = document.getElementById('netflixSkipIntroBtn');
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

function toggleEpisodeDrawer() {
    const drawer = document.getElementById('netflixEpisodesDrawer');
    if (!drawer) return;
    const isHidden = drawer.style.display === 'none';
    if (isHidden) {
        renderEpisodeDrawer();
        drawer.style.display = 'block';
    } else {
        drawer.style.display = 'none';
    }
    resetIdleTimer();
}

function renderEpisodeDrawer() {
    if (!State.currentEpisodes || !State.selectedPlayer) return;

    const urls = State.currentEpisodes.players[State.selectedPlayer] || [];
    const seasonName = formatSeasonName(State.currentEpisodes.season.split('/')[0] || 'saison1');

    const seasonTitle = document.getElementById('netflixDrawerSeasonTitle');
    const countEl = document.getElementById('netflixDrawerCount');
    if (seasonTitle) seasonTitle.textContent = seasonName;
    if (countEl) countEl.textContent = `${urls.length} épisodes`;

    const grid = document.getElementById('netflixDrawerGrid');
    if (!grid) return;

    const slug = State.currentEpisodes.slug;
    grid.innerHTML = urls.map((url, i) => {
        const isCurrent = i === State.currentEpIndex;
        const prog = getEpisodeProgress(slug, i + 1);
        const isSeen = prog && prog.duration > 0 && prog.currentTime >= prog.duration - 25;
        const statusText = isCurrent ? 'En cours' : (isSeen ? 'Vu ✓' : `Épisode ${i + 1}`);

        return `
            <div class="netflix-drawer-card ${isCurrent ? 'active' : ''}" onclick="selectEpisodeFromDrawer(${i})">
                <div class="netflix-drawer-card-num">${i + 1}</div>
                <div class="netflix-drawer-card-status">${statusText}</div>
            </div>
        `;
    }).join('');
}

function selectEpisodeFromDrawer(index) {
    toggleEpisodeDrawer();
    openEpisodePlayer(State.currentEpisodes.slug, State.currentEpisodes.season, index, null);
}

function toggleFullscreen() {
    const container = document.getElementById('netflixPlayerContainer') || document.documentElement;
    const expandIcon = document.querySelector('#netflixFullscreenBtn .icon-expand');
    const compressIcon = document.querySelector('#netflixFullscreenBtn .icon-compress');

    if (!document.fullscreenElement) {
        if (container.requestFullscreen) {
            container.requestFullscreen().catch(() => {});
        } else if (container.webkitRequestFullscreen) {
            container.webkitRequestFullscreen();
        }
        if (expandIcon && compressIcon) {
            expandIcon.style.display = 'none';
            compressIcon.style.display = 'block';
        }
    } else {
        if (document.exitFullscreen) {
            document.exitFullscreen().catch(() => {});
        } else if (document.webkitExitFullscreen) {
            document.webkitExitFullscreen();
        }
        if (expandIcon && compressIcon) {
            expandIcon.style.display = 'block';
            compressIcon.style.display = 'none';
        }
    }
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

    const currentLangEl = document.getElementById('netflixCurrentLang');
    if (currentLangEl) currentLangEl.textContent = newLang.toUpperCase();

    openEpisodePlayer(State.currentEpisodes.slug, newPath, State.currentEpIndex, State.lastCurrentTime || 0);
}

function changeEpisode(delta) {
    flushCurrentPlayback();
    clearInterval(playerState.nextCountdownTimer);
    const newIdx = State.currentEpIndex + delta;
    if (State.currentEpisodes && State.selectedPlayer) {
        const urls = State.currentEpisodes.players[State.selectedPlayer] || [];
        if (newIdx >= 0 && newIdx < urls.length) {
            openEpisodePlayer(State.currentEpisodes.slug, State.currentEpisodes.season, newIdx, null);
        }
    }
}

function closePlayerModal() {
    flushCurrentPlayback();
    stopPlayerPolling();
    clearInterval(playerState.nextCountdownTimer);

    if (document.fullscreenElement) {
        try { document.exitFullscreen(); } catch (e) {}
    }

    const modal = document.getElementById('playerModal');
    if (modal) modal.classList.remove('open');

    const iframe = document.getElementById('videoIframe');
    if (iframe) iframe.src = '';

    document.body.style.overflow = 'auto';
    updateAppTitle(false);
    renderEpisodesGrid();
    updateModalPlayButton(State.currentAnime);

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

    const volSlider = document.getElementById('netflixVolSlider');
    if (volSlider) {
        volSlider.addEventListener('input', (e) => {
            setVolume(e.target.value);
        });
    }

    window.addEventListener('message', (ev) => {
        if (!ev.data) return;

        if (ev.data.type === 'AS_PLAYBACK_PROGRESS') {
            const ct = parseFloat(ev.data.currentTime) || 0;
            const dur = parseFloat(ev.data.duration) || 0;
            const bEnd = parseFloat(ev.data.bufferedEnd) || 0;
            const paused = ev.data.paused !== undefined ? !!ev.data.paused : playerState.paused;
            syncPlaybackState(ct, dur, bEnd, paused, ev.data.volume, ev.data.muted);

            if (ev.data.ep && State.currentAnime) {
                recordEpisodeProgress(State.currentAnime.slug, ev.data.ep, ct, dur);
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
            const audioModal = document.getElementById('netflixAudioSubModal');
            const drawer = document.getElementById('netflixEpisodesDrawer');
            if (audioModal && audioModal.style.display !== 'none') {
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
