//! Alignment guides while a fence is dragged or resized (`snapping.guideLines`): thin
//! click-through layered windows along the edges it lines up with and the screen centre lines
//! it snapped to, placed directly above the moving fence.

use pecofence_platform::layered::LayeredImage;
use pecofence_platform::window::{self, ClassOptions, Window, WindowClass};
use pecofence_platform::{HWND, desktop};
use windows_core::Result;

pub const DRAG_GUIDE_CLASS: &str = "PecoFence.DragGuide";

/// How far a guide runs past the edges it joins, fading out, in DIPs.
const OVERHANG_DIP: f32 = 12.0;

/// One guide in screen px: a vertical line at x = `at` (horizontal at y = `at` otherwise),
/// running from `from` to `to` along it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuideLine {
    pub vertical: bool,
    pub at: i32,
    pub from: i32,
    pub to: i32,
}

/// Window bounds (x, y, w, h) of `line` and the width of its dark edges, at `scale`: a 1 DIP
/// white core centred on the edge between a dark hairline on each side.
fn bounds(line: &GuideLine, scale: f32) -> ((i32, i32, i32, i32), i32, i32) {
    let core = scale.round().max(1.0) as i32;
    let hair = (scale * 0.5).round().max(1.0) as i32;
    let overhang = (OVERHANG_DIP * scale).round() as i32;
    let across = line.at - core / 2 - hair;
    let thick = core + 2 * hair;
    let along = line.from - overhang;
    let len = line.to - line.from + 2 * overhang;
    let rect = if line.vertical {
        (across, along, thick, len)
    } else {
        (along, across, len, thick)
    };
    (rect, hair, overhang)
}

/// Premultiplied BGRA for a `w` x `h` px guide: white core, dark edges, ends fading out over
/// `fade` px.
fn render(w: i32, h: i32, vertical: bool, hair: i32, fade: i32) -> Vec<u8> {
    let (across, along) = if vertical { (w, h) } else { (h, w) };
    let mut bgra = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let (a, l) = if vertical { (x, y) } else { (y, x) };
            let end = ((l.min(along - 1 - l) as f32 + 0.5) / fade.max(1) as f32).min(1.0);
            let core = a >= hair && a < across - hair;
            let (c, alpha) = if core {
                (1.0, 0.9 * end)
            } else {
                (0.0, 0.4 * end)
            };
            let i = ((y * w + x) * 4) as usize;
            let v = (c * alpha * 255.0f32).round() as u8;
            bgra[i] = v;
            bgra[i + 1] = v;
            bgra[i + 2] = v;
            bgra[i + 3] = (alpha * 255.0).round() as u8;
        }
    }
    bgra
}

struct LineWindow {
    window: Window,
    /// Size and direction of the presented bitmap; a pure move only repositions the window.
    last: Option<(i32, i32, bool)>,
    shown: bool,
}

impl LineWindow {
    fn create(class: &WindowClass) -> Result<Self> {
        let window = window::create_click_through(class, "drag guide")?;
        Ok(Self {
            window,
            last: None,
            shown: false,
        })
    }

    fn show(&mut self, line: &GuideLine, scale: f32, above: HWND) {
        let ((x, y, w, h), hair, fade) = bounds(line, scale);
        if w <= 0 || h <= 0 {
            self.hide();
            return;
        }
        if self.last == Some((w, h, line.vertical)) {
            let _ = self
                .window
                .set_bounds_z(x, y, w, h, window::ZOrder::Keep, window::swp::NOSIZE);
        } else {
            let presented = LayeredImage::new(w, h, &render(w, h, line.vertical, hair, fade))
                .and_then(|img| img.present(self.window.hwnd(), x, y, 255));
            if let Err(e) = presented {
                tracing::warn!(error = %e, "drag guide present failed");
                return;
            }
            self.last = Some((w, h, line.vertical));
        }
        let _ = desktop::insert_above(self.window.hwnd(), above);
        if !self.shown {
            self.window.show_no_activate();
            self.shown = true;
        }
    }

    fn hide(&mut self) {
        if self.shown {
            self.window.hide();
            self.shown = false;
        }
    }
}

pub struct DragGuides {
    class: WindowClass,
    /// One window per guide on screen, created on first use and reused.
    lines: Vec<LineWindow>,
}

impl DragGuides {
    pub fn create() -> Result<Self> {
        Ok(Self {
            class: WindowClass::register(DRAG_GUIDE_CLASS, ClassOptions::default())?,
            lines: Vec::new(),
        })
    }

    /// Shows `lines` (and hides any others) directly above `moving` in the z-order.
    pub fn show(&mut self, lines: &[GuideLine], dpi: u32, moving: HWND) {
        while self.lines.len() < lines.len() {
            match LineWindow::create(&self.class) {
                Ok(w) => self.lines.push(w),
                Err(e) => {
                    tracing::warn!(error = %e, "drag guide window unavailable");
                    break;
                }
            }
        }
        let scale = dpi.max(96) as f32 / 96.0;
        for (i, w) in self.lines.iter_mut().enumerate() {
            match lines.get(i) {
                Some(line) => w.show(line, scale, moving),
                None => w.hide(),
            }
        }
    }

    pub fn hide(&mut self) {
        for w in &mut self.lines {
            w.hide();
        }
    }

    /// Window rects of the guides on screen, for test dumps.
    pub fn shown(&self) -> Vec<pecofence_platform::RECT> {
        self.lines
            .iter()
            .filter(|w| w.shown)
            .map(|w| w.window.window_rect())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// At 200 % the core is 2 px straddling the edge with a 1 px dark hairline each side, and
    /// the line runs 24 px past both ends.
    #[test]
    fn guide_straddles_the_edge_and_overhangs_both_ends() {
        let line = GuideLine {
            vertical: true,
            at: 100,
            from: 50,
            to: 250,
        };
        let ((x, y, w, h), hair, fade) = bounds(&line, 2.0);
        assert_eq!((x, w, hair), (98, 4, 1));
        assert_eq!((y, h, fade), (26, 248, 24));
        let px = render(w, h, true, hair, fade);
        let at = |x: i32, y: i32| px[((y * w + x) * 4 + 3) as usize];
        assert_eq!(at(1, h / 2), 230, "white core");
        assert_eq!(at(0, h / 2), 102, "dark hairline");
        assert!(at(1, 0) < 10, "fades out at the end");
        let flat = GuideLine {
            vertical: false,
            ..line
        };
        let ((_, fy, fw, fh), _, _) = bounds(&flat, 1.0);
        assert_eq!((fy, fw, fh), (99, 224, 3));
    }
}
