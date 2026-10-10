//! Notification-area icon and popup menus.

use crate::bindings::*;
use crate::wide::to_wide;
use std::cell::RefCell;
use windows_core::{Error, PCWSTR, Result};

windows_core::link!("user32.dll" "system" fn GetMenuState(hmenu: HMENU, uid: u32, uflags: u32) -> u32);
windows_core::link!("user32.dll" "system" fn GetMenuStringW(hmenu: HMENU, uiditem: u32, lpstring: windows_core::PWSTR, cchmax: i32, flags: u32) -> i32);

/// Creates a 32-bit icon from premultiplied BGRA pixels.
pub fn icon_from_bgra(width: i32, height: i32, bgra: &[u8]) -> Result<HICON> {
    // SAFETY: the pixel buffer matches width*height*4; GDI copies it.
    unsafe {
        let color = CreateBitmap(width, height, 1, 32, Some(bgra.as_ptr().cast()));
        let mask = CreateBitmap(width, height, 1, 1, None);
        let info = ICONINFO {
            fIcon: windows_core::BOOL(1),
            xHotspot: 0,
            yHotspot: 0,
            hbmMask: mask,
            hbmColor: color,
        };
        let icon = CreateIconIndirect(&info);
        let _ = DeleteObject(HGDIOBJ(color.0));
        let _ = DeleteObject(HGDIOBJ(mask.0));
        if icon.0.is_null() {
            return Err(Error::from_thread());
        }
        Ok(icon)
    }
}

/// An HICON destroyed on drop (window caption icons and the like).
pub struct OwnedIcon(HICON);

impl OwnedIcon {
    pub fn from_bgra(size: i32, bgra: &[u8]) -> Result<Self> {
        icon_from_bgra(size, size, bgra).map(Self)
    }

    /// Raw handle value for `WM_SETICON`.
    pub fn raw(&self) -> isize {
        self.0.0 as isize
    }
}

impl Drop for OwnedIcon {
    fn drop(&mut self) {
        destroy_icon(self.0);
    }
}

pub fn destroy_icon(icon: HICON) {
    // SAFETY: plain FFI call.
    unsafe {
        let _ = DestroyIcon(icon);
    }
}

/// A notification-area icon; removed on drop. Keeps what it was added with, so it can be
/// added again when Explorer restarts.
pub struct TrayIcon {
    hwnd: HWND,
    id: u32,
    callback_message: u32,
    icon: HICON,
    tip: String,
}

impl TrayIcon {
    /// An icon for `hwnd`, not in the notification area until [`Self::show`]. Notifications
    /// arrive as `callback_message` on `hwnd` (NOTIFYICON_VERSION_4: `LOWORD(lParam)` = event
    /// such as `WM_CONTEXTMENU`/`NIN_SELECT`, x/y in wParam). `icon` must outlive the tray icon.
    pub fn new(hwnd: HWND, id: u32, callback_message: u32, icon: HICON, tip: &str) -> Self {
        Self {
            hwnd,
            id,
            callback_message,
            icon,
            tip: tip.to_string(),
        }
    }

    /// Adds the icon to the notification area. Call it again on `TaskbarCreated`: a restarted
    /// Explorer starts without anyone's icons. Already present (the message also comes on
    /// other occasions) counts as success.
    pub fn show(&self) -> Result<()> {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: self.id,
            uFlags: (NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP) as u32,
            uCallbackMessage: self.callback_message,
            hIcon: self.icon,
            ..Default::default()
        };
        copy_tip(&mut data.szTip, &self.tip);
        // SAFETY: fully initialized structure.
        unsafe {
            if !Shell_NotifyIconW(NIM_ADD as u32, &data).as_bool() {
                let err = Error::from_thread();
                if !Shell_NotifyIconW(NIM_MODIFY as u32, &data).as_bool() {
                    return Err(err);
                }
            }
            data.Anonymous.uVersion = NOTIFYICON_VERSION_4 as u32;
            let _ = Shell_NotifyIconW(NIM_SETVERSION as u32, &data);
        }
        Ok(())
    }

    pub fn set_tip(&self, tip: &str) {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: self.id,
            uFlags: (NIF_TIP | NIF_SHOWTIP) as u32,
            ..Default::default()
        };
        copy_tip(&mut data.szTip, tip);
        // SAFETY: fully initialized structure.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY as u32, &data);
        }
    }

    /// Shows a toast/balloon notification.
    pub fn show_info(&self, title: &str, text: &str, warning: bool) {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: self.id,
            uFlags: NIF_INFO as u32,
            dwInfoFlags: if warning {
                NIIF_WARNING as u32
            } else {
                NIIF_INFO as u32
            },
            ..Default::default()
        };
        copy_wide(&mut data.szInfoTitle, title);
        copy_wide(&mut data.szInfo, text);
        // SAFETY: fully initialized structure.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY as u32, &data);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: self.id,
            ..Default::default()
        };
        // SAFETY: fully initialized structure.
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE as u32, &data);
        }
    }
}

