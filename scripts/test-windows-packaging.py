"""Verify packaged bytes and exercise the real installer under a unique test identity.

Run after make-windows.ps1. Requires Python 3.11+ and Inno Setup 7. Never installs
the production setup EXE or launches PecoFence. Reports/logs stay under .cache/.
"""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
import tomllib
import uuid
import winreg
import zipfile

ROOT = Path(__file__).resolve().parents[1]
UNINSTALL = r"Software\Microsoft\Windows\CurrentVersion\Uninstall"
RUN = r"Software\Microsoft\Windows\CurrentVersion\Run"
PAYLOAD = {
    "pecofence.exe", "pecofence-cli.exe", "pecofence-watchdog.exe",
    "WebView2Loader.dll", "deployment.json", "release-info.json", "LICENSE",
    "LICENSE-WebView2Loader.txt", "THIRD-PARTY-LICENSES.txt", "README.md",
    "UPGRADING.md", "SKILL.md",
}
HIDDEN = subprocess.STARTUPINFO()
HIDDEN.dwFlags |= subprocess.STARTF_USESHOWWINDOW
HIDDEN.wShowWindow = 0


def digest(data):
    return hashlib.sha256(data).hexdigest()


def reg_value(key, name):
    try:
        with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key) as handle:
            return winreg.QueryValueEx(handle, name)[0]
    except FileNotFoundError:
        return None


def run(command, *, success=True):
    result = subprocess.run([str(arg) for arg in command], cwd=ROOT,
                            startupinfo=HIDDEN, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=90)
    assert (result.returncode == 0) == success, (
        f"Exit {result.returncode}: {command}\n{result.stdout}\n{result.stderr}"
    )
    return result


