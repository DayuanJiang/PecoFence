# Builds a side-by-side before/after animation (WebM) from two rec.ps1 recordings.
# Usage: uv run --with pillow python mkanim.py spec.json
# spec.json (paths relative to the spec file):
# {
#   "out": "S1-explorer-busy",                       # writes <out>.webm and <out>-sheet.jpg next to the spec
#   "title": "Explorer 忙时拖动栅栏",                  # what the viewer does / sees, one line
#   "subtitle": "桌面一次进来 2000 个文件，同时拖动一个栅栏",  # optional
#   "before": {"dir": "rec-S1-before", "result": "✗ 松开 21 秒后栅栏才动"},
#   "after":  {"dir": "rec-S1-after",  "result": "✓ 松开后马上停在新位置"},
#   "steps": ["开始拖动标题", "松开鼠标", "等待"],        # optional; else the marks' texts (same count both sides)
#   "crop": [x, y, w, h],                              # optional zoom (recorded pixels, same for both sides)
#   "highlights": [{"step": 2, "side": "before", "rect": [x, y, w, h], "text": "还在原处", "keep": false}],
#                                                      # keep: also show it on the final result frames
#   "metric": {"label": "命令行等待", "unit": "秒"},     # optional chart of metric.tsv under each side
#   "fast_after_s": 5, "fast_to_s": 2.5,               # steps longer than this play faster, with a badge
#   "slow": {"2": 4},                                  # optional slow motion per step number (1-based)
#   "fps": 10, "width": 1400, "hold_s": 3.5
# }
import bisect
import io
import json
import math
import pathlib
import subprocess
import sys

from PIL import Image, ImageDraw, ImageFont

FONT = "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc"
FFMPEG = str(pathlib.Path.home() / ".cache/ms-playwright/ffmpeg-1013/ffmpeg-linux")
RED, GREEN, INK, MUTED = (217, 48, 37), (24, 128, 56), (29, 35, 48), (92, 101, 119)


def font(size):
    return ImageFont.truetype(FONT, size)


class Side:
    def __init__(self, base, conf):
        self.dir = (base / conf["dir"]).resolve()
        self.result = conf.get("result", "")
        self.frames = []  # (ms, index, x, y, buttons)
        for line in (self.dir / "frames.tsv").read_text(encoding="utf-8-sig").splitlines():
            p = line.split("\t")
            if len(p) >= 5:
                self.frames.append((int(p[1]), int(p[0]), int(p[2]), int(p[3]), int(p[4])))
        self.frames.sort()
        self.ms = [f[0] for f in self.frames]
        self.marks = []
        for line in (self.dir / "marks.tsv").read_text(encoding="utf-8-sig").splitlines():
            p = line.split("\t", 1)
            if len(p) == 2:
                self.marks.append((int(p[0]), p[1]))
        self.metric = []
        mf = self.dir / "metric.tsv"
        if mf.exists():
            for line in mf.read_text(encoding="utf-8-sig").splitlines():
                p = line.split("\t")
                if len(p) == 2:
                    self.metric.append((int(p[0]), float(p[1])))
        self.end = self.ms[-1] if self.ms else 0
        self.cache = {}

    def seg(self, i):
        start = self.marks[i][0]
        stop = self.marks[i + 1][0] if i + 1 < len(self.marks) else self.end
        return start, max(0, stop - start)

    def frame_at(self, ms):
        k = max(0, bisect.bisect_right(self.ms, ms) - 1)
        f = self.frames[k]
        if f[1] not in self.cache:
            if len(self.cache) > 40:
                self.cache.clear()
            self.cache[f[1]] = Image.open(self.dir / f"{f[1]:05d}.jpg").convert("RGB")
        return self.cache[f[1]], f

    def metric_upto(self, ms):
        return [(t, v) for t, v in self.metric if t <= ms]


def draw_cursor(d, x, y, s, down):
    if down:
        d.ellipse([x - 14 * s, y - 14 * s, x + 14 * s, y + 14 * s], outline=(255, 196, 0), width=max(2, int(3 * s)))
    pts = [(0, 0), (0, 17), (4.5, 13), (7.5, 20), (10, 19), (7, 12), (12.5, 12)]
    poly = [(x + px * s * 1.3, y + py * s * 1.3) for px, py in pts]
    d.polygon(poly, fill=(255, 255, 255), outline=(0, 0, 0))


def chip(d, xy, text, fill, f, pad=(10, 5)):
    x, y = xy
    w = d.textlength(text, font=f)
    h = f.size
    d.rounded_rectangle([x, y, x + w + 2 * pad[0], y + h + 2 * pad[1]], radius=8, fill=fill)
    d.text((x + pad[0], y + pad[1] - 1), text, font=f, fill=(255, 255, 255))
    return x + w + 2 * pad[0]


