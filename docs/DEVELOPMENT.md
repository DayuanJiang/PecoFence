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
`--no-hide-icons` and a unique `PECOFENCE_INSTANCE`. Keep autostart disabled in its
configuration. Portable startup leaves the Windows autostart entry alone.
`--exit-after <milliseconds>` closes a smoke-test instance automatically.

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
opens an isolated portable Settings window, changes all eleven languages through the
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
locations (`scripts/find-iscc.ps1`, which the packaging test also uses). `-Python` accepts a Python executable path, including one returned by
`uv python find 3.12`. Packaging does not install development tools locally.
CI uses the pinned URL/SHA-256 in `packaging/inno/toolchain.json`.

For a fork, pass `-Repository owner/repo` and pass the same value to the test with
`--repository owner/repo`. The default is `GITHUB_REPOSITORY`, then
`DayuanJiang/PecoFence` outside Actions. This sets the installer's links; it does
not change application identity. Fork setup
EXEs still target the same installed PecoFence product. Use the test script for
isolated validation instead of installing a fork package over a real installation.

The packaging test checks the ZIP and setup payloads, binary equality and checksums.
It then compiles the production `.iss` with a random **test-only** identity and two
synthetic versions, installs to `.cache/`, upgrades and uninstalls. It exercises
shortcuts, startup ownership, retained data, mutex blocking, destination/version
guards and upgrading a folder that holds an extracted ZIP, without starting PecoFence. Temporary test shortcuts/registry entries are
cleaned up; reports and logs remain under `.cache/installer-*/`. Never distribute
these synthetic test installers. Manually inspect the setup wizard and supported
Windows/DPI combinations before release.

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
