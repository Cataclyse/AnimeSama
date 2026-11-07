const State = {
    homeData: null,
    catalogue: [],
    userData: { watchlist: [], favorites: [], history: [] },
    currentSlide: 0,
    slideTimer: null,
    currentAnime: null,
    currentEpisodes: null,
    selectedPlayer: null,
    selectedLanguage: 'vostfr',
    appLang: 'fr',
    trailerCache: {},
    trailerMuted: false,
    currentEpIndex: 0,
    searchDebounce: null,
    loggedIn: false,
    episodesView: 'list',
    episodesMetadata: null,
    lastCurrentTime: 0,
    lastDuration: 0,
};

const DOM = {
    navbar: document.getElementById('navbar'),
    heroBackdrop: document.getElementById('heroBackdrop'),
    heroBadge: document.getElementById('heroBadge'),
    heroTitle: document.getElementById('heroTitle'),
    heroGenres: document.getElementById('heroGenres'),
    heroSynopsis: document.getElementById('heroSynopsis'),
    heroPlayBtn: document.getElementById('heroPlayBtn'),
    heroInfoBtn: document.getElementById('heroInfoBtn'),
    heroListBtn: document.getElementById('heroListBtn'),
    heroIndicators: document.getElementById('heroIndicators'),
    contentRows: document.getElementById('contentRows'),
    planningTabs: document.getElementById('planningTabs'),
    planningGrid: document.getElementById('planningGrid'),
    catalogueGrid: document.getElementById('catalogueGrid'),
    gridTitle: document.getElementById('gridTitle'),
    gridSubtitle: document.getElementById('gridSubtitle'),
    watchlistGrid: document.getElementById('watchlistGrid'),
    watchlistEmpty: document.getElementById('watchlistEmpty'),
    watchlistCount: document.getElementById('watchlistCount'),
    searchInput: document.getElementById('searchInput'),
    searchClearBtn: document.getElementById('searchClearBtn'),
    searchGrid: document.getElementById('searchGrid'),
    searchStatus: document.getElementById('searchStatus'),
    genreSelect: document.getElementById('genreSelect'),
    
    detailModal: document.getElementById('detailModal'),
    modalCloseBtn: document.getElementById('modalCloseBtn'),
    modalHeroBg: document.getElementById('modalHeroBg'),
    modalTitle: document.getElementById('modalTitle'),
    modalBadge: document.getElementById('modalBadge'),
    modalFlags: document.getElementById('modalFlags'),
    modalStatus: document.getElementById('modalStatus'),
    modalPlayBtn: document.getElementById('modalPlayBtn'),
    modalPlayBtnText: document.getElementById('modalPlayBtnText'),
    modalWatchlistBtn: document.getElementById('modalWatchlistBtn'),
    modalSourceLink: document.getElementById('modalSourceLink'),
    modalSynopsis: document.getElementById('modalSynopsis'),
    modalSeasonSelect: document.getElementById('modalSeasonSelect'),
    modalLanguageTabs: document.getElementById('modalLanguageTabs'),
    modalPlayerTabs: document.getElementById('modalPlayerTabs'),
    modalEpisodesGrid: document.getElementById('modalEpisodesGrid'),
    modalGenres: document.getElementById('modalGenres'),
    modalType: document.getElementById('modalType'),
    modalLanguages: document.getElementById('modalLanguages'),
    modalHeroProgress: document.getElementById('modalHeroProgress'),
    modalHeroProgressFill: document.getElementById('modalHeroProgressFill'),
    modalHeroProgressLabel: document.getElementById('modalHeroProgressLabel'),
    modalYear: document.getElementById('modalYear'),
    modalEpCount: document.getElementById('modalEpCount'),
    modalAgeRating: document.getElementById('modalAgeRating'),
    modalFeaturedEpTitle: document.getElementById('modalFeaturedEpTitle'),
    
    playerModal: document.getElementById('playerModal'),
    playerCloseBtn: document.getElementById('playerCloseBtn'),
    playerTitle: document.getElementById('playerTitle'),
    playerEpisodeTitle: document.getElementById('playerEpisodeTitle'),
    playerLecteurSelect: document.getElementById('playerLecteurSelect'),
    playerLangTabs: document.getElementById('playerLangTabs'),
    playerPrevBtn: document.getElementById('playerPrevBtn'),
    playerNextBtn: document.getElementById('playerNextBtn'),
    playerFrameWrap: document.getElementById('playerFrameWrap'),
    videoIframe: document.getElementById('videoIframe'),
    toast: document.getElementById('toast'),
    
    accountBtn: document.getElementById('accountBtn'),
    accountModal: document.getElementById('accountModal'),
    accountCloseBtn: document.getElementById('accountCloseBtn'),
    modeLocalBtn: document.getElementById('modeLocalBtn'),
    modeCloudBtn: document.getElementById('modeCloudBtn'),
    panelLocal: document.getElementById('panelLocal'),
    panelCloudLoggedOut: document.getElementById('panelCloudLoggedOut'),
    panelCloudLoggedIn: document.getElementById('panelCloudLoggedIn'),
    formAccountLogin: document.getElementById('formAccountLogin'),
    formAccountRegister: document.getElementById('formAccountRegister'),
    toRegisterLink: document.getElementById('toRegisterLink'),
    toLoginLink: document.getElementById('toLoginLink'),
    accountError: document.getElementById('accountError'),
    accountPseudo: document.getElementById('accountPseudo'),
    autoSyncCheckbox: document.getElementById('autoSyncCheckbox'),
    adShieldCheckbox: document.getElementById('adShieldCheckbox'),
    syncAutoBtn: document.getElementById('syncAutoBtn'),
    syncStatus: document.getElementById('syncStatus'),
    accountLogoutBtn: document.getElementById('accountLogoutBtn'),
};


function showToast(message) {
    DOM.toast.textContent = message;
    DOM.toast.classList.add('show');
    setTimeout(() => {
        DOM.toast.classList.remove('show');
    }, 2800);
}

function toProxyUrl(url) {
    try {
        const u = new URL(url);
        return `/proxy/${u.host}${u.pathname}${u.search}`;
    } catch (e) {
        return url;
    }
}

