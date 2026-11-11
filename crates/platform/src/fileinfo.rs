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

/// A shell icon (HICON), destroyed on drop.
pub struct FileIcon(HICON);

// SAFETY: an HICON is a process-wide USER handle, not tied to the thread that made it: the
// Peek list reads icons on a worker thread and draws and destroys them on the UI thread.
unsafe impl Send for FileIcon {}

impl FileIcon {
    pub(crate) fn handle(&self) -> HICON {
        self.0
    }
}

impl Drop for FileIcon {
    fn drop(&mut self) {
        // SAFETY: the icon came from SHGetFileInfoW and is owned here.
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}

/// The small icon Explorer shows for `path`. `exact` reads the file itself (a program's own
/// icon, a shortcut's target icon); otherwise only its type counts, which never touches the
/// disk (for network and removable drives). Namespace items (`::{CLSID}`) go through their id
/// list.
pub fn small_icon(path: &Path, is_folder: bool, exact: bool) -> Option<FileIcon> {
    let w = to_wide(&path.to_string_lossy());
    let mut info = SHFILEINFOW::default();
    let flags = (SHGFI_ICON | SHGFI_SMALLICON) as u32;
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
                flags | SHGFI_PIDL as u32,
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
        let extra = if exact {
            0
        } else {
            SHGFI_USEFILEATTRIBUTES as u32
        };
        // SAFETY: the string outlives the call; `info` is a live out-struct of the declared size.
        unsafe {
            SHGetFileInfoW(
                PCWSTR(w.as_ptr()),
                attrs,
                Some(&mut info),
                size_of::<SHFILEINFOW>() as u32,
                flags | extra,
            )
        }
    };
    (ok != 0 && !info.hIcon.0.is_null()).then(|| FileIcon(info.hIcon))
}

/// Unix seconds → the local time of day without seconds in the user's locale ("14:32",
/// "2:32 PM"). Empty for out-of-range values.
pub fn format_local_time(unix: i64) -> String {
    let Some(st) = local_system_time(unix) else {
        return String::new();
    };
    let mut buf = [0u16; 64];
    // SAFETY: the buffer is live and its length is passed; `st` is a valid SYSTEMTIME.
    let n = unsafe {
        GetTimeFormatEx(
            PCWSTR::null(),
            TIME_NOSECONDS as u32,
            Some(&st),
            PCWSTR::null(),
            Some(PWSTR(buf.as_mut_ptr())),
            buf.len() as i32,
        )
    };
    if n > 1 {
        String::from_utf16_lossy(&buf[..(n - 1) as usize])
    } else {
        String::new()
    }
}

/// 时钟与日期: the local time as Windows' region format writes it (12 or 24 hours; `seconds`
/// adds them) and the region's long date, with the weekday after it when the long date has
/// none ("2026年11月4日 星期三", "Wednesday, November 4, 2026"), in the user's time zone. The
/// third text is the date for a clock too narrow for that line: the region's month and day
/// with the short weekday where the long date has it ("11月4日 周三", "Wed, Nov 4").
pub fn clock_texts(seconds: bool) -> (String, String, String) {
    let unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let Some(st) = local_system_time(unix) else {
        return (String::new(), String::new(), String::new());
    };
    const DATE_LONGDATE: u32 = 2;
    let mut buf = [0u16; 96];
    let text = |n: i32, buf: &[u16]| {
        if n > 1 {
            String::from_utf16_lossy(&buf[..(n - 1) as usize])
        } else {
            String::new()
        }
    };
    let flags = if seconds { 0 } else { TIME_NOSECONDS as u32 };
    // SAFETY: the buffer is live and its length is passed; `st` is a valid SYSTEMTIME.
    let n = unsafe {
        GetTimeFormatEx(
            PCWSTR::null(),
            flags,
            Some(&st),
            PCWSTR::null(),
            Some(PWSTR(buf.as_mut_ptr())),
            buf.len() as i32,
        )
    };
    let time = text(n, &buf);
    let date = |flags: u32, picture: Option<&str>| {
        let picture = picture.map(to_wide);
        let mut buf = [0u16; 96];
        // SAFETY: as above; `picture` outlives the call.
        let n = unsafe {
            GetDateFormatEx(
                PCWSTR::null(),
                flags,
                Some(&st),
                picture
                    .as_ref()
                    .map_or(PCWSTR::null(), |p| PCWSTR(p.as_ptr())),
                Some(PWSTR(buf.as_mut_ptr())),
                buf.len() as i32,
                PCWSTR::null(),
            )
        };
        text(n, &buf)
    };
    let long = date(DATE_LONGDATE, None);
    let weekday = date(0, Some("dddd"));
    // The region's own picture of a month and day ("M月d日", "MMMM d"), with the short month
    // name (GetDateFormatEx refuses its DATE_MONTHDAY flag).
    let month_day = {
        windows_core::link!("kernel32.dll" "system" fn GetLocaleInfoEx(locale: PCWSTR, lctype: u32, data: PWSTR, len: i32) -> i32);
        const LOCALE_SMONTHDAY: u32 = 0x78;
        let mut picture = [0u16; 80];
        // SAFETY: the buffer is live and its length is passed.
        let n = unsafe {
            GetLocaleInfoEx(
                PCWSTR::null(),
                LOCALE_SMONTHDAY,
                PWSTR(picture.as_mut_ptr()),
                picture.len() as i32,
            )
        };
        let picture = text(n, &picture).replace("MMMM", "MMM");
        if picture.is_empty() {
            String::new()
        } else {
            date(0, Some(&picture))
        }
    };
    let short_day = date(0, Some("ddd"));
    let short = match long.strip_prefix(weekday.as_str()) {
        // The long date starts with the weekday: the short one too, with what follows it
        // there (", " in English, a space in French).
        Some(rest) if !weekday.is_empty() && !month_day.is_empty() => {
            let sep: String = rest.chars().take_while(|c| !c.is_alphanumeric()).collect();
            format!("{short_day}{sep}{month_day}")
        }
        _ => format!("{month_day} {short_day}").trim().to_string(),
    };
    let line = if weekday.is_empty() || long.contains(&weekday) {
        long
    } else {
        format!("{long} {weekday}")
    };
    (time, line, short)
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
