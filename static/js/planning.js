function initPlanning() {
    const planning = State.homeData.planning || {};
    const days = ['Dimanche', 'Lundi', 'Mardi', 'Mercredi', 'Jeudi', 'Vendredi', 'Samedi'];
    const todayIndex = new Date().getDay();
    const todayName = days[todayIndex];

    
    const tabs = DOM.planningTabs.querySelectorAll('.planning-tab');
    tabs.forEach(tab => {
        const day = tab.dataset.day;
        if (day.toLowerCase() === todayName.toLowerCase()) {
            tab.classList.add('active');
        }
        tab.onclick = () => {
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            renderPlanningDay(day);
        };
    });

    renderPlanningDay(todayName);
}

function renderPlanningDay(dayName) {
    const planning = State.homeData.planning || {};
    
    let matchedKey = Object.keys(planning).find(k => k.toLowerCase().includes(dayName.toLowerCase()));
    if (!matchedKey && Object.keys(planning).length) {
        matchedKey = Object.keys(planning)[0];
    }

    const cards = matchedKey ? (planning[matchedKey] || []) : [];
    DOM.planningGrid.innerHTML = cards.length 
        ? cards.map(c => createCardHTML(c)).join('')
        : `<p style="color:#888; padding:20px;">${t('planning.empty', 'Aucune sortie programmée pour ce jour.')}</p>`;
}
