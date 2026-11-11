function emptyAccountUserData() {
    return {
        watchlist: [],
        favorites: [],
        history: [],
        playback_positions: [],
        dismissed_continue: [],
        auto_sync: false
    };
}

let accountProfileSequence = 0;
let accountProfileController = null;

function clearAccountUserData() {
    State.userData = emptyAccountUserData();

    if (typeof updateWatchlistCount === 'function') {
        updateWatchlistCount();
    }
}

function sameProfileIdentity(identity) {
    const cache = window.CataclysePlaybackCache;

    if (!cache || typeof cache.snapshot !== 'function') {
        return false;
    }

    const current = cache.snapshot();

    return current.id === identity.id &&
        current.generation === identity.generation;
}

window.addEventListener(
    'cataclyse-playback-identity-change',
    () => {
        accountProfileSequence += 1;

        if (accountProfileController) {
            accountProfileController.abort();
            accountProfileController = null;
        }

        clearAccountUserData();
    }
);

async function loadUserData() {
    const cache = window.CataclysePlaybackCache;

    if (!cache ||
        typeof cache.refresh !== 'function' ||
        typeof cache.snapshot !== 'function') {
        clearAccountUserData();
        return false;
    }

    // L'identité est vérifiée avant de demander le profil.
    const identity = await cache.refresh();

    // Annuler le chargement précédent, même pour le même compte.
    const sequence = ++accountProfileSequence;

    if (accountProfileController) {
        accountProfileController.abort();
        accountProfileController = null;
    }

    clearAccountUserData();

    if (!identity ||
        !Number.isSafeInteger(identity.id) ||
        identity.id <= 0 ||
        !sameProfileIdentity(identity)) {
        return false;
    }

    const controller = new AbortController();
    accountProfileController = controller;

    try {
        const response = await fetch('/api/user', {
            credentials: 'same-origin',
            cache: 'no-store',
            headers: {
                'X-Cataclyse-Expected-User':
                    String(identity.id)
            },
            signal: controller.signal
        });

        if (!response.ok) {
            if (response.status === 401 ||
                response.status === 403) {
                void cache.refresh();
            }
            return false;
        }

        const data = await response.json();

        // Protection contre les réponses tardives.
        if (controller.signal.aborted ||
            sequence !== accountProfileSequence ||
            !sameProfileIdentity(identity)) {
            return false;
        }

        if (!data ||
            !Array.isArray(data.watchlist) ||
            !Array.isArray(data.favorites) ||
            !Array.isArray(data.history)) {
            console.error(
                '[AccountProfile] Réponse utilisateur invalide'
            );
            return false;
        }

        State.userData = data;
        updateWatchlistCount();

        return true;
    } catch (error) {
        if (error.name !== 'AbortError') {
            console.error(
                '[AccountProfile] Chargement impossible',
                error
            );
        }
        return false;
    } finally {
        if (accountProfileController === controller) {
            accountProfileController = null;
        }
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
