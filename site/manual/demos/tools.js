// 7.1 从插件商店添加小工具 (lesson id tools): 设置 → 小工具 (ui/settings.html #page-tools): the
// 小工具 switch (on; the page's lede and the 用自己的脚本 row are left out to make room), then 插件,
// the store's plugins sorted by name, each row titled
// 「名称 · 作者 · 版本」 over its description, grey tags for what reaches past the fence (联网), and
// a button (安装, 正在下载… while it downloads, 已在桌面上 once its fence is there). 安装 opens the
// install warning (plConfirm): 「安装“待办”？」, 「作者 {0}，审查过的版本是 {1}。」, a box with
// 「它会做：」 and the plugin's lines (grey when it only shows content, yellow when it reaches past
// its fence: 天气 联网：wttr.in), then 取消 / 安装并添加. The 待办 plugin (plugins/todo) draws a
// line to type into on top (「添加一件事…」); Enter adds the task as a row with a check box; a
// click ticks it, and done tasks turn grey and go under the others. The 天气 plugin first says
// 「正在从 wttr.in 获取天气…」, then the weather of the network's place.
// The window is the shared settings mock (window.SM, demos/settings-mock.js).
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, size, show, text, fence, pointer, drawPointer,
        taskbar, loopFade, T } = PM;

    const D = 10.4;
    const WIN = { x: 18, y: 16, w: 488, h: 392 };
    // The app makes plugin fences 280 x 240 DIP; scaled here to share the right side of the stage.
    const TODO = { x: 532, y: 16, w: 246, h: 168, title: '待办' };
    const WEATHER = { x: 532, y: 196, w: 246, h: 190, title: '天气' };
    const AUTHOR = 'DayuanJiang', VERSION = '1.0.0';
    // The store's rows in the page's order (by name); `does` (or `net`, a host it connects to) is
    // the install warning's list.
    const PLUGINS = [
        { id: 'notes', name: '便签', s: '桌面上的便签。以 [ ] 开头的行是待办，可以直接勾选。' },
        { id: 'todo', name: '待办', s: '栅栏里的待办清单：输入一件事，做完打勾。内容只存在这台电脑上。', does: ['只显示内容，不联网，也不读剪贴板'] },
        { id: 'clock', name: '时钟', s: '按 Windows 的格式显示时间和日期，字号随栅栏大小变化。' },
        { id: 'weather', name: '天气', s: '现在、今天和明天的天气，数据来自 wttr.in，每 30 分钟更新一次。', tags: ['联网'], net: 'wttr.in', reaches: true },
    ];
    const TASKS = ['交周报', '订会议室'];
    const WEATHER_ROWS = ['Tokyo  18°C  多云', '体感 17°C，湿度 62%，风 9 km/h', '今天 12 到 19°C，明天 13 到 21°C，降雨概率 40%'];

    const INSTALL1 = 1.0, OK1 = 2.1, TODO_IN = 2.6, FOCUS = 3.3, TYPE1 = 3.45, ENTER1 = 4.15,
        TYPE2 = 4.3, ENTER2 = 5.05, TICK = 5.8, INSTALL2 = 6.8, OK2 = 7.8, WEATHER_IN = 8.3, OUTPUT = 8.9;
    const FADE = .12, DOWNLOAD = .5;
    let PATH, CLICKS, NOTES;
    const at = t => pathAt(t, PATH);
    const lang = () => document.documentElement.lang || 'zh-CN';

    function toolsPage(w) {
        const g = SM.page(w, 'tools');
        SM.header(g, '小工具');
        const top = SM.card(g, { ic: '\uEA86', t: '小工具', s: '关闭后，桌面上的小工具都不显示，脚本也全部停下', h: 52, textW: g.cardW - 34 - 52 - 24, gap: 6 });
        Object.assign(top.s.style, { whiteSpace: 'normal', textWrap: 'balance', lineHeight: '12px' });
        top.t.style.top = '8px';
        top.s.style.top = '23px';
        SM.drawToggle(SM.toggle(top), true);
        SM.h3(g, '插件');
        const pl = SM.block(g, 'tl-lede', 15, 4);
        pl.n.textContent = T('插件只能做它列出的事，每个版本都经过自动检查和审查。点一行看详情。');
        // a translation may wrap to a second line: the block grows to the text (layout px)
        pl.n.style.height = 'auto';
        pl.h = Math.max(15, pl.n.offsetHeight + 2);
        size(pl.n, g.cardW, pl.h);
        const rows = {};
        PLUGINS.forEach((p, i) => {
            const h = p.tags ? 52 : 38;
            const c = SM.card(g, { t: p.name, s: p.s, h, textW: g.cardW - 24, gap: i === PLUGINS.length - 1 ? 0 : 1 });
            // the name, then 「 · 作者 · 版本」 in the secondary colour, the description under it
            c.t.style.top = '5px';
            c.s.style.top = '20px';
            el('span', 'tl-meta', c.t, ` · ${AUTHOR} · ${VERSION}`);
            if (p.tags) {
                const tags = el('div', 'tl-tags', c.n);
                tags.style.top = '35px';
                p.tags.forEach(x => el('span', null, tags, T(x)));
            }
            const b = SM.button(c, '安装', 52);
            b.label = '安装';
            // the widest of the button's three labels, so the row's text never moves
            b.wAll = Math.max(...['安装', '正在下载…', '已在桌面上'].map(s => {
                text(b.v, T(s));
                return Math.ceil(SM.natW(b.v)) + 16;
            }), b.w);
            text(b.v, T('安装'));
            b.w0 = b.w;
            rows[p.id] = { c, b, p };
        });
        g.contentH = SM.stack(g) + 10;
        return { g, rows };
    }

    // The install warning over the window: a dimmed window, the dialog in the middle of its page.
    function dialog(w, p) {
        const back = el('div', 'tl-back', w.n);
        size(back, w.w, w.h);
        put(back, 0, 0);
        const d = el('div', 'tl-dlg', back);
        const dw = Math.min(300, w.mainW - 24);
        el('div', 'tl-dlg-h', d, T('安装“{0}”？').replace('{0}', T(p.name)));
        el('div', 'tl-dlg-p', d, T('作者 {0}，审查过的版本是 {1}。').replace('{0}', AUTHOR).replace('{1}', VERSION));
        const box = el('div', 'tl-dlg-warn' + (p.reaches ? '' : ' plain'), d);
        el('div', 'tl-dlg-p', box, T('它会做：'));
        const ul = el('ul', null, box);
        if (p.net) el('li', null, ul, T('联网：{0}').replace('{0}', p.net));
        (p.does || []).forEach(x => el('li', null, ul, T(x)));
        el('div', 'tl-dlg-p', box, T('插件只能做这里列出的事，每个版本都经过自动检查和审查。'));
        const btns = el('div', 'tl-dlg-btns', d);
        const cancel = el('div', 'tl-dlg-btn', btns, T('取消'));
        const ok = el('div', 'tl-dlg-btn acc', btns, T('安装并添加'));
        d.style.width = dw + 'px';
        const dh = d.offsetHeight;
        const x = w.navW + (w.mainW - dw) / 2, y = Math.max(40, (w.h - dh) / 2);
        put(d, x, y);
        // the stage centre of 安装并添加, measured once the dialog is laid out
        const okAt = { x: WIN.x + x + ok.offsetLeft + ok.offsetWidth / 2, y: WIN.y + y + btns.offsetTop + ok.offsetTop + ok.offsetHeight / 2 };
        return { back, d, ok, cancel, okAt };
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;

        const w = r.w = SM.win(S, WIN);
        w.foot.textContent = T('{0} 个栅栏 · {1} 个项目').replace('{0}', '4').replace('{1}', '26');
        const P = r.page = toolsPage(w);
        r.dlg1 = dialog(w, PLUGINS[1]);
        r.dlg2 = dialog(w, PLUGINS[3]);

        // the 待办 fence: the line to type into, then a row per task
        r.T = fence(S, { ...TODO });
        r.add = el('div', 'tl-add', r.T.body);
        r.addText = el('span', null, r.add);
        r.caret = el('span', 'tl-caret', r.add);
        r.lines = el('div', 'tl-lines tl-tasks', r.T.body);
        r.tasks = TASKS.map(s => {
            const n = el('div', 'tl-line todo', r.lines);
            const box = el('div', 'tl-box', n);
            box.innerHTML = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="m2.6 6.3 2.3 2.3 4.6-4.9" fill="none" ' +
                'stroke="#fff" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';
            el('span', null, n, T(s));
            return n;
        });
        r.empty = el('div', 'tl-line tl-gray', r.lines, T('还没有待办。点上面一行，输入后按 Enter。'));

        // the 天气 fence: fetching first, then the weather and the time it came
        r.W = fence(S, { ...WEATHER });
        r.wait = el('div', 'tl-line tl-gray tl-wait', r.W.body, T('正在从 wttr.in 获取天气…'));
        r.out = el('div', 'tl-out', r.W.body);
        WEATHER_ROWS.forEach((l, i) => el('div', 'tl-line' + (i === 0 ? ' tl-big' : ''), r.out, T(l)));
        const time = /^en/.test(lang()) ? '10:24 AM' : '10:24';
        el('div', 'tl-line tl-gray tl-small', r.out, T('{0} 更新，点一下刷新').replace('{0}', time));

        r.ptr = pointer(S);
        taskbar(root);

        // targets. SM.at misses a card's CSS left, as in title-hover.js: measure the real offset once.
        const scale = root.getBoundingClientRect().width / 800 || 1;
        const b0 = P.rows.todo.b;
        const dx = (b0.n.getBoundingClientRect().left - root.getBoundingClientRect().left) / scale - SM.box(b0).x;
        const fix = q => ({ ...q, x: q.x + dx });
        const install1 = fix(SM.at(P.rows.todo.b));
        const install2 = fix(SM.at(P.rows.weather.b));
        const ok1 = r.dlg1.okAt, ok2 = r.dlg2.okAt;
        const addAt = { x: TODO.x + 70, y: TODO.y + 32 + 14 };
        const boxAt = { x: TODO.x + 14 + 6.5, y: TODO.y + 32 + 34 + 8.5 };

        PATH = [
            [0, 330, 380], [0.3, 330, 380],
            [0.85, install1.x, install1.y], [1.25, install1.x, install1.y],   // 待办 › 安装
            [1.85, ok1.x, ok1.y], [2.3, ok1.x, ok1.y],                        // 安装并添加
            [3.1, addAt.x, addAt.y], [3.5, addAt.x, addAt.y],                 // the line to type into
            [4.6, addAt.x + 60, addAt.y + 70], [5.3, addAt.x + 60, addAt.y + 70],   // aside while typing
            [5.65, boxAt.x, boxAt.y], [6.0, boxAt.x, boxAt.y],               // tick the first task
            [6.6, install2.x, install2.y], [7.0, install2.x, install2.y],     // 天气 › 安装
            [7.6, ok2.x, ok2.y], [8.0, ok2.x, ok2.y],                         // 安装并添加
            [8.8, 519, 380], [D, 519, 380],                                   // rest below the fences
        ];
        CLICKS = [{ t: INSTALL1 }, { t: OK1 }, { t: FOCUS }, { t: TICK }, { t: INSTALL2 }, { t: OK2 }];
        NOTES = [[TYPE1 + .2, ENTER2 + .3, '按 Enter 加上', TODO.x + TODO.w, TODO.y + TODO.h + 8, 'abs', 'right']];
        return r;
    }

    // A new plugin fence fades in (167 ms) while growing from 97 % (as portal.js).
    function appear(f, t, at0) {
        if (!show(f.n, t >= at0)) return;
        const o = ramp(t, at0, at0 + .167, E.lin), s = lerp(.97, 1, ramp(t, at0, at0 + .25));
        put(f.n, f.x, f.y, o, ` scale(${s.toFixed(4)})`);
    }

    // The row's button: 安装, then 正在下载… from the click on 安装并添加, then 已在桌面上, greyed.
    function drawRow(row, t, install, ok, p) {
        const b = row.b;
        const label = t < ok ? '安装' : t < ok + DOWNLOAD ? '正在下载…' : '已在桌面上';
        if (label !== b.label) {
            text(b.v, T(label));
            b.label = label;
            b.w = label === '安装' ? b.w0 : b.wAll;
            size(b.n, b.w, b.h);
            // the row's text ends 8 px before the button, whichever label it shows
            SM.stack(row.c.pg);
        }
        b.n.classList.toggle('tl-dis', label !== '安装');
        SM.drawButton(b, label === '安装' ? p : null, install != null && t >= install - .08 && t < install + .03);
    }

    function drawDialog(dlg, t, open, ok, p) {
        const on = t >= open && t < ok + FADE;
        if (!show(dlg.back, on)) return;
        dlg.back.style.opacity = t < ok ? ramp(t, open, open + FADE, E.lin) : 1 - ramp(t, ok, ok + FADE, E.lin);
        const hot = (n, q) => {
            if (!q) return false;
            const r = n.getBoundingClientRect(), s = dlg.back.getBoundingClientRect();
            const k = s.width / WIN.w || 1;
            const x = WIN.x + (r.left - s.left) / k, y = WIN.y + (r.top - s.top) / k;
            return q.x >= x && q.x <= x + r.width / k && q.y >= y && q.y <= y + r.height / k;
        };
        dlg.ok.classList.toggle('hot', hot(dlg.ok, p));
        dlg.ok.classList.toggle('down', t >= ok - .08 && t < ok + .03);
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const P = r.page;

        SM.drawNav(r.w, t, [[0, 'tools']], [], p);
        const dialogOpen = (t >= INSTALL1 && t < OK1) || (t >= INSTALL2 && t < OK2);
        const q = dialogOpen ? null : p;
        drawRow(P.rows.todo, t, INSTALL1, OK1, q);
        drawRow(P.rows.weather, t, INSTALL2, OK2, q);
        for (const id of ['notes', 'clock']) SM.drawButton(P.rows[id].b, q, false);
        drawDialog(r.dlg1, t, INSTALL1 + .03, OK1, p);
        drawDialog(r.dlg2, t, INSTALL2 + .03, OK2, p);

        // the 待办 fence: the typed task in the top line, a row for each Enter; the first task,
        // ticked, turns grey and goes under the other
        appear(r.T, t, TODO_IN);
        const typing1 = t >= TYPE1 && t < ENTER1, typing2 = t >= TYPE2 && t < ENTER2;
        const typed = typing1 ? SM.typed(t, TYPE1, T(TASKS[0]), .12, .6) : typing2 ? SM.typed(t, TYPE2, T(TASKS[1]), .12, .6) : '';
        text(r.addText, typed || T('添加一件事…'));
        r.add.classList.toggle('ph', !typed);
        const focused = t >= FOCUS && t < TICK - .25;
        show(r.caret, focused && Math.floor((t - FOCUS) / .5) % 2 === 0);
        r.add.classList.toggle('focus', focused);
        const added = (t >= ENTER1) + (t >= ENTER2);
        r.tasks.forEach((n, i) => show(n, i < added));
        show(r.empty, added === 0);
        const done = t >= TICK;
        r.tasks[0].classList.toggle('done', done);
        // done tasks go under the open ones
        if (done && r.lines.firstChild === r.tasks[0]) r.lines.appendChild(r.tasks[0]);
        if (!done && r.lines.firstChild !== r.tasks[0]) r.lines.insertBefore(r.tasks[0], r.lines.firstChild);
        PM.drawChip(r.chip || (r.chip = el('div', 'chip', r.scene)), t, NOTES, p);

        // the 天气 fence: fetching, then the weather
        appear(r.W, t, WEATHER_IN);
        const out = t >= OUTPUT;
        show(r.wait, !out);
        show(r.out, out);
        if (out) r.out.style.opacity = ramp(t, OUTPUT, OUTPUT + .12, E.lin);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: [], at });
    }

    PM.register('tools', { duration: D, poster: 9.6, build, render });
})();
