//! PecoFence addition: the WebView2 composition controller ("visual hosting"). The page is drawn
//! into a visual of the host's own Windows.UI.Composition tree instead of a child window, so the
//! host's clips, opacity and animations apply to it. The host forwards mouse input
//! ([`CompositionController::send_mouse_input`]) and shows the page's cursor
//! ([`CompositionController::cursor`]); the keyboard reaches the page through the parent window's
//! focus ([`Controller::move_focus`]).
//!
//! Also `WebView::try_suspend` and `WebView::resume` (`ICoreWebView2_3`), and the drags a
//! page starts ([`CompositionController::on_drag_starting`], `ICoreWebView2CompositionController5`,
//! runtime 144 and later).

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

// The whole vtable from ICoreWebView2CompositionController up: only the last two are called.
interface_decl! {
    unsafe trait ICoreWebView2CompositionController5(
        ICoreWebView2CompositionController5_Vtbl,
        ICoreWebView2CompositionController5_Impl
    ) : IUnknown = 0x8d0f82eb_7c33_5a4c_9108_84ca28ccc3b4 {
        unsafe fn RootVisualTarget(&self, target: *mut Raw) -> Result<()>;
        unsafe fn SetRootVisualTarget(&self, target: Raw) -> Result<()>;
        unsafe fn SendMouseInput(&self, kind: i32, keys: i32, data: u32, point: POINT) -> Result<()>;
        unsafe fn SendPointerInput(&self, kind: i32, info: Raw) -> Result<()>;
        unsafe fn Cursor(&self, cursor: *mut Raw) -> Result<()>;
        unsafe fn SystemCursorId(&self, id: *mut u32) -> Result<()>;
        unsafe fn add_CursorChanged(&self, handler: Raw, token: *mut i64) -> Result<()>;
        unsafe fn remove_CursorChanged(&self, token: i64) -> Result<()>;
        unsafe fn AutomationProvider(&self, provider: *mut Raw) -> Result<()>;
        unsafe fn DragEnter(&self, data: Raw, keys: u32, point: POINT, effect: *mut u32) -> Result<()>;
        unsafe fn DragLeave(&self) -> Result<()>;
        unsafe fn DragOver(&self, keys: u32, point: POINT, effect: *mut u32) -> Result<()>;
        unsafe fn Drop(&self, data: Raw, keys: u32, point: POINT, effect: *mut u32) -> Result<()>;
        unsafe fn GetNonClientRegionAtPoint(&self, point: POINT, kind: *mut i32) -> Result<()>;
        unsafe fn QueryNonClientRegion(&self, kind: i32, rects: *mut Raw) -> Result<()>;
        unsafe fn add_NonClientRegionChanged(&self, handler: Raw, token: *mut i64) -> Result<()>;
        unsafe fn remove_NonClientRegionChanged(&self, token: i64) -> Result<()>;
        unsafe fn add_DragStarting(&self, handler: Raw, token: *mut i64) -> Result<()>;
        unsafe fn remove_DragStarting(&self, token: i64) -> Result<()>;
    }
}

interface_decl! {
    unsafe trait ICoreWebView2DragStartingEventHandler(
        DragStartingHandler_Vtbl,
        DragStartingHandler_Impl
    ) : IUnknown = 0x3b149321_83c3_5d1f_b03f_a42899bc1c15 {
        unsafe fn Invoke(&self, sender: Raw, args: Raw) -> Result<()>;
    }
}

interface_decl! {
    unsafe trait ICoreWebView2DragStartingEventArgs(
        ICoreWebView2DragStartingEventArgs_Vtbl,
        ICoreWebView2DragStartingEventArgs_Impl
    ) : IUnknown = 0x8b8d9c7e_2f1a_4e6b_9d5a_3c8f7b9e1a2d {
        unsafe fn AllowedDropEffects(&self, value: *mut u32) -> Result<()>;
        unsafe fn Data(&self, value: *mut Raw) -> Result<()>;
        unsafe fn Handled(&self, value: *mut BOOL) -> Result<()>;
        unsafe fn SetHandled(&self, value: BOOL) -> Result<()>;
        unsafe fn Position(&self, value: *mut POINT) -> Result<()>;
        unsafe fn GetDeferral(&self, value: *mut Raw) -> Result<()>;
    }
}

