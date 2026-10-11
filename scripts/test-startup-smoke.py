"""Check native first-frame startup in isolated Windows instances (not a cold-logon test).

Build the package binaries first. The test never hides desktop icons, changes the
production config, or creates/deletes production startup registrations.
"""
import argparse
import ctypes
from ctypes import wintypes
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import struct
import time
import uuid
import winreg

ROOT = Path(__file__).resolve().parents[1]
RUN_KEY = r"Software\Microsoft\Windows\CurrentVersion\Run"
USER32 = ctypes.WinDLL("user32", use_last_error=True)
CALLBACK = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
USER32.EnumWindows.argtypes = [CALLBACK, wintypes.LPARAM]
USER32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
USER32.GetClassNameW.argtypes = [wintypes.HWND, wintypes.LPWSTR, ctypes.c_int]
USER32.PostMessageW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]


def startup_snapshot():
    result = {}
    for name in ("PecoFence", "openFence"):
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, RUN_KEY) as key:
                result[name + ":run"] = winreg.QueryValueEx(key, name)
        except FileNotFoundError:
            result[name + ":run"] = None
        task = subprocess.run(
            ["schtasks.exe", "/Query", "/TN", name, "/XML"],
            capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW, timeout=15,
        )
        result[name + ":task"] = task.stdout if task.returncode == 0 else None
    return result


def control_window(pid):
    found = []

    @CALLBACK
    def visit(hwnd, _):
        owner = wintypes.DWORD()
        USER32.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
        if owner.value == pid:
            name = ctypes.create_unicode_buffer(256)
            USER32.GetClassNameW(hwnd, name, len(name))
            if name.value == "PecoFence.Control":
                found.append(hwnd)
        return True

    USER32.EnumWindows(visit, 0)
    return found[0] if found else None


def check_phases(log, *, wallpaper_expected):
    assert log.index("startup wallpaper discovery and decode queued in background") < log.index(
        "startup phase: initializing render stack"), "Wallpaper was not queued before rendering"
    assert "desktop icons hidden" not in log, "The isolated test hid real desktop icons"
    assert "stale icons-hidden marker found" not in log, "False previous-crash warning"
    assert "startup presentation unavailable" not in log, "First-frame handoff failed"
    assert "startup task registered" not in log, "Named instance reconciled production startup"
    assert "startup Run key fallback registered" not in log
    assert "panicked" not in log
    frames = [line for line in log.splitlines() if "[ready] live hwnd=" in line]
    assert frames, "No live fence was inspected"
    if wallpaper_expected:
        phases = [
            "startup wallpaper ready; fences updated",
            "startup phase: committing wallpaper material before showing fences",
            "startup phase: hidden material commit completed",
            "startup phase: showing prepared fences without fade",
            "startup phase: first fence frame presented",
        ]
        offsets = [log.index(phase) for phase in phases]
        assert offsets == sorted(offsets), "A fence was exposed before its real material committed"
        assert all("visible=true" in line and "opacity=1.00" in line and "fade=none" in line
                   for line in frames), "A prepared fence remained hidden or transparent"
        presented = next(line for line in log.splitlines() if phases[-1] in line)
        first_frame_ms = int(re.search(r"elapsed_ms=(\d+)", presented)[1])
    else:
        assert "startup wallpaper unavailable; retaining backdrop and retrying" in log
        assert "startup phase: showing prepared fences" not in log
        assert "startup phase: first fence frame presented" not in log
        assert all("visible=false" in line for line in frames), "A wallpaper error exposed a placeholder"
        first_frame_ms = None
    return {"first_frame_ms": first_frame_ms,
            "inspected_fences": len(frames), "wallpaper_loaded": wallpaper_expected}


