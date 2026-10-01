# Development

## Prerequisites

- Windows 11 x64.
- Rust stable (MSVC toolchain); the workspace's minimum Rust version is in `Cargo.toml`.
- Visual Studio Build Tools with the Desktop development with C++ workload and Windows SDK.
- Microsoft Edge WebView2 Runtime to use Settings.
- Python 3 for catalog checks/source packaging; Node.js for the settings browser tests.
- Inno Setup 7 for the setup EXE; packaging tests use Python 3.11+ (CI uses 3.12).

## Build and run

```powershell
cargo build --locked
Copy-Item third_party/webview2/WebView2Loader.x64.dll target/debug/WebView2Loader.dll
./target/debug/pecofence.exe
```

The WebView2 loader is imported at process startup and must be next to the executable,
even if Settings is not opened. The watchdog should also be packaged beside the app.

For an isolated test instance, use a separate directory, `--portable`,
`--no-hide-icons` and a unique `PECOFENCE_INSTANCE`.
`--exit-after <milliseconds>` closes a smoke-test instance automatically.

### Distribution data paths

The app and CLI read `deployment.json` beside their executables, independently of
the working directory. Both distributions use the same binaries. To test automatic
portable detection, place this UTF-8 file beside a scratch copy of the binaries:

```json
{"schema":1,"appId":"PecoFence","mode":"portable"}
```

`mode: "installed"` selects the existing profile-directory layout. Without a
marker, source builds and existing ZIPs retain their previous behavior; the GUI's
`--portable` flag remains supported. An invalid marker, an installed marker with
`--portable`, or portable mode inside MSIX is an error. MSIX identity otherwise
keeps the existing Store behavior.

| Data | Portable (relative to the executable) | Installed / unmarked / MSIX |
|---|---|---|
| Config and backups | `config/` | `%APPDATA%/PecoFence/` |
| Log | `data/logs/pecofence.log` | `%LOCALAPPDATA%/PecoFence/pecofence.log` |
| Crash dumps | `data/crashes/` | `%LOCALAPPDATA%/PecoFence/` |
| WebView2 data | `data/WebView2Profiles/default/` | `%LOCALAPPDATA%/PecoFence/WebView2Profiles/default/` |
| Desktop recovery marker | `data/recovery/icons-hidden.marker` | `%LOCALAPPDATA%/PecoFence/icons-hidden.marker` |
| Update work files | `data/updates/` | `%LOCALAPPDATA%/PecoFence/updates/` |

Named instances have separate log/marker names and WebView2 profiles, but share
config within a copy. Non-portable modes may reuse pre-rename `OpenFence` data;
portable mode never does. Required directories and existing config/log files must
be writable. Startup failures show the selected path and cause, without falling
back to AppData or Temp. Close the app and its helper/browser processes before
moving a portable folder. Desktop items, portal targets and user-chosen exports
continue to refer to their original external locations.

Portable mode disables the Settings autostart control and rejects autostart
changes through IPC or config import without touching either Run-key entry.
Installed and portable copies retain the shared single-instance mutex to prevent
two copies from managing the desktop simultaneously. A CLI with a deployment
marker verifies the connected server's executable directory before sending any
request; use the CLI beside the running copy.

`python scripts/test-distribution-smoke.py` exercises automatic detection,
local paths, relocation, cross-copy CLI rejection, autostart isolation and blocked
startup paths using disposable copies under `.cache/` and fake profile directories.
Build the debug binaries first. This test does not hide the real desktop icons.

