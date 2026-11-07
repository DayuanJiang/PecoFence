//! Name matching for the search box: how well a typed query matches an icon or file name, and
//! which characters of the name to highlight.
//!
//! Kinds of match, best first: the exact name (also the name without its extension), a prefix
//! of the name, a substring at a word start, pinyin, a substring anywhere, and finally the
//! query's characters in order with gaps ("sbtf" in "Sysbatt.ttf"). Pinyin ranks above a plain
//! substring as in Listary: "ysb" lists 预算表 before Sysbatt.
//!
//! Pinyin: every Chinese character of a contiguous run of the name takes a prefix of its pinyin
//! syllable, so 预算表 matches "ysb", "yusuanbiao" and "yusb", and the last syllable may be
//! partial ("huiyij" for 会议纪要). Other characters in the run match themselves ("10y" for
//! 10月). Only the most common reading of a character is known (会 is "hui", never "kuai"):
//! the table of other readings would add about 320 KB to the exe.

use std::ops::Range;

use pinyin::ToPinyin;

/// A name that matched a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Higher is better; compare the scores of different names for the same query.
    pub score: u32,
    /// Char-index ranges of the name to highlight (merged, ascending).
    pub ranges: Vec<Range<usize>>,
}

/// Kinds of match, worst first, so a bigger value ranks higher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Fuzzy = 1,
    Substring,
    // Above a plain substring, as in Listary: "ysb" finds 预算表 before Sysbatt.
    Pinyin,
    WordStart,
    Prefix,
    Exact,
}

/// Longest query term the pinyin matcher handles (one bit per query position in a `u64`).
const PINYIN_MAX: usize = 63;

/// Whether a [`Match::score`] is only the loosest kind (the query's letters in order, apart),
/// which a filter leaves out.
pub fn is_loose_score(score: u32) -> bool {
    score >> 24 == Kind::Fuzzy as u32
}

/// Matches `query` against `name`, ignoring case. An empty (or blank) query matches nothing.
///
/// Score: the kind of match decides first; within a kind an earlier match wins, then a shorter
/// name, then a match in the name over one in the extension.
///
/// A query with spaces is split into terms that must all match, in any order; the weakest term
/// decides the score (the minimum of the term scores). The whole query is also tried as one
/// term, so "new folder" is still an exact match for "New folder".
pub fn match_name(name: &str, query: &str) -> Option<Match> {
    let query: Vec<char> = query.trim().chars().map(fold).collect();
    if query.is_empty() {
        return None;
    }
    let name = Name::new(name);
    let whole = match_term(&name, &query);
    let terms: Vec<&[char]> = query
        .split(|c| c.is_whitespace())
        .filter(|t| !t.is_empty())
        .collect();
    if terms.len() == 1 {
        return whole.map(|(score, ranges)| Match { score, ranges });
    }
    let mut score = u32::MAX;
    let mut ranges = Vec::new();
    let mut all = true;
    for term in terms {
        match match_term(&name, term) {
            Some((s, r)) => {
                score = score.min(s);
                ranges.extend(r);
            }
            None => {
                all = false;
                break;
            }
        }
    }
    let split = all.then(|| (score, merge(ranges)));
    let best = match (whole, split) {
        (Some(w), Some(s)) => Some(if w.0 >= s.0 { w } else { s }),
        (w, s) => w.or(s),
    };
    best.map(|(score, ranges)| Match { score, ranges })
}

/// The name as characters, original and folded to lower case (same indices).
struct Name {
    orig: Vec<char>,
    low: Vec<char>,
    /// Length without the extension (the whole length when there is none).
    stem: usize,
}

impl Name {
    fn new(name: &str) -> Self {
        let orig: Vec<char> = name.chars().collect();
        let low = orig.iter().map(|&c| fold(c)).collect();
        // A leading dot (".gitignore") starts no extension.
        let stem = match orig.iter().rposition(|&c| c == '.') {
            Some(i) if i > 0 => i,
            _ => orig.len(),
        };
        Name { orig, low, stem }
    }
}

