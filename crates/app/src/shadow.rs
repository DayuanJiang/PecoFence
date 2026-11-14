//! Soft drop shadow for a fence, drawn in a separate click-through layered window so the fence
//! HWND stays exactly the size of its visible body (plan review: no dead zones around fences).

use pecofence_platform::layered::{self, LayeredImage};
use pecofence_platform::window::{self, ClassOptions, Window, WindowClass};
use pecofence_platform::{HWND, RECT};
use pecofence_render::Image;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use windows_core::Result;

pub const SHADOW_CLASS: &str = "PecoFence.Shadow";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowStyle {
    /// Gaussian sigma in DIPs.
    pub sigma: f32,
    /// Vertical offset in DIPs.
    pub offset_y: f32,
    /// Peak opacity 0..1.
    pub opacity: f32,
    /// Corner radius of the casting shape in DIPs.
    pub radius: f32,
}

impl ShadowStyle {
    /// A plate lying on the print, not hovering over it: short offset, tight blur.
    pub const DARK: Self = Self {
        sigma: 8.0,
        offset_y: 4.0,
        opacity: 0.30,
        radius: 8.0,
    };
    pub const LIGHT: Self = Self {
        sigma: 8.0,
        offset_y: 4.0,
        opacity: 0.16,
        radius: 8.0,
    };

    pub fn pad_px(&self, scale: f32) -> i32 {
        ((self.sigma * 3.0 + self.offset_y) * scale).ceil() as i32
    }
}

/// White ring around a fence selected with a marquee on the desktop, drawn into the shadow just
/// outside the body, so it follows every move, resize, roll and fade of the fence. It matches
/// the drop outline: a white 2 DIP stroke with a dark 1 DIP hairline outside it, readable on
/// light and dark wallpapers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRing {
    /// The fence's corner radius (DIPs); the ring is concentric with it.
    pub radius_dip: f32,
}

/// Distance between the body edge and the ring (DIPs).
const RING_GAP_DIP: f32 = 1.0;
/// White stroke, then the dark hairline outside it (DIPs).
const RING_STROKE_DIP: f32 = 2.0;
const RING_HAIR_DIP: f32 = 1.0;
/// Opacity of the stroke and the hairline (the drop outline's).
const RING_WHITE_ALPHA: f32 = 0.92;
const RING_HAIR_ALPHA: f32 = 0.45;

/// Coverage of the distance band `from..to` (px) at signed distance `d`, anti-aliased.
fn ring_band(d: f32, from: f32, to: f32) -> f32 {
    (d - from + 0.5).clamp(0.0, 1.0) * (to - d + 0.5).clamp(0.0, 1.0)
}

