# Windows startup and desktop handoff

## Registration and process priority

Desktop releases use a current-user Task Scheduler `LogonTrigger`, with no trigger
delay, `InteractiveToken`, `LeastPrivilege`, and priority `3` (Above Normal). This
does not require elevation. A registration failure retains startup through the Run
key. If both registrations fail, existing startup entries are preserved and the
error is logged. The task and fallback both preserve `--portable` when that mode was used.
Debug builds leave startup registrations alone; Store builds keep the manifest's
`windows.startupTask`. Reconciliation runs on a worker, serialized with explicit
settings changes so a queued startup enable cannot override a later disable.

Above Normal is a CPU scheduling priority. It cannot make Explorer, graphics
drivers or the compositor ready, or remove disk and CPU contention during logon.
The same program can therefore initialize much faster when launched manually
after Windows has settled.

The research also distinguishes a *service launch* from process priority:

- Coodesker's maintainer described [Task Scheduler startup](https://github.com/coodesker/coodesker-desktop/issues/74#issuecomment-929798317)
  and later [service startup with ordinary startup as a fallback](https://github.com/coodesker/coodesker-desktop/issues/261#issuecomment-1330163292).
  These comments do not expose its current implementation or prove its display
  handoff algorithm.
- Wallpaper Engine's [automatic startup documentation](https://help.wallpaperengine.io/en/functionality/automaticstartup.html)
  explicitly says its high-priority option registers a Windows service.
- Clash Verge Rev's [scheduler implementation](https://github.com/clash-verge-rev/clash-verge-rev/blob/4337bd85758d54d7b9044b2da3c9ac1176eae1f3/src-tauri/src/utils/schtasks.rs)
  uses priority `3`. This is the ordinary-user approach used here.
- Microsoft's [task-priority mapping](https://learn.microsoft.com/en-us/windows/win32/taskschd/tasksettings-priority)
  and [interactive-service guidance](https://learn.microsoft.com/en-us/windows/win32/services/interactive-services)
  explain why adding a service would be a separate elevated installation and
  user-session process-launch feature. This change does not install a service.

## First-frame handoff

`ShowWindow` and `show(false)` are not evidence that a fence has been painted:
the latter still starts a fade at zero opacity. The startup path instead:

1. Resolves the desktop anchor and snapshots the existing recovery marker.
2. Queues wallpaper discovery, disk cache lookup, decoding and contrast sampling on
   a worker **before** initializing the render stack. Both proceed concurrently.
3. Creates fence windows hidden. If the worker has already finished, windows are
   created with their real background; otherwise its result updates them hidden.
   The anchor, Peek and quick-hide paths cannot reveal unfinished startup windows.
4. Commits the real material at full root opacity while the HWNDs remain hidden,
   using `Compositor.RequestCommitAsync` and a worker-side DWM flush.
5. Shows prepared windows without a fade, commits again, flushes DWM and checks
   required windows are visible, not cloaked, and have no known device loss.
6. Requests icon hiding through the existing marker/watchdog/Explorer-retry path.

The taskbar appearing changes monitor *work areas*, not the full monitor rectangles
used by the wallpaper. These changes relayout fences without discarding a running
decode or flushing icon caches. Actual monitor topology, DPI or material changes
still invalidate the wallpaper generation and refresh asynchronously.

`cache/startup-wallpaper.bin` beside the configuration stores one prepared BGRA set
(at most 128 MiB, 32 monitors). Its versioned key includes file size/write-time,
wallpaper mapping, full monitor bounds, DPI and material recipe. Reads validate the
key, image dimensions, byte bounds and pixel checksum. Corrupt or stale entries are
cache misses; writes use a unique temporary file and same-directory replacement.
All cache I/O and full-image CPU sampling happen on the worker. GPU uploads stay on
the UI thread. The cache can be deleted safely and is rebuilt on the next start.

When the wallpaper is unreadable, Explorer stays usable and the incomplete fences
stay hidden. A one-second worker retry continues while the tray and Settings remain
responsive. A missing background never authorizes hiding desktop icons or showing
a black placeholder. `PECOFENCE_SOLID` remains an explicit diagnostic exception.

The asynchronous commit approach follows the API used by
[Avalonia's Windows Composition integration](https://github.com/AvaloniaUI/Avalonia/blob/master/src/Windows/Avalonia.Win32/WinRT/Composition/WinUiCompositorConnection.cs).
See [RequestCommitAsync](https://learn.microsoft.com/en-us/uwp/api/windows.ui.composition.compositor.requestcommitasync)
and [DwmFlush](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmflush).
Neither API is a screenshot or proof that another application does not occlude a
window. Together with window checks they provide a stronger handoff condition than
`ShowWindow` alone. A changed startup layout requests a fresh commit. Failed or
timed-out presentation is retried up to three times (five seconds per commit);
exhaustion leaves Explorer icons available and waits ten seconds before another
batch of attempts. The handoff state is retained so a transient compositor or GPU
failure does not permanently strand hidden fences. A failed redraw also prevents
takeover until a full redraw succeeds.

The recovery marker is sampled before this process can hide icons. Watchdog recovery
only removes it after restoration succeeds. If a previous shutdown already left
Explorer's persisted icon state hidden, that earlier hidden state can still be
visible during initialization; the phase logs distinguish it from a new hide.

## Measurement and real-logon validation

Normal logs are under `%LOCALAPPDATA%\PecoFence`: `pecofence.log` and the rotated
`pecofence.prev.log`. A named test instance has its instance name in the log filename.
The phase ordering should be (concurrent completion can interleave):

```text
startup wallpaper discovery and decode queued in background
startup phase: initializing render stack
render phase: dispatcher queue ready
render phase: compositor ready
render phase: graphics device ready
render phase: composition graphics device ready
startup phase: fence windows prepared
startup phase: ready to pump messages
startup wallpaper ready; fences updated
startup phase: committing wallpaper material before showing fences
startup phase: hidden material commit completed
startup phase: showing prepared fences without fade
startup phase: first fence frame presented
startup phase: requesting desktop icon hide after first frame
```

The hide entry is absent with `--no-hide-icons` or disabled hiding. Renderer,
desktop/portal enumeration, window construction and first-frame entries include
elapsed milliseconds. The first-frame duration starts at `App::create`, not at
Windows sign-in; logs cannot measure the time before the process was launched.
Wallpaper readiness may precede window creation when concurrent preparation is
fast enough. It must always precede the hidden material commit and first reveal.
`cache_hit`, `queue_ms`, wrapper time and each window's redraw time distinguish
decoding, UI queue delays and material upload/drawing in a slow cold logon.

For a real logon test:

1. Install the current test package, start it once and enable startup.
2. Check the `PecoFence` task command points to that installation; a portable-mode
   task must include `--portable`.
3. Sign out and back in or reboot manually, recording when the desktop and fences
   become visible. Retain the startup log before another launch rotates it.
4. Check wallpaper preparation was queued before rendering, hiding follows a
   completed frame with real material, and no presentation timeout/device-loss
   warning appears. Taskbar-only relayout must not queue another wallpaper decode.
5. Repeat with the usual monitor arrangement, then compare with a manual launch.
   Also check normal exit restores Explorer icons and disabling startup removes
   its task and Run fallback.

An isolated warm-start smoke test cannot establish the cold-logon duration or
guarantee the absence of a visible gap on another machine.

This change does **not** guarantee seamless Windows logon. Real cold-logon testing
still showed Explorer's original desktop before the prepared fences appeared.
The first-frame checks reduce incomplete/black-frame handoffs; they do not control
when Windows schedules the process or when the graphics stack becomes ready.
