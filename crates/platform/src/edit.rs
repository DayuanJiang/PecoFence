//! A themed EDIT control (one line, or several for a note) with key interception (Enter/Esc
//! via subclassing).

use crate::bindings::*;
use crate::wide::{from_wide, to_wide};
use windows_core::{Error, PCWSTR, PWSTR, Result};

pub const EN_KILLFOCUS: u32 = crate::bindings::EN_KILLFOCUS as u32;
pub const EN_CHANGE: u32 = crate::bindings::EN_CHANGE as u32;
pub const WM_CTLCOLOREDIT_MSG: u32 = WM_CTLCOLOREDIT as u32;

// Not in the generated bindings (winuser.h).
const ES_CENTER: i32 = 0x0001;
const ES_MULTILINE: i32 = 0x0004;
const ES_AUTOVSCROLL: i32 = 0x0040;
const ES_WANTRETURN: i32 = 0x1000;
const WS_VSCROLL: i32 = 0x0020_0000;
const EM_SETMARGINS: u32 = 0x00D3;
const EM_GETLINECOUNT: u32 = 0x00BA;
const EM_LINESCROLL: u32 = 0x00B6;
const EM_SCROLLCARET: u32 = 0x00B7;
const EM_GETFIRSTVISIBLELINE: u32 = 0x00CE;
const EC_LEFTMARGIN: usize = 1;
const EC_RIGHTMARGIN: usize = 2;

type KeyHandler = fn(u32) -> bool;

/// Background colour (COLORREF) of an [`EditControl`] made with `dark`, for a parent that
/// paints its frame in a handler that does not hold the control.
pub fn back_color(dark: bool) -> u32 {
    if dark { 0x0032_3232 } else { 0x00FB_FBFB }
}

