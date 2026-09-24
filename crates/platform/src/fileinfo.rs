//! Per-file display facts for the details view: the shell's type name ("文本文档",
//! "快捷方式") and locale-formatted local dates, the same way Explorer shows them.

use crate::bindings::*;
use crate::wide::to_wide;
use std::path::Path;
use windows_core::{PCWSTR, PWSTR};

/// Explorer's "类型" column text for a path. Files use the extension/registry only
/// (`SHGFI_USEFILEATTRIBUTES`), so the disk is never touched and the call is cheap enough for
/// the UI thread. Folders report the shell's folder type name. Namespace items (`::{CLSID}`)
/// are parsed to an id list first: the shell does not resolve their names by string here.
pub fn type_name(path: &Path, is_folder: bool) -> Option<String> {
    let w = to_wide(&path.to_string_lossy());
    let mut info = SHFILEINFOW::default();
    let ok = if crate::shell::is_namespace_path(path) {
        let mut pidl: LPITEMIDLIST = std::ptr::null_mut();
        // SAFETY: the string outlives the call; the id list is freed below.
        let hr =
            unsafe { SHParseDisplayName(PCWSTR(w.as_ptr()), None, &mut pidl, SFGAOF(0), None) };
        if hr.is_err() || pidl.is_null() {
            return None;
        }
        // SAFETY: with SHGFI_PIDL the first argument is the id list; `info` is a live
        // out-struct of the declared size.
        let ok = unsafe {
            SHGetFileInfoW(
                PCWSTR(pidl.cast()),
                0,
                Some(&mut info),
                size_of::<SHFILEINFOW>() as u32,
                (SHGFI_TYPENAME | SHGFI_PIDL) as u32,
            )
        };
        // SAFETY: the id list came from SHParseDisplayName.
        unsafe { ILFree(Some(pidl)) };
        ok
    } else {
        let attrs = if is_folder {
            FILE_ATTRIBUTE_DIRECTORY as u32
        } else {
            FILE_ATTRIBUTE_NORMAL as u32
        };
        // SAFETY: the string outlives the call; `info` is a live out-struct of the declared size.
        unsafe {
            SHGetFileInfoW(
                PCWSTR(w.as_ptr()),
                attrs,
                Some(&mut info),
                size_of::<SHFILEINFOW>() as u32,
                (SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES) as u32,
            )
        }
    };
    if ok == 0 {
        return None;
    }
    let len = info
        .szTypeName
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(info.szTypeName.len());
    let s = String::from_utf16_lossy(&info.szTypeName[..len]);
    (!s.is_empty()).then_some(s)
}

/// Unix seconds → "short date + time without seconds" in the user's locale and time zone
/// (Explorer's 修改日期 column). Returns an empty string for out-of-range values.
pub fn format_local_datetime(unix: i64) -> String {
    let Some(st) = local_system_time(unix) else {
        return String::new();
    };
    let mut date = [0u16; 64];
    let mut time = [0u16; 64];
    // SAFETY: the buffers are live and their lengths are passed; `st` is a valid SYSTEMTIME.
    let (nd, nt) = unsafe {
        (
            GetDateFormatEx(
                PCWSTR::null(),
                DATE_SHORTDATE as u32,
                Some(&st),
                PCWSTR::null(),
                Some(PWSTR(date.as_mut_ptr())),
                date.len() as i32,
                PCWSTR::null(),
            ),
            GetTimeFormatEx(
                PCWSTR::null(),
                TIME_NOSECONDS as u32,
                Some(&st),
                PCWSTR::null(),
                Some(PWSTR(time.as_mut_ptr())),
                time.len() as i32,
            ),
        )
    };
    let d = if nd > 1 {
        String::from_utf16_lossy(&date[..(nd - 1) as usize])
    } else {
        String::new()
    };
    let t = if nt > 1 {
        String::from_utf16_lossy(&time[..(nt - 1) as usize])
    } else {
        String::new()
    };
    match (d.is_empty(), t.is_empty()) {
        (false, false) => format!("{d} {t}"),
        (false, true) => d,
        _ => t,
    }
}

