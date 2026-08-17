/* =========================================================
   Oracle — Interactive ASCII Background
   Features:
   1. Falling ASCII rain (streams of characters falling down)
   2. Mouse interaction (particles scatter + morph)
   3. Bacteria colonies that pulse and drift
   4. Floating DNA/bio glyphs that drift horizontally
   ========================================================= */

(function () {
    const canvas = document.getElementById('ascii-bg');
    if (!canvas) return;

    const ctx = canvas.getContext('2d');

    // ── Config ──
    const CELL_SIZE = 14;
    const MOUSE_RADIUS = 160;

    // Glyph sets
    const CHARS_RAIN = '01.:+*·|!;:.,\'`^"~-_=/\\<>{}[]()#@&%$';
    const CHARS_BIO = '○◎●◉⊕⊗⊙⦿※✦∘∙⁘⟨⟩⌇⏣◬△▷◇◈⬡⬢';
    const CHARS_DNA = '╔╗╚╝║═╠╣╬├┤┌┐└┘│─┼▄▀█░▒▓';

    let W, H;
    let mouse = { x: -9999, y: -9999, active: false };
    let time = 0;

    // ═══════════════════════════════════════════
    // 1. RAIN STREAMS (Matrix-style falling columns)
    // ═══════════════════════════════════════════
    let rainColumns = [];

    class RainDrop {
        constructor(col) {
            this.col = col;
            this.x = col * CELL_SIZE;
            this.reset();
        }

        reset() {
            this.y = -Math.random() * H * 1.5;
            this.speed = 1.2 + Math.random() * 3.5;
            this.length = 8 + Math.floor(Math.random() * 20);
            this.chars = [];
            for (let i = 0; i < this.length; i++) {
                this.chars.push(CHARS_RAIN[Math.floor(Math.random() * CHARS_RAIN.length)]);
            }
            this.mutateRate = 0.02 + Math.random() * 0.05;
        }

        update() {
            this.y += this.speed;

            // Mutate chars occasionally
            for (let i = 0; i < this.chars.length; i++) {
                if (Math.random() < this.mutateRate) {
                    this.chars[i] = CHARS_RAIN[Math.floor(Math.random() * CHARS_RAIN.length)];
                }
            }

            if (this.y - this.length * CELL_SIZE > H) {
                this.reset();
            }
        }

        draw() {
            for (let i = 0; i < this.length; i++) {
                const cy = this.y - i * CELL_SIZE;
                if (cy < -CELL_SIZE || cy > H + CELL_SIZE) continue;

                // Mouse repulsion check
                const dx = mouse.x - this.x;
                const dy = mouse.y - cy;
                const dist = Math.sqrt(dx * dx + dy * dy);
                let offsetX = 0;
                let offsetY = 0;
                let boost = 0;

                if (dist < MOUSE_RADIUS && mouse.active) {
                    const force = (MOUSE_RADIUS - dist) / MOUSE_RADIUS;
                    const angle = Math.atan2(dy, dx);
                    offsetX = -Math.cos(angle) * force * 30;
                    offsetY = -Math.sin(angle) * force * 15;
                    boost = force;
                }

                // Fade: head is bright, tail fades
                const headFade = i === 0 ? 1 : (1 - i / this.length);
                const alpha = 0.03 + headFade * 0.15 + boost * 0.5;

                // Color: head is white/red, tail is dim
                if (boost > 0.3) {
                    const r = 255;
                    const g = Math.floor(50 * (1 - boost));
                    const b = Math.floor(60 + 40 * (1 - boost));
                    ctx.fillStyle = `rgba(${r},${g},${b},${Math.min(0.9, alpha)})`;
                } else if (i === 0) {
                    ctx.fillStyle = `rgba(220,220,220,${alpha})`;
                } else {
                    ctx.fillStyle = `rgba(120,120,120,${alpha})`;
                }

                const ch = boost > 0.5
                    ? CHARS_BIO[Math.floor(Math.random() * CHARS_BIO.length)]
                    : this.chars[i];

                ctx.fillText(ch, this.x + offsetX, cy + offsetY);
            }
        }
    }

    // ═══════════════════════════════════════════
    // 2. FLOATING BIO GLYPHS (horizontal drifters)
    // ═══════════════════════════════════════════
    let floaters = [];

    class Floater {
        constructor() {
            this.reset(true);
        }

        reset(initial) {
            this.x = initial ? Math.random() * W : -30;
            this.y = Math.random() * H;
            this.speed = 0.3 + Math.random() * 0.8;
            this.size = 10 + Math.random() * 6;
            this.alpha = 0.04 + Math.random() * 0.1;
            this.char = CHARS_DNA[Math.floor(Math.random() * CHARS_DNA.length)];
            this.wobblePhase = Math.random() * Math.PI * 2;
            this.wobbleAmp = 10 + Math.random() * 30;
            this.wobbleSpeed = 0.005 + Math.random() * 0.015;
        }

        update() {
            this.x += this.speed;
            if (this.x > W + 30) this.reset(false);
        }

        draw() {
            const yOff = Math.sin(time * this.wobbleSpeed + this.wobblePhase) * this.wobbleAmp;
            const drawY = this.y + yOff;

            // Mouse proximity
            const dx = mouse.x - this.x;
            const dy = mouse.y - drawY;
            const dist = Math.sqrt(dx * dx + dy * dy);
            let alpha = this.alpha;

            if (dist < MOUSE_RADIUS && mouse.active) {
                const force = (MOUSE_RADIUS - dist) / MOUSE_RADIUS;
                alpha = Math.min(0.7, alpha + force * 0.5);
                ctx.fillStyle = `rgba(255, 0, 60, ${alpha})`;
                this.char = CHARS_BIO[Math.floor(Math.random() * CHARS_BIO.length)];
            } else {
                ctx.fillStyle = `rgba(100, 100, 100, ${alpha})`;
            }

            ctx.font = `${this.size}px 'Press Start 2P', monospace`;
            ctx.fillText(this.char, this.x, drawY);
        }
    }

    // ═══════════════════════════════════════════
    // 3. BACTERIA COLONIES (pulsing clusters)
    // ═══════════════════════════════════════════
    let colonies = [];

    function createColonies() {
        colonies = [];
        const num = Math.max(3, Math.floor((W * H) / 300000));
        for (let i = 0; i < num; i++) {
            colonies.push({
                cx: Math.random() * W,
                cy: Math.random() * H,
                radius: 50 + Math.random() * 120,
                phase: Math.random() * Math.PI * 2,
                pulseSpeed: 0.008 + Math.random() * 0.02,
                driftX: (Math.random() - 0.5) * 0.2,
                driftY: (Math.random() - 0.5) * 0.15,
            });
        }
    }

    // ═══════════════════════════════════════════
    // 4. STATIC GRID (subtle base texture)
    // ═══════════════════════════════════════════
    let gridChars = [];

    function initGrid() {
        const cols = Math.ceil(W / CELL_SIZE);
        const rows = Math.ceil(H / CELL_SIZE);
        gridChars = [];
        for (let r = 0; r < rows; r++) {
            gridChars[r] = [];
            for (let c = 0; c < cols; c++) {
                gridChars[r][c] = {
                    char: '·.·+·:·*'[Math.floor(Math.random() * 8)],
                    alpha: 0.02 + Math.random() * 0.05,
                    mutateTimer: Math.random() * 500
                };
            }
        }
    }

    // ═══════════════════════════════════════════
    // INIT
    // ═══════════════════════════════════════════
    function resize() {
        W = canvas.width = window.innerWidth;
        H = canvas.height = window.innerHeight;

        // Rain
        const numCols = Math.ceil(W / CELL_SIZE);
        rainColumns = [];
        // Not every column — sparse rain
        for (let c = 0; c < numCols; c++) {
            if (Math.random() < 0.3) {
                rainColumns.push(new RainDrop(c));
            }
        }

        // Floaters
        floaters = [];
        const numFloaters = Math.floor(W / 60);
        for (let i = 0; i < numFloaters; i++) {
            floaters.push(new Floater());
        }

        createColonies();
        initGrid();
    }

    // ═══════════════════════════════════════════
    // DRAW LOOP
    // ═══════════════════════════════════════════
    function draw() {
        ctx.clearRect(0, 0, W, H);
        time++;

        // 4. Base grid (very faint texture)
        ctx.font = `${CELL_SIZE}px 'Press Start 2P', monospace`;
        ctx.textBaseline = 'top';

        const cols = Math.ceil(W / CELL_SIZE);
        const rows = Math.ceil(H / CELL_SIZE);

        for (let r = 0; r < rows && r < gridChars.length; r++) {
            for (let c = 0; c < cols && c < gridChars[r].length; c++) {
                const cell = gridChars[r][c];
                const px = c * CELL_SIZE;
                const py = r * CELL_SIZE;

                // Colony glow
                let colonyBoost = 0;
                for (const col of colonies) {
                    const pulse = Math.sin(time * col.pulseSpeed + col.phase) * 0.35 + 1;
                    const d = Math.sqrt((px - col.cx) ** 2 + (py - col.cy) ** 2);
                    if (d < col.radius * pulse) {
                        colonyBoost = Math.max(colonyBoost, 1 - d / (col.radius * pulse));
                    }
                }

                // Mouse proximity
                const dx = mouse.x - px;
                const dy = mouse.y - py;
                const dist = Math.sqrt(dx * dx + dy * dy);
                let mouseBoost = 0;
                if (dist < MOUSE_RADIUS && mouse.active) {
                    mouseBoost = (MOUSE_RADIUS - dist) / MOUSE_RADIUS;
                }

                let alpha = cell.alpha + colonyBoost * 0.15 + mouseBoost * 0.3;

                // Color
                if (mouseBoost > 0.2) {
                    ctx.fillStyle = `rgba(255, ${Math.floor(60 * (1 - mouseBoost))}, ${Math.floor(80 * (1 - mouseBoost))}, ${alpha})`;
                    cell.char = CHARS_BIO[Math.floor(Math.random() * CHARS_BIO.length)];
                } else if (colonyBoost > 0.1) {
                    ctx.fillStyle = `rgba(255, 40, 70, ${alpha * 0.6})`;
                } else {
                    ctx.fillStyle = `rgba(100, 100, 100, ${alpha})`;
                }

                // Slow mutation
                cell.mutateTimer--;
                if (cell.mutateTimer <= 0) {
                    cell.char = '·.·+·:·*'[Math.floor(Math.random() * 8)];
                    cell.mutateTimer = 200 + Math.random() * 600;
                }

                ctx.fillText(cell.char, px, py);
            }
        }

        // 1. Rain streams
        for (const drop of rainColumns) {
            drop.update();
            drop.draw();
        }

        // 2. Horizontal floaters
        for (const f of floaters) {
            f.update();
            f.draw();
        }

        // 3. Update colonies (drift)
        for (const col of colonies) {
            col.cx += col.driftX;
            col.cy += col.driftY;
            if (col.cx < -col.radius) col.cx = W + col.radius;
            if (col.cx > W + col.radius) col.cx = -col.radius;
            if (col.cy < -col.radius) col.cy = H + col.radius;
            if (col.cy > H + col.radius) col.cy = -col.radius;
        }

        // Mouse glow
        if (mouse.active) {
            const g = ctx.createRadialGradient(mouse.x, mouse.y, 0, mouse.x, mouse.y, MOUSE_RADIUS);
            g.addColorStop(0, 'rgba(255, 0, 60, 0.07)');
            g.addColorStop(0.6, 'rgba(255, 0, 60, 0.02)');
            g.addColorStop(1, 'rgba(255, 0, 60, 0)');
            ctx.fillStyle = g;
            ctx.beginPath();
            ctx.arc(mouse.x, mouse.y, MOUSE_RADIUS, 0, Math.PI * 2);
            ctx.fill();
        }

        requestAnimationFrame(draw);
    }

    // ── Events ──
    window.addEventListener('resize', resize);

    window.addEventListener('mousemove', e => {
        mouse.x = e.clientX;
        mouse.y = e.clientY;
        mouse.active = true;
    });

    window.addEventListener('mouseleave', () => {
        mouse.active = false;
        mouse.x = -9999;
        mouse.y = -9999;
    });

    // Touch support
    window.addEventListener('touchmove', e => {
        const t = e.touches[0];
        mouse.x = t.clientX;
        mouse.y = t.clientY;
        mouse.active = true;
    }, { passive: true });

    window.addEventListener('touchend', () => {
        mouse.active = false;
        mouse.x = -9999;
        mouse.y = -9999;
    });

    // ── Start ──
    resize();
    draw();

    // ── Section Nav ──
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
})();
