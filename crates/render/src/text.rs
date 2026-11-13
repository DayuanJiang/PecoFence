//! Label fitting: Explorer-style two-line labels with an ellipsis, and the shaped labels kept
//! between frames.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use windows_canvas::{TextFormat, TextLayout, Vector2};
use windows_core::Interface;

/// Layouts kept for all fences together; past this many, the half used longest ago goes.
const MAX_LAYOUTS: usize = 512;

/// Shaped captions kept across frames. A scroll or hover frame redraws every label in view;
/// laying each one out again (font fallback for Han text included) was 90 % of a scroll frame's
/// work in a fence of 96 files, and drawing a kept layout four more times for the halo cost
/// next to nothing. A full cache holds about 400 KB.
#[derive(Default)]
pub struct TextLayouts {
    map: HashMap<u64, Shaped>,
    /// Counts the lookups; a layout remembers when it was last used.
    clock: u64,
    /// Row columns shortened to their width (all dropped when there are too many).
    fitted: HashMap<u64, Fitted>,
}

struct Fitted {
    text: String,
    format: TextFormat,
    /// Column width, as bits.
    width: u32,
    fitted: String,
}

struct Shaped {
    text: String,
    /// Holds the format, so its address stays its name while the layout lives.
    format: TextFormat,
    /// Layout box width and height, as bits.
    size: [u32; 2],
    layout: TextLayout,
    used: u64,
}

impl TextLayouts {
    /// `text` laid out with `format` in a `width` x `height` DIP box: the one made for an
    /// earlier frame when there is one.
    pub fn get(
        &mut self,
        text: &str,
        format: &TextFormat,
        width: f32,
        height: f32,
    ) -> windows_core::Result<TextLayout> {
        let size = [width.to_bits(), height.to_bits()];
        let id = format.raw().as_raw() as usize;
        let mut hasher = DefaultHasher::new();
        (text, id, size).hash(&mut hasher);
        let key = hasher.finish();
        self.clock += 1;
        if let Some(s) = self.map.get_mut(&key)
            && s.text == text
            && s.format.raw().as_raw() as usize == id
            && s.size == size
        {
            s.used = self.clock;
            return Ok(s.layout.clone());
        }
        let layout = TextLayout::new(text, format, width, height)?;
        if self.map.len() >= MAX_LAYOUTS {
            let mut used: Vec<u64> = self.map.values().map(|s| s.used).collect();
            used.sort_unstable();
            let keep_from = used[used.len() / 2];
            self.map.retain(|_, s| s.used >= keep_from);
        }
        self.map.insert(
            key,
            Shaped {
                text: text.to_string(),
                format: format.clone(),
                size,
                layout: layout.clone(),
                used: self.clock,
            },
        );
        Ok(layout)
    }

    /// Drops every layout and shortened text (another label font).
    pub fn clear(&mut self) {
        self.map.clear();
        self.fitted.clear();
    }

    /// `text` shortened to `max_width` DIPs with an ellipsis ([`fit_width`]), worked out once
    /// per column width: a row's date, type and size are fitted again on every frame.
    pub fn fit_width(&mut self, text: &str, format: &TextFormat, max_width: f32) -> String {
        let width = max_width.to_bits();
        let id = format.raw().as_raw() as usize;
        let mut hasher = DefaultHasher::new();
        (text, id, width).hash(&mut hasher);
        let key = hasher.finish();
        if let Some(f) = self.fitted.get(&key)
            && f.text == text
            && f.format.raw().as_raw() as usize == id
            && f.width == width
        {
            return f.fitted.clone();
        }
        let fitted = fit_width(text, format, max_width);
        if self.fitted.len() >= MAX_LAYOUTS {
            self.fitted.clear();
        }
        self.fitted.insert(
            key,
            Fitted {
                text: text.to_string(),
                format: format.clone(),
                width,
                fitted: fitted.clone(),
            },
        );
        fitted
    }

