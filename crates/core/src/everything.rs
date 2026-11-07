//! Bytes of Everything's IPC (the voidtools file search tool): the version 2 query PecoFence
//! sends with WM_COPYDATA to the window of class "EVERYTHING_TASKBAR_NOTIFICATION", and the
//! result list Everything sends back with WM_COPYDATA to `reply_hwnd`. Layouts follow the
//! Everything SDK header (`#pragma pack(1)`, little-endian `u32` fields); the window messages
//! themselves are the app's job.

/// `dwData` of the WM_COPYDATA that carries a [`query2w`] buffer.
pub const COPYDATA_QUERY2W: usize = 18;

/// Sort orders for [`query2w`]. Name ascending is always fast; the others are fast only when
/// Everything indexes that property (Tools > Options > Indexes).
pub const SORT_NAME_ASCENDING: u32 = 1;
pub const SORT_DATE_MODIFIED_DESCENDING: u32 = 14;
pub const SORT_RUN_COUNT_DESCENDING: u32 = 20;
pub const SORT_DATE_RUN_DESCENDING: u32 = 26;

const REQUEST_NAME: u32 = 0x1;
const REQUEST_PATH: u32 = 0x2;
const ITEM_FOLDER: u32 = 0x1;
const ITEM_DRIVE: u32 = 0x2;
/// EVERYTHING_IPC_LIST2 header: totitems, numitems, offset, request_flags, sort_type.
const LIST2_HEADER: usize = 20;
/// EVERYTHING_IPC_ITEM2: flags, data_offset.
const ITEM2_SIZE: usize = 8;

/// One result: the full path, and whether it is a folder (a drive root counts as one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub path: String,
    pub is_folder: bool,
}

/// An EVERYTHING_IPC_QUERY2 asking for the name and path of up to `max_results` results of
/// `search`. Everything answers with `dwData = reply_id`.
pub fn query2w(
    reply_hwnd: u32,
    reply_id: u32,
    search: &str,
    max_results: u32,
    sort: u32,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(28 + 2 * (search.len() + 1));
    let fields = [
        reply_hwnd,
        reply_id, // reply_copydata_message
        0,        // search_flags: no case, no whole word, no path, no regex
        0,        // offset
        max_results,
        REQUEST_NAME | REQUEST_PATH,
        sort,
    ];
    for v in fields {
        out.extend_from_slice(&v.to_le_bytes());
    }
    // The search text in UTF-16 with a 0 terminator.
    for unit in search.encode_utf16().chain([0]) {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

/// Reads an EVERYTHING_IPC_LIST2 reply: the number of all matches (more than the hits when
/// `max_results` cut the list) and the hits. None when the buffer is malformed.
pub fn parse_list2(bytes: &[u8]) -> Option<(u32, Vec<Hit>)> {
    let total = read_u32(bytes, 0)?;
    let count = read_u32(bytes, 4)? as usize;
    // The reply lists the fields it carries; name and path are the first two of them.
    let flags = read_u32(bytes, 12)?;
    if flags & (REQUEST_NAME | REQUEST_PATH) != REQUEST_NAME | REQUEST_PATH {
        return None;
    }
    let items_end = count.checked_mul(ITEM2_SIZE)?.checked_add(LIST2_HEADER)?;
    if items_end > bytes.len() {
        return None;
    }
    let mut hits = Vec::with_capacity(count);
    for i in 0..count {
        let item = LIST2_HEADER + i * ITEM2_SIZE;
        let item_flags = read_u32(bytes, item)?;
        let mut at = read_u32(bytes, item + 4)? as usize;
        if at < items_end {
            return None;
        }
        let name = read_text(bytes, &mut at)?;
        let path = read_text(bytes, &mut at)?;
        hits.push(Hit {
            path: full_path(&path, &name, item_flags & ITEM_DRIVE != 0),
            is_folder: item_flags & (ITEM_FOLDER | ITEM_DRIVE) != 0,
        });
    }
    Some((total, hits))
}

/// Everything's search text for what the user typed: the terms (split on whitespace) ANDed.
/// With `pinyin` (Everything 1.5) a term of ASCII letters also matches Chinese names by full
/// pinyin (`py:`) or initials (`pyj:`). A term with an operator character is quoted so it is
/// literal; a double quote inside becomes the `&quot:` entity.
pub fn search_text(query: &str, pinyin: bool) -> String {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|term| {
            if pinyin && term.chars().all(|c| c.is_ascii_alphabetic()) {
                format!("<{term}|py:{term}|pyj:{term}>")
            } else if term.contains(['|', '<', '>', '!', '"']) {
                format!("\"{}\"", term.replace('"', "&quot:"))
            } else {
                term.to_string()
            }
        })
        .collect();
    terms.join(" ")
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let b = bytes.get(at..at.checked_add(4)?)?;
    Some(u32::from_le_bytes(b.try_into().ok()?))
}

/// A text field at `*at`: a `u32` length in UTF-16 units, the text, a 0 terminator. Moves
/// `*at` past it.
fn read_text(bytes: &[u8], at: &mut usize) -> Option<String> {
    let len = read_u32(bytes, *at)? as usize;
    let start = at.checked_add(4)?;
    let end = len.checked_mul(2)?.checked_add(start)?;
    let text = bytes.get(start..end)?;
    if bytes.get(end..end.checked_add(2)?)? != [0, 0] {
        return None;
    }
    *at = end + 2;
    let units: Vec<u16> = text
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| u16::from_le_bytes(c))
        .collect();
    Some(String::from_utf16_lossy(&units))
}

