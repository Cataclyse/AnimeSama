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

let lastDiscordRpcSend = 0;
let lastDiscordRpcStatus = null;
let lastDiscordRpcEpisode = null;
let lastDiscordRpcCurrentTime = 0;
let lastDiscordRpcTimeRecordedAt = 0;

let scanState = {
    active: false,
    mode: 'scroll',
    slug: '',
    season: '',
    chapterIndex: 0,
    currentPage: 1,
    totalPages: 1,
    oeuvre: '',
    chapters: [],
    encodedOeuvre: ''
};


// Lecteur HLS natif — pilote limité au serveur de développement.
let nativeHlsVideo = null;
let nativeHlsForceIframe = false;
let nativeHlsGeneration = 0;
let nativeHlsLastSave = 0;

// PLAYBACK_SESSION_GUARD_11F10
// Identifiant de corrélation propre à chaque iframe.
// Ce n'est pas un jeton d'authentification.
let iframePlaybackSession = null;
let iframePlaybackIdentity = null;

function createIframePlaybackSession() {
    const cryptoApi = window.crypto;
    if (!cryptoApi?.getRandomValues) return null;

    const bytes = new Uint8Array(16);
    cryptoApi.getRandomValues(bytes);

    return Array.from(bytes, byte =>
        byte.toString(16).padStart(2, '0')
    ).join('');
}

function isCurrentPlaybackMessage(event) {
    const frame = document.getElementById('videoIframe');
    const modal = document.getElementById('playerModal');

    if (
        nativeHlsVideo ||
        !frame?.contentWindow ||
        !modal?.classList.contains('open') ||
        !iframePlaybackSession ||
        !iframePlaybackIdentity
    ) {
        return false;
    }

    // L'iframe sandboxée possède une origine opaque.
    if (
        event.source !== frame.contentWindow ||
        event.origin !== 'null'
    ) {
        return false;
    }

    if (
        !event.data ||
        typeof event.data !== 'object' ||
        event.data.asSession !== iframePlaybackSession
    ) {
        return false;
    }

    const current =
        window.CataclysePlaybackCache?.snapshot?.();

    return Boolean(
        current &&
        current.id === iframePlaybackIdentity.id &&
        current.generation === iframePlaybackIdentity.generation
    );
}

window.addEventListener(
    'cataclyse-playback-identity-change',
    () => {
        iframePlaybackSession = null;
        iframePlaybackIdentity = null;
        ++nativeHlsGeneration;
    }
);



function saveNativeHlsProgress(force = false) {
    const v = nativeHlsVideo;
    if (!v || !Number.isFinite(v.currentTime) || v.currentTime < 1) return;
    if (!State.currentEpisodes || !Number.isFinite(v.duration)) return;

    const now = Date.now();
    if (!force && now - nativeHlsLastSave < 10000) return;

    nativeHlsLastSave = now;

    recordEpisodeProgress(
        State.currentEpisodes.slug,
        State.currentEpIndex + 1,
        v.currentTime,
        v.duration
    );
}

function stopNativeHls(save = true) {
    const v = nativeHlsVideo;
    if (!v) return;

    if (save) saveNativeHlsProgress(true);

    nativeHlsVideo = null;
    v.pause();
    v.removeAttribute('src');
    v.load();
    v.remove();
}

