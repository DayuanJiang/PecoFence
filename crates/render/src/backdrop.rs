//! Mica-like backdrop: the monitor's wallpaper, mapped the way Explorer shows it, downscaled,
//! blurred once per monitor, and tinted. Fences then draw a crop of it (plan §6.1).
//!
//! All image math here is plain Rust on 32-bit BGRA buffers. Everything is opaque (alpha 255),
//! so premultiplied and straight alpha coincide.

use windows_canvas::ColorF;

/// Wallpaper fill mode as reported by `IDesktopWallpaper::GetPosition`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum WallpaperPosition {
    Center,
    Tile,
    Stretch,
    Fit,
    #[default]
    Fill,
    Span,
}

/// An opaque BGRA image.
#[derive(Clone, Debug, Default)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl Image {
    pub fn solid(width: u32, height: u32, color: [u8; 3]) -> Self {
        let mut bgra = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..width * height {
            bgra.extend_from_slice(&[color[2], color[1], color[0], 255]);
        }
        Self {
            width,
            height,
            bgra,
        }
    }

    #[inline]
    fn px(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.bgra[i],
            self.bgra[i + 1],
            self.bgra[i + 2],
            self.bgra[i + 3],
        ]
    }

    /// Bilinear sample at continuous source coordinates (clamped).
    pub(crate) fn sample(&self, sx: f32, sy: f32) -> [u8; 4] {
        let maxx = (self.width - 1) as f32;
        let maxy = (self.height - 1) as f32;
        let sx = sx.clamp(0.0, maxx);
        let sy = sy.clamp(0.0, maxy);
        let x0 = sx.floor() as u32;
        let y0 = sy.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let fx = sx - x0 as f32;
        let fy = sy - y0 as f32;
        let p00 = self.px(x0, y0);
        let p10 = self.px(x1, y0);
        let p01 = self.px(x0, y1);
        let p11 = self.px(x1, y1);
        let mut out = [0u8; 4];
        for c in 0..4 {
            let top = p00[c] as f32 * (1.0 - fx) + p10[c] as f32 * fx;
            let bottom = p01[c] as f32 * (1.0 - fx) + p11[c] as f32 * fx;
            out[c] = (top * (1.0 - fy) + bottom * fy).round() as u8;
        }
        out
    }

    /// Dispersion only needs one color channel at each additional sampling position.
    #[inline]
    pub(crate) fn sample_channel(&self, sx: f32, sy: f32, channel: usize) -> u8 {
        let sx = sx.clamp(0.0, (self.width - 1) as f32);
        let sy = sy.clamp(0.0, (self.height - 1) as f32);
        let x0 = sx.floor() as u32;
        let y0 = sy.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let fx = sx - x0 as f32;
        let fy = sy - y0 as f32;
        let pixel = |x, y| self.bgra[((y * self.width + x) * 4) as usize + channel] as f32;
        let top = pixel(x0, y0) * (1.0 - fx) + pixel(x1, y0) * fx;
        let bottom = pixel(x0, y1) * (1.0 - fx) + pixel(x1, y1) * fx;
        (top * (1.0 - fy) + bottom * fy).round() as u8
    }

    /// Resamples into a `dst_w` x `dst_h` image where destination pixel (x, y) maps to the
    /// source rectangle `src` (in source pixels, may extend outside; outside = `bg`).
    fn resample_region(&self, src: [f32; 4], dst_w: u32, dst_h: u32, bg: [u8; 3]) -> Image {
        let mut out = Vec::with_capacity((dst_w * dst_h * 4) as usize);
        let sw = src[2] - src[0];
        let sh = src[3] - src[1];
        for y in 0..dst_h {
            let sy = src[1] + (y as f32 + 0.5) / dst_h as f32 * sh;
            for x in 0..dst_w {
                let sx = src[0] + (x as f32 + 0.5) / dst_w as f32 * sw;
                if sx < 0.0 || sy < 0.0 || sx >= self.width as f32 || sy >= self.height as f32 {
                    out.extend_from_slice(&[bg[2], bg[1], bg[0], 255]);
                } else {
                    // The rectangle describes pixel edges; sample() indexes pixel centres.
                    // Without this half-texel correction even a 1:1 clear wallpaper blurs.
                    out.extend_from_slice(&self.sample(sx - 0.5, sy - 0.5));
                }
            }
        }
        Image {
            width: dst_w,
            height: dst_h,
            bgra: out,
        }
    }

    /// Icon Tint: pulls every pixel toward `tint` scaled by its own luminance (a colour wash that
    /// keeps the icon's shading), by `strength` (0..1). Premultiplied BGRA in, same out.
    pub fn colorize(&mut self, tint: [u8; 3], strength: f32) {
        let k = strength.clamp(0.0, 1.0);
        let (tr, tg, tb) = (
            tint[0] as f32 / 255.0,
            tint[1] as f32 / 255.0,
            tint[2] as f32 / 255.0,
        );
        for px in self.bgra.as_chunks_mut::<4>().0 {
            let (b, g, r) = (px[0] as f32, px[1] as f32, px[2] as f32);
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            px[2] = (r + (lum * tr - r) * k).round().clamp(0.0, 255.0) as u8;
            px[1] = (g + (lum * tg - g) * k).round().clamp(0.0, 255.0) as u8;
            px[0] = (b + (lum * tb - b) * k).round().clamp(0.0, 255.0) as u8;
        }
    }

    /// Chameleon: desaturate by `desat` (0..1) and multiply the opacity by `alpha`, so icons
    /// recede into the glass instead of shouting from it.
    pub fn fade(&mut self, desat: f32, alpha: f32) {
        let d = desat.clamp(0.0, 1.0);
        let a = alpha.clamp(0.0, 1.0);
        for px in self.bgra.as_chunks_mut::<4>().0 {
            let (b, g, r) = (px[0] as f32, px[1] as f32, px[2] as f32);
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            px[2] = ((r + (lum - r) * d) * a).round().clamp(0.0, 255.0) as u8;
            px[1] = ((g + (lum - g) * d) * a).round().clamp(0.0, 255.0) as u8;
            px[0] = ((b + (lum - b) * d) * a).round().clamp(0.0, 255.0) as u8;
            px[3] = (px[3] as f32 * a).round().clamp(0.0, 255.0) as u8;
        }
    }

    /// Crops a pixel rectangle (clamped to bounds; empty rects yield a 1x1 image).
    pub fn crop(&self, x: i32, y: i32, w: i32, h: i32) -> Image {
        let x0 = x.clamp(0, self.width as i32 - 1);
        let y0 = y.clamp(0, self.height as i32 - 1);
        let x1 = (x + w).clamp(x0 + 1, self.width as i32);
        let y1 = (y + h).clamp(y0 + 1, self.height as i32);
        let cw = (x1 - x0) as u32;
        let ch = (y1 - y0) as u32;
        let mut bgra = Vec::with_capacity((cw * ch * 4) as usize);
        for row in y0..y1 {
            let start = ((row as u32 * self.width + x0 as u32) * 4) as usize;
            bgra.extend_from_slice(&self.bgra[start..start + (cw * 4) as usize]);
        }
        Image {
            width: cw,
            height: ch,
            bgra,
        }
    }

    /// Three-pass box blur ≈ Gaussian with the given sigma (in pixels of this image).
    pub fn blur(&mut self, sigma: f32) {
        if sigma <= 0.5 || self.width < 3 || self.height < 3 {
            return;
        }
        let mut tmp = vec![0u8; self.bgra.len()];
        for r in gaussian_box_radii(sigma) {
            box_blur_h(&self.bgra, &mut tmp, self.width, self.height, r);
            box_blur_v(&tmp, &mut self.bgra, self.width, self.height, r);
        }
    }

    /// Same Gaussian approximation as `blur`, for black premultiplied shadow masks.
    /// RGB stays untouched. Single-channel buffers and row-major vertical passes avoid
    /// repeatedly filtering three zero channels and striding through a large BGRA buffer.
    pub fn blur_alpha(&mut self, sigma: f32) {
        if sigma <= 0.5 || self.width < 3 || self.height < 3 {
            return;
        }
        let w = self.width as usize;
        let h = self.height as usize;
        let mut alpha: Vec<u8> = self.bgra.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
        let mut tmp = vec![0; alpha.len()];
        let mut sums = vec![0i32; w];
        for radius in gaussian_box_radii(sigma) {
            let norm = 1.0 / (2 * radius + 1) as f32;
            for y in 0..h {
                let row = y * w;
                let mut sum = 0i32;
                for x in -radius..=radius {
                    sum += alpha[row + x.clamp(0, w as i32 - 1) as usize] as i32;
                }
                for x in 0..w {
                    tmp[row + x] = (sum as f32 * norm).round() as u8;
                    let add = (x as i32 + radius + 1).min(w as i32 - 1) as usize;
                    let sub = (x as i32 - radius).max(0) as usize;
                    sum += alpha[row + add] as i32 - alpha[row + sub] as i32;
                }
            }
            sums.fill(0);
            for y in -radius..=radius {
                let row = y.clamp(0, h as i32 - 1) as usize * w;
                for x in 0..w {
                    sums[x] += tmp[row + x] as i32;
                }
            }
            for y in 0..h {
                let row = y * w;
                let add = (y as i32 + radius + 1).min(h as i32 - 1) as usize * w;
                let sub = (y as i32 - radius).max(0) as usize * w;
                for x in 0..w {
                    alpha[row + x] = (sums[x] as f32 * norm).round() as u8;
                    sums[x] += tmp[add + x] as i32 - tmp[sub + x] as i32;
                }
            }
        }
        for (pixel, alpha) in self.bgra.as_chunks_mut::<4>().0.iter_mut().zip(alpha) {
            pixel[3] = alpha;
        }
    }

    /// Applies the Mica recipe: composite `tint` (alpha = TintOpacity), then move each pixel's
    /// luminance towards the tint's luminance by `luminosity_opacity` (WinUI default 1.0), so the
    /// wallpaper contributes mostly hue, as real Mica does.
    /// WinUI Acrylic/Mica recipe, in the real order: (1) a *luminosity* blend pulls every
    /// pixel's brightness towards the tint's brightness while keeping the wallpaper's hue
    /// (out-of-gamut results are clipped towards the luminance, not clamped per channel, so
    /// colours do not shift); (2) a thin colour tint on top. `noise` is kept for experiments and
    /// is 0 in the shipped recipes — grain is tiled at device resolution when drawing.
    pub fn tint(&mut self, tint: ColorF, luminosity_opacity: f32, noise: f32, chroma: f32) {
        let chroma = chroma.clamp(0.0, 1.5);
        let a = tint.a.clamp(0.0, 1.0);
        let lo = luminosity_opacity.clamp(0.0, 1.0);
        let tr = tint.r * 255.0;
        let tg = tint.g * 255.0;
        let tb = tint.b * 255.0;
        let tint_lum = 0.2126 * tr + 0.7152 * tg + 0.0722 * tb;
        let noise_amp = noise.clamp(0.0, 0.2) * 255.0;
        let mut seed: u32 = 0x9E37_79B9;
        for p in self.bgra.as_chunks_mut::<4>().0 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let n = ((seed >> 8) as f32 / 16_777_216.0 - 0.5) * 2.0 * noise_amp;
            let (mut b, mut g, mut r) = (p[0] as f32, p[1] as f32, p[2] as f32);
            // (1) luminosity blend
            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let target = lum * (1.0 - lo) + tint_lum * lo + n;
            let d = target - lum;
            r += d;
            g += d;
            b += d;
            // ClipColor (W3C compositing spec): keep hue when a channel leaves 0..255.
            let l = target;
            let min = r.min(g).min(b);
            let max = r.max(g).max(b);
            if min < 0.0 && (l - min) > f32::EPSILON {
                let k = l / (l - min);
                r = l + (r - l) * k;
                g = l + (g - l) * k;
                b = l + (b - l) * k;
            }
            if max > 255.0 && (max - l) > f32::EPSILON {
                let k = (255.0 - l) / (max - l);
                r = l + (r - l) * k;
                g = l + (g - l) * k;
                b = l + (b - l) * k;
            }
            // (1b) chroma: scale the colour offsets around the (pinned) luminance.
            if (chroma - 1.0).abs() > f32::EPSILON {
                r = l + (r - l) * chroma;
                g = l + (g - l) * chroma;
                b = l + (b - l) * chroma;
            }
            // (2) colour tint
            b = b * (1.0 - a) + tb * a;
            g = g * (1.0 - a) + tg * a;
            r = r * (1.0 - a) + tr * a;
            p[0] = b.round().clamp(0.0, 255.0) as u8;
            p[1] = g.round().clamp(0.0, 255.0) as u8;
            p[2] = r.round().clamp(0.0, 255.0) as u8;
            p[3] = 255;
        }
    }

    /// Average relative luminance (0..1), used to pick text colours for legibility.
    pub fn mean_luminance(&self) -> f32 {
        if self.bgra.is_empty() {
            return 0.0;
        }
        let mut sum = 0f64;
        for p in self.bgra.as_chunks::<4>().0 {
            sum += 0.0722 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.2126 * p[2] as f64;
        }
        (sum / (self.bgra.len() / 4) as f64 / 255.0) as f32
    }
}

