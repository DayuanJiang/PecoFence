//! Cached Liquid Glass optics over the monitor wallpaper.
//!
//! A rounded-box distance field controls a curved bezel, separate RGB samples provide
//! restrained dispersion, and a gentle dome continues into the interior. This is
//! an original implementation informed by the open-source survey in
//! docs/LIQUID-GLASS-RESEARCH.md. It is an optical approximation, not a physical ray tracer.
//! `gpu_glass` caches geometry independently of wallpaper position and evaluates the
//! material through D2D effects. The CPU path remains for comparison renders.

use crate::theme::{Theme, with_alpha};
use crate::{Image, MonitorBackdrop};
use windows_canvas::{ColorF, DrawingSession, GradientStop, Rect, RoundedRect, Vector2};
use windows_core::Result;

/// Crisp reflections are drawn at device resolution over the cached optical background.
/// Hover raises the top light with the application's existing animation clock.
pub fn draw_reflection(
    session: &DrawingSession<'_>,
    theme: &Theme,
    scale: f32,
    width: f32,
    height: f32,
    hover: f32,
    opacity: f32,
) -> Result<()> {
    let radius = theme.corner_radius.min(width.min(height) * 0.5).max(0.0);
    let light = 0.85 + 0.15 * opacity.min(1.0) + hover.clamp(0.0, 1.0) * 0.18;
    let white = ColorF::new(1.0, 1.0, 1.0, 1.0);
    let clear = ColorF::TRANSPARENT;
    let wash = session.create_linear_gradient(
        Vector2::new(0.0, 0.0),
        Vector2::new(width * 0.3, height.max(1.0)),
        &[
            GradientStop::new(0.0, with_alpha(white, 0.018 * light)),
            GradientStop::new(0.18, clear),
            GradientStop::new(0.88, clear),
            GradientStop::new(1.0, with_alpha(white, 0.008)),
        ],
    )?;
    session.fill_rounded_rect(
        &RoundedRect::uniform(Rect::from_xywh(0.0, 0.0, width, height), radius),
        &wash,
    );
    let hair = 1.0 / scale.max(0.5);
    // Broad reflections are part of the continuous optical surface below. Keep the
    // sharp outer reflection here, without a second closed contour inside the plate.
    for (inset, alpha) in [(0.5 * hair, 1.0), (1.5 * hair, 0.24)] {
        if width <= inset * 2.0 || height <= inset * 2.0 {
            continue;
        }
        let rim = session.create_linear_gradient(
            Vector2::new(0.0, 0.0),
            Vector2::new(width * 0.7, height.max(1.0)),
            &[
                GradientStop::new(0.0, with_alpha(theme.glass_rim_top, alpha * light)),
                GradientStop::new(0.34, with_alpha(white, alpha * 0.12)),
                GradientStop::new(0.64, ColorF::new(0.1, 0.12, 0.15, alpha * 0.18)),
                GradientStop::new(1.0, with_alpha(white, alpha * 0.65)),
            ],
        )?;
        session.draw_rounded_rect(
            &RoundedRect::uniform(
                Rect::from_xywh(inset, inset, width - inset * 2.0, height - inset * 2.0),
                (radius - inset).max(0.0),
            ),
            &rim,
            hair,
        );
    }
    Ok(())
}

/// Optical parameters in DIPs, independent of monitor resolution and crop downsampling.
#[derive(Clone, Copy, Debug)]
pub struct GlassOptics {
    pub radius: f32,
    pub bezel: f32,
    pub refraction: f32,
    pub dispersion: f32,
}

impl Default for GlassOptics {
    fn default() -> Self {
        Self {
            radius: 8.0,
            bezel: 20.0,
            refraction: 16.0,
            dispersion: 0.045,
        }
    }
}

/// Rounded-rectangle outward normal and distance into the glass, in DIPs.
fn edge(x: f32, y: f32, w: f32, h: f32, radius: f32) -> (f32, f32, f32) {
    let radius = radius.min(w.min(h) * 0.5).max(0.0);
    let dx = x - w * 0.5;
    let dy = y - h * 0.5;
    let qx = dx.abs() - (w * 0.5 - radius);
    let qy = dy.abs() - (h * 0.5 - radius);
    let ox = qx.max(0.0);
    let oy = qy.max(0.0);
    let len = (ox * ox + oy * oy).sqrt();
    let depth = -(len + qx.max(qy).min(0.0) - radius);
    let (nx, ny) = if len > 0.001 {
        (dx.signum() * ox / len, dy.signum() * oy / len)
    } else if qx > qy {
        (dx.signum(), 0.0)
    } else {
        (0.0, dy.signum())
    };
    (nx, ny, depth)
}

