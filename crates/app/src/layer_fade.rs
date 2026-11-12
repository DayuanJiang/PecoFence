//! The fade of the click-through layered windows that show drag feedback (the alignment guides
//! and the landing outline): 83 ms linear in and out through the window's constant alpha, as
//! the rest of a fence's chrome fades. They used to appear and vanish in one frame. With
//! Windows' animations off they still do.
//!
//! The fade is ticked on the app's frame clock (`App::on_frame`), like the Peek dimmer's: a
//! window timer would wait behind the mouse moves of the drag the feedback belongs to.

use pecofence_platform::frameclock::FrameClock;
use pecofence_platform::window::{self, Window, WindowClass};
use pecofence_platform::{layered, sysparams};
use pecofence_render::motion;
use std::rc::Rc;
use std::time::Instant;
use windows_core::Result;

fn alpha_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub struct FadingLayer {
    window: Window,
    frames: Rc<FrameClock>,
    /// The fade of the constant alpha: from, to (0..=1) and when it began.
    fade: (f32, f32, Instant),
    /// The fade is still on its way (`tick` moves it on).
    fading: bool,
    /// Showing or fading in; false once hidden or on its way out.
    shown: bool,
}

impl FadingLayer {
    pub fn create(class: &WindowClass, title: &str, frames: Rc<FrameClock>) -> Result<Self> {
        Ok(Self {
            window: window::create_click_through(class, title)?,
            frames,
            fade: (0.0, 0.0, Instant::now()),
            fading: false,
            shown: false,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn is_shown(&self) -> bool {
        self.shown
    }

    fn progress(&self, now: Instant) -> f32 {
        let t = now.duration_since(self.fade.2).as_secs_f32() / motion::FASTER.as_secs_f32();
        t.clamp(0.0, 1.0)
    }

    fn value(&self, now: Instant) -> f32 {
        let (from, to, _) = self.fade;
        from + (to - from) * self.progress(now)
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
        alpha_u8(self.value(Instant::now()))
    }

    /// Shows the window (its bitmap is presented): fades in from where it stands.
    pub fn show(&mut self) {
        if self.shown {
            return;
        }
        self.shown = true;
        let now = Instant::now();
        let animate = sysparams::client_area_animation();
        let from = match (animate, self.window.is_visible()) {
            (false, _) => 1.0,
            (true, true) => self.value(now),
            (true, false) => 0.0,
        };
        self.fade = (from, 1.0, now);
        self.fading = animate;
        let _ = layered::set_alpha(self.window.hwnd(), alpha_u8(from));
        self.window.show_no_activate();
        if animate {
            self.frames.request();
        }
    }

    /// Fades the window out and hides it.
    pub fn hide(&mut self) {
        if !self.shown {
            return;
        }
        self.shown = false;
        if !sysparams::client_area_animation() || !self.window.is_visible() {
            self.fading = false;
            self.window.hide();
            return;
        }
        let now = Instant::now();
        self.fade = (self.value(now), 0.0, now);
        self.fading = true;
        self.frames.request();
    }

    /// One frame of the fade. Returns true while it is still on its way.
    pub fn tick(&mut self, now: Instant) -> bool {
        if !self.fading {
            return false;
        }
        let _ = layered::set_alpha(self.window.hwnd(), alpha_u8(self.value(now)));
        if self.progress(now) >= 1.0 {
            self.fading = false;
            if self.fade.1 <= 0.0 {
                self.window.hide();
            }
        }
        self.fading
    }
}
