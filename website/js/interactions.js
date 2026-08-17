/* =========================================================
   Oracle — Interactions: Cursor, Scroll, Counters,
   Typewriter, Magnetic Buttons, Reveal Animations
   ========================================================= */

(function () {
    // ═══════════════════════════════
    // 1. CUSTOM CURSOR
    // ═══════════════════════════════
    const ring = document.getElementById('cursor-ring');
    const dot = document.getElementById('cursor-dot');
    let cursorX = 0, cursorY = 0;
    let ringX = 0, ringY = 0;

    document.addEventListener('mousemove', e => {
        cursorX = e.clientX;
        cursorY = e.clientY;
        dot.style.left = cursorX + 'px';
        dot.style.top = cursorY + 'px';
    });

    function animateCursor() {
        ringX += (cursorX - ringX) * 0.15;
        ringY += (cursorY - ringY) * 0.15;
        ring.style.left = ringX + 'px';
        ring.style.top = ringY + 'px';
        requestAnimationFrame(animateCursor);
    }
    animateCursor();

    // Hover states: enlarge ring on interactive elements
    const hoverTargets = document.querySelectorAll('a, button, .magnetic, .hover-item, .team-card, .problem-card, .formula-card, input[type=range]');
    hoverTargets.forEach(el => {
        el.addEventListener('mouseenter', () => ring.classList.add('hover'));
        el.addEventListener('mouseleave', () => ring.classList.remove('hover'));
    });

    // ═══════════════════════════════
    // 2. SCROLL PROGRESS BAR
    // ═══════════════════════════════
    const progressBar = document.getElementById('scroll-progress');

    function updateProgress() {
        const scrollTop = window.scrollY;
        const docHeight = document.documentElement.scrollHeight - window.innerHeight;
        const progress = (scrollTop / docHeight) * 100;
        progressBar.style.width = progress + '%';
    }
    window.addEventListener('scroll', updateProgress, { passive: true });

    // ═══════════════════════════════
    // 3. ANIMATED COUNTERS
    // ═══════════════════════════════
    const counters = document.querySelectorAll('.counter');
    const counterObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting && !entry.target.dataset.counted) {
                entry.target.dataset.counted = 'true';
                const target = parseInt(entry.target.dataset.target);
                const duration = 2000;
                const start = performance.now();

                function tick(now) {
                    const elapsed = now - start;
                    const progress = Math.min(elapsed / duration, 1);
                    // Ease-out
                    const eased = 1 - Math.pow(1 - progress, 3);
                    entry.target.textContent = Math.floor(eased * target);
                    if (progress < 1) requestAnimationFrame(tick);
                    else entry.target.textContent = target;
                }
                requestAnimationFrame(tick);
            }
        });
    }, { threshold: 0.5 });
    counters.forEach(c => counterObserver.observe(c));

    // ═══════════════════════════════
    // 4. TYPEWRITER EFFECT
    // ═══════════════════════════════
    const typewriterEls = document.querySelectorAll('.typewriter');
    typewriterEls.forEach(el => {
        const text = el.dataset.text;
        let i = 0;
        el.innerHTML = '<span class="tw-cursor">_</span>';

        const typeObserver = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting && i === 0) {
                    function typeChar() {
                        if (i < text.length) {
                            el.innerHTML = text.substring(0, i + 1) + '<span class="tw-cursor">_</span>';
                            i++;
                            setTimeout(typeChar, 25 + Math.random() * 35);
                        } else {
                            // Blinking cursor
                            el.innerHTML = text + '<span class="tw-cursor blink">_</span>';
                        }
                    }
                    setTimeout(typeChar, 800);
                }
            });
        }, { threshold: 0.5 });
        typeObserver.observe(el);
    });

    // Add blink style dynamically
    const style = document.createElement('style');
    style.textContent = `
        .tw-cursor { color: var(--accent); }
        .tw-cursor.blink { animation: cursorBlink 1s step-end infinite; }
        @keyframes cursorBlink { 0%,100%{opacity:1} 50%{opacity:0} }
    `;
    document.head.appendChild(style);

    // ═══════════════════════════════
    // 5. MAGNETIC BUTTONS
    // ═══════════════════════════════
    const magneticEls = document.querySelectorAll('.magnetic');
    magneticEls.forEach(el => {
        el.addEventListener('mousemove', e => {
            const rect = el.getBoundingClientRect();
            const x = e.clientX - rect.left - rect.width / 2;
            const y = e.clientY - rect.top - rect.height / 2;
            el.style.transform = `translate(${x * 0.15}px, ${y * 0.15}px)`;
        });
        el.addEventListener('mouseleave', () => {
            el.style.transform = 'translate(0, 0)';
            el.style.transition = 'transform 0.4s ease-out';
            setTimeout(() => el.style.transition = '', 400);
        });
    });

    // ═══════════════════════════════
    // 6. REVEAL ON SCROLL
    // ═══════════════════════════════
    const revealEls = document.querySelectorAll('.reveal-left, .reveal-right, .reveal-up');
    const revealObserver = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('visible');
            }
        });
    }, { threshold: 0.15 });
    revealEls.forEach(el => revealObserver.observe(el));

    // ═══════════════════════════════
    // 7. ROI BAR ANIMATION
    // ═══════════════════════════════
    const roiBar = document.querySelector('.roi-bar-fill');
    if (roiBar) {
        const barObserver = new IntersectionObserver((entries) => {
            entries.forEach(entry => {
                if (entry.isIntersecting) {
                    roiBar.style.width = roiBar.dataset.width + '%';
                }
            });
        }, { threshold: 0.5 });
        barObserver.observe(roiBar);
    }

    // ═══════════════════════════════
    // 8. SECTION NAV ACTIVE STATE
    // ═══════════════════════════════
    const sections = document.querySelectorAll('section[id]');
    const navDots = document.querySelectorAll('.section-dot');

    function updateNav() {
        let current = '';
        sections.forEach(sec => {
            if (sec.getBoundingClientRect().top <= window.innerHeight / 2) {
                current = sec.id;
            }
        });
        navDots.forEach(dot => {
            dot.classList.toggle('active', dot.getAttribute('href') === '#' + current);
        });
    }
    window.addEventListener('scroll', updateNav, { passive: true });
    updateNav();

    // ═══════════════════════════════
    // 9. SMOOTH ANCHOR SCROLL
    // ═══════════════════════════════
    document.querySelectorAll('a[href^="#"]').forEach(a => {
        a.addEventListener('click', e => {
            e.preventDefault();
            const target = document.querySelector(a.getAttribute('href'));
            if (target) {
                target.scrollIntoView({ behavior: 'smooth', block: 'start' });
            }
        });
    });
})();
