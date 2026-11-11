// Les comptes locaux ne sont pas des comptes cloud.
let localAccountUser = null;
let localAccountStatusSequence = 0;

function applyAccountStatus(data) {
    const cache = window.CataclysePlaybackCache;
    const identity = cache?.snapshot?.();
    const user = data?.logged_in ? data.user : null;

    const valid = Boolean(
        user &&
        Number.isSafeInteger(user.id) &&
        user.id > 0 &&
        typeof user.username === 'string' &&
        user.id === identity?.id
    );

    localAccountUser = valid
        ? { id: user.id, username: user.username }
        : null;

    // Ce champ contrôlait l'ancienne synchronisation cloud.
    // Ne pas l'activer pour un compte local.
    State.loggedIn = false;

    if (DOM.accountPseudo) {
        DOM.accountPseudo.textContent =
            localAccountUser?.username || '';
    }

    if (DOM.panelCloudLoggedOut) {
        DOM.panelCloudLoggedOut.style.display =
            localAccountUser ? 'none' : 'block';
    }
    if (DOM.panelCloudLoggedIn) {
        DOM.panelCloudLoggedIn.style.display =
            localAccountUser ? 'block' : 'none';
    }

    return valid;
}

async function refreshAccountStatus() {
    const cache = window.CataclysePlaybackCache;

    if (!cache ||
        typeof cache.refresh !== 'function' ||
        typeof cache.snapshot !== 'function') {
        applyAccountStatus(null);
        return false;
    }

    try {
        await cache.refresh();
        const sequence = ++localAccountStatusSequence;
        const initial = cache.snapshot();

        const response = await fetch('/api/local-auth/status', {
            credentials: 'same-origin',
            cache: 'no-store'
        });

        if (!response.ok) {
            throw new Error('HTTP ' + response.status);
        }

        const data = await response.json();
        const verified = await cache.refresh();

        if (
            sequence !== localAccountStatusSequence ||
            verified.generation !== initial.generation ||
            verified.id !== initial.id
        ) {
            return false;
        }

        const responseId = data?.logged_in
            ? data.user?.id
            : null;

        if (responseId !== verified.id) {
            applyAccountStatus(null);
            return false;
        }

        return applyAccountStatus(data) ||
            responseId === null;
    } catch (error) {
        console.error(
            '[LocalAccount] Vérification impossible',
            error
        );
        applyAccountStatus(null);
        return false;
    }
}

window.addEventListener(
    'cataclyse-playback-identity-change',
    () => {
        ++localAccountStatusSequence;
        applyAccountStatus(null);

        if (DOM.accountModal?.classList.contains('open')) {
            void refreshAccountStatus();
        }
    }
);

function switchSettingsTab(tabName) {
    const tabs = document.querySelectorAll('.netflix-sidebar-item');
    const panels = document.querySelectorAll('.netflix-tab-panel');
    tabs.forEach(btn => {
        btn.classList.toggle('active', btn.getAttribute('data-tab') === tabName);
    });
    panels.forEach(panel => {
        panel.classList.toggle('active', panel.id === `tabPanel-${tabName}`);
    });
}

function disableGlobalSettingsControls() {
    // GLOBAL_MAINTENANCE_UI_LOCK_11F9
    for (const id of ['checkUpdateBtn', 'applyUpdateBtn']) {
        const button = document.getElementById(id);
        if (!button) continue;

        button.disabled = true;
        button.title =
            'Mise à jour réservée à l’administration du serveur';
    }


    const controls = [
        DOM.autoSyncCheckbox,
        DOM.adShieldCheckbox,
        DOM.autoUpdateCheckbox,
        DOM.discordRpcCheckbox,
        DOM.syncAutoBtn
    ];

    for (const control of controls) {
        if (!control) continue;
        control.disabled = true;
        control.title =
            'Réservé à une future administration du serveur';

        if (control.type === 'checkbox') {
            control.checked = false;
            control.indeterminate = true;
        }
    }

    if (DOM.syncStatus) {
        DOM.syncStatus.textContent =
            'Synchronisation cloud indisponible en mode multi-utilisateurs.';
    }
}

async function openAccountModal() {
    DOM.accountModal.classList.add('open');
    document.body.style.overflow = 'hidden';

    if (typeof renderLangSelectors === 'function') {
        renderLangSelectors();
    }

    const activeTab = document.querySelector(
        '.netflix-sidebar-item.active'
    );
    if (!activeTab) {
        switchSettingsTab('account');
    }

    // Réutiliser les panneaux existants sans appeler
    // l'ancienne authentification Anime-Sama cloud.
    DOM.modeCloudBtn.removeAttribute('data-i18n');
    DOM.modeCloudBtn.textContent = 'Compte local';
    DOM.modeLocalBtn.removeAttribute('data-i18n');
    DOM.modeLocalBtn.textContent = 'Stockage navigateur';

    disableGlobalSettingsControls();

    // L'écran du compte est le panneau précédemment
    // nommé "cloud" dans les identifiants HTML.
    switchAccountMode('cloud');
    await refreshAccountStatus();
}

