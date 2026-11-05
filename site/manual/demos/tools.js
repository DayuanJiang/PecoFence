// 7.1 便签、时钟和脚本栅栏 (lesson id tools): 设置 → 小工具 (ui/settings.html #page-tools).
// 「新建便签」 turns 便签与待办 on (tools.rs ensure_tool_on) and puts a note on the desktop with its
// editor open (note_edit.rs); a click elsewhere saves, and the lines starting with [ ] become
// to-dos whose box a click ticks (fence_chrome.rs draw_note: the box fills with the accent, the
// text turns secondary). 「新建时钟」 adds the clock (draw_clock: the time large, the long date
// with the weekday under it). Lower on the page 「添加示例…」 opens a menu of the four examples
// at the pointer (tools.rs script_example_menu); 「天气」 adds the weather script fence, named
// after its file (天气.ps1): 「天气.ps1 · 正在运行…」 until the script's lines come, then
// 「… · 10:24 更新」 at the bottom.
// The window is the shared settings mock (window.SM, demos/settings-mock.js).
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, size, show, text, fence, menu, drawMenu, rowY, pointer, drawPointer,
        taskbar, loopFade, T } = PM;

    const D = 11.4;
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
    const NEW_CLOCK = 5.05, SCROLL_A = 5.75, SCROLL_B = 6.55, EXAMPLE = 7.3, NEW_SCRIPT = 7.95, OUTPUT = 8.7;
    // 「添加示例…」's menu (scripts.rs Example::ALL)
    const EXAMPLES = ['天气', 'GitHub 仓库动态', '待办（todo.txt）', '电脑状态（一直运行）'];
    const FADE_IN = .167;
    let PATH, CLICKS, CHIPS, SCROLL, MENU;
    const at = t => pathAt(t, PATH);

    // A card with a switch on the right (the page's .card > .toggle).
    function toggleCard(pg, { ic, t, s }, o = {}) {
        const b = SM.card(pg, { ic, t, s, textW: pg.cardW - 34 - 52 - 24, ...o });
        const c = SM.toggle(b);
        SM.drawToggle(c, false);
        return c;
    }

    function toolsPage(w) {
        const g = SM.page(w, 'tools');
        SM.header(g, '小工具');
        const lede = SM.block(g, 'tl-lede', 28, 4);
        lede.n.textContent = T('小工具放在栅栏里，和普通栅栏一样可以移动、吸附和收起。关着的小工具不运行，也不占内存。');
        // a translation may wrap to a third line: the block grows to the text (layout px)
        lede.n.style.height = 'auto';
        lede.h = Math.max(28, lede.n.offsetHeight + 2);
        size(lede.n, g.cardW, lede.h);
        const r = {};
        r.notes = toggleCard(g, { ic: '', t: '便签与待办', s: '在栅栏里写几行字，单击就能编辑；行首写 [ ] 的那一行变成可以勾选的待办' }, { gap: 1 });
        r.noteRow = SM.card(g, { t: '桌面上还没有便签', h: 32, indent: 22, textW: 260, gap: 6 });
        r.newNote = SM.button(r.noteRow, '新建便签', 64, { accent: true });
        r.clock = toggleCard(g, { ic: '', t: '时钟与日期', s: '显示时间、日期和星期，时间格式跟随 Windows 的区域设置' }, { gap: 1 });
        const sec = SM.card(g, { t: '显示秒', s: '每秒更新一次', h: 32, indent: 22, textW: 200, gap: 1 });
        r.seconds = SM.toggle(sec);
        SM.drawToggle(r.seconds, false);
        r.secCard = sec;
        r.clockRow = SM.card(g, { t: '桌面上还没有时钟', h: 32, indent: 22, textW: 260, gap: 6 });
        r.newClock = SM.button(r.clockRow, '新建时钟', 64, { accent: true });
        SM.h3(g, '搜索与跳转');
        toggleCard(g, { ic: '', t: '对话框跳转', s: '打开或保存文件时，切到资源管理器再切回对话框，它就跳到资源管理器里的文件夹；点文件夹栅栏或按 Ctrl+G 也能跳过去' });
        toggleCard(g, { ic: '', t: '搜索整台电脑（Everything）', s: '速览时打字搜索，除了栅栏里的图标，也列出整台电脑上的文件和文件夹，由免费的 Everything 负责查找' }, { gap: 1 });
        const ev = SM.card(g, { t: '这台电脑还没有 Everything', s: '安装时会打开一个命令行窗口，用 winget 装 1.5 版，Windows 会询问一次管理员权限', h: 36, indent: 22, textW: 260 });
        SM.button(ev, '安装 Everything', 64, { accent: true });
        r.evCard = ev;                             // dimmed while its switch is off (render)
        toggleCard(g, { ic: '', t: '双击 Ctrl 打开搜索', s: '快速按两下 Ctrl，打开速览和搜索框，再按两下关闭；全屏的游戏或视频在前面时不响应' });
        SM.h3(g, '自己写的小工具');
        r.scripts = toggleCard(g, { ic: '', t: '脚本栅栏', s: '定时运行你选的脚本（PowerShell、Python、批处理等），把它打印的文字显示在栅栏里，适合天气、倒计时、服务器状态，也可以让 AI 助手帮你写。PecoFence 自己不联网，联网的是你的脚本' }, { gap: 1 });
        // two buttons: the text above them (the page's .actions wrap under a long text)
        r.scriptRow = SM.card(g, { t: '桌面上还没有脚本栅栏', s: '脚本最多运行 10 秒，不弹出窗口；运行间隔在栅栏的右键菜单里改', h: 64, indent: 22, textW: g.cardW - 34 - 12, gap: 0 });
        r.scriptRow.t.style.top = '7px';
        r.scriptRow.s.style.top = '22px';
        r.newScript = SM.button(r.scriptRow, '新建脚本栅栏…', 64, { accent: true, top: 37 });
        r.example = SM.button(r.scriptRow, '添加示例…', 64, { right: 10 + r.newScript.w + 6, top: 37 });
        g.contentH = SM.stack(g) + 10;
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
        r.thumb = el('div', 'xs-thumb', w.main);

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

        r.menu = menu(S, EXAMPLES, 150);
        r.menu.n.classList.add('tl-menu');       // a popup menu is above the settings window

        r.ptr = pointer(S);
        taskbar(root);

        // targets: the buttons (the script ones once the page is scrolled down). SM.at misses a
        // card's CSS left, as in title-hover.js: measure the real offset once.
        const pg = r.page.g;
        const scale = root.getBoundingClientRect().width / 800 || 1;
        const dx = (r.page.newNote.n.getBoundingClientRect().left - root.getBoundingClientRect().left) / scale - SM.box(r.page.newNote).x;
        const fix = q => ({ ...q, x: q.x + dx });
        const note = fix(SM.at(r.page.newNote));
        const clock = fix(SM.at(r.page.newClock));
        SCROLL = Math.max(0, pg.contentH - r.w.mainH);
        pg.scroll = SCROLL;
        const ex = fix(SM.at(r.page.example));
        pg.scroll = 0;
        // the menu opens at the pointer, above it when it would run under the taskbar
        // (TrackPopupMenu flips it)
        const mh = r.menu.h;
        MENU = { x: ex.x, y: ex.y + mh > 450 - 28 ? ex.y - mh : ex.y, a: EXAMPLE + .02, b: NEW_SCRIPT };
        const pick = { x: Math.min(ex.x, 800 - 4 - r.menu.w) + 44, y: MENU.y + rowY(r.menu, '天气') };
        // the first to-do box (note body: 14 px in, line 1)
        const box = { x: NOTE.x + 14 + 6.5, y: NOTE.y + 32 + 2 + 8.5 };

        PATH = [
            [0, 330, 360], [0.3, 330, 360],
            [0.85, note.x, note.y], [1.3, note.x, note.y],          // 新建便签
            [2.0, 300, 330], [2.9, 300, 330],                       // aside while the note is typed
            [3.25, 92, 330], [3.6, 92, 330],                        // a click elsewhere saves
            [4.0, box.x, box.y], [4.4, box.x, box.y],               // tick the first to-do
            [4.9, clock.x, clock.y], [5.3, clock.x, clock.y],       // 新建时钟
            [5.65, 330, 300], [6.6, 330, 300],                      // wheel down to 脚本栅栏
            [7.1, ex.x, ex.y], [7.45, ex.x, ex.y],                  // 添加示例…
            [7.8, pick.x, pick.y], [8.1, pick.x, pick.y],           // 天气 in its menu
            [8.9, 519, 196], [D, 519, 196],                       // rest between the window and the fences
        ];
        CLICKS = [{ t: NEW_NOTE }, { t: SAVE }, { t: TICK }, { t: NEW_CLOCK }, { t: EXAMPLE }, { t: NEW_SCRIPT }];
        CHIPS = [[3.15, 3.55, '点别处保存', 16, -30], [3.95, 4.3, '点方框', 16, 18], [SCROLL_A - .05, SCROLL_B - .1, '向下滚动', -96, 14]];
        return r;
    }

    // The page's scrollbar thumb (title-hover.js): 12 px gutter, 4 px border, 40 px minimum.
    function drawThumb(n, pg, viewH) {
        const h = Math.max(40, viewH * viewH / pg.contentH);
        const y = (viewH - h) * pg.scroll / Math.max(1, pg.contentH - viewH);
        n.style.height = h.toFixed(1) + 'px';
        n.style.transform = `translateY(${y.toFixed(2)}px)`;
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
        const pg = r.page.g, P = r.page;

        // settings: the page scrolls down to 脚本栅栏; switches follow the buttons
        pg.scroll = SCROLL * ramp(t, SCROLL_A, SCROLL_B, E.out);
        SM.drawNav(r.w, t, [[0, 'tools']], [], p);
        drawThumb(r.thumb, pg, r.w.mainH);
        SM.drawToggle(P.notes, t >= NEW_NOTE + .03);
        SM.drawToggle(P.clock, t >= NEW_CLOCK + .03);
        SM.drawToggle(P.scripts, t >= NEW_SCRIPT + .03);
        P.secCard.n.style.opacity = t >= NEW_CLOCK + .03 ? 1 : .55;   // 显示秒 follows 时钟与日期
        P.evCard.n.style.opacity = .55;                                // 搜索整台电脑 stays off
        text(P.noteRow.t, t >= NEW_NOTE + .03 ? T('桌面上有 {0} 个便签').replace('{0}', '1') : T('桌面上还没有便签'));
        text(P.clockRow.t, t >= NEW_CLOCK + .03 ? T('桌面上有 {0} 个时钟').replace('{0}', '1') : T('桌面上还没有时钟'));
        text(P.scriptRow.t, t >= NEW_SCRIPT + .03 ? T('桌面上有 {0} 个脚本栅栏').replace('{0}', '1') : T('桌面上还没有脚本栅栏'));
        SM.drawButton(P.newNote, p, t >= NEW_NOTE - .08 && t < NEW_NOTE + .08);
        SM.drawButton(P.newClock, p, t >= NEW_CLOCK - .08 && t < NEW_CLOCK + .08);
        SM.drawButton(P.example, p, t >= EXAMPLE - .08 && t < EXAMPLE + .08);
        SM.drawButton(P.newScript, p);

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
        appear(r.C, t, NEW_CLOCK + .05);

        // 「添加示例…」's menu: open until 「天气」 is clicked
        drawMenu(r.menu, t, MENU.x, MENU.y, MENU.a, MENU.b, p);

        // the weather fence: running first, then the lines it printed
        appear(r.W, t, NEW_SCRIPT + .05);
        const done = t >= OUTPUT;
        show(r.out, done);
        if (done) r.out.style.opacity = ramp(t, OUTPUT, OUTPUT + .12, E.lin);
        const time = /^en/.test(lang()) ? '10:24 AM' : '10:24';
        text(r.foot, done ? `${SCRIPT_NAME} · ${T('{0} 更新').replace('{0}', time)}` : T('{0} · 正在运行…').replace('{0}', SCRIPT_NAME));

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at });
    }

    PM.register('tools', { duration: D, poster: 9.2, build, render });
})();
