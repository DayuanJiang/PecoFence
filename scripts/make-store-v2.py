"""Render the second Store presentation from fresh native captures.

uv run --with pillow python scripts/make-store-v2.py
"""
import hashlib
import html
import json
import shutil
import zipfile
from functools import lru_cache
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / ".cache/store-v2"
NATIVE = SOURCE / "native"
OUT = ROOT / "dist/store-listing-v2"
FONTS = Path("C:/Windows/Fonts")
SIZE = (1920, 1080)
INK = "#233048"
LOCALE_ROOT = ROOT / "docs/store/v2-i18n"
LANGUAGES = [dict(entry, storeCode="en-US" if entry["code"] == "en" else entry["code"])
             for entry in json.loads((ROOT / "site/site.json").read_text(encoding="utf-8"))["languages"]]
EXPECTED_LOCALES = {entry["storeCode"] for entry in LANGUAGES}


def load_locales():
    available = {path.stem for path in LOCALE_ROOT.glob("*.json")}
    if available != EXPECTED_LOCALES:
        raise ValueError(f"Store locale coverage differs from site/site.json: missing={EXPECTED_LOCALES-available}, extra={available-EXPECTED_LOCALES}")
    interface = json.loads((ROOT / "docs/store/v2-ui.json").read_text(encoding="utf-8"))
    if set(interface) != EXPECTED_LOCALES:
        raise ValueError("Preview interface locale coverage is incomplete")
    catalog = {}
    for entry in LANGUAGES:
        code = entry["storeCode"]
        data = json.loads((LOCALE_ROOT / f"{code}.json").read_text(encoding="utf-8"))
        site = json.loads((ROOT / "site/i18n" / f"{entry['code']}.json").read_text(encoding="utf-8"))
        data["ui"] = dict(interface[code], language=site["nav.language"], features=site["features.heading"])
        for key in ("titles", "sub", "captions", "uploadCaptions"):
            if len(data[key]) != 5 or not all(isinstance(value, str) and value.strip() for value in data[key]):
                raise ValueError(f"{code}: {key} must contain five nonempty translations")
        if any(len(caption) > 200 for caption in data["captions"] + data["uploadCaptions"]):
            raise ValueError(f"{code}: screenshot caption exceeds 200 characters")
        if len(data["short"]) > 1000 or len(data["description"]) > 10000:
            raise ValueError(f"{code}: listing copy exceeds field limits")
        catalog[code] = data
    reference = catalog["en-US"]
    for code, data in catalog.items():
        for section in (None, "labels", "ui"):
            keys = set(data if section is None else data[section])
            required = set(reference if section is None else reference[section])
            if keys != required:
                raise ValueError(f"{code}: incomplete {section or 'locale'} keys: {required-keys}")
    return catalog


TEXT = load_locales()
NAMES = ["01-desktop", "02-peek", "03-tabs", "04-auto-sort", "05-appearance"]
UPLOAD_SHOTS = [
    ("01-desktop", "overview", None),
    ("02-peek", "peek", None),
    ("03-tabs", "tabs-inspiration", (435, 280, 2147, 1243)),
    ("04-auto-sort", "auto", (250, 145, 2298, 1297)),
    ("05-appearance", "dark", None),
]
UPLOAD_CAPTIONS = {code: data["uploadCaptions"] for code, data in TEXT.items()}

@lru_cache(maxsize=256)
def font(size, language="en-US", bold=False):
    name = {"zh-CN": "msyhbd.ttc" if bold else "msyh.ttc",
            "zh-TW": "msjhbd.ttc" if bold else "msjh.ttc",
            "ja": "YuGothB.ttc" if bold else "YuGothM.ttc",
            "ko": "malgunbd.ttf" if bold else "malgun.ttf"}.get(
                language, "seguisb.ttf" if bold else "segoeui.ttf")
    return ImageFont.truetype(str(FONTS / name), size)


