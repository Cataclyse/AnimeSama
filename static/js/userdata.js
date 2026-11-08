async function toggleWatchlist(slug) {
    try {
        const res = await fetch('/api/watchlist/toggle', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ slug: slug })
        });
        if (res.ok) {
            const data = await res.json();
            State.userData = data.userdata;
            updateWatchlistCount();

            
            if (DOM.heroListBtn) updateWatchlistButton(DOM.heroListBtn, data.in_watchlist);
            if (DOM.modalWatchlistBtn) updateWatchlistButton(DOM.modalWatchlistBtn, data.in_watchlist);

            document.querySelectorAll(`[data-card-watchlist="${slug}"]`).forEach(btn => {
                btn.classList.toggle('in-list', data.in_watchlist);
                btn.title = data.in_watchlist ? t('btn.remove_watchlist', 'Retirer de ma liste') : t('btn.add_watchlist', 'Ajouter à ma liste');
                btn.innerHTML = data.in_watchlist
                    ? '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>'
                    : '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>';
            });

            showToast(data.in_watchlist ? t('toast.added', 'Ajouté à Ma Liste') : t('toast.removed', 'Retiré de Ma Liste'));

            
            if (document.getElementById('viewWatchlist').classList.contains('active')) {
                renderWatchlistView();
            }
        }
    } catch (e) {
        console.error('Erreur toggle watchlist:', e);
    }
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

async function recordHistory(slug, epNumber) {
    try {
        const res = await fetch('/api/history', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                slug: slug,
                title: State.currentAnime ? State.currentAnime.title : slug,
                season: State.currentEpisodes ? State.currentEpisodes.season : 'Saison 1',
                episode: epNumber,
                timestamp: Math.floor(Date.now() / 1000)
            })
        });
        if (res.ok) {
            const data = await res.json();
            State.userData = data;
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
