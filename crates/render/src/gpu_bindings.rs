pub const CLSID_D2D12DAffineTransform: windows_core::GUID =
    windows_core::GUID::from_u128(0x6aa97485_6354_4cfc_908c_e4a74f62c96c);
pub const CLSID_D2D1ArithmeticComposite: windows_core::GUID =
    windows_core::GUID::from_u128(0xfc151437_049a_4784_a24a_f1c4daf20987);
pub const CLSID_D2D1Border: windows_core::GUID =
    windows_core::GUID::from_u128(0x2a2d49c0_4acf_43c7_8c6a_7c4a27874d27);
pub const CLSID_D2D1ColorMatrix: windows_core::GUID =
    windows_core::GUID::from_u128(0x921f03d6_641c_47df_852d_b4bb6153ae11);
pub const CLSID_D2D1Composite: windows_core::GUID =
    windows_core::GUID::from_u128(0x48fc9f51_f6ac_48f1_8b58_3b28ac46f76d);
pub const CLSID_D2D1DisplacementMap: windows_core::GUID =
    windows_core::GUID::from_u128(0xedc48364_0417_4111_9450_43845fa9f890);
pub const CLSID_D2D1GaussianBlur: windows_core::GUID =
    windows_core::GUID::from_u128(0x1feb6d69_2fe6_4ac9_8c58_1d7f93e7a6a5);
pub const CLSID_D2D1Saturation: windows_core::GUID =
    windows_core::GUID::from_u128(0x5cb2d9cf_327d_459f_a0ce_40c0b2086bf7);
