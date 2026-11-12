//! Fence geometry normalization between physical pixels and per-monitor DIPs (plan §7.4).

use crate::model::{Anchor, Fence, FenceId, NormGeometry};

pub const MIN_W_DIP: f32 = 64.0;
pub const MIN_H_DIP: f32 = 36.0;
pub const MAX_DIP: f32 = 8192.0;
/// The widest gap to a screen edge that a fence cut to fit a smaller work area keeps.
const EDGE_GAP_MAX_DIP: f32 = 16.0;

/// A monitor's work area in physical pixels plus its DPI.
#[derive(Clone, Debug, PartialEq)]
pub struct WorkArea {
    /// The monitor's id: its device instance (`GSM7787#5&2C948443&0&UID24832`), else its EDID
    /// model and connector (`GSM7787#5.0`), else the GDI name.
    pub device_path: String,
    /// GDI name (`\\.\DISPLAY1`): all that files from 0.1.3 and earlier saved.
    pub gdi_name: String,
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
    pub fn model(&self) -> Option<&str> {
        monitor_model(&self.device_path)
    }
    /// Monitor resolution in physical px (taskbar included).
    pub fn screen_px(&self) -> [i32; 2] {
        [
            self.mon_right - self.mon_left,
            self.mon_bottom - self.mon_top,
        ]
    }
}