async function launchNativeHls(options) {
    const { generation, episode, resumeSeconds, fallback, title, season, version, retryAttempt = 0 } = options;

    const sourceMode =
        options.sourceMode === 'all' ? 'all' : 'primary';

    const sourceIdentity = source =>
        String(source?.provider || source?.host || '').toLowerCase();

    const triedProviders = Array.isArray(options.triedProviders)
        ? options.triedProviders : [];

    const isCurrent = () =>
        generation === nativeHlsGeneration &&
        document.getElementById('playerModal')?.classList.contains('open');

    let multiEnabled = false;
    try {
        multiEnabled = (
    location.port === '48621' ||
    (
        location.protocol === 'https:' &&
        location.hostname === 'flamingo-thinkpad-l480.tailb1d655.ts.net' &&
        location.port === '8443'
    )
);
    } catch (_) {}

    try {
        const query = new URLSearchParams({
            title,
            season,
            version,
            episode: String(episode)
        });

        let media = null;

        if (multiEnabled) {
            if (!Array.isArray(options.nativeSources)) {
                try {
                    const response = await fetch(
                        '/api/video/sources?' + query +
                            '&mode=' + sourceMode,
                        { cache: 'no-store' }
                    );

                    if (response.ok) {
                        const data = await response.json();

                        options.nativeSources = Array.isArray(data.sources)
                            ? data.sources.filter(source =>
                                source &&
                                ['hls', 'mp4'].includes(source.type) &&
                                ['verified', 'uncertain'].includes(source.health) &&
                                typeof source.url === 'string' &&
                                (
                                    sourceMode !== 'all' ||
                                    !triedProviders.includes(sourceIdentity(source))
                                )
                            )
                            : [];

                        // 11G.1 : ne garder que les formats
                        // lisibles nativement par ce navigateur.
                        const capabilities = document.createElement('video');

                        options.nativeSources = options.nativeSources
                            .filter(source => {
                                if (source.type === 'hls') {
                                    return !!capabilities.canPlayType(
                                        'application/vnd.apple.mpegurl'
                                    );
                                }
                                return !!capabilities.canPlayType('video/mp4');
                            })
                            .sort((a, b) =>
                                Number(b.health === 'verified') -
                                Number(a.health === 'verified')
                            );

                        console.info(
                            '[Native HLS] Sources directes disponibles:',
                            options.nativeSources.length
                        );
                    } else {
                        console.warn(
                            '[Native HLS] Sources HTTP',
                            response.status
                        );
                    }
                } catch (error) {
                    console.warn(
                        '[Native HLS] Sources indisponibles:',
                        error.name
                    );
                }
            }

            let index = options.nativeSourceIndex || 0;

            while (
                sourceMode === 'all' &&
                options.nativeSources?.[index] &&
                triedProviders.includes(
                    sourceIdentity(options.nativeSources[index])
                )
            ) {
                index++;
            }

            options.nativeSourceIndex = index;
            media = options.nativeSources?.[index] || null;
        }

        if (!media && multiEnabled && sourceMode === 'all') {
            console.info('[Native HLS] Aucune alternative directe distincte');
            throw Error('Aucune source alternative');
        }

        if (!media) {
            const response = await fetch('/api/video/resolve?' + query);

            if (!response.ok) {
                throw Error('Résolution HTTP ' + response.status);
            }

            media = await response.json();
        }

        if (!isCurrent()) return;

        if (multiEnabled) {
            const identity = sourceIdentity(media);
            if (identity && !triedProviders.includes(identity)) {
                options.triedProviders = [...triedProviders, identity];
            }
        }

        if (!['hls', 'mp4', 'webm'].includes(media.type)) {
            throw Error('Aucune source vidéo directe compatible');
        }

        if (
            media.type === 'hls' &&
            !document.createElement('video')
                .canPlayType('application/vnd.apple.mpegurl')
        ) {
            throw Error('HLS natif indisponible');
        }

        const remote = new URL(media.url);

        if (
            remote.protocol !== 'https:' ||
            remote.username ||
            remote.password ||
            remote.port
        ) {
            throw Error('Adresse vidéo invalide');
        }

        const viewport = document.getElementById('netflixVideoViewport');
        if (!viewport) throw Error('Viewport introuvable');

        const video = document.createElement('video');
        video.id = 'cataclyseNativeVideo';
        video.playsInline = true;
        video.preload = 'metadata';
        video.volume = playerState.volume;
        video.muted = playerState.muted;
        video.playbackRate = playerState.playbackRate || 1;

        video.style.cssText = [
            'position:absolute',
            'inset:0',
            'width:100%',
            'height:100%',
            'object-fit:contain',
            'background:black',
            'pointer-events:none'
        ].join(';');

        video.addEventListener('loadedmetadata', () => {
            if (nativeHlsVideo !== video) return;

            if (resumeSeconds > 5 && Number.isFinite(video.duration)) {
                video.currentTime = Math.min(
                    resumeSeconds,
                    Math.max(0, video.duration - 1)
                );
            }

            syncPlayerMediaMetadata();
        });

        video.addEventListener('timeupdate', () => {
            if (nativeHlsVideo !== video) return;
            updateFromVideoElement(video);
            saveNativeHlsProgress();
        });

        for (const event of ['playing', 'pause', 'seeked']) {
            video.addEventListener(event, () => {
                if (nativeHlsVideo === video) {
                    updateFromVideoElement(video);

                    // 11J.2B.1 : sauvegarde immédiate
                    // avant une éventuelle suspension iOS.
                    if (event === 'pause' || event === 'seeked') {
                        saveNativeHlsProgress(true);
                    }
                }
            });
        }

        video.addEventListener('ended', () => {
            if (nativeHlsVideo !== video) return;
            saveNativeHlsProgress(true);
            changeEpisode(1);
        });

        video.addEventListener('error', () => {
            if (nativeHlsVideo !== video || !isCurrent()) return;

            const position = (
                Number.isFinite(video.currentTime) &&
                video.currentTime > 0
            ) ? video.currentTime : (
                Number.isFinite(resumeSeconds) ? resumeSeconds : 0
            );

            saveNativeHlsProgress(true);
            stopNativeHls(false);

            if (
                multiEnabled &&
                sourceMode === 'primary' &&
                retryAttempt === 0
            ) {
                console.warn('[Native HLS] Recherche des sources alternatives');

                launchNativeHls({
                    ...options,
                    sourceMode: 'all',
                    nativeSources: null,
                    nativeSourceIndex: 0,
                    resumeSeconds: position
                });
                return;
            }

            const nextIndex = (options.nativeSourceIndex || 0) + 1;

            if (
                multiEnabled &&
                Array.isArray(options.nativeSources) &&
                nextIndex < options.nativeSources.length
            ) {
                console.warn(
                    '[Native HLS] Basculement source',
                    nextIndex + 1,
                    '/',
                    options.nativeSources.length
                );

                launchNativeHls({
                    ...options,
                    nativeSourceIndex: nextIndex,
                    resumeSeconds: position
                });
                return;
            }

            if (retryAttempt < 1) {
                console.warn('[Native HLS] Erreur média, nouvelle résolution');

                launchNativeHls({
                    ...options,
                    sourceMode: 'primary',
                    nativeSources: null,
                    nativeSourceIndex: 0,
                    resumeSeconds: position,
                    retryAttempt: retryAttempt + 1
                });
                return;
            }

            console.warn('[Native HLS] Nouvelle tentative échouée, repli iframe');
            fallback(position);
        });

        nativeHlsVideo = video;
        nativeHlsLastSave = 0;

        viewport.prepend(video);
        video.src = toProxyUrl(media.url);
        video.load();

        startPlayerPolling();
        syncPlayerMediaMetadata();

        video.play().catch(error => {
            if (!isCurrent() || nativeHlsVideo !== video) return;
            if (error.name === 'NotAllowedError') {
                console.info('[Native HLS] Cliquer sur Lecture');
            } else {
                console.warn('[Native HLS] Lecture :', error.name);
            }
        });

    } catch (error) {
        if (!isCurrent()) return;
        console.warn('[Native HLS] Incident de lecture :', error.name);
        stopNativeHls();

        if (
            multiEnabled &&
            sourceMode === 'primary' &&
            retryAttempt === 0
        ) {
            console.warn('[Native HLS] Recherche des sources alternatives');

            launchNativeHls({
                ...options,
                sourceMode: 'all',
                nativeSources: null,
                nativeSourceIndex: 0
            });
            return;
        }

        const nextIndex = (options.nativeSourceIndex || 0) + 1;

        if (
            multiEnabled &&
            Array.isArray(options.nativeSources) &&
            nextIndex < options.nativeSources.length
        ) {
            launchNativeHls({
                ...options,
                nativeSourceIndex: nextIndex
            });
            return;
        }

        if (multiEnabled && retryAttempt < 1) {
            launchNativeHls({
                ...options,
                sourceMode: 'primary',
                nativeSources: null,
                nativeSourceIndex: 0,
                retryAttempt: retryAttempt + 1
            });
            return;
        }

        fallback(resumeSeconds);
    }
}

