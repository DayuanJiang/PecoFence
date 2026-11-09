"""Check whole-fence idle opacity in an isolated native Windows debug instance."""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import uuid

from importlib.util import module_from_spec, spec_from_file_location

ROOT = Path(__file__).resolve().parents[1]
spec = spec_from_file_location("language_smoke", Path(__file__).with_name("test-language-smoke.py"))
language_smoke = module_from_spec(spec)
spec.loader.exec_module(language_smoke)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/pecofence.exe")
    args = parser.parse_args()
    binary = args.binary.resolve()
    instance = "hover-smoke-" + uuid.uuid4().hex[:12]
    stage = ROOT / ".cache" / instance
    stage.mkdir(parents=True)
    for name in ("pecofence.exe", "pecofence-watchdog.exe"):
        shutil.copy2(binary.parent / name, stage / name)
    shutil.copy2(ROOT / "third_party/webview2/WebView2Loader.x64.dll", stage / "WebView2Loader.dll")
    env = dict(os.environ, PECOFENCE_INSTANCE=instance, PECOFENCE_UI_TEST_WINDOWS="1",
               LOCALAPPDATA=str(stage / "local-appdata"), APPDATA=str(stage / "roaming-appdata"),
               RUST_LOG="info")
    before = language_smoke.autostart_value()

    def run(*extra):
        result = subprocess.run([str(stage / "pecofence.exe"), "--portable", "--no-hide-icons", *extra],
                                cwd=stage, env=env, timeout=55, creationflags=subprocess.CREATE_NO_WINDOW)
        assert result.returncode == 0, f"Native exit code: {result.returncode}"

    run("--exit-after", "1200")
    config_path = stage / "config/config.json"
    config = json.loads(config_path.read_text(encoding="utf-8"))
    settings = config["settings"]
    settings.update(language="en", autostart=False, hideRealIcons=False, idleOpacity=1.0)
    settings["peek"].update(enabled=True, dim=False)
    settings["quickHide"]["enabled"] = True
    settings["icons"]["chameleon"] = False
    config["rules"]["enabled"] = False
    # Keep synthetic test content out of the real desktop and use only portal reads.
    portal = stage / "sample"
    portal.mkdir()
    (portal / "Sample.txt").write_text("Hover fade test", encoding="utf-8")
    for layout in config["layouts"]:
        fence = copy.deepcopy(next(f for f in layout["fences"] if f["kind"] == "virtual"))
        fence.update(title="HoverProbe", kind="folderPortal", source={"kind": "folder", "path": str(portal)},
                     items=[], desktopLayout=False, rolledUp=False, tabHost=None, activeTab=None,
                     excludeFromQuickHide=False, locked=False, hideWhenEmpty=False)
        fence["view"].update(autoHeight=False)
        other = copy.deepcopy(fence)
        other.update(id=str(uuid.uuid4()), title="TabProbe")
        layout["fences"] = [fence, other]
    config_path.write_text(json.dumps(config), encoding="utf-8")
    lines = ["sleep 1400", "bounds HoverProbe 50 50 360 280", "bounds TabProbe 440 50 360 280",
             "input HoverProbe hover 0 0", "sleep 500", "dump disabled"]

    def setting(opacity):
        settings["idleOpacity"] = opacity
        lines.append("message " + json.dumps({"type": "patchSettings", "settings": settings}))

    def sample(tag, *commands):
        lines.extend(commands)
        lines.extend(("sleep 650", "dump " + tag))

    setting(0.45)
    sample("idle")
    sample("hover", "input HoverProbe hover 1 0")
    sample("leave", "input HoverProbe hover 0 0")
    sample("reverse", "input HoverProbe hover 1 0", "sleep 40", "input HoverProbe hover 0 0")
    sample("keyboard", "input HoverProbe activate 1 0", "input HoverProbe key 36 0")
    sample("blur", "input HoverProbe activate 0 0")
    sample("rename", "edit-item HoverProbe Sample", "input HoverProbe hover 0 0")
    sample("rename-end", "cancel-rename", "input HoverProbe activate 0 0")
    sample("rolled", "roll HoverProbe")
    sample("rolled-hover", "input HoverProbe hover 1 0")
    sample("expanded", "input HoverProbe hover 0 0", "unroll HoverProbe")
    sample("peek", "peek")
    sample("end-peek", "end-peek")
    sample("hidden", "quick-hide")
    sample("shown", "quick-show", "input HoverProbe hover 0 0")
    sample("tab", "merge TabProbe HoverProbe", "activate TabProbe", "input HoverProbe hover 0 0")
    sample("tab-hover", "input HoverProbe hover 1 0")
    sample("tab-idle", "input HoverProbe hover 0 0")
    setting(1.0)
    sample("off")
    setting(0.45)
    sample("saved")
    lines.append("exit")
    script = stage / "hover.txt"
    script.write_text("\n".join(lines), encoding="utf-8")
    run("--test-script", str(script), "--exit-after", "45000")
    log_path = Path(env["LOCALAPPDATA"]) / "PecoFence" / f"pecofence.{instance}.log"
    log = log_path.read_text(encoding="utf-8")
    checks = {}
    expected = dict(disabled=1, idle=.45, hover=1, leave=.45, reverse=.45, keyboard=1, blur=.45,
                    rolled=.45, expanded=.45, peek=1, shown=.45, tab=.45, off=1, saved=.45)
    expected.update({"rolled-hover": 1, "end-peek": .45, "tab-hover": 1, "tab-idle": .45,
                     "rename": 1, "rename-end": .45})
    for tag, opacity in expected.items():
        # After tab activation the host window reports the active tab's title.
        rows = [line for line in log.splitlines() if f"[{tag}] live" in line
                and ('title="HoverProbe"' in line or (tag.startswith("tab") or tag in ("off", "saved"))
                     and 'title="TabProbe"' in line)]
        assert len(rows) == 1, (tag, rows)
        actual = float(re.search(r"idle_opacity=([\d.]+)", rows[0])[1])
        assert abs(actual - opacity) < .015, (tag, actual, opacity, rows[0])
        checks[tag] = actual
    assert re.search(r'\[hidden\] live.*visible=false.*title="HoverProbe"', log), "Quick Hide failed"
    saved = json.loads(config_path.read_text(encoding="utf-8"))
    assert abs(saved["settings"]["idleOpacity"] - .45) < .00001
    assert language_smoke.autostart_value() == before, "Autostart entry changed"
    assert "script finished" not in log, "Script failed to exit"
    assert not re.search(r"ERROR|panicked at", log), "Native errors; inspect test log"
    report = {"passed": True, "checks": checks, "saved": True, "autostart_preserved": True}
    (stage / "report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(f"PASS: {len(checks)} native opacity states, Quick Hide, persisted setting and autostart")
    print(f"Report: {stage / 'report.json'}")


if __name__ == "__main__":
    main()
