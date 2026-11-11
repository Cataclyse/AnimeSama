(() => {
    'use strict';

    const PREFIX = 'cataclyse:p2:';
    const STATUS_URL = '/api/local-auth/status';

    let userId = null;
    let generation = 0;
    let pendingRefresh = null;

    function snapshot() {
        return { id: userId, generation };
    }

    function updateIdentity(nextId) {
        if (userId === nextId) return;

        userId = nextId;
        generation += 1;

        window.dispatchEvent(new CustomEvent(
            'cataclyse-playback-identity-change',
            { detail: snapshot() }
        ));
    }

    function parseIdentity(data) {
        const id = data?.logged_in ? data.user?.id : null;

        return Number.isSafeInteger(id) && id > 0
            ? id
            : null;
    }

    async function refresh() {
        if (pendingRefresh) return pendingRefresh;

        const task = (async () => {
            try {
                const response = await fetch(STATUS_URL, {
                    credentials: 'same-origin',
                    cache: 'no-store'
                });

                if (!response.ok) {
                    updateIdentity(null);
                    return snapshot();
                }

                updateIdentity(
                    parseIdentity(await response.json())
                );

                return snapshot();
            } catch (_) {
                // Pas de données personnelles sans identité vérifiée.
                updateIdentity(null);
                return snapshot();
            }
        })();

        pendingRefresh = task;

        try {
            return await task;
        } finally {
            if (pendingRefresh === task) {
                pendingRefresh = null;
            }
        }
    }

    function storageKey(type, parts) {
        if (userId === null) return null;

        return PREFIX + userId + ':' + type + ':' +
            encodeURIComponent(JSON.stringify(parts));
    }

    function read(type, parts) {
        const key = storageKey(type, parts);
        if (!key) return null;

        try {
            const value = localStorage.getItem(key);
            return value ? JSON.parse(value) : null;
        } catch (_) {
            return null;
        }
    }

    function write(type, parts, value) {
        const key = storageKey(type, parts);
        if (!key) return;

        try {
            localStorage.setItem(key, JSON.stringify(value));
        } catch (_) {
            // Stockage désactivé ou quota dépassé.
        }
    }

    function validProgress(p) {
        return p &&
            typeof p.slug === 'string' &&
            p.slug.length > 0 && p.slug.length <= 200 &&
            typeof p.season === 'string' &&
            p.season.length > 0 && p.season.length <= 100 &&
            Number.isInteger(Number(p.episode)) &&
            Number(p.episode) >= 1 &&
            Number(p.episode) <= 10000 &&
            Number.isFinite(Number(p.currentTime)) &&
            Number.isFinite(Number(p.duration)) &&
            Number(p.currentTime) >= 0 &&
            Number(p.duration) >= 0;
    }

    function save(progress) {
        if (userId === null || !validProgress(progress)) {
            return;
        }

        const p = {
            ...progress,
            episode: Number(progress.episode),
            currentTime: Number(progress.currentTime),
            duration: Number(progress.duration),
            timestamp: Number(progress.timestamp || Date.now())
        };

        const episodeData = {
            currentTime: p.currentTime,
            duration: p.duration,
            season: p.season,
            episode: p.episode,
            timestamp: p.timestamp
        };

        write('anime', [p.slug], p);
        write('episode', [
            p.slug, p.season, p.episode
        ], episodeData);
    }

    function getPlayback(slug) {
        return read('anime', [String(slug)]);
    }

    function getEpisode(slug, episode, season) {
        const animeSlug = String(slug);
        const episodeNum = Number(episode);

        if (!Number.isInteger(episodeNum)) return null;

        if (season) {
            const exact = read('episode', [
                animeSlug, String(season), episodeNum
            ]);

            if (exact) return exact;
        }

        const anime = getPlayback(animeSlug);

        if (!anime || Number(anime.episode) !== episodeNum) {
            return null;
        }

        // Pas de mélange entre VF, VOSTFR ou saisons.
        if (season && anime.season !== String(season)) {
            return null;
        }

        return anime;
    }

    function list() {
        if (userId === null) return [];

        const prefix = PREFIX + userId + ':anime:';
        const result = [];

        try {
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                if (!key || !key.startsWith(prefix)) continue;

                try {
                    const parts = JSON.parse(
                        decodeURIComponent(key.slice(prefix.length))
                    );

                    if (!Array.isArray(parts) ||
                        parts.length !== 1 ||
                        typeof parts[0] !== 'string') {
                        continue;
                    }

                    const value = JSON.parse(
                        localStorage.getItem(key) || 'null'
                    );

                    if (validProgress(value) &&
                        value.slug === parts[0]) {
                        result.push(value);
                    }
                } catch (_) {}
            }
        } catch (_) {}

        return result;
    }

    function remove(slug) {
        if (userId === null || typeof slug !== 'string') {
            return false;
        }

        const animeKey = storageKey('anime', [slug]);
        const episodePrefix = PREFIX + userId + ':episode:';
        const toDelete = [animeKey];

        try {
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);

                if (!key || !key.startsWith(episodePrefix)) {
                    continue;
                }

                try {
                    const parts = JSON.parse(
                        decodeURIComponent(
                            key.slice(episodePrefix.length)
                        )
                    );

                    if (Array.isArray(parts) && parts[0] === slug) {
                        toDelete.push(key);
                    }
                } catch (_) {}
            }

            for (const key of toDelete) {
                localStorage.removeItem(key);
            }

            return true;
        } catch (_) {
            return false;
        }
    }

    window.CataclysePlaybackCache = Object.freeze({
        snapshot,
        refresh,
        list,
        remove
    });

    // Ces fonctions remplacent les anciennes uniquement
    // lorsque ce module sera chargé après app.js.
    window.savePlaybackProgress = save;
    window.getPlaybackProgress = getPlayback;
    window.getEpisodeProgress = getEpisode;

    window.addEventListener('focus', () => {
        void refresh();
    });

    document.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'visible') {
            void refresh();
        }
    });

    void refresh();
})();