function isScanSeason(season) {
    if (!season && State.currentEpisodes && State.currentEpisodes.season) {
        season = State.currentEpisodes.season;
    }
    const s = (season || '').toLowerCase();
    return s.startsWith('scan') || s.includes('scan') || !!(State.currentEpisodes && State.currentEpisodes.scan_data);
}

function syncDiscordRpc(force = false) {
    const modal = document.getElementById('playerModal');
    if (!modal || !modal.classList.contains('open')) {
        return;
    }
    const isScan = scanState.active || isScanSeason(State.currentEpisodes ? State.currentEpisodes.season : '');
    const isPaused = !isScan && !!playerState.paused;
    const currentStatus = isScan ? 'reading' : (isPaused ? 'paused' : 'playing');
    const epNum = isScan ? (scanState.chapterIndex + 1) : (State.currentEpIndex !== undefined ? (State.currentEpIndex + 1) : 1);
    const curTime = isScan ? 0 : (playerState.currentTime || 0);
    const durTime = isScan ? null : Math.floor(playerState.duration || 0);
    const now = Date.now();

    if (!force) {
        if (lastDiscordRpcStatus !== currentStatus || lastDiscordRpcEpisode !== epNum) {
            force = true;
        } else if (currentStatus === 'playing') {
            const expectedTime = lastDiscordRpcCurrentTime + (now - lastDiscordRpcTimeRecordedAt) / 1000;
            if (Math.abs(curTime - expectedTime) > 10) {
                force = true;
            }
        }
    }

    if (!force) {
        return;
    }

    lastDiscordRpcSend = now;
    lastDiscordRpcStatus = currentStatus;
    lastDiscordRpcEpisode = epNum;
    lastDiscordRpcCurrentTime = curTime;
    lastDiscordRpcTimeRecordedAt = now;

    const animeTitle = State.currentAnime ? State.currentAnime.title : (State.currentEpisodes ? State.currentEpisodes.slug : '');
    const season = State.currentEpisodes ? State.currentEpisodes.season : null;
    const poster = State.currentAnime ? State.currentAnime.banner : null;

    fetch('/api/rpc/presence', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            title: animeTitle,
            episode: epNum,
            season: season,
            status: currentStatus,
            duration: durTime,
            current_time: isScan ? null : Math.floor(curTime),
            poster: poster
        })
    }).catch(() => {});
}

function sendNavigationRpc(title = 'Page d\'accueil') {
    lastDiscordRpcStatus = 'idle';
    lastDiscordRpcEpisode = null;
    fetch('/api/rpc/presence', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            title: title,
            episode: null,
            season: null,
            status: 'idle',
            duration: null,
            current_time: null
        })
    }).catch(() => {});
}

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
    if (nativeHlsVideo?.isConnected) return nativeHlsVideo;
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
        const isScan = scanState.active || isScanSeason(State.currentEpisodes ? State.currentEpisodes.season : '');
        const ep = (State.currentEpIndex !== undefined) ? (State.currentEpIndex + 1) : 1;
        const lang = (State.selectedLanguage || 'vf').toUpperCase();
        if (isScan) {
            document.title = `Anime-Sama — ${title} — Chapitre ${ep}`;
        } else {
            document.title = `Anime-Sama — ${title} — Ep ${ep} (${lang})`;
        }
    } else {
        document.title = 'Anime-Sama — Page d\'accueil';
    }
}

function openScanViewer(slug, season, chapterIndex) {
    scanState.active = true;
    scanState.slug = slug;
    scanState.season = season;
    scanState.chapterIndex = parseInt(chapterIndex, 10) || 0;

    try {
        const savedMode = localStorage.getItem('as_scan_reading_mode');
        if (savedMode === 'page' || savedMode === 'scroll') {
            scanState.mode = savedMode;
        }
    } catch (e) {}

    const scanData = State.currentEpisodes ? State.currentEpisodes.scan_data : null;
    scanState.oeuvre = scanData ? scanData.oeuvre : (State.currentAnime ? State.currentAnime.title : slug);
    scanState.chapters = scanData ? scanData.chapters : [];
    scanState.encodedOeuvre = encodeURIComponent(scanState.oeuvre);

    const videoViewport = document.getElementById('netflixVideoViewport');
    const scanViewport = document.getElementById('netflixScanViewport');
    const bottomBar = document.getElementById('netflixBottomBar');
    const scanBar = document.getElementById('netflixScanBar');
    const scanModeBtn = document.getElementById('netflixScanModeBtn');
    const scanModeLabel = document.getElementById('netflixScanModeLabel');
    const centerPulse = document.getElementById('netflixCenterPulse');
    const nextPrompt = document.getElementById('netflixNextPrompt');
    const pillBtn = document.getElementById('netflixPillBtn');
    const skipBtn = document.getElementById('netflixSkipIntroBtn');

    if (videoViewport) videoViewport.style.display = 'none';
    if (centerPulse) centerPulse.style.display = 'none';
    if (nextPrompt) nextPrompt.style.display = 'none';
    if (skipBtn) skipBtn.style.display = 'none';
    if (bottomBar) bottomBar.style.display = 'none';
    if (pillBtn) pillBtn.style.display = 'none';

    if (scanViewport) scanViewport.style.display = 'flex';
    if (scanBar) scanBar.style.display = 'flex';
    if (scanModeBtn) {
        scanModeBtn.style.display = 'inline-flex';
        if (scanModeLabel) {
            scanModeLabel.textContent = scanState.mode === 'scroll' ? 'Défilement' : 'Page par page';
        }
    }

    const modal = document.getElementById('playerModal');
    if (modal) modal.classList.add('open');
    document.body.style.overflow = 'hidden';

    renderScanChapterSelect();
    loadScanChapter(scanState.chapterIndex, 1);
}

