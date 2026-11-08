function initBillboard() {
    const slides = State.homeData.slides || [];
    if (!slides.length) return;

    
    DOM.heroIndicators.innerHTML = slides.map((_, idx) => `
        <div class="indicator-dot ${idx === 0 ? 'active' : ''}" onclick="goToSlide(${idx})"></div>
    `).join('');

    renderSlide(0);
    startSlideTimer();
}

function renderSlide(index) {
    const slides = State.homeData.slides;
    if (!slides || !slides[index]) return;

    State.currentSlide = index;
    const slide = slides[index];

    
    DOM.heroBackdrop.style.opacity = '0';
    setTimeout(() => {
        const bgImg = slide.bg || `/api/img?slug=${slide.slug}`;
        DOM.heroBackdrop.style.backgroundImage = `url('${bgImg}')`;
        DOM.heroBackdrop.style.opacity = '1';
    }, 200);

    
    DOM.heroBadge.textContent = slide.badge || 'Populaire';
    DOM.heroTitle.textContent = slide.title;
    DOM.heroSynopsis.textContent = slide.synopsis;
    if (typeof translateContent === 'function' && State.appLang && State.appLang !== 'fr' && slide.synopsis) {
        translateContent(slide.synopsis, State.appLang).then(translated => {
            if (State.currentSlide === index && DOM.heroSynopsis) {
                DOM.heroSynopsis.textContent = translated;
            }
        });
    }

    
    DOM.heroGenres.innerHTML = (slide.genres || []).map(g => `<span class="genre-tag">${g}</span>`).join('');

    
    const dots = DOM.heroIndicators.querySelectorAll('.indicator-dot');
    dots.forEach((d, i) => d.classList.toggle('active', i === index));

    
    const inList = State.userData.watchlist.includes(slide.slug);
    updateWatchlistButton(DOM.heroListBtn, inList);

    
    DOM.heroPlayBtn.onclick = () => openEpisodePlayer(slide.slug, getSeasonPathForLang('saison1/vostfr', State.selectedLanguage), 0);
    DOM.heroInfoBtn.onclick = () => openDetailModal(slide.slug);
    DOM.heroListBtn.onclick = () => toggleWatchlist(slide.slug);
}

function goToSlide(index) {
    renderSlide(index);
    startSlideTimer();
}

function startSlideTimer() {
    clearInterval(State.slideTimer);
    State.slideTimer = setInterval(() => {
        const next = (State.currentSlide + 1) % (State.homeData.slides.length || 1);
        renderSlide(next);
    }, 9000);
}
