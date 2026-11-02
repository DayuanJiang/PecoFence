//! Live wallpapers (Wallpaper Engine, Lively, ...) hide the Windows wallpaper behind their
//! player window, so the backdrops show captures of that window instead (feedback #24). A
//! capture is a still picture: Liquid Glass frosts it (see `MonitorRecipe::show_live`), and it
//! is taken again for a new window, after a wallpaper signal, when the desktop comes to the
//! front, and once a minute while a fence on that monitor can be seen.

use super::visuals::{MonitorRecipe, backdrop_signature, monitor_recipes, wallpaper_snapshot};
use super::*;
use crate::commands::WM_APP_LIVE_WALLPAPER;
use pecofence_platform::live_wallpaper;

/// A visible fence's backdrop catches up with the live wallpaper this often.
pub(super) const LIVE_REFRESH: Duration = Duration::from_secs(60);
/// The desktop came to the front: no new capture if the last one is this recent.
pub(super) const LIVE_FRONT_REFRESH: Duration = Duration::from_secs(2);
/// A new window (every window after a wallpaper signal) is captured again at every check (at
/// most every `LIVE_SETTLE_REFRESH`) for this long: a player that just started can show a black
/// or half-loaded picture for seconds.
const LIVE_SETTLE: Duration = Duration::from_secs(15);
const LIVE_SETTLE_REFRESH: Duration = Duration::from_secs(3);
/// Switching to another wallpaper, a player can close a moment before the next one shows: a
/// monitor goes back to the Windows wallpaper once its window is missing this long.
const LIVE_GONE_AFTER: Duration = Duration::from_secs(2);
/// A capture whose result has not come back after this is given up (see
/// `capture_live_wallpapers`).
const LIVE_CAPTURE_GIVE_UP: Duration = Duration::from_secs(30);
/// Captures are kept this many times smaller than the monitor: Fluent builds its backdrop at
/// that size, and frosted glass shows no more detail.
const CAPTURE_DOWNSCALE: u32 = 4;
/// Wallpaper Engine's clone mode shows one monitor's wallpaper on the others in windows of this
/// class, through a DWM thumbnail of the window it copies (feedback #55).
const CLONE_CLASS: &str = "WPECloneView";
/// A settled window whose captures came back empty this many times in a row draws nothing
/// PrintWindow can read.
const BLIND_AFTER: u32 = 3;

impl LiveWallpaper {
    /// The class of the window that shows it (`WPEDesktopDX11Window`), for feedback reports.
    pub(super) fn class(&self) -> String {
        desktop::class_name(HWND(self.hwnd as *mut core::ffi::c_void))
    }
}

/// The live wallpaper covering one monitor.
#[derive(Clone)]
pub(super) struct LiveWallpaper {
    /// The monitor as the wallpaper snapshot lists it (virtual-screen pixels).
    pub(super) monitor: RECT,
    hwnd: isize,
    /// When `hwnd` first covered the monitor, or a wallpaper signal came: it settles from then
    /// (see `LIVE_SETTLE`).
    since: Instant,
    /// Since when no window covers the monitor.
    missing: Option<Instant>,
    /// The last good capture of the monitor; until one works its backdrop shows the Windows
    /// wallpaper. A new window keeps the previous window's capture until its own.
    pub(super) image: Option<Arc<Image>>,
    /// The window and time of the last capture attempt.
    attempted: Option<(isize, Instant)>,
    /// The window whose empty capture was logged (once per window).
    reported: Option<isize>,
    /// Empty captures of `hwnd` in a row once it settled.
    empty: u32,
    /// `hwnd` cannot be captured (no redirection bitmap, or `BLIND_AFTER` empty captures): the
    /// backdrop shows a neutral colour, not an old capture or the Windows wallpaper under it.
    pub(super) blind: bool,
}

