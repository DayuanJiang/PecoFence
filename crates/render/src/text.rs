//! Label fitting: Explorer-style two-line labels with an ellipsis, and the shaped labels kept
//! between frames.

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use windows_canvas::{TextFormat, TextLayout};
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

#[cfg(test)]
mod tests {
    use super::*;

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
