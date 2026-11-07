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
pub type D3D11_BIND_FLAG = i32;
pub const D3D11_BIND_RENDER_TARGET: D3D11_BIND_FLAG = 32;
pub const D3D11_BIND_SHADER_RESOURCE: D3D11_BIND_FLAG = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D11_BOX {
    pub left: u32,
    pub top: u32,
    pub front: u32,
    pub right: u32,
    pub bottom: u32,
    pub back: u32,
}
pub type D3D11_CPU_ACCESS_FLAG = i32;
pub const D3D11_CPU_ACCESS_READ: D3D11_CPU_ACCESS_FLAG = 131072;
pub type D3D11_MAP = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D11_MAPPED_SUBRESOURCE {
    pub pData: *mut core::ffi::c_void,
    pub RowPitch: u32,
    pub DepthPitch: u32,
}
pub type D3D11_MAP_FLAG = i32;
pub const D3D11_MAP_FLAG_DO_NOT_WAIT: D3D11_MAP_FLAG = 1048576;
pub const D3D11_MAP_READ: D3D11_MAP = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D11_SUBRESOURCE_DATA {
    pub pSysMem: *const core::ffi::c_void,
    pub SysMemPitch: u32,
    pub SysMemSlicePitch: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D3D11_TEXTURE2D_DESC {
    pub Width: u32,
    pub Height: u32,
    pub MipLevels: u32,
    pub ArraySize: u32,
    pub Format: DXGI_FORMAT,
    pub SampleDesc: DXGI_SAMPLE_DESC,
    pub Usage: D3D11_USAGE,
    pub BindFlags: u32,
    pub CPUAccessFlags: u32,
    pub MiscFlags: u32,
}
pub type D3D11_USAGE = i32;
pub const D3D11_USAGE_DEFAULT: D3D11_USAGE = 0;
pub const D3D11_USAGE_STAGING: D3D11_USAGE = 3;
pub type DXGI_FORMAT = i32;
pub const DXGI_FORMAT_B8G8R8A8_UNORM: DXGI_FORMAT = 87;
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
    pub(crate) fn SystemRelativeTime(&self) -> windows_core::Result<TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SystemRelativeTime)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
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
    pub(crate) fn FrameArrived<F>(
        &self,
        handler: F,
    ) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<Self>, windows_core::Ref<windows_core::IInspectable>)
            + Send
            + 'static,
    {
        let handler = <TypedEventHandler<Self, windows_core::IInspectable>>::new(move |a0, a1| {
            handler(a0, a1);
            Ok(())
        });
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).FrameArrived)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveFrameArrived,
            ))
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
    pub(crate) fn DispatcherQueue(&self) -> windows_core::Result<DispatcherQueue> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DispatcherQueue)(
                windows_core::Interface::as_raw(self),
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
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatcherQueue(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DispatcherQueue,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl DispatcherQueue {
    pub(crate) fn GetForCurrentThread() -> windows_core::Result<Self> {
        Self::IDispatcherQueueStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetForCurrentThread)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IDispatcherQueueStatics<
        R,
        F: FnOnce(&IDispatcherQueueStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<DispatcherQueue, IDispatcherQueueStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for DispatcherQueue {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDispatcherQueue>();
}
unsafe impl windows_core::Interface for DispatcherQueue {
    type Vtable = <IDispatcherQueue as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDispatcherQueue as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for DispatcherQueue {
    const NAME: &'static str = "Windows.System.DispatcherQueue";
}
unsafe impl Send for DispatcherQueue {}
unsafe impl Sync for DispatcherQueue {}
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
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphicsCaptureItem(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GraphicsCaptureItem,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GraphicsCaptureItem {
    pub(crate) fn DisplayName(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DisplayName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
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
    pub(crate) fn Closed<F>(&self, handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(windows_core::Ref<Self>, windows_core::Ref<windows_core::IInspectable>)
            + Send
            + 'static,
    {
        let handler = <TypedEventHandler<Self, windows_core::IInspectable>>::new(move |a0, a1| {
            handler(a0, a1);
            Ok(())
        });
        unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(self).Closed)(
                windows_core::Interface::as_raw(self),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                self.clone(),
                token__,
                windows_core::Interface::vtable(self).RemoveClosed,
            ))
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
    pub(crate) fn MinUpdateInterval(&self) -> windows_core::Result<TimeSpan> {
        let this = &windows_core::Interface::cast::<IGraphicsCaptureSession5>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).MinUpdateInterval)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetMinUpdateInterval(&self, value: TimeSpan) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IGraphicsCaptureSession5>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetMinUpdateInterval)(
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
    ID3D11Device,
    ID3D11Device_Vtbl,
    0xdb6f6ddb_ac77_4e88_8253_819df9bbf140
);
windows_core::imp::interface_hierarchy!(ID3D11Device, windows_core::IUnknown);
impl ID3D11Device {
    pub(crate) unsafe fn CreateTexture2D(
        &self,
        pdesc: *const D3D11_TEXTURE2D_DESC,
        pinitialdata: Option<*const D3D11_SUBRESOURCE_DATA>,
        pptexture2d: Option<*mut Option<ID3D11Texture2D>>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateTexture2D)(
                windows_core::Interface::as_raw(self),
                pdesc,
                pinitialdata.unwrap_or(core::mem::zeroed()) as _,
                pptexture2d.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn GetImmediateContext(&self) -> windows_core::Result<ID3D11DeviceContext> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetImmediateContext)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
}
#[repr(C)]
pub struct ID3D11Device_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    CreateBuffer: usize,
    CreateTexture1D: usize,
    pub CreateTexture2D: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D3D11_TEXTURE2D_DESC,
        *const D3D11_SUBRESOURCE_DATA,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    CreateTexture3D: usize,
    CreateShaderResourceView: usize,
    CreateUnorderedAccessView: usize,
    CreateRenderTargetView: usize,
    CreateDepthStencilView: usize,
    CreateInputLayout: usize,
    CreateVertexShader: usize,
    CreateGeometryShader: usize,
    CreateGeometryShaderWithStreamOutput: usize,
    CreatePixelShader: usize,
    CreateHullShader: usize,
    CreateDomainShader: usize,
    CreateComputeShader: usize,
    CreateClassLinkage: usize,
    CreateBlendState: usize,
    CreateDepthStencilState: usize,
    CreateRasterizerState: usize,
    CreateSamplerState: usize,
    CreateQuery: usize,
    CreatePredicate: usize,
    CreateCounter: usize,
    CreateDeferredContext: usize,
    OpenSharedResource: usize,
    CheckFormatSupport: usize,
    CheckMultisampleQualityLevels: usize,
    CheckCounterInfo: usize,
    CheckCounter: usize,
    CheckFeatureSupport: usize,
    GetPrivateData: usize,
    SetPrivateData: usize,
    SetPrivateDataInterface: usize,
    GetFeatureLevel: usize,
    GetCreationFlags: usize,
    GetDeviceRemovedReason: usize,
    pub GetImmediateContext:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    SetExceptionMode: usize,
    GetExceptionMode: usize,
}
impl windows_core::RuntimeName for ID3D11Device {}
windows_core::imp::define_interface!(
    ID3D11DeviceChild,
    ID3D11DeviceChild_Vtbl,
    0x1841e5c8_16b0_489b_bcc8_44cfb0d5deae
);
windows_core::imp::interface_hierarchy!(ID3D11DeviceChild, windows_core::IUnknown);
#[repr(C)]
pub struct ID3D11DeviceChild_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetDevice: usize,
    GetPrivateData: usize,
    SetPrivateData: usize,
    SetPrivateDataInterface: usize,
}
impl windows_core::RuntimeName for ID3D11DeviceChild {}
windows_core::imp::define_interface!(
    ID3D11DeviceContext,
    ID3D11DeviceContext_Vtbl,
    0xc0bfa96c_e089_44fb_8eaf_26f8796190da
);
impl core::ops::Deref for ID3D11DeviceContext {
    type Target = ID3D11DeviceChild;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D11DeviceContext,
    windows_core::IUnknown,
    ID3D11DeviceChild
);
impl ID3D11DeviceContext {
    pub(crate) unsafe fn Map<P0>(
        &self,
        presource: P0,
        subresource: u32,
        maptype: D3D11_MAP,
        mapflags: u32,
        pmappedresource: Option<*mut D3D11_MAPPED_SUBRESOURCE>,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<ID3D11Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Map)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                subresource,
                maptype,
                mapflags,
                pmappedresource.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn Unmap<P0>(&self, presource: P0, subresource: u32)
    where
        P0: windows_core::Param<ID3D11Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Unmap)(
                windows_core::Interface::as_raw(self),
                presource.param().abi(),
                subresource,
            );
        }
    }
    pub(crate) unsafe fn CopySubresourceRegion<P0, P5>(
        &self,
        pdstresource: P0,
        dstsubresource: u32,
        dstx: u32,
        dsty: u32,
        dstz: u32,
        psrcresource: P5,
        srcsubresource: u32,
        psrcbox: Option<*const D3D11_BOX>,
    ) where
        P0: windows_core::Param<ID3D11Resource>,
        P5: windows_core::Param<ID3D11Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopySubresourceRegion)(
                windows_core::Interface::as_raw(self),
                pdstresource.param().abi(),
                dstsubresource,
                dstx,
                dsty,
                dstz,
                psrcresource.param().abi(),
                srcsubresource,
                psrcbox.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn CopyResource<P0, P1>(&self, pdstresource: P0, psrcresource: P1)
    where
        P0: windows_core::Param<ID3D11Resource>,
        P1: windows_core::Param<ID3D11Resource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyResource)(
                windows_core::Interface::as_raw(self),
                pdstresource.param().abi(),
                psrcresource.param().abi(),
            );
        }
    }
}
#[repr(C)]
pub struct ID3D11DeviceContext_Vtbl {
    pub base__: ID3D11DeviceChild_Vtbl,
    VSSetConstantBuffers: usize,
    PSSetShaderResources: usize,
    PSSetShader: usize,
    PSSetSamplers: usize,
    VSSetShader: usize,
    DrawIndexed: usize,
    Draw: usize,
    pub Map: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        D3D11_MAP,
        u32,
        *mut D3D11_MAPPED_SUBRESOURCE,
    ) -> windows_core::HRESULT,
    pub Unmap: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, u32),
    PSSetConstantBuffers: usize,
    IASetInputLayout: usize,
    IASetVertexBuffers: usize,
    IASetIndexBuffer: usize,
    DrawIndexedInstanced: usize,
    DrawInstanced: usize,
    GSSetConstantBuffers: usize,
    GSSetShader: usize,
    IASetPrimitiveTopology: usize,
    VSSetShaderResources: usize,
    VSSetSamplers: usize,
    Begin: usize,
    End: usize,
    GetData: usize,
    SetPredication: usize,
    GSSetShaderResources: usize,
    GSSetSamplers: usize,
    OMSetRenderTargets: usize,
    OMSetRenderTargetsAndUnorderedAccessViews: usize,
    OMSetBlendState: usize,
    OMSetDepthStencilState: usize,
    SOSetTargets: usize,
    DrawAuto: usize,
    DrawIndexedInstancedIndirect: usize,
    DrawInstancedIndirect: usize,
    Dispatch: usize,
    DispatchIndirect: usize,
    RSSetState: usize,
    RSSetViewports: usize,
    RSSetScissorRects: usize,
    pub CopySubresourceRegion: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        u32,
        u32,
        u32,
        *mut core::ffi::c_void,
        u32,
        *const D3D11_BOX,
    ),
    pub CopyResource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ),
    UpdateSubresource: usize,
    CopyStructureCount: usize,
    ClearRenderTargetView: usize,
    ClearUnorderedAccessViewUint: usize,
    ClearUnorderedAccessViewFloat: usize,
    ClearDepthStencilView: usize,
    GenerateMips: usize,
    SetResourceMinLOD: usize,
    GetResourceMinLOD: usize,
    ResolveSubresource: usize,
    ExecuteCommandList: usize,
    HSSetShaderResources: usize,
    HSSetShader: usize,
    HSSetSamplers: usize,
    HSSetConstantBuffers: usize,
    DSSetShaderResources: usize,
    DSSetShader: usize,
    DSSetSamplers: usize,
    DSSetConstantBuffers: usize,
    CSSetShaderResources: usize,
    CSSetUnorderedAccessViews: usize,
    CSSetShader: usize,
    CSSetSamplers: usize,
    CSSetConstantBuffers: usize,
    VSGetConstantBuffers: usize,
    PSGetShaderResources: usize,
    PSGetShader: usize,
    PSGetSamplers: usize,
    VSGetShader: usize,
    PSGetConstantBuffers: usize,
    IAGetInputLayout: usize,
    IAGetVertexBuffers: usize,
    IAGetIndexBuffer: usize,
    GSGetConstantBuffers: usize,
    GSGetShader: usize,
    IAGetPrimitiveTopology: usize,
    VSGetShaderResources: usize,
    VSGetSamplers: usize,
    GetPredication: usize,
    GSGetShaderResources: usize,
    GSGetSamplers: usize,
    OMGetRenderTargets: usize,
    OMGetRenderTargetsAndUnorderedAccessViews: usize,
    OMGetBlendState: usize,
    OMGetDepthStencilState: usize,
    SOGetTargets: usize,
    RSGetState: usize,
    RSGetViewports: usize,
    RSGetScissorRects: usize,
    HSGetShaderResources: usize,
    HSGetShader: usize,
    HSGetSamplers: usize,
    HSGetConstantBuffers: usize,
    DSGetShaderResources: usize,
    DSGetShader: usize,
    DSGetSamplers: usize,
    DSGetConstantBuffers: usize,
    CSGetShaderResources: usize,
    CSGetUnorderedAccessViews: usize,
    CSGetShader: usize,
    CSGetSamplers: usize,
    CSGetConstantBuffers: usize,
    ClearState: usize,
    Flush: usize,
    GetType: usize,
    GetContextFlags: usize,
    FinishCommandList: usize,
}
impl windows_core::RuntimeName for ID3D11DeviceContext {}
windows_core::imp::define_interface!(
    ID3D11Resource,
    ID3D11Resource_Vtbl,
    0xdc8e63f3_d12b_4952_b47b_5e45026a862d
);
impl core::ops::Deref for ID3D11Resource {
    type Target = ID3D11DeviceChild;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID3D11Resource, windows_core::IUnknown, ID3D11DeviceChild);