def shell_folder(csidl):
    path = ctypes.create_unicode_buffer(260)
    shell = ctypes.WinDLL("shell32")
    shell.SHGetFolderPathW.argtypes = [wintypes.HWND, ctypes.c_int, wintypes.HANDLE,
                                      wintypes.DWORD, wintypes.LPWSTR]
    assert shell.SHGetFolderPathW(None, csidl, None, 0, path) == 0
    return Path(path.value)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, default=ROOT / "target/package")
    parser.add_argument("--repository", default=os.getenv("GITHUB_REPOSITORY") or "DayuanJiang/PecoFence")
    parser.add_argument("--iscc", default=os.getenv("ISCC"))
    args = parser.parse_args()
    iscc = args.iscc or shutil.which("ISCC.exe")
    if not iscc:
        for base in (os.getenv("ProgramFiles"), os.getenv("ProgramFiles(x86)"),
                     str(Path(os.environ["LOCALAPPDATA"]) / "Programs")):
            candidate = Path(base or "") / "Inno Setup 7/ISCC.exe"
            if candidate.is_file():
                iscc = str(candidate)
                break
    assert iscc, "Install Inno Setup 7 or pass --iscc"
    assert run([iscc, "--version"]).stdout.strip().startswith("7.")
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    base = f"pecofence-v{version}-x64"
    dist = ROOT / "dist"
    payload = dist / ".staging" / f"{base}-installed"
    passed = []

    for suffix in ("portable.zip", "setup.exe"):
        asset = dist / f"{base}-{suffix}"
        expected = f"{digest(asset.read_bytes())}  {asset.name}\n"
        assert Path(str(asset) + ".sha256").read_text(encoding="utf-8") == expected
    assert set(p.name for p in payload.iterdir()) == PAYLOAD
    expected_info = {"schema": 1, "repository": args.repository, "version": version, "tag": f"v{version}"}
    with zipfile.ZipFile(dist / f"{base}-portable.zip") as archive:
        assert set(archive.namelist()) == PAYLOAD, "Unexpected files or directories in ZIP"
        assert json.loads(archive.read("deployment.json")) == {
            "schema": 1, "appId": "PecoFence", "mode": "portable"}
        assert json.loads(archive.read("release-info.json")) == expected_info
        assert json.loads((payload / "release-info.json").read_bytes()) == expected_info
        assert json.loads((payload / "deployment.json").read_bytes()) == {
            "schema": 1, "appId": "PecoFence", "mode": "installed"}
        for name in ("pecofence.exe", "pecofence-cli.exe", "pecofence-watchdog.exe", "WebView2Loader.dll"):
            portable_bytes = archive.read(name)
            assert portable_bytes == (payload / name).read_bytes(), f"Different binaries: {name}"
            source = (ROOT / "third_party/webview2/WebView2Loader.x64.dll" if name.endswith(".dll")
                      else args.target_dir / "release" / name)
            assert portable_bytes == source.read_bytes(), f"Stale payload: {name}"
    passed.append("ZIP payload, both markers, identical build outputs, repository metadata and both checksums")

    identity = uuid.uuid4().hex[:12]
    product_id = f"PecoFence.InstallerTest.{identity}"
    product_name = f"PecoFence Installer Test {identity}"
    uninstall_key = UNINSTALL + "\\" + product_id + "_is1"
    run_names = (product_id, product_id + ".legacy")
    stage = ROOT / ".cache" / f"installer-{identity}"
    stage.mkdir(parents=True)
    installed = stage / "installed copy"
    fixture = stage / "payload"
    shutil.copytree(payload, fixture)
    report = stage / "report.json"
    actual_startup = {name: reg_value(RUN, name) for name in ("PecoFence", "openFence")}
    assert reg_value(uninstall_key, "InstallLocation") is None
    assert all(reg_value(RUN, name) is None for name in run_names)
    desktop_link = shell_folder(0x10) / f"{product_name}.lnk"
    menu_group = shell_folder(0x02) / product_name
    assert not desktop_link.exists() and not menu_group.exists()

    def setup(exe, name, *extra, success=True):
        run([exe, "/SP-", "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART",
             "/LANG=en", f"/LOG={stage / (name + '.log')}", *extra], success=success)

    def uninstall(name):
        exe = installed / "unins000.exe"
        if reg_value(uninstall_key, "InstallLocation") is not None:
            assert exe.is_file(), "Test uninstall entry exists but uninstaller is missing"
            setup(exe, name)
            # Inno's original uninstaller exits before its temporary child finishes.
            # Wait for our final callback, not just the launcher's process exit.
            log = stage / (name + ".log")
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if log.is_file() and "PecoFence uninstall cleanup completed" in log.read_text(encoding="utf-8-sig", errors="replace"):
                    break
                time.sleep(0.1)
            else:
                raise AssertionError(f"Uninstaller did not finish; see {log}")
        assert reg_value(uninstall_key, "InstallLocation") is None, "Test uninstall entry remains"

    try:
        installers = []
        # Synthetic versions test upgrade policy without altering workspace versions.
        for number in (1, 2):
            test_version = f"0.0.{number}"
            info = dict(expected_info, version=test_version, tag=f"v{test_version}")
            (fixture / "release-info.json").write_text(json.dumps(info), encoding="utf-8")
            run([iscc, "--quiet", f"--define=PayloadDir={fixture}",
                 f"--define=AppVersion={test_version}", f"--define=Repository={args.repository}",
                 f"--define=TestIdentity={identity}", f"--output-dir={stage}",
                 f"--output-filename=setup-{number}", ROOT / "packaging/inno/pecofence.iss"])
            installers.append(stage / f"setup-{number}.exe")
        old, new = installers

        for name, marker in (("portable", '{"schema":1,"appId":"PecoFence","mode":"portable"}'),
                             ("unowned", None)):
            folder = stage / name
            folder.mkdir()
            sentinel = folder / "keep.txt"
            sentinel.write_text("unchanged", encoding="utf-8")
            if marker:
                (folder / "deployment.json").write_text(marker, encoding="utf-8")
            before = {p.name: p.read_bytes() for p in folder.iterdir()}
            setup(old, f"reject-{name}", f"/DIR={folder}", success=False)
            assert {p.name: p.read_bytes() for p in folder.iterdir()} == before
            assert reg_value(uninstall_key, "InstallLocation") is None
        passed.append("portable and unrelated nonempty folders are not overwritten")

        # Exercise the same AppMutex mechanism with a unique name, never the real app.
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.CreateMutexW.argtypes = [ctypes.c_void_p, wintypes.BOOL, wintypes.LPCWSTR]
        kernel.CreateMutexW.restype = wintypes.HANDLE
        kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        mutex = kernel.CreateMutexW(None, False, "Local\\" + product_id)
        assert mutex
        try:
            setup(old, "reject-running", f"/DIR={installed}", success=False)
            assert not (installed / "pecofence.exe").exists()
        finally:
            kernel.CloseHandle(mutex)
        passed.append("a running app blocks setup without forced termination")

        setup(old, "install", f"/DIR={installed}", "/TASKS=desktopicon")
        assert Path(reg_value(uninstall_key, "InstallLocation")) == installed
        assert reg_value(uninstall_key, "DisplayVersion") == "0.0.1"
        assert reg_value(uninstall_key, "URLUpdateInfo") == f"https://github.com/{args.repository}/releases"
        assert all(reg_value(RUN, name) is None for name in run_names), "Setup enabled autostart"
        assert desktop_link.is_file() and (menu_group / f"{product_name}.lnk").is_file()
        for name in PAYLOAD - {"release-info.json"}:
            assert (installed / name).read_bytes() == (payload / name).read_bytes()
        passed.append("per-user install, exact payload, shortcuts, fork links and no setup autostart")

        marker_file = installed / "deployment.json"
        valid_marker = marker_file.read_bytes()
        marker_file.write_text('{"schema":1,"appId":"PecoFence","mode":"portable"}', encoding="utf-8")
        setup(new, "reject-changed-marker", success=False)
        assert "portable" in marker_file.read_text(encoding="utf-8")
        marker_file.write_bytes(valid_marker)
        setup(new, "reject-relocation", f"/DIR={stage / 'other install'}", success=False)
        assert not (stage / "other install/pecofence.exe").exists()
        passed.append("upgrade refuses changed distribution marker and installation directory")

        retained = {}
        for name in ("config/config.json", "data/logs/keep.log", "user-file.txt"):
            path = installed / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("user data retained", encoding="utf-8")
            retained[path] = path.read_bytes()
        # Prove installed program files really are replaced on upgrade.
        (installed / "pecofence-cli.exe").write_bytes(b"stale binary fixture")
        # The validated distribution identity must not be rewritten mid-upgrade.
        os.utime(marker_file, (946684800, 946684800))
        marker_stamp = marker_file.stat().st_mtime_ns
        setup(new, "upgrade")
        assert marker_file.stat().st_mtime_ns == marker_stamp
        assert Path(reg_value(uninstall_key, "InstallLocation")) == installed
        assert reg_value(uninstall_key, "DisplayVersion") == "0.0.2"
        assert (installed / "pecofence-cli.exe").read_bytes() == (payload / "pecofence-cli.exe").read_bytes()
        assert json.loads((installed / "release-info.json").read_bytes())["version"] == "0.0.2"
        assert all(p.read_bytes() == data for p, data in retained.items())
        assert len(list(installed.glob("unins*.exe"))) == 1
        passed.append("upgrade reuses directory/uninstaller, replaces binaries and preserves added data")

        setup(old, "reject-downgrade", success=False)
        assert reg_value(uninstall_key, "DisplayVersion") == "0.0.2"
        passed.append("numeric version downgrade is blocked")

        other_command = f'"{stage / "another copy/pecofence.exe"}"'
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, RUN) as key:
            winreg.SetValueEx(key, run_names[0], 0, winreg.REG_SZ, f'"{installed / "pecofence.exe"}"')
            winreg.SetValueEx(key, run_names[1], 0, winreg.REG_SZ, other_command)
        uninstall("uninstall")
        assert not any((installed / name).exists() for name in PAYLOAD)
        assert not desktop_link.exists() and not menu_group.exists()
        assert all(p.read_bytes() == data for p, data in retained.items())
        assert reg_value(RUN, run_names[0]) is None
        assert reg_value(RUN, run_names[1]) == other_command
        passed.append("uninstall removes managed files/shortcuts, keeps data and another copy's startup entry")
    finally:
        # The only persistent writes outside .cache belong to this generated test ID.
        try:
            uninstall("cleanup")
        finally:
            with winreg.CreateKey(winreg.HKEY_CURRENT_USER, RUN) as key:
                for name in run_names:
                    try:
                        winreg.DeleteValue(key, name)
                    except FileNotFoundError:
                        pass
            assert {name: reg_value(RUN, name) for name in actual_startup} == actual_startup
            report.write_text(json.dumps({"passed": passed, "testIdentity": product_id}, indent=2), encoding="utf-8")
            print(f"{len(passed)} packaging/installer checks passed; report: {report}")


if __name__ == "__main__":
    main()
