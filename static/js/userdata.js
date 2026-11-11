const privateActionControllers = new Set();
let privateActionQueue = Promise.resolve();

function samePrivateAccount(identity) {
    const cache = window.CataclysePlaybackCache;
    if (!cache || !identity) return false;

    const current = cache.snapshot();

    return Number.isSafeInteger(identity.id) &&
        identity.id > 0 &&
        current.id === identity.id &&
        current.generation === identity.generation;
}

window.addEventListener(
    'cataclyse-playback-identity-change',
    () => {
        for (const controller of privateActionControllers) {
            controller.abort();
        }
        privateActionControllers.clear();
    }
);

async function privateUserMutation(path, payload) {
    const cache = window.CataclysePlaybackCache;

    if (!cache ||
        typeof cache.refresh !== 'function' ||
        typeof cache.snapshot !== 'function') {
        return null;
    }

    const identity = await cache.refresh();

    if (!samePrivateAccount(identity)) {
        return null;
    }

    const execute = async () => {
        if (!samePrivateAccount(identity)) return null;

        const controller = new AbortController();
        privateActionControllers.add(controller);

        try {
            const response = await fetch(path, {
                method: 'POST',
                credentials: 'same-origin',
                cache: 'no-store',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Cataclyse-Expected-User':
                        String(identity.id)
                },
                body: JSON.stringify(payload),
                signal: controller.signal
            });

            if (!response.ok) {
                if (response.status === 401 ||
                    response.status === 403) {
                    void cache.refresh();
                }
                return null;
            }

            const result = await response.json();

            // Vérifier de nouveau la session après l'écriture.
            await cache.refresh();

            if (controller.signal.aborted ||
                !samePrivateAccount(identity)) {
                return null;
            }

            const data = path === '/api/watchlist/toggle'
                ? result?.userdata
                : result;

            if (!data ||
                !Array.isArray(data.watchlist) ||
                !Array.isArray(data.favorites) ||
                !Array.isArray(data.history) ||
                !Array.isArray(data.dismissed_continue)) {
                return null;
            }

            if (path === '/api/watchlist/toggle' &&
                typeof result.in_watchlist !== 'boolean') {
                return null;
            }

            State.userData = data;
            updateWatchlistCount();

            return { result, identity };
        } catch (error) {
            if (error.name !== 'AbortError') {
                console.error(
                    '[AccountData] Écriture impossible',
                    error
                );
            }
            return null;
        } finally {
            privateActionControllers.delete(controller);
        }
    };

    // Éviter les réponses concurrentes dans le même compte.
    const pending = privateActionQueue.then(execute, execute);
    privateActionQueue = pending.then(
        () => undefined,
        () => undefined
    );

    return pending;
}

async function toggleWatchlist(slug) {
    if (typeof slug !== 'string' || !slug) return false;

    const outcome = await privateUserMutation(
        '/api/watchlist/toggle',
        { slug }
    );

    if (!outcome ||
        !samePrivateAccount(outcome.identity)) {
        return false;
    }

    const inList = outcome.result.in_watchlist;

    if (DOM.heroListBtn) {
        updateWatchlistButton(DOM.heroListBtn, inList);
    }
    if (DOM.modalWatchlistBtn) {
        updateWatchlistButton(DOM.modalWatchlistBtn, inList);
    }

    document.querySelectorAll('[data-card-watchlist]')
        .forEach(btn => {
            if (btn.dataset.cardWatchlist !== slug) return;

            btn.classList.toggle('in-list', inList);
            btn.title = inList
                ? t('btn.remove_watchlist', 'Retirer de ma liste')
                : t('btn.add_watchlist', 'Ajouter à ma liste');

            btn.innerHTML = inList
                ? '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>'
                : '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>';
        });

    showToast(inList
        ? t('toast.added', 'Ajouté à Ma Liste')
        : t('toast.removed', 'Retiré de Ma Liste')
    );

    const view = document.getElementById('viewWatchlist');
    if (view && view.classList.contains('active')) {
        renderWatchlistView();
    }

    return true;
}