/// What the worker hands back: per monitor the recipe (with its new capture), the window when
/// that capture worked, and the backdrop image built from it; then the windows whose capture
/// came back empty, why, and whether the window a clone copies stood in.
type Built = (
    Vec<(MonitorRecipe, Option<isize>, Image)>,
    Vec<(isize, &'static str, bool)>,
);

/// Captures running on a worker thread, with every monitor's backdrop built from them there:
/// the Windows wallpaper of a monitor without a live one is decoded off the UI thread too.
pub(super) struct LiveCapture {
    /// The backdrops shown and `BackdropCache::generation` when it started: a rebuild since
    /// (theme, wallpaper, monitors) makes its set stale.
    shown: Option<String>,
    generation: u64,
    snapshot: wallpaper::WallpaperSnapshot,
    file_signature: String,
    started: Instant,
    result: Arc<Mutex<Option<std::result::Result<Built, String>>>>,
    thread: std::thread::JoinHandle<()>,
}

/// The window's rectangle and opaque pixels, `CAPTURE_DOWNSCALE` times smaller (see
/// `live_wallpaper::capture`), or why there are none.
fn capture(hwnd: HWND) -> std::result::Result<(RECT, Image), &'static str> {
    let (rect, mut image) = live_wallpaper::capture(hwnd, |bgra, width, height| {
        Image::downscale_pixels(bgra, width, height, CAPTURE_DOWNSCALE)
    })?;
    let mut drawn = false;
    for px in image.bgra.as_chunks_mut::<4>().0 {
        drawn |= px[..3] != [0, 0, 0];
        px[3] = 255;
    }
    if drawn {
        Ok((rect, image))
    } else {
        Err("all black")
    }
}

/// The window a Wallpaper Engine clone copies: a live wallpaper window of the same program that
/// is no clone itself. A DWM thumbnail in a window inside the desktop can be left out of its
/// PrintWindow, so the clone itself comes back black.
fn clone_source(clone: HWND) -> Option<HWND> {
    if desktop::class_name(clone) != CLONE_CLASS {
        return None;
    }
    let pid = desktop::window_pid(clone);
    live_wallpaper::windows()
        .into_iter()
        .map(|(hwnd, _)| hwnd)
        .find(|&h| desktop::window_pid(h) == pid && desktop::class_name(h) != CLONE_CLASS)
}

/// `monitor`'s part of a capture of the window at `window`.
fn monitor_part(window: RECT, shot: &Image, monitor: RECT) -> Image {
    let d = CAPTURE_DOWNSCALE as i32;
    shot.crop(
        (monitor.left - window.left) / d,
        (monitor.top - window.top) / d,
        (monitor.right - monitor.left + d - 1) / d,
        (monitor.bottom - monitor.top + d - 1) / d,
    )
}

/// On the worker: captures each monitor's target window (once per window, which may span
/// monitors) and builds every monitor's backdrop image. A capture that failed or came back
/// black leaves the recipe's previous capture, except for a Wallpaper Engine clone: the whole
/// picture of the window it copies stands in, stretched over its monitor as the clone shows it.
fn capture_and_build(work: Vec<(MonitorRecipe, Option<isize>)>) -> Result<Built> {
    // Per window: its rectangle (none for a clone's stand-in) and picture.
    let mut shots: Vec<(isize, Option<(Option<RECT>, Image)>)> = Vec::new();
    let mut empty = Vec::new();
    let mut built = Vec::new();
    for (mut recipe, target) in work {
        let mut captured = None;
        if let Some(hwnd) = target {
            if !shots.iter().any(|(h, _)| *h == hwnd) {
                let window = HWND(hwnd as *mut core::ffi::c_void);
                let shot = match capture(window) {
                    Ok((rect, image)) => Some((Some(rect), image)),
                    Err(reason) => {
                        let copied = clone_source(window).map(capture);
                        // A clone of a window PrintWindow cannot read is as blind as it.
                        let reason = match copied {
                            Some(Err(live_wallpaper::NOT_REDIRECTED)) => {
                                live_wallpaper::NOT_REDIRECTED
                            }
                            _ => reason,
                        };
                        let copied = copied.and_then(|copied| copied.ok());
                        empty.push((hwnd, reason, copied.is_some()));
                        copied.map(|(_, image)| (None, image))
                    }
                };
                shots.push((hwnd, shot));
            }
            let shot = shots.iter().find(|(h, _)| *h == hwnd);
            if let Some((_, Some((window, image)))) = shot {
                let part = match window {
                    Some(window) => monitor_part(*window, image, recipe.rect()),
                    None => image.clone(),
                };
                recipe.show_live(Arc::new(part));
                captured = Some(hwnd);
            }
        }
        let image = recipe.build_image()?;
        built.push((recipe, captured, image));
    }
    Ok((built, empty))
}

