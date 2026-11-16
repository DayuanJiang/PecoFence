//! 写入用户选的文件夹 (plan v6 row 7, change 5): which paths a plugin with
//! `folders: "readwrite"` may write, make a folder at or move to the Recycle Bin. Text only:
//! the app resolves links and passes the real path, so the rules run and are tested anywhere.

/// The kinds of files a plugin may write into a folder the user picked: data, never something
/// Windows would run or open as a program.
pub const DATA_TYPES: [&str; 10] = [
    "txt", "md", "json", "csv", "log", "ics", "png", "jpg", "gif", "webp",
];

/// Whether file name `name` ends in one of [`DATA_TYPES`] (any letter case).
pub fn is_data_file(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && DATA_TYPES.contains(&ext.to_ascii_lowercase().as_str())
    })
}

/// A Windows path's root (`c:` or `\\server\share`, lowercase) and its parts below it, also of
/// the `\\?\` form `canonicalize` gives. None for a path without a drive or share (a relative
/// path, `\\.\` devices).
pub fn split(path: &str) -> Option<(String, Vec<String>)> {
    let path = path.replace('/', "\\");
    let (unc, rest) = if let Some(r) = path.strip_prefix(r"\\?\UNC\") {
        (true, r)
    } else if let Some(r) = path.strip_prefix(r"\\?\") {
        (false, r)
    } else if let Some(r) = path.strip_prefix(r"\\") {
        (true, r)
    } else {
        (false, path.as_str())
    };
    let mut parts = rest.split('\\').filter(|p| !p.is_empty()).map(String::from);
    let root = if unc {
        let (server, share) = (parts.next()?, parts.next()?);
        if server == "." || server == "?" {
            return None;
        }
        format!(r"\\{server}\{share}")
    } else {
        let drive = parts.next()?;
        let b = drive.as_bytes();
        if b.len() != 2 || !b[0].is_ascii_alphabetic() || b[1] != b':' || !rest.starts_with(&drive)
        {
            return None;
        }
        drive
    };
    Some((root.to_lowercase(), parts.collect()))
}

/// Windows' device names: a file of that name (with any extension) opens the device. COM and
/// LPT take a digit 0 to 9 or a superscript one, two or three.
fn device(part: &str) -> bool {
    let stem = part.split('.').next().unwrap_or_default().trim_end();
    let stem = stem.to_ascii_lowercase();
    let numbered = |p: &str| {
        let mut n = stem.strip_prefix(p).unwrap_or_default().chars();
        let digit = n
            .next()
            .is_some_and(|c| c.is_ascii_digit() || "¹²³".contains(c));
        digit && n.next().is_none()
    };
    ["con", "prn", "aux", "nul", "conin$", "conout$"].contains(&stem.as_str())
        || numbered("com")
        || numbered("lpt")
}

/// Why the path text `path` names nothing a plugin may write: no drive or share, a part starting
/// with a dot (`.git`, `.ssh`, `..`), a part ending in a dot or a space (Windows drops them), a
/// colon past the drive (an alternate data stream) or another sign Windows refuses, a device name.
pub fn name_refusal(path: &str) -> Option<&'static str> {
    let Some((_, parts)) = split(path) else {
        return Some("not a full path with a drive");
    };
    for part in &parts {
        if part.starts_with('.') {
            return Some("a part of the path starts with a dot");
        }
        if part.ends_with(['.', ' ']) {
            return Some("a part of the path ends in a dot or a space");
        }
        if part.contains([':', '<', '>', '"', '|', '?', '*']) || part.chars().any(char::is_control)
        {
            return Some("a colon or another sign Windows does not allow in a name");
        }
        if device(part) {
            return Some("a part of the path is a device name (con, nul, com1, ...)");
        }
    }
    None
}

