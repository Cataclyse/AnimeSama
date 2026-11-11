/* 11H.3 : Navigation mobile dans les paramètres */
(() => {
    'use strict';

    const modal = document.getElementById('accountModal');
    const sidebar = modal?.querySelector(
        '.netflix-settings-sidebar'
    );
    const backButton = document.getElementById(
        'catMobileSettingsBack'
    );

    if (!modal || !sidebar || !backButton) return;

    const mobile = window.matchMedia('(max-width: 640px)');

    function showMenu() {
        modal.classList.remove('cat-settings-detail');
    }

    function showDetail() {
        if (!mobile.matches) return;

        modal.classList.add('cat-settings-detail');

        const content = modal.querySelector(
            '.netflix-settings-content'
        );

        if (content) {
            content.scrollTop = 0;
        }
    }

    sidebar.addEventListener('click', event => {
        const button = event.target.closest(
            '.netflix-sidebar-item'
        );

        if (!button || !sidebar.contains(button)) return;

        // Le onclick existant a déjà activé le bon onglet.
        showDetail();
    });

    backButton.addEventListener('click', () => {
        showMenu();

        const selected = sidebar.querySelector(
            '.netflix-sidebar-item.active'
        );

        selected?.focus({ preventScroll: true });
    });

    const observer = new MutationObserver(() => {
        if (
            !modal.classList.contains('open') &&
            modal.classList.contains('cat-settings-detail')
        ) {
            showMenu();
        }
    });

    observer.observe(modal, {
        attributes: true,
        attributeFilter: ['class']
    });

    if (typeof mobile.addEventListener === 'function') {
        mobile.addEventListener('change', showMenu);
    }

    document.addEventListener('keydown', event => {
        if (
            event.key === 'Escape' &&
            mobile.matches &&
            modal.classList.contains('open') &&
            modal.classList.contains('cat-settings-detail')
        ) {
            event.stopPropagation();
            showMenu();
        }
    }, true);
})();