/// The foreground window's rectangle when it belongs to another program than Explorer and this
/// one. The fences sit under other programs' windows: it hides the fences inside it, and a
/// maximized or full-screen window every fence on its monitor.
fn foreground_rect() -> Option<RECT> {
    let root = desktop::root_ancestor(desktop::foreground_window());
    let pid = desktop::window_pid(root);
    let other = !root.0.is_null()
        && pid != std::process::id()
        && pid != desktop::window_pid(desktop::shell_window());
    other.then(|| window::window_rect(root))
}

/// What `monitor`'s backdrops make of a live wallpaper, for `status`'s `liveWallpapers` (see
/// `LiveWallpaperDto`).
fn live_wallpaper_state(live: &[LiveWallpaper], monitor: RECT) -> &'static str {
    match live.iter().find(|l| l.monitor == monitor) {
        None => "none",
        Some(l) if l.image.is_some() => "captured",
        Some(l) if l.blind => "notCapturable",
        Some(_) => "pending",
    }
}

impl App {
    /// `status.get`'s `liveWallpapers`: one entry per connected monitor.
    pub(super) fn live_wallpaper_dtos(&self) -> Vec<pecofence_ipc::LiveWallpaperDto> {
        self.state
            .work_areas
            .iter()
            .map(|w| {
                let monitor = RECT {
                    left: w.mon_left,
                    top: w.mon_top,
                    right: w.mon_right,
                    bottom: w.mon_bottom,
                };
                pecofence_ipc::LiveWallpaperDto {
                    monitor: w.device_path.clone(),
                    state: live_wallpaper_state(&self.live, monitor).to_string(),
                }
            })
            .collect()
    }

    /// Follows which monitors a live wallpaper covers (`live_wallpaper::windows`, ~30 us). A
    /// monitor without one goes back to the Windows wallpaper.
    pub(super) fn detect_live_wallpapers(&mut self, snapshot: &wallpaper::WallpaperSnapshot) {
        let windows = live_wallpaper::windows();
        let mut gone = std::mem::take(&mut self.live);
        for m in &snapshot.monitors {
            let known = gone.iter().position(|l| l.monitor == m.rect);
            let known = known.map(|i| gone.swap_remove(i));
            match (live_wallpaper::covering(&windows, m.rect), known) {
                (Some((hwnd, _)), known) => {
                    let entry = known.unwrap_or_else(|| {
                        let class = desktop::class_name(hwnd);
                        tracing::info!(
                            monitor = ?m.rect,
                            class = %class,
                            "live wallpaper found; backdrops follow it"
                        );
                        LiveWallpaper {
                            monitor: m.rect,
                            hwnd: 0,
                            since: Instant::now(),
                            missing: None,
                            image: None,
                            attempted: None,
                            reported: None,
                            empty: 0,
                            blind: false,
                        }
                    });
                    let hwnd = hwnd.0 as isize;
                    // A new window keeps the previous one's picture (or `blind`) until its own
                    // capture.
                    let same = hwnd == entry.hwnd;
                    self.live.push(LiveWallpaper {
                        hwnd,
                        since: if same { entry.since } else { Instant::now() },
                        missing: None,
                        empty: if same { entry.empty } else { 0 },
                        ..entry
                    });
                }
                (None, Some(mut entry)) => {
                    let since = *entry.missing.get_or_insert_with(Instant::now);
                    if since.elapsed() < LIVE_GONE_AFTER {
                        self.live.push(entry);
                    } else {
                        gone.push(entry);
                    }
                }
                (None, None) => {}
            }
        }
        for l in gone {
            tracing::info!(
                monitor = ?l.monitor,
                "live wallpaper gone; backdrops follow the Windows wallpaper"
            );
        }
    }

    /// A wallpaper signal: the live wallpapers may have changed along with the Windows
    /// wallpaper, and the backdrops may just have been rebuilt from their old captures. They
    /// settle again like a new window: captured at the next check and then at every check for
    /// `LIVE_SETTLE` (feedback #45).
    pub(super) fn resettle_live_wallpapers(&mut self) {
        let now = Instant::now();
        for live in &mut self.live {
            live.since = now;
        }
    }

