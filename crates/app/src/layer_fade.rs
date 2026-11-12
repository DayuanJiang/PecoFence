//! The fade of the click-through layered windows that show drag feedback (the alignment guides
//! and the landing outline): 83 ms linear in and out through the window's constant alpha, as
//! the rest of a fence's chrome fades. They used to appear and vanish in one frame. With
//! Windows' animations off they still do.

use pecofence_platform::window::{self, Window, WindowClass};
use pecofence_platform::{HWND, desktop, layered, sysparams};
use pecofence_render::motion;
use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;
use windows_core::Result;

const TIMER_FADE: usize = 1;
/// One step per frame at 60 Hz.
const TICK_MS: u32 = 15;

/// A fade of the constant alpha from `from` to `to` (0..=1) that began at `start`.
#[derive(Clone, Copy)]
struct Fade {
    from: f32,
    to: f32,
    start: Instant,
}

impl Fade {
    fn progress(&self, now: Instant) -> f32 {
        let t = now.duration_since(self.start).as_secs_f32() / motion::FASTER.as_secs_f32();
        t.clamp(0.0, 1.0)
    }

    fn value(&self, now: Instant) -> f32 {
        self.from + (self.to - self.from) * self.progress(now)
    }
}

fn alpha_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub struct FadingLayer {
    window: Window,
    fade: Rc<Cell<Fade>>,
    /// Showing or fading in; false once hidden or on its way out.
    shown: bool,
}

impl FadingLayer {
    pub fn create(class: &WindowClass, title: &str) -> Result<Self> {
        let fade = Rc::new(Cell::new(Fade {
            from: 0.0,
            to: 0.0,
            start: Instant::now(),
        }));
        let tick = fade.clone();
        let window = window::create_click_through_with(class, title, move |hwnd: HWND| {
            let (f, now) = (tick.get(), Instant::now());
            let _ = layered::set_alpha(hwnd, alpha_u8(f.value(now)));
            if f.progress(now) >= 1.0 {
                window::kill_timer(hwnd, TIMER_FADE);
                if f.to <= 0.0 {
                    desktop::hide_window(hwnd);
                }
            }
        })?;
        Ok(Self {
            window,
            fade,
            shown: false,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// The constant alpha a bitmap presented now must carry: where the fade stands.
    pub fn alpha(&self) -> u8 {
        if !self.window.is_visible() {
            return if sysparams::client_area_animation() {
                0
            } else {
                255
            };
        }
        alpha_u8(self.fade.get().value(Instant::now()))
    }

    /// Shows the window (its bitmap is presented): fades in from where it stands.
    pub fn show(&mut self) {
        if self.shown {
            return;
        }
        self.shown = true;
        let now = Instant::now();
        let from = if self.window.is_visible() {
            self.fade.get().value(now)
        } else {
            0.0
        };
        let animate = sysparams::client_area_animation();
        let from = if animate { from } else { 1.0 };
        self.fade.set(Fade {
            from,
            to: 1.0,
            start: now,
        });
        let _ = layered::set_alpha(self.window.hwnd(), alpha_u8(from));
        self.window.show_no_activate();
        if animate {
            self.window.set_timer(TIMER_FADE, TICK_MS);
        } else {
            self.window.kill_timer(TIMER_FADE);
        }
    }

    /// Fades the window out and hides it.
    pub fn hide(&mut self) {
        if !self.shown {
            return;
        }
        self.shown = false;
        if !sysparams::client_area_animation() || !self.window.is_visible() {
            self.window.kill_timer(TIMER_FADE);
            self.window.hide();
            return;
        }
        let now = Instant::now();
        self.fade.set(Fade {
            from: self.fade.get().value(now),
            to: 0.0,
            start: now,
        });
        self.window.set_timer(TIMER_FADE, TICK_MS);
    }
}
