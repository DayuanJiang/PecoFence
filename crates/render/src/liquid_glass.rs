//! Cached Liquid Glass optics over the monitor wallpaper.
//!
//! The GPU plate (`displacement_field`, evaluated by `gpu_glass`) follows kube.io's model:
//! a convex squircle bezel refracts a vertical ray with Snell's law, all bending sits in a
//! narrow rim and the flat face stays perfectly clear
//! (<https://kube.io/blog/liquid-glass-css-svg/>). Geometry is cached independently of
//! wallpaper position. The older CPU path (`crop`, `refracted_overlay`) keeps its own
//! hand-tuned bezel and remains for comparison renders only.

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

/// Width of the GPU plate's curved bezel in DIPs. kube's profile assumes a corner at least
/// this round, so smaller corners (rolled fences, tabs) shrink the bezel with them.
pub(crate) const BEZEL_DIP: f32 = 16.0;
/// Flat glass thickness and bezel height as multiples of the bezel width. The fence plate
/// uses kube's shape at a 16 DIP bezel: ~18 DIP of displacement at the rim.
const GLASS_BASE: f64 = 2.0;
const GLASS_HEIGHT: f64 = 1.25;
/// kube's bezel folds ~5 DIP from the rim and recovers by ~12 DIP. On a large clear face
/// that recovery reads as an inner frame, so a long monotone tail, amount·(1 − d/width)³,
/// eases the magnification out instead. Short plates scale it so it ends at their centre.
const TAIL_DIP: f32 = 40.0;
const TAIL_AMOUNT_DIP: f32 = 6.0;
/// The profile falls from its maximum to a third of it within ~2 DIP of the rim, so the
/// map needs finer texels than the smooth face would.
const MAP_TEXELS_PER_DIP: f32 = 1.0;
const LUT_SAMPLES: usize = 256;

/// kube.io's refraction model ("Liquid Glass in the Browser: Refraction with CSS and
/// SVG"): a vertical ray through a convex squircle bezel at normalised position `x`
/// (0 = rim, 1 = flat face) lands this far inward, in bezel widths. `base` is the flat
/// glass thickness and `height` the bezel height, both in bezel widths; the factor ½ is
/// kube's SVG map encoding, kept so its published maps are the reference.
fn kube_offset(x: f64, base: f64, height: f64) -> f64 {
    let surface = |x: f64| (1.0 - (1.0 - x).powi(4)).max(0.0).powf(0.25);
    let eta = 1.0 / 1.5;
    let y = surface(x);
    // kube's one-sided difference: the analytic slope is infinite at the rim.
    let step = if x < 1.0 { 1e-4 } else { -1e-4 };
    let slope = (surface(x + step) - y) / step;
    let length = (slope * slope + 1.0).sqrt();
    let (nx, ny) = (-slope / length, -1.0 / length);
    let k = 1.0 - eta * eta * (1.0 - ny * ny);
    if k < 0.0 {
        return 0.0;
    }
    let q = k.sqrt();
    let (rx, ry) = (-(eta * ny + q) * nx, eta - (eta * ny + q) * ny);
    0.5 * rx * (y * height + base) / ry
}

fn offset_lut() -> &'static [f32; LUT_SAMPLES + 1] {
    static LUT: std::sync::OnceLock<[f32; LUT_SAMPLES + 1]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| {
        std::array::from_fn(|i| {
            kube_offset(i as f64 / LUT_SAMPLES as f64, GLASS_BASE, GLASS_HEIGHT) as f32
        })
    })
}

/// Inward displacement in bezel widths at `depth` bezel widths from the rim; zero past the
/// bezel. The rim bends hardest and the mapping folds there, as real glass does.
fn bezel_offset(depth: f32) -> f32 {
    if depth >= 1.0 {
        return 0.0;
    }
    let lut = offset_lut();
    let at = depth.max(0.0) * LUT_SAMPLES as f32;
    let i = (at as usize).min(LUT_SAMPLES - 1);
    let t = at - i as f32;
    lut[i] + (lut[i + 1] - lut[i]) * t
}

/// kube's specular ring: 2 DIP wide, peaking 1 DIP inside the rim, brightest where the
/// edge faces the light at 60 degrees (either side). Returns the ring's alpha.
fn rim_light(nx: f32, ny: f32, depth: f32) -> f32 {
    if !(0.0..=2.0).contains(&depth) {
        return 0.0;
    }
    let profile = (1.0 - (1.0 - depth).powi(2)).max(0.0).sqrt();
    let facing = (nx * 0.5 - ny * 0.866_025_4).abs() * profile;
    facing * facing
}

