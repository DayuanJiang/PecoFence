//! The soft text shadow of a theme file (`textShadow`). Each caption's blurred shape is drawn
//! once into a small alpha mask and kept, so a redraw (a hover fade, a scroll frame) only fills
//! the cached masks in the shadow colour, and a fence at rest costs nothing. The masks share
//! pages of one bitmap each: a driver gives every bitmap a block of memory of its own, several
//! times the size of a small mask.

use crate::gpu_bindings as b;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;
use windows_canvas::{ColorF, DrawingSession, Matrix3x2, Rect, TextFormat, TextLayout, Vector2};
use windows_core::{Interface, Result};

/// Masks kept for all fences together; past this many, the half drawn longest ago goes.
const MAX_MASKS: usize = 1024;
/// Side of a page of masks, in pixels (256 KB, about 50 labels at 100 %).
const PAGE: u32 = 512;

#[derive(Default)]
pub struct TextShadows {
    device: Option<Device>,
    masks: HashMap<u64, Mask>,
    /// Counts the shadows drawn; a mask remembers when it was last used.
    clock: u64,
    /// The device a shadow failed on: the text is drawn without one there.
    failed_on: Option<usize>,
}

/// The device the masks belong to, and the brush that fills them in the shadow colour.
struct Device {
    device: b::ID2D1Device,
    fill: b::ID2D1SolidColorBrush,
    /// Made for new masks, let go by [`TextShadows::trim`].
    maker: Option<Maker>,
    /// The page new masks go to, filled row by row. A page lives while a mask is on it.
    shelf: Option<Shelf>,
}

/// An alpha-only bitmap holding masks.
struct Page(b::ID2D1Bitmap1);

struct Shelf {
    page: Rc<Page>,
    /// Where the next mask of the current row goes, and the row's height so far.
    x: u32,
    y: u32,
    row_h: u32,
}

/// A drawing context of its own to make masks in (the caller's context is inside a surface
/// draw, with its clips and layers pushed), and the opaque brush for the glyphs.
struct Maker {
    context: b::ID2D1DeviceContext,
    glyphs: b::ID2D1SolidColorBrush,
}

/// A caption to shadow: its layout, and the text and format that name its mask.
#[derive(Clone, Copy)]
struct Caption<'a> {
    layout: &'a TextLayout,
    text: &'a str,
    format: &'a TextFormat,
}

struct Mask {
    text: String,
    /// Holds the format, so its address stays its name while the mask lives.
    format: TextFormat,
    /// Layout box size (0 x 0 for one line that fits it), DPI scale and blur, as bits.
    size: [u32; 4],
    /// The page holding the mask and its pixels there; None: no glyphs to shadow.
    bitmap: Option<(Rc<Page>, b::D2D_RECT_F)>,
    /// Where the mask lies, in DIPs from the text's top left in its layout box.
    rect: Rect,
    used: u64,
}

fn color(c: ColorF) -> b::D2D_COLOR_F {
    b::D2D_COLOR_F {
        r: c.r,
        g: c.g,
        b: c.b,
        a: c.a,
    }
}

