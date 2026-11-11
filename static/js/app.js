// Les données publiques (catalogue) peuvent rester visibles.
// Toutes les vues personnelles suivent l'identité locale.
let appAccountReady = false;
let appAccountRenderVersion = 0;

function repaintAccountViews() {
    const watchlist = new Set(
        Array.isArray(State.userData?.watchlist)
            ? State.userData.watchlist
            : []
    );

    if (State.homeData && !State.homeData.error &&
        typeof renderHomeRows === 'function') {
        renderHomeRows();
    }

    if (typeof renderWatchlistView === 'function') {
        renderWatchlistView();
    }

    if (typeof updateWatchlistButton === 'function') {
        const slides = State.homeData?.slides;
        const slide = Array.isArray(slides)
            ? slides[State.currentSlide]
            : null;

        if (slide?.slug && DOM.heroListBtn) {
            updateWatchlistButton(
                DOM.heroListBtn,
                watchlist.has(slide.slug)
            );
        }

        if (State.currentAnime?.slug &&
            DOM.modalWatchlistBtn) {
            updateWatchlistButton(
                DOM.modalWatchlistBtn,
                watchlist.has(State.currentAnime.slug)
            );
        }

        document.querySelectorAll('[data-card-watchlist]')
            .forEach(button => {
                const slug = button.dataset?.cardWatchlist;
                if (slug) {
                    updateWatchlistButton(
                        button,
                        watchlist.has(slug)
                    );
                }
            });
    }
}

async function reloadActiveAccountViews() {
    const cache = window.CataclysePlaybackCache;
    const version = ++appAccountRenderVersion;

    if (!cache || typeof cache.snapshot !== 'function') {
        repaintAccountViews();
        return false;
    }

    const identity = cache.snapshot();
    const loaded = await loadUserData();
    const current = cache.snapshot();

    if (
        version !== appAccountRenderVersion ||
        current.id !== identity.id ||
        current.generation !== identity.generation
    ) {
        return false;
    }

    repaintAccountViews();
    return loaded;
}

window.addEventListener(
    'cataclyse-playback-identity-change',
    () => {
        // Invalider d'abord tous les rendus en attente.
        ++appAccountRenderVersion;

        // L'ancienne connexion cloud n'est plus utilisée.
        State.loggedIn = false;
        if (DOM.accountPseudo) {
            DOM.accountPseudo.textContent = '';
        }

        // Vider immédiatement les données de l'ancien compte.
        if (typeof clearAccountUserData === 'function') {
            clearAccountUserData();
        }

        // Arrêter le lecteur sans enregistrer sa dernière
        // position dans le compte nouvellement actif.
        const playerOpen =
            DOM.playerModal?.classList.contains('open');

        if (playerOpen &&
            typeof closePlayerModal === 'function') {
            try {
                closePlayerModal(false);
            } catch (error) {
                console.error(
                    '[AccountViews] Fermeture lecteur impossible',
                    error
                );
            }
        }

        if (DOM.detailModal?.classList.contains('open') &&
            typeof closeDetailModal === 'function') {
            closeDetailModal();
        }

        State.currentAnime = null;
        State.currentEpisodes = null;
        State.currentEpIndex = 0;
        State.lastCurrentTime = 0;
        State.lastDuration = 0;

        repaintAccountViews();

        if (appAccountReady) {
            void reloadActiveAccountViews();
        }
    }
);

document.addEventListener('DOMContentLoaded', async () => {
    document.title = 'Anime-Sama — Page d\'accueil';
    setupEventListeners();

    // L'identité doit être connue avant tout rendu privé.
    const cache = window.CataclysePlaybackCache;

    if (cache && typeof cache.refresh === 'function') {
        await cache.refresh();
    }

    // Le catalogue est public.
    await loadHomeData();
    await loadFullCatalogue();

    appAccountReady = true;

    // Le chargement final reprend l'identité actuelle,
    // même si elle a changé pendant celui du catalogue.
    await reloadActiveAccountViews();
});