function renderScanChapterSelect() {
    const select = document.getElementById('netflixScanChapSelect');
    if (!select) return;
    const chaps = scanState.chapters;
    if (chaps && chaps.length) {
        select.innerHTML = chaps.map((c, i) => `
            <option value="${i}" ${i === scanState.chapterIndex ? 'selected' : ''}>${escapeHtml(c.name)}</option>
        `).join('');
    } else {
        const total = State.currentEpisodes ? (State.currentEpisodes.total_episodes || 1) : 1;
        const opts = [];
        for (let i = 0; i < total; i++) {
            opts.push(`<option value="${i}" ${i === scanState.chapterIndex ? 'selected' : ''}>Chapitre ${i + 1}</option>`);
        }
        select.innerHTML = opts.join('');
    }
}

function onScanChapSelectChange(val) {
    const idx = parseInt(val, 10);
    if (!isNaN(idx)) {
        loadScanChapter(idx, 1);
    }
}

function toggleScanReadingMode() {
    scanState.mode = scanState.mode === 'scroll' ? 'page' : 'scroll';
    try {
        localStorage.setItem('as_scan_reading_mode', scanState.mode);
    } catch (e) {}

    const scanModeLabel = document.getElementById('netflixScanModeLabel');
    if (scanModeLabel) {
        scanModeLabel.textContent = scanState.mode === 'scroll' ? 'Défilement' : 'Page par page';
    }

    loadScanChapter(scanState.chapterIndex, scanState.currentPage || 1);
}

function loadScanChapter(chapterIndex, pageNumber = 1) {
    scanState.chapterIndex = chapterIndex;
    const totalChapters = scanState.chapters.length || (State.currentEpisodes ? State.currentEpisodes.total_episodes : 1);
    if (scanState.chapterIndex < 0) scanState.chapterIndex = 0;
    if (scanState.chapterIndex >= totalChapters) scanState.chapterIndex = totalChapters - 1;

    const chapMeta = scanState.chapters[scanState.chapterIndex];
    const chapNum = chapMeta ? chapMeta.number : (scanState.chapterIndex + 1);
    const chapName = chapMeta ? chapMeta.name : `Chapitre ${chapNum}`;
    const pageCount = chapMeta ? (chapMeta.pages || 1) : 1;
    scanState.totalPages = pageCount;
    scanState.currentPage = Math.max(1, Math.min(pageNumber, pageCount));

    const titleEl = document.getElementById('playerTitle');
    const epTitleEl = document.getElementById('playerEpisodeTitle');
    const animeTitle = State.currentAnime ? State.currentAnime.title : scanState.slug;
    if (titleEl) titleEl.textContent = animeTitle;
    if (epTitleEl) epTitleEl.textContent = `${chapName} • Scan`;

    const select = document.getElementById('netflixScanChapSelect');
    if (select) select.value = scanState.chapterIndex;

    const prevBtn = document.getElementById('netflixScanBarPrevBtn');
    const nextBtn = document.getElementById('netflixScanBarNextBtn');
    if (prevBtn) prevBtn.disabled = scanState.chapterIndex <= 0;
    if (nextBtn) nextBtn.disabled = scanState.chapterIndex >= totalChapters - 1;

    updateScanPageInfo();

    const scrollContainer = document.getElementById('netflixScanScroll');
    const singleContainer = document.getElementById('netflixScanPageSingle');
    const pagesContainer = document.getElementById('netflixScanPages');
    const endContainer = document.getElementById('netflixScanEnd');
    const endTitle = document.getElementById('netflixScanEndTitle');

    if (scanState.mode === 'scroll') {
        if (singleContainer) singleContainer.style.display = 'none';
        if (scrollContainer) scrollContainer.style.display = 'flex';
        if (pagesContainer) {
            pagesContainer.innerHTML = '';
            for (let i = 1; i <= pageCount; i++) {
                const wrap = document.createElement('div');
                wrap.className = 'netflix-scan-page-wrap';
                wrap.innerHTML = `
                    <span class="netflix-scan-page-badge">Page ${i} / ${pageCount}</span>
                    <img class="netflix-scan-page-img"
                         src="https://anime-sama.to/s2/scans/${scanState.encodedOeuvre}/${chapNum}/${i}.jpg"
                         loading="lazy"
                         decoding="async"
                         alt="${escapeHtml(chapName)} - Page ${i}"
                         onerror="this.onerror=null; this.src='/api/scan/img?oeuvre=${scanState.encodedOeuvre}&chapter=${chapNum}&page=${i}'" />
                `;
                pagesContainer.appendChild(wrap);
            }
        }
        if (endContainer) {
            endContainer.style.display = 'flex';
            if (endTitle) {
                endTitle.textContent = scanState.chapterIndex < totalChapters - 1
                    ? `Fin du ${chapName}`
                    : `Dernier chapitre atteint (${chapName})`;
            }
            const nextScanBtn = document.getElementById('netflixScanNextBtn');
            if (nextScanBtn) {
                nextScanBtn.style.display = scanState.chapterIndex < totalChapters - 1 ? 'inline-block' : 'none';
            }
        }
        if (scrollContainer) scrollContainer.scrollTop = 0;
    } else {
        if (scrollContainer) scrollContainer.style.display = 'none';
        if (singleContainer) singleContainer.style.display = 'flex';
        renderScanSinglePage();
    }

    State.currentEpIndex = scanState.chapterIndex;
    updateAppTitle(true);
    recordHistory(scanState.slug, scanState.chapterIndex + 1);
    syncDiscordRpc(true);
}

