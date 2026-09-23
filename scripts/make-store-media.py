"""Prepare Microsoft Store listing assets from the native demo recordings.

uv run --with pillow --with playwright python scripts/make-store-media.py
Add --skip-trailer when iterating on the artwork or copy.
Requires FFmpeg and an installed Edge or Chrome. Output: dist/store-listing/.
The original local recordings are not included in the source distribution.
"""
import argparse
import hashlib
import html
import importlib.util
import json
import shutil
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont
from playwright.sync_api import sync_playwright

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / "extras/pecofence-promo/public"
OUTPUT = ROOT / "dist/store-listing"
SOURCE = ROOT / "docs/store"
SHOTS = [
    ("01-desktop", "launch/raw/hero-result.png", None, None),
    ("02-peek", "launch/raw/peek-raw.mp4", 7.0, None),
    ("03-tabs", "launch/raw/tabs-raw.mp4", 6.0, (0, 230, 1536, 1094)),
    ("04-auto-sort", "autosort/auto-v2/automatically-grouped.png", None, (800, 280, 2528, 1252)),
    ("05-live-folder", "features/portal-result.png", None, (400, 190, 2256, 1234)),
]


def ffmpeg(*args):
    subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", *map(str, args)], check=True)


def screenshots():
    output = OUTPUT / "screenshots"
    output.mkdir(parents=True, exist_ok=True)
    cache = ROOT / ".cache/store-review"
    cache.mkdir(parents=True, exist_ok=True)
    provenance = []
    for name, relative, second, crop in SHOTS:
        source = PUBLIC / relative
        if second is not None:
            frame = cache / f"{name}-source.png"
            ffmpeg("-ss", second, "-i", source, "-frames:v", 1, "-update", 1, frame)
        else:
            frame = source
        with Image.open(frame) as original:
            image = original.convert("RGB")
            if crop:
                assert 0 <= crop[0] < crop[2] <= image.width
                assert 0 <= crop[1] < crop[3] <= image.height
                image = image.crop(crop)
            assert image.width * 9 == image.height * 16, (name, image.size)
            image = image.resize((1920, 1080), Image.Resampling.LANCZOS)
            image.save(output / f"{name}.png", optimize=True)
        provenance.append({
            "output": f"screenshots/{name}.png",
            "source": source.relative_to(ROOT).as_posix(),
            "sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "frameSeconds": second,
            "crop": crop,
            "editing": "Native pixels only. Rectangular crop and proportional resize; no overlays or compositing.",
        })
    (OUTPUT / "provenance.json").write_text(
        json.dumps(provenance, indent=2) + "\n", encoding="utf-8")


