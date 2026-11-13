// 2.1 让 AI 帮你配置: the request is pasted into a coding agent, which reads pecofence-cli skill and
// describe, looks at the fences, backs up (config export + snapshot save, as in the site's sample),
// then switches the theme style to Liquid Glass and the fences' opacity to 更透明; the running app
// changes as each command lands. Commands follow docs/CLI.md; replies are the compact JSON the CLI
// prints to a pipe (keys sorted, shortened with …). The 「桌面」 of a new installation has no
// plate: its icons are the loose ones at the top-left corner, which neither change touches.
(() => {
    'use strict';
    const { E, ramp, lerp, pathAt, el, put, size, show, local, deskCell, icon, fence, pointer, drawPointer,
        drawKeys, taskbar, loopFade } = PM;

    const D = 11.0;
    const TERM = { x: 96, y: 14, w: 408, h: 394 };
    const VIEW_H = TERM.h - 42 - 68;                         // transcript viewport above the input
    const FENCES = [
        { x: 530, y: 40, w: 246, h: 124, title: '程序',
            items: [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']] },
        { x: 530, y: 176, w: 246, h: 204, title: '文件与文档',
            items: [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['txt', '待办.txt']] },
    ];
    const DESKTOP = [['folder', '素材'], ['folder', '照片'], ['txt', '安装日志.log']];

    // the lesson's example request (the side column's 「可以这样写」)
    const PROMPT = '先阅读 pecofence-cli skill 和 pecofence-cli describe，查看并备份我的配置，再把主题风格换成 Liquid Glass，把所有栅栏调得更透明。';
    const CLICK_AT = .9, PASTE_AT = 1.25, SEND_AT = 1.95;
    // [start, args, reply, seconds until the reply]
    const CMDS = [
        [2.45, 'skill', '--- name: pecofence-cli …', .3],
        [2.95, 'describe', '{"cli":…,"commands":[…],…}', .3],
        [3.5, 'fence list --fields title,opacity', '[{"opacity":"default","title":"程序"},{"opacity":"default","title":"文件与文档"},…]', .32],
        [4.15, 'config export "C:\\Users\\me\\pecofence-before-ai.json"', '{"bytes":12873,"path":"C:\\\\Users\\\\me\\\\pecofence-before-ai.json"}', .32],
        [4.8, 'snapshot save before-ai', '{"changed":true,"snapshot":{"fenceCount":3,…}}', .3],
        [5.7, 'settings set themeStyle liquidGlass', '{"changed":true,"settings":{…}}', .32],
        [6.95, 'fence set --all opacity clear', '{"changed":true,"results":[{"changed":true,…},…]}', .5],
    ];
    const DONE_AT = 8.2;
    const DONE = '改好了：先备份了配置，然后把主题风格换成了 Liquid Glass，所有栅栏也调得更透明了。';
    // The app applies each mutation when its call arrives: the style at once, opacity one fence
    // per call (fence set --all runs one call at a time).
    const STYLE_AT = CMDS[5][0] + CMDS[5][3];
    const CLEAR_AT = i => CMDS[6][0] + .14 + i * .12;

    const PATH = [
        [0, 610, 398], [0.3, 610, 398],
        [0.85, 268, 384], [2.3, 268, 384],                     // click into the input, paste, send
        [3.0, 514, 400], [D, 514, 400],
    ];
    const KEYS = [{ a: 1.02, b: 1.5, keys: ['Ctrl', 'V'], down: PASTE_AT - .03 }, { a: 1.68, b: 2.15, keys: ['Enter'], down: SEND_AT - .02 }];

    const CHECK = '<svg class="xa-ok" viewBox="0 0 14 14" aria-hidden="true"><circle cx="7" cy="7" r="7" fill="#3fbf87"/>' +
        '<path d="M3.9 7.2l2.1 2.1 4.1-4.5" stroke="#fff" stroke-width="1.6" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>';
    const PROMPT_ICON = '<svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2 3l3 3-3 3M6.5 9.5H10" fill="none" stroke="#9fb4ff" ' +
        'stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/></svg>';
    const CAPTION = '<svg viewBox="0 0 10 10" style="right:84px"><path d="M0 5.5h10" stroke="#aebbd6" stroke-width="1"/></svg>' +
        '<svg viewBox="0 0 10 10" style="right:48px"><rect x=".5" y=".5" width="9" height="9" rx="1.5" fill="none" stroke="#aebbd6" stroke-width="1"/></svg>' +
        '<svg viewBox="0 0 10 10" style="right:12px"><path d="M.5.5l9 9M9.5.5l-9 9" stroke="#aebbd6" stroke-width="1"/></svg>';

    function terminal(S) {
        const n = el('div', 'xa-term', S);
        size(n, TERM.w, TERM.h);
        put(n, TERM.x, TERM.y);
        const bar = el('div', 'xa-bar', n);
        const tab = el('div', 'xa-tab', bar, '编程助手');
        tab.insertAdjacentHTML('afterbegin', PROMPT_ICON);
        el('div', 'xa-btns', bar).innerHTML = CAPTION;

        const view = el('div', 'xa-view', n);
        view.style.height = VIEW_H + 'px';
        const log = el('div', 'xa-log', view);
        const you = el('p', 'xa-turn', log);
        el('span', 'xa-who', you, '你');
        you.append(PROMPT);
        const rows = CMDS.map(([, args, reply]) => {
            const row = el('div', 'xa-cmd', log);
            const ic = el('div', 'xa-ic', row);
            const spin = el('div', 'xa-spin', ic);
            ic.insertAdjacentHTML('beforeend', CHECK);
            const line = el('div', 'xa-line', row);
            el('span', 'xa-bin', line, 'pecofence-cli');
            line.append(' ' + args);
            const res = el('div', 'xa-res', row, reply);
            return { row, spin, ok: ic.lastChild, res };
        });
        const done = el('p', 'xa-turn agent', log);
        el('span', 'xa-who', done, '助手');
        done.append(DONE);

        const input = el('div', 'xa-input', n);
        el('div', 'xa-gt', input, '>');
        const typed = el('span', null, input);
        const caret = el('span', 'xa-caret', input);
        return { log, you, rows, done, input, typed, caret };
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        DESKTOP.forEach(([k, l], i) => {
            const n = icon(S, k, l), s = deskCell(0, i);
            n.classList.add('desk');
            put(n, s.x, s.y);
        });
        r.term = terminal(S);
        // the fences' plates as layers under the title and icons, as in the look lesson
        // (settings.css): Fluent and Liquid Glass cross-fade
        r.fences = FENCES.map(f => {
            const x = fence(S, { ...f });
            x.n.classList.add('set-f');
            x.fluent = el('div', 'set-pf', null);
            x.liquid = el('div', 'set-plg', null);
            el('div', 'set-plg-rim', x.liquid);
            el('div', 'set-plg-hi', x.liquid);
            x.n.prepend(x.fluent, x.liquid);
            f.items.forEach(([k, l], i) => {
                const s = local(i, 3);
                put(icon(x.body, k, l), s.x, s.y);
            });
            return x;
        });
        r.keys = el('div', 'keys xa-keys', S);
        r.ptr = pointer(S);
        taskbar(root);
        return r;
    }

    // Transcript scroll: every row stays in the layout (hidden until its time), so the bottoms
    // are fixed; each new row scrolls the log up as far as it needs, 0.3 s decelerate.
    function scrollEvents(r) {
        if (r.scroll) return r.scroll;
        const T0 = r.term;
        const bottom = n => n.offsetTop + n.offsetHeight;
        const marks = [[SEND_AT + .05, bottom(T0.you)], ...CMDS.map(([t], i) => [t - .12, bottom(T0.rows[i].row)]),
            [DONE_AT, bottom(T0.done)]];
        r.scroll = marks.map(([t, b]) => [t, Math.max(0, b - VIEW_H)]);
        return r.scroll;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const p = pathAt(t, PATH);
        const m = r.term;

        // input: focus, paste, send
        m.input.classList.toggle('focus', t >= CLICK_AT);
        const pasted = t >= PASTE_AT && t < SEND_AT;
        const s = pasted ? PROMPT : '';
        if (m.typed.textContent !== s) m.typed.textContent = s;
        show(m.caret, t >= CLICK_AT && Math.floor((t - CLICK_AT) / .5) % 2 === 0);

        // transcript
        const reveal = (n, a) => {
            const q = ramp(t, a, a + .25);
            n.style.opacity = q;
            n.style.transform = `translateY(${(6 * (1 - q)).toFixed(2)}px)`;
        };
        reveal(m.you, SEND_AT + .05);
        CMDS.forEach(([a, , , run], i) => {
            const row = m.rows[i];
            reveal(row.row, a - .12);
            const done = t >= a + run;
            show(row.spin, !done);
            row.spin.style.transform = `rotate(${(t * 760).toFixed(1)}deg)`;
            show(row.ok, done);
            row.ok.style.transform = `scale(${(.5 + .5 * ramp(t, a + run, a + run + .2)).toFixed(3)})`;
            row.res.style.opacity = ramp(t, a + run, a + run + .2, E.lin);
        });
        reveal(m.done, DONE_AT);
        let y = 0;
        for (const [a, to] of scrollEvents(r)) {
            if (t < a) break;
            y = lerp(y, to, ramp(t, a, a + .3));
        }
        m.log.style.transform = `translateY(${(-y).toFixed(2)}px)`;

        // the desktop follows: every fence turns to Liquid Glass, then 更透明 fence by fence
        // (the glass at 0.55 of its strength, fence_options.rs OPACITY_CLEAR)
        const liquid = ramp(t, STYLE_AT, STYLE_AT + .2);
        r.fences.forEach((f, i) => {
            f.fluent.style.opacity = 1 - liquid;
            f.liquid.style.opacity = liquid * lerp(1, .55, ramp(t, CLEAR_AT(i), CLEAR_AT(i) + .15, E.lin));
        });

        drawKeys(r.keys, t, KEYS);
        drawPointer(r.ptr, t, p, { clicks: [{ t: CLICK_AT }], at: u => pathAt(u, PATH) });
    }

    PM.register('ai', { duration: D, poster: 8.6, build, render });

    // Side column: 复制 puts the example request (the same text as PROMPT) on the clipboard; where the
    // Clipboard API is missing or refused, the text is selected so Ctrl+C copies it.
    document.addEventListener('click', e => {
        const b = e.target.closest('.xa-copy');
        if (!b) return;
        const p = b.closest('.xa-prompt').querySelector('p');
        const label = b.querySelector('span');
        const done = ok => {
            b.classList.toggle('done', ok);
            label.textContent = ok ? '已复制' : '按 Ctrl+C 复制';
            clearTimeout(b.xaTimer);
            b.xaTimer = setTimeout(() => {
                b.classList.remove('done');
                label.textContent = '复制';
            }, 2000);
        };
        const select = () => {
            const range = document.createRange();
            range.selectNodeContents(p);
            const s = getSelection();
            s.removeAllRanges();
            s.addRange(range);
            done(false);
        };
        if (navigator.clipboard && window.isSecureContext) {
            navigator.clipboard.writeText(p.textContent.trim()).then(() => done(true), select);
        } else {
            select();
        }
    });
})();