fn smootherstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}

/// `bezel` is the strong curved edge; `reach` includes its gentle continuation into
/// the face. Displacement and coverage share a smooth endpoint at the true flat centre.
struct LensProfile {
    reach: f32,
    inverse_bend_width: f32,
    inverse_coverage_width: f32,
    inverse_tail_width: f32,
    inverse_light_width: f32,
}

impl LensProfile {
    fn new(bezel: f32, half_extent: f32) -> Self {
        let bezel = bezel.max(0.01).min(half_extent.max(0.01));
        let reach = (bezel * 2.0).min(half_extent.max(0.01));
        Self {
            reach,
            inverse_bend_width: (bezel * 1.3).recip(),
            inverse_coverage_width: (bezel * 1.5).recip(),
            inverse_tail_width: (reach - bezel.min(reach * 0.5)).recip(),
            inverse_light_width: (bezel * 0.4).recip(),
        }
    }

    fn weights(&self, depth: f32) -> (f32, f32) {
        let depth = depth.max(0.0);
        if depth >= self.reach {
            return (0.0, 0.0);
        }
        let envelope = smootherstep((self.reach - depth) * self.inverse_tail_width);
        let bend_distance = depth * self.inverse_bend_width;
        let bend = (1.0 + bend_distance * bend_distance).recip().powi(2) * envelope;
        let coverage_distance = depth * self.inverse_coverage_width;
        let coverage = (1.0 + coverage_distance * coverage_distance).recip() * envelope;
        (bend, coverage)
    }
}

/// The surface bends rays inward. Deeper in the face, blend the edge normal into
/// a smooth centre direction before the rounded-box normal becomes discontinuous.
fn inward_direction(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    nx: f32,
    ny: f32,
    depth: f32,
) -> (f32, f32) {
    let radius = radius.min(w.min(h) * 0.5);
    let blend = if radius <= 0.01 {
        1.0
    } else {
        smootherstep(depth * 2.0 / radius - 1.0)
    };
    if blend == 0.0 {
        return (-nx, -ny);
    }
    let (cx, cy) = (w * 0.5 - x, h * 0.5 - y);
    let length = (cx * cx + cy * cy).sqrt();
    if length < 0.001 {
        return (0.0, 0.0);
    }
    let inverse_length = length.recip();
    if blend == 1.0 {
        return (cx * inverse_length, cy * inverse_length);
    }
    let dx = -nx * (1.0 - blend) + cx * inverse_length * blend;
    let dy = -ny * (1.0 - blend) + cy * inverse_length * blend;
    let inverse_length = (dx * dx + dy * dy).sqrt().max(0.001).recip();
    (dx * inverse_length, dy * inverse_length)
}

/// Geometry-only GPU inputs. Built on size/DPI changes, never on window movement.
pub(crate) struct DisplacementField {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<f32>,
    pub glints: Vec<u8>,
    pub scale_px: f32,
}

pub(crate) fn displacement_field(
    width: u32,
    height: u32,
    scale: f32,
    radius: f32,
) -> DisplacementField {
    let scale = scale.max(0.5);
    let (w, h) = (width as f32 / scale, height as f32 / scale);
    let optics = GlassOptics {
        radius,
        ..Default::default()
    };
    let profile = LensProfile::new(optics.bezel, w.min(h) * 0.5);
    let strength = optics.refraction.min(w.min(h) * 0.18);
    // A light dome affects the whole interior, while the strong bezel remains intact.
    let dome_strength = w.min(h) * 0.012;
    let scale_px = (2.0 * (strength + dome_strength) * scale).max(1.0);
    // A smooth vector field needs no device-pixel rasterization. One float texel per
    // 2 DIP retains subpixel displacement and keeps animated resizes inexpensive.
    let map_w = (w / 2.0).ceil().max(2.0) as u32;
    let map_h = (h / 2.0).ceil().max(2.0) as u32;
    let mut rgba = Vec::with_capacity(map_w as usize * map_h as usize * 4);
    let mut glints = Vec::with_capacity(map_w as usize * map_h as usize * 4);
    for y in 0..map_h {
        let py = (y as f32 + 0.5) * h / map_h as f32;
        for x in 0..map_w {
            let px = (x as f32 + 0.5) * w / map_w as f32;
            let (nx, ny, depth) = edge(px, py, w, h, radius);
            let (weight, coverage) = profile.weights(depth);
            let (ix, iy) = inward_direction(px, py, w, h, radius, nx, ny, depth);
            let (u, v) = (px * 2.0 / w - 1.0, py * 2.0 / h - 1.0);
            let dome =
                (1.0 - u * u).max(0.0) * (1.0 - v * v).max(0.0) * (1.0 - weight) * dome_strength;
            let dx = (ix * strength * weight - u * dome) * scale;
            let dy = (iy * strength * weight - v * dome) * scale;
            rgba.extend_from_slice(&[0.5 + dx / scale_px, 0.5 + dy / scale_px, 0.5, 1.0]);
            let facing = ix * 0.55 + iy * 0.83;
            let light_rolloff = (1.0 + depth.max(0.0) * profile.inverse_light_width)
                .recip()
                .powi(2);
            let glint = if depth > 0.0 {
                (0.22 * facing.max(0.0).powi(2) + 0.10 * (-facing).max(0.0).powi(2))
                    * light_rolloff
                    * coverage
                    * 1.18
            } else {
                0.0
            };
            let alpha = (glint * 255.0).round().clamp(0.0, 255.0) as u8;
            glints.extend_from_slice(&[alpha; 4]);
        }
    }
    DisplacementField {
        width: map_w,
        height: map_h,
        rgba,
        glints,
        scale_px,
    }
}

