"""Verify packaged bytes and exercise the real installer under a unique test identity.

Run after make-windows.ps1. Requires Python 3.11+ and Inno Setup 7. Never installs
the production setup EXE or launches PecoFence. Reports/logs stay under .cache/.
"""
import argparse
import ctypes
from ctypes import wintypes
import html
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
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
    "WebView2Loader.dll", "LICENSE",
    "LICENSE-WebView2Loader.txt", "THIRD-PARTY-LICENSES.txt", "README.md",
    "UPGRADING.md", "SKILL.md",
}
HIDDEN = subprocess.STARTUPINFO()
HIDDEN.dwFlags |= subprocess.STARTF_USESHOWWINDOW
HIDDEN.wShowWindow = 0


def task_xml(name):
    result = subprocess.run(["schtasks.exe", "/Query", "/TN", name, "/XML"],
                            startupinfo=HIDDEN, capture_output=True, timeout=15)
    return result.stdout if result.returncode == 0 else None


def delete_task(name):
    subprocess.run(["schtasks.exe", "/Delete", "/TN", name, "/F"],
                   startupinfo=HIDDEN, capture_output=True, timeout=15)


def create_task(name, executable, arguments="", extra_executable=None):
    user = os.environ.get("USERNAME", "")
    domain = os.environ.get("USERDOMAIN", "")
    user_id = f"{domain}\\{user}" if domain and user else user
    argument_xml = f"<Arguments>{html.escape(arguments)}</Arguments>" if arguments else ""
    extra_action = (f"<Exec><Command>{html.escape(str(extra_executable))}</Command></Exec>"
                    if extra_executable else "")
    xml = f'''<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{html.escape(user_id)}</UserId></LogonTrigger></Triggers>
  <Principals><Principal id="Author"><UserId>{html.escape(user_id)}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>
  <Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries><StopIfGoingOnBatteries>false</StopIfGoingOnBatteries><StartWhenAvailable>false</StartWhenAvailable><RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable><RunOnlyIfIdle>false</RunOnlyIfIdle><ExecutionTimeLimit>PT0S</ExecutionTimeLimit><Priority>3</Priority></Settings>
  <Actions Context="Author"><Exec><Command>{html.escape(str(executable))}</Command>{argument_xml}</Exec>{extra_action}</Actions>
</Task>
'''.encode("utf-16")
    path = Path(tempfile.gettempdir()) / f"pecofence-installer-test-{uuid.uuid4().hex}.xml"
    path.write_bytes(xml)
    try:
        run(["schtasks.exe", "/Create", "/TN", name, "/XML", path, "/F"])
    finally:
        path.unlink(missing_ok=True)


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


