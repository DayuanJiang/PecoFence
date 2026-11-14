// 3.1 在栅栏之间移动图标: drag 合同.pdf into 工作 and let go between two of its icons (the fence
// lights up, an insertion line shows where, 「移动到 工作」), Ctrl+click two files and drag them
// together onto empty space of 工作: they go after the others (the drag image is the selection
// as laid out, anchored at the press point: dnd.rs render_drag_image), then drag 合同.pdf onto
// the bare desktop: it stays in the cell where it was let go (「桌面」 of a new installation is
// 像桌面一样排列 with 自由摆放 on: loose icons, no title, no border).
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, spans, spanFade, el, put, show, slot, deskCell, icon, fence,
        pointer, drawPointer, drawChip, drawKeys, taskbar, loopFade, T, CELL_W, CELL_H, PAD, TITLE_H } = PM;

    const D = 11.8;
    const COLS = 4;
    const FENCES = {
        A: { x: 104, y: 36, w: 324, h: 204, title: '文件与文档' },
        W: { x: 452, y: 36, w: 324, h: 204, title: '工作' },
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
    // press, drop, items, source fence, target fence (null = the bare desktop); drops happen at
    // `drop`, the image lifts off at `lift`
    const DRAGS = [
        { press: 1.15, lift: 1.22, drop: 2.6, items: ['contract'], from: 'A', to: 'W' },
        { press: 5.0, lift: 5.2, drop: 6.6, items: ['quote', 'plan'], from: 'A', to: 'W' },
        { press: 7.75, lift: 7.82, drop: 9.15, items: ['contract'], from: 'W', to: null },
    ];
    // Where each drag lets go: between the two icons of 工作, on its empty space, on the desktop.
    const DROPS = [
        [FENCES.W.x + PAD + CELL_W + 4, FENCES.W.y + TITLE_H + 40],
        [FENCES.W.x + 200, FENCES.W.y + TITLE_H + 122],
        [436, 276],
    ];
    const GRIP = [36, 26];      // the pointer presses an icon here, counted from its cell's corner
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

    // Insertion index of a pointer over the item area of a 手动 fence (layout.rs insertion_index):
    // the nearest cell boundary in the row under the pointer, at most after the last icon.
    function insertAt(f, p, count) {
        const col = Math.min(COLS, Math.max(0, Math.round((p.x - f.x - PAD) / CELL_W)));
        const row = Math.max(0, Math.floor((p.y - f.y - TITLE_H - 4) / CELL_H));
        return Math.min(count, row * COLS + col);
    }
    // The insertion line for that index (layout.rs insertion_caret).
    function caretAt(f, p, count) {
        const i = insertAt(f, p, count);
        if (i === count && i % COLS === 0 && count > 0) {
            const c = slot(f, count - 1, COLS);
            return { x: c.x + CELL_W - 1, y: c.y };
        }
        const c = slot(f, i, COLS);
        return { x: c.x - 1, y: c.y };
    }

    // A fence never rearranged by hand shows its icons by name: lowercase, code point order
    // (state.rs sorted_members, SortMode::Manual with no manual_index). Icons let go in it go
    // in at the insertion line, by name among themselves, and the others keep the order they
    // are shown in (state.rs place_last, reorder_items). An icon let go on the desktop stays in
    // the cell under its picture.
    const order = list => [...list].sort((a, b) => {
        const x = ITEMS[a][1].toLowerCase(), y = ITEMS[b][1].toLowerCase();
        return x < y ? -1 : x > y ? 1 : 0;
    });
    const PHASES = [{
        A: order(['invoice', 'contract', 'todo', 'quote', 'plan', 'poster', 'budget']),
        W: order(['weekly', 'home']),
        desk: { assets: [0, 0] },
    }];
    DRAGS.forEach((d, n) => {
        const next = { desk: { ...PHASES[n].desk } };
        for (const f of FS) next[f] = PHASES[n][f].filter(k => !d.items.includes(k));
        const [x, y] = DROPS[n];
        if (d.to) {
            next[d.to].splice(insertAt(FENCES[d.to], { x, y }, next[d.to].length), 0, ...order(d.items));
        } else {
            next.desk[d.items[0]] = [Math.floor((x - GRIP[0] + 39 - 4) / CELL_W), Math.floor((y - GRIP[1] + 40 - 4) / CELL_H)];
        }
        PHASES.push(next);
    });
    // f: the fence an item is in (null = loose on the desktop), x / y: its cell on the stage
    const where = (k, j) => {
        const cell = PHASES[j].desk[k];
        if (cell) return { f: null, ...deskCell(...cell) };
        for (const f of FS) {
            const i = PHASES[j][f].indexOf(k);
            if (i >= 0) return { f, ...slot(FENCES[f], i, COLS) };
        }
        return null;
    };
    const phaseAt = t => DRAGS.filter(d => t >= d.drop).length;

    // The pointer aims at items where they actually sit, so it follows each language's name order.
    const aim = (k, j) => {
        const s = where(k, j);
        return [s.x + GRIP[0], s.y + GRIP[1]];
    };
    const PATH = [
        [0, 600, 390], [0.3, 600, 390],
        [1.0, ...aim('contract', 0)], [1.15, ...aim('contract', 0)],   // press 合同.pdf
        [2.45, ...DROPS[0]], [2.6, ...DROPS[0]],                        // between the icons of 工作, release
        [3.1, ...DROPS[0]],
        [3.6, ...aim('quote', 1)], [3.85, ...aim('quote', 1)],         // click 报价单.xlsx
        [4.3, ...aim('plan', 1)], [5.15, ...aim('plan', 1)],           // Ctrl+click 方案.docx, then press it
        [6.45, ...DROPS[1]], [6.6, ...DROPS[1]],                        // both onto empty space of 工作
        [7.0, ...DROPS[1]],
        [7.6, ...aim('contract', 2)], [7.75, ...aim('contract', 2)],   // press 合同.pdf in 工作
        [9.0, ...DROPS[2]], [9.15, ...DROPS[2]],                        // bare desktop, release
        [9.9, 650, 388], [D, 650, 388],
    ];
    const at = t => pathAt(t, PATH);
    // under the cell it stays in
    const STAY = where('contract', 3);
    const STAY_NOTE = [DRAGS[2].drop + .1, DRAGS[2].drop + 1.5, '停在松开的这一格', STAY.x - 8, STAY.y + 84, 'abs'];

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

        // hover spans of the dragging pointer over each fence, per drag (the drags cross the
        // item areas only, no title row)
        const dragging = (d, t) => t >= d.lift && t < d.drop;
        r.over = DRAGS.map(d => Object.fromEntries(FS.map(f =>
            [f, spans(D, t => dragging(d, t) && inside(at(t), FENCES[f]))])));
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const j = phaseAt(t);

        for (const k of Object.keys(ITEMS)) {
            const n = r.icons[k], echo = r.echo[k];
            const cur = where(k, j);
            const sel = (SELECTED[k] || []).some(([a, b]) => t >= a && t < b);
            n.classList.toggle('sel', sel);
            n.classList.toggle('desk', !cur.f);
            show(echo, false);
            if (j === 0) {
                put(n, cur.x, cur.y);
                continue;
            }
            const d = DRAGS[j - 1], prev = where(k, j - 1);
            if (prev.f === cur.f) {
                // stays where it is kept: slides 250 ms to its new cell
                const q = ramp(t, d.drop + .05, d.drop + .3, E.p2p);
                put(n, lerp(prev.x, cur.x, q), lerp(prev.y, cur.y, q));
                continue;
            }
            // dragged away: the icon in the old cell goes (83 ms)
            if (show(echo, t < d.drop + .083)) put(echo, prev.x, prev.y, 1 - ramp(t, d.drop, d.drop + .083, E.lin));
            if (cur.f) {
                // let go in a fence: it glides from where its drag picture was to its cell
                // (167 ms, no fade: items.rs replace_items, drop_landing)
                const a = at(d.drop), b = at(d.press), q = ramp(t, d.drop, d.drop + .167);
                put(n, lerp(prev.x + a.x - b.x, cur.x, q), lerp(prev.y + a.y - b.y, cur.y, q));
            } else {
                // let go on the bare desktop: it shows in the cell under it (167 ms)
                put(n, cur.x, cur.y, ramp(t, d.drop + .02, d.drop + .19, E.lin));
            }
        }

        // drop feedback: another fence lights up, and the insertion line shows where the icons
        // would go, in the source fence too (手动 order; dnd.rs set_drop_feedback). The badge
        // names another fence; over the source fence it is the arrow alone (a reorder).
        let badge = null;
        for (const f of FS) {
            let wash = 0, caret = 0, cpos = null;
            DRAGS.forEach((d, i) => {
                const list = r.over[i][f];
                const v = spanFade(t, list);
                if (v > caret) {
                    caret = v;
                    const span = list.filter(([a]) => t >= a).pop();
                    const q = t < span[1] ? p : at(span[1] - 1 / 120);
                    cpos = caretAt(FENCES[f], q, PHASES[i][f].length);
                }
                if (d.from !== f) wash = Math.max(wash, spanFade(t, list, .083, .167));
                if (list.some(([a, b]) => t >= a && t < b)) badge = d.from === f ? '' : FENCES[f].title;
            });
            r.F[f].drop.style.opacity = wash;
            if (show(r.carets[f], caret > 0)) put(r.carets[f], cpos.x, cpos.y, caret);
        }
        // over the bare desktop the badge names 「桌面」, whose loose icons stand there (seen on
        // the product: "Move to Desktop")
        if (badge == null && DRAGS.some(d => t >= d.lift && t < d.drop) && !FS.some(f => inside(p, FENCES[f]))) badge = '桌面';

        DRAGS.forEach((d, i) => {
            const g = r.ghosts[i];
            if (show(g.n, t >= d.lift && t < d.drop)) put(g.n, p.x - g.dx, p.y - g.dy, ramp(t, d.lift, d.lift + .06, E.lin));
        });
        if (show(r.badge, badge != null)) {
            const html = r.badgeIcon + (badge ? T('移动到 %1').replace('%1', `<b>${T(badge)}</b>`) : '');
            if (r.badge.innerHTML !== html) r.badge.innerHTML = html;
            r.badge.classList.toggle('icons-bare', !badge);
            put(r.badge, p.x + 2, p.y + 34);
        }

        // on the desktop: a pulse where it stays
        const back = DRAGS[2].drop;
        if (show(r.pulse, t >= back + .05 && t < back + .85)) {
            const q = (t - back - .05) / .8;
            put(r.pulse, STAY.x + 39, STAY.y + 24, (1 - q) * .9, ` scale(${.6 + E.out(q) * 1.1})`);
        }
        drawChip(r.note, t, [STAY_NOTE], p);

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at });
    }

    PM.register('items', { duration: D, poster: 6.2, build, render });
})();
