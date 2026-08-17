/* =========================================================
   Oracle — 2D ASCII Waddington Landscape + Langevin SDE
   No Three.js. Pure Canvas + ASCII characters.
   The landscape is rendered as a grid of ASCII glyphs
   whose character and color represent potential depth.
   A cell particle moves via Langevin dynamics (SDE).
   ========================================================= */

(function () {
    const canvas = document.getElementById('ascii-lab');
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    const FONT_SIZE = 10;
    const FONT = `${FONT_SIZE}px 'Press Start 2P', monospace`;

    // Rich depth-to-character mapping (deep → shallow), 5 zones
    const CHARS_DEEP   = '█▓▓▓▒▒▒';
    const CHARS_MID    = '░░╬╫╪┼┤├';
    const CHARS_SHALLOW = '╌╍┄┈─│┊┆';
    const CHARS_FLAT   = '·:;,\'`˙';
    const CHARS_RIDGE  = ' . ';

    // Bio glyphs for mouse hover (by proximity)
    const HOVER_NEAR  = '◉●◎⊕⊗⦿⟐⏣';
    const HOVER_MID   = '○◌◍◈⬡⬢△▷';
    const HOVER_FAR   = '∘∙·˚°';

    // Well-specific decorative chars
    const M1_CHARS = '◉●◎⊕※✦♦◆';
    const M2_CHARS = '◇◈⬡⬢□■▪▫';
    const M3_CHARS = '△▽◁▷◬◭◮⟨⟩';

    // ── Parameters ──
    let lps = 0.0;
    let il4 = 0.0;
    let noiseSigma = 0.5;

    // ── Cell State ──
    let cellPos = { x: 0, y: 1.5 };
    const dt = 0.016;

    // ── Trail ──
    const trail = [];
    const TRAIL_MAX = 150;
    const TRAIL_CHARS = '◉◎●○◌∘·';

    // ── Mouse ──
    let labMouse = { x: -1, y: -1, active: false };

    // ── Waddington Potential ──
    function V(x, y) {
        const d1 = 1.0 + lps * 0.8;
        const d2 = 1.0 + il4 * 0.8;
        const d3 = 0.8;
        const w = 1.5;

        const w1 = d1 * Math.exp(-(Math.pow(x + 1.5, 2) + Math.pow(y, 2)) / w);
        const w2 = d2 * Math.exp(-(Math.pow(x - 1.5, 2) + Math.pow(y, 2)) / w);
        const w3 = d3 * Math.exp(-(Math.pow(x, 2) + Math.pow(y - 1.5, 2)) / w);

        return -(w1 + w2 + w3) * 1.5 + 0.05 * (x * x + y * y);
    }

    function gradV(x, y) {
        const h = 0.01;
        return {
            dx: (V(x + h, y) - V(x - h, y)) / (2 * h),
            dy: (V(x, y + h) - V(x, y - h)) / (2 * h)
        };
    }

    // ── Gaussian Random ──
    function randn() {
        let u = 0, v = 0;
        while (u === 0) u = Math.random();
        while (v === 0) v = Math.random();
        return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
    }

    // ── Resize ──
    function resize() {
        const rect = canvas.parentElement.getBoundingClientRect();
        const sidebarW = 300;
        canvas.width = Math.max(200, rect.width - sidebarW);
        canvas.height = rect.height || 520;
    }

    // ── Coordinate mapping ──
    const WORLD_MIN = -3.5;
    const WORLD_MAX = 3.5;

    function worldToCanvas(wx, wy) {
        return {
            cx: ((wx - WORLD_MIN) / (WORLD_MAX - WORLD_MIN)) * canvas.width,
            cy: ((WORLD_MAX - wy) / (WORLD_MAX - WORLD_MIN)) * canvas.height
        };
    }

    function canvasToWorld(cx, cy) {
        return {
            wx: (cx / canvas.width) * (WORLD_MAX - WORLD_MIN) + WORLD_MIN,
            wy: WORLD_MAX - (cy / canvas.height) * (WORLD_MAX - WORLD_MIN)
        };
    }

    // ── Which well is closest? ──
    function nearestWell(wx, wy) {
        const d1 = Math.sqrt((wx + 1.5) ** 2 + wy ** 2);
        const d2 = Math.sqrt((wx - 1.5) ** 2 + wy ** 2);
        const d3 = Math.sqrt(wx ** 2 + (wy - 1.5) ** 2);
        if (d1 < d2 && d1 < d3) return { well: 'M1', dist: d1 };
        if (d2 < d1 && d2 < d3) return { well: 'M2', dist: d2 };
        return { well: 'M3', dist: d3 };
    }

    // ── Draw ──
    let time = 0;

    function draw() {
        requestAnimationFrame(draw);
        time++;

        ctx.fillStyle = '#020202';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.font = FONT;
        ctx.textBaseline = 'top';

        const cols = Math.floor(canvas.width / FONT_SIZE);
        const rows = Math.floor(canvas.height / FONT_SIZE);

        // Find min/max V for normalization
        let vMin = Infinity, vMax = -Infinity;
        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                const { wx, wy } = canvasToWorld(c * FONT_SIZE + FONT_SIZE / 2, r * FONT_SIZE + FONT_SIZE / 2);
                const v = V(wx, wy);
                if (v < vMin) vMin = v;
                if (v > vMax) vMax = v;
            }
        }
        const vRange = vMax - vMin || 1;

        // Draw ASCII landscape
        for (let r = 0; r < rows; r++) {
            for (let c = 0; c < cols; c++) {
                const px = c * FONT_SIZE;
                const py = r * FONT_SIZE;
                const { wx, wy } = canvasToWorld(px + FONT_SIZE / 2, py + FONT_SIZE / 2);
                const v = V(wx, wy);

                const norm = (v - vMin) / vRange; // 0=deepest, 1=highest
                const depthInv = 1 - norm;

                // Determine nearest well for character flavor
                const { well, dist } = nearestWell(wx, wy);

                // Pick character based on depth zone + well type
                let ch;
                if (depthInv > 0.8) {
                    // Deep inside well — use well-specific bio chars
                    const wellChars = well === 'M1' ? M1_CHARS : well === 'M2' ? M2_CHARS : M3_CHARS;
                    const shimIdx = Math.floor(time * 0.03 + c * 0.5 + r * 0.3) % wellChars.length;
                    ch = wellChars[shimIdx];
                } else if (depthInv > 0.6) {
                    ch = CHARS_DEEP[Math.floor((depthInv - 0.6) / 0.2 * (CHARS_DEEP.length - 1))];
                } else if (depthInv > 0.4) {
                    ch = CHARS_MID[Math.floor((depthInv - 0.4) / 0.2 * (CHARS_MID.length - 1))];
                } else if (depthInv > 0.2) {
                    ch = CHARS_SHALLOW[Math.floor((depthInv - 0.2) / 0.2 * (CHARS_SHALLOW.length - 1))];
                } else if (depthInv > 0.08) {
                    ch = CHARS_FLAT[Math.floor(depthInv / 0.2 * (CHARS_FLAT.length - 1))];
                } else {
                    ch = CHARS_RIDGE[Math.floor(Math.random() * CHARS_RIDGE.length)];
                }

                // Mouse hover override
                let mouseProx = 0;
                if (labMouse.active) {
                    const mdx = labMouse.x - px;
                    const mdy = labMouse.y - py;
                    const mDist = Math.sqrt(mdx * mdx + mdy * mdy);
                    if (mDist < 100) {
                        mouseProx = 1 - mDist / 100;
                        if (mouseProx > 0.6) {
                            ch = HOVER_NEAR[Math.floor(Math.random() * HOVER_NEAR.length)];
                        } else if (mouseProx > 0.3) {
                            ch = HOVER_MID[Math.floor(Math.random() * HOVER_MID.length)];
                        } else {
                            ch = HOVER_FAR[Math.floor(Math.random() * HOVER_FAR.length)];
                        }
                    }
                }

                // Color: based on well + depth
                let r_c, g_c, b_c, alpha;

                if (mouseProx > 0) {
                    r_c = 255;
                    g_c = Math.floor(60 + 180 * (1 - mouseProx));
                    b_c = Math.floor(60 + 180 * (1 - mouseProx));
                    alpha = 0.3 + mouseProx * 0.7;
                } else if (depthInv > 0.6) {
                    // In or near a well
                    if (well === 'M1') {
                        r_c = 255; g_c = 20; b_c = 50;
                    } else if (well === 'M2') {
                        r_c = 220; g_c = 220; b_c = 230;
                    } else {
                        r_c = 100; g_c = 110; b_c = 120;
                    }
                    alpha = 0.15 + depthInv * 0.55;
                } else if (depthInv > 0.3) {
                    const t = (depthInv - 0.3) / 0.3;
                    r_c = Math.floor(60 + 120 * t);
                    g_c = Math.floor(20 + 30 * t);
                    b_c = Math.floor(30 + 20 * t);
                    alpha = 0.06 + t * 0.12;
                } else {
                    r_c = 50; g_c = 50; b_c = 50;
                    alpha = 0.03 + depthInv * 0.06;
                }

                // Subtle shimmer in deep areas
                if (depthInv > 0.5) {
                    const shimmer = Math.sin(time * 0.06 + c * 0.4 + r * 0.3) * 0.04;
                    alpha = Math.max(0.02, Math.min(1, alpha + shimmer));
                }

                ctx.fillStyle = `rgba(${r_c},${g_c},${b_c},${alpha})`;
                ctx.fillText(ch, px, py);
            }
        }

        // ── Well Labels with ASCII borders ──
        const m1Pos = worldToCanvas(-1.5, 0);
        const m2Pos = worldToCanvas(1.5, 0);
        const m3Pos = worldToCanvas(0, 1.5);

        function drawWellLabel(pos, name, color, sub) {
            ctx.font = `10px 'Press Start 2P', monospace`;
            const lx = pos.cx - 30;
            const ly = pos.cy + 18;

            // Background box
            ctx.fillStyle = 'rgba(0,0,0,0.7)';
            ctx.fillRect(lx - 4, ly - 2, 70, 28);

            // Border
            ctx.strokeStyle = color;
            ctx.lineWidth = 1;
            ctx.strokeRect(lx - 4, ly - 2, 70, 28);

            // Name
            ctx.fillStyle = color;
            ctx.fillText(name, lx + 2, ly + 2);

            // Sub-label
            ctx.font = `7px 'Press Start 2P', monospace`;
            ctx.fillStyle = 'rgba(255,255,255,0.4)';
            ctx.fillText(sub, lx + 2, ly + 16);
        }

        drawWellLabel(m1Pos, 'M1', 'rgba(255,0,60,0.9)', 'ATTACK');
        drawWellLabel(m2Pos, 'M2', 'rgba(255,255,255,0.9)', 'PROTECT');
        drawWellLabel(m3Pos, 'M3', 'rgba(100,110,120,0.9)', 'TRANSIT');

        // ── Trail with fading bio chars ──
        ctx.font = FONT;
        for (let i = 0; i < trail.length; i++) {
            const t = trail[i];
            const pos = worldToCanvas(t.x, t.y);
            const age = 1 - i / trail.length;
            const tci = Math.min(TRAIL_CHARS.length - 1, Math.floor((1 - age) * TRAIL_CHARS.length));
            ctx.fillStyle = `rgba(255,0,60,${age * 0.4})`;
            ctx.fillText(TRAIL_CHARS[tci], pos.cx, pos.cy);
        }

        // ── Langevin Step ──
        const grad = gradV(cellPos.x, cellPos.y);
        cellPos.x += -grad.dx * dt + noiseSigma * 0.5 * randn() * Math.sqrt(dt);
        cellPos.y += -grad.dy * dt + noiseSigma * 0.5 * randn() * Math.sqrt(dt);
        cellPos.x = Math.max(-3.2, Math.min(3.2, cellPos.x));
        cellPos.y = Math.max(-3.2, Math.min(3.2, cellPos.y));

        // Trail
        trail.unshift({ x: cellPos.x, y: cellPos.y });
        if (trail.length > TRAIL_MAX) trail.pop();

        // ── Draw Cell Particle ──
        const cellCanvas = worldToCanvas(cellPos.x, cellPos.y);

        // Glow
        const glow = ctx.createRadialGradient(cellCanvas.cx, cellCanvas.cy, 0, cellCanvas.cx, cellCanvas.cy, 30);
        let glowColor;
        let stateName;

        if (cellPos.x < -0.5) {
            glowColor = 'rgba(255,0,60,';
            stateName = 'M1 (INFLAMMATORY)';
        } else if (cellPos.x > 0.5) {
            glowColor = 'rgba(255,255,255,';
            stateName = 'M2 (SUPPRESSIVE)';
        } else {
            glowColor = 'rgba(100,100,100,';
            stateName = 'M3 (INTERMEDIATE)';
        }

        glow.addColorStop(0, glowColor + '0.25)');
        glow.addColorStop(1, glowColor + '0)');
        ctx.fillStyle = glow;
        ctx.fillRect(cellCanvas.cx - 30, cellCanvas.cy - 30, 60, 60);

        // Cell character
        ctx.font = `14px 'Press Start 2P', monospace`;
        ctx.fillStyle = glowColor + '1)';
        ctx.fillText('◉', cellCanvas.cx - 5, cellCanvas.cy - 5);

        // ── Update Readout ──
        const stateEl = document.getElementById('cell-state');
        const posEl = document.getElementById('cell-pos');
        if (stateEl) stateEl.textContent = stateName;
        if (posEl) posEl.textContent = `x: ${cellPos.x.toFixed(2)}  y: ${cellPos.y.toFixed(2)}`;
    }

    // ── UI Controls ──
    const sliderLps = document.getElementById('slider-lps');
    const sliderIl4 = document.getElementById('slider-il4');
    const sliderNoise = document.getElementById('slider-noise');
    const btnReset = document.getElementById('btn-reset');

    if (sliderLps) {
        sliderLps.addEventListener('input', e => {
            lps = parseFloat(e.target.value);
            document.getElementById('val-lps').textContent = lps.toFixed(1);
        });
    }
    if (sliderIl4) {
        sliderIl4.addEventListener('input', e => {
            il4 = parseFloat(e.target.value);
            document.getElementById('val-il4').textContent = il4.toFixed(1);
        });
    }
    if (sliderNoise) {
        sliderNoise.addEventListener('input', e => {
            noiseSigma = parseFloat(e.target.value);
            document.getElementById('val-noise').textContent = noiseSigma.toFixed(1);
        });
    }
    if (btnReset) {
        btnReset.addEventListener('click', () => {
            cellPos.x = 0;
            cellPos.y = 1.5;
            trail.length = 0;
        });
    }

    // ── Mouse on canvas ──
    canvas.addEventListener('mousemove', e => {
        const rect = canvas.getBoundingClientRect();
        labMouse.x = e.clientX - rect.left;
        labMouse.y = e.clientY - rect.top;
        labMouse.active = true;
    });
    canvas.addEventListener('mouseleave', () => {
        labMouse.active = false;
    });

    // Click to teleport cell
    canvas.addEventListener('click', e => {
        const rect = canvas.getBoundingClientRect();
        const cx = e.clientX - rect.left;
        const cy = e.clientY - rect.top;
        const { wx, wy } = canvasToWorld(cx, cy);
        cellPos.x = wx;
        cellPos.y = wy;
        trail.length = 0;
    });

    // ── Start ──
    window.addEventListener('resize', resize);
    resize();
    draw();
})();
