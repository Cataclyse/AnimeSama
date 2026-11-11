/* Cataclyse 11H.4.4 — diagnostic plein écran iPhone */
(() => {
    'use strict';

    if (!/iPhone|iPod/i.test(navigator.userAgent)) return;

    let fullscreenStarted = false;
    let messageTimer = null;

    function notify(message) {
        let notice = document.getElementById(
            'catMobileFullscreenNotice'
        );

        if (!notice) {
            notice = document.createElement('div');
            notice.id = 'catMobileFullscreenNotice';
            notice.setAttribute('role', 'status');

            Object.assign(notice.style, {
                position: 'fixed',
                top: 'calc(80px + env(safe-area-inset-top, 0px))',
                left: '12px',
                right: '12px',
                padding: '14px',
                background: '#24242a',
                color: '#fff',
                borderRadius: '12px',
                zIndex: '2147483647',
                font: '14px system-ui, sans-serif',
                lineHeight: '1.5',
                boxShadow: '0 8px 30px #000a',
                pointerEvents: 'none'
            });

            document.body.appendChild(notice);
        }

        notice.textContent = message;
        notice.hidden = false;

        clearTimeout(messageTimer);
        messageTimer = setTimeout(() => {
            notice.hidden = true;
        }, 7000);
    }

    document.addEventListener('click', event => {
        if (!(event.target instanceof Element)) return;

        const button = event.target.closest(
            '#netflixFullscreenBtn'
        );

        if (!button) return;

        const modal = document.getElementById('playerModal');

        if (!modal?.classList.contains('open')) return;

        const video = document.getElementById(
            'cataclyseNativeVideo'
        );

        // Sur iPhone, ne pas exécuter le plein écran
        // du conteneur desktop.
        event.preventDefault();
        event.stopImmediatePropagation();

        if (!(video instanceof HTMLVideoElement)) {
            notify(
                'Lecteur externe actif : aucun élément vidéo ' +
                'HTML5 natif disponible pour le plein écran iOS.'
            );
            return;
        }

        if (video.readyState < HTMLMediaElement.HAVE_METADATA) {
            notify(
                'La vidéo n’est pas encore prête. ' +
                'Réessaie lorsque la lecture a commencé.'
            );
            return;
        }

        if (
            typeof video.webkitEnterFullscreen !== 'function'
        ) {
            notify(
                'Ce navigateur ne propose pas ' +
                'webkitEnterFullscreen pour cette vidéo.'
            );
            return;
        }

        if (video.webkitSupportsFullscreen === false) {
            notify(
                'WebKit indique que cette vidéo ' +
                'ne prend pas en charge le plein écran natif.'
            );
            return;
        }

        fullscreenStarted = false;

        video.addEventListener(
            'webkitbeginfullscreen',
            () => {
                fullscreenStarted = true;
            },
            { once: true }
        );

        try {
            // Appel synchrone depuis le bouton tactile.
            video.webkitEnterFullscreen();

            setTimeout(() => {
                if (
                    !fullscreenStarted &&
                    !video.webkitDisplayingFullscreen
                ) {
                    notify(
                        'Safari n’est pas entré en plein écran. ' +
                        'Le navigateur peut refuser cette opération ' +
                        'ou la vidéo peut ne pas être compatible.'
                    );
                }
            }, 1200);

        } catch (error) {
            notify(
                'Échec du plein écran iOS : ' +
                (error?.name || 'erreur inconnue')
            );
        }
    }, true);
})();
