async function performSearch(query) {
    DOM.searchStatus.textContent = `Recherche pour "${query}"...`;
    try {
        const res = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
        if (!res.ok) throw new Error('Erreur HTTP ' + res.status);
        const data = await res.json();
        const results = data.results || [];

        DOM.searchStatus.textContent = `${results.length} résultat(s) pour "${query}"`;
        DOM.searchGrid.innerHTML = results.length
            ? results.map(c => createCardHTML(c)).join('')
            : '<p style="color:#888; grid-column:1/-1;">Aucun résultat trouvé pour votre recherche.</p>';
    } catch (e) {
        console.error('Erreur recherche:', e);
        DOM.searchStatus.textContent = 'Erreur lors de la recherche.';
    }
}

function filterByGenre(genre) {
    document.getElementById('viewGrid').classList.add('active');
    DOM.gridTitle.textContent = `Genre : ${genre}`;
    DOM.gridSubtitle.textContent = `Toutes les œuvres associées au genre ${genre}`;

    const filtered = State.catalogue.filter(c => 
        (c.genres || []).some(g => g.toLowerCase().includes(genre.toLowerCase())) ||
        (c.badges || []).some(b => b.toLowerCase().includes(genre.toLowerCase()))
    );

    DOM.catalogueGrid.innerHTML = filtered.length
        ? filtered.map(c => createCardHTML(c)).join('')
        : '<p style="color:#888; grid-column:1/-1;">Aucune œuvre trouvée pour ce genre.</p>';
}
