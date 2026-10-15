//! GPU glass over a shared wallpaper texture. Movement only updates a transform:
//! the displacement field, specular ring, source upload and effect graph stay resident.
use crate::{Image, MonitorBackdrop, gpu_bindings as b};
use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};
use windows_canvas::{DrawingSession, Matrix3x2, Rect, Vector2};
use windows_core::{Interface, Result};

fn property<T: Copy>(effect: &b::ID2D1Effect, index: i32, kind: i32, value: &T) -> Result<()> {
    // Only POD scalar/float arrays are passed by this module.
    let bytes = unsafe { std::slice::from_raw_parts((value as *const T).cast(), size_of::<T>()) };
    unsafe { effect.SetValue(index as u32, kind, bytes).ok() }
}

fn scalar(effect: &b::ID2D1Effect, index: i32, value: f32) -> Result<()> {
    property(effect, index, b::D2D1_PROPERTY_TYPE_FLOAT, &value)
}

fn enumeration(effect: &b::ID2D1Effect, index: i32, value: i32) -> Result<()> {
    property(effect, index, b::D2D1_PROPERTY_TYPE_ENUM, &value)
}

fn matrix(effect: &b::ID2D1Effect, value: [f32; 6]) -> Result<()> {
    property(
        effect,
        b::D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX,
        b::D2D1_PROPERTY_TYPE_MATRIX_3X2,
        &value,
    )
}

fn color_matrix(effect: &b::ID2D1Effect, value: [f32; 20]) -> Result<()> {
    enumeration(
        effect,
        b::D2D1_COLORMATRIX_PROP_ALPHA_MODE,
        b::D2D1_COLORMATRIX_ALPHA_MODE_STRAIGHT,
    )?;
    property(
        effect,
        b::D2D1_COLORMATRIX_PROP_COLOR_MATRIX,
        b::D2D1_PROPERTY_TYPE_MATRIX_5X4,
        &value,
    )
}

fn bitmap(
    context: &b::ID2D1DeviceContext,
    width: u32,
    height: u32,
    data: *const std::ffi::c_void,
    pitch: u32,
    format: b::DXGI_FORMAT,
) -> Result<b::ID2D1Bitmap1> {
    let properties = b::D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: b::D2D1_PIXEL_FORMAT {
            format,
            alphaMode: b::D2D1_ALPHA_MODE_PREMULTIPLIED,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        bitmapOptions: b::D2D1_BITMAP_OPTIONS_NONE,
        ..Default::default()
    };
    unsafe {
        context.CreateBitmap(
            b::D2D_SIZE_U { width, height },
            Some(data),
            pitch,
            &properties,
        )
    }
}

struct Wallpaper {
    image: b::ID2D1Image,
}

struct Upload {
    source: Weak<Vec<MonitorBackdrop>>,
    wallpaper: Rc<Wallpaper>,
}

/// Shared by every fence on this D2D device. Rc identity tracks wallpaper/theme changes.
/// An upload lives as long as its backdrop set: the full-resolution pixels are gone after
/// the upload, so the app's bounded round-trip cache decides how many textures stay.
#[derive(Default)]
pub struct WallpaperCache {
    /// Most recent first.
    entries: Vec<Upload>,
    pub uploads: u64,
}

