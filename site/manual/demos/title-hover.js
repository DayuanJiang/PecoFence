// 4.2 鼠标移上去才显示标题栏: 设置 → 常规, scrolled down past 外观 and 桌面与启动 to 行为
// (ui/settings.html order), → 「鼠标悬停时才显示标题栏」 hides the title rows (the glass starts at
// the icons), the pointer brings them back fence by fence, and 栅栏 page → 「显示标题栏」 →
// 「始终显示」 keeps 程序's title row for good. The window is the shared settings mock (window.SM,
// demos/settings-mock.js). fence_window/render.rs title_state: the plate's top edge glides 167 ms
// decelerate, the title fades 83 ms linear; the window and its icons never move
// (roll.rs title_visible_target: pointer anywhere over the window).
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, spans, spanFade, el, put, size, local, icon, fence, pointer,
        drawPointer, taskbar, loopFade, T, TITLE_H } = PM;

    const D = 12.0;
    const WIN = { x: 18, y: 16, w: 488, h: 392 };
    const F1 = { x: 532, y: 44, w: 246, h: 124 };            // 程序
    const F2 = { x: 532, y: 176, w: 246, h: 204 };           // 文件与文档
    const PLATE_S = .25, TITLE_S = .12;                      // 167 / 83 ms, slowed a little to read

    // 常规 below what SM.generalPage builds, in the page's order with its Segoe Fluent glyphs;
    // defaults from model.rs (Settings, IconSettings, RollUpSettings, SnappingSettings)
    const ICON_TINTS = [null, '60CDFF', '10893E', 'F7630C', 'E74856', '8764B8', '7A7574'];   // 不着色 … 灰
    const DESKTOP = [
        { ic: '', t: '随 Windows 启动', on: true },
        { ic: '', t: '隐藏 Windows 桌面图标', s: '桌面项目只显示在栅栏里，未归类的进入“桌面”栅栏；回收站等系统图标也会出现在“桌面”栅栏中', on: true },
        { ic: '', t: '双击桌面空白处隐藏或显示栅栏', on: true },
    ];
    const BEHAVIOUR = [
        { ic: '', t: '收起的栅栏悬停时自动展开', s: '鼠标移开后自动收起', on: true },
        { ic: '', t: '收起的栅栏需点击标题才展开', s: '开启后不再悬停展开；拖动标题仍可移动栅栏', on: false },
        { ic: '', t: '展开时把下面的栅栏推开', s: '收起后它们会回到原位', on: true },
        { ic: '', t: '鼠标悬停时才显示标题栏', s: '平时只显示图标，让桌面更清爽；每个栅栏可在“栅栏”页单独设置', on: false },
        { ic: '', t: '自动隐藏滚动条', s: '鼠标移到栅栏上或滚动时才显示', on: false },
    ];
    const TARGET = 3;
    // The pages go on beyond what is built (常规: 调整大小时按整行整列对齐, 速览栅栏; 栅栏: 外观, 布局,
    // the rest of 标题栏, 行为): only the scrollbar thumb needs their height.
    const GEN_MORE = 200, FEN_MORE = 720;
    // 栅栏 page: the 「显示标题栏」 options
    const OPTIONS = ['跟随常规设置', '始终显示', '鼠标悬停时显示'];

    const SCROLL_A = .62, SCROLL_B = 1.45;                   // wheel: top of 常规 → 行为
    const TOG_AT = 1.85, NAV_AT = 7.3, SEL_AT = 8.3, PICK_AT = 9.1;
    let PATH, CLICKS, CHIPS, POP, SCROLL;
    const at = t => pathAt(t, PATH);

    // A card with a switch on the right (the page's .card > .toggle).
    function toggleCard(pg, { ic, t, s, on }, o = {}) {
        const b = SM.card(pg, { ic, t, s, textW: pg.cardW - 34 - 52 - 24, ...o });
        const c = SM.toggle(b);
        SM.drawToggle(c, on);
        return c;
    }

    function generalPage(w) {
        const { g, tiles } = SM.generalPage(w);
        SM.drawTiles(tiles, false, null, false);
        const sw = 130, selW = g.cardW - sw - 60;
        const tint = SM.card(g, { ic: '', t: '图标着色', textW: 100, gap: 1 });
        ICON_TINTS.forEach((hex, i) => {
            const c = SM.ctrl(tint, 'set-sw' + (hex ? '' : ' none on'), 15, 15,
                { right: 10 + (ICON_TINTS.length - 1 - i) * 19.5, label: null });
            if (hex) c.n.style.background = '#' + hex;
        });
        // 图标着色强度 is greyed out while no tint is chosen (bindIconTint → enableStrength)
        const strength = SM.card(g, { t: '图标着色强度', h: 32, indent: 22, textW: 120 });
        SM.select(strength, '中', sw);
        strength.n.style.opacity = .55;
        toggleCard(g, { ic: '', t: '图标融入背景（Chameleon）', s: '降低图标饱和度和不透明度，让桌面看起来更清爽', on: false });
        SM.select(SM.card(g, { ic: '', t: '显示语言', s: '立即切换界面语言；你的文件和自定义名称保持不变', textW: selW }), '跟随系统', sw);

        SM.h3(g, '桌面与启动');
        DESKTOP.forEach(d => toggleCard(g, d));
        SM.select(SM.card(g, { ic: '', t: '显示桌面 (Win+D) 时', textW: selW }), '保持栅栏可见', sw);

        const head = SM.h3(g, '行为');
        const toggles = BEHAVIOUR.map(d => toggleCard(g, d));

        SM.h3(g, '吸附与对齐');
        toggleCard(g, { ic: '', t: '移动栅栏时吸附对齐', s: '拖动时吸附到其他栅栏、屏幕边缘和屏幕中线；按住 Alt 可暂时不吸附', on: true }, { gap: 1 });
        toggleCard(g, { t: '显示对齐参考线', on: true }, { h: 32, indent: 22 });
        SM.select(SM.card(g, { ic: '', t: '栅栏间距', s: '移动和调整大小时，栅栏之间以及与屏幕边缘保持的距离', textW: selW }), '8 px', sw);
        g.contentH = SM.stack(g) + GEN_MORE;
        return { g, head, toggles };
    }

    // 栅栏 page from the top (showPage resets the scroll), 程序 selected.
    function fencesPage(w) {
        const f = SM.page(w, 'fences');
        SM.header(f, '栅栏');
        const picker = SM.block(f, 'set-pick', 30, 0);
        SM.select(picker, '程序', 160, { left: 2, top: 0 });
        const count = el('span', 'set-s', picker.n, T('{0} 个栅栏').replace('{0}', '2'));
        Object.assign(count.style, { right: '16px', top: '5px', color: '#8a8a8a' });
        SM.drawInput(SM.input(SM.card(f, { ic: '', t: '栅栏名称', textW: 160 }), '', 140), '程序', false, 0);
        // 外观 and 布局 come before 标题栏 on the page and are left out here, so 显示标题栏 stays in
        // view under the 标题栏 cards above it.
        SM.h3(f, '标题栏');
        SM.select(SM.card(f, { ic: '', t: '标题颜色', s: '要跟随色调，请先选一种色调', textW: 190 }), '跟随主题', 100);
        SM.select(SM.card(f, { ic: '', t: '标题字号', s: '栅栏标题和标签页文字的大小', textW: 190 }), '标准', 100);
        SM.select(SM.card(f, { ic: '', t: '标题栏位置', s: '放在底部时，标签页留在标题栏里，栅栏收向标题栏', textW: 190 }), '顶部', 100);
        SM.select(SM.card(f, { ic: '\uE8E4', t: '标题对齐', textW: 190 }), OPTIONS[0], 100);
        const mode = SM.select(SM.card(f, { ic: '', t: '显示标题栏', s: '鼠标悬停时显示：平时收起标题行，指针移到栅栏上才出现',
            textW: f.cardW - 34 - 120 - 24 }), OPTIONS[0], 120);
        f.contentH = SM.stack(f) + FEN_MORE;
        return { f, mode };
    }

    // [[t, 0 | 1], ...] where a boolean target flips, sampled like PM.spans.
    function flips(test) {
        const ev = [];
        let cur = test(0);
        for (let t = 0; t <= D; t += 1 / 120) {
            const v = test(t);
            if (v !== cur) ev.push([t, v ? 1 : 0]);
            cur = v;
        }
        return { init: test(0) ? 1 : 0, ev };
    }

    // A tween retargeted at every flip (motion.rs Tween::retarget), as a pure function of t.
    function tween(t, { init, ev }, dur, e) {
        let v = init;
        for (const [te, to] of ev) {
            if (t < te) break;
            v = lerp(v, to, ramp(t, te, te + dur, e));
        }
        return v;
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;

        const w = r.w = SM.win(S, WIN);
        w.foot.textContent = T('{0} 个栅栏 · {1} 个项目').replace('{0}', '2').replace('{1}', '9');
        ({ g: r.gen, head: r.head, toggles: r.toggles } = generalPage(w));
        ({ f: r.fen, mode: r.mode } = fencesPage(w));
        r.thumb = el('div', 'xs-thumb', w.main);       // main::-webkit-scrollbar-thumb (always shown)

        r.A = fence(S, { ...F1, title: '程序' });
        r.B = fence(S, { ...F2, title: '文件与文档' });
        [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']].forEach(([k, l], i) => {
            const s = local(i, 3);
            put(icon(r.A.body, k, l), s.x, s.y);
        });
        [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['txt', '待办.txt'], ['pdf', '发票.pdf'], ['folder', '素材']]
            .forEach(([k, l], i) => {
                const s = local(i, 3);
                put(icon(r.B.body, k, l), s.x, s.y);
            });
        r.pop = SM.popup(S, OPTIONS, r.mode.w);
        r.ptr = pointer(S);
        taskbar(root);

        // targets: the switch once 行为 is at the top, the 栅栏 nav item, the select and its list.
        // SM.box / SM.at miss a card's CSS left (.set-card left 12px on top of stack's CARD_X), so
        // the real offset is measured once here (0 if the mock gets fixed).
        const scale = root.getBoundingClientRect().width / 800 || 1;
        const dx = (r.mode.n.getBoundingClientRect().left - root.getBoundingClientRect().left) / scale - SM.box(r.mode).x;
        const fix = q => ({ ...q, x: q.x + dx });
        SCROLL = r.head.y - 2;
        r.gen.scroll = SCROLL;
        const tog = fix(SM.switchAt(r.toggles[TARGET]));
        r.gen.scroll = 0;
        const nav = SM.navAt(w, 'fences');
        const sel = fix(SM.at(r.mode, -20, 0));
        const sb = fix(SM.box(r.mode));
        const below = sb.y + sb.h + 2;
        POP = { x: sb.x, y: below + r.pop.h <= 422 ? below : sb.y - 2 - r.pop.h };   // no room below: opens upwards
        const pick = { x: POP.x + 44, y: POP.y + SM.popRow(r.pop, '始终显示') };

        PATH = [
            [0, 400, 384], [0.3, 400, 384],
            [0.6, 330, 340], [1.5, 330, 340],                       // over the 常规 page: wheel down
            [1.75, tog.x, tog.y], [2.2, tog.x, tog.y],              // the 行为 switch
            [3.05, 600, 104], [3.35, 600, 104],                     // into 程序
            [3.85, 712, 62], [4.25, 712, 62],                       // over its title row
            [5.05, 650, 292], [5.45, 650, 292],                     // down into 文件与文档
            [6.15, 655, 402], [6.6, 655, 402],                      // out onto the desktop
            [7.2, nav.x, nav.y], [7.65, nav.x, nav.y],
            [8.2, sel.x, sel.y], [8.6, sel.x, sel.y],
            [9.0, pick.x, pick.y], [9.5, pick.x, pick.y],
            [10.25, 655, 402], [D, 655, 402],
        ];
        CLICKS = [{ t: TOG_AT }, { t: NAV_AT }, { t: SEL_AT }, { t: PICK_AT }];
        CHIPS = [[.65, 1.4, '向下滚动', -96, 14], [1.6, 2.1, '点开关', -60, 16], [2.4, 3.25, '移上去', 566, 14, 'abs'],
            [5.85, 6.45, '移开', 16, -14]];

        const globalOn = t => t >= TOG_AT + .03;
        r.showA = flips(t => !globalOn(t) || t >= PICK_AT + .04 || inside(at(t), F1));
        r.showB = flips(t => !globalOn(t) || inside(at(t), F2));
        r.titleRowA = spans(D, t => inside(at(t), { x: F1.x, y: F1.y, w: F1.w, h: TITLE_H }));
        return r;
    }

    // Title on hover: the glass (and its rim and shadow) starts plate_top below the window top;
    // the title row and the icons keep their places.
    function drawFence(f, plate, title, pill) {
        const top = TITLE_H * (1 - plate);
        put(f.n, f.x, f.y + top);
        size(f.n, f.w, f.h - top);
        f.body.style.top = TITLE_H - top + 'px';
        f.bar.style.top = -top + 'px';
        f.bar.style.opacity = title;
        f.pill.style.opacity = pill;
    }

    // The page's scrollbar thumb: 12 px gutter, 4 px border, 40 px minimum.
    function drawThumb(n, pg, viewH) {
        const h = Math.max(40, viewH * viewH / pg.contentH);
        const y = (viewH - h) * pg.scroll / (pg.contentH - viewH);
        n.style.height = h.toFixed(1) + 'px';
        n.style.transform = `translateY(${(y + 8 * (1 - pg.enter)).toFixed(2)}px)`;
        n.style.opacity = pg.enter;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);

        // settings: wheel down to 行为, the switch, then the 栅栏 page
        r.gen.scroll = SCROLL * ramp(t, SCROLL_A, SCROLL_B, E.out);
        const page = SM.drawNav(r.w, t, [[0, 'general'], [NAV_AT + .05, 'fences']], [], p);
        drawThumb(r.thumb, page === 'general' ? r.gen : r.fen, r.w.mainH);
        SM.drawToggle(r.toggles[TARGET], t >= TOG_AT);
        const open = t >= SEL_AT && t < PICK_AT;
        SM.drawSelect(r.mode, t >= PICK_AT ? OPTIONS[1] : OPTIONS[0], p, open);
        SM.drawPopup(r.pop, t, POP.x, POP.y, SEL_AT + .02, PICK_AT, p, OPTIONS[0]);

        // fences: glass top edge 167 ms decelerate, title 83 ms linear
        drawFence(r.A, tween(t, r.showA, PLATE_S, E.out), tween(t, r.showA, TITLE_S, E.lin), spanFade(t, r.titleRowA));
        drawFence(r.B, tween(t, r.showB, PLATE_S, E.out), tween(t, r.showB, TITLE_S, E.lin), 0);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at });
    }

    PM.register('title-hover', { duration: D, poster: 3.6, build, render });
})();
