/* Cataclyse PWA 11I.2 */
(() => {
    'use strict';

    if (!('serviceWorker' in navigator)) return;

    window.addEventListener('load', () => {
        navigator.serviceWorker.register('/sw.js', {
            scope: '/'
        }).catch(error => {
            console.warn('[PWA] Enregistrement refuse :', error);
        });
    });
})();