function closeAccountModal() {
    DOM.accountModal.classList.remove('open');
    document.body.style.overflow = 'auto';
}

function switchAccountMode(mode) {
    const showStorage = mode === 'local';

    DOM.modeLocalBtn.classList.toggle('active', showStorage);
    DOM.modeCloudBtn.classList.toggle('active', !showStorage);
    DOM.panelLocal.style.display =
        showStorage ? 'block' : 'none';

    DOM.panelCloudLoggedOut.style.display =
        showStorage || localAccountUser ? 'none' : 'block';
    DOM.panelCloudLoggedIn.style.display =
        showStorage || !localAccountUser ? 'none' : 'block';
}

function toggleAccountForms(which) {
    DOM.formAccountLogin.classList.toggle('active', which === 'login');
    DOM.formAccountRegister.classList.toggle('active', which === 'register');
    DOM.accountError.style.display = 'none';
}

function showAccountError(message) {
    DOM.accountError.textContent = message;
    DOM.accountError.style.display = 'block';
}

async function localAccountPost(path, payload) {
    const response = await fetch(path, {
        method: 'POST',
        credentials: 'same-origin',
        cache: 'no-store',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload || {})
    });

    let data = {};
    try {
        data = await response.json();
    } catch (_) {}

    if (!response.ok) {
        throw new Error(
            data.error || `Erreur HTTP ${response.status}`
        );
    }

    return data;
}

async function completeLocalLogin(username, password) {
    await localAccountPost(
        '/api/local-auth/login',
        { username, password }
    );

    const cache = window.CataclysePlaybackCache;

    if (!cache || typeof cache.refresh !== 'function') {
        throw new Error('Gestionnaire de session indisponible');
    }

    await cache.refresh();
    const verified = await refreshAccountStatus();

    if (!verified || !localAccountUser) {
        throw new Error('Session locale non vérifiée');
    }

    if (typeof reloadActiveAccountViews === 'function') {
        await reloadActiveAccountViews();
    }

    switchAccountMode('cloud');
    DOM.accountError.style.display = 'none';
    return true;
}

async function handleAccountLogin(event) {
    event.preventDefault();

    const username = document.getElementById(
        'accountLoginPseudo'
    ).value.trim();
    const password = document.getElementById(
        'accountLoginPassword'
    ).value;

    try {
        await completeLocalLogin(username, password);

        document.getElementById(
            'accountLoginPassword'
        ).value = '';

        showToast('Connecté au compte local');
    } catch (error) {
        showAccountError(error.message);
    }
}

async function handleAccountRegister(event) {
    event.preventDefault();

    const username = document.getElementById(
        'accountRegisterPseudo'
    ).value.trim();
    const password = document.getElementById(
        'accountRegisterPassword'
    ).value;
    const confirmation = document.getElementById(
        'accountRegisterPassword2'
    ).value;

    if (password !== confirmation) {
        showAccountError(
            'Les mots de passe ne correspondent pas'
        );
        return;
    }

    try {
        // L'inscription ne suppose pas une connexion automatique.
        await localAccountPost(
            '/api/local-auth/register',
            { username, password }
        );

        await completeLocalLogin(username, password);

        document.getElementById(
            'accountRegisterPassword'
        ).value = '';
        document.getElementById(
            'accountRegisterPassword2'
        ).value = '';

        showToast('Compte local créé et connecté');
    } catch (error) {
        showAccountError(error.message);
    }
}

async function handleAccountLogout() {
    try {
        await localAccountPost('/api/local-auth/logout', {});

        const cache = window.CataclysePlaybackCache;
        if (!cache || typeof cache.refresh !== 'function') {
            throw new Error('Gestionnaire de session indisponible');
        }

        await cache.refresh();
        const verified = await refreshAccountStatus();

        if (!verified || localAccountUser !== null) {
            throw new Error('Déconnexion non vérifiée');
        }

        if (typeof reloadActiveAccountViews === 'function') {
            await reloadActiveAccountViews();
        }

        switchAccountMode('cloud');
        showToast('Déconnecté du compte local');
    } catch (error) {
        showAccountError(error.message);
    }
}

