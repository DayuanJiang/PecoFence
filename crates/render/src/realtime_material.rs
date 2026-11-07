//! A bounded, GPU-only live background shared by every fence on the render device.
use crate::gpu_bindings as b;
use pecofence_core::{RealtimeGlassQuality, RealtimeGlassSettings};
use std::mem::size_of;
use windows_core::{IUnknown, Interface, Result};

pub(crate) struct RealtimeMaterial {
    context: b::ID2D1DeviceContext,
    surface: IUnknown,
    size: (u32, u32),
    bounds: [i32; 4],
    options: RealtimeGlassSettings,
    source: b::ID2D1Bitmap1,
    target: b::ID2D1Bitmap1,
    down: b::ID2D1Effect,
    color: b::ID2D1Effect,
    output: b::ID2D1Effect,
}

fn property<T: Copy>(effect: &b::ID2D1Effect, index: i32, kind: i32, value: &T) -> Result<()> {
    let bytes = unsafe { std::slice::from_raw_parts((value as *const T).cast(), size_of::<T>()) };
    unsafe { effect.SetValue(index as u32, kind, bytes).ok() }
}

fn transform(effect: &b::ID2D1Effect, matrix: [f32; 6]) -> Result<()> {
    property(
        effect,
        b::D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX,
        b::D2D1_PROPERTY_TYPE_MATRIX_3X2,
        &matrix,
    )
}

impl RealtimeMaterial {
    pub(crate) fn matches(
        &self,
        surface: &IUnknown,
        size: (u32, u32),
        bounds: [i32; 4],
        options: &RealtimeGlassSettings,
    ) -> bool {
        self.surface.as_raw() == surface.as_raw()
            && self.size == size
            && self.bounds == bounds
            && self.options == *options
    }

    pub(crate) fn new(
        context: b::ID2D1DeviceContext,
        surface: &IUnknown,
        size: (u32, u32),
        bounds: [i32; 4],
        options: &RealtimeGlassSettings,
    ) -> Result<Self> {
        let divisor = match options.quality {
            RealtimeGlassQuality::Performance => 4,
            RealtimeGlassQuality::Balanced => 2,
            RealtimeGlassQuality::Quality => 1,
        };
        let (width, height) = (
            size.0.div_ceil(divisor).max(1),
            size.1.div_ceil(divisor).max(1),
        );
        // The capture module bounds the native surface too. Keep material allocations bounded.
        if width as u64 * height as u64 > 64_000_000 || size.0 == 0 || size.1 == 0 {
            return Err(windows_core::Error::from_hresult(windows_core::HRESULT(
                0x80070057u32 as i32,
            )));
        }
        let props = |alpha, flags| b::D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: b::D2D1_PIXEL_FORMAT {
                format: b::DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: alpha,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: flags,
            ..Default::default()
        };
        let dxgi: b::IDXGISurface = surface.cast()?;
        let source = unsafe {
            context.CreateBitmapFromDxgiSurface(
                &dxgi,
                Some(&props(
                    b::D2D1_ALPHA_MODE_IGNORE,
                    b::D2D1_BITMAP_OPTIONS_NONE,
                )),
            )?
        };
        let target = unsafe {
            context.CreateBitmap(
                b::D2D_SIZE_U { width, height },
                None,
                0,
                &props(
                    b::D2D1_ALPHA_MODE_PREMULTIPLIED,
                    b::D2D1_BITMAP_OPTIONS_TARGET,
                ),
            )?
        };
        let effect = |id| unsafe { context.CreateEffect(id) };
        let down = effect(&b::CLSID_D2D12DAffineTransform)?;
        let blur = effect(&b::CLSID_D2D1GaussianBlur)?;
        let color = effect(&b::CLSID_D2D1ColorMatrix)?;
        let output = effect(&b::CLSID_D2D12DAffineTransform)?;
        let dx = width as f32 / size.0 as f32;
        let dy = height as f32 / size.1 as f32;
        transform(&down, [dx, 0.0, 0.0, dy, 0.0, 0.0])?;
        property(
            &blur,
            b::D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION,
            b::D2D1_PROPERTY_TYPE_FLOAT,
            &(options.blur * dx.min(dy)),
        )?;
        property(
            &blur,
            b::D2D1_GAUSSIANBLUR_PROP_BORDER_MODE,
            b::D2D1_PROPERTY_TYPE_ENUM,
            &b::D2D1_BORDER_MODE_HARD,
        )?;
        let saturation = options.saturation;
        let tint = options.tint;
        let luma = [0.2126f32, 0.7152, 0.0722];
        let mut colors = [0.0f32; 20];
        for input in 0..3 {
            for channel in 0..3 {
                colors[input * 4 + channel] = (luma[input] * (1.0 - saturation)
                    + if input == channel { saturation } else { 0.0 })
                    * (1.0 - tint);
            }
        }
        colors[15] = 1.0;
        colors[16..19].fill(tint);
        property(
            &color,
            b::D2D1_COLORMATRIX_PROP_COLOR_MATRIX,
            b::D2D1_PROPERTY_TYPE_MATRIX_5X4,
            &colors,
        )?;
        property(
            &color,
            b::D2D1_COLORMATRIX_PROP_CLAMP_OUTPUT,
            b::D2D1_PROPERTY_TYPE_BOOL,
            &1u32,
        )?;
        let target_image: b::ID2D1Image = target.cast()?;
        unsafe {
            down.SetInput(0, &source.cast::<b::ID2D1Image>()?, true);
            blur.SetInput(0, &down.GetOutput()?, true);
            color.SetInput(0, &blur.GetOutput()?, true);
            output.SetInput(0, &target_image, true);
        }
        // Physical desktop coordinates, including monitors left/above the primary display.
        transform(
            &output,
            [
                bounds[2] as f32 / width as f32,
                0.0,
                0.0,
                bounds[3] as f32 / height as f32,
                bounds[0] as f32,
                bounds[1] as f32,
            ],
        )?;
        Ok(Self {
            context,
            surface: surface.clone(),
            size,
            bounds,
            options: options.clone(),
            source,
            target,
            down,
            color,
            output,
        })
    }

    pub(crate) fn update(&mut self) -> Result<()> {
        // External D3D copy does not tell D2D about content changes. Invalidate explicitly.
        let source: b::ID2D1Image = self.source.cast()?;
        let target: b::ID2D1Image = self.target.cast()?;
        let image = unsafe { self.color.GetOutput()? };
        unsafe {
            self.down.SetInput(0, &source, true);
            self.context.SetTarget(&target);
            self.context.BeginDraw();
            self.context.Clear(None);
            self.context.DrawImage(
                &image,
                None,
                None,
                b::D2D1_INTERPOLATION_MODE_LINEAR,
                b::D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );
            let result = self.context.EndDraw(None, None).ok();
            self.context.SetTarget(None::<&b::ID2D1Image>);
            self.output.SetInput(0, &target, true);
            result
        }
    }

    pub(crate) fn image(&self) -> Result<b::ID2D1Image> {
        unsafe { self.output.GetOutput() }
    }

    pub(crate) fn refraction(&self) -> f32 {
        self.options.refraction
    }
}
