//! Peek (Fences "Win+Space"): every fence floats above the current windows over a dimmed
//! backdrop until the hotkey is pressed again, Esc / a click lands on the dimmer, or another
//! application takes the foreground.
//!
//! The dimmer is one plain layered window per monitor work area (solid black at ~30 % alpha,
//! the Fluent smoke layer; the taskbar stays visible and clickable). It is activatable so it
//! can receive Esc; fences stay `WS_EX_NOACTIVATE` as always. Its alpha fades in and out (83 ms linear, the WinUI
//! smoke-layer / Fluent fade) as a client tween: a layered window has no compositor visual, so
//! `App::on_frame` samples the tween and re-applies the alpha each frame.

use crate::anchor;
use crate::commands::{Command, CommandQueue, WM_APP_PEEK_FOCUSED};
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, desktop, monitors, msg};
use pecofence_render::motion::{self, Curve, Motion, Tween};
use std::time::{Duration, Instant};
use windows_core::Result;

pub const PEEK_CLASS: &str = "PecoFence.PeekDimmer";
const WA_INACTIVE: usize = 0;
/// 30 % black (WinUI `SmokeFillColorDefault` #4D000000), the resting alpha of the dimmer.
const DIM_ALPHA: u8 = 0x4D;
/// Never 0: a fully transparent layered window is not hit-tested, and the dimmer must catch
/// the click that ends the peek even while it fades.
const CLEAR_ALPHA: u8 = 1;
/// Fade duration both ways (WinUI `ControlFasterAnimationDuration`, linear).
const PEEK_FADE: Duration = motion::FASTER;

pub struct PeekOverlay {
    windows: Vec<Window>,
    dim: bool,
    /// Layered alpha of every dimmer window (`CLEAR_ALPHA..=DIM_ALPHA`).
    alpha: Tween,
    /// The fade-out is running; the app tears the overlay down when it lands.
    closing: bool,
}

impl PeekOverlay {
    pub fn register_class() -> Result<WindowClass> {
        WindowClass::register(
            PEEK_CLASS,
            ClassOptions {
                double_clicks: false,
                background: Some(0x0000_0000),
            },
        )
    }

