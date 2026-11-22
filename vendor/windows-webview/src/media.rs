//! Interfaces newer than the generated bindings (PecoFence's addition): the screen capture
//! picker (`ICoreWebView2_27::ScreenCaptureStarting`, runtime 131), web notifications
//! (`ICoreWebView2_24::NotificationReceived`) and a permission decision that is not kept in the
//! profile (`ICoreWebView2PermissionRequestedEventArgs3::SavesInProfile`). Written by hand after
//! WebView2.h of SDK 1.0.3912.50: methods nobody calls are `usize` slots, as in `bindings`.

use super::*;
use std::cell::RefCell;
use windows_core::implement_decl;

type Raw = *mut core::ffi::c_void;
type AddEvent = unsafe extern "system" fn(Raw, Raw, *mut i64) -> HRESULT;
type RemoveEvent = unsafe extern "system" fn(Raw, i64) -> HRESULT;
type GetString = unsafe extern "system" fn(Raw, *mut *mut u16) -> HRESULT;
type PutBool = unsafe extern "system" fn(Raw, BOOL) -> HRESULT;

#[repr(C)]
pub struct ICoreWebView2_24_Vtbl {
    base__: ICoreWebView2_22_Vtbl,
    // ICoreWebView2_23
    PostWebMessageAsJsonWithAdditionalObjects: usize,
    add_NotificationReceived: AddEvent,
    remove_NotificationReceived: RemoveEvent,
}
windows_core::imp::define_interface!(
    ICoreWebView2_24,
    ICoreWebView2_24_Vtbl,
    0x39a7ad55_4287_5cc1_88a1_c6f458593824
);
windows_core::imp::interface_hierarchy!(ICoreWebView2_24, IUnknown);

#[repr(C)]
pub struct ICoreWebView2_27_Vtbl {
    base__: ICoreWebView2_24_Vtbl,
    // ICoreWebView2_25
    add_SaveAsUIShowing: usize,
    remove_SaveAsUIShowing: usize,
    ShowSaveAsUI: usize,
    // ICoreWebView2_26
    add_SaveFileSecurityCheckStarting: usize,
    remove_SaveFileSecurityCheckStarting: usize,
    add_ScreenCaptureStarting: AddEvent,
    remove_ScreenCaptureStarting: RemoveEvent,
}
windows_core::imp::define_interface!(
    ICoreWebView2_27,
    ICoreWebView2_27_Vtbl,
    0x00fbe33b_8c07_517c_aa23_0ddd4b5f6fa0
);
windows_core::imp::interface_hierarchy!(ICoreWebView2_27, IUnknown);

#[repr(C)]
pub struct ICoreWebView2PermissionRequestedEventArgs3_Vtbl {
    base__: ICoreWebView2PermissionRequestedEventArgs_Vtbl,
    // ICoreWebView2PermissionRequestedEventArgs2
    get_Handled: usize,
    put_Handled: usize,
    get_SavesInProfile: usize,
    put_SavesInProfile: PutBool,
}
windows_core::imp::define_interface!(
    ICoreWebView2PermissionRequestedEventArgs3,
    ICoreWebView2PermissionRequestedEventArgs3_Vtbl,
    0xe61670bc_3dce_4177_86d2_c629ae3cb6ac
);
windows_core::imp::interface_hierarchy!(ICoreWebView2PermissionRequestedEventArgs3, IUnknown);

#[repr(C)]
pub struct ICoreWebView2ScreenCaptureStartingEventArgs_Vtbl {
    base__: IUnknown_Vtbl,
    get_Cancel: usize,
    put_Cancel: PutBool,
    get_Handled: usize,
    put_Handled: usize,
    get_OriginalSourceFrameInfo: usize,
    GetDeferral: usize,
}
windows_core::imp::define_interface!(
    ICoreWebView2ScreenCaptureStartingEventArgs,
    ICoreWebView2ScreenCaptureStartingEventArgs_Vtbl,
    0x892c03fd_aee3_5eba_a1fa_6fd2f6484b2b
);
windows_core::imp::interface_hierarchy!(ICoreWebView2ScreenCaptureStartingEventArgs, IUnknown);

#[repr(C)]
pub struct ICoreWebView2Notification_Vtbl {
    base__: IUnknown_Vtbl,
    add_CloseRequested: usize,
    remove_CloseRequested: usize,
    ReportShown: unsafe extern "system" fn(Raw) -> HRESULT,
    ReportClicked: usize,
    ReportClosed: usize,
    get_Body: GetString,
    get_Direction: usize,
    get_Language: usize,
    get_Tag: usize,
    get_IconUri: usize,
    get_Title: GetString,
}
windows_core::imp::define_interface!(
    ICoreWebView2Notification,
    ICoreWebView2Notification_Vtbl,
    0xb7434d98_6bc8_419d_9da5_fb5a96d4dacd
);
windows_core::imp::interface_hierarchy!(ICoreWebView2Notification, IUnknown);

