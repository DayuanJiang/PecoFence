"""Prepare and validate a portable native interaction audit.

prepare <seed-config.json> <audit-dir>
check <pecofence.INSTANCE.log>
The seed supplies monitor identities/settings only; file operations target fixtures.
Run the debug exe with PECOFENCE_UI_TEST_WINDOWS=1 --portable --no-hide-icons
--test-script <audit-dir>/workflow.txt. This script does not inject desktop input.
"""
import ast
import copy
import json
import re
import sys
from pathlib import Path


def prepare(seed, root):
    root.mkdir(parents=True, exist_ok=True)
    (root / "config").mkdir(exist_ok=True)
    (root / "files").mkdir(exist_ok=True)
    (root / "empty").mkdir(exist_ok=True)
    cfg = json.loads(seed.read_text(encoding="utf-8-sig"))
    cfg.update(items={}, snapshots=[], undoLog=[])
    cfg["rules"]["list"] = []
    cfg["rules"]["keepUpdated"] = False
    settings = cfg["settings"]
    settings.update(theme="dark", themeStyle="liquidGlass", hideRealIcons=False)
    settings["snapping"]["enabled"] = False
    settings["quickHide"]["enabled"] = False
    settings["peek"]["enabled"] = False
    settings["rollUp"].update(hoverPeek=False, clickToExpand=False)
    layout = cfg["layouts"][0]
    template = copy.deepcopy(layout["fences"][0])
    titles = ["标签甲", "标签乙", "标签丙", "桌面"]
    ids = [f"11111111-1111-4111-8111-{i:012d}" for i in range(1, 5)]
    fences = []
    for i, title in enumerate(titles):
        f = copy.deepcopy(template)
        f.update(id=ids[i], title=title, kind="folderPortal",
                 source={"kind": "folder", "path": str(root / "files")},
                 items=[], tabHost=None, activeTab=None, tabOrder=[],
                 appearance=None, rolledUp=False, locked=False, expandedH=300,
                 excludeFromQuickHide=False)
        f["geometry"].update(x=300+i*420, y=200, w=400, h=300, anchor="leftTop")
        f["view"].update(layout="details", autoHeight=False)
        fences.append(f)
    fences[-1].update(kind="inbox", source={"kind": "desktop"}, rolledUp=True)
    fences[-1]["geometry"].update(x=1600, y=900, w=200)
    layout["fences"] = fences
    cfg["layouts"] = [layout]
    (root / "config" / "config.json").write_text(
        json.dumps(cfg, ensure_ascii=False, indent=2), encoding="utf-8")
    for i in range(12):
        (root / "files" / f"Test-{i:02d}.txt").write_text("Interaction fixture\n", encoding="utf-8")
    lines = ["sleep 500", "pace 8", "pin-test-windows raw",
             "bounds 标签甲 600 400 800 600", "merge 标签乙 标签甲"]

    def snap(tag, delay=400):
        lines.extend([f"sleep {delay}", f"dump {tag}"])

    def inp(action, x=0, y=0, title="标签甲"):
        lines.append(f"input {title} {action} {x} {y}")

    def drag(start, end, tag, finish="up", title="标签甲"):
        inp("down", start, 36, title)
        inp("move", end, 36, title)
        inp(finish, end, 36, title)
        snap(tag)

    def prop(index, key, value):
        lines.append("message " + json.dumps(
            {"type": "setFence", "id": ids[index], "prop": key, "value": value}, ensure_ascii=False))

    def patch_settings():
        lines.append("message " + json.dumps({"type": "patchSettings", "settings": settings}))

    snap("initial")
    drag(80, 320, "two-right")
    drag(220, 40, "two-left")
    for cancel in ("escape", "right"):
        inp("down", 80, 36)
        inp("move", 80, 180)
        inp(cancel)
        snap(f"tear-cancel-{cancel}")
    # Release before the queued detach has created a window.
    inp("down", 80, 36)
    inp("move", 80, 180)
    inp("up", 80, 180)
    snap("tear-fast-release", 650)
    lines.extend(["merge 标签乙 标签甲", "sleep 400", "bounds 标签甲 600 400 800 600"])
    snap("tear-remerged")
    drag(80, 82, "click-threshold")
    drag(80, 320, "escape-order", "escape")
    drag(80, 320, "right-order", "right")
    drag(80, 320, "capture-order", "capture-lost")
    lines.append("reorder 标签甲 0")
    snap("menu-left")
    lines.append("reorder 标签甲 999")
    snap("menu-right-boundary")
    lines.append("reorder 标签甲 0")
    prop(0, "title", "很长的测试标签甲")
    snap("unequal-ready")
    drag(90, 600, "wide-right", title="测试标签甲")
    drag(220, 20, "wide-left", title="测试标签甲")
    prop(0, "title", "标签甲")
    lines.extend(["bounds 标签甲 600 400 272 400"])
    snap("narrow-ready")
    drag(60, 200, "narrow-right")
    drag(170, 20, "narrow-left")
    lines.append("bounds 标签甲 600 400 800 600")
    lines.append("merge 标签丙 标签甲")
    snap("three-ready")
    drag(80, 500, "three-right")
    drag(360, 20, "three-left")
    lines.append("detach 标签甲")
    snap("host-detached", 650)
    lines.append("merge 标签甲 标签乙")
    snap("host-remerged", 650)
    lines.append("detach 标签丙")
    snap("member-detached", 650)
    lines.append("bounds 标签丙 1800 400 600 400")
    inp("caption-down", 400, 36, "标签丙")
    inp("pointer", 800, 436, "标签丙")
    inp("pointer-up", 800, 436, "标签丙")
    snap("member-remerged", 650)
    lines.extend(["merge 标签丙 标签乙", "merge 标签乙 标签乙"])
    snap("repeat-self-merge")
    for _ in range(6):
        lines.extend(["activate 标签甲", "sleep 16", "activate 标签乙", "sleep 16"])
    snap("rapid-switch")
    lines.extend(["bounds 标签乙 600 400 800 600", "roll 标签乙"])
    snap("rolled", 650)
    drag(80, 500, "rolled-reorder", title="标签乙")
    lines.append("unroll 标签乙")
    snap("unrolled", 650)
    lines.append("bounds 标签乙 600 400 800 600")
    snap("move-start")
    inp("caption-down", 650, 36, "标签乙")
    # Hundreds of samples before one frame: the last point wins.
    for x in range(1251, 1451):
        inp("pointer", x, 436, "标签乙")
    snap("move-burst", 100)
    inp("pointer-up", 1470, 466, "标签乙")
    snap("move-release")
    inp("caption-down", 650, 36, "标签乙")
    inp("pointer", 1650, 560, "标签乙")
    snap("move-cancel-pending", 100)
    inp("escape", title="标签乙")
    snap("move-cancelled")
    inp("caption-down", 650, 36, "标签乙")
    inp("pointer", 1600, 530, "标签乙")
    snap("move-right-pending", 100)
    inp("right", title="标签乙")
    snap("move-right-cancelled")
    inp("caption-down", 650, 36, "标签乙")
    inp("pointer", 1550, 466, "标签乙")
    snap("capture-pending", 100)
    inp("capture-lost", title="标签乙")
    snap("capture-released")
    prop(1, "locked", True)
    snap("locked-ready")
    inp("caption-down", 650, 36, "标签乙")
    inp("pointer", 1800, 600, "标签乙")
    inp("pointer-up", 1800, 600, "标签乙")
    snap("locked-move")
    prop(1, "locked", False)
    lines.extend(["quick-hide", "sleep 50", "quick-show"])
    snap("hidden-restored", 650)
    lines.append("bounds 标签乙 600 400 800 600")
    settings["themeStyle"] = "fluent"
    patch_settings()
    snap("theme-fluent", 650)
    settings["themeStyle"] = "liquidGlass"
    patch_settings()
    snap("theme-liquid", 650)
    settings["rollUp"]["clickToExpand"] = True
    patch_settings()
    lines.extend(["sleep 200", "roll 标签乙"])
    snap("click-rolled", 650)
    inp("caption-down", 650, 36, "标签乙")
    inp("pointer-up", 1250, 436, "标签乙")
    snap("click-expanded", 650)
    for i in range(2):
        for action, label in [("repairIcons", "shown"), ("hideDesktopIcons", "hidden"),
                              ("hideDesktopIcons", "hidden-again"), ("repairIcons", "restored")]:
            lines.append("message " + json.dumps({"type": "action", "name": action}))
            snap(f"icons-{i}-{label}", 200)
    snap("final", 1000)
    lines.append("exit")
    (root / "workflow.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(json.dumps({"root": str(root), "commands": len(lines)}, indent=2))


def check(log):
    text = log.read_text(encoding="utf-8")
    assert not re.search(r" ERROR |PANIC|GPU glass unavailable|surface lost its device", text)
    states, totals = {}, {}
    for line in text.splitlines():
        m = re.search(r'\[([^]]+)\] live .*?rect=(\([^ ]+\)) title="([^"]+)"', line)
        if m:
            tag, rect, title = m.groups()
            s = {"rect": ast.literal_eval(rect)}
            s["order"] = [t[0] for t in ast.literal_eval(re.search(r"tab_fonts=(\[.*?\])", line)[1])]
            for key in ("client", "chrome", "client_offset"):
                s[key] = ast.literal_eval(re.search(rf"\b{key}=(\([^)]*\))", line)[1])
            for key in ("capture", "tab_drag", "window_drag", "detach_pending", "rolled"):
                s[key] = re.search(rf"\b{key}=(true|false)", line)[1] == "true"
            for key in ("map_builds", "control_map_builds", "wallpaper_uploads"):
                s[key] = int(re.search(rf"\b{key}(?:=|: )(\d+)", line)[1])
            w, h = s["rect"][2]-s["rect"][0], s["rect"][3]-s["rect"][1]
            assert s["client"] == s["chrome"] == (w, h), (tag, title, "frame mismatch")
            assert s["client_offset"] == (0, 0), (tag, title, "nonclient inset")
            states[tag, title] = s
        m = re.search(r"\[([^]]+)\] fences=(\d+) dying=(\d+).*hide_setting=(true|false) icons_hidden=(true|false)", line)
        if m:
            tag, n, dying, setting, hidden = m.groups()
            totals[tag] = (int(n), int(dying), setting == "true", hidden == "true")
    get = lambda tag, title="标签甲": states[tag, title]
    a, b, c = "标签甲", "标签乙", "标签丙"
    for tag in ("initial", "two-left", "tear-cancel-escape", "tear-cancel-right", "tear-remerged",
                "click-threshold", "escape-order", "right-order", "menu-left", "narrow-left"):
        assert get(tag)["order"] == [a, b], (tag, get(tag)["order"])
    for tag in ("two-right", "capture-order", "menu-right-boundary", "narrow-right"):
        assert get(tag)["order"] == [b, a], (tag, get(tag)["order"])
    assert get("wide-right", "很长的测试标签甲")["order"] == [b, "很长的测试标签甲"]
    assert get("wide-left", "很长的测试标签甲")["order"] == ["很长的测试标签甲", b]
    assert get("three-right")["order"] == [b, c, a]
    assert get("three-left")["order"] == [a, b, c]
    assert totals["tear-fast-release"][:2] == (4, 0)
    assert get("tear-fast-release")["order"] == [a]
    assert get("tear-fast-release", b)["order"] == [b]
    assert totals["host-detached"][:2] == (3, 0)
    assert get("host-detached", b)["order"] == [b, c]
    assert get("host-detached")["order"] == [a]
    for tag in ("host-remerged", "member-remerged", "repeat-self-merge", "rapid-switch"):
        assert set(get(tag, b)["order"]) == {a, b, c}
        assert totals[tag][:2] == (2, 0), tag
    assert totals["member-detached"][:2] == (3, 0)
    assert get("rolled", b)["client"][1] == 72
    assert get("unrolled", b)["client"] == (800, 600)
    assert get("move-burst", b)["rect"] == (800, 400, 1600, 1000)
    assert get("move-release", b)["rect"] == (820, 430, 1620, 1030)
    for tag in ("move-cancelled", "move-right-cancelled"):
        assert get(tag, b)["rect"] == get("move-release", b)["rect"], tag
    assert get("capture-released", b)["rect"] == get("capture-pending", b)["rect"]
    assert get("locked-move", b)["rect"] == get("locked-ready", b)["rect"]
    for tag in ("theme-fluent", "theme-liquid", "click-expanded"):
        assert get(tag, b)["client"] == (800, 600), tag
    assert get("click-rolled", b)["rolled"]
    assert not get("click-expanded", b)["rolled"]
    for key in ("map_builds", "control_map_builds", "wallpaper_uploads"):
        assert get("move-start", b)[key] == get("move-release", b)[key], key
    for (tag, title), state in states.items():
        if "pending" not in tag and tag != "move-burst":
            assert not any(state[k] for k in ("capture", "tab_drag", "window_drag", "detach_pending")), (tag, title)
    for i in range(2):
        for label, hidden in [("shown", False), ("hidden", True), ("hidden-again", True), ("restored", False)]:
            assert totals[f"icons-{i}-{label}"][2:] == (hidden, hidden)
    assert totals["final"] == (2, 0, False, False)
    print(json.dumps({"checkpoints": len(totals), "window_states": len(states), "result": "PASS"}, indent=2))


if __name__ == "__main__":
    if sys.argv[1] == "prepare":
        prepare(Path(sys.argv[2]), Path(sys.argv[3]).resolve())
    else:
        check(Path(sys.argv[2]))