struct DragStartingSink(RefCell<Box<dyn FnMut(&DragStarting)>>);

implement_decl! {
    impl DragStartingSink as DragStartingSink_Impl: [ICoreWebView2DragStartingEventHandler]
}

impl DragStartingHandler_Impl for DragStartingSink_Impl {
    unsafe fn Invoke(&self, _sender: Raw, args: Raw) -> Result<()> {
        // SAFETY: WebView2 passes the event's args, an `ICoreWebView2DragStartingEventArgs` (the
        // type WebView2.h gives it: no QueryInterface that could fail and let the drag go on
        // unchecked), live for the duration of the call.
        let args = unsafe { ICoreWebView2DragStartingEventArgs::from_raw_borrowed(&args) };
        (*self.0.borrow_mut())(&DragStarting(args.ok_or_else(Error::empty)?.clone()));
        Ok(())
    }
}

/// A drag the page starts (`DragStarting`): WebView2 calls `DoDragDrop` with [`Self::data`]
/// after the handler unless it sets [`Self::set_handled`].
pub struct DragStarting(ICoreWebView2DragStartingEventArgs);

impl DragStarting {
    /// The data object WebView2 would drag (an `IDataObject`).
    pub fn data(&self) -> Result<IUnknown> {
        let mut raw = std::ptr::null_mut();
        unsafe {
            self.0.Data(&mut raw)?;
            Ok(IUnknown::from_raw(raw))
        }
    }

    /// The OLE effects the drag allows (`DROPEFFECT_*`).
    pub fn allowed_effects(&self) -> u32 {
        let mut effects = 0;
        let _ = unsafe { self.0.AllowedDropEffects(&mut effects) };
        effects
    }

    /// True: WebView2 does not start the drag (the host does its own, or none).
    pub fn set_handled(&self, handled: bool) -> Result<()> {
        unsafe { self.0.SetHandled(handled.into()) }
    }

    /// Where the drag began, in the page's own pixels.
    pub fn position(&self) -> (i32, i32) {
        let mut p = POINT::default();
        let _ = unsafe { self.0.Position(&mut p) };
        (p.x, p.y)
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

    /// Subscribes to the drags the page starts: `handler` sees what each carries and may stop it
    /// ([`DragStarting::set_handled`]). An error on a runtime older than 144, which has no such
    /// event (and starts every drag itself).
    pub fn on_drag_starting<F: FnMut(&DragStarting) + 'static>(
        &self,
        handler: F,
    ) -> Result<EventRegistration> {
        let source: ICoreWebView2CompositionController5 = self.0.cast()?;
        let handler: ICoreWebView2DragStartingEventHandler =
            DragStartingSink(RefCell::new(Box::new(handler))).into();
        let mut token = 0;
        unsafe {
            source.add_DragStarting(Interface::as_raw(&handler), &mut token)?;
        }
        Ok(EventRegistration::new(move || {
            let _ = unsafe { source.remove_DragStarting(token) };
        }))
    }

    /// A drag over the page, as `IDropTarget` would get it (`step`: 0 enter, 1 over, 2 leave,
    /// 3 drop), `data` the dragged `IDataObject` for enter and drop, `allowed` the source's
    /// effects, (`x`, `y`) in the page's own pixels: the effect the page takes. Runtime 144 and
    /// later.
    pub fn drag(
        &self,
        step: u8,
        data: Option<&IUnknown>,
        (keys, allowed): (u32, u32),
        x: i32,
        y: i32,
    ) -> Result<u32> {
        let target: ICoreWebView2CompositionController5 = self.0.cast()?;
        let raw = data.map_or(std::ptr::null_mut(), Interface::as_raw);
        // In and out, as IDropTarget's: the effects the source allows, then the one taken.
        let (mut effect, point) = (allowed, POINT { x, y });
        unsafe {
            match step {
                0 => target.DragEnter(raw, keys, point, &mut effect)?,
                1 => target.DragOver(keys, point, &mut effect)?,
                3 => target.Drop(raw, keys, point, &mut effect)?,
                _ => target.DragLeave()?,
            }
        }
        Ok(effect)
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
