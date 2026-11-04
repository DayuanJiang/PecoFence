# Real-time Liquid Glass and Wallpaper Engine scene wallpapers

[简体中文](REALTIME_GLASS.zh-CN.md)

This branch adds an optional real-time GPU background path to PecoFence, allowing the existing Liquid Glass fences to continuously update their blur, color and refraction effects. The feature is disabled by default; the refresh limit and effect parameters can be adjusted in Settings.

Wallpaper Engine (WE) scene wallpapers have currently been verified on one Windows device. This document separates the implementation, completed validation and areas that still need testing, so others can use, reproduce and extend the work.

## The problem

The existing live-wallpaper support uses periodic still captures: wallpaper changes, returning to the desktop and other events trigger a refresh, with additional timed updates while visible. This is not a per-frame background, so successfully displaying a video wallpaper does not mean that the image inside a fence updates in real time.

Some WE scene or web renderers also lack the redirection bitmap required by the existing `PrintWindow` path. That path may return a black or stale image; PF uses `notCapturable` to indicate that the still-capture path is unavailable.

This implementation uses Windows Graphics Capture (WGC) to read the desktop host's composed image and feed its GPU texture into the existing glass renderer. It does not call private WE interfaces or parse or replay WE scene files.

## Where the image comes from

```text
WE scene rendering
    ↓
Windows desktop host's composed image (current target: Progman)
    ↓ Windows Graphics Capture, bounded frame pool, latest frame only
Persistent D3D11 texture owned by PF
    ↓ Direct2D downsampling, blur, saturation and a thin white veil
Background material shared by all fences
    ↓ Sample at each fence's screen coordinates; apply edge refraction and highlights
Fence presentation; icon content remains in a separate visual layer
```

Textures and materials for the same source are reused. Normal per-frame updates do not recreate the entire material, and multiple fences share one background update. The production real-time background path does not read background pixels back to the CPU, encode PNG/JPEG/video or pass images to PF through disk files.

Two WGC frame buffers feed one persistent texture on the existing D3D11 device. A D2D context on the same device processes the shared material once per update; each fence samples it in physical desktop coordinates and uses the existing GPU lens and rim effects. Grid icon surfaces remain cached; glass headers in list/detail views refresh as needed. Normal per-frame updates do not create new GPU devices.

Fences are separate top-level windows and are not part of the desktop host being captured. This path therefore does not capture the fences themselves or require hiding them through capture exclusion; fences remain visible in normal screenshots. No full-screen/monitor-capture fallback is enabled.

The source is the desktop host, **not a general filter for arbitrary foreground applications**. For example, floating a fence above another application does not mean that it will refract that application's contents. The actual source, window hierarchy and system composition behavior still affect the result.

The new capture path does not inject into WE or Explorer, change their parent/child window relationships or patch them. It does not restart Explorer, change WE playback settings or change the system-wide timer resolution. The refresh clock uses a process-local high-resolution one-shot waitable timer, keeps at most one pending UI refresh notification and does not call `timeBeginPeriod`; it falls back to an ordinary timer when unavailable. WGC capture borders and access policy remain controlled by the system's capabilities and permissions.

Main implementation entry points:

| Location | Responsibility |
| --- | --- |
| `crates/render/src/live_capture.rs` | WGC sessions, bounded frame pool, persistent GPU texture, source closure and size changes |
| `crates/render/src/realtime_material.rs` | Downsampling, blur and color processing for the shared background |
| `crates/render/src/gpu_glass.rs` | Fence/control lenses, input invalidation and material reuse |
| `crates/app/src/app/realtime_glass.rs` | Refresh scheduling, occlusion pause, retry, device recovery and runtime diagnostics |
| `crates/platform/src/realtime_clock.rs` | Process-local refresh clock, missed-tick dropping and bounded pending notifications |
| `crates/core/src/model.rs`, `ui/settings.html` | Parameter model, validation and Settings UI |

## Configuration and use

In an experimental build, select Liquid Glass under General, then enable Real-time Liquid Glass (experimental). Existing configurations without the new fields keep the feature disabled. Disabling it returns to the original background path.

The keys below belong to the `settings.realtimeGlass` object in `config.json`; use the paths in the last column with the CLI.

| Field | Range or options | Default | CLI path |
| --- | --- | --- | --- |
| `enabled` | `true` / `false` | `false` | `realtimeGlass.enabled` |
| `fps` | Integer, 5–60 | `30` | `realtimeGlass.fps` |
| `quality` | `performance` / `balanced` / `quality` | `balanced` | `realtimeGlass.quality` |
| `blur` | 0–30, in physical pixels of the original image | `8` | `realtimeGlass.blur` |
| `refraction` | 0–2; 0 disables refraction | `1` | `realtimeGlass.refraction` |
| `saturation` | 0–2; 0 is grayscale, 1 preserves the original colors | `1.05` | `realtimeGlass.saturation` |
| `tint` | 0–0.5, strength of a thin white veil | `0.08` | `realtimeGlass.tint` |
| `pauseWhenCovered` | `true` / `false` | `true` | `realtimeGlass.pauseWhenCovered` |