fn gaussian_box_radii(sigma: f32) -> [i32; 3] {
    // Kovesi's three-box Gaussian approximation; shared by color and alpha-only blur.
    let n = 3.0f32;
    let ideal = (12.0 * sigma * sigma / n + 1.0).sqrt();
    let mut wl = ideal.floor() as i32;
    if wl % 2 == 0 {
        wl -= 1;
    }
    let wu = wl + 2;
    let m = ((12.0 * sigma * sigma - n * (wl * wl) as f32 - 4.0 * n * wl as f32 - 3.0 * n)
        / (-4.0 * wl as f32 - 4.0))
        .round() as i32;
    [0, 1, 2].map(|i| if i < m { (wl - 1) / 2 } else { (wu - 1) / 2 }.max(1))
}

fn box_blur_h(src: &[u8], dst: &mut [u8], w: u32, h: u32, r: i32) {
    let w = w as i32;
    let norm = 1.0 / (2 * r + 1) as f32;
    for y in 0..h as i32 {
        let row = (y * w * 4) as usize;
        let mut acc = [0i32; 4];
        for x in -r..=r {
            let xc = x.clamp(0, w - 1) as usize * 4;
            for c in 0..4 {
                acc[c] += src[row + xc + c] as i32;
            }
        }
        for x in 0..w {
            let o = row + x as usize * 4;
            for c in 0..4 {
                dst[o + c] = (acc[c] as f32 * norm).round() as u8;
            }
            let add = (x + r + 1).clamp(0, w - 1) as usize * 4;
            let sub = (x - r).clamp(0, w - 1) as usize * 4;
            for c in 0..4 {
                acc[c] += src[row + add + c] as i32 - src[row + sub + c] as i32;
            }
        }
    }
}

