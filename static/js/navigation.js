function switchView(viewName) {
    const playerModal = document.getElementById('playerModal');
    if (playerModal && playerModal.classList.contains('open') && typeof closePlayerModal === 'function') {
        closePlayerModal();
    }
    const detailModal = document.getElementById('detailModal');
    if (detailModal && detailModal.classList.contains('open') && typeof closeDetailModal === 'function') {
        closeDetailModal();
    }

    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.toggle('active', link.dataset.view === viewName);
    });
    if (typeof window.syncMobileNavigation === 'function') {
        window.syncMobileNavigation(viewName);
    }

    
    document.querySelectorAll('.view-section').forEach(view => {
        view.classList.remove('active');
    });

    window.scrollTo({ top: 0, behavior: 'smooth' });

    switch (viewName) {
        case 'home':
            document.getElementById('viewHome').classList.add('active');
            document.title = 'Anime-Sama — Page d\'accueil';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Page d\'accueil');
            if (typeof renderHomeRows === 'function') renderHomeRows();
            if (typeof renderContinueWatchingRow === 'function') renderContinueWatchingRow();
            if (typeof runSync === 'function' && typeof State !== 'undefined' && State.loggedIn) runSync('auto');
            break;
        case 'animes':
            showCatalogueFiltered('Séries & Animes', 'Anime');
            document.title = 'Anime-Sama — Séries & Animes';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Séries & Animes');
            break;
        case 'scans':
            showCatalogueFiltered('Scans & Webtoons', 'Webtoon');
            document.title = 'Anime-Sama — Scans & Webtoons';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Scans & Webtoons');
            break;
        case 'classiques':
            showClassiquesGrid();
            document.title = 'Anime-Sama — Les Classiques';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Les Classiques');
            break;
        case 'planning':
            document.getElementById('viewPlanning').classList.add('active');
            document.title = 'Anime-Sama — Planning des sorties';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Planning des sorties');
            break;
        case 'watchlist':
            document.getElementById('viewWatchlist').classList.add('active');
            renderWatchlistView();
            document.title = 'Anime-Sama — Ma liste';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Ma liste');
            break;
        case 'search':
            document.getElementById('viewSearch').classList.add('active');
            document.title = 'Anime-Sama — Recherche';
            if (typeof sendNavigationRpc === 'function') sendNavigationRpc('Recherche');
            break;
    }
}

function showCatalogueFiltered(title, badgeFilter) {
    document.getElementById('viewGrid').classList.add('active');
    DOM.gridTitle.textContent = title;
    DOM.gridSubtitle.textContent = `Toutes les œuvres classées "${title}"`;

    const filtered = State.catalogue.filter(c => 
        (c.badges || []).some(b => b.toLowerCase().includes(badgeFilter.toLowerCase()))
    );

    DOM.catalogueGrid.innerHTML = filtered.map(c => createCardHTML(c)).join('');
}

function showClassiquesGrid() {
    document.getElementById('viewGrid').classList.add('active');
    DOM.gridTitle.textContent = 'Les Classiques Incontournables';
    DOM.gridSubtitle.textContent = 'Les œuvres mythiques de l\'animation japonaise et du manga';

    const sec = (State.homeData.sections || []).find(s => s.title.toLowerCase().includes('classiques'));
    const cards = sec ? sec.cards : State.catalogue.slice(0, 30);
    DOM.catalogueGrid.innerHTML = cards.map(c => createCardHTML(c)).join('');
}

function showCategoryGrid(title) {
    document.getElementById('viewGrid').classList.add('active');
    DOM.gridTitle.textContent = title;
    DOM.gridSubtitle.textContent = 'Affichage complet de la catégorie';

    const sec = (State.homeData.sections || []).find(s => formatSectionTitle(s.title) === title);
    const cards = sec ? sec.cards : [];
    DOM.catalogueGrid.innerHTML = cards.map(c => createCardHTML(c)).join('');
}

function renderWatchlistView() {
    const listSlugs = State.userData.watchlist || [];
    if (!listSlugs.length) {
        DOM.watchlistGrid.innerHTML = '';
        DOM.watchlistEmpty.style.display = 'block';
        return;
    }

    DOM.watchlistEmpty.style.display = 'none';
    const cards = listSlugs
        .map(slug => State.catalogue.find(c => c.slug === slug))
        .filter(Boolean);

    DOM.watchlistGrid.innerHTML = cards.map(c => createCardHTML(c)).join('');
}