/// The best match of one term (already folded) in `name`: its score and highlight ranges.
fn match_term(name: &Name, term: &[char]) -> Option<(u32, Vec<Range<usize>>)> {
    let low = &name.low;
    let (n, m) = (low.len(), term.len());
    let found = |kind, ranges: Vec<Range<usize>>| {
        let start = ranges[0].start;
        Some((score(kind, start, n, start < name.stem), ranges))
    };
    let one = |kind, range: Range<usize>| found(kind, vec![range]);
    if low[..] == *term {
        return one(Kind::Exact, 0..n);
    }
    if name.stem < n && low[..name.stem] == *term {
        return one(Kind::Exact, 0..name.stem);
    }
    // The first occurrence at a word start wins; otherwise the first occurrence anywhere.
    let mut first = None;
    for i in 0..(n + 1).saturating_sub(m) {
        if low[i..i + m] == *term {
            if is_word_start(&name.orig, i) {
                let kind = if i == 0 {
                    Kind::Prefix
                } else {
                    Kind::WordStart
                };
                return one(kind, i..i + m);
            }
            first.get_or_insert(i);
        }
    }
    if let Some(run) = pinyin_run(low, term) {
        return one(Kind::Pinyin, run);
    }
    if let Some(i) = first {
        return one(Kind::Substring, i..i + m);
    }
    found(Kind::Fuzzy, in_order(low, term)?)
}

/// Kind, then earlier start, then shorter name, then a start in the base name.
fn score(kind: Kind, start: usize, len: usize, in_base: bool) -> u32 {
    let smaller_is_better = |v: usize| 255 - v.min(255) as u32;
    ((kind as u32) << 24)
        | (smaller_is_better(start) << 16)
        | (smaller_is_better(len) << 8)
        | in_base as u32
}

/// Whether a match starting at char `i` starts a word.
fn is_word_start(orig: &[char], i: usize) -> bool {
    let Some(&prev) = i.checked_sub(1).and_then(|p| orig.get(p)) else {
        return true;
    };
    let cur = orig[i];
    matches!(prev, ' ' | '_' | '-' | '.' | '(' | '[' | '（' | '【')
        || is_cjk(cur)
        || is_cjk(prev)
        || (prev.is_lowercase() && cur.is_uppercase())
        || (prev.is_alphabetic() && cur.is_ascii_digit())
}

/// Chinese, Japanese and Korean characters and their punctuation.
fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x2E80..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF | 0xFF00..=0xFFEF | 0x20000..=0x3FFFF)
}

