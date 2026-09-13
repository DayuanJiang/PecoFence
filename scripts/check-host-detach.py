"""Check saved configurations and event logs from the native host-detach audit."""
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])


def load(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))


def fences(name):
    return {f["id"]: f for f in load(name)["layouts"][0]["fences"]}


initial = load("initial.json")["layouts"][0]["fences"]
a, b, c = [f["id"] for f in initial]
base = {f["id"]: f for f in initial}


def check_content(current, original):
    assert current.keys() == original.keys(), "Fence IDs were lost/duplicated"
    for identity, before in original.items():
        for field in ("title", "kind", "source", "appearance"):
            assert current[identity][field] == before[field], (identity, field)
        # Startup/F5 discovers new desktop files. Every pre-existing assignment must
        # survive unchanged; newly discovered items are allowed and must be unique.
        refs = {ref["itemId"]: ref for ref in current[identity]["items"]}
        assert len(refs) == len(current[identity]["items"]), "Duplicate item assignments"
        for ref in before["items"]:
            assert refs.get(ref["itemId"]) == ref, (identity, "lost/changed item", ref["itemId"])


first = fences("first-host-detached.json")
check_content(first, base)
assert first[a]["tabHost"] is None and first[b]["tabHost"] is None
assert first[b]["geometry"] == base[a]["geometry"], "Remaining fence moved"
assert first[a]["geometry"] != base[a]["geometry"], "Dragged host did not move"

second = fences("second-tab-detached.json")
check_content(second, base)
assert second[a]["tabHost"] is None and second[b]["tabHost"] is None
assert second[a]["geometry"] == first[a]["geometry"], "Host moved when child left"

reordered = fences("reordered-detached.json")
original = fences("reordered-initial.json")
check_content(reordered, original)
assert reordered[a]["tabHost"] is None and reordered[b]["tabHost"] is None
assert reordered[c]["tabHost"] == b
assert reordered[b]["tabOrder"] == [b, c]
assert reordered[b]["geometry"] == original[a]["geometry"]

child = fences("first-child-detached.json")
original = fences("first-child-initial.json")
check_content(child, original)
assert child[b]["tabHost"] is None and child[a]["tabHost"] is None
assert child[c]["tabHost"] == a and child[a]["tabOrder"] == [a, c]
assert child[a]["geometry"] == original[a]["geometry"]

for case, event in [("cancel-escape", "escape"), ("cancel-right", "right")]:
    assert load(case + "-before.json")["layouts"] == load(case + "-after.json")["layouts"]
    text = (root / (case + ".log")).read_text(encoding="utf-8")
    marker = "test cancellation posted after detach cancel=" + event
    assert marker in text, "Cancellation was not delivered during the drag"
    tail = text[text.index(marker):]
    assert "fences=1 dying=0" in tail, "Cancelled windows did not retire"
    assert re.search(r"live .*tabs=3.*client=\(800, 440\)", tail)
    assert not re.search(r" ERROR |PANIC", text)

for name in ("first-host-detached", "second-tab-detached", "reordered", "first-child"):
    text = (root / (name + ".log")).read_text(encoding="utf-8")
    assert "native pointer event" in text
    assert "tab torn off; waiting for the new window" in text
    assert "detach_tab: window created from_drag=true" in text
    assert not re.search(r" ERROR |PANIC", text)

auto = fences("auto-host-after.json")
own_content_log = (root / "auto-a-active.log").read_text(encoding="utf-8")
own_height = re.search(
    r'\[before\] live .*title="文件与文档".*client=\(\d+, (\d+)\)', own_content_log
)
assert own_height
assert auto[a]["tabHost"] is None and auto[c]["tabHost"] == b
assert auto[a]["view"]["autoHeight"]
assert round(auto[a]["geometry"]["h"] * 2) == int(own_height[1])
assert auto[a]["geometry"]["y"] + auto[a]["geometry"]["h"] <= auto[a]["geometry"]["workH"] + 0.5

print(json.dumps({
    "host_in_first_position": "passed: native mouse drag",
    "child_in_second_position": "passed: native mouse drag",
    "host_in_middle_of_three": "passed: native mouse drag",
    "child_in_first_position": "passed: native mouse drag",
    "remaining_group_geometry_and_content": "preserved",
    "escape_and_right_cancel": "passed: messages injected during native mouse drag",
    "cancelled_layouts": "identical to before drag",
    "automatic_height": "fits the detached content and remains inside the work area",
}, indent=2))