function escapeHtml(str) {
    if (!str) return '';
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}


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
                btn.title = data.in_watchlist ? 'Retirer de ma liste' : 'Ajouter à ma liste';
                btn.innerHTML = data.in_watchlist
                    ? '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>'
                    : '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>';
            });

            showToast(data.in_watchlist ? 'Ajouté à Ma Liste' : 'Retiré de Ma Liste');

            
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
        span.textContent = inList ? 'Dans ma liste ✓' : 'Ma Liste';
    } else {
        btn.innerHTML = inList
            ? '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"></polyline></svg>'
            : '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>';
        btn.title = inList ? 'Retirer de ma liste' : 'Ajouter à ma liste';
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


function initBillboard() {
    const slides = State.homeData.slides || [];
    if (!slides.length) return;

    
    DOM.heroIndicators.innerHTML = slides.map((_, idx) => `
        <div class="indicator-dot ${idx === 0 ? 'active' : ''}" onclick="goToSlide(${idx})"></div>
    `).join('');

    renderSlide(0);
    startSlideTimer();
}

function renderSlide(index) {
    const slides = State.homeData.slides;
    if (!slides || !slides[index]) return;

    State.currentSlide = index;
    const slide = slides[index];

    
    DOM.heroBackdrop.style.opacity = '0';
    setTimeout(() => {
        const bgImg = slide.bg || `/api/img?slug=${slide.slug}`;
        DOM.heroBackdrop.style.backgroundImage = `url('${bgImg}')`;
        DOM.heroBackdrop.style.opacity = '1';
    }, 200);

    
    DOM.heroBadge.textContent = slide.badge || 'Populaire';
    DOM.heroTitle.textContent = slide.title;
    DOM.heroSynopsis.textContent = slide.synopsis;

    
    DOM.heroGenres.innerHTML = (slide.genres || []).map(g => `<span class="genre-tag">${g}</span>`).join('');

    
    const dots = DOM.heroIndicators.querySelectorAll('.indicator-dot');
    dots.forEach((d, i) => d.classList.toggle('active', i === index));

    
    const inList = State.userData.watchlist.includes(slide.slug);
    updateWatchlistButton(DOM.heroListBtn, inList);

    
    DOM.heroPlayBtn.onclick = () => openEpisodePlayer(slide.slug, getSeasonPathForLang('saison1/vostfr', State.selectedLanguage), 0);
    DOM.heroInfoBtn.onclick = () => openDetailModal(slide.slug);
    DOM.heroListBtn.onclick = () => toggleWatchlist(slide.slug);
}

function goToSlide(index) {
    renderSlide(index);
    startSlideTimer();
}

function startSlideTimer() {
    clearInterval(State.slideTimer);
    State.slideTimer = setInterval(() => {
        const next = (State.currentSlide + 1) % (State.homeData.slides.length || 1);
        renderSlide(next);
    }, 9000);
}


function renderHomeRows() {
    DOM.contentRows.innerHTML = '';

    
    if (State.homeData.slides && State.homeData.slides.length) {
        const trendingCards = State.homeData.slides.map(s => ({
            slug: s.slug,
            nom: s.title,
            img: s.bg,
            thumb: s.bg,
            badges: ['Tendance'],
            flags: ['VOSTFR'],
            etat: s.badge,
            info: s.genres ? s.genres.slice(0, 2).join(' • ') : ''
        }));
        appendRow('À l\'affiche & Tendances', trendingCards);
    }

    
    const sections = State.homeData.sections || [];
    sections.forEach(sec => {
        appendRow(formatSectionTitle(sec.title), sec.cards);
    });
}

function formatSectionTitle(rawTitle) {
    const t = rawTitle.toLowerCase();
    if (t.includes('derniers épisodes')) return 'Derniers épisodes ajoutés';
    if (t.includes('derniers scans')) return 'Derniers scans & webtoons';
    if (t.includes('derniers contenus')) return 'Nouveautés sorties';
    if (t.includes('classiques')) return 'Les Classiques légendaires';
    if (t.includes('pépites')) return 'Pépites à découvrir';
    return rawTitle;
}

function appendRow(title, cards) {
    if (!cards || !cards.length) return;

    const rowId = 'row_' + Math.random().toString(36).substring(2, 9);

    const rowEl = document.createElement('div');
    rowEl.className = 'content-row';
    rowEl.innerHTML = `
        <div class="row-header">
            <h2 class="row-title">${title}</h2>
            <a href="#" class="row-see-all" onclick="showCategoryGrid('${escapeHtml(title)}'); return false;">Tout voir ›</a>
        </div>
        <div class="rail-container">
            <button class="rail-nav-btn prev" onclick="scrollRail('${rowId}', -1)" aria-label="Défiler à gauche">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"></polyline></svg>
            </button>
            <div class="rail-slider" id="${rowId}">
                ${cards.map(card => createCardHTML(card)).join('')}
            </div>
            <button class="rail-nav-btn next" onclick="scrollRail('${rowId}', 1)" aria-label="Défiler à droite">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="9 18 15 12 9 6"></polyline></svg>
            </button>
        </div>
    `;

    DOM.contentRows.appendChild(rowEl);
}

function scrollRail(rowId, direction) {
    const rail = document.getElementById(rowId);
    if (!rail) return;
    const scrollAmount = rail.clientWidth * 0.8 * direction;
    rail.scrollBy({ left: scrollAmount, behavior: 'smooth' });
}

let currentHoverTimer = null;
let currentPreviewCard = null;

async function fetchTrailerId(title) {
    if (!title) return null;
    if (!State.trailerCache) State.trailerCache = {};
    const key = title.trim().toLowerCase();
    if (key in State.trailerCache) {
        return State.trailerCache[key];
    }

    try {
        const res = await fetch(`/api/trailer?title=${encodeURIComponent(title)}`);
        if (res.ok) {
            const data = await res.json();
            if (data && data.youtube_id) {
                State.trailerCache[key] = data.youtube_id;
                return data.youtube_id;
            }
        }
    } catch (_) {}

    try {
        const fallbackRes = await fetch(`https://kitsu.io/api/edge/anime?filter[text]=${encodeURIComponent(title)}&page[limit]=1`, {
            headers: { 'Accept': 'application/vnd.api+json' }
        });
        if (fallbackRes.ok) {
            const data = await fallbackRes.json();
            const ytid = data && data.data && data.data[0] && data.data[0].attributes ? data.data[0].attributes.youtubeVideoId : null;
            if (ytid) {
                State.trailerCache[key] = ytid;
                return ytid;
            }
        }
    } catch (_) {}

    State.trailerCache[key] = null;
    return null;
}

function handleCardMouseEnter(cardEl) {
    if (currentHoverTimer) {
        clearTimeout(currentHoverTimer);
        currentHoverTimer = null;
    }
    if (currentPreviewCard && currentPreviewCard !== cardEl) {
        cleanupCardTrailer(currentPreviewCard);
    }

    const title = cardEl.getAttribute('data-title');
    if (!title) return;

    currentHoverTimer = setTimeout(async () => {
        if (!cardEl.matches(':hover')) return;
        const ytid = await fetchTrailerId(title);
        if (!ytid || !cardEl.matches(':hover')) return;

        const slot = cardEl.querySelector('.card-trailer-slot');
        if (!slot) return;

        const isMuted = State.trailerMuted === true;

        slot.innerHTML = `
            <div class="card-trailer-wrapper">
                <iframe
                    class="card-trailer-iframe"
                    src="https://www.youtube-nocookie.com/embed/${ytid}?autoplay=1&mute=${isMuted ? 1 : 0}&controls=0&modestbranding=1&loop=1&playlist=${ytid}&rel=0&playsinline=1&enablejsapi=1&start=10"
                    title="Aperçu vidéo"
                    frameborder="0"
                    allow="autoplay; encrypted-media"
                    tabindex="-1">
                </iframe>
                <div class="card-trailer-badge">Aperçu</div>
                <button class="card-sound-btn" onclick="event.stopPropagation(); toggleTrailerSound(this);" title="${isMuted ? 'Activer le son' : 'Couper le son'}" aria-label="Son">
                    <svg class="sound-on-icon" viewBox="0 0 24 24" width="14" height="14" fill="currentColor" style="display: ${isMuted ? 'none' : 'block'};">
                        <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
                    </svg>
                    <svg class="sound-off-icon" viewBox="0 0 24 24" width="14" height="14" fill="currentColor" style="display: ${isMuted ? 'block' : 'none'};">
                        <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/>
                    </svg>
                </button>
            </div>
        `;
        cardEl.classList.add('has-trailer');
        currentPreviewCard = cardEl;

        const wrapper = slot.querySelector('.card-trailer-wrapper');
        setTimeout(() => {
            if (cardEl.matches(':hover') && wrapper) {
                wrapper.classList.add('is-playing');
                const iframe = wrapper.querySelector('iframe');
                if (iframe && iframe.contentWindow && !isMuted) {
                    iframe.contentWindow.postMessage(JSON.stringify({ event: 'command', func: 'unMute', args: [] }), '*');
                    iframe.contentWindow.postMessage(JSON.stringify({ event: 'command', func: 'setVolume', args: [65] }), '*');
                }
            }
        }, 900);
    }, 700);
}

function toggleTrailerSound(btn) {
    const card = btn.closest('.anime-card');
    if (!card) return;
    const iframe = card.querySelector('iframe');
    if (!iframe || !iframe.contentWindow) return;

    State.trailerMuted = !State.trailerMuted;
    const action = State.trailerMuted ? 'mute' : 'unMute';
    iframe.contentWindow.postMessage(JSON.stringify({ event: 'command', func: action, args: [] }), '*');

    const onIcon = btn.querySelector('.sound-on-icon');
    const offIcon = btn.querySelector('.sound-off-icon');
    if (onIcon && offIcon) {
        onIcon.style.display = State.trailerMuted ? 'none' : 'block';
        offIcon.style.display = State.trailerMuted ? 'block' : 'none';
    }
    btn.title = State.trailerMuted ? 'Activer le son' : 'Couper le son';
}

function handleCardMouseLeave(cardEl) {
    if (currentHoverTimer) {
        clearTimeout(currentHoverTimer);
        currentHoverTimer = null;
    }
    cleanupCardTrailer(cardEl);
}

function cleanupCardTrailer(cardEl) {
    if (!cardEl) return;
    cardEl.classList.remove('has-trailer');
    const slot = cardEl.querySelector('.card-trailer-slot');
    if (slot) {
        slot.innerHTML = '';
    }
    if (currentPreviewCard === cardEl) {
        currentPreviewCard = null;
    }
}

function createCardHTML(card) {
    const poster = card.thumb || card.img || `/api/img?slug=${card.slug}`;
    const inList = State.userData && State.userData.watchlist && State.userData.watchlist.includes(card.slug);
    const isWebtoon = (card.badges || []).some(b => b.toLowerCase().includes('webtoon') || b.toLowerCase().includes('scan'));
    const flags = (card.flags && card.flags.length) ? card.flags : ['VOSTFR'];
    const flagBadges = flags.map(f => `<span class="card-badge">${escapeHtml(f)}</span>`).join('');

    return `
        <div class="anime-card" data-slug="${card.slug}" data-title="${escapeHtml(card.nom)}" onclick="openDetailModal('${card.slug}')" onmouseenter="handleCardMouseEnter(this)" onmouseleave="handleCardMouseLeave(this)">
            <button class="card-watchlist-btn ${inList ? 'in-list' : ''}" data-card-watchlist="${card.slug}" onclick="event.stopPropagation(); toggleWatchlist('${card.slug}');" title="${inList ? 'Retirer de ma liste' : 'Ajouter à ma liste'}" aria-label="Ma Liste">
                ${inList 
                    ? '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="20 6 9 17 4 12"/></svg>'
                    : '<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>'
                }
            </button>
            <img class="card-poster" src="${poster}" alt="${escapeHtml(card.nom)}" loading="lazy" onerror="this.src='/api/img?slug=${card.slug}'">
            <div class="card-trailer-slot"></div>
            <div class="card-overlay">
                <div class="card-title" title="${escapeHtml(card.nom)}">${escapeHtml(card.nom)}</div>
                <div class="card-meta">
                    <span class="card-badge">${isWebtoon ? 'Scan' : 'Anime'}</span>
                    ${flagBadges}
                    ${card.etat ? `<span class="card-badge">${escapeHtml(card.etat)}</span>` : ''}
                </div>
                ${card.info ? `<div class="card-info-text">${escapeHtml(card.info)}</div>` : ''}
            </div>
        </div>
    `;
}


function initPlanning() {
    const planning = State.homeData.planning || {};
    const days = ['Dimanche', 'Lundi', 'Mardi', 'Mercredi', 'Jeudi', 'Vendredi', 'Samedi'];
    const todayIndex = new Date().getDay();
    const todayName = days[todayIndex];

    
    const tabs = DOM.planningTabs.querySelectorAll('.planning-tab');
    tabs.forEach(tab => {
        const day = tab.dataset.day;
        if (day.toLowerCase() === todayName.toLowerCase()) {
            tab.classList.add('active');
        }
        tab.onclick = () => {
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            renderPlanningDay(day);
        };
    });

    renderPlanningDay(todayName);
}

function renderPlanningDay(dayName) {
    const planning = State.homeData.planning || {};
    
    let matchedKey = Object.keys(planning).find(k => k.toLowerCase().includes(dayName.toLowerCase()));
    if (!matchedKey && Object.keys(planning).length) {
        matchedKey = Object.keys(planning)[0];
    }

    const cards = matchedKey ? (planning[matchedKey] || []) : [];
    DOM.planningGrid.innerHTML = cards.length 
        ? cards.map(c => createCardHTML(c)).join('')
        : '<p style="color:#888; padding:20px;">Aucune sortie programmée pour ce jour.</p>';
}


const animeMetadataCache = {};

async function fetchAnimeAndEpisodesMeta(title, slug) {
    const key = (title || slug || '').toLowerCase().trim();
    if (!key) return null;
    if (animeMetadataCache[key]) return animeMetadataCache[key];

    try {
        const local = localStorage.getItem('as_meta_' + key);
        if (local) {
            const parsed = JSON.parse(local);
            animeMetadataCache[key] = parsed;
            return parsed;
        }
    } catch (e) {}

    try {
        const cleanSearch = key.replace(/saison\s*\d+/gi, '').replace(/\(tv\)/gi, '').trim();
        const searchRes = await fetch(`https://kitsu.io/api/edge/anime?filter[text]=${encodeURIComponent(cleanSearch)}&page[limit]=1`);
        if (!searchRes.ok) return null;
        const searchData = await searchRes.json();
        if (!searchData.data || !searchData.data.length) return null;

        const animeObj = searchData.data[0];
        const animeId = animeObj.id;
        const attrs = animeObj.attributes || {};
        const totalCount = attrs.episodeCount || 24;

        
        const pageOffsets = [0];
        if (totalCount > 20) pageOffsets.push(20);
        if (totalCount > 40) pageOffsets.push(40);
        if (totalCount > 60) pageOffsets.push(60);

        const pageResponses = await Promise.all(
            pageOffsets.map(offset =>
                fetch(`https://kitsu.io/api/edge/anime/${animeId}/episodes?page[limit]=20&page[offset]=${offset}`)
                    .then(r => r.ok ? r.json() : { data: [] })
                    .catch(() => ({ data: [] }))
            )
        );

        const episodesMap = {};
        for (const epData of pageResponses) {
            (epData.data || []).forEach(ep => {
                const epAttrs = ep.attributes || {};
                const thumb = epAttrs.thumbnail || {};
                const num = epAttrs.number;
                if (num) {
                    episodesMap[num] = {
                        number: num,
                        title: epAttrs.canonicalTitle || `Épisode ${num}`,
                        thumbnail: thumb.original || thumb.large || thumb.medium || thumb.small || null,
                        synopsis: epAttrs.synopsis || '',
                        length: epAttrs.length || 24,
                    };
                }
            });
        }

        const result = {
            canonicalTitle: attrs.canonicalTitle || title,
            year: attrs.startDate ? attrs.startDate.split('-')[0] : null,
            ageRating: attrs.ageRating || '16+',
            episodeCount: attrs.episodeCount || Object.keys(episodesMap).length,
            episodes: episodesMap,
        };

        animeMetadataCache[key] = result;
        try {
            localStorage.setItem('as_meta_' + key, JSON.stringify(result));
        } catch (e) {}

        return result;
    } catch (err) {
        console.warn('Erreur chargement métadonnées Kitsu:', err);
        return null;
    }
}

async function enrichAnimeMetadata(title, slug) {
    const meta = await fetchAnimeAndEpisodesMeta(title, slug);
    if (!meta || !State.currentAnime || State.currentAnime.slug !== slug) return;

    if (meta.year && DOM.modalYear) DOM.modalYear.textContent = meta.year;
    if (meta.episodeCount && DOM.modalEpCount) DOM.modalEpCount.textContent = `${meta.episodeCount} Épisodes`;
    if (meta.ageRating && DOM.modalAgeRating) DOM.modalAgeRating.textContent = meta.ageRating;

    State.episodesMetadata = meta.episodes || {};

    const historyItem = (State.userData.history || []).find(h => h.slug === slug);
    const targetEpNum = (historyItem && historyItem.episode) ? historyItem.episode : 1;
    const epMeta = State.episodesMetadata[targetEpNum];
    if (epMeta && DOM.modalFeaturedEpTitle) {
        DOM.modalFeaturedEpTitle.textContent = `S1:E${targetEpNum} "${epMeta.title}"`;
    }

    renderEpisodesGrid();
    updateModalPlayButton(State.currentAnime);
}

function updateModalPlayButton(anime) {
    if (!anime) anime = State.currentAnime;
    if (!anime) return;

    const savedProgress = getPlaybackProgress(anime.slug);
    const historyItem = (State.userData.history || []).find(h => h.slug === anime.slug);
    const lastWatchedEp = savedProgress ? savedProgress.episode : (historyItem ? historyItem.episode : 0);

    if (savedProgress && savedProgress.currentTime > 5) {
        const curMins = Math.floor(savedProgress.currentTime / 60);
        const totalMins = Math.floor((savedProgress.duration || 1440) / 60);
        const pct = Math.min(100, Math.round((savedProgress.currentTime / (savedProgress.duration || 1440)) * 100));

        if (DOM.modalHeroProgress) {
            DOM.modalHeroProgress.style.display = 'flex';
            if (DOM.modalHeroProgressLabel) {
                DOM.modalHeroProgressLabel.textContent = `${curMins} sur ${totalMins} min (Épisode ${savedProgress.episode})`;
            }
            if (DOM.modalHeroProgressFill) {
                DOM.modalHeroProgressFill.style.width = `${pct}%`;
            }
        }
        if (DOM.modalPlayBtnText) {
            DOM.modalPlayBtnText.textContent = `Reprendre Ep ${savedProgress.episode} (${curMins} min)`;
        }
    } else if (lastWatchedEp > 0) {
        if (DOM.modalHeroProgress) {
            DOM.modalHeroProgress.style.display = 'flex';
            if (DOM.modalHeroProgressLabel) DOM.modalHeroProgressLabel.textContent = `Épisode ${lastWatchedEp}`;
            if (DOM.modalHeroProgressFill) DOM.modalHeroProgressFill.style.width = '75%';
        }
        if (DOM.modalPlayBtnText) DOM.modalPlayBtnText.textContent = `Reprendre Ep ${lastWatchedEp}`;
    } else {
        if (DOM.modalHeroProgress) DOM.modalHeroProgress.style.display = 'none';
        if (DOM.modalPlayBtnText) DOM.modalPlayBtnText.textContent = 'Lecture';
    }

    if (DOM.modalFeaturedEpTitle) {
        const epNum = Math.max(1, lastWatchedEp);
        const epMeta = (State.episodesMetadata && State.episodesMetadata[epNum]) ? State.episodesMetadata[epNum] : null;
        const epTitle = (epMeta && epMeta.title) ? epMeta.title : `Épisode ${epNum}`;
        const seasonNum = ((savedProgress && savedProgress.season) ? savedProgress.season.split('/')[0] : 'saison1').replace(/[^0-9]/g, '') || '1';
        DOM.modalFeaturedEpTitle.textContent = `S${seasonNum}:E${epNum} "${epTitle}"`;
    }

    if (DOM.modalPlayBtn) {
        DOM.modalPlayBtn.onclick = () => {
            const freshProg = getPlaybackProgress(anime.slug);
            const freshHistory = (State.userData.history || []).find(h => h.slug === anime.slug);
            const targetEp = freshProg ? freshProg.episode : (freshHistory ? freshHistory.episode : 1);
            const targetIdx = Math.max(0, targetEp - 1);
            const targetSeason = (freshProg && freshProg.season)
                ? freshProg.season
                : (State.currentEpisodes ? State.currentEpisodes.season : (DOM.modalSeasonSelect ? DOM.modalSeasonSelect.value : 'saison1/vostfr'));
            openEpisodePlayer(anime.slug, targetSeason, targetIdx, null);
        };
    }
}

const LANG_DISPLAY_NAMES = {
    'vostfr': 'VOSTFR',
    'vf': 'VF',
    'vf1': 'VF 1',
    'vf2': 'VF 2',
    'va': 'VA',
    'vastfr': 'VA',
    'vj': 'VJ',
    'var': 'VAR',
    'vkr': 'VKR',
    'vcn': 'VCN',
    'vqc': 'VQC'
};

function formatLanguageName(code) {
    if (!code) return 'VOSTFR';
    const key = code.toLowerCase().trim();
    return LANG_DISPLAY_NAMES[key] || code.toUpperCase();
}

async function openDetailModal(slug) {
    if (!slug) return;
    DOM.detailModal.classList.add('open');
    document.body.style.overflow = 'hidden';

    
    DOM.modalTitle.textContent = 'Chargement...';
    DOM.modalSynopsis.textContent = 'Chargement des détails de l\'œuvre...';
    DOM.modalHeroBg.style.backgroundImage = `url('/api/img?slug=${slug}')`;
    DOM.modalSeasonSelect.innerHTML = '<option>Chargement des saisons...</option>';
    DOM.modalPlayerTabs.innerHTML = '';
    DOM.modalEpisodesGrid.innerHTML = '<div class="episodes-loading">Recherche des épisodes en cours...</div>';
    if (DOM.modalHeroProgress) DOM.modalHeroProgress.style.display = 'none';
    if (DOM.modalLanguages) DOM.modalLanguages.textContent = 'Détection...';

    try {
        const res = await fetch(`/api/anime?slug=${slug}`);
        if (!res.ok) throw new Error('Erreur HTTP ' + res.status);
        const anime = await res.json();
        State.currentAnime = anime;

        DOM.modalTitle.textContent = anime.title;
        DOM.modalSynopsis.textContent = anime.synopsis || 'Aucun synopsis disponible.';
        DOM.modalHeroBg.style.backgroundImage = `url('${anime.banner}')`;
        DOM.modalSourceLink.href = `https://anime-sama.to/catalogue/${anime.slug}/`;

        
        DOM.modalGenres.innerHTML = (anime.genres || []).map(g => `<span class="genre-tag">${g}</span>`).join('');

        
        const inList = State.userData.watchlist.includes(anime.slug);
        updateWatchlistButton(DOM.modalWatchlistBtn, inList);
        DOM.modalWatchlistBtn.onclick = () => toggleWatchlist(anime.slug);

        
        if (DOM.modalYear) DOM.modalYear.textContent = '-';
        if (DOM.modalEpCount) DOM.modalEpCount.textContent = 'Épisodes';
        if (DOM.modalAgeRating) DOM.modalAgeRating.textContent = '-';

        
        updateModalPlayButton(anime);

        
        const seasons = anime.seasons || [{ name: 'Saison 1', path: 'saison1/vostfr' }];
        DOM.modalSeasonSelect.innerHTML = seasons.map(s => `
            <option value="${s.path}">${s.name}</option>
        `).join('');

        DOM.modalSeasonSelect.onchange = () => {
            const targetPath = getSeasonPathForLang(DOM.modalSeasonSelect.value, State.selectedLanguage);
            loadSeasonEpisodes(anime.slug, targetPath);
        };

        
        State.episodesMetadata = null;
        enrichAnimeMetadata(anime.title, anime.slug);

        
        const initialPath = getSeasonPathForLang(seasons[0].path, State.selectedLanguage);
        await loadSeasonEpisodes(anime.slug, initialPath);
    } catch (e) {
        console.error('Erreur chargement détails anime:', e);
        DOM.modalSynopsis.textContent = 'Impossible de charger les épisodes en ligne pour le moment.';
    }
}

function getSeasonPathForLang(path, lang) {
    if (!path) return 'saison1/vostfr';
    if (path.startsWith('scan')) return path;
    const seasonBase = path.split('/')[0] || 'saison1';
    return `${seasonBase}/${lang || 'vostfr'}`;
}

function switchModalLanguage(newLang) {
    if (!State.currentAnime) return;
    const seasonSelectVal = DOM.modalSeasonSelect.value;
    const seasonBase = (State.currentEpisodes ? State.currentEpisodes.season : seasonSelectVal).split('/')[0];
    const newPath = `${seasonBase}/${newLang.toLowerCase()}`;
    State.selectedLanguage = newLang.toLowerCase();
    loadSeasonEpisodes(State.currentAnime.slug, newPath);
}

async function loadSeasonEpisodes(slug, seasonPath) {
    DOM.modalEpisodesGrid.innerHTML = '<div class="episodes-loading">Chargement des épisodes...</div>';
    DOM.modalPlayerTabs.innerHTML = '';

    try {
        const res = await fetch(`/api/episodes?slug=${slug}&season=${encodeURIComponent(seasonPath)}`);
        if (!res.ok) throw new Error('Erreur chargement épisodes');
        const data = await res.json();
        State.currentEpisodes = data;

        
        const currentSeasonBase = (data.season || seasonPath).split('/')[0];
        for (const opt of DOM.modalSeasonSelect.options) {
            if (opt.value.split('/')[0] === currentSeasonBase) {
                DOM.modalSeasonSelect.value = opt.value;
                break;
            }
        }

        
        const availableLangs = data.available_languages && data.available_languages.length
            ? data.available_languages
            : [seasonPath.split('/')[1] || 'vostfr'];

        const currentLang = (data.season.split('/')[1] || seasonPath.split('/')[1] || 'vostfr').toLowerCase();
        State.selectedLanguage = currentLang;

        
        if (DOM.modalLanguages) {
            DOM.modalLanguages.textContent = availableLangs.map(formatLanguageName).join(', ');
        }

        
        if (DOM.modalLanguageTabs) {
            DOM.modalLanguageTabs.innerHTML = availableLangs.map(lang => `
                <button class="lang-tab-btn ${lang.toLowerCase() === currentLang ? 'active' : ''}" onclick="switchModalLanguage('${escapeHtml(lang)}')">
                    ${escapeHtml(formatLanguageName(lang))}
                </button>
            `).join('');
        }

        const players = Object.keys(data.players || {});
        if (!players.length) {
            DOM.modalEpisodesGrid.innerHTML = '<div class="episodes-loading">Aucun lecteur d\'épisodes trouvé pour cette langue/saison.</div>';
            return;
        }

        State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);

        
        DOM.modalPlayerTabs.innerHTML = players.map(p => `
            <button class="player-tab-btn ${p === State.selectedPlayer ? 'active' : ''}" onclick="selectPlayer('${escapeHtml(p)}')">
                ${escapeHtml(p)}
            </button>
        `).join('');

        renderEpisodesGrid();
    } catch (e) {
        DOM.modalEpisodesGrid.innerHTML = '<div class="episodes-loading">Épisodes non disponibles pour cette langue/saison.</div>';
    }
}