/// `GSM7787` from a monitor id (see [`WorkArea::device_path`]); `None` for a GDI name.
pub fn monitor_model(id: &str) -> Option<&str> {
    if id.starts_with('\\') {
        return None;
    }
    id.split_once('#')
        .map(|(model, _)| model)
        .filter(|m| !m.is_empty())
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
    let right_gap = geo.work_w - (geo.x + geo.w);
    let bottom_gap = geo.work_h - (geo.y + geo.h);
    // A fence longer than the work area is cut to fit. One that stood a small gap from an edge
    // (the first-run column, a fence snapped to the screen edge) keeps the smaller of its two
    // gaps at both ends instead of touching the screen edge and the taskbar.
    let edge_gap = |near: f32, far: f32| {
        let small = |g: &f32| (0.0..=EDGE_GAP_MAX_DIP).contains(g);
        [near, far]
            .into_iter()
            .filter(small)
            .fold(None, |a: Option<f32>, g| Some(a.map_or(g, |a| a.min(g))))
            .unwrap_or(0.0)
    };
    let (gap_x, gap_y) = (edge_gap(geo.x, right_gap), edge_gap(geo.y, bottom_gap));
    let fit = |size: f32, min: f32, room: f32, gap: f32| {
        let size = size.clamp(min, MAX_DIP);
        if size > room {
            (room - 2.0 * gap).max(min).min(room.max(min))
        } else {
            size
        }
    };
    let w = fit(geo.w, MIN_W_DIP, work_w, gap_x);
    let h = fit(geo.h, MIN_H_DIP, work_h, gap_y);

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
    // Cut to fit: between its two gaps.
    let x = if geo.w > work_w { gap_x } else { x };
    let y = if geo.h > work_h { gap_y } else { y };
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

/// Which side of each axis an anchor holds on to: 0 the middle, 1 the left or top edge, 2 the
/// right or bottom edge (horizontal, vertical).
fn anchor_sides(anchor: Anchor) -> (u8, u8) {
    let h = match anchor {
        Anchor::LeftTop | Anchor::LeftBottom | Anchor::LeftVCenter => 1,
        Anchor::RightTop | Anchor::RightBottom | Anchor::RightVCenter => 2,
        Anchor::HCenterTop | Anchor::HCenterBottom | Anchor::Center => 0,
    };
    let v = match anchor {
        Anchor::LeftTop | Anchor::RightTop | Anchor::HCenterTop => 1,
        Anchor::LeftBottom | Anchor::RightBottom | Anchor::HCenterBottom => 2,
        Anchor::LeftVCenter | Anchor::RightVCenter | Anchor::Center => 0,
    };
    (h, v)
}

/// Where a span `size` long at `pos` goes when its axis changes from `old` to `new` long, held
/// by `side` (see [`anchor_sides`]), as [`denormalize`] moves one fence. A span longer than the
/// new axis keeps its distance to the side it is held by.
fn carry_axis(side: u8, pos: f32, size: f32, old: f32, new: f32) -> f32 {
    let p = match side {
        1 => pos,
        2 => new - (old - (pos + size)) - size,
        _ => (pos + size / 2.0) / old.max(1.0) * new - size / 2.0,
    };
    if size <= new {
        p.clamp(0.0, new - size)
    } else {
        p
    }
}

/// Fences that stand one snapping gap (`gap` DIPs) apart, side by side or one above the other,
/// are a group: the first-run column, two fences snapped together. [`denormalize`] places each
/// fence by its own anchor, so on a work area of another size the members nearest to different
/// edges, or to the middle, part: the gap between them grows or they overlap. This re-places
/// the fences `ids` that are in such a group for the work area their monitor has now: the
/// group moves as one piece, by the anchor of its bounding box, and every member keeps its
/// place in it (a member that does not fit is cut by [`denormalize`] as before). Returns how
/// many fences got a new geometry.
pub fn keep_groups_together(
    fences: &mut [Fence],
    ids: &[FenceId],
    areas: &[WorkArea],
    gap: f32,
) -> usize {
    // Half a pixel at 150 % and the rounding of a fractional gap.
    const TOLERANCE: f32 = 0.75;
    let members: Vec<usize> = fences
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            ids.contains(&f.id)
                && f.tab_host.is_none()
                && !f.desktop_layout
                && areas.iter().any(|a| a.device_path == f.geometry.monitor)
        })
        .map(|(i, _)| i)
        .collect();
    let same_work = |a: &NormGeometry, b: &NormGeometry| {
        a.monitor == b.monitor
            && (a.work_w - b.work_w).abs() < 1.0
            && (a.work_h - b.work_h).abs() < 1.0
    };
    let beside = |a: &NormGeometry, b: &NormGeometry| {
        (b.x - (a.x + a.w) - gap).abs() <= TOLERANCE && a.y < b.y + b.h && b.y < a.y + a.h
    };
    let above = |a: &NormGeometry, b: &NormGeometry| {
        (b.y - (a.y + a.h) - gap).abs() <= TOLERANCE && a.x < b.x + b.w && b.x < a.x + a.w
    };
    // Group label per member: snapped pairs take the same one.
    let mut label: Vec<usize> = (0..members.len()).collect();
    for i in 0..members.len() {
        for j in i + 1..members.len() {
            let (a, b) = (&fences[members[i]].geometry, &fences[members[j]].geometry);
            let snapped =
                same_work(a, b) && (beside(a, b) || beside(b, a) || above(a, b) || above(b, a));
            if snapped && label[i] != label[j] {
                let (keep, drop) = (label[i], label[j]);
                for l in &mut label {
                    if *l == drop {
                        *l = keep;
                    }
                }
            }
        }
    }
    let mut moved = 0;
    for group in 0..members.len() {
        let of_group: Vec<usize> = (0..members.len())
            .filter(|&m| label[m] == group)
            .map(|m| members[m])
            .collect();
        if of_group.len() < 2 {
            continue;
        }
        let first = &fences[of_group[0]].geometry;
        let Some(work) = areas.iter().find(|a| a.device_path == first.monitor) else {
            continue;
        };
        let (old_w, old_h) = (first.work_w, first.work_h);
        let (new_w, new_h) = (work.width_dip(), work.height_dip());
        if (old_w - new_w).abs() < 0.5 && (old_h - new_h).abs() < 0.5 {
            continue;
        }
        let geos = || of_group.iter().map(|&i| &fences[i].geometry);
        let x0 = geos().map(|g| g.x).fold(f32::MAX, f32::min);
        let y0 = geos().map(|g| g.y).fold(f32::MAX, f32::min);
        let x1 = geos().map(|g| g.x + g.w).fold(f32::MIN, f32::max);
        let y1 = geos().map(|g| g.y + g.h).fold(f32::MIN, f32::max);
        let (h_side, v_side) = anchor_sides(pick_anchor(x0, y0, x1 - x0, y1 - y0, old_w, old_h));
        let dx = carry_axis(h_side, x0, x1 - x0, old_w, new_w) - x0;
        let dy = carry_axis(v_side, y0, y1 - y0, old_h, new_h) - y0;
        for &i in &of_group {
            let g = &mut fences[i].geometry;
            g.x += dx;
            g.y += dy;
            g.work_w = new_w;
            g.work_h = new_h;
            g.anchor = pick_anchor(g.x, g.y, g.w, g.h, new_w, new_h);
            moved += 1;
        }
    }
    moved
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
            gdi_name: String::new(),
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

    fn fence_at(wa: &WorkArea, left: i32, top: i32, w: i32, h: i32) -> Fence {
        let r = PxRect {
            left,
            top,
            right: left + w,
            bottom: top + h,
        };
        Fence::new("f", crate::model::FenceKind::Virtual, normalize(r, wa))
    }

    /// Places `fences` on `wa` as the app does for a new monitor setup: groups first, then each
    /// fence by its own geometry, saved again on that work area.
    fn carry_to(fences: &mut [Fence], wa: &WorkArea) -> Vec<PxRect> {
        let ids: Vec<FenceId> = fences.iter().map(|f| f.id).collect();
        keep_groups_together(fences, &ids, std::slice::from_ref(wa), 8.0);
        fences
            .iter_mut()
            .map(|f| {
                let r = denormalize(&f.geometry, wa);
                f.geometry = normalize(r, wa);
                r
            })
            .collect()
    }

    /// The first-run column (two fences one above the other at the right edge) at 100 %, then
    /// 150 %, then 125 % on a 1600x900 screen: the lower fence is nearer the middle of the
    /// 150 % work area and was carried on by that, 75 px below the upper one at 125 %.
    #[test]
    fn a_snapped_column_keeps_its_gap_over_two_scale_changes() {
        let at100 = work(96, 1600, 852);
        let mut fences = vec![
            fence_at(&at100, 1352, 8, 240, 150),
            fence_at(&at100, 1352, 166, 240, 150),
        ];
        let at150 = work(144, 1600, 828);
        let r = carry_to(&mut fences, &at150);
        assert_eq!(r[1].top - r[0].bottom, 12, "8 DIPs at 150 %: {r:?}");
        let at125 = work(120, 1600, 840);
        let r = carry_to(&mut fences, &at125);
        assert_eq!(r[1].top - r[0].bottom, 10, "8 DIPs at 125 %: {r:?}");
        assert_eq!(r[0].left, r[1].left);
        assert_eq!(r[0].right, 1600 - 10, "still 8 DIPs from the right edge");
        let big = work(96, 1920, 1032);
        let r = carry_to(&mut fences, &big);
        assert_eq!(r[1].top - r[0].bottom, 8, "{r:?}");
        assert_eq!((r[0].top, r[0].right), (8, 1920 - 8));
    }

    /// Two fences side by side left of the middle: on a narrower work area the right one was
    /// placed by the middle and ended 101 DIPs from its neighbour one change later.
    #[test]
    fn a_snapped_pair_side_by_side_stays_together() {
        let at100 = work(96, 1600, 852);
        let mut fences = vec![
            fence_at(&at100, 8, 8, 300, 240),
            fence_at(&at100, 316, 8, 300, 240),
            // Not snapped to anything: placed by its own anchor as before.
            fence_at(&at100, 660, 400, 260, 160),
        ];
        let alone = denormalize(&fences[2].geometry, &work(144, 1600, 828));
        let r = carry_to(&mut fences, &work(144, 1600, 828));
        assert_eq!(r[1].left - r[0].right, 12);
        assert_eq!(r[2], alone);
        let r = carry_to(&mut fences, &work(120, 1600, 840));
        assert_eq!(r[1].left - r[0].right, 10);
        assert_eq!(r[0].left, 10);
    }

    /// A group larger than the new work area keeps the edge it is held by; only the member
    /// that does not fit is cut.
    #[test]
    fn a_group_taller_than_the_work_area_keeps_its_top() {
        let at100 = work(96, 1600, 852);
        let mut fences = vec![
            fence_at(&at100, 1352, 8, 240, 150),
            fence_at(&at100, 1352, 166, 240, 150),
            fence_at(&at100, 1104, 8, 240, 822),
        ];
        let r = carry_to(&mut fences, &work(96, 1280, 672));
        assert_eq!((r[0].top, r[1].top), (8, 166));
        assert_eq!(r[0].left - r[2].right, 8);
        assert_eq!((r[2].top, r[2].bottom), (8, 672 - 8));
    }

    /// A fence 8 px under the top edge and taller than the new work area is cut to fit with
    /// 8 px at both ends (it took the whole height, from the screen edge to the taskbar); one
    /// that stood far from both edges, or flush with one, fills the height as before.
    #[test]
    fn a_fence_cut_to_fit_keeps_its_small_gap_to_the_edge() {
        let at100 = work(96, 1600, 852);
        let small = work(96, 1280, 672);
        let tall = normalize(
            PxRect {
                left: 1104,
                top: 8,
                right: 1344,
                bottom: 830,
            },
            &at100,
        );
        let r = denormalize(&tall, &small);
        assert_eq!((r.top, r.bottom), (8, 664));
        assert_eq!(r.width(), 240);
        // 150 %: 8 DIPs are 12 px.
        let r = denormalize(&tall, &work(144, 1600, 828));
        assert_eq!((r.top, r.bottom), (12, 816));
        // Nearer the bottom edge: the same gap at both ends.
        let low = normalize(
            PxRect {
                left: 8,
                top: 40,
                right: 248,
                bottom: 844,
            },
            &at100,
        );
        let r = denormalize(&low, &small);
        assert_eq!((r.top, r.bottom), (8, 664));
        let flush = normalize(
            PxRect {
                left: 8,
                top: 0,
                right: 248,
                bottom: 800,
            },
            &at100,
        );
        let r = denormalize(&flush, &small);
        assert_eq!((r.top, r.bottom), (0, 672));
        let middle = normalize(
            PxRect {
                left: 8,
                top: 60,
                right: 248,
                bottom: 800,
            },
            &at100,
        );
        let r = denormalize(&middle, &small);
        assert_eq!((r.top, r.bottom), (0, 672));
    }

    /// Fences on the work area they were saved on, and fences not asked for, stay as they are.
    #[test]
    fn groups_on_an_unchanged_work_area_are_left_alone() {
        let wa = work(96, 1600, 852);
        let mut fences = vec![
            fence_at(&wa, 8, 8, 300, 240),
            fence_at(&wa, 316, 8, 300, 240),
        ];
        let before: Vec<NormGeometry> = fences.iter().map(|f| f.geometry.clone()).collect();
        let ids: Vec<FenceId> = fences.iter().map(|f| f.id).collect();
        assert_eq!(
            keep_groups_together(&mut fences, &ids, std::slice::from_ref(&wa), 8.0),
            0
        );
        let small = work(96, 1280, 672);
        assert_eq!(
            keep_groups_together(&mut fences, &[], std::slice::from_ref(&small), 8.0),
            0
        );
        let after: Vec<NormGeometry> = fences.iter().map(|f| f.geometry.clone()).collect();
        assert_eq!(before, after);
    }
}