/// Lower case, one char for one char so indices stay the same.
fn fold(c: char) -> char {
    if c.is_ascii() {
        c.to_ascii_lowercase()
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

/// The pinyin syllable of a Chinese character, without tone ("lü" for 绿). The settings page's
/// search (查找设置) asks for these to match pinyin the same way.
pub fn syllable(c: char) -> Option<&'static str> {
    if (c as u32) < 0x3000 {
        return None;
    }
    c.to_pinyin().map(|p| p.plain())
}

/// The shortest run of name characters that matches `term` as pinyin, starting as early as
/// possible. Each Chinese character takes a non-empty prefix of its syllable; any other
/// character (and a Chinese one typed as itself) takes exactly itself.
fn pinyin_run(low: &[char], term: &[char]) -> Option<Range<usize>> {
    let m = term.len();
    if m > PINYIN_MAX
        || !term.iter().any(char::is_ascii_lowercase)
        || !low.iter().any(|&c| syllable(c).is_some())
    {
        return None;
    }
    for start in 0..low.len() {
        // Bit p set: the run so far can consume exactly term[..p].
        let mut states = 1u64;
        for (i, &c) in low.iter().enumerate().skip(start) {
            let mut next = 0u64;
            let mut bits = states;
            while bits != 0 {
                let p = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                next |= steps(c, &term[p..]) << p;
            }
            if next & (1 << m) != 0 {
                return Some(start..i + 1);
            }
            if next == 0 {
                break;
            }
            states = next;
        }
    }
    None
}

/// Bit k set: name character `c` can consume the first k characters of `rest`.
fn steps(c: char, rest: &[char]) -> u64 {
    let Some(&first) = rest.first() else {
        return 0;
    };
    if c == first {
        return 0b10;
    }
    let Some(syl) = syllable(c) else {
        return 0;
    };
    let mut mask = 0;
    for (k, (s, &q)) in syl.chars().zip(rest).enumerate() {
        let same = s == q || (s == 'ü' && (q == 'v' || q == 'u')) || (s == 'ê' && q == 'e');
        if !same {
            break;
        }
        mask |= 1 << (k + 1);
    }
    mask
}

/// The term's characters in order with gaps, each at its first possible place.
fn in_order(low: &[char], term: &[char]) -> Option<Vec<Range<usize>>> {
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut from = 0;
    for &q in term {
        let at = from + low[from..].iter().position(|&c| c == q)?;
        match ranges.last_mut() {
            Some(r) if r.end == at => r.end += 1,
            _ => ranges.push(at..at + 1),
        }
        from = at + 1;
    }
    Some(ranges)
}

/// Sorts ranges and joins the ones that overlap or touch.
fn merge(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|r| r.start);
    let mut out: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for r in ranges {
        match out.last_mut() {
            Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
            _ => out.push(r),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(name: &str, query: &str) -> Option<Kind> {
        let m = match_name(name, query)?;
        Some(match m.score >> 24 {
            1 => Kind::Fuzzy,
            2 => Kind::Substring,
            3 => Kind::Pinyin,
            4 => Kind::WordStart,
            5 => Kind::Prefix,
            6 => Kind::Exact,
            k => panic!("bad kind {k}"),
        })
    }

    fn ranges(name: &str, query: &str) -> Vec<Range<usize>> {
        match_name(name, query).expect("should match").ranges
    }

    fn score(name: &str, query: &str) -> u32 {
        match_name(name, query).expect("should match").score
    }

    #[test]
    fn empty_query_matches_nothing() {
        assert_eq!(match_name("Report.docx", ""), None);
        assert_eq!(match_name("Report.docx", "   "), None);
        assert_eq!(match_name("", "a"), None);
    }

    #[test]
    fn exact_name_and_name_without_extension() {
        assert_eq!(kind("Report.docx", "report.docx"), Some(Kind::Exact));
        assert_eq!(ranges("Report.docx", "REPORT.DOCX"), vec![0..11]);
        assert_eq!(kind("Report.docx", "report"), Some(Kind::Exact));
        assert_eq!(ranges("Report.docx", "report"), vec![0..6]);
        assert_eq!(kind("预算表.xlsx", "预算表"), Some(Kind::Exact));
        // A leading dot is part of the name, not an extension.
        assert_eq!(kind(".gitignore", "gitignore"), Some(Kind::WordStart));
        assert_eq!(kind(".gitignore", ".gitignore"), Some(Kind::Exact));
    }

    #[test]
    fn query_is_trimmed_and_case_is_ignored() {
        assert_eq!(kind("PecoFence", "  peco "), Some(Kind::Prefix));
        assert_eq!(ranges("PecoFence", "PECO"), vec![0..4]);
        assert_eq!(kind("pecofence", "FENCE"), Some(Kind::Substring));
    }

    #[test]
    fn word_starts() {
        assert_eq!(ranges("My Fence.txt", "fence"), vec![3..8]);
        assert_eq!(kind("My Fence.txt", "fence"), Some(Kind::WordStart));
        assert_eq!(kind("my_fence", "fence"), Some(Kind::WordStart));
        assert_eq!(kind("my-fence", "fence"), Some(Kind::WordStart));
        assert_eq!(kind("my.fence", "fence"), Some(Kind::WordStart));
        assert_eq!(kind("copy (fence)", "fence"), Some(Kind::WordStart));
        assert_eq!(kind("PecoFence", "fence"), Some(Kind::WordStart));
        assert_eq!(ranges("PecoFence", "fence"), vec![4..9]);
        assert_eq!(kind("IMG2024", "2024"), Some(Kind::WordStart));
        // Every Chinese character starts a word, and so does the text after one.
        assert_eq!(kind("2024年预算表", "预算"), Some(Kind::WordStart));
        assert_eq!(ranges("2024年预算表", "预算"), vec![5..7]);
        assert_eq!(kind("预算Report", "report"), Some(Kind::WordStart));
        // Inside a word it is a plain substring.
        assert_eq!(kind("Pecofence", "fence"), Some(Kind::Substring));
        assert_eq!(kind("PecoFence", "ence"), Some(Kind::Substring));
        assert_eq!(ranges("PecoFence", "ence"), vec![5..9]);
    }

    #[test]
    fn a_later_word_start_beats_an_earlier_substring() {
        // "fen" occurs inside "Xfen" first, but at a word start later.
        assert_eq!(kind("Xfen Fence", "fen"), Some(Kind::WordStart));
        assert_eq!(ranges("Xfen Fence", "fen"), vec![5..8]);
    }

    #[test]
    fn extensions() {
        assert_eq!(kind("report.docx", "docx"), Some(Kind::WordStart));
        assert_eq!(ranges("report.docx", "docx"), vec![7..11]);
        // Char indices, not byte indices.
        assert_eq!(ranges("预算表.xlsx", "xlsx"), vec![4..8]);
        assert_eq!(kind("archive.tar.gz", "archive.tar"), Some(Kind::Exact));
    }

    #[test]
    fn pinyin_initials_full_and_mixed() {
        for q in ["ysb", "yusuanbiao", "yusb", "ysbiao", "yusuanb", "YSB"] {
            assert_eq!(kind("预算表.xlsx", q), Some(Kind::Pinyin), "{q}");
            assert_eq!(ranges("预算表.xlsx", q), vec![0..3], "{q}");
        }
        // The last syllable may be partial, and the run may end before the name does.
        assert_eq!(ranges("会议纪要.docx", "huiyij"), vec![0..3]);
        assert_eq!(ranges("会议纪要.docx", "hy"), vec![0..2]);
        // Partial syllables in the middle work too ("su" of suan).
        assert_eq!(ranges("预算表", "yusubiao"), vec![0..3]);
    }

    #[test]
    fn pinyin_needs_backtracking() {
        // 西安: "xi" + "an", not "x" + "ian".
        assert_eq!(ranges("西安旅游", "xian"), vec![0..2]);
        assert_eq!(ranges("先进", "xian"), vec![0..1]);
    }

    #[test]
    fn pinyin_with_other_characters() {
        assert_eq!(kind("10月报告", "10y"), Some(Kind::Pinyin));
        assert_eq!(ranges("10月报告", "10y"), vec![0..3]);
        assert_eq!(ranges("10月报告", "10ybg"), vec![0..5]);
        assert_eq!(ranges("Q3预算", "q3ys"), vec![0..4]);
        // A Chinese character typed as itself.
        assert_eq!(ranges("预算表", "ys表"), vec![0..3]);
        // ü can be typed as v or u.
        assert_eq!(ranges("绿色", "lvs"), vec![0..2]);
        assert_eq!(ranges("绿色", "lus"), vec![0..2]);
    }

    #[test]
    fn pinyin_run_starts_as_early_as_possible() {
        assert_eq!(ranges("2024年预算表", "ysb"), vec![5..8]);
        // 报 (bao) at the start wins over 表 (biao) later.
        assert_eq!(ranges("报表", "b"), vec![0..1]);
        assert!(score("报表", "b") > score("年报", "b"));
    }

    #[test]
    fn pinyin_must_be_contiguous() {
        // 预 then 表 skips 算, so it is no pinyin match; "y" is not in the name either.
        assert_eq!(match_name("预算表", "yb"), None);
        assert_eq!(match_name("预算表.xlsx", "ysbx"), None);
        assert_eq!(match_name("预算表", "ysbq"), None);
    }

    #[test]
    fn only_the_main_reading() {
        // 会 is hui here; kuai (会计) is not known.
        assert_eq!(kind("会计", "kj"), None);
        assert_eq!(kind("会计", "hj"), Some(Kind::Pinyin));
    }

    #[test]
    fn letters_in_order() {
        assert_eq!(kind("Sysbatt.ttf", "ysb"), Some(Kind::Substring));
        assert_eq!(kind("Sysbatt.ttf", "sbtf"), Some(Kind::Fuzzy));
        assert_eq!(
            ranges("Sysbatt.ttf", "sbtf"),
            vec![0..1, 3..4, 5..6, 10..11]
        );
        assert_eq!(ranges("Sysbatt.ttf", "sbtt"), vec![0..1, 3..4, 5..7]);
        assert_eq!(kind("预算表", "预表"), Some(Kind::Fuzzy));
        assert_eq!(ranges("预算表", "预表"), vec![0..1, 2..3]);
        assert_eq!(match_name("Sysbatt.ttf", "tts"), None);
        assert_eq!(match_name("abc", "abcd"), None);
    }

    #[test]
    fn kinds_rank_in_order() {
        let names = ["fe", "Fence", "My fence", "费用", "coffee", "fire"];
        let kinds: Vec<_> = names.iter().map(|n| kind(n, "fe").unwrap()).collect();
        assert_eq!(
            kinds,
            [
                Kind::Exact,
                Kind::Prefix,
                Kind::WordStart,
                Kind::Pinyin,
                Kind::Substring,
                Kind::Fuzzy
            ]
        );
        let scores: Vec<_> = names.iter().map(|n| score(n, "fe")).collect();
        assert!(scores.windows(2).all(|w| w[0] > w[1]), "{scores:?}");
        // The kind decides before position and length.
        assert!(score("a very long name with fe in it", "fe") > score("x费", "fe"));
        // Listary's order for "ysb": the pinyin match first.
        assert!(score("预算表.xlsx", "ysb") > score("Sysbatt.ttf", "ysb"));
    }

    #[test]
    fn within_a_kind() {
        // Earlier position first.
        assert!(score("the big fence", "fence") < score("my fence", "fence"));
        // Then the shorter name.
        assert!(score("fence-long.txt", "fence") < score("fence.txt", "fence"));
        // Then a match in the name over one in the extension (same kind, place and length).
        assert!(score("ab.txt", "txt") < score("ab_txt", "txt"));
        // Earlier position beats a shorter name.
        assert!(score("a fence that is long", "fence") > score("abc fence", "fence"));
    }

    #[test]
    fn several_terms_must_all_match() {
        let m = match_name("New Text Document.txt", "new doc").unwrap();
        assert_eq!(m.ranges, vec![0..3, 9..12]);
        // The weaker term (a word start) decides.
        assert_eq!(m.score >> 24, Kind::WordStart as u32);
        // Any order, extra spaces.
        assert_eq!(
            ranges("New Text Document.txt", "  doc   new "),
            vec![0..3, 9..12]
        );
        assert_eq!(match_name("New Text Document.txt", "new zzz"), None);
        // Overlapping terms merge into one range.
        assert_eq!(ranges("Document", "doc cum"), vec![0..5]);
        // Terms can mix kinds.
        assert_eq!(ranges("2024 预算表.xlsx", "ysb 2024"), vec![0..4, 5..8]);
    }

    #[test]
    fn whole_query_with_spaces_is_tried_too() {
        assert_eq!(kind("New folder", "new folder"), Some(Kind::Exact));
        assert_eq!(kind("New folder (2)", "new fol"), Some(Kind::Prefix));
        assert_eq!(ranges("New folder (2)", "new fol"), vec![0..7]);
    }

    #[test]
    fn unicode_case_folding_keeps_indices() {
        assert_eq!(ranges("ÄBC İx", "äb"), vec![0..2]);
        assert_eq!(ranges("ÄBC İx", "ix"), vec![4..6]);
    }

    /// Run with `cargo test --release -p pecofence-core matcher::tests::timing -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn timing() {
        let templates = [
            "Project Report {}.docx",
            "预算表{}.xlsx",
            "会议纪要_{}.pdf",
            "IMG_{}.JPG",
            "Visual Studio Code {}",
            "微信 {}",
            "2024年{}月工作总结.pptx",
            "SomeLongCamelCaseName{}.lnk",
            "新建文件夹 ({})",
            "readme-{}.md",
        ];
        let names: Vec<String> = (0..1000)
            .map(|i| templates[i % templates.len()].replace("{}", &i.to_string()))
            .collect();
        for query in ["ysb", "rep", "vsc", "huiyij", "zzz", "gzzj", "new doc", "x"] {
            let rounds = 200;
            let start = std::time::Instant::now();
            let mut hits = 0;
            for _ in 0..rounds {
                hits += names
                    .iter()
                    .filter(|n| match_name(n, query).is_some())
                    .count();
            }
            let per = start.elapsed() / rounds;
            println!(
                "{query:>8}: {per:?} for 1000 names ({} hits)",
                hits / rounds as usize
            );
        }
    }
}