/// A small, position-dependent contrast sample. No full-window CPU crop is needed by
/// the GPU material; the text still adapts as the fence crosses light/dark wallpaper.
pub fn foreground_sample(backdrops: &[MonitorBackdrop], rect: [i32; 4]) -> Image {
    if backdrops
        .iter()
        .all(|b| b.image.width == 0 || b.image.height == 0)
    {
        return Image::default();
    }
    let mut bgra = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            bgra.extend_from_slice(&sample_desktop(
                backdrops,
                rect[0] as f32 + (x as f32 + 0.5) * rect[2] as f32 / 16.0,
                rect[1] as f32 + (y as f32 + 0.5) * rect[3] as f32 / 16.0,
            ));
        }
    }
    Image {
        width: 16,
        height: 16,
        bgra,
    }
}

/// Only the curved bezel needs a wallpaper image. The flat centre is real transparency:
/// copying unchanged wallpaper there makes it travel with the HWND until the asynchronous
/// composition surface catches up, producing a full-panel wobble during dragging.
///
/// Keep the opaque crop separately for foreground contrast sampling. This premultiplied
/// overlay is uploaded once per crop. Keep the full-strength outer glass edge and blend
/// smoothly across the wide bezel and its soft inner shoulder into the clear centre.
pub fn refracted_overlay(image: &Image, width_dip: f32, height_dip: f32, radius: f32) -> Image {
    let mut overlay = Image {
        width: image.width,
        height: image.height,
        bgra: vec![0; image.bgra.len()],
    };
    if width_dip <= 0.0 || height_dip <= 0.0 || image.width == 0 || image.height == 0 {
        return overlay;
    }
    let profile = LensProfile::new(
        GlassOptics::default().bezel,
        width_dip.min(height_dip) * 0.5,
    );
    let pixel_w = width_dip / image.width as f32;
    let pixel_h = height_dip / image.height as f32;
    let margin_x = (radius.max(profile.reach) / pixel_w).ceil() as u32;
    let margin_y = (radius.max(profile.reach) / pixel_h).ceil() as u32;
    for y in 0..image.height {
        let inner_row = y >= margin_y && y < image.height.saturating_sub(margin_y);
        for x in 0..image.width {
            if inner_row && x >= margin_x && x < image.width.saturating_sub(margin_x) {
                continue;
            }
            let px = (x as f32 + 0.5) * pixel_w;
            let py = (y as f32 + 0.5) * pixel_h;
            let (nx, ny, depth) = edge(px, py, width_dip, height_dip, radius);
            if depth <= 0.0 || depth >= profile.reach {
                continue;
            }
            let (_, alpha) = profile.weights(depth);
            let (ix, iy) = inward_direction(px, py, width_dip, height_dip, radius, nx, ny, depth);
            let facing = ix * 0.55 + iy * 0.83;
            let light_rolloff = (1.0 + depth * profile.inverse_light_width).recip().powi(2);
            // Continuous directional glints, not alternating bright/dark inset rings.
            let glint = (0.22 * facing.max(0.0).powi(2) + 0.10 * (-facing).max(0.0).powi(2))
                * light_rolloff;
            let i = ((y * image.width + x) * 4) as usize;
            for c in 0..3 {
                let source = image.bgra[i + c] as f32;
                let reflected = source + (image.bgra[i + 3] as f32 - source) * glint;
                overlay.bgra[i + c] = (reflected * alpha).round() as u8;
            }
            overlay.bgra[i + 3] = (image.bgra[i + 3] as f32 * alpha).round() as u8;
        }
    }
    overlay
}

