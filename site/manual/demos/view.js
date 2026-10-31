// 2.2 视图、图标大小和排序: Ctrl+wheel steps 48 → 64 in Icons view (fences.rs step_icon_size; the
// cells snap and the height refits, fit_height_to_content), 排序方式 → 按类型 slides the icons
// to their new cells, 视图 → 详细信息 switches to rows under a 名称 / 修改日期 / 类型 / 大小 header.
// Submenus open to the right of the fence menu after the menu show delay.
(() => {
    'use strict';
    const { E, ramp, lerp, inside, pathAt, firstWhen, el, put, size, show, local, icon, fence, menu, drawMenu,
        rowY, pointer, drawPointer, drawKeys, taskbar, loopFade, T, PAD, TITLE_H } = PM;

    const D = 11.6;
    const F0 = { x: 40, y: 40, w: 480, h: 204, title: '文件与文档' };
    const P0 = { x: 604, y: 40, w: 168, h: 204, title: '程序' };
    // key: icon, label, extension, 修改日期, 类型 (Explorer's names), 大小
    const ITEMS = [
        ['pdf', '发票.pdf', 'pdf', '2026/9/30 16:42', 'Microsoft Edge PDF 文档', '96 KB'],
        ['pdf', '合同.pdf', 'pdf', '2026/10/5 11:08', 'Microsoft Edge PDF 文档', '1,206 KB'],
        ['docx', '周报.docx', 'docx', '2026/10/6 9:15', 'Microsoft Word 文档', '38 KB'],
        ['txt', '待办.txt', 'txt', '2026/10/6 10:02', '文本文档', '1 KB'],
        ['docx', '方案.docx', 'docx', '2026/9/28 14:30', 'Microsoft Word 文档', '214 KB'],
        ['assets/icons/icons-poster.png', '海报.png', 'png', '2026/9/21 20:17', 'PNG 文件', '412 KB'],
        ['xlsx', '预算.xlsx', 'xlsx', '2026/10/2 17:46', 'Microsoft Excel 工作表', '27 KB'],
    ];
    // Dates and sizes as Explorer shows them in the page's language (zh-CN keeps the text above).
    const LANG = document.documentElement.lang;
    if (LANG !== 'zh-CN') {
        const dt = new Intl.DateTimeFormat(LANG, LANG === 'en'
            ? { year: 'numeric', month: 'numeric', day: 'numeric', hour: 'numeric', minute: '2-digit' }
            : { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
        const num = new Intl.NumberFormat(LANG);
        const unit = { fr: 'Ko', ru: 'КБ' }[LANG] || 'KB';
        for (const row of ITEMS) {
            const [y, mo, d, h, mi] = row[3].split(/[/ :]/).map(Number);
            row[3] = dt.format(new Date(y, mo - 1, d, h, mi)).replace(/,\s*/, ' ');
            row[5] = `${num.format(Number(row[5].replace(/[^\d]/g, '')))} ${unit}`;
        }
    }
    const lower = i => ITEMS[i][1].toLowerCase();
    const cmp = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
    // 手动 with no manual index = name in code point order; 按类型 = (extension, name)
    // (state.rs items_of).
    const MANUAL = ITEMS.map((_, i) => i).sort((a, b) => cmp(lower(a), lower(b)));
    const BY_TYPE = ITEMS.map((_, i) => i).sort((a, b) => cmp(ITEMS[a][2], ITEMS[b][2]) || cmp(lower(a), lower(b)));

    // Icon cells in stage px for 48 / 64 (layout.rs GridMetrics: icon + 32 wide, icon + 48 tall).
    const M48 = { cw: 78, ch: 80 };
    const M64 = { cw: 78 * 96 / 80, ch: 80 * 112 / 96 };
    const colsOf = m => Math.floor((F0.w - 2 * PAD) / m.cw + 1e-6);
    const cell = (m, i) => ({ x: PAD + (i % colsOf(m)) * m.cw, y: 4 + Math.floor(i / colsOf(m)) * m.ch });
    const fitH = m => TITLE_H + 4 + Math.ceil(ITEMS.length / colsOf(m)) * m.ch + 8;

    // Details columns (layout.rs DetailColumns::for_width): 名称 | 修改日期 136 | 类型 108 | 大小 76.
    const COLS = (() => {
        const G = 8, right0 = F0.w - G;
        const size = [right0 - 76, 76], type = [size[0] - G - 108, 108], date = [type[0] - G - 136, 136];
        return { name: [8, date[0] - G - 8], date, type, size };
    })();
    const HEAD_H = 24, ROW_H = 24;

    const PATH = [
        [0, 650, 380], [0.3, 650, 380],
        [1.0, 330, 205], [2.9, 330, 205],       // Ctrl + wheel over the fence
        [3.5, 236, 56], [3.75, 236, 56],        // right-click the title
        [4.2, 380, 162], [4.75, 380, 162],      // 排序方式 (aim right of the words)
        [5.05, 470, 162], [5.35, 545, 214],     // 按类型
        [5.6, 545, 214], [6.4, 420, 300],
        [6.9, 256, 56], [7.1, 256, 56],         // right-click the title
        [7.55, 400, 136], [8.05, 400, 136],     // 视图
        [8.35, 492, 136], [8.7, 560, 188],      // 详细信息
        [8.95, 560, 188], [9.7, 650, 340], [D, 650, 340],
    ];
    const at = t => pathAt(t, PATH);
    const SIZE_AT = 1.8, SORT_AT = 5.5, VIEW_AT = 8.85;
    const RCLICK1 = 3.62, RCLICK2 = 7.0;
    const CLICKS = [{ t: RCLICK1, right: true }, { t: SORT_AT }, { t: RCLICK2, right: true }, { t: VIEW_AT }];
    const CHIPS = [[1.4, 2.3, '向上滚动滚轮'], [3.3, 3.7, '右键标题栏', -30, -48], [6.72, 7.06, '右键标题栏', -30, -48]];
    const KEYS = [{ a: 1.05, b: 2.55, keys: ['Ctrl'], down: 1.2, up: 2.5 }];
    const MENU_DELAY = .4;      // SPI_GETMENUSHOWDELAY default

    const FENCE_MENU = ['收起', '重命名…', '-', { t: '视图', sub: 1 }, { t: '排序方式', sub: 1 }, { t: '新建', sub: 1 },
        { t: '粘贴', key: 'Ctrl+V' }, { t: '标签页', sub: 1 }, '锁定位置和大小', '-', '栅栏选项…', '新建栅栏',
        { t: '在桌面显示文件夹', sub: 1 }, '删除栅栏'];
    const CHECK = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.3 6.3 4.9 8.9 9.8 3.4" fill="none" ' +
        'stroke="#1b2133" stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"/></svg>';
    const CHEVRON_UP = '<svg viewBox="0 0 8 5" aria-hidden="true"><path d="M.7 4.3 4 1l3.3 3.3" fill="none" ' +
        'stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"/></svg>';

    // menu() plus MF_CHECKED marks and MF_GRAYED rows (local: the engine has neither).
    function menuX(parent, items, width, { checks = [], off = [] } = {}) {
        const m = menu(parent, items, width);
        if (checks.length) m.n.classList.add('icons-checks');
        for (const row of m.rows) {
            if (checks.includes(row.t)) el('div', 'icons-check', row.r).innerHTML = CHECK;
            if (off.includes(row.t)) row.r.classList.add('icons-off');
        }
        return m;
    }

    const rowRect = (m, x, y, label) => {
        const r = m.rows.find(o => o.t === label);
        return { x: x + 4, y: y + r.y0, w: m.w - 8, h: r.y1 - r.y0 };
    };

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.P = fence(S, { ...P0 });
        [['exe', '截图工具'], ['url', '项目主页'], ['url', '在线文档'], ['exe', '录屏']].forEach(([k, l], i) => {
            const s = local(i, 2);
            put(icon(r.P.body, k, l), s.x, s.y);
        });
        r.F = fence(S, { ...F0 });
        r.grid = el('div', 'icons-layer', r.F.body);
        r.icons = ITEMS.map(([k, l]) => icon(r.grid, k, l));

        // details view
        r.det = el('div', 'icons-layer', r.F.body);
        const head = el('div', 'icons-dhead', r.det);
        const cap = (label, [x, w], right) => {
            const n = el('div', 'icons-dcap' + (right ? ' r' : ''), head, T(label));
            n.style.left = (right ? x : x + (label === '名称' ? 4 : 0)) + 'px';
            n.style.width = w + 'px';
        };
        cap('名称', COLS.name);
        cap('修改日期', COLS.date);
        cap('类型', COLS.type);
        cap('大小', COLS.size, true);
        for (const c of [COLS.date, COLS.type, COLS.size]) el('div', 'icons-ddiv', head).style.left = (c[0] - 4.5) + 'px';
        const chev = el('div', 'icons-dsort', head);
        chev.innerHTML = CHEVRON_UP;
        chev.style.left = (COLS.type[0] + (COLS.type[1] - 12) / 2) + 'px';
        BY_TYPE.forEach((i, row) => {
            const [k, label, , date, type, sz] = ITEMS[i];
            const n = el('div', 'icons-drow', r.det);
            n.style.top = (HEAD_H + 4 + row * ROW_H) + 'px';
            const img = el('img', null, n);
            img.src = k.includes('/') ? k : `assets/icons/${k}.png`;
            img.alt = '';
            const span = (cls, text, [x, w]) => {
                const s = el('span', cls, n, text);
                s.style.left = x + 'px';
                s.style.width = w + 'px';
            };
            span('n', label, [34, COLS.name[0] + COLS.name[1] - 34]);
            span('s', date, COLS.date);
            span('s', type, COLS.type);
            span('s r', sz, COLS.size);
        });

        r.m1 = menuX(S, FENCE_MENU, 196, { off: ['粘贴'] });
        r.m2 = menuX(S, FENCE_MENU, 196, { off: ['粘贴'] });
        r.sortMenu = menuX(S, ['手动', '按名称', '按类型', '按修改日期', '按大小', '按打开次数', '-', '倒序', '文件夹优先', '-', '按时间分组'],
            150, { checks: ['手动'] });
        r.viewMenu = menuX(S, ['图标', '列表', '详细信息', '-', '小图标', '中等图标', '较大图标', '大图标'], 140,
            { checks: ['图标', '较大图标'] });
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);

        // menu geometry and the moments the submenus open (pointer rests on the row)
        r.M1 = { x: 238, y: 58, a: RCLICK1 + .04, b: SORT_AT };
        r.M2 = { x: 258, y: 58, a: RCLICK2 + .04, b: VIEW_AT };
        const sub = (M, m, label, sm) => {
            const row = rowRect(m, M.x, M.y, label);
            const open = firstWhen(M.a, M.b, t => inside(at(t), row)) + MENU_DELAY;
            return { x: M.x + m.w - 3, y: M.y + m.rows.find(o => o.t === label).y0 - 4, a: open, b: M.b, label, sm };
        };
        r.S1 = sub(r.M1, r.m1, '排序方式', r.sortMenu);
        r.S2 = sub(r.M2, r.m2, '视图', r.viewMenu);
        return r;
    }

    function drawMenus(t, p, m, M, S) {
        drawMenu(m, t, M.x, M.y, M.a, M.b, p);
        drawMenu(S.sm, t, S.x, S.y, S.a, S.b, p);
        // the parent row stays highlighted while its submenu is open
        if (t >= S.a && t < S.b) m.rows.forEach(row => row.r.classList.toggle('hot', row.t === S.label));
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);

        // icon size: 48 → 64 at once (cells snap, no glide), the height refits the content
        const big = t >= SIZE_AT;
        const m = big ? M64 : M48;
        const h = fitH(m);
        size(r.F.n, F0.w, h);

        // sort: 250 ms point-to-point glide (shown at 400 ms so it reads)
        const q = ramp(t, SORT_AT + .05, SORT_AT + .45, E.p2p);
        const details = t >= VIEW_AT;
        show(r.grid, !details);
        show(r.det, details);
        r.icons.forEach((n, i) => {
            n.classList.toggle('icons-s64', big);
            const a = cell(m, MANUAL.indexOf(i)), b = cell(m, BY_TYPE.indexOf(i));
            put(n, lerp(a.x, b.x, q), lerp(a.y, b.y, q));
        });

        drawMenus(t, p, r.m1, r.M1, r.S1);
        drawMenus(t, p, r.m2, r.M2, r.S2);

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at });
    }

    PM.register('view', { duration: D, poster: 8.6, build, render });
})();