Floating-point parameters must be finite. `blur` uses physical pixels and does not increase with Windows DIP scaling. `tint` is a white veil, not an automatically selected light or dark background tint.

Sampling quality sets the width and height of the intermediate background-processing texture: each dimension is 1/4 of the original in Performance mode, 1/2 in Balanced mode and unchanged in Quality mode. Lower settings reduce some material-processing and texture costs; **the native WGC frame pool and input copy still use the original resolution**. FPS is a refresh limit; actual speed also depends on wallpaper playback, system scheduling and the GPU.

After confirming that the intended PF instance is running, use the CLI supplied with the build to change its settings. For example, from the repository root:

```powershell
.\target\release\pecofence-cli.exe settings get realtimeGlass
.\target\release\pecofence-cli.exe settings set themeStyle liquidGlass
.\target\release\pecofence-cli.exe settings set realtimeGlass.fps 30
.\target\release\pecofence-cli.exe settings set realtimeGlass.quality balanced
.\target\release\pecofence-cli.exe settings set realtimeGlass.enabled true
```

To disable the real-time feature:

```powershell
.\target\release\pecofence-cli.exe settings set realtimeGlass.enabled false
```

With `pauseWhenCovered` enabled, PF checks windows actually positioned above each fence. When no fence needs an update, the capture session and high-frequency refresh clock stop; they are recreated when fences become visible again. Window-order checks occur approximately every 250 milliseconds, so pause and resume are not instantaneous. The current check asks whether a single window above a fence covers it completely; combined coverage by several windows is not equivalent to a verified, complete occlusion analysis.

Source closure, an initial-frame timeout, capture errors or GPU errors lead to retries or the original background path. Retry intervals are bounded rather than spinning indefinitely; device loss uses the existing GPU recovery path. An `active` state and increasing frame count indicate that capture and updates are running; those values alone do not prove every frame is correct or guarantee automatic detection of all black output.

## Build and tests

Run the following commands from the repository root. Prerequisites match upstream: the Windows/MSVC toolchain, C++ Build Tools, Windows SDK and WebView2 Runtime for Settings. The current `Cargo.toml` requires Rust 1.95 or later; this validation used Rust 1.95.0. Python is used for resource checks and Node.js for Settings UI tests. See the [development guide](docs/DEVELOPMENT.md) for the complete setup.

```powershell
cargo build --locked --release -p pecofence -p pecofence-watchdog
cargo build --locked --release -p pecofence-cli
Copy-Item -LiteralPath third_party\webview2\WebView2Loader.x64.dll -Destination target\release\WebView2Loader.dll
```

`WebView2Loader.dll` must be next to the executable even when Settings is not opened. Retain the watchdog, licenses and third-party notices when distributing or trying the build. Exit the other desktop-management instance normally and back up its configuration before trying a portable build in a separate directory. To reuse a configuration, copy an exported configuration to that separate directory's `config/config.json` while the target instance is stopped. Retain normal single-instance protection: only one PF instance should manage this desktop at a time.

The example below starts from the build output directory. If the files have been copied to a separate portable directory, run the executable in that directory instead:

```powershell
.\target\release\pecofence.exe --portable
```

Portable configuration is stored in the `config` directory beside the executable. Set `PECOFENCE_RUNTIME_DIR` to an absolute directory to isolate this build's logs and WebView2 runtime data without overriding `LOCALAPPDATA`; it does not change the portable configuration directory. The runtime `status.runtimeLogDir` field is used by CLI `paths` / `log`. Named instances can be used for isolated testing as described in the development guide, but an instance name alone does not create a separate configuration directory for `--portable`.

Rust checks relevant to this feature:

```powershell
cargo fmt --all --check
cargo test --locked -p pecofence-core --features schema
cargo test --locked -p pecofence-render --lib
cargo test --locked -p pecofence --bin pecofence
cargo test --locked -p pecofence-cli --bin pecofence-cli
cargo test --locked -p pecofence-ipc
```

Before contributing, also complete the repository checks required by [CONTRIBUTING.md](CONTRIBUTING.md), including:

```powershell
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/check-locales.py
python scripts/check-readme-translations.py
node scripts/test-settings-ui.mjs
```

