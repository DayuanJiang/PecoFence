//! Directory Opus listers for 对话框跳转 (#240): a lister is Directory Opus's file window. It is
//! no Explorer window, so the ShellWindows object does not know its folder; Directory Opus's own
//! `dopusrt.exe /info <file>,paths` writes the folders of all open tabs to a file instead, the
//! way DialogJump.ahk (Chaoses-Ib) reads them.

use crate::bindings::{CREATE_NO_WINDOW, FindWindowW, HWND};
use crate::desktop::{class_name, is_visible, window_pid};
use crate::wide::to_wide;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use windows_core::PCWSTR;

const LISTER_CLASS: &str = "dopus.lister";
/// dopusrt answers within tens of ms; a hung Directory Opus must not hold the jump for long.
const ANSWER_MAX: Duration = Duration::from_secs(3);

/// `hwnd` is a Directory Opus lister.
pub fn is_lister(hwnd: HWND) -> bool {
    !hwnd.0.is_null() && class_name(hwnd) == LISTER_CLASS
}

/// The topmost visible lister, if Directory Opus has one open.
pub fn topmost_lister() -> Option<HWND> {
    let class = to_wide(LISTER_CLASS);
    // SAFETY: plain FFI call with a NUL-terminated class name.
    let h = unsafe { FindWindowW(PCWSTR(class.as_ptr()), PCWSTR::null()) };
    (!h.0.is_null() && is_visible(h)).then_some(h)
}

/// The folder of the active tab of `lister`, asked from Directory Opus's `dopusrt.exe` next to
/// the lister's own dopus.exe. Blocks for up to a few seconds: call it on a worker thread.
/// `None` (with the reason in the log) when dopusrt is missing, does not answer, or the tab
/// shows no file-system folder.
pub fn lister_folder(lister: HWND) -> Option<PathBuf> {
    let started = Instant::now();
    let found = read_folder(lister);
    let ms = started.elapsed().as_millis() as u64;
    match found {
        Ok(p) => {
            tracing::info!(lister = lister.0 as isize, ms, path = %p.display(), "Directory Opus folder read");
            Some(p)
        }
        Err(why) => {
            tracing::info!(
                lister = lister.0 as isize,
                ms,
                why,
                "Directory Opus folder not read"
            );
            None
        }
    }
}

fn read_folder(lister: HWND) -> Result<PathBuf, String> {
    let exe =
        crate::process::image_path(window_pid(lister)).ok_or("no image path of the lister")?;
    let rt = exe.with_file_name("dopusrt.exe");
    if !rt.is_file() {
        return Err(format!("no {}", rt.display()));
    }
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let file = std::env::temp_dir().join(format!("pecofence-dopus-{}-{n}.xml", std::process::id()));
    let _ = std::fs::remove_file(&file);
    // The path in quotes and `,paths` after them, as Directory Opus's own examples write it.
    let mut child = std::process::Command::new(&rt)
        .raw_arg(format!("/info \"{}\",paths", file.display()))
        .creation_flags(CREATE_NO_WINDOW as u32)
        .spawn()
        .map_err(|e| format!("dopusrt did not start: {e}"))?;
    let deadline = Instant::now() + ANSWER_MAX;
    // dopusrt hands the command to dopus.exe, which may finish the file just after dopusrt
    // exits: the answer is the file once its size stays the same between two reads. It has no
    // closing element of its own (measured with 13.25: a declaration, then one <path> per tab).
    let mut exited = false;
    let mut last_len = None;
    let text = loop {
        exited = exited || child.try_wait().is_ok_and(|s| s.is_some());
        if exited && let Ok(b) = std::fs::read(&file) {
            if !b.is_empty() && last_len == Some(b.len()) {
                break decode(&b);
            }
            last_len = Some(b.len());
        }
        if Instant::now() >= deadline {
            if !exited {
                let _ = child.kill();
                let _ = child.wait();
            }
            let _ = std::fs::remove_file(&file);
            return Err(format!("no answer from dopusrt in time (exited {exited})"));
        }
        std::thread::sleep(Duration::from_millis(30));
    };
    let _ = std::fs::remove_file(&file);
    let path =
        active_path(&text, lister.0 as isize).ok_or("no active tab of the lister in the answer")?;
    if path.starts_with("::") || path.is_empty() {
        return Err(format!("not a file-system folder: {path}"));
    }
    Ok(PathBuf::from(path))
}

