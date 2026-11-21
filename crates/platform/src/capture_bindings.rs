#[inline]
pub unsafe fn CreateDirect3D11DeviceFromDXGIDevice<P0>(
    dxgidevice: P0,
) -> windows_core::Result<windows_core::IInspectable>
where
    P0: windows_core::Param<IDXGIDevice>,
{
    windows_core::link!("d3d11.dll" "system" fn CreateDirect3D11DeviceFromDXGIDevice(dxgidevice : *mut core::ffi::c_void, graphicsdevice : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        CreateDirect3D11DeviceFromDXGIDevice(dxgidevice.param().abi(), &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AppCapabilityAccessStatus(pub i32);
impl AppCapabilityAccessStatus {
    pub const DeniedBySystem: Self = Self(0);
    pub const NotDeclaredByApp: Self = Self(1);
    pub const DeniedByUser: Self = Self(2);
    pub const UserPromptRequired: Self = Self(3);
    pub const Allowed: Self = Self(4);
}
impl windows_core::imp::TypeKind for AppCapabilityAccessStatus {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AppCapabilityAccessStatus {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Security.Authorization.AppCapabilityAccess.AppCapabilityAccessStatus;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Security.Authorization.AppCapabilityAccess.AppCapabilityAccessStatus",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionAnimation(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionAnimation,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(CompositionAnimation, CompositionObject);
impl CompositionAnimation {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
}
impl windows_core::RuntimeType for CompositionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionAnimation>();
}
unsafe impl windows_core::Interface for CompositionAnimation {
    type Vtable = <ICompositionAnimation as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionAnimation as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionAnimation {
    const NAME: &'static str = "Windows.UI.Composition.CompositionAnimation";
}
unsafe impl Send for CompositionAnimation {}
unsafe impl Sync for CompositionAnimation {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionObject(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    CompositionObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl CompositionObject {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
    pub(crate) fn StartAnimationWithIAnimationObject<P0, P2>(
        target: P0,
        propertyname: &windows_core::HSTRING,
        animation: P2,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IAnimationObject>,
        P2: windows_core::Param<CompositionAnimation>,
    {
        Self::ICompositionObjectStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).StartAnimationWithIAnimationObject)(
                windows_core::Interface::as_raw(this),
                target.param().abi(),
                core::mem::transmute_copy(propertyname),
                animation.param().abi(),
            )
            .ok()
        })
    }
    pub(crate) fn StartAnimationGroupWithIAnimationObject<P0, P1>(
        target: P0,
        animation: P1,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IAnimationObject>,
        P1: windows_core::Param<ICompositionAnimationBase>,
    {
        Self::ICompositionObjectStatics(|this| unsafe {
            (windows_core::Interface::vtable(this).StartAnimationGroupWithIAnimationObject)(
                windows_core::Interface::as_raw(this),
                target.param().abi(),
                animation.param().abi(),
            )
            .ok()
        })
    }
    fn ICompositionObjectStatics<
        R,
        F: FnOnce(&ICompositionObjectStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            CompositionObject,
            ICompositionObjectStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for CompositionObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ICompositionObject>();
}
unsafe impl windows_core::Interface for CompositionObject {
    type Vtable = <ICompositionObject as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ICompositionObject as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for CompositionObject {
    const NAME: &'static str = "Windows.UI.Composition.CompositionObject";
}
unsafe impl Send for CompositionObject {}
unsafe impl Sync for CompositionObject {}
pub type DXGI_FORMAT = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_MAPPED_RECT {
    pub Pitch: i32,
    pub pBits: *mut u8,
}
pub type DXGI_RESIDENCY = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_SAMPLE_DESC {
    pub Count: u32,
    pub Quality: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_SHARED_RESOURCE {
    pub Handle: HANDLE,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DXGI_SURFACE_DESC {
    pub Width: u32,
    pub Height: u32,
    pub Format: DXGI_FORMAT,
    pub SampleDesc: DXGI_SAMPLE_DESC,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DXGI_USAGE(pub u32);
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Direct3D11CaptureFrame(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Direct3D11CaptureFrame,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(Direct3D11CaptureFrame, IClosable);
impl Direct3D11CaptureFrame {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
    pub(crate) fn Surface(&self) -> windows_core::Result<IDirect3DSurface> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Surface)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn ContentSize(&self) -> windows_core::Result<SizeInt32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn DirtyRegions(
        &self,
    ) -> windows_core::Result<windows_collections::IVectorView<RectInt32>> {
        let this = &windows_core::Interface::cast::<IDirect3D11CaptureFrame2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).DirtyRegions)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for Direct3D11CaptureFrame {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDirect3D11CaptureFrame>();
}
unsafe impl windows_core::Interface for Direct3D11CaptureFrame {
    type Vtable = <IDirect3D11CaptureFrame as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDirect3D11CaptureFrame as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Direct3D11CaptureFrame {
    const NAME: &'static str = "Windows.Graphics.Capture.Direct3D11CaptureFrame";
}
unsafe impl Send for Direct3D11CaptureFrame {}
unsafe impl Sync for Direct3D11CaptureFrame {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Direct3D11CaptureFramePool(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Direct3D11CaptureFramePool,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(Direct3D11CaptureFramePool, IClosable);
impl Direct3D11CaptureFramePool {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
    pub(crate) fn Recreate<P0>(
        &self,
        device: P0,
        pixelformat: DirectXPixelFormat,
        numberofbuffers: i32,
        size: SizeInt32,
    ) -> windows_core::Result<()>
    where
        P0: windows_core::Param<IDirect3DDevice>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Recreate)(
                windows_core::Interface::as_raw(self),
                device.param().abi(),
                pixelformat,
                numberofbuffers,
                size,
            )
            .ok()
        }
    }
    pub(crate) fn TryGetNextFrame(&self) -> windows_core::Result<Direct3D11CaptureFrame> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetNextFrame)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateCaptureSession<P0>(
        &self,
        item: P0,
    ) -> windows_core::Result<GraphicsCaptureSession>
    where
        P0: windows_core::Param<GraphicsCaptureItem>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCaptureSession)(
                windows_core::Interface::as_raw(self),
                item.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Create<P0>(
        device: P0,
        pixelformat: DirectXPixelFormat,
        numberofbuffers: i32,
        size: SizeInt32,
    ) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IDirect3DDevice>,
    {
        Self::IDirect3D11CaptureFramePoolStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Create)(
                windows_core::Interface::as_raw(this),
                device.param().abi(),
                pixelformat,
                numberofbuffers,
                size,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn CreateFreeThreaded<P0>(
        device: P0,
        pixelformat: DirectXPixelFormat,
        numberofbuffers: i32,
        size: SizeInt32,
    ) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IDirect3DDevice>,
    {
        Self::IDirect3D11CaptureFramePoolStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFreeThreaded)(
                windows_core::Interface::as_raw(this),
                device.param().abi(),
                pixelformat,
                numberofbuffers,
                size,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IDirect3D11CaptureFramePoolStatics<
        R,
        F: FnOnce(&IDirect3D11CaptureFramePoolStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            Direct3D11CaptureFramePool,
            IDirect3D11CaptureFramePoolStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IDirect3D11CaptureFramePoolStatics2<
        R,
        F: FnOnce(&IDirect3D11CaptureFramePoolStatics2) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            Direct3D11CaptureFramePool,
            IDirect3D11CaptureFramePoolStatics2,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for Direct3D11CaptureFramePool {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDirect3D11CaptureFramePool>();
}
unsafe impl windows_core::Interface for Direct3D11CaptureFramePool {
    type Vtable = <IDirect3D11CaptureFramePool as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDirect3D11CaptureFramePool as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Direct3D11CaptureFramePool {
    const NAME: &'static str = "Windows.Graphics.Capture.Direct3D11CaptureFramePool";
}
unsafe impl Send for Direct3D11CaptureFramePool {}
unsafe impl Sync for Direct3D11CaptureFramePool {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Direct3DMultisampleDescription {
    pub Count: i32,
    pub Quality: i32,
}
impl windows_core::imp::TypeKind for Direct3DMultisampleDescription {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Direct3DMultisampleDescription {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"struct(Windows.Graphics.DirectX.Direct3D11.Direct3DMultisampleDescription;i4;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.DirectX.Direct3D11.Direct3DMultisampleDescription",
    );
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Direct3DSurfaceDescription {
    pub Width: i32,
    pub Height: i32,
    pub Format: DirectXPixelFormat,
    pub MultisampleDescription: Direct3DMultisampleDescription,
}
impl windows_core::imp::TypeKind for Direct3DSurfaceDescription {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Direct3DSurfaceDescription {
    const SIGNATURE : windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice (b"struct(Windows.Graphics.DirectX.Direct3D11.Direct3DSurfaceDescription;i4;i4;enum(Windows.Graphics.DirectX.DirectXPixelFormat;i4);struct(Windows.Graphics.DirectX.Direct3D11.Direct3DMultisampleDescription;i4;i4))") ;
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.DirectX.Direct3D11.Direct3DSurfaceDescription",
    );
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectXPixelFormat(pub i32);
impl DirectXPixelFormat {
    pub const Unknown: Self = Self(0);
    pub const R32G32B32A32Typeless: Self = Self(1);
    pub const R32G32B32A32Float: Self = Self(2);
    pub const R32G32B32A32UInt: Self = Self(3);
    pub const R32G32B32A32Int: Self = Self(4);
    pub const R32G32B32Typeless: Self = Self(5);
    pub const R32G32B32Float: Self = Self(6);
    pub const R32G32B32UInt: Self = Self(7);
    pub const R32G32B32Int: Self = Self(8);
    pub const R16G16B16A16Typeless: Self = Self(9);
    pub const R16G16B16A16Float: Self = Self(10);
    pub const R16G16B16A16UIntNormalized: Self = Self(11);
    pub const R16G16B16A16UInt: Self = Self(12);
    pub const R16G16B16A16IntNormalized: Self = Self(13);
    pub const R16G16B16A16Int: Self = Self(14);
    pub const R32G32Typeless: Self = Self(15);
    pub const R32G32Float: Self = Self(16);
    pub const R32G32UInt: Self = Self(17);
    pub const R32G32Int: Self = Self(18);
    pub const R32G8X24Typeless: Self = Self(19);
    pub const D32FloatS8X24UInt: Self = Self(20);
    pub const R32FloatX8X24Typeless: Self = Self(21);
    pub const X32TypelessG8X24UInt: Self = Self(22);
    pub const R10G10B10A2Typeless: Self = Self(23);
    pub const R10G10B10A2UIntNormalized: Self = Self(24);
    pub const R10G10B10A2UInt: Self = Self(25);
    pub const R11G11B10Float: Self = Self(26);
    pub const R8G8B8A8Typeless: Self = Self(27);
    pub const R8G8B8A8UIntNormalized: Self = Self(28);
    pub const R8G8B8A8UIntNormalizedSrgb: Self = Self(29);
    pub const R8G8B8A8UInt: Self = Self(30);
    pub const R8G8B8A8IntNormalized: Self = Self(31);
    pub const R8G8B8A8Int: Self = Self(32);
    pub const R16G16Typeless: Self = Self(33);
    pub const R16G16Float: Self = Self(34);
    pub const R16G16UIntNormalized: Self = Self(35);
    pub const R16G16UInt: Self = Self(36);
    pub const R16G16IntNormalized: Self = Self(37);
    pub const R16G16Int: Self = Self(38);
    pub const R32Typeless: Self = Self(39);
    pub const D32Float: Self = Self(40);
    pub const R32Float: Self = Self(41);
    pub const R32UInt: Self = Self(42);
    pub const R32Int: Self = Self(43);
    pub const R24G8Typeless: Self = Self(44);
    pub const D24UIntNormalizedS8UInt: Self = Self(45);
    pub const R24UIntNormalizedX8Typeless: Self = Self(46);
    pub const X24TypelessG8UInt: Self = Self(47);
    pub const R8G8Typeless: Self = Self(48);
    pub const R8G8UIntNormalized: Self = Self(49);
    pub const R8G8UInt: Self = Self(50);
    pub const R8G8IntNormalized: Self = Self(51);
    pub const R8G8Int: Self = Self(52);
    pub const R16Typeless: Self = Self(53);
    pub const R16Float: Self = Self(54);
    pub const D16UIntNormalized: Self = Self(55);
    pub const R16UIntNormalized: Self = Self(56);
    pub const R16UInt: Self = Self(57);
    pub const R16IntNormalized: Self = Self(58);
    pub const R16Int: Self = Self(59);
    pub const R8Typeless: Self = Self(60);
    pub const R8UIntNormalized: Self = Self(61);
    pub const R8UInt: Self = Self(62);
    pub const R8IntNormalized: Self = Self(63);
    pub const R8Int: Self = Self(64);
    pub const A8UIntNormalized: Self = Self(65);
    pub const R1UIntNormalized: Self = Self(66);
    pub const R9G9B9E5SharedExponent: Self = Self(67);
    pub const R8G8B8G8UIntNormalized: Self = Self(68);
    pub const G8R8G8B8UIntNormalized: Self = Self(69);
    pub const BC1Typeless: Self = Self(70);
    pub const BC1UIntNormalized: Self = Self(71);
    pub const BC1UIntNormalizedSrgb: Self = Self(72);
    pub const BC2Typeless: Self = Self(73);
    pub const BC2UIntNormalized: Self = Self(74);
    pub const BC2UIntNormalizedSrgb: Self = Self(75);
    pub const BC3Typeless: Self = Self(76);
    pub const BC3UIntNormalized: Self = Self(77);
    pub const BC3UIntNormalizedSrgb: Self = Self(78);
    pub const BC4Typeless: Self = Self(79);
    pub const BC4UIntNormalized: Self = Self(80);
    pub const BC4IntNormalized: Self = Self(81);
    pub const BC5Typeless: Self = Self(82);
    pub const BC5UIntNormalized: Self = Self(83);
    pub const BC5IntNormalized: Self = Self(84);
    pub const B5G6R5UIntNormalized: Self = Self(85);
    pub const B5G5R5A1UIntNormalized: Self = Self(86);
    pub const B8G8R8A8UIntNormalized: Self = Self(87);
    pub const B8G8R8X8UIntNormalized: Self = Self(88);
    pub const R10G10B10XRBiasA2UIntNormalized: Self = Self(89);
    pub const B8G8R8A8Typeless: Self = Self(90);
    pub const B8G8R8A8UIntNormalizedSrgb: Self = Self(91);
    pub const B8G8R8X8Typeless: Self = Self(92);
    pub const B8G8R8X8UIntNormalizedSrgb: Self = Self(93);
    pub const BC6HTypeless: Self = Self(94);
    pub const BC6H16UnsignedFloat: Self = Self(95);
    pub const BC6H16Float: Self = Self(96);
    pub const BC7Typeless: Self = Self(97);
    pub const BC7UIntNormalized: Self = Self(98);
    pub const BC7UIntNormalizedSrgb: Self = Self(99);
    pub const Ayuv: Self = Self(100);
    pub const Y410: Self = Self(101);
    pub const Y416: Self = Self(102);
    pub const NV12: Self = Self(103);
    pub const P010: Self = Self(104);
    pub const P016: Self = Self(105);
    pub const Opaque420: Self = Self(106);
    pub const Yuy2: Self = Self(107);
    pub const Y210: Self = Self(108);
    pub const Y216: Self = Self(109);
    pub const NV11: Self = Self(110);
    pub const AI44: Self = Self(111);
    pub const IA44: Self = Self(112);
    pub const P8: Self = Self(113);
    pub const A8P8: Self = Self(114);
    pub const B4G4R4A4UIntNormalized: Self = Self(115);
    pub const P208: Self = Self(130);
    pub const V208: Self = Self(131);
    pub const V408: Self = Self(132);
    pub const SamplerFeedbackMinMipOpaque: Self = Self(189);
    pub const SamplerFeedbackMipRegionUsedOpaque: Self = Self(190);
    pub const A4B4G4R4: Self = Self(191);
}
impl windows_core::imp::TypeKind for DirectXPixelFormat {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for DirectXPixelFormat {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Graphics.DirectX.DirectXPixelFormat;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Graphics.DirectX.DirectXPixelFormat");
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DisplayId {
    pub Value: u64,
}
impl windows_core::imp::TypeKind for DisplayId {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for DisplayId {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Graphics.DisplayId;u8)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Graphics.DisplayId");
}
pub struct GraphicsCaptureAccess;
impl GraphicsCaptureAccess {
    pub(crate) fn RequestAccessAsync(
        request: GraphicsCaptureAccessKind,
    ) -> windows_core::Result<windows_future::IAsyncOperation<AppCapabilityAccessStatus>> {
        Self::IGraphicsCaptureAccessStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RequestAccessAsync)(
                windows_core::Interface::as_raw(this),
                request,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IGraphicsCaptureAccessStatics<
        R,
        F: FnOnce(&IGraphicsCaptureAccessStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            GraphicsCaptureAccess,
            IGraphicsCaptureAccessStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeName for GraphicsCaptureAccess {
    const NAME: &'static str = "Windows.Graphics.Capture.GraphicsCaptureAccess";
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GraphicsCaptureAccessKind(pub i32);
impl GraphicsCaptureAccessKind {
    pub const Borderless: Self = Self(0);
    pub const Programmatic: Self = Self(1);
}
impl windows_core::imp::TypeKind for GraphicsCaptureAccessKind {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for GraphicsCaptureAccessKind {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Graphics.Capture.GraphicsCaptureAccessKind;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.GraphicsCaptureAccessKind",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphicsCaptureItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GraphicsCaptureItem,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GraphicsCaptureItem {
    pub(crate) fn Size(&self) -> windows_core::Result<SizeInt32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn CreateFromVisual<P0>(visual: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<Visual>,
    {
        Self::IGraphicsCaptureItemStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateFromVisual)(
                windows_core::Interface::as_raw(this),
                visual.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn TryCreateFromWindowId(windowid: WindowId) -> windows_core::Result<Self> {
        Self::IGraphicsCaptureItemStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TryCreateFromWindowId)(
                windows_core::Interface::as_raw(this),
                windowid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn TryCreateFromDisplayId(displayid: DisplayId) -> windows_core::Result<Self> {
        Self::IGraphicsCaptureItemStatics2(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).TryCreateFromDisplayId)(
                windows_core::Interface::as_raw(this),
                displayid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IGraphicsCaptureItemStatics<
        R,
        F: FnOnce(&IGraphicsCaptureItemStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            GraphicsCaptureItem,
            IGraphicsCaptureItemStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IGraphicsCaptureItemStatics2<
        R,
        F: FnOnce(&IGraphicsCaptureItemStatics2) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            GraphicsCaptureItem,
            IGraphicsCaptureItemStatics2,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for GraphicsCaptureItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IGraphicsCaptureItem>();
}
unsafe impl windows_core::Interface for GraphicsCaptureItem {
    type Vtable = <IGraphicsCaptureItem as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGraphicsCaptureItem as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for GraphicsCaptureItem {
    const NAME: &'static str = "Windows.Graphics.Capture.GraphicsCaptureItem";
}
unsafe impl Send for GraphicsCaptureItem {}
unsafe impl Sync for GraphicsCaptureItem {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphicsCaptureSession(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GraphicsCaptureSession,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(GraphicsCaptureSession, IClosable);
impl GraphicsCaptureSession {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
    pub(crate) fn StartCapture(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).StartCapture)(windows_core::Interface::as_raw(
                self,
            ))
            .ok()
        }
    }
    pub(crate) fn IsCursorCaptureEnabled(&self) -> windows_core::Result<bool> {
        let this = &windows_core::Interface::cast::<IGraphicsCaptureSession2>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsCursorCaptureEnabled)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIsCursorCaptureEnabled(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IGraphicsCaptureSession2>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsCursorCaptureEnabled)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IsBorderRequired(&self) -> windows_core::Result<bool> {
        let this = &windows_core::Interface::cast::<IGraphicsCaptureSession3>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsBorderRequired)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetIsBorderRequired(&self, value: bool) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IGraphicsCaptureSession3>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetIsBorderRequired)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn IsSupported() -> windows_core::Result<bool> {
        Self::IGraphicsCaptureSessionStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).IsSupported)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    fn IGraphicsCaptureSessionStatics<
        R,
        F: FnOnce(&IGraphicsCaptureSessionStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            GraphicsCaptureSession,
            IGraphicsCaptureSessionStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for GraphicsCaptureSession {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IGraphicsCaptureSession>();
}
unsafe impl windows_core::Interface for GraphicsCaptureSession {
    type Vtable = <IGraphicsCaptureSession as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IGraphicsCaptureSession as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for GraphicsCaptureSession {
    const NAME: &'static str = "Windows.Graphics.Capture.GraphicsCaptureSession";
}
unsafe impl Send for GraphicsCaptureSession {}
unsafe impl Sync for GraphicsCaptureSession {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HMONITOR(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWND(pub *mut core::ffi::c_void);
windows_core::imp::define_interface!(
    IAnimationObject,
    IAnimationObject_Vtbl,
    0xe7141e0a_04b8_4fc5_a4dc_195392e57807
);
impl windows_core::RuntimeType for IAnimationObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IAnimationObject");
}
windows_core::imp::interface_hierarchy!(
    IAnimationObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IAnimationObject {
    const NAME: &'static str = "Windows.UI.Composition.IAnimationObject";
}
#[repr(C)]
pub struct IAnimationObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IClosable,
    IClosable_Vtbl,
    0x30d5a829_7fa4_4026_83bb_d75bae4ea99e
);
impl windows_core::RuntimeType for IClosable {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Foundation.IClosable");
}
windows_core::imp::interface_hierarchy!(
    IClosable,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl IClosable {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Close)(windows_core::Interface::as_raw(self))
                .ok()
        }
    }
}
impl windows_core::RuntimeName for IClosable {
    const NAME: &'static str = "Windows.Foundation.IClosable";
}
pub trait IClosable_Impl: windows_core::IUnknownImpl {
    fn Close(&self) -> windows_core::Result<()>;
}
impl IClosable_Vtbl {
    pub const fn new<Identity: IClosable_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Close<Identity: IClosable_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IClosable_Impl::Close(this).into()
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IClosable, OFFSET>(),
            Close: Close::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IClosable as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IClosable_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Close: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ICompositionAnimation,
    ICompositionAnimation_Vtbl,
    0x464c4c2c_1caa_4061_9b40_e13fde1503ca
);
impl windows_core::RuntimeType for ICompositionAnimation {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionAnimation");
}
#[repr(C)]
pub struct ICompositionAnimation_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionAnimationBase,
    ICompositionAnimationBase_Vtbl,
    0x1c2c2999_e818_48d3_a6dd_d78c82f8ace9
);
impl windows_core::RuntimeType for ICompositionAnimationBase {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionAnimationBase",
    );
}
windows_core::imp::interface_hierarchy!(
    ICompositionAnimationBase,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for ICompositionAnimationBase {
    const NAME: &'static str = "Windows.UI.Composition.ICompositionAnimationBase";
}
pub trait ICompositionAnimationBase_Impl: windows_core::IUnknownImpl {}
impl ICompositionAnimationBase_Vtbl {
    pub const fn new<Identity: ICompositionAnimationBase_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                ICompositionAnimationBase,
                OFFSET,
            >(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ICompositionAnimationBase as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct ICompositionAnimationBase_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionObject,
    ICompositionObject_Vtbl,
    0xbcb4ad45_7609_4550_934f_16002a68fded
);
impl windows_core::RuntimeType for ICompositionObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.ICompositionObject");
}
#[repr(C)]
pub struct ICompositionObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    ICompositionObjectStatics,
    ICompositionObjectStatics_Vtbl,
    0xc1ed052f_1ba2_44ba_a904_6a882a0a5adb
);
impl windows_core::RuntimeType for ICompositionObjectStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.UI.Composition.ICompositionObjectStatics",
    );
}
#[repr(C)]
pub struct ICompositionObjectStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub StartAnimationWithIAnimationObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub StartAnimationGroupWithIAnimationObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    ID3D11Multithread,
    ID3D11Multithread_Vtbl,
    0x9b7e4e00_342c_4106_a19f_4f2704f689f0
);
windows_core::imp::interface_hierarchy!(ID3D11Multithread, windows_core::IUnknown);
impl ID3D11Multithread {
    pub(crate) unsafe fn Enter(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).Enter)(windows_core::Interface::as_raw(self));
        }
    }
    pub(crate) unsafe fn Leave(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).Leave)(windows_core::Interface::as_raw(self));
        }
    }
    pub(crate) unsafe fn SetMultithreadProtected(&self, bmtprotect: bool) -> windows_core::BOOL {
        unsafe {
            (windows_core::Interface::vtable(self).SetMultithreadProtected)(
                windows_core::Interface::as_raw(self),
                bmtprotect.into(),
            )
        }
    }
    pub(crate) unsafe fn GetMultithreadProtected(&self) -> windows_core::BOOL {
        unsafe {
            (windows_core::Interface::vtable(self).GetMultithreadProtected)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
}
#[repr(C)]
pub struct ID3D11Multithread_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Enter: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub Leave: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub SetMultithreadProtected:
        unsafe extern "system" fn(*mut core::ffi::c_void, windows_core::BOOL) -> windows_core::BOOL,
    pub GetMultithreadProtected:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::BOOL,
}
pub trait ID3D11Multithread_Impl: windows_core::IUnknownImpl {
    fn Enter(&self);
    fn Leave(&self);
    fn SetMultithreadProtected(&self, bmtprotect: windows_core::BOOL) -> windows_core::BOOL;
    fn GetMultithreadProtected(&self) -> windows_core::BOOL;
}
impl ID3D11Multithread_Vtbl {
    pub const fn new<Identity: ID3D11Multithread_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Enter<Identity: ID3D11Multithread_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID3D11Multithread_Impl::Enter(this);
            }
        }
        unsafe extern "system" fn Leave<Identity: ID3D11Multithread_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID3D11Multithread_Impl::Leave(this);
            }
        }
        unsafe extern "system" fn SetMultithreadProtected<
            Identity: ID3D11Multithread_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            bmtprotect: windows_core::BOOL,
        ) -> windows_core::BOOL {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID3D11Multithread_Impl::SetMultithreadProtected(
                    this,
                    core::mem::transmute_copy(&bmtprotect),
                )
            }
        }
        unsafe extern "system" fn GetMultithreadProtected<
            Identity: ID3D11Multithread_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::BOOL {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                ID3D11Multithread_Impl::GetMultithreadProtected(this)
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Enter: Enter::<Identity, OFFSET>,
            Leave: Leave::<Identity, OFFSET>,
            SetMultithreadProtected: SetMultithreadProtected::<Identity, OFFSET>,
            GetMultithreadProtected: GetMultithreadProtected::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<ID3D11Multithread as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for ID3D11Multithread {}
windows_core::imp::define_interface!(
    IDXGIAdapter,
    IDXGIAdapter_Vtbl,
    0x2411e7e1_12ac_4ccf_bd14_9798e8534dc0
);
impl core::ops::Deref for IDXGIAdapter {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIAdapter, windows_core::IUnknown, IDXGIObject);
#[repr(C)]
pub struct IDXGIAdapter_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    EnumOutputs: usize,
    GetDesc: usize,
    CheckInterfaceSupport: usize,
}
impl windows_core::RuntimeName for IDXGIAdapter {}
windows_core::imp::define_interface!(
    IDXGIDevice,
    IDXGIDevice_Vtbl,
    0x54ec77fa_1377_44e6_8c32_88fd5f44c84c
);
impl core::ops::Deref for IDXGIDevice {
    type Target = IDXGIObject;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IDXGIDevice, windows_core::IUnknown, IDXGIObject);
impl IDXGIDevice {
    pub(crate) unsafe fn GetAdapter(&self) -> windows_core::Result<IDXGIAdapter> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAdapter)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateSurface(
        &self,
        pdesc: *const DXGI_SURFACE_DESC,
        numsurfaces: u32,
        usage: DXGI_USAGE,
        psharedresource: Option<*const DXGI_SHARED_RESOURCE>,
        ppsurface: *mut Option<IDXGISurface>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateSurface)(
                windows_core::Interface::as_raw(self),
                pdesc,
                numsurfaces,
                usage,
                psharedresource.unwrap_or(core::mem::zeroed()) as _,
                core::mem::transmute(ppsurface),
            )
        }
    }
    pub(crate) unsafe fn QueryResourceResidency(
        &self,
        ppresources: *const Option<windows_core::IUnknown>,
        presidencystatus: *mut DXGI_RESIDENCY,
        numresources: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).QueryResourceResidency)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(ppresources),
                presidencystatus as _,
                numresources,
            )
        }
    }
    pub(crate) unsafe fn SetGPUThreadPriority(&self, priority: i32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetGPUThreadPriority)(
                windows_core::Interface::as_raw(self),
                priority,
            )
        }
    }
    pub(crate) unsafe fn GetGPUThreadPriority(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetGPUThreadPriority)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IDXGIDevice_Vtbl {
    pub base__: IDXGIObject_Vtbl,
    pub GetAdapter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSurface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const DXGI_SURFACE_DESC,
        u32,
        DXGI_USAGE,
        *const DXGI_SHARED_RESOURCE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub QueryResourceResidency: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const *mut core::ffi::c_void,
        *mut DXGI_RESIDENCY,
        u32,
    ) -> windows_core::HRESULT,
    pub SetGPUThreadPriority:
        unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub GetGPUThreadPriority:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDXGIDevice {}
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
impl IDXGISurface {
    pub(crate) unsafe fn GetDesc(&self, pdesc: *mut DXGI_SURFACE_DESC) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesc)(
                windows_core::Interface::as_raw(self),
                pdesc as _,
            )
        }
    }
    pub(crate) unsafe fn Map(
        &self,
        plockedrect: *mut DXGI_MAPPED_RECT,
        mapflags: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Map)(
                windows_core::Interface::as_raw(self),
                plockedrect as _,
                mapflags,
            )
        }
    }
    pub(crate) unsafe fn Unmap(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Unmap)(windows_core::Interface::as_raw(self))
        }
    }
}
#[repr(C)]
pub struct IDXGISurface_Vtbl {
    pub base__: IDXGIDeviceSubObject_Vtbl,
    pub GetDesc: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DXGI_SURFACE_DESC,
    ) -> windows_core::HRESULT,
    pub Map: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DXGI_MAPPED_RECT,
        u32,
    ) -> windows_core::HRESULT,
    pub Unmap: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IDXGISurface {}