const PREFERRED_PLAYERS = ['Uqload', 'VOE', 'VidMoly', 'Sibnet', 'Sendvid', 'Earnvids', 'DoodStream', 'StreamTape', 'OneUpload', 'FileMoon'];

function pickBestPlayer(availablePlayers, currentSelection) {
    if (currentSelection && availablePlayers.includes(currentSelection)) {
        return currentSelection;
    }
    for (const pref of PREFERRED_PLAYERS) {
        const found = availablePlayers.find(p => p.toLowerCase().includes(pref.toLowerCase()));
        if (found) return found;
    }
    return availablePlayers[0] || null;
}

function updateAppTitle(isPlaying) {
    if (isPlaying && State.currentAnime) {
        const title = State.currentAnime.title || State.currentAnime.slug;
        const ep = State.currentEpIndex + 1;
        const lang = (State.selectedLanguage || 'vostfr').toUpperCase();
        document.title = `Anime-Sama — ${title} — Épisode ${ep} (${lang})`;
    } else {
        document.title = 'Anime-Sama';
    }
}

function selectPlayer(playerName) {
    State.selectedPlayer = playerName;
    const tabs = DOM.modalPlayerTabs.querySelectorAll('.player-tab-btn');
    tabs.forEach(t => t.classList.toggle('active', t.textContent.trim() === playerName));
    if (DOM.playerLecteurSelect) {
        DOM.playerLecteurSelect.value = playerName;
    }
    renderEpisodesGrid();
}

