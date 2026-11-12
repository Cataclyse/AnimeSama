/* Cataclyse 11J.2A - Sauvegarde à la suspension iOS */
(() => {
    'use strict';

    const standalone =
        window.matchMedia?.('(display-mode: standalone)').matches ||
        navigator.standalone === true;

    if (!standalone) return;

    let lastSavedAt = 0;
    let lastSavedPosition = -1;

    function saveBeforeSuspension() {
        const modal = document.getElementById('playerModal');
        if (!modal?.classList.contains('open')) return;

        const video = document.getElementById(
            'cataclyseNativeVideo'
        );

        // Ne modifier ni les scans ni le lecteur iframe.
        if (!(video instanceof HTMLVideoElement)) return;

        const position = video.currentTime;
        const duration = video.duration;

        if (
            !Number.isFinite(position) ||
            position < 1 ||
            !Number.isFinite(duration) ||
            duration <= 0
        ) return;

        if (typeof saveNativeHlsProgress !== 'function') {
            return;
        }

        const now = Date.now();

        // Éviter deux écritures identiques lorsque
        // visibilitychange et pagehide se succèdent.
        if (
            now - lastSavedAt < 1500 &&
            Math.abs(position - lastSavedPosition) < 0.5
        ) {
            return;
        }

        try {
            saveNativeHlsProgress(true);
            lastSavedAt = now;
            lastSavedPosition = position;
        } catch (error) {
            console.warn(
                '[Cataclyse PWA] Sauvegarde suspension :',
                error
            );
        }
    }

    document.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'hidden') {
            saveBeforeSuspension();
        }
    });

    window.addEventListener('pagehide', () => {
        saveBeforeSuspension();
    });

    // Ne pas lancer play() ou recharger la source
    // automatiquement lorsque la PWA redevient visible.
    console.info(
        '[Cataclyse PWA] Sauvegarde à la suspension activée'
    );
})();