fn box_blur_v(src: &[u8], dst: &mut [u8], w: u32, h: u32, r: i32) {
    let h = h as i32;
    let stride = (w * 4) as usize;
    let norm = 1.0 / (2 * r + 1) as f32;
    for x in 0..w as usize {
        let col = x * 4;
        let mut acc = [0i32; 4];
        for y in -r..=r {
            let yc = y.clamp(0, h - 1) as usize * stride;
            for c in 0..4 {
                acc[c] += src[yc + col + c] as i32;
            }
        }
        for y in 0..h {
            let o = y as usize * stride + col;
            for c in 0..4 {
                dst[o + c] = (acc[c] as f32 * norm).round() as u8;
            }
            let add = (y + r + 1).clamp(0, h - 1) as usize * stride;
            let sub = (y - r).clamp(0, h - 1) as usize * stride;
            for c in 0..4 {
                acc[c] += src[add + col + c] as i32 - src[sub + col + c] as i32;
            }
        }
    }
}

/// Maps a wallpaper onto a monitor of `mon_w` x `mon_h` pixels the way Explorer does, producing
/// an image downscaled by `downscale` (e.g. 4 → quarter resolution).
pub fn map_wallpaper(
    wallpaper: &Image,
    position: WallpaperPosition,
    mon_w: u32,
    mon_h: u32,
    downscale: u32,
    background: [u8; 3],
) -> Image {
    let downscale = downscale.max(1);
    let dst_w = (mon_w / downscale).max(1);
    let dst_h = (mon_h / downscale).max(1);
    let (mw, mh) = (mon_w as f32, mon_h as f32);
    let (ww, wh) = (wallpaper.width as f32, wallpaper.height as f32);

    // Source rectangle (in wallpaper pixels) that covers the whole monitor.
    let src: [f32; 4] = match position {
        WallpaperPosition::Stretch => [0.0, 0.0, ww, wh],
        WallpaperPosition::Fill | WallpaperPosition::Span => {
            // Scale to cover, centre-crop.
            let scale = (mw / ww).max(mh / wh);
            let vis_w = mw / scale;
            let vis_h = mh / scale;
            let x0 = (ww - vis_w) / 2.0;
            let y0 = (wh - vis_h) / 2.0;
            [x0, y0, x0 + vis_w, y0 + vis_h]
        }
        WallpaperPosition::Fit => {
            // Scale to contain, letterbox with background.
            let scale = (mw / ww).min(mh / wh);
            let vis_w = mw / scale;
            let vis_h = mh / scale;
            let x0 = (ww - vis_w) / 2.0;
            let y0 = (wh - vis_h) / 2.0;
            [x0, y0, x0 + vis_w, y0 + vis_h]
        }
        WallpaperPosition::Center => {
            let x0 = (ww - mw) / 2.0;
            let y0 = (wh - mh) / 2.0;
            [x0, y0, x0 + mw, y0 + mh]
        }
        WallpaperPosition::Tile => {
            // Approximate: tile by wrapping coordinates. Build explicitly.
            let mut out = Vec::with_capacity((dst_w * dst_h * 4) as usize);
            for y in 0..dst_h {
                let sy = ((y as f32 + 0.5) * downscale as f32) % wh;
                for x in 0..dst_w {
                    let sx = ((x as f32 + 0.5) * downscale as f32) % ww;
                    out.extend_from_slice(&wallpaper.sample(sx - 0.5, sy - 0.5));
                }
            }
            return Image {
                width: dst_w,
                height: dst_h,
                bgra: out,
            };
        }
    };
    wallpaper.resample_region(src, dst_w, dst_h, background)
}

