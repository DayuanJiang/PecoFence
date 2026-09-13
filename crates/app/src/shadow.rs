//! Soft drop shadow for a fence, drawn in a separate click-through layered window so the fence
//! HWND stays exactly the size of its visible body (plan review: no dead zones around fences).

use pecofence_platform::layered::LayeredImage;
use pecofence_platform::window::{
    self, ClassOptions, MessageHandler, Window, WindowBuilder, WindowClass, style,
};
use pecofence_platform::{HWND, RECT, msg};
use pecofence_render::Image;
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
        radius: 4.0,
    };
    pub const LIGHT: Self = Self {
        sigma: 8.0,
        offset_y: 4.0,
        opacity: 0.16,
        radius: 4.0,
    };

    pub fn pad_px(&self, scale: f32) -> i32 {
        ((self.sigma * 3.0 + self.offset_y) * scale).ceil() as i32
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
    // Apply opacity and cut the shadow out from under the DWM-rounded body (RoundSmall, the
    // same radius as the SDF) so the anti-aliased corner arc blends into the shadow instead of
    // leaving square notches; the cut-out also avoids double darkening of the Mica backdrop
    // through the fence's translucent pixels. On the straight edges coverage is exactly 0 or 1,
    // identical to a rectangle test.
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

pub struct ShadowWindow {
    window: Window,
    style: ShadowStyle,
    template: Option<(u32, ShadowTemplate)>,
    /// Horizontal-only template for the current short body height (DPI, body height): a
    /// rolled fence is 36 DIP tall, under the two-corner minimum of `template`, so its width
    /// changes would otherwise re-run the full Gaussian.
    short: Option<(u32, i32, ShadowTemplate)>,
    last: Option<(RECT, u32)>,
    /// The padded shadow bitmap for `last`, resident in a DIB section: `set_alpha` re-presents
    /// it with a new constant alpha without re-rendering or copying the pixels.
    image: Option<LayeredImage>,
    /// Constant alpha of the layered window (`SourceConstantAlpha`): the fence's whole-window
    /// fade drives it so the shadow fades with the plate.
    alpha: u8,
}

impl ShadowWindow {
    pub fn register_class() -> Result<WindowClass> {
        WindowClass::register(SHADOW_CLASS, ClassOptions::default())
    }

    pub fn create(class: &WindowClass, style: ShadowStyle) -> Result<Self> {
        let handler: MessageHandler = Box::new(|_hwnd, message, _wparam, _lparam| match message {
            msg::WM_MOUSEACTIVATE => Some(msg::MA_NOACTIVATE),
            msg::WM_NCHITTEST => Some(msg::HTTRANSPARENT),
            msg::WM_DESTROY => Some(0),
            _ => None,
        });
        let window = WindowBuilder::new(class)
            .title("shadow")
            .style(style::POPUP)
            .ex_style(
                style::EX_LAYERED
                    | style::EX_TRANSPARENT
                    | style::EX_NOACTIVATE
                    | style::EX_TOOLWINDOW,
            )
            .bounds(0, 0, 1, 1)
            .create(handler)?;
        Ok(Self {
            window,
            style,
            template: None,
            short: None,
            last: None,
            image: None,
            alpha: 255,
        })
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
            self.image = None;
        }
    }

    /// Constant alpha the shadow is shown with (0 = invisible, 255 = the style's opacity).
    /// Changing it re-presents the resident bitmap with the new blend alpha — no allocation,
    /// copy or re-render — so a whole-window fade can carry the shadow along frame by frame.
    /// Current constant alpha (see `set_alpha`).
    pub fn alpha(&self) -> u8 {
        self.alpha
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        if alpha == self.alpha {
            return;
        }
        self.alpha = alpha;
        if let (Some((body, dpi)), Some(img)) = (self.last, self.image.as_ref()) {
            let scale = dpi.max(96) as f32 / 96.0;
            let pad = self.style.pad_px(scale);
            if let Err(e) = img.present(self.window.hwnd(), body.left - pad, body.top - pad, alpha)
            {
                tracing::warn!(error = %e, "shadow alpha update failed");
            }
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
            self.template = Some((dpi, ShadowTemplate::build(scale, &self.style)));
        }
        let tpl = &self.template.as_ref().unwrap().1;
        let min_body = tpl.min_body();
        let img = if w >= min_body && h >= min_body {
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
        // Uploaded once into a DIB section and kept: `set_alpha` re-presents the same bitmap
        // with a new constant alpha (no allocation or copy per fade frame).
        let uploaded =
            LayeredImage::new(img.width as i32, img.height as i32, &img.bgra).and_then(|up| {
                up.present(
                    self.window.hwnd(),
                    body.left - pad,
                    body.top - pad,
                    self.alpha,
                )?;
                Ok(up)
            });
        self.image = match uploaded {
            Ok(up) => Some(up),
            Err(e) => {
                tracing::warn!(error = %e, "shadow update failed");
                None
            }
        };
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

/// Class check used by hit-testing code.
#[allow(dead_code)]
pub fn is_shadow_class(hwnd: HWND) -> bool {
    pecofence_platform::desktop::class_name(hwnd) == SHADOW_CLASS
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
        // Bottom-left body corner pixel lies outside the 4 px arc: shadowed.
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
        assert_eq!(alpha(&img, pad + 2, pad + 2), 0);
        assert_eq!(alpha(&img, pad + w / 2, pad), 0);
        assert_eq!(alpha(&img, pad + w / 2, pad + h - 1), 0);
        assert_eq!(alpha(&img, pad, pad + h / 2), 0);
        // Same corner at 150 %: still continuous.
        let img = render_shadow(w, h, 1.5, &style);
        let pad = style.pad_px(1.5);
        assert!(alpha(&img, pad, pad + h - 1) > 0);
        assert_eq!(alpha(&img, pad + 3, pad + 3), 0);
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
}
