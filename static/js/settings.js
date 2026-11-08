async function refreshAccountStatus(verify) {
    try {
        const res = await fetch(`/api/auth/status${verify ? '?verify=1' : ''}`);
        const data = await res.json();
        applyAccountStatus(data);
    } catch (e) {
        console.error('Erreur statut compte:', e);
    }
}

function applyAccountStatus(data) {
    State.loggedIn = !!data.logged_in;
    DOM.accountPseudo.textContent = State.loggedIn ? (data.pseudo || '') : '';
}

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

async function openAccountModal() {
    DOM.accountModal.classList.add('open');
    document.body.style.overflow = 'hidden';
    if (typeof renderLangSelectors === 'function') {
        renderLangSelectors();
    }

    const activeTab = document.querySelector('.netflix-sidebar-item.active');
    if (!activeTab) {
        switchSettingsTab('account');
    }

    fetch('/api/settings')
        .then(r => r.json())
        .then(d => {
            if (DOM.autoSyncCheckbox) DOM.autoSyncCheckbox.checked = !!d.auto_sync;
            if (DOM.adShieldCheckbox) DOM.adShieldCheckbox.checked = d.ad_shield !== false;
            if (DOM.autoUpdateCheckbox) DOM.autoUpdateCheckbox.checked = d.auto_update !== false;
            if (d.version) updateCurrentVersionDisplay(d.version);
        })
        .catch(() => {});

    await refreshAccountStatus(true);
    switchAccountMode(State.loggedIn ? 'cloud' : 'local');
}

function closeAccountModal() {
    DOM.accountModal.classList.remove('open');
    document.body.style.overflow = 'auto';
}

function switchAccountMode(mode) {
    DOM.modeLocalBtn.classList.toggle('active', mode === 'local');
    DOM.modeCloudBtn.classList.toggle('active', mode === 'cloud');
    DOM.panelLocal.style.display = mode === 'local' ? 'block' : 'none';

    if (mode === 'local') {
        DOM.panelCloudLoggedOut.style.display = 'none';
        DOM.panelCloudLoggedIn.style.display = 'none';
        return;
    }

    DOM.panelCloudLoggedOut.style.display = State.loggedIn ? 'none' : 'block';
    DOM.panelCloudLoggedIn.style.display = State.loggedIn ? 'block' : 'none';
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

async function handleAccountLogin(e) {
    e.preventDefault();
    const pseudo = document.getElementById('accountLoginPseudo').value.trim();
    const password = document.getElementById('accountLoginPassword').value;

    try {
        const res = await fetch('/api/auth/login', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ pseudo, password })
        });
        const data = await res.json();
        if (!res.ok) {
            showAccountError(data.error || 'Erreur de connexion');
            return;
        }
        DOM.accountError.style.display = 'none';
        applyAccountStatus({ logged_in: true, pseudo: data.pseudo });
        switchAccountMode('cloud');
        showToast(t('toast.connected', 'Connecté à anime-sama.to'));
    } catch (err) {
        showAccountError('Erreur de connexion au serveur local.');
    }
}

async function handleAccountRegister(e) {
    e.preventDefault();
    const pseudo = document.getElementById('accountRegisterPseudo').value.trim();
    const password = document.getElementById('accountRegisterPassword').value;
    const password2 = document.getElementById('accountRegisterPassword2').value;

    if (password !== password2) {
        showAccountError('Les mots de passe ne correspondent pas');
        return;
    }

    try {
        const res = await fetch('/api/auth/register', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ pseudo, password })
        });
        const data = await res.json();
        if (!res.ok) {
            showAccountError(data.error || 'Erreur de création de compte');
            return;
        }
        DOM.accountError.style.display = 'none';
        applyAccountStatus({ logged_in: true, pseudo: data.pseudo });
        switchAccountMode('cloud');
        showToast(t('toast.connected', 'Compte créé et connecté'));
    } catch (err) {
        showAccountError('Erreur de connexion au serveur local.');
    }
}

async function handleAccountLogout() {
    try {
        await fetch('/api/auth/logout', { method: 'POST' });
    } catch (e) {}
    applyAccountStatus({ logged_in: false, pseudo: null });
    switchAccountMode('cloud');
    showToast(t('toast.disconnected', 'Déconnecté'));
}

async function handleAutoSyncToggle(e) {
    const enabled = e.target.checked;
    try {
        await fetch('/api/settings/auto-sync', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ enabled })
        });
        showToast(enabled ? t('settings.autosync_enabled', 'Synchronisation automatique activée') : t('settings.autosync_disabled', 'Synchronisation automatique désactivée'));
        if (enabled) runSync('auto');
    } catch (err) {
        showToast('Impossible de sauvegarder ce réglage');
        e.target.checked = !enabled;
    }
}

async function handleAdShieldToggle(e) {
    const enabled = e.target.checked;
    try {
        await fetch('/api/settings/ad-shield', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ enabled })
        });
        showToast(enabled ? t('settings.adshield_enabled', 'Ad Shield activé') : t('settings.adshield_disabled', 'Ad Shield désactivé'));
    } catch (err) {
        showToast('Impossible de sauvegarder ce réglage');
        e.target.checked = !enabled;
    }
}

async function handleAutoUpdateToggle(e) {
    const enabled = e.target.checked;
    try {
        await fetch('/api/settings/auto-update', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ enabled })
        });
        showToast(enabled ? t('settings.autoupdate_enabled', 'Mise à jour automatique activée') : t('settings.autoupdate_disabled', 'Mise à jour automatique désactivée'));
    } catch (err) {
        showToast(t('settings.save_error', 'Impossible de sauvegarder ce réglage'));
        e.target.checked = !enabled;
    }
}

async function runSync(direction = 'auto') {
    DOM.syncAutoBtn.disabled = true;
    DOM.syncAutoBtn.classList.add('syncing');
    DOM.syncStatus.textContent = t('settings.sync_in_progress', 'Synchronisation en cours...');

    try {
        const res = await fetch(`/api/sync/${direction}`, { method: 'POST' });
        const data = await res.json();
        if (!res.ok) throw new Error(data.error || 'Erreur de synchronisation');

        if (data.userdata) {
            State.userData = data.userdata;
            updateWatchlistCount();
        }

        DOM.syncStatus.textContent = `Synchronisé : ${data.updated_local} en local, ${data.updated_cloud} vers le cloud.`;
        showToast(t('toast.sync_success', 'Synchronisation réussie'));
    } catch (e) {
        DOM.syncStatus.textContent = 'Échec de la synchronisation : ' + e.message;
        showToast(t('toast.sync_fail', 'Échec de la synchronisation'));
    } finally {
        DOM.syncAutoBtn.disabled = false;
        DOM.syncAutoBtn.classList.remove('syncing');
    }
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

    const badge = document.getElementById('updateBadgePill');
    if (badge && info.current_version && info.latest_version) {
        badge.textContent = `v${info.current_version} → ${info.latest_version}`;
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
                        bypassBtn.style.color = '#46d369';
                        bypassBtn.style.fontWeight = '700';
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
                    bypassBtn.textContent = t('update.bypass', 'Passer et accéder au catalogue');
                    bypassBtn.style.color = '#ffffff';
                }
            }
        } catch (e) {
        }
    }, 120);
}

async function applyUpdateDirect() {
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