function switchEpisodesView(view) {
    State.episodesView = view;
    const listBtn = document.getElementById('viewListBtn');
    const gridBtn = document.getElementById('viewGridBtn');
    if (listBtn) listBtn.classList.toggle('active', view === 'list');
    if (gridBtn) gridBtn.classList.toggle('active', view === 'compact');
    renderEpisodesGrid();
}

function renderEpisodesGrid() {
    if (!State.currentEpisodes || !State.selectedPlayer) return;
    const urls = State.currentEpisodes.players[State.selectedPlayer] || [];

    if (!urls.length) {
        DOM.modalEpisodesGrid.innerHTML = '<div class="episodes-loading">Aucun lien disponible sur ce lecteur.</div>';
        return;
    }

    const view = State.episodesView || 'list';
    DOM.modalEpisodesGrid.className = view === 'list' ? 'episodes-container ep-list-mode' : 'episodes-container ep-grid-mode';

    const slug = State.currentEpisodes.slug;
    const season = State.currentEpisodes.season;
    const anime = State.currentAnime || {};
    const bannerUrl = anime.banner || `/api/img?slug=${encodeURIComponent(slug)}`;

    
    const historyItem = (State.userData.history || []).find(h => h.slug === slug);
    const lastWatchedEp = historyItem ? historyItem.episode : 0;

    if (view === 'compact') {
        DOM.modalEpisodesGrid.innerHTML = urls.map((url, idx) => {
            const epNum = idx + 1;
            const isWatched = epNum <= lastWatchedEp;
            const isCurrent = epNum === lastWatchedEp;
            const epProg = getEpisodeProgress(slug, epNum);
            let progressSeconds = 0;
            if (epProg && epProg.currentTime > 5) {
                if (epProg.duration && epProg.duration > 60 && epProg.currentTime >= epProg.duration - 20) {
                    progressSeconds = 0;
                } else {
                    progressSeconds = epProg.currentTime;
                }
            }
            return `
                <button class="ep-btn ${isWatched ? 'watched' : ''} ${isCurrent ? 'active' : ''}" 
                        onclick="openEpisodePlayer('${escapeHtml(slug)}', '${escapeHtml(season)}', ${idx})"
                        title="Épisode ${epNum}">
                    Ep ${epNum}
                </button>
            `;
        }).join('');
        return;
    }

    
    DOM.modalEpisodesGrid.innerHTML = urls.map((url, idx) => {
        const epNum = idx + 1;
        const isWatched = epNum <= lastWatchedEp;
        const isCurrent = epNum === lastWatchedEp;
        const isScan = season.toLowerCase().startsWith('scan');

        
        const epMeta = (State.episodesMetadata && State.episodesMetadata[epNum]) ? State.episodesMetadata[epNum] : null;

        const epThumb = (epMeta && epMeta.thumbnail) ? epMeta.thumbnail : bannerUrl;
        const epTitle = (epMeta && epMeta.title) ? epMeta.title : `Épisode ${epNum}`;
        const epDuration = isScan ? 'Scan' : ((epMeta && epMeta.length) ? `${epMeta.length} min` : '24 min');

        
        const seasonNum = (season.split('/')[0] || 'saison1').replace(/[^0-9]/g, '') || '1';
        const epDesc = (epMeta && epMeta.synopsis && epMeta.synopsis.trim().length > 0)
            ? epMeta.synopsis
            : `Épisode ${epNum} • Saison ${seasonNum}.`;

        
        const epProg = getEpisodeProgress(slug, epNum);
        let progressPercent = 0;
        let progressSeconds = 0;
        if (epProg && epProg.currentTime > 5) {
            if (epProg.duration && epProg.duration > 60 && epProg.currentTime >= epProg.duration - 20) {
                progressSeconds = 0;
                progressPercent = 100;
            } else {
                progressSeconds = epProg.currentTime;
                progressPercent = Math.min(100, Math.round((epProg.currentTime / (epProg.duration || 1440)) * 100));
            }
        } else if (isWatched) {
            progressPercent = 100;
        }

        return `
            <div class="ep-card ${isCurrent ? 'active' : ''} ${isWatched ? 'watched' : ''}" 
                 onclick="openEpisodePlayer('${escapeHtml(slug)}', '${escapeHtml(season)}', ${idx})">
                <div class="ep-card-num">${epNum}</div>
                <div class="ep-card-thumb-wrap">
                    <img class="ep-card-thumb" 
                         src="${escapeHtml(epThumb)}" 
                         alt="Épisode ${epNum}" 
                         loading="lazy" 
                         onerror="this.onerror=null; this.src='${escapeHtml(bannerUrl)}';" />
                    <div class="ep-card-play-overlay">
                        <div class="ep-card-play-icon">
                            <svg viewBox="0 0 24 24" fill="currentColor"><polygon points="6 3 20 12 6 21"></polygon></svg>
                        </div>
                    </div>
                    <span class="ep-card-thumb-badge">EP ${epNum}</span>
                    ${progressPercent > 0 ? `<div class="ep-card-progress"><div class="ep-card-progress-fill" style="width: ${progressPercent}%;"></div></div>` : ''}
                </div>
                <div class="ep-card-info">
                    <div class="ep-card-header">
                        <span class="ep-card-title">${escapeHtml(epTitle)}</span>
                        <span class="ep-card-duration">${escapeHtml(epDuration)}</span>
                    </div>
                    <p class="ep-card-desc">${escapeHtml(epDesc)}</p>
                </div>
            </div>
        `;
    }).join('');
}

