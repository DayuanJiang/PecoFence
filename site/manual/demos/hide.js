// 3.2 双击桌面隐藏栅栏: a double-click on the bare desktop fades every fence out (167 ms linear;
// the toggle fires when the second click is released) except one set to 双击桌面隐藏时保留此栅栏;
// another double-click brings them back; the tray menu's 隐藏所有栅栏 does the same.
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, show, local, icon, fence, menu, drawMenu, rowY, pointer,
        drawPointer, drawChip, taskbar, TRAY, loopFade } = PM;

    const D = 8.8;
    const FADE = .2;        // 167 ms linear, slowed a touch
    const FENCES = [
        { x: 40, y: 40, w: 246, h: 124, title: '程序', cols: 3,
            items: [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']] },
        { x: 40, y: 184, w: 324, h: 204, title: '文件与文档', cols: 4,
            items: [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['txt', '会议记录.txt'],
                ['pdf', '发票.pdf'], ['docx', '方案.docx']] },
        { x: 306, y: 40, w: 168, h: 124, title: '常用', cols: 2, keep: true,
            items: [['txt', '待办.txt'], ['xlsx', '日程.xlsx']] },
        { x: 494, y: 40, w: 246, h: 124, title: '文件夹', cols: 3,
            items: [['folder', '工作资料'], ['folder', '照片'], ['folder', '素材']] },
    ];
    const KEEP = FENCES.find(f => f.keep);
    const SPOT = { x: 604, y: 288 };     // an empty spot of the desktop

    const MENU_W = 236;
    let trayMenu = null;    // built once; its height places it and aims the pointer
    const menuBox = () => ({ x: TRAY.x - MENU_W, y: TRAY.y - 10 - trayMenu.h });

    const DBL1 = [1.05, 1.25], DBL2 = [3.75, 3.95], TRAY_RCLICK = 5.55, HIDE_CLICK = 6.4;
    // Hide / show toggles: [time, hidden] — on the release of the second click, and on the
    // menu click.
    const TOGGLES = [[DBL1[1] + .08, 1], [DBL2[1] + .08, 0], [HIDE_CLICK + .03, 1]];
    const hiddenAt = t => {
        let v = 0;
        for (const [te, to] of TOGGLES) {
            if (t < te) break;
            v = lerp(v, to, ramp(t, te, te + FADE, E.lin));
        }
        return v;
    };

    let PATH, CLICKS, CHIPS, NOTE;
    function plan() {
        const m = menuBox();
        const item = { x: m.x + 150, y: m.y + rowY(trayMenu, '隐藏所有栅栏') };
        PATH = [
            [0, 560, 404], [0.3, 560, 404],
            [0.9, SPOT.x, SPOT.y], [4.5, SPOT.x, SPOT.y],
            [5.25, TRAY.x, TRAY.y], [5.7, TRAY.x, TRAY.y],     // right-click the tray icon
            [6.2, item.x, item.y], [6.55, item.x, item.y],     // 隐藏所有栅栏
            [7.2, 600, 330], [D, 600, 330],
        ];
        CLICKS = [{ t: DBL1[0] }, { t: DBL1[1] }, { t: DBL2[0] }, { t: DBL2[1] }, { t: TRAY_RCLICK, right: true },
            { t: HIDE_CLICK }];
        CHIPS = [[0.95, 1.6, '双击空白处'], [3.65, 4.3, '再双击一次'], [5.3, 5.75, '右键', 12, -34]];
        NOTE = [[1.75, 3.45, '保留此栅栏', KEEP.x + 34, KEEP.y + KEEP.h + 8, 'abs']];
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.fences = FENCES.map(f => {
            const o = fence(S, { ...f });
            f.items.forEach(([k, l], i) => {
                const s = local(i, f.cols);
                put(icon(o.body, k, l), s.x, s.y);
            });
            return o;
        });
        r.note = el('div', 'chip', S);
        taskbar(root);
        trayMenu = menu(S, ['隐藏所有栅栏', '临时显示 Windows 桌面', '新建栅栏', { t: '在桌面显示文件夹', sub: 1 },
            { t: '速览所有栅栏', key: 'Ctrl+Alt+空格' }, '立即应用整理规则', '-', '恢复显示桌面图标',
            '修复桌面图标（图标消失时使用）', '发送反馈…', '设置…', '-', '退出 PecoFence'], MENU_W);
        r.menu = trayMenu;
        r.ptr = pointer(S);
        plan();
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);
        const hidden = hiddenAt(t);
        r.fences.forEach((f, i) => {
            const o = FENCES[i].keep ? 1 : 1 - hidden;
            if (show(f.n, o > 0)) put(f.n, f.x, f.y, o);
        });
        drawChip(r.note, t, NOTE, p);

        const m = menuBox();
        drawMenu(r.menu, t, m.x, m.y, TRAY_RCLICK + .05, HIDE_CLICK + .03, p, true);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('hide', { duration: D, poster: 2.4, build, render });
})();
