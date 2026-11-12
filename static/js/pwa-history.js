/* Cataclyse 11J.1A - Historique des rubriques PWA */
(() => {
    'use strict';

    const standalone =
        window.matchMedia?.('(display-mode: standalone)').matches ||
        navigator.standalone === true;

    if (!standalone || typeof window.switchView !== 'function') {
        return;
    }

    const views = new Set([
        'home',
        'animes',
        'scans',
        'classiques',
        'planning',
        'watchlist',
        'search'
    ]);

    const routeKey = '__cataclysePwaView';
    const depthKey = '__cataclysePwaDepth';

    function stateObject(value) {
        return value && typeof value === 'object'
            ? value
            : {};
    }

    const currentActive =
        document.querySelector('.nav-link.active')
            ?.dataset.view || 'home';

    const initialView = views.has(currentActive)
        ? currentActive
        : 'home';

    // La page courante devient le point de départ.
    // On n'ajoute pas d'entrée artificielle.
    try {
        history.replaceState({
            ...stateObject(history.state),
            [routeKey]: initialView,
            [depthKey]: 0
        }, '', location.href);
    } catch (error) {
        console.warn('[PWA History] Initialisation :', error);
        return;
    }

    const originalSwitchView = window.switchView;
    let restoring = false;

    window.switchView = function (viewName, ...args) {
        const previous = history.state?.[routeKey];

        // La fonction d'origine reste responsable de l'UI,
        // des modales, du catalogue et de la synchronisation.
        const result = originalSwitchView.call(
            this, viewName, ...args
        );

        if (
            !restoring &&
            views.has(viewName) &&
            previous !== viewName
        ) {
            const depth = Number.isSafeInteger(
                history.state?.[depthKey]
            ) ? history.state[depthKey] : 0;

            try {
                history.pushState({
                    ...stateObject(history.state),
                    [routeKey]: viewName,
                    [depthKey]: depth + 1
                }, '', location.href);
            } catch (error) {
                console.warn('[PWA History] Navigation :', error);
            }
        }

        return result;
    };

    window.addEventListener('popstate', event => {
        const view = event.state?.[routeKey];

        if (!views.has(view)) return;

        restoring = true;

        try {
            originalSwitchView.call(window, view);
        } finally {
            restoring = false;
        }
    });

    window.cataclysePwaCanGoBack = () =>
        Number(history.state?.[depthKey]) > 0;

    window.cataclysePwaGoBack = () => {
        if (!window.cataclysePwaCanGoBack()) {
            return false;
        }

        history.back();
        return true;
    };

    console.info('[Cataclyse] Historique PWA activé');
})();