function closeDetailModal() {
    DOM.detailModal.classList.remove('open');
    document.body.style.overflow = 'auto';
}


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
                showToast('Lien d\'épisode indisponible');
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
        DOM.playerEpisodeTitle.textContent = `Épisode ${episodeIndex + 1}`;

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


async function performSearch(query) {
    DOM.searchStatus.textContent = `Recherche pour "${query}"...`;
    try {
        const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
        if (!res.ok) throw new Error('Erreur HTTP ' + res.status);
        const data = await res.json();
        const results = data.results || [];

        DOM.searchStatus.textContent = `${results.length} résultat(s) pour "${query}"`;
        DOM.searchGrid.innerHTML = results.length
            ? results.map(c => createCardHTML(c)).join('')
            : '<p style="color:#888; grid-column:1/-1;">Aucun résultat trouvé pour votre recherche.</p>';
    } catch (e) {
        console.error('Erreur recherche:', e);
        DOM.searchStatus.textContent = 'Erreur lors de la recherche.';
    }
}

function filterByGenre(genre) {
    document.getElementById('viewGrid').classList.add('active');
    DOM.gridTitle.textContent = `Genre : ${genre}`;
    DOM.gridSubtitle.textContent = `Toutes les œuvres associées au genre ${genre}`;

    const filtered = State.catalogue.filter(c => 
        (c.genres || []).some(g => g.toLowerCase().includes(genre.toLowerCase())) ||
        (c.badges || []).some(b => b.toLowerCase().includes(genre.toLowerCase()))
    );

    DOM.catalogueGrid.innerHTML = filtered.length
        ? filtered.map(c => createCardHTML(c)).join('')
        : '<p style="color:#888; grid-column:1/-1;">Aucune œuvre trouvée pour ce genre.</p>';
}


