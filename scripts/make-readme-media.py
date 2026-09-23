"""Build README and share artwork from the revision-2 native desktop capture.

Requires Pillow and FFmpeg. The original recordings remain in the optional local
video project; the small, selected PNG/GIF outputs are checked in under docs/assets.
One hero image is rendered per README language (see HERO_TEXT).
"""
import argparse
import json
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
PROMO = ROOT / "extras/pecofence-promo"
OUTPUT = ROOT / "docs/assets"
FONTS = Path("C:/Windows/Fonts")


# Regular/bold font files per script. Segoe UI covers Latin and Cyrillic.
FONT_FILES = {
    "latin": ("segoeui.ttf", "seguisb.ttf"),
    "zh-CN": ("msyh.ttc", "msyhbd.ttc"),
    "zh-TW": ("msjh.ttc", "msjhbd.ttc"),
    "ja": ("YuGothM.ttc", "YuGothB.ttc"),
    "ko": ("malgun.ttf", "malgunbd.ttf"),
}

# Share presentation copy and the language list with the Store and homepage.
LANGUAGES = json.loads((ROOT / "site/site.json").read_text(encoding="utf-8"))["languages"]
HERO_TEXT = {}
for entry in LANGUAGES:
    code = entry["code"]
    store_code = "en-US" if code == "en" else code
    copy = json.loads((ROOT / "docs/store/v2-i18n" / f"{store_code}.json").read_text(encoding="utf-8"))
    HERO_TEXT[code] = (copy["titles"][0], copy["sub"][0], copy["captions"][0], copy["label"])


def font(size, *, bold=False, script="latin"):
    regular, heavy = FONT_FILES.get(script, FONT_FILES["latin"])
    return ImageFont.truetype(str(FONTS / (heavy if bold else regular)), size)


def fit(draw, text, size, max_width, **kwargs):
    face = font(size, **kwargs)
    while size > 10 and draw.textlength(text, font=face) > max_width:
        size -= 1
        face = font(size, **kwargs)
    return face


def hero(language):
    headline, subline, caption, category = HERO_TEXT[language]
    script = language if language in FONT_FILES else "latin"
    source = ROOT / ".cache/store-v2/native/overview.png"
    image = Image.open(source).convert("RGB").resize((1600, 900), Image.Resampling.LANCZOS)
    draw = ImageDraw.Draw(image)
    draw.text((64, 26), "PecoFence", font=font(18, bold=True), fill="#233048", anchor="lt")
    draw.text((185, 29), category, font=font(13, script=script), fill="#596277", anchor="lt")
    face = fit(draw, headline, 57, 1475, bold=True, script=script)
    draw.text((60, 78), headline, font=face, fill="#233048", anchor="lt")
    face = fit(draw, subline, 20, 1468, script=script)
    draw.text((64, 156), subline, font=face, fill="#596277", anchor="lt")
    image.save(OUTPUT / f"hero-{language}.png", optimize=True)


def gif(name, start, duration):
    # The crop keeps the real app and keyboard callouts, excluding the video's
    # English chapter headings so the same recording works in both READMEs.
    filters = (
        "crop=1760:720:80:255,fps=12,scale=960:-2:flags=lanczos,split[a][b];"
        "[a]palettegen=max_colors=128:stats_mode=diff[p];"
        "[b][p]paletteuse=dither=bayer:bayer_scale=3"
    )
    recording = PROMO / "out/PecoFence-features-en-1080p.mp4"
    if not recording.exists():
        # These branding-free crops can reuse the original pre-rename recording.
        recording = PROMO / "out/openFence-features-en-1080p.mp4"
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-y",
        "-ss", str(start), "-t", str(duration),
        "-i", str(recording),
        "-filter_complex", filters, "-an", "-loop", "0",
        str(OUTPUT / f"{name}.gif"),
    ], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stills-only", action="store_true")
    parser.add_argument("--languages", nargs="*", default=sorted(HERO_TEXT),
                        help="hero images to render (default: every language)")
    args = parser.parse_args()
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for language in args.languages:
        hero(language)
    if not args.stills_only:
        gif("tabs", 40.0, 9.7)
        gif("peek", 50.8, 10.0)
    for path in sorted(OUTPUT.glob("*")):
        if path.is_file():
            print(f"{path.name}: {path.stat().st_size / 1024:.0f} KiB")


if __name__ == "__main__":
    main()
