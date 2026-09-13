"""Export the product site's media from the local promo captures.

Requires Pillow and FFmpeg. Reads the ignored recordings under
extras/pecofence-promo/public and writes small web assets into site/assets,
which are checked in so the site builds without the promo project.
"""
from pathlib import Path
import subprocess

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PUBLIC = ROOT / "extras/pecofence-promo/public"
OUT = ROOT / "site/assets"
CLIPS = ["groups", "portal", "tabs", "peek", "hide", "rules"]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
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
            "ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-ss", "0.5", "-i", str(source),
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
    for path in sorted(OUT.iterdir()):
        print(f"{path.name}: {path.stat().st_size / 1024:.0f} KiB")


if __name__ == "__main__":
    main()