pub type D2D1_2DAFFINETRANSFORM_PROP = i32;
pub const D2D1_2DAFFINETRANSFORM_PROP_TRANSFORM_MATRIX: D2D1_2DAFFINETRANSFORM_PROP = 2;
pub type D2D1_ALPHA_MODE = i32;
pub const D2D1_ALPHA_MODE_IGNORE: D2D1_ALPHA_MODE = 3;
pub const D2D1_ALPHA_MODE_PREMULTIPLIED: D2D1_ALPHA_MODE = 1;
pub type D2D1_ANTIALIAS_MODE = i32;
pub type D2D1_ARITHMETICCOMPOSITE_PROP = i32;
pub const D2D1_ARITHMETICCOMPOSITE_PROP_CLAMP_OUTPUT: D2D1_ARITHMETICCOMPOSITE_PROP = 1;
pub const D2D1_ARITHMETICCOMPOSITE_PROP_COEFFICIENTS: D2D1_ARITHMETICCOMPOSITE_PROP = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_BITMAP_BRUSH_PROPERTIES {
    pub extendModeX: D2D1_EXTEND_MODE,
    pub extendModeY: D2D1_EXTEND_MODE,
    pub interpolationMode: D2D1_BITMAP_INTERPOLATION_MODE,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_BITMAP_BRUSH_PROPERTIES1 {
    pub extendModeX: D2D1_EXTEND_MODE,
    pub extendModeY: D2D1_EXTEND_MODE,
    pub interpolationMode: D2D1_INTERPOLATION_MODE,
}
pub type D2D1_BITMAP_INTERPOLATION_MODE = i32;
pub const D2D1_BITMAP_INTERPOLATION_MODE_LINEAR: D2D1_BITMAP_INTERPOLATION_MODE = 1;
pub type D2D1_BITMAP_OPTIONS = u32;
pub const D2D1_BITMAP_OPTIONS_NONE: D2D1_BITMAP_OPTIONS = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_BITMAP_PROPERTIES {
    pub pixelFormat: D2D1_PIXEL_FORMAT,
    pub dpiX: f32,
    pub dpiY: f32,
}
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct D2D1_BITMAP_PROPERTIES1 {
    pub pixelFormat: D2D1_PIXEL_FORMAT,
    pub dpiX: f32,
    pub dpiY: f32,
    pub bitmapOptions: D2D1_BITMAP_OPTIONS,
    pub colorContext: core::mem::ManuallyDrop<Option<ID2D1ColorContext>>,
}
pub type D2D1_BORDER_EDGE_MODE = i32;
pub const D2D1_BORDER_EDGE_MODE_CLAMP: D2D1_BORDER_EDGE_MODE = 0;
pub type D2D1_BORDER_MODE = i32;
pub const D2D1_BORDER_MODE_HARD: D2D1_BORDER_MODE = 1;
pub type D2D1_BORDER_PROP = i32;
pub const D2D1_BORDER_PROP_EDGE_MODE_X: D2D1_BORDER_PROP = 0;
pub const D2D1_BORDER_PROP_EDGE_MODE_Y: D2D1_BORDER_PROP = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_BRUSH_PROPERTIES {
    pub opacity: f32,
    pub transform: windows_numerics::Matrix3x2,
}
pub type D2D1_BUFFER_PRECISION = i32;
pub type D2D1_CHANNEL_SELECTOR = i32;
pub const D2D1_CHANNEL_SELECTOR_G: D2D1_CHANNEL_SELECTOR = 1;
pub const D2D1_CHANNEL_SELECTOR_R: D2D1_CHANNEL_SELECTOR = 0;
pub type D2D1_COLORMATRIX_ALPHA_MODE = i32;
pub const D2D1_COLORMATRIX_ALPHA_MODE_STRAIGHT: D2D1_COLORMATRIX_ALPHA_MODE = 2;
pub type D2D1_COLORMATRIX_PROP = i32;
pub const D2D1_COLORMATRIX_PROP_ALPHA_MODE: D2D1_COLORMATRIX_PROP = 1;
pub const D2D1_COLORMATRIX_PROP_COLOR_MATRIX: D2D1_COLORMATRIX_PROP = 0;
pub type D2D1_COLOR_INTERPOLATION_MODE = i32;
pub type D2D1_COLOR_SPACE = i32;
pub type D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS = u32;
pub type D2D1_COMPOSITE_MODE = i32;
pub const D2D1_COMPOSITE_MODE_SOURCE_OVER: D2D1_COMPOSITE_MODE = 0;
pub type D2D1_DISPLACEMENTMAP_PROP = i32;
pub const D2D1_DISPLACEMENTMAP_PROP_SCALE: D2D1_DISPLACEMENTMAP_PROP = 0;
pub const D2D1_DISPLACEMENTMAP_PROP_X_CHANNEL_SELECT: D2D1_DISPLACEMENTMAP_PROP = 1;
pub const D2D1_DISPLACEMENTMAP_PROP_Y_CHANNEL_SELECT: D2D1_DISPLACEMENTMAP_PROP = 2;
pub type D2D1_DRAW_TEXT_OPTIONS = u32;
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct D2D1_EFFECT_INPUT_DESCRIPTION {
    pub effect: core::mem::ManuallyDrop<Option<ID2D1Effect>>,
    pub inputIndex: u32,
    pub inputRectangle: D2D_RECT_F,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_ELLIPSE {
    pub point: windows_numerics::Vector2,
    pub radiusX: f32,
    pub radiusY: f32,
}
pub type D2D1_EXTEND_MODE = i32;
pub const D2D1_EXTEND_MODE_CLAMP: D2D1_EXTEND_MODE = 0;
pub type D2D1_FEATURE_LEVEL = i32;
pub type D2D1_GAMMA = i32;
pub type D2D1_GAUSSIANBLUR_PROP = i32;
pub const D2D1_GAUSSIANBLUR_PROP_BORDER_MODE: D2D1_GAUSSIANBLUR_PROP = 2;
pub const D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION: D2D1_GAUSSIANBLUR_PROP = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_GRADIENT_STOP {
    pub position: f32,
    pub color: D2D_COLOR_F,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_IMAGE_BRUSH_PROPERTIES {
    pub sourceRectangle: D2D_RECT_F,
    pub extendModeX: D2D1_EXTEND_MODE,
    pub extendModeY: D2D1_EXTEND_MODE,
    pub interpolationMode: D2D1_INTERPOLATION_MODE,
}
pub type D2D1_INTERPOLATION_MODE = i32;
pub const D2D1_INTERPOLATION_MODE_LINEAR: D2D1_INTERPOLATION_MODE = 1;
pub type D2D1_LAYER_OPTIONS = u32;
pub type D2D1_LAYER_OPTIONS1 = u32;
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct D2D1_LAYER_PARAMETERS {
    pub contentBounds: D2D_RECT_F,
    pub geometricMask: core::mem::ManuallyDrop<Option<ID2D1Geometry>>,
    pub maskAntialiasMode: D2D1_ANTIALIAS_MODE,
    pub maskTransform: windows_numerics::Matrix3x2,
    pub opacity: f32,
    pub opacityBrush: core::mem::ManuallyDrop<Option<ID2D1Brush>>,
    pub layerOptions: D2D1_LAYER_OPTIONS,
}
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct D2D1_LAYER_PARAMETERS1 {
    pub contentBounds: D2D_RECT_F,
    pub geometricMask: core::mem::ManuallyDrop<Option<ID2D1Geometry>>,
    pub maskAntialiasMode: D2D1_ANTIALIAS_MODE,
    pub maskTransform: windows_numerics::Matrix3x2,
    pub opacity: f32,
    pub opacityBrush: core::mem::ManuallyDrop<Option<ID2D1Brush>>,
    pub layerOptions: D2D1_LAYER_OPTIONS1,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
    pub startPoint: windows_numerics::Vector2,
    pub endPoint: windows_numerics::Vector2,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_MAPPED_RECT {
    pub pitch: u32,
    pub bits: *mut u8,
}
pub type D2D1_MAP_OPTIONS = u32;
pub type D2D1_OPACITY_MASK_CONTENT = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_PIXEL_FORMAT {
    pub format: DXGI_FORMAT,
    pub alphaMode: D2D1_ALPHA_MODE,
}
pub type D2D1_PRIMITIVE_BLEND = i32;
pub type D2D1_PROPERTY = i32;
pub const D2D1_PROPERTY_CACHED: D2D1_PROPERTY = -2147483642;
pub type D2D1_PROPERTY_TYPE = i32;
pub const D2D1_PROPERTY_TYPE_BOOL: D2D1_PROPERTY_TYPE = 2;
pub const D2D1_PROPERTY_TYPE_ENUM: D2D1_PROPERTY_TYPE = 11;
pub const D2D1_PROPERTY_TYPE_FLOAT: D2D1_PROPERTY_TYPE = 5;
pub const D2D1_PROPERTY_TYPE_MATRIX_3X2: D2D1_PROPERTY_TYPE = 14;
pub const D2D1_PROPERTY_TYPE_MATRIX_5X4: D2D1_PROPERTY_TYPE = 17;
pub const D2D1_PROPERTY_TYPE_VECTOR4: D2D1_PROPERTY_TYPE = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
    pub center: windows_numerics::Vector2,
    pub gradientOriginOffset: windows_numerics::Vector2,
    pub radiusX: f32,
    pub radiusY: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_RENDERING_CONTROLS {
    pub bufferPrecision: D2D1_BUFFER_PRECISION,
    pub tileSize: D2D_SIZE_U,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_RENDER_TARGET_PROPERTIES {
    pub r#type: D2D1_RENDER_TARGET_TYPE,
    pub pixelFormat: D2D1_PIXEL_FORMAT,
    pub dpiX: f32,
    pub dpiY: f32,
    pub usage: D2D1_RENDER_TARGET_USAGE,
    pub minLevel: D2D1_FEATURE_LEVEL,
}
pub type D2D1_RENDER_TARGET_TYPE = i32;
pub type D2D1_RENDER_TARGET_USAGE = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_ROUNDED_RECT {
    pub rect: D2D_RECT_F,
    pub radiusX: f32,
    pub radiusY: f32,
}
pub type D2D1_SATURATION_PROP = i32;
pub const D2D1_SATURATION_PROP_SATURATION: D2D1_SATURATION_PROP = 0;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct D2D1_TAG(pub u64);
pub type D2D1_TEXT_ANTIALIAS_MODE = i32;
pub type D2D1_UNIT_MODE = i32;
pub type D2D_COLOR_F = D3DCOLORVALUE;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D_RECT_F {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D_SIZE_F {
    pub width: f32,
    pub height: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D_SIZE_U {
    pub width: u32,
    pub height: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3DCOLORVALUE {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_OFFSET {
    pub advanceOffset: f32,
    pub ascenderOffset: f32,
}
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_RUN {
    pub fontFace: core::mem::ManuallyDrop<Option<IDWriteFontFace>>,
    pub fontEmSize: f32,
    pub glyphCount: u32,
    pub glyphIndices: *const u16,
    pub glyphAdvances: *const f32,
    pub glyphOffsets: *const DWRITE_GLYPH_OFFSET,
    pub isSideways: windows_core::BOOL,
    pub bidiLevel: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DWRITE_GLYPH_RUN_DESCRIPTION {
    pub localeName: *const u16,
    pub string: *const u16,
    pub stringLength: u32,
    pub clusterMap: *const u16,
    pub textPosition: u32,
}
pub type DWRITE_MEASURING_MODE = i32;
pub type DXGI_FORMAT = i32;
pub const DXGI_FORMAT_B8G8R8A8_UNORM: DXGI_FORMAT = 87;
pub const DXGI_FORMAT_R32G32B32A32_FLOAT: DXGI_FORMAT = 2;
windows_core::imp::define_interface!(
    ID2D1Bitmap,
    ID2D1Bitmap_Vtbl,
    0xa2296057_ea42_4099_983b_539fb6505426
);
impl core::ops::Deref for ID2D1Bitmap {
    type Target = ID2D1Image;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1Bitmap,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Image
);
#[repr(C)]
pub struct ID2D1Bitmap_Vtbl {
    pub base__: ID2D1Image_Vtbl,
    GetSize: usize,
    GetPixelSize: usize,
    GetPixelFormat: usize,
    GetDpi: usize,
    CopyFromBitmap: usize,
    CopyFromRenderTarget: usize,
    CopyFromMemory: usize,
}
impl windows_core::RuntimeName for ID2D1Bitmap {}
windows_core::imp::define_interface!(
    ID2D1Bitmap1,
    ID2D1Bitmap1_Vtbl,
    0xa898a84c_3873_4588_b08b_ebbf978df041
);
impl core::ops::Deref for ID2D1Bitmap1 {
    type Target = ID2D1Bitmap;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1Bitmap1,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Image,
    ID2D1Bitmap
);
impl ID2D1Bitmap1 {
    pub(crate) unsafe fn GetColorContext(&self) -> windows_core::Result<ID2D1ColorContext> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetColorContext)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn GetOptions(&self) -> D2D1_BITMAP_OPTIONS {
        unsafe {
            (windows_core::Interface::vtable(self).GetOptions)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn GetSurface(&self) -> windows_core::Result<IDXGISurface> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSurface)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn Map(
        &self,
        options: D2D1_MAP_OPTIONS,
    ) -> windows_core::Result<D2D1_MAPPED_RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Map)(
                windows_core::Interface::as_raw(self),
                options,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Unmap(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Unmap)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct ID2D1Bitmap1_Vtbl {
    pub base__: ID2D1Bitmap_Vtbl,
    pub GetColorContext:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub GetOptions: unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_BITMAP_OPTIONS,
    pub GetSurface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Map: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D2D1_MAP_OPTIONS,
        *mut D2D1_MAPPED_RECT,
    ) -> windows_core::HRESULT,
    pub Unmap: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for ID2D1Bitmap1 {}
windows_core::imp::define_interface!(
    ID2D1BitmapBrush,
    ID2D1BitmapBrush_Vtbl,
    0x2cd906aa_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1BitmapBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1BitmapBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1BitmapBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetExtendModeX: usize,
    SetExtendModeY: usize,
    SetInterpolationMode: usize,
    SetBitmap: usize,
    GetExtendModeX: usize,
    GetExtendModeY: usize,
    GetInterpolationMode: usize,
    GetBitmap: usize,
}
impl windows_core::RuntimeName for ID2D1BitmapBrush {}
windows_core::imp::define_interface!(
    ID2D1BitmapBrush1,
    ID2D1BitmapBrush1_Vtbl,
    0x41343a53_e41a_49a2_91cd_21793bbb62e5
);
impl core::ops::Deref for ID2D1BitmapBrush1 {
    type Target = ID2D1BitmapBrush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1BitmapBrush1,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush,
    ID2D1BitmapBrush
);
#[repr(C)]
pub struct ID2D1BitmapBrush1_Vtbl {
    pub base__: ID2D1BitmapBrush_Vtbl,
    SetInterpolationMode1: usize,
    GetInterpolationMode1: usize,
}
impl windows_core::RuntimeName for ID2D1BitmapBrush1 {}
windows_core::imp::define_interface!(
    ID2D1BitmapRenderTarget,
    ID2D1BitmapRenderTarget_Vtbl,
    0x2cd90695_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1BitmapRenderTarget {
    type Target = ID2D1RenderTarget;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1BitmapRenderTarget,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget
);
#[repr(C)]
pub struct ID2D1BitmapRenderTarget_Vtbl {
    pub base__: ID2D1RenderTarget_Vtbl,
    GetBitmap: usize,
}
impl windows_core::RuntimeName for ID2D1BitmapRenderTarget {}
windows_core::imp::define_interface!(
    ID2D1Brush,
    ID2D1Brush_Vtbl,
    0x2cd906a8_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Brush {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Brush, windows_core::IUnknown, ID2D1Resource);
impl ID2D1Brush {
    pub(crate) unsafe fn SetOpacity(&self, opacity: f32) {
        unsafe {
            (windows_core::Interface::vtable(self).SetOpacity)(
                windows_core::Interface::as_raw(self),
                opacity,
            );
        }
    }
    pub(crate) unsafe fn SetTransform(&self, transform: *const windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransform)(
                windows_core::Interface::as_raw(self),
                transform,
            );
        }
    }
    pub(crate) unsafe fn GetOpacity(&self) -> f32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetOpacity)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn GetTransform(&self, transform: *mut windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).GetTransform)(
                windows_core::Interface::as_raw(self),
                transform as _,
            );
        }
    }
}
#[repr(C)]
pub struct ID2D1Brush_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    pub SetOpacity: unsafe extern "system" fn(*mut core::ffi::c_void, f32),
    pub SetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const windows_numerics::Matrix3x2),
    pub GetOpacity: unsafe extern "system" fn(*mut core::ffi::c_void) -> f32,
    pub GetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_numerics::Matrix3x2),
}
impl windows_core::RuntimeName for ID2D1Brush {}
windows_core::imp::define_interface!(
    ID2D1ColorContext,
    ID2D1ColorContext_Vtbl,
    0x1c4820bb_5771_4518_a581_2fe4dd0ec657
);
impl core::ops::Deref for ID2D1ColorContext {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1ColorContext, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1ColorContext_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetColorSpace: usize,
    GetProfileSize: usize,
    GetProfile: usize,
}
impl windows_core::RuntimeName for ID2D1ColorContext {}
windows_core::imp::define_interface!(
    ID2D1CommandList,
    ID2D1CommandList_Vtbl,
    0xb4f34a19_2383_4d76_94f6_ec343657c3dc
);
impl core::ops::Deref for ID2D1CommandList {
    type Target = ID2D1Image;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1CommandList,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Image
);
#[repr(C)]
pub struct ID2D1CommandList_Vtbl {
    pub base__: ID2D1Image_Vtbl,
    Stream: usize,
    Close: usize,
}
impl windows_core::RuntimeName for ID2D1CommandList {}
windows_core::imp::define_interface!(
    ID2D1Device,
    ID2D1Device_Vtbl,
    0x47dd575d_ac05_4cdd_8049_9b02cd16f44c
);
impl core::ops::Deref for ID2D1Device {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Device, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Device_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    CreateDeviceContext: usize,
    CreatePrintControl: usize,
    SetMaximumTextureMemory: usize,
    GetMaximumTextureMemory: usize,
    ClearResources: usize,
}
impl windows_core::RuntimeName for ID2D1Device {}
windows_core::imp::define_interface!(
    ID2D1DeviceContext,
    ID2D1DeviceContext_Vtbl,
    0xe8f7fe7a_191c_466d_ad95_975678bda998
);
impl core::ops::Deref for ID2D1DeviceContext {
    type Target = ID2D1RenderTarget;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DeviceContext,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget
);
impl ID2D1DeviceContext {
    pub(crate) unsafe fn CreateBitmap(
        &self,
        size: D2D_SIZE_U,
        sourcedata: Option<*const core::ffi::c_void>,
        pitch: u32,
        bitmapproperties: *const D2D1_BITMAP_PROPERTIES1,
    ) -> windows_core::Result<ID2D1Bitmap1> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmap)(
                windows_core::Interface::as_raw(self),
                size,
                sourcedata.unwrap_or(core::mem::zeroed()) as _,
                pitch,
                bitmapproperties,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromWicBitmap<P0>(
        &self,
        wicbitmapsource: P0,
        bitmapproperties: Option<*const D2D1_BITMAP_PROPERTIES1>,
    ) -> windows_core::Result<ID2D1Bitmap1>
    where
        P0: windows_core::Param<IWICBitmapSource>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromWicBitmap)(
                windows_core::Interface::as_raw(self),
                wicbitmapsource.param().abi(),
                bitmapproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateColorContext(
        &self,
        space: D2D1_COLOR_SPACE,
        profile: Option<&[u8]>,
    ) -> windows_core::Result<ID2D1ColorContext> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorContext)(
                windows_core::Interface::as_raw(self),
                space,
                profile.map_or(core::ptr::null(), |slice| slice.as_ptr()),
                profile.map_or(0, |slice| slice.len().try_into().unwrap()),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateColorContextFromFilename<P0>(
        &self,
        filename: P0,
    ) -> windows_core::Result<ID2D1ColorContext>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorContextFromFilename)(
                windows_core::Interface::as_raw(self),
                filename.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateColorContextFromWicColorContext<P0>(
        &self,
        wiccolorcontext: P0,
    ) -> windows_core::Result<ID2D1ColorContext>
    where
        P0: windows_core::Param<IWICColorContext>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorContextFromWicColorContext)(
                windows_core::Interface::as_raw(self),
                wiccolorcontext.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromDxgiSurface<P0>(
        &self,
        surface: P0,
        bitmapproperties: Option<*const D2D1_BITMAP_PROPERTIES1>,
    ) -> windows_core::Result<ID2D1Bitmap1>
    where
        P0: windows_core::Param<IDXGISurface>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromDxgiSurface)(
                windows_core::Interface::as_raw(self),
                surface.param().abi(),
                bitmapproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateEffect(
        &self,
        effectid: *const windows_core::GUID,
    ) -> windows_core::Result<ID2D1Effect> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateEffect)(
                windows_core::Interface::as_raw(self),
                effectid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateGradientStopCollection(
        &self,
        straightalphagradientstops: &[D2D1_GRADIENT_STOP],
        preinterpolationspace: D2D1_COLOR_SPACE,
        postinterpolationspace: D2D1_COLOR_SPACE,
        bufferprecision: D2D1_BUFFER_PRECISION,
        extendmode: D2D1_EXTEND_MODE,
        colorinterpolationmode: D2D1_COLOR_INTERPOLATION_MODE,
    ) -> windows_core::Result<ID2D1GradientStopCollection1> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateGradientStopCollection)(
                windows_core::Interface::as_raw(self),
                straightalphagradientstops.as_ptr(),
                straightalphagradientstops.len().try_into().unwrap(),
                preinterpolationspace,
                postinterpolationspace,
                bufferprecision,
                extendmode,
                colorinterpolationmode,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateImageBrush<P0>(
        &self,
        image: P0,
        imagebrushproperties: *const D2D1_IMAGE_BRUSH_PROPERTIES,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
    ) -> windows_core::Result<ID2D1ImageBrush>
    where
        P0: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateImageBrush)(
                windows_core::Interface::as_raw(self),
                image.param().abi(),
                imagebrushproperties,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapBrush<P0>(
        &self,
        bitmap: P0,
        bitmapbrushproperties: Option<*const D2D1_BITMAP_BRUSH_PROPERTIES1>,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
    ) -> windows_core::Result<ID2D1BitmapBrush1>
    where
        P0: windows_core::Param<ID2D1Bitmap>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapBrush)(
                windows_core::Interface::as_raw(self),
                bitmap.param().abi(),
                bitmapbrushproperties.unwrap_or(core::mem::zeroed()) as _,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateCommandList(&self) -> windows_core::Result<ID2D1CommandList> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCommandList)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn IsDxgiFormatSupported(&self, format: DXGI_FORMAT) -> windows_core::BOOL {
        unsafe {
            (windows_core::Interface::vtable(self).IsDxgiFormatSupported)(
                windows_core::Interface::as_raw(self),
                format,
            )
        }
    }
    pub(crate) unsafe fn IsBufferPrecisionSupported(
        &self,
        bufferprecision: D2D1_BUFFER_PRECISION,
    ) -> windows_core::BOOL {
        unsafe {
            (windows_core::Interface::vtable(self).IsBufferPrecisionSupported)(
                windows_core::Interface::as_raw(self),
                bufferprecision,
            )
        }
    }
    pub(crate) unsafe fn GetImageLocalBounds<P0>(
        &self,
        image: P0,
    ) -> windows_core::Result<D2D_RECT_F>
    where
        P0: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetImageLocalBounds)(
                windows_core::Interface::as_raw(self),
                image.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetImageWorldBounds<P0>(
        &self,
        image: P0,
    ) -> windows_core::Result<D2D_RECT_F>
    where
        P0: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetImageWorldBounds)(
                windows_core::Interface::as_raw(self),
                image.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetGlyphRunWorldBounds(
        &self,
        baselineorigin: windows_numerics::Vector2,
        glyphrun: *const DWRITE_GLYPH_RUN,
        measuringmode: DWRITE_MEASURING_MODE,
    ) -> windows_core::Result<D2D_RECT_F> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetGlyphRunWorldBounds)(
                windows_core::Interface::as_raw(self),
                baselineorigin,
                glyphrun,
                measuringmode,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetDevice(&self) -> windows_core::Result<ID2D1Device> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDevice)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn SetTarget<P0>(&self, image: P0)
    where
        P0: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTarget)(
                windows_core::Interface::as_raw(self),
                image.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn GetTarget(&self) -> windows_core::Result<ID2D1Image> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTarget)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn SetRenderingControls(
        &self,
        renderingcontrols: *const D2D1_RENDERING_CONTROLS,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SetRenderingControls)(
                windows_core::Interface::as_raw(self),
                renderingcontrols,
            );
        }
    }
    pub(crate) unsafe fn GetRenderingControls(&self) -> D2D1_RENDERING_CONTROLS {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRenderingControls)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn SetPrimitiveBlend(&self, primitiveblend: D2D1_PRIMITIVE_BLEND) {
        unsafe {
            (windows_core::Interface::vtable(self).SetPrimitiveBlend)(
                windows_core::Interface::as_raw(self),
                primitiveblend,
            );
        }
    }
    pub(crate) unsafe fn GetPrimitiveBlend(&self) -> D2D1_PRIMITIVE_BLEND {
        unsafe {
            (windows_core::Interface::vtable(self).GetPrimitiveBlend)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn SetUnitMode(&self, unitmode: D2D1_UNIT_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetUnitMode)(
                windows_core::Interface::as_raw(self),
                unitmode,
            );
        }
    }
    pub(crate) unsafe fn GetUnitMode(&self) -> D2D1_UNIT_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetUnitMode)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn DrawGlyphRun<P3>(
        &self,
        baselineorigin: windows_numerics::Vector2,
        glyphrun: *const DWRITE_GLYPH_RUN,
        glyphrundescription: Option<*const DWRITE_GLYPH_RUN_DESCRIPTION>,
        foregroundbrush: P3,
        measuringmode: DWRITE_MEASURING_MODE,
    ) where
        P3: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGlyphRun)(
                windows_core::Interface::as_raw(self),
                baselineorigin,
                glyphrun,
                glyphrundescription.unwrap_or(core::mem::zeroed()) as _,
                foregroundbrush.param().abi(),
                measuringmode,
            );
        }
    }
    pub(crate) unsafe fn DrawImage<P0>(
        &self,
        image: P0,
        targetoffset: Option<*const windows_numerics::Vector2>,
        imagerectangle: Option<*const D2D_RECT_F>,
        interpolationmode: D2D1_INTERPOLATION_MODE,
        compositemode: D2D1_COMPOSITE_MODE,
    ) where
        P0: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawImage)(
                windows_core::Interface::as_raw(self),
                image.param().abi(),
                targetoffset.unwrap_or(core::mem::zeroed()) as _,
                imagerectangle.unwrap_or(core::mem::zeroed()) as _,
                interpolationmode,
                compositemode,
            );
        }
    }
    pub(crate) unsafe fn DrawGdiMetafile<P0>(
        &self,
        gdimetafile: P0,
        targetoffset: Option<*const windows_numerics::Vector2>,
    ) where
        P0: windows_core::Param<ID2D1GdiMetafile>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGdiMetafile)(
                windows_core::Interface::as_raw(self),
                gdimetafile.param().abi(),
                targetoffset.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn DrawBitmap<P0>(
        &self,
        bitmap: P0,
        destinationrectangle: Option<*const D2D_RECT_F>,
        opacity: f32,
        interpolationmode: D2D1_INTERPOLATION_MODE,
        sourcerectangle: Option<*const D2D_RECT_F>,
        perspectivetransform: Option<*const windows_numerics::Matrix4x4>,
    ) where
        P0: windows_core::Param<ID2D1Bitmap>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawBitmap)(
                windows_core::Interface::as_raw(self),
                bitmap.param().abi(),
                destinationrectangle.unwrap_or(core::mem::zeroed()) as _,
                opacity,
                interpolationmode,
                sourcerectangle.unwrap_or(core::mem::zeroed()) as _,
                perspectivetransform.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn PushLayer<P1>(
        &self,
        layerparameters: *const D2D1_LAYER_PARAMETERS1,
        layer: P1,
    ) where
        P1: windows_core::Param<ID2D1Layer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PushLayer)(
                windows_core::Interface::as_raw(self),
                layerparameters,
                layer.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn InvalidateEffectInputRectangle<P0>(
        &self,
        effect: P0,
        input: u32,
        inputrectangle: *const D2D_RECT_F,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID2D1Effect>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).InvalidateEffectInputRectangle)(
                windows_core::Interface::as_raw(self),
                effect.param().abi(),
                input,
                inputrectangle,
            )
        }
    }
    pub(crate) unsafe fn GetEffectInvalidRectangleCount<P0>(
        &self,
        effect: P0,
    ) -> windows_core::Result<u32>
    where
        P0: windows_core::Param<ID2D1Effect>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetEffectInvalidRectangleCount)(
                windows_core::Interface::as_raw(self),
                effect.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetEffectInvalidRectangles<P0>(
        &self,
        effect: P0,
        rectangles: *mut D2D_RECT_F,
        rectanglescount: u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID2D1Effect>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).GetEffectInvalidRectangles)(
                windows_core::Interface::as_raw(self),
                effect.param().abi(),
                rectangles as _,
                rectanglescount,
            )
        }
    }
    pub(crate) unsafe fn GetEffectRequiredInputRectangles<P0>(
        &self,
        rendereffect: P0,
        renderimagerectangle: Option<*const D2D_RECT_F>,
        inputdescriptions: *const D2D1_EFFECT_INPUT_DESCRIPTION,
        requiredinputrects: *mut D2D_RECT_F,
        inputcount: u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID2D1Effect>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).GetEffectRequiredInputRectangles)(
                windows_core::Interface::as_raw(self),
                rendereffect.param().abi(),
                renderimagerectangle.unwrap_or(core::mem::zeroed()) as _,
                inputdescriptions,
                requiredinputrects as _,
                inputcount,
            )
        }
    }
    pub(crate) unsafe fn FillOpacityMask<P0, P1>(
        &self,
        opacitymask: P0,
        brush: P1,
        destinationrectangle: Option<*const D2D_RECT_F>,
        sourcerectangle: Option<*const D2D_RECT_F>,
    ) where
        P0: windows_core::Param<ID2D1Bitmap>,
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillOpacityMask)(
                windows_core::Interface::as_raw(self),
                opacitymask.param().abi(),
                brush.param().abi(),
                destinationrectangle.unwrap_or(core::mem::zeroed()) as _,
                sourcerectangle.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
}
#[repr(C)]
pub struct ID2D1DeviceContext_Vtbl {
    pub base__: ID2D1RenderTarget_Vtbl,
    pub CreateBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D2D_SIZE_U,
        *const core::ffi::c_void,
        u32,
        *const D2D1_BITMAP_PROPERTIES1,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromWicBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_PROPERTIES1,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorContext: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D2D1_COLOR_SPACE,
        *const u8,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorContextFromFilename: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorContextFromWicColorContext: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
    pub CreateBitmapFromDxgiSurface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_PROPERTIES1,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateEffect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateGradientStopCollection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_GRADIENT_STOP,
        u32,
        D2D1_COLOR_SPACE,
        D2D1_COLOR_SPACE,
        D2D1_BUFFER_PRECISION,
        D2D1_EXTEND_MODE,
        D2D1_COLOR_INTERPOLATION_MODE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateImageBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_IMAGE_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_BRUSH_PROPERTIES1,
        *const D2D1_BRUSH_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCommandList: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub IsDxgiFormatSupported:
        unsafe extern "system" fn(*mut core::ffi::c_void, DXGI_FORMAT) -> windows_core::BOOL,
    pub IsBufferPrecisionSupported: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D2D1_BUFFER_PRECISION,
    ) -> windows_core::BOOL,
    pub GetImageLocalBounds: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut D2D_RECT_F,
    ) -> windows_core::HRESULT,
    pub GetImageWorldBounds: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut D2D_RECT_F,
    ) -> windows_core::HRESULT,
    pub GetGlyphRunWorldBounds: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *const DWRITE_GLYPH_RUN,
        DWRITE_MEASURING_MODE,
        *mut D2D_RECT_F,
    ) -> windows_core::HRESULT,
    pub GetDevice: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub SetTarget: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub GetTarget: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub SetRenderingControls:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D1_RENDERING_CONTROLS),
    pub GetRenderingControls:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D1_RENDERING_CONTROLS),
    pub SetPrimitiveBlend: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_PRIMITIVE_BLEND),
    pub GetPrimitiveBlend:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_PRIMITIVE_BLEND,
    pub SetUnitMode: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_UNIT_MODE),
    pub GetUnitMode: unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_UNIT_MODE,
    pub DrawGlyphRun: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *const DWRITE_GLYPH_RUN,
        *const DWRITE_GLYPH_RUN_DESCRIPTION,
        *mut core::ffi::c_void,
        DWRITE_MEASURING_MODE,
    ),
    pub DrawImage: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_numerics::Vector2,
        *const D2D_RECT_F,
        D2D1_INTERPOLATION_MODE,
        D2D1_COMPOSITE_MODE,
    ),
    pub DrawGdiMetafile: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_numerics::Vector2,
    ),
    pub DrawBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        f32,
        D2D1_INTERPOLATION_MODE,
        *const D2D_RECT_F,
        *const windows_numerics::Matrix4x4,
    ),
    pub PushLayer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_LAYER_PARAMETERS1,
        *mut core::ffi::c_void,
    ),
    pub InvalidateEffectInputRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        *const D2D_RECT_F,
    ) -> windows_core::HRESULT,
    pub GetEffectInvalidRectangleCount: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub GetEffectInvalidRectangles: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut D2D_RECT_F,
        u32,
    ) -> windows_core::HRESULT,
    pub GetEffectRequiredInputRectangles: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *const D2D1_EFFECT_INPUT_DESCRIPTION,
        *mut D2D_RECT_F,
        u32,
    ) -> windows_core::HRESULT,
    pub FillOpacityMask: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *const D2D_RECT_F,
    ),
}
impl windows_core::RuntimeName for ID2D1DeviceContext {}
windows_core::imp::define_interface!(
    ID2D1DrawingStateBlock,
    ID2D1DrawingStateBlock_Vtbl,
    0x28506e39_ebf6_46a1_bb47_fd85565ab957
);
impl core::ops::Deref for ID2D1DrawingStateBlock {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DrawingStateBlock,
    windows_core::IUnknown,
    ID2D1Resource
);
#[repr(C)]
pub struct ID2D1DrawingStateBlock_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetDescription: usize,
    SetDescription: usize,
    SetTextRenderingParams: usize,
    GetTextRenderingParams: usize,
}
impl windows_core::RuntimeName for ID2D1DrawingStateBlock {}
windows_core::imp::define_interface!(
    ID2D1Effect,
    ID2D1Effect_Vtbl,
    0x28211a43_7d89_476f_8181_2d6159b220ad
);
impl core::ops::Deref for ID2D1Effect {
    type Target = ID2D1Properties;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Effect, windows_core::IUnknown, ID2D1Properties);
