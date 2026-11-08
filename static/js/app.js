document.addEventListener('DOMContentLoaded', async () => {
    setupEventListeners();
    await loadUserData();
    await loadHomeData();
    await loadFullCatalogue();
    await refreshAccountStatus();
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

    
    window.addEventListener('message', (event) => {
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
        const epKey = `as_ep_progress_${progress.slug}_${progress.episode}`;
        localStorage.setItem(epKey, JSON.stringify({
            currentTime: progress.currentTime,
            duration: progress.duration,
            timestamp: progress.timestamp || Date.now()
        }));
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

function getEpisodeProgress(slug, epNum) {
    try {
        const val = localStorage.getItem(`as_ep_progress_${slug}_${epNum}`);
        return val ? JSON.parse(val) : null;
    } catch(e) {
        return null;
    }
}
