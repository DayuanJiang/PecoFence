// 3.3 自动整理规则: 设置 → 整理规则 → 新建规则 (规则名称「发票」, 名称包含「发票」→「报销」) → 添加规则.
// The new rule goes to the top of the list (settings.html nrAdd: rules match in list order, so a
// new one is put before the broad first-run 「文件与文档」 rule), a new invoice lands in 报销, and
// 立即应用 re-sorts the invoice already on the desktop and shows the app's notice card with 撤销
// above the right end of the taskbar (undo.rs offer_undo_refiled; the page's own toast shows only
// when the card cannot). The fences leave that corner free, as the card keeps off fences.
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, show, text, local, icon, fence, pointer, drawPointer, drawChip,
        notice, drawNotice, taskbar, loopFade, T } = PM;

    const D = 13.0;
    const WIN = { x: 16, y: 14, w: 504, h: 392 };
    const S1 = 40;                                // page scroll: the top cards (after the wheel)
    let S0 = 222;                                 // the rules list + form; more when a translation wraps

    // ---- timeline ----
    const OPEN_AT = .95;                         // 新建规则 summary
    const NAME_CLICK = 1.42, NAME_AT = 1.52;     // 规则名称 「发票」
    const KIND_CLICK = 2.1, KIND_AT = 2.95;      // 条件类型 → 名称包含
    const VALUE_CLICK = 3.35, TYPE_AT = 3.47;    // 条件值 「发票」
    const TARGET_CLICK = 4.05, TARGET_AT = 4.85; // 目标栅栏 → 报销
    const ADD_AT = 5.4;                          // 添加规则 (the rule appears first in the list)
    const NEW_AT = 7.45;                         // 10月发票.pdf appears on the desktop
    const SCROLL = [9.0, 9.5];
    const APPLY_AT = 10.15;                      // 立即应用
    const TOAST = 1.6;                           // the page's toast lasts 2.4 s; shorter so it leaves rule 4 sooner
    const GONE = 1;                              // 9月发票.pdf: the icon of 文件与文档 that 立即应用 moves

    // state of the page at t (what the real page would show)
    const st = t => ({
        open: t >= OPEN_AT + .02,
        kind: t >= KIND_AT + .02 ? '名称包含' : '文件类型',
        name: t >= ADD_AT + .05 ? '' : SM.typed(t, NAME_AT, '发票', .15, .2),   // the form clears both after 添加规则
        nameFocus: t >= NAME_CLICK && t < KIND_CLICK,
        value: t >= ADD_AT + .05 ? '' : SM.typed(t, TYPE_AT, '发票', .15, .2),
        valueFocus: t >= VALUE_CLICK && t < TARGET_CLICK,
        target: t >= TARGET_AT + .02 ? '报销' : '程序',
        added: t >= ADD_AT + .05,
        scroll: lerp(S0, S1, ramp(t, SCROLL[0], SCROLL[1], E.out)),
    });

    const KINDS = ['文件类型', '扩展名（逗号分隔）', '名称包含', '名称不包含', '名称开头是', '名称结尾是',
        '完整文件名（含扩展名，用逗号分隔）', '通配符（*.png）', '快捷方式目标包含', '创建时段', '创建于星期几',
        '大小范围（MB）', '闲置天数（未修改也未打开）', '仅文件夹', '仅文件', '普通文件（程序和快捷方式除外）'];
    const FENCES = ['程序', '文件夹', '文件与文档', '桌面', '报销'];
    const CATS = ['程序', '快捷方式', '文件夹', '文档', '图片', '音乐', '视频', '压缩包', '安装包'];
    // [name, conditions] as condLabel() + "→ 放入“…”" render them
    const PRESETS = [
        ['程序与快捷方式', '类型：程序、快捷方式 → 放入“程序”'],
        ['文件夹', '类型：文件夹 → 放入“文件夹”'],
        ['文件与文档', '类型：文档、图片、音乐、视频、压缩包 → 放入“文件与文档”'],
    ];
    const NEW_RULE = ['发票', '名称包含“发票” → 放入“报销”'];

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;

        // desktop: the first-run documents fence and an empty 报销 fence
        r.F1 = fence(S, { x: 538, y: 14, w: 246, h: 122, title: '文件与文档' });
        r.F2 = fence(S, { x: 538, y: 144, w: 246, h: 122, title: '报销' });
        r.F2.hint.textContent = T('将项目拖到此处');
        r.f1Icons = [['pdf', '合同.pdf'], ['pdf', '9月发票.pdf'], ['docx', '周报.docx']]
            .map(([k, l]) => icon(r.F1.body, k, l));
        r.newIcon = icon(r.F2.body, 'pdf', '10月发票.pdf');
        r.movedIcon = icon(r.F2.body, 'pdf', '9月发票.pdf');
        r.pulse = el('div', 'ring pulse', S);
        r.note = el('div', 'chip', S);

        // settings window on 整理规则
        const w = r.w = SM.win(S, WIN);
        w.foot.textContent = T('{0} 个栅栏 · {1} 个项目').replace('{0}', '5').replace('{1}', '23');
        const pg = r.pg = SM.page(w, 'rules');
        SM.header(pg, '整理规则');
        const auto = SM.card(pg, { ic: '\uE8CB', t: '新项目出现在桌面时自动归类', s: '先看快捷方式目标规则，再按列表顺序匹配；都不符合的放进“桌面”栅栏', textW: 240 });
        SM.drawToggle(SM.toggle(auto), true);
        const unmatched = SM.card(pg, { ic: '\uE718', t: '没有命中规则的项目', s: '应用规则时，已在某个栅栏里但不符合任何规则的项目放在哪里；不符合规则的新项目总是放进“桌面”栅栏', textW: 240 });
        SM.select(unmatched, '回到“桌面”栅栏', 104);
        const apply = SM.card(pg, { ic: '\uE8AB', t: '立即对桌面上的全部项目应用规则', s: '手动放进栅栏的项目保持原位', textW: 240 });
        r.apply = SM.button(apply, '立即应用', 64);
        SM.h3(pg, '快速添加');
        const tpl = SM.card(pg, { ic: '\uE8F4', t: '一键创建栅栏并添加规则', s: '“待清理”只归拢 30 天没用过的安装包和压缩包，不删除任何文件', h: 64, textW: 300 });
        tpl.t.style.top = '8px';
        tpl.s.style.top = '23px';
        // template chips sized to their labels; they wrap onto more rows when a translation is long
        let cx = 34, cy = 39;
        [['\uEB9F', '图片'], ['\uE8D6', '音乐'], ['\uE714', '视频'], ['\uE7FC', '游戏'], ['\uF012', '压缩包'], ['\uE7B8', '安装包'], ['\uEA99', '待清理']].forEach(([g, l]) => {
            const c = SM.ctrl(tpl, 'set-btn', 0, 20, { left: cx, top: cy, label: null });
            c.n.style.borderRadius = '10px';
            c.n.style.height = '20px';
            c.n.innerHTML = `<span class="set-ic" style="left:7px;top:5.5px;font-size:9.5px">${g}</span>` +
                `<span style="left:20px;top:0;line-height:20px;font-size:10px">${T(l)}</span>`;
            const wdt = 22 + SM.natW(c.n.lastChild);
            if (cx > 34 && cx + wdt > pg.cardW - 6) {
                cx = 34;
                cy += 24;
            }
            Object.assign(c, { left: cx, top: cy, w: wdt });
            Object.assign(c.n.style, { left: cx + 'px', top: cy + 'px', width: wdt + 'px' });
            cx += wdt + 5;
        });
        const chipExtra = cy - 39;
        tpl.h += chipExtra;
        PM.size(tpl.n, pg.cardW, tpl.h);
        r.listHead = SM.h3(pg, '规则列表');

        // rule rows: preset ×3 + the new one
        const row = ([name, conds]) => {
            const b = SM.card(pg, { t: name, s: conds, h: 34, textW: 205, indent: 14 });
            b.num = el('span', 'set-num', b.n);
            b.num.style.left = '6px';
            b.num.style.top = '10px';
            b.edit = SM.iconButton(b, 'edit', { right: 112 });
            b.up = SM.iconButton(b, 'up', { right: 90 });
            b.down = SM.iconButton(b, 'down', { right: 68 });
            SM.drawToggle(SM.toggle(b, { caption: false, right: 32 }), true);
            b.del = SM.iconButton(b, 'del', { right: 8 });
            return b;
        };
        r.rows = PRESETS.map(row);
        r.newRow = row(NEW_RULE);

        // 新建规则 (a <details> card): summary + the new-rule form
        const d = r.det = SM.card(pg, { ic: '\uE710', t: '新建规则', h: 32 });
        d.t.style.top = '8.5px';
        d.chev = el('span', 'set-ic set-chev', d.n, '\uE70D');
        d.chev.style.left = 'auto';
        d.chev.style.right = '14px';
        d.chev.style.top = '11px';
        d.line = el('div', null, d.n);
        Object.assign(d.line.style, { left: 0, right: 0, top: '32px', height: '1px', background: 'rgba(0,0,0,.06)' });
        const FW = Math.min(147, Math.floor((pg.cardW - 56) / 2)), COL = [34, 44 + FW];   // two columns
        const label = (s, x, y) => {
            const n = el('span', 'set-lbl', d.n, T(s));
            n.style.left = x + 'px';
            n.style.top = y + 'px';
            return n;
        };
        r.form = [label('规则名称', COL[0], 42), label('目标栅栏', COL[1], 42), label('条件（全部符合时放入目标栅栏）', COL[0], 86)];
        r.condBox = el('div', 'set-cond', d.n);
        Object.assign(r.condBox.style, { left: (COL[0] - 6) + 'px', top: '100px', width: (2 * FW + 22) + 'px' });
        r.form.push(r.condBox, label('条件类型', COL[0], 106), label('条件值', COL[1], 106));
        r.nameIn = SM.input(d, '例如：截图', FW, { left: COL[0], top: 57 });
        r.target = SM.select(d, '程序', FW, { left: COL[1], top: 57 });
        r.kind = SM.select(d, '文件类型', FW, { left: COL[0], top: 120 });
        r.valueIn = SM.input(d, '截图', FW, { left: COL[1], top: 120 });  // placeholder 示例\x04截图
        r.hint = label('有空格也直接输入，不用加引号', COL[1], 146);
        // the condition heading and the hint end at the form's edge (a longer translation ellipsizes)
        [[r.form[2], 2 * FW + 10], [r.hint, FW]].forEach(([n, w]) => Object.assign(n.style,
            { width: w + 'px', overflow: 'hidden', textOverflow: 'ellipsis' }));
        // the type checkboxes, measured; rows past the second push the button and the scroll down
        const catsW = 2 * FW + 10;
        r.cats = el('div', null, d.n);
        Object.assign(r.cats.style, { left: '34px', top: '148px', width: catsW + 'px' });
        let x = 0, y = 0;
        for (const c of CATS) {
            const n = el('div', 'set-chk', r.cats);
            n.innerHTML = `<i></i><span>${T(c)}</span>`;
            const wdt = 16 + SM.natW(n.lastChild);
            if (x > 0 && x + wdt > catsW) {
                x = 0;
                y += 18;
            }
            n.style.left = x + 'px';
            n.style.top = y + 'px';
            x += wdt + 12;
        }
        r.cats.style.height = (y + 14) + 'px';
        r.typeExtra = Math.max(0, y - 18);
        // the scroll shows the rules from the first row (the 规则列表 heading just above the view)
        // with the form open below them; type checkboxes that wrap further run past the bottom
        S0 = 288 + chipExtra;
        r.addCond = el('div', 'set-lbl set-link', d.n, T('添加条件'));
        r.addCond.style.left = COL[0] + 'px';
        r.addBtn = SM.button(d, '添加规则', 70, { accent: true, right: 12, top: 0 });
        r.form.push(r.nameIn.n, r.target.n, r.kind.n, r.valueIn.n, d.line, r.addCond);

        r.kindPop = SM.popup(S, KINDS, 196, 8);
        r.targetPop = SM.popup(S, FENCES, 147);
        r.card = notice(S, { title: T('已按规则整理 {0} 个项目').replace('{0}', '1'),
            text: T('点「撤销」，或在栅栏上按 Ctrl+Z，就能把它们放回原处。'), button: T('撤销') });
        r.ptr = pointer(S);
        taskbar(root);

        r.layout = s => layout(r, s);
        plan(r);
        return r;
    }

    // Applies the page state: form open / kind, rule rows and their order, scroll.
    function layout(r, s) {
        const d = r.det, name = s.kind !== '文件类型';
        r.newRow.on = s.added;
        const boxH = name ? 64 : 82 + r.typeExtra;              // 条件类型 / 条件值 (+ hint or checkboxes)
        r.condBox.style.height = boxH + 'px';
        r.addCond.style.top = (100 + boxH + 8) + 'px';
        r.addBtn.top = 100 + boxH + 30;
        r.addBtn.n.style.top = r.addBtn.top + 'px';
        d.h = !s.open ? 32 : r.addBtn.top + 22 + 10;
        PM.size(d.n, r.pg.cardW, d.h);
        r.pg.scroll = s.scroll;
        const rows = [r.newRow, ...r.rows];
        const blocks = r.pg.blocks.filter(b => !rows.includes(b) && b !== d);
        const at = blocks.indexOf(r.listHead) + 1;
        r.pg.blocks = [...blocks.slice(0, at), ...rows, d];
        SM.stack(r.pg);
        rows.filter(b => b.on).forEach((b, i, list) => {
            text(b.num, String(i + 1));
            b.up.n.classList.toggle('off', i === 0);
            b.down.n.classList.toggle('off', i === list.length - 1);
        });
    }

    // Stage rect of a page block (card) as laid out now.
    const blockBox = (r, b) => ({
        x: r.w.x + r.w.navW + SM.CARD_X, y: r.w.y + SM.TITLE_H + b.y - r.pg.scroll, w: r.pg.cardW, h: b.h,
    });

    let PATH, CLICKS, PRESSES, CHIPS, POP;
    function plan(r) {
        // pointer targets read off the laid-out page at the moment they are used
        const pos = (t, c, dx, dy) => {
            r.layout(st(t));
            return SM.at(c, dx, dy);
        };
        r.layout(st(.5));
        const det = { x: blockBox(r, r.det).x + 60, y: blockBox(r, r.det).y + 16 };
        const name = pos(NAME_CLICK, r.nameIn, -20, 0);
        const kind = pos(KIND_CLICK, r.kind, -10, 0);
        const value = pos(VALUE_CLICK, r.valueIn, -20, 0);
        const target = pos(TARGET_CLICK, r.target, -14, 0);
        const add = pos(ADD_AT, r.addBtn, 0, 0);
        const applyBtn = pos(APPLY_AT, r.apply, 0, 0);

        // popups: the kind list opens above (no room below), the fence list below
        const kb = (r.layout(st(KIND_CLICK)), SM.box(r.kind));
        const tb = (r.layout(st(TARGET_CLICK)), SM.box(r.target));
        POP = {
            kind: { x: kb.x, y: kb.y - 2 - r.kindPop.h },
            target: { x: tb.x, y: tb.y + tb.h + 2 },
        };
        const kindRow = { x: POP.kind.x + 130, y: POP.kind.y + SM.popRow(r.kindPop, '名称包含') };
        const targetRow = { x: POP.target.x + 40, y: POP.target.y + SM.popRow(r.targetPop, '报销') };

        PATH = [
            [0, 430, 392], [0.3, 430, 392],
            [0.85, det.x, det.y], [1.0, det.x, det.y],               // 新建规则
            [1.35, name.x, name.y], [1.85, name.x, name.y],          // 规则名称
            [2.02, kind.x, kind.y], [2.15, kind.x, kind.y],          // 条件类型
            [2.55, kindRow.x, kindRow.y], [3.0, kindRow.x, kindRow.y],
            [3.27, value.x, value.y], [3.7, value.x, value.y],       // 条件值
            [3.97, target.x, target.y], [4.1, target.x, target.y],   // 目标栅栏
            [4.45, targetRow.x, targetRow.y], [4.9, targetRow.x, targetRow.y],
            [5.3, add.x, add.y], [5.7, add.x, add.y],                // 添加规则
            [6.5, 455, 250], [8.65, 455, 250],
            [8.95, 300, 250], [9.55, 300, 250],                      // wheel up
            [10.05, applyBtn.x, applyBtn.y], [10.65, applyBtn.x, applyBtn.y], // 立即应用
            [11.0, applyBtn.x - 34, applyBtn.y + 42], [D, applyBtn.x - 34, applyBtn.y + 42],   // rests clear of the card
        ];
        CLICKS = [{ t: OPEN_AT }, { t: NAME_CLICK }, { t: KIND_CLICK }, { t: KIND_AT }, { t: VALUE_CLICK }, { t: TARGET_CLICK }, { t: TARGET_AT },
            { t: ADD_AT }, { t: APPLY_AT }];
        PRESSES = [[ADD_AT - .08, ADD_AT + .02], [APPLY_AT - .08, APPLY_AT + .02]];
        CHIPS = [[8.97, 9.55, '滚动']];
        r.layout(st(0));
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);
        const s = st(t);

        // ---- settings page ----
        SM.drawNav(r.w, t, [[0, 'rules']], [], p);
        r.layout(s);
        const d = r.det, name = s.kind !== '文件类型';
        d.chev.style.transform = `rotate(${180 * ramp(t, OPEN_AT, OPEN_AT + .15, E.out)}deg)`;
        const sb = blockBox(r, d);
        d.n.style.background = !s.open && PM.inside(p, { ...sb, h: 32 }) ? '#f4f4f4' : '';
        for (const n of r.form) show(n, s.open);
        show(r.addBtn.n, s.open);
        show(r.valueIn.n, s.open && name);
        show(r.hint, s.open && name);
        show(r.cats, s.open && !name);
        SM.drawInput(r.nameIn, s.name, s.nameFocus, t);
        SM.drawInput(r.valueIn, s.value, s.valueFocus, t);
        const kindOpen = t >= KIND_CLICK && t < KIND_AT, targetOpen = t >= TARGET_CLICK && t < TARGET_AT;
        SM.drawSelect(r.kind, s.kind, p, kindOpen);
        SM.drawSelect(r.target, s.target, p, targetOpen);
        SM.drawButton(r.addBtn, p, t >= ADD_AT - .08 && t < ADD_AT + .02);
        SM.drawButton(r.apply, p, t >= APPLY_AT - .08 && t < APPLY_AT + .02);
        SM.drawPopup(r.kindPop, t, POP.kind.x, POP.kind.y, KIND_CLICK + .02, KIND_AT, p, '文件类型');
        SM.drawPopup(r.targetPop, t, POP.target.x, POP.target.y, TARGET_CLICK + .02, TARGET_AT, p, '程序');
        SM.drawToast(r.w, t, [[ADD_AT + .05, T('已添加规则'), TOAST]]);
        drawNotice(r.card, t, APPLY_AT + .05, D, p);

        // ---- desktop ----
        // 9月发票.pdf leaves 文件与文档 (83 ms fade), 周报.docx glides into its cell (250 ms), and
        // it fades into 报销 (167 ms); the new file fades in as the first item of 报销.
        r.f1Icons.forEach((n, i) => {
            let a = local(i, 3), o = 1;
            if (i === GONE) o = 1 - ramp(t, APPLY_AT + .05, APPLY_AT + .133, E.lin);
            if (i > GONE) {
                const b = local(i - 1, 3), q = ramp(t, APPLY_AT + .05, APPLY_AT + .3, E.p2p);
                a = { x: lerp(a.x, b.x, q), y: lerp(a.y, b.y, q) };
            }
            put(n, a.x, a.y, o);
        });
        const s0 = local(0, 3), s1 = local(1, 3);
        if (show(r.newIcon, t >= NEW_AT)) put(r.newIcon, s0.x, s0.y, ramp(t, NEW_AT, NEW_AT + .167, E.lin));
        if (show(r.movedIcon, t >= APPLY_AT)) put(r.movedIcon, s1.x, s1.y, ramp(t, APPLY_AT + .05, APPLY_AT + .217, E.lin));
        r.F2.hint.style.opacity = 1 - ramp(t, NEW_AT, NEW_AT + .1, E.lin);

        // annotations: pulse + caption on the new file and on the re-sorted one
        const cx = r.F2.x + s0.x + 39, cy = r.F2.y + 32 + s0.y + 24;
        const mx = r.F2.x + s1.x + 39;
        const pulses = [[NEW_AT, cx], [APPLY_AT + .05, mx]];
        const pu = pulses.find(([a]) => t >= a && t < a + .8);
        if (show(r.pulse, !!pu)) {
            const q = (t - pu[0]) / .8;
            put(r.pulse, pu[1], cy, (1 - q) * .9, ` scale(${.6 + E.out(q) * 1.1})`);
        }
        drawChip(r.note, t, [[NEW_AT, NEW_AT + 1.4, '新文件', cx - 34, cy + 40, 'abs']], p);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('rules', { duration: D, poster: 8.0, build, render });
})();