function updateWatchlistButton(btn, inList) {
    if (!btn) return;
    btn.classList.toggle('in-list', inList);
    const span = btn.querySelector('span');
    if (span) {
        span.textContent = inList ? t('hero.in_list', 'Dans ma liste ✓') : t('hero.list', 'Ma Liste');
    } else {
        btn.innerHTML = inList
            ? '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>'
            : '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>';
        btn.title = inList ? t('btn.remove_watchlist', 'Retirer de ma liste') : t('btn.add_watchlist', 'Ajouter à ma liste');
    }
}

function updateWatchlistCount() {
    const count = State.userData.watchlist ? State.userData.watchlist.length : 0;
    DOM.watchlistCount.textContent = count;
}

function unDismissContinue(slug) {
    if (!slug) return;

    const cache = window.CataclysePlaybackCache;
    if (!cache || !Number.isSafeInteger(cache.snapshot().id)) {
        return;
    }

    // Modification en mémoire uniquement.
    // Le serveur persiste le retrait du masquage via /api/history.
    if (Array.isArray(State.userData.dismissed_continue)) {
        State.userData.dismissed_continue =
            State.userData.dismissed_continue.filter(
                item => item !== slug
            );
    }
}

async function recordHistory(slug, epNumber) {
    if (!slug) return false;

    // Capturer les valeurs avant les opérations asynchrones.
    const payload = {
        slug,
        title: State.currentAnime
            ? State.currentAnime.title
            : slug,
        season: State.currentEpisodes
            ? State.currentEpisodes.season
            : 'Saison 1',
        episode: epNumber,
        timestamp: Math.floor(Date.now() / 1000)
    };

    const outcome = await privateUserMutation(
        '/api/history',
        payload
    );

    return !!outcome &&
        samePrivateAccount(outcome.identity);
}

async function removeContinueItem(slug) {
    if (!slug) return false;

    const outcome = await privateUserMutation(
        '/api/history/remove',
        { slug }
    );

    if (!outcome ||
        !samePrivateAccount(outcome.identity)) {
        return false;
    }

    // Ne jamais toucher aux anciennes clés globales.
    const cache = window.CataclysePlaybackCache;
    if (cache && typeof cache.remove === 'function') {
        cache.remove(slug);
    }

    if (State.currentEpisodes &&
        State.currentEpisodes.slug === slug) {
        State.currentEpisodes = null;
        State.lastCurrentTime = 0;
        State.lastDuration = 0;
    }

    const card = Array.from(
        document.querySelectorAll('.continue-card')
    ).find(el => el.dataset.slug === slug);

    if (card) {
        card.style.transition =
            'opacity 0.25s ease, transform 0.25s ease';
        card.style.opacity = '0';
        card.style.transform = 'scale(0.85)';

        setTimeout(() => {
            if (!samePrivateAccount(outcome.identity)) return;

            if (typeof renderContinueWatchingRow === 'function') {
                renderContinueWatchingRow();
            } else if (card.parentElement) {
                card.remove();
            }
        }, 250);
    } else if (
        typeof renderContinueWatchingRow === 'function' &&
        samePrivateAccount(outcome.identity)
    ) {
        renderContinueWatchingRow();
    }

    return true;
}

function updateAppTitle(isPlaying) {
    if (isPlaying && State.currentAnime) {
        const title = State.currentAnime.title || State.currentAnime.slug;
        const ep = (State.currentEpIndex !== undefined) ? (State.currentEpIndex + 1) : 1;
        const lang = (State.selectedLanguage || 'vostfr').toUpperCase();
        document.title = `Anime-Sama — ${title} — Ep ${ep} (${lang})`;
    } else {
        document.title = 'Anime-Sama — Page d\'accueil';
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
        const animeProg = getPlaybackProgress(slug);
        if (animeProg && animeProg.episode === epNum) {
            if (!season || !animeProg.season || animeProg.season.split('/')[0].toLowerCase() === season.split('/')[0].toLowerCase()) {
                return animeProg;
            }
        }
        const general = localStorage.getItem(`as_ep_progress_${slug}_${epNum}`);
        if (general) {
            const parsed = JSON.parse(general);
            if (season && parsed.season && parsed.season.split('/')[0].toLowerCase() === season.split('/')[0].toLowerCase()) {
                return parsed;
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
