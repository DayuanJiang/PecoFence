//! White outline while a fence is dragged: the spot it will land on when let go over another
//! fence (`FenceDropped` moves it to the nearest free spot), or the fence it will join as a tab
//! when its title is over that fence's title. A click-through layered window placed directly
//! below the dragged fence (so above the fence it outlines). It fades in and out (83 ms,
//! `layer_fade`).

use crate::layer_fade::FadingLayer;
use pecofence_platform::frameclock::FrameClock;
use pecofence_platform::layered::LayeredImage;
use pecofence_platform::window::{self, ClassOptions, WindowClass};
use pecofence_platform::{HWND, RECT, desktop};
use std::rc::Rc;
use std::time::Instant;
use windows_core::Result;

pub const DROP_PREVIEW_CLASS: &str = "PecoFence.DropPreview";

pub struct DropPreview {
    _class: WindowClass,
    layer: FadingLayer,
    /// Size, DPI and corner radius of the presented outline; a pure move only repositions the
    /// window.
    last: Option<(i32, i32, u32, f32)>,
}

/// Premultiplied BGRA outline for a `w` x `h` px rectangle with `radius_dip` corners (the
/// fences' own): a light wash, a white 2 DIP stroke and a dark 1 DIP hairline outside it,
/// readable on light and dark wallpapers.
fn render(w: i32, h: i32, scale: f32, radius_dip: f32) -> Vec<u8> {
    let radius = (radius_dip * scale).min(w.min(h) as f32 * 0.5);
    let stroke = 2.0 * scale;
    let hair = 1.0 * scale;
    let mut bgra = vec![0u8; (w * h * 4) as usize];
    // Signed distance to the rounded rectangle's edge (negative inside).
    let sdf = |fx: f32, fy: f32| -> f32 {
        let dx = (fx - w as f32 / 2.0).abs() - (w as f32 / 2.0 - radius);
        let dy = (fy - h as f32 / 2.0).abs() - (h as f32 / 2.0 - radius);
        (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() + dx.max(dy).min(0.0) - radius
    };
    let band = |d: f32, from: f32, to: f32| {
        ((d - from + 0.5).clamp(0.0, 1.0)) * ((to - d + 0.5).clamp(0.0, 1.0))
    };
    for y in 0..h {
        for x in 0..w {
            let d = sdf(x as f32 + 0.5, y as f32 + 0.5);
            // Inside out: wash, then the white stroke, then the dark hairline at the edge.
            let hairline = band(d, -hair, 0.0);
            let white = band(d, -hair - stroke, -hair);
            let wash = band(d, f32::MIN / 2.0, -hair - stroke);
            let (mut r, mut a) = (0.0f32, 0.0f32);
            let mut over = |c: f32, alpha: f32| {
                r = c * alpha + r * (1.0 - alpha);
                a = alpha + a * (1.0 - alpha);
            };
            over(1.0, 0.14 * wash);
            over(1.0, 0.92 * white);
            over(0.0, 0.45 * hairline);
            let i = ((y * w + x) * 4) as usize;
            let c = (r * 255.0).round() as u8;
            bgra[i] = c;
            bgra[i + 1] = c;
            bgra[i + 2] = c;
            bgra[i + 3] = (a * 255.0).round() as u8;
        }
    }
    bgra
}

impl DropPreview {
    pub fn create(frames: Rc<FrameClock>) -> Result<Self> {
        let class = WindowClass::register(DROP_PREVIEW_CLASS, ClassOptions::default())?;
        let layer = FadingLayer::create(&class, "drop preview", frames)?;
        Ok(Self {
            _class: class,
            layer,
            last: None,
        })
    }

    /// Shows the outline at `rect` (screen px), directly below `dragged` in the z-order.
    pub fn show_at(&mut self, rect: RECT, dpi: u32, radius_dip: f32, dragged: HWND) {
        let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
        if w <= 0 || h <= 0 {
            return;
        }
        let window = self.layer.window();
        if self.last == Some((w, h, dpi, radius_dip)) {
            let _ = window.set_bounds_z(
                rect.left,
                rect.top,
                w,
                h,
                window::ZOrder::Keep,
                window::swp::NOSIZE,
            );
        } else {
            let scale = dpi.max(96) as f32 / 96.0;
            let presented =
                LayeredImage::new(w, h, &render(w, h, scale, radius_dip)).and_then(|img| {
                    img.present(window.hwnd(), rect.left, rect.top, self.layer.alpha())
                });
            if let Err(e) = presented {
                tracing::warn!(error = %e, "drop preview present failed");
                return;
            }
            self.last = Some((w, h, dpi, radius_dip));
        }
        let _ = desktop::insert_after(window.hwnd(), dragged);
        self.layer.show();
    }

    pub fn hide(&mut self) {
        self.layer.hide();
    }

    /// One frame of the outline's fade. Returns true while it is still on its way.
    pub fn tick(&mut self, now: Instant) -> bool {
        self.layer.tick(now)
    }

    /// The outline's window rect while it is on screen, for test dumps.
    pub fn shown(&self) -> Option<RECT> {
        self.layer
            .is_shown()
            .then(|| self.layer.window().window_rect())
    }
}
