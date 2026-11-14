// 6.2 按 Ctrl+Z 撤销 (lesson id undo): Ctrl+Z on a fence takes back the newest step (undo.rs). A
// fence dragged away jumps back; an icon dragged into another fence returns to its fence and
// place; 「删除栅栏」 from the menu asks first when the fence holds icons (fences.rs
// delete_question: a Windows message box, 是 / 否), then the card 「已删除栅栏“工作”」 with 「撤销」
// shows above the taskbar (undo.rs offer_undo_delete) and brings the fence back with its icons.
// 「桌面」 is arranged like the desktop: the icons of a deleted fence stand in its first free
// cells at the top-left corner.
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, spans, spanFade, el, put, show, slot, local, deskCell, icon,
        fence, placeFence, menu, drawMenu, rowY, pointer, drawPointer, drawChip, drawKeys, notice, noticeBox,
        drawNotice, taskbar, loopFade, T } = PM;

    const D = 12.6;
    const A0 = { x: 516, y: 28, w: 246, h: 204, title: '文件与文档' };
    const W0 = { x: 150, y: 44, w: 246, h: 124, title: '工作' };
    const COLS = 3;
    // id: [icon, label]; ids are ASCII so translations never touch them
    const ITEMS = {
        contract: ['pdf', '合同.pdf'],
        invoice: ['pdf', '发票.pdf'],
        plan: ['docx', '方案.docx'],
        quote: ['xlsx', '报价单.xlsx'],
        weekly: ['docx', '周报.docx'],
        budget: ['xlsx', '预算.xlsx'],
        todo: ['txt', '待办.txt'],
    };
    // A fence never arranged by hand shows its icons by name (lowercase, code point order).
    const order = list => [...list].sort((a, b) => {
        const x = ITEMS[a][1].toLowerCase(), y = ITEMS[b][1].toLowerCase();
        return x < y ? -1 : x > y ? 1 : 0;
    });
    const A_ALL = order(['contract', 'invoice', 'plan', 'quote']);
    const A_REST = A_ALL.filter(k => k !== 'contract');
    const W_OWN = order(['weekly', 'budget']);

    // 1. the fence dragged away, Ctrl+Z
    const DRAG_F = { press: 1.05, lift: 1.12, drop: 2.3, dx: 96, dy: 196 };
    const UNDO1 = 3.0;
    // 2. 合同.pdf dragged into 工作 (it lands after the others), Ctrl+Z
    const DRAG_I = { press: 4.55, lift: 4.62, drop: 5.75 };
    const UNDO2 = 6.45;
    // 3. 删除栅栏 from the menu, the question, 是, then 撤销 on the card
    const RC = 7.95, PICK = 8.7, ASK = 8.8, YES = 9.6, CARD = 9.85, UNDO3 = 10.85;
    const KEYS = [{ a: UNDO1 - .3, b: UNDO1 + .55, keys: ['Ctrl', 'Z'], down: UNDO1 },
        { a: UNDO2 - .3, b: UNDO2 + .55, keys: ['Ctrl', 'Z'], down: UNDO2 }];

    const MENU_ITEMS = ['收起', '重命名…', '-', { t: '视图', sub: 1 }, { t: '排序方式', sub: 1 }, { t: '新建', sub: 1 },
        { t: '粘贴', key: 'Ctrl+V', dis: 1 }, { t: '标签页', sub: 1 }, '-', '适应内容大小', '锁定位置和大小', '栅栏选项…', '删除栅栏', '-',
        { t: '添加到桌面', sub: 1 }];
    const GRAB = { x: W0.x + 118, y: W0.y + 16 };         // on the title of 工作
    const MENU = { x: GRAB.x + 2, y: GRAB.y + 2, a: RC + .05, b: PICK + .03 };
    const ASK_BOX = { x: 254, y: 150 };

    const WARN = '<svg viewBox="0 0 26 26" aria-hidden="true"><path d="M13 2.6 24.6 22.8H1.4z" fill="#fcd116" stroke="#b58b00" ' +
        'stroke-width="1" stroke-linejoin="round"/><path d="M13 9.4v7.2" stroke="#1b1b1b" stroke-width="2" stroke-linecap="round"/>' +
        '<circle cx="13" cy="19.6" r="1.25" fill="#1b1b1b"/></svg>';
    const ARROW = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M1.5 6h8M6.5 2.8 9.7 6 6.5 9.2" fill="none" ' +
        'stroke="#1663c7" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';

    let PATH, CLICKS, CHIPS, NOTES;
    const at = t => pathAt(t, PATH);
    const centre = s => [s.x + 39, s.y + 25];

    // Where 工作 is: it follows the pointer while dragged, and Ctrl+Z puts it back in one step.
    function workAt(t) {
        if (t < DRAG_F.press || t >= UNDO1) return { x: W0.x, y: W0.y };
        const p = at(Math.min(t, DRAG_F.drop));
        return { x: W0.x + p.x - GRAB.x, y: W0.y + p.y - GRAB.y };
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.A = fence(S, { ...A0 });
        r.W = fence(S, { ...W0 });
        r.own = W_OWN.map((k, i) => {
            const n = icon(r.W.body, ...ITEMS[k]), s = local(i, COLS);
            put(n, s.x, s.y);
            return n;
        });
        r.inA = Object.fromEntries(A_ALL.map(k => [k, icon(S, ...ITEMS[k])]));
        r.moved = icon(S, ...ITEMS.contract);           // 合同.pdf while it is in 工作
        // loose desktop icons: 待办.txt, and the icons of 工作 while it is deleted
        r.todo = icon(S, ...ITEMS.todo);
        r.todo.classList.add('desk');
        put(r.todo, deskCell(0, 0).x, deskCell(0, 0).y);
        r.back = W_OWN.map((k, i) => {
            const n = icon(S, ...ITEMS[k]), s = deskCell(0, i + 1);
            n.classList.add('desk');
            put(n, s.x, s.y);
            return n;
        });
        r.ghost = icon(S, ...ITEMS.contract);
        r.ghost.classList.add('ghost');
        r.badge = el('div', 'badge', S);
        r.badge.innerHTML = ARROW + T('移动到 %1').replace('%1', `<b>${T('工作')}</b>`);
        r.note = el('div', 'chip', S);
        r.menu = menu(S, MENU_ITEMS, 196);

        // the question before a fence with icons goes (a Windows message box; 否 is its default)
        r.ask = el('div', 'msgbox', S);
        const cap = el('div', 'msgbox-cap', r.ask, T('删除栅栏'));
        el('i', null, cap);
        const body = el('div', 'msgbox-body', r.ask);
        body.innerHTML = WARN;
        const words = el('div', null, body);
        el('p', null, words, T('要删除栅栏“{0}”吗？').replace('{0}', T('工作')));
        el('p', null, words, T('其中的 {0} 个图标会回到“{1}”栅栏，文件本身不受影响。').replace('{0}', W_OWN.length).replace('{1}', T('桌面')));
        const foot = el('div', 'msgbox-foot', r.ask);
        r.yes = el('span', 'msgbox-btn', foot, '是(Y)');
        el('span', 'msgbox-btn def', foot, '否(N)');
        put(r.ask, ASK_BOX.x, ASK_BOX.y);

        r.card = notice(S, { title: T('已删除栅栏“{0}”').replace('{0}', T('工作')),
            text: '点“撤销”，或在栅栏上按 Ctrl+Z，就能把它找回来。', button: '撤销' });
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);

        // pointer targets, measured where they are drawn
        const scale = root.getBoundingClientRect().width / 800 || 1, o = root.getBoundingClientRect();
        const b = r.yes.getBoundingClientRect();
        const yes = [(b.left + b.width / 2 - o.left) / scale, (b.top + b.height / 2 - o.top) / scale];
        r.yesBox = { x: yes[0] - b.width / scale / 2, y: yes[1] - b.height / scale / 2, w: b.width / scale, h: b.height / scale };
        const undo = noticeBox(r.card).btn;
        const del = [MENU.x + 76, MENU.y + rowY(r.menu, '删除栅栏')];
        const from = centre(slot(A0, A_ALL.indexOf('contract'), COLS));
        const into = centre(slot(W0, W_OWN.length, COLS));       // the first free cell of 工作
        PATH = [
            [0, 430, 392], [0.3, 430, 392],
            [0.95, GRAB.x, GRAB.y], [DRAG_F.lift, GRAB.x, GRAB.y],              // press the title of 工作
            [DRAG_F.drop - .15, GRAB.x + DRAG_F.dx, GRAB.y + DRAG_F.dy], [3.95, GRAB.x + DRAG_F.dx, GRAB.y + DRAG_F.dy],
            [4.45, ...from], [DRAG_I.lift, ...from],                             // press 合同.pdf
            [DRAG_I.drop - .15, ...into], [7.3, ...into],                        // into 工作, release
            [7.85, GRAB.x, GRAB.y], [RC + .2, GRAB.x, GRAB.y],                   // right-click the title
            [8.5, ...del], [PICK + .15, ...del],                                 // 删除栅栏
            [9.4, ...yes], [YES + .2, ...yes],                                   // 是
            [10.6, undo.x, undo.y], [UNDO3 + .3, undo.x, undo.y],                // 撤销 on the card
            [11.8, 430, 330], [D, 430, 330],
        ];
        CLICKS = [{ t: RC, right: true }, { t: PICK }, { t: YES }, { t: UNDO3 }];
        CHIPS = [[DRAG_F.press, 1.75, '拖错了地方', 16, -30]];
        // under the fence each one is about
        NOTES = [
            [UNDO1 + .12, UNDO1 + 1.0, '回到原处', W0.x, W0.y + W0.h + 8, 'abs'],
            [UNDO2 + .12, UNDO2 + 1.0, '图标回到原来的栅栏', A0.x, A0.y + A0.h + 8, 'abs'],
            [UNDO3 + .15, UNDO3 + 1.35, '栅栏和图标都回来了', W0.x, W0.y + W0.h + 8, 'abs'],
        ];
        r.overW = spans(D, t => t >= DRAG_I.lift && t < DRAG_I.drop && inside(at(t), W0));
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);

        // 工作: where it is, and gone between 是 and 撤销 (167 ms fades)
        const w = workAt(t);
        const there = (1 - ramp(t, YES + .05, YES + .217, E.lin)) + (t >= UNDO3 ? ramp(t, UNDO3 + .05, UNDO3 + .217, E.lin) : 0);
        if (show(r.W.n, there > 0)) placeFence(r.W, w.x, w.y, W0.w, W0.h, there);
        r.W.drop.style.opacity = spanFade(t, r.overW, .083, .167);
        r.back.forEach(n => show(n, there < 1) && (n.style.opacity = 1 - there));

        // 合同.pdf: in 文件与文档, in 工作 after the drop (it glides from the pointer, 167 ms), back
        // again after Ctrl+Z; the others close and reopen its place (250 ms)
        const inW = t >= DRAG_I.drop && t < UNDO2 + .083;
        const closed = ramp(t, DRAG_I.drop + .05, DRAG_I.drop + .3, E.p2p) * (1 - ramp(t, UNDO2, UNDO2 + .25, E.p2p));
        A_ALL.forEach((k, i) => {
            const n = r.inA[k], a = slot(A0, i, COLS);
            if (k === 'contract') {
                const o = t < DRAG_I.drop ? 1 : t < UNDO2 ? 1 - ramp(t, DRAG_I.drop, DRAG_I.drop + .083, E.lin)
                    : ramp(t, UNDO2 + .02, UNDO2 + .19, E.lin);
                put(n, a.x, a.y, o);
                n.classList.toggle('sel', t >= DRAG_I.press && t < DRAG_I.drop);
                return;
            }
            const b = slot(A0, A_REST.indexOf(k), COLS);
            put(n, lerp(a.x, b.x, closed), lerp(a.y, b.y, closed));
        });
        if (show(r.moved, inW)) {
            const cell = slot(W0, W_OWN.length, COLS), from = at(DRAG_I.drop), q = ramp(t, DRAG_I.drop, DRAG_I.drop + .167);
            put(r.moved, lerp(from.x - 39, cell.x, q), lerp(from.y - 25, cell.y, q), 1 - ramp(t, UNDO2, UNDO2 + .083, E.lin));
        }
        if (show(r.ghost, t >= DRAG_I.lift && t < DRAG_I.drop)) put(r.ghost, p.x - 39, p.y - 25, .72);
        if (show(r.badge, r.overW.some(([a, b]) => t >= a && t < b))) put(r.badge, p.x + 2, p.y + 34);

        // 删除栅栏: the menu, the question, the card with 撤销
        drawMenu(r.menu, t, MENU.x, MENU.y, MENU.a, MENU.b, p);
        if (show(r.ask, t >= ASK && t < YES + .1)) {
            r.ask.style.opacity = ramp(t, ASK, ASK + .1, E.lin) * (1 - ramp(t, YES + .03, YES + .1, E.lin));
            r.yes.classList.toggle('hot', t < YES + .03 && inside(p, r.yesBox));
        }
        drawNotice(r.card, t, CARD, UNDO3 + .03, p);

        drawChip(r.note, t, NOTES, p);
        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: [[DRAG_F.press, DRAG_F.drop], [DRAG_I.press, DRAG_I.drop]], chips: CHIPS, at });
    }

    PM.register('undo', { duration: D, poster: 10.3, build, render });
})();
