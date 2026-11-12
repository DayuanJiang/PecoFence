//! 便签与待办: the lines of a note fence. A line that starts with `[ ]` (or `[]`) is an open
//! to-do and one that starts with `[x]` a done one, also after a Markdown `- ` or `* `; every
//! other line is plain text.

/// One line of a note as it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteLine<'a> {
    Text(&'a str),
    Todo { done: bool, text: &'a str },
}

/// Splits `line` into its check box (if any) and the text after it.
pub fn parse_line(line: &str) -> NoteLine<'_> {
    match marker(line) {
        Some((_, end, done)) => NoteLine::Todo {
            done,
            text: line[end..].trim_start(),
        },
        None => NoteLine::Text(line),
    }
}

/// The lines of `text` (a trailing line break makes no empty last line, as in an editor).
pub fn lines(text: &str) -> Vec<NoteLine<'_>> {
    text.lines().map(parse_line).collect()
}

/// The lines of a note that is being edited: every line break starts a line, so a line break
/// at the very end leaves an empty last line for the caret to stand on.
pub fn edit_lines(text: &str) -> Vec<NoteLine<'_>> {
    text.split('\n').map(parse_line).collect()
}

/// Where one line of a note that is being edited (`edit_lines`) sits in its text, in UTF-16
/// units as an edit control counts them: the line's start, how many units come before the
/// text that is drawn (a to-do's marker and the spaces around it; 0 for a plain line), and
/// the line's length without its line break.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineSpan {
    pub start: usize,
    pub prefix: usize,
    pub len: usize,
}

fn units(s: &str) -> usize {
    s.encode_utf16().count()
}

/// The spans of `text`'s lines (line breaks are `\n`).
pub fn line_spans(text: &str) -> Vec<LineSpan> {
    let mut start = 0;
    text.split('\n')
        .map(|line| {
            let shown = match parse_line(line) {
                NoteLine::Todo { text, .. } => text,
                NoteLine::Text(text) => text,
            };
            let span = LineSpan {
                start,
                prefix: units(&line[..line.len() - shown.len()]),
                len: units(line),
            };
            start += span.len + 1;
            span
        })
        .collect()
}

/// The line that holds offset `offset` of the text, and the offset in that line's drawn
/// text (0 for a place inside a to-do's marker).
pub fn locate(spans: &[LineSpan], offset: usize) -> (usize, usize) {
    let line = spans
        .iter()
        .rposition(|s| s.start <= offset)
        .unwrap_or_default();
    let Some(span) = spans.get(line) else {
        return (0, 0);
    };
    let in_line = (offset - span.start).min(span.len);
    (line, in_line.saturating_sub(span.prefix))
}

/// The offset in the text of place `at` of line `line`'s drawn text.
pub fn offset_of(spans: &[LineSpan], line: usize, at: usize) -> usize {
    match spans.get(line) {
        Some(span) => span.start + (span.prefix + at).min(span.len),
        None => spans.last().map_or(0, |s| s.start + s.len),
    }
}

/// The part of each line that the selection `from..to` covers: the line, the range in its
/// drawn text, and whether the selection goes on past the line's end.
pub fn selection_by_line(
    spans: &[LineSpan],
    from: usize,
    to: usize,
) -> Vec<(usize, usize, usize, bool)> {
    let mut out = Vec::new();
    if from >= to {
        return out;
    }
    for (i, span) in spans.iter().enumerate() {
        let end = span.start + span.len;
        if to <= span.start || from > end {
            continue;
        }
        let shown = span.len - span.prefix.min(span.len);
        let place = |offset: usize| {
            (offset.clamp(span.start, end) - span.start)
                .saturating_sub(span.prefix)
                .min(shown)
        };
        out.push((i, place(from), place(to), to > end));
    }
    out
}

/// Offset `offset` of an edit control's text (`units`, its line breaks are `\r\n`) as an
/// offset of the same text with `\n` line breaks.
pub fn crlf_to_lf_offset(units: &[u16], offset: usize) -> usize {
    let offset = offset.min(units.len());
    let breaks = (0..offset)
        .filter(|&i| units[i] == 13 && units.get(i + 1) == Some(&10))
        .count();
    offset - breaks
}

/// The reverse of [`crlf_to_lf_offset`].
pub fn lf_to_crlf_offset(units: &[u16], offset: usize) -> usize {
    let (mut i, mut n) = (0, 0);
    while n < offset && i < units.len() {
        i += if units[i] == 13 && units.get(i + 1) == Some(&10) {
            2
        } else {
            1
        };
        n += 1;
    }
    i
}