impl Device {
    /// The glyphs of `layout` over `area` (DIPs from the layout box's top left), blurred, on a
    /// page: the page and the mask's pixels there.
    fn mask(
        &mut self,
        layout: &TextLayout,
        area: Rect,
        scale: f32,
        blur: f32,
    ) -> Result<(Rc<Page>, b::D2D_RECT_F)> {
        let maker = match &mut self.maker {
            Some(maker) => maker,
            none => none.insert(Maker::new(&self.device)?),
        };
        let size = (
            (area.width() * scale).ceil() as u32,
            (area.height() * scale).ceil() as u32,
        );
        let drawn = maker.render(layout, area, size, scale, blur)?;
        // A clear pixel right of and below each mask keeps its neighbours out of its edges.
        let (w, h) = (size.0 + 1, size.1 + 1);
        if w > PAGE || h > PAGE {
            // Wider or taller than a page (a long title at a high DPI): a page of its own.
            let page = Rc::new(maker.page(w, h)?);
            unsafe { page.0.CopyFromBitmap(None, &drawn, None).ok()? };
            return Ok((page, pixels(0, 0, size)));
        }
        let fresh = || -> Result<Shelf> {
            let page = Rc::new(maker.page(PAGE, PAGE)?);
            Ok(Shelf {
                page,
                x: 0,
                y: 0,
                row_h: 0,
            })
        };
        let mut shelf = match self.shelf.take() {
            Some(shelf) => shelf,
            None => fresh()?,
        };
        if shelf.x + w > PAGE {
            (shelf.x, shelf.y, shelf.row_h) = (0, shelf.y + shelf.row_h, 0);
        }
        if shelf.y + h > PAGE {
            shelf = fresh()?;
        }
        let at = b::D2D_POINT_2U {
            x: shelf.x,
            y: shelf.y,
        };
        unsafe { shelf.page.0.CopyFromBitmap(Some(&at), &drawn, None).ok()? };
        let placed = (shelf.page.clone(), pixels(shelf.x, shelf.y, size));
        shelf.x += w;
        shelf.row_h = shelf.row_h.max(h);
        self.shelf = Some(shelf);
        Ok(placed)
    }
}

/// The rectangle of `size` pixels at (`x`, `y`) on a page (whose DIPs are pixels).
fn pixels(x: u32, y: u32, (w, h): (u32, u32)) -> b::D2D_RECT_F {
    b::D2D_RECT_F {
        left: x as f32,
        top: y as f32,
        right: (x + w) as f32,
        bottom: (y + h) as f32,
    }
}

impl Maker {
    fn new(device: &b::ID2D1Device) -> Result<Self> {
        let context = unsafe { device.CreateDeviceContext(b::D2D1_DEVICE_CONTEXT_OPTIONS_NONE)? };
        let glyphs = unsafe { context.CreateSolidColorBrush(&color(ColorF::BLACK), None)? };
        Ok(Self { context, glyphs })
    }

    /// A bitmap of `w` x `h` pixels in `format`, to draw into (`options` TARGET) or to keep.
    fn bitmap(
        &self,
        w: u32,
        h: u32,
        format: b::DXGI_FORMAT,
        options: b::D2D1_BITMAP_OPTIONS,
    ) -> Result<b::ID2D1Bitmap1> {
        let properties = b::D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: b::D2D1_PIXEL_FORMAT {
                format,
                alphaMode: b::D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: options,
            ..Default::default()
        };
        let size = b::D2D_SIZE_U {
            width: w,
            height: h,
        };
        unsafe { self.context.CreateBitmap(size, None, 0, &properties) }
    }

