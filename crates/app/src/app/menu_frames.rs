//! Fence animations while a popup menu is open. `TrackPopupMenuEx` runs inside an App method,
//! so the App stays borrowed until the menu closes and `on_frame` cannot run: a fence that was
//! expanding stopped half way for as long as the menu stayed open. The fence windows' frame
//! handles are registered here for that time, and a frame that finds the App borrowed ticks
//! them instead. What needs the App waits for the menu: pushing the neighbours of a growing
//! fence, the peek's fade, and every queued command (the CLI included).

use super::*;
use crate::fence_window::FrameHandle;

/// The fence windows a frame ticks while a menu holds the App, and that run's frame cadence.
struct MenuFrames {
    windows: Vec<FrameHandle>,
    frames: Rc<FrameClock>,
    /// The last frame before the menu or during it, for the gap to the next one.
    last: Option<Instant>,
    first: Option<Instant>,
    ticks: u32,
    /// Frames that followed another one (the gaps measured).
    gaps: u32,
    max_gap: Duration,
    max_frame: Duration,
    /// The last frame found nothing moving any more: the run ended then.
    ended: Option<Instant>,
}

thread_local! {
    static MENU_FRAMES: RefCell<Option<MenuFrames>> = const { RefCell::new(None) };
}

impl App {
    /// Runs `show` (a popup menu's `show` / `show_context`) with the fence windows' animations
    /// still ticking, and returns its command.
    pub(super) fn track_menu(&mut self, show: impl FnOnce() -> u32) -> u32 {
        let windows = self
            .fences
            .values()
            .chain(self.dying.iter())
            .map(|w| w.frame_handle())
            .collect();
        MENU_FRAMES.with(|m| {
            *m.borrow_mut() = Some(MenuFrames {
                windows,
                frames: self.ctx.frames.clone(),
                last: self.frame_prev,
                first: None,
                ticks: 0,
                gaps: 0,
                max_gap: Duration::ZERO,
                max_frame: Duration::ZERO,
                ended: None,
            })
        });
        let cmd = show();
        let Some(run) = MENU_FRAMES.with(|m| m.borrow_mut().take()) else {
            return cmd;
        };
        if run.ticks > 0 {
            // The frames during the menu belong to the animation run in progress (or start
            // one), so its stutter line measures them rather than the whole menu as one gap.
            if self.frame_prev.is_none() {
                self.frame_run = FrameRun {
                    started: run.first,
                    ..FrameRun::default()
                };
            }
            self.frame_run.frames += run.gaps;
            self.frame_run.max_gap = self.frame_run.max_gap.max(run.max_gap);
            self.frame_run.max_frame = self.frame_run.max_frame.max(run.max_frame);
            match run.ended {
                Some(at) => self.end_frame_run(at),
                None => {
                    // Still moving: the App's own frames carry on with the run.
                    self.frame_prev = run.last;
                    self.ctx.frames.request();
                }
            }
            let ms = crate::share_log::millis;
            tracing::info!(
                frames = run.ticks,
                max_gap_ms = ms(run.max_gap),
                max_frame_ms = ms(run.max_frame),
                "fence animations ran while a popup menu was open"
            );
        }
        cmd
    }
}

/// A frame that found the App borrowed: ticks the fence windows `track_menu` registered.
/// Returns false when no menu registered any (the caller retries the frame later).
pub(super) fn menu_frame() -> bool {
    MENU_FRAMES.with(|m| {
        let Ok(mut guard) = m.try_borrow_mut() else {
            return false;
        };
        let Some(run) = guard.as_mut() else {
            return false;
        };
        let now = Instant::now();
        if let Some(prev) = run.last {
            run.gaps += 1;
            run.max_gap = run.max_gap.max(now.duration_since(prev));
        }
        run.first.get_or_insert(now);
        run.last = Some(now);
        run.ticks += 1;
        let mut more = false;
        for w in &run.windows {
            more |= w.on_frame(now);
        }
        run.max_frame = run.max_frame.max(now.elapsed());
        if more {
            run.ended = None;
            run.frames.request();
        } else {
            run.ended = Some(now);
            // A later frame (an animation starting during the menu) is a run of its own.
            run.last = None;
        }
        true
    })
}