def fit(draw, text, size, max_width, language, bold=False):
    while size >= 16:
        f = font(size, language, bold)
        if max(draw.textlength(line, font=f) for line in text.split("\n")) <= max_width:
            return f
        size -= 1
    raise ValueError(f"Copy does not fit: {text}")


def label(image, text, pos, size=22, color=INK, language="en-US", bold=False):
    draw = ImageDraw.Draw(image)
    if "\n" in text:
        draw.multiline_text(pos, text, font=font(size, language, bold), fill=color, spacing=9)
    else:
        draw.text(pos, text, font=font(size, language, bold), fill=color, anchor="lt")


def native(name, crop=None):
    image = Image.open(NATIVE / f"{name}.png").convert("RGB")
    if crop:
        assert crop[0] >= 0 and crop[1] >= 0 and crop[2] <= image.width and crop[3] <= image.height
        image = image.crop(crop)
    return image


def paste(image, source, xy, width, radius=0, shadow=False):
    height = round(source.height * width / source.width)
    source = source.convert("RGBA").resize((width, height), Image.Resampling.LANCZOS)
    if radius:
        mask = Image.new("L", source.size)
        ImageDraw.Draw(mask).rounded_rectangle((0, 0, width-1, height-1), radius, fill=255)
        source.putalpha(mask)
    if shadow:
        layer = Image.new("RGBA", image.size)
        alpha = source.getchannel("A").point(lambda a: round(a * .13))
        shade = Image.new("RGBA", source.size, (31, 39, 67, 0))
        shade.putalpha(alpha)
        layer.paste(shade, (xy[0]+4, xy[1]+18))
        layer = layer.filter(ImageFilter.GaussianBlur(22))
        image.paste(layer, (0, 0), layer)
    image.paste(source, xy, source)


def header(image, language, number, white=False):
    copy = TEXT[language]
    d = ImageDraw.Draw(image)
    foreground = "#f7f7ff" if white else INK
    muted = "#c3c8de" if white else "#596277"
    label(image, "PecoFence", (76, 32), 22, foreground, bold=True)
    label(image, copy["label"], (220, 35), 16, muted, language)
    title = copy["titles"][number]
    face = fit(d, title, 68, 1770, language, True)
    d.text((72, 93), title, font=face, fill=foreground, anchor="lt")
    if number != 1:
        label(image, copy["sub"][number], (77, 187), 23, muted, language)


def cover(language):
    image = native("overview").resize(SIZE, Image.Resampling.LANCZOS)
    header(image, language, 0)
    return image


def peek(language):
    image = native("peek").resize(SIZE, Image.Resampling.LANCZOS)
    # The original capture has clear wallpaper above the actual document window.
    d = ImageDraw.Draw(image)
    d.rectangle((0, 0, 1919, 145), fill="#161c31")
    label(image, "PecoFence / PEEK", (77, 27), 19, "#b4bfd9", bold=True)
    f = fit(d, TEXT[language]["titles"][1], 65, 1760, language, True)
    d.text((74, 68), TEXT[language]["titles"][1], font=f, fill="#ffffff", anchor="lt")
    d.rounded_rectangle((605, 989, 1315, 1055), radius=16, fill="#131b32", outline="#51627d", width=1)
    text = "Ctrl  +  Alt  +  Space"
    f = font(27, bold=True)
    d.text((960, 1022), text, font=f, fill="white", anchor="mm")
    return image


def tabs(language):
    image = Image.new("RGB", SIZE, "#f2eee9")
    header(image, language, 2)
    crop = (700, 300, 1880, 1220)
    paste(image, native("tabs-projects", crop), (72, 327), 805, radius=20, shadow=True)
    paste(image, native("tabs-inspiration", crop), (1043, 327), 805, radius=20, shadow=True)
    label(image, TEXT[language]["labels"]["projects"], (98, 278), 20, "#686279", language, True)
    label(image, TEXT[language]["labels"]["inspiration"], (1069, 278), 20, "#686279", language, True)
    label(image, "→", (912, 595), 68, "#66718d")
    return image