The Settings UI script serves a local test page; open it and inspect the results. Starting the server alone does not mean that the browser tests have passed. Some Windows platform tests call real system APIs and require a desktop session.

Regenerate the production Windows bindings with `cargo run -p tool_bindgen`. This branch includes WGC bindings in the default generation list and the CI consistency check. Regenerate the optional native-backdrop probe separately with `cargo run -p tool_bindgen -- tools/bindgen/host-backdrop.txt`.

### Validation of the initial source migration

The following checks were rerun in the target workspace on 2026-11-04 with Rust 1.95.0:

- `cargo fmt --all -- --check` and strict Clippy across the entire workspace and all targets passed.
- `cargo test --workspace --locked`: **524 passed, 1 existing test ignored**.
- Browser Settings UI regressions: **80 passed, 0 failed**, using a mock bridge that does not change the running PF instance's settings.
- Coverage checks passed for 616 messages across 11 languages and for 11 README translations.
- All 5 production Windows binding files regenerated identically to the migrated files.

To satisfy strict checks, two lints concerning system type names/generated projections are allowed locally on generated-binding modules. An existing Boolean expression in the temporary-file rule was also simplified equivalently without changing the rule's behavior. These checks are recorded separately from the earlier experimental build's visual and performance measurements below.

### Validation after merging upstream 0.2.5

On 2026-11-04, upstream commit `ef888bf878f7cb634971123eae1dff931e537b30` was merged into this branch and the following checks were rerun with Rust 1.95.0:

- `cargo fmt --all -- --check` and strict Clippy across the entire workspace and all targets passed.
- `cargo test --workspace --locked`: **529 passed, 1 existing test ignored**.
- Browser Settings UI regressions: **95 passed, 0 failed**, using the mock bridge.
- Coverage checks passed for 633 messages across 11 languages and for 11 README translations.
- All 5 production Windows binding files regenerated identically to the merged files.

The merged desktop application was not launched for new visual or performance measurements. The earlier measurements below still describe the previous experimental build.

## Bounded diagnostics

An independent probe can check whether the desktop host image keeps changing without starting another PF desktop-management instance:

```powershell
cargo build --locked --release -p pecofence-render --example realtime_glass_probe
.\target\release\examples\realtime_glass_probe.exe --exit-after 20
```

The probe creates a small visible preview, defaults to a 30 FPS limit and Balanced quality, exits at the time limit and writes a JSON summary to standard output. Its `--exit-after` argument is in **seconds**, accepts 1–300 and defaults to 20; the main application's argument of the same name uses milliseconds. The probe also exits after three cumulative errors.

The probe supports `--source-hwnd` and `--inspectable`. Keep the default desktop-host source for normal testing. It does not change desktop icons, wallpaper or PF configuration, and does not save screenshots or videos.

To distinguish increasing frame counts from actual pixel changes, **only the probe** reads nine 16×16 pixel regions once per second, about 9 KiB, and calculates non-black samples, distinct hashes and adjacent-sample changes. The probe's JSON `cpuReadback: true` refers to these diagnostic samples; the production real-time path still has no CPU pixel readback. Changes in small sampled regions do not prove that the entire image is correct or free of visual artifacts.

The source also retains `crates/render/examples/host_backdrop_probe.rs` to explore a native-backdrop material alternative, and `crates/render/examples/occlusion_probe.rs` to create an opaque occlusion-test window that closes automatically after 8 seconds. These are independent diagnostic tools, not the main application's everyday capture path.

Inspect refresh state and log locations in a running PF instance with:

```powershell
.\target\release\pecofence-cli.exe status --fields realtimeGlass,runtimeLogDir
.\target\release\pecofence-cli.exe paths
```

`realtimeGlass` includes `state`, `frames`, `paneDraws`, `averageFps`, `lastFrameAgeMs`, `maxTickMs`, `error` and `cpuReadback`. Counters restart when a session is recreated; the average FPS shown while paused is not a useful running-performance measure. `maxTickMs` includes UI ticks spent initializing sessions and handling failures; it is neither GPU per-frame time nor end-to-end latency.

## Earlier short local measurements

The following figures come from **the earlier experimental build's records on 2026-11-04**: Windows build 26300, a single 2560×1440 SDR display, Ryzen 7 9700X (16 logical processors), RTX 5070 and the WE scene wallpaper running at the time. These are short local examples, not cross-device performance promises or new performance measurements after this source migration.