/// Material recipe applied to the blurred wallpaper (Mica and Acrylic share the pipeline).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MicaTint {
    /// Tint colour; alpha = TintOpacity.
    pub color: ColorF,
    /// How far each pixel's luminance is pulled towards the tint's luminance (1.0 = flat Mica).
    pub luminosity_opacity: f32,
    /// Blur sigma in DIPs (scaled to the monitor's DPI by the caller; DWM Acrylic uses 30).
    pub blur_sigma_dip: f32,
    /// Peak per-pixel noise amplitude (0..1); Acrylic uses ~0.02–0.03.
    pub noise: f32,
    /// Saturation kept after the luminosity blend (1.0 = wallpaper chroma untouched). Without
    /// it a saturated wallpaper region reads as a different material than a dull one.
    pub chroma: f32,
}

impl MicaTint {
    /// Clear, colourless glass. Preserve the wallpaper's luminance and detail: thickness
    /// comes from the curved bezel, not a mode-coloured acrylic veil.
    pub const LIQUID_DARK: Self = Self {
        color: ColorF::new(1.0, 1.0, 1.0, 0.0),
        luminosity_opacity: 0.0,
        blur_sigma_dip: 0.0,
        noise: 0.0,
        chroma: 1.0,
    };
    pub const LIQUID_LIGHT: Self = Self::LIQUID_DARK;
    /// Mica dark: TintColor #202020 — wallpaper contributes mostly hue.
    pub const MICA_DARK: Self = Self {
        color: ColorF::new(
            0x20 as f32 / 255.0,
            0x20 as f32 / 255.0,
            0x20 as f32 / 255.0,
            0.80,
        ),
        luminosity_opacity: 1.0,
        blur_sigma_dip: 36.0,
        noise: 0.0,
        chroma: 1.0,
    };
    /// Mica light: TintColor #F3F3F3.
    pub const MICA_LIGHT: Self = Self {
        color: ColorF::new(
            0xF3 as f32 / 255.0,
            0xF3 as f32 / 255.0,
            0xF3 as f32 / 255.0,
            0.50,
        ),
        luminosity_opacity: 1.0,
        blur_sigma_dip: 36.0,
        noise: 0.0,
        chroma: 1.0,
    };
    /// Acrylic dark: frosted glass — strong blur, wallpaper colours and brightness stay visible.
    pub const ACRYLIC_DARK: Self = Self {
        color: ColorF::new(
            0x2C as f32 / 255.0,
            0x2C as f32 / 255.0,
            0x2C as f32 / 255.0,
            0.06,
        ),
        // 2026-09-09: user asked for more see-through glass — less luminance pull towards the
        // tint and more of the wallpaper's own colour (was 0.70 / 0.75; 0.40 was still too much).
        luminosity_opacity: 0.10,
        blur_sigma_dip: 30.0,
        noise: 0.0,
        chroma: 0.85,
    };
    /// Acrylic light.
    pub const ACRYLIC_LIGHT: Self = Self {
        color: ColorF::new(
            0xFC as f32 / 255.0,
            0xFC as f32 / 255.0,
            0xFC as f32 / 255.0,
            0.0,
        ),
        luminosity_opacity: 0.20,
        blur_sigma_dip: 30.0,
        noise: 0.0,
        chroma: 0.92,
    };
    pub const DARK: Self = Self::MICA_DARK;
    pub const LIGHT: Self = Self::MICA_LIGHT;
}

