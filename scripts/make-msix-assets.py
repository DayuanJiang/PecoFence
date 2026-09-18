"""Render the Store/tile PNGs for the MSIX package from site/assets/mark.svg.

The mark is four rounded corner brackets plus a 2x2 grid of rounded squares; it is
redrawn here with Pillow (no SVG renderer needed) at every scale the Store expects.

Icon style ("variant E"): white mark on an indigo rounded-square plate. The plate's
corner radius is 22% of the side and the mark is scaled to 64% of the side (18% margin).
Wide tiles and the splash screen are the white mark alone on a transparent background;
the manifest BackgroundColor (the same indigo) shows behind them.

    python scripts/make-msix-assets.py <output directory> [--preview <png>]
    python scripts/make-msix-assets.py --preview .cache/icon-sheet.png

`scripts/make-app-icon.py` reuses `render_icon` for the executable's .ico.
"""

import argparse
from pathlib import Path

from PIL import Image, ImageDraw

PLATE = (0x47, 0x68, 0xDE)
WHITE = (0xFF, 0xFF, 0xFF)
SUPERSAMPLE = 8
PLATE_RADIUS = 0.22  # of the plate side
TILE_MARGIN = 0.18  # mark inset on tiles: mark = 64% of the side
ICON_MARGIN = 0.12  # taskbar / Explorer icons: the plate is the icon, fill it a bit more

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
# Mark only, no plate; the manifest BackgroundColor fills the rest of the tile.
MARK_ONLY = {"Wide310x150Logo", "SplashScreen"}
SCALES = (100, 125, 150, 200, 400)
# Taskbar / Start list icons: the square 44 asset also ships in target sizes.
TARGET_SIZES = (16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256)


def draw_mark(d: ImageDraw.ImageDraw, cx: float, cy: float, size: float, color=WHITE) -> None:
    """Draw the 64-unit mark centred at (cx, cy) with `size` pixels per 64 units.

    Coordinates are in the target image of `d` (call with a supersampled image and scaled
    coordinates for anti-aliasing).
    """
    unit = size / 64.0
    ox = cx - 32 * unit
    oy = cy - 32 * unit

    def pt(x, y):
        return (ox + x * unit, oy + y * unit)

    def box(x0, y0, x1, y1):
        return [pt(x0, y0), pt(x1, y1)]

    stroke = max(1, int(round(6 * unit)))

    # Corner brackets: arcs of radius 6 (centre offset 6 from the corners) plus straight legs.
    # Path: M27 8 H14 a6 6 0 0 0 -6 6 V27, etc. Drawn as thick lines with round caps.
    def leg(a, b):
        d.line([pt(*a), pt(*b)], fill=color, width=stroke)
        r = stroke / 2
        for x, y in (pt(*a), pt(*b)):
            d.ellipse([x - r, y - r, x + r, y + r], fill=color)

    def arc(cx_, cy_, start, end):
        # Stroke centre line radius 6; Pillow strokes inward from the box, so use the
        # outer radius 9 and a 6-unit band to cover radii 3..9.
        d.arc(box(cx_ - 9, cy_ - 9, cx_ + 9, cy_ + 9), start, end, fill=color, width=stroke)

    # Top-left
    leg((27, 8), (14, 8)); arc(14, 14, 180, 270); leg((8, 14), (8, 27))
    # Top-right
    leg((37, 8), (50, 8)); arc(50, 14, 270, 360); leg((56, 14), (56, 27))
    # Bottom-left
    leg((8, 37), (8, 50)); arc(14, 50, 90, 180); leg((14, 56), (27, 56))
    # Bottom-right
    leg((56, 37), (56, 50)); arc(50, 50, 0, 90); leg((50, 56), (37, 56))

    def square(x, y, alpha):
        d.rounded_rectangle(box(x, y, x + 8, y + 8), radius=2 * unit, fill=color + (int(255 * alpha),))

    square(22, 22, 1.0)
    square(35, 22, 0.65)
    square(22, 35, 0.65)
    square(35, 35, 1.0)