| Short-test scenario | Sample seconds | Session-average FPS | PF CPU | Working set MiB | Dedicated GPU memory MiB | Mean sampled PF 3D engine utilization |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Original still-background path | 11.2 | — | 0.00% | 15.3 | 116.6 | 0.00% |
| 15 FPS / Performance, continuous-update stress test | 11.4 | 14.9 | 0.22% | 105.0 | 204.3 | 0.00% |
| 30 FPS / Balanced, continuous-update stress test | 21.2 | 29.8 | 0.30% | 104.8 | 184.9 | 1.00% |
| 60 FPS / Quality, continuous-update stress test | 11.1 | 59.3 | 0.53% | 105.2 | 233.0 | 1.67% |
| 30 FPS / Balanced, another run after confirming WE playback | 21.1 | 28.7 | 0.35% | 110.8 | 353.4 | 1.00% |
| Final build, 30 FPS / Balanced, default occlusion-pause policy | 11.1 | 29.3 | 0.23% | 102.8 | 169.7 | 0.00% |

Measurement definitions and limitations:

- CPU is the PF main process's CPU-time delta divided by wall-clock time and the logical processor count. The table excludes additional WE, DWM and WebView2 child-process costs.
- GPU 3D is the mean, across three samples, of the busiest 3D engine instance belonging to this process. Different engines are not added together. This is not whole-machine GPU load or power consumption; a short 0% reading does not mean zero cost.
- FPS is the capture session's cumulative average at the end of sampling, including initialization. It is not calculated from the frame-count delta within that sampling interval and does not measure content-to-screen latency.
- Stress tests disabled occlusion pause; the final run restored the default policy and had a different visible update area. Quality changes, session recreation and the Settings window also affect driver/composition caches, so the rows cannot establish fixed GPU-memory budgets for individual quality levels.
- PF's mean dedicated GPU memory across these runs was about 170–353 MiB, versus about 117 MiB for the original build. Windows memory reclamation affects the working set. Wall-outlet power, whole-machine energy use and long-duration GPU-memory trends were not measured.

Earlier validation also included an independent dynamic probe running for 60 seconds: 1,784 frames at about 29.7 FPS, 59 non-black background samples producing 59 distinct hashes, changes in all 58 adjacent comparisons and 0 errors. Short full-occlusion pause/resume checks, four enable/disable cycles and normal exit/rollback to the original build were also performed. No monotonic handle/thread growth was observed in those short checks; they do not replace long-duration stability testing.

The earlier experimental build passed Core 91, Render 72, App 235, CLI 49 and IPC 20 tests, totaling **467**, with one existing App test ignored. The 467 count belongs only to that earlier build; see the validation sections above for the complete tests rerun during source migration and after merging upstream 0.2.5. The earlier experimental release build also passed.

## Compatibility limits and further validation

Actual visual and performance validation currently covers only the WE scene wallpaper in the single-display SDR environment above. The original build's ability to capture video wallpapers does not establish this branch's real-time compatibility with video, web or other live-wallpaper software.

Multiple displays, mixed DPI, HDR, other Windows builds, other GPUs/drivers, WE web/video wallpapers, sleep/resume and long playback sessions still need separate reproduction. Having size-change and device-recovery code does not mean that all of those scenarios have been tested. The current real-time texture uses 8-bit BGRA and does not claim to preserve HDR highlights.

The normal real-time GPU path is inactive when WGC is unavailable, software rasterization is in use or the current material is not Liquid Glass. Some older Windows versions lack the optional capture-producer frame-rate limit property; the consumer refresh limit still applies there. WE's own pause/stop rules remain effective, and PF does not force it to keep rendering. The shared desktop host's extent and composition behavior vary by system and may include the desktop-icon layer or other content; verify that the source matches expectations in each environment.

A reproduction report should include the Windows build, monitor topology/scaling/HDR, GPU/driver, wallpaper type, refresh/quality settings, occlusion conditions and bounded-probe/status summaries. Check personal paths, window identifiers and log content before submitting. User configurations, screenshots, raw local evidence and machine-specific deployment scripts are not included in the public source.

## Origin, version and license

The upstream project is [DayuanJiang/PecoFence](https://github.com/DayuanJiang/PecoFence). This experiment initially used upstream commit `40b2acf6dcc349b477b426862acdf78900a79768`, whose base version is `0.2.4`, with the experimental package version `0.2.4-liveglass.1`. It now incorporates upstream `0.2.5` at commit `ef888bf878f7cb634971123eae1dff931e537b30`. The package version in `Cargo.toml` follows upstream at `0.2.5`; real-time glass remains an optional addition on this feature branch, not a claim that upstream has released it.

The migration retains the full upstream project, existing functionality and `vendor` dependencies, adding the real-time glass implementation and settings on that base. The license remains [Apache License 2.0](LICENSE). Retain the original author's attribution, third-party notices and each vendored dependency's license files; see the [third-party dependency notes](third_party/README.md).
