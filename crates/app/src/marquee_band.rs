//! The rubber band of a marquee drawn on the bare desktop. Explorer draws none while its icons
//! are hidden (PecoFence's usual state), so this one does, in the white of the other drag
//! feedback (drop outline, guides): a light white wash inside a 1 DIP white border with a dark
//! hairline outside it, readable on light and dark wallpapers. Five click-through layered
//! windows: the wash, painted by its class brush at a constant alpha, and the four edges, small
//! bitmaps redrawn as the band grows. They sit directly above the desktop icon host, below the
//! fences, where Explorer's own band would be.

use pecofence_platform::layered::LayeredImage;
use pecofence_platform::window::{self, ClassOptions, Window, WindowClass};
use pecofence_platform::{HWND, RECT, desktop};
use windows_core::Result;

pub const MARQUEE_BAND_CLASS: &str = "PecoFence.MarqueeBand";

/// Alpha of the wash (the drop outline's 14 % white).
const FILL_ALPHA: u8 = 0x24;
/// Premultiplied BGRA of the border line and of the hairline outside it (the drop outline's
/// 92 % white and 45 % black).
const LINE_PX: [u8; 4] = [235, 235, 235, 235];
const HAIR_PX: [u8; 4] = [0, 0, 0, 115];

pub struct MarqueeBand {
    // Windows before the class: they are destroyed before it is unregistered.
    fill: Window,
    /// Top, bottom, left, right.
    edges: [Window; 4],
    shown: bool,
    _class: WindowClass,
}

/// A `w` x `h` edge bitmap: hairline where `dark(x, y)`, the white line elsewhere.
fn edge_pixels(w: i32, h: i32, dark: impl Fn(i32, i32) -> bool) -> Vec<u8> {
    let mut bgra = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            bgra.extend_from_slice(if dark(x, y) { &HAIR_PX } else { &LINE_PX });
        }
    }
    bgra
}

impl MarqueeBand {
    pub fn create() -> Result<Self> {
        let class = WindowClass::register(
            MARQUEE_BAND_CLASS,
            ClassOptions {
                double_clicks: false,
                background: Some(0x00FF_FFFF),
            },
        )?;
        let make = || -> Result<Window> { window::create_click_through(&class, "marquee") };
        let fill = make()?;
        window::set_layered_alpha(fill.hwnd(), FILL_ALPHA);
        Ok(Self {
            fill,
            edges: [make()?, make()?, make()?, make()?],
            shown: false,
            _class: class,
        })
    }

    /// Spans `rect` (screen px) at `dpi`, just above the icon `host` when it first shows. The
    /// white line lies inside `rect`, the hairline just outside it.
    pub fn show(&mut self, rect: RECT, dpi: u32, host: Option<HWND>) {
        let t = ((dpi.max(96) as f32 / 96.0).round() as i32).max(1);
        let (w, h) = (
            (rect.right - rect.left).max(1),
            (rect.bottom - rect.top).max(1),
        );
        let _ = self.fill.set_bounds(rect.left, rect.top, w, h);
        // Top and bottom span the corners outside the rect too (hairline there).
        let across = w + 2 * t;
        let edges = [
            (rect.left - t, rect.top - t, across, 2 * t, true),
            (rect.left - t, rect.bottom - t, across, 2 * t, false),
            (rect.left - t, rect.top, 2 * t, h, true),
            (rect.right - t, rect.top, 2 * t, h, false),
        ];
        for (i, (x, y, ew, eh, outer_first)) in edges.into_iter().enumerate() {
            let horizontal = i < 2;
            let bgra = edge_pixels(ew, eh, |px, py| {
                let depth = if horizontal { py } else { px };
                let outer = if outer_first { depth < t } else { depth >= t };
                outer || (horizontal && (px < t || px >= ew - t))
            });
            let presented = LayeredImage::new(ew, eh, &bgra)
                .and_then(|img| img.present(self.edges[i].hwnd(), x, y, 255));
            if let Err(e) = presented {
                tracing::warn!(error = %e, "marquee band edge present failed");
            }
        }
        if !self.shown {
            for win in std::iter::once(&self.fill).chain(&self.edges) {
                if let Some(host) = host {
                    let _ = desktop::insert_above(win.hwnd(), host);
                }
                win.show_no_activate();
            }
            self.shown = true;
        }
    }

    pub fn hide(&mut self) {
        if self.shown {
            for win in std::iter::once(&self.fill).chain(&self.edges) {
                win.hide();
            }
            self.shown = false;
        }
    }

    /// The band's rect while it is on screen, for test dumps.
    pub fn shown(&self) -> Option<RECT> {
        self.shown.then(|| self.fill.window_rect())
    }
}
