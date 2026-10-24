// 1.2 新建栅栏: marquee on the empty desktop → 在此新建栅栏 → drag an icon in → rename.
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, firstWhen, el, put, show, text, slot, icon, fence, menu, drawMenu,
        pointer, drawPointer, drawKeys, taskbar, loopFade, T } = PM;

    const D = 11.6;
    const PATH = [
        [0, 560, 400], [0.3, 560, 400],
        [1.0, 318, 56], [1.2, 318, 56],          // marquee start
        [2.4, 642, 236], [2.7, 642, 236],        // marquee end, release
        [3.1, 765, 257], [3.4, 765, 257],        // 在此新建栅栏
        [4.5, 151, 244], [4.8, 151, 244],        // grab 周报.docx
        [6.0, 400, 205], [6.7, 400, 205],        // drop into the new fence
        [7.3, 372, 74], [7.6, 372, 74],          // right-click its title
        [8.0, 430, 121], [8.35, 430, 121],       // 重命名…
        [8.9, 548, 150], [D, 548, 150],
    ];
    const CLICKS = [{ t: 3.3 }, { t: 7.45, right: true }, { t: 8.2 }];
    const PRESSES = [[1.2, 2.5], [4.8, 6.1]];
    const CHIPS = [[1.2, 2.4, '按住拖动'], [4.8, 6.0, '拖进新栅栏', 16, -30], [7.3, 7.58, '右键标题栏', -30, -48], [8.7, 9.5, '输入名字']];
    const KEYS = [{ a: 9.35, b: 10.1, keys: ['Enter'], down: 9.65 }];
    const MENU1 = { x: 644, y: 238, a: 2.55, b: 3.36 };
    const MENU2 = { x: 374, y: 76, a: 7.5, b: 8.24 };
    const DROP_AT = 6.1;

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.prog = fence(S, { x: 28, y: 40, w: 246, h: 124, title: '程序' });
        r.desk = fence(S, { x: 28, y: 184, w: 246, h: 204, title: '桌面' });
        r.neu = fence(S, { x: 318, y: 56, w: 324, h: 180, title: '新栅栏' });

        [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']].forEach(([k, l], i) => {
            const s = slot(r.prog, i, 3);
            put(icon(S, k, l), s.x, s.y);
        });
        r.deskIcons = [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['txt', '待办.txt'], ['folder', '素材']]
            .map(([k, l]) => icon(S, k, l));
        r.moved = icon(S, 'docx', '周报.docx');

        r.marquee = el('div', 'marquee', S);
        r.ghost = icon(S, 'docx', '周报.docx');
        r.ghost.classList.add('ghost');
        r.badge = el('div', 'badge', S);
        r.badge.innerHTML = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M1.5 6h8M6.5 2.8 9.7 6 6.5 9.2" ' +
            'fill="none" stroke="#1663c7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>移动到 <b>新栅栏</b>';
        r.menu1 = menu(S, ['在此新建栅栏'], 148);
        r.menu2 = menu(S, ['收起', '重命名…', '-', { t: '视图', sub: 1 }, { t: '排序方式', sub: 1 }, { t: '新建', sub: 1 },
            { t: '粘贴', key: 'Ctrl+V', dis: 1 }, { t: '标题栏', sub: 1 }, { t: '标签页', sub: 1 }, '锁定位置和大小', '-', '栅栏选项…', '新建栅栏',
            { t: '在桌面显示文件夹', sub: 1 }, '删除栅栏'], 196);
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);

        r.enterNeu = firstWhen(4.8, DROP_AT, t => inside(pathAt(t, PATH), r.neu));
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);

        // 1. marquee, drawn by PecoFence while the desktop icons are hidden
        if (show(r.marquee, t >= 1.22 && t < 2.58)) {
            const x0 = 318, y0 = 56, x1 = Math.max(x0, p.x), y1 = Math.max(y0, p.y);
            r.marquee.style.width = (x1 - x0) + 'px';
            r.marquee.style.height = (y1 - y0) + 'px';
            put(r.marquee, x0, y0, 1 - ramp(t, 2.5, 2.58, E.lin));
        }

        // 2. one-item menu, then the fence fades in where the box was
        drawMenu(r.menu1, t, MENU1.x, MENU1.y, MENU1.a, MENU1.b, p);
        if (show(r.neu.n, t >= 3.4)) {
            const o = ramp(t, 3.4, 3.62);
            put(r.neu.n, r.neu.x, r.neu.y, o, ` scale(${lerp(.965, 1, o)})`);
        }
        r.neu.hint.style.opacity = 1 - ramp(t, DROP_AT, DROP_AT + .1, E.lin);
        // render.rs empty_text: an empty fence under a drag says 松开即可放入
        text(r.neu.hint, t >= r.enterNeu && t < DROP_AT + .1 ? T('松开即可放入') : T('将项目拖到此处'));

        // 3. drag 周报.docx from 桌面 into the new fence
        const dragging = t >= 4.8 && t < DROP_AT;
        r.deskIcons.forEach((n, i) => {
            const a = slot(r.desk, i, 3);
            if (i === 1) {
                put(n, a.x, a.y, 1 - ramp(t, DROP_AT, DROP_AT + .083, E.lin));
                n.classList.toggle('sel', dragging);
            } else if (i > 1) {
                const b = slot(r.desk, i - 1, 3), q = ramp(t, DROP_AT + .05, DROP_AT + .3, E.p2p);
                put(n, lerp(a.x, b.x, q), lerp(a.y, b.y, q));
            } else {
                put(n, a.x, a.y);
            }
        });
        if (show(r.moved, t >= DROP_AT)) {
            const s = slot(r.neu, 0, 4);
            put(r.moved, s.x, s.y, ramp(t, DROP_AT + .02, DROP_AT + .19, E.lin));
        }
        if (show(r.ghost, t >= 4.92 && t < DROP_AT)) put(r.ghost, p.x - 39, p.y - 24, .72);
        const over = t >= r.enterNeu && t < DROP_AT;
        r.neu.drop.style.opacity = over ? ramp(t, r.enterNeu, r.enterNeu + .083, E.lin)
            : t >= DROP_AT ? 1 - ramp(t, DROP_AT, DROP_AT + .167, E.lin) : 0;
        if (show(r.badge, over)) put(r.badge, p.x + 2, p.y + 34);

        // 4. right-click the title → 重命名… → type → Enter
        drawMenu(r.menu2, t, MENU2.x, MENU2.y, MENU2.a, MENU2.b, p);
        const editing = t >= 8.3 && t < 9.7;
        show(r.neu.edit, editing);
        show(r.neu.name, !editing);
        if (editing) {
            const caret = Math.floor((t - 8.3) / .5) % 2 === 0 ? '<span class="caret"></span>' : '';
            const html = t < 8.85 ? '<span class="sel">新栅栏</span>'
                : (t < 9.05 ? '工' : '工作') + caret;
            if (r.neu.edit.innerHTML !== html) r.neu.edit.innerHTML = html;
        }
        text(r.neu.name, t >= 9.7 ? '工作' : T('新栅栏'));

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('create', { duration: D, poster: 6.4, build, render });
})();