/// The word around offset `offset` of `text` (UTF-16 units): the run of letters and digits,
/// of spaces, or of other characters that holds it, within its line. A double-click selects
/// it.
pub fn word_at(text: &str, offset: usize) -> (usize, usize) {
    let chars: Vec<u16> = text.encode_utf16().collect();
    if chars.is_empty() {
        return (0, 0);
    }
    let class = |u: u16| match char::from_u32(u32::from(u)) {
        Some('\n') => 0,
        Some(c) if c.is_whitespace() => 1,
        Some(c) if c.is_alphanumeric() => 2,
        // Half of a surrogate pair: part of a word.
        None => 2,
        Some(_) => 3,
    };
    let at = offset.min(chars.len() - 1);
    let kind = class(chars[at]);
    if kind == 0 {
        return (offset.min(chars.len()), offset.min(chars.len()));
    }
    let mut start = at;
    while start > 0 && class(chars[start - 1]) == kind {
        start -= 1;
    }
    let mut end = at + 1;
    while end < chars.len() && class(chars[end]) == kind {
        end += 1;
    }
    (start, end)
}

/// `text` with the check box of line `index` flipped; None when that line is no to-do.
pub fn toggle(text: &str, index: usize) -> Option<String> {
    let mut out = String::with_capacity(text.len() + 1);
    let mut found = false;
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let (body, cr) = match line.strip_suffix('\r') {
            Some(b) => (b, "\r"),
            None => (line, ""),
        };
        match marker(body).filter(|_| i == index) {
            Some((start, end, done)) => {
                out.push_str(&body[..start]);
                out.push_str(if done { "[ ]" } else { "[x]" });
                out.push_str(&body[end..]);
                found = true;
            }
            None => out.push_str(body),
        }
        out.push_str(cr);
    }
    found.then_some(out)
}

/// What Enter does in a to-do of a note that is being edited, as lists do in editors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TodoEnter {
    /// A to-do with text: this goes in at the caret, a line break and an empty check box led
    /// in as the line's own is (its indent, its `- `).
    NewBox(String),
    /// A to-do without text: the line's check box (these UTF-16 units of the text) goes and
    /// the line is a plain one again.
    RemoveBox(usize, usize),
}

/// What Enter does with the selection `first..last` (UTF-16 units, line breaks are `\n`) of
/// `text`: None in a plain line and when the selection starts inside a check box's marker
/// (the editor breaks the line itself).
pub fn todo_enter(text: &str, first: usize, last: usize) -> Option<TodoEnter> {
    let mut start = 0;
    for line in text.split('\n') {
        let len = units(line);
        if first <= start + len {
            let (lead, end, _) = marker(line)?;
            let prefix = units(&line[..line.len() - line[end..].trim_start().len()]);
            if first < start + prefix {
                return None;
            }
            return Some(if prefix == len && first == last {
                TodoEnter::RemoveBox(start, start + len)
            } else {
                TodoEnter::NewBox(format!("\n{}[ ] ", &line[..lead]))
            });
        }
        start += len + 1;
    }
    None
}

/// What 「添加待办」 puts after `text`: an empty check box on a line of its own. Nothing when
/// the last line already is an empty check box.
pub fn new_todo_suffix(text: &str) -> &'static str {
    let last = text.rsplit('\n').next().unwrap_or("");
    match parse_line(last) {
        NoteLine::Todo { text: "", .. } => "",
        _ if last.is_empty() => "[ ] ",
        _ => "\n[ ] ",
    }
}

