"""Windows integration tests using disposable copies; never hide desktop icons.

Run after building the app, watchdog and CLI (default: target/debug).
Use --bin-dir target/package/release to test release artifacts.
"""
import argparse
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import uuid
import winreg

ROOT = Path(__file__).resolve().parents[1]
HIDDEN = subprocess.STARTUPINFO()
HIDDEN.dwFlags |= subprocess.STARTF_USESHOWWINDOW
HIDDEN.wShowWindow = 0


def startup_entries():
    values = {}
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER,
                           r"Software\Microsoft\Windows\CurrentVersion\Run") as key:
            for name in ("PecoFence", "openFence"):
                try:
                    values[name] = winreg.QueryValueEx(key, name)
                except FileNotFoundError:
                    values[name] = None
    except FileNotFoundError:
        pass
    return values


def dismiss_startup_error(process, expected):
    """Inspect and dismiss only the error dialog belonging to this test process."""
    user = ctypes.WinDLL("user32", use_last_error=True)
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
    user.GetWindowTextW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
    user.GetClassNameW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
    user.PostMessageW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
    user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
    user.EnumChildWindows.argtypes = [wintypes.HWND, callback_type, wintypes.LPARAM]
    found = []

    def window(hwnd, _):
        pid = wintypes.DWORD()
        user.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        kind = ctypes.create_unicode_buffer(128)
        user.GetClassNameW(hwnd, kind, len(kind))
        if pid.value != process.pid or kind.value != "#32770":
            return True
        text = []

        def child(control, _):
            value = ctypes.create_unicode_buffer(8192)
            user.GetWindowTextW(control, value, len(value))
            text.append(value.value)
            return True

        user.EnumChildWindows(hwnd, callback_type(child), 0)
        found.append("\n".join(text))
        user.PostMessageW(hwnd, 0x0010, 0, 0)  # WM_CLOSE on this test's message box
        return True

    deadline = time.monotonic() + 15
    while not found and process.poll() is None and time.monotonic() < deadline:
        user.EnumWindows(callback_type(window), 0)
        time.sleep(0.1)
    assert found and expected in found[0], f"Missing startup error for {expected}: {found}"
    assert process.wait(timeout=5) == 1, "Startup error did not fail the process"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", type=Path, default=ROOT / "target/debug")
    args = parser.parse_args()
    binaries = args.bin_dir.resolve()
    stage = ROOT / ".cache" / ("distribution-" + uuid.uuid4().hex[:10])
    stage.mkdir(parents=True)
    original = stage / "portable copy"
    other = stage / "installed copy"
    unrelated_cwd = stage / "unrelated cwd"
    unrelated_cwd.mkdir()
    instance = "distribution-" + uuid.uuid4().hex[:10]
    environment = dict(os.environ, PECOFENCE_INSTANCE=instance, RUST_LOG="info",
                       APPDATA=str(stage / "roaming"), LOCALAPPDATA=str(stage / "local"))
    before = startup_entries()
    children = []
    passed = []

    def stage_copy(folder, mode):
        folder.mkdir()
        for name in ("pecofence.exe", "pecofence-cli.exe", "pecofence-watchdog.exe"):
            shutil.copy2(binaries / name, folder / name)
        shutil.copy2(ROOT / "third_party/webview2/WebView2Loader.x64.dll",
                     folder / "WebView2Loader.dll")
        (folder / "deployment.json").write_text(json.dumps(
            {"schema": 1, "appId": "PecoFence", "mode": mode}), encoding="utf-8")

    def cli(folder, *arguments, code=0):
        result = subprocess.run([str(folder / "pecofence-cli.exe"), "--compact", *arguments],
                                env=environment, cwd=unrelated_cwd, capture_output=True,
                                text=True, encoding="utf-8", timeout=20,
                                creationflags=subprocess.CREATE_NO_WINDOW)
        assert result.returncode == code, f"{arguments}: {result.returncode}: {result.stderr}"
        return json.loads(result.stdout if code == 0 else result.stderr)

    def launch(folder, *arguments, lifetime=9000):
        process = subprocess.Popen([str(folder / "pecofence.exe"), "--no-hide-icons",
                                    "--exit-after", str(lifetime), *arguments],
                                   env=environment, cwd=unrelated_cwd, startupinfo=HIDDEN,
                                   creationflags=subprocess.CREATE_NO_WINDOW)
        children.append(process)
        return process

    def ready(folder):
        deadline = time.monotonic() + 12
        while time.monotonic() < deadline:
            try:
                paths = cli(folder, "paths")
                if paths["running"]:
                    return paths
            except AssertionError:
                pass
            time.sleep(0.1)
        raise AssertionError("App never became ready")

    def local_paths(folder, paths):
        assert paths["distribution"] == "portable", paths
        for key in ("config", "backupsDir", "log", "crashDir", "webviewDataDir", "recoveryMarker"):
            assert Path(paths[key]).is_relative_to(folder), (key, paths[key])
        assert Path(paths["config"]) == folder / "config/config.json"

    try:
        stage_copy(original, "portable")
        stage_copy(other, "installed")
        # Deliberately broken legacy data must never be adopted by the portable app.
        legacy = stage / "roaming/OpenFence/config.json"
        legacy.parent.mkdir(parents=True)
        legacy.write_text("legacy sentinel", encoding="utf-8")
        offline = cli(original, "paths")
        local_paths(original, offline)
        assert not offline["running"] and not (original / "data").exists()
        assert Path(cli(other, "paths")["config"]) == legacy
        passed.append("offline paths and legacy selection")

        app = launch(original, "--open-settings", lifetime=18000)
        paths = ready(original)
        assert paths["running"] and paths["logExists"]
        local_paths(original, paths)
        monitors = cli(original, "monitor", "list")
        # Pointer/keyboard checks live in the browser suite; this exercises the backend.
        initial = json.loads((original / "config/config.json").read_text(encoding="utf-8"))
        preference = initial["settings"]["autostart"]
        cli(original, "settings", "set", "autostart", str(not preference).lower())
        config = json.loads((original / "config/config.json").read_text(encoding="utf-8"))
        assert config["settings"]["autostart"] == preference, "IPC changed portable autostart"
        imported = json.loads(json.dumps(config))
        imported["settings"]["autostart"] = not preference
        import_path = stage / "import.json"
        import_path.write_text(json.dumps(imported), encoding="utf-8")
        cli(original, "config", "import", str(import_path))
        config = json.loads((original / "config/config.json").read_text(encoding="utf-8"))
        assert config["settings"]["autostart"] == preference, "Import changed portable autostart"
        foreign = cli(other, "settings", "set", "autostart", "true", code=1)
        assert foreign["error"]["code"] == "version_mismatch", foreign
        assert startup_entries() == before, "Autostart registry entries changed"
        assert app.wait(timeout=25) == 0
        log = Path(paths["log"]).read_text(encoding="utf-8")
        assert "settings: page ready" in log, "Native WebView2 did not initialize"
        assert list(Path(paths["webviewDataDir"]).iterdir()), "WebView2 profile stayed empty"
        assert not (stage / "local/PecoFence").exists(), "Portable wrote to installed local data"
        assert legacy.read_text(encoding="utf-8") == "legacy sentinel"
        passed.extend(["automatic portable detection and native WebView2 paths",
                       "IPC/import autostart isolation", "cross-copy CLI rejection"])

        moved = stage / "moved portable copy"
        assert original.resolve().is_relative_to(stage.resolve())
        assert moved.resolve().is_relative_to(stage.resolve())
        deadline = time.monotonic() + 10
        while True:
            try:
                original.rename(moved)
                break
            except PermissionError:
                if time.monotonic() >= deadline:
                    raise
                time.sleep(0.2)  # WebView2 children may still be closing.
        app = launch(moved)
        moved_paths = ready(moved)
        local_paths(moved, moved_paths)
        assert not original.exists()
        assert app.wait(timeout=15) == 0
        passed.append("relocation from unrelated working directory")

        (moved / "deployment.json").unlink()
        app = launch(moved, "--portable")
        local_paths(moved, ready(moved))
        assert app.wait(timeout=15) == 0
        passed.append("legacy --portable CLI diagnostics")

        marker = moved / "deployment.json"
        marker.write_text('{"schema":1,"appId":"PecoFence","mode":"portable"}', encoding="utf-8")
        log_dir = moved / "data/logs"
        # Only move our known scratch directory, after every owned app has exited.
        assert log_dir.resolve().is_relative_to(stage.resolve())
        assert (moved / "saved-logs").resolve().is_relative_to(stage.resolve())
        log_dir.rename(moved / "saved-logs")
        log_dir.write_text("blocked", encoding="utf-8")
        app = launch(moved)
        dismiss_startup_error(app, str(log_dir))
        assert not (stage / "local/PecoFence").exists()
        marker.write_text("invalid marker", encoding="utf-8")
        app = launch(moved)
        dismiss_startup_error(app, "deployment.json")
        assert cli(moved, "paths", code=1)["error"]["code"] == "invalid_value"
        passed.append("blocked paths and invalid markers fail without fallback")
        assert startup_entries() == before
        report = {"passed": passed, "autostart_preserved": True,
                  "monitors": monitors,
                  "windows_version": str(__import__("sys").getwindowsversion())}
        (stage / "report.json").write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(f"PASS: {len(passed)} distribution scenarios; report: {stage / 'report.json'}")
    finally:
        for child in children:
            if child.poll() is None:
                child.terminate()
                child.wait(timeout=10)


if __name__ == "__main__":
    main()
