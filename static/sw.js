/* Cataclyse PWA 11I.2 - Aucun cache applicatif */
'use strict';

self.addEventListener('install', () => {
    self.skipWaiting();
});

self.addEventListener('activate', event => {
    event.waitUntil(self.clients.claim());
});

// Pas de gestionnaire fetch.
// API, comptes et flux video : reseau normal.
