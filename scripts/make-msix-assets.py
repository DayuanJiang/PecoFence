"""Render the Store/tile PNGs for the MSIX package from site/assets/mark.svg.

The mark is four rounded corner brackets plus a 2x2 grid of rounded squares; it is
redrawn here with Pillow (no SVG renderer needed) at every scale the Store expects.

    python scripts/make-msix-assets.py <output directory>
"""

import sys
from pathlib import Path

from PIL import Image, ImageDraw

ACCENT = (0x7B, 0xD2, 0xFF)
SUPERSAMPLE = 8

# Logical name -> (base width, base height) at 100% scale.
TILES = {
    "Square44x44Logo": (44, 44),
    "Square150x150Logo": (150, 150),
    "SmallTile": (71, 71),
    "LargeTile": (310, 310),
    "Wide310x150Logo": (310, 150),
    "StoreLogo": (50, 50),
    "SplashScreen": (620, 300),
}
SCALES = (100, 125, 150, 200, 400)
# Unplated taskbar / Start icons: the square 44 asset also ships in target sizes.
TARGET_SIZES = (16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256)


def draw_mark(canvas: Image.Image, cx: float, cy: float, size: float) -> None:
    """Draw the 64-unit mark centered at (cx, cy) with `size` pixels per 64 units."""
    s = SUPERSAMPLE
    big = Image.new("RGBA", (canvas.width * s, canvas.height * s), (0, 0, 0, 0))
    d = ImageDraw.Draw(big)
    unit = size * s / 64.0
    ox = cx * s - 32 * unit
    oy = cy * s - 32 * unit

    def pt(x, y):
        return (ox + x * unit, oy + y * unit)

    def box(x0, y0, x1, y1):
        return [pt(x0, y0), pt(x1, y1)]

    stroke = int(round(6 * unit))
    # Corner brackets: arcs of radius 6 (centre offset 6 from the corners) plus straight legs.
    # Path: M27 8 H14 a6 6 0 0 0 -6 6 V27, etc. Draw as thick lines with round caps.
    def leg(a, b):
        d.line([pt(*a), pt(*b)], fill=ACCENT, width=stroke)
        r = stroke / 2
        for x, y in (pt(*a), pt(*b)):
            d.ellipse([x - r, y - r, x + r, y + r], fill=ACCENT)

    def arc(cx_, cy_, start, end):
        # Stroke centre line radius 6; Pillow strokes inward from the box, so use the
        # outer radius 9 and a 6-unit band to cover radii 3..9.
        d.arc(box(cx_ - 9, cy_ - 9, cx_ + 9, cy_ + 9), start, end, fill=ACCENT, width=stroke)

    # Top-left
    leg((27, 8), (14, 8)); arc(14, 14, 180, 270); leg((8, 14), (8, 27))
    # Top-right
    leg((37, 8), (50, 8)); arc(50, 14, 270, 360); leg((56, 14), (56, 27))
    # Bottom-left
    leg((8, 37), (8, 50)); arc(14, 50, 90, 180); leg((14, 56), (27, 56))
    # Bottom-right
    leg((56, 37), (56, 50)); arc(50, 50, 0, 90); leg((50, 56), (37, 56))

    def square(x, y, alpha):
        d.rounded_rectangle(box(x, y, x + 8, y + 8), radius=2 * unit, fill=ACCENT + (int(255 * alpha),))

    square(22, 22, 1.0)
    square(35, 22, 0.65)
    square(22, 35, 0.65)
    square(35, 35, 1.0)

    small = big.resize(canvas.size, Image.LANCZOS)
    canvas.alpha_composite(small)


def render(width: int, height: int, margin: float = 0.18) -> Image.Image:
    img = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    size = min(width, height) * (1 - 2 * margin)
    draw_mark(img, width / 2, height / 2, size)
    return img


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    out = Path(sys.argv[1])
    out.mkdir(parents=True, exist_ok=True)
    count = 0
    for name, (w, h) in TILES.items():
        for scale in SCALES:
            f = scale / 100
            render(round(w * f), round(h * f)).save(out / f"{name}.scale-{scale}.png")
            count += 1
    for size in TARGET_SIZES:
        # Taskbar / Start list icons fill the whole canvas.
        render(size, size, margin=0.04).save(out / f"Square44x44Logo.targetsize-{size}.png")
        render(size, size, margin=0.04).save(out / f"Square44x44Logo.targetsize-{size}_altform-unplated.png")
        count += 2
    print(f"wrote {count} PNGs to {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