def autosort(language):
    image = Image.new("RGB", SIZE, "#e8f1ed")
    label(image, "PecoFence / " + TEXT[language]["labels"]["autosort"], (80, 50), 20, "#416961", language, True)
    d = ImageDraw.Draw(image)
    copy = TEXT[language]
    f = fit(d, copy["titles"][3], 80, 470, language, True)
    d.multiline_text((76, 205), copy["titles"][3], font=f, fill="#183e37", spacing=18)
    f = fit(d, copy["sub"][3], 24, 450, language)
    d.multiline_text((83, 475), copy["sub"][3], font=f, fill="#4b6b64", spacing=12)
    # Actual native groups after the watched PNG/Markdown arrivals.
    crop = (335, 440, 2270, 970)
    paste(image, native("auto", crop), (570, 318), 1290, radius=22, shadow=True)
    for y, left, right in [(700, ".PNG", "Images"), (786, ".PDF  .TXT  .MD", "Documents")]:
        d.line((84, y-22, 503, y-22), fill="#b9cec6", width=1)
        label(image, left, (86, y), 21, "#315f53", bold=True)
        label(image, "→  " + right, (88, y+33), 23, "#315f53")
    return image


def appearance(language):
    image = Image.new("RGB", SIZE, "#f3f0eb")
    d = ImageDraw.Draw(image)
    d.rectangle((960, 250, 1920, 1080), fill="#202237")
    d.rectangle((0, 250, 960, 1080), fill="#e6e2eb")
    header(image, language, 4)
    crop = (800, 410, 1760, 1220)
    paste(image, native("overview", crop), (78, 346), 810, radius=24, shadow=True)
    paste(image, native("dark", crop), (1032, 346), 810, radius=24, shadow=True)
    label(image, TEXT[language]["labels"]["light"], (85, 285), 20, "#64627d", language, True)
    label(image, TEXT[language]["labels"]["dark"], (1040, 285), 20, "#d1cee4", language, True)
    return image


