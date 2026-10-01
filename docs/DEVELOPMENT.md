# Development

## Prerequisites

- Windows 11 x64.
- Rust stable (MSVC toolchain); the workspace's minimum Rust version is in `Cargo.toml`.
- Visual Studio Build Tools with the Desktop development with C++ workload and Windows SDK.
- Microsoft Edge WebView2 Runtime to use Settings.
- Python 3 for catalog checks/source packaging; Node.js for the settings browser tests.

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

## Architecture

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