async function refreshAccountStatus(verify) {
    try {
        const res = await fetch(`/api/auth/status${verify ? '?verify=1' : ''}`);
        const data = await res.json();
        applyAccountStatus(data);
    } catch (e) {
        console.error('Erreur statut compte:', e);
    }
}

function applyAccountStatus(data) {
    State.loggedIn = !!data.logged_in;
    DOM.accountPseudo.textContent = State.loggedIn ? (data.pseudo || '') : '';
}

async function openAccountModal() {
    DOM.accountModal.classList.add('open');
    document.body.style.overflow = 'hidden';

    fetch('/api/settings')
        .then(r => r.json())
        .then(d => {
            DOM.autoSyncCheckbox.checked = !!d.auto_sync;
            DOM.adShieldCheckbox.checked = d.ad_shield !== false;
        })
        .catch(() => {});

    
    await refreshAccountStatus(true);
    switchAccountMode(State.loggedIn ? 'cloud' : 'local');
}

function closeAccountModal() {
    DOM.accountModal.classList.remove('open');
    document.body.style.overflow = 'auto';
}

function switchAccountMode(mode) {
    DOM.modeLocalBtn.classList.toggle('active', mode === 'local');
    DOM.modeCloudBtn.classList.toggle('active', mode === 'cloud');
    DOM.panelLocal.style.display = mode === 'local' ? 'block' : 'none';

    if (mode === 'local') {
        DOM.panelCloudLoggedOut.style.display = 'none';
        DOM.panelCloudLoggedIn.style.display = 'none';
        return;
    }

    DOM.panelCloudLoggedOut.style.display = State.loggedIn ? 'none' : 'block';
    DOM.panelCloudLoggedIn.style.display = State.loggedIn ? 'block' : 'none';
}

