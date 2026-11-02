// PecoFence manual: a miniature of the settings window (ui/settings.html, light theme) shared
// by the settings lessons (rules, look, backup). Labels are the page's zh-CN source strings.
// Layout is explicit (fixed heights, absolute positions) so a demo's render(t) stays a pure
// function of t and pointer targets can be computed up front. Exported as window.SM.
(() => {
    'use strict';
    const { SW, E, bezier, ramp, el, put, size, show, text, T } = PM;

    const TITLE_H = 30;
    const CARD_X = 12, CARD_R = 14;
    // nav order, labels and Segoe Fluent glyphs from ui/settings.html <nav>
    const NAV = [['general', '常规', '\uE713'], ['fences', '栅栏', '\uE71D'], ['rules', '整理规则', '\uE8FD'],
        ['layout', '布局与备份', '\uE7F4'], ['feedback', '反馈', '\uED15'], ['about', '关于', '\uE946']];
    const pageIn = bezier(.1, .9, .2, 1);       // .page.on animation: page-in .2s

    const svg = (vb, body) => `<svg viewBox="${vb}" aria-hidden="true">${body}</svg>`;
    const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"';
    const ICON = {
        chevDown: svg('0 0 12 12', `<path d="m2.5 4.25 3.5 3.5 3.5-3.5" ${stroke} stroke-width="1.2" style="color:#5d5d5d"/>`),
        up: svg('0 0 10 10', `<path d="M1.6 6.6 5 3.2l3.4 3.4" ${stroke} stroke-width="1.15"/>`),
        down: svg('0 0 10 10', `<path d="M1.6 3.4 5 6.8l3.4-3.4" ${stroke} stroke-width="1.15"/>`),
        del: svg('0 0 10 10', `<path d="M1.2 2.3h7.6M3.6 2.3V1.2h2.8v1.1M2.2 2.3l.5 6.5h4.6l.5-6.5" ${stroke} stroke-width="1"/>`),
        min: svg('0 0 10 10', `<path d="M.5 5h9" ${stroke} stroke-width="1"/>`),
        max: svg('0 0 10 10', `<rect x="1" y="1" width="8" height="8" rx="1" ${stroke} stroke-width="1"/>`),
        close: svg('0 0 10 10', `<path d="m1 1 8 8M9 1 1 9" ${stroke} stroke-width="1"/>`),
        ok: svg('0 0 12 12', '<circle cx="6" cy="6" r="6" fill="#0f7b0f"/><path d="m3.4 6.2 1.7 1.7 3.5-3.6" fill="none" ' +
            'stroke="#fff" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/>'),
    };

    // The glyphs come from Segoe Fluent Icons (Segoe MDL2 Assets on Windows 10), which only
    // Windows has; elsewhere they are dropped rather than drawn as boxes.
    function checkIconFont() {
        if (!/Windows/.test(navigator.userAgent)) document.documentElement.classList.add('set-noic');
    }

    // ---------- measuring (translations are longer than the zh-CN text) ----------

    // Natural width of a text element in stage px, ignoring its right / width / max-width
    // constraints (labels are nowrap). 0 while it is not laid out; callers keep their size then.
    function natW(n) {
        const st = n.style, keep = [st.right, st.width, st.maxWidth];
        st.right = 'auto';
        st.width = 'auto';
        st.maxWidth = 'none';
        const stage = n.closest('.stage-inner');
        const scale = stage ? stage.getBoundingClientRect().width / SW || 1 : 1;
        const w = n.getBoundingClientRect().width / scale;
        [st.right, st.width, st.maxWidth] = keep;
        return w;
    }

    // Widens control c to w (never narrows it); right-anchored controls grow to the left.
    function grow(c, w) {
        if (!(w > c.w)) return c;
        c.w = w;
        size(c.n, c.w, c.h);
        return c;
    }

    // ---------- window ----------

    // {x, y, w, h, navW}: the window at stage (x, y). Pages go into win.main. A nav label that
    // does not fit widens the nav by up to 24 px; what still does not fit is ellipsized.
    function win(parent, o) {
        checkIconFont();
        const w = { x: o.x, y: o.y, w: o.w, h: o.h, navW: o.navW || 128 };
        w.n = el('div', 'set-win', parent);
        size(w.n, w.w, w.h);
        put(w.n, w.x, w.y);

        const cap = el('div', 'set-cap', w.n);
        const mark = el('img', null, cap);
        mark.src = 'assets/mark.svg';
        mark.alt = '';
        el('span', 'set-cap-t', cap, T('PecoFence 设置'));
        ['min', 'max', 'close'].forEach((k, i) => {
            const b = el('div', 'set-capb', cap);
            b.style.left = (w.w - 38 * (3 - i)) + 'px';
            b.innerHTML = ICON[k];
        });

        const nav = el('div', 'set-nav', w.n);
        const bm = el('img', 'set-brand-m', nav);
        bm.src = 'assets/mark.svg';
        bm.alt = '';
        el('span', 'set-brand-t', nav, 'PecoFence');
        w.nav = {};
        const labels = NAV.map(([key, label, glyph], i) => {
            const it = el('div', 'set-nav-i', nav);
            it.style.top = (34 + i * 28) + 'px';
            el('span', 'set-ic', it, glyph);
            const t = el('span', 'set-nav-t', it, T(label));
            const pill = el('div', 'set-nav-pill', it);
            w.nav[key] = { n: it, pill, y: 34 + i * 28 };
            return t;
        });
        // a label starts 6 + 32 px from the nav's left and ends 6 px before its item (6 px inset)
        const widths = labels.map(natW);
        w.navW = Math.min(Math.max(w.navW, Math.ceil(Math.max(...widths)) + 50), w.navW + 24);
        labels.forEach((t, i) => t.classList.toggle('fit', widths[i] > w.navW - 50));   // ellipsize
        nav.style.width = w.navW + 'px';
        w.mainW = w.w - w.navW;
        w.mainH = w.h - TITLE_H;
        w.foot = el('div', 'set-nav-foot', nav);

        w.main = el('div', 'set-main', w.n);
        w.main.style.left = w.navW + 'px';
        w.pages = {};
        w.toast = el('div', 'set-toast', w.n);
        w.toast.innerHTML = ICON.ok + '<span></span>';
        w.toastText = w.toast.querySelector('span');
        w.toast.style.display = 'none';
        return w;
    }

    // Stage centre of nav item `key`.
    const navAt = (w, key) => ({ x: w.x + 6 + 50, y: w.y + TITLE_H + w.nav[key].y + 13 });

    // pages: [[t, key], ...] → the page shown at t, with the page-in animation after a switch.
    // hover: [[a, b, key]] nav hover spans. Returns the page key and its entrance offset.
    function drawNav(w, t, pages, hover = [], p = null) {
        if (w.footFit == null) {   // the demo sets the footer text after win(): ellipsize it once laid out
            const fw = natW(w.foot);
            if (fw) w.foot.classList.toggle('fit', w.footFit = fw > w.navW - 22);
        }
        let cur = pages[0], since = -1;
        for (const e of pages) if (t >= e[0]) { cur = e; since = e[0]; }
        const key = cur[1];
        for (const [k, item] of Object.entries(w.nav)) {
            const sel = k === key;
            item.n.classList.toggle('sel', sel);
            item.pill.style.opacity = sel ? 1 : 0;
            const r = { x: w.x + 6, y: w.y + TITLE_H + item.y, w: w.navW - 12, h: 26 };
            item.n.classList.toggle('hot', !!p && p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h);
        }
        const q = since < 0 || since === pages[0][0] ? 1 : pageIn(Math.min(1, (t - since) / .2));
        for (const [k, pg] of Object.entries(w.pages)) {
            if (show(pg.n, k === key)) {
                pg.enter = q;
                placePage(pg);
            }
        }
        return key;
    }

    // ---------- page content ----------

    function page(w, key) {
        const pg = { w, key, n: el('div', 'set-page', w.main), blocks: [], scroll: 0, enter: 1 };
        pg.cardW = w.mainW - CARD_X - CARD_R;
        w.pages[key] = pg;
        return pg;
    }

    function block(pg, cls, h, gap) {
        const b = { pg, n: el('div', cls, pg.n), h, gap, y: 0, on: true, ctrls: [] };
        size(b.n, pg.cardW, h);
        pg.blocks.push(b);
        return b;
    }

    const header = (pg, s) => {
        const b = block(pg, 'set-h2', 36, 0);
        b.n.textContent = T(s);
        return b;
    };
    const h3 = (pg, s) => {
        const b = block(pg, 'set-h3', 26, 0);
        b.n.textContent = T(s);
        return b;
    };

    // A settings card: optional glyph, title and description, controls added with ctrl().
    // `textW` keeps the text clear of the controls on the right.
    function card(pg, { ic, t, s, h = 40, textW = 190, indent = 0, gap = 3 }) {
        const b = block(pg, 'set-card', h, gap);
        const x0 = ic ? 34 : 12 + indent;
        b.x0 = x0;
        b.textW = textW;
        if (ic) {
            const g = el('span', 'set-ic', b.n, ic);
            g.style.top = (h / 2 - 6.5) + 'px';
        }
        const ty = s ? h / 2 - 14 : h / 2 - 7.5;
        if (t != null) {
            b.t = el('div', 'set-t', b.n, T(t));
            b.t.style.left = x0 + 'px';
            b.t.style.top = ty + 'px';
            b.t.style.width = textW + 'px';
        }
        if (s != null) {
            b.s = el('div', 'set-s', b.n, T(s));
            b.s.style.left = x0 + 'px';
            b.s.style.top = (ty + 15) + 'px';
            b.s.style.width = textW + 'px';
        }
        return b;
    }

    // Scroll position and the page-in entrance (opacity, 8 px rise).
    function placePage(pg) {
        pg.n.style.transform = `translateY(${(8 * (1 - pg.enter) - pg.scroll).toFixed(2)}px)`;
        pg.n.style.opacity = pg.enter;
    }

    // Card text ends where a control on its line starts (8 px before a switch or button), so a longer
    // translation ellipsizes there instead of running under the control.
    const FIT = ['set-tg', 'set-btn', 'set-sel', 'set-inp', 'set-ibtn', 'set-sl'], GAP = { 'set-tg': 8, 'set-btn': 8, 'set-sl': 8 };
    function fitText(b) {
        for (const n of [b.t, b.s]) {
            if (!n) continue;
            const top = parseFloat(n.style.top) || 0, bottom = top + 15;
            let lim = b.textW;
            for (const c of b.ctrls) {
                if (!FIT.some(k => c.n.classList.contains(k)) || c.top >= bottom || c.top + c.h <= top) continue;
                const x = c.left != null ? c.left : b.pg.cardW - c.right - c.w;
                if (x > b.x0) lim = Math.min(lim, x - b.x0 - (GAP[c.n.classList[0]] || 0));
            }
            n.style.width = Math.max(0, lim) + 'px';
        }
    }

    // Lays out the page's visible blocks top to bottom (from y0) and applies scroll + page-in.
    // Every block sits CARD_X from the page's left edge (the only horizontal offset: settings.css
    // gives blocks no left of their own), which is what rect / at / box assume.
    function stack(pg, y0 = 8) {
        let y = y0;
        for (const b of pg.blocks) {
            if (!show(b.n, b.on)) continue;
            b.y = y;
            if (b.x0 != null) fitText(b);
            put(b.n, CARD_X, y);
            y += b.h + b.gap;
        }
        placePage(pg);
        return y;
    }

    // A control inside card b: {right, top} from the card's right/top edge (or {left}).
    function ctrl(b, cls, w, h, { right = 10, left = null, top = null, label = '' } = {}) {
        const c = { b, w, h, right, left, top: top ?? (b.h - h) / 2, n: el('div', cls, b.n) };
        if (b.ctrls) b.ctrls.push(c);
        size(c.n, w, h);
        if (left != null) c.n.style.left = left + 'px';
        else c.n.style.right = right + 'px';
        c.n.style.top = c.top + 'px';
        if (label !== null && cls !== 'set-tg') {
            c.v = el('span', 'set-v', c.n, T(label));
        }
        return c;
    }
    const select = (b, label, w = 100, o = {}) => {
        const c = ctrl(b, 'set-sel', w, 22, { ...o, label });
        c.n.insertAdjacentHTML('beforeend', ICON.chevDown);
        return c;
    };
    // At least w wide, wider when its label needs it (8 px padding each side).
    const button = (b, label, w = 64, o = {}) => {
        const c = ctrl(b, 'set-btn' + (o.accent ? ' acc' : ''), w, 22, { ...o, label });
        return grow(c, Math.ceil(natW(c.v)) + 16);
    };
    const input = (b, ph, w = 120, o = {}) => {
        const c = ctrl(b, 'set-inp', w, 22, { ...o, label: '' });
        c.ph = T(ph);
        c.caret = el('span', 'set-caret', c.n);
        return c;
    };
    // Windows 11 switch with its 开 / 关 caption on the left (page script wraps .card > .toggle).
    // The caption ends 10 px before the 30 px switch; the control widens for longer captions.
    const toggle = (b, { caption = true, ...o } = {}) => {
        const c = ctrl(b, 'set-tg', caption ? 52 : 30, 15, { ...o, label: null });
        if (caption) c.cap = el('span', 'set-tog-l', c.n);
        c.tog = el('div', 'set-tog', c.n);
        if (caption) {
            const capW = Math.max(...[T('开'), T('关')].map(s => {
                c.cap.textContent = s;
                return natW(c.cap);
            }));
            c.cap.textContent = '';
            grow(c, Math.ceil(capW) + 40);
        }
        return c;
    };
    // Windows 11 slider: the 80 px rail fills with the accent up to the thumb at `frac` (0-1) of
    // the way, with ticks at `ticks` (fractions too).
    const slider = (b, frac, ticks, o = {}) => {
        const c = ctrl(b, 'set-sl', 80, 15, { ...o, label: null });
        el('span', 'set-sl-rail', c.n).style.setProperty('--fill', frac * 100 + '%');
        for (const at of ticks) el('span', 'set-sl-tick', c.n).style.left = at * 80 + 'px';
        el('span', 'set-sl-thumb', c.n).style.left = frac * 80 - 6.5 + 'px';
        return c;
    };
    const iconButton = (b, kind, o = {}) => {
        const c = ctrl(b, 'set-ibtn', 20, 20, { ...o, label: null });
        c.n.innerHTML = ICON[kind];
        return c;
    };

    // Page coordinates of a control, and its stage centre.
    const rect = c => ({
        x: CARD_X + (c.left != null ? c.left : c.b.pg.cardW - c.right - c.w),
        y: c.b.y + c.top,
        w: c.w,
        h: c.h,
    });
    function at(c, dx = 0, dy = 0) {
        const pg = c.b.pg, w = pg.w, r = rect(c);
        return { x: w.x + w.navW + r.x + r.w / 2 + dx, y: w.y + TITLE_H + r.y - pg.scroll + r.h / 2 + dy };
    }
    // Stage point on a toggle's switch (dx from the switch's centre), however wide its caption.
    const switchAt = (c, dx = 0) => at(c, c.w / 2 - 15 + dx, 0);
    // Stage rect of a control (for hover tests).
    function box(c) {
        const pg = c.b.pg, w = pg.w, r = rect(c);
        return { x: w.x + w.navW + r.x, y: w.y + TITLE_H + r.y - pg.scroll, w: r.w, h: r.h };
    }
    const over = (c, p) => {
        if (!p || !c.b.on) return false;
        const r = box(c);
        return p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h;
    };

    function drawSelect(c, label, p, open = false) {
        text(c.v, T(label));
        c.n.classList.toggle('hot', over(c, p) && !open);
        c.n.classList.toggle('down', open);
    }
    function drawButton(c, p, pressed = false) {
        c.n.classList.toggle('hot', over(c, p));
        c.n.classList.toggle('down', pressed);
    }
    function drawInput(c, value, focused, t) {
        const has = value.length > 0;
        text(c.v, has ? value : c.ph);
        c.v.classList.toggle('ph', !has);
        c.n.classList.toggle('focus', focused);
        if (show(c.caret, focused && Math.floor(t / .5) % 2 === 0)) {
            const wv = has ? c.v.offsetWidth + 1 : 0;
            c.caret.style.left = (8 + Math.min(wv, c.w - 14)) + 'px';
        }
    }
    function drawToggle(c, on) {
        c.tog.classList.toggle('on', on);
        if (c.cap) text(c.cap, T(on ? '开' : '关'));
    }

    // ---------- select popup ----------

    // Chromium's <select> list. `visible` caps the rows (a scrollbar hints at the rest). At least
    // w (the select's width) wide, wider when an option would be clipped.
    function popup(parent, options, w, visible = options.length) {
        const n = el('div', 'set-pop', parent);
        const rows = options.slice(0, visible).map((o, i) => {
            const r = el('div', 'set-pop-i', n);
            r.style.top = (4 + i * 22) + 'px';
            const label = el('span', null, r, T(o));
            return { r, label, t: o, y0: 4 + i * 22, y1: 26 + i * 22 };
        });
        const text = Math.max(...rows.map(o => natW(o.label)));   // text starts 12 px in (row 4 + padding 8)
        if (12 + text > w + 4) w = Math.ceil(text) + 24;   // 4: a CJK bracket's empty right half
        n.style.width = w + 'px';
        const h = 8 + rows.length * 22;
        n.style.height = h + 'px';
        if (visible < options.length) {
            const bar = el('div', 'set-pop-bar', n);
            bar.style.height = Math.round((h - 12) * visible / options.length) + 'px';
        }
        return { n, rows, w, h };
    }
    // Open over [a, b] (Chromium shows it at once; a 60 ms fade reads better on video).
    function drawPopup(pop, t, x, y, a, b, p, cur) {
        if (!show(pop.n, t >= a && t < b + .06)) return;
        pop.n.style.opacity = Math.min(ramp(t, a, a + .06, E.lin), 1 - ramp(t, b, b + .06, E.lin));
        pop.n.style.transform = `translate(${x}px, ${y}px)`;
        for (const row of pop.rows) {
            row.r.classList.toggle('cur', row.t === cur);
            row.r.classList.toggle('hot', t < b && p.x >= x && p.x <= x + pop.w && p.y >= y + row.y0 && p.y < y + row.y1);
        }
    }
    const popRow = (pop, label) => {
        const r = pop.rows.find(o => o.t === label);
        return (r.y0 + r.y1) / 2;
    };

    // ---------- toast (.infobar: bottom-centre of the client area, 2.4 s) ----------

    function drawToast(w, t, list) {
        const e = list.filter(([a]) => t >= a).pop();
        const dur = e ? (e[2] || 2.4) : 0;
        if (!show(w.toast, !!e && t < e[0] + dur + .18)) return;
        text(w.toastText, e[1]);
        const o = Math.min(ramp(t, e[0], e[0] + .18, E.lin), 1 - ramp(t, e[0] + dur, e[0] + dur + .18, E.lin));
        const tw = w.toast.offsetWidth;
        put(w.toast, (w.w - tw) / 2, w.h - 14 - 30 + 8 * (1 - o), o);
    }

    // ---------- the 常规 page (top of it) ----------

    // 外观: 主题风格 tiles (the page's own Fluent / Liquid Glass previews), 颜色模式, 标题对齐,
    // 默认图标大小. Returns the page and its two tiles (draw with drawTiles).
    function generalPage(w) {
        const g = page(w, 'general');
        header(g, '常规');
        h3(g, '外观');
        const style = card(g, { ic: '\uE771', t: '主题风格', h: 134 });
        style.t.style.top = '10px';
        style.n.querySelector('.set-ic').style.top = '11px';
        const tw = Math.floor((g.cardW - 34 - 12 - 8) / 2);
        const tiles = [['fl', 'Fluent · 经典'], ['lg', 'Liquid Glass · 液态玻璃']].map(([k, cap], i) => {
            const c = ctrl(style, 'set-tile', tw, 92, { left: 34 + i * (tw + 8), top: 32, label: null });
            const shot = el('div', 'set-shot', c.n);
            const mini = el('div', 'set-mini ' + k, shot);
            el('span', 'set-mini-t', mini, T('工作空间'));
            [['folder', '\uE8B7'], ['', '\uE8A5'], ['sheet', '\uE9F9']].forEach(([cls, gl], j) => {
                const ic = el('span', 'set-mini-i set-ic ' + cls, mini, gl);
                ic.style.left = (8 + j * 21) + 'px';
            });
            el('span', 'set-cap-r', c.n);
            el('span', 'set-tile-t', c.n, cap);
            return c;
        });
        const sw = Math.min(130, g.cardW - 180);
        select(card(g, { ic: '\uE7A1', t: '颜色模式', textW: 120 }), '跟随 Windows 模式', sw);
        select(card(g, { ic: '\uE8E4', t: '标题对齐', textW: 120 }), '左对齐', sw);
        // The icon size slider at 48 px, with ticks at 32, 48, 64 and 96 (24 to 128 px).
        slider(card(g, { ic: '\uE8B9', t: '默认图标大小', s: '新建栅栏时使用，每个栅栏可在“栅栏”页或右键菜单中单独设置', textW: g.cardW - sw - 60 }), 24 / 104, [8, 24, 40, 72].map(v => v / 104));
        stack(g);
        return { g, tiles };
    }
    function drawTiles(tiles, liquidGlass, p, hover = true) {
        tiles.forEach((c, i) => {
            c.n.classList.toggle('on', liquidGlass === (i === 1));
            c.n.classList.toggle('hot', hover && over(c, p));
        });
    }

    // Typed text: one character every `step` seconds from a; with `within`, faster when needed so
    // the last character is in by a + within (longer translations type quicker).
    const typed = (t, a, s, step = .13, within = Infinity) => {
        if (s.length > 1) step = Math.min(step, within / (s.length - 1));
        return t < a ? '' : s.slice(0, Math.min(s.length, 1 + Math.floor((t - a) / step)));
    };

    // A date and time as the app shows them (format_local_datetime: the locale's short date and
    // time). zh-CN keeps the demo's own text; other languages format `date` for the page's lang.
    function dateTime(zh, date) {
        const lang = document.documentElement.lang || 'zh-CN';
        if (lang === 'zh-CN') return zh;
        try {
            return new Intl.DateTimeFormat(lang, { dateStyle: 'short', timeStyle: 'short' }).format(date);
        } catch {
            return zh;
        }
    }

    window.SM = {
        TITLE_H, CARD_X, ICON, win, navAt, drawNav, page, header, h3, card, block, stack, ctrl, select, button,
        input, toggle, slider, iconButton, rect, at, box, over, drawSelect, drawButton, drawInput, drawToggle, popup,
        drawPopup, popRow, drawToast, typed, generalPage, drawTiles, natW, grow, switchAt, dateTime,
    };
})();