/// The ready-to-sample backdrop for one monitor.
#[derive(Clone, Debug)]
pub struct MonitorBackdrop {
    /// Monitor rectangle in virtual-screen pixels.
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
    /// Downscale factor between screen pixels and `image` pixels.
    pub downscale: u32,
    /// Blurred + tinted image at monitor_size / downscale.
    pub image: Image,
}

impl MonitorBackdrop {
    /// Builds the backdrop: map → blur (σ in *screen* pixels) → tint.
    pub fn build(
        wallpaper: &Image,
        position: WallpaperPosition,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        background: [u8; 3],
        tint: MicaTint,
        downscale: u32,
    ) -> Self {
        // `tint.blur_sigma_dip` has already been scaled to this monitor's DPI by the caller.
        let blur_sigma_screen_px = tint.blur_sigma_dip;
        let mut image = map_wallpaper(
            wallpaper,
            position,
            width.max(1) as u32,
            height.max(1) as u32,
            downscale,
            background,
        );
        image.blur(blur_sigma_screen_px / downscale as f32);
        image.tint(tint.color, tint.luminosity_opacity, tint.noise, tint.chroma);
        Self {
            left,
            top,
            width,
            height,
            downscale,
            image,
        }
    }

    /// Crops the backdrop under a screen rectangle (virtual-screen pixels).
    pub fn crop_screen_rect(&self, x: i32, y: i32, w: i32, h: i32) -> Image {
        let d = self.downscale as i32;
        let lx = (x - self.left) / d;
        let ly = (y - self.top) / d;
        self.image.crop(lx, ly, (w / d).max(1), (h / d).max(1))
    }
}