    /// The glyphs of `layout` in an alpha mask that covers `rect` (DIPs from the layout box's
    /// top left) at `scale`, blurred by `blur` DIPs. The blur is a Gaussian of weighted copies
    /// of the glyphs added up, across and then down: plain bitmap draws, which need none of
    /// the memory a Direct2D blur effect keeps once it has run. The passes run in colour
    /// bitmaps (Direct2D does not add up alpha-only ones); only the result is alpha-only.
    fn render(
        &self,
        layout: &TextLayout,
        rect: Rect,
        (w, h): (u32, u32),
        scale: f32,
        blur: f32,
    ) -> Result<b::ID2D1Bitmap1> {
        let target = |format| self.bitmap(w, h, format, b::D2D1_BITMAP_OPTIONS_TARGET);
        let glyphs = target(b::DXGI_FORMAT_B8G8R8A8_UNORM)?;
        let weights = gaussian(blur * scale);
        let across = if weights.len() > 1 {
            Some(target(b::DXGI_FORMAT_B8G8R8A8_UNORM)?)
        } else {
            None
        };
        let alpha = target(b::DXGI_FORMAT_A8_UNORM)?;
        let layout: b::IDWriteTextLayout = layout.raw().cast()?;
        let clear = color(ColorF::TRANSPARENT);
        let context = &self.context;
        let r = (weights.len() / 2) as f32;
        unsafe {
            // Pixels: the text is scaled, and the copies move by whole pixels.
            context.SetDpi(96.0, 96.0);
            context.SetTarget(&glyphs);
            context.BeginDraw();
            context.SetTransform(
                &(Matrix3x2::translation(-rect.left, -rect.top) * Matrix3x2::scale(scale, scale)),
            );
            context.SetTextAntialiasMode(b::D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            context.Clear(Some(&clear));
            context.DrawTextLayout(
                Vector2::zero(),
                &layout,
                &self.glyphs,
                b::D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
            context.SetTransform(&Matrix3x2::identity());
            if let Some(across) = &across {
                context.SetPrimitiveBlend(b::D2D1_PRIMITIVE_BLEND_ADD);
                for (from, to, horizontal) in [(&glyphs, across, true), (across, &glyphs, false)] {
                    context.SetTarget(to);
                    context.Clear(Some(&clear));
                    for (i, weight) in weights.iter().enumerate() {
                        let d = i as f32 - r;
                        let (dx, dy) = if horizontal { (d, 0.0) } else { (0.0, d) };
                        let dest = b::D2D_RECT_F {
                            left: dx,
                            top: dy,
                            right: w as f32 + dx,
                            bottom: h as f32 + dy,
                        };
                        context.DrawBitmap(
                            from,
                            Some(&dest),
                            *weight,
                            b::D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                            None,
                            None,
                        );
                    }
                }
                context.SetPrimitiveBlend(b::D2D1_PRIMITIVE_BLEND_SOURCE_OVER);
            }
            context.SetTarget(&alpha);
            context.Clear(Some(&clear));
            context.DrawBitmap(
                &glyphs,
                None,
                1.0,
                b::D2D1_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                None,
                None,
            );
            let ended = context.EndDraw(None, None);
            context.SetTarget(None::<&b::ID2D1Image>);
            ended.ok()?;
        }
        Ok(alpha)
    }

    /// A clear alpha-only page of `w` x `h` pixels to copy masks into.
    fn page(&self, w: u32, h: u32) -> Result<Page> {
        let properties = b::D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: b::D2D1_PIXEL_FORMAT {
                format: b::DXGI_FORMAT_A8_UNORM,
                alphaMode: b::D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: b::D2D1_BITMAP_OPTIONS_NONE,
            ..Default::default()
        };
        let clear = vec![0u8; (w * h) as usize];
        let size = b::D2D_SIZE_U {
            width: w,
            height: h,
        };
        let bitmap = unsafe {
            self.context
                .CreateBitmap(size, Some(clear.as_ptr().cast()), w, &properties)?
        };
        Ok(Page(bitmap))
    }
}

/// Normalised weights of a Gaussian with standard deviation `sigma` pixels, out to three of
/// them on each side; a single tap when it is too narrow to blur.
fn gaussian(sigma: f32) -> Vec<f32> {
    if sigma < 0.3 {
        return vec![1.0];
    }
    let r = (sigma * 3.0).ceil() as i32;
    let taps: Vec<f32> = (-r..=r)
        .map(|d| (-((d * d) as f32) / (2.0 * sigma * sigma)).exp())
        .collect();
    let sum: f32 = taps.iter().sum();
    taps.iter().map(|t| t / sum).collect()
}

impl TextShadows {
    /// Fills the shadow of `text`, laid out with `format` as `layout` at `origin` (the layout
    /// box's top left), in `shadow`: blurred by `blur` DIPs (the Gaussian's standard deviation)
    /// and `offset` DIPs below the text. When the device cannot make the masks, the log says so
    /// once and the text has no shadow.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        session: &DrawingSession<'_>,
        layout: &TextLayout,
        text: &str,
        format: &TextFormat,
        origin: Vector2,
        blur: f32,
        offset: f32,
        shadow: ColorF,
    ) {
        if shadow.a <= 0.0 {
            return;
        }
        let Ok(context) = session.raw().cast::<b::ID2D1DeviceContext>() else {
            return;
        };
        let Ok(device) = (unsafe { context.GetDevice() }) else {
            return;
        };
        let id = device.as_raw() as usize;
        if self.failed_on == Some(id) {
            return;
        }
        let caption = Caption {
            layout,
            text,
            format,
        };
        if let Err(error) = self.try_draw(&context, device, caption, origin, blur, offset, shadow) {
            tracing::warn!(%error, "text shadow unavailable; drawing text without it");
            self.failed_on = Some(id);
            self.masks.clear();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn try_draw(
        &mut self,
        context: &b::ID2D1DeviceContext,
        device: b::ID2D1Device,
        caption: Caption<'_>,
        origin: Vector2,
        blur: f32,
        offset: f32,
        shadow: ColorF,
    ) -> Result<()> {
        // A rebuilt device (after a device loss) cannot draw the old masks.
        if self
            .device
            .as_ref()
            .is_none_or(|d| d.device.as_raw() != device.as_raw())
        {
            self.masks.clear();
            self.device = Some(Device {
                fill: unsafe { context.CreateSolidColorBrush(&color(ColorF::BLACK), None)? },
                device,
                maker: None,
                shelf: None,
            });
        }
        let mut dpi = (96.0, 96.0);
        unsafe { context.GetDpi(&mut dpi.0, &mut dpi.1) };
        let scale = dpi.0 / 96.0;
        let m = caption.layout.metrics();
        // One line that fits its box has the same glyphs in any box it fits: only where they
        // sit changes (alignment), so a tab or title that widens or narrows keeps its mask.
        let fits = m.line_count <= 1
            && m.width_including_trailing_whitespace <= m.layout_width
            && m.height <= m.layout_height;
        let size = if fits {
            [0.0, 0.0, scale, blur]
        } else {
            [m.layout_width, m.layout_height, scale, blur]
        }
        .map(f32::to_bits);
        let (text, id) = (caption.text, caption.format.raw().as_raw() as usize);
        let mut hasher = DefaultHasher::new();
        (text, id, size).hash(&mut hasher);
        let key = hasher.finish();
        self.clock += 1;
        let fresh = self.masks.get(&key).is_some_and(|mask| {
            mask.text == text && mask.format.raw().as_raw() as usize == id && mask.size == size
        });
        if !fresh {
            let mask = self.make(caption, scale, blur, size)?;
            if self.masks.len() >= MAX_MASKS {
                self.forget_older_half();
            }
            self.masks.insert(key, mask);
        }
        let Some(mask) = self.masks.get_mut(&key) else {
            return Ok(());
        };
        mask.used = self.clock;
        let (Some((page, source)), Some(device)) = (&mask.bitmap, &self.device) else {
            return Ok(());
        };
        let (x, y) = (origin.x + m.left, origin.y + m.top + offset);
        let dest = b::D2D_RECT_F {
            left: x + mask.rect.left,
            top: y + mask.rect.top,
            right: x + mask.rect.right,
            bottom: y + mask.rect.bottom,
        };
        unsafe {
            device.fill.SetColor(&color(shadow));
            // FillOpacityMask needs aliased geometry; the mask itself is soft.
            let mode = context.GetAntialiasMode();
            context.SetAntialiasMode(b::D2D1_ANTIALIAS_MODE_ALIASED);
            context.FillOpacityMask(&page.0, &device.fill, Some(&dest), Some(source));
            context.SetAntialiasMode(mode);
        }
        Ok(())
    }

    fn make(
        &mut self,
        caption: Caption<'_>,
        scale: f32,
        blur: f32,
        size: [u32; 4],
    ) -> Result<Mask> {
        let m = caption.layout.metrics();
        // The blur reaches three standard deviations; a little more holds glyph overhangs.
        let pad = (blur * 3.0).ceil() + 2.0;
        let left = (m.left - pad).floor();
        let top = (m.top - pad).floor();
        let area = Rect::from_xywh(
            left,
            top,
            (m.left + m.width + pad).ceil() - left,
            (m.top + m.height + pad).ceil() - top,
        );
        let bitmap = match &mut self.device {
            Some(device) if !caption.text.trim().is_empty() && m.width > 0.0 => {
                Some(device.mask(caption.layout, area, scale, blur)?)
            }
            _ => None,
        };
        Ok(Mask {
            text: caption.text.to_string(),
            format: caption.format.clone(),
            size,
            bitmap,
            // From the top left of the text, which the next layouts may place elsewhere.
            rect: Rect::from_xywh(
                area.left - m.left,
                area.top - m.top,
                area.width(),
                area.height(),
            ),
            used: self.clock,
        })
    }

    /// Lets go of the context new masks are made in, and the blur's resources with it, once
    /// drawing has gone quiet; the masks stay, and the next new caption makes one again.
    pub fn trim(&mut self) {
        if let Some(device) = &mut self.device {
            device.maker = None;
        }
    }

    /// Drops every mask (another theme: other blurs, other fonts).
    pub fn clear(&mut self) {
        self.masks.clear();
    }

    fn forget_older_half(&mut self) {
        let mut used: Vec<u64> = self.masks.values().map(|m| m.used).collect();
        used.sort_unstable();
        let keep_from = used[used.len() / 2];
        self.masks.retain(|_, m| m.used >= keep_from);
    }

    /// Masks held (diagnostics and tests).
    pub fn len(&self) -> usize {
        self.masks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_canvas::{GpuDevice, TextAlignment};

    /// Shadows `text` laid out in `rect`, as the fence chrome does.
    fn shadow(
        shadows: &mut TextShadows,
        s: &DrawingSession<'_>,
        text: &str,
        format: &TextFormat,
        rect: Rect,
    ) {
        let layout = TextLayout::new(text, format, rect.width(), rect.height()).unwrap();
        let black = ColorF::new(0.0, 0.0, 0.0, 1.0);
        let origin = Vector2::new(rect.left, rect.top);
        shadows.draw(s, &layout, text, format, origin, 3.0, 4.0, black);
    }

    /// Alpha of the pixel at (x, y) of a premultiplied BGRA read-back `width` pixels wide.
    fn alpha(pixels: &[u8], width: u32, x: u32, y: u32) -> u8 {
        pixels[((y * width + x) * 4 + 3) as usize]
    }

    #[test]
    fn a_blurred_shadow_is_drawn_from_a_cached_mask() {
        let _ole = pecofence_platform::com::OleGuard::init().unwrap();
        let gpu = GpuDevice::new_or_warp().unwrap();
        let (w, h) = (160, 60);
        let target = gpu.create_render_target(w, h).unwrap();
        let format = TextFormat::new("Segoe UI", 20.0).unwrap();
        let rect = Rect::from_xywh(10.0, 10.0, 140.0, 30.0);
        let mut shadows = TextShadows::default();
        for _ in 0..2 {
            target
                .draw(|s| {
                    s.clear(ColorF::TRANSPARENT);
                    shadow(&mut shadows, s, "Shadow", &format, rect);
                    Ok(())
                })
                .unwrap();
        }
        // The second frame reused the mask; a blank caption makes an empty one.
        assert_eq!(shadows.len(), 1);
        let pixels = target.read_pixels().unwrap();
        let column = |x: u32| (0..h).map(|y| alpha(&pixels, w, x, y)).max().unwrap();
        let row = |y: u32| (0..w).map(|x| alpha(&pixels, w, x, y)).max().unwrap();
        // Soft: it reaches past the glyphs, fading out, and nothing is fully opaque.
        assert!((10..90).any(|x| column(x) > 30), "a shadow is drawn");
        assert!(row(40) > 0, "the blur and offset reach below the text");
        assert!(
            pixels.chunks(4).all(|p| p[3] < 250),
            "a blur leaves no solid pixel"
        );
        assert_eq!(column(w - 2), 0, "nothing far right of the text");
        target
            .draw(|s| {
                shadow(&mut shadows, s, "  ", &format, rect);
                Ok(())
            })
            .unwrap();
        assert_eq!(shadows.len(), 2);
    }

    #[test]
    fn one_line_keeps_its_mask_while_its_box_widens() {
        let _ole = pecofence_platform::com::OleGuard::init().unwrap();
        let gpu = GpuDevice::new_or_warp().unwrap();
        let (w, h) = (160, 40);
        let target = gpu.create_render_target(w, h).unwrap();
        let format = TextFormat::new("Segoe UI", 16.0)
            .unwrap()
            .with_alignment(TextAlignment::Center);
        let mut shadows = TextShadows::default();
        // The alpha-weighted middle of what was drawn, in pixels from the left.
        let mut middle = |box_w: f32| {
            target
                .draw(|s| {
                    s.clear(ColorF::TRANSPARENT);
                    shadow(
                        &mut shadows,
                        s,
                        "Tab",
                        &format,
                        Rect::from_xywh(0.0, 0.0, box_w, 30.0),
                    );
                    Ok(())
                })
                .unwrap();
            let pixels = target.read_pixels().unwrap();
            let (mut sum, mut weight) = (0.0, 0.0);
            for y in 0..h {
                for x in 0..w {
                    let a = alpha(&pixels, w, x, y) as f32;
                    sum += a * x as f32;
                    weight += a;
                }
            }
            sum / weight
        };
        let (narrow, wide) = (middle(100.0), middle(140.0));
        assert_eq!(shadows.len(), 1, "one mask for both boxes");
        assert!((wide - narrow - 20.0).abs() < 1.5, "{narrow} {wide}");
    }

    #[test]
    fn masks_share_pages_and_a_long_one_gets_its_own() {
        let _ole = pecofence_platform::com::OleGuard::init().unwrap();
        let gpu = GpuDevice::new_or_warp().unwrap();
        let target = gpu.create_render_target(100, 40).unwrap();
        let format = TextFormat::new("Segoe UI", 12.0).unwrap();
        let mut shadows = TextShadows::default();
        let pages = |shadows: &TextShadows| {
            let mut pages: Vec<*const Page> = shadows
                .masks
                .values()
                .filter_map(|m| m.bitmap.as_ref().map(|(page, _)| Rc::as_ptr(page)))
                .collect();
            pages.sort();
            pages.dedup();
            pages.len()
        };
        target
            .draw(|s| {
                for i in 0..60 {
                    let rect = Rect::from_xywh(0.0, 0.0, 80.0, 32.0);
                    shadow(&mut shadows, s, &format!("Label {i}"), &format, rect);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(shadows.len(), 60);
        assert_eq!(pages(&shadows), 1, "60 small labels fit one page");
        let long = TextFormat::new("Segoe UI", 40.0).unwrap();
        target
            .draw(|s| {
                let rect = Rect::from_xywh(0.0, 0.0, 2000.0, 60.0);
                shadow(&mut shadows, s, &"A long title ".repeat(6), &long, rect);
                shadow(
                    &mut shadows,
                    s,
                    "Label 60",
                    &format,
                    Rect::from_xywh(0.0, 0.0, 80.0, 32.0),
                );
                Ok(())
            })
            .unwrap();
        assert_eq!(
            pages(&shadows),
            2,
            "the long one alone, the next label on the first page"
        );
    }

    #[test]
    fn the_blur_weights_are_a_normalised_gaussian() {
        assert_eq!(gaussian(0.0), vec![1.0]);
        let w = gaussian(2.0);
        assert_eq!(w.len(), 13);
        assert!((w.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert_eq!(w[0], w[12]);
        assert!(w[6] > w[5] && w[5] > w[0]);
    }

    #[test]
    fn the_oldest_masks_go_first() {
        let _ole = pecofence_platform::com::OleGuard::init().unwrap();
        let gpu = GpuDevice::new_or_warp().unwrap();
        let target = gpu.create_render_target(40, 20).unwrap();
        let format = TextFormat::new("Segoe UI", 12.0).unwrap();
        let rect = Rect::from_xywh(0.0, 0.0, 40.0, 20.0);
        let mut shadows = TextShadows::default();
        target
            .draw(|s| {
                for i in 0..MAX_MASKS + 1 {
                    // Blank captions make no bitmap, so this stays quick.
                    shadow(&mut shadows, s, &" ".repeat(i + 1), &format, rect);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(shadows.len(), MAX_MASKS / 2 + 1);
    }
}
