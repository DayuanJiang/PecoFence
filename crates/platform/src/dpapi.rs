//! DPAPI: bytes only this Windows account on this computer can read again (a plugin's secret
//! settings in the configuration).

use crate::bindings::{HANDLE, LocalFree};
use core::ffi::c_void;
use core::ptr::{null, null_mut};

#[repr(C)]
struct Blob {
    len: u32,
    data: *mut u8,
}

windows_core::link!("crypt32.dll" "system" fn CryptProtectData(input: *const Blob, description: *const u16, entropy: *const Blob, reserved: *const c_void, prompt: *const c_void, flags: u32, output: *mut Blob) -> windows_core::BOOL);
windows_core::link!("crypt32.dll" "system" fn CryptUnprotectData(input: *const Blob, description: *mut *mut u16, entropy: *const Blob, reserved: *const c_void, prompt: *const c_void, flags: u32, output: *mut Blob) -> windows_core::BOOL);

/// `CRYPTPROTECT_UI_FORBIDDEN`: never a prompt.
const UI_FORBIDDEN: u32 = 1;

fn crypt(input: &[u8], protect: bool) -> Option<Vec<u8>> {
    let blob = Blob {
        len: input.len() as u32,
        data: input.as_ptr().cast_mut(),
    };
    let mut out = Blob {
        len: 0,
        data: null_mut(),
    };
    // SAFETY: plain FFI calls; `input` outlives them, the output is copied out and freed with
    // LocalFree as DPAPI asks.
    unsafe {
        let ok = if protect {
            CryptProtectData(
                &blob,
                null(),
                null(),
                null(),
                null(),
                UI_FORBIDDEN,
                &mut out,
            )
        } else {
            CryptUnprotectData(
                &blob,
                null_mut(),
                null(),
                null(),
                null(),
                UI_FORBIDDEN,
                &mut out,
            )
        };
        if !ok.as_bool() || out.data.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(out.data, out.len as usize).to_vec();
        let _ = LocalFree(HANDLE(out.data.cast()));
        Some(bytes)
    }
}

/// `data` encrypted for this Windows account.
pub fn protect(data: &[u8]) -> Option<Vec<u8>> {
    crypt(data, true)
}

/// What [`protect`] encrypted, when this account on this computer did it.
pub fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
    crypt(data, false)
}