/// A complete plate, including portions crossing a monitor edge. Sampling is clamped at
/// the source edge instead of shortening/stretching the crop. Fractional source coordinates
/// preserve wallpaper alignment when dragging at non-integral DPI scales.
pub fn crop(
    backdrop: &MonitorBackdrop,
    rect: [i32; 4],
    dpi_scale: f32,
    optics: GlassOptics,
) -> Image {
    crop_with_neighbors(backdrop, &[], rect, dpi_scale, optics)
}

/// A fence can straddle displays or refract pixels beyond its own display. Keep the lens
/// geometry continuous and resolve each displaced sample in virtual-desktop coordinates.
pub fn crop_from_monitors(
    backdrops: &[MonitorBackdrop],
    rect: [i32; 4],
    dpi_scale: f32,
    optics: GlassOptics,
) -> Image {
    let cx = rect[0] as f32 + rect[2] as f32 * 0.5;
    let cy = rect[1] as f32 + rect[3] as f32 * 0.5;
    match nearest_monitor(backdrops, cx, cy) {
        Some(backdrop) => crop_with_neighbors(backdrop, backdrops, rect, dpi_scale, optics),
        None => Image::default(),
    }
}

fn nearest_monitor(backdrops: &[MonitorBackdrop], x: f32, y: f32) -> Option<&MonitorBackdrop> {
    backdrops
        .iter()
        .filter(|b| b.image.width > 0 && b.image.height > 0)
        .min_by(|a, b| {
            let distance = |m: &MonitorBackdrop| {
                let dx = (m.left as f32 - x)
                    .max(0.0)
                    .max(x - (m.left + m.width) as f32);
                let dy = (m.top as f32 - y)
                    .max(0.0)
                    .max(y - (m.top + m.height) as f32);
                dx * dx + dy * dy
            };
            distance(a).total_cmp(&distance(b))
        })
}

fn sample_desktop(backdrops: &[MonitorBackdrop], x: f32, y: f32) -> [u8; 4] {
    let point = |x: f32, y: f32| {
        let b = nearest_monitor(backdrops, x, y).unwrap();
        let ds = b.downscale.max(1) as f32;
        b.image.sample(
            (x - b.left as f32) / ds - 0.5,
            (y - b.top as f32) / ds - 0.5,
        )
    };
    let b = nearest_monitor(backdrops, x, y).unwrap();
    let ds = b.downscale.max(1) as f32;
    let sx = (x - b.left as f32) / ds - 0.5;
    let sy = (y - b.top as f32) / ds - 0.5;
    if sx >= 0.0
        && sy >= 0.0
        && sx <= (b.image.width - 1) as f32
        && sy <= (b.image.height - 1) as f32
    {
        return b.image.sample(sx, sy);
    }
    // Bilinear footprints may themselves cross a monitor seam. Sample the four physical
    // pixel centers independently instead of clamping all taps to a single display.
    let x0 = (x - 0.5).floor() + 0.5;
    let y0 = (y - 0.5).floor() + 0.5;
    let (fx, fy) = (x - x0, y - y0);
    let (a, b, c, d) = (
        point(x0, y0),
        point(x0 + 1.0, y0),
        point(x0, y0 + 1.0),
        point(x0 + 1.0, y0 + 1.0),
    );
    std::array::from_fn(|i| {
        let top = a[i] as f32 * (1.0 - fx) + b[i] as f32 * fx;
        let bottom = c[i] as f32 * (1.0 - fx) + d[i] as f32 * fx;
        (top * (1.0 - fy) + bottom * fy).round() as u8
    })
}

