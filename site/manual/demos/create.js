// 1.2 新建栅栏: marquee on the empty desktop → the 新建栅栏 button at its corner (the box stays,
// fainter, while the buttons are offered) → the new fence fades in and its title opens as the
// name box → type the name, Enter → drag a loose desktop icon in.
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, firstWhen, el, put, show, text, slot, deskCell, icon, fence,
        pointer, drawPointer, drawKeys, taskbar, loopFade, T, TITLE_H } = PM;

    const D = 11.6;
    // the box drawn on the desktop, then the fence in its place
    const NEU = { x: 150, y: 96, w: 324, h: 180, title: '新栅栏' };
    const X1 = NEU.x + NEU.w, Y1 = NEU.y + NEU.h;
    // 「桌面」 of a new installation: loose icons from the top-left corner, no title, no border
    const DESK = [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['txt', '待办.txt'], ['folder', '素材']];
    const PICK = 1;                                     // 周报.docx goes into the new fence
    const GRAB = { x: deskCell(0, PICK).x + 39, y: deskCell(0, PICK).y + 24 };
    const PATH = [
        [0, 560, 400], [0.3, 560, 400],
        [1.0, NEU.x, NEU.y], [1.2, NEU.x, NEU.y],           // marquee start
        [2.4, X1, Y1], [2.7, X1, Y1],                       // marquee end, release
        [3.1, X1 - 50, Y1 + 21], [5.8, X1 - 50, Y1 + 21],   // 新建栅栏 at the box's corner; rests while the name is typed
        [6.7, GRAB.x, GRAB.y], [6.9, GRAB.x, GRAB.y],       // grab 周报.docx on the desktop
        [8.1, 400, 196], [8.6, 400, 196],                   // let go in the new fence
        [9.3, 560, 340], [D, 560, 340],
    ];
    const RELEASE = 2.5, DRAG = 6.9, DROP_AT = 8.3;
    const CLICKS = [{ t: 3.3 }];
    const PRESSES = [[1.2, RELEASE], [DRAG, DROP_AT]];
    // The two small buttons under the box's bottom-right corner (right edges lined up).
    const OFFER = { right: X1, y: Y1 + 7, a: 2.55, b: 3.36 };
    // create_fence_with_rename: the fence fades in, then its title opens as the name box with
    // the name selected; typing replaces it
    const FADE = 3.4, EDIT = [3.7, 5.4], TYPE = [4.5, 4.8];
    const CHIPS = [[1.2, 2.4, '按住拖动'], [3.9, 5.0, '输入名字', NEU.x + 128, NEU.y + 5, 'abs'],
        [DRAG, 8.1, '拖进新栅栏', 16, -30]];
    const KEYS = [{ a: 5.0, b: 5.8, keys: ['Enter'], down: 5.35 }];

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.prog = fence(S, { x: 530, y: 24, w: 246, h: 124, title: '程序' });
        r.neu = fence(S, { ...NEU });
        r.caret = el('div', 'icons-caret', S);

        [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']].forEach(([k, l], i) => {
            const s = slot(r.prog, i, 3);
            put(icon(S, k, l), s.x, s.y);
        });
        r.deskIcons = DESK.map(([k, l]) => {
            const n = icon(S, k, l);
            n.classList.add('desk');
            return n;
        });
        r.moved = icon(S, ...DESK[PICK]);

        r.marquee = el('div', 'marquee', S);
        r.ghost = icon(S, ...DESK[PICK]);
        r.ghost.classList.add('desk', 'ghost');
        r.badge = el('div', 'badge', S);
        // the fence has its name by then (the name is a literal of its own, as everywhere else)
        r.badge.innerHTML = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M1.5 6h8M6.5 2.8 9.7 6 6.5 9.2" ' +
            'fill="none" stroke="#1663c7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>' +
            T('移动到 %1').replace('%1', '<b>' + '工作' + '</b>');
        r.offer = el('div', 'offer', S);
        const folderBtn = el('div', 'sec', r.offer);
        folderBtn.innerHTML = '<svg viewBox="0 0 14 14" aria-hidden="true"><path d="M1.5 3.5h4l1.2 1.3h5.8v6.7h-11z" ' +
            'fill="none" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/></svg>';
        el('span', null, folderBtn, '在此显示文件夹…');
        r.newBtn = el('div', 'acc', r.offer);
        el('span', 'plus', r.newBtn, '+');
        el('span', null, r.newBtn, '新建栅栏');
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);

        // when the dragged icon comes over the new fence (highlight, badge) and over its item
        // area (insertion line)
        const body = { x: NEU.x, y: NEU.y + TITLE_H, w: NEU.w, h: NEU.h - TITLE_H };
        r.enterNeu = firstWhen(DRAG, DROP_AT, t => inside(pathAt(t, PATH), NEU));
        r.enterBody = firstWhen(DRAG, DROP_AT, t => inside(pathAt(t, PATH), body));
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);

        // 1. marquee, drawn by PecoFence while the desktop icons are hidden. After the release it
        //    stays at half strength for as long as the buttons are offered (marquee_band.rs
        //    OFFERED) and goes with them.
        if (show(r.marquee, t >= 1.22 && t < OFFER.b + .1)) {
            const drawn = t < RELEASE;
            const x1 = drawn ? Math.max(NEU.x, p.x) : X1, y1 = drawn ? Math.max(NEU.y, p.y) : Y1;
            r.marquee.style.width = (x1 - NEU.x) + 'px';
            r.marquee.style.height = (y1 - NEU.y) + 'px';
            put(r.marquee, NEU.x, NEU.y, (drawn ? 1 : .5) * (1 - ramp(t, OFFER.b, OFFER.b + .08, E.lin)));
        }

        // 2. the buttons at the box's corner (they take no keyboard), then the fence fades in
        //    where the box was
        if (show(r.offer, t >= OFFER.a && t < OFFER.b + .1)) {
            const o = ramp(t, OFFER.a, OFFER.a + .083, E.lin) * (1 - ramp(t, OFFER.b, OFFER.b + .08, E.lin));
            put(r.offer, OFFER.right - r.offer.offsetWidth, OFFER.y, o);
            // the pointer reaches 新建栅栏 at 3.1 s (PATH)
            r.newBtn.classList.toggle('hot', t >= 3.05 && t < OFFER.b);
        }
        if (show(r.neu.n, t >= FADE)) {
            const o = ramp(t, FADE, FADE + .22);
            put(r.neu.n, NEU.x, NEU.y, o, ` scale(${lerp(.965, 1, o)})`);
        }

        // 3. the name box opens by itself once the fence has faded in: type → Enter
        const editing = t >= EDIT[0] && t < EDIT[1];
        show(r.neu.edit, editing);
        show(r.neu.name, !editing);
        if (editing) {
            const caret = Math.floor((t - EDIT[0]) / .5) % 2 === 0 ? '<span class="caret"></span>' : '';
            const html = t < TYPE[0] ? '<span class="sel">新栅栏</span>'
                : (t < TYPE[1] ? '工' : '工作') + caret;
            if (r.neu.edit.innerHTML !== html) r.neu.edit.innerHTML = html;
        }
        text(r.neu.name, t >= EDIT[1] ? '工作' : T('新栅栏'));

        // 4. drag 周报.docx from the desktop into the fence. The other loose icons keep their
        //    cells (自由摆放); the dropped one glides from where it was let go to its cell
        //    (167 ms, no fade).
        const dragging = t >= DRAG && t < DROP_AT;
        r.deskIcons.forEach((n, i) => {
            const c = deskCell(0, i);
            put(n, c.x, c.y, i === PICK ? 1 - ramp(t, DROP_AT, DROP_AT + .083, E.lin) : 1);
            n.classList.toggle('sel', i === PICK && dragging);
        });
        const cell = slot(NEU, 0, 4);
        if (show(r.moved, t >= DROP_AT)) {
            const from = pathAt(DROP_AT, PATH), q = ramp(t, DROP_AT, DROP_AT + .167);
            put(r.moved, lerp(from.x - 39, cell.x, q), lerp(from.y - 24, cell.y, q));
        }
        if (show(r.ghost, t >= DRAG + .12 && t < DROP_AT)) put(r.ghost, p.x - 39, p.y - 24, .72);

        // drop feedback of the fence: highlight, insertion line at the first cell, the hint's
        // text (render.rs empty_text: an empty fence under a drag says 松开即可放入) and the badge
        const over = t >= r.enterNeu && t < DROP_AT;
        r.neu.drop.style.opacity = over ? ramp(t, r.enterNeu, r.enterNeu + .083, E.lin)
            : t >= DROP_AT ? 1 - ramp(t, DROP_AT, DROP_AT + .167, E.lin) : 0;
        put(r.caret, cell.x, cell.y, t < DROP_AT ? ramp(t, r.enterBody, r.enterBody + .083, E.lin)
            : 1 - ramp(t, DROP_AT, DROP_AT + .083, E.lin));
        r.neu.hint.style.opacity = 1 - ramp(t, DROP_AT, DROP_AT + .1, E.lin);
        text(r.neu.hint, t >= r.enterNeu && t < DROP_AT + .1 ? T('松开即可放入') : T('将项目拖到此处'));
        if (show(r.badge, over)) put(r.badge, p.x + 2, p.y + 34);

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('create', { duration: D, poster: 8.2, build, render });
})();
