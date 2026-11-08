const animeMetadataCache = {};

async function fetchAnimeAndEpisodesMeta(title, slug, seasonBase = 'saison1') {
    const cleanSeason = (seasonBase || 'saison1').toLowerCase();
    const seasonNum = cleanSeason.replace(/[^0-9]/g, '') || '1';
    const key = `${(title || slug || '').toLowerCase().trim()}_s${seasonNum}`;
    if (!title && !slug) return null;
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
        const cleanSearch = (title || slug).replace(/saison\s*\d+/gi, '').replace(/\(tv\)/gi, '').trim();
        const searchTerms = seasonNum === '1' 
            ? [cleanSearch] 
            : [`${cleanSearch} Season ${seasonNum}`, `${cleanSearch} ${seasonNum}`, cleanSearch];

        let animeObj = null;
        for (const term of searchTerms) {
            const searchRes = await fetch(`https://kitsu.io/api/edge/anime?filter[text]=${encodeURIComponent(term)}&page[limit]=3`);
            if (searchRes.ok) {
                const searchData = await searchRes.json();
                if (searchData.data && searchData.data.length) {
                    if (seasonNum !== '1') {
                        const match = searchData.data.find(item => {
                            const t = ((item.attributes || {}).canonicalTitle || '').toLowerCase();
                            return t.includes(`season ${seasonNum}`) || t.includes(` ${seasonNum}`) || t.includes(`s${seasonNum}`);
                        });
                        animeObj = match || searchData.data[0];
                    } else {
                        animeObj = searchData.data[0];
                    }
                    if (animeObj) break;
                }
            }
        }

        if (!animeObj) return null;

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
                        title: epAttrs.canonicalTitle || `${t('modal.episode_label', 'Épisode')} ${num}`,
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

async function enrichAnimeMetadata(title, slug, seasonBase = 'saison1') {
    const meta = await fetchAnimeAndEpisodesMeta(title, slug, seasonBase);
    if (!meta || !State.currentAnime || State.currentAnime.slug !== slug) return;

    if (meta.year && DOM.modalYear) DOM.modalYear.textContent = meta.year;
    if (meta.episodeCount && DOM.modalEpCount) DOM.modalEpCount.textContent = `${meta.episodeCount} ${t('modal.episodes', 'Épisodes')}`;
    if (meta.ageRating && DOM.modalAgeRating) DOM.modalAgeRating.textContent = meta.ageRating;

    State.episodesMetadata = meta.episodes || {};

    renderEpisodesGrid();
    updateModalPlayButton(State.currentAnime);
}

function updateModalPlayButton(anime) {
    if (!anime) anime = State.currentAnime;
    if (!anime) return;

    const activeSeason = (DOM.modalSeasonSelect && DOM.modalSeasonSelect.value)
        ? DOM.modalSeasonSelect.value
        : (State.currentEpisodes ? State.currentEpisodes.season : 'saison1/vostfr');
    const currentSeasonBase = (activeSeason || 'saison1').split('/')[0];
    const seasonNum = currentSeasonBase.replace(/[^0-9]/g, '') || '1';

    const savedProgress = getPlaybackProgress(anime.slug);
    const progForThisSeason = (savedProgress && savedProgress.season && savedProgress.season.split('/')[0] === currentSeasonBase)
        ? savedProgress
        : null;

    const historyItem = (State.userData.history || []).find(h => h.slug === anime.slug);
    const historyForThisSeason = (historyItem && historyItem.season && historyItem.season.split('/')[0] === currentSeasonBase)
        ? historyItem
        : null;

    const lastWatchedEp = progForThisSeason ? progForThisSeason.episode : (historyForThisSeason ? historyForThisSeason.episode : 0);

    if (progForThisSeason && progForThisSeason.currentTime > 5) {
        const curMins = Math.floor(progForThisSeason.currentTime / 60);
        const totalMins = Math.floor((progForThisSeason.duration || 1440) / 60);
        const pct = Math.min(100, Math.round((progForThisSeason.currentTime / (progForThisSeason.duration || 1440)) * 100));

        if (DOM.modalHeroProgress) {
            DOM.modalHeroProgress.style.display = 'flex';
            if (DOM.modalHeroProgressLabel) {
                DOM.modalHeroProgressLabel.textContent = `${curMins} sur ${totalMins} min (${t('modal.episode_label', 'Épisode')} ${progForThisSeason.episode})`;
            }
            if (DOM.modalHeroProgressFill) {
                DOM.modalHeroProgressFill.style.width = `${pct}%`;
            }
        }
        if (DOM.modalPlayBtnText) {
            DOM.modalPlayBtnText.textContent = `${t('modal.resume_ep', 'Reprendre Ep')} ${progForThisSeason.episode} (${curMins} min)`;
        }
    } else if (lastWatchedEp > 0) {
        if (DOM.modalHeroProgress) {
            DOM.modalHeroProgress.style.display = 'flex';
            if (DOM.modalHeroProgressLabel) DOM.modalHeroProgressLabel.textContent = `${t('modal.episode_label', 'Épisode')} ${lastWatchedEp}`;
            if (DOM.modalHeroProgressFill) DOM.modalHeroProgressFill.style.width = '75%';
        }
        if (DOM.modalPlayBtnText) DOM.modalPlayBtnText.textContent = `${t('modal.resume_ep', 'Reprendre Ep')} ${lastWatchedEp}`;
    } else {
        if (DOM.modalHeroProgress) DOM.modalHeroProgress.style.display = 'none';
        if (DOM.modalPlayBtnText) DOM.modalPlayBtnText.textContent = t('modal.play', 'Lecture');
    }

    const targetEp = lastWatchedEp > 0 ? lastWatchedEp : 1;
    const targetIdx = Math.max(0, targetEp - 1);

    if (DOM.modalFeaturedEpTitle) {
        const epMeta = (State.episodesMetadata && State.episodesMetadata[targetEp]) ? State.episodesMetadata[targetEp] : null;
        const epTitle = (epMeta && epMeta.title) ? epMeta.title : `${t('modal.episode_label', 'Épisode')} ${targetEp}`;
        DOM.modalFeaturedEpTitle.textContent = `S${seasonNum}:E${targetEp} "${epTitle}"`;
    }

    if (DOM.modalPlayBtn) {
        DOM.modalPlayBtn.onclick = () => {
            openEpisodePlayer(anime.slug, activeSeason, targetIdx, null);
        };
    }
}

const LANG_DISPLAY_NAMES = {
    'vostfr': 'VOSTFR',
    'vf': 'VF',
    'vf1': 'VF 1',
    'vf2': 'VF 2',
    'vqc': 'VQC (Québec)',
    'va': 'VA (Anglais)',
    'vosten': 'VOSTEN (English)',
    'vosteng': 'VOSTEN (English)',
    'vostes': 'VOSTES (Español)',
    'vostde': 'VOSTDE (Deutsch)',
    'vostit': 'VOSTIT (Italiano)',
    'vostpt': 'VOSTPT (Português)',
    'vostru': 'VOSTRU (Русский)',
    'vostkr': 'VOSTKR (한국어)',
    'vostcn': 'VOSTCN (中文)',
    'vostar': 'VOSTAR (العربية)',
    'vj': 'VO (Japonais)',
    'vo': 'VO (Japonais)',
    'raw': 'RAW (Japonais pur)',
    'vkr': 'VKR (Coréen)',
    'vcn': 'VCN (Chinois)',
    'var': 'VAR (Arabe)',
    'ves': 'VES (Espagnol)',
    'vde': 'VDE (Allemand)',
    'vit': 'VIT (Italien)',
    'vpt': 'VPT (Portugais)'
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

    DOM.modalTitle.textContent = t('modal.loading', 'Chargement...');
    DOM.modalSynopsis.textContent = t('modal.loading_details', 'Chargement des détails de l\'œuvre...');
    DOM.modalHeroBg.style.backgroundImage = `url('/api/img?slug=${slug}')`;
    DOM.modalSeasonSelect.innerHTML = `<option>${t('modal.loading_seasons', 'Chargement des saisons...')}</option>`;
    DOM.modalPlayerTabs.innerHTML = '';
    DOM.modalEpisodesGrid.innerHTML = `<div class="episodes-loading">${t('modal.episodes_search', 'Recherche des épisodes en cours...')}</div>`;
    if (DOM.modalHeroProgress) DOM.modalHeroProgress.style.display = 'none';
    if (DOM.modalLanguages) DOM.modalLanguages.textContent = 'Détection...';

    try {
        const res = await fetch(`/api/anime?slug=${slug}`);
        if (!res.ok) throw new Error('Erreur HTTP ' + res.status);
        const anime = await res.json();
        State.currentAnime = anime;

        DOM.modalTitle.textContent = anime.title;
        DOM.modalSynopsis.textContent = anime.synopsis || 'Aucun synopsis disponible.';
        if (State.appLang && State.appLang !== 'fr' && anime.synopsis) {
            translateContent(anime.synopsis, State.appLang).then(translated => {
                if (DOM.modalSynopsis && State.currentAnime && State.currentAnime.slug === slug) {
                    DOM.modalSynopsis.textContent = translated;
                }
            });
        }

        DOM.modalHeroBg.style.backgroundImage = `url('${anime.banner}')`;
        DOM.modalSourceLink.href = `https://anime-sama.to/catalogue/${anime.slug}/`;

        DOM.modalGenres.innerHTML = (anime.genres || []).map(g => `<span class="genre-tag">${g}</span>`).join('');

        const inList = State.userData.watchlist.includes(anime.slug);
        updateWatchlistButton(DOM.modalWatchlistBtn, inList);
        DOM.modalWatchlistBtn.onclick = () => toggleWatchlist(anime.slug);

        if (DOM.modalYear) DOM.modalYear.textContent = '-';
        if (DOM.modalEpCount) DOM.modalEpCount.textContent = t('modal.episodes', 'Épisodes');
        if (DOM.modalAgeRating) DOM.modalAgeRating.textContent = '-';

        const seasons = anime.seasons || [{ name: 'Saison 1', path: 'saison1/vostfr' }];
        DOM.modalSeasonSelect.innerHTML = seasons.map(s => `
            <option value="${s.path}">${s.name}</option>
        `).join('');

        DOM.modalSeasonSelect.onchange = () => {
            const selectedVal = DOM.modalSeasonSelect.value;
            const targetPath = getSeasonPathForLang(selectedVal, State.selectedLanguage);
            loadSeasonEpisodes(anime.slug, targetPath);
        };

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
    DOM.modalEpisodesGrid.innerHTML = `<div class="episodes-loading">${t('modal.loading', 'Chargement des épisodes...')}</div>`;
    DOM.modalPlayerTabs.innerHTML = '';
    State.episodesMetadata = null;

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
            DOM.modalEpisodesGrid.innerHTML = `<div class="episodes-loading">${t('modal.no_players', 'Aucun lecteur d\'épisodes trouvé pour cette langue/saison.')}</div>`;
            return;
        }

        State.selectedPlayer = pickBestPlayer(players, State.selectedPlayer);

        DOM.modalPlayerTabs.innerHTML = players.map(p => `
            <button class="player-tab-btn ${p === State.selectedPlayer ? 'active' : ''}" onclick="selectPlayer('${escapeHtml(p)}')">
                ${escapeHtml(p)}
            </button>
        `).join('');

        renderEpisodesGrid();
        updateModalPlayButton(State.currentAnime);

        if (State.currentAnime) {
            enrichAnimeMetadata(State.currentAnime.title, State.currentAnime.slug, currentSeasonBase);
        }
    } catch (e) {
        DOM.modalEpisodesGrid.innerHTML = `<div class="episodes-loading">${t('modal.episodes_unavailable', 'Épisodes non disponibles pour cette langue/saison.')}</div>`;
    }
}