def render_icon(width: int, height: int, *, plate: bool = True, margin: float = TILE_MARGIN) -> Image.Image:
    """Variant E at the given size: indigo rounded plate (square, centred) with the white mark.

    With plate=False only the mark is drawn, on a transparent background.
    """
    s = SUPERSAMPLE
    # Transparent pixels carry the colour of the outermost element so any resampling
    # halo at the edges stays in that colour.
    big = Image.new("RGBA", (width * s, height * s), (PLATE if plate else WHITE) + (0,))
    d = ImageDraw.Draw(big)
    side = min(width, height)
    cx, cy = width * s / 2, height * s / 2
    if plate:
        half = side * s / 2
        d.rounded_rectangle(
            [cx - half, cy - half, cx + half - 1, cy + half - 1],
            radius=PLATE_RADIUS * side * s,
            fill=PLATE + (255,),
        )
    draw_mark(d, cx, cy, side * s * (1 - 2 * margin))
    return big.resize((width, height), Image.LANCZOS)


def render_tile(name: str, width: int, height: int) -> Image.Image:
    return render_icon(width, height, plate=name not in MARK_ONLY)


def render_target(size: int) -> Image.Image:
    # Plated and unplated forms are identical: our plate is the icon.
    return render_icon(size, size, margin=ICON_MARGIN)


def write_assets(out: Path) -> int:
    out.mkdir(parents=True, exist_ok=True)
    count = 0
    for name, (w, h) in TILES.items():
        for scale in SCALES:
            f = scale / 100
            render_tile(name, round(w * f), round(h * f)).save(out / f"{name}.scale-{scale}.png")
            count += 1
    for size in TARGET_SIZES:
        icon = render_target(size)
        icon.save(out / f"Square44x44Logo.targetsize-{size}.png")
        icon.save(out / f"Square44x44Logo.targetsize-{size}_altform-unplated.png")
        count += 2
    return count


def write_preview(path: Path) -> None:
    """Contact sheet: target-size icons on light and dark, tiles on the manifest background."""
    pad = 16
    light, dark = (0xF3, 0xF3, 0xF3, 255), (0x20, 0x20, 0x20, 255)
    rows: list[tuple[str, tuple, list[tuple[str, Image.Image]]]] = []
    icons = [(f"{n}", render_target(n)) for n in TARGET_SIZES]
    rows.append(("targetsize (plated + unplated) on light", light, icons))
    rows.append(("targetsize on dark", dark, icons))
    tiles = [(f"{name} {w}x{h}", render_tile(name, w, h)) for name, (w, h) in TILES.items() if name != "SplashScreen"]
    rows.append(("tiles at scale-100 on BackgroundColor", PLATE + (255,), tiles))
    rows.append(("SplashScreen 620x300 on BackgroundColor", PLATE + (255,), [("SplashScreen", render_tile("SplashScreen", 620, 300))]))
    rows.append(("Square150x150Logo scale-200 / LargeTile scale-100", (0xFF, 0xFF, 0xFF, 255), [("150 @200", render_tile("Square150x150Logo", 300, 300)), ("310", render_tile("LargeTile", 310, 310))]))

    label_h = 18
    measure = ImageDraw.Draw(Image.new("RGBA", (1, 1)))

    def cell_width(label, img):
        return max(img.width, int(measure.textlength(label)) + 4)

    row_widths = [sum(cell_width(l, img) + pad for l, img in items) + pad for _, _, items in rows]
    row_heights = [max(img.height for _, img in items) + label_h + 2 * pad for _, _, items in rows]
    sheet = Image.new("RGBA", (max(row_widths), sum(row_heights)), (0xFF, 0xFF, 0xFF, 255))
    y = 0
    for (title, bg, items), h in zip(rows, row_heights):
        band = Image.new("RGBA", (sheet.width, h), bg)
        d = ImageDraw.Draw(band)
        text = (0, 0, 0, 255) if sum(bg[:3]) > 384 else (255, 255, 255, 255)
        d.text((pad, 4), title, fill=text)
        x = pad
        for label, img in items:
            band.alpha_composite(img, (x, label_h + pad))
            d.text((x, h - pad + 2), label, fill=text)
            x += cell_width(label, img) + pad
        sheet.alpha_composite(band, (0, y))
        y += h
    path.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(path)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("out", nargs="?", type=Path, help="directory for the MSIX Assets PNGs")
    ap.add_argument("--preview", type=Path, metavar="PNG", help="also write a contact sheet of all sizes")
    args = ap.parse_args()
    if args.out is None and args.preview is None:
        ap.error("give an output directory and/or --preview")
    if args.out is not None:
        count = write_assets(args.out)
        print(f"wrote {count} PNGs to {args.out}")
    if args.preview is not None:
        write_preview(args.preview)
        print(f"wrote preview {args.preview}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