impl ID2D1Effect {
    pub(crate) unsafe fn SetInput<P1>(&self, index: u32, input: P1, invalidate: bool)
    where
        P1: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetInput)(
                windows_core::Interface::as_raw(self),
                index,
                input.param().abi(),
                invalidate.into(),
            );
        }
    }
    pub(crate) unsafe fn SetInputCount(&self, inputcount: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetInputCount)(
                windows_core::Interface::as_raw(self),
                inputcount,
            )
        }
    }
    pub(crate) unsafe fn GetInput(&self, index: u32) -> windows_core::Result<ID2D1Image> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetInput)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn GetInputCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetInputCount)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn GetOutput(&self) -> windows_core::Result<ID2D1Image> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetOutput)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
}
#[repr(C)]
pub struct ID2D1Effect_Vtbl {
    pub base__: ID2D1Properties_Vtbl,
    pub SetInput: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ),
    pub SetInputCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub GetInput:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut *mut core::ffi::c_void),
    pub GetInputCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetOutput: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
}
pub trait ID2D1Effect_Impl: ID2D1Properties_Impl {
    fn SetInput(
        &self,
        index: u32,
        input: windows_core::Ref<ID2D1Image>,
        invalidate: windows_core::BOOL,
    );
    fn SetInputCount(&self, inputcount: u32) -> windows_core::Result<()>;
    fn GetInput(&self, index: u32, input: windows_core::OutRef<ID2D1Image>);
    fn GetInputCount(&self) -> u32;
    fn GetOutput(&self, outputimage: windows_core::OutRef<ID2D1Image>);
}
impl ID2D1Effect_Vtbl {
    pub const fn new<Identity: ID2D1Effect_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetInput<Identity: ID2D1Effect_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            index: u32,
            input: *mut core::ffi::c_void,
            invalidate: windows_core::BOOL,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Effect_Impl::SetInput(
                    this,
                    core::mem::transmute_copy(&index),
                    core::mem::transmute_copy(&input),
                    core::mem::transmute_copy(&invalidate),
                );
            }
        }
        unsafe extern "system" fn SetInputCount<Identity: ID2D1Effect_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            inputcount: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Effect_Impl::SetInputCount(this, core::mem::transmute_copy(&inputcount)).into()
            }
        }
        unsafe extern "system" fn GetInput<Identity: ID2D1Effect_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            index: u32,
            input: *mut *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Effect_Impl::GetInput(
                    this,
                    core::mem::transmute_copy(&index),
                    core::mem::transmute_copy(&input),
                );
            }
        }
        unsafe extern "system" fn GetInputCount<Identity: ID2D1Effect_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Effect_Impl::GetInputCount(this)
            }
        }
        unsafe extern "system" fn GetOutput<Identity: ID2D1Effect_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            outputimage: *mut *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Effect_Impl::GetOutput(this, core::mem::transmute_copy(&outputimage));
            }
        }
        Self {
            base__: ID2D1Properties_Vtbl::new::<Identity, OFFSET>(),
            SetInput: SetInput::<Identity, OFFSET>,
            SetInputCount: SetInputCount::<Identity, OFFSET>,
            GetInput: GetInput::<Identity, OFFSET>,
            GetInputCount: GetInputCount::<Identity, OFFSET>,
            GetOutput: GetOutput::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ID2D1Effect as windows_core::Interface>::IID
            || iid == &<ID2D1Properties as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for ID2D1Effect {}
windows_core::imp::define_interface!(
    ID2D1GdiMetafile,
    ID2D1GdiMetafile_Vtbl,
    0x2f543dc3_cfc1_4211_864f_cfd91c6f3395
);
impl core::ops::Deref for ID2D1GdiMetafile {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1GdiMetafile, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1GdiMetafile_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    Stream: usize,
    GetBounds: usize,
}
impl windows_core::RuntimeName for ID2D1GdiMetafile {}
windows_core::imp::define_interface!(
    ID2D1Geometry,
    ID2D1Geometry_Vtbl,
    0x2cd906a1_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Geometry {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Geometry, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Geometry_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetBounds: usize,
    GetWidenedBounds: usize,
    StrokeContainsPoint: usize,
    FillContainsPoint: usize,
    CompareWithGeometry: usize,
    Simplify: usize,
    Tessellate: usize,
    CombineWithGeometry: usize,
    Outline: usize,
    ComputeArea: usize,
    ComputeLength: usize,
    ComputePointAtLength: usize,
    Widen: usize,
}
impl windows_core::RuntimeName for ID2D1Geometry {}
windows_core::imp::define_interface!(
    ID2D1GradientStopCollection,
    ID2D1GradientStopCollection_Vtbl,
    0x2cd906a7_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1GradientStopCollection {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1GradientStopCollection,
    windows_core::IUnknown,
    ID2D1Resource
);
#[repr(C)]
pub struct ID2D1GradientStopCollection_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetGradientStopCount: usize,
    GetGradientStops: usize,
    GetColorInterpolationGamma: usize,
    GetExtendMode: usize,
}
impl windows_core::RuntimeName for ID2D1GradientStopCollection {}
windows_core::imp::define_interface!(
    ID2D1GradientStopCollection1,
    ID2D1GradientStopCollection1_Vtbl,
    0xae1572f4_5dd0_4777_998b_9279472ae63b
);
impl core::ops::Deref for ID2D1GradientStopCollection1 {
    type Target = ID2D1GradientStopCollection;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1GradientStopCollection1,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1GradientStopCollection
);
#[repr(C)]
pub struct ID2D1GradientStopCollection1_Vtbl {
    pub base__: ID2D1GradientStopCollection_Vtbl,
    GetGradientStops1: usize,
    GetPreInterpolationSpace: usize,
    GetPostInterpolationSpace: usize,
    GetBufferPrecision: usize,
    GetColorInterpolationMode: usize,
}
impl windows_core::RuntimeName for ID2D1GradientStopCollection1 {}
windows_core::imp::define_interface!(
    ID2D1Image,
    ID2D1Image_Vtbl,
    0x65019f75_8da2_497c_b32c_dfa34e48ede6
);
impl core::ops::Deref for ID2D1Image {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Image, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Image_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
}
impl windows_core::RuntimeName for ID2D1Image {}
windows_core::imp::define_interface!(
    ID2D1ImageBrush,
    ID2D1ImageBrush_Vtbl,
    0xfe9e984d_3f95_407c_b5db_cb94d4e8f87c
);
impl core::ops::Deref for ID2D1ImageBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1ImageBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
impl ID2D1ImageBrush {
    pub(crate) unsafe fn SetImage<P0>(&self, image: P0)
    where
        P0: windows_core::Param<ID2D1Image>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetImage)(
                windows_core::Interface::as_raw(self),
                image.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn SetExtendModeX(&self, extendmodex: D2D1_EXTEND_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetExtendModeX)(
                windows_core::Interface::as_raw(self),
                extendmodex,
            );
        }
    }
    pub(crate) unsafe fn SetExtendModeY(&self, extendmodey: D2D1_EXTEND_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetExtendModeY)(
                windows_core::Interface::as_raw(self),
                extendmodey,
            );
        }
    }
    pub(crate) unsafe fn SetInterpolationMode(&self, interpolationmode: D2D1_INTERPOLATION_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetInterpolationMode)(
                windows_core::Interface::as_raw(self),
                interpolationmode,
            );
        }
    }
    pub(crate) unsafe fn SetSourceRectangle(&self, sourcerectangle: *const D2D_RECT_F) {
        unsafe {
            (windows_core::Interface::vtable(self).SetSourceRectangle)(
                windows_core::Interface::as_raw(self),
                sourcerectangle,
            );
        }
    }
    pub(crate) unsafe fn GetImage(&self) -> windows_core::Result<ID2D1Image> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetImage)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn GetExtendModeX(&self) -> D2D1_EXTEND_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetExtendModeX)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn GetExtendModeY(&self) -> D2D1_EXTEND_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetExtendModeY)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn GetInterpolationMode(&self) -> D2D1_INTERPOLATION_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetInterpolationMode)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn GetSourceRectangle(&self) -> D2D_RECT_F {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSourceRectangle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
}
#[repr(C)]
pub struct ID2D1ImageBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    pub SetImage: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub SetExtendModeX: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_EXTEND_MODE),
    pub SetExtendModeY: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_EXTEND_MODE),
    pub SetInterpolationMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_INTERPOLATION_MODE),
    pub SetSourceRectangle: unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D_RECT_F),
    pub GetImage: unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub GetExtendModeX: unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_EXTEND_MODE,
    pub GetExtendModeY: unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_EXTEND_MODE,
    pub GetInterpolationMode:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_INTERPOLATION_MODE,
    pub GetSourceRectangle: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D_RECT_F),
}
impl windows_core::RuntimeName for ID2D1ImageBrush {}
windows_core::imp::define_interface!(
    ID2D1Layer,
    ID2D1Layer_Vtbl,
    0x2cd9069b_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Layer {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Layer, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Layer_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetSize: usize,
}
impl windows_core::RuntimeName for ID2D1Layer {}
windows_core::imp::define_interface!(
    ID2D1LinearGradientBrush,
    ID2D1LinearGradientBrush_Vtbl,
    0x2cd906ab_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1LinearGradientBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1LinearGradientBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1LinearGradientBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetStartPoint: usize,
    SetEndPoint: usize,
    GetStartPoint: usize,
    GetEndPoint: usize,
    GetGradientStopCollection: usize,
}
impl windows_core::RuntimeName for ID2D1LinearGradientBrush {}
windows_core::imp::define_interface!(
    ID2D1Mesh,
    ID2D1Mesh_Vtbl,
    0x2cd906c2_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Mesh {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Mesh, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Mesh_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    Open: usize,
}
impl windows_core::RuntimeName for ID2D1Mesh {}
windows_core::imp::define_interface!(
    ID2D1Properties,
    ID2D1Properties_Vtbl,
    0x483473d7_cd46_4f9d_9d3a_3112aa80159d
);
windows_core::imp::interface_hierarchy!(ID2D1Properties, windows_core::IUnknown);
impl ID2D1Properties {
    pub(crate) unsafe fn GetPropertyCount(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetPropertyCount)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn GetPropertyName(
        &self,
        index: u32,
        name: windows_core::PWSTR,
        namecount: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetPropertyName)(
                windows_core::Interface::as_raw(self),
                index,
                name,
                namecount,
            )
        }
    }
    pub(crate) unsafe fn GetPropertyNameLength(&self, index: u32) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetPropertyNameLength)(
                windows_core::Interface::as_raw(self),
                index,
            )
        }
    }
    pub(crate) unsafe fn GetType(&self, index: u32) -> D2D1_PROPERTY_TYPE {
        unsafe {
            (windows_core::Interface::vtable(self).GetType)(
                windows_core::Interface::as_raw(self),
                index,
            )
        }
    }
    pub(crate) unsafe fn GetPropertyIndex<P0>(&self, name: P0) -> u32
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).GetPropertyIndex)(
                windows_core::Interface::as_raw(self),
                name.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetValueByName<P0>(
        &self,
        name: P0,
        r#type: D2D1_PROPERTY_TYPE,
        data: &[u8],
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetValueByName)(
                windows_core::Interface::as_raw(self),
                name.param().abi(),
                r#type,
                data.as_ptr(),
                data.len().try_into().unwrap(),
            )
        }
    }
    pub(crate) unsafe fn SetValue(
        &self,
        index: u32,
        r#type: D2D1_PROPERTY_TYPE,
        data: &[u8],
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetValue)(
                windows_core::Interface::as_raw(self),
                index,
                r#type,
                data.as_ptr(),
                data.len().try_into().unwrap(),
            )
        }
    }
    pub(crate) unsafe fn GetValueByName<P0>(
        &self,
        name: P0,
        r#type: D2D1_PROPERTY_TYPE,
        data: *mut u8,
        datasize: u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).GetValueByName)(
                windows_core::Interface::as_raw(self),
                name.param().abi(),
                r#type,
                data as _,
                datasize,
            )
        }
    }
    pub(crate) unsafe fn GetValue(
        &self,
        index: u32,
        r#type: D2D1_PROPERTY_TYPE,
        data: *mut u8,
        datasize: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetValue)(
                windows_core::Interface::as_raw(self),
                index,
                r#type,
                data as _,
                datasize,
            )
        }
    }
    pub(crate) unsafe fn GetValueSize(&self, index: u32) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetValueSize)(
                windows_core::Interface::as_raw(self),
                index,
            )
        }
    }
    pub(crate) unsafe fn GetSubProperties(&self, index: u32) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSubProperties)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct ID2D1Properties_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetPropertyCount: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub GetPropertyName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        windows_core::PWSTR,
        u32,
    ) -> windows_core::HRESULT,
    pub GetPropertyNameLength: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> u32,
    pub GetType: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> D2D1_PROPERTY_TYPE,
    pub GetPropertyIndex:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_core::PCWSTR) -> u32,
    pub SetValueByName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        D2D1_PROPERTY_TYPE,
        *const u8,
        u32,
    ) -> windows_core::HRESULT,
    pub SetValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        D2D1_PROPERTY_TYPE,
        *const u8,
        u32,
    ) -> windows_core::HRESULT,
    pub GetValueByName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        D2D1_PROPERTY_TYPE,
        *mut u8,
        u32,
    ) -> windows_core::HRESULT,
    pub GetValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        D2D1_PROPERTY_TYPE,
        *mut u8,
        u32,
    ) -> windows_core::HRESULT,
    pub GetValueSize: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> u32,
    pub GetSubProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait ID2D1Properties_Impl: windows_core::IUnknownImpl {
    fn GetPropertyCount(&self) -> u32;
    fn GetPropertyName(
        &self,
        index: u32,
        name: windows_core::PWSTR,
        namecount: u32,
    ) -> windows_core::Result<()>;
    fn GetPropertyNameLength(&self, index: u32) -> u32;
    fn GetType(&self, index: u32) -> D2D1_PROPERTY_TYPE;
    fn GetPropertyIndex(&self, name: &windows_core::PCWSTR) -> u32;
    fn SetValueByName(
        &self,
        name: &windows_core::PCWSTR,
        r#type: D2D1_PROPERTY_TYPE,
        data: *const u8,
        datasize: u32,
    ) -> windows_core::Result<()>;
    fn SetValue(
        &self,
        index: u32,
        r#type: D2D1_PROPERTY_TYPE,
        data: *const u8,
        datasize: u32,
    ) -> windows_core::Result<()>;
    fn GetValueByName(
        &self,
        name: &windows_core::PCWSTR,
        r#type: D2D1_PROPERTY_TYPE,
        data: *mut u8,
        datasize: u32,
    ) -> windows_core::Result<()>;
    fn GetValue(
        &self,
        index: u32,
        r#type: D2D1_PROPERTY_TYPE,
        data: *mut u8,
        datasize: u32,
    ) -> windows_core::Result<()>;
    fn GetValueSize(&self, index: u32) -> u32;
    fn GetSubProperties(&self, index: u32) -> windows_core::Result<ID2D1Properties>;
}
impl ID2D1Properties_Vtbl {
    pub const fn new<Identity: ID2D1Properties_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetPropertyCount<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetPropertyCount(this)
            }
        }
        unsafe extern "system" fn GetPropertyName<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: u32,
            name: windows_core::PWSTR,
            namecount: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetPropertyName(
                    this,
                    core::mem::transmute_copy(&index),
                    core::mem::transmute_copy(&name),
                    core::mem::transmute_copy(&namecount),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetPropertyNameLength<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: u32,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetPropertyNameLength(this, core::mem::transmute_copy(&index))
            }
        }
        unsafe extern "system" fn GetType<Identity: ID2D1Properties_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            index: u32,
        ) -> D2D1_PROPERTY_TYPE {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetType(this, core::mem::transmute_copy(&index))
            }
        }
        unsafe extern "system" fn GetPropertyIndex<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            name: windows_core::PCWSTR,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetPropertyIndex(this, core::mem::transmute(&name))
            }
        }
        unsafe extern "system" fn SetValueByName<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            name: windows_core::PCWSTR,
            r#type: D2D1_PROPERTY_TYPE,
            data: *const u8,
            datasize: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::SetValueByName(
                    this,
                    core::mem::transmute(&name),
                    core::mem::transmute_copy(&r#type),
                    core::mem::transmute_copy(&data),
                    core::mem::transmute_copy(&datasize),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetValue<Identity: ID2D1Properties_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            index: u32,
            r#type: D2D1_PROPERTY_TYPE,
            data: *const u8,
            datasize: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::SetValue(
                    this,
                    core::mem::transmute_copy(&index),
                    core::mem::transmute_copy(&r#type),
                    core::mem::transmute_copy(&data),
                    core::mem::transmute_copy(&datasize),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetValueByName<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            name: windows_core::PCWSTR,
            r#type: D2D1_PROPERTY_TYPE,
            data: *mut u8,
            datasize: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetValueByName(
                    this,
                    core::mem::transmute(&name),
                    core::mem::transmute_copy(&r#type),
                    core::mem::transmute_copy(&data),
                    core::mem::transmute_copy(&datasize),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetValue<Identity: ID2D1Properties_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            index: u32,
            r#type: D2D1_PROPERTY_TYPE,
            data: *mut u8,
            datasize: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetValue(
                    this,
                    core::mem::transmute_copy(&index),
                    core::mem::transmute_copy(&r#type),
                    core::mem::transmute_copy(&data),
                    core::mem::transmute_copy(&datasize),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetValueSize<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: u32,
        ) -> u32 {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID2D1Properties_Impl::GetValueSize(this, core::mem::transmute_copy(&index))
            }
        }
        unsafe extern "system" fn GetSubProperties<
            Identity: ID2D1Properties_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: u32,
            subproperties: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match ID2D1Properties_Impl::GetSubProperties(
                    this,
                    core::mem::transmute_copy(&index),
                ) {
                    Ok(ok__) => {
                        subproperties.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetPropertyCount: GetPropertyCount::<Identity, OFFSET>,
            GetPropertyName: GetPropertyName::<Identity, OFFSET>,
            GetPropertyNameLength: GetPropertyNameLength::<Identity, OFFSET>,
            GetType: GetType::<Identity, OFFSET>,
            GetPropertyIndex: GetPropertyIndex::<Identity, OFFSET>,
            SetValueByName: SetValueByName::<Identity, OFFSET>,
            SetValue: SetValue::<Identity, OFFSET>,
            GetValueByName: GetValueByName::<Identity, OFFSET>,
            GetValue: GetValue::<Identity, OFFSET>,
            GetValueSize: GetValueSize::<Identity, OFFSET>,
            GetSubProperties: GetSubProperties::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ID2D1Properties as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for ID2D1Properties {}
windows_core::imp::define_interface!(
    ID2D1RadialGradientBrush,
    ID2D1RadialGradientBrush_Vtbl,
    0x2cd906ac_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1RadialGradientBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1RadialGradientBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1RadialGradientBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetCenter: usize,
    SetGradientOriginOffset: usize,
    SetRadiusX: usize,
    SetRadiusY: usize,
    GetCenter: usize,
    GetGradientOriginOffset: usize,
    GetRadiusX: usize,
    GetRadiusY: usize,
    GetGradientStopCollection: usize,
}
impl windows_core::RuntimeName for ID2D1RadialGradientBrush {}
windows_core::imp::define_interface!(
    ID2D1RenderTarget,
    ID2D1RenderTarget_Vtbl,
    0x2cd90694_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1RenderTarget {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1RenderTarget, windows_core::IUnknown, ID2D1Resource);
impl ID2D1RenderTarget {
    pub(crate) unsafe fn CreateBitmap(
        &self,
        size: D2D_SIZE_U,
        srcdata: Option<*const core::ffi::c_void>,
        pitch: u32,
        bitmapproperties: *const D2D1_BITMAP_PROPERTIES,
    ) -> windows_core::Result<ID2D1Bitmap> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmap)(
                windows_core::Interface::as_raw(self),
                size,
                srcdata.unwrap_or(core::mem::zeroed()) as _,
                pitch,
                bitmapproperties,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromWicBitmap<P0>(
        &self,
        wicbitmapsource: P0,
        bitmapproperties: Option<*const D2D1_BITMAP_PROPERTIES>,
    ) -> windows_core::Result<ID2D1Bitmap>
    where
        P0: windows_core::Param<IWICBitmapSource>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromWicBitmap)(
                windows_core::Interface::as_raw(self),
                wicbitmapsource.param().abi(),
                bitmapproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateSharedBitmap(
        &self,
        riid: *const windows_core::GUID,
        data: *mut core::ffi::c_void,
        bitmapproperties: Option<*const D2D1_BITMAP_PROPERTIES>,
        bitmap: *mut Option<ID2D1Bitmap>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateSharedBitmap)(
                windows_core::Interface::as_raw(self),
                riid,
                data as _,
                bitmapproperties.unwrap_or(core::mem::zeroed()) as _,
                core::mem::transmute(bitmap),
            )
        }
    }
    pub(crate) unsafe fn CreateBitmapBrush<P0>(
        &self,
        bitmap: P0,
        bitmapbrushproperties: Option<*const D2D1_BITMAP_BRUSH_PROPERTIES>,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
    ) -> windows_core::Result<ID2D1BitmapBrush>
    where
        P0: windows_core::Param<ID2D1Bitmap>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapBrush)(
                windows_core::Interface::as_raw(self),
                bitmap.param().abi(),
                bitmapbrushproperties.unwrap_or(core::mem::zeroed()) as _,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateSolidColorBrush(
        &self,
        color: *const D2D_COLOR_F,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
    ) -> windows_core::Result<ID2D1SolidColorBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSolidColorBrush)(
                windows_core::Interface::as_raw(self),
                color,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateGradientStopCollection(
        &self,
        gradientstops: &[D2D1_GRADIENT_STOP],
        colorinterpolationgamma: D2D1_GAMMA,
        extendmode: D2D1_EXTEND_MODE,
    ) -> windows_core::Result<ID2D1GradientStopCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateGradientStopCollection)(
                windows_core::Interface::as_raw(self),
                gradientstops.as_ptr(),
                gradientstops.len().try_into().unwrap(),
                colorinterpolationgamma,
                extendmode,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateLinearGradientBrush<P2>(
        &self,
        lineargradientbrushproperties: *const D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
        gradientstopcollection: P2,
    ) -> windows_core::Result<ID2D1LinearGradientBrush>
    where
        P2: windows_core::Param<ID2D1GradientStopCollection>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateLinearGradientBrush)(
                windows_core::Interface::as_raw(self),
                lineargradientbrushproperties,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                gradientstopcollection.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateRadialGradientBrush<P2>(
        &self,
        radialgradientbrushproperties: *const D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
        gradientstopcollection: P2,
    ) -> windows_core::Result<ID2D1RadialGradientBrush>
    where
        P2: windows_core::Param<ID2D1GradientStopCollection>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateRadialGradientBrush)(
                windows_core::Interface::as_raw(self),
                radialgradientbrushproperties,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                gradientstopcollection.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateCompatibleRenderTarget(
        &self,
        desiredsize: Option<*const D2D_SIZE_F>,
        desiredpixelsize: Option<*const D2D_SIZE_U>,
        desiredformat: Option<*const D2D1_PIXEL_FORMAT>,
        options: D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS,
    ) -> windows_core::Result<ID2D1BitmapRenderTarget> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCompatibleRenderTarget)(
                windows_core::Interface::as_raw(self),
                desiredsize.unwrap_or(core::mem::zeroed()) as _,
                desiredpixelsize.unwrap_or(core::mem::zeroed()) as _,
                desiredformat.unwrap_or(core::mem::zeroed()) as _,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateLayer(
        &self,
        size: Option<*const D2D_SIZE_F>,
    ) -> windows_core::Result<ID2D1Layer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateLayer)(
                windows_core::Interface::as_raw(self),
                size.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateMesh(&self) -> windows_core::Result<ID2D1Mesh> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateMesh)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn DrawLine<P2, P4>(
        &self,
        point0: windows_numerics::Vector2,
        point1: windows_numerics::Vector2,
        brush: P2,
        strokewidth: f32,
        strokestyle: P4,
    ) where
        P2: windows_core::Param<ID2D1Brush>,
        P4: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawLine)(
                windows_core::Interface::as_raw(self),
                point0,
                point1,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawRectangle<P1, P3>(
        &self,
        rect: *const D2D_RECT_F,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawRectangle)(
                windows_core::Interface::as_raw(self),
                rect,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillRectangle<P1>(&self, rect: *const D2D_RECT_F, brush: P1)
    where
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillRectangle)(
                windows_core::Interface::as_raw(self),
                rect,
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawRoundedRectangle<P1, P3>(
        &self,
        roundedrect: *const D2D1_ROUNDED_RECT,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawRoundedRectangle)(
                windows_core::Interface::as_raw(self),
                roundedrect,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillRoundedRectangle<P1>(
        &self,
        roundedrect: *const D2D1_ROUNDED_RECT,
        brush: P1,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillRoundedRectangle)(
                windows_core::Interface::as_raw(self),
                roundedrect,
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawEllipse<P1, P3>(
        &self,
        ellipse: *const D2D1_ELLIPSE,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawEllipse)(
                windows_core::Interface::as_raw(self),
                ellipse,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillEllipse<P1>(&self, ellipse: *const D2D1_ELLIPSE, brush: P1)
    where
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillEllipse)(
                windows_core::Interface::as_raw(self),
                ellipse,
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawGeometry<P0, P1, P3>(
        &self,
        geometry: P0,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P0: windows_core::Param<ID2D1Geometry>,
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGeometry)(
                windows_core::Interface::as_raw(self),
                geometry.param().abi(),
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillGeometry<P0, P1, P2>(&self, geometry: P0, brush: P1, opacitybrush: P2)
    where
        P0: windows_core::Param<ID2D1Geometry>,
        P1: windows_core::Param<ID2D1Brush>,
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillGeometry)(
                windows_core::Interface::as_raw(self),
                geometry.param().abi(),
                brush.param().abi(),
                opacitybrush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillMesh<P0, P1>(&self, mesh: P0, brush: P1)
    where
        P0: windows_core::Param<ID2D1Mesh>,
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillMesh)(
                windows_core::Interface::as_raw(self),
                mesh.param().abi(),
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillOpacityMask<P0, P1>(
        &self,
        opacitymask: P0,
        brush: P1,
        content: D2D1_OPACITY_MASK_CONTENT,
        destinationrectangle: Option<*const D2D_RECT_F>,
        sourcerectangle: Option<*const D2D_RECT_F>,
    ) where
        P0: windows_core::Param<ID2D1Bitmap>,
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillOpacityMask)(
                windows_core::Interface::as_raw(self),
                opacitymask.param().abi(),
                brush.param().abi(),
                content,
                destinationrectangle.unwrap_or(core::mem::zeroed()) as _,
                sourcerectangle.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn DrawBitmap<P0>(
        &self,
        bitmap: P0,
        destinationrectangle: Option<*const D2D_RECT_F>,
        opacity: f32,
        interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE,
        sourcerectangle: Option<*const D2D_RECT_F>,
    ) where
        P0: windows_core::Param<ID2D1Bitmap>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawBitmap)(
                windows_core::Interface::as_raw(self),
                bitmap.param().abi(),
                destinationrectangle.unwrap_or(core::mem::zeroed()) as _,
                opacity,
                interpolationmode,
                sourcerectangle.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn DrawText<P2, P4>(
        &self,
        string: &[u16],
        textformat: P2,
        layoutrect: *const D2D_RECT_F,
        defaultfillbrush: P4,
        options: D2D1_DRAW_TEXT_OPTIONS,
        measuringmode: DWRITE_MEASURING_MODE,
    ) where
        P2: windows_core::Param<IDWriteTextFormat>,
        P4: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawText)(
                windows_core::Interface::as_raw(self),
                string.as_ptr(),
                string.len().try_into().unwrap(),
                textformat.param().abi(),
                layoutrect,
                defaultfillbrush.param().abi(),
                options,
                measuringmode,
            );
        }
    }
    pub(crate) unsafe fn DrawTextLayout<P1, P2>(
        &self,
        origin: windows_numerics::Vector2,
        textlayout: P1,
        defaultfillbrush: P2,
        options: D2D1_DRAW_TEXT_OPTIONS,
    ) where
        P1: windows_core::Param<IDWriteTextLayout>,
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawTextLayout)(
                windows_core::Interface::as_raw(self),
                origin,
                textlayout.param().abi(),
                defaultfillbrush.param().abi(),
                options,
            );
        }
    }
    pub(crate) unsafe fn DrawGlyphRun<P2>(
        &self,
        baselineorigin: windows_numerics::Vector2,
        glyphrun: *const DWRITE_GLYPH_RUN,
        foregroundbrush: P2,
        measuringmode: DWRITE_MEASURING_MODE,
    ) where
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGlyphRun)(
                windows_core::Interface::as_raw(self),
                baselineorigin,
                glyphrun,
                foregroundbrush.param().abi(),
                measuringmode,
            );
        }
    }
    pub(crate) unsafe fn SetTransform(&self, transform: *const windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransform)(
                windows_core::Interface::as_raw(self),
                transform,
            );
        }
    }
    pub(crate) unsafe fn GetTransform(&self, transform: *mut windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).GetTransform)(
                windows_core::Interface::as_raw(self),
                transform as _,
            );
        }
    }
    pub(crate) unsafe fn SetAntialiasMode(&self, antialiasmode: D2D1_ANTIALIAS_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetAntialiasMode)(
                windows_core::Interface::as_raw(self),
                antialiasmode,
            );
        }
    }
    pub(crate) unsafe fn GetAntialiasMode(&self) -> D2D1_ANTIALIAS_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetAntialiasMode)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn SetTextAntialiasMode(&self, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextAntialiasMode)(
                windows_core::Interface::as_raw(self),
                textantialiasmode,
            );
        }
    }
    pub(crate) unsafe fn GetTextAntialiasMode(&self) -> D2D1_TEXT_ANTIALIAS_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetTextAntialiasMode)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn SetTextRenderingParams<P0>(&self, textrenderingparams: P0)
    where
        P0: windows_core::Param<IDWriteRenderingParams>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextRenderingParams)(
                windows_core::Interface::as_raw(self),
                textrenderingparams.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn GetTextRenderingParams(
        &self,
    ) -> windows_core::Result<IDWriteRenderingParams> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTextRenderingParams)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn SetTags(&self, tag1: D2D1_TAG, tag2: D2D1_TAG) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTags)(
                windows_core::Interface::as_raw(self),
                tag1,
                tag2,
            );
        }
    }
    pub(crate) unsafe fn GetTags(&self, tag1: Option<*mut D2D1_TAG>, tag2: Option<*mut D2D1_TAG>) {
        unsafe {
            (windows_core::Interface::vtable(self).GetTags)(
                windows_core::Interface::as_raw(self),
                tag1.unwrap_or(core::mem::zeroed()) as _,
                tag2.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn PushLayer<P1>(
        &self,
        layerparameters: *const D2D1_LAYER_PARAMETERS,
        layer: P1,
    ) where
        P1: windows_core::Param<ID2D1Layer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PushLayer)(
                windows_core::Interface::as_raw(self),
                layerparameters,
                layer.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn PopLayer(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).PopLayer)(windows_core::Interface::as_raw(self));
        }
    }
    pub(crate) unsafe fn Flush(
        &self,
        tag1: Option<*mut D2D1_TAG>,
        tag2: Option<*mut D2D1_TAG>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Flush)(
                windows_core::Interface::as_raw(self),
                tag1.unwrap_or(core::mem::zeroed()) as _,
                tag2.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn SaveDrawingState(
        &self,
        drawingstateblock: &Option<ID2D1DrawingStateBlock>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SaveDrawingState)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(drawingstateblock),
            );
        }
    }
    pub(crate) unsafe fn RestoreDrawingState<P0>(&self, drawingstateblock: P0)
    where
        P0: windows_core::Param<ID2D1DrawingStateBlock>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RestoreDrawingState)(
                windows_core::Interface::as_raw(self),
                drawingstateblock.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn PushAxisAlignedClip(
        &self,
        cliprect: *const D2D_RECT_F,
        antialiasmode: D2D1_ANTIALIAS_MODE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).PushAxisAlignedClip)(
                windows_core::Interface::as_raw(self),
                cliprect,
                antialiasmode,
            );
        }
    }
    pub(crate) unsafe fn PopAxisAlignedClip(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).PopAxisAlignedClip)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
    pub(crate) unsafe fn Clear(&self, clearcolor: Option<*const D2D_COLOR_F>) {
        unsafe {
            (windows_core::Interface::vtable(self).Clear)(
                windows_core::Interface::as_raw(self),
                clearcolor.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn BeginDraw(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).BeginDraw)(windows_core::Interface::as_raw(
                self,
            ));
        }
    }
    pub(crate) unsafe fn EndDraw(
        &self,
        tag1: Option<*mut D2D1_TAG>,
        tag2: Option<*mut D2D1_TAG>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).EndDraw)(
                windows_core::Interface::as_raw(self),
                tag1.unwrap_or(core::mem::zeroed()) as _,
                tag2.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn GetPixelFormat(&self) -> D2D1_PIXEL_FORMAT {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPixelFormat)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn SetDpi(&self, dpix: f32, dpiy: f32) {
        unsafe {
            (windows_core::Interface::vtable(self).SetDpi)(
                windows_core::Interface::as_raw(self),
                dpix,
                dpiy,
            );
        }
    }
    pub(crate) unsafe fn GetDpi(&self, dpix: *mut f32, dpiy: *mut f32) {
        unsafe {
            (windows_core::Interface::vtable(self).GetDpi)(
                windows_core::Interface::as_raw(self),
                dpix as _,
                dpiy as _,
            );
        }
    }
    pub(crate) unsafe fn GetSize(&self) -> D2D_SIZE_F {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn GetPixelSize(&self) -> D2D_SIZE_U {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPixelSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn GetMaximumBitmapSize(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetMaximumBitmapSize)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn IsSupported(
        &self,
        rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES,
    ) -> windows_core::BOOL {
        unsafe {
            (windows_core::Interface::vtable(self).IsSupported)(
                windows_core::Interface::as_raw(self),
                rendertargetproperties,
            )
        }
    }
}
#[repr(C)]
pub struct ID2D1RenderTarget_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    pub CreateBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D2D_SIZE_U,
        *const core::ffi::c_void,
        u32,
        *const D2D1_BITMAP_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromWicBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSharedBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSolidColorBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_COLOR_F,
        *const D2D1_BRUSH_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateGradientStopCollection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_GRADIENT_STOP,
        u32,
        D2D1_GAMMA,
        D2D1_EXTEND_MODE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLinearGradientBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateRadialGradientBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCompatibleRenderTarget: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_SIZE_F,
        *const D2D_SIZE_U,
        *const D2D1_PIXEL_FORMAT,
        D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLayer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_SIZE_F,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateMesh: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DrawLine: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        windows_numerics::Vector2,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub DrawRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *mut core::ffi::c_void,
    ),
    pub DrawRoundedRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ROUNDED_RECT,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillRoundedRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ROUNDED_RECT,
        *mut core::ffi::c_void,
    ),
    pub DrawEllipse: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ELLIPSE,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillEllipse: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ELLIPSE,
        *mut core::ffi::c_void,
    ),
    pub DrawGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ),
    pub FillMesh: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ),
    pub FillOpacityMask: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D2D1_OPACITY_MASK_CONTENT,
        *const D2D_RECT_F,
        *const D2D_RECT_F,
    ),
    pub DrawBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        f32,
        D2D1_BITMAP_INTERPOLATION_MODE,
        *const D2D_RECT_F,
    ),
    pub DrawText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u16,
        u32,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *mut core::ffi::c_void,
        D2D1_DRAW_TEXT_OPTIONS,
        DWRITE_MEASURING_MODE,
    ),
    pub DrawTextLayout: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D2D1_DRAW_TEXT_OPTIONS,
    ),
    pub DrawGlyphRun: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *const DWRITE_GLYPH_RUN,
        *mut core::ffi::c_void,
        DWRITE_MEASURING_MODE,
    ),
    pub SetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const windows_numerics::Matrix3x2),
    pub GetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_numerics::Matrix3x2),
    pub SetAntialiasMode: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_ANTIALIAS_MODE),
    pub GetAntialiasMode: unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_ANTIALIAS_MODE,
    pub SetTextAntialiasMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_TEXT_ANTIALIAS_MODE),
    pub GetTextAntialiasMode:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_TEXT_ANTIALIAS_MODE,
    pub SetTextRenderingParams:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub GetTextRenderingParams:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub SetTags: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_TAG, D2D1_TAG),
    pub GetTags: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D1_TAG, *mut D2D1_TAG),
    pub PushLayer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_LAYER_PARAMETERS,
        *mut core::ffi::c_void,
    ),
    pub PopLayer: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub Flush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D2D1_TAG,
        *mut D2D1_TAG,
    ) -> windows_core::HRESULT,
    pub SaveDrawingState: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub RestoreDrawingState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub PushAxisAlignedClip:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D_RECT_F, D2D1_ANTIALIAS_MODE),
    pub PopAxisAlignedClip: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub Clear: unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D_COLOR_F),
    pub BeginDraw: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub EndDraw: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D2D1_TAG,
        *mut D2D1_TAG,
    ) -> windows_core::HRESULT,
    pub GetPixelFormat: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D1_PIXEL_FORMAT),
    pub SetDpi: unsafe extern "system" fn(*mut core::ffi::c_void, f32, f32),
    pub GetDpi: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32, *mut f32),
    pub GetSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D_SIZE_F),
    pub GetPixelSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D_SIZE_U),
    pub GetMaximumBitmapSize: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub IsSupported: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_RENDER_TARGET_PROPERTIES,
    ) -> windows_core::BOOL,
}
impl windows_core::RuntimeName for ID2D1RenderTarget {}
windows_core::imp::define_interface!(
    ID2D1Resource,
    ID2D1Resource_Vtbl,
    0x2cd90691_12e2_11dc_9fed_001143a055f9
);
windows_core::imp::interface_hierarchy!(ID2D1Resource, windows_core::IUnknown);
#[repr(C)]
pub struct ID2D1Resource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFactory: usize,
}
impl windows_core::RuntimeName for ID2D1Resource {}
windows_core::imp::define_interface!(
    ID2D1SolidColorBrush,
    ID2D1SolidColorBrush_Vtbl,
    0x2cd906a9_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1SolidColorBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1SolidColorBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1SolidColorBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetColor: usize,
    GetColor: usize,
}
impl windows_core::RuntimeName for ID2D1SolidColorBrush {}
windows_core::imp::define_interface!(
    ID2D1StrokeStyle,
    ID2D1StrokeStyle_Vtbl,
    0x2cd9069d_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1StrokeStyle {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1StrokeStyle, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1StrokeStyle_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetStartCap: usize,
    GetEndCap: usize,
    GetDashCap: usize,
    GetMiterLimit: usize,
    GetLineJoin: usize,
    GetDashOffset: usize,
    GetDashStyle: usize,
    GetDashesCount: usize,
    GetDashes: usize,
}
impl windows_core::RuntimeName for ID2D1StrokeStyle {}
windows_core::imp::define_interface!(
    IDWriteFontFace,
    IDWriteFontFace_Vtbl,
    0x5f49804d_7024_4d43_bfa9_d25984f53849
);
windows_core::imp::interface_hierarchy!(IDWriteFontFace, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteFontFace_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetType: usize,
    GetFiles: usize,
    GetIndex: usize,
    GetSimulations: usize,
    IsSymbolFont: usize,
    GetMetrics: usize,
    GetGlyphCount: usize,
    GetDesignGlyphMetrics: usize,
    GetGlyphIndices: usize,
    TryGetFontTable: usize,
    ReleaseFontTable: usize,
    GetGlyphRunOutline: usize,
    GetRecommendedRenderingMode: usize,
    GetGdiCompatibleMetrics: usize,
    GetGdiCompatibleGlyphMetrics: usize,
}
impl windows_core::RuntimeName for IDWriteFontFace {}
windows_core::imp::define_interface!(
    IDWriteRenderingParams,
    IDWriteRenderingParams_Vtbl,
    0x2f0da53a_2add_47cd_82ee_d9ec34688e75
);
windows_core::imp::interface_hierarchy!(IDWriteRenderingParams, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteRenderingParams_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetGamma: usize,
    GetEnhancedContrast: usize,
    GetClearTypeLevel: usize,
    GetPixelGeometry: usize,
    GetRenderingMode: usize,
}
impl windows_core::RuntimeName for IDWriteRenderingParams {}
windows_core::imp::define_interface!(
    IDWriteTextFormat,
    IDWriteTextFormat_Vtbl,
    0x9c906818_31d7_4fd3_a151_7c5e225db55a
);
windows_core::imp::interface_hierarchy!(IDWriteTextFormat, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteTextFormat_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetTextAlignment: usize,
    SetParagraphAlignment: usize,
    SetWordWrapping: usize,
    SetReadingDirection: usize,
    SetFlowDirection: usize,
    SetIncrementalTabStop: usize,
    SetTrimming: usize,
    SetLineSpacing: usize,
    GetTextAlignment: usize,
    GetParagraphAlignment: usize,
    GetWordWrapping: usize,
    GetReadingDirection: usize,
    GetFlowDirection: usize,
    GetIncrementalTabStop: usize,
    GetTrimming: usize,
    GetLineSpacing: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
}
impl windows_core::RuntimeName for IDWriteTextFormat {}
windows_core::imp::define_interface!(
    IDWriteTextLayout,
    IDWriteTextLayout_Vtbl,
    0x53737037_6d14_410b_9bfe_0b182bb70961
);
impl core::ops::Deref for IDWriteTextLayout {
    type Target = IDWriteTextFormat;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDWriteTextLayout,
    windows_core::IUnknown,
    IDWriteTextFormat
);
#[repr(C)]
pub struct IDWriteTextLayout_Vtbl {
    pub base__: IDWriteTextFormat_Vtbl,
    SetMaxWidth: usize,
    SetMaxHeight: usize,
    SetFontCollection: usize,
    SetFontFamilyName: usize,
    SetFontWeight: usize,
    SetFontStyle: usize,
    SetFontStretch: usize,
    SetFontSize: usize,
    SetUnderline: usize,
    SetStrikethrough: usize,
    SetDrawingEffect: usize,
    SetInlineObject: usize,
    SetTypography: usize,
    SetLocaleName: usize,
    GetMaxWidth: usize,
    GetMaxHeight: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetUnderline: usize,
    GetStrikethrough: usize,
    GetDrawingEffect: usize,
    GetInlineObject: usize,
    GetTypography: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
    Draw: usize,
    GetLineMetrics: usize,
    GetMetrics: usize,
    GetOverhangMetrics: usize,
    GetClusterMetrics: usize,
    DetermineMinWidth: usize,
    HitTestPoint: usize,
    HitTestTextPosition: usize,
    HitTestTextRange: usize,
}
impl windows_core::RuntimeName for IDWriteTextLayout {}
windows_core::imp::define_interface!(
    IDXGIDeviceSubObject,
    IDXGIDeviceSubObject_Vtbl,
    0x3d3e0379_f9de_4d58_bb6c_18d62992f1a6
);
impl core::ops::Deref for IDXGIDeviceSubObject {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIDeviceSubObject, windows_core::IUnknown, IDXGIObject);
#[repr(C)]
pub struct IDXGIDeviceSubObject_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    GetDevice: usize,
}
impl windows_core::RuntimeName for IDXGIDeviceSubObject {}
windows_core::imp::define_interface!(
    IDXGIObject,
    IDXGIObject_Vtbl,
    0xaec22fb8_76f3_4639_9be0_28eb43a67a2e
);
windows_core::imp::interface_hierarchy!(IDXGIObject, windows_core::IUnknown);
#[repr(C)]
pub struct IDXGIObject_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetPrivateData: usize,
    SetPrivateDataInterface: usize,
    GetPrivateData: usize,
    GetParent: usize,
}
impl windows_core::RuntimeName for IDXGIObject {}
windows_core::imp::define_interface!(
    IDXGISurface,
    IDXGISurface_Vtbl,
    0xcafcb56c_6ac3_4889_bf47_9e23bbd260ec
);
impl core::ops::Deref for IDXGISurface {
    type Target = IDXGIDeviceSubObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDXGISurface,
    windows_core::IUnknown,
    IDXGIObject,
    IDXGIDeviceSubObject
);
#[repr(C)]
pub struct IDXGISurface_Vtbl {
    pub base__: IDXGIDeviceSubObject_Vtbl,
    GetDesc: usize,
    Map: usize,
    Unmap: usize,
}
impl windows_core::RuntimeName for IDXGISurface {}
windows_core::imp::define_interface!(
    IWICBitmapSource,
    IWICBitmapSource_Vtbl,
    0x00000120_a8f2_4877_ba0a_fd2b6645fb94
);
windows_core::imp::interface_hierarchy!(IWICBitmapSource, windows_core::IUnknown);
#[repr(C)]
pub struct IWICBitmapSource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetSize: usize,
    GetPixelFormat: usize,
    GetResolution: usize,
    CopyPalette: usize,
    CopyPixels: usize,
}
impl windows_core::RuntimeName for IWICBitmapSource {}
windows_core::imp::define_interface!(
    IWICColorContext,
    IWICColorContext_Vtbl,
    0x3c613a02_34b2_44ea_9a7c_45aea9c6fd6d
);
windows_core::imp::interface_hierarchy!(IWICColorContext, windows_core::IUnknown);
#[repr(C)]
pub struct IWICColorContext_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    InitializeFromFilename: usize,
    InitializeFromMemory: usize,
    InitializeFromExifColorSpace: usize,
    GetType: usize,
    GetProfileBytes: usize,
    GetExifColorSpace: usize,
}
impl windows_core::RuntimeName for IWICColorContext {}
