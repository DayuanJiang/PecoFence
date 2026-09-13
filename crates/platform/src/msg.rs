//! Window message and hit-test constants as plain `u32` / `isize`, so consumers never touch
//! the generated bindings (whose constant types vary between `i32` and `u32`).

macro_rules! u32_consts {
    ($($name:ident),* $(,)?) => {
        $(pub const $name: u32 = crate::bindings::$name as u32;)*
    };
}

macro_rules! isize_consts {
    ($($name:ident),* $(,)?) => {
        $(pub const $name: isize = crate::bindings::$name as isize;)*
    };
}

u32_consts!(
    WM_CREATE,
    WM_DESTROY,
    WM_NCDESTROY,
    WM_CLOSE,
    WM_QUIT,
    WM_SIZE,
    WM_MOVE,
    WM_PAINT,
    WM_ERASEBKGND,
    WM_NCCALCSIZE,
    WM_NCPAINT,
    WM_NCHITTEST,
    WM_NCACTIVATE,
    WM_ACTIVATE,
    WM_ACTIVATEAPP,
    WM_MOUSEACTIVATE,
    WM_SETFOCUS,
    WM_KILLFOCUS,
    WM_WINDOWPOSCHANGING,
    WM_WINDOWPOSCHANGED,
    WM_DPICHANGED,
    WM_DISPLAYCHANGE,
    WM_SETTINGCHANGE,
    WM_THEMECHANGED,
    WM_TIMER,
    WM_INPUT,
    WM_HOTKEY,
    WM_APP,
    WM_USER,
    WM_COMMAND,
    WM_CONTEXTMENU,
    WM_MOUSEMOVE,
    WM_MOUSELEAVE,
    WM_MOUSEWHEEL,
    WM_LBUTTONDOWN,
    WM_LBUTTONUP,
    WM_LBUTTONDBLCLK,
    WM_RBUTTONDOWN,
    WM_RBUTTONUP,
    WM_MBUTTONDOWN,
    WM_NCLBUTTONDOWN,
    WM_NCLBUTTONDBLCLK,
    WM_NCRBUTTONUP,
    WM_NCMOUSEMOVE,
    WM_NCMOUSELEAVE,
    WM_ENTERMENULOOP,
    WM_EXITMENULOOP,
    WM_KEYDOWN,
    WM_KEYUP,
    WM_CHAR,
    WM_SYSKEYDOWN,
    WM_SYSCOMMAND,
    WM_CAPTURECHANGED,
    WM_SETCURSOR,
    WM_GETMINMAXINFO,
    WM_ENTERSIZEMOVE,
    WM_EXITSIZEMOVE,
    WM_SIZING,
    WM_MOVING,
    WM_SHOWWINDOW,
    WM_DWMCOMPOSITIONCHANGED,
    WM_DWMCOLORIZATIONCOLORCHANGED,
    WM_ENDSESSION,
    WM_QUERYENDSESSION,
    MK_LBUTTON,
    MK_CONTROL,
    MK_SHIFT,
    VK_ESCAPE,
    VK_RETURN,
    VK_DELETE,
    VK_F2,
    VK_SHIFT,
    VK_CONTROL,
    VK_MENU,
    VK_LBUTTON,
    SC_MOVE,
    SC_SIZE,
);

isize_consts!(
    HTCLIENT,
    HTCAPTION,
    HTTRANSPARENT,
    HTNOWHERE,
    HTLEFT,
    HTRIGHT,
    HTTOP,
    HTTOPLEFT,
    HTTOPRIGHT,
    HTBOTTOM,
    HTBOTTOMLEFT,
    HTBOTTOMRIGHT,
    MA_NOACTIVATE,
    MA_ACTIVATE,
    MA_NOACTIVATEANDEAT,
);

pub const WHEEL_DELTA: i32 = crate::bindings::WHEEL_DELTA;

/// Extracts the low-order 16-bit signed value (e.g. x coordinate) from an `LPARAM`.
pub fn lo_i16(lparam: isize) -> i32 {
    (lparam & 0xffff) as u16 as i16 as i32
}

/// Extracts the high-order 16-bit signed value (e.g. y coordinate) from an `LPARAM`.
pub fn hi_i16(lparam: isize) -> i32 {
    ((lparam >> 16) & 0xffff) as u16 as i16 as i32
}

/// Packs two 16-bit signed values (e.g. a point) into an `LPARAM` (`MAKELPARAM`).
pub fn make_lparam(lo: i32, hi: i32) -> isize {
    ((lo as i16 as u16 as isize) & 0xffff) | (((hi as i16 as u16 as isize) & 0xffff) << 16)
}

/// Low-order word of a `WPARAM` as `u32`.
pub fn lo_u16(wparam: usize) -> u32 {
    (wparam & 0xffff) as u32
}

/// High-order word of a `WPARAM` as `u32`.
pub fn hi_u16(wparam: usize) -> u32 {
    ((wparam >> 16) & 0xffff) as u32
}
