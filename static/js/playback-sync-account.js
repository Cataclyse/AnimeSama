(() => {
    'use strict';

    const endpoint = '/api/playback/position';
    const originalSave = window.savePlaybackProgress;
    const originalOpen = window.openEpisodePlayer;
    const cache = window.CataclysePlaybackCache;

    if (typeof originalSave !== 'function' ||
        typeof originalOpen !== 'function' ||
        !cache ||
        typeof cache.refresh !== 'function' ||
        typeof cache.snapshot !== 'function') {
        console.warn('[PlaybackSync] Lecteur indisponible');
        return;
    }

    const slots = new Map();
    const activeReads = new Set();
    const activeWrites = new Set();
    let opening = 0;

    function sameAccount(snapshot) {
        const now = cache.snapshot();
        return now.id === snapshot.id &&
            now.generation === snapshot.generation;
    }

    window.addEventListener(
        'cataclyse-playback-identity-change',
        () => {
            opening += 1;

            for (const controller of activeReads) {
                controller.abort();
            }
            activeReads.clear();

            for (const controller of activeWrites) {
                controller.abort();
            }
            activeWrites.clear();

            for (const slot of slots.values()) {
                if (slot.timer !== null) {
                    clearTimeout(slot.timer);
                }
                slot.pending = null;
                slot.blocked = true;
            }
            slots.clear();
        }
    );

    function normalize(p) {
        if (!p) return null;

        const value = {
            slug: String(p.slug || ''),
            season: String(p.season || ''),
            episode: Number(p.episode),
            currentTime: Number(p.currentTime),
            duration: Number(p.duration),
            timestamp: Number(p.timestamp || Date.now()),
            revision: Number(p.revision ?? 0)
        };

        if (!value.slug || value.slug.length > 200 ||
            !value.season || value.season.length > 100 ||
            !Number.isInteger(value.episode) ||
            value.episode < 1 || value.episode > 10000 ||
            !Number.isFinite(value.currentTime) ||
            !Number.isFinite(value.duration) ||
            value.currentTime < 0 ||
            value.duration < 0 ||
            value.currentTime > 864000 ||
            value.duration > 864000 ||
            !Number.isSafeInteger(value.timestamp) ||
            value.timestamp <= 0 ||
            !Number.isSafeInteger(value.revision) ||
            value.revision < 0) {
            return null;
        }

        return value;
    }

    function keyFor(p) {
        return JSON.stringify([p.slug, p.season, p.episode]);
    }

    function slotFor(key) {
        if (!slots.has(key)) {
            slots.set(key, {
                revision: null,
                blocked: false,
                pending: null,
                timer: null,
                busy: false,
                retryDelay: 3000
            });
        }
        return slots.get(key);
    }

    function schedule(key, delay = 3000) {
        const slot = slotFor(key);

        if (slot.timer !== null ||
            slot.busy ||
            slot.blocked ||
            slot.revision === null ||
            !slot.pending) {
            return;
        }

        slot.timer = setTimeout(() => {
            slot.timer = null;
            void flush(key);
        }, delay);
    }

    async function flush(key, keepalive = false) {
        const slot = slotFor(key);

        if (slot.busy ||
            slot.blocked ||
            slot.revision === null ||
            !slot.pending) {
            return;
        }

        const identity = cache.snapshot();

        if (!Number.isSafeInteger(identity.id) ||
            identity.id <= 0) {
            slot.pending = null;
            slot.blocked = true;
            return;
        }

        const position = slot.pending;
        slot.pending = null;
        slot.busy = true;

        const controller = new AbortController();
        activeWrites.add(controller);

        try {
            const response = await fetch(endpoint, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-Cataclyse-Expected-User': String(identity.id)
                },
                credentials: 'same-origin',
                signal: controller.signal,
                body: JSON.stringify({
                    ...position,
                    revision: slot.revision
                }),
                keepalive
            });

            if (!sameAccount(identity)) return;

            if (response.status === 401 ||
                response.status === 403) {
                slot.blocked = true;
                slot.pending = null;
                void cache.refresh();
                return;
            }

            if (response.status === 409) {
                slot.blocked = true;
                slot.pending = null;

                if (slot.timer !== null) {
                    clearTimeout(slot.timer);
                    slot.timer = null;
                }

                console.warn(
                    '[PlaybackSync] Conflit de révision :',
                    key
                );

                if (typeof window.showToast === 'function') {
                    window.showToast(
                        'Progression modifiée sur un autre appareil. ' +
                        'Rouvrez cet épisode pour la récupérer.'
                    );
                }
                return;
            }

            if (!response.ok) {
                throw new Error('HTTP ' + response.status);
            }

            const result = await response.json();

            if (!sameAccount(identity)) return;

            const accepted = normalize(result.position);

            if (!accepted ||
                accepted.revision <= slot.revision) {
                throw new Error('Révision serveur invalide');
            }

            slot.revision = accepted.revision;
            slot.retryDelay = 3000;

        } catch (error) {
            if (sameAccount(identity) && !slot.blocked) {
                if (!slot.pending ||
                    position.timestamp > slot.pending.timestamp) {
                    slot.pending = position;
                }
                slot.retryDelay = 15000;
            }

            console.warn(
                '[PlaybackSync] Sauvegarde distante différée :',
                error
            );
        } finally {
            activeWrites.delete(controller);
            slot.busy = false;

            if (!keepalive &&
                sameAccount(identity) &&
                slots.get(key) === slot) {
                schedule(key, slot.retryDelay);
            }
        }
    }

    window.savePlaybackProgress = function(progress) {
        const result = originalSave.apply(this, arguments);
        const position = normalize(progress);

        if (!position || cache.snapshot().id === null) {
            return result;
        }

        const key = keyFor(position);
        const slot = slotFor(key);

        if (!slot.blocked) {
            slot.pending = position;
            schedule(key);
        }

        return result;
    };

    window.addEventListener('pagehide', () => {
        for (const [key, slot] of slots) {
            if (slot.timer !== null) {
                clearTimeout(slot.timer);
                slot.timer = null;
            }

            if (slot.pending && !slot.blocked) {
                void flush(key, true);
            }
        }
    });

    window.openEpisodePlayer = async function(...args) {
        await cache.refresh();

        const sequence = ++opening;
        const identity = cache.snapshot();
        const [slug, season, index] = args;
        const episode = Number(index) + 1;

        if (slug && season &&
            Number.isInteger(episode) &&
            episode > 0 && episode <= 10000 &&
            Number.isSafeInteger(identity.id) &&
            identity.id > 0) {

            const key = keyFor({
                slug: String(slug),
                season: String(season),
                episode
            });

            const controller = new AbortController();
            activeReads.add(controller);

            const timeout = setTimeout(
                () => controller.abort(),
                1500
            );

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
                        credentials: 'same-origin',
                        headers: {
                            'X-Cataclyse-Expected-User':
                                String(identity.id)
                        },
                        signal: controller.signal
                    }
                );

                if (sequence !== opening || !sameAccount(identity)) return;

                if (response.status === 401 ||
                    response.status === 403) {
                    void cache.refresh();
                    return;
                }

                if (!response.ok) {
                    throw new Error('HTTP ' + response.status);
                }

                const body = await response.json();

                if (sequence !== opening ||
                    !sameAccount(identity)) return;
                const remote = body.position == null
                    ? null
                    : normalize(body.position);

                if (body.position != null && !remote) {
                    throw new Error('Position distante invalide');
                }

                const slot = slotFor(key);
                const wasBlocked = slot.blocked;

                slot.revision = remote ? remote.revision : 0;
                slot.blocked = false;
                slot.retryDelay = 3000;

                if (wasBlocked) {
                    slot.pending = null;
                    if (slot.timer !== null) {
                        clearTimeout(slot.timer);
                        slot.timer = null;
                    }
                }

                const local =
                    typeof window.getEpisodeProgress === 'function'
                    ? window.getEpisodeProgress(
                        slug, episode, season
                    )
                    : null;

                const localTimestamp =
                    Number(local?.timestamp || 0);

                if (remote &&
                    (wasBlocked ||
                     remote.timestamp > localTimestamp)) {

                    originalSave(remote);
                    args[3] = remote.currentTime;
                }

                schedule(key);

            } catch (error) {
                console.debug(
                    '[PlaybackSync] Reprise locale :',
                    error.message
                );
            } finally {
                activeReads.delete(controller);
                clearTimeout(timeout);
            }
        }

        if (sequence !== opening || !sameAccount(identity)) return;
        return originalOpen.apply(this, args);
    };
})();
