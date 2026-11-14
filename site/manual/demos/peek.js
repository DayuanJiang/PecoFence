// 3.3 速览: a browser window covers the desktop and its fences; Ctrl+Alt+空格 floats every fence
// above the windows over a 30 % black dimmer (peek.rs). The fences here are completely under
// the window, so they fade in with the dimmer and fade out with it at the end (a fence on
// visible desktop would stay as it is). A click on empty space ends the Peek; peeking again
// and opening a file brings that app to the front, which ends it by itself (anchor.rs
// on_foreground).
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, show, icon, fence, pointer, drawPointer, drawKeys, taskbar,
        loopFade, slot } = PM;

    const D = 9.8;
    const FENCES = [
        { x: 88, y: 52, w: 246, h: 124, title: '程序', items: [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']] },
        { x: 88, y: 196, w: 246, h: 124, title: '文件夹', items: [['folder', '工作资料'], ['folder', '照片'], ['folder', '2026 报销']] },
        { x: 466, y: 52, w: 246, h: 204, title: '文件与文档', items: [['pdf', '合同.pdf'], ['docx', '周报.docx'],
            ['xlsx', '预算.xlsx'], ['txt', '待办.txt'], ['pdf', '发票.pdf'], ['docx', '方案.docx']] },
    ];
    const WIN = { x: 40, y: 16, w: 720, h: 386 };
    const DOC = { x: 196, y: 46, w: 470, h: 328 };
    const DOC_ICON = slot(FENCES[2], 1, 3);                 // 周报.docx
    const DOC_PT = { x: DOC_ICON.x + 39, y: DOC_ICON.y + 25 };

    const WIN_OPEN = 1.12;
    // The dimmer's 83 ms linear fade, slowed a little so it reads.
    const FADE = .15;
    const KEYS = [
        { a: 1.75, b: 2.75, keys: ['Ctrl', 'Alt', '空格'], down: 2.15 },
        { a: 5.0, b: 6.0, keys: ['Ctrl', 'Alt', '空格'], down: 5.4 },
    ];
    const CLICK_EMPTY = 4.15, DBL = [6.6, 6.77], DOC_OPEN = 7.05;
    // [peek on, end requested]: on = the hotkey; end = click on the dimmer / another app in front.
    const PEEKS = [[2.18, CLICK_EMPTY], [5.43, DOC_OPEN]];

    const PATH = [
        [0, 600, 330], [0.3, 600, 330],
        [0.95, 484, 436], [1.2, 484, 436],          // taskbar: open the browser
        [1.9, 620, 352], [3.5, 620, 352],           // rest while the hotkey is pressed
        [4.0, 380, 345], [5.7, 380, 345],           // click empty space (the dimmer)
        [6.4, DOC_PT.x, DOC_PT.y], [7.2, DOC_PT.x, DOC_PT.y],   // double-click 周报.docx
        [7.9, 700, 330], [D, 700, 330],
    ];
    const CLICKS = [{ t: 1.05 }, { t: CLICK_EMPTY }, { t: DBL[0] }, { t: DBL[1] }];
    const CHIPS = [[4.05, 4.7, '点栅栏外面'], [6.5, 7.05, '双击', -14, -46]];

    const SVG = (vb, d, extra = '') => `<svg viewBox="${vb}" aria-hidden="true"><path d="${d}" fill="none" ` +
        `stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round" ${extra}/></svg>`;
    const CAPS = '<div class="pk-caps"><i>' + SVG('0 0 10 10', 'M.5 5h9') + '</i><i>' +
        '<svg viewBox="0 0 10 10" aria-hidden="true"><rect x=".5" y=".5" width="9" height="9" rx="1.6" fill="none" ' +
        'stroke="currentColor" stroke-width="1"/></svg></i><i>' + SVG('0 0 10 10', 'M.8.8l8.4 8.4M9.2.8 .8 9.2') + '</i></div>';
    const LN = w => `<i class="pk-ln" style="width:${w}%"></i>`;

    function browser(parent) {
        const n = el('div', 'pk-win pk-browser', parent);
        n.innerHTML =
            '<div class="pk-tabs"><div class="pk-tab"><img src="assets/icons/url.png" alt=""><span>第三季度计划 - 在线文档</span>' +
            SVG('0 0 8 8', 'M1 1l6 6M7 1 1 7') + '</div><div class="pk-plus">' + SVG('0 0 10 10', 'M5 .5v9M.5 5h9') + '</div>' +
            CAPS + '</div>' +
            '<div class="pk-bar"><span class="pk-nav">' + SVG('0 0 14 14', 'M12.5 7h-11M6 2.5 1.5 7 6 11.5') + '</span>' +
            '<span class="pk-nav off">' + SVG('0 0 14 14', 'M1.5 7h11M8 2.5l4.5 4.5L8 11.5') + '</span>' +
            '<span class="pk-nav">' + SVG('0 0 14 14', 'M12 7a5 5 0 1 1-1.46-3.54M12 1.8v2.9H9.1') + '</span>' +
            '<div class="pk-addr">' + SVG('0 0 12 12', 'M3 5.5V4a3 3 0 0 1 6 0v1.5M2.5 5.5h7v5h-7z') +
            '<span>docs.example.com<em>/team/q3-plan</em></span></div></div>' +
            '<div class="pk-page"><div class="pk-side"><b>目录</b>' + LN(78) + LN(62) + LN(84) + LN(55) + LN(70) + '</div>' +
            '<div class="pk-main"><h1>第三季度计划</h1><p class="pk-meta">产品组 · 更新于 10 月 6 日</p>' +
            '<div class="pk-cards"><div class="pk-card c1">新功能<small>12</small></div>' +
            '<div class="pk-card c2">待修复<small>5</small></div><div class="pk-card c3">已上线<small>28</small></div></div>' +
            LN(96) + LN(88) + LN(92) + LN(54) + '<i class="pk-ln h" style="width:22%"></i>' + LN(90) + LN(94) + LN(70) +
            '</div></div>';
        put(n, WIN.x, WIN.y, 0);
        n.style.width = WIN.w + 'px';
        n.style.height = WIN.h + 'px';
        return n;
    }

    function documentWindow(parent) {
        const n = el('div', 'pk-win pk-doc', parent);
        n.innerHTML =
            '<div class="pk-titlebar"><img src="assets/icons/docx.png" alt=""><span>周报.docx</span>' + CAPS + '</div>' +
            '<div class="pk-ribbon"><i></i><i></i><i></i><i class="s"></i><i class="w"></i><i></i><i></i><i></i>' +
            '<i class="s"></i><i></i><i></i></div>' +
            '<div class="pk-canvas"><div class="pk-paper"><h2>本周工作</h2>' + LN(100) + LN(92) + LN(97) + LN(60) +
            '<i class="pk-ln h" style="width:30%"></i>' + LN(95) + LN(88) + LN(76) + LN(93) + LN(40) + '</div></div>';
        put(n, DOC.x, DOC.y, 0);
        n.style.width = DOC.w + 'px';
        n.style.height = DOC.h + 'px';
        return n;
    }

    // Fluent glass = blurred crop of the wallpaper under the fence (see peek.css).
    function glass(f) {
        f.n.classList.add('pk-glass');
        const crop = el('div', 'pk-crop');
        const pic = el('div', null, crop);
        pic.style.transform = `translate(${-f.x}px, ${-f.y}px)`;
        f.n.prepend(crop);
    }

    // Window open (Windows 11): fades in while growing from 94 %.
    function opening(n, t, at, box) {
        if (!show(n, t >= at)) return;
        const o = ramp(t, at, at + .2, E.lin), s = lerp(.94, 1, ramp(t, at, at + .3));
        put(n, box.x, box.y, o, ` scale(${s.toFixed(4)})`);
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.fences = FENCES.map(spec => {
            const f = fence(S, { ...spec });
            glass(f);
            f.icons = spec.items.map(([k, l], i) => {
                const n = icon(f.body, k, l);
                const s = PM.local(i, 3);
                put(n, s.x, s.y);
                return n;
            });
            return f;
        });
        r.browser = browser(S);
        r.doc = documentWindow(S);
        r.dim = el('div', 'pk-dim', S);
        r.keys = el('div', 'keys', S);
        r.ptr = pointer(S);
        taskbar(root);
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);

        // Windows: the browser from the taskbar, later the opened document on top.
        opening(r.browser, t, WIN_OPEN, WIN);
        opening(r.doc, t, DOC_OPEN, DOC);

        // Peek: the fences, all hidden by the browser window, come above the windows and fade in
        // with the dimmer; ending fades both out, then the fences are back under the windows.
        let dim = 0, floating = false;
        for (const [on, end] of PEEKS) {
            if (t < on) continue;
            dim = Math.max(dim, ramp(t, on, on + FADE, E.lin) * (1 - ramp(t, end, end + FADE, E.lin)));
            if (t < end + FADE) floating = true;
        }
        if (show(r.dim, dim > 0)) r.dim.style.opacity = (.302 * dim).toFixed(3);
        r.fences.forEach(f => {
            f.n.style.zIndex = floating ? 3 : 1;
            f.n.style.opacity = floating ? dim.toFixed(3) : 1;
        });

        // 周报.docx: selected by the double-click, grey once its app is the active window.
        const docIcon = r.fences[2].icons[1];
        docIcon.classList.toggle('sel', t >= DBL[0] && t < DOC_OPEN);
        docIcon.classList.toggle('pk-dull', t >= DOC_OPEN);

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: CLICKS, chips: CHIPS, at: u => pathAt(u, PATH) });
    }

    PM.register('peek', { duration: D, poster: 3.0, build, render });
})();