fn copy_tip(dst: &mut [u16; 128], s: &str) {
    copy_wide(dst, s);
}

fn copy_wide<const N: usize>(dst: &mut [u16; N], s: &str) {
    let w = to_wide(s);
    let n = w.len().min(N - 1);
    dst[..n].copy_from_slice(&w[..n]);
    dst[n] = 0;
}

/// Tray callback decoding for NOTIFYICON_VERSION_4.
pub struct TrayEvent {
    pub event: u32,
    pub x: i32,
    pub y: i32,
}

pub fn decode_tray_message(wparam: usize, lparam: isize) -> TrayEvent {
    TrayEvent {
        event: (lparam & 0xffff) as u32,
        x: (wparam & 0xffff) as u16 as i16 as i32,
        y: ((wparam >> 16) & 0xffff) as u16 as i16 as i32,
    }
}

pub const TRAY_EVENT_CONTEXTMENU: u32 = WM_CONTEXTMENU as u32;
pub const TRAY_EVENT_SELECT: u32 = NIN_SELECT as u32;
pub const TRAY_EVENT_KEYSELECT: u32 = NIN_KEYSELECT as u32;
/// The `show_info` balloon was clicked / went away unclicked (timed out or replaced).
pub const TRAY_EVENT_BALLOONUSERCLICK: u32 = NIN_BALLOONUSERCLICK as u32;
pub const TRAY_EVENT_BALLOONTIMEOUT: u32 = NIN_BALLOONTIMEOUT as u32;
pub const TRAY_EVENT_BALLOONHIDE: u32 = NIN_BALLOONHIDE as u32;

/// Draws the Segoe Fluent Icons `glyph` into a `px`-square top-down 32-bit DIB section in
/// `color` (a COLORREF) with premultiplied alpha, the format themed menus alpha-blend as an
/// item bitmap. GDI renders white-on-black grayscale coverage that becomes the alpha.
fn glyph_bitmap(glyph: char, px: i32, color: u32) -> Option<HBITMAP> {
    // SAFETY: the DC, font and bitmap created here are deselected and freed before return
    // (the bitmap is handed to the caller); `bits` spans px*px 4-byte pixels of the DIB and
    // is read only after `GdiFlush`.
    unsafe {
        let mem = CreateCompatibleDC(None);
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: px,
                biHeight: -px,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
        let dib = CreateDIBSection(Some(mem), &info, DIB_RGB_COLORS as u32, &mut bits, None, 0);
        if dib.0.is_null() || bits.is_null() {
            let _ = DeleteDC(mem);
            return None;
        }
        let face = to_wide(crate::fonts::icon_face());
        let font = CreateFontW(
            -px,
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
            ANTIALIASED_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            PCWSTR(face.as_ptr()),
        );
        let old_bitmap = SelectObject(mem, HGDIOBJ(dib.0));
        let old_font = SelectObject(mem, HGDIOBJ(font.0));
        let _ = SetBkColor(mem, COLORREF(0));
        let _ = SetTextColor(mem, COLORREF(0xFF_FFFF));
        let mut buf = [0u16; 2];
        let text = glyph.encode_utf16(&mut buf);
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: px,
            bottom: px,
        };
        DrawTextW(
            mem,
            PCWSTR(text.as_ptr()),
            text.len() as i32,
            &mut rect,
            (DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX) as u32,
        );
        let _ = GdiFlush();
        SelectObject(mem, old_font);
        SelectObject(mem, old_bitmap);
        let _ = DeleteObject(HGDIOBJ(font.0));
        let _ = DeleteDC(mem);
        let pixels = core::slice::from_raw_parts_mut(bits.cast::<[u8; 4]>(), (px * px) as usize);
        let [r, g, b] = [color & 0xFF, (color >> 8) & 0xFF, (color >> 16) & 0xFF];
        for p in pixels {
            let a = p[0].max(p[1]).max(p[2]) as u32;
            *p = [
                (b * a / 255) as u8,
                (g * a / 255) as u8,
                (r * a / 255) as u8,
                a as u8,
            ];
        }
        Some(dib)
    }
}

/// A popup menu built from items; destroyed on drop.
pub struct PopupMenu {
    hmenu: HMENU,
    /// Item bitmaps from `icon`, its submenus' included: a menu does not own them.
    bitmaps: RefCell<Vec<HBITMAP>>,
}

impl Default for PopupMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupMenu {
    pub fn new() -> Self {
        // SAFETY: plain FFI call.
        Self {
            hmenu: unsafe { CreatePopupMenu() },
            bitmaps: RefCell::default(),
        }
    }