/// `path\name` without doubling the backslash after a root ("C:\"). A drive root itself
/// ("C:") gets its backslash: "C:" alone means the current folder on that drive.
fn full_path(path: &str, name: &str, drive: bool) -> String {
    let mut full = if path.is_empty() {
        name.to_string()
    } else if path.ends_with('\\') {
        format!("{path}{name}")
    } else {
        format!("{path}\\{name}")
    };
    if drive && full.ends_with(':') {
        full.push('\\');
    }
    full
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u32_at(b: &[u8], at: usize) -> u32 {
        u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
    }

    fn push_text(out: &mut Vec<u8>, s: &str) {
        let units: Vec<u16> = s.encode_utf16().collect();
        out.extend_from_slice(&(units.len() as u32).to_le_bytes());
        for u in units.into_iter().chain([0]) {
            out.extend_from_slice(&u.to_le_bytes());
        }
    }

    /// A LIST2 reply with name and path (plus a size after them when `with_size`).
    fn list2(total: u32, items: &[(u32, &str, &str)], with_size: bool) -> Vec<u8> {
        let flags = REQUEST_NAME | REQUEST_PATH | if with_size { 0x10 } else { 0 };
        let mut data = Vec::new();
        let mut offsets = Vec::new();
        let data_start = LIST2_HEADER + items.len() * ITEM2_SIZE;
        for (_, name, path) in items {
            offsets.push((data_start + data.len()) as u32);
            push_text(&mut data, name);
            push_text(&mut data, path);
            if with_size {
                data.extend_from_slice(&1234u64.to_le_bytes());
            }
        }
        let mut out = Vec::new();
        for v in [total, items.len() as u32, 0, flags, SORT_NAME_ASCENDING] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for ((item_flags, _, _), off) in items.iter().zip(offsets) {
            out.extend_from_slice(&item_flags.to_le_bytes());
            out.extend_from_slice(&off.to_le_bytes());
        }
        out.extend_from_slice(&data);
        out
    }

    fn sample() -> Vec<u8> {
        list2(
            1234,
            &[
                (ITEM_FOLDER, "Projects", "C:\\Users\\me"),
                (0, "预算表.xlsx", "D:\\工作"),
                (ITEM_FOLDER | ITEM_DRIVE, "C:", ""),
                (0, "pagefile.sys", "C:\\"),
            ],
            false,
        )
    }

    fn hit(path: &str, is_folder: bool) -> Hit {
        Hit {
            path: path.to_string(),
            is_folder,
        }
    }

    #[test]
    fn query_layout() {
        let b = query2w(0x1234_5678, 7, "预算 a", 50, SORT_DATE_RUN_DESCENDING);
        let fields: Vec<u32> = (0..7).map(|i| u32_at(&b, i * 4)).collect();
        assert_eq!(fields, [0x1234_5678, 7, 0, 0, 50, 0x3, 26]);
        let text: Vec<u16> = b[28..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&c| u16::from_le_bytes(c))
            .collect();
        assert_eq!(text, [0x9884, 0x7b97, 0x20, 0x61, 0]);
        assert_eq!(b.len(), 28 + 5 * 2);
        // An empty search is just the terminator.
        assert_eq!(query2w(1, 2, "", 0, 1).len(), 30);
    }

    #[test]
    fn parse_folder_file_and_roots() {
        let (total, hits) = parse_list2(&sample()).unwrap();
        assert_eq!(total, 1234);
        assert_eq!(
            hits,
            [
                hit("C:\\Users\\me\\Projects", true),
                hit("D:\\工作\\预算表.xlsx", false),
                hit("C:\\", true),
                hit("C:\\pagefile.sys", false),
            ]
        );
    }

    #[test]
    fn parse_skips_fields_after_name_and_path() {
        let b = list2(
            2,
            &[(0, "a.txt", "C:\\x"), (ITEM_FOLDER, "b", "C:\\x")],
            true,
        );
        let (_, hits) = parse_list2(&b).unwrap();
        assert_eq!(hits, [hit("C:\\x\\a.txt", false), hit("C:\\x\\b", true)]);
    }

    #[test]
    fn parse_empty_list() {
        assert_eq!(parse_list2(&list2(0, &[], false)), Some((0, vec![])));
    }

    #[test]
    fn every_truncation_is_rejected() {
        let b = sample();
        for len in 0..b.len() {
            assert_eq!(parse_list2(&b[..len]), None, "length {len}");
        }
    }

    #[test]
    fn garbage_is_rejected() {
        let good = sample();
        let set = |at: usize, v: u32| {
            let mut b = good.clone();
            b[at..at + 4].copy_from_slice(&v.to_le_bytes());
            b
        };
        // Item count far past the buffer (and one that would overflow).
        assert_eq!(parse_list2(&set(4, 5)), None);
        assert_eq!(parse_list2(&set(4, u32::MAX)), None);
        // Name and path not both in the reply.
        assert_eq!(parse_list2(&set(12, REQUEST_NAME)), None);
        // Data offset past the end, into the header, or at the edge of the buffer.
        assert_eq!(parse_list2(&set(24, u32::MAX)), None);
        assert_eq!(parse_list2(&set(24, 4)), None);
        assert_eq!(parse_list2(&set(24, good.len() as u32 - 2)), None);
        // Text length past the end.
        let first_data = u32_at(&good, 24) as usize;
        assert_eq!(parse_list2(&set(first_data, u32::MAX)), None);
        assert_eq!(parse_list2(&set(first_data, 1000)), None);
        // A missing terminator ("Projects" is 8 units long).
        let mut b = good.clone();
        b[first_data + 4 + 16] = b'x';
        assert_eq!(parse_list2(&b), None);
    }

    #[test]
    fn random_bytes_do_not_panic() {
        let mut seed = 0x2545_f491_u32;
        for len in 0..400 {
            let mut b = vec![0u8; len];
            for byte in &mut b {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                *byte = (seed >> 24) as u8;
            }
            // Small counts and offsets so the parser gets past the header now and then.
            if len >= 20 {
                b[4..8].copy_from_slice(&(seed % 4).to_le_bytes());
                b[12..16].copy_from_slice(&3u32.to_le_bytes());
            }
            let _ = parse_list2(&b);
        }
    }

    #[test]
    fn search_text_plain() {
        assert_eq!(search_text("", false), "");
        assert_eq!(search_text("   ", true), "");
        assert_eq!(search_text("  foo   bar ", false), "foo bar");
        assert_eq!(search_text("ysb", false), "ysb");
        assert_eq!(search_text("预算 ext:xlsx", false), "预算 ext:xlsx");
    }

    #[test]
    fn search_text_pinyin() {
        assert_eq!(search_text("ysb", true), "<ysb|py:ysb|pyj:ysb>");
        assert_eq!(
            search_text("YuSuan 2024", true),
            "<YuSuan|py:YuSuan|pyj:YuSuan> 2024"
        );
        // Only terms made of ASCII letters.
        assert_eq!(search_text("预算 a.txt q3", true), "预算 a.txt q3");
    }

    #[test]
    fn search_text_quotes_operators() {
        assert_eq!(search_text("a|b", false), "\"a|b\"");
        assert_eq!(search_text("!readme", true), "\"!readme\"");
        assert_eq!(search_text("<x> y", false), "\"<x>\" y");
        assert_eq!(search_text("say\"hi\"", false), "\"say&quot:hi&quot:\"");
        assert_eq!(search_text("\"", false), "\"&quot:\"");
    }
}