/// Side length (device pixels) of the tiled acrylic noise texture.
pub const NOISE_TILE_PX: u32 = 128;

/// A premultiplied, very faint grey grain tile, drawn 1:1 in device pixels over acrylic.
pub fn noise_tile() -> &'static Image {
    static TILE: std::sync::OnceLock<Image> = std::sync::OnceLock::new();
    TILE.get_or_init(|| {
        let n = NOISE_TILE_PX;
        let mut bgra = vec![0u8; (n * n * 4) as usize];
        let mut seed: u32 = 0x1234_5678;
        for p in bgra.as_chunks_mut::<4>().0 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let v = (seed >> 24) as u8; // 0..255 grey
            let a = 5u32; // ≈2 % opacity: felt, not seen
            let c = (v as u32 * a / 255) as u8;
            p[0] = c;
            p[1] = c;
            p[2] = c;
            p[3] = a as u8;
        }
        Image {
            width: n,
            height: n,
            bgra,
        }
    })
}

#[cfg(test)]
mod pixel_regressions {
    use super::*;

    #[test]
    fn alpha_only_blur_matches_the_original_bgra_blur() {
        for (width, height) in [(3, 5), (73, 31), (110, 92)] {
            let mut image = Image::solid(width, height, [0; 3]);
            for (i, pixel) in image.bgra.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                pixel[3] = ((i * 37 + i / width as usize * 71) % 256) as u8;
            }
            for sigma in [0.25, 1.0, 8.0, 12.0, 24.0] {
                let mut reference = image.clone();
                let mut optimized = image.clone();
                reference.blur(sigma);
                optimized.blur_alpha(sigma);
                assert_eq!(
                    optimized.bgra, reference.bgra,
                    "{width}x{height} sigma={sigma}"
                );
            }
        }
    }

    #[test]
    fn channel_sampler_preserves_dispersion_pixels() {
        let mut image = Image::solid(23, 17, [0; 3]);
        for (i, pixel) in image.bgra.iter_mut().enumerate() {
            *pixel = ((i * 73 + i / 4 * 29) % 256) as u8;
        }
        for y in [-2.5, 0.0, 0.25, 7.75, 15.9, 18.0] {
            for x in [-1.0, 0.5, 8.8, 21.99, 25.0] {
                let reference = image.sample(x, y);
                for (channel, expected) in reference.into_iter().enumerate() {
                    assert_eq!(image.sample_channel(x, y, channel), expected);
                }
            }
        }
    }
}
