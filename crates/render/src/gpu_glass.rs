//! GPU glass over a shared wallpaper texture. Movement only updates a transform:
//! the displacement field, glints, source upload and effect graph stay resident.
use crate::{Image, MonitorBackdrop, gpu_bindings as b};
use std::{collections::HashMap, rc::Rc};
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
    float: bool,
) -> Result<b::ID2D1Bitmap1> {
    let properties = b::D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: b::D2D1_PIXEL_FORMAT {
            format: if float {
                b::DXGI_FORMAT_R32G32B32A32_FLOAT
            } else {
                b::DXGI_FORMAT_B8G8R8A8_UNORM
            },
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

/// Shared by every fence on this D2D device. Rc identity tracks wallpaper/theme changes.
#[derive(Default)]
pub struct WallpaperCache {
    source: Option<Rc<Vec<MonitorBackdrop>>>,
    uploaded: Option<Rc<Wallpaper>>,
    pub uploads: u64,
}

impl WallpaperCache {
    pub fn clear(&mut self) {
        self.source = None;
        self.uploaded = None;
    }

    fn get(
        &mut self,
        context: &b::ID2D1DeviceContext,
        sources: &Rc<Vec<MonitorBackdrop>>,
    ) -> Result<Rc<Wallpaper>> {
        if self.source.as_ref().is_some_and(|s| Rc::ptr_eq(s, sources)) {
            return Ok(self.uploaded.as_ref().unwrap().clone());
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
        let composite = unsafe { context.CreateEffect(&b::CLSID_D2D1Composite)? };
        unsafe { composite.SetInputCount(valid.len() as u32).ok()? };
        for (index, source) in valid.into_iter().enumerate() {
            let Image {
                width,
                height,
                bgra,
            } = &source.image;
            let image = bitmap(
                context,
                *width,
                *height,
                bgra.as_ptr().cast(),
                width * 4,
                false,
            )?;
            let transform = unsafe { context.CreateEffect(&b::CLSID_D2D12DAffineTransform)? };
            matrix(
                &transform,
                [
                    source.downscale as f32,
                    0.0,
                    0.0,
                    source.downscale as f32,
                    source.left as f32,
                    source.top as f32,
                ],
            )?;
            unsafe {
                transform.SetInput(0, &image.cast::<b::ID2D1Image>()?, true);
                composite.SetInput(index as u32, &transform.GetOutput()?, true);
            }
            self.uploads += 1;
        }
        let uploaded = Rc::new(Wallpaper {
            image: unsafe { composite.GetOutput()? },
        });
        self.source = Some(sources.clone());
        self.uploaded = Some(uploaded.clone());
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

struct Effects {
    source: Option<Rc<Wallpaper>>,
    blur: b::ID2D1Effect,
    position: b::ID2D1Effect,
    displace: [b::ID2D1Effect; 3],
    output: b::ID2D1Effect,
    surface: Rc<Wallpaper>,
    glints: Option<b::ID2D1Bitmap1>,
    geometry: Option<(u32, u32, u32, u32)>,
    opacity: Option<u32>,
}

impl Effects {
    fn new(context: &b::ID2D1DeviceContext) -> Result<Self> {
        let effect = |id| unsafe { context.CreateEffect(id) };
        let blur = effect(&b::CLSID_D2D1GaussianBlur)?;
        enumeration(
            &blur,
            b::D2D1_GAUSSIANBLUR_PROP_BORDER_MODE,
            b::D2D1_BORDER_MODE_HARD,
        )?;
        property(
            &blur,
            b::D2D1_PROPERTY_CACHED,
            b::D2D1_PROPERTY_TYPE_BOOL,
            &1u32,
        )?;
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
        unsafe {
            border.SetInput(0, &blur.GetOutput()?, true);
            position.SetInput(0, &border.GetOutput()?, true);
        }
        let displace = [
            effect(&b::CLSID_D2D1DisplacementMap)?,
            effect(&b::CLSID_D2D1DisplacementMap)?,
            effect(&b::CLSID_D2D1DisplacementMap)?,
        ];
        let mut channels = Vec::new();
        for (i, displacement) in displace.iter().enumerate() {
            enumeration(
                displacement,
                b::D2D1_DISPLACEMENTMAP_PROP_X_CHANNEL_SELECT,
                b::D2D1_CHANNEL_SELECTOR_R,
            )?;
            enumeration(
                displacement,
                b::D2D1_DISPLACEMENTMAP_PROP_Y_CHANNEL_SELECT,
                b::D2D1_CHANNEL_SELECTOR_G,
            )?;
            unsafe { displacement.SetInput(0, &position.GetOutput()?, true) };
            let channel = effect(&b::CLSID_D2D1ColorMatrix)?;
            let mut values = [0.0; 20];
            values[i * 5] = 1.0;
            values[15] = 1.0;
            color_matrix(&channel, values)?;
            unsafe { channel.SetInput(0, &displacement.GetOutput()?, true) };
            channels.push(channel);
        }
        let add = |a: &b::ID2D1Effect, c: &b::ID2D1Effect| -> Result<b::ID2D1Effect> {
            let sum = effect(&b::CLSID_D2D1ArithmeticComposite)?;
            property(
                &sum,
                b::D2D1_ARITHMETICCOMPOSITE_PROP_COEFFICIENTS,
                b::D2D1_PROPERTY_TYPE_VECTOR4,
                &[0.0f32, 1.0, 1.0, 0.0],
            )?;
            property(
                &sum,
                b::D2D1_ARITHMETICCOMPOSITE_PROP_CLAMP_OUTPUT,
                b::D2D1_PROPERTY_TYPE_BOOL,
                &1u32,
            )?;
            unsafe {
                sum.SetInput(0, &a.GetOutput()?, true);
                sum.SetInput(1, &c.GetOutput()?, true);
            }
            Ok(sum)
        };
        let red_green = add(&channels[0], &channels[1])?;
        let rgb = add(&red_green, &channels[2])?;
        let saturation = effect(&b::CLSID_D2D1Saturation)?;
        scalar(&saturation, b::D2D1_SATURATION_PROP_SATURATION, 1.08)?;
        let output = effect(&b::CLSID_D2D1ColorMatrix)?;
        unsafe {
            saturation.SetInput(0, &rgb.GetOutput()?, true);
            output.SetInput(0, &saturation.GetOutput()?, true);
        }
        Ok(Self {
            source: None,
            blur,
            position,
            displace,
            output,
            surface: Rc::new(Wallpaper {
                image: unsafe { saturation.GetOutput()? },
            }),
            glints: None,
            geometry: None,
            opacity: None,
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
            field.width * 16,
            true,
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
        unsafe {
            map_border.SetInput(0, &map.cast::<b::ID2D1Image>()?, true);
            map_scale.SetInput(0, &map_border.GetOutput()?, true);
        }
        for (index, displacement) in self.displace.iter().enumerate() {
            let factor = [1.0 - 0.045, 1.0, 1.0 + 0.045][index];
            scalar(
                displacement,
                b::D2D1_DISPLACEMENTMAP_PROP_SCALE,
                field.scale_px * factor,
            )?;
            unsafe { displacement.SetInput(1, &map_scale.GetOutput()?, true) };
        }
        self.glints = Some(bitmap(
            context,
            field.width,
            field.height,
            field.glints.as_ptr().cast(),
            field.width * 4,
            false,
        )?);
        if self.geometry.is_none_or(|old| old.2 != scale.to_bits()) {
            scalar(
                &self.blur,
                b::D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION,
                1.25 * scale,
            )?;
        }
        self.geometry = Some(key);
        Ok(true)
    }
}

#[derive(Default)]
pub struct GpuGlass {
    effects: Option<Effects>,
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

    pub fn draw(
        &mut self,
        session: &DrawingSession<'_>,
        cache: &mut WallpaperCache,
        sources: &Rc<Vec<MonitorBackdrop>>,
        rect: [i32; 4],
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
        self.text_inks
            .retain(|_, (_, frame)| *frame + 1 >= self.stats.frames);
        session.set_transform(&Matrix3x2::identity());
        let result = self.draw_inner(session, cache, sources, rect, scale, radius, opacity, hover);
        session.set_transform(&saved);
        if let Err(error) = &result {
            if !windows_canvas::is_device_lost(error.code()) {
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
            unsafe { effects.blur.SetInput(0, &source.image, true) };
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
        matrix(&effects.position, [1.0, 0.0, 0.0, 1.0, -x, -y])?;
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

    fn draw_inner(
        &mut self,
        session: &DrawingSession<'_>,
        cache: &mut WallpaperCache,
        sources: &Rc<Vec<MonitorBackdrop>>,
        rect: [i32; 4],
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
            unsafe { effects.blur.SetInput(0, &source.image, true) };
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
        let area = b::D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: w as f32,
            bottom: h as f32,
        };
        unsafe {
            context.DrawImage(
                &effects.output.GetOutput()?,
                Some(&Vector2::new(0.0, 0.0)),
                Some(&area),
                b::D2D1_INTERPOLATION_MODE_LINEAR,
                b::D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );
            let light = (0.85 + 0.15 * opacity + 0.18 * hover.clamp(0.0, 1.0)) / 1.18;
            context.DrawBitmap(
                effects.glints.as_ref().unwrap(),
                Some(&area),
                opacity * light,
                b::D2D1_INTERPOLATION_MODE_LINEAR,
                None,
                None,
            );
        }
        self.stats.frames += 1;
        Ok(())
    }
}
