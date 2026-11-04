//! UTF-16 string helpers.

/// Encodes `s` as a NUL-terminated UTF-16 buffer.
pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Decodes a NUL-terminated (or full-length) UTF-16 buffer.
pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// Decodes text in the system's ANSI code page (what a console program writes when it was
/// not asked for UTF-8).
pub fn from_ansi(bytes: &[u8]) -> String {
    use crate::bindings::{CP_ACP, MultiByteToWideChar};
    if bytes.is_empty() {
        return String::new();
    }
    let len = bytes.len().min(i32::MAX as usize) as i32;
    // SAFETY: the input pointer and length describe `bytes`; the first call only measures, the
    // second writes at most `n` units into a buffer of that size.
    unsafe {
        let n = MultiByteToWideChar(CP_ACP as u32, 0, bytes.as_ptr().cast(), len, None, 0);
        if n <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut buf = vec![0u16; n as usize];
        let n = MultiByteToWideChar(
            CP_ACP as u32,
            0,
            bytes.as_ptr().cast(),
            len,
            Some(windows_core::PWSTR(buf.as_mut_ptr())),
            n,
        );
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}
