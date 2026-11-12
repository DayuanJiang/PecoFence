// 1.1 第一次打开: the cluttered desktop sorts itself into the first-run fences (state.rs
// default_fences: 程序 / 文件夹 / 文件与文档 in a column at the right edge; the 桌面 inbox is
// 像桌面一样排列 with 自由摆放 on a new configuration, so what no rule takes stays a desktop icon
// in the column and row it had on the Windows desktop, desktop_import.rs), a new file lands in
// its fence, and quitting from the tray puts the desktop back.
(() => {
    'use strict';
    const { E, bezier, ramp, lerp, pathAt, el, put, show, icon, fence, menu, drawMenu, rowY, pointer,
        drawPointer, drawChip, taskbar, TRAY, loopFade, slot } = PM;

    const D = 12.2;
    const Z = .85;                      // the desktop is shown zoomed out
    const fly = bezier(.3, 0, .1, 1);

    // Desktop icon grid (zoom coordinates) and the three fences at the right edge.
    const grid = (c, r) => ({ x: 4 + c * 80, y: 4 + r * 88 });
    // The column stands one snapping gap (8 px) from the screen's edges and its fences one gap
    // apart (state.rs default_fences), drawn as 5 here: the demo's fences are about 0.6 of theirs.
    const GAP = 5;
    const FX = 941 - GAP - 246, FH = 111.5;
    const FENCES = ['程序', '文件夹', '文件与文档'].map((title, i) => ({ x: FX, y: GAP + i * (FH + GAP), w: 246, h: FH, title }));
    const INBOX = 3;    // the arranged 桌面: its icons keep their places on the desktop
    // [icon, label, desktop column, row, fence, slot]
    const ITEMS = [
        ['assets/mark.svg', 'PecoFence', 0, 0, 0, 0],
        ['pdf', '合同.pdf', 0, 1, 2, 0],
        ['folder', '工作资料', 0, 2, 1, 0],
        ['exe', '截图工具', 0, 3, 0, 1],
        ['docx', '周报.docx', 0, 4, 2, 1],
        ['folder', '照片', 1, 0, 1, 1],
        ['url', '项目主页', 1, 1, 0, 2],
        ['txt', '安装日志.log', 1, 2, 3, 0],
        ['folder', '2026 报销', 1, 3, 1, 2],
    ];
    const NEW = ['xlsx', '预算.xlsx', 1, 4, 2, 2];
    const NEW_AT = 5.3;
    const QUIT_AT = 8.85;

    const MENU_W = 236;
    let trayMenu = null;    // built once; its height sets the pointer path
    const menuBox = () => ({ x: TRAY.x - MENU_W, y: TRAY.y - 10 - trayMenu.h });

    let PATH, CLICKS, CHIPS;
    function plan() {
        const m = menuBox();
        const quit = { x: m.x + 170, y: m.y + rowY(trayMenu, '退出 PecoFence') };
        PATH = [
            [0, 520, 300], [0.3, 520, 300],
            [1.0, 37, 25], [1.5, 37, 25],                  // double-click PecoFence
            [2.2, 430, 250], [6.9, 430, 250],
            [7.6, TRAY.x, TRAY.y], [8.05, TRAY.x, TRAY.y], // right-click the tray icon
            [8.6, quit.x, quit.y], [9.2, quit.x, quit.y],  // 退出 PecoFence
            [9.9, 470, 230], [D, 470, 230],
        ];
        CLICKS = [{ t: 1.12 }, { t: 1.3 }, { t: 7.8, right: true }, { t: QUIT_AT }];
        CHIPS = [[1.05, 1.45, '双击'], [7.55, 7.9, '右键', 12, -34]];
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        r.zoom = el('div', 'zoom', r.scene);
        r.zoom.style.transform = `scale(${Z})`;
        r.fences = FENCES.map(f => fence(r.zoom, { ...f }));
        r.icons = [...ITEMS, NEW].map(([k, l]) => icon(r.zoom, k, l));
        r.pulse = el('div', 'ring pulse', r.scene);
        r.note = el('div', 'chip', r.scene);
        r.tb = taskbar(root);
        trayMenu = menu(r.scene, ['隐藏所有栅栏', '临时显示 Windows 桌面', '新建栅栏', '新建便签', { t: '新建小工具', sub: 1 },
            { t: '在桌面显示文件夹', sub: 1 }, '新建智能栅栏…', { t: '速览所有栅栏', key: 'Ctrl+Alt+空格' },
            '立即应用整理规则', '-', '恢复显示桌面图标', '修复桌面图标（图标消失时使用）', '发送反馈…', '设置…', '-',
            '退出 PecoFence'], MENU_W);
        r.menu = trayMenu;
        r.ptr = pointer(r.scene);
        plan();
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);

        // PecoFence starts: tray icon, then the fences fade in one after another
        const on = ramp(t, 1.6, 1.85), off = ramp(t, QUIT_AT + .15, QUIT_AT + .35, E.lin);
        put(r.tb.mark, 0, 0, on * (1 - off), ` scale(${lerp(.4, 1, on)})`);
        r.fences.forEach((f, i) => {
            const o = ramp(t, 1.9 + i * .08, 2.15 + i * .08) * (1 - ramp(t, QUIT_AT + .1, QUIT_AT + .27, E.lin));
            if (show(f.n, o > 0)) put(f.n, f.x, f.y, o, ` scale(${lerp(.97, 1, Math.min(1, o * 1.5))})`);
        });

        // icons fly into their fences on start and back to the desktop grid on quit
        [...ITEMS, NEW].forEach(([, , c, row, fi, si], i) => {
            const n = r.icons[i];
            const home = grid(c, row), dest = fi === INBOX ? home : slot(FENCES[fi], si, 3);   // INBOX: it stays
            const isNew = i === ITEMS.length;
            const inP = isNew ? 1 : fly(PM.clamp01((t - (2.35 + i * .11)) / .62));
            const outP = fly(PM.clamp01((t - (QUIT_AT + .2 + i * .05)) / .55));
            let x, y, o = 1;
            if (fi === INBOX) {
                ({ x, y } = home);
            } else if (isNew && t < QUIT_AT + .2) {
                o = t < NEW_AT ? 0 : ramp(t, NEW_AT, NEW_AT + .167, E.lin);
                ({ x, y } = dest);
            } else if (outP > 0) {
                x = lerp(dest.x, home.x, outP);
                y = lerp(dest.y, home.y, outP) - Math.sin(Math.PI * outP) * 26;
            } else {
                x = lerp(home.x, dest.x, inP);
                y = lerp(home.y, dest.y, inP) - Math.sin(Math.PI * inP) * 26;
            }
            const onDesk = fi === INBOX || (isNew ? outP > .5 : (outP > .5 || inP < .5));
            n.classList.toggle('desk', onDesk);
            n.classList.toggle('sel', i === 0 && t >= 1.12 && t < 2.3);
            put(n, x, y, o);
        });

        // the new file: pulse around it and a caption
        const d = slot(FENCES[2], 2, 3);
        const cx = (d.x + 39) * Z, cy = (d.y + 24) * Z;
        if (show(r.pulse, t >= NEW_AT && t < NEW_AT + .8)) {
            const q = (t - NEW_AT) / .8;
            put(r.pulse, cx, cy, (1 - q) * .9, ` scale(${.6 + E.out(q) * 1.1})`);
        }
        drawChip(r.note, t, [[NEW_AT, NEW_AT + 1.4, '新文件', FX * Z - 8, cy - 11, 'abs', 'right']], p);

        // quit from the tray menu
        const m = menuBox();
        drawMenu(r.menu, t, m.x, m.y, 7.85, QUIT_AT + .03, p, true);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('first-run', { duration: D, poster: 4.4, build, render });
})();
