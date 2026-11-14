// PecoFence manual: the demo engine and player. Every demo is a pure function of time
// (render(t)), so the player can play, loop, scrub and jump to a step, and ?seek=<s> freezes
// a frame for review. Demos live in demos/*.js and register themselves with PM.register.
(() => {
    'use strict';

    const SW = 800, SH = 450;

    // In-stage UI text goes through T(): the zh-CN strings are the app's own catalog keys
    // (locales/<lang>.json), so other languages can reuse the app's translations.
    const T = s => s;

    // ---------- easing and timing ----------

    function bezier(x1, y1, x2, y2) {
        const cx = 3 * x1, bx = 3 * (x2 - x1) - cx, ax = 1 - cx - bx;
        const cy = 3 * y1, by = 3 * (y2 - y1) - cy, ay = 1 - cy - by;
        const fx = s => ((ax * s + bx) * s + cx) * s;
        const fy = s => ((ay * s + by) * s + cy) * s;
        const dx = s => (3 * ax * s + 2 * bx) * s + cx;
        return x => {
            if (x <= 0) return 0;
            if (x >= 1) return 1;
            let s = x;
            for (let i = 0; i < 8; i++) {
                const d = dx(s);
                if (Math.abs(d) < 1e-6) break;
                s -= (fx(s) - x) / d;
            }
            return fy(Math.min(1, Math.max(0, s)));
        };
    }

    const E = {
        lin: x => x,
        hand: bezier(.42, 0, .2, 1),   // pointer travel
        out: bezier(0, 0, 0, 1),       // WinUI decelerate
        p2p: bezier(.55, .55, 0, 1),   // WinUI point to point (layout slides, roll)
    };
    const clamp01 = x => (x < 0 ? 0 : x > 1 ? 1 : x);
    const ramp = (t, a, b, e = E.out) => e(clamp01((t - a) / (b - a)));
    const lerp = (a, b, p) => a + (b - a) * p;
    const inside = (p, r) => p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;

    // keys: [[time, x, y], ...]; holds where two neighbours share a position.
    function pathAt(t, keys, e = E.hand) {
        if (t <= keys[0][0]) return { x: keys[0][1], y: keys[0][2] };
        for (let i = 1; i < keys.length; i++) {
            const b = keys[i];
            if (t < b[0]) {
                const a = keys[i - 1];
                const p = e((t - a[0]) / (b[0] - a[0]));
                return { x: lerp(a[1], b[1], p), y: lerp(a[2], b[2], p) };
            }
        }
        const z = keys[keys.length - 1];
        return { x: z[1], y: z[2] };
    }

    // First moment in [a, b] where test(t) holds (for hover / drop-target fades).
    function firstWhen(a, b, test) {
        for (let t = a; t <= b; t += 1 / 120) if (test(t)) return t;
        return Infinity;
    }

    // [[enter, leave], ...] of the times test(t) holds, sampled over [0, d].
    function spans(d, test) {
        const out = [];
        let open = null;
        for (let t = 0; t <= d + 1e-9; t += 1 / 120) {
            const on = test(t);
            if (on && open == null) open = t;
            if (!on && open != null) {
                out.push([open, t]);
                open = null;
            }
        }
        if (open != null) out.push([open, Infinity]);
        return out;
    }

    // Opacity of something that fades in over `fi` when a span starts and out over `fo`.
    function spanFade(t, list, fi = .083, fo = .083) {
        let v = 0;
        for (const [a, b] of list) {
            if (t < a) break;
            v = t < b ? ramp(t, a, a + fi, E.lin) : Math.max(0, 1 - (t - b) / fo) * ramp(b, a, a + fi, E.lin);
        }
        return v;
    }

    // ---------- DOM builders ----------

    function el(tag, cls, parent, text) {
        const n = document.createElement(tag);
        if (cls) n.className = cls;
        if (text != null) n.textContent = text;
        if (parent) parent.appendChild(n);
        return n;
    }

    function put(n, x, y, opacity = 1, extra = '') {
        n.style.transform = `translate(${x.toFixed(2)}px, ${y.toFixed(2)}px)${extra}`;
        n.style.opacity = opacity;
    }

    function size(n, w, h) {
        n.style.width = w.toFixed(2) + 'px';
        n.style.height = h.toFixed(2) + 'px';
    }

    function show(n, on) {
        n.style.display = on ? '' : 'none';
        return on;
    }

    function text(n, s) {
        if (n.textContent !== s) n.textContent = s;
    }

    const CELL_W = 78, CELL_H = 80, PAD = 6, TITLE_H = 32;
    const cols = w => Math.max(1, Math.floor((w - 2 * PAD) / CELL_W));
    // Stage coordinates of item i in fence f; local = inside the fence's body.
    const slot = (f, i, c) => ({
        x: f.x + PAD + (i % c) * CELL_W,
        y: f.y + TITLE_H + 4 + Math.floor(i / c) * CELL_H,
    });
    const local = (i, c) => ({ x: PAD + (i % c) * CELL_W, y: 4 + Math.floor(i / c) * CELL_H });
    // Stage coordinates of a loose desktop icon's cell: the 「桌面」 fence of a new installation is
    // arranged like the desktop (no title, no border), its cells counted from the top-left
    // corner of the screen, column by column. Such icons take the class `desk` (white names).
    const deskCell = (col, row) => ({ x: 4 + col * CELL_W, y: 4 + row * CELL_H });

    function icon(parent, kind, label) {
        const n = el('div', 'ico', parent);
        const img = el('img', null, n);
        img.src = kind.includes('/') ? kind : `assets/icons/${kind}.png`;
        img.alt = '';
        img.draggable = false;
        el('span', null, n, label);
        return n;
    }

    const CHEVRON = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 10 8 5.5l4.5 4.5" fill="none" ' +
        'stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/></svg>';

    // A fence window: glass plate, drop highlight, title row (name / rename box / rolled
    // count / chevron), empty hint and a clipping body for icons in fence-local coordinates.
    function fence(parent, f) {
        const n = el('div', 'fence', parent);
        const drop = el('div', 'f-drop', n);
        const body = el('div', 'f-body', n);
        const hint = el('div', 'f-hint', n);
        const bar = el('div', 'f-title', n);
        const pill = el('div', 'f-pill', bar);
        const name = el('span', 'f-name', bar, T(f.title));
        const edit = el('div', 'f-edit', bar);
        const count = el('span', 'f-count', bar);
        const chev = el('span', 'f-chev', bar);
        chev.innerHTML = CHEVRON;
        edit.style.display = 'none';
        pill.style.opacity = 0;
        count.style.opacity = 0;
        chev.style.opacity = 0;
        size(n, f.w, f.h);
        put(n, f.x, f.y);
        return Object.assign(f, { n, drop, body, hint, bar, pill, name, edit, count, chev });
    }

    function placeFence(f, x, y, w, h, opacity = 1, extra = '') {
        f.x = x;
        f.y = y;
        f.w = w;
        f.h = h;
        size(f.n, w, h);
        put(f.n, x, y, opacity, extra);
    }

    // items: 'label' | '-' | {t, sub, key, dis, check}; a menu with a ticked item keeps a
    // column for the tick before every label, as Windows does
    function menu(parent, items, width) {
        const n = el('div', 'menu', parent);
        n.style.minWidth = width + 'px';   // grows when a translation is longer
        const rows = [];
        let y = 4;
        for (const it of items) {
            if (it === '-') {
                el('div', 'menu-sep', n);
                y += 9;
                continue;
            }
            const o = typeof it === 'string' ? { t: it } : it;
            const r = el('div', o.dis ? 'menu-item dis' : 'menu-item', n);
            if (o.check) {
                r.classList.add('check');
                n.classList.add('checks');
            }
            el('span', null, r, T(o.t));
            if (o.key) el('span', 'menu-key', r, T(o.key));
            if (o.sub) el('span', 'menu-sub', r);
            rows.push({ r, t: o.t, y0: y, y1: y + 26, dis: !!o.dis });
            y += 26;
        }
        // w = the drawn width (measured once laid out), for hover hit-testing
        return { n, rows, h: y + 4, get w() { return n.offsetWidth || width; } };
    }

    // Shown over [a, b]; `up` menus (tray) grow from below like Windows' taskbar menus.
    function drawMenu(m, t, x, y, a, b, p, up = false) {
        if (!show(m.n, t >= a && t < b + .1)) return;
        x = Math.max(4, Math.min(x, SW - 4 - m.w));
        const pin = ramp(t, a, a + .16), pout = ramp(t, b, b + .08, E.lin);
        m.n.style.opacity = pin * (1 - pout);
        m.n.style.transform = `translate(${x}px, ${y + (up ? 6 : -6) * (1 - pin)}px)`;
        for (const row of m.rows) {
            const hot = !row.dis && t < b && p.x >= x + 4 && p.x <= x + m.w - 4 && p.y >= y + row.y0 && p.y < y + row.y1;
            row.r.classList.toggle('hot', hot);
        }
    }

    // Item y (centre) of the row labelled `label`, for aiming the pointer.
    const rowY = (m, label) => {
        const r = m.rows.find(o => o.t === label);
        return (r.y0 + r.y1) / 2;
    };

    const ARROW = '<svg class="k-arrow" viewBox="0 0 16 24" width="16" height="24" aria-hidden="true">' +
        '<path d="M1.5 1.5v18.4l4.5-4.3 3.1 7 2.9-1.2-3.1-6.9h6.3z" fill="#fff" stroke="#151515" ' +
        'stroke-width="1.15" stroke-linejoin="round"/></svg>' +
        '<svg class="k-size" viewBox="-12 -12 24 24" width="24" height="24" aria-hidden="true">' +
        '<path d="M0-10.5 5.2-5.3H2v10.6h3.2L0 10.5-5.2 5.3H-2V-5.3h-3.2z" fill="#fff" stroke="#151515" ' +
        'stroke-width="1.1" stroke-linejoin="round"/></svg>';
    const SIZE_ANGLE = { ns: 0, we: 90, nwse: -45, nesw: 45 };

    function pointer(parent) {
        const n = el('div', 'cursor', parent);
        n.innerHTML = ARROW;
        return {
            rings: [el('div', 'ring', parent), el('div', 'ring', parent)],
            hold: el('div', 'hold', parent),
            chip: el('div', 'chip', parent),
            n,
            arrow: n.querySelector('.k-arrow'),
            sizer: n.querySelector('.k-size'),
        };
    }

    // clicks: [{t, right}], presses: [[a, b]], chips: [[a, b, text, dx?, dy?]] (offset from the
    // pointer), kinds: [[a, b, 'ns' | 'we' | 'nwse' | 'nesw']] (resize cursors, else the arrow),
    // at: t => {x, y} (the pointer path) so a click ring stays where the click happened.
    function drawPointer(c, t, p, { clicks = [], presses = [], chips = [], kinds = [], at = null }) {
        const pressed = presses.some(([a, b]) => t >= a && t < b);
        const kind = kinds.find(([a, b]) => t >= a && t < b);
        show(c.arrow, !kind);
        show(c.sizer, !!kind);
        if (kind) {
            c.sizer.style.transform = `rotate(${SIZE_ANGLE[kind[2]]}deg)`;
            c.n.style.transform = `translate(${(p.x - 12).toFixed(2)}px, ${(p.y - 12).toFixed(2)}px)`;
        } else {
            c.n.style.transform = `translate(${(p.x - 1.5).toFixed(2)}px, ${(p.y - 1.5).toFixed(2)}px) scale(${pressed ? .9 : 1})`;
        }

        const recent = clicks.filter(k => t >= k.t && t < k.t + .5).slice(-2);
        c.rings.forEach((r, i) => {
            const k = recent[i];
            if (!show(r, !!k)) return;
            const q = (t - k.t) / .5;
            r.classList.toggle('right', !!k.right);
            const w = at ? at(k.t) : p;
            put(r, w.x, w.y, (1 - q) * .9, ` scale(${.25 + E.out(q) * .9})`);
        });

        const hold = presses.find(([a, b]) => t >= a - .1 && t < b + .15);
        if (show(c.hold, !!hold)) {
            const o = Math.min(ramp(t, hold[0], hold[0] + .12, E.lin), 1 - ramp(t, hold[1], hold[1] + .15, E.lin));
            put(c.hold, p.x, p.y, o, ` scale(${.8 + .2 * o})`);
        }

        drawChip(c.chip, t, chips, p);
    }

    // Gesture captions. Attached to `at` (the pointer) unless an entry carries absolute
    // coordinates as [a, b, text, x, y, 'abs'] (add 'right' to right-align the caption at x).
    function drawChip(n, t, chips, at) {
        const chip = chips.find(([a, b]) => t >= a && t < b + .2);
        if (!show(n, !!chip)) return;
        text(n, chip[2]);
        const o = Math.min(ramp(t, chip[0], chip[0] + .15), 1 - ramp(t, chip[1], chip[1] + .2, E.lin));
        const abs = chip[5] === 'abs';
        const [dx = 16, dy = 22] = chip.slice(3, 5);
        const left = (abs ? 0 : at.x) + dx - (chip[6] === 'right' ? n.offsetWidth : 0);
        const x = Math.min(Math.max(4, left), SW - 4 - n.offsetWidth);
        put(n, x, (abs ? 0 : at.y) + dy + 4 * (1 - o), o);
    }

    // keys overlay: [{a, b, keys: [..], down: t, up?: t}] — `up` keeps keys held down.
    function drawKeys(n, t, list) {
        const k = list.find(e => t >= e.a && t < e.b + .2);
        if (!show(n, !!k)) return;
        const id = k.keys.join('+');
        if (n.dataset.id !== id) {
            n.dataset.id = id;
            n.replaceChildren(...k.keys.map(name => el('kbd', null, null, name)));
        }
        const down = t >= k.down && t < (k.up ?? k.down + .22);
        for (const kb of n.children) kb.classList.toggle('down', down);
        n.style.opacity = Math.min(ramp(t, k.a, k.a + .18), 1 - ramp(t, k.b, k.b + .2, E.lin));
    }

    // PecoFence's notice card above the right end of the taskbar (notice.rs): a title, a line of
    // text, a close box and one accent button, such as 「已删除栅栏“工作”」 with 「撤销」.
    function notice(parent, { title, text: body, button }) {
        const n = el('div', 'notice', parent);
        el('b', null, n, title);
        el('i', null, n);
        el('p', null, n, body);
        const btn = el('span', 'notice-btn', n, button);
        n.style.display = 'none';
        return { n, btn };
    }

    // Where the card sits (12 px in from the stage's right edge and above the taskbar) and the
    // centre of its button, for aiming the pointer.
    function noticeBox(c) {
        const shown = c.n.style.display;
        c.n.style.display = '';
        const w = c.n.offsetWidth, h = c.n.offsetHeight;
        const x = SW - 12 - w, y = SH - 28 - 10 - h;
        const btn = { x: x + c.btn.offsetLeft + c.btn.offsetWidth / 2, y: y + c.btn.offsetTop + c.btn.offsetHeight / 2 };
        c.n.style.display = shown;
        return { x, y, w, h, btn };
    }

    // Shown over [a, b]: it slides up 8 px while fading in (167 ms) and fades out (83 ms).
    function drawNotice(c, t, a, b, p) {
        if (!show(c.n, t >= a && t < b + .1)) return;
        const box = noticeBox(c);
        const o = ramp(t, a, a + .167) * (1 - ramp(t, b, b + .083, E.lin));
        put(c.n, box.x, box.y + 8 * (1 - ramp(t, a, a + .167)), o);
        c.btn.classList.toggle('hot', t < b && Math.abs(p.x - box.btn.x) <= c.btn.offsetWidth / 2
            && Math.abs(p.y - box.btn.y) <= c.btn.offsetHeight / 2);
    }

    // Alignment guide (drag_guides.rs): 1 DIP white core between dark hairlines, running
    // 12 DIP past the edges it joins.
    function guide(parent) {
        return el('div', 'guide', parent);
    }

    function drawGuide(g, opacity, vertical, at, from, to) {
        if (!show(g, opacity > 0)) return;
        g.classList.toggle('v', vertical);
        if (vertical) {
            size(g, 3, to - from + 24);
            put(g, at - 1.5, from - 12, opacity);
        } else {
            size(g, to - from + 24, 3);
            put(g, from - 12, at - 1.5, opacity);
        }
    }

    function taskbar(parent, { tray = true } = {}) {
        const n = el('div', 'taskbar', parent);
        el('div', 'tb-start', n);
        el('div', 'tb-search', n);
        [['folder', 452], ['url', 476]].forEach(([k, x]) => {
            const img = el('img', 'tb-app', n);
            img.src = `assets/icons/${k}.png`;
            img.alt = '';
            img.style.left = x + 'px';
        });
        const mark = el('img', 'tb-mark', n);
        mark.src = 'assets/mark.svg';
        mark.alt = '';
        if (!tray) mark.style.opacity = 0;
        el('div', 'tb-time', n, '10:24');
        return { n, mark };
    }
    // Stage coordinates of the PecoFence tray icon's centre.
    const TRAY = { x: SW - 62 - 7, y: SH - 14 };

    // ---------- registry and player ----------

    const DEMOS = {};
    const register = (name, demo) => {
        DEMOS[name] = demo;
    };

    // Whole-scene fade at the loop seam.
    const loopFade = (t, d, looped) => Math.min(looped ? ramp(t, 0, .3, E.lin) : 1, 1 - ramp(t, d - .35, d, E.lin));

    const params = new URLSearchParams(location.search);
    const frozen = params.has('seek');
    const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
    const players = [];

    class Player {
        constructor(root, demo) {
            this.demo = demo;
            this.D = demo.duration;
            this.t = frozen ? +params.get('seek') : reduced ? demo.poster : 0;
            this.speed = 1;
            this.playing = false;
            this.userPaused = frozen || reduced;
            this.looped = false;
            this.visible = false;

            this.stage = root.querySelector('.stage');
            this.inner = el('div', 'stage-inner', this.stage);
            this.refs = demo.build(this.inner);

            this.steps = [...root.querySelectorAll('.steps li')].map(li => ({ li, at: +li.dataset.at }));
            this.btn = root.querySelector('[data-play]');
            this.track = root.querySelector('[data-track]');
            this.fill = this.track.querySelector('.track-fill');
            this.ticks = this.steps.map(s => {
                const k = el('span', 'tick', this.track);
                k.style.left = (s.at / this.D * 100) + '%';
                return k;
            });

            new ResizeObserver(() => {
                this.inner.style.transform = `scale(${this.stage.clientWidth / SW})`;
            }).observe(this.stage);

            this.btn.addEventListener('click', () => this.toggle());
            root.querySelector('[data-speed]').addEventListener('click', e => {
                this.speed = this.speed === 1 ? .5 : 1;
                e.currentTarget.setAttribute('aria-pressed', String(this.speed !== 1));
            });
            this.steps.forEach(s => {
                s.li.tabIndex = 0;
                const go = () => {
                    this.seek(s.at);
                    this.userPaused = false;
                    this.play();
                };
                s.li.addEventListener('click', go);
                s.li.addEventListener('keydown', e => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        go();
                    }
                });
            });
            this.bindTrack();
            this.render();
        }

        bindTrack() {
            const at = e => {
                const b = this.track.getBoundingClientRect();
                return clamp01((e.clientX - b.left) / b.width) * this.D;
            };
            let resume = false;
            this.track.addEventListener('pointerdown', e => {
                this.track.setPointerCapture(e.pointerId);
                resume = this.playing;
                this.pause();
                this.seek(at(e));
            });
            this.track.addEventListener('pointermove', e => {
                if (this.track.hasPointerCapture(e.pointerId)) this.seek(at(e));
            });
            this.track.addEventListener('pointerup', () => {
                if (resume) this.play();
            });
            this.track.addEventListener('keydown', e => {
                const step = { ArrowLeft: -.5, ArrowRight: .5 }[e.key];
                if (step == null) return;
                e.preventDefault();
                this.seek(Math.min(this.D, Math.max(0, this.t + step)));
            });
        }

        play() {
            this.playing = true;
            this.btn.classList.remove('paused');
            this.btn.setAttribute('aria-label', '暂停');
        }

        pause() {
            this.playing = false;
            this.btn.classList.add('paused');
            this.btn.setAttribute('aria-label', '播放');
        }

        toggle() {
            if (this.playing) {
                this.userPaused = true;
                this.pause();
            } else {
                this.userPaused = false;
                if (this.t >= this.D - .01) this.seek(0);
                this.play();
            }
        }

        seek(t) {
            this.t = t;
            this.looped = false;
            this.render();
        }

        tick(dt) {
            if (!this.playing) return;
            this.t += dt * this.speed;
            if (this.t >= this.D) {
                this.t = 0;
                this.looped = true;
            }
            this.render();
        }

        render() {
            this.demo.render(this.t, this.refs, { looped: this.looped });
            let k = 0;
            this.steps.forEach((s, i) => {
                if (this.t >= s.at) k = i;
            });
            this.steps.forEach((s, i) => {
                s.li.classList.toggle('on', i === k);
                s.li.classList.toggle('done', i < k);
                this.ticks[i].classList.toggle('done', i <= k);
            });
            const end = k + 1 < this.steps.length ? this.steps[k + 1].at : this.D;
            this.steps[k].li.style.setProperty('--p', clamp01((this.t - this.steps[k].at) / (end - this.steps[k].at)).toFixed(3));
            this.fill.style.transform = `scaleX(${(this.t / this.D).toFixed(4)})`;
            this.track.setAttribute('aria-valuenow', String(Math.round(this.t / this.D * 100)));
        }
    }

    function boot() {
        const only = params.get('only');
        if (only) document.body.classList.add('only');
        if (params.has('bare')) document.body.classList.add('bare');
        if (params.has('embed')) document.body.classList.add('bare', 'embed');
        if (only) document.querySelectorAll('.lesson').forEach(s => s.id !== only && s.remove());
        document.querySelectorAll('[data-lesson]').forEach(root => {
            if (only && root.dataset.lesson !== only) {
                root.remove();
                return;
            }
            const demo = DEMOS[root.dataset.lesson];
            if (!demo) return;
            let p;
            try {
                p = new Player(root, demo);
            } catch (err) {
                console.error(`demo ${root.dataset.lesson} failed`, err);
                return;
            }
            if (p.userPaused) p.pause();
            players.push(p);
            new IntersectionObserver(([e]) => {
                p.visible = e.isIntersecting;
                if (p.visible && !p.userPaused) p.play();
                else if (!p.visible) p.pause();
            }, { threshold: .4 }).observe(p.stage);
        });

        document.addEventListener('visibilitychange', () => {
            if (document.hidden) players.forEach(p => p.pause());
            else players.forEach(p => p.visible && !p.userPaused && p.play());
        });

        let last = performance.now();
        requestAnimationFrame(function frame(now) {
            const dt = Math.min(.05, (now - last) / 1000);
            last = now;
            for (const p of players) if (p.visible) p.tick(dt);
            requestAnimationFrame(frame);
        });

        // Table of contents: the current lesson is the last one whose top is above 40 % of the
        // viewport; a sidebar on wide screens, a collapsed <details> above the lessons otherwise.
        const links = new Map([...document.querySelectorAll('.toc a[href^="#"]')].map(a => [a.hash.slice(1), a]));
        const lessons = [...document.querySelectorAll('.lesson[id]')];
        let queued = false;
        const mark = () => {
            queued = false;
            let cur = lessons[0];
            for (const s of lessons) if (s.getBoundingClientRect().top <= innerHeight * .4) cur = s;
            links.forEach((a, id) => a.classList.toggle('here', !!cur && id === cur.id));
        };
        addEventListener('scroll', () => {
            if (!queued) {
                queued = true;
                requestAnimationFrame(mark);
            }
        }, { passive: true });
        addEventListener('resize', mark);
        mark();

        const box = document.querySelector('.toc-box');
        const narrow = matchMedia('(max-width: 1360px)');
        const fold = () => {
            if (box) box.open = !narrow.matches;
        };
        narrow.addEventListener('change', fold);
        fold();
        box?.addEventListener('click', e => {
            if (narrow.matches && e.target.closest('a')) box.open = false;
        });
    }

    window.PM = {
        SW, SH, T, E, bezier, clamp01, ramp, lerp, inside, pathAt, firstWhen, spans, spanFade,
        el, put, size, show, text, CELL_W, CELL_H, PAD, TITLE_H, cols, slot, local, deskCell, icon, fence,
        placeFence, menu, drawMenu, rowY, pointer, drawPointer, drawChip, drawKeys, notice, noticeBox, drawNotice,
        guide, drawGuide, taskbar, TRAY, loopFade, register,
    };
    document.addEventListener('DOMContentLoaded', boot);
})();