/// Byte range of the `[ ]` / `[]` / `[x]` marker at the start of `line` (after leading spaces
/// and an optional `- ` or `* `), and whether it is ticked.
fn marker(line: &str) -> Option<(usize, usize, bool)> {
    let indent = line.len() - line.trim_start().len();
    let mut start = indent;
    let rest = &line[start..];
    if let Some(after) = rest.strip_prefix("- ").or_else(|| rest.strip_prefix("* ")) {
        start = line.len() - after.len();
    }
    let rest = &line[start..];
    for (m, done) in [("[ ]", false), ("[]", false), ("[x]", true), ("[X]", true)] {
        if rest.starts_with(m) {
            return Some((start, start + m.len(), done));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Types `typed` one character at a time after `state`'s text, the caret at the end.
    fn type_in(history: &mut EditHistory, state: &mut EditState, typed: &str) {
        for c in typed.chars() {
            let mut text = state.0.clone();
            text.push(c);
            history.record(state, &text);
            let end = text.encode_utf16().count();
            *state = (text, end, end);
        }
    }

    #[test]
    fn edit_history_goes_back_step_by_step_and_forward_again() {
        let mut history = EditHistory::default();
        let mut state: EditState = ("买菜".into(), 2, 2);
        type_in(&mut history, &mut state, " 西红柿 鸡蛋");
        // Two words typed: two steps (each word with the space before it).
        assert_eq!(history.depth(), (2, 0));
        // A paste over a selection is a step of its own.
        let selected = (state.0.clone(), 0, 2);
        history.break_run();
        history.record(&selected, "今天 西红柿 鸡蛋");
        state = ("今天 西红柿 鸡蛋".into(), 2, 2);
        // A check box ticked (the text keeps its length) too.
        history.record(&state, "今天 西红柿 鸡蛋!");
        history.record(&("今天 西红柿 鸡蛋!".into(), 2, 2), "今天 西红柿 鸡蛋?");
        let mut now: EditState = ("今天 西红柿 鸡蛋?".into(), 2, 2);
        let mut seen = Vec::new();
        while let Some(to) = history.undo(now.clone()) {
            seen.push(to.0.clone());
            now = to;
        }
        assert_eq!(
            seen,
            [
                "今天 西红柿 鸡蛋!",
                "今天 西红柿 鸡蛋",
                "买菜 西红柿 鸡蛋",
                "买菜 西红柿",
                "买菜"
            ]
        );
        // The selection comes back with the text.
        assert_eq!(now, ("买菜".to_string(), 2, 2));
        // Forward again, every step.
        let mut forward = Vec::new();
        while let Some(to) = history.redo(now.clone()) {
            forward.push(to.0.clone());
            now = to;
        }
        assert_eq!(forward.len(), 5);
        assert_eq!(now.0, "今天 西红柿 鸡蛋?");
        // A second Ctrl+Z goes one step further back, it does not redo.
        let one = history.undo(now.clone()).unwrap();
        let two = history.undo(one.clone()).unwrap();
        assert_eq!(
            (one.0.as_str(), two.0.as_str()),
            ("今天 西红柿 鸡蛋!", "今天 西红柿 鸡蛋")
        );
        // Typing after an undo drops what was undone.
        let mut state = two;
        type_in(&mut history, &mut state, "x");
        assert_eq!(history.depth().1, 0);
        assert!(history.redo(state.clone()).is_none());
    }

    #[test]
    fn edit_history_joins_a_run_of_backspaces_and_breaks_at_a_caret_move() {
        let mut history = EditHistory::default();
        let mut state: EditState = ("abcdef".into(), 6, 6);
        for text in ["abcde", "abcd", "abc"] {
            history.record(&state, text);
            state = (text.into(), text.len(), text.len());
        }
        assert_eq!(history.depth(), (1, 0));
        // Typing after deleting is the next step; after a caret move another one.
        type_in(&mut history, &mut state, "xy");
        assert_eq!(history.depth(), (2, 0));
        history.break_run();
        type_in(&mut history, &mut state, "z");
        assert_eq!(history.depth(), (3, 0));
        assert_eq!(history.undo(state.clone()).unwrap().0, "abcxy");
    }

    #[test]
    fn edited_note_lines_and_offsets() {
        let text = "周会\n[ ] 回邮件\n  - [x] done\n";
        assert_eq!(edit_lines(text).len(), 4);
        let spans = line_spans(text);
        assert_eq!(
            spans,
            vec![
                LineSpan {
                    start: 0,
                    prefix: 0,
                    len: 2
                },
                LineSpan {
                    start: 3,
                    prefix: 4,
                    len: 7
                },
                LineSpan {
                    start: 11,
                    prefix: 8,
                    len: 12
                },
                LineSpan {
                    start: 24,
                    prefix: 0,
                    len: 0
                },
            ]
        );
        // A place inside a marker is the start of the line's drawn text.
        assert_eq!(locate(&spans, 0), (0, 0));
        assert_eq!(locate(&spans, 2), (0, 2));
        assert_eq!(locate(&spans, 3), (1, 0));
        assert_eq!(locate(&spans, 5), (1, 0));
        assert_eq!(locate(&spans, 7), (1, 0));
        assert_eq!(locate(&spans, 9), (1, 2));
        assert_eq!(locate(&spans, 24), (3, 0));
        assert_eq!(locate(&spans, 99), (3, 0));
        assert_eq!(offset_of(&spans, 1, 0), 7);
        assert_eq!(offset_of(&spans, 1, 3), 10);
        assert_eq!(offset_of(&spans, 1, 9), 10);
        assert_eq!(offset_of(&spans, 3, 0), 24);
        assert_eq!(offset_of(&spans, 9, 0), 24);
        // From the middle of line 0 into line 1's text.
        assert_eq!(
            selection_by_line(&spans, 1, 9),
            vec![(0, 1, 2, true), (1, 0, 2, false)]
        );
        assert_eq!(selection_by_line(&spans, 4, 4), vec![]);
        // Everything: the empty last line is part of it.
        assert_eq!(selection_by_line(&spans, 0, 24).len(), 3);
    }

    /// Enter in a to-do starts the next one; in an empty to-do it takes the check box away
    /// (feedback #141).
    #[test]
    fn enter_in_a_todo() {
        let text = "周会\n[ ] 回邮件\n  - [x] done\n[ ] \n[]";
        // A plain line: the editor's own line break.
        assert_eq!(todo_enter(text, 2, 2), None);
        // At the end and in the middle of a to-do's text, and with a selection in it.
        let new_box = Some(TodoEnter::NewBox("\n[ ] ".to_string()));
        assert_eq!(todo_enter(text, 10, 10), new_box);
        assert_eq!(todo_enter(text, 8, 8), new_box);
        assert_eq!(todo_enter(text, 7, 10), new_box);
        // Inside the marker, which is not shown: left to the editor.
        assert_eq!(todo_enter(text, 5, 5), None);
        // The new line is led in as the old one, and its box is empty.
        assert_eq!(
            todo_enter(text, 23, 23),
            Some(TodoEnter::NewBox("\n  - [ ] ".to_string()))
        );
        // An empty to-do loses its check box, with or without the space after it.
        assert_eq!(todo_enter(text, 28, 28), Some(TodoEnter::RemoveBox(24, 28)));
        assert_eq!(todo_enter(text, 31, 31), Some(TodoEnter::RemoveBox(29, 31)));
        assert_eq!(todo_enter("", 0, 0), None);
        assert_eq!(todo_enter("[ ] 买菜", 6, 6), new_box);
    }

    #[test]
    fn add_todo_appends_an_empty_check_box() {
        assert_eq!(new_todo_suffix(""), "[ ] ");
        assert_eq!(new_todo_suffix("周会"), "\n[ ] ");
        assert_eq!(new_todo_suffix("周会\n"), "[ ] ");
        assert_eq!(new_todo_suffix("[x] 回邮件"), "\n[ ] ");
        // The last line is an empty check box already: the caret just goes there.
        assert_eq!(new_todo_suffix("周会\n[ ] "), "");
        assert_eq!(new_todo_suffix("[]"), "");
    }

    #[test]
    fn edit_control_offsets() {
        let crlf: Vec<u16> = "ab\r\n\r\ncd".encode_utf16().collect();
        // "ab\n\ncd"
        for (rn, n) in [(0, 0), (2, 2), (4, 3), (6, 4), (8, 6)] {
            assert_eq!(crlf_to_lf_offset(&crlf, rn), n);
            assert_eq!(lf_to_crlf_offset(&crlf, n), rn);
        }
        assert_eq!(crlf_to_lf_offset(&crlf, 99), 6);
        assert_eq!(lf_to_crlf_offset(&crlf, 99), 8);
    }

    #[test]
    fn double_click_word() {
        let text = "回邮件 today, ok\n第二行";
        assert_eq!(word_at(text, 1), (0, 3));
        assert_eq!(word_at(text, 5), (4, 9));
        assert_eq!(word_at(text, 9), (9, 10));
        assert_eq!(word_at(text, 3), (3, 4));
        assert_eq!(word_at(text, 14), (14, 17));
        assert_eq!(word_at(text, 99), (14, 17));
        assert_eq!(word_at("", 0), (0, 0));
    }

    #[test]
    fn plain_and_todo_lines() {
        assert_eq!(parse_line("买牛奶"), NoteLine::Text("买牛奶"));
        assert_eq!(
            parse_line("[ ] 交房租"),
            NoteLine::Todo {
                done: false,
                text: "交房租"
            }
        );
        assert_eq!(
            parse_line("- [x] done"),
            NoteLine::Todo {
                done: true,
                text: "done"
            }
        );
        assert_eq!(
            parse_line("  []x"),
            NoteLine::Todo {
                done: false,
                text: "x"
            }
        );
        // A bracket later in the line is text.
        assert_eq!(parse_line("见 [x] 号"), NoteLine::Text("见 [x] 号"));
        assert_eq!(lines("a\n[ ] b\n").len(), 2);
    }

    #[test]
    fn toggle_flips_only_that_line() {
        let text = "标题\r\n[ ] 一\r\n- [X] 二\n[] 三";
        assert_eq!(
            toggle(text, 1).as_deref(),
            Some("标题\r\n[x] 一\r\n- [X] 二\n[] 三")
        );
        assert_eq!(
            toggle(text, 2).as_deref(),
            Some("标题\r\n[ ] 一\r\n- [ ] 二\n[] 三")
        );
        assert_eq!(
            toggle(text, 3).as_deref(),
            Some("标题\r\n[ ] 一\r\n- [X] 二\n[x] 三")
        );
        assert_eq!(toggle(text, 0), None);
        assert_eq!(toggle(text, 9), None);
    }
}
