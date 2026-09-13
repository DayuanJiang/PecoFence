//! Fence geometry normalization between physical pixels and per-monitor DIPs (plan §7.4).

use crate::model::{Anchor, NormGeometry};

pub const MIN_W_DIP: f32 = 64.0;
pub const MIN_H_DIP: f32 = 36.0;
pub const MAX_DIP: f32 = 8192.0;

/// A monitor's work area in physical pixels plus its DPI.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkArea {
    pub device_path: String,
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub dpi: u32,
    /// Full monitor rectangle (rcMonitor, taskbar included) in virtual-screen px, for labels.
    pub mon_left: i32,
    pub mon_top: i32,
    pub mon_right: i32,
    pub mon_bottom: i32,
}

impl WorkArea {
    pub fn scale(&self) -> f32 {
        self.dpi.max(1) as f32 / 96.0
    }
    pub fn width_dip(&self) -> f32 {
        (self.right - self.left) as f32 / self.scale()
    }
    pub fn height_dip(&self) -> f32 {
        (self.bottom - self.top) as f32 / self.scale()
    }
}

/// A window rectangle in physical (virtual-screen) pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PxRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl PxRect {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
    pub fn center(&self) -> (i32, i32) {
        ((self.left + self.right) / 2, (self.top + self.bottom) / 2)
    }
}

/// Picks the anchor by which edges the fence is closest to.
fn pick_anchor(x: f32, y: f32, w: f32, h: f32, work_w: f32, work_h: f32) -> Anchor {
    let left = x;
    let right = work_w - (x + w);
    let top = y;
    let bottom = work_h - (y + h);
    let hcenter = (x + w / 2.0 - work_w / 2.0).abs();
    let vcenter = (y + h / 2.0 - work_h / 2.0).abs();
    let h_side = if hcenter < left.min(right) * 0.5 {
        0 // centre
    } else if left <= right {
        1
    } else {
        2
    };
    let v_side = if vcenter < top.min(bottom) * 0.5 {
        0
    } else if top <= bottom {
        1
    } else {
        2
    };
    match (h_side, v_side) {
        (1, 1) => Anchor::LeftTop,
        (1, 2) => Anchor::LeftBottom,
        (1, 0) => Anchor::LeftVCenter,
        (2, 1) => Anchor::RightTop,
        (2, 2) => Anchor::RightBottom,
        (2, 0) => Anchor::RightVCenter,
        (0, 1) => Anchor::HCenterTop,
        (0, 2) => Anchor::HCenterBottom,
        _ => Anchor::Center,
    }
}

/// Converts a physical rectangle on `work` into monitor-relative DIPs.
pub fn normalize(rect: PxRect, work: &WorkArea) -> NormGeometry {
    let s = work.scale();
    let x = (rect.left - work.left) as f32 / s;
    let y = (rect.top - work.top) as f32 / s;
    let w = (rect.width() as f32 / s).clamp(MIN_W_DIP, MAX_DIP);
    let h = (rect.height() as f32 / s).clamp(MIN_H_DIP, MAX_DIP);
    let work_w = work.width_dip();
    let work_h = work.height_dip();
    NormGeometry {
        monitor: work.device_path.clone(),
        x,
        y,
        w,
        h,
        work_w,
        work_h,
        anchor: pick_anchor(x, y, w, h, work_w, work_h),
    }
}