function renderScanSinglePage() {
    const chapMeta = scanState.chapters[scanState.chapterIndex];
    const chapNum = chapMeta ? chapMeta.number : (scanState.chapterIndex + 1);
    const chapName = chapMeta ? chapMeta.name : `Chapitre ${chapNum}`;
    const img = document.getElementById('netflixScanSingleImg');
    if (!img) return;

    img.alt = `${chapName} - Page ${scanState.currentPage}`;
    img.src = `https://anime-sama.to/s2/scans/${scanState.encodedOeuvre}/${chapNum}/${scanState.currentPage}.jpg`;
    img.onerror = function() {
        this.onerror = null;
        this.src = `/api/scan/img?oeuvre=${scanState.encodedOeuvre}&chapter=${chapNum}&page=${scanState.currentPage}`;
    };

    img.onclick = function(e) {
        const rect = img.getBoundingClientRect();
        const clickX = e.clientX - rect.left;
        if (clickX < rect.width / 2) {
            prevScanPage();
        } else {
            nextScanPage();
        }
    };

    updateScanPageInfo();
}

function updateScanPageInfo() {
    const pageInfo = document.getElementById('netflixScanBarPageInfo');
    if (!pageInfo) return;
    if (scanState.mode === 'page') {
        pageInfo.textContent = `Page ${scanState.currentPage} / ${scanState.totalPages}`;
    } else {
        pageInfo.textContent = `${scanState.totalPages} pages`;
    }
}

function prevScanPage() {
    if (scanState.mode !== 'page') return;
    if (scanState.currentPage > 1) {
        scanState.currentPage--;
        renderScanSinglePage();
    } else if (scanState.chapterIndex > 0) {
        const prevChap = scanState.chapters[scanState.chapterIndex - 1];
        const prevPages = prevChap ? (prevChap.pages || 1) : 1;
        loadScanChapter(scanState.chapterIndex - 1, prevPages);
    }
}

function nextScanPage() {
    if (scanState.mode !== 'page') return;
    if (scanState.currentPage < scanState.totalPages) {
        scanState.currentPage++;
        renderScanSinglePage();
    } else {
        nextScanChapter();
    }
}

function prevScanChapter() {
    if (scanState.chapterIndex > 0) {
        loadScanChapter(scanState.chapterIndex - 1, 1);
    }
}

function nextScanChapter() {
    const totalChapters = scanState.chapters.length || (State.currentEpisodes ? State.currentEpisodes.total_episodes : 1);
    if (scanState.chapterIndex < totalChapters - 1) {
        loadScanChapter(scanState.chapterIndex + 1, 1);
    }
}


// 11G.4 - Surveillance d'une source iframe.
// Aucun choix de diffuseur n'est exposé à l'utilisateur.
let iframeRecoveryMonitor = null;

function stopIframeRecoveryMonitor() {
    if (iframeRecoveryMonitor?.timer) {
        clearInterval(iframeRecoveryMonitor.timer);
    }
    iframeRecoveryMonitor = null;
}

function observeIframeRecoverySignal(data) {
    const monitor = iframeRecoveryMonitor;
    if (!monitor || !data ||
        iframePlaybackSession !== monitor.session) return;

    const now = Date.now();

    if (data.type === 'AS_PLAY_STATE' &&
        typeof data.paused === 'boolean') {
        if (monitor.paused === true && !data.paused) {
            monitor.lastMoveAt = now;
        }
        monitor.paused = data.paused;
    }

    if (data.type === 'AS_WAITING' &&
        typeof data.waiting === 'boolean') {
        monitor.waiting = data.waiting;
    }

    if (data.type !== 'AS_PLAYBACK_PROGRESS') return;

    if (typeof data.paused === 'boolean') {
        if (monitor.paused === true && !data.paused) {
            monitor.lastMoveAt = now;
        }
        monitor.paused = data.paused;
    }

    const time = Number(data.currentTime);
    if (!Number.isFinite(time) || time < 0) return;

    if (monitor.lastTime !== null) {
        const difference = time - monitor.lastTime;

        if (!monitor.paused && difference > 0.5) {
            monitor.started = true;
            monitor.lastMoveAt = now;
            monitor.waiting = false;
        } else if (difference < -2) {
            // Recherche manuelle vers une position antérieure.
            monitor.lastMoveAt = now;
        }
    }

    monitor.lastTime = time;
    monitor.position = time;
}