/// The answer file in UTF-16 (with its byte-order mark) or UTF-8.
fn decode(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let units: Vec<u16> = rest
            .as_chunks::<2>()
            .0
            .iter()
            .map(|c| u16::from_le_bytes(*c))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    let rest = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8_lossy(rest).into_owned()
}

/// From the `paths` answer, the folder of the active tab (`tab_state="1"`) of the lister whose
/// window handle is `lister` (`lister="0x…"`); failing that, of the active lister.
fn active_path(xml: &str, lister: isize) -> Option<String> {
    let mut fallback = None;
    for part in xml.split("<path").skip(1) {
        let Some((attrs, rest)) = part.split_once('>') else {
            continue;
        };
        let Some((value, _)) = rest.split_once("</path>") else {
            continue;
        };
        // With the space before it: `lister="` is also the end of `active_lister="`.
        let attr = |name: &str| {
            let key = format!(" {name}=\"");
            let at = attrs.find(&key)? + key.len();
            attrs[at..].split('"').next()
        };
        if attr("tab_state") != Some("1") {
            continue;
        }
        let value = unescape(value.trim());
        let handle =
            attr("lister").and_then(|h| isize::from_str_radix(h.trim_start_matches("0x"), 16).ok());
        if handle == Some(lister) {
            return Some(value);
        }
        if fallback.is_none() && attr("active_lister") == Some("1") {
            fallback = Some(value);
        }
    }
    fallback
}

fn unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What dopusrt 13.25 wrote on a test box with two listers open (the second one active),
    /// plus a tab with an `&` in its path.
    const ANSWER: &str = r#"<?xml version="1.0"?>
	<path active_lister="0" active_tab="1" display_path="C:\" lister="0x20212" side="1" tab="0x2021a" tab_state="1">C:\</path>
	<path active_lister="0" active_tab="2" display_path="C:\" lister="0x20212" side="2" tab="0x2022e" tab_state="2">C:\</path>
	<path active_lister="1" active_tab="1" display_path="C:\pf\run\demo\Opus A" lister="0xc0684" side="1" tab="0x206c4" tab_state="1">C:\pf\run\demo\Opus A</path>
	<path active_lister="1" active_tab="2" display_path="C:\pf\run\demo\Opus A" lister="0xc0684" side="2" tab="0x106e6" tab_state="2">C:\pf\run\demo\Opus A</path>
	<path active_lister="0" active_tab="0" lister="0x1d0e6" side="1" tab="0x60d4b" tab_state="0">C:\Temp</path>
	<path active_lister="0" active_tab="1" lister="0x1d0e6" side="1" tab="0x60d5c" tab_state="1">C:\Data\R&amp;D</path>
"#;

    #[test]
    fn the_active_tab_of_the_named_lister() {
        assert_eq!(
            active_path(ANSWER, 0xc0684).as_deref(),
            Some(r"C:\pf\run\demo\Opus A")
        );
        // Not the active lister: its own active tab, not the other lister's.
        assert_eq!(active_path(ANSWER, 0x20212).as_deref(), Some(r"C:\"));
        assert_eq!(
            active_path(ANSWER, 0x1d0e6).as_deref(),
            Some(r"C:\Data\R&D")
        );
    }

    #[test]
    fn an_unknown_lister_gets_the_active_listers_tab() {
        assert_eq!(
            active_path(ANSWER, 0x999).as_deref(),
            Some(r"C:\pf\run\demo\Opus A")
        );
        assert_eq!(active_path("<?xml version=\"1.0\"?>", 1), None);
    }

    #[test]
    fn the_answer_in_utf16_or_utf8() {
        let mut wide = vec![0xFF, 0xFE];
        for u in "<path tab_state=\"1\">C:\\Ü</path>".encode_utf16() {
            wide.extend_from_slice(&u.to_le_bytes());
        }
        assert_eq!(decode(&wide), "<path tab_state=\"1\">C:\\Ü</path>");
        let utf8 = [&[0xEF, 0xBB, 0xBF][..], "<a>é</a>".as_bytes()].concat();
        assert_eq!(decode(&utf8), "<a>é</a>");
    }
}
