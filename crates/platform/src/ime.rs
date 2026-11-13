//! The input method (IME) calls the Peek dimmer needs: a key the input method took there
//! (the first letter of a Chinese or Japanese word) is cancelled and typed again, so its
//! composition starts in the search box.

use crate::bindings::*;

windows_core::link!("imm32.dll" "system" fn ImmGetContext(hwnd: HWND) -> isize);
windows_core::link!("imm32.dll" "system" fn ImmReleaseContext(hwnd: HWND, himc: isize) -> windows_core::BOOL);
windows_core::link!("imm32.dll" "system" fn ImmNotifyIME(himc: isize, action: u32, index: u32, value: u32) -> windows_core::BOOL);
windows_core::link!("imm32.dll" "system" fn ImmGetVirtualKey(hwnd: HWND) -> u32);
windows_core::link!("user32.dll" "system" fn GetMessageExtraInfo() -> isize);

/// wParam of a `WM_KEYDOWN` the input method took for a composition.
pub const VK_PROCESSKEY: u32 = 0xE5;
const NI_COMPOSITIONSTR: u32 = 0x0015;
const CPS_CANCEL: u32 = 0x0004;
const MAPVK_VSC_TO_VK: u32 = 1;
/// `dwExtraInfo` of the key [`type_again`] injects ("PFIM").
const TYPED_AGAIN: usize = 0x5046_494D;

/// The key behind a `WM_KEYDOWN` with [`VK_PROCESSKEY`] (`lparam` is the message's). Once
/// TranslateMessage has run, Windows answers VK_PROCESSKEY again: the scan code still says.
pub fn key_taken(hwnd: HWND, lparam: isize) -> u32 {
    // SAFETY: plain FFI calls.
    unsafe {
        let vk = ImmGetVirtualKey(hwnd);
        if vk != 0 && vk != VK_PROCESSKEY {
            return vk;
        }
        let scan = (lparam >> 16) as u32 & 0xFF;
        MapVirtualKeyExW(scan, MAPVK_VSC_TO_VK, Some(GetKeyboardLayout(0)))
    }
}

/// Drops the composition `hwnd` has open, with nothing committed. False when the window has
/// no input context or the input method refused.
pub fn cancel_composition(hwnd: HWND) -> bool {
    // SAFETY: the context is released before returning.
    unsafe {
        let himc = ImmGetContext(hwnd);
        if himc == 0 {
            return false;
        }
        let ok = ImmNotifyIME(himc, NI_COMPOSITIONSTR, CPS_CANCEL, 0).as_bool();
        let _ = ImmReleaseContext(hwnd, himc);
        ok
    }
}

/// Presses and releases `vk` as if typed, marked so [`typed_again`] knows it: it reaches the
/// window that has the keyboard by the time it is read, input method included.
pub fn type_again(vk: u32, lparam: isize) {
    let scan = (lparam >> 16) as u8;
    // SAFETY: plain FFI calls injecting one key press.
    unsafe {
        keybd_event(vk as u8, scan, 0, TYPED_AGAIN);
        keybd_event(vk as u8, scan, KEYEVENTF_KEYUP as u32, TYPED_AGAIN);
    }
}

/// Whether the key message being handled is one [`type_again`] injected.
pub fn typed_again() -> bool {
    // SAFETY: plain FFI call.
    unsafe { GetMessageExtraInfo() as usize == TYPED_AGAIN }
}