function toggleAccountForms(which) {
    DOM.formAccountLogin.classList.toggle('active', which === 'login');
    DOM.formAccountRegister.classList.toggle('active', which === 'register');
    DOM.accountError.style.display = 'none';
}

function showAccountError(message) {
    DOM.accountError.textContent = message;
    DOM.accountError.style.display = 'block';
}

async function handleAccountLogin(e) {
    e.preventDefault();
    const pseudo = document.getElementById('accountLoginPseudo').value.trim();
    const password = document.getElementById('accountLoginPassword').value;

    try {
        const res = await fetch('/api/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ pseudo, password })
        });
        const data = await res.json();
        if (!res.ok) {
            showAccountError(data.error || 'Erreur de connexion');
            return;
        }
        DOM.accountError.style.display = 'none';
        applyAccountStatus({ logged_in: true, pseudo: data.pseudo });
        switchAccountMode('cloud');
        showToast('Connecté à anime-sama.to');
    } catch (err) {
        showAccountError('Erreur de connexion au serveur local.');
    }
}

async function handleAccountRegister(e) {
    e.preventDefault();
    const pseudo = document.getElementById('accountRegisterPseudo').value.trim();
    const password = document.getElementById('accountRegisterPassword').value;
    const password2 = document.getElementById('accountRegisterPassword2').value;

    if (password !== password2) {
        showAccountError('Les mots de passe ne correspondent pas');
        return;
    }

    try {
        const res = await fetch('/api/auth/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ pseudo, password })
        });
        const data = await res.json();
        if (!res.ok) {
            showAccountError(data.error || 'Erreur de création de compte');
            return;
        }
        DOM.accountError.style.display = 'none';
        applyAccountStatus({ logged_in: true, pseudo: data.pseudo });
        switchAccountMode('cloud');
        showToast('Compte créé et connecté');
    } catch (err) {
        showAccountError('Erreur de connexion au serveur local.');
    }
}