def artwork():
    """Text-free original image studies grouped into glass-like frames, not app UI."""
    image = Image.open(SOURCE / "paper-light.png").convert("RGB").resize(SIZE)
    crop = (100, 315, 810, 980)
    studies = SOURCE / "studies"
    cards = [
        ("Still study.png", (520, 160), 340, -11),
        ("Terra study.png", (1300, 212), 330, 13),
    ]
    for name, xy, width, angle in cards:
        source = Image.open(studies / name).convert("RGB").crop(crop)
        tile = Image.new("RGBA", (width+32, width+32), "#f9f6ef")
        resized = ImageOps.fit(source, (width, width), method=Image.Resampling.LANCZOS)
        tile.paste(resized, (16, 16))
        tile = tile.rotate(angle, Image.Resampling.BICUBIC, expand=True)
        paste(image, tile, xy, tile.width, shadow=True)
    panel = Image.new("RGBA", (650, 475))
    d = ImageDraw.Draw(panel)
    d.rounded_rectangle((1, 1, 648, 473), 40, fill=(239, 235, 247, 230),
                        outline=(255, 255, 255, 255), width=3)
    for n, name in enumerate(("Form", "Sol", "Tide", "Atelier")):
        block = Image.open(studies / f"{name} study.png").convert("RGB").crop(crop)
        block = ImageOps.fit(block, (267, 184), Image.Resampling.LANCZOS)
        panel.paste(block, (43+(n % 2)*297, 40+(n // 2)*211))
    panel = panel.rotate(-7, Image.Resampling.BICUBIC, expand=True)
    paste(image, panel, (778, 136), panel.width, shadow=True)
    image.save(OUT / "art/super-hero-1920x1080.png", optimize=True)
    shutil.copy2(ROOT / "dist/store-listing/art/app-icon-300x300.png", OUT / "art/app-icon-300x300.png")


def proofs():
    state = json.loads((NATIVE / "auto-state.json").read_text(encoding="utf-8-sig"))
    membership = {}
    for fence in state["layouts"][0]["fences"]:
        for assignment in fence["items"]:
            item = state["items"][assignment["itemId"]]
            membership[item["displayName"]] = (fence["title"], assignment["assignedBy"])
    for name, target in [("Form study.png", "Images"), ("Launch checklist.md", "Documents")]:
        group, assigned = membership[name]
        assert group == target and isinstance(assigned, dict) and "rule" in assigned
    (OUT / "native").mkdir(exist_ok=True)
    for path in NATIVE.glob("*"):
        shutil.copy2(path, OUT / "native" / path.name)
    (OUT / "native/automatic-routing-proof.json").write_text(json.dumps(membership, indent=2), encoding="utf-8")


def copy_fields():
    listings = {}
    for language, copy in TEXT.items():
        listings[language] = {key: copy[key] for key in (
            "description", "features", "keywords", "trailerTitle", "captions")}
        listings[language]["shortDescription"] = copy["short"]
        folder = OUT / "listings" / language
        folder.mkdir(parents=True, exist_ok=True)
        for key, value in listings[language].items():
            text = "\n".join(value) if isinstance(value, list) else value
            (folder / f"{key}.txt").write_text(text + "\n", encoding="utf-8")
    (OUT / "listings/listings.json").write_text(json.dumps(listings, ensure_ascii=False, indent=2), encoding="utf-8")
    return listings


def upload_pack(listings):
    """Store screenshots contain only captured app pixels, without campaign overlays."""
    folder = OUT / "upload"
    (folder / "screenshots").mkdir(parents=True, exist_ok=True)
    evidence = []
    for name, source_name, crop in UPLOAD_SHOTS:
        source = NATIVE / f"{source_name}.png"
        image = native(source_name, crop)
        assert image.width * 9 == image.height * 16
        target = folder / "screenshots" / f"{name}.png"
        image.resize(SIZE, Image.Resampling.LANCZOS).save(target, optimize=True)
        evidence.append({
            "file": target.relative_to(folder).as_posix(),
            "source": source.relative_to(ROOT).as_posix(),
            "sourceSha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "outputSha256": hashlib.sha256(target.read_bytes()).hexdigest(),
            "crop": crop,
            "resize": list(SIZE),
            "overlays": False,
        })
    store_copy = {}
    for language, fields in listings.items():
        fields = dict(fields, captions=UPLOAD_CAPTIONS[language])
        destination = folder / "listings" / language
        destination.mkdir(parents=True, exist_ok=True)
        for key, value in fields.items():
            text = "\n".join(value) if isinstance(value, list) else value
            (destination / f"{key}.txt").write_text(text + "\n", encoding="utf-8")
        store_copy[language] = dict(TEXT[language], captions=fields["captions"],
                                    description=fields["description"], features=fields["features"])
    (folder / "listings/listings.json").write_text(
        json.dumps({language: dict(fields, captions=UPLOAD_CAPTIONS[language])
                    for language, fields in listings.items()}, ensure_ascii=False, indent=2),
        encoding="utf-8")
    (folder / "provenance.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
    (folder / "art").mkdir(exist_ok=True)
    for name in ("super-hero-1920x1080.png", "app-icon-300x300.png"):
        shutil.copy2(OUT / "art" / name, folder / "art" / name)
    shutil.copy2(ROOT / "docs/store/V2.md", OUT / "README.md")
    shutil.copy2(ROOT / "docs/store/V2.md", folder / "README.md")
    with zipfile.ZipFile(OUT / "PecoFence-store-v2.zip", "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(folder.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(folder).as_posix())
    return store_copy


def contact_sheet():
    sheet = Image.new("RGB", (1600, 1450), "#f5f5f7")
    d = ImageDraw.Draw(sheet)
    label(sheet, "PecoFence / 第二版", (38, 25), 34, INK, "zh-CN", True)
    label(sheet, "重新拍摄的原生界面 · 一图一个主题", (40, 77), 20, "#667184", "zh-CN")
    for index, name in enumerate(NAMES):
        col, row = index % 2, index // 2
        x, y = 36 + col*782, 130 + row*435
        with Image.open(OUT / f"screenshots/zh-CN/{name}.png") as image:
            sheet.paste(image.resize((746, 420), Image.Resampling.LANCZOS), (x, y))
    d.rounded_rectangle((818, 1000, 1564, 1418), radius=16, fill="#e9edf4")
    label(sheet, "原生 UI，没有重绘。", (860, 1050), 32, INK, "zh-CN", True)
    label(sheet, f"新的桌面、新的演示文件、\n新的构图和信息层级。\n\n{len(TEXT)} 种语言 · {len(TEXT)*len(NAMES)} 张宣传图\n1920 × 1080", (860, 1130), 25, "#566177", "zh-CN")
    sheet.save(OUT / "contact-sheet.jpg", quality=94)
    overview = Image.new("RGB", (1600, 100 + ((len(LANGUAGES)+1)//2)*485), "#f5f5f7")
    label(overview, f"PecoFence / {len(TEXT)} 种语言", (38, 25), 34, INK, "zh-CN", True)
    for index, entry in enumerate(LANGUAGES):
        x, y = 36 + (index % 2)*782, 104 + (index // 2)*485
        label(overview, entry["name"], (x, y), 26, INK, entry["storeCode"], True)
        with Image.open(OUT / f"screenshots/{entry['storeCode']}/01-desktop.png") as image:
            overview.paste(image.resize((746, 420), Image.Resampling.LANCZOS), (x, y+42))
    overview.save(OUT / "languages-contact-sheet.jpg", quality=94)


def preview(store_copy):
    data = json.dumps(TEXT, ensure_ascii=False)
    names = json.dumps(NAMES)
    page = (ROOT / "docs/store/v2-preview.html").read_text(encoding="utf-8")
    page = page.replace("{{DATA}}", data).replace("{{NAMES}}", names)
    page = page.replace("{{STORE_DATA}}", json.dumps(store_copy, ensure_ascii=False))
    page = page.replace("{{UI_DATA}}", json.dumps({code: data["ui"] for code, data in TEXT.items()}, ensure_ascii=False))
    options = "".join(
        f'<option value="{entry["storeCode"]}" lang="{entry["code"]}">{html.escape(entry["name"])}</option>'
        for entry in LANGUAGES)
    page = page.replace("{{LANGUAGE_OPTIONS}}", options)
    (OUT / "index.html").write_text(page, encoding="utf-8")
    (OUT / "gallery.html").write_text(
        page.replace('data-mode="store" data-assets="upload"', 'data-mode="gallery" data-assets="campaign"'),
        encoding="utf-8")


def main():
    for folder in ("art", "screenshots"):
        (OUT / folder).mkdir(parents=True, exist_ok=True)
    functions = [cover, peek, tabs, autosort, appearance]
    for language in TEXT:
        folder = OUT / "screenshots" / language
        folder.mkdir(exist_ok=True)
        for name, render in zip(NAMES, functions):
            image = render(language)
            assert image.size == SIZE
            image.save(folder / f"{name}.png", optimize=True)
    artwork()
    proofs()
    listings = copy_fields()
    store_copy = upload_pack(listings)
    contact_sheet()
    preview(store_copy)
    print(f"Rendered {len(TEXT)*len(NAMES)} campaign images in {len(TEXT)} languages, "
          f"{len(UPLOAD_SHOTS)} native upload screenshots, upload ZIP and localized preview: {OUT}")


if __name__ == "__main__":
    main()
