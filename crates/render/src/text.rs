//! Label fitting: Explorer-style two-line labels with an ellipsis.

use windows_canvas::{TextFormat, TextLayout};

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
    let (mut lo, mut hi) = (0usize, chars.len());
    // Binary search the longest prefix that fits with an ellipsis.
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
    // Trimming the trailing whitespace can only shorten the wrap; measure the final text once.
    let n = lines(&fitted).unwrap_or(max_lines).max(1);
    (fitted, n)
}

/// Single-line fit: the longest prefix of `text` (plus an ellipsis) whose width is at most
/// `max_width` DIPs in `format`. Returns `text` unchanged when it fits.
pub fn fit_width(text: &str, format: &TextFormat, max_width: f32) -> String {
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

/// Advance width of a single line of `text` in `format`, in DIPs.
pub fn measure_width(text: &str, format: &TextFormat) -> f32 {
    TextLayout::new(text, format, 10_000.0, 10_000.0)
        .map(|l| l.metrics().width_including_trailing_whitespace)
        .unwrap_or(0.0)
}