def find_iscc(path):
    """Resolve the compiler exactly as make-windows.ps1 does."""
    script = str(ROOT / "scripts/find-iscc.ps1").replace("'", "''")
    argument = (path or "").replace("'", "''")
    # PowerShell 7's PSModulePath breaks module autoloading in Windows PowerShell 5.1.
    env = {k: v for k, v in os.environ.items() if k.upper() != "PSMODULEPATH"}
    result = subprocess.run(["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command",
                             f". '{script}'; Find-Iscc '{argument}'"], cwd=ROOT, env=env,
                            startupinfo=HIDDEN, capture_output=True, text=True,
                            encoding="utf-8", errors="replace", timeout=90)
    assert result.returncode == 0, f"Inno Setup 7 not found:\n{result.stdout}\n{result.stderr}"
    return result.stdout.strip().splitlines()[-1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target-dir", type=Path, default=ROOT / "target/package")
    parser.add_argument("--repository", default=os.getenv("GITHUB_REPOSITORY") or "DayuanJiang/PecoFence")
    parser.add_argument("--iscc", help="ISCC.exe; default: the ISCC variable, PATH, then common install folders")
    parser.add_argument("--staging-tag", default="", help="StagingTag used by make-windows.ps1")
    args = parser.parse_args()
    iscc = find_iscc(args.iscc)
    version = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]["version"]
    base = f"pecofence-v{version}-x64"
    dist = ROOT / "dist"
    if args.staging_tag and not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]{0,63}", args.staging_tag):
        parser.error("Invalid --staging-tag")
    payload = dist / ".staging" / args.staging_tag / f"{base}-installed"
    passed = []

    for suffix in ("portable.zip", "setup.exe"):
        asset = dist / f"{base}-{suffix}"
        expected = f"{digest(asset.read_bytes())}  {asset.name}\n"
        assert Path(str(asset) + ".sha256").read_text(encoding="utf-8") == expected
    assert set(p.name for p in payload.iterdir()) == PAYLOAD
    with zipfile.ZipFile(dist / f"{base}-portable.zip") as archive:
        assert set(archive.namelist()) == PAYLOAD, "Unexpected files or directories in ZIP"
        for name in ("pecofence.exe", "pecofence-cli.exe", "pecofence-watchdog.exe", "WebView2Loader.dll"):
            portable_bytes = archive.read(name)
            assert portable_bytes == (payload / name).read_bytes(), f"Different binaries: {name}"
            source = (ROOT / "third_party/webview2/WebView2Loader.x64.dll" if name.endswith(".dll")
                      else args.target_dir / "release" / name)
            assert portable_bytes == source.read_bytes(), f"Stale payload: {name}"
    passed.append("ZIP and setup payloads, identical build outputs and both checksums")

    identity = uuid.uuid4().hex[:12]
    product_id = f"PecoFence.InstallerTest.{identity}"
    product_name = f"PecoFence Installer Test {identity}"
    uninstall_key = UNINSTALL + "\\" + product_id + "_is1"
    run_names = (product_id, product_id + ".legacy")
    task_names = (product_id + ".task", product_id + ".legacy-task")
    stage = ROOT / ".cache" / f"installer-{identity}"
    stage.mkdir(parents=True)
    installed = stage / "installed copy"
    fixture = stage / "payload"
    shutil.copytree(payload, fixture)
    report = stage / "report.json"
    actual_startup = {name: reg_value(RUN, name) for name in ("PecoFence", "openFence")}
    actual_tasks = {name: task_xml(name) for name in ("PecoFence", "openFence")}
    assert reg_value(uninstall_key, "InstallLocation") is None
    assert all(reg_value(RUN, name) is None for name in run_names)
    assert all(task_xml(name) is None for name in task_names)
    desktop_link = shell_folder(0x10) / f"{product_name}.lnk"
    menu_group = shell_folder(0x02) / product_name
    assert not desktop_link.exists() and not menu_group.exists()

    def setup(exe, name, *extra, success=True):
        run([exe, "/SP-", "/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART",
             "/LANG=en", f"/LOG={stage / (name + '.log')}", *extra], success=success)

    def uninstall(name):
        location = reg_value(uninstall_key, "InstallLocation")
        if location is not None:
            exe = Path(location) / "unins000.exe"
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
            run([iscc, "--quiet", f"--define=PayloadDir={fixture}",
                 f"--define=AppVersion={test_version}", f"--define=Repository={args.repository}",
                 f"--define=TestIdentity={identity}", f"--output-dir={stage}",
                 f"--output-filename=setup-{number}", ROOT / "packaging/inno/pecofence.iss"])
            installers.append(stage / f"setup-{number}.exe")
        old, new = installers

        unowned = stage / "unowned"
        unowned.mkdir()
        (unowned / "keep.txt").write_text("unchanged", encoding="utf-8")
        setup(old, "reject-unowned", f"/DIR={unowned}", success=False)
        assert [p.name for p in unowned.iterdir()] == ["keep.txt"]
        assert reg_value(uninstall_key, "InstallLocation") is None
        passed.append("an unrelated nonempty folder is not overwritten")

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
        for name in PAYLOAD:
            assert (installed / name).read_bytes() == (payload / name).read_bytes()
        assert all(task_xml(name) is None for name in task_names), "Setup registered a startup task"
        passed.append("per-user install, exact payload, shortcuts, fork links and no setup autostart")

        setup(new, "reject-relocation", f"/DIR={stage / 'other install'}", success=False)
        assert not (stage / "other install/pecofence.exe").exists()
        passed.append("upgrade refuses to move the installation directory")

        retained = {}
        for name in ("config/config.json", "data/logs/keep.log", "user-file.txt"):
            path = installed / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("user data retained", encoding="utf-8")
            retained[path] = path.read_bytes()
        # Prove installed program files really are replaced on upgrade.
        (installed / "pecofence-cli.exe").write_bytes(b"stale binary fixture")
        setup(new, "upgrade")
        assert Path(reg_value(uninstall_key, "InstallLocation")) == installed
        assert reg_value(uninstall_key, "DisplayVersion") == "0.0.2"
        assert (installed / "pecofence-cli.exe").read_bytes() == (payload / "pecofence-cli.exe").read_bytes()
        assert all(p.read_bytes() == data for p, data in retained.items())
        assert len(list(installed.glob("unins*.exe"))) == 1
        passed.append("upgrade reuses directory/uninstaller, replaces binaries and preserves added data")

        setup(old, "reject-downgrade", success=False)
        assert reg_value(uninstall_key, "DisplayVersion") == "0.0.2"
        passed.append("numeric version downgrade is blocked")

        # The in-app updater: setup waits for the exiting app, then starts it again; a failed
        # update starts the old copy instead. Test installers only log the start.
        def log_of(name):
            return (stage / f"{name}.log").read_text(encoding="utf-8-sig", errors="replace")

        start_line = f"would start {installed / 'pecofence.exe'}"
        assert "would start" not in log_of("upgrade"), "a plain silent upgrade started the app"
        exiting = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(3)"],
                                   startupinfo=HIDDEN)
        setup(new, "update", f"/UPDATE={exiting.pid}")
        assert exiting.poll() is not None, "setup did not wait for the app"
        waited = re.search(r"Update: waited (\d+) ms", log_of("update"))
        assert waited and int(waited.group(1)) >= 2000, "setup did not wait for the app"
        assert log_of("update").count("would start") == 1 and start_line in log_of("update")
        setup(old, "update-refused", "/UPDATE=0", success=False)
        assert log_of("update-refused").count("would start") == 1
        assert start_line in log_of("update-refused")
        passed.append("an update waits for the app, then restarts it once; a failed one restarts the old copy")

        other_command = f'"{stage / "another copy/pecofence.exe"}"'
        with winreg.CreateKey(winreg.HKEY_CURRENT_USER, RUN) as key:
            winreg.SetValueEx(key, run_names[0], 0, winreg.REG_SZ, f'"{installed / "pecofence.exe"}"')
            winreg.SetValueEx(key, run_names[1], 0, winreg.REG_SZ, other_command)
        create_task(task_names[0], installed / "pecofence.exe")
        create_task(task_names[1], installed / "pecofence.exe", '--no-hide-icons "--portable"')
        uninstall("uninstall")
        assert not any((installed / name).exists() for name in PAYLOAD)
        assert not desktop_link.exists() and not menu_group.exists()
        assert all(p.read_bytes() == data for p, data in retained.items())
        assert reg_value(RUN, run_names[0]) is None
        assert reg_value(RUN, run_names[1]) == other_command
        assert task_xml(task_names[0]) is None, "Uninstall kept the installed copy's startup task"
        assert task_xml(task_names[1]) is not None, "Uninstall removed a portable startup task"
        passed.append("uninstall removes managed files/shortcuts, keeps data and other startup owners")

        # A folder an older ZIP was extracted into (the default directory, for some users).
        extracted = stage / "extracted zip"
        extracted.mkdir()
        (extracted / "pecofence.exe").write_bytes(b"older ZIP build fixture")
        (extracted / "config").mkdir()
        (extracted / "config/config.json").write_text("portable config", encoding="utf-8")
        setup(new, "adopt-zip-folder", f"/DIR={extracted}")
        assert Path(reg_value(uninstall_key, "InstallLocation")) == extracted
        assert (extracted / "pecofence.exe").read_bytes() == (payload / "pecofence.exe").read_bytes()
        assert (extracted / "config/config.json").read_text(encoding="utf-8") == "portable config"
        passed.append("a folder holding an extracted ZIP is upgraded in place and keeps its files")

        # Deleting the folder by hand must not pin later installs to the stale registration.
        shutil.rmtree(extracted)
        relocated = stage / "new location"
        setup(new, "reinstall-after-deleted-folder", f"/DIR={relocated}")
        assert Path(reg_value(uninstall_key, "InstallLocation")) == relocated
        create_task(task_names[0], stage / "another copy/pecofence.exe")
        foreign_task = task_xml(task_names[0])
        assert foreign_task is not None
        create_task(task_names[1], relocated / "pecofence.exe",
                    extra_executable=stage / "another copy/pecofence.exe")
        mixed_task = task_xml(task_names[1])
        assert mixed_task is not None
        uninstall("uninstall-reinstalled")
        assert not (relocated / "pecofence.exe").exists()
        assert task_xml(task_names[0]) == foreign_task, "Uninstall changed another directory's task"
        assert task_xml(task_names[1]) == mixed_task, "Uninstall changed a task with mixed owners"
        passed.append("uninstall preserves a startup task owned by another directory")
        passed.append("uninstall preserves a task with multiple actions")
        passed.append("a registration whose folder was deleted does not block a new location")
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
            for name in task_names:
                delete_task(name)
            assert {name: reg_value(RUN, name) for name in actual_startup} == actual_startup
            assert {name: task_xml(name) for name in actual_tasks} == actual_tasks
            report.write_text(json.dumps({"passed": passed, "testIdentity": product_id}, indent=2), encoding="utf-8")
            print(f"{len(passed)} packaging/installer checks passed; report: {report}")


if __name__ == "__main__":
    main()