function rejectGlobalSettingToggle(event) {
    if (event?.target) {
        event.target.checked = false;
        event.target.indeterminate = true;
    }
    showToast('Réglage réservé à l’administration du serveur');
    return false;
}

async function handleAutoSyncToggle(event) {
    return rejectGlobalSettingToggle(event);
}

async function handleAdShieldToggle(event) {
    return rejectGlobalSettingToggle(event);
}

async function handleAutoUpdateToggle(event) {
    return rejectGlobalSettingToggle(event);
}

async function handleDiscordRpcToggle(event) {
    return rejectGlobalSettingToggle(event);
}

async function runSync() {
    if (DOM.syncStatus) {
        DOM.syncStatus.textContent =
            'Synchronisation cloud indisponible en mode multi-utilisateurs.';
    }
    return false;
}

let currentUpdateInfo = null;

function updateCurrentVersionDisplay(version) {
    if (!version) return;
    const vSpan = document.getElementById('updateStatusVersion');
    if (vSpan) {
        vSpan.textContent = `v${version}`;
    } else {
        const title = document.getElementById('updateStatusTitle');
        if (title) {
            title.textContent = `${t('settings.update_status', 'Version actuelle')} : v${version}`;
        }
    }
    const badge = document.getElementById('settingsVersionBadge');
    if (badge) {
        badge.textContent = `v${version}`;
    }
    const infoVer = document.getElementById('settingsInfoVersion');
    if (infoVer) {
        infoVer.textContent = `v${version}`;
    }
}

async function checkForUpdates(silent = false) {
    // GLOBAL_MAINTENANCE_UI_LOCK_11F9
    // Réservé à une future administration explicite.
    return false;

    const btn = document.getElementById('checkUpdateBtn');
    const title = document.getElementById('updateStatusTitle');
    const desc = document.getElementById('updateStatusDesc');
    const applyBtn = document.getElementById('applyUpdateBtn');

    if (!silent && btn) {
        btn.disabled = true;
        btn.textContent = t('settings.checking_updates', 'Vérification...');
    }

    try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 12000);
        const res = await fetch('/api/update/check', { signal: controller.signal });
        clearTimeout(timeoutId);
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || 'Erreur');

        currentUpdateInfo = data;
        if (data.current_version) {
            updateCurrentVersionDisplay(data.current_version);
        }

        if (data.update_available) {
            if (desc) {
                const kindLabel = data.install_kind === 'windows_installer' ? ' (Installateur)'
                    : data.install_kind === 'windows_standalone' ? ' (Portable)'
                    : data.install_kind === 'linux_deb' ? ' (Paquet .deb)'
                    : data.install_kind === 'linux_gui_standalone' ? ' (Portable Linux)'
                    : '';
                desc.textContent = `${t('settings.update_available', 'Nouvelle version')} ${data.latest_version}${kindLabel} ${t('settings.available', 'disponible !')}`;
                desc.style.color = '#46d369';
            }
            if (applyBtn) {
                applyBtn.style.display = 'inline-block';
                applyBtn.textContent = `${t('settings.install_update', 'Mettre à jour')} (${data.latest_version})`;
            }
            if (silent) {
                showToast(`${t('settings.update_available', 'Nouvelle version')} ${data.latest_version} ${t('settings.available', 'disponible !')}`);
            }
        } else {
            if (desc) {
                desc.textContent = t('settings.up_to_date', 'Vous utilisez la dernière version.');
                desc.style.color = '#a3a3a3';
            }
            if (applyBtn) {
                applyBtn.style.display = 'none';
            }
            if (!silent) {
                showToast(t('settings.up_to_date', 'Vous utilisez la dernière version.'));
            }
        }
    } catch (e) {
        if (!silent) {
            showToast(t('settings.update_error', 'Impossible de vérifier les mises à jour'));
        }
    } finally {
        if (!silent && btn) {
            btn.disabled = false;
            btn.textContent = t('settings.check_updates', 'Vérifier');
        }
    }
}

function checkForUpdatesManual() {
    // GLOBAL_MAINTENANCE_UI_LOCK_11F9
    // Réservé à une future administration explicite.
    return false;

    checkForUpdates(false);
}

let updatePollInterval = null;

function formatUpdateBytes(bytes) {
    if (!bytes || bytes <= 0) return '0 Mo';
    const mb = bytes / (1024 * 1024);
    if (mb < 1) {
        return (bytes / 1024).toFixed(0) + ' Ko';
    }
    return mb.toFixed(1) + ' Mo';
}

