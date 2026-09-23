"""Prepare original graphic studies and isolated native Store demo fixtures.

uv run --with pillow --with numpy python scripts/prepare-store-v2.py
"""
import copy
import json
import shutil
import uuid
import zipfile
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
PROMO = ROOT / "extras/pecofence-promo"
BASE = PROMO / ".capture/reviewed/store-v2"
ASSETS = ROOT / ".cache/store-v2"
FONTS = Path("C:/Windows/Fonts")


def font(size, bold=False):
    return ImageFont.truetype(str(FONTS / ("seguisb.ttf" if bold else "segoeui.ttf")), size)


def wallpaper(name, dark=False):
    """Original flowing-paper artwork, rendered from an analytic curved surface."""
    w, h = 1920, 1080
    y, x = np.mgrid[0:h, 0:w].astype(np.float32)
    x /= w
    y /= h
    distance = (y - .60 + .27 * np.sin(x * 4.6 - .5) + .31 * x)
    folded = .5 + .5 * np.tanh(distance * 8)
    palette = (np.array([[248, 245, 239], [245, 225, 204], [232, 200, 186], [220, 215, 230], [196, 214, 219]], dtype=np.float32)
               if not dark else np.array([[22, 28, 47], [49, 50, 88], [115, 82, 146], [198, 119, 161], [242, 185, 161]], dtype=np.float32))
    # A broad lit fold and its soft shadow; no texture or stock-photo dependency.
    t = np.clip(folded * 3.1 + .32 * x, 0, 3.999)
    idx = np.floor(t).astype(int)
    frac = (t - idx)[..., None]
    pixels = palette[idx] * (1 - frac) + palette[idx + 1] * frac
    highlight = np.exp(-((distance + .13) / .026) ** 2) * (12 if dark else 20)
    shadow = np.exp(-((distance + .075) / .034) ** 2) * 11
    pixels += (highlight - shadow)[..., None]
    image = Image.fromarray(np.uint8(np.clip(pixels, 0, 255)))
    image.resize((3840, 2160), Image.Resampling.LANCZOS).save(ASSETS / f"{name}.png")


