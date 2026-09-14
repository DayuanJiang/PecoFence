//! Thin, safe-ish wrappers over the Win32 / DWM / OLE / shell APIs PecoFence needs.
//!
//! All `unsafe` in the project lives in this crate and in `pecofence-render`. Every
//! unsafe block carries a `// SAFETY:` note. The generated `bindings` module is private;
//! other crates use the wrappers exported here.

#![allow(clippy::missing_safety_doc)]

#[allow(
    non_snake_case,
    non_upper_case_globals,
    non_camel_case_types,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations,
    clippy::useless_transmute,
    clippy::too_many_arguments,
    dead_code,
    unused_imports
)]
mod bindings;

pub mod autostart;
pub mod clipboard;
pub mod com;
pub mod crashlog;
pub mod d2d;
pub mod desktop;
pub mod dispatcher;
pub mod dragdrop;
pub mod dwm;
pub mod edit;
pub mod filedialog;
pub mod fileinfo;
pub mod frameclock;
pub mod hotkey;
pub mod layered;
pub mod locale;
pub mod memstats;
pub mod monitors;
pub mod msg;
pub mod process;
pub mod rawinput;
pub mod shell;
pub mod shell_icons;
pub mod shell_menu;
pub mod shell_notify;
pub mod sysparams;
pub mod theme;
pub mod tooltip;
pub mod tray;
pub mod wallpaper;
pub mod watcher;
pub mod wide;
pub mod window;
pub mod winevent;

pub use bindings::{HWND, POINT, RECT};
pub use windows_core::{Error, Result};

/// Raw pointer form of an `HWND`, for interop with the windows-* crates.
pub fn hwnd_ptr(hwnd: HWND) -> *mut core::ffi::c_void {
    hwnd.0
}