/// Converts saved DIP geometry back into physical pixels on `work`, keeping the anchored edge
/// distances when the work area changed size, then clamping into the work area.
pub fn denormalize(geo: &NormGeometry, work: &WorkArea) -> PxRect {
    let s = work.scale();
    let work_w = work.width_dip();
    let work_h = work.height_dip();
    let w = geo.w.clamp(MIN_W_DIP, MAX_DIP).min(work_w.max(MIN_W_DIP));
    let h = geo.h.clamp(MIN_H_DIP, MAX_DIP).min(work_h.max(MIN_H_DIP));

    let right_gap = geo.work_w - (geo.x + geo.w);
    let bottom_gap = geo.work_h - (geo.y + geo.h);
    let x = match geo.anchor {
        Anchor::LeftTop | Anchor::LeftBottom | Anchor::LeftVCenter => geo.x,
        Anchor::RightTop | Anchor::RightBottom | Anchor::RightVCenter => work_w - right_gap - w,
        Anchor::HCenterTop | Anchor::HCenterBottom | Anchor::Center => {
            (geo.x + geo.w / 2.0) / geo.work_w.max(1.0) * work_w - w / 2.0
        }
    };
    let y = match geo.anchor {
        Anchor::LeftTop | Anchor::RightTop | Anchor::HCenterTop => geo.y,
        Anchor::LeftBottom | Anchor::RightBottom | Anchor::HCenterBottom => work_h - bottom_gap - h,
        Anchor::LeftVCenter | Anchor::RightVCenter | Anchor::Center => {
            (geo.y + geo.h / 2.0) / geo.work_h.max(1.0) * work_h - h / 2.0
        }
    };
    let x = x.clamp(0.0, (work_w - w).max(0.0));
    let y = y.clamp(0.0, (work_h - h).max(0.0));

    let left = work.left + (x * s).round() as i32;
    let top = work.top + (y * s).round() as i32;
    PxRect {
        left,
        top,
        right: left + (w * s).round() as i32,
        bottom: top + (h * s).round() as i32,
    }
}

/// Chooses the work area to place `geo` on: exact device path, else the primary/first.
pub fn resolve_work_area<'a>(geo: &NormGeometry, areas: &'a [WorkArea]) -> Option<&'a WorkArea> {
    areas
        .iter()
        .find(|a| a.device_path == geo.monitor)
        .or_else(|| areas.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn work(dpi: u32, w: i32, h: i32) -> WorkArea {
        WorkArea {
            device_path: "m".into(),
            left: 0,
            top: 0,
            right: w,
            bottom: h,
            dpi,
            mon_left: 0,
            mon_top: 0,
            mon_right: w,
            mon_bottom: h,
        }
    }

    #[test]
    fn roundtrip_same_monitor() {
        let wa = work(192, 3840, 2100);
        let r = PxRect {
            left: 100,
            top: 200,
            right: 740,
            bottom: 720,
        };
        let g = normalize(r, &wa);
        assert_eq!(g.anchor, Anchor::LeftTop);
        assert_eq!(denormalize(&g, &wa), r);
    }

    #[test]
    fn right_anchored_fence_stays_right_when_work_area_widens() {
        let wa = work(96, 1920, 1040);
        let r = PxRect {
            left: 1920 - 16 - 320,
            top: 16,
            right: 1920 - 16,
            bottom: 400,
        };
        let g = normalize(r, &wa);
        assert_eq!(g.anchor, Anchor::RightTop);
        let wide = work(96, 2560, 1392);
        let r2 = denormalize(&g, &wide);
        assert_eq!(r2.right, 2560 - 16);
        assert_eq!(r2.top, 16);
        assert_eq!(r2.width(), 320);
    }

    #[test]
    fn dpi_change_scales_pixels_not_dips() {
        let wa = work(96, 1920, 1040);
        let r = PxRect {
            left: 100,
            top: 100,
            right: 400,
            bottom: 300,
        };
        let g = normalize(r, &wa);
        let hi = work(144, 2880, 1560);
        let r2 = denormalize(&g, &hi);
        assert_eq!(r2.left, 150);
        assert_eq!(r2.width(), 450);
    }

    #[test]
    fn clamps_into_work_area_and_min_size() {
        let wa = work(96, 800, 600);
        let g = NormGeometry {
            monitor: "m".into(),
            x: 5000.0,
            y: -50.0,
            w: 10.0,
            h: 10.0,
            work_w: 1920.0,
            work_h: 1040.0,
            anchor: Anchor::LeftTop,
        };
        let r = denormalize(&g, &wa);
        assert!(r.left >= 0 && r.right <= 800 && r.top >= 0 && r.bottom <= 600);
        assert_eq!(r.width(), MIN_W_DIP as i32);
        assert_eq!(r.height(), MIN_H_DIP as i32);
    }
}
