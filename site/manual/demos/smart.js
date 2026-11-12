// 4.6 智能栅栏 (lesson id smart): 添加到桌面 › 新建智能栅栏… in a fence's menu opens the editor, a
// window of its own (ui/settings.html #seWindow, 880 x 660 DIP: settings_host.rs): a name, five
// templates, the conditions row by row and the live count. A click on 「最近 7 天下载」 fills the name and the rows (the Downloads
// folder with subfolders, the last 7 days, files); 「创建」 makes the fence at the first free spot
// from the top left, since the spot by the menu is taken (fences.rs place_new_fence). Its title
// has the folder glyph with a magnifier and the count in a capsule; the files are newest first,
// and a new download shows up by itself (smart.rs watches the folder).
(() => {
    'use strict';
    const { SW, SH, E, ramp, lerp, inside, pathAt, el, put, size, show, text, local, slot, deskCell, icon, fence, menu,
        drawMenu, rowY, pointer, drawPointer, drawChip, taskbar, loopFade, T } = PM;

    const D = 9.8;
    const F0 = { x: 530, y: 24, w: 246, h: 124, title: '文件与文档',
        items: [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx']] };
    const WIN = { x: 145, y: 18, w: 510, h: 384 };      // 880 x 660 at 0.58
    // 4 columns x 260 DIP beside the loose icons' column (the first free spot from the top left)
    const S0 = { x: 92, y: 12, w: 324, h: 204, title: '最近 7 天下载' };
    const COLS = 4;
    // what the search finds, newest first; 发票.pdf is downloaded later
    const FOUND = [['pdf', '季度报告.pdf'], ['docx', '方案.docx'], ['assets/icons/icons-poster.png', '海报.png'],
        ['exe', '安装程序.exe'], ['xlsx', '数据表.xlsx'], ['txt', '说明.txt']];
    const NEW = ['pdf', '发票.pdf'];

    const RC = 1.05, SUB_AT = 1.9, PICK = 2.65;             // right-click, 添加到桌面, 新建智能栅栏…
    const ED_AT = 2.8, TEMPLATE = 3.75, COUNTED = 4.05, CREATE = 5.2;
    const MADE = 5.32, NEW_AT = 7.3;

    const MENU_ITEMS = ['收起', '重命名…', '-', { t: '视图', sub: 1 }, { t: '排序方式', sub: 1 }, { t: '新建', sub: 1 },
        { t: '粘贴', key: 'Ctrl+V', dis: 1 }, { t: '标签页', sub: 1 }, '-', '适应内容大小', '锁定位置和大小', '栅栏选项…', '删除栅栏', '-',
        { t: '添加到桌面', sub: 1 }];
    const SUB_ITEMS = ['新建栅栏', '新建便签', { t: '新建小工具', sub: 1 }, { t: '在桌面显示文件夹', sub: 1 }, '新建智能栅栏…'];
    const GRAB = { x: F0.x + 150, y: F0.y + 16 };

    const svg = (vb, body) => `<svg viewBox="${vb}" aria-hidden="true">${body}</svg>`;
    const line = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round"';
    const FIND = svg('0 0 12 12', `<circle cx="5.2" cy="5.2" r="3.6" ${line} stroke-width="1.2"/><path d="m8 8 2.8 2.8" ${line} stroke-width="1.2"/>`);
    // Folder glyph with the accent magnifier at its lower right (fence_chrome.rs smart mark).
    const MARK = svg('0 0 16 14', `<path d="M1.5 3.2c0-.4.3-.7.7-.7h3.1l1.4 1.5h5.1c.4 0 .7.3.7.7v2.1M1.5 3.2v7.3c0 .4.3.7.7.7h5.4" ${line} stroke-width="1.05"/>` +
        '<circle cx="11.2" cy="9.6" r="2.4" fill="none" stroke="#0067c0" stroke-width="1.2"/><path d="m13 11.4 1.9 1.9" fill="none" stroke="#0067c0" stroke-width="1.2" stroke-linecap="round"/>');
    const CAP = { min: svg('0 0 10 10', `<path d="M.5 5h9" ${line} stroke-width="1"/>`),
        max: svg('0 0 10 10', `<rect x="1" y="1" width="8" height="8" rx="1" ${line} stroke-width="1"/>`),
        close: svg('0 0 10 10', `<path d="m1 1 8 8M9 1 1 9" ${line} stroke-width="1"/>`) };

    const sel = (parent, label) => el('span', 'se-sel', parent, label);
    // One condition row: its kind, what follows, the − and + buttons.
    function row(parent, kind, fill) {
        const n = el('div', 'se-row', parent);
        sel(n, kind);
        fill(n);
        el('span', 'se-pm', n, '−');
        el('span', 'se-pm', n, '+');
        return n;
    }

    // The editor as it opens (整台电脑 and an empty name row) and after the template's click.
    function editor(parent) {
        const w = { n: el('div', 'se-win', parent) };
        size(w.n, WIN.w, WIN.h);
        const cap = el('div', 'se-cap', w.n);
        const mark = el('img', null, cap);
        mark.src = 'assets/mark.svg';
        mark.alt = '';
        el('span', null, cap, T('新建智能栅栏'));
        for (const k of ['min', 'max', 'close']) el('div', 'se-capb', cap).innerHTML = CAP[k];
        const page = el('div', 'se-page', w.n);
        el('div', 'se-h', page, T('新建智能栅栏'));
        const name = el('div', 'se-line', page);
        el('span', 'se-dim', name, T('名称'));
        w.name = el('div', 'se-in focus', name);
        const presets = el('div', 'se-line', page);
        el('span', 'se-dim', presets, T('从模板开始'));
        w.chips = ['最近 7 天下载', '最近一周的截图', '大于 1 GB 的文件', '所有 .psd 文件', '最近打开的文件']
            .map(label => el('span', 'se-chip', presets, label));
        // 「包含符合以下{0}条件的文件」 around the 全部 / 任一 choice
        const match = el('div', 'se-line', page);
        const [before, after] = T('包含符合以下{0}条件的文件').split('{0}');
        el('span', null, match, before.trim());
        sel(match, T('全部'));
        el('span', null, match, after.trim());

        const rows = w.rows = el('div', 'se-rows', page);
        w.blank = [
            row(rows, T('位置'), n => {
                sel(n, T('整台电脑'));
                el('span', 'se-path', n, T('不含 Windows、AppData 和回收站'));
            }),
            row(rows, T('名称'), n => {
                sel(n, T('包含'));
                el('span', 'se-in', n);
            }),
        ];
        const [timeBefore, timeAfter] = T('新建或改过于{0}').split('{0}');
        w.filled = [
            row(rows, T('位置'), n => {
                // 位置 › 文件夹: one folder, where the fence title 文件夹 means several (the
                // catalog keeps the two apart with a context before \u0004, as here)
                sel(n, T('位置\u0004文件夹').split('\u0004').pop());
                el('span', 'se-path', n, 'C:\\Users\\me\\Downloads');
                el('span', 'se-btn', n, T('选择…'));
            }),
            row(rows, T('子文件夹'), n => {
                sel(n, T('包含子文件夹'));
                el('span', 'se-path', n);
            }),
            row(rows, T('时间'), n => {
                const g = el('span', 'se-grow', n);
                if (timeBefore.trim()) el('span', 'se-dim', g, timeBefore.trim());
                sel(g, T('最近 {0} 天').replace('{0}', 7));
                if (timeAfter.trim()) el('span', 'se-dim', g, timeAfter.trim());
            }),
            row(rows, T('类型'), n => {
                el('span', 'se-dim', n, '是');
                const g = el('span', 'se-grow', n);
                ['文件', '程序', '快捷方式', '文件夹', '文档', '图片', '音乐', '视频', '压缩包', '安装包']
                    .forEach((label, i) => el('span', i ? 'se-chip' : 'se-chip on', g, label));
            }),
        ];

        const foot = el('div', 'se-foot', page);
        const count = el('div', 'se-count', foot);
        count.innerHTML = FIND;
        const words = el('div', null, count);
        w.count = el('span', null, words);
        w.how = el('span', 'se-dim', words, T('由 Everything 搜索'));
        el('span', 'se-dim', foot, T('最多显示'));
        sel(foot, T('{0} 个').replace('{0}', 200));
        el('span', 'se-btn', foot, T('取消'));
        w.create = el('span', 'se-btn acc', foot, T('创建'));
        return w;
    }

    let PATH, CLICKS, NOTES, MENU, SUB;
    const at = t => pathAt(t, PATH);

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        // a loose icon of 「桌面」 in the first desktop column
        const todo = icon(S, 'txt', '待办.txt');
        todo.classList.add('desk');
        put(todo, deskCell(0, 0).x, deskCell(0, 0).y);
        const f = fence(S, { ...F0 });
        F0.items.forEach(([k, l], i) => {
            const s = local(i, 3);
            put(icon(f.body, k, l), s.x, s.y);
        });

        r.S = fence(S, { ...S0 });
        el('div', 'sm-glyph', r.S.bar).innerHTML = MARK;
        r.S.name.style.paddingLeft = '22px';
        r.S.name.style.paddingRight = '64px';
        r.count = el('span', 'sm-count', r.S.bar);
        r.found = FOUND.map(([k, l]) => icon(r.S.body, k, l));
        r.fresh = icon(r.S.body, ...NEW);
        r.pulse = el('div', 'ring pulse', S);

        r.w = editor(S);
        put(r.w.n, WIN.x, WIN.y);
        r.menu = menu(S, MENU_ITEMS, 196);
        r.sub = menu(S, SUB_ITEMS, 168);
        r.note = el('div', 'chip', S);
        r.ptr = pointer(S);
        taskbar(root);

        // The menu is kept inside the screen; its submenu opens on the side that has room and is
        // moved up to fit above the taskbar.
        MENU = { x: Math.min(GRAB.x + 2, SW - 4 - r.menu.w), y: GRAB.y + 2, a: RC + .05, b: PICK + .03 };
        const addRow = MENU.y + rowY(r.menu, '添加到桌面');
        SUB = { x: MENU.x - r.sub.w + 4, y: Math.min(addRow - 17, SH - 28 - r.sub.h - 2), a: SUB_AT, b: PICK + .03 };
        const pick = [SUB.x + 70, SUB.y + rowY(r.sub, '新建智能栅栏…')];

        // pointer targets inside the editor, measured where they are drawn
        const o = root.getBoundingClientRect(), scale = o.width / 800 || 1;
        const box = n => {
            const b = n.getBoundingClientRect();
            return { x: (b.left - o.left) / scale, y: (b.top - o.top) / scale, w: b.width / scale, h: b.height / scale };
        };
        const mid = b => [b.x + b.w / 2, b.y + b.h / 2];
        r.w.blank.forEach(n => show(n, false));         // measure the filled page: its buttons do not move later
        // As in the app, every row's first menu is as wide as the widest of them, and the list
        // of conditions scrolls when a language's longer texts leave it too little room.
        const kinds = [...r.w.rows.querySelectorAll('.se-row > .se-sel:first-child')];
        kinds.forEach(n => { n.style.width = 'auto'; });
        const kindW = Math.max(84, ...kinds.map(n => n.offsetWidth));
        kinds.forEach(n => { n.style.width = kindW + 'px'; });
        const over = r.w.rows.scrollHeight - r.w.rows.clientHeight;
        r.w.bar = el('div', 'se-scrollbar', r.w.rows);
        el('i', null, r.w.bar).style.height = Math.round(100 * r.w.rows.clientHeight / r.w.rows.scrollHeight) + '%';
        r.w.scrolls = over > 2;
        r.chipBox = box(r.w.chips[0]);
        r.createBox = box(r.w.create);
        PATH = [
            [0, 470, 392], [0.3, 470, 392],
            [0.95, GRAB.x, GRAB.y], [RC + .2, GRAB.x, GRAB.y],                 // right-click the title
            [1.75, MENU.x + 110, addRow], [2.0, MENU.x + 40, addRow],            // 添加到桌面
            [2.45, ...pick], [PICK + .2, ...pick],                              // 新建智能栅栏…
            [3.55, ...mid(r.chipBox)], [TEMPLATE + .45, ...mid(r.chipBox)],     // the template 最近 7 天下载
            [5.0, ...mid(r.createBox)], [CREATE + .2, ...mid(r.createBox)],     // 创建
            [6.0, 520, 300], [D, 520, 300],
        ];
        CLICKS = [{ t: RC, right: true }, { t: PICK }, { t: TEMPLATE }, { t: CREATE }];
        NOTES = [
            // left of the menu, which is kept inside the screen and so covers the pointer
            [RC - .12, RC + .35, '右键标题栏', MENU.x - 8, GRAB.y - 12, 'abs', 'right'],
            [MADE + .3, NEW_AT - .35, '符合条件的文件都在这里，最新的在前', S0.x, S0.y + S0.h + 8, 'abs'],
            [NEW_AT + .1, NEW_AT + 1.9, '新下载的文件自己出现', S0.x, S0.y + S0.h + 8, 'abs'],
        ];
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const w = r.w;

        drawMenu(r.menu, t, MENU.x, MENU.y, MENU.a, MENU.b, p);
        // the row of an open submenu stays lit
        if (t >= SUB_AT && t < MENU.b) r.menu.rows.find(o => o.t === '添加到桌面').r.classList.add('hot');
        drawMenu(r.sub, t, SUB.x, SUB.y, SUB.a, SUB.b, p);

        // the editor: opens centred, the template fills it, 创建 closes it
        if (show(w.n, t >= ED_AT && t < CREATE + .12)) {
            const o = ramp(t, ED_AT, ED_AT + .167, E.lin) * (1 - ramp(t, CREATE + .04, CREATE + .12, E.lin));
            put(w.n, WIN.x, WIN.y, o, ` scale(${lerp(.97, 1, ramp(t, ED_AT, ED_AT + .25)).toFixed(4)})`);
            const filled = t >= TEMPLATE + .03;
            w.blank.forEach(n => show(n, !filled));
            w.filled.forEach(n => show(n, filled));
            show(w.bar, filled && w.scrolls);
            const caret = Math.floor((t - ED_AT) / .5) % 2 === 0 ? '|' : '';
            const html = filled ? T('最近 7 天下载') : `${caret}<span class="se-dim">${T('智能栅栏')}</span>`;
            if (w.name.innerHTML !== html) w.name.innerHTML = html;
            w.name.classList.toggle('focus', !filled);
            w.chips[0].classList.toggle('hot', !filled && inside(p, r.chipBox));
            const counted = t >= COUNTED;
            const count = !filled ? T('填好条件后显示找到几个文件') : !counted ? T('正在搜索…')
                : T('现在找到 {0} 个文件').replace('{0}', `<b>${FOUND.length}</b>`);
            if (w.count.innerHTML !== count) w.count.innerHTML = count;
            show(w.how, counted);
            w.create.classList.toggle('hot', inside(p, r.createBox));
        }

        // the smart fence: fades in while growing from 97 %; a new download takes the first cell
        // (it is the newest) and the others move one on
        if (show(r.S.n, t >= MADE)) {
            const o = ramp(t, MADE, MADE + .167, E.lin), s = lerp(.97, 1, ramp(t, MADE, MADE + .25));
            put(r.S.n, S0.x, S0.y, o, ` scale(${s.toFixed(4)})`);
            const moved = ramp(t, NEW_AT + .05, NEW_AT + .3, E.p2p);
            r.found.forEach((n, i) => {
                const a = local(i, COLS), b = local(i + 1, COLS);
                put(n, lerp(a.x, b.x, moved), lerp(a.y, b.y, moved));
            });
            if (show(r.fresh, t >= NEW_AT)) put(r.fresh, local(0, COLS).x, local(0, COLS).y, ramp(t, NEW_AT + .1, NEW_AT + .27, E.lin));
            text(r.count, String(FOUND.length + (t >= NEW_AT ? 1 : 0)));
        }
        if (show(r.pulse, t >= NEW_AT && t < NEW_AT + .8)) {
            const s = slot(S0, 0, COLS), q = (t - NEW_AT) / .8;
            put(r.pulse, s.x + 39, s.y + 24, (1 - q) * .9, ` scale(${.6 + E.out(q) * 1.1})`);
        }

        drawChip(r.note, t, NOTES, p);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, at });
    }

    PM.register('smart', { duration: D, poster: 6.4, build, render });
})();