def write_test_wallpaper(path):
    # A tiny, valid BGRA BMP; WIC maps it to the real monitor geometry in the worker.
    pixels = bytes([64, 128, 192, 255]) * 16
    path.write_bytes(struct.pack("<2sIHHI", b"BM", 54 + len(pixels), 0, 0, 54)
                     + struct.pack("<IiiHHIIiiII", 40, 4, 4, 1, 32, 0, len(pixels), 0, 0, 0, 0)
                     + pixels)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/package/release/pecofence.exe")
    args = parser.parse_args()
    binary = args.binary.resolve()
    instance = "startup-smoke-" + uuid.uuid4().hex[:12]
    stage = ROOT / ".cache" / instance
    stage.mkdir(parents=True)
    for name in ("pecofence.exe", "pecofence-watchdog.exe"):
        shutil.copy2(binary.parent / name, stage / name)
    shutil.copy2(ROOT / "third_party/webview2/WebView2Loader.x64.dll", stage / "WebView2Loader.dll")
    env = dict(os.environ, PECOFENCE_INSTANCE=instance, RUST_LOG="info",
               LOCALAPPDATA=str(stage / "local-appdata"), APPDATA=str(stage / "roaming-appdata"))
    # Do not inherit diagnostic overrides from the developer's shell.
    for key in ("PECOFENCE_SOLID", "OPENFENCE_SOLID"):
        env.pop(key, None)
    script = stage / "startup.txt"
    script.write_text("sleep 1800\ndump waiting\nsleep 4200\ndump ready\nexit\n", encoding="utf-8")
    log_path = Path(env["LOCALAPPDATA"]) / "PecoFence" / f"pecofence.{instance}.log"
    before = startup_snapshot()
    reports = {}
    try:
        for case in ("first-run", "portable-restart", "cached-restart", "corrupt-cache",
                     "wallpaper-error", "wallpaper-recovery"):
            if case == "corrupt-cache":
                cache = stage / "config/cache/startup-wallpaper.bin"
                assert cache.is_file(), "Startup wallpaper cache was not written"
                cache.write_bytes(b"invalid cache")
            command = [str(stage / "pecofence.exe"), "--portable", "--no-hide-icons",
                       "--test-script", str(script), "--exit-after", "12000"]
            if case in ("wallpaper-error", "wallpaper-recovery"):
                command += ["--wallpaper", str(stage / "missing-wallpaper.png")]
            process = subprocess.Popen(command, cwd=stage, env=env,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
            try:
                deadline = time.monotonic() + 10
                hwnd = None
                while process.poll() is None and time.monotonic() < deadline:
                    hwnd = control_window(process.pid)
                    if hwnd:
                        break
                    time.sleep(0.02)
                assert hwnd, "Test instance did not create its control window"
                if case in ("portable-restart", "cached-restart"):
                    # Only the test process receives the display-change notification.
                    assert USER32.PostMessageW(hwnd, 0x007E, 32, 0)
                if case == "wallpaper-recovery":
                    time.sleep(2.8)
                    assert process.poll() is None, "The UI did not survive a wallpaper failure"
                    write_test_wallpaper(stage / "missing-wallpaper.png")
                assert process.wait(timeout=25) == 0, "Test instance failed"
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=5)
            log = log_path.read_text(encoding="utf-8")
            (stage / f"{case}.log").write_text(log, encoding="utf-8")
            reports[case] = check_phases(log, wallpaper_expected=case != "wallpaper-error")
            if case == "wallpaper-recovery":
                waiting = [line for line in log.splitlines() if "[waiting] live hwnd=" in line]
                assert waiting and all("visible=false" in line for line in waiting), (
                    "The anchor revealed incomplete fences while wallpaper was unavailable")
            if case in ("cached-restart", "corrupt-cache"):
                cache_hit = "cache_hit=true" in next(line for line in log.splitlines()
                                                     if "startup wallpaper prepared in background" in line)
                assert cache_hit == (case == "cached-restart"), "Unexpected disk cache reuse"
                reports[case]["cache_hit"] = cache_hit
            config = stage / "config/config.json"
            assert config.is_file() and str(config) in log, "Portable config path was not used"
            if case in ("portable-restart", "cached-restart"):
                assert "work area changed; re-laying out fences" in log
                assert "work-area-only change; keeping prepared wallpaper" in log
                assert log.count("startup wallpaper discovery and decode queued in background") == 1
                assert "startup backdrop refresh queued; retaining current frame" not in log
                assert "backdrops ready" not in log, "Display change synchronously decoded wallpaper"
                assert not json.loads(config.read_text(encoding="utf-8"))["settings"]["autostart"]
            if case == "first-run":
                saved = json.loads(config.read_text(encoding="utf-8"))
                saved["settings"].update(autostart=False, hideRealIcons=False,
                                         theme="dark", themeStyle="liquidGlass")
                config.write_text(json.dumps(saved, ensure_ascii=False), encoding="utf-8")
    finally:
        assert startup_snapshot() == before, "Production startup registrations changed"
    reports["production_startup_unchanged"] = True
    (stage / "report.json").write_text(json.dumps(reports, indent=2), encoding="utf-8")
    print(json.dumps(reports, indent=2))
    print(f"Report: {stage / 'report.json'}")


if __name__ == "__main__":
    main()