    /// Starts a worker capturing the live wallpapers that are due: a new window, or no attempt
    /// for `min_age` (shorter while the window settles), on a monitor where a fence is shown
    /// and not hidden behind the foreground window.
    pub(super) fn capture_live_wallpapers(&mut self, min_age: Duration) {
        let due = |l: &LiveWallpaper| {
            let min_age = if l.since.elapsed() < LIVE_SETTLE {
                min_age.min(LIVE_SETTLE_REFRESH)
            } else {
                min_age
            };
            l.attempted
                .is_none_or(|(hwnd, at)| hwnd != l.hwnd || at.elapsed() >= min_age)
        };
        // A job whose result has not come back is given up, so it cannot stop every later
        // capture. Its worker may still wait in PrintWindow for the player's thread, with a
        // full-size bitmap: while one given-up worker still waits, the next job is kept.
        let abandoned_waits = self
            .live_abandoned
            .as_ref()
            .is_some_and(|thread| !thread.is_finished());
        if let Some(job) = self.live_capture.take_if(|job| {
            job.started.elapsed() >= LIVE_CAPTURE_GIVE_UP
                && (job.thread.is_finished() || !abandoned_waits)
        }) {
            tracing::warn!("live wallpaper capture brought no result; starting again");
            if !job.thread.is_finished() {
                self.live_abandoned = Some(job.thread);
            }
        }
        if self.live_capture.is_some()
            || !self.live.iter().any(due)
            || pecofence_core::brand::var_os("PECOFENCE_SOLID").is_some()
        {
            return;
        }
        let foreground = foreground_rect();
        let fences: Vec<RECT> = self
            .fences
            .values()
            .map(FenceWindow::hwnd)
            .filter(|&h| desktop::is_visible(h) && !desktop::is_cloaked(h))
            .map(window::window_rect)
            .filter(|f| {
                !foreground.is_some_and(|g| {
                    g.left <= f.left && g.top <= f.top && g.right >= f.right && g.bottom >= f.bottom
                })
            })
            .collect();
        let now = Instant::now();
        let mut targets = Vec::new();
        for l in self.live.iter_mut().filter(|l| due(l)) {
            let m = l.monitor;
            let shown = fences.iter().any(|f| {
                f.left < m.right && f.right > m.left && f.top < m.bottom && f.bottom > m.top
            });
            if shown {
                l.attempted = Some((l.hwnd, now));
                targets.push((m, l.hwnd));
            }
        }
        if targets.is_empty() {
            return;
        }
        let Ok(snapshot) = wallpaper_snapshot(self.wallpaper_override.as_deref()) else {
            return;
        };
        let theme = *self.ctx.theme.borrow();
        let work: Vec<_> = monitor_recipes(&theme, &snapshot, &self.live)
            .into_iter()
            .map(|recipe| {
                let target = targets.iter().find(|(m, _)| *m == recipe.rect());
                (recipe, target.map(|(_, hwnd)| *hwnd))
            })
            .collect();
        let result = Arc::new(Mutex::new(None));
        let spawned = {
            let result = result.clone();
            let control = self.control.hwnd().0 as isize;
            std::thread::Builder::new()
                .name("pecofence-live-wallpaper".into())
                .spawn(move || {
                    let built = pecofence_platform::com::MtaGuard::init()
                        .and_then(|_com| capture_and_build(work))
                        .map_err(|error| error.to_string());
                    if let Ok(mut slot) = result.lock() {
                        *slot = Some(built);
                    }
                    window::post_message(
                        HWND(control as *mut core::ffi::c_void),
                        WM_APP_LIVE_WALLPAPER,
                        0,
                        0,
                    );
                })
        };
        let thread = match spawned {
            Ok(thread) => thread,
            Err(error) => {
                tracing::warn!(%error, "live wallpaper capture thread failed to start");
                return;
            }
        };
        self.live_capture = Some(LiveCapture {
            shown: self.wallpaper_sig.clone(),
            generation: self.wallpaper_cache.generation(),
            file_signature: snapshot.signature(),
            snapshot,
            started: now,
            result,
            thread,
        });
    }

