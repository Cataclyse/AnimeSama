function showToast(message) {
    DOM.toast.textContent = message;
    DOM.toast.classList.add('show');
    setTimeout(() => {
        DOM.toast.classList.remove('show');
    }, 2800);
}

function toProxyUrl(url) {
    try {
        const u = new URL(url);
        return `/proxy/${u.host}${u.pathname}${u.search}`;
    } catch (e) {
        return url;
    }
}

function escapeHtml(str) {
    if (!str) return '';
    return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
}

const PREFERRED_PLAYERS = ['Uqload', 'VOE', 'VidMoly', 'Sibnet', 'Sendvid', 'Earnvids', 'DoodStream', 'StreamTape', 'OneUpload', 'FileMoon'];

function pickBestPlayer(players, current) {
    if (!players || !players.length) return null;
    if (current && players.includes(current)) return current;
    for (const pref of PREFERRED_PLAYERS) {
        const found = players.find(p => p.toLowerCase().includes(pref.toLowerCase()));
        if (found) return found;
    }
    return players[0];
}
