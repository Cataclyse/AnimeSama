const I18N_LANGUAGES = {
    'fr': {
        name: 'Français',
        code: 'FR',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="1" height="2" fill="#002395"/><rect width="1" height="2" x="1" fill="#fff"/><rect width="1" height="2" x="2" fill="#ed2939"/></svg>'
    },
    'en': {
        name: 'English',
        code: 'EN',
        flag: '<svg viewBox="0 0 60 30" class="lang-flag-svg"><rect width="60" height="30" fill="#012169"/><path d="M0,0 L60,30 M60,0 L0,30" stroke="#fff" stroke-width="6"/><path d="M0,0 L60,30 M60,0 L0,30" stroke="#c8102e" stroke-width="3"/><path d="M30,0 v30 M0,15 h60" stroke="#fff" stroke-width="10"/><path d="M30,0 v30 M0,15 h60" stroke="#c8102e" stroke-width="6"/></svg>'
    },
    'ja': {
        name: '日本語',
        code: 'JA',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="3" height="2" fill="#fff"/><circle cx="1.5" cy="1" r="0.6" fill="#bc002d"/></svg>'
    },
    'es': {
        name: 'Español',
        code: 'ES',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="3" height="0.5" y="0" fill="#aa151b"/><rect width="3" height="1" y="0.5" fill="#f1bf00"/><rect width="3" height="0.5" y="1.5" fill="#aa151b"/></svg>'
    },
    'de': {
        name: 'Deutsch',
        code: 'DE',
        flag: '<svg viewBox="0 0 5 3" class="lang-flag-svg"><rect width="5" height="1" y="0" fill="#000"/><rect width="5" height="1" y="1" fill="#d00"/><rect width="5" height="1" y="2" fill="#ffce00"/></svg>'
    },
    'it': {
        name: 'Italiano',
        code: 'IT',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="1" height="2" fill="#009246"/><rect width="1" height="2" x="1" fill="#fff"/><rect width="1" height="2" x="2" fill="#ce2b37"/></svg>'
    },
    'pt': {
        name: 'Português',
        code: 'PT',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="1.2" height="2" fill="#046a38"/><rect width="1.8" height="2" x="1.2" fill="#da291c"/><circle cx="1.2" cy="1" r="0.4" fill="#ffdd00"/></svg>'
    },
    'ar': {
        name: 'العربية',
        code: 'AR',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="3" height="2" fill="#006c35"/><rect width="1.6" height="0.15" x="0.7" y="1.3" fill="#fff"/><circle cx="1.5" cy="0.8" r="0.3" fill="#fff"/></svg>'
    },
    'ru': {
        name: 'Русский',
        code: 'RU',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="3" height="0.667" y="0" fill="#fff"/><rect width="3" height="0.667" y="0.667" fill="#0039a6"/><rect width="3" height="0.667" y="1.333" fill="#d52b1e"/></svg>'
    },
    'ko': {
        name: '한국어',
        code: 'KO',
        flag: '<svg viewBox="0 0 3 2" class="lang-flag-svg"><rect width="3" height="2" fill="#fff"/><circle cx="1.5" cy="1" r="0.5" fill="#cd2e3a"/><path d="M1.5,0.5 a0.5,0.5 0 0,0 0,1 a0.25,0.25 0 0,0 0,-0.5 a0.25,0.25 0 0,1 0,-0.5" fill="#0047a0"/></svg>'
    }
};

const I18N_CACHE = {};
let currentI18nStrings = {};

function parseLangFile(text) {
    const dict = {};
    if (!text) return dict;
    const lines = text.split('\n');
    for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;
        const eqIdx = trimmed.indexOf('=');
        if (eqIdx > 0) {
            const key = trimmed.slice(0, eqIdx).trim();
            const val = trimmed.slice(eqIdx + 1).trim();
            dict[key] = val;
        }
    }
    return dict;
}

async function fetchLanguageData(code) {
    if (I18N_CACHE[code]) return I18N_CACHE[code];
    try {
        const res = await fetch(`/lang/${code}.lang`);
        if (res.ok) {
            const text = await res.text();
            const dict = parseLangFile(text);
            I18N_CACHE[code] = dict;
            return dict;
        }
    } catch (_) {}
    return null;
}

const dynamicTranslationCache = {};