impl WallpaperCache {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Releases the textures of sets nobody holds any more (a replaced wallpaper, or every
    /// set after leaving Liquid Glass, when `get` no longer runs).
    pub fn prune(&mut self) {
        self.entries.retain(|e| e.source.strong_count() > 0);
    }

    /// Uploads `sources` now instead of at its first draw (see `BitmapCache::preload_wallpaper`).
    pub(crate) fn preload(
        &mut self,
        context: &b::ID2D1DeviceContext,
        sources: &Rc<Vec<MonitorBackdrop>>,
    ) -> Result<()> {
        self.get(context, sources).map(drop)
    }

    fn get(
        &mut self,
        context: &b::ID2D1DeviceContext,
        sources: &Rc<Vec<MonitorBackdrop>>,
    ) -> Result<Rc<Wallpaper>> {
        self.entries.retain(|e| e.source.strong_count() > 0);
        let key = Rc::downgrade(sources);
        if let Some(index) = self.entries.iter().position(|e| e.source.ptr_eq(&key)) {
            let entry = self.entries.remove(index);
            let uploaded = entry.wallpaper.clone();
            self.entries.insert(0, entry);
            return Ok(uploaded);
        }
        let valid: Vec<_> = sources
            .iter()
            .filter(|s| s.image.width > 0 && s.image.height > 0)
            .collect();
        if valid.is_empty() {
            return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
                0x80070057u32 as i32,
            )));
        }
        // Full-resolution pixels leave process memory with this upload. If they cannot be
        // rebuilt, the small sampling copy gives a softer wallpaper, kept (not retried) until
        // the set is replaced: retrying would repeat a failed full decode on the UI thread.
        let full: Vec<Option<Image>> = valid
            .iter()
            .map(|s| s.gpu.as_ref().and_then(|g| g.take()))
            .collect();
        let built = (|| -> Result<Rc<Wallpaper>> {
            let composite = unsafe { context.CreateEffect(&b::CLSID_D2D1Composite)? };
            unsafe { composite.SetInputCount(valid.len() as u32).ok()? };
            for (index, (source, full)) in valid.iter().zip(&full).enumerate() {
                let (pixels, downscale) = match (full, &source.gpu) {
                    (Some(image), Some(gpu)) => (image, gpu.downscale),
                    _ => (&source.image, source.downscale),
                };
                let Image {
                    width,
                    height,
                    bgra,
                } = pixels;
                let image = bitmap(
                    context,
                    *width,
                    *height,
                    bgra.as_ptr().cast(),
                    width * 4,
                    b::DXGI_FORMAT_B8G8R8A8_UNORM,
                )?;
                let transform = unsafe { context.CreateEffect(&b::CLSID_D2D12DAffineTransform)? };
                matrix(
                    &transform,
                    [
                        downscale as f32,
                        0.0,
                        0.0,
                        downscale as f32,
                        source.left as f32,
                        source.top as f32,
                    ],
                )?;
                unsafe {
                    transform.SetInput(0, &image.cast::<b::ID2D1Image>()?, true);
                    composite.SetInput(index as u32, &transform.GetOutput()?, true);
                }
            }
            Ok(Rc::new(Wallpaper {
                image: unsafe { composite.GetOutput()? },
            }))
        })();
        let uploaded = match built {
            Ok(uploaded) => uploaded,
            Err(error) => {
                // Keep the pixels for the retry (typically on a rebuilt device).
                for (source, full) in valid.iter().zip(full) {
                    if let (Some(gpu), Some(image)) = (&source.gpu, full) {
                        gpu.restore(image);
                    }
                }
                return Err(error);
            }
        };
        self.uploads += valid.len() as u64;
        self.entries.insert(
            0,
            Upload {
                source: key,
                wallpaper: uploaded.clone(),
            },
        );
        Ok(uploaded)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub frames: u64,
    pub map_builds: u64,
    pub control_frames: u64,
    pub control_map_builds: u64,
}

/// kube.io's SVG chain: displace the backdrop once, let a strongly saturated copy of the
/// refracted backdrop show through the specular ring, then light the ring itself.
const RIM_SATURATION: f32 = 6.0;
const RIM_LIGHT: f32 = 0.4;

/// feColorMatrix `saturate` (Rec. 709 luma weights) in D2D's input-row layout.
fn saturation_matrix(s: f32) -> [f32; 20] {
    let luma = [0.213, 0.715, 0.072];
    let mut values = [0.0; 20];
    for input in 0..3 {
        for output in 0..3 {
            values[input * 4 + output] =
                luma[input] * (1.0 - s) + if input == output { s } else { 0.0 };
        }
    }
    values[15] = 1.0;
    values
}

/// Premultiplied white from the ring's alpha, scaled by `level`. `color_matrix` applies
/// the matrix to premultiplied values, so colour must come from alpha, not an offset.
fn light_matrix(level: f32) -> [f32; 20] {
    let mut values = [0.0; 20];
    values[12..16].fill(level);
    values
}

struct Effects {
    source: Option<Rc<Wallpaper>>,
    border: b::ID2D1Effect,
    position: b::ID2D1Effect,
    displace: b::ID2D1Effect,
    rim: b::ID2D1Effect,
    light: b::ID2D1Effect,
    output: b::ID2D1Effect,
    surface: Rc<Wallpaper>,
    geometry: Option<(u32, u32, u32, u32)>,
    opacity: Option<u32>,
    light_level: Option<u32>,
}

