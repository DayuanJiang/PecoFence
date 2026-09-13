"""Measure wallpaper registration from lossless 700x64 probe recordings.

The magenta square is part of the displayed fence, so it measures its presented
position rather than GetWindowRect. The grey ramp encodes wallpaper X. Calibration
uses settled frames from the same recording, including display colour conversion.
"""
import csv
import argparse
import json
import statistics
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("video", type=Path)
parser.add_argument("--min-motion-frames", type=int, default=120)
parser.add_argument("--max-error-px", type=float, default=3.0)
args = parser.parse_args()
video = args.video
width, height, origin_x = 700, 64, 900
decoder = subprocess.Popen(
    ["ffmpeg", "-v", "error", "-i", str(video), "-f", "rawvideo", "-pix_fmt", "rgb24", "-"],
    stdout=subprocess.PIPE,
)
samples = []
frame_index = 0
while True:
    data = decoder.stdout.read(width * height * 3)
    if not data:
        break
    if len(data) != width * height * 3:
        raise RuntimeError("Incomplete decoded frame")
    points = []
    at = 0
    while True:
        at = data.find(b"\xff\x00\xff", at)
        if at < 0:
            break
        if at % 3 == 0:
            pixel = at // 3
            points.append((pixel % width, pixel // width))
        at += 3
    if len(points) >= 16:
        xs, ys = zip(*points)
        if max(xs) - min(xs) <= 10 and max(ys) - min(ys) <= 10:
            x = (min(xs) + max(xs) + 1) * 0.5
            y = round((min(ys) + max(ys) + 1) * 0.5 - 16)
            values = [
                data[((y + dy) * width + round(x) + dx) * 3 + 1]
                for dy in range(-1, 2) for dx in range(-1, 2)
            ]
            samples.append({"frame": frame_index, "x": origin_x + x, "grey": statistics.mean(values)})
    frame_index += 1
decoder.wait()
assert decoder.returncode == 0 and len(samples) > 60, "Probe was not visible"

runs = []
for sample in samples:
    if not runs or runs[-1][0]["x"] != sample["x"]:
        runs.append([])
    runs[-1].append(sample)
settled = {}
for run in runs:
    if len(run) >= 40:
        settled.setdefault(run[0]["x"], []).extend(s["grey"] for s in run[-30:])
calibration = sorted((x, statistics.median(g)) for x, g in settled.items())
assert len(calibration) >= 2, "Need settled frames at two distinct positions"
x0, g0 = calibration[0]
x1, g1 = calibration[-1]
assert x1 - x0 >= 100 and abs(g1 - g0) > 20, "Insufficient motion for calibration"
slope = (g1 - g0) / (x1 - x0)
intercept = g0 - slope * x0

motion_frames = set()
for previous, current in zip(samples, samples[1:]):
    if current["x"] != previous["x"]:
        motion_frames.update(range(current["frame"], current["frame"] + 3))
errors = []
for sample in samples:
    sample["error_px"] = (sample["grey"] - intercept) / slope - sample["x"]
    if sample["frame"] in motion_frames:
        errors.append(abs(sample["error_px"]))
assert len(errors) >= args.min_motion_frames, "Too few moving frames to validate continuous motion"
errors.sort()
report = {
    "recorded_frames": frame_index,
    "probe_frames": len(samples),
    "moving_frames": len(errors),
    "settled_positions_px": [x for x, _ in calibration],
    "calibrated_grey_per_pixel": slope,
    "registration_error_median_px": statistics.median(errors),
    "registration_error_p95_px": errors[int((len(errors)-1)*0.95)],
    "registration_error_max_px": max(errors),
}
video.with_suffix(".json").write_text(json.dumps(report, indent=2), encoding="utf-8")
with video.with_suffix(".csv").open("w", newline="", encoding="utf-8") as out:
    writer = csv.DictWriter(out, fieldnames=["frame", "x", "grey", "error_px"])
    writer.writeheader()
    writer.writerows(samples)
print(json.dumps(report, indent=2))
assert report["registration_error_max_px"] <= args.max_error_px, "Wallpaper and window presented out of step"