#[repr(C)]
pub struct ICoreWebView2NotificationReceivedEventArgs_Vtbl {
    base__: IUnknown_Vtbl,
    get_SenderOrigin: GetString,
    get_Notification: unsafe extern "system" fn(Raw, *mut Raw) -> HRESULT,
    put_Handled: PutBool,
    get_Handled: usize,
    GetDeferral: usize,
}
windows_core::imp::define_interface!(
    ICoreWebView2NotificationReceivedEventArgs,
    ICoreWebView2NotificationReceivedEventArgs_Vtbl,
    0x1512dd5b_5514_4f85_886e_21c3a4c9cfe6
);
windows_core::imp::interface_hierarchy!(ICoreWebView2NotificationReceivedEventArgs, IUnknown);

/// The two event handlers WebView2 calls: `Invoke(sender, args)`, both pointers.
macro_rules! handler_interface {
    ($name:ident, $vtbl:ident, $trait:ident, $iid:literal) => {
        windows_core::imp::define_interface!($name, $vtbl, $iid);
        windows_core::imp::interface_hierarchy!($name, IUnknown);
        impl RuntimeName for $name {}
        #[repr(C)]
        pub struct $vtbl {
            base__: IUnknown_Vtbl,
            Invoke: unsafe extern "system" fn(Raw, Raw, Raw) -> HRESULT,
        }
        pub trait $trait: IUnknownImpl {
            fn Invoke(&self, args: Raw) -> Result<()>;
        }
        impl $vtbl {
            pub const fn new<Identity: $trait, const OFFSET: isize>() -> Self {
                unsafe extern "system" fn Invoke<Identity: $trait, const OFFSET: isize>(
                    this: Raw,
                    _sender: Raw,
                    args: Raw,
                ) -> HRESULT {
                    unsafe {
                        let this: &Identity =
                            &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                        $trait::Invoke(this, args).into()
                    }
                }
                Self { base__: IUnknown_Vtbl::new::<Identity, OFFSET>(), Invoke: Invoke::<Identity, OFFSET> }
            }
            pub fn matches(iid: &GUID) -> bool {
                iid == &<$name as Interface>::IID
            }
        }
    };
}
handler_interface!(
    ICoreWebView2ScreenCaptureStartingEventHandler,
    ICoreWebView2ScreenCaptureStartingEventHandler_Vtbl,
    ICoreWebView2ScreenCaptureStartingEventHandler_Impl,
    0xe24ff05a_1db5_59d9_89f3_3c864268db4a
);
handler_interface!(
    ICoreWebView2NotificationReceivedEventHandler,
    ICoreWebView2NotificationReceivedEventHandler_Vtbl,
    ICoreWebView2NotificationReceivedEventHandler_Impl,
    0x89c5d598_8788_423b_be97_e6e01c0f9ee3
);

/// A page asks to capture the screen (`getDisplayMedia`), before WebView2 shows its picker.
pub struct ScreenCaptureStartingArgs(ICoreWebView2ScreenCaptureStartingEventArgs);

impl ScreenCaptureStartingArgs {
    /// Cancels the capture: no picker shows, the page gets a `NotAllowedError`.
    pub fn set_cancel(&self, cancel: bool) -> Result<()> {
        unsafe { (Interface::vtable(&self.0).put_Cancel)(Interface::as_raw(&self.0), cancel.into()) }.ok()
    }
}

/// A page shows a web notification (`new Notification(...)`), with the permission granted.
pub struct NotificationReceivedArgs(ICoreWebView2NotificationReceivedEventArgs);

impl NotificationReceivedArgs {
    /// The origin of the page that shows it.
    pub fn sender_origin(&self) -> String {
        let mut s = core::ptr::null_mut();
        let r = unsafe { (Interface::vtable(&self.0).get_SenderOrigin)(Interface::as_raw(&self.0), &mut s) };
        unsafe { string::take_result(r.map(|| s)) }
    }

    /// The host shows it itself: WebView2 shows no Windows notification of its own.
    pub fn set_handled(&self, handled: bool) -> Result<()> {
        unsafe { (Interface::vtable(&self.0).put_Handled)(Interface::as_raw(&self.0), handled.into()) }.ok()
    }

    fn notification(&self) -> Result<ICoreWebView2Notification> {
        let mut raw = core::ptr::null_mut();
        unsafe { (Interface::vtable(&self.0).get_Notification)(Interface::as_raw(&self.0), &mut raw) }.ok()?;
        // SAFETY: an [out] interface pointer WebView2 AddRef'd for the caller.
        unsafe { Ok(ICoreWebView2Notification::from_raw(raw)) }
    }

