// 5.1 布局快照和备份: tray → 设置… → 布局与备份, name and save a snapshot, drag two fences
// elsewhere (snapping as snap.rs does), then 恢复: the fences jump back (relayout_from_state sets
// the saved bounds directly) and the list gains the automatic 「恢复快照前」 snapshot.
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, spans, spanFade, el, put, show, local, icon, fence, placeFence, menu, drawMenu,
        rowY, guide, drawGuide, pointer, drawPointer, drawChip, taskbar, TRAY, loopFade, T } = PM;

    const D = 12.6;
    const WIN = { x: 372, y: 14, w: 420, h: 392, navW: 120 };
    const WORK = { left: 0, top: 0, right: 800, bottom: 422 };
    const GAP = 8, DIST = 8;
    const SAVED = {
        prog: { x: 8, y: 8, w: 168, h: 122 },
        desk: { x: 184, y: 8, w: 168, h: 122 },
        docs: { x: 8, y: 138, w: 344, h: 122 },
    };
    const NOW = '2026/10/6 10:24';                 // format_local_datetime: short date + time (zh-CN)

    // ---- timeline ----
    const TRAY_CLICK = 1.0, SETTINGS_AT = 2.05, OPEN_AT = 2.15;
    const NAV_AT = 2.9;                             // 布局与备份
    const NAME_CLICK = 3.75, TYPE_AT = 3.9, SAVE_AT = 4.9;
    const DRAG1 = [5.9, 6.8], DRAG2 = [7.35, 8.3];
    const GRAB = { x: 60, y: 16 };
    const RESTORE_AT = 9.2;
    const TOAST = 1.6;                              // the page's toast lasts 2.4 s; shorter so it leaves the backup rows sooner

    // snap.rs snap_among: the shortest pull within DIST towards outer edges a GAP from a nearby
    // fence's, edges level with it, the work area's edges (GAP inside) and its centre lines.
    function snapAmong(r, others) {
        let dx = null, dy = null;
        const cx = c => {
            const d = c - r.x;
            if (Math.abs(d) <= DIST && (dx == null || Math.abs(d) < Math.abs(dx))) dx = d;
        };
        const cy = c => {
            const d = c - r.y;
            if (Math.abs(d) <= DIST && (dy == null || Math.abs(d) < Math.abs(dy))) dy = d;
        };
        for (const o of others) {
            if (r.y < o.y + o.h + DIST && r.y + r.h > o.y - DIST) [o.x + o.w + GAP, o.x - GAP - r.w, o.x, o.x + o.w - r.w].forEach(cx);
            if (r.x < o.x + o.w + DIST && r.x + r.w > o.x - DIST) [o.y + o.h + GAP, o.y - GAP - r.h, o.y, o.y + o.h - r.h].forEach(cy);
        }
        [WORK.left + GAP, WORK.right - GAP - r.w, Math.trunc((WORK.left + WORK.right) / 2 - r.w / 2)].forEach(cx);
        [WORK.top + GAP, WORK.bottom - GAP - r.h, Math.trunc((WORK.top + WORK.bottom) / 2 - r.h / 2)].forEach(cy);
        return { ...r, x: r.x + (dx ?? 0), y: r.y + (dy ?? 0) };
    }
    // alignment_guides: only the screen centre lines can line up on these paths.
    const centreGuideY = r => (r.y === Math.trunc(211 - r.h / 2) ? 211 : null);

    let PATH;
    const at = t => pathAt(t, PATH);

    // Fence rectangles at t (and the dragged one's centre guide).
    function layout(t) {
        const L = { prog: { ...SAVED.prog }, desk: { ...SAVED.desk }, docs: { ...SAVED.docs }, guide: null };
        if (t >= RESTORE_AT + .02) return L;
        const drag = (key, [a, b]) => {
            if (t < a) return;
            const p = at(Math.min(t, b));
            const others = Object.keys(SAVED).filter(k => k !== key).map(k => L[k]);
            L[key] = snapAmong({ ...L[key], x: p.x - GRAB.x, y: p.y - GRAB.y }, others);
            if (t < b) L.guide = centreGuideY(L[key]);
        };
        drag('docs', DRAG1);
        drag('prog', DRAG2);
        return L;
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;

        // desktop
        r.F = {
            prog: fence(S, { ...SAVED.prog, title: '程序' }),
            desk: fence(S, { ...SAVED.desk, title: '桌面' }),
            docs: fence(S, { ...SAVED.docs, title: '文件与文档' }),
        };
        const fill = (f, list, c) => list.forEach(([k, l], i) => {
            const s = local(i, c);
            put(icon(f.body, k, l), s.x, s.y);
        });
        fill(r.F.prog, [['url', '项目主页'], ['exe', '截图工具']], 2);
        fill(r.F.desk, [['txt', '安装日志.log'], ['folder', '素材']], 2);
        fill(r.F.docs, [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['docx', '方案.docx']], 4);
        r.g = guide(S);
        r.g.style.zIndex = 1;      // guide windows sit just above the dragged fence, under other windows
        r.note = el('div', 'chip', S);

        // settings window: 常规 (as it opens) and 布局与备份
        const w = r.w = SM.win(S, WIN);
        w.foot.textContent = T('{0} 个栅栏 · {1} 个项目').replace('{0}', '3').replace('{1}', '8');
        ({ tiles: r.tiles } = SM.generalPage(w));
        const pg = r.pg = SM.page(w, 'layout');
        SM.header(pg, '布局与备份');
        SM.h3(pg, '布局快照');
        const save = SM.card(pg, { ic: '\uE722', t: '把当前布局存为快照', s: '记录所有栅栏的位置、大小和内容，随时恢复', h: 70, gap: 1, textW: 220 });
        save.t.style.top = '9px';
        save.s.style.top = '24px';
        save.n.querySelector('.set-ic').style.top = '10px';
        save.n.style.borderRadius = '4px 4px 0 0';
        // the button sizes to its label; the name box sits 8 px left of it
        r.save = SM.button(save, '保存快照', 62, { accent: true, right: 10, top: 40 });
        const nameRight = 18 + r.save.w;
        r.name = SM.input(save, '快照名称（可选）', Math.min(132, pg.cardW - nameRight - 12), { right: nameRight, top: 40 });
        r.empty = SM.card(pg, { s: '尚无快照', h: 34, indent: 22, textW: 200 });
        r.empty.s.style.top = '10px';
        const now = SM.dateTime(NOW, new Date(2026, 9, 6, 10, 24));
        const snapRow = (name, gap) => {
            const b = SM.card(pg, { t: name, s: `${now} · ${T('{0} 个栅栏').replace('{0}', '3')}`, h: 40, indent: 22, textW: 140, gap });
            b.restore = SM.button(b, '恢复', 46, { right: 36 });
            b.del = SM.iconButton(b, 'del', { right: 10 });
            return b;
        };
        r.before = snapRow(T('恢复快照前'), 1);
        r.before.n.style.borderRadius = '0';
        r.saved = snapRow('整理好了', 3);
        [r.empty, r.saved].forEach(b => { b.n.style.borderRadius = '0 0 4px 4px'; });
        SM.h3(pg, '备份');
        const io = SM.card(pg, { ic: '\uE78C', t: '导出 / 导入配置文件', s: '导出为 JSON 以便迁移到另一台电脑；导入前会自动保存一份快照', h: 70, gap: 1, textW: 220 });
        io.t.style.top = '9px';
        io.s.style.top = '24px';
        io.n.querySelector('.set-ic').style.top = '10px';
        io.n.style.borderRadius = '4px 4px 0 0';
        const imp = SM.button(io, '导入…', 56, { right: 10, top: 40 });
        SM.button(io, '导出…', 56, { right: 16 + imp.w, top: 40 });
        // 配置文件: where the configuration is (#cfgPath), then the daily backups
        const cfg = SM.card(pg, { ic: '\uE8A5', t: '配置文件', s: 'C:\\Users\\Lin\\AppData\\Roaming\\PecoFence\\config.json', h: 40, gap: 1, textW: 200 });
        cfg.n.style.borderRadius = '0';
        SM.button(cfg, '打开配置文件夹', 70, { right: 10 });
        // daily backups are listed by file name (backups/YYYY-MM-DD.json) in every language
        ['2026-10-06', '2026-10-05', '2026-10-04'].forEach((day, i, all) => {
            const b = SM.card(pg, { t: day, s: '每日自动备份', h: 40, indent: 22, textW: 140, gap: i < all.length - 1 ? 1 : 3 });
            b.n.style.borderRadius = i < all.length - 1 ? '0' : '0 0 4px 4px';
            SM.button(b, '恢复', 46, { right: 10 });
        });

        r.tray = menu(S, ['隐藏所有栅栏', '临时显示 Windows 桌面', '新建栅栏', { t: '在桌面显示文件夹', sub: 1 },
            { t: '速览所有栅栏', key: 'Ctrl+Alt+空格' }, '立即应用整理规则', '-', '恢复显示桌面图标',
            '修复桌面图标（图标消失时使用）', '发送反馈…', '设置…', '-', '退出 PecoFence'], 236);
        r.tray.n.style.zIndex = 3;
        r.trayAt = { x: TRAY.x - 236, y: TRAY.y - 10 - r.tray.h };
        // the 右键 caption goes above the tray icon, just right of the menu (which grows for longer
        // translations, up to the stage edge); one too wide to clear the menu is gone by the time
        // the menu has faded in
        const menuRight = Math.min(r.trayAt.x + r.tray.w, PM.SW - 4);
        const probe = el('div', 'chip', S, T('右键'));
        r.trayChip = { x: menuRight + 4, end: menuRight + 4 + SM.natW(probe) > PM.SW - 4 ? TRAY_CLICK : 1.25 };
        probe.remove();
        r.ptr = pointer(S);
        taskbar(root);

        plan(r);
        r.guideSpans = spans(D, t => layout(t).guide != null);
        return r;
    }

    const st = t => ({
        name: t >= SAVE_AT + .05 ? '' : SM.typed(t, TYPE_AT, '整理好了', .13, .4),
        focus: t >= NAME_CLICK && t < SAVE_AT + .05,
        saved: t >= SAVE_AT + .05,
        restored: t >= RESTORE_AT + .02,
    });
    function layoutPage(r, s) {
        r.empty.on = !s.saved;
        r.saved.on = s.saved;
        r.before.on = s.restored;
        r.pg.blocks = r.pg.blocks.filter(b => b !== r.before);
        r.pg.blocks.splice(r.pg.blocks.indexOf(r.saved), 0, r.before);   // newest first
        SM.stack(r.pg);
    }

    let CLICKS, PRESSES, CHIPS;
    function plan(r) {
        const pos = (t, c, dx = 0, dy = 0) => {
            layoutPage(r, st(t));
            return SM.at(c, dx, dy);
        };
        const set = { x: r.trayAt.x + 90, y: r.trayAt.y + rowY(r.tray, '设置…') };
        const nav = SM.navAt(r.w, 'layout');
        const name = pos(NAME_CLICK, r.name, Math.min(44, r.name.w / 2 - 6), 0);   // right of the text being typed
        const save = pos(SAVE_AT, r.save);
        const restore = pos(RESTORE_AT, r.saved.restore);
        const g = k => ({ x: SAVED[k].x + GRAB.x, y: SAVED[k].y + GRAB.y });
        PATH = [
            [0, 300, 380], [0.3, 300, 380],
            [0.9, TRAY.x, TRAY.y], [1.3, TRAY.x, TRAY.y],         // right-click the tray icon
            [1.8, set.x, set.y], [2.2, set.x, set.y],             // 设置…
            [2.8, nav.x, nav.y], [3.05, nav.x, nav.y],            // 布局与备份
            [3.65, name.x, name.y], [4.45, name.x, name.y],       // name the snapshot
            [4.8, save.x, save.y], [5.1, save.x, save.y],         // 保存快照
            [5.8, g('docs').x, g('docs').y], [DRAG1[0], g('docs').x, g('docs').y],
            [6.7, 84, 294], [6.85, 84, 294],                      // 文件与文档 to the bottom
            [7.3, g('prog').x, g('prog').y], [DRAG2[0], g('prog').x, g('prog').y],
            [7.8, 70, 159], [8.2, 170, 159], [8.4, 170, 159],     // 程序 down, then right
            [9.1, restore.x, restore.y], [9.55, restore.x, restore.y], // 恢复
            [10.35, 330, 386], [D, 330, 386],
        ];
        CLICKS = [{ t: TRAY_CLICK, right: true }, { t: SETTINGS_AT }, { t: NAV_AT }, { t: NAME_CLICK }, { t: SAVE_AT },
            { t: RESTORE_AT }];
        PRESSES = [DRAG1, DRAG2];
        // the second drag starts at the top of the stage, so its chip goes on the title bar
        CHIPS = [[0.75, r.trayChip.end, '右键', r.trayChip.x, TRAY.y - 46, 'abs'], [DRAG1[0], DRAG1[1] - .1, '按住标题栏拖动', 16, -34],
            [DRAG2[0], DRAG2[1] - .1, '按住标题栏拖动', 22, -10]];
        layoutPage(r, st(0));
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const s = st(t);

        // ---- desktop ----
        const L = layout(t);
        for (const k of Object.keys(r.F)) placeFence(r.F[k], L[k].x, L[k].y, L[k].w, L[k].h);
        drawGuide(r.g, spanFade(t, r.guideSpans), false, 211, WORK.left, WORK.right);
        drawChip(r.note, t, [[RESTORE_AT + .05, RESTORE_AT + 1.6, '回到保存时的样子', 116, 300, 'abs']], p);

        // ---- tray menu, then the settings window opening on 常规 ----
        drawMenu(r.tray, t, r.trayAt.x, r.trayAt.y, 1.05, SETTINGS_AT + .03, p, true);
        if (show(r.w.n, t >= OPEN_AT)) {
            const o = ramp(t, OPEN_AT, OPEN_AT + .2);
            put(r.w.n, WIN.x, WIN.y, o, ` scale(${lerp(.96, 1, o)})`);
        }
        SM.drawNav(r.w, t, [[0, 'general'], [NAV_AT + .03, 'layout']], [], p);
        SM.drawTiles(r.tiles, false, p);
        layoutPage(r, s);
        SM.drawInput(r.name, s.name, s.focus, t);
        SM.drawButton(r.save, p, t >= SAVE_AT - .08 && t < SAVE_AT + .02);
        for (const b of [r.saved, r.before]) SM.drawButton(b.restore, p, b === r.saved && t >= RESTORE_AT - .08 && t < RESTORE_AT + .02);
        SM.drawToast(r.w, t, [[SAVE_AT + .05, T('已保存快照'), TOAST], [RESTORE_AT + .05, T('已恢复快照'), TOAST]]);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, presses: PRESSES, chips: CHIPS, at });
    }

    PM.register('backup', { duration: D, poster: 9.75, build, render });
})();