/// The folders no plugin writes in, as real paths, each with what it is for the log: nothing at
/// or below `below` (Windows, the program folders, ProgramData, AppData, PecoFence's own
/// folders), and neither `itself` (the home folder, the folder of homes) nor anything directly
/// in it.
#[derive(Debug, Default, Clone)]
pub struct Forbidden {
    pub below: Vec<(String, &'static str)>,
    pub itself: Vec<(String, &'static str)>,
}

/// Why the real path `real` (links resolved; for a file or folder that does not exist yet, its
/// parent's real path and its name) may not be written, made or moved to the Recycle Bin: the
/// rules of [`name_refusal`], a drive root or a file directly in one, a system folder of a
/// drive's root (`$Recycle.Bin`, `System Volume Information`), or a folder of `forbidden`.
pub fn place_refusal(real: &str, forbidden: &Forbidden) -> Option<String> {
    if let Some(why) = name_refusal(real) {
        return Some(why.into());
    }
    let (root, parts) = split(real)?;
    let lower: Vec<String> = parts.iter().map(|p| p.to_lowercase()).collect();
    // The first `n` parts of `real` are folder `f`.
    let is = |f: &str, n: usize| {
        split(f).is_some_and(|(r, fp)| {
            r == root && fp.len() == n && fp.iter().zip(&lower).all(|(a, b)| a.to_lowercase() == *b)
        })
    };
    if let Some((_, what)) = forbidden.below.iter().find(|(f, _)| {
        let n = split(f).map_or(usize::MAX, |(_, p)| p.len());
        n <= lower.len() && is(f, n)
    }) {
        return Some(format!("in {what}"));
    }
    if lower
        .first()
        .is_some_and(|p| p.starts_with('$') || p == "system volume information")
    {
        return Some("a system folder of the drive".into());
    }
    let n = lower.len();
    if n <= 1 {
        return Some("a drive root or directly in one".into());
    }
    if let Some((_, what)) = (forbidden.itself.iter()).find(|(f, _)| is(f, n) || is(f, n - 1)) {
        return Some(format!("{what} or directly in it"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forbidden() -> Forbidden {
        Forbidden {
            below: vec![
                (r"C:\Windows".into(), "Windows"),
                (r"\\?\C:\Program Files".into(), "a program folder"),
                (r"C:\ProgramData".into(), "ProgramData"),
                (r"C:\Users\me\AppData".into(), "AppData"),
                (r"D:\PecoFence".into(), "PecoFence's own folder"),
            ],
            itself: vec![
                (r"C:\Users\me".into(), "the home folder"),
                (r"C:\Users".into(), "the folder of homes"),
            ],
        }
    }

    #[test]
    fn paths_split_into_a_root_and_parts() {
        let s = |p: &str| split(p).map(|(r, parts)| (r, parts.join("|")));
        assert_eq!(s(r"C:\a\b.txt"), Some(("c:".into(), "a|b.txt".into())));
        assert_eq!(s(r"\\?\C:\a\\b"), Some(("c:".into(), "a|b".into())));
        assert_eq!(s("c:/a/b"), Some(("c:".into(), "a|b".into())));
        assert_eq!(
            s(r"\\?\UNC\Nas\Share\x"),
            Some((r"\\nas\share".into(), "x".into()))
        );
        assert_eq!(
            s(r"\\nas\share"),
            Some((r"\\nas\share".into(), String::new()))
        );
        for bad in [
            r"a\b",
            r"\\.\C:\a",
            r"\\?\GLOBALROOT\x",
            r"C:a",
            r"\a\b",
            r"\\nas",
            "1:\\a",
        ] {
            assert_eq!(s(bad), None, "{bad}");
        }
    }

    #[test]
    fn only_data_files_are_written() {
        for good in [
            "a.txt",
            "Notes.MD",
            "x.y.json",
            "2026-11-16.log",
            "a.webp",
            "b.ics",
        ] {
            assert!(is_data_file(good), "{good}");
        }
        for bad in [
            "a.exe",
            "a.hta",
            "a.lnk",
            "a.txt.exe",
            "a.bat",
            "txt",
            ".txt",
            "a.",
            "a.js",
        ] {
            assert!(!is_data_file(bad), "{bad}");
        }
    }

    #[test]
    fn names_with_streams_dots_and_devices_are_refused_by_their_text() {
        assert_eq!(name_refusal(r"C:\Users\me\Documents\Notes\a.md"), None);
        assert_eq!(name_refusal(r"\\nas\share\log\b.txt"), None);
        for (bad, why) in [
            (r"C:\Notes\a.txt:hidden", "colon"),
            (r"C:\Notes\a.txt::$DATA", "colon"),
            (r"C:\Notes\a.txt.", "ends in a dot"),
            (r"C:\Notes\a.txt ", "ends in a dot or a space"),
            (r"C:\Notes \a.txt", "ends in a dot or a space"),
            (r"C:\Notes\nul.txt", "device"),
            (r"C:\Notes\COM1", "device"),
            (r"C:\Notes\lpt9.md", "device"),
            (r"C:\Notes\COM0.txt", "device"),
            (r"C:\Notes\lpt0", "device"),
            ("C:\\Notes\\com\u{b9}.txt", "device"),
            ("C:\\Notes\\LPT\u{b3}", "device"),
            (r"C:\Notes\.git\config.txt", "dot"),
            (r"C:\Notes\..\x.txt", "dot"),
            (r"C:\Notes\a?.txt", "sign"),
            ("C:\\Notes\\a\u{1}.txt", "sign"),
            (r"Notes\a.txt", "drive"),
        ] {
            let got = name_refusal(bad).unwrap_or_default();
            assert!(got.contains(why), "{bad}: {got}");
        }
        // Names that only look like devices are files.
        assert_eq!(name_refusal(r"C:\Notes\console.txt"), None);
        assert_eq!(name_refusal(r"C:\Notes\com10.txt"), None);
        assert_eq!(name_refusal("C:\\Notes\\com\u{b9}0.txt"), None);
        assert_eq!(name_refusal("C:\\Notes\\com\u{b2}x"), None);
    }

    #[test]
    fn system_folders_roots_the_home_and_pecofence_s_own_are_refused_by_their_real_path() {
        let f = forbidden();
        let why = |p: &str| place_refusal(p, &f);
        for good in [
            r"\\?\C:\Users\me\Documents\Notes\a.md",
            r"C:\Users\me\Documents\new folder",
            r"D:\Notes\x.txt",
            r"D:\PecoFence notes\x.txt",
            r"C:\Users\Public\Documents\x.txt",
            r"\\?\UNC\nas\share\notes\x.txt",
        ] {
            assert_eq!(why(good), None, "{good}");
        }
        for (bad, what) in [
            (r"\\?\C:\Windows\System32\x.txt", "in Windows"),
            (r"C:\windows", "in Windows"),
            (r"C:\Program Files\App\x.txt", "in a program folder"),
            (r"C:\ProgramData\x\y.json", "in ProgramData"),
            (r"C:\Users\ME\AppData\Roaming\x.txt", "in AppData"),
            (r"D:\PecoFence\config\plugins-data\x.txt", "PecoFence's own"),
            (r"C:\Users\me\x.txt", "the home folder or directly in it"),
            (
                r"C:\Users\me\Documents",
                "the home folder or directly in it",
            ),
            (r"C:\Users\me", "the home folder"),
            (r"C:\Users\x.txt", "the folder of homes"),
            (r"C:\x.txt", "drive root"),
            (r"D:\", "drive root"),
            (r"\\nas\share\x.txt", "drive root"),
            (r"C:\$Recycle.Bin\x\y.txt", "system folder of the drive"),
            (r"E:\System Volume Information\x\y.txt", "system folder"),
            (r"C:\Users\me\Documents\.ssh\id.txt", "dot"),
            (r"C:\Users\me\Documents\a.txt:s", "colon"),
        ] {
            let got = why(bad).unwrap_or_default();
            assert!(got.contains(what), "{bad}: {got:?}");
        }
    }
}