#[repr(C)]
pub struct ID3D11Resource_Vtbl {
    pub base__: ID3D11DeviceChild_Vtbl,
    GetType: usize,
    SetEvictionPriority: usize,
    GetEvictionPriority: usize,
}
impl windows_core::RuntimeName for ID3D11Resource {}
windows_core::imp::define_interface!(
    ID3D11Texture2D,
    ID3D11Texture2D_Vtbl,
    0x6f15aaf2_d208_4e89_9ab4_489535d34f9c
);
impl core::ops::Deref for ID3D11Texture2D {
    type Target = ID3D11Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID3D11Texture2D,
    windows_core::IUnknown,
    ID3D11DeviceChild,
    ID3D11Resource
);
impl ID3D11Texture2D {
    pub(crate) unsafe fn GetDesc(&self, pdesc: *mut D3D11_TEXTURE2D_DESC) {
        unsafe {
            (windows_core::Interface::vtable(self).GetDesc)(
                windows_core::Interface::as_raw(self),
                pdesc as _,
            );
        }
    }
}
#[repr(C)]
pub struct ID3D11Texture2D_Vtbl {
    pub base__: ID3D11Resource_Vtbl,
    pub GetDesc: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D3D11_TEXTURE2D_DESC),
}
impl windows_core::RuntimeName for ID3D11Texture2D {}
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
    pub SystemRelativeTime:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut TimeSpan) -> windows_core::HRESULT,
    pub ContentSize:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut SizeInt32) -> windows_core::HRESULT,
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
    pub FrameArrived: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut i64,
    ) -> windows_core::HRESULT,
    pub RemoveFrameArrived:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    pub CreateCaptureSession: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DispatcherQueue: unsafe extern "system" fn(
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
    IDispatcherQueue,
    IDispatcherQueue_Vtbl,
    0x603e88e4_a338_4ffe_a457_a5cfb9ceb899
);
impl windows_core::RuntimeType for IDispatcherQueue {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.System.IDispatcherQueue");
}
#[repr(C)]
pub struct IDispatcherQueue_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDispatcherQueueStatics,
    IDispatcherQueueStatics_Vtbl,
    0xa96d83d7_9371_4517_9245_d0824ac12c74
);
impl windows_core::RuntimeType for IDispatcherQueueStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.System.IDispatcherQueueStatics");
}
#[repr(C)]
pub struct IDispatcherQueueStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub GetForCurrentThread: unsafe extern "system" fn(
        *mut core::ffi::c_void,
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
    pub DisplayName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Size:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut SizeInt32) -> windows_core::HRESULT,
    pub Closed: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut i64,
    ) -> windows_core::HRESULT,
    pub RemoveClosed:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
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
    IGraphicsCaptureSession5,
    IGraphicsCaptureSession5_Vtbl,
    0x67c0ea62_1f85_5061_925a_239be0ac09cb
);
impl windows_core::RuntimeType for IGraphicsCaptureSession5 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Graphics.Capture.IGraphicsCaptureSession5",
    );
}
#[repr(C)]
pub struct IGraphicsCaptureSession5_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub MinUpdateInterval:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut TimeSpan) -> windows_core::HRESULT,
    pub SetMinUpdateInterval:
        unsafe extern "system" fn(*mut core::ffi::c_void, TimeSpan) -> windows_core::HRESULT,
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
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TimeSpan {
    pub Duration: i64,
}
impl windows_core::imp::TypeKind for TimeSpan {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for TimeSpan {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.TimeSpan;i8)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Foundation.TimeSpan");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedEventHandler<TSender, TResult>(
    windows_core::IUnknown,
    core::marker::PhantomData<TSender>,
    core::marker::PhantomData<TResult>,
)
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static;
unsafe impl<
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
> windows_core::Interface for TypedEventHandler<TSender, TResult>
{
    type Vtable = TypedEventHandler_Vtbl<TSender, TResult>;
    const IID: windows_core::GUID =
        windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    windows_core::RuntimeType for TypedEventHandler<TSender, TResult>
{
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({9de1c534-6ae1-11e0-84e1-18a905bcc53f}")
        .push_slice(b";")
        .push_other(TSender::SIGNATURE)
        .push_slice(b";")
        .push_other(TResult::SIGNATURE)
        .push_slice(b")");
}
impl<TSender: windows_core::RuntimeType + 'static, TResult: windows_core::RuntimeType + 'static>
    TypedEventHandler<TSender, TResult>
{
    pub(crate) fn new<
        F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()>
            + Send
            + 'static,
    >(
        invoke: F,
    ) -> Self {
        let com = windows_core::imp::DelegateBox::<Self, F>::new(
            &TypedEventHandlerBox::<TSender, TResult, F>::VTABLE,
            invoke,
        );
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
    pub(crate) fn Invoke<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<TSender>,
        P1: windows_core::Param<TResult>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Invoke)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct TypedEventHandler_Vtbl<TSender, TResult>
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
{
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: windows_core::imp::AbiType<TSender>,
        args: windows_core::imp::AbiType<TResult>,
    ) -> windows_core::HRESULT,
    TSender: core::marker::PhantomData<TSender>,
    TResult: core::marker::PhantomData<TResult>,
}
struct TypedEventHandlerBox<
    TSender,
    TResult,
    F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()>
        + Send
        + 'static,
>(core::marker::PhantomData<(TSender, TResult, fn() -> F)>)
where
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static;
impl<
    TSender: windows_core::RuntimeType + 'static,
    TResult: windows_core::RuntimeType + 'static,
    F: Fn(windows_core::Ref<TSender>, windows_core::Ref<TResult>) -> windows_core::Result<()>
        + Send
        + 'static,
> TypedEventHandlerBox<TSender, TResult, F>
{
    const VTABLE : TypedEventHandler_Vtbl < TSender , TResult , > = TypedEventHandler_Vtbl::< TSender , TResult , > { base__ : windows_core::IUnknown_Vtbl { QueryInterface : windows_core::imp::DelegateBox::< TypedEventHandler < TSender , TResult > , F >::QueryInterface , AddRef : windows_core::imp::DelegateBox::< TypedEventHandler < TSender , TResult > , F >::AddRef , Release : windows_core::imp::DelegateBox::< TypedEventHandler < TSender , TResult > , F >::Release , } , Invoke : Self::Invoke , TSender : core::marker::PhantomData::< TSender > , TResult : core::marker::PhantomData::< TResult > } ;
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: windows_core::imp::AbiType<TSender>,
        args: windows_core::imp::AbiType<TResult>,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &mut *(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<TypedEventHandler<TSender, TResult>, F>);
            (this.invoke)(
                core::mem::transmute_copy(&sender),
                core::mem::transmute_copy(&args),
            )
            .into()
        }
    }
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