async function handleAccountLogout() {
    try {
        await fetch('/api/auth/logout', { method: 'POST' });
    } catch (e) {}
    applyAccountStatus({ logged_in: false, pseudo: null });
    switchAccountMode('cloud');
    showToast('Déconnecté');
}

async function handleAutoSyncToggle(e) {
    const enabled = e.target.checked;
    try {
        await fetch('/api/settings/auto-sync', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ enabled })
        });
        showToast(enabled ? 'Synchronisation automatique activée' : 'Synchronisation automatique désactivée');
        if (enabled) runSync('auto');
    } catch (err) {
        showToast('Impossible de sauvegarder ce réglage');
        e.target.checked = !enabled;
    }
}

async function handleAdShieldToggle(e) {
    const enabled = e.target.checked;
    try {
        await fetch('/api/settings/ad-shield', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ enabled })
        });
        showToast(enabled ? 'Ad Shield activé' : 'Ad Shield désactivé');
    } catch (err) {
        showToast('Impossible de sauvegarder ce réglage');
        e.target.checked = !enabled;
    }
}

async function runSync(direction = 'auto') {
    DOM.syncAutoBtn.disabled = true;
    DOM.syncAutoBtn.classList.add('syncing');
    DOM.syncStatus.textContent = 'Synchronisation en cours...';

    try {
        const res = await fetch(`/api/sync/${direction}`, { method: 'POST' });
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || 'Erreur de synchronisation');

        if (data.userdata) {
            State.userData = data.userdata;
            updateWatchlistCount();
        }

        DOM.syncStatus.textContent = `Synchronisé : ${data.updated_local} en local, ${data.updated_cloud} vers le cloud.`;
        showToast('Synchronisation réussie');
    } catch (e) {
        DOM.syncStatus.textContent = 'Échec de la synchronisation : ' + e.message;
        showToast('Échec de la synchronisation');
    } finally {
        DOM.syncAutoBtn.disabled = false;
        DOM.syncAutoBtn.classList.remove('syncing');
    }
}


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

    
    DOM.playerCloseBtn.addEventListener('click', closePlayerModal);
    DOM.playerPrevBtn.addEventListener('click', () => changeEpisode(-1));
    DOM.playerNextBtn.addEventListener('click', () => changeEpisode(1));

    
    document.addEventListener('keydown', (e) => {
        if (DOM.playerModal && DOM.playerModal.classList.contains('open')) {
            const tag = (e.target && e.target.tagName || '').toUpperCase();
            if (tag === 'INPUT' || tag === 'TEXTAREA' || (e.target && e.target.isContentEditable)) {
                return;
            }
            if (e.key === 'Escape') {
                closePlayerModal();
                return;
            }
            if (e.code === 'Space' || e.key === ' ' || e.key === 'k' || e.key === 'K') {
                e.preventDefault();
                try {
                    DOM.videoIframe.contentWindow.postMessage({ type: 'AS_TOGGLE_PLAY' }, '*');
                } catch(err) {}
                return;
            }
            if (e.code === 'ArrowLeft' || e.key === 'ArrowLeft') {
                e.preventDefault();
                try {
                    DOM.videoIframe.contentWindow.postMessage({ type: 'AS_SEEK_RELATIVE', delta: -5 }, '*');
                } catch(err) {}
                return;
            }
            if (e.code === 'ArrowRight' || e.key === 'ArrowRight') {
                e.preventDefault();
                try {
                    DOM.videoIframe.contentWindow.postMessage({ type: 'AS_SEEK_RELATIVE', delta: 5 }, '*');
                } catch(err) {}
                return;
            }
        } else {
            if (e.key === 'Escape') {
                if (DOM.detailModal.classList.contains('open')) {
                    closeDetailModal();
                } else if (DOM.accountModal.classList.contains('open')) {
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
