// 7.2 脚本栅栏里能点的行 (lesson id script-rows): two script fences added from 设置 → 小工具, the
// GitHub example (assets/scripts/github.*.ps1) and 电脑状态 (status.*.ps1, a streaming script). A
// script prints `text | key=value` lines (core script_output.rs); rows that do something get a
// light plate under the pointer, the hand cursor and a tooltip (the script's, else the target),
// and rows that open a URL or a file a small arrow on the right (fence_chrome.rs draw_script).
// A click on the PR row opens it in the browser (href=); 刷新 (refresh=true) runs the script
// again: 「… · 正在运行…」, then a new time. 电脑状态 keeps running and prints a new screen every
// 2 s (its footer says it is running); its IP row copies the address (copy=, nothing shows in
// the app: a caption says so) and 「打开任务管理器」 starts Task Manager (run=).
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, size, show, text, fence, pointer, drawPointer, taskbar,
        loopFade, T, TITLE_H } = PM;

    const D = 10.6;
    const lang = () => document.documentElement.lang || 'zh-CN';

    // A script fence is titled with its file's name (tools.rs create_tool_fence).
    const GH_FILE = 'GitHub 动态.ps1', PC_FILE = '电脑状态.ps1';
    const stem = f => f.replace(/\.ps1$/, '');
    const GH = { x: 24, y: 18, w: 300, h: 196, title: stem(GH_FILE) };
    const PC = { x: 24, y: 226, w: 300, h: 182, title: stem(PC_FILE) };
    const BROWSER = { x: 346, y: 18, w: 432, h: 292 };
    const TASKMGR = { x: 404, y: 150, w: 360, h: 214 };

    // What the two scripts printed, row by row ('-' = the `---` separator line).
    const PR = '对话框跳转和搜索列表';
    const GH_ROWS = [
        { icon: '🐙', t: 'DayuanJiang/PecoFence', opens: true },
        '-',
        { icon: '🔀', t: '#186  ' + PR, opens: true, tip: 'DayuanJiang: ' + PR },
        { icon: '🟢', t: '#185  ' + '图标的排列顺序', opens: true },
        { icon: '🟢', t: '#183  ' + '更多快捷键', opens: true },
        '-',
        { icon: '🔄', t: '刷新', click: true },
    ];
    // 电脑状态 prints a new screen every 2 s: CPU and memory change, the rest stays.
    const CPU = [12, 18, 9, 23, 14, 11], MEM = [41, 41, 42, 42, 41, 41];
    const IP = '192.168.1.23';
    const PC_ROWS = [
        { icon: '🧠', t: 'CPU {0}%', cpu: true, color: 'green' },
        { icon: '📊', t: '内存已用 {0}%', mem: true, color: 'green' },
        { icon: '💽', t: 'C 盘剩余 128 GB（27%）', opens: true, color: 'green' },
        '-',
        { icon: '📋', t: '本机 IP ' + IP, click: true, tip: '点一下复制' },
        { icon: '📈', t: '打开任务管理器', click: true },
    ];

    // draw_script: rows 15 px of text with 3 px above and below, separators 11 px; the plate
    // reaches 6 px beyond the 14 px text margin.
    const ROW_H = 21, SEP_H = 11, TOP = 2;

    const HOVER_PR = 1.0, CLICK_PR = 2.5, BROWSER_AT = 2.62;
    const AT_REFRESH = 3.5, CLICK_REFRESH = 3.85, RAN = 4.6;
    const AT_IP = 5.2, CLICK_IP = 6.3, AT_TM = 7.3, CLICK_TM = 7.65, TM_AT = 7.85;
    const TIP_WAIT = .4;      // tooltip.rs: the hover time

    let PATH, CLICKS, CHIPS, TIPS;
    const at = t => pathAt(t, PATH);

    const SVG = (vb, d, extra = '') => `<svg viewBox="${vb}" aria-hidden="true"><path d="${d}" fill="none" ` +
        `stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" ${extra}/></svg>`;
    // Segoe's OpenInNewWindow glyph (E8A7): a window with an arrow leaving its corner
    const GO = SVG('0 0 12 12', 'M5 1.5H2.2a.7.7 0 0 0-.7.7v7.6c0 .4.3.7.7.7h7.6c.4 0 .7-.3.7-.7V7M7 1.5h3.5V5M10.5 1.5 5.6 6.4');
    // the Windows hand cursor (IDC_HAND), its fingertip on the hot spot
    const HAND = '<svg class="k-hand" viewBox="0 0 18 22" width="18" height="22" aria-hidden="true">' +
        '<path d="M6.5 1.2c-.9 0-1.6.7-1.6 1.6v8.6l-1.2-1.3c-.6-.7-1.7-.7-2.3-.1-.6.6-.6 1.5-.1 2.1l3.6 4.6c1.2 1.6 ' +
        '2.6 3.3 5.3 3.3h1.6c2.9 0 4.7-2.2 4.7-5V9.9c0-.8-.6-1.4-1.4-1.4s-1.4.6-1.4 1.4v-.6c0-.8-.6-1.4-1.4-1.4s-1.4.6' +
        '-1.4 1.4v-.8c0-.8-.6-1.4-1.4-1.4s-1.3.5-1.4 1.2V2.8c0-.9-.7-1.6-1.6-1.6z" fill="#fff" stroke="#151515" ' +
        'stroke-width="1.1" stroke-linejoin="round"/><path d="M9.6 9.2v4.4M12.4 9.8v3.8M15.1 10.6v3" stroke="#151515" ' +
        'stroke-width=".9" stroke-linecap="round"/></svg>';
    const CAPS = '<div class="pk-caps"><i>' + SVG('0 0 10 10', 'M.5 5h9') + '</i><i>' +
        '<svg viewBox="0 0 10 10" aria-hidden="true"><rect x=".5" y=".5" width="9" height="9" rx="1.6" fill="none" ' +
        'stroke="currentColor" stroke-width="1"/></svg></i><i>' + SVG('0 0 10 10', 'M.8.8l8.4 8.4M9.2.8 .8 9.2') + '</i></div>';
    const LN = w => `<i class="pk-ln" style="width:${w}%"></i>`;

    // The rows of a script fence, laid out as draw_script does; each clickable row keeps its
    // plate in stage coordinates for the hover test.
    function scriptRows(f, rows) {
        const box = el('div', 'sx-rows', f.body);
        let y = TOP;
        f.rows = [];
        for (const row of rows) {
            if (row === '-') {
                put(el('div', 'sx-sep', box), 0, y + (SEP_H - 1) / 2);
                y += SEP_H;
                continue;
            }
            const n = el('div', 'sx-row', box);
            put(n, 0, y);
            const plate = el('div', 'sx-plate', n);
            let x = 6;
            const r = { n, plate, row, x0: f.x + 8, x1: f.x + f.w - 8, y0: f.y + TITLE_H + y, y1: f.y + TITLE_H + y + ROW_H };
            if (row.icon) {
                el('div', 'sx-ic', n, row.icon);
                x += 20;
            }
            r.t = el('div', 'sx-t' + (row.color ? ' c-' + row.color : ''), n, row.t);
            r.t.style.left = x + 'px';
            if (row.opens) {
                r.t.style.right = '24px';
                r.go = el('div', 'sx-go', n);
                r.go.innerHTML = GO;
            }
            f.rows.push(r);
            y += ROW_H;
        }
        f.foot = el('div', 'tl-foot', f.body);
        return f;
    }

    function browser(parent) {
        const n = el('div', 'pk-win pk-browser sx-browser', parent);
        n.innerHTML =
            '<div class="pk-tabs"><div class="pk-tab"><img src="assets/icons/url.png" alt=""><span></span>' +
            SVG('0 0 8 8', 'M1 1l6 6M7 1 1 7') + '</div><div class="pk-plus">' + SVG('0 0 10 10', 'M5 .5v9M.5 5h9') + '</div>' +
            CAPS + '</div>' +
            '<div class="pk-bar"><span class="pk-nav">' + SVG('0 0 14 14', 'M12.5 7h-11M6 2.5 1.5 7 6 11.5') + '</span>' +
            '<span class="pk-nav off">' + SVG('0 0 14 14', 'M1.5 7h11M8 2.5l4.5 4.5L8 11.5') + '</span>' +
            '<span class="pk-nav">' + SVG('0 0 14 14', 'M12 7a5 5 0 1 1-1.46-3.54M12 1.8v2.9H9.1') + '</span>' +
            '<div class="pk-addr">' + SVG('0 0 12 12', 'M3 5.5V4a3 3 0 0 1 6 0v1.5M2.5 5.5h7v5h-7z') +
            '<span>github.com<em>/DayuanJiang/PecoFence/pull/186</em></span></div></div>' +
            '<div class="pk-page"><div class="pk-main"><h1></h1><p class="pk-meta sx-open">Open</p>' +
            '<i class="pk-ln h" style="width:26%"></i>' + LN(94) + LN(86) + LN(91) + LN(58) +
            '<i class="pk-ln h" style="width:18%"></i>' + LN(90) + LN(72) + '</div></div>';
        n.querySelector('.pk-tab span').textContent = PR + ' · Pull Request #186';
        n.querySelector('h1').textContent = PR + ' #186';
        put(n, BROWSER.x, BROWSER.y, 0);
        size(n, BROWSER.w, BROWSER.h);
        return n;
    }

    // Task Manager (Windows 11): the nav strip on the left, 进程 with a few rows.
    function taskManager(parent) {
        const n = el('div', 'pk-win sx-taskmgr', parent);
        n.innerHTML = '<div class="pk-titlebar"><img src="assets/icons/exe.png" alt=""><span></span>' + CAPS + '</div>' +
            '<div class="sx-tm"><div class="sx-tm-nav"><i></i><i class="on"></i><i></i><i></i><i></i></div>' +
            '<div class="sx-tm-main"><b></b></div></div>';
        n.querySelector('.pk-titlebar span').textContent = '任务管理器';
        const main = n.querySelector('.sx-tm-main');
        main.querySelector('b').textContent = '进程';
        [['名称', 'CPU', '内存'], ['PecoFence', '0.4%', '38 MB'], ['Microsoft Edge', '3.1%', '412 MB'],
            ['Windows 资源管理器', '0.8%', '96 MB'], ['Windows PowerShell', '0.6%', '62 MB'], ['任务管理器', '1.2%', '24 MB']].forEach(([a, b, c]) => {
            const row = el('div', 'sx-tm-row', main);
            el('span', null, row, a);
            el('em', null, row, b);
            el('em', null, row, c);
        });
        put(n, TASKMGR.x, TASKMGR.y, 0);
        size(n, TASKMGR.w, TASKMGR.h);
        return n;
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.G = scriptRows(fence(S, { ...GH }), GH_ROWS);
        r.P = scriptRows(fence(S, { ...PC }), PC_ROWS);
        r.browser = browser(S);
        r.taskmgr = taskManager(S);
        r.tip = el('div', 'pk-tip', S);

        r.ptr = pointer(S);
        r.ptr.n.insertAdjacentHTML('beforeend', HAND);
        r.hand = r.ptr.n.querySelector('.k-hand');
        taskbar(root);

        const mid = (q, dx = 116) => ({ x: q.x0 + dx, y: (q.y0 + q.y1) / 2 });
        const pr = mid(r.G.rows[1]);
        const refresh = mid(r.G.rows[4], 40);
        const ip = mid(r.P.rows[3], 90);
        const tm = mid(r.P.rows[4], 80);
        PATH = [
            [0, 560, 398], [0.3, 560, 398],
            [HOVER_PR, pr.x, pr.y], [CLICK_PR + .4, pr.x, pr.y],          // rest on the PR row, click it
            [AT_REFRESH, refresh.x, refresh.y], [CLICK_REFRESH + .3, refresh.x, refresh.y],   // 刷新
            [AT_IP, ip.x, ip.y], [CLICK_IP + .35, ip.x, ip.y],             // rest on the IP row, click it
            [AT_TM, tm.x, tm.y], [CLICK_TM + .3, tm.x, tm.y],             // 打开任务管理器
            [8.9, 372, 400], [D, 372, 400],
        ];
        CLICKS = [{ t: CLICK_PR }, { t: CLICK_REFRESH }, { t: CLICK_IP }, { t: CLICK_TM }];
        CHIPS = [[CLICK_IP + .1, CLICK_IP + .95, 'IP 地址已复制', 18, 18]];
        // the script's tooltip after the hover time at rest, at the pointer + (12, 20)
        // (tooltip.rs show_tip); the press hides it
        TIPS = [[HOVER_PR + TIP_WAIT, CLICK_PR, r.G.rows[1].row.tip, pr], [AT_IP + TIP_WAIT, CLICK_IP, r.P.rows[3].row.tip, ip]];
        return r;
    }

    // Window open (Windows 11): fades in while growing from 94 % (as peek.js).
    function opening(n, t, at0, box) {
        if (!show(n, t >= at0)) return;
        const o = ramp(t, at0, at0 + .2, E.lin), s = lerp(.94, 1, ramp(t, at0, at0 + .3));
        put(n, box.x, box.y, o, ` scale(${s.toFixed(4)})`);
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = at(t);
        const time = /^en/.test(lang()) ? ' AM' : '';

        // the row under the pointer (none while a window covers it)
        let hot = null;
        for (const f of [r.G, r.P]) {
            for (const q of f.rows) {
                const clickable = q.row.opens || q.row.click;
                const on = clickable && p.x >= q.x0 && p.x < q.x1 && p.y >= q.y0 && p.y < q.y1;
                if (on) hot = q;
                show(q.plate, on);
                if (q.go) show(q.go, on);
            }
        }
        text(r.G.foot, t >= CLICK_REFRESH && t < RAN ? T('{0} · 正在运行…').replace('{0}', GH_FILE)
            : `${GH_FILE} · ${T('{0} 更新').replace('{0}', (t >= RAN ? '10:25' : '10:24') + time)}`);

        // 电脑状态: a new screen every 2 s; a script that keeps running says so at the bottom
        const screen = Math.min(CPU.length - 1, Math.floor(t / 2));
        text(r.P.rows[0].t, T('CPU {0}%').replace('{0}', CPU[screen]));
        text(r.P.rows[1].t, T('内存已用 {0}%').replace('{0}', MEM[screen]));
        text(r.P.foot, T('{0} · 正在运行…').replace('{0}', PC_FILE));

        const tip = TIPS.find(([a, b]) => t >= a && t < b);
        if (show(r.tip, !!tip)) {
            text(r.tip, tip[2]);
            put(r.tip, tip[3].x + 12, tip[3].y + 20, ramp(t, tip[0], tip[0] + .1, E.lin));
        }

        opening(r.browser, t, BROWSER_AT, BROWSER);
        opening(r.taskmgr, t, TM_AT, TASKMGR);

        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at });
        show(r.ptr.arrow, !hot);
        show(r.hand, !!hot);
    }

    PM.register('script-rows', { duration: D, poster: 8.9, build, render });
})();
