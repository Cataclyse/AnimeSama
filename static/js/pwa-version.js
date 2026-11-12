/* Cataclyse — Version dynamique */
(() => {
    'use strict';

    async function updateVersion() {
        try {
            const response = await fetch('/api/version', {
                cache: 'no-store',
                credentials: 'same-origin'
            });

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }

            const data = await response.json();

            if (
                typeof data.version !== 'string' ||
                !/^\d+\.\d+\.\d+(?:[-+][\w.-]+)?$/.test(data.version)
            ) {
                throw new Error('Version invalide');
            }

            const version = `v${data.version}`;

            for (const id of [
                'settingsVersionBadge',
                'updateStatusVersion',
                'settingsInfoVersion'
            ]) {
                const element = document.getElementById(id);

                if (element) {
                    element.textContent = version;
                }
            }

            const subtitle = document.getElementById(
                'updateScreenSubtitle'
            );

            if (
                subtitle &&
                subtitle.textContent.trim() === 'Vérification…'
            ) {
                subtitle.textContent =
                    `Version actuelle : ${version}`;
            }

            console.info(
                '[Cataclyse] Version serveur :',
                version
            );

        } catch (error) {
            console.warn(
                '[Cataclyse] Version indisponible :',
                error
            );
        }
    }

    updateVersion();

    window.addEventListener('pageshow', event => {
        if (event.persisted) updateVersion();
    });
})();