/// The code a key handler gets for the menu key or Shift+F10 (a `WM_CONTEXTMENU` from the
/// keyboard). Outside the virtual-key range; returning true keeps the control's own text menu
/// away. A right-click always gives the text menu.
pub const KEY_CONTEXT_MENU: u32 = 0x1_0000;

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

    /// Like [`Self::create`] with the text centred in the box: the rename box of an icon, whose
    /// name is centred under it (Explorer).
    #[allow(clippy::too_many_arguments)]
    pub fn create_centered(
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
            ES_AUTOHSCROLL | ES_CENTER,
        )
    }

    /// A box for one name that wraps like an icon's label: centred lines, no line breaks
    /// (Enter and Tab go to the key handler, Ctrl+Enter does nothing), no scroll bar, no
    /// margins. The owner sizes it to its lines ([`Self::line_count`], [`Self::line_height`]).
    #[allow(clippy::too_many_arguments)]
    pub fn create_wrapping(
        parent: HWND,
        width: i32,
        height: i32,
        dpi: u32,
        font_dip: f32,
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
            ES_MULTILINE | ES_AUTOVSCROLL | ES_CENTER,
        )?;
        ctl.set_margins(0);
        Ok(ctl)
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
        let text_color = if dark { 0x00FF_FFFF } else { 0x001B_1B1B };
        let back_color = back_color(dark);
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

    /// Left and right text margins in pixels (the default follows the font).
    pub fn set_margins(&self, px: i32) {
        let m = px.clamp(0, 0xFFFF) as usize;
        // SAFETY: a plain message to our control.
        unsafe {
            SendMessageW(
                self.hwnd,
                EM_SETMARGINS,
                WPARAM(EC_LEFTMARGIN | EC_RIGHTMARGIN),
                LPARAM((m | (m << 16)) as isize),
            );
        }
    }

    /// Lines the text takes at the control's width (1 for an empty or one-line control).
    pub fn line_count(&self) -> i32 {
        // SAFETY: a plain message to our control.
        let n = unsafe { SendMessageW(self.hwnd, EM_GETLINECOUNT, WPARAM(0), LPARAM(0)) };
        (n.0 as i32).max(1)
    }

    /// Height of one text line in pixels (the font's cell height, which is the control's
    /// line pitch).
    pub fn line_height(&self) -> i32 {
        let probe: Vec<u16> = "Ag".encode_utf16().collect();
        let mut size = SIZE::default();
        // SAFETY: the DC is released before returning; the font stays ours and is selected
        // out again; the buffer and `size` outlive the call.
        unsafe {
            let hdc = GetDC(Some(self.hwnd));
            let old = SelectObject(hdc, HGDIOBJ(self.font.0));
            let _ = GetTextExtentPoint32W(hdc, &probe, &mut size);
            SelectObject(hdc, old);
            ReleaseDC(Some(self.hwnd), hdc);
        }
        size.cy.max(1)
    }

    /// Shows the text from its first line (a box that fits every line).
    pub fn scroll_to_top(&self) {
        // SAFETY: plain messages to our control.
        unsafe {
            let first = SendMessageW(self.hwnd, EM_GETFIRSTVISIBLELINE, WPARAM(0), LPARAM(0)).0;
            if first > 0 {
                SendMessageW(self.hwnd, EM_LINESCROLL, WPARAM(0), LPARAM(-first));
            }
        }
    }

    /// Scrolls the caret into view (a text with more lines than the box shows).
    pub fn scroll_caret(&self) {
        // SAFETY: a plain message to our control.
        unsafe {
            SendMessageW(self.hwnd, EM_SCROLLCARET, WPARAM(0), LPARAM(0));
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

    /// Grey hint text shown while the box is empty (EM_SETCUEBANNER), also while it has the
    /// focus.
    pub fn set_cue(&self, text: &str) {
        const EM_SETCUEBANNER: u32 = 0x1501;
        let wide = to_wide(text);
        // SAFETY: the buffer outlives the call; a plain message to our control.
        unsafe {
            SendMessageW(
                self.hwnd,
                EM_SETCUEBANNER,
                WPARAM(1),
                LPARAM(wide.as_ptr() as isize),
            );
        }
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

/// A key or IME message the box handled slower than this is logged (its name and ms only).
const SLOW_MESSAGE_MS: u128 = 30;

/// [`SLOW_MESSAGE_MS`]; debug builds take `PECOFENCE_TEST_SLOW_MS` (0 logs every message, for
/// the kit's per-stage numbers).
pub fn slow_ms() -> u128 {
    #[cfg(debug_assertions)]
    {
        static MS: std::sync::OnceLock<u128> = std::sync::OnceLock::new();
        *MS.get_or_init(|| {
            std::env::var("PECOFENCE_TEST_SLOW_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(SLOW_MESSAGE_MS)
        })
    }
    #[cfg(not(debug_assertions))]
    SLOW_MESSAGE_MS
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    refdata: usize,
) -> LRESULT {
    let started = std::time::Instant::now();
    // SAFETY: as `handle`'s.
    let result = unsafe { handle(hwnd, msg, wparam, lparam, refdata) };
    let stage = match msg {
        0x0100 => "edit_keydown",
        0x0102 => "edit_char",
        0x010F => "edit_ime_composition",
        0x0286 => "edit_ime_char",
        _ => return result,
    };
    let elapsed = started.elapsed();
    if elapsed.as_millis() > slow_ms() || slow_ms() == 0 {
        tracing::info!(
            stage,
            ms = elapsed.as_millis() as u64,
            us = elapsed.as_micros() as u64,
            "edit: a message was slow"
        );
    }
    result
}

/// The subclass's own handling of one message (the key handler, Enter / Esc / Tab, the rest to
/// the EDIT control, which sends EN_CHANGE to the owner from inside WM_CHAR).
unsafe fn handle(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, refdata: usize) -> LRESULT {
    // SAFETY: refdata is the `KeyHandler` fn pointer registered in `create`.
    unsafe {
        if msg == WM_KEYDOWN as u32 && refdata != 0 {
            let handler: KeyHandler = core::mem::transmute::<usize, KeyHandler>(refdata);
            if handler(wparam.0 as u32) {
                return LRESULT(0);
            }
        }
        // The menu key and Shift+F10: DefWindowProc sends WM_CONTEXTMENU with lParam -1.
        if msg == WM_CONTEXTMENU as u32 && lparam.0 as u32 == u32::MAX && refdata != 0 {
            let handler: KeyHandler = core::mem::transmute::<usize, KeyHandler>(refdata);
            if handler(KEY_CONTEXT_MENU) {
                return LRESULT(0);
            }
        }
        // A control that takes line breaks: several lines with ES_WANTRETURN (a note). A
        // wrapping one-name box (`create_wrapping`) is multi-line without it.
        let style = GetWindowLongW(hwnd, GWL_STYLE);
        let multiline = style & ES_MULTILINE != 0;
        let breaks = multiline && style & ES_WANTRETURN != 0;
        // Enter, Esc and (one name) Tab: their keys act through the handler; the character
        // would beep or, for Tab, sit hidden in the text and eat the next Backspace. Ctrl+Enter
        // arrives as a line feed, which a multi-line control would insert.
        if msg == WM_CHAR as u32
            && (((wparam.0 == 13 || wparam.0 == 9) && !breaks)
                || (wparam.0 == 10 && multiline && !breaks)
                || wparam.0 == 27)
        {
            return LRESULT(0);
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }
}
