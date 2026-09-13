"""Build README artwork from the project's existing native demo captures.

Requires Pillow and FFmpeg. The original recordings remain in the optional local
video project; the small, selected PNG/GIF outputs are checked in under docs/assets.
One hero image is rendered per README language (see HERO_TEXT).
"""
import argparse
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

# Headline, subline, caption and badge painted onto each localized hero image.
# Keep these in step with the matching README in docs/readme/.
HERO_TEXT = {
    "en": ("A calmer desktop. Everything within reach.",
           "Glass panels. Live folders. Your desktop, one shortcut away.",
           "An actual PecoFence demo desktop · Fluent theme", "10 LANGUAGES"),
    "zh-CN": ("把桌面还给壁纸，把文件放在手边。",
              "玻璃栅栏 · 文件夹门户 · 标签页 · 随时浮现",
              "PecoFence 实际演示桌面 · Fluent 主题", "10 种语言"),
    "zh-TW": ("把桌面還給桌布，把檔案放在手邊。",
              "玻璃圍欄 · 資料夾入口 · 分頁 · 隨時浮現",
              "PecoFence 實際示範桌面 · Fluent 主題", "10 種語言"),
    "ja": ("静かなデスクトップ。すべてが手の届く場所に。",
           "ガラスのフェンス · フォルダーポータル · タブ · ショートカットひとつで手前に",
           "PecoFence の実際のデモデスクトップ · Fluent テーマ", "10 言語対応"),
    "ko": ("더 차분한 바탕 화면. 모든 것이 손닿는 곳에.",
           "유리 패널. 살아 있는 폴더. 단축키 하나 거리의 바탕 화면.",
           "실제 PecoFence 데모 바탕 화면 · Fluent 테마", "10개 언어"),
    "de": ("Ein ruhigerer Desktop. Alles in Reichweite.",
           "Glasflächen. Live-Ordner. Ihr Desktop, eine Tastenkombination entfernt.",
           "Ein echter PecoFence-Demo-Desktop · Fluent-Thema", "10 SPRACHEN"),
    "fr": ("Un Bureau plus calme. Tout à portée de main.",
           "Panneaux de verre. Dossiers en direct. Votre Bureau, à un raccourci.",
           "Un vrai Bureau de démonstration PecoFence · Thème Fluent", "10 LANGUES"),
    "es": ("Un escritorio más tranquilo. Todo a tu alcance.",
           "Paneles de cristal. Carpetas en vivo. Tu escritorio, a un atajo de distancia.",
           "Un escritorio de demostración real de PecoFence · Tema Fluent", "10 IDIOMAS"),
    "pt-BR": ("Uma área de trabalho mais tranquila. Tudo ao seu alcance.",
              "Painéis de vidro. Pastas ao vivo. Sua área de trabalho a um atalho de distância.",
              "Uma área de trabalho real de demonstração do PecoFence · Tema Fluent", "10 IDIOMAS"),
    "ru": ("Спокойный рабочий стол. Всё под рукой.",
           "Стеклянные панели. Живые папки. Рабочий стол — одним сочетанием клавиш.",
           "Настоящий демо-рабочий стол PecoFence · тема Fluent", "10 ЯЗЫКОВ"),
}


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
    headline, subline, caption, badge = HERO_TEXT[language]
    script = language if language in FONT_FILES else "latin"
    cjk = script != "latin"
    screenshot = Image.open(PROMO / "public/product-desktop.png").convert("RGBA")
    # Preserve the actual panels; use the empty wallpaper above them for the headline.
    image = screenshot.crop((0, 0, 2560, 1152)).resize((1600, 720), Image.Resampling.LANCZOS)
    overlay = Image.new("RGBA", image.size)
    draw = ImageDraw.Draw(overlay)
    for y in range(260):
        draw.line((0, y, 1600, y), fill=(4, 13, 26, round(44 * (1 - y / 260))))
    image = Image.alpha_composite(image, overlay)
    draw = ImageDraw.Draw(image)
    blue = "#7bd2ff"
    x, y, size = 92, 49, 42
    for points in (
        [(x+12, y), (x+4, y), (x, y+4), (x, y+12)],
        [(x+30, y), (x+38, y), (x+42, y+4), (x+42, y+12)],
        [(x, y+30), (x, y+38), (x+4, y+42), (x+12, y+42)],
        [(x+30, y+42), (x+38, y+42), (x+42, y+38), (x+42, y+30)],
    ):
        draw.line(points, fill=blue, width=4, joint="curve")
    for ox, oy in ((13, 13), (25, 13), (13, 25), (25, 25)):
        draw.rounded_rectangle((x+ox-3, y+oy-3, x+ox+4, y+oy+4), 1, fill=blue)
    draw.text((151, 39), "PecoFence", font=font(48, bold=True), fill="#f5f9ff")
    right = 1508
    for label in reversed(["WINDOWS 11", badge, "MIT"]):
        face = font(15, script=script)
        width = int(draw.textlength(label, font=face)) + 30
        draw.rounded_rectangle((right-width, 51, right, 86), 17,
                               fill="#183d5d", outline="#426d8e")
        draw.text((right-width+15, 59), label, font=face, fill="#c4dff1")
        right -= width + 10
    face = fit(draw, headline, 57 if cjk else 58, 1416, bold=True, script=script)
    draw.text((92, 124), headline, font=face, fill="#f3f8ff")
    face = fit(draw, subline, 24, 1416, script=script)
    draw.text((96, 211), subline, font=face, fill="#b2cadf")
    draw.text((96, 663), caption, font=font(17, script=script), fill="#9fbad2")
    mask = Image.new("L", image.size)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, 1599, 719), radius=24, fill=255)
    image.putalpha(mask)
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
