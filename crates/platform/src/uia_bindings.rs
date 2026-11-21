#[inline]
pub unsafe fn AccessibleObjectFromWindow<T>(hwnd: HWND, dwid: u32) -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    windows_core::link!("oleacc.dll" "system" fn AccessibleObjectFromWindow(hwnd : HWND, dwid : u32, riid : *const windows_core::GUID, ppvobject : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        AccessibleObjectFromWindow(hwnd, dwid, &T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn CoCreateInstance<P1, T>(
    rclsid: *const windows_core::GUID,
    punkouter: P1,
    dwclscontext: u32,
) -> windows_core::Result<T>
where
    P1: windows_core::Param<windows_core::IUnknown>,
    T: windows_core::Interface,
{
    windows_core::link!("ole32.dll" "system" fn CoCreateInstance(rclsid : *const windows_core::GUID, punkouter : *mut core::ffi::c_void, dwclscontext : u32, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        CoCreateInstance(
            rclsid,
            punkouter.param().abi(),
            dwclscontext,
            &T::IID,
            &mut result__,
        )
        .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SafeArrayAccessData(
    psa: *const SAFEARRAY,
    ppvdata: *mut *mut core::ffi::c_void,
) -> windows_core::HRESULT {
    windows_core::link!("oleaut32.dll" "system" fn SafeArrayAccessData(psa : *const SAFEARRAY, ppvdata : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe { SafeArrayAccessData(psa, ppvdata as _) }
}
#[inline]
pub unsafe fn SafeArrayDestroy(psa: *const SAFEARRAY) -> windows_core::HRESULT {
    windows_core::link!("oleaut32.dll" "system" fn SafeArrayDestroy(psa : *const SAFEARRAY) -> windows_core::HRESULT);
    unsafe { SafeArrayDestroy(psa) }
}
#[inline]
pub unsafe fn SafeArrayGetLBound(psa: *const SAFEARRAY, ndim: u32) -> windows_core::Result<i32> {
    windows_core::link!("oleaut32.dll" "system" fn SafeArrayGetLBound(psa : *const SAFEARRAY, ndim : u32, pllbound : *mut i32) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SafeArrayGetLBound(psa, ndim, &mut result__).map(|| result__)
    }
}
#[inline]
pub unsafe fn SafeArrayGetUBound(psa: *const SAFEARRAY, ndim: u32) -> windows_core::Result<i32> {
    windows_core::link!("oleaut32.dll" "system" fn SafeArrayGetUBound(psa : *const SAFEARRAY, ndim : u32, plubound : *mut i32) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SafeArrayGetUBound(psa, ndim, &mut result__).map(|| result__)
    }
}
#[inline]
pub unsafe fn SafeArrayUnaccessData(psa: *const SAFEARRAY) -> windows_core::HRESULT {
    windows_core::link!("oleaut32.dll" "system" fn SafeArrayUnaccessData(psa : *const SAFEARRAY) -> windows_core::HRESULT);
    unsafe { SafeArrayUnaccessData(psa) }
}
pub type CLSCTX = u32;
pub const CLSCTX_INPROC_SERVER: CLSCTX = 1;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CONTROLTYPEID(pub i32);
pub const CUIAutomation: windows_core::GUID =
    windows_core::GUID::from_u128(0xff48dba4_60ef_4201_aa87_54103eef594e);
#[repr(C)]
#[derive(Clone, Copy)]
pub union CY {
    pub Anonymous: CY_0,
    pub int64: i64,
}
impl Default for CY {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CY_0 {
    pub Lo: u32,
    pub Hi: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DECIMAL {
    pub wReserved: u16,
    pub Anonymous: DECIMAL_0,
    pub Hi32: u32,
    pub Anonymous2: DECIMAL_1,
}
impl Default for DECIMAL {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DECIMAL_0 {
    pub Anonymous: DECIMAL_0_0,
    pub signscale: u16,
}
impl Default for DECIMAL_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DECIMAL_0_0 {
    pub scale: u8,
    pub sign: u8,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union DECIMAL_1 {
    pub Anonymous: DECIMAL_1_0,
    pub Lo64: u64,
}
impl Default for DECIMAL_1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DECIMAL_1_0 {
    pub Lo32: u32,
    pub Mid32: u32,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct EVENTID(pub i32);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWND(pub *mut core::ffi::c_void);
windows_core::imp::define_interface!(
    IAccessible,
    IAccessible_Vtbl,
    0x618736e0_3c3d_11cf_810c_00aa00389b71
);
impl core::ops::Deref for IAccessible {
    type Target = IDispatch;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IAccessible, windows_core::IUnknown, IDispatch);
impl IAccessible {
    pub(crate) unsafe fn accParent(&self) -> windows_core::Result<IDispatch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accParent)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn accChildCount(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accChildCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn accChild(&self, varchild: &VARIANT) -> windows_core::Result<IDispatch> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accChild)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn accName(
        &self,
        varchild: &VARIANT,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accName)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accValue(
        &self,
        varchild: &VARIANT,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accValue)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accDescription(
        &self,
        varchild: &VARIANT,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accDescription)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accRole(&self, varchild: &VARIANT) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accRole)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accState(&self, varchild: &VARIANT) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accState)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accHelp(
        &self,
        varchild: &VARIANT,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accHelp)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accHelpTopic(
        &self,
        pszhelpfile: *mut windows_core::BSTR,
        varchild: &VARIANT,
    ) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accHelpTopic)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(pszhelpfile),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn accKeyboardShortcut(
        &self,
        varchild: &VARIANT,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accKeyboardShortcut)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accFocus(&self) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accFocus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accSelection(&self) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accSelection)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accDefaultAction(
        &self,
        varchild: &VARIANT,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accDefaultAction)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accSelect(
        &self,
        flagsselect: i32,
        varchild: &VARIANT,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).accSelect)(
                windows_core::Interface::as_raw(self),
                flagsselect,
                core::mem::transmute_copy(varchild),
            )
        }
    }
    pub(crate) unsafe fn accLocation(
        &self,
        pxleft: *mut i32,
        pytop: *mut i32,
        pcxwidth: *mut i32,
        pcyheight: *mut i32,
        varchild: &VARIANT,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).accLocation)(
                windows_core::Interface::as_raw(self),
                pxleft as _,
                pytop as _,
                pcxwidth as _,
                pcyheight as _,
                core::mem::transmute_copy(varchild),
            )
        }
    }
    pub(crate) unsafe fn accNavigate(
        &self,
        navdir: i32,
        varstart: &VARIANT,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accNavigate)(
                windows_core::Interface::as_raw(self),
                navdir,
                core::mem::transmute_copy(varstart),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accHitTest(&self, xleft: i32, ytop: i32) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).accHitTest)(
                windows_core::Interface::as_raw(self),
                xleft,
                ytop,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn accDoDefaultAction(&self, varchild: &VARIANT) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).accDoDefaultAction)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
            )
        }
    }
    pub(crate) unsafe fn SetaccName(
        &self,
        varchild: &VARIANT,
        szname: &windows_core::BSTR,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetaccName)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                core::mem::transmute_copy(szname),
            )
        }
    }
    pub(crate) unsafe fn SetaccValue(
        &self,
        varchild: &VARIANT,
        szvalue: &windows_core::BSTR,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetaccValue)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(varchild),
                core::mem::transmute_copy(szvalue),
            )
        }
    }
}
#[repr(C)]
pub struct IAccessible_Vtbl {
    pub base__: IDispatch_Vtbl,
    pub accParent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accChildCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub accChild: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accRole: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub accState: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub accHelp: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accHelpTopic: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
        VARIANT,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub accKeyboardShortcut: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accFocus:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut VARIANT) -> windows_core::HRESULT,
    pub accSelection:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut VARIANT) -> windows_core::HRESULT,
    pub accDefaultAction: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub accSelect:
        unsafe extern "system" fn(*mut core::ffi::c_void, i32, VARIANT) -> windows_core::HRESULT,
    pub accLocation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut i32,
        *mut i32,
        *mut i32,
        *mut i32,
        VARIANT,
    ) -> windows_core::HRESULT,
    pub accNavigate: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        i32,
        VARIANT,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub accHitTest: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        i32,
        i32,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub accDoDefaultAction:
        unsafe extern "system" fn(*mut core::ffi::c_void, VARIANT) -> windows_core::HRESULT,
    pub SetaccName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetaccValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IAccessible {}
windows_core::imp::define_interface!(
    IDispatch,
    IDispatch_Vtbl,
    0x00020400_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IDispatch, windows_core::IUnknown);
#[repr(C)]
pub struct IDispatch_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetTypeInfoCount: usize,
    GetTypeInfo: usize,
    GetIDsOfNames: usize,
    Invoke: usize,
}
impl windows_core::RuntimeName for IDispatch {}
windows_core::imp::define_interface!(
    IRecordInfo,
    IRecordInfo_Vtbl,
    0x0000002f_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IRecordInfo, windows_core::IUnknown);
