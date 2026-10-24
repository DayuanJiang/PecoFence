// 4.1 主题、透明度和颜色: 常规 → 主题风格 Liquid Glass (all fences switch), 栅栏菜单 → 栅栏选项…
// (settings jump to 栅栏 with the fence selected), 色调 绿 → 只给标题栏上色, 标题字号 大,
// 不透明度 全透明 (bare icons and title; the default glass fades in under the pointer).
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, spans, spanFade, inside, el, put, show, text, local, icon, fence, menu, drawMenu,
        rowY, pointer, drawPointer, taskbar, loopFade, T } = PM;

    const D = 12.8;
    const WIN = { x: 16, y: 16, w: 500, h: 390 };
    const F0 = { x: 536, y: 60, w: 246, h: 236 };
    const GREEN = '10893E';                         // fence_options.rs TINT_PALETTE 绿

    // ---- timeline ----
    const THEME_AT = 1.05;                          // Liquid Glass tile
    const RCLICK = 2.75, MENU = { x: 598, y: 78, a: 2.8, b: 3.5 };
    const OPTIONS_AT = 3.5;                         // 栅栏选项… → 栅栏 page
    const TINT_AT = 4.55, TOP_AT = 5.45;            // 色调 绿, 只给标题栏上色
    const SIZE_CLICK = 7.0, SIZE_AT = 7.6;          // 标题字号 → 大
    const OPA_CLICK = 8.3, OPA_AT = 8.85;           // 不透明度 → 全透明

    const SWATCHES = [null, 'E74856', 'F7630C', 'FFB900', '10893E', '00B7C3', '0078D4', '8764B8', 'E3008C', '7A7574'];
    const OPACITY = ['全透明', '更透明', '默认', '更厚实'];
    const SIZES = ['小', '标准', '大', '特大'];

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;

        // the fence on the desktop, with its plates as separate layers
        const F = r.F = fence(S, { ...F0, title: '工作' });
        F.n.classList.add('set-f');
        r.tintFull = el('div', 'set-tint', null);
        r.tintTop = el('div', 'set-tint set-tint-top', null);
        r.pf = el('div', 'set-pf', null);
        r.plg = el('div', 'set-plg', null);
        el('div', 'set-plg-rim', r.plg);
        el('div', 'set-plg-hi', r.plg);
        [r.tintTop, r.tintFull, r.plg, r.pf].forEach(n => F.n.insertBefore(n, F.n.firstChild));
        [r.tintFull, r.tintTop].forEach(n => { n.style.background = `#${GREEN}38`; });
        [['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['pdf', '合同.pdf'], ['url', '项目主页'], ['txt', '待办.txt'], ['folder', '素材']]
            .forEach(([k, l], i) => {
                const s = local(i, 3);
                put(icon(F.body, k, l), s.x, s.y);
            });

        // settings window: 常规 and 栅栏 pages
        const w = r.w = SM.win(S, WIN);
        w.foot.textContent = T('{0} 个栅栏 · {1} 个项目').replace('{0}', '4').replace('{1}', '21');
        ({ g: r.gen, tiles: r.tiles } = SM.generalPage(w));

        const f = r.fen = SM.page(w, 'fences');
        SM.header(f, '栅栏');
        const picker = SM.block(f, 'set-pick', 30, 0);
        r.dot = el('span', null, picker.n);
        Object.assign(r.dot.style, { left: '2px', top: '5.5px', width: '11px', height: '11px', borderRadius: '50%', background: '#' + GREEN,
            boxShadow: 'inset 0 0 0 1px rgba(0,0,0,.2)' });
        r.fenceSel = SM.select(picker, '工作', 160, { left: 2, top: 0 });
        const count = el('span', 'set-s', picker.n, T('{0} 个栅栏').replace('{0}', '4'));
        Object.assign(count.style, { right: '10px', top: '5px', color: '#8a8a8a' });
        SM.drawInput(SM.input(SM.card(f, { ic: '\uE8AC', t: '栅栏名称', textW: 160 }), '', 140), '工作', false, 0);
        SM.h3(f, '外观');
        r.opa = SM.select(SM.card(f, { ic: '\uE744', t: '不透明度', s: '默认：透出桌面。更厚实：多一层底色。全透明：鼠标悬停时才显示玻璃。', textW: 190 }), '默认', 100);
        const tint = SM.card(f, { ic: '\uE790', t: '色调', s: '给玻璃染上一层颜色，标签页的色条随之变化', textW: 118 });
        r.sw = SWATCHES.map((hex, i) => {
            const c = SM.ctrl(tint, 'set-sw' + (hex ? '' : ' none'), 15, 15, { right: 10 + (SWATCHES.length - 1 - i) * 19.5, label: null });
            if (hex) c.n.style.background = '#' + hex;
            return c;
        });
        r.topCard = SM.card(f, { ic: '\uE737', t: '只给标题栏上色', s: '关掉的话，整个栅栏都会染上色调', textW: 200 });
        r.topTog = SM.toggle(r.topCard);
        // 布局 and 标题颜色 come before 标题字号 on the page; left out here, 标题字号 stays in view.
        SM.h3(f, '标题栏');
        r.size = SM.select(SM.card(f, { ic: '\uE8E9', t: '标题字号', s: '栅栏标题和标签页文字的大小', textW: 190 }), '标准', 100);

        // the fence menu (menus.rs show_fence_menu; 标签页 is there because other fences exist)
        r.menu = menu(S, ['收起', '重命名…', '-', { t: '视图', sub: 1 }, { t: '排序方式', sub: 1 }, { t: '新建', sub: 1 },
            { t: '粘贴', key: 'Ctrl+V' }, { t: '标题栏', sub: 1 }, { t: '标签页', sub: 1 }, '锁定位置和大小', '-', '栅栏选项…', '新建栅栏',
            { t: '在桌面显示文件夹', sub: 1 }, '删除栅栏'], 196);
        r.menu.n.style.zIndex = 3;
        r.opaPop = SM.popup(S, OPACITY, 100);
        r.sizePop = SM.popup(S, SIZES, 100);
        r.ptr = pointer(S);
        taskbar(root);

        plan(r);
        r.glassHover = spans(D, t => t > OPA_AT && inside(pathAt(t, PATH), F0));
        return r;
    }

    const st = t => ({
        lg: t >= THEME_AT + .02,
        page: t >= OPTIONS_AT + .05 ? 'fences' : 'general',
        tint: t >= TINT_AT + .02,
        top: t >= TOP_AT + .02,
        size: t >= SIZE_AT + .02 ? '大' : '标准',
        opacity: t >= OPA_AT + .02 ? '全透明' : '默认',
    });

    function layout(r, s) {
        r.topCard.on = s.tint;
        SM.stack(r.gen);
        SM.stack(r.fen);
        show(r.dot, s.tint);
        r.fenceSel.left = s.tint ? 22 : 2;
        r.fenceSel.n.style.left = r.fenceSel.left + 'px';
    }

    let PATH, CLICKS, CHIPS, POP;
    function plan(r) {
        const at = (t, c, dx = 0, dy = 0) => {
            layout(r, st(t));
            return SM.at(c, dx, dy);
        };
        const tile = at(THEME_AT, r.tiles[1], -10, -10);
        const opt = { x: MENU.x + 70, y: MENU.y + rowY(r.menu, '栅栏选项…') };
        const green = at(TINT_AT, r.sw[4]);
        const tog = (layout(r, st(TOP_AT)), SM.switchAt(r.topTog));   // the switch, however long its caption
        const size = at(SIZE_CLICK, r.size, -16, 0);
        const sb = (layout(r, st(SIZE_CLICK)), SM.box(r.size));
        const opa = at(OPA_CLICK, r.opa, -16, 0);
        const ob = (layout(r, st(OPA_CLICK)), SM.box(r.opa));
        POP = {
            size: { x: sb.x, y: sb.y - 2 - r.sizePop.h },     // no room below: opens upwards
            opa: { x: ob.x, y: ob.y + ob.h + 2 },
        };
        const big = { x: POP.size.x + 40, y: POP.size.y + SM.popRow(r.sizePop, '大') };
        const bare = { x: POP.opa.x + 40, y: POP.opa.y + SM.popRow(r.opaPop, '全透明') };
        PATH = [
            [0, 420, 380], [0.3, 420, 380],
            [0.9, tile.x, tile.y], [1.4, tile.x, tile.y],           // Liquid Glass
            [1.9, tile.x, tile.y], [2.6, 596, 76], [2.85, 596, 76], // right-click the fence title
            [3.3, opt.x, opt.y], [3.6, opt.x, opt.y],               // 栅栏选项…
            [4.4, green.x, green.y], [4.7, green.x, green.y],       // 色调 绿
            [5.3, tog.x, tog.y], [6.4, tog.x, tog.y],               // 只给标题栏上色
            [6.9, size.x, size.y], [7.05, size.x, size.y],          // 标题字号
            [7.45, big.x, big.y], [7.7, big.x, big.y],
            [8.2, opa.x, opa.y], [8.35, opa.x, opa.y],              // 不透明度
            [8.7, bare.x, bare.y], [9.0, bare.x, bare.y],
            [9.9, 704, 276], [10.9, 704, 276],                      // over the bare fence
            [11.7, 662, 372], [D, 662, 372],
        ];
        CLICKS = [{ t: THEME_AT }, { t: RCLICK, right: true }, { t: OPTIONS_AT }, { t: TINT_AT }, { t: TOP_AT },
            { t: SIZE_CLICK }, { t: SIZE_AT }, { t: OPA_CLICK }, { t: OPA_AT }];
        CHIPS = [[2.45, 2.95, '右键标题栏', 548, 36, 'abs'], [9.75, 10.8, '移上去']];
        layout(r, st(0));
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);
        const s = st(t);

        // ---- settings window ----
        r.w.n.classList.toggle('set-lg', s.lg);   // :root.liquid-glass: rounder cards and controls
        SM.drawNav(r.w, t, [[0, 'general'], [OPTIONS_AT + .05, 'fences']], [], p);
        layout(r, s);
        SM.drawTiles(r.tiles, s.lg, p, s.page === 'general');
        r.sw.forEach((c, i) => c.n.classList.toggle('on', s.tint ? i === 4 : i === 0));
        SM.drawToggle(r.topTog, s.top);
        SM.drawSelect(r.size, s.size, p, t >= SIZE_CLICK && t < SIZE_AT);
        SM.drawSelect(r.opa, s.opacity, p, t >= OPA_CLICK && t < OPA_AT);
        SM.drawPopup(r.sizePop, t, POP.size.x, POP.size.y, SIZE_CLICK + .02, SIZE_AT, p, '标准');
        SM.drawPopup(r.opaPop, t, POP.opa.x, POP.opa.y, OPA_CLICK + .02, OPA_AT, p, '默认');
        drawMenu(r.menu, t, MENU.x, MENU.y, MENU.a, MENU.b, p);

        // ---- the fence ----
        // material: Fluent ↔ Liquid Glass cross-fade; 全透明 drops the plate (and its tint) at
        // rest and fades the default plate in for 167 ms while the pointer is over the fence.
        const lg = ramp(t, THEME_AT, THEME_AT + .2);
        const plate = t < OPA_AT ? 1 : Math.max(1 - ramp(t, OPA_AT, OPA_AT + .167, E.lin), spanFade(t, r.glassHover, .167, .167));
        r.pf.style.opacity = (1 - lg) * plate;
        r.plg.style.opacity = lg * plate;
        const tint = ramp(t, TINT_AT, TINT_AT + .12, E.lin) * plate, top = ramp(t, TOP_AT, TOP_AT + .12, E.lin);
        r.tintFull.style.opacity = tint * (1 - top);
        r.tintTop.style.opacity = tint * top;
        r.F.n.classList.toggle('set-bare', plate < .5);
        r.F.name.style.fontSize = s.size === '大' ? '15px' : '';   // 标题字号 14 → 16 DIP, title row height unchanged

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('look', { duration: D, poster: 6.2, build, render });
})();