def studies():
    output = ASSETS / "studies"
    output.mkdir(exist_ok=True)
    colors = [("#e9e1d0", "#8d5445", "TERRA", "A quieter kind of design."),
              ("#d9e8e3", "#175e5b", "STILL", "Make space for the everyday."),
              ("#deddef", "#4a456c", "FORM", "Shapes. Light. A new perspective."),
              ("#e5ae79", "#693b41", "SOL", "A little warmth goes a long way."),
              ("#d9e4ef", "#335c80", "TIDE", "An identity in motion."),
              ("#eddfd5", "#8b544f", "ATELIER", "Objects with a point of view.")]
    for index, (paper, ink, title, line) in enumerate(colors):
        image = Image.new("RGB", (900, 1120), paper)
        d = ImageDraw.Draw(image)
        d.text((58, 45), "STUDIO / GRAPHIC STUDIES", font=font(21), fill=ink)
        d.text((55, 123), title, font=font(100, True), fill=ink)
        d.line((60, 270, 840, 270), fill=ink, width=2)
        # Original poster artwork doubles as meaningful image thumbnails in the app.
        if index % 3 == 0:
            for n in range(7):
                offset = n * 28
                d.arc((135 + offset, 342 + offset, 770 - offset, 920 - offset), 180, 360, fill=ink, width=12)
            d.rectangle((135, 640, 770, 820), fill=ink)
        elif index % 3 == 1:
            d.ellipse((140, 347, 745, 952), fill=ink)
            d.rectangle((105, 660, 810, 975), fill=paper)
            for n in range(7):
                d.line((135, 680 + 34 * n, 765, 680 + 34 * n), fill=ink, width=5)
        else:
            for n in range(5):
                d.rounded_rectangle((115 + n * 30, 350 + n * 83, 740 + n * 14, 430 + n * 83), radius=38, fill=ink)
        d.text((60, 1010), line, font=font(25), fill=ink)
        d.text((60, 1060), f"COLLECTION 0{index+1}                         2026", font=font(19), fill=ink)
        image.save(output / f"{title.title()} study.png")
    brief = Image.new("RGB", (850, 1100), "#faf8f4")
    d = ImageDraw.Draw(brief)
    d.text((70, 60), "TERRA / CREATIVE BRIEF", font=font(22), fill="#78594f")
    d.text((70, 155), "A calmer\nway to work.", font=font(64, True), fill="#272638", spacing=8)
    d.line((70, 365, 780, 365), fill="#d5cabe", width=2)
    for y, title, text in [(425, "01  PURPOSE", "Make the everyday feel considered."),
                           (570, "02  DIRECTION", "Warm materials. Simple shapes. Clear type."),
                           (715, "03  DELIVERABLES", "Identity / Website / Launch campaign")]:
        d.text((70, y), title, font=font(19, True), fill="#78594f")
        d.text((70, y + 45), text, font=font(24), fill="#272638")
    brief.save(output / "Creative brief.pdf", resolution=144)
    brief.save(ASSETS / "brief-page.png")
    desk = Image.new("RGB", (2360, 1120), "#ecebe7")
    d = ImageDraw.Draw(desk)
    d.rectangle((0, 0, 2360, 86), fill="#faf9f5")
    d.text((52, 22), "TERRA / Creative direction", font=font(29, True), fill="#33333a")
    d.text((2020, 29), "PROJECT BRIEF", font=font(19), fill="#827468")
    page = brief.resize((680, 880), Image.Resampling.LANCZOS)
    desk.paste(page, (275, 155))
    for index, title in enumerate(("Terra study.png", "Still study.png")):
        poster = Image.open(output / title).resize((480, 597), Image.Resampling.LANCZOS)
        desk.paste(poster, (1070 + index * 540, 210))
    d.text((1070, 878), "IDENTITY STUDIES", font=font(27, True), fill="#33333a")
    d.text((1070, 936), "Warm materials. Simple shapes. Clear type.", font=font(27), fill="#766a61")
    desk.save(ASSETS / "brief-window.png")
    for name, text in [
        ("Meeting notes.txt", "Terra studio / September review\n\nFinalize the identity.\nReview the website.\nPrepare the launch images.\n"),
        ("Launch checklist.md", "# Launch checklist\n\n- [x] Brand direction\n- [x] Image studies\n- [ ] Final review\n- [ ] Publish\n"),
        ("Project links.url", "[InternetShortcut]\nURL=https://example.com/\n"),
    ]:
        (output / name).write_text(text, encoding="utf-8")
    with zipfile.ZipFile(output / "Assets.zip", "w") as archive:
        archive.write(output / "Launch checklist.md", "Launch checklist.md")


def fence(title, rect, path=None, kind="folderPortal", icon=64, view="icons"):
    x, y, w, h = rect
    return {
        "id": str(uuid.uuid4()), "title": title, "kind": kind,
        "source": {"kind": "folder", "path": str(path), "recursive": False, "filter": None} if path else {"kind": "desktop"},
        # Capture runner resolves the actual monitor/DPI and physical rectangle.
        "geometry": {"monitor": "\\\\.\\DISPLAY1", "x": x, "y": y, "w": w, "h": h, "workW": 1920, "workH": 1032, "anchor": "leftTop"},
        "rolledUp": False, "expandedH": h,
        "view": {"iconSize": icon, "sort": "name", "labelLines": 2, "autoHeight": False, "reverse": False,
                 "layout": view, "spacing": "normal", "columnWidths": None, "columnsVisible": None},
        "appearance": {"tintRgb": None, "opacity": .48, "backdrop": "acrylic", "titleRgb": None, "titleSize": "large"},
        "excludeFromQuickHide": False, "locked": False, "tabHost": None, "activeTab": None, "tabOrder": [],
        "portalNavigate": True, "hideTitleIcon": False, "items": [],
    }