def artwork():
    output = OUTPUT / "art"
    output.mkdir(exist_ok=True)
    executable = next((path for path in [
        Path("C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe"),
        Path("C:/Program Files/Google/Chrome/Application/chrome.exe"),
    ] if path.exists()), None)
    if executable is None:
        raise SystemExit("Install Edge or Chrome to render the Store hero SVG.")
    with sync_playwright() as p:
        browser = p.chromium.launch(executable_path=str(executable), headless=True)
        page = browser.new_page(viewport={"width": 1920, "height": 1080}, device_scale_factor=1)
        page.set_content('<html><body style="margin:0">' + (SOURCE / "hero.svg").read_text(encoding="utf-8") + "</body></html>")
        page.screenshot(path=str(output / "super-hero-1920x1080.png"))
        browser.close()
    # Reuse the actual package icon renderer; this does not change the app icon.
    spec = importlib.util.spec_from_file_location("msix_assets", ROOT / "scripts/make-msix-assets.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.render_icon(300, 300).save(output / "app-icon-300x300.png")


def trailer(encode):
    output = OUTPUT / "trailer"
    output.mkdir(exist_ok=True)
    source = PUBLIC.parent / "out/PecoFence-autosort-en-1080p.mp4"
    # An actual frame of the same trailer, with its existing on-screen title.
    ffmpeg("-ss", 3.5, "-i", source, "-frames:v", 1, "-update", 1, output / "thumbnail-1920x1080.png")
    if encode:
        ffmpeg("-i", source, "-c:v", "libx264", "-preset", "medium",
               "-profile:v", "high", "-pix_fmt", "yuv420p",
               "-b:v", "50M", "-minrate", "50M", "-maxrate", "50M", "-bufsize", "100M",
               "-g", 15, "-keyint_min", 15, "-sc_threshold", 0, "-bf", 2,
               "-x264-params", "open-gop=0:b-adapt=0:nal-hrd=cbr",
               "-c:a", "aac", "-b:a", "384k", "-ar", 48000, "-ac", 2,
               "-use_editlist", 0, "-movflags", "+faststart", output / "PecoFence-30s-en.mp4")


def copy_and_preview():
    listings = json.loads((SOURCE / "listings.json").read_text(encoding="utf-8"))
    output = OUTPUT / "listings"
    output.mkdir(exist_ok=True)
    for language, listing in listings.items():
        assert len(listing["shortDescription"]) <= 270
        assert len(listing["description"]) <= 10000
        assert len(listing["keywords"]) <= 7
        assert len(listing["captions"]) == len(SHOTS)
        assert all(len(caption) <= 200 for caption in listing["captions"])
        assert all(len(feature) <= 200 for feature in listing["features"])
        assert len(listing["trailerTitle"]) <= 255
        folder = output / language
        folder.mkdir(exist_ok=True)
        for field in ("shortDescription", "description", "trailerTitle"):
            (folder / f"{field}.txt").write_text(listing[field] + "\n", encoding="utf-8")
        for field in ("features", "keywords", "captions"):
            (folder / f"{field}.txt").write_text("\n".join(listing[field]) + "\n", encoding="utf-8")
    shutil.copy2(SOURCE / "listings.json", output / "listings.json")
    listing = listings["zh-CN"]
    cards = "\n".join(
        f'<figure><a href="screenshots/{name}.png"><img src="screenshots/{name}.png" alt="{html.escape(caption)}"></a>'
        f'<figcaption><b>{index:02d}</b> {html.escape(caption)}</figcaption></figure>'
        for index, ((name, *_), caption) in enumerate(zip(SHOTS, listing["captions"]), 1)
    )
    copy_links = "".join(f'<a href="listings/{lang}/description.txt">{lang} 文案 ↗</a>' for lang in listings)
    video = '<video controls preload="none" poster="trailer/thumbnail-1920x1080.png" src="trailer/PecoFence-30s-en.mp4"></video>' if (OUTPUT / "trailer/PecoFence-30s-en.mp4").exists() else '<img src="trailer/thumbnail-1920x1080.png" alt="宣传片封面">'
    page = f'''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>PecoFence · Microsoft Store 素材预览</title><style>
*{{box-sizing:border-box}}body{{margin:0;background:#f6f7fb;color:#19243a;font:16px/1.8 "Segoe UI","Microsoft YaHei",sans-serif}}
main{{max-width:1120px;margin:auto;padding:48px 24px}}h1{{font-size:38px;letter-spacing:-.04em;margin:12px 0}}
h2{{font-size:27px;margin:54px 0 20px}}p{{max-width:760px;color:#566177}}a{{color:#3452bf;text-underline-offset:4px}}
nav{{display:flex;flex-wrap:wrap;gap:25px}}img,video{{width:100%;height:auto;display:block;border-radius:13px}}
figure{{margin:0}}figcaption{{padding:14px 0 24px;color:#566177;font-size:13px}}b{{color:#3452bf;margin-right:8px}}
.grid{{display:grid;grid-template-columns:1fr 1fr;gap:24px}}.note{{font-size:13px}}.copy{{white-space:pre-wrap;background:white;padding:28px;border:1px solid #dfe4ed;border-radius:12px}}
@media(max-width:680px){{.grid{{grid-template-columns:1fr}}h1{{font-size:30px}}}}
</style><main><span>PecoFence / Microsoft Store</span><h1>桌面清爽。灵感自在。</h1>
<p>上架素材预览。截图使用真实演示画面，宣传文字放在商店标题和说明字段中。</p><nav>{copy_links}<a href="listings/listings.json">全部字段 JSON ↗</a></nav>
<h2>独立主视觉</h2><img src="art/super-hero-1920x1080.png" alt="以 PecoFence 四角与四方块标志延展的玻璃主题主视觉">
<p class="note">1920 × 1080 PNG · 无文字 · 用于 Super hero art 字段，商店实际显示位置由微软决定。</p>
<h2>30 秒宣传片</h2>{video}<p class="note">原有宣传片的商店编码版本，画面文字为英文。不要用官网的 720p 文件上传。</p>
<h2>截图顺序与说明</h2><div class="grid">{cards}</div>
<h2>中文简介</h2><div class="copy">{html.escape(listing["shortDescription"])}</div>
<h2>完整说明</h2><div class="copy">{html.escape(listing["description"])}</div>
<p class="note">应用内截图为英文演示文件；中文、英文、日文说明分别在 listings/ 下。此页面用于预览，不是已发布的商店页面。</p></main></html>'''
    (OUTPUT / "index.html").write_text(page, encoding="utf-8")


def contact_sheet():
    sheet = Image.new("RGB", (1440, 1190), "#f6f7fb")
    draw = ImageDraw.Draw(sheet)
    font = ImageFont.truetype("C:/Windows/Fonts/segoeui.ttf", 20)
    assets = [("SUPER HERO / NO TEXT", OUTPUT / "art/super-hero-1920x1080.png")]
    assets += [(name.upper(), OUTPUT / f"screenshots/{name}.png") for name, *_ in SHOTS]
    for index, (label, path) in enumerate(assets):
        x, y = 24 + (index % 2) * 712, 20 + (index // 2) * 390
        with Image.open(path) as image:
            image = image.convert("RGB").resize((688, 387), Image.Resampling.LANCZOS)
            # Preview only: labels are outside the upload images.
            image.thumbnail((688, 344), Image.Resampling.LANCZOS)
            sheet.paste(image, (x, y))
        draw.text((x, y + 351), label, font=font, fill="#19243a")
    sheet.save(OUTPUT / "contact-sheet.jpg", quality=90)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skip-trailer", action="store_true")
    args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    screenshots()
    artwork()
    trailer(not args.skip_trailer)
    copy_and_preview()
    contact_sheet()
    print(f"Prepared 5 screenshots, hero art, icon, trailer thumbnail and 3 listings: {OUTPUT}")


if __name__ == "__main__":
    main()