function startIframeRecoveryMonitor(options) {
    stopIframeRecoveryMonitor();

    const now = Date.now();
    const monitor = {
        ...options,
        startedAt: now,
        lastMoveAt: now,
        lastTime: null,
        position: null,
        started: false,
        waiting: false,
        paused: null,
        recovering: false,
        timer: null
    };

    iframeRecoveryMonitor = monitor;

    const isCurrent = () => {
        if (iframeRecoveryMonitor !== monitor ||
            monitor.generation !== nativeHlsGeneration ||
            iframePlaybackSession !== monitor.session ||
            nativeHlsVideo ||
            !document.getElementById('playerModal')
                ?.classList.contains('open') ||
            State.currentEpisodes?.slug !== monitor.slug ||
            State.currentEpIndex !== monitor.episodeIndex) {
            return false;
        }

        const account =
            window.CataclysePlaybackCache?.snapshot?.() || null;

        return account?.id === monitor.identity?.id &&
            account?.generation === monitor.identity?.generation;
    };

    monitor.timer = setInterval(() => {
        if (!isCurrent()) {
            stopIframeRecoveryMonitor();
            return;
        }

        const now = Date.now();

        // Ne pas interpréter l'absence d'activité en arrière-plan
        // comme une panne du lecteur.
        if (document.visibilityState === 'hidden') {
            monitor.startedAt = now;
            monitor.lastMoveAt = now;
            return;
        }

        // Avant la première lecture, paused=true peut
        // simplement indiquer un lecteur bloqué.
        // Après démarrage, respecter les pauses volontaires.
        if (
            monitor.recovering ||
            (monitor.started && monitor.paused === true)
        ) return;

        const noStart =
            !monitor.started &&
            now - monitor.startedAt >= 45000;

        const stalled =
            monitor.started &&
            now - monitor.lastMoveAt >=
                (monitor.waiting ? 30000 : 60000);

        if (!noStart && !stalled) return;

        monitor.recovering = true;

        const tried = [...new Set([
            ...(monitor.triedPlayers || []),
            monitor.provider
        ])];

        const players = State.currentEpisodes?.players || {};

        const alternatives = Object.keys(players).filter(name =>
            !tried.includes(name) &&
            Array.isArray(players[name]) &&
            typeof players[name][monitor.episodeIndex] === 'string' &&
            players[name][monitor.episodeIndex].trim()
        );

        // Avant le premier démarrage, préserver la
        // reprise demandée plutôt qu'une position provisoire.
        const position = (
            monitor.started &&
            Number.isFinite(monitor.position)
        ) ? monitor.position : monitor.resumeSeconds;

        stopIframeRecoveryMonitor();

        if (!alternatives.length || tried.length >= 3) {
            console.warn(
                '[Player] Toutes les alternatives iframe épuisées'
            );
            showToast(
                'Aucune autre source vidéo disponible pour cet épisode'
            );
            return;
        }

        console.warn(
            '[Player] Basculement iframe automatique',
            noStart ? 'démarrage absent' : 'lecture bloquée'
        );

        openEpisodePlayer(
            monitor.slug,
            monitor.season,
            monitor.episodeIndex,
            position,
            {
                forceIframe: true,
                triedPlayers: tried
            }
        );
    }, 2000);
}

