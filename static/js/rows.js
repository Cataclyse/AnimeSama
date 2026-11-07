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
        appendRow(t('row.trending', 'À l\'affiche & Tendances'), trendingCards);
    }

    const sections = State.homeData.sections || [];
    sections.forEach(sec => {
        appendRow(formatSectionTitle(sec.title), sec.cards);
    });
}

function formatSectionTitle(rawTitle) {
    const raw = (rawTitle || '').toLowerCase();
    if (raw.includes('derniers épisodes')) return t('row.recent_episodes', 'Derniers épisodes ajoutés');
    if (raw.includes('derniers scans')) return t('row.recent_scans', 'Derniers scans & webtoons');
    if (raw.includes('derniers contenus')) return t('row.recent_content', 'Nouveautés sorties');
    if (raw.includes('classiques')) return t('row.classics', 'Les Classiques légendaires');
    if (raw.includes('pépites')) return t('row.gems', 'Pépites à découvrir');
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
            <a href="#" class="row-see-all" onclick="showCategoryGrid('${escapeHtml(title)}'); return false;">${t('row.see_all', 'Tout voir')} ›</a>
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
                <div class="card-trailer-badge">${t('badge.preview', 'Aperçu')}</div>
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
            <button class="card-watchlist-btn ${inList ? 'in-list' : ''}" data-card-watchlist="${card.slug}" onclick="event.stopPropagation(); toggleWatchlist('${card.slug}');" title="${inList ? t('btn.remove_watchlist', 'Retirer de ma liste') : t('btn.add_watchlist', 'Ajouter à ma liste')}" aria-label="${t('nav.watchlist', 'Ma Liste')}">
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
                    <span class="card-badge">${isWebtoon ? t('badge.scan', 'Scan') : t('badge.anime', 'Anime')}</span>
                    ${flagBadges}
                    ${card.etat ? `<span class="card-badge">${escapeHtml(card.etat)}</span>` : ''}
                </div>
                ${card.info ? `<div class="card-info-text">${escapeHtml(card.info)}</div>` : ''}
            </div>
        </div>
    `;
}
