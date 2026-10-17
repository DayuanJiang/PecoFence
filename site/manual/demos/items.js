// 2.1 在栅栏之间移动图标: drag 合同.pdf into 工作 (drop wash + 「移动到 工作」), Ctrl+click two
// files and drag them together (the drag image is the selection as laid out, anchored at
// the press point: dnd.rs render_drag_image), then drag 合同.pdf onto the bare desktop: back to
// the inbox, 桌面 (MoveItemsToInbox).
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, spans, spanFade, el, put, show, slot, icon, fence,
        pointer, drawPointer, drawChip, drawKeys, taskbar, loopFade, T, CELL_W, CELL_H, PAD, TITLE_H } = PM;

    const D = 11.8;
    const COLS = 4;
    const FENCES = {
        A: { x: 28, y: 36, w: 324, h: 204, title: '文件与文档' },
        W: { x: 448, y: 36, w: 324, h: 204, title: '工作' },
        I: { x: 28, y: 268, w: 324, h: 118, title: '桌面' },
    };
    const FS = Object.keys(FENCES);
    // id: [icon, label]; ids are ASCII so translations never touch them; .url hides its extension like Explorer
    const ITEMS = {
        invoice: ['pdf', '发票.pdf'],
        contract: ['pdf', '合同.pdf'],
        todo: ['txt', '待办.txt'],
        quote: ['xlsx', '报价单.xlsx'],
        plan: ['docx', '方案.docx'],
        poster: ['assets/icons/icons-poster.png', '海报.png'],
        budget: ['xlsx', '预算.xlsx'],
        weekly: ['docx', '周报.docx'],
        home: ['url', '项目主页'],
        assets: ['folder', '素材'],
    };

    const CLICK1 = 3.72, CLICK2 = 4.5;
    // press, drop, items, source fence (drops happen at `drop`, the image lifts off at `lift`)
    const DRAGS = [
        { press: 1.15, lift: 1.22, drop: 2.6, items: ['contract'], from: 'A', to: 'W' },
        { press: 5.0, lift: 5.2, drop: 6.6, items: ['quote', 'plan'], from: 'A', to: 'W' },
        { press: 7.75, lift: 7.82, drop: 9.15, items: ['contract'], from: 'W', to: 'I' },
    ];
    const SELECTED = {
        contract: [[DRAGS[0].press, DRAGS[0].drop], [DRAGS[2].press, DRAGS[2].drop]],
        quote: [[CLICK1, DRAGS[1].drop]],
        plan: [[CLICK2, DRAGS[1].drop]],
    };
    const CLICKS = [{ t: CLICK1 }, { t: CLICK2 }];
    const PRESSES = DRAGS.map(d => [d.press, d.drop]);
    const CHIPS = [
        [1.15, 1.7, '按住拖动', -30, -50],
        [3.62, 3.98, '点一下', 16, 46],         // below the label row (row 2 is empty there)
        [4.4, 4.85, '按住 Ctrl 点', 16, 46],
        [5.05, 6.0, '一起拖走', 60, -12],
        [8.1, 8.95, '拖到桌面空白处', 48, -6],
    ];
    const KEYS = [{ a: 3.95, b: 4.8, keys: ['Ctrl'], down: 4.05, up: 4.75 }];
    // right of 桌面, where it landed
    const INBOX_NOTE = [DRAGS[2].drop + .1, DRAGS[2].drop + 1.5, '放回「桌面」',
        FENCES.I.x + FENCES.I.w + 14, FENCES.I.y + 44, 'abs'];

    // Manual order of items never rearranged by hand: lowercase name, code point order
    // (state.rs items_of, SortMode::Manual with no manual_index).
    const order = list => [...list].sort((a, b) => {
        const x = ITEMS[a][1].toLowerCase(), y = ITEMS[b][1].toLowerCase();
        return x < y ? -1 : x > y ? 1 : 0;
    });
    const PHASES = [{
        A: order(['invoice', 'contract', 'todo', 'quote', 'plan', 'poster', 'budget']),
        W: order(['weekly', 'home']),
        I: ['assets'],
    }];
    for (const d of DRAGS) {
        const p = PHASES[PHASES.length - 1];
        const next = Object.fromEntries(FS.map(f => [f, p[f].filter(k => !d.items.includes(k))]));
        next[d.to] = order([...next[d.to], ...d.items]);
        PHASES.push(next);
    }
    const where = (k, j) => {
        for (const f of FS) {
            const i = PHASES[j][f].indexOf(k);
            if (i >= 0) return { f, i, ...slot(FENCES[f], i, COLS) };
        }
        return null;
    };
    const phaseAt = t => DRAGS.filter(d => t >= d.drop).length;

    // The pointer aims at items where they actually sit, so it follows each language's name order.
    const aim = (k, j) => {
        const s = where(k, j);
        return [s.x + 36, s.y + 26];
    };
    const PATH = [
        [0, 600, 390], [0.3, 600, 390],
        [1.0, ...aim('contract', 0)], [1.15, ...aim('contract', 0)],   // press 合同.pdf
        [2.45, 598, 150], [2.6, 598, 150],                              // over 工作, release
        [3.1, 598, 150],
        [3.6, ...aim('quote', 1)], [3.85, ...aim('quote', 1)],         // click 报价单.xlsx
        [4.3, ...aim('plan', 1)], [5.15, ...aim('plan', 1)],           // Ctrl+click 方案.docx, then press it
        [6.45, 652, 190], [6.6, 652, 190],                              // both into 工作
        [7.0, 652, 190],
        [7.6, ...aim('contract', 2)], [7.75, ...aim('contract', 2)],   // press 合同.pdf in 工作
        [9.0, 562, 330], [9.15, 562, 330],                              // bare desktop, release
        [9.9, 650, 388], [D, 650, 388],
    ];
    const at = t => pathAt(t, PATH);

    // Insertion slot of a pointer over a 手动 fence (layout.rs insertion_index / insertion_caret).
    function caretAt(f, p, count) {
        const x = p.x - f.x, y = p.y - f.y - TITLE_H;
        const col = Math.min(COLS, Math.max(0, Math.round((x - PAD) / CELL_W)));
        const row = Math.max(0, Math.floor((y - 4) / CELL_H));
        const i = Math.min(count, row * COLS + col);
        if (i === count && i % COLS === 0 && count > 0) {
            const c = slot(f, count - 1, COLS);
            return { x: c.x + CELL_W - 1, y: c.y };
        }
        const c = slot(f, i, COLS);
        return { x: c.x - 1, y: c.y };
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.F = Object.fromEntries(FS.map(f => [f, fence(S, { ...FENCES[f] })]));
        r.carets = Object.fromEntries(FS.map(f => [f, el('div', 'icons-caret', S)]));
        r.icons = {};
        r.echo = {};
        for (const k of Object.keys(ITEMS)) {
            r.echo[k] = icon(S, ...ITEMS[k]);
            r.icons[k] = icon(S, ...ITEMS[k]);
        }
        r.pulse = el('div', 'ring pulse', S);
        r.note = el('div', 'chip', S);
        // one drag image per drag: the selection at press time, relative to its bounding box
        r.ghosts = DRAGS.map(d => {
            const n = el('div', 'icons-ghost', S);
            const j = DRAGS.indexOf(d);
            const cells = d.items.map(k => where(k, j));
            const ox = Math.min(...cells.map(c => c.x)), oy = Math.min(...cells.map(c => c.y));
            d.items.forEach((k, i) => put(icon(n, ...ITEMS[k]), cells[i].x - ox, cells[i].y - oy));
            const g = at(d.press);
            return { n, dx: g.x - ox, dy: g.y - oy };
        });
        r.badge = el('div', 'badge', S);
        r.badgeIcon = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M1.5 6h8M6.5 2.8 9.7 6 6.5 9.2" ' +
            'fill="none" stroke="#1663c7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);

        // hover spans of the dragging pointer over each fence, per drag
        const dragging = (d, t) => t >= d.lift && t < d.drop;
        r.over = DRAGS.map(d => Object.fromEntries(FS.map(f =>
            [f, spans(D, t => dragging(d, t) && inside(at(t), FENCES[f]))])));
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const j = phaseAt(t);

        // icons: slide 250 ms to new cells, or fade out (83 ms) / in (167 ms) across fences
        for (const k of Object.keys(ITEMS)) {
            const n = r.icons[k], echo = r.echo[k];
            const cur = where(k, j);
            const sel = (SELECTED[k] || []).some(([a, b]) => t >= a && t < b);
            n.classList.toggle('sel', sel);
            show(echo, false);
            if (j === 0) {
                put(n, cur.x, cur.y);
                continue;
            }
            const d = DRAGS[j - 1], prev = where(k, j - 1);
            if (prev.f !== cur.f) {
                put(n, cur.x, cur.y, ramp(t, d.drop + .02, d.drop + .19, E.lin));
                if (show(echo, t < d.drop + .083)) put(echo, prev.x, prev.y, 1 - ramp(t, d.drop, d.drop + .083, E.lin));
            } else {
                const q = ramp(t, d.drop + .05, d.drop + .3, E.p2p);
                put(n, lerp(prev.x, cur.x, q), lerp(prev.y, cur.y, q));
            }
        }

        // drop feedback: wash on another fence, insertion caret on the source (手动 order)
        let badge = null;
        for (const f of FS) {
            let wash = 0, caret = 0, cpos = null;
            DRAGS.forEach((d, i) => {
                const list = r.over[i][f];
                if (d.from === f) {
                    const v = spanFade(t, list);
                    if (v > caret) {
                        caret = v;
                        const span = list.filter(([a]) => t >= a).pop();
                        const q = t < span[1] ? p : at(span[1] - 1 / 120);
                        cpos = caretAt(FENCES[f], q, PHASES[i][f].length);
                    }
                } else {
                    wash = Math.max(wash, spanFade(t, list, .083, .167));
                }
                if (list.some(([a, b]) => t >= a && t < b)) badge = FENCES[f].title;
            });
            r.F[f].drop.style.opacity = wash;
            if (show(r.carets[f], caret > 0)) put(r.carets[f], cpos.x, cpos.y, caret);
        }

        DRAGS.forEach((d, i) => {
            const g = r.ghosts[i];
            if (show(g.n, t >= d.lift && t < d.drop)) put(g.n, p.x - g.dx, p.y - g.dy, ramp(t, d.lift, d.lift + .06, E.lin));
        });
        if (show(r.badge, badge != null)) {
            const html = r.badgeIcon + T('移动到 %1').replace('%1', `<b>${T(badge)}</b>`);
            if (r.badge.innerHTML !== html) r.badge.innerHTML = html;
            put(r.badge, p.x + 2, p.y + 34);
        }

        // back in 桌面: a pulse where it landed
        const back = DRAGS[2].drop, home = where('contract', 3);
        if (show(r.pulse, t >= back + .05 && t < back + .85)) {
            const q = (t - back - .05) / .8;
            put(r.pulse, home.x + 39, home.y + 24, (1 - q) * .9, ` scale(${.6 + E.out(q) * 1.1})`);
        }
        drawChip(r.note, t, [INBOX_NOTE], p);

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at });
    }

    PM.register('items', { duration: D, poster: 6.2, build, render });
})();