async function openEpisodePlayer(slug, seasonPath, episodeIndex, resumeSeconds, recovery = null) {
    const nativeGeneration = ++nativeHlsGeneration;
    stopIframeRecoveryMonitor();

    iframePlaybackSession = null;
    iframePlaybackIdentity = null;

    const openingIdentity =
        window.CataclysePlaybackCache?.snapshot?.() || null;

    const isOpeningCurrent = () => {
        if (nativeGeneration !== nativeHlsGeneration) {
            return false;
        }

        const current =
            window.CataclysePlaybackCache?.snapshot?.() || null;

        return (
            openingIdentity?.id === current?.id &&
            openingIdentity?.generation === current?.generation
        );
    };

    stopNativeHls();
    if (typeof unDismissContinue === 'function') {
        unDismissContinue(slug);
    }
    try {
        if (!State.currentAnime || State.currentAnime.slug !== slug) {
            try {
                const aRes = await fetch(`/api/anime?slug=${encodeURIComponent(slug)}`);
                if (!isOpeningCurrent()) return;
                if (aRes.ok) {
                    const nextAnime = await aRes.json();
                    if (!isOpeningCurrent()) return;
                    State.currentAnime = nextAnime;
                }
            } catch (e) {}
        }

        let normalizedSeason = seasonPath;
        if (!normalizedSeason || typeof normalizedSeason !== 'string') {
            normalizedSeason = 'saison1/vostfr';
        } else if (!normalizedSeason.includes('/') && !normalizedSeason.toLowerCase().startsWith('scan')) {
            normalizedSeason = `${normalizedSeason}/vostfr`;
        }

        if (!State.currentEpisodes || State.currentEpisodes.slug !== slug || State.currentEpisodes.season !== normalizedSeason) {
            let res = await fetch(`/api/episodes?slug=${slug}&season=${encodeURIComponent(normalizedSeason)}`);
            if (!isOpeningCurrent()) return;
            if (!res.ok) {
                const animeRes = await fetch(`/api/anime?slug=${slug}`);
                if (!isOpeningCurrent()) return;
                if (animeRes.ok) {
                    const animeData = await animeRes.json();
                    if (!isOpeningCurrent()) return;
                    if (animeData.seasons && animeData.seasons.length > 0) {
                        let altSeason = animeData.seasons[0].path;
                        if (!altSeason.includes('/') && !altSeason.toLowerCase().startsWith('scan')) altSeason = `${altSeason}/vostfr`;
                        normalizedSeason = altSeason;
                        res = await fetch(`/api/episodes?slug=${slug}&season=${encodeURIComponent(normalizedSeason)}`);
                        if (!isOpeningCurrent()) return;
                    }
                }
            }
            if (!res.ok) throw new Error('Erreur récupération épisodes');
            const nextEpisodes = await res.json();
            if (!isOpeningCurrent()) return;
            State.currentEpisodes = nextEpisodes;
            const players = Object.keys(State.currentEpisodes.players || {});
            State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);
        }

        if (isScanSeason(normalizedSeason)) {
            openScanViewer(slug, normalizedSeason, episodeIndex);
            return;
        }

        // 11G.3 : ne jamais ramener un épisode demandé
        // vers le dernier épisode d'un autre diffuseur.
        const episodeIndices =
            getPlayableEpisodeIndices(State.currentEpisodes);

        const safeIdx = Number.parseInt(episodeIndex, 10);

        if (
            !Number.isInteger(safeIdx) ||
            safeIdx < 0 ||
            !episodeIndices.includes(safeIdx)
        ) {
            showToast(t('player.link_unavailable',
                "Lien d'épisode indisponible"));
            return;
        }

        episodeIndex = safeIdx;

        const players =
            Object.keys(State.currentEpisodes.players || {});

        const triedPlayers = new Set(
            Array.isArray(recovery?.triedPlayers)
                ? recovery.triedPlayers : []
        );

        const eligiblePlayers = players.filter(name => {
            if (triedPlayers.has(name)) return false;
            const links = State.currentEpisodes.players[name];
            return Array.isArray(links) &&
                typeof links[episodeIndex] === 'string' &&
                links[episodeIndex].trim().length > 0;
        });

        State.selectedPlayer = eligiblePlayers.length
            ? pickBestPlayer(eligiblePlayers, null)
            : null;

        const urls =
            State.currentEpisodes.players[State.selectedPlayer] || [];
        const videoUrl = urls[episodeIndex];

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

        const useNativeHls = (() => {
            try {
                return (
    location.port === '48621' ||
    (
        location.protocol === 'https:' &&
        location.hostname === 'flamingo-thinkpad-l480.tailb1d655.ts.net' &&
        location.port === '8443'
    )
) &&
                    !recovery?.forceIframe &&
                    !nativeHlsForceIframe &&
                    !!animeTitle &&
                    ['vf', 'vf2', 'vostfr'].includes(currentLang.toLowerCase());
            } catch (_) {
                return false;
            }
        })();

        const startIframePlayback = (resumeAt = resumeSeconds) => {
            if (!isOpeningCurrent()) return;

            const token = createIframePlaybackSession();

            if (!token) {
                console.error(
                    '[Player] Identifiant de lecture indisponible'
                );
                showToast('Lecteur sécurisé indisponible');
                return;
            }

            iframePlaybackSession = token;
            iframePlaybackIdentity =
                window.CataclysePlaybackCache?.snapshot?.() || null;
            const targetResume = (
                Number.isFinite(resumeAt) && resumeAt >= 0
            ) ? resumeAt : 0;

            const stillCurrent = () =>
                nativeGeneration === nativeHlsGeneration &&
                !!document.getElementById('playerModal')
                    ?.classList.contains('open');
            if (iframe) iframe.style.display = '';
            if (iframe) {
                iframe.onload = () => {
                    if (!stillCurrent()) return;
                    syncPlayerMediaMetadata();
                    startPlayerPolling();
                    sendFrameMessage({ type: 'AS_PLAY' });
                    if (targetResume > 5) {
                        sendFrameMessage({ type: 'AS_SEEK_TO', time: targetResume });
                    }
                    setTimeout(() => {
                        if (!stillCurrent()) return;
                        sendFrameMessage({ type: 'AS_PLAY' });
                        if (targetResume > 5) {
                            sendFrameMessage({ type: 'AS_SEEK_TO', time: targetResume });
                        }
                    }, 800);
                    setTimeout(() => {
                        if (!stillCurrent()) return;
                        sendFrameMessage({ type: 'AS_PLAY' });
                        if (targetResume > 5) {
                            sendFrameMessage({ type: 'AS_SEEK_TO', time: targetResume });
                        }
                    }, 2000);
                };
                iframe.src =
                    proxyUrl + '#as_session=' +
                    encodeURIComponent(token);

                startIframeRecoveryMonitor({
                    generation: nativeGeneration,
                    session: token,
                    identity: openingIdentity,
                    slug,
                    season: normalizedSeason,
                    episodeIndex,
                    provider: State.selectedPlayer,
                    triedPlayers: recovery?.triedPlayers || [],
                    resumeSeconds: targetResume
                });
            }

        };

        if (useNativeHls) {
            if (iframe) {
                iframe.onload = null;
                iframe.src = 'about:blank';
                iframe.style.display = 'none';
            }
        } else {
            startIframePlayback();
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
        if (prevBtn) prevBtn.disabled =
            !episodeIndices.some(index => index < episodeIndex);
        if (nextBtn) nextBtn.disabled =
            !episodeIndices.some(index => index > episodeIndex);

        updateAppTitle(true);
        recordHistory(slug, episodeIndex + 1);
        resetIdleTimer();
        startPlayerPolling();
        // 11J.2B.2 : ne pas annoncer une lecture
        // avant que le lecteur confirme son démarrage.
        updatePlayPauseButton(false);
        syncPlayerMediaMetadata();
        syncDiscordRpc(true);

        renderScrubber(resumeSeconds, estimatedDuration, 0);
        renderTimeDisplay(resumeSeconds, estimatedDuration);

        const promptEl = document.getElementById('netflixNextPrompt');
        if (promptEl) promptEl.style.display = 'none';

        const skipBtn = document.getElementById('netflixSkipIntroBtn');
        if (skipBtn) skipBtn.style.display = 'none';

        if (useNativeHls) {
            launchNativeHls({
                generation: nativeGeneration,
                episode: episodeIndex + 1,
                resumeSeconds,
                title: animeTitle,
                season: currentSeasonBase,
                version: currentLang.toLowerCase(),
                fallback: startIframePlayback
            });
        }
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
    const modal = document.getElementById('playerModal');
    if (!modal || !modal.classList.contains('open')) return;
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
    syncDiscordRpc(false);
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
    syncDiscordRpc(true);
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
    syncDiscordRpc(true);
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
    syncDiscordRpc(true);
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
        const nextIdx =
            getPlayableEpisodeIndices(State.currentEpisodes)
                .find(index => index > State.currentEpIndex);

        if (nextIdx === undefined) {
            prompt.style.display = 'none';
            return;
        }

        playerState.nextPromptActive = true;
        prompt.style.display = 'flex';

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

    // 11G.2 : conserver les langues, masquer les diffuseurs.
    const serverList = document.getElementById('netflixServerList');
    if (serverList) {
        serverList.replaceChildren();
        serverList.hidden = true;
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

    if (!State.currentEpisodes) {
        grid.innerHTML = '<div class="netflix-drawer-empty">Aucun épisode disponible</div>';
        return;
    }

    const episodeIndices =
        getPlayableEpisodeIndices(State.currentEpisodes);

    if (!episodeIndices.length) {
        grid.innerHTML = '<div class="netflix-drawer-empty">Aucun épisode disponible</div>';
        return;
    }
    const seasonName = formatSeasonLabel(State.currentEpisodes.season);

    const seasonTitle = document.getElementById('netflixDrawerSeasonTitle');
    const countEl = document.getElementById('netflixDrawerCount');
    if (seasonTitle) seasonTitle.textContent = seasonName;
    if (countEl) countEl.textContent = `${episodeIndices.length} épisode${episodeIndices.length > 1 ? 's' : ''}`;

    const slug = State.currentEpisodes.slug;
    grid.innerHTML = episodeIndices.map(i => {
        const isCurrent = i === State.currentEpIndex;
        const prog = getEpisodeProgress(slug, i + 1, State.currentEpisodes.season);
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
    nativeHlsForceIframe = true;
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
    if (scanState.active) {
        if (delta > 0) nextScanChapter();
        else prevScanChapter();
        return;
    }

    if (playerState.isChangingEpisode) return;
    playerState.isChangingEpisode = true;
    setTimeout(() => {
        playerState.isChangingEpisode = false;
    }, 400);

    flushCurrentPlayback();
    clearInterval(playerState.nextCountdownTimer);
    if (!State.currentEpisodes || !delta) return;

    const available =
        getPlayableEpisodeIndices(State.currentEpisodes);

    const newIdx = delta > 0
        ? available.find(i => i > State.currentEpIndex)
        : available.filter(i => i < State.currentEpIndex).pop();

    if (newIdx !== undefined) {
        openEpisodePlayer(
            State.currentEpisodes.slug,
            State.currentEpisodes.season,
            newIdx,
            null
        );
    } else if (
        delta > 0 &&
        State.currentAnime &&
        State.currentAnime.seasons
    ) {
        const currentSeasonBase =
            (State.currentEpisodes.season || '').split('/')[0];

        const seasonIdx = State.currentAnime.seasons.findIndex(
            s => s.path.split('/')[0] === currentSeasonBase
        );

        if (
            seasonIdx !== -1 &&
            seasonIdx + 1 < State.currentAnime.seasons.length
        ) {
            const nextSeasonPath = getSeasonPathForLang(
                State.currentAnime.seasons[seasonIdx + 1].path,
                State.selectedLanguage
            );

            openEpisodePlayer(
                State.currentEpisodes.slug,
                nextSeasonPath,
                0,
                null
            );
        }
    }
}

function closePlayerModal(saveProgress = true) {
    ++nativeHlsGeneration;
    stopIframeRecoveryMonitor();

    iframePlaybackSession = null;
    iframePlaybackIdentity = null;
    stopNativeHls(saveProgress);
    scanState.active = false;
    const scanViewport = document.getElementById('netflixScanViewport');
    const scanBar = document.getElementById('netflixScanBar');
    const scanModeBtn = document.getElementById('netflixScanModeBtn');
    const scanPages = document.getElementById('netflixScanPages');
    const videoViewport = document.getElementById('netflixVideoViewport');
    const bottomBar = document.getElementById('netflixBottomBar');
    const pillBtn = document.getElementById('netflixPillBtn');

    if (scanViewport) scanViewport.style.display = 'none';
    if (scanBar) scanBar.style.display = 'none';
    if (scanModeBtn) scanModeBtn.style.display = 'none';
    if (scanPages) scanPages.innerHTML = '';
    if (videoViewport) videoViewport.style.display = 'block';
    if (bottomBar) bottomBar.style.display = 'flex';
    if (pillBtn) pillBtn.style.display = 'inline-flex';

    stopPlayerPolling();
    stopDynamicFilter();
    if (saveProgress) flushCurrentPlayback();
    clearInterval(playerState.nextCountdownTimer);

    const modal = document.getElementById('playerModal');
    if (modal) modal.classList.remove('open');

    const iframe = document.getElementById('videoIframe');
    if (iframe) {
        iframe.onload = null;
        iframe.src = 'about:blank';
    }

    playerState.paused = true;
    playerState.currentTime = 0;
    lastDiscordRpcStatus = 'idle';
    lastDiscordRpcEpisode = null;
    lastDiscordRpcCurrentTime = 0;
    lastDiscordRpcTimeRecordedAt = 0;

    document.title = 'Anime-Sama — Page d\'accueil';
    sendNavigationRpc('Page d\'accueil');
    updateAppTitle(false);

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

    document.body.style.overflow = 'auto';
    renderEpisodesGrid();
    updateModalPlayButton(State.currentAnime);
    if (typeof renderHomeRows === 'function') {
        renderHomeRows();
    }
    if (typeof renderContinueWatchingRow === 'function') {
        renderContinueWatchingRow();
    }
    if (typeof runSync === 'function' && typeof State !== 'undefined' && State.loggedIn) {
        runSync('auto');
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
        if (!isCurrentPlaybackMessage(ev)) return;
        observeIframeRecoverySignal(ev.data);
        if (nativeHlsVideo) return;
        if (!ev.data) return;
        const modal = document.getElementById('playerModal');
        if (!modal || !modal.classList.contains('open')) return;

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

        if (scanState.active) {
            if (e.code === 'ArrowLeft') {
                e.preventDefault();
                prevScanPage();
            } else if (e.code === 'ArrowRight') {
                e.preventDefault();
                nextScanPage();
            } else if (e.key === 'n' || e.key === 'N') {
                e.preventDefault();
                nextScanChapter();
            } else if (e.key === 'p' || e.key === 'P') {
                e.preventDefault();
                prevScanChapter();
            } else if (e.key === 'm' || e.key === 'M') {
                e.preventDefault();
                toggleScanReadingMode();
            } else if (e.key === 'Escape') {
                closePlayerModal();
            }
            return;
        }

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