async function translateContent(text, targetLang) {
    if (!text || typeof text !== 'string') return text || '';
    const clean = text.trim();
    if (!clean) return clean;
    const lang = (targetLang || (typeof State !== 'undefined' ? State.appLang : null) || 'fr').toLowerCase();
    if (lang === 'fr') return clean;

    const cacheKey = `${lang}:${clean}`;
    if (dynamicTranslationCache[cacheKey]) {
        return dynamicTranslationCache[cacheKey];
    }

    try {
        const res = await fetch(`/api/translate?target=${encodeURIComponent(lang)}&text=${encodeURIComponent(clean)}`);
        if (res.ok) {
            const data = await res.json();
            if (data.translated && data.translated.trim()) {
                dynamicTranslationCache[cacheKey] = data.translated;
                return data.translated;
            }
        }
    } catch (_) {}

    return clean;
}

function t(key, fallback) {
    if (currentI18nStrings && key in currentI18nStrings) {
        return currentI18nStrings[key];
    }
    return fallback || key;
}

function toggleNavLangDropdown(e) {
    if (e) e.stopPropagation();
    const menu = document.getElementById('navLangMenu');
    if (menu) {
        menu.classList.toggle('open');
    }
}

function selectLanguage(code) {
    setAppLanguage(code);
    const menu = document.getElementById('navLangMenu');
    if (menu) menu.classList.remove('open');
}

function renderLangSelectors() {
    const active = State.appLang || 'fr';
    const activeInfo = I18N_LANGUAGES[active] || I18N_LANGUAGES['fr'];

    const flagSlot = document.getElementById('navLangCurrentFlag');
    if (flagSlot) flagSlot.innerHTML = activeInfo.flag;

    const codeSlot = document.getElementById('navLangCurrentCode');
    if (codeSlot) codeSlot.textContent = activeInfo.code;

    const navMenu = document.getElementById('navLangMenu');
    if (navMenu) {
        navMenu.innerHTML = Object.entries(I18N_LANGUAGES).map(([code, item]) => `
            <button class="custom-lang-item ${code === active ? 'active' : ''}" onclick="selectLanguage('${code}')">
                <span class="custom-lang-flag-wrap">${item.flag}</span>
                <span class="custom-lang-label">${item.name}</span>
                <span class="custom-lang-tag">(${item.code})</span>
            </button>
        `).join('');
    }

    const settingsGrid = document.getElementById('settingsLangGrid');
    if (settingsGrid) {
        settingsGrid.innerHTML = Object.entries(I18N_LANGUAGES).map(([code, item]) => `
            <button class="settings-lang-pill ${code === active ? 'active' : ''}" onclick="selectLanguage('${code}')">
                <span class="custom-lang-flag-wrap">${item.flag}</span>
                <span class="settings-lang-name">${item.name}</span>
                ${code === active ? '<span class="settings-lang-check">✓</span>' : ''}
            </button>
        `).join('');
    }
}

function applyTranslations() {
    document.querySelectorAll('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        if (key && currentI18nStrings[key]) {
            el.textContent = currentI18nStrings[key];
        }
    });

    document.querySelectorAll('[data-i18n-html]').forEach(el => {
        const key = el.getAttribute('data-i18n-html');
        if (key && currentI18nStrings[key]) {
            el.innerHTML = currentI18nStrings[key];
        }
    });

    document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
        const key = el.getAttribute('data-i18n-placeholder');
        if (key && currentI18nStrings[key]) {
            el.placeholder = currentI18nStrings[key];
        }
    });

    document.querySelectorAll('[data-i18n-title]').forEach(el => {
        const key = el.getAttribute('data-i18n-title');
        if (key && currentI18nStrings[key]) {
            el.title = currentI18nStrings[key];
        }
    });

    const activeCode = State.appLang || 'fr';
    document.documentElement.lang = activeCode;
    document.documentElement.dir = activeCode === 'ar' ? 'rtl' : 'ltr';

    renderLangSelectors();

    if (typeof updateDynamicTranslations === 'function') {
        updateDynamicTranslations();
    }
}