    /// Layouts held (tests).
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Shortens `text` until it fits in `max_lines` lines of `width` DIPs with `format`.
pub fn fit_lines(text: &str, format: &TextFormat, width: f32, max_lines: u32) -> String {
    fit_lines_counted(text, format, width, max_lines).0
}

/// [`fit_lines`] plus the number of lines the returned text occupies at `width` (measured
/// here, so the caller does not need a second layout to size the label box).
pub fn fit_lines_counted(
    text: &str,
    format: &TextFormat,
    width: f32,
    max_lines: u32,
) -> (String, u32) {
    // Line count of `s` when it fits within `max_lines`, None when it does not (or 1 when
    // DirectWrite refuses the layout: the text is drawn as-is then).
    let lines = |s: &str| -> Option<u32> {
        match TextLayout::new(s, format, width, 10_000.0) {
            Ok(l) => Some(l.metrics().line_count).filter(|n| *n <= max_lines),
            Err(_) => Some(1),
        }
    };
    if let Some(n) = lines(text) {
        return (text.to_string(), n.max(1));
    }
    let chars: Vec<char> = text.chars().collect();
    // The longest prefix that fits with an ellipsis, by halving. No guess here: in wrapped
    // lines one more character can make a name fit that did not (「v1.2.3-beta…」 needs four
    // lines of 40 DIPs, 「v1.2.3-beta.…」 three, since the dot lets the line break), so a
    // search from a guess can stop at another cut than the halving does.
    let lo = longest_fitting(chars.len(), None, |k| {
        lines(&(chars[..k].iter().collect::<String>() + "…")).is_some()
    });
    let fitted = chars[..lo]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string()
        + "…";
    // Trimming the trailing whitespace can only shorten the wrap; measure the final text once.
    let n = lines(&fitted).unwrap_or(max_lines).max(1);
    (fitted, n)
}

/// [`fit_lines`] for a name that may take a wider box when that shows more of it: the fitted
/// text and whether it is for `wide` DIPs. The wider box is taken only when the whole name fits
/// it and the usual `width` either cuts the name or needs more lines (「README.md」 on one line
/// in place of 「README.m / d」, twelve CJK glyphs whole in place of ten and an ellipsis). A
/// name that fills its lines either way keeps the usual width and so the room beside it.
pub fn fit_lines_wider(
    text: &str,
    format: &TextFormat,
    width: f32,
    wide: f32,
    max_lines: u32,
) -> (String, bool) {
    let (usual, lines) = fit_lines_counted(text, format, width, max_lines);
    if wide <= width || (usual == text && lines <= 1) {
        return (usual, false);
    }
    let (wider, wide_lines) = fit_lines_counted(text, format, wide, max_lines);
    if wider == text && (usual != text || wide_lines < lines) {
        (wider, true)
    } else {
        (usual, false)
    }
}

/// Single-line fit: the longest prefix of `text` (plus an ellipsis) whose width is at most
/// `max_width` DIPs in `format`. Returns `text` unchanged when it fits.
pub fn fit_width(text: &str, format: &TextFormat, max_width: f32) -> String {
    let fits = |s: &str| -> bool {
        TextLayout::new(s, format, 10_000.0, 10_000.0)
            .map(|l| l.metrics().width_including_trailing_whitespace <= max_width)
            .unwrap_or(true)
    };
    // The whole text, laid out once: whether it fits and, when not, about where the cut is.
    let whole = TextLayout::new(text, format, 10_000.0, 10_000.0);
    let fits_whole = whole
        .as_ref()
        .map(|l| l.metrics().width_including_trailing_whitespace <= max_width)
        .unwrap_or(true);
    if fits_whole {
        return text.to_string();
    }
    // No room for the ellipsis itself: nothing to show.
    let ellipsis = measure_width("…", format);
    if ellipsis > max_width {
        return String::new();
    }
    let chars: Vec<char> = text.chars().collect();
    // The character under the point where the ellipsis has to begin.
    let guess = whole
        .as_ref()
        .ok()
        .filter(|_| stand_alone(&chars))
        .map(|l| {
            let m = l.metrics();
            let at = Vector2::new(
                m.left + (max_width - ellipsis).max(0.0),
                m.top + m.height / 2.0,
            );
            chars_before(&chars, l.hit_test_point(at).text_position as usize)
        });
    let lo = longest_fitting(chars.len(), guess, |k| {
        fits(&(chars[..k].iter().collect::<String>() + "…"))
    });
    chars[..lo]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string()
        + "…"
}

/// How many of `chars` lie wholly before UTF-16 offset `units` of their text.
fn chars_before(chars: &[char], units: usize) -> usize {
    let mut end = 0;
    chars
        .iter()
        .take_while(|c| {
            end += c.len_utf16();
            end <= units
        })
        .count()
}

/// Whether every character of a name stands for itself on its line: Latin, Greek and
/// Cyrillic letters, digits, punctuation, Chinese characters, kana, Hangul syllables. A
/// longer piece of such a name is never narrower than a shorter one, which the search from a
/// guess relies on. Emoji joined into one picture, flags, combining accents and the scripts
/// whose letters change shape beside their neighbours (Arabic, Thai, the Indic ones) can
/// get narrower with one more character: their names keep the halving, whose answer for
/// them cannot be found another way.
fn stand_alone(chars: &[char]) -> bool {
    chars.iter().all(|c| {
        matches!(
            *c as u32,
            0x20..=0x7E
                | 0xA0..=0x24F
                | 0x370..=0x3FF
                | 0x400..=0x482
                | 0x48A..=0x52F
                | 0x2010..=0x2027
                | 0x2030..=0x205E
                | 0x3000..=0x303F
                | 0x3041..=0x3096
                | 0x309B..=0x30FF
                | 0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xAC00..=0xD7A3
                | 0xFF01..=0xFF60
                | 0xFFE0..=0xFFE6
        )
    })
}

/// The largest `k` in `1..=len` for which `fits(k)` holds, 0 when none does. Without a
/// guess, by halving `0..=len`, a question per halving: six to eight layouts for a long
/// name, and a screenful of such names in one frame of a fast scroll. With a `guess` of
/// where the answer lies, `fits` is asked there and one beside it, and once more for every
/// step the guess is off: the same answer as the halving wherever `fits` holds up to some
/// length and fails beyond it (`stand_alone`).
fn longest_fitting(len: usize, guess: Option<usize>, fits: impl Fn(usize) -> bool) -> usize {
    let Some(guess) = guess else {
        let (mut lo, mut hi) = (0usize, len);
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            if fits(mid) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        return lo;
    };
    let mut k = guess.min(len);
    if k > 0 && !fits(k) {
        k -= 1;
        while k > 0 && !fits(k) {
            k -= 1;
        }
        return k;
    }
    while k < len && fits(k + 1) {
        k += 1;
    }
    k
}

/// Advance width of a single line of `text` in `format`, in DIPs.
pub fn measure_width(text: &str, format: &TextFormat) -> f32 {
    TextLayout::new(text, format, 10_000.0, 10_000.0)
        .map(|l| l.metrics().width_including_trailing_whitespace)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `fit_lines_counted` as it was: a binary search over the prefix length, a layout per
    /// step. The reference the one-layout fit must agree with, name for name.
    fn fit_lines_by_halving(
        text: &str,
        format: &TextFormat,
        width: f32,
        max_lines: u32,
    ) -> (String, u32) {
        let lines = |s: &str| -> Option<u32> {
            match TextLayout::new(s, format, width, 10_000.0) {
                Ok(l) => Some(l.metrics().line_count).filter(|n| *n <= max_lines),
                Err(_) => Some(1),
            }
        };
        if let Some(n) = lines(text) {
            return (text.to_string(), n.max(1));
        }
        let chars: Vec<char> = text.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let candidate: String = chars[..mid].iter().collect::<String>() + "…";
            if lines(&candidate).is_some() {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let fitted = chars[..lo]
            .iter()
            .collect::<String>()
            .trim_end()
            .to_string()
            + "…";
        let n = lines(&fitted).unwrap_or(max_lines).max(1);
        (fitted, n)
    }

    /// `fit_width` as it was.
    fn fit_width_by_halving(text: &str, format: &TextFormat, max_width: f32) -> String {
        let fits = |s: &str| -> bool {
            TextLayout::new(s, format, 10_000.0, 10_000.0)
                .map(|l| l.metrics().width_including_trailing_whitespace <= max_width)
                .unwrap_or(true)
        };
        if fits(text) {
            return text.to_string();
        }
        if !fits("…") {
            return String::new();
        }
        let chars: Vec<char> = text.chars().collect();
        let (mut lo, mut hi) = (0usize, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let candidate: String = chars[..mid].iter().collect::<String>() + "…";
            if fits(&candidate) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        chars[..lo]
            .iter()
            .collect::<String>()
            .trim_end()
            .to_string()
            + "…"
    }

    /// File names of every kind a folder holds: short and long, Chinese, Japanese, Korean,
    /// Latin with accents, Cyrillic, Arabic, Thai, emoji (pairs and joined ones), combining
    /// marks, digits, dots and dashes, with and without an extension, spaces at odd places.
    fn names() -> Vec<String> {
        let mut stems: Vec<String> = [
            "报告",
            "会议记录",
            "年度预算表",
            "设计稿 第二版",
            "合同扫描件",
            "周报",
            "项目计划书 最终版 已确认 可以发给客户的那一份",
            "很长很长的文件名甲乙丙丁戊己庚辛壬癸子丑寅卯",
            "Quarterly report",
            "budget",
            "Meeting notes from the long planning session in March",
            "IMG",
            "release notes draft",
            "README",
            "a",
            "Wwwwwwwwwwwwwwwwwwwwwwwwwwwwwwww",
            "iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii",
            "The-quick-brown-fox-jumps-over-the-lazy-dog",
            "snake_case_file_name_with_many_parts_and_no_spaces_at_all",
            "CamelCaseFileNameWithoutAnySpacesInItAtAll",
            "a b c d e f g h i j k l m n o p q r s t u v w x y z",
            "two  spaces   and    more     between",
            "trailing space before the cut      and after",
            "プロジェクト計画書 最終版",
            "会議の議事録（第３回）改訂版",
            "한국어 파일 이름이 길어지는 경우의 예시",
            "Résumé façade naïve coöperate Ångström",
            "Отчёт за третий квартал года",
            "تقرير الربع الثالث من السنة",
            "รายงานประจำไตรมาสที่สามของปี",
            "photo 😀😃😄😁 party 🎉🎉🎉 2026",
            "family 👨‍👩‍👧‍👦 trip 🇯🇵🇫🇷🇩🇪 photos",
            "e\u{301}te\u{301} a\u{300} la plage cafe\u{301}",
            "2026-11-12 14.30.05",
            "v1.2.3-beta.4+build.5678",
            "发票 0001 报销单 （已盖章）",
            "新建 文本文档 (2)",
            "混合 Mixed 名字 name 与 and 数字 12345 以及 symbols #@&",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        stems.push("x".repeat(80));
        stems.push("文".repeat(60));
        let exts = ["", ".txt", ".md", ".docx", ".tar.gz", ".png", ".照片"];
        let mut out = Vec::new();
        for (i, stem) in stems.iter().enumerate() {
            for (j, ext) in exts.iter().enumerate() {
                out.push(format!("{stem}{ext}"));
                out.push(format!("{stem} {:04}{ext}", i * 7 + j));
                // Every prefix length of a few long names: the cut at every place.
                if stem.chars().count() > 20 && j == 1 {
                    let chars: Vec<char> = stem.chars().collect();
                    for n in (3..chars.len()).step_by(2) {
                        out.push(chars[..n].iter().collect::<String>() + ext);
                    }
                }
            }
        }
        out
    }

    /// The fit that lays the name out once agrees with the one that halved, string for
    /// string and width for width, in the list and details columns at many widths and in
    /// the fonts the app draws with. The icons' wrapped labels still halve (see
    /// `fit_lines_counted`); they are compared too, so a later change there is caught.
    #[test]
    fn a_name_laid_out_once_is_cut_where_the_halving_cut_it() {
        let names = names();
        let (mut compared, mut cut) = (0usize, 0usize);
        for (family, size) in [
            ("Segoe UI", 12.0),
            ("Microsoft YaHei UI", 12.0),
            ("Segoe UI", 15.0),
        ] {
            let label = TextFormat::new(family, size)
                .unwrap()
                .with_alignment(windows_canvas::TextAlignment::Center)
                .with_word_wrapping(windows_canvas::WordWrapping::Wrap);
            let row = TextFormat::new(family, size)
                .unwrap()
                .with_word_wrapping(windows_canvas::WordWrapping::NoWrap);
            for name in &names {
                for width in [40.0, 52.0, 64.0, 74.0, 96.0, 120.0] {
                    for max_lines in [1, 2, 3] {
                        let want = fit_lines_by_halving(name, &label, width, max_lines);
                        let got = fit_lines_counted(name, &label, width, max_lines);
                        assert_eq!(
                            got, want,
                            "{name:?} in {max_lines} lines of {width} DIPs, {family} {size}"
                        );
                        compared += 1;
                        cut += (got.0 != *name) as usize;
                    }
                }
                for width in [20.0, 37.0, 60.0, 88.5, 120.0, 180.0, 260.0, 400.0] {
                    let want = fit_width_by_halving(name, &row, width);
                    let got = fit_width(name, &row, width);
                    assert_eq!(
                        got, want,
                        "{name:?} in a column of {width} DIPs, {family} {size}"
                    );
                    assert_eq!(measure_width(&got, &row), measure_width(&want, &row));
                    compared += 1;
                    cut += (got != *name) as usize;
                }
            }
        }
        println!("{compared} fits compared, {cut} of them cut");
        assert!(
            compared > 5000 && cut > 2000,
            "{compared} compared, {cut} cut"
        );
    }

    /// Which names take the search from a guess: the ones whose characters each stand alone.
    #[test]
    fn names_whose_characters_join_keep_the_halving() {
        let alone = |s: &str| stand_alone(&s.chars().collect::<Vec<_>>());
        for name in [
            "Quarterly report 0012.docx",
            "项目计划书 最终版（已确认）.pdf",
            "プロジェクト計画書 最終版",
            "한국어 파일 이름",
            "Résumé façade Ångström",
            "Отчёт за третий квартал",
            "“quoted” – dashed… name",
        ] {
            assert!(alone(name), "{name}");
        }
        for name in [
            "family 👨\u{200d}👩\u{200d}👧 trip",
            "flags 🇯🇵🇫🇷",
            "photo 😀 party",
            "cafe\u{301}",
            "تقرير الربع",
            "รายงานประจำ",
            "ガ as カ\u{3099}",
        ] {
            assert!(!alone(name), "{name}");
        }
    }

    /// The search from a guess finds what the halving finds, wherever the guess lies.
    #[test]
    fn the_search_from_a_guess_finds_the_longest_that_fits() {
        for len in 0..40usize {
            for limit in 0..=len {
                let halving = {
                    let (mut lo, mut hi) = (0usize, len);
                    while lo < hi {
                        let mid = (lo + hi).div_ceil(2);
                        if mid <= limit {
                            lo = mid;
                        } else {
                            hi = mid - 1;
                        }
                    }
                    lo
                };
                assert_eq!(longest_fitting(len, None, |k| k <= limit), halving);
                for guess in 0..=len + 2 {
                    let asked = std::cell::Cell::new(0);
                    let found = longest_fitting(len, Some(guess), |k| {
                        asked.set(asked.get() + 1);
                        k <= limit
                    });
                    assert_eq!(found, halving, "len {len}, limit {limit}, guess {guess}");
                    // Two questions when the guess is right, one more per step it is off.
                    let off = guess.min(len).abs_diff(limit);
                    assert!(
                        asked.get() <= off + 2,
                        "{} questions, {off} off",
                        asked.get()
                    );
                }
            }
        }
    }

    /// The wider box is for the names it helps: one line in place of two, the whole name in
    /// place of a cut one. Names that fill their lines either way keep the usual width.
    #[test]
    fn a_name_takes_the_wider_box_only_when_it_shows_more() {
        let format = TextFormat::new("Segoe UI", 12.0)
            .unwrap()
            .with_word_wrapping(windows_canvas::WordWrapping::Wrap);
        let fit = |name: &str| fit_lines_wider(name, &format, 64.0, 74.0, 2);
        // Two lines at 64 DIPs, one at 74.
        assert_eq!(fit("README.md"), ("README.md".to_string(), true));
        assert_eq!(fit("项目计划书稿"), ("项目计划书稿".to_string(), true));
        // One line already: nothing to gain.
        assert_eq!(fit("报告"), ("报告".to_string(), false));
        // Two whole lines either way (5 + 3 or 6 + 2 glyphs): the usual width.
        assert_eq!(
            fit("项目计划书稿修订"),
            ("项目计划书稿修订".to_string(), false)
        );
        // Cut at 64 DIPs (ten glyphs in two lines), whole at 74 (twelve).
        assert_eq!(
            fit("很长的文件名甲乙丙丁戊己"),
            ("很长的文件名甲乙丙丁戊己".to_string(), true)
        );
        // Cut either way: the usual width, with its ellipsis.
        let (long, wide) = fit("很长很长的文件名甲乙丙丁戊己庚辛");
        assert!(!wide && long.ends_with('…'), "{long}");
        // No wider box on offer (a fence that scrolls).
        assert!(!fit_lines_wider("README.md", &format, 64.0, 64.0, 2).1);
    }

    #[test]
    fn a_label_is_shaped_once_per_box_size() {
        let format = TextFormat::new("Segoe UI", 12.0).unwrap();
        let mut layouts = TextLayouts::default();
        let a = layouts.get("会议记录31", &format, 80.0, 32.0).unwrap();
        let b = layouts.get("会议记录31", &format, 80.0, 32.0).unwrap();
        assert_eq!(
            a.raw().as_raw(),
            b.raw().as_raw(),
            "the next frame reuses the layout"
        );
        let wider = layouts.get("会议记录31", &format, 90.0, 32.0).unwrap();
        assert_ne!(
            a.raw().as_raw(),
            wider.raw().as_raw(),
            "another box is shaped anew"
        );
        let other = TextFormat::new("Segoe UI", 14.0).unwrap();
        let larger = layouts.get("会议记录31", &other, 80.0, 32.0).unwrap();
        assert_ne!(
            a.raw().as_raw(),
            larger.raw().as_raw(),
            "another format too"
        );
        assert_eq!(layouts.len(), 3);
    }

    #[test]
    fn the_layouts_used_longest_ago_go_first() {
        let format = TextFormat::new("Segoe UI", 12.0).unwrap();
        let mut layouts = TextLayouts::default();
        for i in 0..MAX_LAYOUTS + 1 {
            layouts
                .get(&format!("label {i}"), &format, 80.0, 32.0)
                .unwrap();
        }
        assert_eq!(layouts.len(), MAX_LAYOUTS / 2 + 1);
        let last = format!("label {MAX_LAYOUTS}");
        let kept = layouts.get(&last, &format, 80.0, 32.0).unwrap();
        let again = layouts.get(&last, &format, 80.0, 32.0).unwrap();
        assert_eq!(kept.raw().as_raw(), again.raw().as_raw());
        layouts.clear();
        assert!(layouts.is_empty());
    }

    #[test]
    fn a_column_is_shortened_as_fit_width_does() {
        let format = TextFormat::new("Segoe UI", 12.0).unwrap();
        let mut layouts = TextLayouts::default();
        let date = "2026/11/08 10:38";
        for width in [30.0, 60.0, 200.0] {
            let want = fit_width(date, &format, width);
            assert_eq!(layouts.fit_width(date, &format, width), want);
            assert_eq!(
                layouts.fit_width(date, &format, width),
                want,
                "again from the cache"
            );
        }
        assert!(layouts.fit_width(date, &format, 30.0).ends_with('…'));
        assert_eq!(layouts.fit_width(date, &format, 200.0), date);
    }
}