impl Effects {
    fn new(context: &b::ID2D1DeviceContext) -> Result<Self> {
        let effect = |id| unsafe { context.CreateEffect(id) };
        let border = effect(&b::CLSID_D2D1Border)?;
        enumeration(
            &border,
            b::D2D1_BORDER_PROP_EDGE_MODE_X,
            b::D2D1_BORDER_EDGE_MODE_CLAMP,
        )?;
        enumeration(
            &border,
            b::D2D1_BORDER_PROP_EDGE_MODE_Y,
            b::D2D1_BORDER_EDGE_MODE_CLAMP,
        )?;
        let position = effect(&b::CLSID_D2D12DAffineTransform)?;
        let displace = effect(&b::CLSID_D2D1DisplacementMap)?;
        enumeration(
            &displace,
            b::D2D1_DISPLACEMENTMAP_PROP_X_CHANNEL_SELECT,
            b::D2D1_CHANNEL_SELECTOR_R,
        )?;
        enumeration(
            &displace,
            b::D2D1_DISPLACEMENTMAP_PROP_Y_CHANNEL_SELECT,
            b::D2D1_CHANNEL_SELECTOR_G,
        )?;
        let saturate = effect(&b::CLSID_D2D1ColorMatrix)?;
        color_matrix(&saturate, saturation_matrix(RIM_SATURATION))?;
        property(
            &saturate,
            b::D2D1_COLORMATRIX_PROP_CLAMP_OUTPUT,
            b::D2D1_PROPERTY_TYPE_BOOL,
            &1u32,
        )?;
        // Input 1 (the ring) arrives with the geometry.
        let rim = effect(&b::CLSID_D2D1AlphaMask)?;
        let light = effect(&b::CLSID_D2D1ColorMatrix)?;
        color_matrix(&light, light_matrix(RIM_LIGHT))?;
        let composite = effect(&b::CLSID_D2D1Composite)?;
        let output = effect(&b::CLSID_D2D1ColorMatrix)?;
        unsafe {
            position.SetInput(0, &border.GetOutput()?, true);
            displace.SetInput(0, &position.GetOutput()?, true);
            saturate.SetInput(0, &displace.GetOutput()?, true);
            rim.SetInput(0, &saturate.GetOutput()?, true);
            composite.SetInputCount(3).ok()?;
            composite.SetInput(0, &displace.GetOutput()?, true);
            composite.SetInput(1, &rim.GetOutput()?, true);
            composite.SetInput(2, &light.GetOutput()?, true);
            output.SetInput(0, &composite.GetOutput()?, true);
        }
        Ok(Self {
            source: None,
            border,
            position,
            displace,
            rim,
            light,
            output,
            surface: Rc::new(Wallpaper {
                image: unsafe { composite.GetOutput()? },
            }),
            geometry: None,
            opacity: None,
            light_level: Some(RIM_LIGHT.to_bits()),
        })
    }

    fn geometry(
        &mut self,
        context: &b::ID2D1DeviceContext,
        w: u32,
        h: u32,
        scale: f32,
        radius: f32,
    ) -> Result<bool> {
        let key = (w, h, scale.to_bits(), radius.to_bits());
        if self.geometry == Some(key) {
            return Ok(false);
        }
        let field = crate::liquid_glass::displacement_field(w, h, scale, radius);
        let map = bitmap(
            context,
            field.width,
            field.height,
            field.rgba.as_ptr().cast(),
            field.width * 8,
            b::DXGI_FORMAT_R16G16B16A16_FLOAT,
        )?;
        // Effects use pixel coordinates; bitmap DPI alone does not resize input 1.
        // Explicitly resample the cached field to the window, clamping its outer texels.
        let map_border = unsafe { context.CreateEffect(&b::CLSID_D2D1Border)? };
        enumeration(
            &map_border,
            b::D2D1_BORDER_PROP_EDGE_MODE_X,
            b::D2D1_BORDER_EDGE_MODE_CLAMP,
        )?;
        enumeration(
            &map_border,
            b::D2D1_BORDER_PROP_EDGE_MODE_Y,
            b::D2D1_BORDER_EDGE_MODE_CLAMP,
        )?;
        let map_scale = unsafe { context.CreateEffect(&b::CLSID_D2D12DAffineTransform)? };
        matrix(
            &map_scale,
            [
                w as f32 / field.width as f32,
                0.0,
                0.0,
                h as f32 / field.height as f32,
                0.0,
                0.0,
            ],
        )?;
        let ring = bitmap(
            context,
            field.rim_width,
            field.rim_height,
            field.rim.as_ptr().cast(),
            field.rim_width,
            b::DXGI_FORMAT_A8_UNORM,
        )?
        .cast::<b::ID2D1Image>()?;
        scalar(
            &self.displace,
            b::D2D1_DISPLACEMENTMAP_PROP_SCALE,
            field.scale_px,
        )?;
        unsafe {
            map_border.SetInput(0, &map.cast::<b::ID2D1Image>()?, true);
            map_scale.SetInput(0, &map_border.GetOutput()?, true);
            self.displace.SetInput(1, &map_scale.GetOutput()?, true);
            self.rim.SetInput(1, &ring, true);
            self.light.SetInput(0, &ring, true);
        }
        self.geometry = Some(key);
        Ok(true)
    }