    pub fn handle(&self) -> HMENU {
        self.hmenu
    }

    /// Number of items currently in the menu (= the insertion index for appending).
    pub fn len(&self) -> u32 {
        // SAFETY: plain FFI call on a menu we own.
        unsafe { GetMenuItemCount(Some(self.hmenu)).max(0) as u32 }
    }

    pub fn item(&self, id: u32, text: &str, checked: bool, disabled: bool) -> &Self {
        let w = to_wide(text);
        let mut flags = MF_STRING as u32;
        if checked {
            flags |= MF_CHECKED as u32;
        }
        if disabled {
            flags |= (MF_GRAYED | MF_DISABLED) as u32;
        }
        // SAFETY: string outlives the call.
        unsafe {
            let _ = AppendMenuW(self.hmenu, flags, id as usize, PCWSTR(w.as_ptr()));
        }
        self
    }

    /// Marks `id` as the default item (drawn bold; Enter / double-click pick it), as the shell's
    /// right-drag menu does for the effect the modifiers would have produced.
    pub fn default_item(&self, id: u32) -> &Self {
        windows_core::link!("user32.dll" "system" fn SetMenuDefaultItem(hmenu: HMENU, uitem: u32, fbypos: u32) -> windows_core::BOOL);
        // SAFETY: plain FFI call on a menu we own; `fbypos` = FALSE means `uitem` is an id.
        unsafe {
            let _ = SetMenuDefaultItem(self.hmenu, id, 0);
        }
        self
    }

    /// Shows the Segoe Fluent Icons `glyph` beside the item added last, in the menu text
    /// colour (grey for a disabled item) at the system small-icon size for the monitor under
    /// the cursor, where context and tray menus open: 16 DIP, the size the font is drawn
    /// crisp at. Like Windows' own menu icons it follows the DPI, not Settings › Text size.
    pub fn icon(&self, glyph: char) -> &Self {
        let Some(pos) = self.len().checked_sub(1) else {
            return self;
        };
        let mut cursor = POINT::default();
        // SAFETY: out-pointer references a local.
        unsafe {
            let _ = GetCursorPos(&mut cursor);
        }
        let dpi = crate::monitors::query(crate::monitors::monitor_from_point(cursor.x, cursor.y))
            .map_or(USER_DEFAULT_SCREEN_DPI as u32, |m| m.dpi);
        // SAFETY: plain FFI calls on a menu we own; the bitmap outlives the menu (`Drop`).
        unsafe {
            let px = GetSystemMetricsForDpi(SM_CXSMICON, dpi);
            let disabled = GetMenuState(self.hmenu, pos, MF_BYPOSITION as u32) & MF_GRAYED as u32;
            let color = GetSysColor(if disabled != 0 {
                COLOR_GRAYTEXT
            } else {
                COLOR_MENUTEXT
            });
            let Some(bitmap) = glyph_bitmap(glyph, px, color) else {
                return self;
            };
            let info = MENUITEMINFOW {
                cbSize: size_of::<MENUITEMINFOW>() as u32,
                fMask: MIIM_BITMAP as u32,
                hbmpItem: bitmap,
                ..Default::default()
            };
            let _ = SetMenuItemInfoW(self.hmenu, pos, true, &info);
            self.bitmaps.borrow_mut().push(bitmap);
        }
        self
    }

    pub fn separator(&self) -> &Self {
        // SAFETY: plain FFI call.
        unsafe {
            let _ = AppendMenuW(self.hmenu, MF_SEPARATOR as u32, 0, PCWSTR::null());
        }
        self
    }

    /// Adds `sub` as a submenu (ownership transfers to this menu).
    pub fn submenu(&self, text: &str, sub: PopupMenu) -> &Self {
        let w = to_wide(text);
        let h = sub.hmenu;
        self.bitmaps
            .borrow_mut()
            .append(&mut sub.bitmaps.borrow_mut());
        std::mem::forget(sub);
        // SAFETY: string outlives the call; submenu handle is now owned by the parent.
        unsafe {
            let _ = AppendMenuW(
                self.hmenu,
                (MF_STRING | MF_POPUP) as u32,
                h.0 as usize,
                PCWSTR(w.as_ptr()),
            );
        }
        self
    }

    /// Top-level labels in order, separators as `-` (test dumps).
    pub fn labels(&self) -> Vec<String> {
        let mut out = Vec::new();
        for pos in 0..self.len() {
            if self.is_separator(pos) {
                out.push("-".to_string());
                continue;
            }
            // SAFETY: plain FFI call on a menu we own; the buffer size is passed in chars.
            unsafe {
                let mut buf = [0u16; 256];
                let n = GetMenuStringW(
                    self.hmenu,
                    pos,
                    windows_core::PWSTR(buf.as_mut_ptr()),
                    buf.len() as i32,
                    MF_BYPOSITION as u32,
                );
                out.push(String::from_utf16_lossy(&buf[..n.max(0) as usize]));
            }
        }
        out
    }