/// IEEE half precision; the map stays within 0..=1, where 0.5 (no displacement) is exact.
fn half(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    if exponent <= 0 {
        return sign;
    }
    if exponent >= 31 {
        return sign | 0x7c00;
    }
    let mantissa = bits & 0x7f_ffff;
    // Round to nearest; a carry into the exponent is the correctly rounded value.
    let rounded = ((exponent as u32) << 10 | mantissa >> 13) + ((mantissa >> 12) & 1);
    sign | rounded as u16
}

/// Geometry-only GPU inputs. Built on size/DPI changes, never on window movement.
pub(crate) struct DisplacementField {
    /// Half-float RGBA texels; R/G hold the x/y displacement around 0.5. One spare flat row
    /// follows the last (see `rim`).
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u16>,
    /// A8 specular ring at device resolution, the size of the plate: rows `rim_pitch` bytes
    /// apart (a multiple of 4), then one spare row. An AMD driver crashed reading just past a
    /// tight buffer's end while uploading it (feedback #69).
    pub rim: Vec<u8>,
    pub rim_width: u32,
    pub rim_height: u32,
    pub rim_pitch: u32,
    pub scale_px: f32,
}

/// Bytes between two rows of an A8 ring `width` pixels wide.
fn rim_pitch(width: u32) -> u32 {
    width.next_multiple_of(4)
}

