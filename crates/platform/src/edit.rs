//! A themed EDIT control (one line, or several for a note) with key interception (Enter/Esc
//! via subclassing).

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use windows_core::{Error, PCWSTR, PWSTR, Result};

pub const EN_KILLFOCUS: u32 = crate::bindings::EN_KILLFOCUS as u32;
pub const EN_CHANGE: u32 = crate::bindings::EN_CHANGE as u32;
pub const WM_CTLCOLOREDIT_MSG: u32 = WM_CTLCOLOREDIT as u32;

// Not in the generated bindings (winuser.h).
const ES_MULTILINE: i32 = 0x0004;
const ES_AUTOVSCROLL: i32 = 0x0040;
const ES_WANTRETURN: i32 = 0x1000;
const WS_VSCROLL: i32 = 0x0020_0000;
const EM_SETMARGINS: u32 = 0x00D3;
const EC_LEFTMARGIN: usize = 1;
const EC_RIGHTMARGIN: usize = 2;

type KeyHandler = fn(u32) -> bool;

pub struct EditControl {
    hwnd: HWND,
    font: HFONT,
    brush: HBRUSH,
    text_color: u32,
    back_color: u32,
}

impl EditControl {
    /// Creates the control filling `parent`'s client area, its text `font_dip` DIPs high.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        parent: HWND,
        width: i32,
        height: i32,
        dpi: u32,
        font_dip: f32,
        text: &str,
        dark: bool,
        on_key: KeyHandler,
    ) -> Result<Self> {
        Self::create_styled(
            parent,
            width,
            height,
            dpi,
            font_dip,
            text,
            dark,
            on_key,
            ES_AUTOHSCROLL,
        )
    }

    /// A control for several lines: Enter starts a new line, long lines wrap, and a scroll bar
    /// appears when the text is taller than the box. `margin_dip` keeps the text off the
    /// left and right edges.
    #[allow(clippy::too_many_arguments)]
    pub fn create_multiline(
        parent: HWND,
        width: i32,
        height: i32,
        dpi: u32,
        font_dip: f32,
        margin_dip: f32,
        text: &str,
        dark: bool,
        on_key: KeyHandler,
    ) -> Result<Self> {
        let ctl = Self::create_styled(
            parent,
            width,
            height,
            dpi,
            font_dip,
            text,
            dark,
            on_key,
            ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN | WS_VSCROLL,
        )?;
        let m = (margin_dip * dpi as f32 / 96.0).round() as usize & 0xFFFF;
        // SAFETY: plain messages and theme call on our control; the buffer outlives the call.
        unsafe {
            if dark {
                // A dark scroll bar to go with the dark background.
                let theme = to_wide("DarkMode_Explorer");
                let _ = SetWindowTheme(ctl.hwnd, PCWSTR(theme.as_ptr()), PCWSTR::null());
            }
            SendMessageW(
                ctl.hwnd,
                EM_SETMARGINS,
                WPARAM(EC_LEFTMARGIN | EC_RIGHTMARGIN),
                LPARAM((m | (m << 16)) as isize),
            );
        }
        Ok(ctl)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_styled(
        parent: HWND,
        width: i32,
        height: i32,
        dpi: u32,
        font_dip: f32,
        text: &str,
        dark: bool,
        on_key: KeyHandler,
        edit_style: i32,
    ) -> Result<Self> {
        let class = to_wide("EDIT");
        let text_w = to_wide(text);
        let (text_color, back_color) = if dark {
            (0x00FF_FFFF, 0x0032_3232)
        } else {
            (0x001B_1B1B, 0x00FB_FBFB)
        };
        // SAFETY: standard control creation; all buffers outlive the calls.
        unsafe {
            let hwnd = CreateWindowExW(
                0,
                PCWSTR(class.as_ptr()),
                PCWSTR(text_w.as_ptr()),
                (WS_CHILD | WS_VISIBLE | edit_style) as u32,
                0,
                0,
                width,
                height,
                Some(parent),
                None,
                None,
                None,
            );
            if hwnd.0.is_null() {
                return Err(Error::from_thread());
            }
            let face = to_wide(crate::fonts::text_face());
            let font = CreateFontW(
                -(font_dip * dpi as f32 / 96.0).round() as i32,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET as u32,
                OUT_DEFAULT_PRECIS as u32,
                CLIP_DEFAULT_PRECIS as u32,
                CLEARTYPE_QUALITY as u32,
                (DEFAULT_PITCH | FF_DONTCARE) as u32,
                PCWSTR(face.as_ptr()),
            );
            SendMessageW(hwnd, WM_SETFONT as u32, WPARAM(font.0 as usize), LPARAM(1));
            let brush = CreateSolidBrush(COLORREF(back_color));
            let _ = SetWindowSubclass(hwnd, Some(subclass_proc), 1, on_key as usize);
            Ok(Self {
                hwnd,
                font,
                brush,
                text_color,
                back_color,
            })
        }
    }

    pub fn focus(&self) {
        // SAFETY: plain FFI call on our control.
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
        }
    }

    pub fn focus_and_select_all(&self) {
        self.focus_and_select(0, -1);
    }

    /// Focuses the control with the caret after the text.
    pub fn focus_at_end(&self) {
        let end = self.text().encode_utf16().count() as i32;
        self.focus_and_select(end, end);
    }

    /// Focuses the control and selects `start..end` (UTF-16 units; `end` = -1 selects to the
    /// end). Explorer's rename box selects only the stem this way.
    pub fn focus_and_select(&self, start: i32, end: i32) {
        // SAFETY: plain FFI calls on our control.
        unsafe {
            let _ = SetFocus(Some(self.hwnd));
            SendMessageW(
                self.hwnd,
                EM_SETSEL as u32,
                WPARAM(start as usize),
                LPARAM(end as isize),
            );
        }
    }

    /// Moves the control within its parent (client pixels).
    pub fn set_bounds(&self, x: i32, y: i32, width: i32, height: i32) {
        // SAFETY: plain FFI call on our control.
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                x,
                y,
                width,
                height,
                (SWP_NOZORDER | SWP_NOACTIVATE) as u32,
            );
        }
    }

    /// Replaces the text (the parent gets `EN_CHANGE`) and puts the caret after it.
    pub fn set_text(&self, text: &str) {
        let wide = to_wide(text);
        let end = text.encode_utf16().count();
        // SAFETY: the buffer outlives the call; plain messages to our control.
        unsafe {
            let _ = SetWindowTextW(self.hwnd, PCWSTR(wide.as_ptr()));
            SendMessageW(
                self.hwnd,
                EM_SETSEL as u32,
                WPARAM(end),
                LPARAM(end as isize),
            );
        }
    }

    /// True when the caret sits after the last character with nothing selected (Right then
    /// has nothing left to move over).
    pub fn caret_at_end(&self) -> bool {
        let mut start = 0u32;
        let mut end = 0u32;
        // SAFETY: EM_GETSEL writes the selection into the two locals.
        unsafe {
            SendMessageW(
                self.hwnd,
                EM_GETSEL as u32,
                WPARAM(&mut start as *mut u32 as usize),
                LPARAM(&mut end as *mut u32 as isize),
            );
        }
        let len = self.text().encode_utf16().count() as u32;
        start == end && end >= len
    }

    /// Background colour (COLORREF), for a parent that paints around the control.
    pub fn back_color(&self) -> u32 {
        self.back_color
    }

    pub fn text(&self) -> String {
        // SAFETY: buffer sized from the control's own length.
        unsafe {
            let len = GetWindowTextLengthW(self.hwnd);
            let mut buf = vec![0u16; (len.max(0) as usize) + 1];
            let n = GetWindowTextW(self.hwnd, PWSTR(buf.as_mut_ptr()), buf.len() as i32);
            from_wide(&buf[..n.max(0) as usize])
        }
    }

    /// Handles `WM_CTLCOLOREDIT` for the parent: sets colours and returns the brush.
    pub fn ctl_color(&self, hdc_wparam: usize) -> isize {
        // SAFETY: wParam is the control's HDC for this message.
        unsafe {
            let hdc = HDC(hdc_wparam as *mut core::ffi::c_void);
            let _ = SetTextColor(hdc, COLORREF(self.text_color));
            let _ = SetBkColor(hdc, COLORREF(self.back_color));
        }
        self.brush.0 as isize
    }
}

impl Drop for EditControl {
    fn drop(&mut self) {
        // SAFETY: we own these GDI objects; the control dies with its parent window.
        unsafe {
            let _ = RemoveWindowSubclass(self.hwnd, Some(subclass_proc), 1);
            let _ = DestroyWindow(self.hwnd);
            let _ = DeleteObject(HGDIOBJ(self.font.0));
            let _ = DeleteObject(HGDIOBJ(self.brush.0));
        }
    }
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    refdata: usize,
) -> LRESULT {
    // SAFETY: refdata is the `KeyHandler` fn pointer registered in `create`.
    unsafe {
        if msg == WM_KEYDOWN as u32 && refdata != 0 {
            let handler: KeyHandler = core::mem::transmute::<usize, KeyHandler>(refdata);
            if handler(wparam.0 as u32) {
                return LRESULT(0);
            }
        }
        let multiline = GetWindowLongW(hwnd, GWL_STYLE) & ES_MULTILINE != 0;
        if msg == WM_CHAR as u32 && ((wparam.0 == 13 && !multiline) || wparam.0 == 27) {
            return LRESULT(0); // swallow the beep for Enter/Esc
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }
}