fn crop_with_neighbors(
    backdrop: &MonitorBackdrop,
    backdrops: &[MonitorBackdrop],
    rect: [i32; 4],
    dpi_scale: f32,
    optics: GlassOptics,
) -> Image {
    let [left, top, width, height] = rect;
    if width <= 0 || height <= 0 || backdrop.image.width == 0 || backdrop.image.height == 0 {
        return Image::default();
    }
    let downscale = backdrop.downscale.max(1);
    let scale = dpi_scale.max(0.5);
    let out_w = (width as u32).div_ceil(downscale);
    let out_h = (height as u32).div_ceil(downscale);
    let w_dip = width as f32 / scale;
    let h_dip = height as f32 / scale;
    let pixel_w = w_dip / out_w as f32;
    let pixel_h = h_dip / out_h as f32;
    let profile = LensProfile::new(optics.bezel, w_dip.min(h_dip) * 0.5);
    // Short rolled plates must not fold their source coordinates back on themselves.
    let strength = optics.refraction.max(0.0).min(w_dip.min(h_dip) * 0.18);
    let dispersion = optics.dispersion.clamp(0.0, 0.3);
    let origin_x = (left as f32 - backdrop.left as f32) / downscale as f32;
    let origin_y = (top as f32 - backdrop.top as f32) / downscale as f32;
    let texels_per_dip = scale / downscale as f32;
    let mut bgra = Vec::with_capacity((out_w * out_h * 4) as usize);
    // At native source resolution the flat interior is an exact copy. Avoid distance-field
    // math and bilinear sampling for most pixels, particularly on high-DPI animated fences.
    let margin = (optics.radius.max(profile.reach) * scale).ceil() as u32;
    let source_x = left as i64 - backdrop.left as i64;
    let source_y = top as i64 - backdrop.top as i64;
    let copy_start = (margin as i64).max(-source_x).clamp(0, out_w as i64) as u32;
    let copy_end = (out_w.saturating_sub(margin) as i64)
        .min(backdrop.image.width as i64 - source_x)
        .clamp(0, out_w as i64) as u32;
    for y in 0..out_h {
        let py = (y as f32 + 0.5) * pixel_h;
        let row = source_y + y as i64;
        let copy_row = downscale == 1
            && y >= margin
            && y < out_h.saturating_sub(margin)
            && row >= 0
            && row < backdrop.image.height as i64
            && copy_start < copy_end;
        let mut x = 0;
        while x < out_w {
            if copy_row && x == copy_start {
                let begin = (row * backdrop.image.width as i64 + source_x + x as i64) as usize * 4;
                let len = (copy_end - copy_start) as usize * 4;
                bgra.extend_from_slice(&backdrop.image.bgra[begin..begin + len]);
                x = copy_end;
                continue;
            }
            let px = (x as f32 + 0.5) * pixel_w;
            let sx = origin_x + px * texels_per_dip - 0.5;
            let sy = origin_y + py * texels_per_dip - 0.5;
            let (nx, ny, depth) = edge(px, py, w_dip, h_dip, optics.radius);
            let (weight, _) = profile.weights(depth);
            let bend = strength * weight * texels_per_dip;
            let (nx, ny) = if bend > 0.0 {
                inward_direction(px, py, w_dip, h_dip, optics.radius, nx, ny, depth)
            } else {
                (0.0, 0.0)
            };
            let outside = |x: f32, y: f32| {
                !backdrops.is_empty()
                    && (x < 0.0
                        || y < 0.0
                        || x > (backdrop.image.width - 1) as f32
                        || y > (backdrop.image.height - 1) as f32)
            };
            let desktop = |x: f32, y: f32| {
                sample_desktop(
                    backdrops,
                    backdrop.left as f32 + (x + 0.5) * downscale as f32,
                    backdrop.top as f32 + (y + 0.5) * downscale as f32,
                )
            };
            let (gx, gy) = (sx + nx * bend, sy + ny * bend);
            let mut pixel = if outside(gx, gy) {
                desktop(gx, gy)
            } else {
                backdrop.image.sample(gx, gy)
            };
            if bend > 0.01 && dispersion > 0.0 {
                for (channel, factor) in [(2, 1.0 - dispersion), (0, 1.0 + dispersion)] {
                    let (x, y) = (sx + nx * bend * factor, sy + ny * bend * factor);
                    pixel[channel] = if outside(x, y) {
                        desktop(x, y)[channel]
                    } else {
                        backdrop.image.sample_channel(x, y, channel)
                    };
                }
            }
            bgra.extend_from_slice(&pixel);
            x += 1;
        }
    }
    Image {
        width: out_w,
        height: out_h,
        bgra,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_field_has_a_continuous_interior_and_preserves_the_bezel() {
        let field = displacement_field(800, 600, 2.0, 24.0);
        assert_eq!((field.width, field.height), (200, 150));
        let offset = |x: usize, y: usize| {
            let i = (y * field.width as usize + x) * 4;
            (
                (field.rgba[i] - 0.5) * field.scale_px,
                (field.rgba[i + 1] - 0.5) * field.scale_px,
            )
        };
        assert!(
            offset(50, 75).0 > 1.0,
            "the face must have gentle refraction too"
        );
        assert!(
            offset(1, 75).0 > 25.0,
            "the broad glass bezel must remain strong"
        );
        for y in 0..field.height as usize {
            for x in 0..field.width as usize {
                let a = offset(x, y);
                let b = offset(field.width as usize - 1 - x, field.height as usize - 1 - y);
                assert!((a.0 + b.0).abs() < 0.0001 && (a.1 + b.1).abs() < 0.0001);
            }
        }
        assert!(
            field
                .rgba
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        );
        let high_dpi = displacement_field(1600, 1200, 4.0, 24.0);
        assert_eq!(
            field.rgba, high_dpi.rgba,
            "DPI must scale rays, not change the surface"
        );
        assert_eq!(high_dpi.scale_px, 2.0 * field.scale_px);
    }

    #[test]
    fn curved_face_has_a_soft_tail_without_folding_the_background() {
        for half_extent in [8.0, 18.0, 24.0, 32.0, 74.5, 160.0] {
            let profile = LensProfile::new(20.0, half_extent);
            let strength = 16.0f32.min(half_extent * 2.0 * 0.18);
            let mut previous = strength;
            for i in 1..=1000 {
                let depth = profile.reach * i as f32 / 1000.0;
                let (bend, alpha) = profile.weights(depth);
                let source = depth + strength * bend;
                assert!(
                    source >= previous - 1e-4,
                    "background folded at depth={depth}"
                );
                assert!((0.0..=1.0).contains(&alpha));
                previous = source;
            }
            assert_eq!(profile.weights(profile.reach), (0.0, 0.0));
            assert_eq!(profile.weights(profile.reach + 1.0), (0.0, 0.0));
        }
        let profile = LensProfile::new(20.0, 75.0);
        assert!(
            profile.weights(20.0).0 > 0.3,
            "the face must not go flat at the bezel boundary"
        );
        assert!(
            profile.weights(30.0).0 > 0.0,
            "missing gentle inward continuation"
        );
    }

    #[test]
    fn deeper_corner_directions_do_not_form_a_diagonal_seam() {
        let direction = |x, y| {
            let (nx, ny, depth) = edge(x, y, 200.0, 200.0, 24.0);
            inward_direction(x, y, 200.0, 200.0, 24.0, nx, ny, depth)
        };
        let a = direction(26.01, 25.99);
        let b = direction(25.99, 26.01);
        assert!((a.0 - b.0).abs() < 0.001 && (a.1 - b.1).abs() < 0.001);
        assert!(
            a.0 > 0.0 && a.1 > 0.0,
            "convex refraction must point inward"
        );
    }

    #[test]
    fn wide_glass_keeps_its_strong_edge_and_blends_gradually_into_the_centre() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let image = Image::solid(
                (320.0 * scale) as u32,
                (120.0 * scale) as u32,
                [80, 140, 160],
            );
            let overlay = refracted_overlay(&image, 320.0, 120.0, 24.0);
            let alpha =
                |x: u32| overlay.bgra[((overlay.height / 2 * overlay.width + x) * 4 + 3) as usize];
            assert!(
                alpha(0) >= 250,
                "the strong outer glass edge must be preserved"
            );
            let band_px = (GlassOptics::default().bezel * scale).ceil() as u32;
            assert!(
                alpha(band_px / 4) > 200,
                "the broad bezel must retain its glass body"
            );
            assert!(
                alpha(band_px / 2) > 200,
                "the broad curved edge must keep its glass strength"
            );
            assert!(
                (140..=210).contains(&alpha(band_px)),
                "the curved face must continue past the former inner border"
            );
            assert!((30..=100).contains(&alpha(band_px * 3 / 2)));
            let reach_px = (LensProfile::new(20.0, 60.0).reach * scale).ceil() as u32;
            for x in 1..=reach_px {
                assert!(
                    alpha(x) <= alpha(x - 1),
                    "opacity must fall toward the centre"
                );
                assert!(alpha(x - 1) - alpha(x) <= 25, "visible inner opacity step");
            }
            for x in reach_px..overlay.width / 2 {
                assert_eq!(alpha(x), 0, "wallpaper fill leaked into the clear centre");
            }
        }
    }

    #[test]
    fn lens_samples_both_monitors_without_stretching_or_restarting_at_the_seam() {
        let monitors = [
            MonitorBackdrop {
                left: -100,
                top: 0,
                width: 100,
                height: 100,
                downscale: 1,
                image: Image::solid(100, 100, [255, 0, 0]),
            },
            MonitorBackdrop {
                left: 0,
                top: 0,
                width: 100,
                height: 100,
                downscale: 1,
                image: Image::solid(100, 100, [0, 0, 255]),
            },
        ];
        let plate = crop_from_monitors(
            &monitors,
            [-30, 10, 60, 60],
            1.5,
            GlassOptics {
                refraction: 0.0,
                ..Default::default()
            },
        );
        for y in 0..60 {
            for x in 0..60 {
                let i = (y * 60 + x) * 4;
                assert_eq!(
                    &plate.bgra[i..i + 4],
                    if x < 30 {
                        &[0, 0, 255, 255]
                    } else {
                        &[255, 0, 0, 255]
                    }
                );
            }
        }
        // The right bezel lies on display B and bends inward into display A.
        let plate = crop_from_monitors(&monitors, [-80, 10, 90, 60], 1.0, GlassOptics::default());
        let i = (30 * 90 + 89) * 4;
        assert_eq!(&plate.bgra[i..i + 4], &[0, 0, 255, 255]);
        assert_eq!(sample_desktop(&monitors, 0.0, 30.5), [128, 0, 128, 255]);
        let reversed = [monitors[1].clone(), monitors[0].clone()];
        assert_eq!(sample_desktop(&reversed, 0.0, 30.5), [128, 0, 128, 255]);
    }

    #[test]
    fn overlay_leaves_flat_desktop_and_outer_corners_transparent_at_every_dpi() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            for height in [36.0, 180.0, 300.0] {
                let width = 320.0;
                let source = Image::solid(
                    (width * scale) as u32,
                    (height * scale) as u32,
                    [60, 120, 180],
                );
                let overlay = refracted_overlay(&source, width, height, 24.0);
                let band = GlassOptics::default().bezel.min(height * 0.5);
                let reach = LensProfile::new(band, height * 0.5).reach;
                let mut partial = 0;
                for y in 0..overlay.height {
                    for x in 0..overlay.width {
                        let i = ((y * overlay.width + x) * 4) as usize;
                        let p = &overlay.bgra[i..i + 4];
                        let (_, _, depth) = edge(
                            (x as f32 + 0.5) / scale,
                            (y as f32 + 0.5) / scale,
                            width,
                            height,
                            24.0,
                        );
                        if depth <= 0.0 || depth >= reach {
                            assert_eq!(p, [0, 0, 0, 0], "{scale} DPI, {height} DIP, {x},{y}");
                        } else if depth <= band * 0.1 {
                            assert!(p[3] >= 250, "outer glass strength was lost");
                        } else if depth >= reach * 0.95 {
                            assert!(
                                p[3] <= 3,
                                "the inner edge must meet the clear centre gently"
                            );
                        }
                        assert!(p[..3].iter().all(|&c| c <= p[3]), "not premultiplied");
                        partial += usize::from(p[3] > 0 && p[3] < 255);
                    }
                }
                assert!(
                    partial > 0,
                    "inner bezel must feather, not form a hard seam"
                );
                assert!(
                    source.bgra.as_chunks::<4>().0.iter().all(|p| p[3] == 255),
                    "contrast sampling must retain the original opaque crop"
                );
            }
        }
    }

    #[test]
    fn delayed_drag_frame_cannot_move_the_flat_desktop_pixels() {
        let mut image = Image::solid(800, 600, [0, 0, 0]);
        for (i, p) in image.bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let value = if (i % 800 / 7 + i / 800 / 9) % 2 == 0 {
                20
            } else {
                235
            };
            p[..3].fill(value);
        }
        let bg = MonitorBackdrop {
            left: -400,
            top: -200,
            width: 800,
            height: 600,
            downscale: 1,
            image,
        };
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let (w, h) = ((250.0 * scale) as i32, (180.0 * scale) as i32);
            let previous = crop(&bg, [-250, -120, w, h], scale, GlassOptics::default());
            let stale = refracted_overlay(&previous, 250.0, 180.0, 24.0);
            // Simulate DWM already moving the HWND 37px horizontally / 19px vertically
            // while the surface still holds the previous position's crop.
            let (left, top) = (-213, -101);
            let flat_start = LensProfile::new(GlassOptics::default().bezel, 90.0).reach + 1.0;
            let margin = (flat_start * scale) as usize;
            for y in margin..h as usize - margin {
                for x in margin..w as usize - margin {
                    let i = (y * w as usize + x) * 4;
                    let s =
                        (((top - bg.top) as usize + y) * 800 + (left - bg.left) as usize + x) * 4;
                    let a = stale.bgra[i + 3] as u32;
                    for c in 0..3 {
                        let desktop = bg.image.bgra[s + c] as u32;
                        let shown = stale.bgra[i + c] as u32 + desktop * (255 - a) / 255;
                        assert_eq!(shown, desktop, "stale crop moved desktop at {x},{y}");
                    }
                }
            }
        }
    }

    fn backdrop() -> MonitorBackdrop {
        let mut image = Image::solid(100, 80, [0, 0, 0]);
        for y in 0..80 {
            for x in 0..100 {
                let v = (x * 2 + y / 2) as u8;
                let i = ((y * 100 + x) * 4) as usize;
                image.bgra[i..i + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }
        MonitorBackdrop {
            left: -100,
            top: 20,
            width: 400,
            height: 320,
            downscale: 4,
            image,
        }
    }

    #[test]
    fn refraction_bends_only_the_bezel_and_dispersion_separates_channels() {
        let bg = backdrop();
        let plain = crop(
            &bg,
            [-60, 60, 240, 200],
            1.0,
            GlassOptics {
                refraction: 0.0,
                ..Default::default()
            },
        );
        let glass = crop(&bg, [-60, 60, 240, 200], 1.0, GlassOptics::default());
        let centre = ((25 * glass.width + 30) * 4) as usize;
        assert_eq!(
            &plain.bgra[centre..centre + 4],
            &glass.bgra[centre..centre + 4]
        );
        let left = ((25 * glass.width) * 4) as usize;
        assert!(
            glass.bgra[left + 1] > plain.bgra[left + 1],
            "convex left bezel samples inward toward the plate centre"
        );
        assert!(
            (0..4).any(|x| {
                let i = left + x * 4;
                glass.bgra[i] > glass.bgra[i + 2]
            }),
            "blue bends further inward than red"
        );
        assert!(glass.bgra.as_chunks::<4>().0.iter().all(|p| p[3] == 255));
    }

    #[test]
    fn dragging_preserves_fractional_sampling_and_offscreen_size() {
        let bg = backdrop();
        let flat = GlassOptics {
            refraction: 0.0,
            ..Default::default()
        };
        let a = crop(&bg, [-20, 60, 120, 120], 1.5, flat);
        let b = crop(&bg, [-18, 60, 120, 120], 1.5, flat);
        assert_eq!(
            b.bgra[0],
            a.bgra[0] + 1,
            "sub-texel move must not snap to the downsample grid"
        );
        let crossing = crop(&bg, [-130, 0, 241, 201], 1.5, flat);
        assert_eq!((crossing.width, crossing.height), (61, 51));
        assert_eq!(crossing.bgra.len(), 61 * 51 * 4);
        assert!(crop(&bg, [0, 0, 0, 10], 1.0, flat).bgra.is_empty());
        assert_eq!(crop(&bg, [0, 0, 1, 1], 2.0, flat).bgra.len(), 4);
    }

    #[test]
    fn clear_wallpaper_build_preserves_one_pixel_detail() {
        let mut image = Image::solid(48, 32, [0, 0, 0]);
        for y in 0..32 {
            for x in 0..48 {
                let i = ((y * 48 + x) * 4) as usize;
                let v = if (x + y) % 2 == 0 { 30 } else { 230 };
                image.bgra[i..i + 3].fill(v);
            }
        }
        for position in [
            crate::WallpaperPosition::Fill,
            crate::WallpaperPosition::Tile,
        ] {
            let bg = MonitorBackdrop::build(
                &image,
                position,
                0,
                0,
                48,
                32,
                [0, 0, 0],
                crate::MicaTint::LIQUID_LIGHT,
                1,
            );
            assert_eq!(
                bg.image.bgra, image.bgra,
                "1:1 clear wallpaper must not be softened"
            );
        }
    }

    #[test]
    fn native_resolution_preserves_every_interior_pixel_at_fractional_dpi() {
        let mut bg = backdrop();
        bg.width = 100;
        bg.height = 80;
        bg.downscale = 1;
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let plate = crop(
                &bg,
                [-105, 25, 100, 70],
                scale,
                GlassOptics {
                    radius: 8.0,
                    bezel: 8.0,
                    ..Default::default()
                },
            );
            let margin =
                ((LensProfile::new(8.0, 35.0 / scale).reach + 0.5) * scale).ceil() as usize;
            let mut checked = 0;
            for y in margin..70 - margin {
                for x in margin..100 - margin {
                    let dst = (y * 100 + x) * 4;
                    let src = ((y + 5) * 100 + x - 5) * 4;
                    assert_eq!(&plate.bgra[dst..dst + 4], &bg.image.bgra[src..src + 4]);
                    checked += 1;
                }
            }
            assert!(checked > 0, "fixture must retain a genuine flat centre");
        }
    }

    #[test]
    fn flat_background_stays_flat_at_corners_and_all_dpis() {
        let mut bg = backdrop();
        bg.image = Image::solid(100, 80, [44, 80, 130]);
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let plate = crop(&bg, [-110, 10, 300, 36], scale, GlassOptics::default());
            assert!(
                plate
                    .bgra
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| *p == [130, 80, 44, 255])
            );
        }
    }
}
