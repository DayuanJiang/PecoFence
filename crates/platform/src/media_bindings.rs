#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataReader(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DataReader,
    windows_core::IUnknown,
    windows_core::IInspectable,
    IDataReader
);
windows_core::imp::required_hierarchy!(DataReader, IClosable);
impl DataReader {
    pub(crate) fn ReadBytes(&self, value: &mut [u8]) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ReadBytes)(
                windows_core::Interface::as_raw(self),
                value.len().try_into().unwrap(),
                value.as_mut_ptr(),
            )
            .ok()
        }
    }
    pub(crate) fn LoadAsync(&self, count: u32) -> windows_core::Result<DataReaderLoadOperation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LoadAsync)(
                windows_core::Interface::as_raw(self),
                count,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn CreateDataReader<P0>(inputstream: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IInputStream>,
    {
        Self::IDataReaderFactory(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreateDataReader)(
                windows_core::Interface::as_raw(this),
                inputstream.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    pub(crate) fn FromBuffer<P0>(buffer: P0) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IBuffer>,
    {
        Self::IDataReaderStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).FromBuffer)(
                windows_core::Interface::as_raw(this),
                buffer.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IDataReaderFactory<R, F: FnOnce(&IDataReaderFactory) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<DataReader, IDataReaderFactory> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    fn IDataReaderStatics<R, F: FnOnce(&IDataReaderStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<DataReader, IDataReaderStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for DataReader {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDataReader>();
}
unsafe impl windows_core::Interface for DataReader {
    type Vtable = <IDataReader as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDataReader as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for DataReader {
    const NAME: &'static str = "Windows.Storage.Streams.DataReader";
}
unsafe impl Send for DataReader {}
unsafe impl Sync for DataReader {}
pub type DataReaderLoadOperation = windows_future::IAsyncOperation<u32>;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSystemMediaTransportControlsSession(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GlobalSystemMediaTransportControlsSession,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GlobalSystemMediaTransportControlsSession {
    pub(crate) fn SourceAppUserModelId(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SourceAppUserModelId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn TryGetMediaPropertiesAsync(
        &self,
    ) -> windows_core::Result<
        windows_future::IAsyncOperation<GlobalSystemMediaTransportControlsSessionMediaProperties>,
    > {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryGetMediaPropertiesAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn GetTimelineProperties(
        &self,
    ) -> windows_core::Result<GlobalSystemMediaTransportControlsSessionTimelineProperties> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTimelineProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn GetPlaybackInfo(
        &self,
    ) -> windows_core::Result<GlobalSystemMediaTransportControlsSessionPlaybackInfo> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPlaybackInfo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn TrySkipNextAsync(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<bool>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TrySkipNextAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn TrySkipPreviousAsync(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<bool>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TrySkipPreviousAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn TryTogglePlayPauseAsync(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<bool>> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryTogglePlayPauseAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for GlobalSystemMediaTransportControlsSession {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IGlobalSystemMediaTransportControlsSession,
    >();
}
unsafe impl windows_core::Interface for GlobalSystemMediaTransportControlsSession {
    type Vtable = <IGlobalSystemMediaTransportControlsSession as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IGlobalSystemMediaTransportControlsSession as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for GlobalSystemMediaTransportControlsSession {
    const NAME: &'static str = "Windows.Media.Control.GlobalSystemMediaTransportControlsSession";
}
unsafe impl Send for GlobalSystemMediaTransportControlsSession {}
unsafe impl Sync for GlobalSystemMediaTransportControlsSession {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSystemMediaTransportControlsSessionManager(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GlobalSystemMediaTransportControlsSessionManager,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GlobalSystemMediaTransportControlsSessionManager {
    pub(crate) fn GetCurrentSession(
        &self,
    ) -> windows_core::Result<GlobalSystemMediaTransportControlsSession> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentSession)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn RequestAsync() -> windows_core::Result<windows_future::IAsyncOperation<Self>> {
        Self::IGlobalSystemMediaTransportControlsSessionManagerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RequestAsync)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IGlobalSystemMediaTransportControlsSessionManagerStatics<
        R,
        F: FnOnce(
            &IGlobalSystemMediaTransportControlsSessionManagerStatics,
        ) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            GlobalSystemMediaTransportControlsSessionManager,
            IGlobalSystemMediaTransportControlsSessionManagerStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for GlobalSystemMediaTransportControlsSessionManager {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IGlobalSystemMediaTransportControlsSessionManager,
    >();
}
unsafe impl windows_core::Interface for GlobalSystemMediaTransportControlsSessionManager {
    type Vtable =
        <IGlobalSystemMediaTransportControlsSessionManager as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IGlobalSystemMediaTransportControlsSessionManager as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for GlobalSystemMediaTransportControlsSessionManager {
    const NAME: &'static str =
        "Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager";
}
unsafe impl Send for GlobalSystemMediaTransportControlsSessionManager {}
unsafe impl Sync for GlobalSystemMediaTransportControlsSessionManager {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSystemMediaTransportControlsSessionMediaProperties(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GlobalSystemMediaTransportControlsSessionMediaProperties,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GlobalSystemMediaTransportControlsSessionMediaProperties {
    pub(crate) fn Title(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Title)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn Artist(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Artist)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn AlbumTitle(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).AlbumTitle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn Thumbnail(&self) -> windows_core::Result<IRandomAccessStreamReference> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Thumbnail)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for GlobalSystemMediaTransportControlsSessionMediaProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IGlobalSystemMediaTransportControlsSessionMediaProperties,
    >();
}
unsafe impl windows_core::Interface for GlobalSystemMediaTransportControlsSessionMediaProperties {
    type Vtable = < IGlobalSystemMediaTransportControlsSessionMediaProperties as windows_core::Interface >::Vtable ;
    const IID: windows_core::GUID =
        <IGlobalSystemMediaTransportControlsSessionMediaProperties as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for GlobalSystemMediaTransportControlsSessionMediaProperties {
    const NAME: &'static str =
        "Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties";
}
unsafe impl Send for GlobalSystemMediaTransportControlsSessionMediaProperties {}
unsafe impl Sync for GlobalSystemMediaTransportControlsSessionMediaProperties {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSystemMediaTransportControlsSessionPlaybackInfo(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GlobalSystemMediaTransportControlsSessionPlaybackInfo,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GlobalSystemMediaTransportControlsSessionPlaybackInfo {
    pub(crate) fn PlaybackStatus(
        &self,
    ) -> windows_core::Result<GlobalSystemMediaTransportControlsSessionPlaybackStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).PlaybackStatus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
impl windows_core::RuntimeType for GlobalSystemMediaTransportControlsSessionPlaybackInfo {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IGlobalSystemMediaTransportControlsSessionPlaybackInfo,
    >();
}
unsafe impl windows_core::Interface for GlobalSystemMediaTransportControlsSessionPlaybackInfo {
    type Vtable =
        <IGlobalSystemMediaTransportControlsSessionPlaybackInfo as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IGlobalSystemMediaTransportControlsSessionPlaybackInfo as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for GlobalSystemMediaTransportControlsSessionPlaybackInfo {
    const NAME: &'static str =
        "Windows.Media.Control.GlobalSystemMediaTransportControlsSessionPlaybackInfo";
}
unsafe impl Send for GlobalSystemMediaTransportControlsSessionPlaybackInfo {}
unsafe impl Sync for GlobalSystemMediaTransportControlsSessionPlaybackInfo {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GlobalSystemMediaTransportControlsSessionPlaybackStatus(pub i32);
impl GlobalSystemMediaTransportControlsSessionPlaybackStatus {
    pub const Closed: Self = Self(0);
    pub const Opened: Self = Self(1);
    pub const Changing: Self = Self(2);
    pub const Stopped: Self = Self(3);
    pub const Playing: Self = Self(4);
    pub const Paused: Self = Self(5);
}
impl windows_core::imp::TypeKind for GlobalSystemMediaTransportControlsSessionPlaybackStatus {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for GlobalSystemMediaTransportControlsSessionPlaybackStatus {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Media.Control.GlobalSystemMediaTransportControlsSessionPlaybackStatus;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.GlobalSystemMediaTransportControlsSessionPlaybackStatus",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalSystemMediaTransportControlsSessionTimelineProperties(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    GlobalSystemMediaTransportControlsSessionTimelineProperties,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl GlobalSystemMediaTransportControlsSessionTimelineProperties {
    pub(crate) fn StartTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).StartTime)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn EndTime(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).EndTime)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn Position(&self) -> windows_core::Result<windows_time::TimeSpan> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Position)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn LastUpdatedTime(&self) -> windows_core::Result<windows_time::DateTime> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LastUpdatedTime)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
impl windows_core::RuntimeType for GlobalSystemMediaTransportControlsSessionTimelineProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::for_class::<
        Self,
        IGlobalSystemMediaTransportControlsSessionTimelineProperties,
    >();
}
unsafe impl windows_core::Interface
    for GlobalSystemMediaTransportControlsSessionTimelineProperties
{
    type Vtable = < IGlobalSystemMediaTransportControlsSessionTimelineProperties as windows_core::Interface >::Vtable ;
    const IID : windows_core::GUID = < IGlobalSystemMediaTransportControlsSessionTimelineProperties as windows_core::Interface >::IID ;
}
impl windows_core::RuntimeName for GlobalSystemMediaTransportControlsSessionTimelineProperties {
    const NAME: &'static str =
        "Windows.Media.Control.GlobalSystemMediaTransportControlsSessionTimelineProperties";
}
unsafe impl Send for GlobalSystemMediaTransportControlsSessionTimelineProperties {}
unsafe impl Sync for GlobalSystemMediaTransportControlsSessionTimelineProperties {}
windows_core::imp::define_interface!(
    IBuffer,
    IBuffer_Vtbl,
    0x905a0fe0_bc53_11df_8c49_001e4fc686da
);
impl windows_core::RuntimeType for IBuffer {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IBuffer");
}
windows_core::imp::interface_hierarchy!(
    IBuffer,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IBuffer {
    const NAME: &'static str = "Windows.Storage.Streams.IBuffer";
}
#[repr(C)]
pub struct IBuffer_Vtbl {
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
impl windows_core::RuntimeName for IClosable {
    const NAME: &'static str = "Windows.Foundation.IClosable";
}
#[repr(C)]
pub struct IClosable_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IContentTypeProvider,
    IContentTypeProvider_Vtbl,
    0x97d098a5_3b99_4de9_88a5_e11d2f50c795
);
impl windows_core::RuntimeType for IContentTypeProvider {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IContentTypeProvider");
}
windows_core::imp::interface_hierarchy!(
    IContentTypeProvider,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IContentTypeProvider {
    const NAME: &'static str = "Windows.Storage.Streams.IContentTypeProvider";
}
#[repr(C)]
pub struct IContentTypeProvider_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDataReader,
    IDataReader_Vtbl,
    0xe2b50029_b4c1_4314_a4b8_fb813a2f275e
);
impl windows_core::RuntimeType for IDataReader {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IDataReader");
}
windows_core::imp::interface_hierarchy!(
    IDataReader,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl IDataReader {
    pub(crate) fn ReadBytes(&self, value: &mut [u8]) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).ReadBytes)(
                windows_core::Interface::as_raw(self),
                value.len().try_into().unwrap(),
                value.as_mut_ptr(),
            )
            .ok()
        }
    }
    pub(crate) fn LoadAsync(&self, count: u32) -> windows_core::Result<DataReaderLoadOperation> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).LoadAsync)(
                windows_core::Interface::as_raw(self),
                count,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeName for IDataReader {
    const NAME: &'static str = "Windows.Storage.Streams.IDataReader";
}
#[repr(C)]
pub struct IDataReader_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    UnconsumedBufferLength: usize,
    UnicodeEncoding: usize,
    SetUnicodeEncoding: usize,
    ByteOrder: usize,
    SetByteOrder: usize,
    InputStreamOptions: usize,
    SetInputStreamOptions: usize,
    ReadByte: usize,
    pub ReadBytes:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32, *mut u8) -> windows_core::HRESULT,
    ReadBuffer: usize,
    ReadBoolean: usize,
    ReadGuid: usize,
    ReadInt16: usize,
    ReadInt32: usize,
    ReadInt64: usize,
    ReadUInt16: usize,
    ReadUInt32: usize,
    ReadUInt64: usize,
    ReadSingle: usize,
    ReadDouble: usize,
    ReadString: usize,
    ReadDateTime: usize,
    ReadTimeSpan: usize,
    pub LoadAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDataReaderFactory,
    IDataReaderFactory_Vtbl,
    0xd7527847_57da_4e15_914c_06806699a098
);
impl windows_core::RuntimeType for IDataReaderFactory {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IDataReaderFactory");
}
#[repr(C)]
pub struct IDataReaderFactory_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub CreateDataReader: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IDataReaderStatics,
    IDataReaderStatics_Vtbl,
    0x11fcbfc8_f93a_471b_b121_f379e349313c
);
impl windows_core::RuntimeType for IDataReaderStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IDataReaderStatics");
}
#[repr(C)]
pub struct IDataReaderStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub FromBuffer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGlobalSystemMediaTransportControlsSession,
    IGlobalSystemMediaTransportControlsSession_Vtbl,
    0x7148c835_9b14_5ae2_ab85_dc9b1c14e1a8
);
impl windows_core::RuntimeType for IGlobalSystemMediaTransportControlsSession {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.IGlobalSystemMediaTransportControlsSession",
    );
}
#[repr(C)]
pub struct IGlobalSystemMediaTransportControlsSession_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub SourceAppUserModelId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub TryGetMediaPropertiesAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetTimelineProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPlaybackInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    TryPlayAsync: usize,
    TryPauseAsync: usize,
    TryStopAsync: usize,
    TryRecordAsync: usize,
    TryFastForwardAsync: usize,
    TryRewindAsync: usize,
    pub TrySkipNextAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub TrySkipPreviousAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    TryChangeChannelUpAsync: usize,
    TryChangeChannelDownAsync: usize,
    pub TryTogglePlayPauseAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGlobalSystemMediaTransportControlsSessionManager,
    IGlobalSystemMediaTransportControlsSessionManager_Vtbl,
    0xcace8eac_e86e_504a_ab31_5ff8ff1bce49
);
impl windows_core::RuntimeType for IGlobalSystemMediaTransportControlsSessionManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.IGlobalSystemMediaTransportControlsSessionManager",
    );
}
#[repr(C)]
pub struct IGlobalSystemMediaTransportControlsSessionManager_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub GetCurrentSession: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGlobalSystemMediaTransportControlsSessionManagerStatics,
    IGlobalSystemMediaTransportControlsSessionManagerStatics_Vtbl,
    0x2050c4ee_11a0_57de_aed7_c97c70338245
);
impl windows_core::RuntimeType for IGlobalSystemMediaTransportControlsSessionManagerStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.IGlobalSystemMediaTransportControlsSessionManagerStatics",
    );
}
#[repr(C)]
pub struct IGlobalSystemMediaTransportControlsSessionManagerStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub RequestAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGlobalSystemMediaTransportControlsSessionMediaProperties,
    IGlobalSystemMediaTransportControlsSessionMediaProperties_Vtbl,
    0x68856cf6_adb4_54b2_ac16_05837907acb6
);
impl windows_core::RuntimeType for IGlobalSystemMediaTransportControlsSessionMediaProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.IGlobalSystemMediaTransportControlsSessionMediaProperties",
    );
}
#[repr(C)]
pub struct IGlobalSystemMediaTransportControlsSessionMediaProperties_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Title: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    Subtitle: usize,
    AlbumArtist: usize,
    pub Artist: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AlbumTitle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    TrackNumber: usize,
    Genres: usize,
    AlbumTrackCount: usize,
    PlaybackType: usize,
    pub Thumbnail: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGlobalSystemMediaTransportControlsSessionPlaybackInfo,
    IGlobalSystemMediaTransportControlsSessionPlaybackInfo_Vtbl,
    0x94b4b6cf_e8ba_51ad_87a7_c10ade106127
);
impl windows_core::RuntimeType for IGlobalSystemMediaTransportControlsSessionPlaybackInfo {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.IGlobalSystemMediaTransportControlsSessionPlaybackInfo",
    );
}
#[repr(C)]
pub struct IGlobalSystemMediaTransportControlsSessionPlaybackInfo_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Controls: usize,
    pub PlaybackStatus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IGlobalSystemMediaTransportControlsSessionTimelineProperties,
    IGlobalSystemMediaTransportControlsSessionTimelineProperties_Vtbl,
    0xede34136_6f25_588d_8ecf_ea5b6735aaa5
);
impl windows_core::RuntimeType for IGlobalSystemMediaTransportControlsSessionTimelineProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Media.Control.IGlobalSystemMediaTransportControlsSessionTimelineProperties",
    );
}
#[repr(C)]
pub struct IGlobalSystemMediaTransportControlsSessionTimelineProperties_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub StartTime: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub EndTime: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    MinSeekTime: usize,
    MaxSeekTime: usize,
    pub Position: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_time::TimeSpan,
    ) -> windows_core::HRESULT,
    pub LastUpdatedTime: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_time::DateTime,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IInputStream,
    IInputStream_Vtbl,
    0x905a0fe2_bc53_11df_8c49_001e4fc686da
);
impl windows_core::RuntimeType for IInputStream {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IInputStream");
}
windows_core::imp::interface_hierarchy!(
    IInputStream,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(IInputStream, IClosable);
impl IInputStream {
    pub(crate) fn ReadAsync<P0>(
        &self,
        buffer: P0,
        count: u32,
        options: InputStreamOptions,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<IBuffer, u32>>
    where
        P0: windows_core::Param<IBuffer>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReadAsync)(
                windows_core::Interface::as_raw(self),
                buffer.param().abi(),
                count,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeName for IInputStream {
    const NAME: &'static str = "Windows.Storage.Streams.IInputStream";
}
#[repr(C)]
pub struct IInputStream_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ReadAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        InputStreamOptions,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IOutputStream,
    IOutputStream_Vtbl,
    0x905a0fe6_bc53_11df_8c49_001e4fc686da
);
impl windows_core::RuntimeType for IOutputStream {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IOutputStream");
}
windows_core::imp::interface_hierarchy!(
    IOutputStream,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeName for IOutputStream {
    const NAME: &'static str = "Windows.Storage.Streams.IOutputStream";
}
#[repr(C)]
pub struct IOutputStream_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IRandomAccessStream,
    IRandomAccessStream_Vtbl,
    0x905a0fe1_bc53_11df_8c49_001e4fc686da
);
impl windows_core::RuntimeType for IRandomAccessStream {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.IRandomAccessStream");
}
windows_core::imp::interface_hierarchy!(
    IRandomAccessStream,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl IRandomAccessStream {
    pub(crate) fn Size(&self) -> windows_core::Result<u64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Size)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: u64) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).SetSize)(
                windows_core::Interface::as_raw(self),
                value,
            )
            .ok()
        }
    }
    pub(crate) fn ReadAsync<P0>(
        &self,
        buffer: P0,
        count: u32,
        options: InputStreamOptions,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<IBuffer, u32>>
    where
        P0: windows_core::Param<IBuffer>,
    {
        let this = &windows_core::Interface::cast::<IInputStream>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ReadAsync)(
                windows_core::Interface::as_raw(this),
                buffer.param().abi(),
                count,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeName for IRandomAccessStream {
    const NAME: &'static str = "Windows.Storage.Streams.IRandomAccessStream";
}
#[repr(C)]
pub struct IRandomAccessStream_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Size: unsafe extern "system" fn(*mut core::ffi::c_void, *mut u64) -> windows_core::HRESULT,
    pub SetSize: unsafe extern "system" fn(*mut core::ffi::c_void, u64) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IRandomAccessStreamReference,
    IRandomAccessStreamReference_Vtbl,
    0x33ee3134_1dd6_4e3a_8067_d1c162e8642b
);
impl windows_core::RuntimeType for IRandomAccessStreamReference {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Storage.Streams.IRandomAccessStreamReference",
    );
}
windows_core::imp::interface_hierarchy!(
    IRandomAccessStreamReference,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl IRandomAccessStreamReference {
    pub(crate) fn OpenReadAsync(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<IRandomAccessStreamWithContentType>>
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).OpenReadAsync)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeName for IRandomAccessStreamReference {
    const NAME: &'static str = "Windows.Storage.Streams.IRandomAccessStreamReference";
}
pub trait IRandomAccessStreamReference_Impl: windows_core::IUnknownImpl {
    fn OpenReadAsync(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<IRandomAccessStreamWithContentType>>;
}
impl IRandomAccessStreamReference_Vtbl {
    pub const fn new<Identity: IRandomAccessStreamReference_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn OpenReadAsync<
            Identity: IRandomAccessStreamReference_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            result__: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IRandomAccessStreamReference_Impl::OpenReadAsync(this) {
                    Ok(ok__) => {
                        result__.write(core::mem::transmute_copy(&ok__));
                        core::mem::forget(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IInspectable_Vtbl::new::<
                Identity,
                IRandomAccessStreamReference,
                OFFSET,
            >(),
            OpenReadAsync: OpenReadAsync::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IRandomAccessStreamReference as windows_core::Interface>::IID
    }
}
#[repr(C)]
pub struct IRandomAccessStreamReference_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub OpenReadAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IRandomAccessStreamWithContentType,
    IRandomAccessStreamWithContentType_Vtbl,
    0xcc254827_4b3d_438f_9232_10c76bc7e038
);
impl windows_core::RuntimeType for IRandomAccessStreamWithContentType {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Storage.Streams.IRandomAccessStreamWithContentType",
    );
}
windows_core::imp::interface_hierarchy!(
    IRandomAccessStreamWithContentType,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    IRandomAccessStreamWithContentType,
    IClosable,
    IContentTypeProvider,
    IInputStream,
    IOutputStream,
    IRandomAccessStream
);
impl IRandomAccessStreamWithContentType {
    pub(crate) fn ReadAsync<P0>(
        &self,
        buffer: P0,
        count: u32,
        options: InputStreamOptions,
    ) -> windows_core::Result<windows_future::IAsyncOperationWithProgress<IBuffer, u32>>
    where
        P0: windows_core::Param<IBuffer>,
    {
        let this = &windows_core::Interface::cast::<IInputStream>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ReadAsync)(
                windows_core::Interface::as_raw(this),
                buffer.param().abi(),
                count,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn Size(&self) -> windows_core::Result<u64> {
        let this = &windows_core::Interface::cast::<IRandomAccessStream>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).Size)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn SetSize(&self, value: u64) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IRandomAccessStream>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).SetSize)(
                windows_core::Interface::as_raw(this),
                value,
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeName for IRandomAccessStreamWithContentType {
    const NAME: &'static str = "Windows.Storage.Streams.IRandomAccessStreamWithContentType";
}
#[repr(C)]
pub struct IRandomAccessStreamWithContentType_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputStreamOptions(pub u32);
impl InputStreamOptions {
    pub const None: Self = Self(0);
    pub const Partial: Self = Self(1);
    pub const ReadAhead: Self = Self(2);
}
impl windows_core::imp::TypeKind for InputStreamOptions {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for InputStreamOptions {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Storage.Streams.InputStreamOptions;u4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Storage.Streams.InputStreamOptions");
}
impl InputStreamOptions {
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
impl core::ops::BitOr for InputStreamOptions {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl core::ops::BitAnd for InputStreamOptions {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl core::ops::BitOrAssign for InputStreamOptions {
    fn bitor_assign(&mut self, other: Self) {
        self.0.bitor_assign(other.0);
    }
}
impl core::ops::BitAndAssign for InputStreamOptions {
    fn bitand_assign(&mut self, other: Self) {
        self.0.bitand_assign(other.0);
    }
}
impl core::ops::Not for InputStreamOptions {
    type Output = Self;
    fn not(self) -> Self {
        Self(self.0.not())
    }
}