pub(crate) fn displacement_field(
    width: u32,
    height: u32,
    scale: f32,
    radius: f32,
) -> DisplacementField {
    let scale = scale.max(0.5);
    let (w, h) = (width as f32 / scale, height as f32 / scale);
    let radius = radius.min(w.min(h) * 0.5).max(0.0);
    let bezel = BEZEL_DIP.min(radius);
    let tail = TAIL_DIP.min(w.min(h) * 0.5);
    let tail_amount = TAIL_AMOUNT_DIP * (tail / TAIL_DIP).min(bezel / BEZEL_DIP);
    // The tail reaches past the corner radius: take its direction from a box rounded as
    // far as the tail reaches, so the corner diagonals cannot crease.
    let tail_radius = tail.max(radius);
    let reach = bezel.max(tail);
    let max_offset = offset_lut().iter().fold(0.0f32, |m, v| m.max(v.abs())) * bezel + tail_amount;
    let scale_px = (2.0 * max_offset * scale).max(1.0);
    let map_w = (w * MAP_TEXELS_PER_DIP).ceil().max(2.0) as u32;
    let map_h = (h * MAP_TEXELS_PER_DIP).ceil().max(2.0) as u32;
    let neutral = half(0.5);
    let flat = [neutral, neutral, neutral, half(1.0)];
    let mut rgba = flat.repeat(map_w as usize * (map_h as usize + 1));
    let mut put = |x: u32, y: u32, dx: f32, dy: f32| {
        let i = (y * map_w + x) as usize * 4;
        rgba[i] = half(0.5 - dx / scale_px);
        rgba[i + 1] = half(0.5 - dy / scale_px);
    };
    // The field is mirror-symmetric: evaluate the top-left quadrant and reflect it.
    for y in 0..map_h.div_ceil(2) {
        let py = (y as f32 + 0.5) * h / map_h as f32;
        let (my, inner_row) = (map_h - 1 - y, py > reach + 1.0);
        for x in 0..map_w.div_ceil(2) {
            let px = (x as f32 + 0.5) * w / map_w as f32;
            if inner_row && px > reach + 1.0 {
                continue;
            }
            let (nx, ny, depth) = edge(px, py, w, h, radius);
            let rim = if bezel > 0.0 && depth < bezel {
                bezel * bezel_offset(depth / bezel)
            } else {
                0.0
            };
            let (tx, ty, tail_depth) = edge(px, py, w, h, tail_radius);
            let eased = if tail > 0.0 && tail_depth < tail {
                tail_amount * (1.0 - tail_depth.max(0.0) / tail).powi(3)
            } else {
                0.0
            };
            if rim == 0.0 && eased == 0.0 {
                continue;
            }
            // Sample inward: the convex rim compresses the backdrop behind it.
            let dx = (nx * rim + tx * eased) * scale;
            let dy = (ny * rim + ty * eased) * scale;
            let mx = map_w - 1 - x;
            put(x, y, dx, dy);
            if mx != x {
                put(mx, y, -dx, dy);
            }
            if my != y {
                put(x, my, dx, -dy);
                if mx != x {
                    put(mx, my, -dx, -dy);
                }
            }
        }
    }
    let pitch = rim_pitch(width);
    let mut rim = vec![0u8; pitch as usize * (height as usize + 1)];
    // Only pixels within the ring's 2 DIP (plus the corners) can be lit.
    let band = ((2.0 + radius) * scale).ceil() as u32 + 1;
    let side = (2.0 * scale).ceil() as u32 + 1;
    // The light is lit from both sides of one diagonal: a half turn maps the ring onto
    // itself, so the lower half is the upper half rotated.
    for y in 0..height.div_ceil(2) {
        let py = (y as f32 + 0.5) / scale;
        let full = y < band;
        let mut x = 0;
        while x < width {
            if !full && x == side && width > 2 * side {
                x = width - side;
                continue;
            }
            let (nx, ny, depth) = edge((x as f32 + 0.5) / scale, py, w, h, radius);
            let alpha = rim_light(nx, ny, depth);
            let value = (alpha * 255.0).round().clamp(0.0, 255.0) as u8;
            rim[(y * pitch + x) as usize] = value;
            rim[((height - 1 - y) * pitch + width - 1 - x) as usize] = value;
            x += 1;
        }
    }
    DisplacementField {
        width: map_w,
        height: map_h,
        rgba,
        rim,
        rim_width: width,
        rim_height: height,
        rim_pitch: pitch,
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

    fn from_half(bits: u16) -> f32 {
        let exponent = ((bits >> 10) & 0x1f) as i32;
        let mantissa = (bits & 0x3ff) as f32 / 1024.0;
        let value = if exponent == 0 {
            mantissa * 2f32.powi(-14)
        } else {
            (1.0 + mantissa) * 2f32.powi(exponent - 15)
        };
        if bits & 0x8000 != 0 { -value } else { value }
    }

    /// Displacement in device pixels at map texel (x, y).
    fn texel_offset(field: &DisplacementField, x: usize, y: usize) -> (f32, f32) {
        let i = (y * field.width as usize + x) * 4;
        (
            (from_half(field.rgba[i]) - 0.5) * field.scale_px,
            (from_half(field.rgba[i + 1]) - 0.5) * field.scale_px,
        )
    }

    #[test]
    fn half_floats_round_trip_the_map_range() {
        assert_eq!(half(0.5), 0x3800, "no displacement must be exact");
        assert_eq!(half(1.0), 0x3c00);
        assert_eq!(half(0.0), 0);
        for i in 0..=1000 {
            let v = i as f32 / 1000.0;
            assert!((from_half(half(v)) - v).abs() <= 0.0005, "{v}");
        }
    }

    #[test]
    fn profile_reproduces_kube_published_map() {
        // kube.io ships a 640x63 search bar map (bezel 26, glass 50, bezel height 20 CSS
        // px); these offsets were read from its centre column.
        let measured = [
            (0.0, 27.96),
            (1.5, 19.47),
            (4.5, 9.81),
            (6.0, 7.17),
            (9.0, 4.24),
        ];
        for (depth, expected) in measured {
            let x = depth / 26.0;
            let got = 26.0 * kube_offset(x, 50.0 / 26.0, 20.0 / 26.0);
            assert!((got - expected).abs() < 0.8, "{depth}: {got} vs {expected}");
        }
        let rim = BEZEL_DIP * bezel_offset(0.0);
        assert!((17.0..19.0).contains(&rim), "fence rim bends {rim} DIP");
        // The peak sits just inside the rim, where the squircle has already risen.
        assert!(bezel_offset(0.004) > bezel_offset(0.0));
        let mut previous = f32::INFINITY;
        for i in 1..=100 {
            let offset = bezel_offset(i as f32 / 100.0);
            assert!(offset.is_finite() && offset >= 0.0 && offset <= previous + 1e-6);
            previous = offset;
        }
        assert_eq!(bezel_offset(1.0), 0.0);
        assert_eq!(bezel_offset(2.0), 0.0);
    }

    #[test]
    fn gpu_field_bends_only_the_rim_and_keeps_the_face_clear() {
        let field = displacement_field(800, 600, 2.0, 16.0);
        assert_eq!((field.width, field.height), (400, 300));
        assert_eq!((field.rim_width, field.rim_height), (800, 600));
        let rim = texel_offset(&field, 0, 150).0;
        assert!(rim > 30.0, "left rim samples inward by {rim} px");
        assert!(texel_offset(&field, 399, 150).0 < -30.0);
        assert!(
            texel_offset(&field, 200, 0).1 > 30.0,
            "top rim samples downward"
        );
        for y in 40..260 {
            for x in 40..360 {
                let i = (y * 400 + x) * 4;
                assert_eq!(field.rgba[i], 0x3800, "flat face moved at {x},{y}");
                assert_eq!(field.rgba[i + 1], 0x3800);
            }
        }
        // Past kube's bezel only the easing tail remains, fading to nothing by 40 DIP.
        let tail = texel_offset(&field, 20, 150).0;
        assert!(tail > 0.1 && tail < 3.0, "tail at 20 DIP moves {tail} px");
        assert!(texel_offset(&field, 38, 150).0.abs() < 0.05);
        for y in 0..300 {
            for x in 0..400 {
                let a = texel_offset(&field, x, y);
                let b = texel_offset(&field, 399 - x, 299 - y);
                assert!((a.0 + b.0).abs() < 0.05 && (a.1 + b.1).abs() < 0.05);
            }
        }
        assert!(
            field
                .rgba
                .iter()
                .all(|&v| (0.0..=1.0).contains(&from_half(v)))
        );
        let high_dpi = displacement_field(1600, 1200, 4.0, 16.0);
        assert_eq!(
            field.rgba, high_dpi.rgba,
            "DPI must scale rays, not change the surface"
        );
        assert_eq!(high_dpi.scale_px, 2.0 * field.scale_px);
    }

    #[test]
    fn magnification_eases_out_instead_of_ending_at_an_inner_frame() {
        // Sample position along the middle row, in DIP from the left rim (scale 1).
        let field = displacement_field(400, 300, 1.0, 16.0);
        let sample = |x: usize| x as f32 + 0.5 + texel_offset(&field, x, 150).0;
        let slope = |x: usize| sample(x + 1) - sample(x);
        // kube's fold stays at the rim ...
        assert!(
            slope(0) < 0.0 && slope(2) < 0.0,
            "the rim must keep its reflection"
        );
        // ... and past kube's recovery the zoom relaxes steadily, without a step where its
        // bezel ends, reaching 1:1 only at the tail's end (half floats jitter by ~0.02).
        let turn = (0..20).find(|&x| slope(x) > 0.0).unwrap();
        assert!((5..=8).contains(&turn), "fold turns at {turn} DIP");
        for x in 12..44 {
            assert!(
                slope(x + 3) >= slope(x) - 0.03,
                "zoom tightens again at {x} DIP"
            );
            assert!(
                (slope(x + 1) - slope(x)).abs() < 0.06,
                "visible step at {x} DIP"
            );
        }
        assert!(
            slope(16) < 0.95,
            "the face must still be easing at the bezel end"
        );
        assert!((slope(45) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn short_plates_end_the_tail_at_their_centre_and_corners_stay_smooth() {
        // A rolled 36 DIP fence: both halves meet at the centre line without a seam.
        let rolled = displacement_field(640, 72, 2.0, 16.0);
        for x in [40usize, 160, 280] {
            for y in [17usize, 18] {
                let (dx, dy) = texel_offset(&rolled, x, y);
                assert!(
                    dx.abs() < 0.05 && dy.abs() < 0.2,
                    "centre moved {dx},{dy} at {x},{y}"
                );
            }
        }
        // The tail passes the 16 DIP corner: neighbouring texels across the diagonal must
        // move alike instead of flipping between horizontal and vertical.
        let square = displacement_field(300, 300, 1.0, 16.0);
        for d in 18..36 {
            let a = texel_offset(&square, d, d + 1);
            let b = texel_offset(&square, d + 1, d);
            assert!((a.0 - b.1).abs() < 0.05 && (a.1 - b.0).abs() < 0.05);
            let c = texel_offset(&square, d, d);
            assert!(
                (a.0 - c.0).abs() < 0.3 && (a.1 - c.1).abs() < 0.3,
                "crease at {d}"
            );
        }
    }

    #[test]
    fn rim_light_is_a_thin_ring_lit_across_one_diagonal() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let (w, h) = ((320.0 * scale) as u32, (120.0 * scale) as u32);
            let field = displacement_field(w, h, scale, 16.0);
            let alpha = |x: u32, y: u32| field.rim[(y * field.rim_pitch + x) as usize];
            let mid = h / 2;
            let peak = (1.0 * scale) as u32;
            assert!(
                alpha(peak, mid) > 40,
                "{scale}: left ring {}",
                alpha(peak, mid)
            );
            assert!(
                alpha(w / 2, peak) > alpha(peak, mid),
                "top faces the 60 degree light more than the side"
            );
            for x in (3.0 * scale).ceil() as u32..w - (3.0 * scale).ceil() as u32 {
                assert_eq!(alpha(x, mid), 0, "{scale}: ring leaked inward at {x}");
            }
            // The ring crosses each corner's diagonal ~5.4 DIP in from both edges.
            let near = |x: f32, y: f32| {
                let span = (2.0 * scale) as i32;
                let (x, y) = ((x * scale) as i32, (y * scale) as i32);
                (-span..=span)
                    .flat_map(|dy| (-span..=span).map(move |dx| (x + dx, y + dy)))
                    .map(|(x, y)| alpha(x as u32, y as u32))
                    .max()
                    .unwrap()
            };
            let (left, right) = (near(5.4, 5.4), near(320.0 - 5.4, 5.4));
            assert!(
                right > 150 && left < right / 3,
                "{scale}: top-right faces the light ({right}), top-left does not ({left})"
            );
        }
    }

    #[test]
    fn short_and_sharp_plates_scale_the_bezel_without_nan() {
        for (w, h, scale, radius) in [
            (640, 64, 2.0, 16.0),
            (300, 40, 1.25, 16.0),
            (120, 28, 1.0, 12.0),
            (200, 200, 1.5, 0.0),
            (2, 2, 1.0, 16.0),
        ] {
            let field = displacement_field(w, h, scale, radius);
            assert!(field.scale_px.is_finite() && field.scale_px >= 1.0);
            assert!(
                field
                    .rgba
                    .iter()
                    .all(|&v| (0.0..=1.0).contains(&from_half(v)))
            );
            assert_eq!(field.rim.len(), (field.rim_pitch * (h + 1)) as usize);
        }
        let sharp = displacement_field(200, 200, 1.5, 0.0);
        assert!(
            sharp
                .rgba
                .chunks(4)
                .all(|t| t[0] == 0x3800 && t[1] == 0x3800)
        );
    }

    #[test]
    fn uploads_have_padded_rows_and_a_spare_row() {
        for (w, h, scale) in [
            (517, 301, 1.25),
            (640, 64, 2.0),
            (3, 5, 1.0),
            (1001, 777, 1.5),
        ] {
            let field = displacement_field(w, h, scale, 16.0);
            let pitch = field.rim_pitch;
            assert!(
                pitch.is_multiple_of(4) && pitch >= w && pitch - w < 4,
                "{w}: pitch {pitch}"
            );
            assert_eq!(field.rim.len(), (pitch * (h + 1)) as usize);
            let at = |x: u32, y: u32| field.rim[(y * pitch + x) as usize];
            for y in 0..h {
                for x in 0..w {
                    assert_eq!(
                        at(x, y),
                        at(w - 1 - x, h - 1 - y),
                        "{w}x{h}: half turn at {x},{y}"
                    );
                }
                assert!(
                    (w..pitch).all(|x| at(x, y) == 0),
                    "{w}x{h}: padding of row {y}"
                );
            }
            assert!(field.rim[(pitch * h) as usize..].iter().all(|&v| v == 0));
            let row = field.width as usize * 4;
            assert_eq!(field.rgba.len(), row * (field.height as usize + 1));
            let spare = &field.rgba[row * field.height as usize..];
            assert!(
                spare
                    .chunks(4)
                    .all(|t| t == [0x3800, 0x3800, 0x3800, 0x3C00])
            );
        }
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
                gpu: None,
            },
            MonitorBackdrop {
                left: 0,
                top: 0,
                width: 100,
                height: 100,
                downscale: 1,
                image: Image::solid(100, 100, [0, 0, 255]),
                gpu: None,
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
            gpu: None,
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
            gpu: None,
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