## Verification

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/check-locales.py
python scripts/check-readme-translations.py
node scripts/test-settings-ui.mjs
```

Open the address printed by the last command. The browser suite uses a mock host
bridge and does not change the running desktop application's settings. It covers
language switching, draft/user data preservation and narrow-window layout.

Some platform tests use real Windows APIs and a desktop session. They are not a
substitute for testing native menus, multiple monitors and supported Windows builds.

After building the package binaries, `python scripts/test-language-smoke.py`
opens an isolated portable Settings window, changes all ten languages through the
real IPC handler, and checks saved preferences and preservation of names/autostart.
It keeps its generated configuration and report under `.cache/`.

`powershell -File scripts/cli-smoke.ps1` starts an isolated `PECOFENCE_INSTANCE=clitest`
instance from a scratch copy of the debug binaries and drives it with `pecofence-cli`
(create, move, resize, set options, settings, rules, snapshots, delete), asserting the JSON
replies and exit codes. It never touches the real configuration or desktop icons.

## Windows packaging

```powershell
./scripts/make-windows.ps1
python scripts/test-windows-packaging.py
```

This builds the app/watchdog and CLI in separate Cargo invocations, then packages
identical binaries as a portable ZIP and an Inno Setup installer. Reuse a current
release build with `-SkipBuild -TargetDir target/package`. `-Format Portable` avoids
requiring Inno Setup; `scripts/make-portable.ps1` remains a compatibility entry point.
`-Format Installer` builds only setup. No MSIX staging directory is reused.

The compiler is found through `-Iscc`, `ISCC`, PATH or common Inno Setup 7 install
locations. `-Python` accepts a Python executable path, including one returned by
`uv python find 3.12`. Packaging does not install development tools locally.
CI uses the pinned URL/SHA-256 in `packaging/inno/toolchain.json`.

For a fork, pass `-Repository owner/repo` and pass the same value to the test with
`--repository owner/repo`. The default is `GITHUB_REPOSITORY`, then
`DayuanJiang/PecoFence` outside Actions. This sets installer links and package
provenance and the manual updater's repository; it does not change application identity. Fork setup
EXEs still target the same installed PecoFence product. Use the test script for
isolated validation instead of installing a fork package over a real installation.

The packaging test checks ZIP contents, markers, binary equality and checksums.
It then compiles the production `.iss` with a random **test-only** identity and two
synthetic versions, installs to `.cache/`, upgrades and uninstalls. It exercises
shortcuts, startup ownership, retained data, mutex blocking and destination/version
guards without starting PecoFence. Temporary test shortcuts/registry entries are
cleaned up; reports and logs remain under `.cache/installer-*/`. Never distribute
these synthetic test installers. Manually inspect the setup wizard and supported
Windows/DPI combinations before release.

## Architecture

### Manual updater

`core::updates` validates release identity, stable version ordering and exact asset
names/URLs. `app::updates` owns the asynchronous Settings state and polls a child
worker without blocking the UI. `platform::updates` starts Windows PowerShell 5.1
from its system location with no visible console and the embedded
`scripts/runtime/update-worker.ps1`. Arguments and the saved plan are data, never
interpolated executable expressions. The package payload is unchanged.

The worker bounds HTTPS responses and redirects, verifies size/SHA-256, validates
the portable ZIP allowlist and rejects links/junctions before writes. Installation
requires a separate confirmed action. A handoff captures the exact parent process;
the worker waits for normal exit, and portable replacements hold the normal and
legacy instance mutexes. Each original program file is backed up and flushed before
a durable `applying` journal authorizes replacement. Recovery validates all backups
before restoring, is idempotent, and never falls back to AppData for portable data.
Installer recovery re-runs the retained verified setup through its normal guards.

After building the release app/watchdog/CLI, run:

```powershell
powershell -NoProfile -File scripts/test-updates.ps1 -TargetDir target/package
```

Fixtures stay in `.cache/update-tests-*/`; only test-created child processes are
terminated. Tests cover checksum/size errors, ZIP traversal and mode/source mixups,
retained user data, failed replacements, corrupted recovery backups, and abrupt
termination during backup, journaling, replacement and rollback. Every abrupt-stop
case recovers in a fresh process. Setup launch is replaced by a fixture, so the
tests do not install over a real copy or modify its registry. The existing installer
integration suite separately checks real Inno Setup behavior. Actual power loss and
hardware/filesystem failure are outside this simulation.

Settings browser tests also cover manual checks, busy states, verification, explicit
installation confirmation, cancellation, Store restrictions and both recovery modes.
Do not publish fake releases to test these paths; the worker tests inject transport
and process boundaries only in their separate harness, without production bypasses.

| Directory | Responsibility |
|---|---|
| `crates/core` | Platform-independent models, configuration, rules, geometry and translation |
| `crates/platform` | Win32, shell, desktop integration and display-language detection |
| `crates/render` | Direct2D/Composition drawing, motion and glass |
| `crates/app` | Native windows, input, application state and Settings IPC |
| `crates/watchdog` | Restores desktop icons after an abnormal app exit |
| `crates/ipc` | Wire protocol shared by the app and the CLI: methods, DTOs, selectors, JSON Schema export |
| `crates/cli` | `pecofence-cli.exe`: console front end that drives a running instance over a named pipe (see [CLI.md](CLI.md)) |
| `ui` | Offline settings HTML and localization helper embedded into the executable |
| `locales` | Shared native and settings messages |
| `site` | Static product website, built by `scripts/build-site.py` (see [WEBSITE.md](WEBSITE.md)) |

Regenerate Win32 bindings with `cargo run -p tool_bindgen`. The definitions in
`tools/bindgen` generate platform, Composition and GPU-glass bindings.

`vendor/windows-composition` carries the small upstream wrapper patch needed by
the renderer, and `vendor/windows-canvas` adds `TextFormat::with_locale` so fence
text picks the right CJK font (upstream hard-codes `en-us`). Preserve their license
files when changing or distributing them.
