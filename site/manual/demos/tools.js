// 7.1 便签、时钟和脚本栅栏 (lesson id tools): 设置 → 小工具 (ui/settings.html #page-tools): one
// 小工具 switch at the top (on), then a list with a button on each row. 「新建便签」 puts a note on
// the desktop with its editor open (note_edit.rs); a click elsewhere saves, and the lines
// starting with [ ] become to-dos whose box a click ticks (fence_chrome.rs draw_note: the box
// fills with the accent, the text turns secondary). 时钟's 「添加」 adds the clock (draw_clock: the
// time large, the long date with the weekday under it); there is one clock at most, so the
// button then reads 「已在桌面上」, greyed. 天气's 「添加」 adds the weather script fence, named
// after its file (天气.ps1): 「天气.ps1 · 正在运行…」 until the script's lines come, then
// 「… · 10:24 更新」 at the bottom.
// The window is the shared settings mock (window.SM, demos/settings-mock.js).
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, size, show, text, fence, pointer, drawPointer,
        taskbar, loopFade, T } = PM;

    const D = 9.4;
    const WIN = { x: 18, y: 16, w: 488, h: 392 };
    // The app makes notes 280 x 240, clocks 240 x 140 and script fences 280 x 160 DIP; scaled
    // here to share the right side of the stage.
    const NOTE = { x: 532, y: 16, w: 246, h: 128, title: '便签' };
    const CLOCK = { x: 532, y: 152, w: 246, h: 94, title: '时钟' };
    const SCRIPT_NAME = '天气.ps1';
    // a script fence is titled with its file's name (tools.rs create_tool_fence)
    const SCRIPT = { x: 532, y: 254, w: 246, h: 158, title: SCRIPT_NAME.replace(/\.ps1$/, '') };

    const lang = () => document.documentElement.lang || 'zh-CN';
    const TODO = '[ ] 交周报\n[ ] 订会议室\n周五前寄出合同';
    const DATE = '2026年11月5日 星期四';
    const WEATHER = ['Tokyo  18°C  多云', '体感 17°C，湿度 62%，风 9 km/h', '今天 12 到 19°C，明天 13 到 21°C，降雨概率 40%'];

    const NEW_NOTE = 1.0, TYPE_AT = 1.35, TYPE_STEP = .06, SAVE = 3.4, TICK = 4.15;
    const ADD_CLOCK = 5.05, ADD_WEATHER = 6.2, OUTPUT = 6.95;
    const FADE_IN = .167;
    let PATH, CLICKS, CHIPS;
    const at = t => pathAt(t, PATH);

    // A list row under the 小工具 card (the page's .card.child in one .group, text indented): its
    // title and description on the left, buttons on the right; the description is cut at them.
    function row(pg, t, s, gap = 1) {
        return SM.card(pg, { t, s, h: 36, indent: 22, textW: pg.cardW - 34 - 12, gap });
    }

    function toolsPage(w) {
        const g = SM.page(w, 'tools');
        SM.header(g, '小工具');
        const lede = SM.block(g, 'tl-lede', 28, 4);
        lede.n.textContent = T('便签、时钟和脚本放在栅栏里，和普通栅栏一样可以移动、吸附和收起。');
        // a translation may wrap to a second line: the block grows to the text (layout px)
        lede.n.style.height = 'auto';
        lede.h = Math.max(15, lede.n.offsetHeight + 2);
        size(lede.n, g.cardW, lede.h);
        const r = {};
        // the one switch for every tool, on, with the page's glyph; its description wraps to a second
        // line beside the switch, and the rows' text lines up with its text
        const top = SM.card(g, { ic: '\uEA86', t: '小工具', s: '关上后，桌面上的小工具都不显示，脚本也全部停下', h: 52, textW: g.cardW - 34 - 52 - 24, gap: 1 });
        Object.assign(top.s.style, { whiteSpace: 'normal', textWrap: 'balance', lineHeight: '12px' });
        top.t.style.top = '8px';
        top.s.style.top = '23px';
        SM.drawToggle(SM.toggle(top), true);
        r.newNote = SM.button(row(g, '便签与待办', '在栅栏里写几行字，单击就能编辑。行首写 [ ] 的一行会变成可以勾选的待办'), '新建便签', 64, { accent: true });
        r.addClock = SM.button(row(g, '时钟', '显示时间、日期和星期。右键时钟可以打开「显示秒」'), '添加', 52);
        r.addWeather = SM.button(row(g, '天气', '今天和明天的天气，每 30 分钟更新，数据来自 wttr.in。添加后右键它选「编辑脚本」，把 Tokyo 换成你的城市'), '添加', 52);
        SM.button(row(g, 'GitHub 仓库动态', '一个仓库最新的 issue 和 PR，点一行在浏览器里打开。添加后右键它选「编辑脚本」，换成你关心的仓库'), '添加', 52);
        SM.button(row(g, '电脑状态', '一直运行，每 2 秒更新 CPU、内存和 C 盘剩余空间。点一行可以复制本机 IP 或打开任务管理器'), '添加', 52);
        const own = row(g, '用自己的脚本', '运行你选的脚本（PowerShell、Python、批处理等），把它打印的文字显示在栅栏里。脚本最多运行 10 秒（一直运行的除外），不弹出窗口。PecoFence 本身不会为脚本联网', 0);
        const pick = SM.button(own, '选择脚本…', 64);
        SM.button(own, '写法说明', 52, { right: 10 + pick.w + 6 });
        // 时钟's button reads 「已在桌面上」 once the clock is there: as wide as that label needs
        const b = r.addClock;
        b.wAdd = b.w;
        text(b.v, T('已在桌面上'));
        b.wOn = Math.max(b.w, Math.ceil(SM.natW(b.v)) + 16);
        text(b.v, T('添加'));
        g.contentH = SM.stack(g) + 10;
        // the row's text ends 8 px before the button, whichever label it shows
        const fitW = n => parseFloat(n.style.width);
        b.fit = [b.b.t, b.b.s].map(n => [n, fitW(n), fitW(n) - (b.wOn - b.wAdd)]);
        r.g = g;
        return r;
    }

    // A to-do line's parts: [ ] / [] / [x] at the start (after an optional - or *) makes a box.
    function noteLines(textValue) {
        return textValue.split('\n').map(line => {
            const m = /^\s*(?:[-*]\s+)?\[( |x|X)?\]\s?(.*)$/.exec(line);
            return m ? { todo: true, done: /x/i.test(m[1] || ''), text: m[2] } : { todo: false, text: line };
        });
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;

        const w = r.w = SM.win(S, WIN);
        w.foot.textContent = T('{0} 个栅栏 · {1} 个项目').replace('{0}', '5').replace('{1}', '26');
        r.page = toolsPage(w);

        // the note: editor while typing, then its lines
        r.N = fence(S, { ...NOTE });
        r.edit = el('div', 'tl-edit', r.N.body);
        r.editText = el('div', 'tl-edit-t', r.edit);
        r.editCaret = el('div', 'tl-caret', r.edit);
        r.measure = el('span', 'tl-measure', r.edit);
        r.lines = el('div', 'tl-lines', r.N.body);
        r.todo = noteLines(TODO).map(l => {
            const n = el('div', 'tl-line' + (l.todo ? ' todo' : ''), r.lines);
            const box = l.todo ? el('div', 'tl-box', n) : null;
            if (box) box.innerHTML = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="m2.6 6.3 2.3 2.3 4.6-4.9" fill="none" ' +
                'stroke="#fff" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';
            el('span', null, n, l.text);
            return { n, box, todo: l.todo };
        });

        // the clock: time large, the long date and the weekday under it
        r.C = fence(S, { ...CLOCK });
        el('div', 'tl-time', r.C.body, /^en/.test(lang()) ? '10:24 AM' : '10:24');
        el('div', 'tl-date', r.C.body, DATE);

        // the weather script fence
        r.W = fence(S, { ...SCRIPT });
        r.out = el('div', 'tl-out', r.W.body);
        WEATHER.forEach(l => el('div', 'tl-line', r.out, l));
        r.foot = el('div', 'tl-foot', r.W.body);

        r.ptr = pointer(S);
        taskbar(root);

        // targets: the buttons. SM.at misses a card's CSS left, as in title-hover.js: measure the
        // real offset once.
        const scale = root.getBoundingClientRect().width / 800 || 1;
        const dx = (r.page.newNote.n.getBoundingClientRect().left - root.getBoundingClientRect().left) / scale - SM.box(r.page.newNote).x;
        const fix = q => ({ ...q, x: q.x + dx });
        const note = fix(SM.at(r.page.newNote));
        const clock = fix(SM.at(r.page.addClock));
        const weather = fix(SM.at(r.page.addWeather));
        // the first to-do box (note body: 14 px in, line 1)
        const box = { x: NOTE.x + 14 + 6.5, y: NOTE.y + 32 + 2 + 8.5 };

        PATH = [
            [0, 330, 380], [0.3, 330, 380],
            [0.85, note.x, note.y], [1.3, note.x, note.y],          // 新建便签
            [2.0, 300, 384], [2.9, 300, 384],                       // aside while the note is typed
            [3.25, 160, 384], [3.6, 160, 384],                      // a click elsewhere saves
            [4.0, box.x, box.y], [4.4, box.x, box.y],               // tick the first to-do
            [4.9, clock.x, clock.y], [5.3, clock.x, clock.y],       // 时钟 › 添加
            [6.05, weather.x, weather.y], [6.45, weather.x, weather.y],   // 天气 › 添加
            [7.3, 519, 196], [D, 519, 196],                         // rest between the window and the fences
        ];
        CLICKS = [{ t: NEW_NOTE }, { t: SAVE }, { t: TICK }, { t: ADD_CLOCK }, { t: ADD_WEATHER }];
        CHIPS = [[3.15, 3.55, '点别处保存', 16, -30], [3.95, 4.3, '点方框', 16, 18]];
        return r;
    }

    // A new tool fence fades in (167 ms) while growing from 97 % (as portal.js).
    function appear(f, t, at0) {
        if (!show(f.n, t >= at0)) return;
        const o = ramp(t, at0, at0 + FADE_IN, E.lin), s = lerp(.97, 1, ramp(t, at0, at0 + .25));
        put(f.n, f.x, f.y, o, ` scale(${s.toFixed(4)})`);
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const P = r.page;

        SM.drawNav(r.w, t, [[0, 'tools']], [], p);
        // 时钟's button: 「已在桌面上」, greyed, once the clock is on the desktop
        const b = P.addClock, has = t >= ADD_CLOCK + .03;
        text(b.v, T(has ? '已在桌面上' : '添加'));
        b.w = has ? b.wOn : b.wAdd;
        size(b.n, b.w, b.h);
        b.n.classList.toggle('tl-dis', has);
        for (const [n, off, on] of b.fit) n.style.width = Math.max(0, has ? on : off) + 'px';
        SM.drawButton(b, has ? null : p, t >= ADD_CLOCK - .08 && t < ADD_CLOCK + .03);
        SM.drawButton(P.newNote, p, t >= NEW_NOTE - .08 && t < NEW_NOTE + .08);
        SM.drawButton(P.addWeather, p, t >= ADD_WEATHER - .08 && t < ADD_WEATHER + .08);

        // the note: the editor opens with it; typed text; a click elsewhere saves and draws the lines
        appear(r.N, t, NEW_NOTE + .05);
        const editing = t < SAVE;
        show(r.edit, editing);
        show(r.lines, !editing);
        if (editing) {
            const typed = SM.typed(t, TYPE_AT, TODO, TYPE_STEP, 1.7);   // longer translations type quicker
            text(r.editText, typed);
            const lines = typed.split('\n');
            const last = lines[lines.length - 1];
            r.editCaret.style.display = Math.floor((t - NEW_NOTE) / .5) % 2 === 0 ? '' : 'none';
            // caret after the last typed character: measured with a hidden copy of the last line
            text(r.measure, last);
            r.editCaret.style.left = (10 + r.measure.offsetWidth) + 'px';
            r.editCaret.style.top = (6 + (lines.length - 1) * 17) + 'px';
        }
        r.todo[0].n.classList.toggle('done', t >= TICK);

        // the clock
        appear(r.C, t, ADD_CLOCK + .05);

        // the weather fence: running first, then the lines it printed
        appear(r.W, t, ADD_WEATHER + .05);
        const done = t >= OUTPUT;
        show(r.out, done);
        if (done) r.out.style.opacity = ramp(t, OUTPUT, OUTPUT + .12, E.lin);
        const time = /^en/.test(lang()) ? '10:24 AM' : '10:24';
        text(r.foot, done ? `${SCRIPT_NAME} · ${T('{0} 更新').replace('{0}', time)}` : T('{0} · 正在运行…').replace('{0}', SCRIPT_NAME));

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at });
    }

    PM.register('tools', { duration: D, poster: 8.2, build, render });
})();