const PREFERRED_PLAYERS = ['Uqload', 'VOE', 'VidMoly', 'Sibnet', 'Sendvid', 'Earnvids', 'DoodStream', 'StreamTape', 'OneUpload', 'FileMoon'];

function pickBestPlayer(players, current) {
    if (!players || !players.length) return null;
    if (current && players.includes(current)) return current;
    for (const pref of PREFERRED_PLAYERS) {
        const found = players.find(p => p.toLowerCase().includes(pref.toLowerCase()));
        if (found) return found;
    }
    return players[0];
}

function selectPlayer(player) {
    State.selectedPlayer = player;
    document.querySelectorAll('.player-tab-btn').forEach(btn => {
        btn.classList.toggle('active', btn.textContent.trim() === player);
    });
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
        DOM.modalEpisodesGrid.innerHTML = `<div class="episodes-loading">${t('modal.no_links', 'Aucun lien disponible sur ce lecteur.')}</div>`;
        return;
    }

    const view = State.episodesView || 'list';
    DOM.modalEpisodesGrid.className = view === 'list' ? 'episodes-container ep-list-mode' : 'episodes-container ep-grid-mode';

    const slug = State.currentEpisodes.slug;
    const season = State.currentEpisodes.season;
    const anime = State.currentAnime || {};
    const bannerUrl = anime.banner || `/api/img?slug=${encodeURIComponent(slug)}`;
    const currentSeasonBase = (season || 'saison1').split('/')[0];
    const seasonNum = currentSeasonBase.replace(/[^0-9]/g, '') || '1';

    const historyItem = (State.userData.history || []).find(h => h.slug === slug && (!h.season || h.season.split('/')[0] === currentSeasonBase));
    const lastWatchedEp = historyItem ? historyItem.episode : 0;

    if (view === 'compact') {
        DOM.modalEpisodesGrid.innerHTML = urls.map((url, idx) => {
            const epNum = idx + 1;
            const isWatched = epNum <= lastWatchedEp;
            const isCurrent = epNum === lastWatchedEp;
            return `
                <button class="ep-btn ${isWatched ? 'watched' : ''} ${isCurrent ? 'active' : ''}" 
                        onclick="openEpisodePlayer('${escapeHtml(slug)}', '${escapeHtml(season)}', ${idx})"
                        title="${t('modal.episode_label', 'Épisode')} ${epNum}">
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
        const epTitle = (epMeta && epMeta.title) ? epMeta.title : `${t('modal.episode_label', 'Épisode')} ${epNum}`;
        const epDuration = isScan ? 'Scan' : ((epMeta && epMeta.length) ? `${epMeta.length} min` : '24 min');

        const defaultDesc = `${t('modal.episode_label', 'Épisode')} ${epNum} • ${t('modal.season_label', 'Saison')} ${seasonNum}.`;
        const epDesc = (epMeta && epMeta.synopsis && epMeta.synopsis.trim().length > 0)
            ? epMeta.synopsis
            : defaultDesc;

        const epProg = getEpisodeProgress(slug, epNum);
        let progressPercent = 0;
        if (epProg && epProg.currentTime > 5) {
            if (epProg.duration && epProg.duration > 60 && epProg.currentTime >= epProg.duration - 20) {
                progressPercent = 100;
            } else {
                progressPercent = Math.min(100, Math.round((epProg.currentTime / (epProg.duration || 1440)) * 100));
            }
        } else if (isWatched) {
            progressPercent = 100;
        }

        return `
            <div class="ep-card ${isCurrent ? 'active' : ''} ${isWatched ? 'watched' : ''}" 
                 data-ep="${epNum}"
                 onclick="openEpisodePlayer('${escapeHtml(slug)}', '${escapeHtml(season)}', ${idx})">
                <div class="ep-card-num">${epNum}</div>
                <div class="ep-card-thumb-wrap">
                    <img class="ep-card-thumb" 
                         src="${escapeHtml(epThumb)}" 
                         alt="${t('modal.episode_label', 'Épisode')} ${epNum}" 
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

    if (State.appLang && State.appLang !== 'fr') {
        urls.forEach((_, idx) => {
            const epNum = idx + 1;
            const epMeta = (State.episodesMetadata && State.episodesMetadata[epNum]) ? State.episodesMetadata[epNum] : null;
            if (epMeta && epMeta.synopsis && epMeta.synopsis.trim().length > 0) {
                translateContent(epMeta.synopsis, State.appLang).then(translated => {
                    const cardDesc = DOM.modalEpisodesGrid.querySelector(`.ep-card[data-ep="${epNum}"] .ep-card-desc`);
                    if (cardDesc) cardDesc.textContent = translated;
                });
            }
        });
    }
}

function closeDetailModal() {
    DOM.detailModal.classList.remove('open');
    document.body.style.overflow = 'auto';
}
