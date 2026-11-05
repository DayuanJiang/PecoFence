// 7.2 对话框跳转 (lesson id dialog-jump; dialog_jump.rs): a program's Open dialog opens in its own
// folder (文档) while File Explorer shows 合同 behind it. A click on Explorer and one back on the
// dialog make the dialog jump, once, to Explorer's folder; pressing the 设计稿 folder fence with
// the dialog in front jumps it there at once (the fence takes no activation, so the dialog stays
// in front). A jump opens the dialog's address bar, types the path there and presses Enter
// (dialogs.rs), so the path shows in the address bar for a moment before the list changes.
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, spans, spanFade, el, put, size, show, text, local, icon, fence, pointer,
        drawPointer, taskbar, loopFade } = PM;

    const D = 8.6;
    const EXP = { x: 300, y: 16, w: 486, h: 236 };
    const DLG = { x: 24, y: 48, w: 456, h: 344 };
    const FF = { x: 540, y: 270, w: 246, h: 124, title: '设计稿' };

    // folders: breadcrumb segments and what the dialog lists there
    const PC = '此电脑', DRIVE = '本地磁盘 (D:)', WORK = '工作资料';
    const FOLDERS = {
        docs: { crumbs: [PC, '文档'], items: [['folder', '自定义 Office 模板'], ['docx', '简历.docx'], ['pdf', '说明书.pdf']] },
        contracts: { crumbs: [DRIVE, WORK, '合同'], path: [WORK, '合同'],
            items: [['pdf', '合同 A.pdf'], ['pdf', '合同 B.pdf'], ['pdf', '签字页.pdf'], ['docx', '补充协议.docx']] },
        design: { crumbs: [DRIVE, WORK, FF.title], path: [WORK, FF.title],
            items: [['tabs-tide', '海报 A.png'], ['tabs-sol', '海报 B.png'], ['tabs-terra', '封面.png']] },
    };
    // Explorer's navigation pane, shortened (图片 is left out: the rules demo uses that word for
    // its 图片 category, which translates differently from the Pictures folder).
    const PLACES = ['主文件夹', '桌面', '下载', '文档', PC];

    const DLG_OPEN = .35;
    const CLICK_EXP = 2.0, CLICK_DLG = 3.0, CLICK_FENCE = 5.3;
    // a jump: the typed path shows from a, the dialog lists the new folder from b
    const JUMPS = [{ a: CLICK_DLG + .06, b: CLICK_DLG + .42, to: 'contracts' }, { a: CLICK_FENCE + .06, b: CLICK_FENCE + .42, to: 'design' }];

    const PATH = [
        [0, 600, 400], [0.4, 600, 400],
        [1.55, 690, 150], [2.25, 690, 150],                 // click File Explorer
        [2.75, 150, 290], [3.5, 150, 290],                  // click back on the dialog
        [4.75, 720, 287], [5.6, 720, 287],                  // press the folder fence (its title row)
        [6.4, 610, 410], [D, 610, 410],
    ];
    const CLICKS = [{ t: CLICK_EXP }, { t: CLICK_DLG }, { t: CLICK_FENCE }];
    const CHIPS = [[1.75, 2.35, '切到资源管理器', -40, 22], [2.8, 3.4, '切回对话框', 16, 22], [4.9, 5.7, '点文件夹栅栏', FF.x + FF.w, FF.y - 32, 'abs', 'right']];

    const SVG = (vb, d, w = 1.1) => `<svg viewBox="${vb}" aria-hidden="true"><path d="${d}" fill="none" stroke="currentColor" ` +
        `stroke-width="${w}" stroke-linecap="round" stroke-linejoin="round"/></svg>`;
    const CLOSE = SVG('0 0 10 10', 'M1 1l8 8M9 1 1 9', 1);
    const CAPS = '<div class="pk-caps"><i>' + SVG('0 0 10 10', 'M.5 5h9', 1) + '</i><i>' +
        SVG('0 0 10 10', 'M1 1h8v8H1z', 1) + '</i><i>' + CLOSE + '</i></div>';
    const NAV = '<span class="dj-nav">' + SVG('0 0 14 14', 'M12.5 7h-11M6 2.5 1.5 7 6 11.5') + '</span>' +
        '<span class="dj-nav off">' + SVG('0 0 14 14', 'M1.5 7h11M8 2.5l4.5 4.5L8 11.5') + '</span>' +
        '<span class="dj-nav">' + SVG('0 0 14 14', 'M7 12.5v-11M2.5 6 7 1.5 11.5 6') + '</span>';
    const MAG = SVG('0 0 12 12', 'M8 8l3 3M5 9a4 4 0 1 1 0-8 4 4 0 0 1 0 8z');
    const FOLDER_GLYPH = '<svg viewBox="0 0 14 14" aria-hidden="true"><path d="M1.5 3.2c0-.4.3-.7.7-.7h3.1l1.4 1.5h5.1' +
        'c.4 0 .7.3.7.7v6.6c0 .4-.3.7-.7.7H2.2a.7.7 0 0 1-.7-.7z" fill="none" stroke="currentColor" stroke-width="1.05" ' +
        'stroke-linejoin="round"/></svg>';
    const iconSrc = k => `assets/icons/${k}.png`;

    // The command row of both windows: back / forward / up, the address bar (breadcrumbs, or the
    // typed path while a jump runs) and the search box.
    function commandRow(parent) {
        const row = el('div', 'dj-cmd', parent);
        row.innerHTML = NAV;
        const addr = el('div', 'dj-addr', row);
        const crumbs = el('div', 'dj-crumbs', addr);
        const typed = el('div', 'dj-typed', addr);
        el('div', 'dj-search', row).innerHTML = MAG;
        return { crumbs, typed, shown: null };
    }

    function drawCrumbs(c, list) {
        const id = list.join('/');
        if (c.shown === id) return;
        c.shown = id;
        c.crumbs.replaceChildren();
        list.forEach((s, i) => {
            if (i) el('i', null, c.crumbs, '›');
            el('span', null, c.crumbs, s);
        });
    }

    function explorer(parent) {
        const n = el('div', 'pk-win dj-exp', parent);
        const tabs = el('div', 'dj-tabs', n);
        const tab = el('div', 'dj-tab', tabs);
        const img = el('img', null, tab);
        img.src = iconSrc('folder');
        img.alt = '';
        el('span', null, tab, '合同');
        tabs.insertAdjacentHTML('beforeend', CAPS);
        const cmd = commandRow(n);
        drawCrumbs(cmd, FOLDERS.contracts.crumbs);
        const body = el('div', 'dj-icons', n);
        FOLDERS.contracts.items.forEach(([k, l], i) => {
            const ic = icon(body, k, l);
            const s = local(i, 5);
            ic.style.left = s.x + 4 + 'px';
            ic.style.top = s.y + 'px';
        });
        put(n, EXP.x, EXP.y);
        size(n, EXP.w, EXP.h);
        return n;
    }

    function dialog(parent) {
        const n = el('div', 'pk-win dj-dlg', parent);
        const bar = el('div', 'pk-titlebar', n);
        el('span', null, bar, '打开');
        bar.insertAdjacentHTML('beforeend', '<div class="pk-caps"><i>' + CLOSE + '</i></div>');
        const cmd = commandRow(n);
        const mid = el('div', 'dj-mid', n);
        const pane = el('div', 'dj-pane', mid);
        PLACES.forEach(p => {
            const it = el('div', 'dj-place', pane);
            const img = el('img', null, it);
            img.src = iconSrc('folder');
            img.alt = '';
            el('span', null, it, p);
        });
        const list = el('div', 'dj-list', mid);
        el('div', 'dj-colhead', list, '名称');
        const views = {};
        for (const [key, f] of Object.entries(FOLDERS)) {
            const v = el('div', 'dj-rows', list);
            f.items.forEach(([k, l]) => {
                const r = el('div', 'dj-row', v);
                const img = el('img', null, r);
                img.src = iconSrc(k);
                img.alt = '';
                el('span', null, r, l);
            });
            views[key] = v;
        }
        const foot = el('div', 'dj-foot', n);
        const line = el('div', 'dj-fline', foot);
        el('span', 'dj-flabel', line, '文件名(N):');
        el('div', 'dj-finput', line);
        el('div', 'dj-ftype', line, '所有文件 (*.*)');
        const btns = el('div', 'dj-btns', foot);
        el('div', 'dj-btn acc', btns, '打开(O)');
        el('div', 'dj-btn', btns, '取消');
        put(n, DLG.x, DLG.y, 0);
        size(n, DLG.w, DLG.h);
        return { n, cmd, views };
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.F = fence(S, { ...FF });
        r.F.name.style.paddingLeft = '24px';
        const fold = el('div', 'pk-fold', r.F.bar);
        fold.innerHTML = FOLDER_GLYPH;
        fold.style.left = '13px';
        FOLDERS.design.items.forEach(([k, l], i) => {
            const s = local(i, 3);
            put(icon(r.F.body, iconSrc(k), l), s.x, s.y);
        });
        r.exp = explorer(S);
        r.dlg = dialog(S);
        r.ptr = pointer(S);
        taskbar(root);
        r.titleHover = spans(D, t => inside(pathAt(t, PATH), { x: FF.x, y: FF.y, w: FF.w, h: 32 }));
        return r;
    }

    // The folder the dialog shows at t, and the path being typed into its address bar (or null).
    function dialogFolder(t) {
        let folder = 'docs', typing = null;
        for (const j of JUMPS) {
            if (t >= j.b) folder = j.to;
            else if (t >= j.a) typing = j;
        }
        return { folder, typing };
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);

        // z-order: the dialog in front, File Explorer between the click on it and the click back
        const expFront = t >= CLICK_EXP && t < CLICK_DLG;
        r.exp.style.zIndex = expFront ? 4 : 2;
        r.dlg.n.style.zIndex = expFront ? 2 : 4;
        r.exp.classList.toggle('dj-inactive', !expFront);
        r.dlg.n.classList.toggle('dj-inactive', expFront);

        // the dialog opens like a Windows 11 window: fades in while growing from 94 %
        if (show(r.dlg.n, t >= DLG_OPEN)) {
            put(r.dlg.n, DLG.x, DLG.y, ramp(t, DLG_OPEN, DLG_OPEN + .2, E.lin),
                ` scale(${lerp(.94, 1, ramp(t, DLG_OPEN, DLG_OPEN + .3)).toFixed(4)})`);
        }
        const { folder, typing } = dialogFolder(t);
        const c = r.dlg.cmd;
        drawCrumbs(c, FOLDERS[folder].crumbs);
        show(c.crumbs, !typing);
        if (show(c.typed, !!typing)) text(c.typed, 'D:\\' + FOLDERS[typing.to].path.join('\\'));
        c.typed.parentNode.classList.toggle('edit', !!typing);
        for (const [key, v] of Object.entries(r.dlg.views)) show(v, key === folder);

        // the folder fence's title row: the hover pill while the pointer is on it (83 ms)
        r.F.pill.style.opacity = spanFade(t, r.titleHover);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('dialog-jump', { duration: D, poster: 4.0, build, render });
})();