#[repr(C)]
pub struct IRecordInfo_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    RecordInit: usize,
    RecordClear: usize,
    RecordCopy: usize,
    GetGuid: usize,
    GetName: usize,
    GetSize: usize,
    GetTypeInfo: usize,
    GetField: usize,
    GetFieldNoCopy: usize,
    PutField: usize,
    PutFieldNoCopy: usize,
    GetFieldNames: usize,
    IsMatchingType: usize,
    RecordCreate: usize,
    RecordCreateCopy: usize,
    RecordDestroy: usize,
}
impl windows_core::RuntimeName for IRecordInfo {}
windows_core::imp::define_interface!(
    IUIAutomation,
    IUIAutomation_Vtbl,
    0x30cbe57d_d9d0_452a_ab13_7ac5ac4825ee
);
windows_core::imp::interface_hierarchy!(IUIAutomation, windows_core::IUnknown);
impl IUIAutomation {
    pub(crate) unsafe fn CompareElements<P0, P1>(
        &self,
        el1: P0,
        el2: P1,
    ) -> windows_core::Result<windows_core::BOOL>
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P1: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CompareElements)(
                windows_core::Interface::as_raw(self),
                el1.param().abi(),
                el2.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CompareRuntimeIds(
        &self,
        runtimeid1: *const SAFEARRAY,
        runtimeid2: *const SAFEARRAY,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CompareRuntimeIds)(
                windows_core::Interface::as_raw(self),
                runtimeid1,
                runtimeid2,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetRootElement(&self) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRootElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromHandle(
        &self,
        hwnd: UIA_HWND,
    ) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromHandle)(
                windows_core::Interface::as_raw(self),
                hwnd,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromPoint(
        &self,
        pt: POINT,
    ) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromPoint)(
                windows_core::Interface::as_raw(self),
                pt,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetFocusedElement(&self) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFocusedElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetRootElementBuildCache<P0>(
        &self,
        cacherequest: P0,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRootElementBuildCache)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromHandleBuildCache<P1>(
        &self,
        hwnd: UIA_HWND,
        cacherequest: P1,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P1: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromHandleBuildCache)(
                windows_core::Interface::as_raw(self),
                hwnd,
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromPointBuildCache<P1>(
        &self,
        pt: POINT,
        cacherequest: P1,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P1: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromPointBuildCache)(
                windows_core::Interface::as_raw(self),
                pt,
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetFocusedElementBuildCache<P0>(
        &self,
        cacherequest: P0,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFocusedElementBuildCache)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateTreeWalker<P0>(
        &self,
        pcondition: P0,
    ) -> windows_core::Result<IUIAutomationTreeWalker>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTreeWalker)(
                windows_core::Interface::as_raw(self),
                pcondition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ControlViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ControlViewWalker)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ContentViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentViewWalker)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn RawViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RawViewWalker)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn RawViewCondition(&self) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RawViewCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ControlViewCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ControlViewCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ContentViewCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ContentViewCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateCacheRequest(
        &self,
    ) -> windows_core::Result<IUIAutomationCacheRequest> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCacheRequest)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateTrueCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateTrueCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateFalseCondition(
        &self,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFalseCondition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreatePropertyCondition(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreatePropertyCondition)(
                windows_core::Interface::as_raw(self),
                propertyid,
                core::mem::transmute_copy(value),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreatePropertyConditionEx(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
        flags: PropertyConditionFlags,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreatePropertyConditionEx)(
                windows_core::Interface::as_raw(self),
                propertyid,
                core::mem::transmute_copy(value),
                flags,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateAndCondition<P0, P1>(
        &self,
        condition1: P0,
        condition2: P1,
    ) -> windows_core::Result<IUIAutomationCondition>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateAndCondition)(
                windows_core::Interface::as_raw(self),
                condition1.param().abi(),
                condition2.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateAndConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateAndConditionFromArray)(
                windows_core::Interface::as_raw(self),
                conditions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateAndConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateAndConditionFromNativeArray)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(conditions),
                conditioncount,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateOrCondition<P0, P1>(
        &self,
        condition1: P0,
        condition2: P1,
    ) -> windows_core::Result<IUIAutomationCondition>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOrCondition)(
                windows_core::Interface::as_raw(self),
                condition1.param().abi(),
                condition2.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateOrConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOrConditionFromArray)(
                windows_core::Interface::as_raw(self),
                conditions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateOrConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateOrConditionFromNativeArray)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute(conditions),
                conditioncount,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateNotCondition<P0>(
        &self,
        condition: P0,
    ) -> windows_core::Result<IUIAutomationCondition>
    where
        P0: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateNotCondition)(
                windows_core::Interface::as_raw(self),
                condition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn AddAutomationEventHandler<P1, P3, P4>(
        &self,
        eventid: EVENTID,
        element: P1,
        scope: TreeScope,
        cacherequest: P3,
        handler: P4,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IUIAutomationElement>,
        P3: windows_core::Param<IUIAutomationCacheRequest>,
        P4: windows_core::Param<IUIAutomationEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddAutomationEventHandler)(
                windows_core::Interface::as_raw(self),
                eventid,
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveAutomationEventHandler<P1, P2>(
        &self,
        eventid: EVENTID,
        element: P1,
        handler: P2,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveAutomationEventHandler)(
                windows_core::Interface::as_raw(self),
                eventid,
                element.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn AddPropertyChangedEventHandlerNativeArray<P0, P2, P3>(
        &self,
        element: P0,
        scope: TreeScope,
        cacherequest: P2,
        handler: P3,
        propertyarray: *const PROPERTYID,
        propertycount: i32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
        P3: windows_core::Param<IUIAutomationPropertyChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddPropertyChangedEventHandlerNativeArray)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
                propertyarray,
                propertycount,
            )
        }
    }
    pub(crate) unsafe fn AddPropertyChangedEventHandler<P0, P2, P3>(
        &self,
        element: P0,
        scope: TreeScope,
        cacherequest: P2,
        handler: P3,
        propertyarray: *const SAFEARRAY,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
        P3: windows_core::Param<IUIAutomationPropertyChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddPropertyChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
                propertyarray,
            )
        }
    }
    pub(crate) unsafe fn RemovePropertyChangedEventHandler<P0, P1>(
        &self,
        element: P0,
        handler: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P1: windows_core::Param<IUIAutomationPropertyChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemovePropertyChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn AddStructureChangedEventHandler<P0, P2, P3>(
        &self,
        element: P0,
        scope: TreeScope,
        cacherequest: P2,
        handler: P3,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
        P3: windows_core::Param<IUIAutomationStructureChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddStructureChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                scope,
                cacherequest.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveStructureChangedEventHandler<P0, P1>(
        &self,
        element: P0,
        handler: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
        P1: windows_core::Param<IUIAutomationStructureChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveStructureChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                element.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn AddFocusChangedEventHandler<P0, P1>(
        &self,
        cacherequest: P0,
        handler: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
        P1: windows_core::Param<IUIAutomationFocusChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddFocusChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveFocusChangedEventHandler<P0>(
        &self,
        handler: P0,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationFocusChangedEventHandler>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveFocusChangedEventHandler)(
                windows_core::Interface::as_raw(self),
                handler.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RemoveAllEventHandlers(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveAllEventHandlers)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn IntNativeArrayToSafeArray(
        &self,
        array: *const i32,
        arraycount: i32,
    ) -> windows_core::Result<*mut SAFEARRAY> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IntNativeArrayToSafeArray)(
                windows_core::Interface::as_raw(self),
                array,
                arraycount,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn IntSafeArrayToNativeArray(
        &self,
        intarray: *const SAFEARRAY,
        array: *mut *mut i32,
    ) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).IntSafeArrayToNativeArray)(
                windows_core::Interface::as_raw(self),
                intarray,
                array as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn RectToVariant(&self, rc: RECT) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RectToVariant)(
                windows_core::Interface::as_raw(self),
                rc,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn VariantToRect(&self, var: &VARIANT) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).VariantToRect)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(var),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SafeArrayToRectNativeArray(
        &self,
        rects: *const SAFEARRAY,
        rectarray: *mut *mut RECT,
    ) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SafeArrayToRectNativeArray)(
                windows_core::Interface::as_raw(self),
                rects,
                rectarray as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CreateProxyFactoryEntry<P0>(
        &self,
        factory: P0,
    ) -> windows_core::Result<IUIAutomationProxyFactoryEntry>
    where
        P0: windows_core::Param<IUIAutomationProxyFactory>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateProxyFactoryEntry)(
                windows_core::Interface::as_raw(self),
                factory.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ProxyFactoryMapping(
        &self,
    ) -> windows_core::Result<IUIAutomationProxyFactoryMapping> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ProxyFactoryMapping)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetPropertyProgrammaticName(
        &self,
        property: PROPERTYID,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPropertyProgrammaticName)(
                windows_core::Interface::as_raw(self),
                property,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetPatternProgrammaticName(
        &self,
        pattern: PATTERNID,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPatternProgrammaticName)(
                windows_core::Interface::as_raw(self),
                pattern,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn PollForPotentialSupportedPatterns<P0>(
        &self,
        pelement: P0,
        patternids: *mut *mut SAFEARRAY,
        patternnames: *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PollForPotentialSupportedPatterns)(
                windows_core::Interface::as_raw(self),
                pelement.param().abi(),
                patternids as _,
                patternnames as _,
            )
        }
    }
    pub(crate) unsafe fn PollForPotentialSupportedProperties<P0>(
        &self,
        pelement: P0,
        propertyids: *mut *mut SAFEARRAY,
        propertynames: *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PollForPotentialSupportedProperties)(
                windows_core::Interface::as_raw(self),
                pelement.param().abi(),
                propertyids as _,
                propertynames as _,
            )
        }
    }
    pub(crate) unsafe fn CheckNotSupported(
        &self,
        value: &VARIANT,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CheckNotSupported)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(value),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn ReservedNotSupportedValue(
        &self,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReservedNotSupportedValue)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ReservedMixedAttributeValue(
        &self,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ReservedMixedAttributeValue)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromIAccessible<P0>(
        &self,
        accessible: P0,
        childid: i32,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IAccessible>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromIAccessible)(
                windows_core::Interface::as_raw(self),
                accessible.param().abi(),
                childid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ElementFromIAccessibleBuildCache<P0, P2>(
        &self,
        accessible: P0,
        childid: i32,
        cacherequest: P2,
    ) -> windows_core::Result<IUIAutomationElement>
    where
        P0: windows_core::Param<IAccessible>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ElementFromIAccessibleBuildCache)(
                windows_core::Interface::as_raw(self),
                accessible.param().abi(),
                childid,
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIAutomation_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub CompareElements: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CompareRuntimeIds: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *const SAFEARRAY,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetRootElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromHandle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        UIA_HWND,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        POINT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFocusedElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetRootElementBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromHandleBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        UIA_HWND,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromPointBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        POINT,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFocusedElementBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTreeWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ControlViewWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ContentViewWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RawViewWalker: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RawViewCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ControlViewCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ContentViewCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCacheRequest: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateTrueCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFalseCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePropertyCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        VARIANT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePropertyConditionEx: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        VARIANT,
        PropertyConditionFlags,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAndCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAndConditionFromArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateAndConditionFromNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateOrCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateOrConditionFromArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateOrConditionFromNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateNotCondition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddAutomationEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        EVENTID,
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveAutomationEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        EVENTID,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddPropertyChangedEventHandlerNativeArray:
        unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            TreeScope,
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            *const PROPERTYID,
            i32,
        ) -> windows_core::HRESULT,
    pub AddPropertyChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub RemovePropertyChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddStructureChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveStructureChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddFocusChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveFocusChangedEventHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RemoveAllEventHandlers:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub IntNativeArrayToSafeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const i32,
        i32,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub IntSafeArrayToNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut i32,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub RectToVariant: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        RECT,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub VariantToRect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut RECT,
    ) -> windows_core::HRESULT,
    pub SafeArrayToRectNativeArray: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const SAFEARRAY,
        *mut *mut RECT,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub CreateProxyFactoryEntry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ProxyFactoryMapping: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPropertyProgrammaticName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPatternProgrammaticName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub PollForPotentialSupportedPatterns: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub PollForPotentialSupportedProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
        *mut *mut SAFEARRAY,
    )
        -> windows_core::HRESULT,
    pub CheckNotSupported: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        VARIANT,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub ReservedNotSupportedValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ReservedMixedAttributeValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromIAccessible: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ElementFromIAccessibleBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        i32,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomation_Impl: windows_core::IUnknownImpl {
    fn CompareElements(
        &self,
        el1: windows_core::Ref<IUIAutomationElement>,
        el2: windows_core::Ref<IUIAutomationElement>,
    ) -> windows_core::Result<windows_core::BOOL>;
    fn CompareRuntimeIds(
        &self,
        runtimeid1: *const SAFEARRAY,
        runtimeid2: *const SAFEARRAY,
    ) -> windows_core::Result<windows_core::BOOL>;
    fn GetRootElement(&self) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromHandle(&self, hwnd: UIA_HWND) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromPoint(&self, pt: &POINT) -> windows_core::Result<IUIAutomationElement>;
    fn GetFocusedElement(&self) -> windows_core::Result<IUIAutomationElement>;
    fn GetRootElementBuildCache(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromHandleBuildCache(
        &self,
        hwnd: UIA_HWND,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromPointBuildCache(
        &self,
        pt: &POINT,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn GetFocusedElementBuildCache(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn CreateTreeWalker(
        &self,
        pcondition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn ControlViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn ContentViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn RawViewWalker(&self) -> windows_core::Result<IUIAutomationTreeWalker>;
    fn RawViewCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn ControlViewCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn ContentViewCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateCacheRequest(&self) -> windows_core::Result<IUIAutomationCacheRequest>;
    fn CreateTrueCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateFalseCondition(&self) -> windows_core::Result<IUIAutomationCondition>;
    fn CreatePropertyCondition(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreatePropertyConditionEx(
        &self,
        propertyid: PROPERTYID,
        value: &VARIANT,
        flags: PropertyConditionFlags,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateAndCondition(
        &self,
        condition1: windows_core::Ref<IUIAutomationCondition>,
        condition2: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateAndConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateAndConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateOrCondition(
        &self,
        condition1: windows_core::Ref<IUIAutomationCondition>,
        condition2: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateOrConditionFromArray(
        &self,
        conditions: *const SAFEARRAY,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateOrConditionFromNativeArray(
        &self,
        conditions: *const Option<IUIAutomationCondition>,
        conditioncount: i32,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn CreateNotCondition(
        &self,
        condition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationCondition>;
    fn AddAutomationEventHandler(
        &self,
        eventid: EVENTID,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveAutomationEventHandler(
        &self,
        eventid: EVENTID,
        element: windows_core::Ref<IUIAutomationElement>,
        handler: windows_core::Ref<IUIAutomationEventHandler>,
    ) -> windows_core::Result<()>;
    fn AddPropertyChangedEventHandlerNativeArray(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationPropertyChangedEventHandler>,
        propertyarray: *const PROPERTYID,
        propertycount: i32,
    ) -> windows_core::Result<()>;
    fn AddPropertyChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationPropertyChangedEventHandler>,
        propertyarray: *const SAFEARRAY,
    ) -> windows_core::Result<()>;
    fn RemovePropertyChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        handler: windows_core::Ref<IUIAutomationPropertyChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn AddStructureChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        scope: TreeScope,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationStructureChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveStructureChangedEventHandler(
        &self,
        element: windows_core::Ref<IUIAutomationElement>,
        handler: windows_core::Ref<IUIAutomationStructureChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn AddFocusChangedEventHandler(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
        handler: windows_core::Ref<IUIAutomationFocusChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveFocusChangedEventHandler(
        &self,
        handler: windows_core::Ref<IUIAutomationFocusChangedEventHandler>,
    ) -> windows_core::Result<()>;
    fn RemoveAllEventHandlers(&self) -> windows_core::Result<()>;
    fn IntNativeArrayToSafeArray(
        &self,
        array: *const i32,
        arraycount: i32,
    ) -> windows_core::Result<*mut SAFEARRAY>;
    fn IntSafeArrayToNativeArray(
        &self,
        intarray: *const SAFEARRAY,
        array: *mut *mut i32,
    ) -> windows_core::Result<i32>;
    fn RectToVariant(&self, rc: &RECT) -> windows_core::Result<VARIANT>;
    fn VariantToRect(&self, var: &VARIANT) -> windows_core::Result<RECT>;
    fn SafeArrayToRectNativeArray(
        &self,
        rects: *const SAFEARRAY,
        rectarray: *mut *mut RECT,
    ) -> windows_core::Result<i32>;
    fn CreateProxyFactoryEntry(
        &self,
        factory: windows_core::Ref<IUIAutomationProxyFactory>,
    ) -> windows_core::Result<IUIAutomationProxyFactoryEntry>;
    fn ProxyFactoryMapping(&self) -> windows_core::Result<IUIAutomationProxyFactoryMapping>;
    fn GetPropertyProgrammaticName(
        &self,
        property: PROPERTYID,
    ) -> windows_core::Result<windows_core::BSTR>;
    fn GetPatternProgrammaticName(
        &self,
        pattern: PATTERNID,
    ) -> windows_core::Result<windows_core::BSTR>;
    fn PollForPotentialSupportedPatterns(
        &self,
        pelement: windows_core::Ref<IUIAutomationElement>,
        patternids: *mut *mut SAFEARRAY,
        patternnames: *mut *mut SAFEARRAY,
    ) -> windows_core::Result<()>;
    fn PollForPotentialSupportedProperties(
        &self,
        pelement: windows_core::Ref<IUIAutomationElement>,
        propertyids: *mut *mut SAFEARRAY,
        propertynames: *mut *mut SAFEARRAY,
    ) -> windows_core::Result<()>;
    fn CheckNotSupported(&self, value: &VARIANT) -> windows_core::Result<windows_core::BOOL>;
    fn ReservedNotSupportedValue(&self) -> windows_core::Result<windows_core::IUnknown>;
    fn ReservedMixedAttributeValue(&self) -> windows_core::Result<windows_core::IUnknown>;
    fn ElementFromIAccessible(
        &self,
        accessible: windows_core::Ref<IAccessible>,
        childid: i32,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn ElementFromIAccessibleBuildCache(
        &self,
        accessible: windows_core::Ref<IAccessible>,
        childid: i32,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
}
impl IUIAutomation_Vtbl {
    pub const fn new<Identity: IUIAutomation_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CompareElements<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            el1: *mut core::ffi::c_void,
            el2: *mut core::ffi::c_void,
            aresame: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CompareElements(
                    this,
                    core::mem::transmute_copy(&el1),
                    core::mem::transmute_copy(&el2),
                ) {
                    Ok(ok__) => {
                        aresame.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CompareRuntimeIds<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            runtimeid1: *const SAFEARRAY,
            runtimeid2: *const SAFEARRAY,
            aresame: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CompareRuntimeIds(
                    this,
                    core::mem::transmute_copy(&runtimeid1),
                    core::mem::transmute_copy(&runtimeid2),
                ) {
                    Ok(ok__) => {
                        aresame.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetRootElement<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            root: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetRootElement(this) {
                    Ok(ok__) => {
                        root.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromHandle<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwnd: UIA_HWND,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromHandle(this, core::mem::transmute_copy(&hwnd))
                {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromPoint<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pt: POINT,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromPoint(this, core::mem::transmute(&pt)) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetFocusedElement<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetFocusedElement(this) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetRootElementBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            root: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetRootElementBuildCache(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        root.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromHandleBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwnd: UIA_HWND,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromHandleBuildCache(
                    this,
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromPointBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pt: POINT,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromPointBuildCache(
                    this,
                    core::mem::transmute(&pt),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetFocusedElementBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetFocusedElementBuildCache(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateTreeWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pcondition: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateTreeWalker(
                    this,
                    core::mem::transmute_copy(&pcondition),
                ) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ControlViewWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ControlViewWalker(this) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ContentViewWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ContentViewWalker(this) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RawViewWalker<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            walker: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::RawViewWalker(this) {
                    Ok(ok__) => {
                        walker.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RawViewCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::RawViewCondition(this) {
                    Ok(ok__) => {
                        condition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ControlViewCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ControlViewCondition(this) {
                    Ok(ok__) => {
                        condition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ContentViewCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ContentViewCondition(this) {
                    Ok(ok__) => {
                        condition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateCacheRequest<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateCacheRequest(this) {
                    Ok(ok__) => {
                        cacherequest.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateTrueCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateTrueCondition(this) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateFalseCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateFalseCondition(this) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreatePropertyCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            value: VARIANT,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreatePropertyCondition(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute(&value),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreatePropertyConditionEx<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            value: VARIANT,
            flags: PropertyConditionFlags,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreatePropertyConditionEx(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute(&value),
                    core::mem::transmute_copy(&flags),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateAndCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition1: *mut core::ffi::c_void,
            condition2: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateAndCondition(
                    this,
                    core::mem::transmute_copy(&condition1),
                    core::mem::transmute_copy(&condition2),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateAndConditionFromArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const SAFEARRAY,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateAndConditionFromArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateAndConditionFromNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const *mut core::ffi::c_void,
            conditioncount: i32,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateAndConditionFromNativeArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                    core::mem::transmute_copy(&conditioncount),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateOrCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition1: *mut core::ffi::c_void,
            condition2: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateOrCondition(
                    this,
                    core::mem::transmute_copy(&condition1),
                    core::mem::transmute_copy(&condition2),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateOrConditionFromArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const SAFEARRAY,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateOrConditionFromArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateOrConditionFromNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            conditions: *const *mut core::ffi::c_void,
            conditioncount: i32,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateOrConditionFromNativeArray(
                    this,
                    core::mem::transmute_copy(&conditions),
                    core::mem::transmute_copy(&conditioncount),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateNotCondition<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            condition: *mut core::ffi::c_void,
            newcondition: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateNotCondition(
                    this,
                    core::mem::transmute_copy(&condition),
                ) {
                    Ok(ok__) => {
                        newcondition.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn AddAutomationEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            eventid: EVENTID,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddAutomationEventHandler(
                    this,
                    core::mem::transmute_copy(&eventid),
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveAutomationEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            eventid: EVENTID,
            element: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveAutomationEventHandler(
                    this,
                    core::mem::transmute_copy(&eventid),
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddPropertyChangedEventHandlerNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            propertyarray: *const PROPERTYID,
            propertycount: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddPropertyChangedEventHandlerNativeArray(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                    core::mem::transmute_copy(&propertyarray),
                    core::mem::transmute_copy(&propertycount),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddPropertyChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
            propertyarray: *const SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddPropertyChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                    core::mem::transmute_copy(&propertyarray),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemovePropertyChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemovePropertyChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddStructureChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            scope: TreeScope,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddStructureChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveStructureChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            element: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveStructureChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&element),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn AddFocusChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::AddFocusChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveFocusChangedEventHandler<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            handler: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveFocusChangedEventHandler(
                    this,
                    core::mem::transmute_copy(&handler),
                )
                .into()
            }
        }
        unsafe extern "system" fn RemoveAllEventHandlers<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::RemoveAllEventHandlers(this).into()
            }
        }
        unsafe extern "system" fn IntNativeArrayToSafeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            array: *const i32,
            arraycount: i32,
            safearray: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::IntNativeArrayToSafeArray(
                    this,
                    core::mem::transmute_copy(&array),
                    core::mem::transmute_copy(&arraycount),
                ) {
                    Ok(ok__) => {
                        safearray.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn IntSafeArrayToNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            intarray: *const SAFEARRAY,
            array: *mut *mut i32,
            arraycount: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::IntSafeArrayToNativeArray(
                    this,
                    core::mem::transmute_copy(&intarray),
                    core::mem::transmute_copy(&array),
                ) {
                    Ok(ok__) => {
                        arraycount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RectToVariant<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            rc: RECT,
            var: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::RectToVariant(this, core::mem::transmute(&rc)) {
                    Ok(ok__) => {
                        var.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn VariantToRect<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            var: VARIANT,
            rc: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::VariantToRect(this, core::mem::transmute(&var)) {
                    Ok(ok__) => {
                        rc.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SafeArrayToRectNativeArray<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            rects: *const SAFEARRAY,
            rectarray: *mut *mut RECT,
            rectarraycount: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::SafeArrayToRectNativeArray(
                    this,
                    core::mem::transmute_copy(&rects),
                    core::mem::transmute_copy(&rectarray),
                ) {
                    Ok(ok__) => {
                        rectarraycount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateProxyFactoryEntry<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            factory: *mut core::ffi::c_void,
            factoryentry: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CreateProxyFactoryEntry(
                    this,
                    core::mem::transmute_copy(&factory),
                ) {
                    Ok(ok__) => {
                        factoryentry.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ProxyFactoryMapping<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            factorymapping: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ProxyFactoryMapping(this) {
                    Ok(ok__) => {
                        factorymapping.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetPropertyProgrammaticName<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            property: PROPERTYID,
            name: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetPropertyProgrammaticName(
                    this,
                    core::mem::transmute_copy(&property),
                ) {
                    Ok(ok__) => {
                        name.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetPatternProgrammaticName<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pattern: PATTERNID,
            name: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::GetPatternProgrammaticName(
                    this,
                    core::mem::transmute_copy(&pattern),
                ) {
                    Ok(ok__) => {
                        name.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn PollForPotentialSupportedPatterns<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pelement: *mut core::ffi::c_void,
            patternids: *mut *mut SAFEARRAY,
            patternnames: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::PollForPotentialSupportedPatterns(
                    this,
                    core::mem::transmute_copy(&pelement),
                    core::mem::transmute_copy(&patternids),
                    core::mem::transmute_copy(&patternnames),
                )
                .into()
            }
        }
        unsafe extern "system" fn PollForPotentialSupportedProperties<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pelement: *mut core::ffi::c_void,
            propertyids: *mut *mut SAFEARRAY,
            propertynames: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomation_Impl::PollForPotentialSupportedProperties(
                    this,
                    core::mem::transmute_copy(&pelement),
                    core::mem::transmute_copy(&propertyids),
                    core::mem::transmute_copy(&propertynames),
                )
                .into()
            }
        }
        unsafe extern "system" fn CheckNotSupported<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            value: VARIANT,
            isnotsupported: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::CheckNotSupported(this, core::mem::transmute(&value)) {
                    Ok(ok__) => {
                        isnotsupported.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ReservedNotSupportedValue<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            notsupportedvalue: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ReservedNotSupportedValue(this) {
                    Ok(ok__) => {
                        notsupportedvalue.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ReservedMixedAttributeValue<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            mixedattributevalue: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ReservedMixedAttributeValue(this) {
                    Ok(ok__) => {
                        mixedattributevalue.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromIAccessible<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            accessible: *mut core::ffi::c_void,
            childid: i32,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromIAccessible(
                    this,
                    core::mem::transmute_copy(&accessible),
                    core::mem::transmute_copy(&childid),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ElementFromIAccessibleBuildCache<
            Identity: IUIAutomation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            accessible: *mut core::ffi::c_void,
            childid: i32,
            cacherequest: *mut core::ffi::c_void,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomation_Impl::ElementFromIAccessibleBuildCache(
                    this,
                    core::mem::transmute_copy(&accessible),
                    core::mem::transmute_copy(&childid),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            CompareElements: CompareElements::<Identity, OFFSET>,
            CompareRuntimeIds: CompareRuntimeIds::<Identity, OFFSET>,
            GetRootElement: GetRootElement::<Identity, OFFSET>,
            ElementFromHandle: ElementFromHandle::<Identity, OFFSET>,
            ElementFromPoint: ElementFromPoint::<Identity, OFFSET>,
            GetFocusedElement: GetFocusedElement::<Identity, OFFSET>,
            GetRootElementBuildCache: GetRootElementBuildCache::<Identity, OFFSET>,
            ElementFromHandleBuildCache: ElementFromHandleBuildCache::<Identity, OFFSET>,
            ElementFromPointBuildCache: ElementFromPointBuildCache::<Identity, OFFSET>,
            GetFocusedElementBuildCache: GetFocusedElementBuildCache::<Identity, OFFSET>,
            CreateTreeWalker: CreateTreeWalker::<Identity, OFFSET>,
            ControlViewWalker: ControlViewWalker::<Identity, OFFSET>,
            ContentViewWalker: ContentViewWalker::<Identity, OFFSET>,
            RawViewWalker: RawViewWalker::<Identity, OFFSET>,
            RawViewCondition: RawViewCondition::<Identity, OFFSET>,
            ControlViewCondition: ControlViewCondition::<Identity, OFFSET>,
            ContentViewCondition: ContentViewCondition::<Identity, OFFSET>,
            CreateCacheRequest: CreateCacheRequest::<Identity, OFFSET>,
            CreateTrueCondition: CreateTrueCondition::<Identity, OFFSET>,
            CreateFalseCondition: CreateFalseCondition::<Identity, OFFSET>,
            CreatePropertyCondition: CreatePropertyCondition::<Identity, OFFSET>,
            CreatePropertyConditionEx: CreatePropertyConditionEx::<Identity, OFFSET>,
            CreateAndCondition: CreateAndCondition::<Identity, OFFSET>,
            CreateAndConditionFromArray: CreateAndConditionFromArray::<Identity, OFFSET>,
            CreateAndConditionFromNativeArray: CreateAndConditionFromNativeArray::<Identity, OFFSET>,
            CreateOrCondition: CreateOrCondition::<Identity, OFFSET>,
            CreateOrConditionFromArray: CreateOrConditionFromArray::<Identity, OFFSET>,
            CreateOrConditionFromNativeArray: CreateOrConditionFromNativeArray::<Identity, OFFSET>,
            CreateNotCondition: CreateNotCondition::<Identity, OFFSET>,
            AddAutomationEventHandler: AddAutomationEventHandler::<Identity, OFFSET>,
            RemoveAutomationEventHandler: RemoveAutomationEventHandler::<Identity, OFFSET>,
            AddPropertyChangedEventHandlerNativeArray: AddPropertyChangedEventHandlerNativeArray::<
                Identity,
                OFFSET,
            >,
            AddPropertyChangedEventHandler: AddPropertyChangedEventHandler::<Identity, OFFSET>,
            RemovePropertyChangedEventHandler: RemovePropertyChangedEventHandler::<Identity, OFFSET>,
            AddStructureChangedEventHandler: AddStructureChangedEventHandler::<Identity, OFFSET>,
            RemoveStructureChangedEventHandler: RemoveStructureChangedEventHandler::<
                Identity,
                OFFSET,
            >,
            AddFocusChangedEventHandler: AddFocusChangedEventHandler::<Identity, OFFSET>,
            RemoveFocusChangedEventHandler: RemoveFocusChangedEventHandler::<Identity, OFFSET>,
            RemoveAllEventHandlers: RemoveAllEventHandlers::<Identity, OFFSET>,
            IntNativeArrayToSafeArray: IntNativeArrayToSafeArray::<Identity, OFFSET>,
            IntSafeArrayToNativeArray: IntSafeArrayToNativeArray::<Identity, OFFSET>,
            RectToVariant: RectToVariant::<Identity, OFFSET>,
            VariantToRect: VariantToRect::<Identity, OFFSET>,
            SafeArrayToRectNativeArray: SafeArrayToRectNativeArray::<Identity, OFFSET>,
            CreateProxyFactoryEntry: CreateProxyFactoryEntry::<Identity, OFFSET>,
            ProxyFactoryMapping: ProxyFactoryMapping::<Identity, OFFSET>,
            GetPropertyProgrammaticName: GetPropertyProgrammaticName::<Identity, OFFSET>,
            GetPatternProgrammaticName: GetPatternProgrammaticName::<Identity, OFFSET>,
            PollForPotentialSupportedPatterns: PollForPotentialSupportedPatterns::<Identity, OFFSET>,
            PollForPotentialSupportedProperties: PollForPotentialSupportedProperties::<
                Identity,
                OFFSET,
            >,
            CheckNotSupported: CheckNotSupported::<Identity, OFFSET>,
            ReservedNotSupportedValue: ReservedNotSupportedValue::<Identity, OFFSET>,
            ReservedMixedAttributeValue: ReservedMixedAttributeValue::<Identity, OFFSET>,
            ElementFromIAccessible: ElementFromIAccessible::<Identity, OFFSET>,
            ElementFromIAccessibleBuildCache: ElementFromIAccessibleBuildCache::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomation as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomation {}
windows_core::imp::define_interface!(
    IUIAutomationCacheRequest,
    IUIAutomationCacheRequest_Vtbl,
    0xb32a92b5_bc25_4078_9c08_d7ee95c48e03
);
windows_core::imp::interface_hierarchy!(IUIAutomationCacheRequest, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationCacheRequest_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    AddProperty: usize,
    AddPattern: usize,
    Clone: usize,
    TreeScope: usize,
    SetTreeScope: usize,
    TreeFilter: usize,
    SetTreeFilter: usize,
    AutomationElementMode: usize,
    SetAutomationElementMode: usize,
}
impl windows_core::RuntimeName for IUIAutomationCacheRequest {}
windows_core::imp::define_interface!(
    IUIAutomationCondition,
    IUIAutomationCondition_Vtbl,
    0x352ffba8_0973_437c_a61f_f64cafd81df9
);
windows_core::imp::interface_hierarchy!(IUIAutomationCondition, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationCondition_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
}
pub trait IUIAutomationCondition_Impl: windows_core::IUnknownImpl {}
impl IUIAutomationCondition_Vtbl {
    pub const fn new<Identity: IUIAutomationCondition_Impl, const OFFSET: isize>() -> Self {
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationCondition as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationCondition {}
windows_core::imp::define_interface!(
    IUIAutomationElement,
    IUIAutomationElement_Vtbl,
    0xd22108aa_8ac5_49a5_837b_37bbb3d7591e
);
windows_core::imp::interface_hierarchy!(IUIAutomationElement, windows_core::IUnknown);
impl IUIAutomationElement {
    pub(crate) unsafe fn SetFocus(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetFocus)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn GetRuntimeId(&self) -> windows_core::Result<*mut SAFEARRAY> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetRuntimeId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn FindFirst<P1>(
        &self,
        scope: TreeScope,
        condition: P1,
    ) -> windows_core::Result<Self>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindFirst)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindAll<P1>(
        &self,
        scope: TreeScope,
        condition: P1,
    ) -> windows_core::Result<IUIAutomationElementArray>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAll)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindFirstBuildCache<P1, P2>(
        &self,
        scope: TreeScope,
        condition: P1,
        cacherequest: P2,
    ) -> windows_core::Result<Self>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindFirstBuildCache)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindAllBuildCache<P1, P2>(
        &self,
        scope: TreeScope,
        condition: P1,
        cacherequest: P2,
    ) -> windows_core::Result<IUIAutomationElementArray>
    where
        P1: windows_core::Param<IUIAutomationCondition>,
        P2: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAllBuildCache)(
                windows_core::Interface::as_raw(self),
                scope,
                condition.param().abi(),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn BuildUpdatedCache<P0>(
        &self,
        cacherequest: P0,
    ) -> windows_core::Result<Self>
    where
        P0: windows_core::Param<IUIAutomationCacheRequest>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).BuildUpdatedCache)(
                windows_core::Interface::as_raw(self),
                cacherequest.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPropertyValue(
        &self,
        propertyid: PROPERTYID,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentPropertyValue)(
                windows_core::Interface::as_raw(self),
                propertyid,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: bool,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentPropertyValueEx)(
                windows_core::Interface::as_raw(self),
                propertyid,
                ignoredefaultvalue.into(),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPropertyValue(
        &self,
        propertyid: PROPERTYID,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedPropertyValue)(
                windows_core::Interface::as_raw(self),
                propertyid,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: bool,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedPropertyValueEx)(
                windows_core::Interface::as_raw(self),
                propertyid,
                ignoredefaultvalue.into(),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPatternAs<T>(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetCurrentPatternAs)(
                windows_core::Interface::as_raw(self),
                patternid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPatternAs<T>(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetCachedPatternAs)(
                windows_core::Interface::as_raw(self),
                patternid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentPattern)(
                windows_core::Interface::as_raw(self),
                patternid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedPattern)(
                windows_core::Interface::as_raw(self),
                patternid,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedParent(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedParent)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCachedChildren(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCachedChildren)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentProcessId(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentProcessId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentControlType(&self) -> windows_core::Result<CONTROLTYPEID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentLocalizedControlType(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentLocalizedControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAcceleratorKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentAccessKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAccessKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentHasKeyboardFocus(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentHasKeyboardFocus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsKeyboardFocusable(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsKeyboardFocusable)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsEnabled(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentAutomationId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAutomationId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentClassName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentClassName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentHelpText(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentHelpText)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentCulture(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentCulture)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsControlElement(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsControlElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsContentElement(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsContentElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentIsPassword(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsPassword)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentNativeWindowHandle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentItemType(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentItemType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsOffscreen)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentOrientation(&self) -> windows_core::Result<OrientationType> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentOrientation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentFrameworkId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentFrameworkId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsRequiredForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsRequiredForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentItemStatus(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentItemStatus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentBoundingRectangle(&self) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentBoundingRectangle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentLabeledBy(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentLabeledBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentAriaRole(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAriaRole)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentAriaProperties(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentAriaProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CurrentIsDataValidForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentIsDataValidForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CurrentControllerFor(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentControllerFor)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentDescribedBy(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentDescribedBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentFlowsTo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CurrentProviderDescription(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CurrentProviderDescription)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedProcessId(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedProcessId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedControlType(&self) -> windows_core::Result<CONTROLTYPEID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedLocalizedControlType(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedLocalizedControlType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAcceleratorKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedAccessKey(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAccessKey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedHasKeyboardFocus(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedHasKeyboardFocus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsKeyboardFocusable(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsKeyboardFocusable)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsEnabled(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsEnabled)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedAutomationId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAutomationId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedClassName(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedClassName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedHelpText(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedHelpText)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedCulture(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedCulture)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsControlElement(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsControlElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsContentElement(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsContentElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedIsPassword(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsPassword)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedNativeWindowHandle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedItemType(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedItemType)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsOffscreen)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedOrientation(&self) -> windows_core::Result<OrientationType> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedOrientation)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedFrameworkId(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedFrameworkId)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsRequiredForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsRequiredForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedItemStatus(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedItemStatus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedBoundingRectangle(&self) -> windows_core::Result<RECT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedBoundingRectangle)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedLabeledBy(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedLabeledBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedAriaRole(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAriaRole)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedAriaProperties(&self) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedAriaProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn CachedIsDataValidForForm(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedIsDataValidForForm)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CachedControllerFor(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedControllerFor)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedDescribedBy(
        &self,
    ) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedDescribedBy)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedFlowsTo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CachedProviderDescription(
        &self,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CachedProviderDescription)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetClickablePoint(
        &self,
        clickable: *mut POINT,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetClickablePoint)(
                windows_core::Interface::as_raw(self),
                clickable as _,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IUIAutomationElement_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub SetFocus: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetRuntimeId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub FindFirst: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindAll: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindFirstBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindAllBuildCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TreeScope,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub BuildUpdatedCache: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCurrentPropertyValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCurrentPropertyValueEx: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        windows_core::BOOL,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCachedPropertyValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCachedPropertyValueEx: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PROPERTYID,
        windows_core::BOOL,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetCurrentPatternAs: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedPatternAs: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCurrentPattern: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedPattern: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        PATTERNID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedParent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCachedChildren: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentProcessId:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CurrentControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CONTROLTYPEID,
    ) -> windows_core::HRESULT,
    pub CurrentLocalizedControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAcceleratorKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAccessKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentHasKeyboardFocus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsKeyboardFocusable: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsEnabled: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentAutomationId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentClassName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentHelpText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentCulture:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CurrentIsControlElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsContentElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentIsPassword: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentNativeWindowHandle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut UIA_HWND) -> windows_core::HRESULT,
    pub CurrentItemType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsOffscreen: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentOrientation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut OrientationType,
    ) -> windows_core::HRESULT,
    pub CurrentFrameworkId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsRequiredForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentItemStatus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentBoundingRectangle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut RECT) -> windows_core::HRESULT,
    pub CurrentLabeledBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAriaRole: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentAriaProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentIsDataValidForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CurrentControllerFor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentDescribedBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentFlowsTo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CurrentProviderDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedProcessId:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CachedControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut CONTROLTYPEID,
    ) -> windows_core::HRESULT,
    pub CachedLocalizedControlType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAcceleratorKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAccessKey: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedHasKeyboardFocus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsKeyboardFocusable: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsEnabled: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedAutomationId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedClassName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedHelpText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedCulture:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub CachedIsControlElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsContentElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedIsPassword: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedNativeWindowHandle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut UIA_HWND) -> windows_core::HRESULT,
    pub CachedItemType: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsOffscreen: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedOrientation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut OrientationType,
    ) -> windows_core::HRESULT,
    pub CachedFrameworkId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsRequiredForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedItemStatus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedBoundingRectangle:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut RECT) -> windows_core::HRESULT,
    pub CachedLabeledBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAriaRole: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedAriaProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedIsDataValidForForm: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CachedControllerFor: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedDescribedBy: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedFlowsTo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CachedProviderDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetClickablePoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut POINT,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationElement_Impl: windows_core::IUnknownImpl {
    fn SetFocus(&self) -> windows_core::Result<()>;
    fn GetRuntimeId(&self) -> windows_core::Result<*mut SAFEARRAY>;
    fn FindFirst(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn FindAll(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
    ) -> windows_core::Result<IUIAutomationElementArray>;
    fn FindFirstBuildCache(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn FindAllBuildCache(
        &self,
        scope: TreeScope,
        condition: windows_core::Ref<IUIAutomationCondition>,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElementArray>;
    fn BuildUpdatedCache(
        &self,
        cacherequest: windows_core::Ref<IUIAutomationCacheRequest>,
    ) -> windows_core::Result<IUIAutomationElement>;
    fn GetCurrentPropertyValue(&self, propertyid: PROPERTYID) -> windows_core::Result<VARIANT>;
    fn GetCurrentPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: windows_core::BOOL,
    ) -> windows_core::Result<VARIANT>;
    fn GetCachedPropertyValue(&self, propertyid: PROPERTYID) -> windows_core::Result<VARIANT>;
    fn GetCachedPropertyValueEx(
        &self,
        propertyid: PROPERTYID,
        ignoredefaultvalue: windows_core::BOOL,
    ) -> windows_core::Result<VARIANT>;
    fn GetCurrentPatternAs(
        &self,
        patternid: PATTERNID,
        riid: *const windows_core::GUID,
        patternobject: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetCachedPatternAs(
        &self,
        patternid: PATTERNID,
        riid: *const windows_core::GUID,
        patternobject: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetCurrentPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown>;
    fn GetCachedPattern(
        &self,
        patternid: PATTERNID,
    ) -> windows_core::Result<windows_core::IUnknown>;
    fn GetCachedParent(&self) -> windows_core::Result<IUIAutomationElement>;
    fn GetCachedChildren(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentProcessId(&self) -> windows_core::Result<i32>;
    fn CurrentControlType(&self) -> windows_core::Result<CONTROLTYPEID>;
    fn CurrentLocalizedControlType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentAccessKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentHasKeyboardFocus(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsKeyboardFocusable(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsEnabled(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentAutomationId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentClassName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentHelpText(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentCulture(&self) -> windows_core::Result<i32>;
    fn CurrentIsControlElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsContentElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentIsPassword(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND>;
    fn CurrentItemType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentOrientation(&self) -> windows_core::Result<OrientationType>;
    fn CurrentFrameworkId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsRequiredForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentItemStatus(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentBoundingRectangle(&self) -> windows_core::Result<RECT>;
    fn CurrentLabeledBy(&self) -> windows_core::Result<IUIAutomationElement>;
    fn CurrentAriaRole(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentAriaProperties(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CurrentIsDataValidForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CurrentControllerFor(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentDescribedBy(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CurrentProviderDescription(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedProcessId(&self) -> windows_core::Result<i32>;
    fn CachedControlType(&self) -> windows_core::Result<CONTROLTYPEID>;
    fn CachedLocalizedControlType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedAcceleratorKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedAccessKey(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedHasKeyboardFocus(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsKeyboardFocusable(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsEnabled(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedAutomationId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedClassName(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedHelpText(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedCulture(&self) -> windows_core::Result<i32>;
    fn CachedIsControlElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsContentElement(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedIsPassword(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedNativeWindowHandle(&self) -> windows_core::Result<UIA_HWND>;
    fn CachedItemType(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsOffscreen(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedOrientation(&self) -> windows_core::Result<OrientationType>;
    fn CachedFrameworkId(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsRequiredForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedItemStatus(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedBoundingRectangle(&self) -> windows_core::Result<RECT>;
    fn CachedLabeledBy(&self) -> windows_core::Result<IUIAutomationElement>;
    fn CachedAriaRole(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedAriaProperties(&self) -> windows_core::Result<windows_core::BSTR>;
    fn CachedIsDataValidForForm(&self) -> windows_core::Result<windows_core::BOOL>;
    fn CachedControllerFor(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CachedDescribedBy(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CachedFlowsTo(&self) -> windows_core::Result<IUIAutomationElementArray>;
    fn CachedProviderDescription(&self) -> windows_core::Result<windows_core::BSTR>;
    fn GetClickablePoint(&self, clickable: *mut POINT) -> windows_core::Result<windows_core::BOOL>;
}
impl IUIAutomationElement_Vtbl {
    pub const fn new<Identity: IUIAutomationElement_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetFocus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationElement_Impl::SetFocus(this).into()
            }
        }
        unsafe extern "system" fn GetRuntimeId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            runtimeid: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetRuntimeId(this) {
                    Ok(ok__) => {
                        runtimeid.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindFirst<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindFirst(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindAll<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindAll(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindFirstBuildCache<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindFirstBuildCache(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindAllBuildCache<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            scope: TreeScope,
            condition: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::FindAllBuildCache(
                    this,
                    core::mem::transmute_copy(&scope),
                    core::mem::transmute_copy(&condition),
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn BuildUpdatedCache<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cacherequest: *mut core::ffi::c_void,
            updatedelement: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::BuildUpdatedCache(
                    this,
                    core::mem::transmute_copy(&cacherequest),
                ) {
                    Ok(ok__) => {
                        updatedelement.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentPropertyValue<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCurrentPropertyValue(
                    this,
                    core::mem::transmute_copy(&propertyid),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentPropertyValueEx<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            ignoredefaultvalue: windows_core::BOOL,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCurrentPropertyValueEx(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute_copy(&ignoredefaultvalue),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedPropertyValue<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedPropertyValue(
                    this,
                    core::mem::transmute_copy(&propertyid),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedPropertyValueEx<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            propertyid: PROPERTYID,
            ignoredefaultvalue: windows_core::BOOL,
            retval: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedPropertyValueEx(
                    this,
                    core::mem::transmute_copy(&propertyid),
                    core::mem::transmute_copy(&ignoredefaultvalue),
                ) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentPatternAs<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            riid: *const windows_core::GUID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationElement_Impl::GetCurrentPatternAs(
                    this,
                    core::mem::transmute_copy(&patternid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&patternobject),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetCachedPatternAs<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            riid: *const windows_core::GUID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationElement_Impl::GetCachedPatternAs(
                    this,
                    core::mem::transmute_copy(&patternid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&patternobject),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetCurrentPattern<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCurrentPattern(
                    this,
                    core::mem::transmute_copy(&patternid),
                ) {
                    Ok(ok__) => {
                        patternobject.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedPattern<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            patternid: PATTERNID,
            patternobject: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedPattern(
                    this,
                    core::mem::transmute_copy(&patternid),
                ) {
                    Ok(ok__) => {
                        patternobject.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedParent<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            parent: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedParent(this) {
                    Ok(ok__) => {
                        parent.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCachedChildren<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            children: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetCachedChildren(this) {
                    Ok(ok__) => {
                        children.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentProcessId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentProcessId(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut CONTROLTYPEID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentControlType(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentLocalizedControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentLocalizedControlType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAcceleratorKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAcceleratorKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAccessKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAccessKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentHasKeyboardFocus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentHasKeyboardFocus(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsKeyboardFocusable<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsKeyboardFocusable(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsEnabled<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsEnabled(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAutomationId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAutomationId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentClassName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentClassName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentHelpText<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentHelpText(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentCulture<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentCulture(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsControlElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsControlElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsContentElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsContentElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsPassword<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsPassword(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentNativeWindowHandle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut UIA_HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentNativeWindowHandle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentItemType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentItemType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsOffscreen<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsOffscreen(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentOrientation<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut OrientationType,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentOrientation(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentFrameworkId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentFrameworkId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsRequiredForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsRequiredForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentItemStatus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentItemStatus(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentBoundingRectangle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentBoundingRectangle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentLabeledBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentLabeledBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAriaRole<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAriaRole(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentAriaProperties<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentAriaProperties(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentIsDataValidForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentIsDataValidForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentControllerFor<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentControllerFor(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentDescribedBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentDescribedBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentFlowsTo<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentFlowsTo(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CurrentProviderDescription<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CurrentProviderDescription(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedProcessId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedProcessId(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut CONTROLTYPEID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedControlType(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedLocalizedControlType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedLocalizedControlType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAcceleratorKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAcceleratorKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAccessKey<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAccessKey(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedHasKeyboardFocus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedHasKeyboardFocus(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsKeyboardFocusable<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsKeyboardFocusable(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsEnabled<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsEnabled(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAutomationId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAutomationId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedClassName<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedClassName(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedHelpText<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedHelpText(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedCulture<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedCulture(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsControlElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsControlElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsContentElement<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsContentElement(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsPassword<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsPassword(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedNativeWindowHandle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut UIA_HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedNativeWindowHandle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedItemType<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedItemType(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsOffscreen<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsOffscreen(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedOrientation<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut OrientationType,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedOrientation(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedFrameworkId<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedFrameworkId(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsRequiredForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsRequiredForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedItemStatus<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedItemStatus(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedBoundingRectangle<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedBoundingRectangle(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedLabeledBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedLabeledBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAriaRole<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAriaRole(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedAriaProperties<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedAriaProperties(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedIsDataValidForForm<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedIsDataValidForForm(this) {
                    Ok(ok__) => {
                        retval.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedControllerFor<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedControllerFor(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedDescribedBy<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedDescribedBy(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedFlowsTo<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedFlowsTo(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CachedProviderDescription<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            retval: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::CachedProviderDescription(this) {
                    Ok(ok__) => {
                        retval.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetClickablePoint<
            Identity: IUIAutomationElement_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clickable: *mut POINT,
            gotclickable: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationElement_Impl::GetClickablePoint(
                    this,
                    core::mem::transmute_copy(&clickable),
                ) {
                    Ok(ok__) => {
                        gotclickable.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            SetFocus: SetFocus::<Identity, OFFSET>,
            GetRuntimeId: GetRuntimeId::<Identity, OFFSET>,
            FindFirst: FindFirst::<Identity, OFFSET>,
            FindAll: FindAll::<Identity, OFFSET>,
            FindFirstBuildCache: FindFirstBuildCache::<Identity, OFFSET>,
            FindAllBuildCache: FindAllBuildCache::<Identity, OFFSET>,
            BuildUpdatedCache: BuildUpdatedCache::<Identity, OFFSET>,
            GetCurrentPropertyValue: GetCurrentPropertyValue::<Identity, OFFSET>,
            GetCurrentPropertyValueEx: GetCurrentPropertyValueEx::<Identity, OFFSET>,
            GetCachedPropertyValue: GetCachedPropertyValue::<Identity, OFFSET>,
            GetCachedPropertyValueEx: GetCachedPropertyValueEx::<Identity, OFFSET>,
            GetCurrentPatternAs: GetCurrentPatternAs::<Identity, OFFSET>,
            GetCachedPatternAs: GetCachedPatternAs::<Identity, OFFSET>,
            GetCurrentPattern: GetCurrentPattern::<Identity, OFFSET>,
            GetCachedPattern: GetCachedPattern::<Identity, OFFSET>,
            GetCachedParent: GetCachedParent::<Identity, OFFSET>,
            GetCachedChildren: GetCachedChildren::<Identity, OFFSET>,
            CurrentProcessId: CurrentProcessId::<Identity, OFFSET>,
            CurrentControlType: CurrentControlType::<Identity, OFFSET>,
            CurrentLocalizedControlType: CurrentLocalizedControlType::<Identity, OFFSET>,
            CurrentName: CurrentName::<Identity, OFFSET>,
            CurrentAcceleratorKey: CurrentAcceleratorKey::<Identity, OFFSET>,
            CurrentAccessKey: CurrentAccessKey::<Identity, OFFSET>,
            CurrentHasKeyboardFocus: CurrentHasKeyboardFocus::<Identity, OFFSET>,
            CurrentIsKeyboardFocusable: CurrentIsKeyboardFocusable::<Identity, OFFSET>,
            CurrentIsEnabled: CurrentIsEnabled::<Identity, OFFSET>,
            CurrentAutomationId: CurrentAutomationId::<Identity, OFFSET>,
            CurrentClassName: CurrentClassName::<Identity, OFFSET>,
            CurrentHelpText: CurrentHelpText::<Identity, OFFSET>,
            CurrentCulture: CurrentCulture::<Identity, OFFSET>,
            CurrentIsControlElement: CurrentIsControlElement::<Identity, OFFSET>,
            CurrentIsContentElement: CurrentIsContentElement::<Identity, OFFSET>,
            CurrentIsPassword: CurrentIsPassword::<Identity, OFFSET>,
            CurrentNativeWindowHandle: CurrentNativeWindowHandle::<Identity, OFFSET>,
            CurrentItemType: CurrentItemType::<Identity, OFFSET>,
            CurrentIsOffscreen: CurrentIsOffscreen::<Identity, OFFSET>,
            CurrentOrientation: CurrentOrientation::<Identity, OFFSET>,
            CurrentFrameworkId: CurrentFrameworkId::<Identity, OFFSET>,
            CurrentIsRequiredForForm: CurrentIsRequiredForForm::<Identity, OFFSET>,
            CurrentItemStatus: CurrentItemStatus::<Identity, OFFSET>,
            CurrentBoundingRectangle: CurrentBoundingRectangle::<Identity, OFFSET>,
            CurrentLabeledBy: CurrentLabeledBy::<Identity, OFFSET>,
            CurrentAriaRole: CurrentAriaRole::<Identity, OFFSET>,
            CurrentAriaProperties: CurrentAriaProperties::<Identity, OFFSET>,
            CurrentIsDataValidForForm: CurrentIsDataValidForForm::<Identity, OFFSET>,
            CurrentControllerFor: CurrentControllerFor::<Identity, OFFSET>,
            CurrentDescribedBy: CurrentDescribedBy::<Identity, OFFSET>,
            CurrentFlowsTo: CurrentFlowsTo::<Identity, OFFSET>,
            CurrentProviderDescription: CurrentProviderDescription::<Identity, OFFSET>,
            CachedProcessId: CachedProcessId::<Identity, OFFSET>,
            CachedControlType: CachedControlType::<Identity, OFFSET>,
            CachedLocalizedControlType: CachedLocalizedControlType::<Identity, OFFSET>,
            CachedName: CachedName::<Identity, OFFSET>,
            CachedAcceleratorKey: CachedAcceleratorKey::<Identity, OFFSET>,
            CachedAccessKey: CachedAccessKey::<Identity, OFFSET>,
            CachedHasKeyboardFocus: CachedHasKeyboardFocus::<Identity, OFFSET>,
            CachedIsKeyboardFocusable: CachedIsKeyboardFocusable::<Identity, OFFSET>,
            CachedIsEnabled: CachedIsEnabled::<Identity, OFFSET>,
            CachedAutomationId: CachedAutomationId::<Identity, OFFSET>,
            CachedClassName: CachedClassName::<Identity, OFFSET>,
            CachedHelpText: CachedHelpText::<Identity, OFFSET>,
            CachedCulture: CachedCulture::<Identity, OFFSET>,
            CachedIsControlElement: CachedIsControlElement::<Identity, OFFSET>,
            CachedIsContentElement: CachedIsContentElement::<Identity, OFFSET>,
            CachedIsPassword: CachedIsPassword::<Identity, OFFSET>,
            CachedNativeWindowHandle: CachedNativeWindowHandle::<Identity, OFFSET>,
            CachedItemType: CachedItemType::<Identity, OFFSET>,
            CachedIsOffscreen: CachedIsOffscreen::<Identity, OFFSET>,
            CachedOrientation: CachedOrientation::<Identity, OFFSET>,
            CachedFrameworkId: CachedFrameworkId::<Identity, OFFSET>,
            CachedIsRequiredForForm: CachedIsRequiredForForm::<Identity, OFFSET>,
            CachedItemStatus: CachedItemStatus::<Identity, OFFSET>,
            CachedBoundingRectangle: CachedBoundingRectangle::<Identity, OFFSET>,
            CachedLabeledBy: CachedLabeledBy::<Identity, OFFSET>,
            CachedAriaRole: CachedAriaRole::<Identity, OFFSET>,
            CachedAriaProperties: CachedAriaProperties::<Identity, OFFSET>,
            CachedIsDataValidForForm: CachedIsDataValidForForm::<Identity, OFFSET>,
            CachedControllerFor: CachedControllerFor::<Identity, OFFSET>,
            CachedDescribedBy: CachedDescribedBy::<Identity, OFFSET>,
            CachedFlowsTo: CachedFlowsTo::<Identity, OFFSET>,
            CachedProviderDescription: CachedProviderDescription::<Identity, OFFSET>,
            GetClickablePoint: GetClickablePoint::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationElement as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationElement {}
windows_core::imp::define_interface!(
    IUIAutomationElementArray,
    IUIAutomationElementArray_Vtbl,
    0x14314595_b4bc_4055_95f2_58f2e42c9855
);
windows_core::imp::interface_hierarchy!(IUIAutomationElementArray, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationElementArray_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Length: usize,
    GetElement: usize,
}
impl windows_core::RuntimeName for IUIAutomationElementArray {}
windows_core::imp::define_interface!(
    IUIAutomationEventHandler,
    IUIAutomationEventHandler_Vtbl,
    0x146c3c17_f12e_4e22_8c27_f894b9b79c69
);
windows_core::imp::interface_hierarchy!(IUIAutomationEventHandler, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandleAutomationEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationFocusChangedEventHandler,
    IUIAutomationFocusChangedEventHandler_Vtbl,
    0xc270f6b5_5c69_4290_9745_7a7f97169468
);
windows_core::imp::interface_hierarchy!(
    IUIAutomationFocusChangedEventHandler,
    windows_core::IUnknown
);
#[repr(C)]
pub struct IUIAutomationFocusChangedEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandleFocusChangedEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationFocusChangedEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationPropertyChangedEventHandler,
    IUIAutomationPropertyChangedEventHandler_Vtbl,
    0x40cd37d4_c756_4b0c_8c6f_bddfeeb13b50
);
windows_core::imp::interface_hierarchy!(
    IUIAutomationPropertyChangedEventHandler,
    windows_core::IUnknown
);
#[repr(C)]
pub struct IUIAutomationPropertyChangedEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandlePropertyChangedEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationPropertyChangedEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationProxyFactory,
    IUIAutomationProxyFactory_Vtbl,
    0x85b94ecd_849d_42b6_b94d_d6db23fdf5a4
);
windows_core::imp::interface_hierarchy!(IUIAutomationProxyFactory, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationProxyFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    CreateProvider: usize,
    ProxyFactoryId: usize,
}
impl windows_core::RuntimeName for IUIAutomationProxyFactory {}
windows_core::imp::define_interface!(
    IUIAutomationProxyFactoryEntry,
    IUIAutomationProxyFactoryEntry_Vtbl,
    0xd50e472e_b64b_490c_bca1_d30696f9f289
);
windows_core::imp::interface_hierarchy!(IUIAutomationProxyFactoryEntry, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationProxyFactoryEntry_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    ProxyFactory: usize,
    ClassName: usize,
    ImageName: usize,
    AllowSubstringMatch: usize,
    CanCheckBaseClass: usize,
    NeedsAdviseEvents: usize,
    SetClassName: usize,
    SetImageName: usize,
    SetAllowSubstringMatch: usize,
    SetCanCheckBaseClass: usize,
    SetNeedsAdviseEvents: usize,
    SetWinEventsForAutomationEvent: usize,
    GetWinEventsForAutomationEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationProxyFactoryEntry {}
windows_core::imp::define_interface!(
    IUIAutomationProxyFactoryMapping,
    IUIAutomationProxyFactoryMapping_Vtbl,
    0x09e31e18_872d_4873_93d1_1e541ec133fd
);
windows_core::imp::interface_hierarchy!(IUIAutomationProxyFactoryMapping, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationProxyFactoryMapping_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Count: usize,
    GetTable: usize,
    GetEntry: usize,
    SetTable: usize,
    InsertEntries: usize,
    InsertEntry: usize,
    RemoveEntry: usize,
    ClearTable: usize,
    RestoreDefaultTable: usize,
}
impl windows_core::RuntimeName for IUIAutomationProxyFactoryMapping {}
windows_core::imp::define_interface!(
    IUIAutomationStructureChangedEventHandler,
    IUIAutomationStructureChangedEventHandler_Vtbl,
    0xe81d1b4e_11c5_42f8_9754_e7036c79f054
);
windows_core::imp::interface_hierarchy!(
    IUIAutomationStructureChangedEventHandler,
    windows_core::IUnknown
);
#[repr(C)]
pub struct IUIAutomationStructureChangedEventHandler_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    HandleStructureChangedEvent: usize,
}
impl windows_core::RuntimeName for IUIAutomationStructureChangedEventHandler {}
windows_core::imp::define_interface!(
    IUIAutomationTextPattern,
    IUIAutomationTextPattern_Vtbl,
    0x32eba289_3583_42c9_9c59_3b6d9a1e9b6a
);
windows_core::imp::interface_hierarchy!(IUIAutomationTextPattern, windows_core::IUnknown);
impl IUIAutomationTextPattern {
    pub(crate) unsafe fn RangeFromPoint(
        &self,
        pt: POINT,
    ) -> windows_core::Result<IUIAutomationTextRange> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RangeFromPoint)(
                windows_core::Interface::as_raw(self),
                pt,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn RangeFromChild<P0>(
        &self,
        child: P0,
    ) -> windows_core::Result<IUIAutomationTextRange>
    where
        P0: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RangeFromChild)(
                windows_core::Interface::as_raw(self),
                child.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetSelection(&self) -> windows_core::Result<IUIAutomationTextRangeArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSelection)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetVisibleRanges(
        &self,
    ) -> windows_core::Result<IUIAutomationTextRangeArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetVisibleRanges)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn DocumentRange(&self) -> windows_core::Result<IUIAutomationTextRange> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DocumentRange)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn SupportedTextSelection(
        &self,
    ) -> windows_core::Result<SupportedTextSelection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SupportedTextSelection)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IUIAutomationTextPattern_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub RangeFromPoint: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        POINT,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RangeFromChild: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetSelection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetVisibleRanges: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DocumentRange: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SupportedTextSelection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut SupportedTextSelection,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationTextPattern_Impl: windows_core::IUnknownImpl {
    fn RangeFromPoint(&self, pt: &POINT) -> windows_core::Result<IUIAutomationTextRange>;
    fn RangeFromChild(
        &self,
        child: windows_core::Ref<IUIAutomationElement>,
    ) -> windows_core::Result<IUIAutomationTextRange>;
    fn GetSelection(&self) -> windows_core::Result<IUIAutomationTextRangeArray>;
    fn GetVisibleRanges(&self) -> windows_core::Result<IUIAutomationTextRangeArray>;
    fn DocumentRange(&self) -> windows_core::Result<IUIAutomationTextRange>;
    fn SupportedTextSelection(&self) -> windows_core::Result<SupportedTextSelection>;
}
impl IUIAutomationTextPattern_Vtbl {
    pub const fn new<Identity: IUIAutomationTextPattern_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn RangeFromPoint<
            Identity: IUIAutomationTextPattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pt: POINT,
            range: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern_Impl::RangeFromPoint(this, core::mem::transmute(&pt))
                {
                    Ok(ok__) => {
                        range.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn RangeFromChild<
            Identity: IUIAutomationTextPattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            child: *mut core::ffi::c_void,
            range: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern_Impl::RangeFromChild(
                    this,
                    core::mem::transmute_copy(&child),
                ) {
                    Ok(ok__) => {
                        range.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetSelection<
            Identity: IUIAutomationTextPattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ranges: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern_Impl::GetSelection(this) {
                    Ok(ok__) => {
                        ranges.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetVisibleRanges<
            Identity: IUIAutomationTextPattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ranges: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern_Impl::GetVisibleRanges(this) {
                    Ok(ok__) => {
                        ranges.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn DocumentRange<
            Identity: IUIAutomationTextPattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            range: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern_Impl::DocumentRange(this) {
                    Ok(ok__) => {
                        range.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SupportedTextSelection<
            Identity: IUIAutomationTextPattern_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            supportedtextselection: *mut SupportedTextSelection,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern_Impl::SupportedTextSelection(this) {
                    Ok(ok__) => {
                        supportedtextselection.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            RangeFromPoint: RangeFromPoint::<Identity, OFFSET>,
            RangeFromChild: RangeFromChild::<Identity, OFFSET>,
            GetSelection: GetSelection::<Identity, OFFSET>,
            GetVisibleRanges: GetVisibleRanges::<Identity, OFFSET>,
            DocumentRange: DocumentRange::<Identity, OFFSET>,
            SupportedTextSelection: SupportedTextSelection::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationTextPattern as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationTextPattern {}
windows_core::imp::define_interface!(
    IUIAutomationTextPattern2,
    IUIAutomationTextPattern2_Vtbl,
    0x506a921a_fcc9_409f_b23b_37eb74106872
);
impl core::ops::Deref for IUIAutomationTextPattern2 {
    type Target = IUIAutomationTextPattern;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IUIAutomationTextPattern2,
    windows_core::IUnknown,
    IUIAutomationTextPattern
);
impl IUIAutomationTextPattern2 {
    pub(crate) unsafe fn RangeFromAnnotation<P0>(
        &self,
        annotation: P0,
    ) -> windows_core::Result<IUIAutomationTextRange>
    where
        P0: windows_core::Param<IUIAutomationElement>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).RangeFromAnnotation)(
                windows_core::Interface::as_raw(self),
                annotation.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCaretRange(
        &self,
        isactive: *mut windows_core::BOOL,
    ) -> windows_core::Result<IUIAutomationTextRange> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCaretRange)(
                windows_core::Interface::as_raw(self),
                isactive as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIAutomationTextPattern2_Vtbl {
    pub base__: IUIAutomationTextPattern_Vtbl,
    pub RangeFromAnnotation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCaretRange: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationTextPattern2_Impl: IUIAutomationTextPattern_Impl {
    fn RangeFromAnnotation(
        &self,
        annotation: windows_core::Ref<IUIAutomationElement>,
    ) -> windows_core::Result<IUIAutomationTextRange>;
    fn GetCaretRange(
        &self,
        isactive: *mut windows_core::BOOL,
    ) -> windows_core::Result<IUIAutomationTextRange>;
}
impl IUIAutomationTextPattern2_Vtbl {
    pub const fn new<Identity: IUIAutomationTextPattern2_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn RangeFromAnnotation<
            Identity: IUIAutomationTextPattern2_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            annotation: *mut core::ffi::c_void,
            range: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern2_Impl::RangeFromAnnotation(
                    this,
                    core::mem::transmute_copy(&annotation),
                ) {
                    Ok(ok__) => {
                        range.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCaretRange<
            Identity: IUIAutomationTextPattern2_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            isactive: *mut windows_core::BOOL,
            range: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextPattern2_Impl::GetCaretRange(
                    this,
                    core::mem::transmute_copy(&isactive),
                ) {
                    Ok(ok__) => {
                        range.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: IUIAutomationTextPattern_Vtbl::new::<Identity, OFFSET>(),
            RangeFromAnnotation: RangeFromAnnotation::<Identity, OFFSET>,
            GetCaretRange: GetCaretRange::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationTextPattern2 as windows_core::Interface>::IID
            || iid == &<IUIAutomationTextPattern as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationTextPattern2 {}
windows_core::imp::define_interface!(
    IUIAutomationTextRange,
    IUIAutomationTextRange_Vtbl,
    0xa543cc6a_f4ae_494b_8239_c814481187a8
);
windows_core::imp::interface_hierarchy!(IUIAutomationTextRange, windows_core::IUnknown);
impl IUIAutomationTextRange {
    pub(crate) unsafe fn Clone(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Clone)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn Compare<P0>(&self, range: P0) -> windows_core::Result<windows_core::BOOL>
    where
        P0: windows_core::Param<Self>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Compare)(
                windows_core::Interface::as_raw(self),
                range.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn CompareEndpoints<P1>(
        &self,
        srcendpoint: TextPatternRangeEndpoint,
        range: P1,
        targetendpoint: TextPatternRangeEndpoint,
    ) -> windows_core::Result<i32>
    where
        P1: windows_core::Param<Self>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CompareEndpoints)(
                windows_core::Interface::as_raw(self),
                srcendpoint,
                range.param().abi(),
                targetendpoint,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn ExpandToEnclosingUnit(&self, textunit: TextUnit) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).ExpandToEnclosingUnit)(
                windows_core::Interface::as_raw(self),
                textunit,
            )
        }
    }
    pub(crate) unsafe fn FindAttribute(
        &self,
        attr: TEXTATTRIBUTEID,
        val: &VARIANT,
        backward: bool,
    ) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindAttribute)(
                windows_core::Interface::as_raw(self),
                attr,
                core::mem::transmute_copy(val),
                backward.into(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn FindText(
        &self,
        text: &windows_core::BSTR,
        backward: bool,
        ignorecase: bool,
    ) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FindText)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(text),
                backward.into(),
                ignorecase.into(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetAttributeValue(
        &self,
        attr: TEXTATTRIBUTEID,
    ) -> windows_core::Result<VARIANT> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAttributeValue)(
                windows_core::Interface::as_raw(self),
                attr,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetBoundingRectangles(&self) -> windows_core::Result<*mut SAFEARRAY> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetBoundingRectangles)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetEnclosingElement(&self) -> windows_core::Result<IUIAutomationElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetEnclosingElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetText(
        &self,
        maxlength: i32,
    ) -> windows_core::Result<windows_core::BSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetText)(
                windows_core::Interface::as_raw(self),
                maxlength,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn Move(&self, unit: TextUnit, count: i32) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Move)(
                windows_core::Interface::as_raw(self),
                unit,
                count,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn MoveEndpointByUnit(
        &self,
        endpoint: TextPatternRangeEndpoint,
        unit: TextUnit,
        count: i32,
    ) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).MoveEndpointByUnit)(
                windows_core::Interface::as_raw(self),
                endpoint,
                unit,
                count,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn MoveEndpointByRange<P1>(
        &self,
        srcendpoint: TextPatternRangeEndpoint,
        range: P1,
        targetendpoint: TextPatternRangeEndpoint,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<Self>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).MoveEndpointByRange)(
                windows_core::Interface::as_raw(self),
                srcendpoint,
                range.param().abi(),
                targetendpoint,
            )
        }
    }
    pub(crate) unsafe fn Select(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Select)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn AddToSelection(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).AddToSelection)(windows_core::Interface::as_raw(
                self,
            ))
        }
    }
    pub(crate) unsafe fn RemoveFromSelection(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).RemoveFromSelection)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn ScrollIntoView(&self, aligntotop: bool) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).ScrollIntoView)(
                windows_core::Interface::as_raw(self),
                aligntotop.into(),
            )
        }
    }
    pub(crate) unsafe fn GetChildren(&self) -> windows_core::Result<IUIAutomationElementArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetChildren)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIAutomationTextRange_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Clone: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Compare: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub CompareEndpoints: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TextPatternRangeEndpoint,
        *mut core::ffi::c_void,
        TextPatternRangeEndpoint,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub ExpandToEnclosingUnit:
        unsafe extern "system" fn(*mut core::ffi::c_void, TextUnit) -> windows_core::HRESULT,
    pub FindAttribute: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TEXTATTRIBUTEID,
        VARIANT,
        windows_core::BOOL,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub FindText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_core::BOOL,
        windows_core::BOOL,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetAttributeValue: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TEXTATTRIBUTEID,
        *mut VARIANT,
    ) -> windows_core::HRESULT,
    pub GetBoundingRectangles: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut SAFEARRAY,
    ) -> windows_core::HRESULT,
    pub GetEnclosingElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub Move: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TextUnit,
        i32,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub MoveEndpointByUnit: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TextPatternRangeEndpoint,
        TextUnit,
        i32,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub MoveEndpointByRange: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        TextPatternRangeEndpoint,
        *mut core::ffi::c_void,
        TextPatternRangeEndpoint,
    ) -> windows_core::HRESULT,
    pub Select: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub AddToSelection: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub RemoveFromSelection:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub ScrollIntoView: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetChildren: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationTextRange_Impl: windows_core::IUnknownImpl {
    fn Clone(&self) -> windows_core::Result<IUIAutomationTextRange>;
    fn Compare(
        &self,
        range: windows_core::Ref<IUIAutomationTextRange>,
    ) -> windows_core::Result<windows_core::BOOL>;
    fn CompareEndpoints(
        &self,
        srcendpoint: TextPatternRangeEndpoint,
        range: windows_core::Ref<IUIAutomationTextRange>,
        targetendpoint: TextPatternRangeEndpoint,
    ) -> windows_core::Result<i32>;
    fn ExpandToEnclosingUnit(&self, textunit: TextUnit) -> windows_core::Result<()>;
    fn FindAttribute(
        &self,
        attr: TEXTATTRIBUTEID,
        val: &VARIANT,
        backward: windows_core::BOOL,
    ) -> windows_core::Result<IUIAutomationTextRange>;
    fn FindText(
        &self,
        text: &windows_core::BSTR,
        backward: windows_core::BOOL,
        ignorecase: windows_core::BOOL,
    ) -> windows_core::Result<IUIAutomationTextRange>;
    fn GetAttributeValue(&self, attr: TEXTATTRIBUTEID) -> windows_core::Result<VARIANT>;
    fn GetBoundingRectangles(&self) -> windows_core::Result<*mut SAFEARRAY>;
    fn GetEnclosingElement(&self) -> windows_core::Result<IUIAutomationElement>;
    fn GetText(&self, maxlength: i32) -> windows_core::Result<windows_core::BSTR>;
    fn Move(&self, unit: TextUnit, count: i32) -> windows_core::Result<i32>;
    fn MoveEndpointByUnit(
        &self,
        endpoint: TextPatternRangeEndpoint,
        unit: TextUnit,
        count: i32,
    ) -> windows_core::Result<i32>;
    fn MoveEndpointByRange(
        &self,
        srcendpoint: TextPatternRangeEndpoint,
        range: windows_core::Ref<IUIAutomationTextRange>,
        targetendpoint: TextPatternRangeEndpoint,
    ) -> windows_core::Result<()>;
    fn Select(&self) -> windows_core::Result<()>;
    fn AddToSelection(&self) -> windows_core::Result<()>;
    fn RemoveFromSelection(&self) -> windows_core::Result<()>;
    fn ScrollIntoView(&self, aligntotop: windows_core::BOOL) -> windows_core::Result<()>;
    fn GetChildren(&self) -> windows_core::Result<IUIAutomationElementArray>;
}
impl IUIAutomationTextRange_Vtbl {
    pub const fn new<Identity: IUIAutomationTextRange_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Clone<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clonedrange: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::Clone(this) {
                    Ok(ok__) => {
                        clonedrange.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Compare<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            range: *mut core::ffi::c_void,
            aresame: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::Compare(this, core::mem::transmute_copy(&range))
                {
                    Ok(ok__) => {
                        aresame.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CompareEndpoints<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            srcendpoint: TextPatternRangeEndpoint,
            range: *mut core::ffi::c_void,
            targetendpoint: TextPatternRangeEndpoint,
            compvalue: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::CompareEndpoints(
                    this,
                    core::mem::transmute_copy(&srcendpoint),
                    core::mem::transmute_copy(&range),
                    core::mem::transmute_copy(&targetendpoint),
                ) {
                    Ok(ok__) => {
                        compvalue.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ExpandToEnclosingUnit<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            textunit: TextUnit,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTextRange_Impl::ExpandToEnclosingUnit(
                    this,
                    core::mem::transmute_copy(&textunit),
                )
                .into()
            }
        }
        unsafe extern "system" fn FindAttribute<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            attr: TEXTATTRIBUTEID,
            val: VARIANT,
            backward: windows_core::BOOL,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::FindAttribute(
                    this,
                    core::mem::transmute_copy(&attr),
                    core::mem::transmute(&val),
                    core::mem::transmute_copy(&backward),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn FindText<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            text: *mut core::ffi::c_void,
            backward: windows_core::BOOL,
            ignorecase: windows_core::BOOL,
            found: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::FindText(
                    this,
                    core::mem::transmute(&text),
                    core::mem::transmute_copy(&backward),
                    core::mem::transmute_copy(&ignorecase),
                ) {
                    Ok(ok__) => {
                        found.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetAttributeValue<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            attr: TEXTATTRIBUTEID,
            value: *mut VARIANT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::GetAttributeValue(
                    this,
                    core::mem::transmute_copy(&attr),
                ) {
                    Ok(ok__) => {
                        value.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetBoundingRectangles<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            boundingrects: *mut *mut SAFEARRAY,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::GetBoundingRectangles(this) {
                    Ok(ok__) => {
                        boundingrects.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetEnclosingElement<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            enclosingelement: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::GetEnclosingElement(this) {
                    Ok(ok__) => {
                        enclosingelement.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetText<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            maxlength: i32,
            text: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::GetText(
                    this,
                    core::mem::transmute_copy(&maxlength),
                ) {
                    Ok(ok__) => {
                        text.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Move<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            unit: TextUnit,
            count: i32,
            moved: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::Move(
                    this,
                    core::mem::transmute_copy(&unit),
                    core::mem::transmute_copy(&count),
                ) {
                    Ok(ok__) => {
                        moved.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn MoveEndpointByUnit<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            endpoint: TextPatternRangeEndpoint,
            unit: TextUnit,
            count: i32,
            moved: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::MoveEndpointByUnit(
                    this,
                    core::mem::transmute_copy(&endpoint),
                    core::mem::transmute_copy(&unit),
                    core::mem::transmute_copy(&count),
                ) {
                    Ok(ok__) => {
                        moved.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn MoveEndpointByRange<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            srcendpoint: TextPatternRangeEndpoint,
            range: *mut core::ffi::c_void,
            targetendpoint: TextPatternRangeEndpoint,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTextRange_Impl::MoveEndpointByRange(
                    this,
                    core::mem::transmute_copy(&srcendpoint),
                    core::mem::transmute_copy(&range),
                    core::mem::transmute_copy(&targetendpoint),
                )
                .into()
            }
        }
        unsafe extern "system" fn Select<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTextRange_Impl::Select(this).into()
            }
        }
        unsafe extern "system" fn AddToSelection<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTextRange_Impl::AddToSelection(this).into()
            }
        }
        unsafe extern "system" fn RemoveFromSelection<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTextRange_Impl::RemoveFromSelection(this).into()
            }
        }
        unsafe extern "system" fn ScrollIntoView<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            aligntotop: windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IUIAutomationTextRange_Impl::ScrollIntoView(
                    this,
                    core::mem::transmute_copy(&aligntotop),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetChildren<
            Identity: IUIAutomationTextRange_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            children: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRange_Impl::GetChildren(this) {
                    Ok(ok__) => {
                        children.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Clone: Clone::<Identity, OFFSET>,
            Compare: Compare::<Identity, OFFSET>,
            CompareEndpoints: CompareEndpoints::<Identity, OFFSET>,
            ExpandToEnclosingUnit: ExpandToEnclosingUnit::<Identity, OFFSET>,
            FindAttribute: FindAttribute::<Identity, OFFSET>,
            FindText: FindText::<Identity, OFFSET>,
            GetAttributeValue: GetAttributeValue::<Identity, OFFSET>,
            GetBoundingRectangles: GetBoundingRectangles::<Identity, OFFSET>,
            GetEnclosingElement: GetEnclosingElement::<Identity, OFFSET>,
            GetText: GetText::<Identity, OFFSET>,
            Move: Move::<Identity, OFFSET>,
            MoveEndpointByUnit: MoveEndpointByUnit::<Identity, OFFSET>,
            MoveEndpointByRange: MoveEndpointByRange::<Identity, OFFSET>,
            Select: Select::<Identity, OFFSET>,
            AddToSelection: AddToSelection::<Identity, OFFSET>,
            RemoveFromSelection: RemoveFromSelection::<Identity, OFFSET>,
            ScrollIntoView: ScrollIntoView::<Identity, OFFSET>,
            GetChildren: GetChildren::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationTextRange as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationTextRange {}
windows_core::imp::define_interface!(
    IUIAutomationTextRangeArray,
    IUIAutomationTextRangeArray_Vtbl,
    0xce4ae76a_e717_4c98_81ea_47371d028eb6
);
windows_core::imp::interface_hierarchy!(IUIAutomationTextRangeArray, windows_core::IUnknown);
impl IUIAutomationTextRangeArray {
    pub(crate) unsafe fn Length(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Length)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetElement(
        &self,
        index: i32,
    ) -> windows_core::Result<IUIAutomationTextRange> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetElement)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IUIAutomationTextRangeArray_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Length:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub GetElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IUIAutomationTextRangeArray_Impl: windows_core::IUnknownImpl {
    fn Length(&self) -> windows_core::Result<i32>;
    fn GetElement(&self, index: i32) -> windows_core::Result<IUIAutomationTextRange>;
}
impl IUIAutomationTextRangeArray_Vtbl {
    pub const fn new<Identity: IUIAutomationTextRangeArray_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Length<
            Identity: IUIAutomationTextRangeArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            length: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRangeArray_Impl::Length(this) {
                    Ok(ok__) => {
                        length.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetElement<
            Identity: IUIAutomationTextRangeArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: i32,
            element: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IUIAutomationTextRangeArray_Impl::GetElement(
                    this,
                    core::mem::transmute_copy(&index),
                ) {
                    Ok(ok__) => {
                        element.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Length: Length::<Identity, OFFSET>,
            GetElement: GetElement::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IUIAutomationTextRangeArray as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IUIAutomationTextRangeArray {}
windows_core::imp::define_interface!(
    IUIAutomationTreeWalker,
    IUIAutomationTreeWalker_Vtbl,
    0x4042c624_389c_4afc_a630_9df854a541fc
);
windows_core::imp::interface_hierarchy!(IUIAutomationTreeWalker, windows_core::IUnknown);
#[repr(C)]
pub struct IUIAutomationTreeWalker_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetParentElement: usize,
    GetFirstChildElement: usize,
    GetLastChildElement: usize,
    GetNextSiblingElement: usize,
    GetPreviousSiblingElement: usize,
    NormalizeElement: usize,
    GetParentElementBuildCache: usize,
    GetFirstChildElementBuildCache: usize,
    GetLastChildElementBuildCache: usize,
    GetNextSiblingElementBuildCache: usize,
    GetPreviousSiblingElementBuildCache: usize,
    NormalizeElementBuildCache: usize,
    Condition: usize,
}
impl windows_core::RuntimeName for IUIAutomationTreeWalker {}
pub const OBJID_CARET: i32 = -8;
pub type OrientationType = i32;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PATTERNID(pub i32);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct PROPERTYID(pub i32);
pub type PropertyConditionFlags = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SAFEARRAY {
    pub cDims: u16,
    pub fFeatures: u16,
    pub cbElements: u32,
    pub cLocks: u32,
    pub pvData: *mut core::ffi::c_void,
    pub rgsabound: [SAFEARRAYBOUND; 1],
}
impl Default for SAFEARRAY {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SAFEARRAYBOUND {
    pub cElements: u32,
    pub lLbound: i32,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SCODE(pub i32);
pub type SupportedTextSelection = i32;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct TEXTATTRIBUTEID(pub i32);
pub type TextPatternRangeEndpoint = i32;
pub type TextUnit = i32;
pub type TreeScope = i32;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct UIA_HWND(pub *mut core::ffi::c_void);
#[repr(C)]
pub struct VARIANT {
    pub Anonymous: VARIANT_0,
}
impl Clone for VARIANT {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub union VARIANT_0 {
    pub Anonymous: core::mem::ManuallyDrop<VARIANT_0_0>,
    pub decVal: DECIMAL,
}
impl Clone for VARIANT_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub struct VARIANT_0_0 {
    pub vt: VARTYPE,
    pub wReserved1: u16,
    pub wReserved2: u16,
    pub wReserved3: u16,
    pub Anonymous: VARIANT_0_0_0,
}
impl Clone for VARIANT_0_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT_0_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub union VARIANT_0_0_0 {
    pub llVal: i64,
    pub lVal: i32,
    pub bVal: u8,
    pub iVal: i16,
    pub fltVal: f32,
    pub dblVal: f64,
    pub boolVal: VARIANT_BOOL,
    pub __OBSOLETE__VARIANT_BOOL: VARIANT_BOOL,
    pub scode: SCODE,
    pub cyVal: CY,
    pub date: f64,
    pub bstrVal: core::mem::ManuallyDrop<windows_core::BSTR>,
    pub punkVal: core::mem::ManuallyDrop<Option<windows_core::IUnknown>>,
    pub pdispVal: core::mem::ManuallyDrop<Option<IDispatch>>,
    pub parray: *mut SAFEARRAY,
    pub pbVal: *mut u8,
    pub piVal: *mut i16,
    pub plVal: *mut i32,
    pub pllVal: *mut i64,
    pub pfltVal: *mut f32,
    pub pdblVal: *mut f64,
    pub pboolVal: *mut VARIANT_BOOL,
    pub __OBSOLETE__VARIANT_PBOOL: *mut VARIANT_BOOL,
    pub pscode: *mut SCODE,
    pub pcyVal: *mut CY,
    pub pdate: *mut f64,
    pub pbstrVal: *mut windows_core::BSTR,
    pub ppunkVal: *mut Option<windows_core::IUnknown>,
    pub ppdispVal: *mut Option<IDispatch>,
    pub pparray: *mut *mut SAFEARRAY,
    pub pvarVal: *mut VARIANT,
    pub byref: *mut core::ffi::c_void,
    pub cVal: i8,
    pub uiVal: u16,
    pub ulVal: u32,
    pub ullVal: u64,
    pub intVal: i32,
    pub uintVal: u32,
    pub pdecVal: *mut DECIMAL,
    pub pcVal: *mut i8,
    pub puiVal: *mut u16,
    pub pulVal: *mut u32,
    pub pullVal: *mut u64,
    pub pintVal: *mut i32,
    pub puintVal: *mut u32,
    pub Anonymous: core::mem::ManuallyDrop<VARIANT_0_0_0_0>,
}
impl Clone for VARIANT_0_0_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for VARIANT_0_0_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VARIANT_0_0_0_0 {
    pub pvRecord: *mut core::ffi::c_void,
    pub pRecInfo: core::mem::ManuallyDrop<Option<IRecordInfo>>,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct VARIANT_BOOL(pub i16);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct VARTYPE(pub u16);
