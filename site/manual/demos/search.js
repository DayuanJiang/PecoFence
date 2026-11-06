// 4.4 搜索图标和文件 (lesson id search): Ctrl tapped twice (double_ctrl.rs) opens the Peek with
// its search box (peek_search.rs) in the upper middle of the screen; typing lists the matches
// under the box. The fence match comes first and lights up in its fence while the other icons
// fade to 15 %; Everything's whole-computer results follow under their own heading a moment
// later (search.rs asks Everything after a 120 ms pause), with 「在 Everything 里查看全部结果」
// last. ↓ moves the selection, Enter opens the row and ends the Peek (the file's app comes up).
// Chinese pages type the pinyin initials (hyjl finds 会议记录); the others type the start of
// the translated word, Japanese and Korean its first two characters.
(() => {
    'use strict';
    const { E, ramp, lerp, el, put, show, text, size, icon, fence, pointer, drawPointer, taskbar, loopFade,
        local } = PM;

    const D = 7.8;
    const WORD = '会议记录';
    const NAME = '会议记录.docx';
    const FENCES = [
        { x: 16, y: 24, w: 168, h: 124, title: '程序', items: [['url', '项目主页'], ['exe', '截图工具']] },
        { x: 16, y: 160, w: 168, h: 204, title: '文件夹', items: [['folder', '工作资料'], ['folder', '照片'], ['folder', '合同']] },
        { x: 616, y: 24, w: 168, h: 284, title: '文件与文档', items: [['pdf', '合同.pdf'], ['docx', '周报.docx'],
            ['docx', NAME], ['xlsx', '预算.xlsx'], ['pdf', '发票.pdf'], ['txt', '待办.txt']] },
    ];
    const HIT = [2, 2];                                       // fence 文件与文档, item 会议记录.docx
    // Everything's rows: name, folder (the row's second line), icon
    const FOUND = [['xlsx', '会议记录 2025.xlsx', 'D:\\项目\\2025'], ['docx', '会议记录模板.docx', 'D:\\模板']];
    const TOTAL = 12;

    // The box (640 DIP in the app) scaled to the stage, rows 44 → 36, the heading 24 → 18.
    const BOX = { x: 196, y: 52, w: 408, h: 44 };
    const ROW_H = 36, HEAD_H = 18, GAP = 3, PAD_B = 4;

    const TAPS = [[.55, .68], [.82, .95]];                    // Ctrl down / up, twice
    const PEEK_ON = .97;
    const TYPE_AT = 1.45, TYPE_STEP = .12;
    const FENCE_ROWS = 1.85, EV_ROWS = 2.35;
    const DOWN = 3.95, ENTER = 4.95, OPEN = 5.12;
    const FADE = .15;
    const KEYS = [{ a: 3.7, b: 4.4, keys: ['↓'], down: DOWN }, { a: 4.7, b: 5.4, keys: ['Enter'], down: ENTER }];

    const MAG = '<svg viewBox="0 0 16 16" aria-hidden="true"><circle cx="6.6" cy="6.6" r="4.9" fill="none" ' +
        'stroke="currentColor" stroke-width="1.3"/><path d="m10.2 10.2 4.3 4.3" stroke="currentColor" stroke-width="1.3" ' +
        'stroke-linecap="round"/></svg>';
    const CAP = (d, extra = '') => `<i><svg viewBox="0 0 10 10" aria-hidden="true"><path d="${d}" fill="none" ` +
        `stroke="currentColor" stroke-width="1" ${extra}/></svg></i>`;
    const CAPS = '<div class="pk-caps">' + CAP('M.5 5h9') + CAP('M1 1h8v8H1z', 'stroke-linejoin="round"') +
        CAP('M.8.8l8.4 8.4M9.2.8 .8 9.2') + '</div>';

    // What is typed and which part of a name lights up.
    function query() {
        const lang = document.documentElement.lang || 'zh-CN';
        if (/^zh/.test(lang)) return 'hyjl';
        const word = WORD.trim();
        const n = /[\u3040-\u30ff\u3400-\u9fff\uac00-\ud7af]/.test(word) ? 2 : 4;
        return [...word].slice(0, n).join('').toLowerCase();
    }
    // [start, end) of the highlight in `name`: the whole word for pinyin, else the typed letters.
    function marks(name, q, pinyin) {
        const i = name.toLowerCase().indexOf(WORD.toLowerCase());
        if (i < 0) return [0, 0];
        return [i, i + (pinyin ? WORD.length : q.length)];
    }

    // Fluent glass = blurred crop of the wallpaper under the fence (peek.css), so the fences look
    // the same over the Peek's dimmer.
    function glass(f) {
        f.n.classList.add('pk-glass');
        const crop = el('div', 'pk-crop');
        const pic = el('div', null, crop);
        pic.style.transform = `translate(${-f.x}px, ${-f.y}px)`;
        f.n.prepend(crop);
    }

    function row(parent, { kind, name, detail, key }) {
        const n = el('div', 'sr-row', parent);
        const bg = el('div', 'sr-selbg', n);
        if (kind) {
            const img = el('img', null, n);
            img.src = `assets/icons/${kind}.png`;
            img.alt = '';
        }
        const nm = el('div', 'sr-name', n);
        const parts = [el('span', null, nm), el('b', null, nm), el('span', null, nm)];
        el('div', 'sr-detail', n, detail);
        el('div', 'sr-key', n, 'Ctrl+' + key);
        if (!kind) n.classList.add('sr-all');
        return { n, bg, name, parts };
    }

    function sheet(parent) {
        const n = el('div', 'pk-win sr-app', parent);
        const bar = el('div', 'pk-titlebar', n);
        const img = el('img', null, bar);
        img.src = 'assets/icons/xlsx.png';
        img.alt = '';
        el('span', null, bar, FOUND[0][1]);
        bar.insertAdjacentHTML('beforeend', CAPS);
        el('div', 'pk-ribbon', n).innerHTML = '<i></i><i></i><i></i><i class="s"></i><i class="w"></i><i></i><i></i>';
        el('div', 'sr-grid', n);
        put(n, 150, 50, 0);
        size(n, 500, 330);
        return n;
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.fences = FENCES.map(spec => {
            const f = fence(S, { ...spec });
            glass(f);
            const c = spec.w > 200 ? 3 : 2;
            f.icons = spec.items.map(([k, l], i) => {
                const n = icon(f.body, k, l);
                const s = local(i, c);
                put(n, s.x, s.y);
                return n;
            });
            return f;
        });
        r.dim = el('div', 'pk-dim', S);

        r.q = query();
        const pinyin = r.q === 'hyjl';
        r.box = el('div', 'sr-box', S);
        el('div', 'sr-glyph', r.box).innerHTML = MAG;
        r.typed = el('div', 'sr-q', r.box);
        r.caret = el('div', 'sr-caret', r.box);
        r.status = el('div', 'sr-status', r.box);
        r.line = el('div', 'sr-line', r.box);
        r.heads = [el('div', 'sr-head', r.box, '栅栏里'), el('div', 'sr-head', r.box, '整台电脑（Everything）')];
        r.rows = [
            row(r.box, { kind: 'docx', name: NAME, detail: FENCES[2].title, key: 1 }),
            ...FOUND.map(([kind, name, detail], i) => row(r.box, { kind, name, detail, key: i + 2 })),
            row(r.box, { kind: null, name: '在 Everything 里查看全部结果', detail: '共 {0} 个'.replace('{0}', TOTAL), key: 4 }),
        ];
        r.rows.forEach((o, i) => {
            const [a, b] = i < 3 ? marks(o.name, r.q, pinyin) : [0, 0];
            const chars = [...o.name];
            text(o.parts[0], chars.slice(0, a).join(''));
            text(o.parts[1], chars.slice(a, b).join(''));
            text(o.parts[2], chars.slice(b).join(''));
        });

        r.app = sheet(S);
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);
        return r;
    }

    // Two Ctrl keycaps, each pressed once (drawKeys presses all keys of an entry together).
    function drawTaps(n, t) {
        const on = t >= .35 && t < 1.45;
        if (!show(n, on)) return;
        if (n.dataset.id !== 'taps') {
            n.dataset.id = 'taps';
            n.replaceChildren(el('kbd', null, null, 'Ctrl'), el('kbd', null, null, 'Ctrl'));
        }
        [...n.children].forEach((k, i) => k.classList.toggle('down', t >= TAPS[i][0] && t < TAPS[i][1]));
        n.style.opacity = Math.min(ramp(t, .35, .53), 1 - ramp(t, 1.25, 1.45, E.lin));
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);

        // the Peek: dimmer in (83 ms, slowed a little), fences above it; Enter ends both
        const peek = t >= PEEK_ON && t < ENTER + FADE;
        const dim = ramp(t, PEEK_ON, PEEK_ON + FADE, E.lin) * (1 - ramp(t, ENTER, ENTER + FADE, E.lin));
        if (show(r.dim, dim > 0)) r.dim.style.opacity = (.302 * dim).toFixed(3);
        r.fences.forEach(f => {
            f.n.style.zIndex = peek ? 3 : 1;
        });

        // the box: typed text, the list growing as the fence rows and then Everything's arrive
        const boxOn = t >= PEEK_ON + .03 && t < ENTER + .1;
        if (show(r.box, boxOn)) {
            const o = Math.min(ramp(t, PEEK_ON + .03, PEEK_ON + .13, E.lin), 1 - ramp(t, ENTER, ENTER + .1, E.lin));
            put(r.box, BOX.x, BOX.y, o, ` scale(${lerp(.98, 1, ramp(t, PEEK_ON, PEEK_ON + .2)).toFixed(4)})`);
            const n = t < TYPE_AT ? 0 : Math.min(r.q.length, 1 + Math.floor((t - TYPE_AT) / TYPE_STEP));
            text(r.typed, r.q.slice(0, n));
            show(r.caret, Math.floor((t - PEEK_ON) / .5) % 2 === 0);
            r.caret.style.left = (40 + r.typed.offsetWidth + 1) + 'px';

            const fenceRows = t >= FENCE_ROWS, evRows = t >= EV_ROWS;
            const sel = t >= DOWN ? 1 : 0;
            text(r.status, !fenceRows ? '' : evRows ? `${sel + 1} / 3` : '1 / 1');
            show(r.line, fenceRows);
            let y = BOX.h + GAP;
            show(r.heads[0], evRows);
            if (evRows) {
                put(r.heads[0], 12, y);
                y += HEAD_H;
            }
            r.rows.forEach((o, i) => {
                const on = i === 0 ? fenceRows : evRows;
                if (!show(o.n, on)) return;
                if (i === 1) {
                    show(r.heads[1], true);
                    put(r.heads[1], 12, y, ramp(t, EV_ROWS, EV_ROWS + .12, E.lin));
                    y += HEAD_H;
                }
                put(o.n, 0, y, i === 0 ? 1 : ramp(t, EV_ROWS, EV_ROWS + .12, E.lin));
                o.bg.style.opacity = i === sel ? 1 : 0;
                y += ROW_H;
            });
            if (!evRows) show(r.heads[1], false);
            size(r.box, BOX.w, fenceRows ? y + PAD_B : BOX.h);
        }

        // fences: the match lights up (the current row also gets the focus frame), the rest fade
        const searching = t >= FENCE_ROWS && t < ENTER + .1;
        const fade = searching ? ramp(t, FENCE_ROWS, FENCE_ROWS + .083, E.lin) : 0;
        r.fences.forEach((f, fi) => f.icons.forEach((n, i) => {
            const hit = fi === HIT[0] && i === HIT[1];
            n.style.opacity = hit ? 1 : (1 - .85 * fade).toFixed(3);
            n.classList.toggle('sr-hit', hit && searching);
            n.classList.toggle('sr-cur', hit && searching && t < DOWN);
        }));

        // Enter: the spreadsheet opens (fades in while growing from 94 %, as Windows 11 does)
        if (show(r.app, t >= OPEN)) {
            put(r.app, 150, 50, ramp(t, OPEN, OPEN + .2, E.lin), ` scale(${lerp(.94, 1, ramp(t, OPEN, OPEN + .3)).toFixed(4)})`);
        }

        drawTaps(r.keys, t);
        if (t >= 1.45) PM.drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, { x: 700, y: 384 }, {});
    }

    PM.register('search', { duration: D, poster: 3.0, build, render });
})();