    fn is_separator(&self, pos: u32) -> bool {
        // SAFETY: plain FFI call on a menu we own.
        unsafe { GetMenuState(self.hmenu, pos, MF_BYPOSITION as u32) & MF_SEPARATOR as u32 != 0 }
    }

    /// Drops separators at either end and all but one of any run (left behind when items
    /// between them were removed, e.g. filtered shell verbs).
    pub fn tidy_separators(&self) {
        let mut after_is_separator = true; // the end counts as one: trailing ones go
        for pos in (0..self.len()).rev() {
            if !self.is_separator(pos) {
                after_is_separator = false;
            } else if after_is_separator {
                // SAFETY: plain FFI call on a menu we own; separators own no submenu.
                unsafe {
                    let _ = RemoveMenu(self.hmenu, pos, MF_BYPOSITION as u32);
                }
            } else {
                after_is_separator = true;
            }
        }
        if self.len() > 0 && self.is_separator(0) {
            // SAFETY: as above.
            unsafe {
                let _ = RemoveMenu(self.hmenu, 0, MF_BYPOSITION as u32);
            }
        }
    }

    /// Shows the menu at screen coordinates and returns the chosen command id (0 = dismissed).
    /// `owner` is brought to the foreground first (required for tray menus to dismiss properly).
    pub fn show(&self, owner: HWND, x: i32, y: i32) -> u32 {
        self.show_aligned(owner, x, y, true)
    }

    /// Context-menu placement: opens downwards from the click point (Explorer-style).
    pub fn show_context(&self, owner: HWND, x: i32, y: i32) -> u32 {
        self.show_aligned(owner, x, y, false)
    }

    fn show_aligned(&self, owner: HWND, x: i32, y: i32, bottom_align: bool) -> u32 {
        let mut flags = (TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_LEFTALIGN) as u32;
        if bottom_align {
            flags |= TPM_BOTTOMALIGN as u32;
        }
        // SAFETY: plain FFI calls; WM_NULL nudge is the documented tray-menu workaround.
        unsafe {
            let _ = SetForegroundWindow(owner);
            let cmd = TrackPopupMenuEx(self.hmenu, flags, x, y, owner, None);
            let _ = PostMessageW(Some(owner), WM_NULL as u32, WPARAM(0), LPARAM(0));
            cmd.0 as u32
        }
    }
}

impl Drop for PopupMenu {
    fn drop(&mut self) {
        // SAFETY: we own the menu and its item bitmaps; the menu goes first.
        unsafe {
            let _ = DestroyMenu(self.hmenu);
            for bitmap in self.bitmaps.take() {
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    windows_core::link!("gdi32.dll" "system" fn GetDIBits(hdc: HDC, hbm: HBITMAP, start: u32, lines: u32, bits: *mut core::ffi::c_void, info: *mut BITMAPINFO, usage: u32) -> i32);

    /// The glyph lands centred in its square with premultiplied colour: no channel above
    /// its alpha, which themed menus would otherwise blend as a brightened halo.
    #[test]
    fn glyph_bitmaps_are_centred_and_premultiplied() {
        for px in [16, 24, 32] {
            let bitmap = glyph_bitmap('\u{E713}', px, 0x0000_00FF).expect("bitmap");
            let mut bits = vec![[0u8; 4]; (px * px) as usize];
            // SAFETY: the bitmap is a px*px 32-bit DIB section; the buffer matches.
            unsafe {
                let mem = CreateCompatibleDC(None);
                let mut info = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: px,
                        biHeight: -px,
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB as u32,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                GetDIBits(
                    mem,
                    bitmap,
                    0,
                    px as u32,
                    bits.as_mut_ptr().cast(),
                    &mut info,
                    DIB_RGB_COLORS as u32,
                );
                let _ = DeleteDC(mem);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
            }
            let (mut x0, mut y0, mut x1, mut y1) = (px, px, 0, 0);
            for (i, p) in bits.iter().enumerate() {
                assert!(p[0] == 0 && p[1] == 0 && p[2] <= p[3], "{p:?}");
                if p[3] > 0 {
                    let (x, y) = (i as i32 % px, i as i32 / px);
                    (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                }
            }
            assert!(
                x1 - x0 >= px / 2 && y1 - y0 >= px / 2,
                "glyph drawn at {px}"
            );
            assert!((x0 - (px - 1 - x1)).abs() <= 1, "x {x0}..{x1} of {px}");
            assert!((y0 - (px - 1 - y1)).abs() <= 1, "y {y0}..{y1} of {px}");
        }
    }
}