def main():
    spec_path = pathlib.Path(sys.argv[1]).resolve()
    base = spec_path.parent
    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    sides = [Side(base, spec["before"]), Side(base, spec["after"])]
    n = min(len(sides[0].marks), len(sides[1].marks))
    if n == 0 or len(sides[0].marks) != len(sides[1].marks):
        raise SystemExit(f"marks differ or missing: before {len(sides[0].marks)}, after {len(sides[1].marks)}")
    steps = spec.get("steps") or [m[1] for m in sides[0].marks]
    fps = spec.get("fps", 10)
    W = spec.get("width", 1400)
    pad = 16
    first = sides[0].frame_at(0)[0]
    crop = spec.get("crop") or [0, 0, first.width, first.height]
    pw = (W - 3 * pad) // 2
    sc = pw / crop[2]
    ph = int(crop[3] * sc)
    metric = spec.get("metric")
    mh = 70 if metric else 0
    top = 78 if spec.get("subtitle") else 52
    H = top + 40 + ph + mh + 64
    H += H % 2
    ftitle, fsub, fchip, fcap, fbig, fsmall = font(24), font(16), font(17), font(19), font(22), font(14)

    # Timeline: each step plays as long as the longer side needs (sped up or slowed if asked).
    plan = []
    for i in range(n):
        lens = [s.seg(i)[1] for s in sides]
        dur = max(lens) / 1000.0
        slow = float(spec.get("slow", {}).get(str(i + 1), 0) or 0)
        if slow > 0:
            speed, badge = 1.0 / slow, f"慢放 ×{slow:g}"
        elif dur > spec.get("fast_after_s", 5):
            k = max(2, math.ceil(dur / spec.get("fast_to_s", 2.5)))
            speed, badge = float(k), f"快进 ×{k}"
        else:
            speed, badge = 1.0, ""
        plan.append((i, dur, speed, badge))
    hold = spec.get("hold_s", 3.5)
    frames_out = []
    for i, dur, speed, badge in plan:
        count = max(1, int(round(dur / speed * fps)))
        for j in range(count):
            frames_out.append((i, j / fps * speed, badge, False))
    for j in range(int(hold * fps)):
        frames_out.append((n - 1, plan[-1][1] + 1, "", True))

    out = base / (spec.get("out", spec_path.stem))
    # This ffmpeg build reads only files: write the frames as one MJPEG stream first.
    mjpeg = pathlib.Path(str(out) + ".frames.mjpeg")
    enc = open(mjpeg, "wb")
    sheet = []
    # The sheet shows how each step ends (the last frame of every step) and the result.
    pick = {k for k, fo in enumerate(frames_out) if not fo[3] and (k + 1 == len(frames_out) or frames_out[k + 1][0] != fo[0] or frames_out[k + 1][3])}
    pick.add(len(frames_out) - 1)
    for idx, (i, r, badge, final) in enumerate(frames_out):
        img = Image.new("RGB", (W, H), (246, 247, 249))
        d = ImageDraw.Draw(img)
        d.text((pad, 12), spec["title"], font=ftitle, fill=INK)
        if spec.get("subtitle"):
            d.text((pad, 46), spec["subtitle"], font=fsub, fill=MUTED)
        for k, s in enumerate(sides):
            x0 = pad + k * (pw + pad)
            y0 = top + 40
            chip(d, (x0, top + 4), "修复前（0.3.1）" if k == 0 else "修复后", RED if k == 0 else GREEN, fchip)
            start, length = s.seg(i)
            real = min(r * 1000.0, length)
            fr, f = s.frame_at(start + real)
            view = fr.crop((crop[0], crop[1], crop[0] + crop[2], crop[1] + crop[3])).resize((pw, ph), Image.LANCZOS)
            img.paste(view, (x0, y0))
            d.rectangle([x0 - 1, y0 - 1, x0 + pw, y0 + ph], outline=(200, 205, 214))
            cx, cy = (f[2] - crop[0]) * sc + x0, (f[3] - crop[1]) * sc + y0
            if x0 <= cx <= x0 + pw and y0 <= cy <= y0 + ph:
                draw_cursor(d, cx, cy, max(0.8, sc * 1.6), f[4] != 0)
            for hl in spec.get("highlights", []):
                if hl.get("side", "both") in ("both", "before" if k == 0 else "after") and (hl["step"] - 1 == i or (final and hl.get("keep", False))):
                    hx, hy, hw, hh = hl["rect"]
                    rx0, ry0 = (hx - crop[0]) * sc + x0, (hy - crop[1]) * sc + y0
                    col = RED if k == 0 else GREEN
                    d.rounded_rectangle([rx0, ry0, rx0 + hw * sc, ry0 + hh * sc], radius=6, outline=col, width=4)
                    if hl.get("text"):
                        chip(d, (rx0, max(y0, ry0 - 30)), hl["text"], col, fsmall, (7, 4))
            # Step timer: runs while this side is still busy with the step, then turns green.
            # (The step's last frame counts as done: the next output frame is the next step.)
            done = (r + plan[i][2] / fps) * 1000.0 >= length
            t = f"用时 {real / 1000.0:.1f} 秒"
            tw = d.textlength(t, font=fchip) + 20
            d.rounded_rectangle([x0 + pw - tw - 8, y0 + 8, x0 + pw - 8, y0 + 8 + 30], radius=8,
                                fill=(24, 128, 56) if done else (40, 44, 52))
            d.text((x0 + pw - tw + 2, y0 + 12), t, font=fchip, fill=(255, 255, 255))
            if metric and s.metric:
                my = y0 + ph + 8
                pts = s.metric_upto(start + real)
                allv = [v for _, v in sides[0].metric + sides[1].metric] or [1]
                vmax = metric.get("max") or max(allv) or 1
                tmax = max(sides[0].end, sides[1].end) or 1
                d.rectangle([x0, my, x0 + pw, my + mh - 14], fill=(255, 255, 255), outline=(220, 224, 230))
                if len(pts) > 1:
                    line = [(x0 + tt / tmax * pw, my + mh - 16 - v / vmax * (mh - 22)) for tt, v in pts]
                    d.line(line, fill=RED if k == 0 else GREEN, width=3)
                if pts:
                    d.text((x0 + 8, my + 4), f"{metric['label']} {pts[-1][1]:g} {metric.get('unit', '')}", font=fsmall, fill=INK)
            if final and s.result:
                col = RED if k == 0 else GREEN
                text = s.result.lstrip("✗✓✔✘ ")
                bw = d.textlength(text, font=fbig) + 64
                bx, by = x0 + (pw - bw) / 2, y0 + ph - 60
                d.rounded_rectangle([bx, by, bx + bw, by + 46], radius=10, fill=col)
                ix, iy = bx + 16, by + 13
                if k == 0:  # a cross for the old build, a check mark for the fixed one
                    d.line([(ix, iy), (ix + 20, iy + 20)], fill=(255, 255, 255), width=4)
                    d.line([(ix + 20, iy), (ix, iy + 20)], fill=(255, 255, 255), width=4)
                else:
                    d.line([(ix, iy + 11), (ix + 7, iy + 19), (ix + 21, iy + 2)], fill=(255, 255, 255), width=4, joint="curve")
                d.text((bx + 48, by + 10), text, font=fbig, fill=(255, 255, 255))
        cap = f"第 {i + 1} 步，共 {n} 步：{steps[i]}" if not final else "结果"
        cy = H - 50
        d.rounded_rectangle([pad, cy, W - pad, cy + 40], radius=10, fill=(32, 36, 46))
        d.text((pad + 16, cy + 9), cap, font=fcap, fill=(255, 255, 255))
        if badge:
            bw = d.textlength(badge, font=fchip) + 22
            d.rounded_rectangle([W - pad - bw - 10, cy + 6, W - pad - 10, cy + 34], radius=8, fill=(255, 196, 0))
            d.text((W - pad - bw + 1, cy + 9), badge, font=fchip, fill=(32, 36, 46))
        buf = io.BytesIO()
        img.save(buf, "JPEG", quality=88)
        enc.write(buf.getvalue())
        if idx in pick:
            sheet.append(img.copy())
    enc.close()
    cmd = [FFMPEG, "-y", "-loglevel", "error", "-f", "image2pipe", "-c:v", "mjpeg", "-framerate", str(fps),
           "-i", "file:" + str(mjpeg), "-c:v", "libvpx-vp9", "-b:v", "0", "-crf", "36", "-deadline", "good",
           "-cpu-used", "4", "-pix_fmt", "yuv420p", str(out) + ".webm"]
    rc = subprocess.run(cmd).returncode
    mjpeg.unlink()
    if rc != 0:
        raise SystemExit("ffmpeg failed")
    sw = W // 2
    sh = int(H * sw / W)
    canvas = Image.new("RGB", (sw * 2, sh * ((len(sheet) + 1) // 2)), (255, 255, 255))
    for k, im in enumerate(sheet):
        canvas.paste(im.resize((sw, sh)), ((k % 2) * sw, (k // 2) * sh))
    canvas.save(str(out) + "-sheet.jpg", quality=85)
    secs = len(frames_out) / fps
    print(f"wrote {out}.webm ({secs:.1f} s, {len(frames_out)} frames) and {out}-sheet.jpg")


main()
