// 4.7 每个虚拟桌面一套栅栏 (lesson id vdesk): two Windows virtual desktops, 工作 and 游戏. The fence
// menu's 在哪个虚拟桌面显示 lists them by their Task View names (vdesktops.rs); picking 游戏 hides
// the fence on the current desktop and a notification says where it went. Win+Ctrl+→ slides to
// the other desktop (Windows moves the whole desktop sideways and shows the desktop's name above
// the taskbar): there the fence shows with the Task View glyph before its title, the fence that
// belongs to 工作 does not, and a fence without a desktop of its own shows on both.
(() => {
    'use strict';
    const { SW, SH, E, ramp, pathAt, el, put, show, text, local, icon, fence, menu, drawMenu, rowY, pointer,
        drawPointer, drawChip, drawKeys, taskbar, loopFade, T } = PM;

    const D = 10.4;
    const G0 = { x: 130, y: 40, w: 246, h: 124, title: '游戏',
        items: [['exe', '纸牌'], ['exe', '象棋'], ['url', '游戏攻略']] };
    const W0 = { x: 130, y: 196, w: 246, h: 124, title: '工作',
        items: [['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['pdf', '合同.pdf']] };
    const P0 = { x: 530, y: 40, w: 246, h: 124, title: '程序',
        items: [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']] };

    const RC = 1.1, SUB_AT = 1.95, PICK = 2.85, SENT = 2.88;       // right-click, submenu, 游戏 picked
    const NOTE_AT = 3.15, NOTE_END = 8.6;
    const GO = 5.0, BACK = 7.7, SLIDE = .35;                       // Win+Ctrl+→ and Win+Ctrl+←
    // the keys leave before Windows shows the desktop's name in the same place
    const KEYS = [{ a: GO - .5, b: GO + .15, keys: ['Win', 'Ctrl', '→'], down: GO - .08 },
        { a: BACK - .5, b: BACK + .15, keys: ['Win', 'Ctrl', '←'], down: BACK - .08 }];
    const slide = bez => t => bez(Math.min(1, Math.max(0, t)));
    const ease = slide(PM.bezier(.3, 0, .1, 1));

    const MENU_ITEMS = ['收起', '重命名…', '-', { t: '视图', sub: 1 }, { t: '排序方式', sub: 1 }, { t: '新建', sub: 1 },
        { t: '粘贴', key: 'Ctrl+V', dis: 1 }, { t: '标签页', sub: 1 }, '-', '适应内容大小', '锁定位置和大小',
        { t: '在哪个虚拟桌面显示', sub: 1 }, '栅栏选项…', '删除栅栏', '-', { t: '添加到桌面', sub: 1 }];
    const GRAB = { x: G0.x + 104, y: G0.y + 16 };

    // Task View glyph (E7C4): two overlapping windows.
    const GLYPH = '<svg viewBox="0 0 13 12" aria-hidden="true"><rect x="3.6" y=".7" width="8.7" height="7.2" rx="1.4" fill="none" ' +
        'stroke="currentColor" stroke-width="1.05"/><path d="M2.6 3.3H2A1.3 1.3 0 0 0 .7 4.6V10A1.3 1.3 0 0 0 2 11.3h6.4A1.3 1.3 0 0 0 9.7 10v-.6" ' +
        'fill="none" stroke="currentColor" stroke-width="1.05" stroke-linecap="round"/></svg>';

    // One virtual desktop: wallpaper, its fences, the taskbar (Windows slides all of it).
    function desktop(parent, fences) {
        const n = el('div', 'scene', parent);
        el('div', 'wall', n);
        const out = fences.map(([f, bound]) => {
            const o = fence(n, { ...f });
            f.items.forEach(([k, l], i) => {
                const s = local(i, 3);
                put(icon(o.body, k, l), s.x, s.y);
            });
            if (bound) {        // shows on this desktop only: the glyph, then the name one slot on
                el('div', 'vd-glyph', o.bar).innerHTML = GLYPH;
                o.name.style.paddingLeft = '20px';
            }
            return o;
        });
        taskbar(n);
        return { n, fences: out };
    }

    let PATH, CLICKS, CHIPS, NOTES, MENU, SUB;
    const at = t => pathAt(t, PATH);

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.work = desktop(S, [[G0, false], [W0, true], [P0, false]]);
        r.games = desktop(S, [[G0, true], [P0, false]]);
        r.sent = r.work.fences[0];                      // 游戏 while it still shows on 工作

        r.menu = menu(S, MENU_ITEMS, 196);
        r.sub = menu(S, [{ t: '所有虚拟桌面', check: 1 }, '-', T('{0}（当前）').replace('{0}', T('工作')), '游戏'], 128);
        r.toast = el('div', 'wtoast', S);
        const head = el('div', 'wtoast-head', r.toast);
        const mark = el('img', null, head);
        mark.src = 'assets/mark.svg';
        mark.alt = '';
        el('span', null, head, 'PecoFence');
        const body = el('div', 'wtoast-body', r.toast);
        el('div', 'wtoast-info', body, 'i');
        const words = el('div', null, body);
        el('b', null, words, 'PecoFence');
        el('p', null, words, T('栅栏“{0}”现在只在“{1}”上显示，切换到那个虚拟桌面就能看到它。').replace('{0}', T('游戏')).replace('{1}', T('游戏')));
        r.name = el('div', 'vd-name', S);
        r.note = el('div', 'chip', S);
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);

        // Windows moves a menu up when it would not fit above the taskbar; the submenu opens
        // beside its row.
        MENU = { x: GRAB.x + 2, y: Math.min(GRAB.y + 2, SH - 28 - r.menu.h - 2), a: RC + .05, b: PICK + .03 };
        const row = rowY(r.menu, '在哪个虚拟桌面显示');
        SUB = { x: MENU.x + r.menu.w - 4, y: MENU.y + row - 17, a: SUB_AT, b: PICK + .03 };
        const pick = [SUB.x + 46, SUB.y + rowY(r.sub, '游戏')];
        PATH = [
            [0, 470, 380], [0.3, 470, 380],
            [1.0, GRAB.x, GRAB.y], [RC + .2, GRAB.x, GRAB.y],              // right-click the title of 游戏
            [1.8, MENU.x + 96, MENU.y + row], [2.1, MENU.x + 150, MENU.y + row],   // 在哪个虚拟桌面显示
            [2.6, ...pick], [PICK + .2, ...pick],                          // 游戏
            [3.7, 470, 300], [D, 470, 300],
        ];
        CLICKS = [{ t: RC, right: true }, { t: PICK }];
        CHIPS = [[RC - .12, RC + .3, '右键标题栏', -30, -48]];
        NOTES = [
            [GO + SLIDE + .25, BACK - .55, '标题前的小图标：只在这个桌面显示', G0.x, G0.y + G0.h + 8, 'abs'],
            [BACK + SLIDE + .25, BACK + SLIDE + 1.75, '「游戏」栅栏不在这个桌面了', G0.x, G0.y + 44, 'abs'],
        ];
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);

        // which desktop shows: 0 = 工作, 1 = 游戏; both slide as one strip
        const q = ease((t - GO) / SLIDE) - ease((t - BACK) / SLIDE);
        put(r.work.n, -SW * q, 0);
        if (show(r.games.n, q > 0)) put(r.games.n, SW * (1 - q), 0);

        // 游戏 leaves 工作 once it is sent to its own desktop (167 ms fade)
        const left = ramp(t, SENT, SENT + .167, E.lin);
        if (show(r.sent.n, left < 1)) r.sent.n.style.opacity = 1 - left;

        drawMenu(r.menu, t, MENU.x, MENU.y, MENU.a, MENU.b, p);
        // the row of an open submenu stays lit
        if (t >= SUB_AT && t < MENU.b) r.menu.rows.find(o => o.t === '在哪个虚拟桌面显示').r.classList.add('hot');
        drawMenu(r.sub, t, SUB.x, SUB.y, SUB.a, SUB.b, p);

        // Windows' notification, above the right end of the taskbar
        if (show(r.toast, t >= NOTE_AT && t < NOTE_END + .2)) {
            const o = ramp(t, NOTE_AT, NOTE_AT + .2) * (1 - ramp(t, NOTE_END, NOTE_END + .2, E.lin));
            put(r.toast, SW - 12 - r.toast.offsetWidth + 24 * (1 - ramp(t, NOTE_AT, NOTE_AT + .2)), SH - 28 - 10 - r.toast.offsetHeight, o);
        }
        // the name of the desktop just switched to, above the middle of the taskbar
        const shown = [[GO + SLIDE + .08, GO + SLIDE + 1.6, '游戏'], [BACK + SLIDE + .08, BACK + SLIDE + 1.6, '工作']]
            .find(([a, b]) => t >= a && t < b + .2);
        if (show(r.name, !!shown)) {
            text(r.name, T(shown[2]));
            const o = ramp(t, shown[0], shown[0] + .12, E.lin) * (1 - ramp(t, shown[1], shown[1] + .2, E.lin));
            put(r.name, (SW - r.name.offsetWidth) / 2, SH - 28 - 8 - r.name.offsetHeight, o);
        }

        drawChip(r.note, t, NOTES, p);
        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at });
    }

    PM.register('vdesk', { duration: D, poster: 6.4, build, render });
})();
