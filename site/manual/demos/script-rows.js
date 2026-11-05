// 7.2 脚本栅栏里能点的行 (lesson id script-rows): two script fences made with 「添加示例…」, the
// GitHub example (assets/scripts/github.*.ps1) and the todo.txt one (todo.*.ps1). A script
// prints `text | key=value` lines (core script_output.rs); rows that do something get a light
// plate under the pointer, the hand cursor and a tooltip (the script's, else the target), and
// rows that open a URL or a file a small arrow on the right (fence_chrome.rs draw_script).
// A click on the PR row opens it in the browser (href=); a click on a to-do ticks its box at
// once and runs the script again with the action (tools.rs script_row_clicked), which writes
// the tick into todo.txt; 「打开 todo.txt」 (open=) opens the file in Notepad.
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, size, show, text, fence, pointer, drawPointer, taskbar,
        loopFade, T, TITLE_H } = PM;

    const D = 9.8;
    const lang = () => document.documentElement.lang || 'zh-CN';

    // A script fence is titled with its file's name (tools.rs create_tool_fence).
    const GH_FILE = 'GitHub 动态.ps1', TODO_FILE = '待办.ps1';
    const stem = f => f.replace(/\.ps1$/, '');
    const GH = { x: 24, y: 20, w: 300, h: 198, title: stem(GH_FILE) };
    const TD = { x: 24, y: 232, w: 300, h: 168, title: stem(TODO_FILE) };
    const BROWSER = { x: 346, y: 20, w: 432, h: 292 };
    const NOTEPAD = { x: 392, y: 164, w: 340, h: 196 };

    // What the two scripts printed, row by row ('-' = the `---` separator line).
    const PR = '对话框跳转和搜索列表';
    const GH_ROWS = [
        { icon: '🐙', t: 'DayuanJiang/PecoFence', opens: true },
        '-',
        { icon: '🔀', t: '#186  ' + PR, opens: true, tip: 'DayuanJiang: ' + PR },
        { icon: '🟢', t: '#185  ' + '图标的排列顺序', opens: true },
        { icon: '🟢', t: '#183  ' + '更多快捷键', opens: true },
        '-',
        { icon: '🔄', t: '刷新', refresh: true },
    ];
    const TODOS = ['交房租', '回复邮件', '买牛奶'];
    const TD_ROWS = [
        { todo: 0, t: TODOS[0] }, { todo: 1, t: TODOS[1] }, { todo: 2, t: TODOS[2] },
        '-',
        { icon: '📄', t: '打开 todo.txt', opens: true },
    ];

    // draw_script: rows 15 px of text with 3 px above and below, separators 11 px; the plate
    // reaches 6 px beyond the 14 px text margin.
    const ROW_H = 21, SEP_H = 11, TOP = 2;

    const HOVER_PR = 1.0, CLICK_PR = 2.5, BROWSER_AT = 2.62;
    const CLICK_TODO = 4.55, RAN = 5.35, CLICK_OPEN = 6.85, NOTEPAD_AT = 7.0;

    let PATH, CLICKS;
    const at = t => pathAt(t, PATH);

    const SVG = (vb, d, extra = '') => `<svg viewBox="${vb}" aria-hidden="true"><path d="${d}" fill="none" ` +
        `stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" ${extra}/></svg>`;
    const CHECK = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="m2.6 6.3 2.3 2.3 4.6-4.9" fill="none" ' +
        'stroke="#fff" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';
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
            if (row.todo !== undefined) {
                r.box = el('div', 'tl-box sx-box', n);
                r.box.innerHTML = CHECK;
                x += 20;
            }
            if (row.icon) {
                el('div', 'sx-ic', n, row.icon);
                x += 20;
            }
            r.t = el('div', 'sx-t', n, row.t);
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

    function notepad(parent) {
        const n = el('div', 'pk-win sx-notepad', parent);
        n.innerHTML = '<div class="pk-titlebar"><img src="assets/icons/txt.png" alt=""><span></span>' + CAPS + '</div>' +
            '<div class="sx-menu"><i></i><i></i><i></i></div><div class="sx-text"></div>';
        n.querySelector('.pk-titlebar span').textContent = 'todo.txt - ' + '记事本';
        put(n, NOTEPAD.x, NOTEPAD.y, 0);
        size(n, NOTEPAD.w, NOTEPAD.h);
        return n;
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.G = scriptRows(fence(S, { ...GH }), GH_ROWS);
        r.T = scriptRows(fence(S, { ...TD }), TD_ROWS);
        r.browser = browser(S);
        r.notepad = notepad(S);
        r.lines = r.notepad.querySelector('.sx-text');
        r.tip = el('div', 'pk-tip', S);

        r.ptr = pointer(S);
        r.ptr.n.insertAdjacentHTML('beforeend', HAND);
        r.hand = r.ptr.n.querySelector('.k-hand');
        taskbar(root);

        const mid = q => ({ x: q.x0 + 116, y: (q.y0 + q.y1) / 2 });
        const pr = mid(r.G.rows[1]);
        const boxPt = { x: r.T.rows[0].x0 + 6 + 6.5, y: r.T.rows[0].y0 + 10.5 };
        const open = mid(r.T.rows[3]);
        open.x = r.T.rows[3].x0 + 70;
        PATH = [
            [0, 560, 392], [0.3, 560, 392],
            [HOVER_PR, pr.x, pr.y], [CLICK_PR + .4, pr.x, pr.y],      // rest on the PR row, click it
            [3.6, 250, 300], [3.75, 250, 300],
            [4.2, boxPt.x, boxPt.y], [CLICK_TODO + .35, boxPt.x, boxPt.y],   // tick the first to-do
            [6.3, open.x, open.y], [CLICK_OPEN + .3, open.x, open.y],          // 打开 todo.txt
            [8.0, 360, 398], [D, 360, 398],
        ];
        CLICKS = [{ t: CLICK_PR }, { t: CLICK_TODO }, { t: CLICK_OPEN }];
        r.prPt = pr;
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
        const ticked = t >= CLICK_TODO;
        const done = i => i === 2 || (i === 0 && ticked);
        const time = /^en/.test(lang()) ? ' AM' : '';

        // the row under the pointer (none while a window covers it)
        let hot = null;
        for (const f of [r.G, r.T]) {
            for (const q of f.rows) {
                const clickable = q.row.opens || q.row.refresh || q.row.todo !== undefined;
                const on = clickable && p.x >= q.x0 && p.x < q.x1 && p.y >= q.y0 && p.y < q.y1;
                if (on) hot = q;
                show(q.plate, on);
                if (q.go) show(q.go, on);
                if (q.box) {
                    q.n.classList.toggle('done', done(q.row.todo));
                }
            }
        }
        text(r.G.foot, `${GH_FILE} · ${T('{0} 更新').replace('{0}', '10:24' + time)}`);
        text(r.T.foot, ticked && t < RAN ? T('{0} · 正在运行…').replace('{0}', TODO_FILE)
            : `${TODO_FILE} · ${T('{0} 更新').replace('{0}', (ticked ? '10:24' : '10:20') + time)}`);

        // the script's tooltip after the hover time (400 ms) at rest, at the pointer + (12, 20)
        // (tooltip.rs show_tip); the press hides it
        const tipOn = t >= HOVER_PR + .4 && t < CLICK_PR;
        if (show(r.tip, tipOn)) {
            text(r.tip, r.G.rows[1].row.tip);
            put(r.tip, r.prPt.x + 12, r.prPt.y + 20, ramp(t, HOVER_PR + .4, HOVER_PR + .5, E.lin));
        }

        opening(r.browser, t, BROWSER_AT, BROWSER);
        opening(r.notepad, t, NOTEPAD_AT, NOTEPAD);
        text(r.lines, TODOS.map((s, i) => (done(i) ? '[x] ' : '[ ] ') + s).join('\n'));

        drawPointer(r.ptr, t, p, { clicks: CLICKS, at });
        show(r.ptr.arrow, !hot);
        show(r.hand, !!hot);
    }

    PM.register('script-rows', { duration: D, poster: 8.4, build, render });
})();