    /// The worker finished: keep the captures that worked and show the backdrops built from
    /// them. Without a new capture the current backdrop stays; after a rebuild meanwhile the
    /// backdrops are built again with them.
    pub(super) fn finish_live_capture(&mut self) {
        let Some(job) = self.live_capture.take() else {
            return;
        };
        let Some(result) = job.result.lock().ok().and_then(|mut slot| slot.take()) else {
            self.live_capture = Some(job);
            return;
        };
        let (built, empty) = match result {
            Ok(built) => built,
            Err(error) => {
                tracing::debug!(%error, "live wallpaper backdrop failed");
                return;
            }
        };
        // A window without a redirection bitmap never shows PrintWindow anything: its monitors
        // go blind at once, others after BLIND_AFTER empty captures, not before the window
        // settled (a player that just started can be black). Logged once per window once it
        // settled, and when it goes blind.
        let mut blinded = false;
        for (hwnd, reason, copied) in empty {
            let never = reason == live_wallpaper::NOT_REDIRECTED;
            let mut report = None;
            for live in self.live.iter_mut().filter(|l| l.hwnd == hwnd) {
                let settled = never || live.since.elapsed() >= LIVE_SETTLE;
                let mut now_blind = false;
                if !copied {
                    live.empty += 1;
                    now_blind = !live.blind && settled && (never || live.empty >= BLIND_AFTER);
                }
                if !settled {
                    continue;
                }
                if now_blind {
                    live.blind = true;
                    live.image = None;
                    blinded = true;
                }
                if live.reported != Some(hwnd) || now_blind {
                    live.reported = Some(hwnd);
                    report = Some((live.monitor, live.blind));
                }
            }
            let Some((monitor, blind)) = report else {
                continue;
            };
            let class = desktop::class_name(HWND(hwnd as *mut core::ffi::c_void));
            if copied {
                tracing::info!(?monitor, %class, reason, "live wallpaper capture came back empty; showing the window it clones");
            } else if blind {
                tracing::info!(?monitor, %class, reason, "live wallpaper cannot be captured; its backdrop shows a neutral colour");
            } else {
                tracing::info!(?monitor, %class, reason, "live wallpaper capture came back empty; the backdrop keeps its last picture");
            }
        }
        let mut captured = 0;
        for (recipe, hwnd, _) in &built {
            if let Some(hwnd) = *hwnd
                && let Some(live) = self
                    .live
                    .iter_mut()
                    .find(|l| l.monitor == recipe.rect() && l.hwnd == hwnd)
            {
                live.image = recipe.live.clone();
                live.empty = 0;
                live.blind = false;
                captured += 1;
            }
        }
        tracing::debug!(
            captured,
            blinded,
            elapsed_ms = job.started.elapsed().as_millis(),
            "live wallpaper captured"
        );
        if captured == 0 && !blinded {
            return;
        }
        // The worker built these monitors from their old captures: build again.
        if blinded
            || self.wallpaper_sig != job.shown
            || self.wallpaper_cache.generation() != job.generation
            || job.snapshot.signature() != job.file_signature
        {
            self.wallpaper_sig = None;
            self.check_wallpaper("live wallpaper");
            return;
        }
        let liquid_glass = self.ctx.theme.borrow().liquid_glass;
        let set: Vec<MonitorBackdrop> = built
            .into_iter()
            .map(|(recipe, _, image)| recipe.finish(image, liquid_glass))
            .collect();
        let backdrops = Rc::new(BackdropSets {
            acrylic: Rc::new(set),
        });
        self.wallpaper_sig = Some(backdrop_signature(&job.snapshot, &self.live));
        *self.ctx.backdrops.borrow_mut() = backdrops.clone();
        for window in self.fences.values() {
            window.set_backdrops(backdrops.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_tells_each_monitor_s_live_wallpaper_state() {
        let monitor = |left| RECT {
            left,
            top: 0,
            right: left + 1920,
            bottom: 1080,
        };
        let entry = |left, image: Option<Arc<Image>>, blind| LiveWallpaper {
            monitor: monitor(left),
            hwnd: 1,
            since: Instant::now(),
            missing: None,
            image,
            attempted: None,
            reported: None,
            empty: 0,
            blind,
        };
        let live = [
            entry(0, Some(Arc::new(Image::solid(4, 4, [9, 9, 9]))), false),
            entry(1920, None, true),
            entry(3840, None, false),
        ];
        assert_eq!(live_wallpaper_state(&live, monitor(0)), "captured");
        assert_eq!(live_wallpaper_state(&live, monitor(1920)), "notCapturable");
        assert_eq!(live_wallpaper_state(&live, monitor(3840)), "pending");
        assert_eq!(live_wallpaper_state(&live, monitor(-1920)), "none");
    }
}
