"""Export the product site's media from the local promo captures.

Requires Pillow and FFmpeg. Reads the ignored recordings under
extras/pecofence-promo/public and writes small web assets into site/assets,
which are checked in so the site builds without the promo project.
"""
import argparse
from pathlib import Path
import subprocess

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / "extras/pecofence-promo/public"
OUT = ROOT / "site/assets"
CLIPS = ["groups", "portal", "tabs", "peek", "hide", "rules"]
POSTER_SECONDS = {"groups": 6, "portal": 9, "tabs": 5, "peek": 6.8, "hide": 7, "rules": 10}
REVISION = ROOT / ".cache/store-v2"


def presentation_stills():
    """Use the same original desktop and native scenes as the Store revision."""
    wallpaper = Image.open(REVISION / "paper-light.png").convert("RGB")
    wallpaper.resize((1920, 1080), Image.Resampling.LANCZOS).save(
        OUT / "showcase-wallpaper.jpg", quality=88, optimize=True, progressive=True)
    native = Image.open(REVISION / "native/overview.png").convert("RGB")
    panels = {
        "work": (90, 390, 770, 980),
        "art": (840, 450, 1720, 1165),
        "folder": (1790, 605, 2465, 960),
    }
    for name, rect in panels.items():
        panel = native.crop(rect).convert("RGBA")
        mask = Image.new("L", panel.size)
        ImageDraw.Draw(mask).rounded_rectangle(
            (0, 0, panel.width-1, panel.height-1), radius=48, fill=255)
        panel.putalpha(mask)
        panel.thumbnail((880, 715), Image.Resampling.LANCZOS)
        panel.save(OUT / f"showcase-{name}.webp", quality=94, method=6)
    scenes = {
        "groups": ("overview", None),
        "peek": ("peek", None),
        "tabs": ("tabs-inspiration", (435, 280, 2147, 1243)),
        "rules": ("auto", (250, 145, 2298, 1297)),
        "portal": ("portal", (540, 215, 2476, 1304)),
    }
    for name, (scene, crop) in scenes.items():
        image = Image.open(REVISION / "native" / f"{scene}.png").convert("RGB")
        if crop:
            image = image.crop(crop)
        image.thumbnail((1290, 968), Image.Resampling.LANCZOS)
        image.save(OUT / f"{name}.jpg", quality=91, optimize=True, progressive=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stills-only", action="store_true",
                        help="update the revision-2 images without re-encoding unchanged videos")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    if args.stills_only:
        presentation_stills()
        print("Updated revision-2 hero panels, wallpaper and five native feature covers")
        return
    wallpaper = Image.open(PUBLIC / "wallpaper.png").convert("RGB")
    wallpaper.resize((1920, 1080), Image.Resampling.LANCZOS).save(
        OUT / "wallpaper.jpg", quality=82, optimize=True, progressive=True)
    for index in range(3):
        panel = Image.open(PUBLIC / f"panel-{index}.png").convert("RGBA")
        panel.resize((640, round(640 * panel.height / panel.width)), Image.Resampling.LANCZOS)
        panel.thumbnail((640, 640), Image.Resampling.LANCZOS)
        panel.save(OUT / f"panel-{index}.png", optimize=True)
    for clip in CLIPS:
        source = PUBLIC / f"features/{clip}.mp4"
        subprocess.run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(source),
            "-an", "-vf", "scale=1290:-2", "-c:v", "libx264", "-preset", "slow", "-crf", "24",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(OUT / f"{clip}.mp4"),
        ], check=True)
        subprocess.run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-ss", str(POSTER_SECONDS[clip]), "-i", str(source),
            "-frames:v", "1", "-vf", "scale=1290:-2", "-q:v", "4", str(OUT / f"{clip}.jpg"),
        ], check=True)
    # The 30-second English spot, kept with its soundtrack; plays only on request.
    promo = PUBLIC.parent / "out/PecoFence-autosort-en-1080p.mp4"
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(promo),
        "-vf", "scale=1280:-2", "-c:v", "libx264", "-preset", "slow", "-crf", "25",
        "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "96k", "-movflags", "+faststart",
        str(OUT / "promo.mp4"),
    ], check=True)
    poster = Image.open(PUBLIC.parent / "out/PecoFence-autosort-en-1080p-poster.png").convert("RGB")
    poster.resize((1280, 720), Image.Resampling.LANCZOS).save(
        OUT / "promo.jpg", quality=84, optimize=True, progressive=True)
    presentation_stills()
    for path in sorted(OUT.iterdir()):
        print(f"{path.name}: {path.stat().st_size / 1024:.0f} KiB")


if __name__ == "__main__":
    main()