    /// The notification's title and body.
    pub fn title_and_body(&self) -> (String, String) {
        let Ok(n) = self.notification() else {
            return (String::new(), String::new());
        };
        let text = |get: GetString| {
            let mut s = core::ptr::null_mut();
            let r = unsafe { get(Interface::as_raw(&n), &mut s) };
            unsafe { string::take_result(r.map(|| s)) }
        };
        let vtbl = Interface::vtable(&n);
        (text(vtbl.get_Title), text(vtbl.get_Body))
    }

    /// Tells the page it showed (its `show` event).
    pub fn report_shown(&self) -> Result<()> {
        let n = self.notification()?;
        unsafe { (Interface::vtable(&n).ReportShown)(Interface::as_raw(&n)) }.ok()
    }
}

impl PermissionRequestedArgs {
    /// Whether the decision is kept in the profile (by default it is, and it is the answer of
    /// the next requests too). Runtimes before 1.0.2420 cannot say; then nothing changes.
    pub fn set_saves_in_profile(&self, saves: bool) -> Result<()> {
        let args: ICoreWebView2PermissionRequestedEventArgs3 = self.0.cast()?;
        unsafe { (Interface::vtable(&args).put_SavesInProfile)(Interface::as_raw(&args), saves.into()) }.ok()
    }
}

macro_rules! handler_object {
    ($name:ident / $impl:ident, $iface:ident / $trait:ident, $args:ident => $wrapper:ident) => {
        pub(crate) struct $name(RefCell<Box<dyn FnMut($wrapper)>>);
        implement_decl! {
            impl $name as pub(crate) $impl: [$iface]
        }
        impl $trait for $impl {
            fn Invoke(&self, args: Raw) -> Result<()> {
                // SAFETY: WebView2 passes its args interface, valid for the call; borrowed, so
                // it is AddRef'd by the clone.
                let args = unsafe { $args::from_raw_borrowed(&args) }.ok_or(Error::from_hresult(HRESULT(0x8000_4003_u32 as _)))?.clone();
                (*self.0.borrow_mut())($wrapper(args));
                Ok(())
            }
        }
    };
}
handler_object!(ScreenCaptureStarting / ScreenCaptureStarting_Impl, ICoreWebView2ScreenCaptureStartingEventHandler / ICoreWebView2ScreenCaptureStartingEventHandler_Impl, ICoreWebView2ScreenCaptureStartingEventArgs => ScreenCaptureStartingArgs);
handler_object!(NotificationReceived / NotificationReceived_Impl, ICoreWebView2NotificationReceivedEventHandler / ICoreWebView2NotificationReceivedEventHandler_Impl, ICoreWebView2NotificationReceivedEventArgs => NotificationReceivedArgs);

impl WebView {
    /// The page asks to capture the screen (`getDisplayMedia`); the handler may cancel it before
    /// the picker shows. Fails on a runtime older than 131.
    pub fn on_screen_capture_starting<F: FnMut(ScreenCaptureStartingArgs) + 'static>(
        &self,
        handler: F,
    ) -> Result<EventRegistration> {
        let source: ICoreWebView2_27 = self.0.cast()?;
        let handler: ICoreWebView2ScreenCaptureStartingEventHandler =
            ScreenCaptureStarting(RefCell::new(Box::new(handler))).into();
        let mut token = 0;
        unsafe { (Interface::vtable(&source).add_ScreenCaptureStarting)(Interface::as_raw(&source), Interface::as_raw(&handler), &mut token) }.ok()?;
        Ok(EventRegistration::new(move || {
            let _ = unsafe { (Interface::vtable(&source).remove_ScreenCaptureStarting)(Interface::as_raw(&source), token) };
        }))
    }

    /// The page shows a web notification. Fails on a runtime that has no notification event.
    pub fn on_notification_received<F: FnMut(NotificationReceivedArgs) + 'static>(
        &self,
        handler: F,
    ) -> Result<EventRegistration> {
        let source: ICoreWebView2_24 = self.0.cast()?;
        let handler: ICoreWebView2NotificationReceivedEventHandler =
            NotificationReceived(RefCell::new(Box::new(handler))).into();
        let mut token = 0;
        unsafe { (Interface::vtable(&source).add_NotificationReceived)(Interface::as_raw(&source), Interface::as_raw(&handler), &mut token) }.ok()?;
        Ok(EventRegistration::new(move || {
            let _ = unsafe { (Interface::vtable(&source).remove_NotificationReceived)(Interface::as_raw(&source), token) };
        }))
    }
}
