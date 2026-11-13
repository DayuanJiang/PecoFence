// 3.4 自由摆放桌面图标 (lesson id free-place): the icons outside every fence belong to 「桌面」, which a
// new installation arranges like the desktop with 自由摆放 on: no title, no plate, cells counted
// from the top-left corner of the screen (FEATURES.md 「桌面」栏自由摆放). An icon dragged to empty
// desktop stays in the cell where it is let go; a new file that no rule takes goes into the
// first free cell, column by column; unticking 自由摆放 in an icon's menu lines the icons up in
// the order of 「桌面」 (the new file last), ticking it again puts each one back in its cell.
(() => {
    'use strict';
    const { SH, E, ramp, lerp, pathAt, el, put, show, deskCell, local, icon, fence, menu, drawMenu, rowY,
        pointer, drawPointer, drawChip, taskbar, loopFade, CELL_H } = PM;

    const D = 13.2;
    const FENCES = [
        { x: 530, y: 24, w: 246, h: 124, title: '程序',
            items: [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']] },
        { x: 530, y: 160, w: 246, h: 204, title: '文件与文档',
            items: [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['pdf', '发票.pdf'], ['docx', '方案.docx']] },
    ];
    // id: [icon, label]; ids are ASCII so translations never touch them
    const ITEMS = {
        budget: ['xlsx', '预算.xlsx'],
        todo: ['txt', '待办.txt'],
        poster: ['assets/icons/icons-poster.png', '海报.png'],
        draft: ['docx', '草稿.docx'],
        log: ['txt', '安装日志.log'],      // the new file
    };
    const ROWS = Math.floor((SH - 28 - 4) / CELL_H);      // cells of one desktop column
    // 「桌面」 in order: by name (lowercase, code point order), one column after the other.
    const order = list => [...list].sort((a, b) => {
        const x = ITEMS[a][1].toLowerCase(), y = ITEMS[b][1].toLowerCase();
        return x < y ? -1 : x > y ? 1 : 0;
    });
    const inOrder = list => Object.fromEntries(order(list).map((k, i) => [k, [Math.floor(i / ROWS), i % ROWS]]));

    const DRAG1 = { press: 1.15, lift: 1.22, drop: 2.55, item: 'budget', to: [3, 2] };
    const DRAG2 = { press: 3.75, lift: 3.82, drop: 5.0, item: 'todo', to: [4, 2] };
    const NEW_AT = 5.8;
    const RC1 = 7.55, PICK1 = 8.5, OFF_AT = 8.53;       // right-click, 自由摆放 unticked
    const RC2 = 10.0, PICK2 = 10.95, ON_AT = 10.98;     // ticked again

    // The cells each icon has while 自由摆放 is on: after the two drags, then with the new file
    // in the first free cell counted from the top left.
    const START = inOrder(['budget', 'todo', 'poster', 'draft']);
    const PLACED = { ...START, [DRAG1.item]: DRAG1.to, [DRAG2.item]: DRAG2.to };
    const firstFree = cells => {
        for (let c = 0; ; c++) {
            for (let r = 0; r < ROWS; r++) {
                if (!Object.values(cells).some(([x, y]) => x === c && y === r)) return [c, r];
            }
        }
    };
    PLACED.log = firstFree(PLACED);
    // 自由摆放 off: the order of 「桌面」, which is 手动: the icons it had stand by name, a file
    // that came later stands after them (state.rs place_last)
    const LINED = Object.fromEntries([...order(['budget', 'todo', 'poster', 'draft']), 'log']
        .map((k, i) => [k, [Math.floor(i / ROWS), i % ROWS]]));
    const centre = ([c, r]) => {
        const s = deskCell(c, r);
        return [s.x + 39, s.y + 25];
    };

    const MENU_ITEMS = [{ t: '打开', key: 'Enter' }, '打开文件所在位置', { t: '重命名', key: 'F2' }, { t: '删除', key: 'Delete' }, '-',
        { t: '移动到栅栏', sub: 1 }, '以后把 {0} 文件都放到这里'.replace('{0}', '.xlsx'), { t: '移出栅栏（放回“桌面”）', dis: 1 }, '-',
        { t: '像桌面一样排列', check: 1 }, { t: '自由摆放', check: 1 }, '栅栏选项…',
        // Explorer's own menu for the file follows (shortened here).
        '-', '剪切(T)', '复制(C)', '-', '属性(R)'];

    let PATH, CLICKS, CHIPS, NOTES, MENUS;
    const at = t => pathAt(t, PATH);
    function plan(m) {
        const a = centre(DRAG1.to), b = centre(DRAG2.to), lined = centre(LINED.budget);
        // Windows moves a menu up when it would not fit above the taskbar.
        const box = ([x, y]) => ({ x: x + 2, y: Math.min(y, SH - 28 - m.h - 2) });
        MENUS = [{ ...box(a), a: RC1 + .05, b: PICK1 + .03 }, { ...box(lined), a: RC2 + .05, b: PICK2 + .03 }];
        const pick = i => [MENUS[i].x + 86, MENUS[i].y + rowY(m, '自由摆放')];
        PATH = [
            [0, 440, 388], [0.3, 440, 388],
            [1.0, ...centre(START.budget)], [DRAG1.press, ...centre(START.budget)],     // press 预算.xlsx
            [DRAG1.drop - .15, ...a], [3.1, ...a],                                      // empty desktop, release
            [3.6, ...centre(START.todo)], [DRAG2.press, ...centre(START.todo)],         // press 待办.txt
            [DRAG2.drop - .15, ...b], [5.25, ...b],                                     // beside it, release
            [5.75, 440, 330], [7.0, 440, 330],                                          // aside: the new file
            [7.45, ...a], [RC1 + .2, ...a],                                             // right-click 预算.xlsx
            [8.3, ...pick(0)], [8.7, ...pick(0)],                                       // 自由摆放 (ticked)
            [9.85, ...lined], [RC2 + .2, ...lined],                                     // right-click it in the column
            [10.75, ...pick(1)], [11.15, ...pick(1)],                                   // 自由摆放 again
            [11.9, 440, 330], [D, 440, 330],
        ];
        CLICKS = [{ t: RC1, right: true }, { t: PICK1 }, { t: RC2, right: true }, { t: PICK2 }];
        CHIPS = [[DRAG1.press, 1.8, '按住拖动', 16, -30], [DRAG2.press, 4.35, '再拖一个', 16, -30]];
        const cellA = deskCell(...DRAG1.to), cellNew = deskCell(...PLACED.log);
        NOTES = [
            // left of the pointer: the menu opens to its right and is moved up to fit
            [RC1 - .1, RC1 + .35, '右键', a[0] - 14, a[1] - 14, 'abs', 'right'],
            [DRAG1.drop + .15, DRAG1.drop + 1.1, '停在松开的这一格', cellA.x - 8, cellA.y + 84, 'abs'],
            [NEW_AT + .05, NEW_AT + 1.45, '新文件：第一个空格子', cellNew.x + 84, cellNew.y + 26, 'abs'],
            [OFF_AT + .25, RC2 - .25, '关掉：按顺序排好', 96, 180, 'abs'],
            [ON_AT + .25, ON_AT + 1.75, '再打开：回到你摆的位置', cellA.x - 8, cellA.y + 84, 'abs'],
        ];
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        FENCES.forEach(f => {
            const o = fence(S, { ...f });
            f.items.forEach(([k, l], i) => {
                const s = local(i, 3);
                put(icon(o.body, k, l), s.x, s.y);
            });
        });
        r.icons = {};
        for (const k of Object.keys(ITEMS)) {
            r.icons[k] = icon(S, ...ITEMS[k]);
            r.icons[k].classList.add('desk');
        }
        r.ghosts = [DRAG1, DRAG2].map(d => {
            const n = icon(S, ...ITEMS[d.item]);
            n.classList.add('desk', 'ghost');
            return n;
        });
        r.pulse = el('div', 'ring pulse', S);
        r.note = el('div', 'chip', S);
        r.menu = menu(S, MENU_ITEMS, 214);
        r.menu.rows[0].r.style.fontWeight = '600';      // 打开 is the default item
        r.tick = r.menu.rows.find(o => o.t === '自由摆放').r;
        r.ptr = pointer(S);
        taskbar(root);
        plan(r.menu);
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);

        // where each icon stands: its cell while 自由摆放 is on, its place in the order while off
        // (250 ms slides both ways)
        const off = ramp(t, OFF_AT, OFF_AT + .25, E.p2p) * (1 - ramp(t, ON_AT, ON_AT + .25, E.p2p));
        for (const k of Object.keys(ITEMS)) {
            const n = r.icons[k];
            let cell = START[k] || PLACED[k], x = null, y = null, o = 1;
            for (const d of [DRAG1, DRAG2]) {
                if (d.item !== k || t < d.drop) continue;
                // let go: the icon glides from the pointer to the cell (167 ms)
                cell = d.to;
                const from = at(d.drop), q = ramp(t, d.drop, d.drop + .167);
                const to = deskCell(...cell);
                x = lerp(from.x - 39, to.x, q);
                y = lerp(from.y - 25, to.y, q);
            }
            if (k === 'log') {
                if (!show(n, t >= NEW_AT)) continue;
                o = ramp(t, NEW_AT, NEW_AT + .167, E.lin);
            }
            if (x == null) ({ x, y } = deskCell(...cell));
            const lined = deskCell(...LINED[k]);
            put(n, lerp(x, lined.x, off), lerp(y, lined.y, off), o);
            const dragged = [DRAG1, DRAG2].some(d => d.item === k && t >= d.press && t < d.drop);
            n.classList.toggle('sel', dragged || (k === 'budget' && ((t >= RC1 && t < OFF_AT + .6) || (t >= RC2 && t < ON_AT + .6))));
        }
        [DRAG1, DRAG2].forEach((d, i) => {
            if (show(r.ghosts[i], t >= d.lift && t < d.drop)) put(r.ghosts[i], p.x - 39, p.y - 25, .72);
        });

        // the new file: a pulse around it
        if (show(r.pulse, t >= NEW_AT && t < NEW_AT + .8)) {
            const s = deskCell(...PLACED.log), q = (t - NEW_AT) / .8;
            put(r.pulse, s.x + 39, s.y + 24, (1 - q) * .9, ` scale(${.6 + E.out(q) * 1.1})`);
        }
        drawChip(r.note, t, NOTES, p);

        // the icon's menu: 自由摆放 is ticked the first time, unticked the second
        const second = t >= MENUS[1].a;
        const m = MENUS[second ? 1 : 0];
        r.tick.classList.toggle('check', !second);
        drawMenu(r.menu, t, m.x, m.y, m.a, m.b, p);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: [[DRAG1.press, DRAG1.drop], [DRAG2.press, DRAG2.drop]], chips: CHIPS, at });
    }

    PM.register('free-place', { duration: D, poster: 6.6, build, render });
})();