/// Composites `ring` over the padded shadow `img` of a `w` x `h` body at `pad`. Only the band
/// around the body edge is visited.
fn draw_ring(img: &mut Image, pad: i32, w: i32, h: i32, scale: f32, ring: &SelectionRing) {
    let gap = RING_GAP_DIP * scale;
    let white = gap + RING_STROKE_DIP * scale;
    let outer = white + RING_HAIR_DIP * scale;
    let radius = (ring.radius_dip * scale).min(w.min(h) as f32 * 0.5);
    let reach = outer.ceil() as i32 + 1;
    let (sw, sh) = (img.width as i32, img.height as i32);
    let (x0, x1) = ((pad - reach).max(0), (pad + w + reach).min(sw));
    // Rows this close to the top or bottom edge cross a corner or the horizontal stroke.
    let corner = radius.ceil() as i32 + reach;
    let sdf = |fx: f32, fy: f32| -> f32 {
        let dx = (fx - w as f32 / 2.0).abs() - (w as f32 / 2.0 - radius);
        let dy = (fy - h as f32 / 2.0).abs() - (h as f32 / 2.0 - radius);
        (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() + dx.max(dy).min(0.0) - radius
    };
    for y in (pad - reach).max(0)..(pad + h + reach).min(sh) {
        let by = y - pad;
        let spans = if by < corner || by >= h - corner {
            [(x0, x1), (0, 0)]
        } else {
            [(x0, pad + 1), (pad + w - 1, x1)]
        };
        for (from, to) in spans {
            for x in from..to {
                let d = sdf(x as f32 + 0.5 - pad as f32, by as f32 + 0.5);
                if d <= gap - 0.5 || d >= outer + 0.5 {
                    continue;
                }
                let i = ((y * sw + x) * 4) as usize;
                let (mut c, mut a) = (img.bgra[i] as f32 / 255.0, img.bgra[i + 3] as f32 / 255.0);
                // Premultiplied "over", inside out: the white stroke, then the hairline.
                for (colour, alpha) in [
                    (1.0, RING_WHITE_ALPHA * ring_band(d, gap, white)),
                    (0.0, RING_HAIR_ALPHA * ring_band(d, white, outer)),
                ] {
                    c = colour * alpha + c * (1.0 - alpha);
                    a = alpha + a * (1.0 - alpha);
                }
                let c = (c * 255.0).round() as u8;
                img.bgra[i..i + 4].copy_from_slice(&[c, c, c, (a * 255.0).round() as u8]);
            }
        }
    }
}

/// Renders the shadow bitmap for a body of `w` x `h` pixels, returning the padded image.
fn render_shadow(w: i32, h: i32, scale: f32, style: &ShadowStyle) -> Image {
    let pad = style.pad_px(scale);
    let sw = w + pad * 2;
    let sh = h + pad * 2;
    let radius = (style.radius * scale).min(w.min(h) as f32 * 0.5);
    let oy = style.offset_y * scale;
    let mut img = Image {
        width: sw as u32,
        height: sh as u32,
        bgra: vec![0u8; (sw * sh * 4) as usize],
    };
    // Anti-aliased coverage of the rounded body (signed-distance field) at a body-local point.
    let body_cov = |fx: f32, fy: f32| -> f32 {
        let dx = (fx - w as f32 / 2.0).abs() - (w as f32 / 2.0 - radius);
        let dy = (fy - h as f32 / 2.0).abs() - (h as f32 / 2.0 - radius);
        let d = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt() + dx.max(dy).min(0.0) - radius;
        (0.5 - d).clamp(0.0, 1.0)
    };
    // Alpha mask of the rounded body, shifted down by the offset.
    for y in 0..sh {
        for x in 0..sw {
            let fx = x as f32 + 0.5 - pad as f32;
            let fy = y as f32 + 0.5 - pad as f32 - oy;
            let a = (body_cov(fx, fy) * 255.0) as u8;
            let i = ((y * sw + x) * 4) as usize;
            img.bgra[i + 3] = a; // black, premultiplied: colour stays 0
        }
    }
    img.blur_alpha(style.sigma * scale);
    // Apply opacity and cut the shadow out from under the rounded body (the fence clips
    // itself to the same radius as the SDF) so the anti-aliased corner arc blends into the
    // shadow instead of leaving square notches; the cut-out also avoids double darkening of
    // the Mica backdrop through the fence's translucent pixels. On the straight edges coverage
    // is exactly 0 or 1, identical to a rectangle test.
    for y in 0..sh {
        for x in 0..sw {
            let i = ((y * sw + x) * 4) as usize;
            let cov = body_cov(x as f32 + 0.5 - pad as f32, y as f32 + 0.5 - pad as f32);
            let a = img.bgra[i + 3] as f32 * style.opacity * (1.0 - cov);
            img.bgra[i] = 0;
            img.bgra[i + 1] = 0;
            img.bgra[i + 2] = 0;
            img.bgra[i + 3] = a.round().clamp(0.0, 255.0) as u8;
        }
    }
    img
}

/// Nine-slice template: a shadow rendered once for a small body; any larger body is produced
/// by stretching the middle row/column (exact, because the shadow only depends on the distance
/// to the nearest body edge once past the rounded corners).
struct ShadowTemplate {
    img: Image,
    pad: i32,
    /// Fixed corner extent (pixels) copied verbatim; the rest is stretched.
    corner: i32,
}

impl ShadowTemplate {
    /// Corner extent (body pixels) past which the shadow no longer depends on the corner.
    fn corner_px(scale: f32, style: &ShadowStyle) -> i32 {
        ((style.radius + style.sigma * 3.0 + style.offset_y) * scale).ceil() as i32 + 2
    }

    /// Smallest body edge `make` / `make_wide` can stretch along.
    fn min_body(&self) -> i32 {
        self.corner * 2 + 2
    }

    fn build(scale: f32, style: &ShadowStyle) -> Self {
        let corner = Self::corner_px(scale, style);
        let body = corner * 2 + 8;
        Self::build_for(scale, style, body, body)
    }

    /// Template for bodies of exactly `h` px that stretch only horizontally (a rolled fence's
    /// title row, or one frame of a roll whose height is under the two-corner minimum): a
    /// 76 px (100 %) wide render instead of the full-width Gaussian.
    fn build_for_height(scale: f32, style: &ShadowStyle, h: i32) -> Self {
        let corner = Self::corner_px(scale, style);
        Self::build_for(scale, style, corner * 2 + 8, h)
    }

    fn build_for(scale: f32, style: &ShadowStyle, w: i32, h: i32) -> Self {
        Self {
            img: render_shadow(w, h, scale, style),
            pad: style.pad_px(scale),
            corner: Self::corner_px(scale, style),
        }
    }

    /// One destination row (`sw` px wide) from template row `ty`: both corners verbatim, the
    /// middle repeats the template's centre pixel (slice copies instead of per-pixel mapping).
    fn build_row(&self, ty: i32, sw: i32, dst: &mut [u8]) {
        let tw = self.img.width as i32;
        let c = self.pad + self.corner; // template pixels before the stretch zone
        let rb = (sw * 4) as usize;
        let cl = (c * 4) as usize;
        let t = &self.img.bgra;
        let trow = (ty * tw * 4) as usize;
        dst[..cl].copy_from_slice(&t[trow..trow + cl]);
        let tr = trow + ((tw - c) * 4) as usize;
        dst[rb - cl..].copy_from_slice(&t[tr..tr + cl]);
        let px: [u8; 4] = t[trow + cl..trow + cl + 4].try_into().unwrap();
        for chunk in dst[cl..rb - cl].as_chunks_mut::<4>().0 {
            *chunk = px;
        }
    }

    /// Builds a `w`x`h` body shadow (padded) from the template.
    fn make(&self, w: i32, h: i32) -> Image {
        let pad = self.pad;
        let sw = w + pad * 2;
        let sh = h + pad * 2;
        let tw = self.img.width as i32;
        let th = self.img.height as i32;
        let c = pad + self.corner; // template pixels before the stretch zone
        debug_assert!(sw >= 2 * c + 2 && sh >= 2 * c + 2 && tw > 2 * c);
        let rb = (sw * 4) as usize; // destination row bytes
        let mut out = vec![0u8; rb * sh as usize];
        let cu = c as usize;
        for y in 0..cu {
            self.build_row(y as i32, sw, &mut out[y * rb..(y + 1) * rb]);
        }
        // Middle band: every row is identical → build once, replicate with memcpy.
        let mid_rows = (sh as usize) - 2 * cu;
        {
            let (head, tail) = out.split_at_mut((cu + 1) * rb);
            self.build_row(c, sw, &mut head[cu * rb..]);
            let src = &head[cu * rb..];
            for row in tail[..(mid_rows - 1) * rb].chunks_exact_mut(rb) {
                row.copy_from_slice(src);
            }
        }
        for y in (sh as usize - cu)..(sh as usize) {
            let ty = th - (sh - y as i32);
            self.build_row(ty, sw, &mut out[y * rb..(y + 1) * rb]);
        }
        Image {
            width: sw as u32,
            height: sh as u32,
            bgra: out,
        }
    }

    /// The rows of the padded image of a body `h` px tall that differ from those of a body
    /// `prev_h` px tall and as wide (both at least `min_body`): the bottom corner rows of the
    /// new image, and, when it grew, the rows from the old bottom corner rows down to them.
    /// The top corner rows and the middle rows above are the same in both.
    fn restretch_rows(&self, prev_h: i32, h: i32) -> std::ops::Range<i32> {
        let c = self.pad + self.corner;
        (prev_h.min(h) + self.pad * 2 - c)..(h + self.pad * 2)
    }

    /// Turns the padded image of a `w`x`prev_h` body at the top of `canvas` (rows of
    /// `(w + 2 pad) * 4` bytes, room for the new image) into that of a `w`x`h` body by
    /// writing `restretch_rows` only. The result is what `make(w, h)` builds.
    fn restretch(&self, w: i32, prev_h: i32, h: i32, canvas: &mut [u8]) {
        let sw = w + self.pad * 2;
        let sh = h + self.pad * 2;
        let th = self.img.height as i32;
        let c = self.pad + self.corner;
        debug_assert!(prev_h.min(h) >= self.min_body() && w >= self.min_body());
        let rb = (sw * 4) as usize;
        for y in self.restretch_rows(prev_h, h) {
            // A middle row above the bottom corner rows, the template's own row inside them.
            let ty = if y < sh - c { c } else { th - (sh - y) };
            let at = y as usize * rb;
            self.build_row(ty, sw, &mut canvas[at..at + rb]);
        }
    }

    /// Builds a body shadow `w` wide and exactly the template's body height (padded) by
    /// stretching every row horizontally (`build_for_height` templates).
    fn make_wide(&self, w: i32) -> Image {
        let pad = self.pad;
        let sw = w + pad * 2;
        let sh = self.img.height as i32;
        let c = pad + self.corner;
        debug_assert!(sw >= 2 * c + 2 && self.img.width as i32 > 2 * c);
        let rb = (sw * 4) as usize;
        let mut out = vec![0u8; rb * sh as usize];
        for y in 0..sh as usize {
            self.build_row(y as i32, sw, &mut out[y * rb..(y + 1) * rb]);
        }
        Image {
            width: sw as u32,
            height: sh as u32,
            bgra: out,
        }
    }
}

thread_local! {
    /// Templates in use, by (DPI, style): every fence on a monitor shares one.
    static TEMPLATES: RefCell<Vec<((u32, [u32; 4]), Weak<ShadowTemplate>)>> =
        const { RefCell::new(Vec::new()) };
}

/// The template for `dpi` and `style`, built once while any shadow window holds it.
fn shared_template(dpi: u32, scale: f32, style: &ShadowStyle) -> Rc<ShadowTemplate> {
    let key = (
        dpi,
        [style.sigma, style.offset_y, style.opacity, style.radius].map(f32::to_bits),
    );
    TEMPLATES.with_borrow_mut(|templates| {
        templates.retain(|(_, t)| t.strong_count() > 0);
        if let Some(t) = templates
            .iter()
            .find(|(k, _)| *k == key)
            .and_then(|(_, t)| t.upgrade())
        {
            return t;
        }
        let t = Rc::new(ShadowTemplate::build(scale, style));
        templates.push((key, Rc::downgrade(&t)));
        t
    })
}

pub struct ShadowWindow {
    window: Window,
    style: ShadowStyle,
    template: Option<(u32, Rc<ShadowTemplate>)>,
    /// Horizontal-only template for the current short body height (DPI, body height): a
    /// rolled fence is 36 DIP tall, under the two-corner minimum of `template`, so its width
    /// changes would otherwise re-run the full Gaussian.
    short: Option<(u32, i32, ShadowTemplate)>,
    last: Option<(RECT, u32)>,
    /// The bitmap for `last` has been presented. The window keeps it, so its pixels are
    /// released right away and `set_alpha` only changes the blend.
    presented: bool,
    /// Constant alpha of the layered window (`SourceConstantAlpha`): the fence's whole-window
    /// fade drives it so the shadow fades with the plate.
    alpha: u8,
    ring: Option<SelectionRing>,
    /// While the body's height animates (`keep_upload`): the tallest body the animation
    /// reaches, and the DIB section the frames' bitmaps go through. A section of its own for
    /// every frame was up to half of what a roll frame's shadow cost.
    keep: Option<i32>,
    canvas: Option<LayeredImage>,
}

impl ShadowWindow {
    pub fn register_class() -> Result<WindowClass> {
        WindowClass::register(SHADOW_CLASS, ClassOptions::default())
    }

    pub fn create(class: &WindowClass, style: ShadowStyle) -> Result<Self> {
        let window = window::create_click_through(class, "shadow")?;
        Ok(Self {
            window,
            style,
            template: None,
            short: None,
            last: None,
            presented: false,
            alpha: 255,
            ring: None,
            keep: None,
            canvas: None,
        })
    }

    /// `Some(h)` while the body's height animates towards at most `h` px: the bitmaps of the
    /// following `update`s share one DIB section tall enough for that body. `None` when the
    /// animation is over: the section is let go (the window keeps what was presented).
    pub fn keep_upload(&mut self, tallest_body: Option<i32>) {
        self.keep = tallest_body;
        if tallest_body.is_none() {
            self.canvas = None;
        }
    }

    pub fn ring(&self) -> Option<SelectionRing> {
        self.ring
    }

    /// Adds or removes the selection ring; the next `update` renders it.
    pub fn set_ring(&mut self, ring: Option<SelectionRing>) {
        if self.ring != ring {
            self.ring = ring;
            self.invalidate();
        }
    }

    /// Body size (device px) the current shadow bitmap was rendered for.
    pub fn body_size(&self) -> Option<(i32, i32)> {
        self.last.map(|(r, _)| (r.right - r.left, r.bottom - r.top))
    }

    pub fn hwnd(&self) -> HWND {
        self.window.hwnd()
    }

    pub fn set_style(&mut self, style: ShadowStyle) {
        if self.style != style {
            self.style = style;
            self.template = None;
            self.short = None;
            self.last = None;
            self.presented = false;
        }
    }

    /// Forgets the presented bitmap so the next `update` renders and presents it again (the
    /// window may have lost its content, e.g. across a display change).
    pub fn invalidate(&mut self) {
        self.last = None;
        self.presented = false;
    }

    /// Constant alpha the shadow is shown with (0 = invisible, 255 = the style's opacity).
    /// Changing it updates only the window's blend alpha — no bitmap, copy or re-render — so a
    /// whole-window fade can carry the shadow along frame by frame.
    /// Current constant alpha (see `set_alpha`).
    pub fn alpha(&self) -> u8 {
        self.alpha
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        if alpha == self.alpha {
            return;
        }
        self.alpha = alpha;
        if let Some((body, dpi)) = self.last
            && self.presented
            && let Err(e) = layered::set_alpha(self.window.hwnd(), alpha)
        {
            // Rare: the call failed outright. (Content the system silently dropped is not
            // detected here; `invalidate` on display changes covers that.)
            tracing::warn!(error = %e, "shadow alpha update failed; re-rendering");
            self.last = None;
            self.update(body, dpi);
        }
    }

    /// Repositions/redraws the shadow for a fence body rectangle (screen pixels).
    pub fn update(&mut self, body: RECT, dpi: u32) {
        let scale = dpi.max(96) as f32 / 96.0;
        let w = body.right - body.left;
        let h = body.bottom - body.top;
        if w <= 0 || h <= 0 {
            return;
        }
        if self.last == Some((body, dpi)) {
            return;
        }
        let pad = self.style.pad_px(scale);
        // Pure move (same size, same DPI): the bitmap is still valid, so just reposition the
        // layered window — no re-upload, no lag behind the body while dragging.
        if let Some((prev, prev_dpi)) = self.last
            && prev_dpi == dpi
            && (prev.right - prev.left, prev.bottom - prev.top) == (w, h)
        {
            let sw = w + pad * 2;
            let sh = h + pad * 2;
            let _ = self.window.set_bounds_z(
                body.left - pad,
                body.top - pad,
                sw,
                sh,
                window::ZOrder::Keep,
                window::swp::NOSIZE,
            );
            self.last = Some((body, dpi));
            return;
        }
        if self.template.as_ref().is_none_or(|(d, _)| *d != dpi) {
            self.template = Some((dpi, shared_template(dpi, scale, &self.style)));
        }
        let tpl = &self.template.as_ref().unwrap().1;
        let min_body = tpl.min_body();
        let started = std::time::Instant::now();
        // A step of a height animation (`keep_upload`): the image in the kept section differs
        // from this one only in its height, so only the rows that differ are written, in
        // place. (Building the whole image and copying it into the section was 1.2 to 2.2 ms
        // of a roll frame for a 780 x 600 px fence at 150 %; these rows are a tenth of it.)
        if self.ring.is_none()
            && w >= min_body
            && h >= min_body
            && let Some((prev, prev_dpi)) = self.last
            && prev_dpi == dpi
            && prev.right - prev.left == w
            && prev.bottom - prev.top >= min_body
            && let Some(section) = self.canvas.as_mut()
            && section.size() == (w + pad * 2, prev.bottom - prev.top + pad * 2)
            && section.set_rows(h + pad * 2)
        {
            let prev_h = prev.bottom - prev.top;
            tpl.restretch(w, prev_h, h, section.pixels_mut());
            let written = started.elapsed();
            let presented = section.present(
                self.window.hwnd(),
                body.left - pad,
                body.top - pad,
                self.alpha,
            );
            tracing::trace!(
                target: "pecofence::render_timing",
                w,
                h,
                rows = tpl.restretch_rows(prev_h, h).len(),
                make_us = written.as_micros(),
                copy_us = 0,
                present_us = (started.elapsed() - written).as_micros(),
                "shadow stages"
            );
            match presented {
                Ok(()) => {
                    self.presented = true;
                    self.last = Some((body, dpi));
                }
                Err(e) => {
                    tracing::warn!(error = %e, "shadow update failed");
                    self.presented = false;
                    // Retry with a full render on the next update.
                    self.last = None;
                    self.canvas = None;
                }
            }
            return;
        }
        let mut img = if w >= min_body && h >= min_body {
            tpl.make(w, h)
        } else if w >= min_body {
            // Short body (rolled title row, the tail of a roll / expand): a narrow template
            // for this exact height, stretched sideways — rebuilt only when the height or DPI
            // changes, so width changes of a rolled fence are memcpy-only.
            if self
                .short
                .as_ref()
                .is_none_or(|(d, sh, _)| (*d, *sh) != (dpi, h))
            {
                self.short = Some((
                    dpi,
                    h,
                    ShadowTemplate::build_for_height(scale, &self.style, h),
                ));
            }
            self.short.as_ref().unwrap().2.make_wide(w)
        } else {
            render_shadow(w, h, scale, &self.style)
        };
        if let Some(ring) = self.ring {
            draw_ring(&mut img, pad, w, h, scale, &ring);
        }
        if pecofence_core::brand::var_os("PECOFENCE_DEBUG_SHADOW").is_some() {
            let mut bmp = Vec::new();
            let (iw, ih) = (img.width as i32, img.height as i32);
            let size = 54 + (iw * ih * 4) as u32;
            bmp.extend_from_slice(b"BM");
            bmp.extend_from_slice(&size.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&54u32.to_le_bytes());
            bmp.extend_from_slice(&40u32.to_le_bytes());
            bmp.extend_from_slice(&iw.to_le_bytes());
            bmp.extend_from_slice(&(-ih).to_le_bytes());
            bmp.extend_from_slice(&1u16.to_le_bytes());
            bmp.extend_from_slice(&32u16.to_le_bytes());
            bmp.extend_from_slice(&[0u8; 24]);
            bmp.extend_from_slice(&img.bgra);
            let _ = std::fs::write("target/shadow_debug.bmp", bmp);
            let nz = img
                .bgra
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[3] > 0)
                .count();
            let maxa = img
                .bgra
                .as_chunks::<4>()
                .0
                .iter()
                .map(|p| p[3])
                .max()
                .unwrap_or(0);
            tracing::info!(
                w = iw,
                h = ih,
                pad,
                nonzero_alpha = nz,
                max_alpha = maxa,
                "shadow bitmap"
            );
        }
        // Presented once; the window keeps the content, so the DIB is released here and
        // fades go through `set_alpha`. While the height animates the section is kept for the
        // next frame's bitmap instead.
        let (iw, ih) = (img.width as i32, img.height as i32);
        let made = started.elapsed();
        let kept = self
            .canvas
            .take()
            .and_then(|mut section| section.refill(iw, ih, &img.bgra).then_some(section));
        let upload = match kept {
            Some(section) => Ok(section),
            None => {
                let room = self.keep.map_or(ih, |tallest| tallest + pad * 2);
                LayeredImage::with_room(iw, ih, room, &img.bgra)
            }
        };
        let copied = started.elapsed();
        let presented = upload.and_then(|up| {
            let done = up.present(
                self.window.hwnd(),
                body.left - pad,
                body.top - pad,
                self.alpha,
            );
            if self.keep.is_some() {
                self.canvas = Some(up);
            }
            done
        });
        tracing::trace!(
            target: "pecofence::render_timing",
            w,
            h,
            rows = ih,
            make_us = made.as_micros(),
            copy_us = (copied - made).as_micros(),
            present_us = (started.elapsed() - copied).as_micros(),
            "shadow stages"
        );
        self.presented = match presented {
            Ok(()) => true,
            Err(e) => {
                tracing::warn!(error = %e, "shadow update failed");
                false
            }
        };
        if !self.presented {
            // Retry with a full render on the next update.
            self.last = None;
            return;
        }
        if pecofence_core::brand::var_os("PECOFENCE_DEBUG_SHADOW").is_some() {
            let r = window::window_rect(self.window.hwnd());
            tracing::info!(?r, visible = self.window.is_visible(), body = ?body, "shadow window placed");
        }
        self.last = Some((body, dpi));
    }

    /// Hides the shadow (showing and z-placing it happens through `hwnd()` by the fence
    /// window code, with no view borrow held).
    pub fn hide(&self) {
        self.window.hide();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(img: &Image, x: i32, y: i32) -> u8 {
        img.bgra[((y * img.width as i32 + x) * 4 + 3) as usize]
    }

    /// The cut-out follows the rounded silhouette: the corner pixels outside the arc keep
    /// shadow (no light notch), the straight edges and the interior stay clear.
    #[test]
    fn shadow_follows_rounded_corners() {
        let style = ShadowStyle::DARK;
        let (w, h) = (40, 40);
        let img = render_shadow(w, h, 1.0, &style);
        let pad = style.pad_px(1.0);
        // Bottom-left body corner pixel lies outside the 8 px arc: shadowed.
        let corner = alpha(&img, pad, pad + h - 1);
        assert!(corner > 0, "corner pixel unshadowed: {corner}");
        // Just outside the body beside it is shadowed too, and the corner pixel (closer to the
        // body's mass) is no lighter than half of it: no bright notch against the shadow.
        let outside = alpha(&img, pad - 1, pad + h - 1);
        assert!(
            outside > 0 && corner * 2 >= outside,
            "{corner} vs {outside}"
        );
        // Interior and straight edges are fully cut out.
        assert_eq!(alpha(&img, pad + w / 2, pad + h / 2), 0);
        assert_eq!(alpha(&img, pad + w / 2, pad), 0);
        assert_eq!(alpha(&img, pad + w / 2, pad + h - 1), 0);
        assert_eq!(alpha(&img, pad, pad + h / 2), 0);
        // Same corner at 150 %: still continuous.
        let img = render_shadow(w, h, 1.5, &style);
        let pad = style.pad_px(1.5);
        assert!(alpha(&img, pad, pad + h - 1) > 0);
        assert_eq!(alpha(&img, pad + w / 2, pad + h / 2), 0);
    }

    fn assert_close(a: &Image, b: &Image, tol: i32, what: &str) {
        assert_eq!((a.width, a.height), (b.width, b.height), "{what}: size");
        let worst = a
            .bgra
            .iter()
            .zip(&b.bgra)
            .map(|(x, y)| (*x as i32 - *y as i32).abs())
            .max()
            .unwrap_or(0);
        assert!(worst <= tol, "{what}: max alpha difference {worst}");
    }

    /// The horizontal-only template reproduces the direct render for short bodies at every
    /// DPI (the shadow depends only on the distance to the nearest edge past the corners).
    #[test]
    fn short_body_template_matches_direct_render() {
        let style = ShadowStyle::DARK;
        for (w, h, scale) in [
            (400, 36, 1.0),
            (400, 36, 2.0),
            (800, 72, 2.0),
            (300, 60, 1.5),
        ] {
            let tpl = ShadowTemplate::build_for_height(scale, &style, h);
            assert!(w >= tpl.min_body());
            let sliced = tpl.make_wide(w);
            let direct = render_shadow(w, h, scale, &style);
            assert_close(&sliced, &direct, 1, &format!("{w}x{h}@{scale}"));
        }
    }

    /// The two-way nine-slice still matches the direct render after the row refactor.
    #[test]
    fn full_template_matches_direct_render() {
        let style = ShadowStyle::LIGHT;
        let tpl = ShadowTemplate::build(1.0, &style);
        let (w, h) = (200, 200);
        assert!(w >= tpl.min_body());
        assert_close(
            &tpl.make(w, h),
            &render_shadow(w, h, 1.0, &style),
            1,
            "200x200",
        );
    }

    /// A roll's frames: the rows rewritten in place give, byte for byte, the image built whole
    /// for every height, rolling up and opening again, at 100 % and at 150 %.
    #[test]
    fn rows_rewritten_for_a_new_height_give_the_image_built_whole() {
        for scale in [1.0, 1.5] {
            let style = ShadowStyle::DARK;
            let tpl = ShadowTemplate::build(scale, &style);
            let w = tpl.min_body() + 40;
            let tall = tpl.min_body() + 300;
            let rb = ((w + tpl.pad * 2) * 4) as usize;
            let mut canvas = vec![0u8; rb * (tall + tpl.pad * 2) as usize];
            let first = tpl.make(w, tall);
            canvas[..first.bgra.len()].copy_from_slice(&first.bgra);
            let mut prev = tall;
            let down = [
                tall - 1,
                tall - 40,
                tall - 170,
                tpl.min_body() + 3,
                tpl.min_body(),
            ];
            let up = [tpl.min_body() + 1, tpl.min_body() + 90, tall - 7, tall];
            for h in down.into_iter().chain(up) {
                tpl.restretch(w, prev, h, &mut canvas);
                let whole = tpl.make(w, h);
                assert!(
                    canvas[..whole.bgra.len()] == whole.bgra[..],
                    "scale {scale}: {prev} px to {h} px"
                );
                // Shrinking writes the bottom corner rows only; growing, those and the rows
                // the body grew by.
                let rows = tpl.restretch_rows(prev, h).len() as i32;
                assert_eq!(rows, tpl.pad + tpl.corner + (h - prev).max(0));
                prev = h;
            }
        }
    }

    /// The selection ring sits outside the body (white 1–3 DIP out, the hairline 3–4 DIP),
    /// concentric with its corners, and the band-only walk paints exactly what a walk over
    /// every pixel would.
    #[test]
    fn selection_ring_hugs_the_body_outside() {
        let ring = SelectionRing { radius_dip: 8.0 };
        for (w, h, scale, pad) in [(200, 100, 2.0, 56), (120, 72, 1.0, 28), (90, 36, 1.5, 42)] {
            let blank = || Image {
                width: (w + pad * 2) as u32,
                height: (h + pad * 2) as u32,
                bgra: vec![0u8; ((w + pad * 2) * (h + pad * 2) * 4) as usize],
            };
            let mut img = blank();
            draw_ring(&mut img, pad, w, h, scale, &ring);
            // Reference: the same coverage evaluated at every pixel.
            let radius = (ring.radius_dip * scale).min(w.min(h) as f32 * 0.5);
            let gap = RING_GAP_DIP * scale;
            let white = gap + RING_STROKE_DIP * scale;
            let outer = white + RING_HAIR_DIP * scale;
            let sw = w + pad * 2;
            for y in 0..h + pad * 2 {
                for x in 0..sw {
                    let fx = x as f32 + 0.5 - pad as f32;
                    let fy = y as f32 + 0.5 - pad as f32;
                    let dx = (fx - w as f32 / 2.0).abs() - (w as f32 / 2.0 - radius);
                    let dy = (fy - h as f32 / 2.0).abs() - (h as f32 / 2.0 - radius);
                    let d = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt()
                        + dx.max(dy).min(0.0)
                        - radius;
                    let aw = RING_WHITE_ALPHA * ring_band(d, gap, white);
                    let ah = RING_HAIR_ALPHA * ring_band(d, white, outer);
                    let c = aw * (1.0 - ah);
                    let a = ah + aw * (1.0 - ah);
                    let want = [c, c, c, a].map(|v| (v * 255.0).round() as u8);
                    let i = ((y * sw + x) * 4) as usize;
                    assert_eq!(&img.bgra[i..i + 4], &want, "{w}x{h}@{scale} at ({x},{y})");
                }
            }
            // Left of the body: mid-stroke white, then the hairline; the body stays clear.
            let row = (pad + h / 2) * sw;
            let mid = (pad as f32 - 2.0 * scale) as i32;
            let i = ((row + mid) * 4) as usize;
            assert_eq!(&img.bgra[i..i + 4], &[235, 235, 235, 235]);
            let hair = pad - white.ceil() as i32 - 1;
            let i = ((row + hair) * 4) as usize;
            assert_eq!(&img.bgra[i..i + 4], &[0, 0, 0, 115]);
            let inside = ((row + pad + 1) * 4) as usize;
            assert_eq!(img.bgra[inside + 3], 0);
        }
    }
}