/// Local minutes since midnight and weekday (0 = Monday … 6 = Sunday) for a Unix time — the
/// facts the rule engine's time conditions compare against.
pub fn local_time_parts(unix: i64) -> Option<(u16, u8)> {
    let st = local_system_time(unix)?;
    let minutes = st.wHour * 60 + st.wMinute;
    // SYSTEMTIME: 0 = Sunday … 6 = Saturday → rules: 0 = Monday … 6 = Sunday.
    let weekday = ((st.wDayOfWeek + 6) % 7) as u8;
    Some((minutes, weekday))
}

/// Local calendar date (year, month 1..=12, day 1..=31) of a Unix time — the input of the
/// "按时间分组" bucketing in `pecofence_core::date_group`.
pub fn local_civil_date(unix: i64) -> Option<(i32, u8, u8)> {
    let st = local_system_time(unix)?;
    Some((i32::from(st.wYear), st.wMonth as u8, st.wDay as u8))
}

/// Unix seconds → `yyyy-mm-dd HH:MM:SS` in the local time zone (sortable, locale-independent;
/// used for generated names). Empty for out-of-range values.
pub fn format_local_timestamp(unix: i64) -> String {
    let Some(st) = local_system_time(unix) else {
        return String::new();
    };
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond
    )
}

fn local_system_time(unix: i64) -> Option<SYSTEMTIME> {
    // FILETIME: 100-ns intervals since 1601-01-01.
    let ticks = unix.checked_add(11_644_473_600)?.checked_mul(10_000_000)?;
    if ticks < 0 {
        return None;
    }
    let ft = FILETIME {
        dwLowDateTime: (ticks as u64 & 0xFFFF_FFFF) as u32,
        dwHighDateTime: (ticks as u64 >> 32) as u32,
    };
    let mut utc = SYSTEMTIME::default();
    let mut local = SYSTEMTIME::default();
    // SAFETY: all pointers reference live locals.
    unsafe {
        if !FileTimeToSystemTime(&ft, &mut utc).as_bool() {
            return None;
        }
        if !SystemTimeToTzSpecificLocalTime(None, &utc, &mut local).as_bool() {
            return None;
        }
    }
    Some(local)
}

/// Explorer's "大小" column: whole kilobytes with thousands separators, folders blank.
pub fn format_size_kb(bytes: u64, is_folder: bool) -> String {
    if is_folder {
        return String::new();
    }
    let kb = bytes.div_ceil(1024).max(if bytes > 0 { 1 } else { 0 });
    let digits = kb.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.push_str(" KB");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_round_up_like_explorer() {
        assert_eq!(format_size_kb(0, false), "0 KB");
        assert_eq!(format_size_kb(1, false), "1 KB");
        assert_eq!(format_size_kb(1024, false), "1 KB");
        assert_eq!(format_size_kb(1025, false), "2 KB");
        assert_eq!(format_size_kb(1_234_567, false), "1,206 KB");
        assert_eq!(format_size_kb(5, true), "");
    }

    #[test]
    fn dates_format_in_local_time() {
        let s = format_local_datetime(1_757_300_000);
        assert!(!s.is_empty());
        assert!(format_local_datetime(-99_999_999_999).is_empty());
    }

    #[test]
    fn weekday_is_monday_first() {
        // 2026-09-07 00:00 UTC is a Monday; every local zone within ±12 h stays on Monday
        // between 12:00 and 12:00, so use noon UTC.
        let (_minutes, weekday) = local_time_parts(1_788_782_400).expect("valid time");
        assert_eq!(weekday, 0, "2026-09-07 is a Monday → 0");
        let (_m, sunday) = local_time_parts(1_788_782_400 + 6 * 86_400).unwrap();
        assert_eq!(sunday, 6);
    }

    #[test]
    fn civil_dates_are_local() {
        // 2026-09-07 12:00 UTC stays on the 7th in every zone within ±12 h.
        assert_eq!(local_civil_date(1_788_782_400), Some((2026, 9, 7)));
        assert_eq!(local_civil_date(-99_999_999_999), None);
    }

    #[test]
    fn type_names_come_from_the_shell() {
        let t = type_name(Path::new(r"C:\x\y.txt"), false);
        assert!(t.is_some());
        assert!(type_name(Path::new(r"C:\x\folder"), true).is_some());
    }
}
