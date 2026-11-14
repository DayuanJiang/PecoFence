// 3.4 在桌面显示文件夹 (lesson id portal): right-click a folder → 在桌面显示此文件夹 → a new fence (4 columns × 260, right
// of the source fence) lists the folder's real contents, sorted by name, with a folder glyph
// before the title; double-click a subfolder to go in (title = its name, up button at the far
// left), click up to come back; a file dragged in from the desktop (the loose icons of 「桌面」 at
// the top-left corner) is really moved into the folder.
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, firstWhen, spans, spanFade, el, put, show, text, local, slot, deskCell,
        icon, fence, menu, drawMenu, rowY, pointer, drawPointer, taskbar, loopFade } = PM;

    const D = 11.6;
    const A0 = { x: 120, y: 24, w: 246, h: 124, title: '文件夹' };
    // place_new_fence(4 columns, 260 DIP, near = the source fence): to its right, 12 DIP apart.
    const P0 = { x: A0.x + A0.w + 12, y: A0.y, w: 324, h: 260, title: '工作资料' };
    const PC = 4;

    // Portals sort by name (state.rs natural_cmp: lower-cased, code-point order, digit runs as
    // numbers); folders are not put first.
    const natural = (a, b) => {
        const x = a.toLowerCase(), y = b.toLowerCase();
        const re = /(\d+)|(\D)/g;
        const ta = [...x.matchAll(re)], tb = [...y.matchAll(re)];
        for (let i = 0; i < Math.min(ta.length, tb.length); i++) {
            const [ma, mb] = [ta[i], tb[i]];
            if (ma[1] && mb[1]) {
                if (+ma[1] !== +mb[1]) return +ma[1] - +mb[1];
            } else if (ma[0] !== mb[0]) {
                return ma[0] < mb[0] ? -1 : 1;
            }
        }
        return ta.length - tb.length;
    };
    const byName = list => [...list].sort((a, b) => natural(a[1], b[1]));
    const ROOT = byName([['docx', '会议纪要.docx'], ['folder', '合同'], ['peek-thumb-terra', '封面设计.png'],
        ['folder', '设计稿'], ['pdf', '需求说明.pdf'], ['xlsx', '项目计划.xlsx']]);
    const MOVED = ['xlsx', '报价单.xlsx'];
    // 「桌面」 of a new installation: loose icons in a column from the top-left corner
    const DESK = [['txt', '待办.txt'], MOVED, ['pdf', '发票.pdf'], ['docx', '草稿.docx']];
    const PICK = DESK.indexOf(MOVED);
    const AFTER = byName([...ROOT, MOVED]);
    const SUB = byName([['pdf', '合同 A.pdf'], ['pdf', '合同 B.pdf'], ['pdf', '签字页.pdf'], ['docx', '补充协议.docx']]);
    const SUB_DIR = ROOT.findIndex(([, l]) => l === '合同');

    const MENU_ITEMS = [{ t: '打开', key: 'Enter' }, '打开文件所在位置', { t: '重命名', key: 'F2' },
        { t: '删除', key: 'Delete' }, '-', '在桌面显示此文件夹', { t: '移动到栅栏', sub: 1 }, '移出栅栏（放回“桌面”）',
        // Explorer's own menu for the folder follows (shortened here).
        '-', { t: '发送到(N)', sub: 1 }, '-', '剪切(T)', '复制(C)', '-', '属性(R)'];
    const MENU = { x: A0.x + 47, y: 86, a: 1.15, b: 2.04 };
    const OPEN_AT = 2.15;                           // the portal fence fades in
    const DBL = [4.08, 4.25], NAV_IN = 4.28;        // double-click 合同
    const UP_PRESS = [6.15, 6.27], NAV_UP = 6.27;   // the up button acts on release
    const DRAG = [7.35, 8.85], DROP = 8.85;
    const ADD = DROP + .12;                         // the folder watcher sees the new file

    const at = (f, i, c) => {
        const s = slot(f, i, c);
        return { x: s.x + 39, y: s.y + 25 };
    };
    const FOLDER_PT = at(A0, 0, 3);
    const SUBDIR_PT = at(P0, SUB_DIR, PC);
    const UP = { x: P0.x + 6, y: P0.y + 2, w: 28, h: 28 };
    const UP_PT = { x: UP.x + 22, y: UP.y + 24 };       // lower right, so the arrow glyph stays visible
    const FILE_PT = { x: deskCell(0, PICK).x + 39, y: deskCell(0, PICK).y + 25 };
    const DROP_PT = { x: P0.x + 218, y: 246 };

    let PATH;
    function plan(m) {
        const pick = { x: MENU.x + 82, y: MENU.y + rowY(m, '在桌面显示此文件夹') };
        PATH = [
            [0, 560, 380], [0.3, 560, 380],
            [1.0, FOLDER_PT.x, FOLDER_PT.y], [1.4, FOLDER_PT.x, FOLDER_PT.y],   // right-click 工作资料
            [1.85, pick.x, pick.y], [2.4, pick.x, pick.y],                        // 在桌面显示此文件夹
            [3.0, 470, 340], [3.3, 470, 340],
            [3.95, SUBDIR_PT.x, SUBDIR_PT.y], [4.5, SUBDIR_PT.x, SUBDIR_PT.y],   // double-click 合同
            [5.15, UP_PT.x, UP_PT.y], [6.4, UP_PT.x, UP_PT.y],                   // the up button
            [6.95, FILE_PT.x, FILE_PT.y], [DRAG[0], FILE_PT.x, FILE_PT.y],       // grab 报价单.xlsx
            [8.7, DROP_PT.x, DROP_PT.y], [DROP, DROP_PT.x, DROP_PT.y],           // drop into the portal
            [9.6, 650, 360], [D, 650, 360],
        ];
    }
    const CLICKS = [{ t: 1.1, right: true }, { t: 2.0 }, { t: DBL[0] }, { t: DBL[1] }, { t: UP_PRESS[0] }];
    const PRESSES = [DRAG];
    const CHIPS = [[1.0, 1.3, '右键', 30, -48], [3.8, NAV_IN - .2, '双击', -14, -46], [DRAG[0], 8.7, '拖进来', 16, -30],
        [ADD + .1, ADD + 1.5, '已移进这个文件夹', P0.x + 136, 252, 'abs']];

    const FOLDER_GLYPH = '<svg viewBox="0 0 14 14" aria-hidden="true"><path d="M1.5 3.2c0-.4.3-.7.7-.7h3.1l1.4 1.5h5.1' +
        'c.4 0 .7.3.7.7v6.6c0 .4-.3.7-.7.7H2.2a.7.7 0 0 1-.7-.7z" fill="none" stroke="currentColor" stroke-width="1.05" ' +
        'stroke-linejoin="round"/></svg>';
    const UP_GLYPH = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M6 10.8V1.6M1.9 5.6 6 1.4l4.1 4.2" fill="none" ' +
        'stroke="currentColor" stroke-width="1.05" stroke-linecap="round" stroke-linejoin="round"/></svg>';

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.A = fence(S, { ...A0 });
        r.P = fence(S, { ...P0 });
        r.aIcons = [['folder', '工作资料'], ['folder', '照片'], ['folder', '2026 报销']].map(([k, l], i) => {
            const n = icon(r.A.body, k, l);
            const s = local(i, 3);
            put(n, s.x, s.y);
            return n;
        });
        r.desk = DESK.map(([k, l], i) => {
            const n = icon(S, k, l);
            n.classList.add('desk');
            put(n, deskCell(0, i).x, deskCell(0, i).y);
            return n;
        });
        r.root = AFTER.map(([k, l]) => icon(r.P.body, k, l));
        r.sub = SUB.map(([k, l], i) => {
            const n = icon(r.P.body, k, l);
            const s = local(i, PC);
            put(n, s.x, s.y);
            return n;
        });
        r.up = el('div', 'pk-up', r.P.bar);
        r.up.innerHTML = UP_GLYPH;
        r.upFill = el('i', null);
        r.up.prepend(r.upFill);
        r.fold = el('div', 'pk-fold', r.P.bar);
        r.fold.innerHTML = FOLDER_GLYPH;

        r.ghost = icon(S, ...MOVED);
        r.ghost.classList.add('desk', 'ghost');
        r.badge = el('div', 'badge', S);
        r.badge.innerHTML = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M1.5 6h8M6.5 2.8 9.7 6 6.5 9.2" ' +
            'fill="none" stroke="#1663c7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>移动到 <b>工作资料</b>';
        r.pulse = el('div', 'ring pulse', S);
        r.menu = menu(S, MENU_ITEMS, 206);
        r.menu.rows[0].r.style.fontWeight = '600';      // 打开 is the default item
        r.tip = el('div', 'pk-tip', S, '向上');
        r.ptr = pointer(S);
        taskbar(root);

        plan(r.menu);
        const p = t => pathAt(t, PATH);
        r.titleHover = [A0, P0].map(f => spans(D, t => inside(p(t), { x: f.x, y: f.y, w: f.w, h: 32 })
            && (f !== P0 || t >= OPEN_AT)));
        r.upHover = spans(D, t => t >= NAV_IN && t < NAV_UP && inside(p(t), UP));
        r.upRest = firstWhen(NAV_IN, NAV_UP, t => inside(p(t), UP) && t >= 5.15);
        r.enterP = firstWhen(DRAG[0], DROP, t => inside(p(t), P0));
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);

        // the context menu of 工作资料 (right-click selects it; grey once another fence is active)
        drawMenu(r.menu, t, MENU.x, MENU.y, MENU.a, MENU.b, p);
        r.aIcons[0].classList.toggle('sel', t >= 1.1 && t < DBL[0]);
        r.aIcons[0].classList.toggle('pk-dull', t >= DBL[0]);

        // the portal fence: entrance fade (167 ms) + scale from 0.97 (250 ms)
        if (show(r.P.n, t >= OPEN_AT)) {
            const o = ramp(t, OPEN_AT, OPEN_AT + .167, E.lin), s = lerp(.97, 1, ramp(t, OPEN_AT, OPEN_AT + .25));
            put(r.P.n, P0.x, P0.y, o, ` scale(${s.toFixed(4)})`);
        }

        // title rows: hover pill + chevron (83 ms)
        [r.A, r.P].forEach((f, i) => {
            const h = spanFade(t, r.titleHover[i]);
            f.pill.style.opacity = h;
            f.chev.style.opacity = h;
        });

        // navigation: content switches at once; the title shows the folder; up button at the left
        const inSub = t >= NAV_IN && t < NAV_UP;
        text(r.P.name, inSub ? '合同' : '工作资料');
        r.P.name.style.paddingLeft = inSub ? '52px' : '24px';
        r.fold.style.left = (inSub ? 41 : 13) + 'px';
        if (show(r.up, inSub)) {
            const pressed = t >= UP_PRESS[0] && t < UP_PRESS[1];
            r.up.classList.toggle('pressed', pressed);
            r.upFill.style.opacity = pressed ? 1 : spanFade(t, r.upHover);
        }
        r.sub.forEach(n => show(n, inSub));

        // root items; after the drop the new file fades in at its place by name, the rest slide
        const slide = ramp(t, ADD, ADD + .25, E.p2p);
        AFTER.forEach((it, i) => {
            const n = r.root[i];
            if (!show(n, !inSub)) return;
            const isNew = it === MOVED;
            if (isNew) {
                if (show(n, t >= ADD)) {
                    const s = local(i, PC);
                    put(n, s.x, s.y, ramp(t, ADD, ADD + .167, E.lin));
                }
                return;
            }
            const a = local(ROOT.indexOf(it), PC), b = local(i, PC);
            put(n, lerp(a.x, b.x, slide), lerp(a.y, b.y, slide));
            n.classList.toggle('sel', it[1] === '合同' && t >= DBL[0] && t < NAV_IN);
        });
        if (show(r.pulse, t >= ADD && t < ADD + .8)) {
            const s = slot(r.P, AFTER.indexOf(MOVED), PC);
            put(r.pulse, s.x + 39, s.y + 24, (1 - (t - ADD) / .8) * .9, ` scale(${.6 + E.out((t - ADD) / .8) * 1.1})`);
        }

        // the desktop: 报价单.xlsx leaves (83 ms); the other loose icons keep their cells (自由摆放)
        r.desk[PICK].style.opacity = 1 - ramp(t, DROP + .08, DROP + .163, E.lin);
        r.desk[PICK].classList.toggle('sel', t >= DRAG[0]);

        // drag: ghost, drop highlight and Explorer's badge over the portal
        if (show(r.ghost, t >= DRAG[0] + .1 && t < DROP)) put(r.ghost, p.x - 39, p.y - 24, .72);
        const over = t >= r.enterP && t < DROP;
        r.P.drop.style.opacity = over ? ramp(t, r.enterP, r.enterP + .083, E.lin)
            : t >= DROP ? 1 - ramp(t, DROP, DROP + .167, E.lin) : 0;
        if (show(r.badge, over)) put(r.badge, p.x + 2, p.y + 34);

        // tooltip 「向上」 after the system hover time (400 ms) at rest, at cursor + (12, 20) as in
        // tooltip.rs show_tip; a press hides it
        const tipOn = t >= r.upRest + .4 && t < UP_PRESS[0];
        if (show(r.tip, tipOn)) put(r.tip, UP_PT.x + 12, UP_PT.y + 20, ramp(t, r.upRest + .4, r.upRest + .5, E.lin));

        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('portal', { duration: D, poster: 4.9, build, render });
})();