    /// Creates and shows one dimmer per monitor (topmost), starting its fade-in. Fences must
    /// be raised above them afterwards (see `DesktopAnchor::set_peek`); the caller requests a
    /// frame while [`PeekOverlay::animating`].
    pub fn show(
        class: &WindowClass,
        queue: CommandQueue,
        control: HWND,
        dim: bool,
        motion: &Motion,
    ) -> Result<Self> {
        let mut windows = Vec::new();
        for m in monitors::enumerate() {
            let handler: MessageHandler = {
                let queue = queue.clone();
                Box::new(
                    move |_hwnd: HWND, message: u32, wparam: usize, lparam: isize| {
                        match message {
                            msg::WM_LBUTTONDOWN | msg::WM_RBUTTONDOWN | msg::WM_MBUTTONDOWN => {
                                queue.push(Command::EndPeek);
                                Some(0)
                            }
                            // Esc ends the peek (no auto-repeats). Modifier keys must NOT: the
                            // toggle hotkey's own Ctrl / Win / Shift arrive here as WM_KEYDOWN
                            // before WM_HOTKEY fires, and ending the peek on them would make the
                            // hotkey restart it instead of ending it.
                            msg::WM_KEYDOWN
                                if wparam as u32 == msg::VK_ESCAPE && lparam & (1 << 30) == 0 =>
                            {
                                queue.push(Command::EndPeek);
                                Some(0)
                            }
                            msg::WM_SETFOCUS => {
                                tracing::debug!("peek dimmer received keyboard focus");
                                None
                            }
                            msg::WM_ACTIVATE if wparam & 0xFFFF == WA_INACTIVE => {
                                // Focus went elsewhere. A fence (click inside its content) or another
                                // dimmer keeps the peek; anything else ends it.
                                let other = HWND(lparam as *mut core::ffi::c_void);
                                let cls = desktop::class_name(desktop::root_ancestor(other));
                                if !other.0.is_null()
                                    && cls != anchor::FENCE_CLASS
                                    && cls != PEEK_CLASS
                                    && cls != "PecoFence.Rename"
                                {
                                    queue.push(Command::EndPeek);
                                }
                                None
                            }
                            msg::WM_CLOSE => Some(0),
                            _ => None,
                        }
                    },
                )
            };
            // Work area, not bounds: the taskbar must stay visible and reachable.
            let b = m.work_area;
            let w = WindowBuilder::new(class)
                .title("PecoFence peek")
                .style(style::POPUP)
                .ex_style(style::EX_TOOLWINDOW | style::EX_LAYERED | style::EX_TOPMOST)
                .bounds(b.left, b.top, b.right - b.left, b.bottom - b.top)
                .create(handler)?;
            // Black class brush + layered alpha = the dimmer. Without dimming the window is
            // (nearly) invisible but still catches Esc / clicks to end the peek.
            window::set_layered_alpha(w.hwnd(), CLEAR_ALPHA);
            w.show_no_activate();
            windows.push(w);
        }
        // Take the keyboard on the primary dimmer so Esc works. SetForegroundWindow can block
        // while Windows hands focus between input queues, so keep it off the UI thread. Once
        // it completes the app restores the fence block above the activated topmost dimmer.
        if let Some(first) = windows.first() {
            let dimmer = first.hwnd().0 as usize;
            let control = control.0 as usize;
            let spawned = std::thread::Builder::new()
                .name("pecofence-peek-focus".into())
                .spawn(move || {
                    window::bring_to_front(HWND(dimmer as *mut core::ffi::c_void));
                    window::post_message(
                        HWND(control as *mut core::ffi::c_void),
                        WM_APP_PEEK_FOCUSED,
                        dimmer,
                        0,
                    );
                });
            if let Err(e) = spawned {
                tracing::warn!(error = %e, "peek focus thread failed to start");
            }
        }
        // Do not let synchronous window creation consume any of the fade's timeline.
        let now = Instant::now();
        let alpha = if dim {
            motion.tween(
                CLEAR_ALPHA as f32,
                DIM_ALPHA as f32,
                PEEK_FADE,
                Curve::Linear,
                now,
            )
        } else {
            Tween::at(CLEAR_ALPHA as f32, now)
        };
        let overlay = Self {
            windows,
            dim,
            alpha,
            closing: false,
        };
        // Applied once synchronously so a zero-duration tween (animations off) lands at the
        // resting alpha before the first frame is composed.
        overlay.apply(now);
        Ok(overlay)
    }

    pub fn hwnds(&self) -> Vec<HWND> {
        self.windows.iter().map(|w| w.hwnd()).collect()
    }

    pub fn contains(&self, hwnd: HWND) -> bool {
        self.windows.iter().any(|w| w.hwnd() == hwnd)
    }

    fn apply(&self, now: Instant) {
        let a = self.alpha.value_at(now).round().clamp(1.0, 255.0) as u8;
        for w in &self.windows {
            window::set_layered_alpha(w.hwnd(), a);
        }
    }

    /// Starts the fade-out (continuing from wherever a running fade-in is). Idempotent.
    pub fn begin_close(&mut self, now: Instant) {
        if self.closing {
            return;
        }
        self.closing = true;
        if self.dim {
            self.alpha
                .retarget(CLEAR_ALPHA as f32, PEEK_FADE, Curve::Linear, now);
        }
    }

    /// The hotkey was pressed again during the fade-out: dim again without recreating windows.
    pub fn reopen(&mut self, now: Instant) {
        self.closing = false;
        if self.dim {
            self.alpha
                .retarget(DIM_ALPHA as f32, PEEK_FADE, Curve::Linear, now);
        }
    }

    pub fn is_closing(&self) -> bool {
        self.closing
    }

    /// The fade-out has landed: the overlay can be torn down.
    pub fn close_finished(&self, now: Instant) -> bool {
        self.closing && self.alpha.is_done(now)
    }

    pub fn animating(&self, now: Instant) -> bool {
        !self.alpha.is_done(now)
    }

    /// One frame: re-applies the tweened alpha. Returns true while the fade still runs.
    pub fn on_frame(&mut self, now: Instant) -> bool {
        self.apply(now);
        self.animating(now)
    }
}
