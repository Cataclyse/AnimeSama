function switchView(viewName) {
    
    document.querySelectorAll('.nav-link').forEach(link => {
        link.classList.toggle('active', link.dataset.view === viewName);
    });

    
    document.querySelectorAll('.view-section').forEach(view => {
        view.classList.remove('active');
    });

    window.scrollTo({ top: 0, behavior: 'smooth' });

    switch (viewName) {
        case 'home':
            document.getElementById('viewHome').classList.add('active');
            break;
        case 'animes':
            showCatalogueFiltered('Séries & Animes', 'Anime');
            break;
        case 'scans':
            showCatalogueFiltered('Scans & Webtoons', 'Webtoon');
            break;
        case 'classiques':
            showClassiquesGrid();
            break;
        case 'planning':
            document.getElementById('viewPlanning').classList.add('active');
            break;
        case 'watchlist':
            document.getElementById('viewWatchlist').classList.add('active');
            renderWatchlistView();
            break;
        case 'search':
            document.getElementById('viewSearch').classList.add('active');
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
