"""Export the product site's media from the local promo captures.

Requires Pillow and FFmpeg. Reads the ignored recordings under
extras/pecofence-promo/public and writes small web assets into site/assets,
which are checked in so the site builds without the promo project.
"""
import argparse
from pathlib import Path
import subprocess

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / "extras/pecofence-promo/public"
OUT = ROOT / "site/assets"
CLIPS = ["groups", "portal", "tabs", "peek", "hide", "rules"]
POSTER_SECONDS = {"groups": 6, "portal": 9, "tabs": 5, "peek": 6.8, "hide": 7, "rules": 10}
REVISION = ROOT / ".cache/store-v2"
# Features without a promo recording use their manual lesson instead: the Store trailer
# renderer's caption-free English render (.cache/store-trailers/render.py en <lesson>
# --no-captions --out .cache/store-trailers/out/en/<lesson>-bare.mp4, which also writes the
# poster frame as <lesson>-bare.png), cropped to the 1600x900 stage.
LESSONS = {"search": "search", "dialog": "dialog-jump"}
LESSON_RENDERS = ROOT / ".cache/store-trailers/out/en"
STAGE = (160, 22, 1760, 922)


def presentation_stills():
    """Use the same original desktop and native scenes as the Store revision."""
    # The hero's backdrop; the trailer (promo) is the hero's visual.
    wallpaper = Image.open(REVISION / "paper-light.png").convert("RGB")
    wallpaper.resize((1920, 1080), Image.Resampling.LANCZOS).save(
        OUT / "showcase-wallpaper.jpg", quality=88, optimize=True, progressive=True)
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


def lesson_clips():
    left, top, right, bottom = STAGE
    for name, lesson in LESSONS.items():
        source = LESSON_RENDERS / f"{lesson}-bare"
        subprocess.run([
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(source.with_suffix(".mp4")),
            "-an", "-vf", f"crop={right - left}:{bottom - top}:{left}:{top},fps=30,scale=1290:-2",
            "-c:v", "libx264", "-preset", "slow", "-crf", "24",
            "-pix_fmt", "yuv420p", "-movflags", "+faststart", str(OUT / f"{name}.mp4"),
        ], check=True)
        poster = Image.open(source.with_suffix(".png")).convert("RGB").crop(STAGE)
        poster.resize((1290, 726), Image.Resampling.LANCZOS).save(
            OUT / f"{name}.jpg", quality=91, optimize=True, progressive=True)


def promo():
    """The 30-second trailer (v3) with its soundtrack; plays only on request."""
    master = PUBLIC.parent / "out/PecoFence-trailer-v3-1080p60.mp4"
    subprocess.run([
        "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(master),
        "-vf", "fps=30,scale=1280:720:flags=lanczos", "-c:v", "libx264", "-preset", "slow", "-crf", "23",
        "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart",
        str(OUT / "promo.mp4"),
    ], check=True)
    # Poster: the "Your desktop. Configured by your AI." frame, also the Store thumbnail.
    poster = Image.open(PUBLIC.parent / "out/PecoFence-trailer-v3-thumbnail.png").convert("RGB")
    poster.resize((1280, 720), Image.Resampling.LANCZOS).save(
        OUT / "promo.jpg", quality=84, optimize=True, progressive=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stills-only", action="store_true",
                        help="update the revision-2 images without re-encoding unchanged videos")
    parser.add_argument("--promo-only", action="store_true",
                        help="re-export only the trailer and its poster")
    parser.add_argument("--lessons-only", action="store_true",
                        help="re-export only the features shown by their manual lesson (LESSONS)")
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    if args.promo_only:
        promo()
        print("Updated promo.mp4 and promo.jpg")
        return
    if args.lessons_only:
        lesson_clips()
        print("Updated " + ", ".join(f"{name}.mp4 and {name}.jpg" for name in LESSONS))
        return
    if args.stills_only:
        presentation_stills()
        print("Updated the revision-2 hero wallpaper and five native feature covers")
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
    lesson_clips()
    promo()
    presentation_stills()
    for path in sorted(OUT.iterdir()):
        print(f"{path.name}: {path.stat().st_size / 1024:.0f} KiB")


if __name__ == "__main__":
    main()
