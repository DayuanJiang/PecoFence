#[inline]
pub unsafe fn AllowSetForegroundWindow(dwprocessid: u32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn AllowSetForegroundWindow(dwprocessid : u32) -> windows_core::BOOL);
    unsafe { AllowSetForegroundWindow(dwprocessid) }
}
#[inline]
pub unsafe fn SHQueryUserNotificationState() -> windows_core::Result<QUERY_USER_NOTIFICATION_STATE>
{
    windows_core::link!("shell32.dll" "system" fn SHQueryUserNotificationState(pquns : *mut QUERY_USER_NOTIFICATION_STATE) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SHQueryUserNotificationState(&mut result__).map(|| result__)
    }
}
pub const ASFW_ANY: u32 = 4294967295;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWND(pub *mut core::ffi::c_void);
windows_core::imp::define_interface!(
    IInitializeWithWindow,
    IInitializeWithWindow_Vtbl,
    0x3e68d4bd_7135_4d10_8018_9fb6d9f33fa1
);
windows_core::imp::interface_hierarchy!(IInitializeWithWindow, windows_core::IUnknown);
impl IInitializeWithWindow {
    pub(crate) unsafe fn Initialize(&self, hwnd: HWND) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Initialize)(
                windows_core::Interface::as_raw(self),
                hwnd,
            )
        }
    }
}
#[repr(C)]
pub struct IInitializeWithWindow_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Initialize:
        unsafe extern "system" fn(*mut core::ffi::c_void, HWND) -> windows_core::HRESULT,
}
pub trait IInitializeWithWindow_Impl: windows_core::IUnknownImpl {
    fn Initialize(&self, hwnd: HWND) -> windows_core::Result<()>;
}
impl IInitializeWithWindow_Vtbl {
    pub const fn new<Identity: IInitializeWithWindow_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Initialize<
            Identity: IInitializeWithWindow_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwnd: HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IInitializeWithWindow_Impl::Initialize(this, core::mem::transmute_copy(&hwnd))
                    .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Initialize: Initialize::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IInitializeWithWindow as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IInitializeWithWindow {}
windows_core::imp::define_interface!(
    IStartupTask,
    IStartupTask_Vtbl,
    0xf75c23c8_b5f2_4f6c_88dd_36cb1d599d17
);
impl windows_core::RuntimeType for IStartupTask {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.ApplicationModel.IStartupTask");
}
#[repr(C)]
pub struct IStartupTask_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    RequestEnableAsync: usize,
    Disable: usize,
    pub State: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut StartupTaskState,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IStartupTaskStatics,
    IStartupTaskStatics_Vtbl,
    0xee5b60bd_a148_41a7_b26e_e8b88a1e62f8
);
impl windows_core::RuntimeType for IStartupTaskStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.ApplicationModel.IStartupTaskStatics");
}
#[repr(C)]
pub struct IStartupTaskStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    GetForCurrentPackageAsync: usize,
    pub GetAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IStoreContext,
    IStoreContext_Vtbl,
    0xac98b6be_f4fd_4912_babd_5035e5e8bcab
);
impl windows_core::RuntimeType for IStoreContext {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Services.Store.IStoreContext");
}
#[repr(C)]
pub struct IStoreContext_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IStoreContext4,
    IStoreContext4_Vtbl,
    0xaf9c6f69_bea1_4bf4_8e74_ae03e206c6b0
);
impl windows_core::RuntimeType for IStoreContext4 {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Services.Store.IStoreContext4");
}
#[repr(C)]
pub struct IStoreContext4_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub RequestRateAndReviewAppAsync: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IStoreContextStatics,
    IStoreContextStatics_Vtbl,
    0x9c06ee5f_15c0_4e72_9330_d6191cebd19c
);
impl windows_core::RuntimeType for IStoreContextStatics {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Services.Store.IStoreContextStatics");
}
#[repr(C)]
pub struct IStoreContextStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub GetDefault: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_interface!(
    IStoreRateAndReviewResult,
    IStoreRateAndReviewResult_Vtbl,
    0x9d209d56_a6b5_4121_9b61_ee6d0fbdbdbb
);
impl windows_core::RuntimeType for IStoreRateAndReviewResult {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Services.Store.IStoreRateAndReviewResult",
    );
}
#[repr(C)]
pub struct IStoreRateAndReviewResult_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub ExtendedError: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::HRESULT,
    ) -> windows_core::HRESULT,
    pub ExtendedJsonData: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub WasUpdated:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
    pub Status: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut StoreRateAndReviewStatus,
    ) -> windows_core::HRESULT,
}
pub type QUERY_USER_NOTIFICATION_STATE = i32;
pub const QUNS_ACCEPTS_NOTIFICATIONS: QUERY_USER_NOTIFICATION_STATE = 5;
pub const QUNS_APP: QUERY_USER_NOTIFICATION_STATE = 7;
pub const QUNS_BUSY: QUERY_USER_NOTIFICATION_STATE = 2;
pub const QUNS_NOT_PRESENT: QUERY_USER_NOTIFICATION_STATE = 1;
pub const QUNS_PRESENTATION_MODE: QUERY_USER_NOTIFICATION_STATE = 4;
pub const QUNS_QUIET_TIME: QUERY_USER_NOTIFICATION_STATE = 6;
pub const QUNS_RUNNING_D3D_FULL_SCREEN: QUERY_USER_NOTIFICATION_STATE = 3;
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupTask(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    StartupTask,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl StartupTask {
    pub(crate) fn State(&self) -> windows_core::Result<StartupTaskState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).State)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn GetAsync(
        taskid: &windows_core::HSTRING,
    ) -> windows_core::Result<windows_future::IAsyncOperation<Self>> {
        Self::IStartupTaskStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetAsync)(
                windows_core::Interface::as_raw(this),
                core::mem::transmute_copy(taskid),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IStartupTaskStatics<R, F: FnOnce(&IStartupTaskStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<StartupTask, IStartupTaskStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for StartupTask {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IStartupTask>();
}
unsafe impl windows_core::Interface for StartupTask {
    type Vtable = <IStartupTask as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStartupTask as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for StartupTask {
    const NAME: &'static str = "Windows.ApplicationModel.StartupTask";
}
unsafe impl Send for StartupTask {}
unsafe impl Sync for StartupTask {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StartupTaskState(pub i32);
impl StartupTaskState {
    pub const Disabled: Self = Self(0);
    pub const DisabledByUser: Self = Self(1);
    pub const Enabled: Self = Self(2);
    pub const DisabledByPolicy: Self = Self(3);
    pub const EnabledByPolicy: Self = Self(4);
}
impl windows_core::imp::TypeKind for StartupTaskState {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for StartupTaskState {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.ApplicationModel.StartupTaskState;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.ApplicationModel.StartupTaskState");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreContext(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    StoreContext,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl StoreContext {
    pub(crate) fn RequestRateAndReviewAppAsync(
        &self,
    ) -> windows_core::Result<windows_future::IAsyncOperation<StoreRateAndReviewResult>> {
        let this = &windows_core::Interface::cast::<IStoreContext4>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).RequestRateAndReviewAppAsync)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) fn GetDefault() -> windows_core::Result<Self> {
        Self::IStoreContextStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetDefault)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        })
    }
    fn IStoreContextStatics<R, F: FnOnce(&IStoreContextStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<StoreContext, IStoreContextStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for StoreContext {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IStoreContext>();
}
unsafe impl windows_core::Interface for StoreContext {
    type Vtable = <IStoreContext as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStoreContext as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for StoreContext {
    const NAME: &'static str = "Windows.Services.Store.StoreContext";
}
unsafe impl Send for StoreContext {}
unsafe impl Sync for StoreContext {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoreRateAndReviewResult(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    StoreRateAndReviewResult,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl StoreRateAndReviewResult {
    pub(crate) fn ExtendedError(&self) -> windows_core::Result<windows_core::HRESULT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ExtendedError)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn ExtendedJsonData(&self) -> windows_core::Result<windows_core::HSTRING> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ExtendedJsonData)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) fn WasUpdated(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).WasUpdated)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) fn Status(&self) -> windows_core::Result<StoreRateAndReviewStatus> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Status)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
impl windows_core::RuntimeType for StoreRateAndReviewResult {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IStoreRateAndReviewResult>();
}
unsafe impl windows_core::Interface for StoreRateAndReviewResult {
    type Vtable = <IStoreRateAndReviewResult as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IStoreRateAndReviewResult as windows_core::Interface>::IID;
}
impl windows_core::RuntimeName for StoreRateAndReviewResult {
    const NAME: &'static str = "Windows.Services.Store.StoreRateAndReviewResult";
}
unsafe impl Send for StoreRateAndReviewResult {}
unsafe impl Sync for StoreRateAndReviewResult {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StoreRateAndReviewStatus(pub i32);
impl StoreRateAndReviewStatus {
    pub const Succeeded: Self = Self(0);
    pub const CanceledByUser: Self = Self(1);
    pub const NetworkError: Self = Self(2);
    pub const Error: Self = Self(3);
}
impl windows_core::imp::TypeKind for StoreRateAndReviewStatus {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for StoreRateAndReviewStatus {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Windows.Services.Store.StoreRateAndReviewStatus;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Windows.Services.Store.StoreRateAndReviewStatus",
    );
}
