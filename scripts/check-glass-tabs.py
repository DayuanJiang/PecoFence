"""Validate the portable glass-tab-audit scenario log (no desktop interaction)."""
import ast
import json
import re
import sys
from pathlib import Path

text = Path(sys.argv[1]).read_text(encoding="utf-8")
assert not re.search(r" ERROR |PANIC|GPU glass unavailable|surface lost its device", text)
states = {}
for line in text.splitlines():
    match = re.search(r'\[([^]]+)\] live .*?rect=(\([^ ]+\)) title="([^"]+)"', line)
    if not match:
        continue
    tag, rect, title = match.groups()
    state = {"rect": ast.literal_eval(rect)}
    for key in ("title_size", "frames", "map_builds", "control_frames", "control_map_builds", "wallpaper_uploads"):
        pattern = rf"\b{key}(?:=|: )(\d+)"
        state[key] = int(re.search(pattern, line).group(1))
    for key in ("client", "client_offset", "chrome"):
        state[key] = ast.literal_eval(re.search(rf"\b{key}=(\([^)]*\))", line).group(1))
    state["fonts"] = dict(ast.literal_eval(re.search(r"tab_fonts=(\[.*?\])", line).group(1)))
    state["tabs"] = ast.literal_eval(re.search(r"tab_rects=(\[.*?\])", line).group(1))
    left, top, right, bottom = state["rect"]
    assert state["client"] == state["chrome"] == (right-left, bottom-top), (tag, title, state)
    assert state["client_offset"] == (0, 0), (tag, title, state)
    states[(tag, title)] = state

doc, picture = "文件与文档", "图片"
get = lambda tag, title=doc: states[(tag, title)]
assert get("separate")["title_size"] == get("separate", picture)["title_size"] == 1
assert get("merged")["fonts"] == {doc: 1, picture: 1}
assert get("picture-large")["fonts"] == {doc: 1, picture: 2}
assert get("host-small")["fonts"] == {doc: 0, picture: 2}
assert get("detached-large")["title_size"] == 0
assert get("detached-large", picture)["title_size"] == 2
assert get("remerged-large")["fonts"] == {doc: 0, picture: 2}
for tag in ("merged", "selected-picture", "picture-large", "host-small", "detached-large", "remerged-large"):
    assert get(tag)["rect"] == get("separate")["rect"], (tag, "host resized")
assert get("detached-large", picture)["client"] == get("separate", picture)["client"]

narrow = get("narrow-collapsed")
assert narrow["client"] == (272, 72)
assert narrow["fonts"] == {doc: 1, picture: 1}
assert all(x+w <= 136-44+0.001 for x, w in narrow["tabs"]), "tab covered chevron"

start, end = get("move-start"), get("move-end")
for key in ("map_builds", "control_map_builds", "wallpaper_uploads"):
    assert start[key] == end[key], (key, "rebuilt on move")
assert end["frames"] - start["frames"] >= 40
timing_section = text[text.index("[move-start]"):text.index("[move-end]")]
times = sorted(int(n)/1000 for n in re.findall(r"GPU glass move preparation us=(\d+)", timing_section))
assert len(times) >= 40
assert get("rapid-switch")["fonts"] == {doc: 1, picture: 1}
assert get("ready")["client"] == (800, 400)
print(json.dumps({
    "states_checked": len(states),
    "standalone_and_tab_fonts_match": True,
    "per_tab_title_size_survives_detach": True,
    "merge_detach_preserve_geometry": True,
    "narrow_tabs_stay_clear_of_chevron": True,
    "moving_frames": len(times),
    "moving_did_not_rebuild_material": True,
    "move_prepare_median_ms": times[len(times)//2],
    "move_prepare_p95_ms": times[int(len(times)*.95)],
}, indent=2))