function dismissUpdateOverlay() {
    if (updatePollInterval) {
        clearInterval(updatePollInterval);
        updatePollInterval = null;
    }
    const overlay = document.getElementById('appUpdateOverlay');
    if (overlay) {
        overlay.style.display = 'none';
    }
}

function showUpdateOverlay(info) {
    const overlay = document.getElementById('appUpdateOverlay');
    if (!overlay) return;
    overlay.style.display = 'flex';

    const sub = document.getElementById('updateScreenSubtitle');
    if (sub && info.current_version && info.latest_version) {
        sub.textContent = `v${info.current_version} → ${info.latest_version}`;
    }

    const checkStep = document.getElementById('updateStepCheck');
    const checkDesc = document.getElementById('updateStepCheckDesc');
    if (checkStep) {
        checkStep.classList.remove('active');
        checkStep.classList.add('done');
    }
    if (checkDesc && info.latest_version) {
        checkDesc.textContent = `${t('update.version_detected', 'Version détectée')} : ${info.latest_version}`;
    }

    const dlStep = document.getElementById('updateStepDownload');
    if (dlStep) {
        dlStep.classList.add('active');
        dlStep.classList.remove('done');
    }

    const progBar = document.getElementById('updateProgressBar');
    if (progBar) progBar.style.width = '0%';
    const progText = document.getElementById('updateProgressText');
    if (progText) progText.textContent = `${t('update.downloading', 'Téléchargement de')} ${info.latest_version || ''}...`;
    const progPct = document.getElementById('updateProgressPercent');
    if (progPct) progPct.textContent = '0%';
}

function startUpdateStepTracking(targetVersion) {
    if (updatePollInterval) {
        clearInterval(updatePollInterval);
    }

    updatePollInterval = setInterval(async () => {
        try {
            const res = await fetch('/api/update/status');
            if (!res.ok) return;
            const data = await res.json();

            const checkStep = document.getElementById('updateStepCheck');
            const dlStep = document.getElementById('updateStepDownload');
            const installStep = document.getElementById('updateStepInstall');
            const restartStep = document.getElementById('updateStepRestart');

            const dlDesc = document.getElementById('updateStepDownloadDesc');
            const dlVal = document.getElementById('updateStepDownloadValue');
            const installDesc = document.getElementById('updateStepInstallDesc');
            const restartDesc = document.getElementById('updateStepRestartDesc');

            const progBar = document.getElementById('updateProgressBar');
            const progText = document.getElementById('updateProgressText');
            const progPct = document.getElementById('updateProgressPercent');
            const bypassBtn = document.getElementById('updateBypassBtn');

            if (checkStep) {
                checkStep.classList.remove('active');
                checkStep.classList.add('done');
            }

            if (data.step_index === 2) {
                if (dlStep) {
                    dlStep.classList.add('active');
                    dlStep.classList.remove('done');
                }
                if (installStep) {
                    installStep.classList.remove('active', 'done');
                }
                if (restartStep) {
                    restartStep.classList.remove('active', 'done');
                }

                if (dlDesc) {
                    if (data.total_bytes > 0) {
                        dlDesc.textContent = `${formatUpdateBytes(data.bytes_downloaded)} / ${formatUpdateBytes(data.total_bytes)}`;
                    } else {
                        dlDesc.textContent = data.message || t('update.step_download_desc', 'Téléchargement en cours...');
                    }
                }
                if (dlVal) dlVal.textContent = `${data.percent}%`;
                if (progPct) progPct.textContent = `${data.percent}%`;
                if (progBar) progBar.style.width = `${data.percent}%`;
                if (progText) progText.textContent = `${t('update.downloading', 'Téléchargement de')} ${targetVersion || data.version || ''}...`;
            } else if (data.step_index === 3) {
                if (dlStep) {
                    dlStep.classList.remove('active');
                    dlStep.classList.add('done');
                }
                if (dlVal) dlVal.textContent = '100%';
                if (installStep) {
                    installStep.classList.add('active');
                    installStep.classList.remove('done');
                }
                if (restartStep) {
                    restartStep.classList.remove('active', 'done');
                }

                if (installDesc) {
                    installDesc.textContent = data.message || t('update.installing', 'Installation et finalisation...');
                }
                if (progPct) progPct.textContent = '100%';
                if (progBar) progBar.style.width = '100%';
                if (progText) progText.textContent = t('update.installing', 'Installation et finalisation...');
            } else if (data.step_index >= 4) {
                if (dlStep) {
                    dlStep.classList.remove('active');
                    dlStep.classList.add('done');
                }
                if (installStep) {
                    installStep.classList.remove('active');
                    installStep.classList.add('done');
                }

                if (data.step === 'done') {
                    if (restartStep) {
                        restartStep.classList.remove('active');
                        restartStep.classList.add('done');
                    }
                    if (restartDesc) {
                        restartDesc.textContent = t('update.ready', 'Mise à jour prête !');
                    }
                    if (progText) progText.textContent = t('update.ready', 'Mise à jour prête !');
                    if (bypassBtn) {
                        bypassBtn.textContent = t('update.enter_app', 'Entrer dans Anime-Sama');
                        bypassBtn.className = 'btn-netflix-primary btn-update-skip';
                    }
                    clearInterval(updatePollInterval);
                    updatePollInterval = null;
                } else {
                    if (restartStep) {
                        restartStep.classList.add('active');
                        restartStep.classList.remove('done');
                    }
                    if (restartDesc) {
                        restartDesc.textContent = data.message || t('update.restarting', 'Redémarrage imminent...');
                    }
                    if (progText) progText.textContent = t('update.restarting', 'Redémarrage imminent...');
                }
            }

            if (data.step === 'error') {
                clearInterval(updatePollInterval);
                updatePollInterval = null;
                if (progText) {
                    progText.textContent = t('update.error', 'Erreur lors de la mise à jour');
                    progText.style.color = '#ff4747';
                }
                if (installDesc) {
                    installDesc.textContent = data.message;
                    installDesc.style.color = '#ff4747';
                }
                if (bypassBtn) {
                    bypassBtn.textContent = t('update.bypass', 'Ignorer');
                    bypassBtn.className = 'btn-netflix-gray btn-update-skip';
                }
            }
        } catch (e) {
        }
    }, 120);
}

