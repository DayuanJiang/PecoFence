// 1.3 移动和调整大小: drag a title (snaps level with the other fence + guide), drag an edge
// (snaps to the neighbour's edge + guide), drag a corner (icons glide into more columns).
(() => {
    'use strict';
    const { E, ramp, lerp, clamp01, pathAt, spans, spanFade, el, put, show, cols, local, icon, fence,
        placeFence, guide, drawGuide, pointer, drawPointer, taskbar, loopFade } = PM;

    const D = 8.6;
    const GAP = 8, MOVE_SNAP = 8, SIZE_SNAP = 10, WORK_H = 422;
    const A0 = { x: 70, y: 70, w: 246, h: 124 };
    const B0 = { x: 430, y: 170, w: 324, h: 204 };
    const GRAB = { x: 70, y: 16 };      // where the title is held, relative to B

    const PATH = [
        [0, 620, 380], [0.3, 620, 380],
        [1.0, 500, 186], [1.15, 500, 186],      // B's title
        [2.4, 398, 90], [2.8, 398, 90],         // dropped level with A
        [3.3, 190, 194], [3.5, 190, 194],       // A's bottom edge
        [4.6, 190, 271], [5.1, 190, 271],
        [5.7, 648, 274], [5.9, 648, 274],       // B's bottom-right corner
        [6.9, 736, 330], [D, 736, 330],
    ];
    const MOVE = [1.15, 2.6], EDGE = [3.5, 4.9], CORNER = [5.9, 7.2];
    const PRESSES = [MOVE, EDGE, CORNER];
    const KINDS = [[3.28, 5.15, 'ns'], [5.66, 7.5, 'nwse']];
    const CHIPS = [[1.15, 2.45, '按住标题栏拖动', 16, -34], [3.5, 4.75, '拖动边缘'], [5.9, 7.05, '拖动右下角']];

    const nearest = (v, cands, within) => {
        let best = null;
        for (const [c, tag] of cands) if (Math.abs(v - c) <= within && (!best || Math.abs(v - c) < Math.abs(v - best[0]))) best = [c, tag];
        return best;
    };

    // The whole layout as a pure function of t.
    function layout(t) {
        const p = pathAt(t, PATH);
        const A = { ...A0 }, B = { ...B0 };
        let top = null, edge = null, corner = null;

        // phase 1: move B by its title; snapped once released too
        if (t >= MOVE[0]) {
            const q = t < MOVE[1] ? p : pathAt(MOVE[1], PATH);
            let x = q.x - GRAB.x, y = q.y - GRAB.y;
            const sx = nearest(x, [[A.x, 'left'], [A.x + A.w + GAP, 'gap'], [A.x + A.w - B.w, 'right'], [GAP, 'screen']], MOVE_SNAP);
            const sy = nearest(y, [[A.y, 'top'], [A.y + A.h - B.h, 'bottom'], [GAP, 'screen']], MOVE_SNAP);
            if (sx) x = sx[0];
            if (sy) y = sy[0];
            B.x = x;
            B.y = y;
            top = sy && sy[1] === 'top' && t < MOVE[1];
        }
        // phase 2: A's bottom edge follows the pointer, snapping to B's bottom
        if (t >= EDGE[0]) {
            const q = t < EDGE[1] ? p : pathAt(EDGE[1], PATH);
            let bottom = q.y;
            const s = nearest(bottom, [[B.y + B.h, 'level'], [WORK_H - GAP, 'screen']], SIZE_SNAP);
            if (s) bottom = s[0];
            A.h = bottom - A.y;
            edge = s && s[1] === 'level' && t < EDGE[1];
        }
        // phase 3: B's bottom-right corner
        if (t >= CORNER[0]) {
            const q = t < CORNER[1] ? p : pathAt(CORNER[1], PATH);
            let bottom = q.y;
            const s = nearest(bottom, [[A.y + A.h, 'level']], SIZE_SNAP);
            if (s) bottom = s[0];
            B.w = q.x - B.x;
            B.h = bottom - B.y;
            corner = s && t < CORNER[1];
        }
        return { p, A, B, top, edge, corner };
    }

    function build(root) {
        const r = {};
        el('div', 'wall', root);
        r.scene = el('div', 'scene', root);
        const S = r.scene;
        r.A = fence(S, { ...A0, title: '程序' });
        r.B = fence(S, { ...B0, title: '文件与文档' });
        r.aIcons = [['url', '项目主页'], ['exe', '截图工具'], ['url', '在线文档']].map(([k, l]) => icon(r.A.body, k, l));
        r.bIcons = [['pdf', '合同.pdf'], ['docx', '周报.docx'], ['xlsx', '预算.xlsx'], ['txt', '待办.txt'], ['pdf', '发票.pdf'], ['folder', '素材']]
            .map(([k, l]) => icon(r.B.body, k, l));
        r.g1 = guide(S);
        r.g2 = guide(S);
        r.ptr = pointer(S);
        taskbar(root);

        r.topSpans = spans(D, t => layout(t).top);
        r.edgeSpans = spans(D, t => layout(t).edge);
        r.cornerSpans = spans(D, t => layout(t).corner);
        // column changes of B while its corner is dragged, for the reflow glide
        r.colChanges = [[0, cols(B0.w)]];
        for (let t = 0; t <= D; t += 1 / 120) {
            const c = cols(layout(t).B.w);
            if (c !== r.colChanges[r.colChanges.length - 1][1]) r.colChanges.push([t, c]);
        }
        return r;
    }

    function render(t, r, { looped }) {
        r.scene.style.opacity = loopFade(t, D, looped);
        const { p, A, B } = layout(t);
        placeFence(r.A, A.x, A.y, A.w, A.h);
        placeFence(r.B, B.x, B.y, B.w, B.h);

        r.aIcons.forEach((n, i) => {
            const s = local(i, 3);
            put(n, s.x, s.y);
        });
        // A dragged edge that changes the column count glides the icons to their new cells
        // (items.rs reflow_items: 167 ms decelerate; one already gliding goes on from its place).
        r.bIcons.forEach((n, i) => {
            let s = local(i, r.colChanges[0][1]);
            for (const [ct, c] of r.colChanges.slice(1)) {
                if (t < ct) break;
                const b = local(i, c), q = ramp(t, ct, ct + .167);
                s = { x: lerp(s.x, b.x, q), y: lerp(s.y, b.y, q) };
            }
            put(n, s.x, s.y);
        });

        // guides: tops level after the move, bottoms level while resizing
        const gTop = spanFade(t, r.topSpans);
        const gBottom = Math.max(spanFade(t, r.edgeSpans), spanFade(t, r.cornerSpans));
        drawGuide(r.g1, gTop, false, A.y, Math.min(A.x, B.x), Math.max(A.x + A.w, B.x + B.w));
        drawGuide(r.g2, gBottom, false, A.y + A.h, Math.min(A.x, B.x), Math.max(A.x + A.w, B.x + B.w));

        drawPointer(r.ptr, t, p, { presses: PRESSES, kinds: KINDS, chips: CHIPS });
    }

    PM.register('move', { duration: D, poster: 2.5, build, render });
})();
