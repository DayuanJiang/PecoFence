"""Validate the recorded real-mouse audit (sky / SendInput, no scripted input messages)."""
import ast
import json
import re
from pathlib import Path

root = Path(".cache/real-mouse-audit")


def read(name):
    text = (root / name).read_text(encoding="utf-8")
    assert not re.search(r" ERROR |PANIC|GPU glass unavailable", text)
    assert not re.search(r">> input ", text), "Application-handler input is not a real-mouse audit"
    result = []
    for line in text.splitlines():
        match = re.search(r'\[([^]]+)\] live .*?rect=(\([^ ]+\)) title="([^"]+)"', line)
        if not match:
            continue
        tag, rect, title = match.groups()
        r = ast.literal_eval(rect)
        client = ast.literal_eval(re.search(r"client=(\([^)]*\))", line)[1])
        chrome = ast.literal_eval(re.search(r"chrome=(\([^)]*\))", line)[1])
        assert client == chrome == (r[2]-r[0], r[3]-r[1]), (tag, title)
        assert "client_offset=(0,0)" in line, (tag, title)
        order = [name for name, _ in ast.literal_eval(re.search(r"tab_fonts=(\[.*?\])", line)[1])]
        result.append(dict(tag=tag, title=title, rect=r, order=order, line=line))
    return text, result


first, states = read("mouse-first.log")
second, more = read("mouse-second.log")
a, b, wide = "标签甲", "标签乙", "宽标签测试文档"
assert any(s["title"] == a and s["order"] == [b, a] for s in states)
assert any(s["title"] == a and s["order"] == [a, b] for s in states)
for title in (a, b):
    assert any(s["title"] == title and s["order"] == [title] for s in states), "Tear-off"
assert any(s["title"] == b and s["order"] == [b, a] for s in states), "Drag merge"
narrow = [s for s in states if s["title"] == b and s["rect"][2]-s["rect"][0] <= 300]
assert any(s["order"] == [a, b] for s in narrow)
assert any(s["order"] == [b, a] for s in narrow)
assert any(s["title"] == b and s["rect"] == (500, 264, 1500, 858) for s in states)
assert "capture lost during tab tear-off" not in first, "Unexpected tear-off capture loss"
assert any(s["title"] == b and s["order"] == [wide, b] for s in more)
assert any(s["title"] == b and s["order"] == [b, wide] for s in more)
locked = [s for s in more if s["title"] == b and "locked=true" in s["line"]]
assert locked and {s["rect"] for s in locked} == {(200, 0, 1200, 72)}
assert any(s["title"] == b and "rolled=false rolling=false" in s["line"] for s in more)
assert any(s["title"] == b and "rolled=true rolling=false" in s["line"] for s in more)
assert "hide_setting=true icons_hidden=true" in second
assert "hide_setting=false icons_hidden=false" in second

motion = Path(".cache/real-mouse-motion")
driver = (motion / "driver.log").read_text(encoding="utf-8-sig")
assert driver.count("SendInput ") == 4, "Four actual pointer passes are required"
assert sum(map(int, re.findall(r"samples=(\d+)", driver))) >= 1000
cancels = (motion / "cancel.log").read_text(encoding="utf-8-sig")
for action in ("escape", "right"):
    line = next(l for l in cancels.splitlines() if f'Some("{action}")' in l)
    assert "source_before=(600, 400, 1600, 1200)" in line
    assert "source_after=Some((600, 400, 1600, 1200))" in line
    assert "capture_released=true" in line
measurement = json.loads((motion / "physical-pointer.json").read_text(encoding="utf-8"))
assert measurement["moving_frames"] >= 120
assert measurement["registration_error_max_px"] <= 3.0
report = {
    "result": "PASS",
    "native_window_snapshots": len(states) + len(more),
    "real_pointer_passes": 4,
    "moving_video_frames": measurement["moving_frames"],
    "max_registration_error_px": measurement["registration_error_max_px"],
    "cancellation_geometry_and_capture": "PASS",
    "desktop_icon_roundtrip": "PASS",
}
(root / "validation.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
print(json.dumps(report, indent=2))