async function applyUpdateDirect() {
    // GLOBAL_MAINTENANCE_UI_LOCK_11F9
    // Réservé à une future administration explicite.
    return false;

    const applyBtn = document.getElementById('applyUpdateBtn');
    if (applyBtn) {
        applyBtn.disabled = true;
        applyBtn.textContent = t('settings.updating', 'Mise à jour en cours...');
    }

    try {
        if (currentUpdateInfo) {
            showUpdateOverlay(currentUpdateInfo);
            startUpdateStepTracking(currentUpdateInfo.latest_version);
        }
        const res = await fetch('/api/update/apply', { method: 'POST' });
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || 'Erreur lors de la mise à jour');

        if (!currentUpdateInfo) {
            showToast(data.message || t('settings.update_success', 'Mise à jour réussie ! Redémarrage...'));
        }
    } catch (e) {
        showToast(t('settings.update_error', 'Erreur de mise à jour') + ' : ' + e.message);
        if (applyBtn) {
            applyBtn.disabled = false;
            applyBtn.textContent = t('settings.install_update', 'Mettre à jour');
        }
    }
}

async function runAutoUpdateOnLaunch() {
    // GLOBAL_MAINTENANCE_UI_LOCK_11F9
    // Réservé à une future administration explicite.
    return false;

    try {
        const isPlayerActive = () => {
            const modal = document.getElementById('playerModal');
            if (modal && (modal.classList.contains('open') || modal.style.display === 'flex' || modal.style.display === 'block')) return true;
            if (typeof playerState !== 'undefined' && (!playerState.paused || playerState.currentTime > 0)) return true;
            return false;
        };

        if (isPlayerActive()) return;

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 12000);
        const res = await fetch('/api/update/check', { signal: controller.signal });
        clearTimeout(timeoutId);
        const data = await res.json();
        if (!res.ok || !data.update_available) return;

        if (isPlayerActive()) return;

        currentUpdateInfo = data;
        showUpdateOverlay(data);
        startUpdateStepTracking(data.latest_version);

        await fetch('/api/update/apply', { method: 'POST' });
    } catch (e) {
    }
}

let appSettings = { auto_sync: false, ad_shield: true, auto_update: true };

fetch('/api/settings')
    .then(r => r.json())
    .then(d => {
        appSettings = d;
        if (DOM.autoSyncCheckbox) DOM.autoSyncCheckbox.checked = !!d.auto_sync;
        if (DOM.adShieldCheckbox) DOM.adShieldCheckbox.checked = d.ad_shield !== false;
        if (DOM.autoUpdateCheckbox) DOM.autoUpdateCheckbox.checked = d.auto_update !== false;
        if (DOM.discordRpcCheckbox) DOM.discordRpcCheckbox.checked = d.discord_rpc !== false;
        if (d.version) updateCurrentVersionDisplay(d.version);

        if (appSettings.auto_update !== false) {
            runAutoUpdateOnLaunch();
        } else {
            checkForUpdates(true);
        }
    })
    .catch(() => {
        runAutoUpdateOnLaunch();
    });
