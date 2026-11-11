/* Cataclyse 11H.2 — Navigation mobile */
(() => {
    'use strict';

    if (document.getElementById('catMobileNav')) return;

    const iconPaths = {
        home: '<path d="m3 10 9-7 9 7v11h-6v-7H9v7H3z"/>',
        animes: '<rect x="3" y="5" width="18" height="16" rx="2"/><path d="M3 10h18M8 5l3-3M15 5l3-3"/>',
        search: '<circle cx="11" cy="11" r="7"/><path d="m16 16 5 5"/>',
        list: '<path d="M5 4h14v17l-7-5-7 5z"/>',
        more: '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>'
    };

    const tabs = [
        ['home', 'Accueil', 'home'],
        ['animes', 'Animes', 'animes'],
        ['search', 'Recherche', 'search'],
        ['watchlist', 'Ma liste', 'list'],
        ['more', 'Plus', 'more']
    ];

    const secondary = new Set([
        'scans', 'classiques', 'planning'
    ]);

    function svg(name) {
        return '<svg viewBox="0 0 24 24" fill="none" ' +
            'stroke="currentColor" stroke-width="1.8" ' +
            'stroke-linecap="round" stroke-linejoin="round" ' +
            'aria-hidden="true">' +
            iconPaths[name] + '</svg>';
    }

    const nav = document.createElement('nav');
    nav.id = 'catMobileNav';
    nav.className = 'cat-mobile-nav';
    nav.setAttribute('aria-label', 'Navigation mobile');

    nav.innerHTML = tabs.map(([view, label, icon]) =>
        '<button type="button" class="cat-mobile-tab" ' +
        'data-mobile-view="' + view + '" ' +
        (view === 'more'
            ? 'aria-expanded="false" aria-controls="catMobileMore" '
            : '') +
        'aria-label="' + label + '">' +
        svg(icon) + '<span>' + label + '</span></button>'
    ).join('');

    const menu = document.createElement('div');
    menu.id = 'catMobileMore';
    menu.className = 'cat-mobile-more';
    menu.setAttribute('aria-label', 'Autres rubriques');
    menu.hidden = true;

    const moreButton = nav.querySelector(
        '[data-mobile-view="more"]'
    );

    function addMenuItem(label, view, action = '') {
        const button = document.createElement('button');
        button.type = 'button';
        button.className = 'cat-mobile-menu-item';
        button.textContent = label;

        if (view) button.dataset.mobileTarget = view;
        if (action) button.dataset.mobileAction = action;

        menu.appendChild(button);
    }

    addMenuItem('Scans & Webtoons', 'scans');
    addMenuItem('Classiques', 'classiques');
    addMenuItem('Planning des sorties', 'planning');

    const settingsButton = document.querySelector(
        '.profile-menu .settings-nav-btn'
    );

    if (settingsButton) {
        addMenuItem('Paramètres', '', 'settings');
    }

    function closeMenu() {
        menu.hidden = true;
        moreButton.setAttribute('aria-expanded', 'false');
    }

    function toggleMenu() {
        const opening = menu.hidden;
        menu.hidden = !opening;
        moreButton.setAttribute(
            'aria-expanded', String(opening)
        );

        if (opening) {
            menu.querySelector('button')?.focus({
                preventScroll: true
            });
        }
    }

    function sync(view) {
        const selected = secondary.has(view) ? 'more' : view;

        nav.querySelectorAll('[data-mobile-view]').forEach(
            button => {
                const active = button.dataset.mobileView === selected;

                button.classList.toggle('is-active', active);

                if (active) {
                    button.setAttribute('aria-current', 'page');
                } else {
                    button.removeAttribute('aria-current');
                }
            }
        );

        menu.querySelectorAll('[data-mobile-target]').forEach(
            button => {
                button.classList.toggle(
                    'is-active',
                    button.dataset.mobileTarget === view
                );
            }
        );

        closeMenu();
    }

    function navigate(view) {
        closeMenu();

        if (typeof window.switchView !== 'function') return;

        window.switchView(view);

        if (view === 'search') {
            requestAnimationFrame(() => {
                const input = document.getElementById('searchInput');
                const toggle = document.getElementById('searchToggleBtn');

                if (!input) return;

                if (!input.getClientRects().length && toggle) {
                    toggle.click();
                }

                if (input.getClientRects().length) {
                    input.focus({preventScroll: true});
                }
            });
        }
    }

    nav.addEventListener('click', event => {
        const button = event.target.closest('[data-mobile-view]');
        if (!button) return;

        const view = button.dataset.mobileView;

        if (view === 'more') {
            toggleMenu();
        } else {
            navigate(view);
        }
    });

    menu.addEventListener('click', event => {
        const button = event.target.closest('button');
        if (!button) return;

        if (button.dataset.mobileAction === 'settings') {
            closeMenu();
            settingsButton?.click();
            return;
        }

        if (button.dataset.mobileTarget) {
            navigate(button.dataset.mobileTarget);
        }
    });

    document.addEventListener('pointerdown', event => {
        if (
            !menu.hidden &&
            !menu.contains(event.target) &&
            !moreButton.contains(event.target)
        ) {
            closeMenu();
        }
    });

    document.addEventListener('keydown', event => {
        if (event.key === 'Escape' && !menu.hidden) {
            closeMenu();
            moreButton.focus();
        }
    });

    window.addEventListener('resize', () => {
        if (window.innerWidth > 640) closeMenu();
    });

    document.body.appendChild(nav);
    document.body.appendChild(menu);

    window.syncMobileNavigation = sync;

    const active = document.querySelector('.nav-link.active');
    sync(active?.dataset.view || 'home');
})();
