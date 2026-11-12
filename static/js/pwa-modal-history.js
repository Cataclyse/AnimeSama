/* Cataclyse 11J.1B — historique des modales PWA */
(() => {
    'use strict';

    const standalone =
        window.matchMedia?.('(display-mode: standalone)').matches ||
        navigator.standalone === true;

    if (!standalone) return;

    const detail = document.getElementById('detailModal');
    const player = document.getElementById('playerModal');

    const openDetail = window.openDetailModal;
    const closeDetail = window.closeDetailModal;
    const closePlayer = window.closePlayerModal;
    const switchView = window.switchView;

    if (!detail || !player || [
        openDetail, closeDetail, closePlayer, switchView
    ].every(fn => typeof fn === 'function') === false) {
        return;
    }

    const VIEW = '__cataclysePwaView';
    const DEPTH = '__cataclysePwaDepth';
    const LAYER = '__cataclysePwaLayer';
    const SLUG = '__cataclysePwaSlug';

    let navigating = false;
    let restoring = false;
    let playerWasOpen = player.classList.contains('open');

    const isOpen = el => el.classList.contains('open');

    function pushLayer(layer, slug = null) {
        if (
            restoring ||
            navigating ||
            window.__cataclysePwaRestoringView ||
            !history.state?.[VIEW]
        ) return;

        if (history.state[LAYER] === layer) return;

        const depth = Number.isSafeInteger(history.state[DEPTH])
            ? history.state[DEPTH]
            : 0;

        try {
            history.pushState({
                ...history.state,
                [LAYER]: layer,
                [SLUG]: slug,
                [DEPTH]: depth + 1
            }, '', location.href);
        } catch (error) {
            console.warn('[PWA] Historique modale :', error);
        }
    }

    // L'ouverture de la fiche est synchrone,
    // même si ses données sont chargées ensuite.
    window.openDetailModal = function(slug, ...args) {
        const wasOpen = isOpen(detail);
        const result = openDetail.call(this, slug, ...args);

        if (!wasOpen && isOpen(detail)) {
            pushLayer('detail', slug);
        }
        return result;
    };

    window.closeDetailModal = function(...args) {
        const consume =
            !restoring &&
            !navigating &&
            !window.__cataclysePwaRestoringView &&
            isOpen(detail) &&
            history.state?.[LAYER] === 'detail';

        const result = closeDetail.apply(this, args);

        if (consume) history.back();
        return result;
    };

    window.closePlayerModal = function(...args) {
        const consume =
            !restoring &&
            !navigating &&
            !window.__cataclysePwaRestoringView &&
            isOpen(player) &&
            history.state?.[LAYER] === 'player';

        // La fonction d'origine gère la sauvegarde
        // de progression et le nettoyage HLS.
        const result = closePlayer.apply(this, args);

        if (consume) history.back();
        return result;
    };

    // Un changement de rubrique peut fermer
    // les modales sans effectuer plusieurs retours.
    window.switchView = function(...args) {
        navigating = true;
        try {
            return switchView.apply(this, args);
        } finally {
            navigating = false;
        }
    };

    // Vidéo et scans utilisent la même fenêtre.
    new MutationObserver(() => {
        const nowOpen = isOpen(player);

        if (nowOpen && !playerWasOpen) {
            pushLayer('player');
        }

        playerWasOpen = nowOpen;
    }).observe(player, {
        attributes: true,
        attributeFilter: ['class']
    });

    window.addEventListener('popstate', event => {
        if (!event.state?.[VIEW]) return;

        const layer = event.state[LAYER] || 'view';

        restoring = true;

        try {
            if (layer !== 'player' && isOpen(player)) {
                closePlayer();
            }

            if (layer === 'view' && isOpen(detail)) {
                closeDetail();
            }

            // Revenir à une fiche déjà fermée est possible.
            if (
                layer === 'detail' &&
                !isOpen(detail) &&
                event.state[SLUG]
            ) {
                openDetail(event.state[SLUG]);
            }

            // Ne jamais redémarrer une vidéo automatiquement
            // lors d'une navigation vers l'avant.
            if (layer === 'player' && !isOpen(player)) {
                const normalized = {...event.state};
                delete normalized[LAYER];
                delete normalized[SLUG];

                history.replaceState(
                    normalized, '', location.href
                );
            }
        } finally {
            restoring = false;
        }
    });

    console.info('[Cataclyse] Historique modales PWA activé');
})();
