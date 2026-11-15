//! PecoFence addition: the WebView2 composition controller ("visual hosting"). The page is drawn
//! into a visual of the host's own Windows.UI.Composition tree instead of a child window, so the
//! host's clips, opacity and animations apply to it. The host forwards mouse input
//! ([`CompositionController::send_mouse_input`]) and shows the page's cursor
//! ([`CompositionController::cursor`]); the keyboard reaches the page through the parent window's
//! focus ([`Controller::move_focus`]).
//!
//! Also `WebView::try_suspend` and `WebView::resume` (`ICoreWebView2_3`).

#![allow(non_snake_case, non_camel_case_types)]

use super::*;
use std::cell::{Cell, RefCell};
use windows_core::{implement_decl, interface_decl};

type Raw = *mut core::ffi::c_void;

interface_decl! {
    unsafe trait ICoreWebView2CompositionController(
        ICoreWebView2CompositionController_Vtbl,
        ICoreWebView2CompositionController_Impl
    ) : IUnknown = 0x3df9b733_b9ae_4a15_86b4_eb9ee9826469 {
        unsafe fn RootVisualTarget(&self, target: *mut Raw) -> Result<()>;
        unsafe fn SetRootVisualTarget(&self, target: Raw) -> Result<()>;
        unsafe fn SendMouseInput(&self, kind: i32, keys: i32, data: u32, point: POINT) -> Result<()>;
        unsafe fn SendPointerInput(&self, kind: i32, info: Raw) -> Result<()>;
        unsafe fn Cursor(&self, cursor: *mut Raw) -> Result<()>;
        unsafe fn SystemCursorId(&self, id: *mut u32) -> Result<()>;
        unsafe fn add_CursorChanged(&self, handler: Raw, token: *mut i64) -> Result<()>;
        unsafe fn remove_CursorChanged(&self, token: i64) -> Result<()>;
    }
}

interface_decl! {
    unsafe trait ICoreWebView2CreateCoreWebView2CompositionControllerCompletedHandler(
        CompositionCompletedHandler_Vtbl,
        CompositionCompletedHandler_Impl
    ) : IUnknown = 0x02fab84b_1428_4fb7_ad45_1b2e64736184 {
        unsafe fn Invoke(&self, error: HRESULT, controller: Raw) -> Result<()>;
    }
}

interface_decl! {
    unsafe trait ICoreWebView2CursorChangedEventHandler(
        CursorChangedHandler_Vtbl,
        CursorChangedHandler_Impl
    ) : IUnknown = 0x9da43ccc_26e1_4dad_b56c_d8961c94c571 {
        unsafe fn Invoke(&self, sender: Raw, args: Raw) -> Result<()>;
    }
}

interface_decl! {
    unsafe trait ICoreWebView2TrySuspendCompletedHandler(
        TrySuspendCompletedHandler_Vtbl,
        TrySuspendCompletedHandler_Impl
    ) : IUnknown = 0x00f206a7_9d17_4605_91f6_4e8e4de192e3 {
        unsafe fn Invoke(&self, error: HRESULT, suspended: BOOL) -> Result<()>;
    }
}

struct CompositionCompleted(Cell<Option<Box<dyn FnOnce(Result<CompositionController>)>>>);

implement_decl! {
    impl CompositionCompleted as CompositionCompleted_Impl:
        [ICoreWebView2CreateCoreWebView2CompositionControllerCompletedHandler]
}

impl CompositionCompletedHandler_Impl for CompositionCompleted_Impl {
    unsafe fn Invoke(&self, error: HRESULT, controller: Raw) -> Result<()> {
        let outcome = error.ok().and_then(|()| {
            // SAFETY: WebView2 passes a live controller for the duration of the call.
            let made = unsafe { IUnknown::from_raw_borrowed(&controller) }.ok_or_else(Error::empty)?;
            Ok(CompositionController(made.cast()?))
        });
        if let Some(handler) = self.0.take() {
            handler(outcome);
        }
        Ok(())
    }
}

struct CursorChanged(RefCell<Box<dyn FnMut()>>);

implement_decl! {
    impl CursorChanged as CursorChanged_Impl: [ICoreWebView2CursorChangedEventHandler]
}

impl CursorChangedHandler_Impl for CursorChanged_Impl {
    unsafe fn Invoke(&self, _sender: Raw, _args: Raw) -> Result<()> {
        (*self.0.borrow_mut())();
        Ok(())
    }
}

struct TrySuspendCompleted(Cell<Option<Box<dyn FnOnce(Result<bool>)>>>);

implement_decl! {
    impl TrySuspendCompleted as TrySuspendCompleted_Impl: [ICoreWebView2TrySuspendCompletedHandler]
}

