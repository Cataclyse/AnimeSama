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