function updateDynamicTranslations() {
    if (typeof State !== 'undefined' && State.homeData && State.homeData.sections && typeof renderHomeRows === 'function') {
        renderHomeRows();
    }
    if (typeof DOM !== 'undefined' && DOM.heroListBtn && typeof updateWatchlistButton === 'function' && typeof State !== 'undefined' && State.homeData && State.homeData.slides && State.homeData.slides[State.currentSlide]) {
        const currentSlide = State.homeData.slides[State.currentSlide];
        const inList = State.userData && State.userData.watchlist && State.userData.watchlist.includes(currentSlide.slug);
        updateWatchlistButton(DOM.heroListBtn, inList);
    }
    if (typeof DOM !== 'undefined' && DOM.modalWatchlistBtn && typeof updateWatchlistButton === 'function' && typeof State !== 'undefined' && State.currentAnime) {
        const inList = State.userData && State.userData.watchlist && State.userData.watchlist.includes(State.currentAnime.slug);
        updateWatchlistButton(DOM.modalWatchlistBtn, inList);
    }
    if (typeof updateModalPlayButton === 'function' && typeof State !== 'undefined' && State.currentAnime) {
        updateModalPlayButton(State.currentAnime);
    }
    if (typeof renderEpisodesGrid === 'function' && typeof State !== 'undefined' && State.currentEpisodes) {
        renderEpisodesGrid();
    }
    if (typeof DOM !== 'undefined' && DOM.heroSynopsis && typeof State !== 'undefined' && State.homeData && State.homeData.slides && State.homeData.slides[State.currentSlide]) {
        const slide = State.homeData.slides[State.currentSlide];
        if (State.appLang === 'fr') {
            DOM.heroSynopsis.textContent = slide.synopsis;
        } else if (typeof translateContent === 'function' && slide.synopsis) {
            translateContent(slide.synopsis, State.appLang).then(t => {
                if (DOM.heroSynopsis) DOM.heroSynopsis.textContent = t;
            });
        }
    }
    if (typeof DOM !== 'undefined' && DOM.modalSynopsis && typeof State !== 'undefined' && State.currentAnime && State.currentAnime.synopsis) {
        if (State.appLang === 'fr') {
            DOM.modalSynopsis.textContent = State.currentAnime.synopsis;
        } else if (typeof translateContent === 'function') {
            translateContent(State.currentAnime.synopsis, State.appLang).then(translated => {
                if (DOM.modalSynopsis && State.currentAnime) {
                    DOM.modalSynopsis.textContent = translated;
                }
            });
        }
    }
}

async function fetchAvailableLanguages() {
    try {
        const res = await fetch('/api/languages');
        if (res.ok) {
            const list = await res.json();
            for (const item of list) {
                const existing = I18N_LANGUAGES[item.id];
                if (!existing) {
                    const fallbackFlag = item.flag_svg || `
                        <svg viewBox="0 0 32 22" class="lang-flag-svg custom-flag-svg">
                            <rect width="32" height="22" rx="3" fill="#2d3748"/>
                            <text x="16" y="15" font-size="10" font-weight="900" text-anchor="middle" fill="#e2e8f0">${item.code}</text>
                        </svg>
                    `;
                    I18N_LANGUAGES[item.id] = {
                        name: item.name,
                        code: item.code,
                        flag: fallbackFlag,
                        isCustom: true
                    };
                } else if (item.flag_svg) {
                    existing.flag = item.flag_svg;
                }
            }
            renderLangSelectors();
        }
    } catch (_) {}
}

async function openLangFolder() {
    try {
        const res = await fetch('/api/open-lang-folder', { method: 'POST' });
        if (res.ok) {
            if (typeof showToast === 'function') {
                showToast(t('toast.lang_folder_opened', 'Dossier des langues ouvert'));
            }
        }
    } catch (_) {}
}

async function openAppDataFolder() {
    try {
        const res = await fetch('/api/open-appdata', { method: 'POST' });
        if (res.ok) {
            if (typeof showToast === 'function') {
                showToast(t('toast.appdata_folder_opened', 'Dossier de l\'application ouvert'));
            }
        }
    } catch (_) {}
}

async function refreshLanguages() {
    await fetchAvailableLanguages();
    if (typeof showToast === 'function') {
        showToast(t('toast.lang_refreshed', 'Langues actualisées'));
    }
}

async function setAppLanguage(code) {
    if (!I18N_LANGUAGES[code]) code = 'fr';
    State.appLang = code;
    try {
        localStorage.setItem('as_app_lang', code);
    } catch (_) {}

    let dict = await fetchLanguageData(code);
    if (!dict && code !== 'fr') {
        dict = await fetchLanguageData('fr');
    }
    if (dict) {
        currentI18nStrings = dict;
        applyTranslations();
    }
}

async function initI18n() {
    await fetchAvailableLanguages();
    let saved = 'fr';
    try {
        saved = localStorage.getItem('as_app_lang') || 'fr';
    } catch (_) {}
    await setAppLanguage(saved);
}

document.addEventListener('click', (e) => {
    const selector = document.getElementById('navLangSelector');
    if (selector && !selector.contains(e.target)) {
        const menu = document.getElementById('navLangMenu');
        if (menu) menu.classList.remove('open');
    }
});

document.addEventListener('DOMContentLoaded', () => {
    initI18n();
});