    fn set_light(&mut self, level: f32) -> Result<()> {
        if self.light_level != Some(level.to_bits()) {
            color_matrix(&self.light, light_matrix(level))?;
            self.light_level = Some(level.to_bits());
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct GpuGlass {
    effects: Option<Effects>,
    /// Surface row (device px) the last `draw` placed the plate's lens at: a folded title row
    /// moves it down, and header lenses sample the refracted plate relative to it.
    origin_y: f32,
    failed: bool,
    stats: Stats,
    controls: HashMap<u128, Control>,
    text_inks: HashMap<u128, (bool, u64)>,
}

struct Control {
    effects: Effects,
    brush: b::ID2D1ImageBrush,
    frame: u64,
}

impl GpuGlass {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// Draws the plate lens for the screen rect `rect` at surface row `origin_y` (device px;
    /// non-zero while title on hover folds the title row away).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        session: &DrawingSession<'_>,
        cache: &mut WallpaperCache,
        sources: &Rc<Vec<MonitorBackdrop>>,
        rect: [i32; 4],
        origin_y: f32,
        scale: f32,
        radius: f32,
        opacity: f32,
        hover: f32,
    ) -> Result<()> {
        if self.failed {
            return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
                0x80004005u32 as i32,
            )));
        }
        let saved = session.transform();
        self.controls
            .retain(|key, control| *key < 5 || control.frame + 1 >= self.stats.frames);
        // Rows, column captions and section headers (keys 0x10000 to 0x3FFFF) are drawn on
        // the content surface, which does not count frames: chrome-only frames (a title hover)
        // must not forget their ink. Visible slots bound how many there are.
        self.text_inks.retain(|key, (_, frame)| {
            (0x10000..0x40000).contains(key) || *frame + 1 >= self.stats.frames
        });
        session.set_transform(&Matrix3x2::identity());
        self.origin_y = origin_y;
        let result = self.draw_inner(
            session, cache, sources, rect, origin_y, scale, radius, opacity, hover,
        );
        session.set_transform(&saved);
        if let Err(error) = &result {
            // E_OUTOFMEMORY, D3DERR_OUTOFVIDEOMEMORY: this frame is solid, the next tries again.
            let out_of_memory = [0x8007000Eu32, 0x8876017C].contains(&(error.code().0 as u32));
            if out_of_memory {
                tracing::debug!(%error, "GPU glass out of memory for this frame");
            } else if !windows_canvas::is_device_lost(error.code()) {
                tracing::warn!(%error, "GPU glass unavailable; using readable solid material");
                self.failed = true;
            }
        }
        result
    }

    pub fn control_foreground(
        &mut self,
        key: u128,
        theme: crate::Theme,
        sources: &[MonitorBackdrop],
        area: [i32; 4],
        opacity: f32,
        tint: Option<windows_canvas::ColorF>,
    ) -> crate::Theme {
        if self.failed {
            return theme;
        }
        // Selected tabs still own their inactive lens. Keep it warm for the next switch.
        if let Some(control) = self.controls.get_mut(&key) {
            control.frame = self.stats.frames;
        }
        let sample = crate::liquid_glass::foreground_sample(sources, area);
        let previous = self.text_inks.get(&key).map(|(dark, _)| *dark);
        let foreground = theme.over_glass_with_previous_ink(&sample, opacity, tint, previous);
        self.text_inks
            .insert(key, (foreground.text_primary.r < 0.5, self.stats.frames));
        foreground
    }

    /// A small lens samples the already-refracted fence surface, before its opacity and
    /// text. This shares the parent effect graph and never captures the desktop or uploads
    /// another wallpaper. ImageBrush provides an antialiased rounded mask at full DPI.
    pub fn draw_control(
        &mut self,
        session: &DrawingSession<'_>,
        key: u128,
        rect: Rect,
        scale: f32,
        radius: f32,
        opacity: f32,
    ) -> Result<bool> {
        let Some(parent) = self.effects.as_ref().filter(|_| !self.failed) else {
            return Ok(false);
        };
        if rect.width() <= 0.0 || rect.height() <= 0.0 || opacity <= 0.0 {
            return Ok(true);
        }
        let context: b::ID2D1DeviceContext = session.raw().cast()?;
        let source = parent.surface.clone();
        if let std::collections::hash_map::Entry::Vacant(entry) = self.controls.entry(key) {
            let effects = Effects::new(&context)?;
            let props = b::D2D1_IMAGE_BRUSH_PROPERTIES {
                sourceRectangle: b::D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: 1.0,
                    bottom: 1.0,
                },
                extendModeX: b::D2D1_EXTEND_MODE_CLAMP,
                extendModeY: b::D2D1_EXTEND_MODE_CLAMP,
                interpolationMode: b::D2D1_INTERPOLATION_MODE_LINEAR,
            };
            let brush = unsafe { context.CreateImageBrush(&effects.surface.image, &props, None)? };
            entry.insert(Control {
                effects,
                brush,
                frame: self.stats.frames,
            });
        }
        let control = self.controls.get_mut(&key).unwrap();
        control.frame = self.stats.frames;
        let effects = &mut control.effects;
        if effects
            .source
            .as_ref()
            .is_none_or(|old| !Rc::ptr_eq(old, &source))
        {
            unsafe { effects.border.SetInput(0, &source.image, true) };
            effects.source = Some(source);
        }
        let (w, h) = (
            (rect.width() * scale).ceil().max(1.0) as u32,
            (rect.height() * scale).ceil().max(1.0) as u32,
        );
        if effects.geometry(&context, w, h, scale, radius)? {
            self.stats.control_map_builds += 1;
        }
        let (x, y) = (rect.left * scale, rect.top * scale);
        // The parent surface starts at the plate top (`origin_y`), not at the window top.
        matrix(
            &effects.position,
            [1.0, 0.0, 0.0, 1.0, -x, -(y - self.origin_y)],
        )?;
        let bounds = b::D2D_RECT_F {
            left: x,
            top: y,
            right: rect.right * scale,
            bottom: rect.bottom * scale,
        };
        let round = b::D2D1_ROUNDED_RECT {
            rect: bounds,
            radiusX: radius * scale,
            radiusY: radius * scale,
        };
        let saved = session.transform();
        session.set_transform(&Matrix3x2::identity());
        unsafe {
            control.brush.SetSourceRectangle(&b::D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: w as f32,
                bottom: h as f32,
            });
            control
                .brush
                .SetTransform(&windows_numerics::Matrix3x2::translation(x, y));
            control.brush.SetOpacity(opacity.clamp(0.0, 1.0));
            context.FillRoundedRectangle(&round, &control.brush);
        }
        session.set_transform(&saved);
        self.stats.control_frames += 1;
        Ok(true)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_inner(
        &mut self,
        session: &DrawingSession<'_>,
        cache: &mut WallpaperCache,
        sources: &Rc<Vec<MonitorBackdrop>>,
        rect: [i32; 4],
        origin_y: f32,
        scale: f32,
        radius: f32,
        opacity: f32,
        hover: f32,
    ) -> Result<()> {
        let context: b::ID2D1DeviceContext = session.raw().cast()?;
        let source = cache.get(&context, sources)?;
        if self.effects.is_none() {
            self.effects = Some(Effects::new(&context)?);
        }
        let effects = self.effects.as_mut().unwrap();
        if effects
            .source
            .as_ref()
            .is_none_or(|old| !Rc::ptr_eq(old, &source))
        {
            unsafe { effects.border.SetInput(0, &source.image, true) };
            effects.source = Some(source);
        }
        let w = rect[2].max(1) as u32;
        let h = rect[3].max(1) as u32;
        if effects.geometry(&context, w, h, scale, radius)? {
            self.stats.map_builds += 1;
        }
        matrix(
            &effects.position,
            [1.0, 0.0, 0.0, 1.0, -rect[0] as f32, -rect[1] as f32],
        )?;
        let opacity = opacity.clamp(0.0, 1.0);
        if effects.opacity != Some(opacity.to_bits()) {
            let mut values = [0.0; 20];
            for i in [0, 5, 10, 15] {
                values[i] = opacity;
            }
            color_matrix(&effects.output, values)?;
            effects.opacity = Some(opacity.to_bits());
        }
        // Hover lifts the rim light with the application's existing animation clock.
        effects.set_light(RIM_LIGHT * (1.0 + 0.35 * hover.clamp(0.0, 1.0)))?;
        let area = b::D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: w as f32,
            bottom: h as f32,
        };
        unsafe {
            context.DrawImage(
                &effects.output.GetOutput()?,
                Some(&Vector2::new(0.0, origin_y)),
                Some(&area),
                b::D2D1_INTERPOLATION_MODE_LINEAR,
                b::D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );
        }
        self.stats.frames += 1;
        Ok(())
    }
}
