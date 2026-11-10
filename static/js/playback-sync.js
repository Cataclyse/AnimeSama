(() => {
    'use strict';

    const endpoint = '/api/playback/position';
    const originalSave = window.savePlaybackProgress;
    const originalOpen = window.openEpisodePlayer;

    if (typeof originalSave !== 'function' ||
        typeof originalOpen !== 'function') {
        console.warn('[PlaybackSync] Fonctions du lecteur indisponibles');
        return;
    }

    const pending = new Map();
    const timers = new Map();

    function normalize(p) {
        if (!p) return null;

        const slug = String(p.slug || '');
        const season = String(p.season || '');
        const episode = Number(p.episode);
        const currentTime = Number(p.currentTime);
        const duration = Number(p.duration);
        const timestamp = Number(p.timestamp || Date.now());

        if (!slug || slug.length > 200 ||
            !season || season.length > 100 ||
            !Number.isInteger(episode) ||
            episode < 1 || episode > 10000 ||
            !Number.isFinite(currentTime) ||
            !Number.isFinite(duration) ||
            currentTime < 0 || duration < 0 ||
            currentTime > 864000 || duration > 864000 ||
            !Number.isSafeInteger(timestamp) ||
            timestamp <= 0) {
            return null;
        }

        return {
            slug, season, episode,
            currentTime, duration, timestamp
        };
    }

    function positionKey(p) {
        return JSON.stringify([p.slug, p.season, p.episode]);
    }

    function sendPending(key, keepalive = false) {
        const value = pending.get(key);
        pending.delete(key);

        if (!value) return;

        fetch(endpoint, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(value),
            keepalive
        }).then(response => {
            if (!response.ok) {
                throw new Error('HTTP ' + response.status);
            }
        }).catch(error => {
            console.warn('[PlaybackSync] Enregistrement distant échoué:', error);
        });
    }

    window.savePlaybackProgress = function(progress) {
        const result = originalSave.apply(this, arguments);
        const value = normalize(progress);

        if (!value) return result;

        const key = positionKey(value);
        pending.set(key, value);

        if (!timers.has(key)) {
            const timer = setTimeout(() => {
                timers.delete(key);
                sendPending(key);
            }, 3000);
            timers.set(key, timer);
        }

        return result;
    };

    window.addEventListener('pagehide', () => {
        for (const timer of timers.values()) {
            clearTimeout(timer);
        }
        timers.clear();

        for (const key of [...pending.keys()]) {
            sendPending(key, true);
        }
    });

    window.openEpisodePlayer = async function(...args) {
        const [slug, season, index] = args;
        const episode = Number(index) + 1;

        if (slug && season &&
            Number.isInteger(episode) && episode > 0) {

            const controller = new AbortController();
            const timeout = setTimeout(() => controller.abort(), 1500);

            try {
                const params = new URLSearchParams({
                    slug: String(slug),
                    season: String(season),
                    episode: String(episode)
                });

                const response = await fetch(
                    endpoint + '?' + params,
                    {
                        cache: 'no-store',
                        signal: controller.signal
                    }
                );

                if (response.ok) {
                    const body = await response.json();
                    const remote = normalize(body.position);

                    const local = typeof window.getEpisodeProgress === 'function'
                        ? window.getEpisodeProgress(slug, episode, season)
                        : null;

                    const localTimestamp = Number(local?.timestamp || 0);

                    if (remote &&
                        remote.slug === slug &&
                        remote.season === season &&
                        remote.episode === episode &&
                        remote.timestamp > localTimestamp) {

                        originalSave(remote);

                        // Les appels existants peuvent fournir une position
                        // locale explicite. La position distante plus récente
                        // doit alors être utilisée pour cette ouverture.
                        args[3] = remote.currentTime;
                    }
                }
            } catch (error) {
                console.debug('[PlaybackSync] Reprise locale:', error.message);
            } finally {
                clearTimeout(timeout);
            }
        }

        return originalOpen.apply(this, args);
    };
})();
