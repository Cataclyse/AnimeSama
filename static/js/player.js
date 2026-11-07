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
        const videoUrl = (State.currentEpisodes.players[State.selectedPlayer] || [])[episodeIndex];

        DOM.playerTitle.textContent = State.currentAnime ? State.currentAnime.title : slug;
        DOM.playerEpisodeTitle.textContent = `${t('modal.episode_single', 'Épisode')} ${episodeIndex + 1}`;

        let proxyUrl = toProxyUrl(videoUrl);
        const sep = proxyUrl.includes('?') ? '&' : '?';
        const qParams = [];
        if (resumeSeconds > 0) {
            qParams.push(`as_resume=${Math.floor(resumeSeconds)}`);
        }
        qParams.push(`as_ep=${episodeIndex + 1}`);
        proxyUrl += `${sep}${qParams.join('&')}`;
        DOM.videoIframe.src = proxyUrl;

        
        if (DOM.playerLecteurSelect) {
            DOM.playerLecteurSelect.innerHTML = players.map(p => `
                <option value="${escapeHtml(p)}" ${p === State.selectedPlayer ? 'selected' : ''}>
                    ${escapeHtml(p)}
                </option>
            `).join('');

            DOM.playerLecteurSelect.onchange = () => {
                switchLecteur(DOM.playerLecteurSelect.value);
            };
        }

        
        if (DOM.playerLangTabs) {
            const currentLang = (State.currentEpisodes.season.split('/')[1] || 'vostfr').toLowerCase();
            const availableLangs = (State.currentEpisodes && State.currentEpisodes.available_languages && State.currentEpisodes.available_languages.length)
                ? State.currentEpisodes.available_languages
                : [currentLang];
            DOM.playerLangTabs.innerHTML = availableLangs.map(lang => `
                <button class="player-lang-btn ${lang.toLowerCase() === currentLang ? 'active' : ''}" onclick="switchPlayerLanguage('${escapeHtml(lang)}')">
                    ${escapeHtml(formatLanguageName(lang))}
                </button>
            `).join('');
        }

        DOM.playerModal.classList.add('open');
        document.body.style.overflow = 'hidden';

        
        DOM.playerPrevBtn.disabled = episodeIndex <= 0;
        DOM.playerNextBtn.disabled = episodeIndex >= urls.length - 1;

        
        updateAppTitle(true);

        
        recordHistory(slug, episodeIndex + 1);

        
        setTimeout(() => {
            try {
                if (DOM.videoIframe) DOM.videoIframe.focus();
            } catch (e) {}
        }, 250);
    } catch (e) {
        console.error('Erreur lecture vidéo:', e);
        showToast('Impossible de lancer le lecteur');
    }
}

function switchLecteur(newPlayer) {
    if (!State.currentEpisodes || !newPlayer) return;
    State.selectedPlayer = newPlayer;

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
    openEpisodePlayer(State.currentEpisodes.slug, newPath, State.currentEpIndex, State.lastCurrentTime || 0);
}

function changeEpisode(delta) {
    flushCurrentPlayback();
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
    DOM.playerModal.classList.remove('open');
    DOM.videoIframe.src = '';
    document.body.style.overflow = 'auto';
    updateAppTitle(false);
    renderEpisodesGrid();
    updateModalPlayButton(State.currentAnime);
}
