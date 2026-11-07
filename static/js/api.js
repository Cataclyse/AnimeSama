async function loadUserData() {
    try {
        const res = await fetch('/api/user');
        if (res.ok) {
            State.userData = await res.json();
            updateWatchlistCount();
        }
    } catch (e) {
        console.error('Erreur chargement profil utilisateur:', e);
    }
}

async function loadHomeData() {
    let res;
    try {
        res = await fetch('/api/home');
        if (!res.ok) throw new Error('Erreur HTTP ' + res.status);
        State.homeData = await res.json();
    } catch (e) {
        console.error('Erreur réseau accueil:', e);
        showCatalogError('Impossible de contacter le serveur local.');
        return;
    }

    if (State.homeData.error) {
        showCatalogError(State.homeData.error);
        return;
    }

    try {
        initBillboard();
    } catch (e) {
        console.error('Erreur billboard:', e);
    }
    try {
        renderHomeRows();
    } catch (e) {
        console.error('Erreur rows:', e);
    }
    try {
        initPlanning();
    } catch (e) {
        console.error('Erreur planning:', e);
    }
}

function showCatalogError(message) {
    DOM.heroTitle.textContent = 'Catalogue indisponible';
    DOM.heroBadge.textContent = '';
    DOM.heroSynopsis.textContent = '';
    DOM.heroGenres.innerHTML = '';
    DOM.heroIndicators.innerHTML = '';

    DOM.contentRows.innerHTML = `
        <div class="catalog-error">
            <h3>Impossible de charger le catalogue</h3>
            <p>${escapeHtml(message)}</p>
            <button class="btn btn-play" onclick="location.reload()">Réessayer</button>
        </div>
    `;
}

async function loadFullCatalogue() {
    try {
        const res = await fetch('/api/catalogue');
        if (res.ok) {
            State.catalogue = await res.json();
        }
    } catch (e) {
        console.error('Erreur chargement catalogue complet:', e);
    }
}