function setupEventListeners() {
    
    window.addEventListener('scroll', () => {
        if (window.scrollY > 40) {
            DOM.navbar.classList.add('scrolled');
        } else {
            DOM.navbar.classList.remove('scrolled');
        }
    });

    
    DOM.searchInput.addEventListener('input', (e) => {
        const val = e.target.value.trim();
        DOM.searchClearBtn.style.display = val ? 'block' : 'none';

        clearTimeout(State.searchDebounce);
        State.searchDebounce = setTimeout(() => {
            if (val.length >= 2) {
                switchView('search');
                performSearch(val);
            } else if (val.length === 0) {
                switchView('home');
            }
        }, 220);
    });

    DOM.searchClearBtn.addEventListener('click', () => {
        DOM.searchInput.value = '';
        DOM.searchClearBtn.style.display = 'none';
        switchView('home');
    });

    
    DOM.genreSelect.addEventListener('change', (e) => {
        const genre = e.target.value;
        if (genre) {
            filterByGenre(genre);
        } else {
            switchView('home');
        }
    });

    
    DOM.modalCloseBtn.addEventListener('click', closeDetailModal);
    DOM.detailModal.addEventListener('click', (e) => {
        if (e.target === DOM.detailModal) closeDetailModal();
    });

    document.addEventListener('keydown', (e) => {
        if (!DOM.playerModal || !DOM.playerModal.classList.contains('open')) {
            if (e.key === 'Escape') {
                if (DOM.detailModal && DOM.detailModal.classList.contains('open')) {
                    closeDetailModal();
                } else if (DOM.accountModal && DOM.accountModal.classList.contains('open')) {
                    closeAccountModal();
                }
            }
        }
    });

    if (DOM.playerFrameWrap) {
        DOM.playerFrameWrap.addEventListener('click', () => {
            try {
                if (DOM.videoIframe) DOM.videoIframe.focus();
            } catch(e) {}
        });
    }

    
    DOM.accountBtn.addEventListener('click', openAccountModal);
    DOM.accountCloseBtn.addEventListener('click', closeAccountModal);
    DOM.accountModal.addEventListener('click', (e) => {
        if (e.target === DOM.accountModal) closeAccountModal();
    });

    DOM.modeLocalBtn.addEventListener('click', () => switchAccountMode('local'));
    DOM.modeCloudBtn.addEventListener('click', () => switchAccountMode('cloud'));

    DOM.toRegisterLink.addEventListener('click', () => toggleAccountForms('register'));
    DOM.toLoginLink.addEventListener('click', () => toggleAccountForms('login'));

    DOM.formAccountLogin.addEventListener('submit', handleAccountLogin);
    DOM.formAccountRegister.addEventListener('submit', handleAccountRegister);

    DOM.accountLogoutBtn.addEventListener('click', handleAccountLogout);
    DOM.syncAutoBtn.addEventListener('click', () => runSync('auto'));
    DOM.autoSyncCheckbox.addEventListener('change', handleAutoSyncToggle);
    DOM.adShieldCheckbox.addEventListener('change', handleAdShieldToggle);
    if (DOM.autoUpdateCheckbox) DOM.autoUpdateCheckbox.addEventListener('change', handleAutoUpdateToggle);
    if (DOM.discordRpcCheckbox) DOM.discordRpcCheckbox.addEventListener('change', handleDiscordRpcToggle);

    
    window.addEventListener('message', (event) => {
        // PLAYBACK_SESSION_GUARD_11F10
        if (!isCurrentPlaybackMessage(event)) return;

        if (event.data && event.data.type === 'AS_PLAYBACK_PROGRESS') {
            if (!DOM.playerModal || !DOM.playerModal.classList.contains('open')) return;
            const curTime = parseFloat(event.data.currentTime);
            const duration = parseFloat(event.data.duration);
            if (!State.currentEpisodes || isNaN(curTime) || curTime <= 0) return;

            
            if (event.data.ep && event.data.ep > 0 && event.data.ep !== State.currentEpIndex + 1) {
                return;
            }

            State.lastCurrentTime = curTime;
            State.lastDuration = duration;

            const slug = State.currentEpisodes.slug;
            const season = State.currentEpisodes.season;
            const epNum = State.currentEpIndex + 1;

            savePlaybackProgress({
                slug: slug,
                season: season,
                episode: epNum,
                currentTime: curTime,
                duration: duration,
                timestamp: Date.now()
            });
        }
    });

    
    window.addEventListener('beforeunload', flushCurrentPlayback);
    window.addEventListener('pagehide', flushCurrentPlayback);
}

function flushCurrentPlayback() {
    if (State.currentEpisodes && State.lastCurrentTime > 0) {
        const epNum = State.currentEpIndex + 1;
        savePlaybackProgress({
            slug: State.currentEpisodes.slug,
            season: State.currentEpisodes.season,
            episode: epNum,
            currentTime: State.lastCurrentTime,
            duration: State.lastDuration || 0,
            timestamp: Date.now()
        });
    }
}

function savePlaybackProgress(progress) {
    try {
        const key = 'as_progress_' + progress.slug;
        localStorage.setItem(key, JSON.stringify(progress));

        const seasonBase = (progress.season || 'saison1').split('/')[0].toLowerCase();
        const epData = {
            currentTime: progress.currentTime,
            duration: progress.duration,
            season: progress.season,
            episode: progress.episode,
            timestamp: progress.timestamp || Date.now()
        };
        localStorage.setItem(`as_ep_progress_${progress.slug}_${seasonBase}_${progress.episode}`, JSON.stringify(epData));
        localStorage.setItem(`as_ep_progress_${progress.slug}_${progress.episode}`, JSON.stringify(epData));
    } catch(e) {}
}

function getPlaybackProgress(slug) {
    try {
        const val = localStorage.getItem('as_progress_' + slug);
        return val ? JSON.parse(val) : null;
    } catch(e) {
        return null;
    }
}

function getEpisodeProgress(slug, epNum, season) {
    try {
        if (season) {
            const seasonBase = season.split('/')[0].toLowerCase();
            const val = localStorage.getItem(`as_ep_progress_${slug}_${seasonBase}_${epNum}`);
            if (val) return JSON.parse(val);
        }
        const general = localStorage.getItem(`as_ep_progress_${slug}_${epNum}`);
        if (general) {
            const parsed = JSON.parse(general);
            if (!season || !parsed.season || parsed.season.split('/')[0].toLowerCase() === season.split('/')[0].toLowerCase()) {
                return parsed;
            }
        }
        const animeProg = getPlaybackProgress(slug);
        if (animeProg && animeProg.episode === epNum) {
            if (!season || !animeProg.season || animeProg.season.split('/')[0].toLowerCase() === season.split('/')[0].toLowerCase()) {
                return animeProg;
            }
        }
        return null;
    } catch(e) {
        return null;
    }
}

function recordEpisodeProgress(slug, epNum, currentTime, duration) {
    if (!slug || !epNum) return;
    const season = (State.currentEpisodes && State.currentEpisodes.season)
        ? State.currentEpisodes.season
        : 'saison1/vostfr';
    savePlaybackProgress({
        slug: slug,
        season: season,
        episode: parseInt(epNum, 10),
        currentTime: parseFloat(currentTime) || 0,
        duration: parseFloat(duration) || 0,
        timestamp: Date.now()
    });
}