impl TrySuspendCompletedHandler_Impl for TrySuspendCompleted_Impl {
    unsafe fn Invoke(&self, error: HRESULT, suspended: BOOL) -> Result<()> {
        if let Some(handler) = self.0.take() {
            handler(error.ok().map(|()| suspended.as_bool()));
        }
        Ok(())
    }
}

/// A WebView2 browser drawn into the host's visual tree.
#[derive(Clone)]
pub struct CompositionController(ICoreWebView2CompositionController);

impl Environment {
    /// Starts creating a composition controller for the window `parent` (its focus, its monitor's
    /// scale) with `options` (the profile, a transparent background). `handler` runs from the
    /// message loop once it is ready; nothing is pumped here.
    ///
    /// # Safety
    ///
    /// `parent` must be a valid window handle that outlives the controller.
    pub unsafe fn create_composition_controller<F>(
        &self,
        parent: Raw,
        options: &ControllerOptions,
        handler: F,
    ) -> Result<()>
    where
        F: FnOnce(Result<CompositionController>) + 'static,
    {
        let environment: ICoreWebView2Environment10 = self.0.cast()?;
        let options = options.raw_options(&environment)?;
        let handler: ICoreWebView2CreateCoreWebView2CompositionControllerCompletedHandler =
            CompositionCompleted(Cell::new(Some(Box::new(handler)))).into();
        unsafe {
            (Interface::vtable(&environment).CreateCoreWebView2CompositionControllerWithOptions)(
                Interface::as_raw(&environment),
                parent,
                Interface::as_raw(&options),
                Interface::as_raw(&handler),
            )
            .ok()
        }
    }
}

impl CompositionController {
    /// The controller of the same browser: bounds, visibility, focus, scale, the `WebView`.
    pub fn controller(&self) -> Result<Controller> {
        Ok(Controller(self.0.cast()?))
    }

    /// Hangs the page into `visual` (a `Windows.UI.Composition.ContainerVisual`).
    pub fn set_root_visual_target(&self, visual: &IUnknown) -> Result<()> {
        unsafe { self.0.SetRootVisualTarget(Interface::as_raw(visual)) }
    }

    /// Sends a mouse message to the page: `kind` is the window message (`WM_MOUSEMOVE`,
    /// `WM_LBUTTONDOWN`, `WM_MOUSEWHEEL`, `WM_MOUSELEAVE`, ...), `keys` its `MK_*` flags, `data`
    /// the wheel delta or the X button, (`x`, `y`) the point in the page's own pixels.
    pub fn send_mouse_input(&self, kind: u32, keys: u32, data: u32, x: i32, y: i32) -> Result<()> {
        unsafe {
            self.0
                .SendMouseInput(kind as i32, keys as i32, data, POINT { x, y })
        }
    }

    /// The cursor the page wants where the mouse last was (an `HCURSOR`).
    pub fn cursor(&self) -> Raw {
        let mut cursor = std::ptr::null_mut();
        let _ = unsafe { self.0.Cursor(&mut cursor) };
        cursor
    }

    /// Subscribes to changes of [`cursor`](Self::cursor).
    pub fn on_cursor_changed<F: FnMut() + 'static>(&self, handler: F) -> Result<EventRegistration> {
        let handler: ICoreWebView2CursorChangedEventHandler =
            CursorChanged(RefCell::new(Box::new(handler))).into();
        let mut token = 0;
        unsafe {
            self.0
                .add_CursorChanged(Interface::as_raw(&handler), &mut token)?;
        }
        let source = self.0.clone();
        Ok(EventRegistration::new(move || {
            let _ = unsafe { source.remove_CursorChanged(token) };
        }))
    }
}

impl WebView {
    /// Asks the browser to suspend this page (timers, script, rendering); the controller must be
    /// invisible first. `handler` gets whether it did. Showing the controller again resumes it.
    pub fn try_suspend<F: FnOnce(Result<bool>) + 'static>(&self, handler: F) -> Result<()> {
        let webview: ICoreWebView2_3 = self.0.cast()?;
        let handler: ICoreWebView2TrySuspendCompletedHandler =
            TrySuspendCompleted(Cell::new(Some(Box::new(handler)))).into();
        unsafe {
            (Interface::vtable(&webview).TrySuspend)(
                Interface::as_raw(&webview),
                Interface::as_raw(&handler),
            )
            .ok()
        }
    }

    /// The id of the browser process.
    pub fn browser_process_id(&self) -> u32 {
        let mut pid = 0;
        let _ = unsafe { (Interface::vtable(&self.0).BrowserProcessId)(Interface::as_raw(&self.0), &mut pid) };
        pid
    }

    /// Resumes a suspended page.
    pub fn resume(&self) -> Result<()> {
        let webview: ICoreWebView2_3 = self.0.cast()?;
        unsafe { (Interface::vtable(&webview).Resume)(Interface::as_raw(&webview)).ok() }
    }
}