def prepare_scene(name, dark=False):
    case = BASE / name
    app = case / "app"
    fixture = case / "desktop-fixture"
    (app / "config").mkdir(parents=True, exist_ok=True)
    fixture.mkdir(exist_ok=True)
    source = ASSETS / "studies"
    for group in ("Projects", "Inspiration", "Today"):
        (case / group).mkdir(exist_ok=True)
    legacy_brand = case / "Projects/Brand system"
    if legacy_brand.exists() and not any(legacy_brand.iterdir()):
        legacy_brand.rmdir()
    legacy_zip = case / "Today/Assets.zip"
    if legacy_zip.exists():
        legacy_zip.unlink()
    for title in ("Brand", "Website", "Campaign", "Archive"):
        (case / "Projects" / title).mkdir(exist_ok=True)
    for title in ("Creative brief.pdf", "Launch checklist.md"):
        shutil.copy2(source / title, case / "Projects" / title)
    for p in source.glob("*.png"):
        shutil.copy2(p, case / "Inspiration" / p.name)
    for title in ("Creative brief.pdf", "Meeting notes.txt", "Launch checklist.md"):
        shutil.copy2(source / title, case / "Today" / title)
    reference = json.loads((PROMO / ".capture/reviewed/auto-v2/app/config/config.json").read_text(encoding="utf-8-sig"))
    settings = copy.deepcopy(reference["settings"])
    settings.update(language="en", theme="dark" if dark else "light", themeStyle="liquidGlass",
                    hideRealIcons=False, showRealIconsWhenFencesHidden=False, desktopPath=str(fixture), autostart=False)
    settings["peek"] = {"enabled": False, "dim": True, "hotkey": "ctrlAltSpace"}
    settings["snapping"]["sizeToCells"] = False
    config = {"schemaVersion": 1, "settings": settings, "items": {}, "snapshots": [], "undoLog": [],
              "rules": {"defaultTarget": "inbox", "keepUpdated": False, "list": []},
              "layouts": [{"fingerprint": reference["layouts"][0]["fingerprint"], "fences": []}]}
    if name in ("overview", "dark"):
        fences = [
            fence("Projects", (90, 390, 680, 590), case / "Projects", icon=64),
            fence("Inspiration", (840, 450, 880, 715), case / "Inspiration", icon=96),
            fence("Today", (1790, 605, 675, 355), case / "Today", icon=64),
        ]
    elif name == "tabs":
        fences = [fence("Projects", (730, 330, 1120, 850), case / "Projects", icon=96),
                  fence("Inspiration", (730, 330, 1120, 850), case / "Inspiration", icon=96)]
    elif name == "peek":
        fences = [fence("Projects", (360, 430, 830, 660), case / "Projects", icon=64),
                  fence("Inspiration", (1410, 330, 890, 755), case / "Inspiration", icon=96)]
    elif name == "auto":
        for title in ("Form study.png", "Launch checklist.md"):
            incoming = fixture / title
            if incoming.exists():
                incoming.unlink()
        for title in ("Terra study.png", "Still study.png", "Creative brief.pdf", "Meeting notes.txt"):
            shutil.copy2(source / title, fixture / title)
        fences = [fence("Images", (380, 490, 910, 430), kind="virtual", icon=96),
                  fence("Documents", (1420, 510, 800, 370), kind="virtual", icon=64)]
        for f, extensions in zip(fences, [["png"], ["pdf", "txt", "md"]]):
            config["rules"]["list"].append({"id": str(uuid.uuid4()), "name": f["title"], "enabled": True,
                "target": {"fence": f["id"]}, "allOf": [{"cond": "ext", "value": extensions}], "priorityClass": "type"})
        config["rules"]["keepUpdated"] = True
    else:
        fences = [fence("Inspiration", (770, 300, 1330, 880), case / "Inspiration", icon=96)]
    inbox = fence("Capture inbox", (-600, -200, 300, 100), kind="inbox", icon=32)
    inbox["rolledUp"] = True
    config["layouts"][0]["fences"] = fences + [inbox]
    (case / "template.json").write_text(json.dumps(config, indent=2), encoding="utf-8")
    commands = ["sleep 1000", "pin-test-windows", "sleep 400"]
    if name == "tabs":
        commands += ["merge Inspiration Projects", "sleep 3500", "activate Projects", "sleep 4500", "activate Inspiration", "sleep 7000"]
    elif name == "peek":
        commands = ["sleep 5800", "peek", "sleep 8500"]
    else:
        commands += ["sleep 11500"]
    commands += ["dump capture-end", "exit"]
    (case / "commands.txt").write_text("\n".join(commands), encoding="utf-8")


def main():
    ASSETS.mkdir(parents=True, exist_ok=True)
    BASE.mkdir(parents=True, exist_ok=True)
    wallpaper("paper-light")
    wallpaper("paper-dark", True)
    studies()
    for name in ("overview", "dark", "tabs", "peek", "auto", "portal"):
        prepare_scene(name, name in ("dark", "peek"))
    print(f"Prepared original artwork and six isolated fixtures: {BASE}")


if __name__ == "__main__":
    main()