windows_core::imp::define_interface!(
    IDirect3D11CaptureFrame,
    IDirect3D11CaptureFrame_Vtbl,
    0xfa50c623_38da_4b32_acf3_fa9734ad800e
);
impl windows_core::RuntimeType for IDirect3D11CaptureFrame {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IDirect3D11CaptureFrame",
    );
}
#[repr(C)]
pub struct IDirect3D11CaptureFrame_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Surface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    SystemRelativeTime: usize,
    pub ContentSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut SizeInt32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDirect3D11CaptureFrame2,
    IDirect3D11CaptureFrame2_Vtbl,
    0x37869cfa_2b48_5ebf_9afb_dffd805defdb
);
impl windows_core::RuntimeType for IDirect3D11CaptureFrame2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IDirect3D11CaptureFrame2",
    );
}
#[repr(C)]
pub struct IDirect3D11CaptureFrame2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub DirtyRegions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDirect3D11CaptureFramePool,
    IDirect3D11CaptureFramePool_Vtbl,
    0x24eb6d22_1975_422e_82e7_780dbd8ddf24
);
impl windows_core::RuntimeType for IDirect3D11CaptureFramePool {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IDirect3D11CaptureFramePool",
    );
}
#[repr(C)]
pub struct IDirect3D11CaptureFramePool_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Recreate: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        DirectXPixelFormat,
        i32,
        SizeInt32,
    ) -> windows_core::HRESULT,
    pub TryGetNextFrame: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    FrameArrived: usize,
    RemoveFrameArrived: usize,
    pub CreateCaptureSession: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDirect3D11CaptureFramePoolStatics,
    IDirect3D11CaptureFramePoolStatics_Vtbl,
    0x7784056a_67aa_4d53_ae54_1088d5a8ca21
);
impl windows_core::RuntimeType for IDirect3D11CaptureFramePoolStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IDirect3D11CaptureFramePoolStatics",
    );
}
#[repr(C)]
pub struct IDirect3D11CaptureFramePoolStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Create: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        DirectXPixelFormat,
        i32,
        SizeInt32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDirect3D11CaptureFramePoolStatics2,
    IDirect3D11CaptureFramePoolStatics2_Vtbl,
    0x589b103f_6bbc_5df5_a991_02e28b3b66d5
);
impl windows_core::RuntimeType for IDirect3D11CaptureFramePoolStatics2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IDirect3D11CaptureFramePoolStatics2",
    );
}
#[repr(C)]
pub struct IDirect3D11CaptureFramePoolStatics2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateFreeThreaded: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        DirectXPixelFormat,
        i32,
        SizeInt32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDirect3DDevice,
    IDirect3DDevice_Vtbl,
    0xa37624ab_8d5f_4650_9d3e_9eae3d9bc670
);
impl windows_core::RuntimeType for IDirect3DDevice {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.DirectX.Direct3D11.IDirect3DDevice",
    );
}
windows_core::imp::interface_hierarchy!(
    IDirect3DDevice,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(IDirect3DDevice, IClosable);
impl IDirect3DDevice {
    pub(crate) fn Trim(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).Trim)(windows_core::Interface::as_raw(self)).ok()
        }
    }
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
}
impl windows_core::RuntimeName for IDirect3DDevice {
    const NAME: &'static str = "Windows.Graphics.DirectX.Direct3D11.IDirect3DDevice";
}
pub trait IDirect3DDevice_Impl: IClosable_Impl {
    fn Trim(&self) -> windows_core::Result<()>;
}
impl IDirect3DDevice_Vtbl {
    pub const fn new<Identity: IDirect3DDevice_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Trim<Identity: IDirect3DDevice_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDirect3DDevice_Impl::Trim(this).into()
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IDirect3DDevice, OFFSET>(),
            Trim: Trim::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDirect3DDevice as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IDirect3DDevice_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Trim: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDirect3DDxgiInterfaceAccess,
    IDirect3DDxgiInterfaceAccess_Vtbl,
    0xa9b3d012_3df2_4ee3_b8d1_8695f457d3c1
);
windows_core::imp::interface_hierarchy!(IDirect3DDxgiInterfaceAccess, windows_core::IUnknown);
impl IDirect3DDxgiInterfaceAccess {
    pub(crate) unsafe fn GetInterface<T>(&self) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetInterface)(
                windows_core::Interface::as_raw(self),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDirect3DDxgiInterfaceAccess_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetInterface: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IDirect3DDxgiInterfaceAccess_Impl: windows_core::IUnknownImpl {
    fn GetInterface(
        &self,
        iid: *const windows_core::GUID,
        p: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
}
impl IDirect3DDxgiInterfaceAccess_Vtbl {
    pub const fn new<Identity: IDirect3DDxgiInterfaceAccess_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetInterface<
            Identity: IDirect3DDxgiInterfaceAccess_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            iid: *const windows_core::GUID,
            p: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDirect3DDxgiInterfaceAccess_Impl::GetInterface(
                    this,
                    core::mem::transmute_copy(&iid),
                    core::mem::transmute_copy(&p),
                )
                .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetInterface: GetInterface::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDirect3DDxgiInterfaceAccess as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDirect3DDxgiInterfaceAccess {}
windows_core::imp::define_interface!(
    IDirect3DSurface,
    IDirect3DSurface_Vtbl,
    0x0bf4a146_13c1_4694_bee3_7abf15eaf586
);
impl windows_core::RuntimeType for IDirect3DSurface {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.DirectX.Direct3D11.IDirect3DSurface",
    );
}
windows_core::imp::interface_hierarchy!(
    IDirect3DSurface,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(IDirect3DSurface, IClosable);
impl IDirect3DSurface {
    pub(crate) fn Description(&self) -> windows_core::Result<Direct3DSurfaceDescription> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Description)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
}
impl windows_core::RuntimeName for IDirect3DSurface {
    const NAME: &'static str = "Windows.Graphics.DirectX.Direct3D11.IDirect3DSurface";
}
pub trait IDirect3DSurface_Impl: IClosable_Impl {
    fn Description(&self) -> windows_core::Result<Direct3DSurfaceDescription>;
}
impl IDirect3DSurface_Vtbl {
    pub const fn new<Identity: IDirect3DSurface_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Description<
            Identity: IDirect3DSurface_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            result__: *mut Direct3DSurfaceDescription,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDirect3DSurface_Impl::Description(this) {
                    Ok(ok__) => {
                        result__.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<Identity, IDirect3DSurface, OFFSET>(),
            Description: Description::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDirect3DSurface as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IDirect3DSurface_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Description: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut Direct3DSurfaceDescription,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureAccessStatics,
    IGraphicsCaptureAccessStatics_Vtbl,
    0x743ed370_06ec_5040_a58a_901f0f757095
);
impl windows_core::RuntimeType for IGraphicsCaptureAccessStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureAccessStatics",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureAccessStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub RequestAccessAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GraphicsCaptureAccessKind,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureItem,
    IGraphicsCaptureItem_Vtbl,
    0x79c3f95b_31f7_4ec2_a464_632ef5d30760
);
impl windows_core::RuntimeType for IGraphicsCaptureItem {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureItem",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureItem_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    DisplayName: usize,
    pub Size:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut SizeInt32) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureItemInterop,
    IGraphicsCaptureItemInterop_Vtbl,
    0x3628e81b_3cac_4c60_b7f4_23ce0e0c3356
);
windows_core::imp::interface_hierarchy!(IGraphicsCaptureItemInterop, windows_core::IUnknown);
impl IGraphicsCaptureItemInterop {
    pub(crate) unsafe fn CreateForWindow<T>(&self, window: HWND) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateForWindow)(
                windows_core::Interface::as_raw(self),
                window,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateForMonitor<T>(&self, monitor: HMONITOR) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateForMonitor)(
                windows_core::Interface::as_raw(self),
                monitor,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IGraphicsCaptureItemInterop_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub CreateForWindow: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateForMonitor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HMONITOR,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IGraphicsCaptureItemInterop_Impl: windows_core::IUnknownImpl {
    fn CreateForWindow(
        &self,
        window: HWND,
        riid: *const windows_core::GUID,
        result: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn CreateForMonitor(
        &self,
        monitor: HMONITOR,
        riid: *const windows_core::GUID,
        result: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
}
impl IGraphicsCaptureItemInterop_Vtbl {
    pub const fn new<Identity: IGraphicsCaptureItemInterop_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CreateForWindow<
            Identity: IGraphicsCaptureItemInterop_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            window: HWND,
            riid: *const windows_core::GUID,
            result: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGraphicsCaptureItemInterop_Impl::CreateForWindow(
                    this,
                    core::mem::transmute_copy(&window),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&result),
                )
                .into()
            }
        }
        unsafe extern "system" fn CreateForMonitor<
            Identity: IGraphicsCaptureItemInterop_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            monitor: HMONITOR,
            riid: *const windows_core::GUID,
            result: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IGraphicsCaptureItemInterop_Impl::CreateForMonitor(
                    this,
                    core::mem::transmute_copy(&monitor),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&result),
                )
                .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            CreateForWindow: CreateForWindow::<Identity, OFFSET>,
            CreateForMonitor: CreateForMonitor::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IGraphicsCaptureItemInterop as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IGraphicsCaptureItemInterop {}
windows_core::imp::define_interface!(
    IGraphicsCaptureItemStatics,
    IGraphicsCaptureItemStatics_Vtbl,
    0xa87ebea5_457c_5788_ab47_0cf1d3637e74
);
impl windows_core::RuntimeType for IGraphicsCaptureItemStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureItemStatics",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureItemStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateFromVisual: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureItemStatics2,
    IGraphicsCaptureItemStatics2_Vtbl,
    0x3b92acc9_e584_5862_bf5c_9c316c6d2dbb
);
impl windows_core::RuntimeType for IGraphicsCaptureItemStatics2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureItemStatics2",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureItemStatics2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub TryCreateFromWindowId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        WindowId,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub TryCreateFromDisplayId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DisplayId,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureSession,
    IGraphicsCaptureSession_Vtbl,
    0x814e42a9_f70f_4ad7_939b_fddcc6eb880d
);
impl windows_core::RuntimeType for IGraphicsCaptureSession {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureSession",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureSession_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub StartCapture: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureSession2,
    IGraphicsCaptureSession2_Vtbl,
    0x2c39ae40_7d2e_5044_804e_8b6799d4cf9e
);
impl windows_core::RuntimeType for IGraphicsCaptureSession2 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureSession2",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureSession2_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsCursorCaptureEnabled:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsCursorCaptureEnabled:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureSession3,
    IGraphicsCaptureSession3_Vtbl,
    0xf2cdd966_22ae_5ea1_9596_3a289344c3be
);
impl windows_core::RuntimeType for IGraphicsCaptureSession3 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureSession3",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureSession3_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsBorderRequired:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub SetIsBorderRequired:
        unsafe extern "system" fn(*mut core::ffi::c_void, bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGraphicsCaptureSessionStatics,
    IGraphicsCaptureSessionStatics_Vtbl,
    0x2224a540_5974_49aa_b232_0882536f4cb5
);
impl windows_core::RuntimeType for IGraphicsCaptureSessionStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureSessionStatics",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureSessionStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub IsSupported:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IVisual,
    IVisual_Vtbl,
    0x117e202d_a859_4c89_873b_c2aa566788e3
);
impl windows_core::RuntimeType for IVisual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.Composition.IVisual");
}
#[repr(C)]
pub struct IVisual_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RectInt32 {
    pub X: i32,
    pub Y: i32,
    pub Width: i32,
    pub Height: i32,
}
impl windows_core::imp::TypeKind for RectInt32 {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for RectInt32 {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"struct(Windows.Graphics.RectInt32;i4;i4;i4;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Graphics.RectInt32");
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SizeInt32 {
    pub Width: i32,
    pub Height: i32,
}
impl windows_core::imp::TypeKind for SizeInt32 {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for SizeInt32 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Graphics.SizeInt32;i4;i4)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Graphics.SizeInt32");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Visual(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(Visual, windows_core::IUnknown, windows_core::IInspectable);
windows_core::imp::required_hierarchy!(Visual, CompositionObject);
impl Visual {
    pub(crate) fn Close(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IClosable>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).Close)(windows_core::Interface::as_raw(this))
                .ok()
        }
    }
}
impl windows_core::RuntimeType for Visual {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisual>();
}
unsafe impl windows_core::Interface for Visual {
    type Vtable = <IVisual as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisual as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for Visual {
    const NAME: &'static str = "Windows.UI.Composition.Visual";
}
unsafe impl Send for Visual {}
unsafe impl Sync for Visual {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WindowId {
    pub Value: u64,
}
impl windows_core::imp::TypeKind for WindowId {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for WindowId {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.UI.WindowId;u8)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.UI.WindowId");
}
