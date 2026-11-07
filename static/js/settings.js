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

async function openAccountModal() {
    DOM.accountModal.classList.add('open');
    document.body.style.overflow = 'hidden';
    if (typeof renderLangSelectors === 'function') {
        renderLangSelectors();
    }

    fetch('/api/settings')
        .then(r => r.json())
        .then(d => {
            DOM.autoSyncCheckbox.checked = !!d.auto_sync;
            DOM.adShieldCheckbox.checked = d.ad_shield !== false;
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
