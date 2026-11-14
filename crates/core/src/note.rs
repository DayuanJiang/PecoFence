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

/// Where the moving end of a selection goes when it has stopped inside a to-do's marker
/// (`[ ] `, which is not shown): on to the start of the line's text when the selection grows
/// forward, back to the start of the line when it grows backward. None when `active` is not
/// inside a marker. (Places in UTF-16 units, line breaks are `\n`.)
pub fn selection_end_out_of_marker(text: &str, anchor: usize, active: usize) -> Option<usize> {
    let span = line_spans(text)
        .into_iter()
        .find(|s| active > s.start && active < s.start + s.prefix.min(s.len))?;
    Some(if active > anchor {
        span.start + span.prefix.min(span.len)
    } else {
        span.start
    })
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

/// `text` with its lines `first..first + count` taken out and put back so that the first of
/// them is line `to` of the result: a to-do dragged to another place. Only these lines
/// change places; every line break stays where it is in the text (a `\r\n` stays one, and a
/// note whose last line has no line break still has none). None when the lines are not all
/// there, `to` is past the place they fit, or they are at `to` already.
pub fn move_lines(text: &str, first: usize, count: usize, to: usize) -> Option<String> {
    let (mut lines, mut breaks) = (Vec::new(), Vec::new());
    for piece in text.split_inclusive('\n') {
        let line = piece
            .strip_suffix("\r\n")
            .or_else(|| piece.strip_suffix('\n'))
            .unwrap_or(piece);
        lines.push(line);
        breaks.push(&piece[line.len()..]);
    }
    if count == 0 || to == first || first + count > lines.len() || to + count > lines.len() {
        return None;
    }
    let moved: Vec<&str> = lines.drain(first..first + count).collect();
    lines.splice(to..to, moved);
    Some(
        lines
            .iter()
            .zip(breaks)
            .flat_map(|(l, b)| [*l, b])
            .collect(),
    )
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

/// The web addresses in `text`, the drawn text of one line of a note: each one's start and
/// end in UTF-16 units. An address starts with `http://` or `https://` where a word starts
/// and runs to the next space, quote or angle bracket, or the next character that is not
/// ASCII (Chinese text often follows an address without a space); what a sentence puts after it (`.`, `,`, `;`, `:`,
/// `!`, `?`, a quote, a closing bracket that was never opened) is not part of it. Nothing
/// else is an address: a click on a note opens these two kinds only, as a script fence's row
/// does ([`crate::script_output::is_web_address`]).
pub fn links(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    // Only ASCII letters change, so the lower-cased text has the same byte offsets.
    let lower = text.to_ascii_lowercase();
    let mut from = 0;
    while from < text.len() {
        let Some(at) = ["https://", "http://"]
            .iter()
            .filter_map(|scheme| lower[from..].find(scheme).map(|i| (i, scheme.len())))
            .min()
        else {
            break;
        };
        let (start, scheme) = (from + at.0, at.1);
        let word_start = text[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        let run = text[start..]
            .find(|c: char| {
                !c.is_ascii()
                    || c.is_ascii_whitespace()
                    || c.is_ascii_control()
                    || matches!(c, '"' | '<' | '>')
            })
            .unwrap_or(text.len() - start);
        let mut address = &text[start..start + run];
        // What follows the address in its sentence.
        while let Some(last) = address.chars().next_back() {
            let closes = match last {
                ')' => Some('('),
                ']' => Some('['),
                '}' => Some('{'),
                _ => None,
            };
            let ends_sentence = matches!(last, '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"')
                || closes.is_some_and(|open| {
                    address.matches(open).count() < address.matches(last).count()
                });
            if !ends_sentence {
                break;
            }
            address = &address[..address.len() - 1];
        }
        if word_start && address.len() > scheme && crate::script_output::is_web_address(address) {
            let first = units(&text[..start]);
            // ASCII: one unit per byte.
            out.push((first, first + address.len()));
        }
        from = start + run.max(scheme);
    }
    out
}

/// A note's text and selection while it is edited (the two ends in UTF-16 units, line breaks
/// counted as `\n`).
pub type EditState = (String, usize, usize);

/// `text` cut in two at `units` UTF-16 units; None when that is past its end or inside a
/// character.
fn split_at_units(text: &str, units: usize) -> Option<(&str, &str)> {
    let mut seen = 0;
    for (i, c) in text.char_indices() {
        if seen >= units {
            return (seen == units).then(|| text.split_at(i));
        }
        seen += c.len_utf16();
    }
    (seen == units).then_some((text, ""))
}

/// The undo history of one editing session of a note: the states Ctrl+Z goes back through
/// and Ctrl+Y forward again. A run of typing (or of deleting) is one step; a space typed after
/// a word starts the next step, and so does everything else: a paste, a cut, a check box
/// ticked, typing after the caret was moved.
#[derive(Debug, Default)]
pub struct EditHistory {
    undo: Vec<EditState>,
    redo: Vec<EditState>,
    run: Option<Run>,
}

/// The run of single characters the last change belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Run {
    /// Typed one after the other; `space`: the last one was a space or a line break.
    Typing {
        space: bool,
    },
    Deleting,
}

impl EditHistory {
    /// The states kept: older ones are dropped.
    const KEPT: usize = 200;

    /// The text changed from the state `before` to `text`: `before` becomes a step to go back
    /// to, unless the change goes on with the run of typing or deleting before it. What was
    /// undone can no longer be redone.
    pub fn record(&mut self, before: &EditState, text: &str) {
        let (old, new) = (before.0.chars().count(), text.chars().count());
        let bare_caret = before.1 == before.2;
        // The text before and after the caret: typing puts one character between the two,
        // Backspace takes the last one of the first, Delete the first one of the second. A
        // change of that length anywhere else (a check box `[]` ticked to `[x]`) is a step of
        // its own.
        let at_caret = split_at_units(&before.0, before.1).filter(|_| bare_caret);
        let run = match at_caret {
            Some((head, tail)) if new == old + 1 => {
                (text.starts_with(head) && text.ends_with(tail)).then(|| Run::Typing {
                    space: text[head.len()..]
                        .chars()
                        .next()
                        .is_some_and(char::is_whitespace),
                })
            }
            Some((head, tail)) if new + 1 == old => {
                let backspace =
                    text.ends_with(tail) && head.starts_with(&text[..text.len() - tail.len()]);
                let delete = text.starts_with(head) && tail.ends_with(&text[head.len()..]);
                (backspace || delete).then_some(Run::Deleting)
            }
            _ => None,
        };
        let goes_on = match (self.run, run) {
            // A space after a word ends the step: the next word goes with its space.
            (Some(Run::Typing { space: was_space }), Some(Run::Typing { space })) => {
                was_space || !space
            }
            (Some(Run::Deleting), Some(Run::Deleting)) => true,
            _ => false,
        };
        self.run = run;
        self.redo.clear();
        if !goes_on {
            self.undo.push(before.clone());
            if self.undo.len() > Self::KEPT {
                self.undo.remove(0);
            }
        }
    }

    /// The caret or the selection moved without a change of the text: what is typed next is
    /// a step of its own.
    pub fn break_run(&mut self) {
        self.run = None;
    }

    /// Ctrl+Z from the state `now`: the state to go back to, if there is one.
    pub fn undo(&mut self, now: EditState) -> Option<EditState> {
        let to = self.undo.pop()?;
        self.redo.push(now);
        self.run = None;
        Some(to)
    }

    /// Ctrl+Y from the state `now`: the state that was undone last, if there is one.
    pub fn redo(&mut self, now: EditState) -> Option<EditState> {
        let to = self.redo.pop()?;
        self.undo.push(now);
        self.run = None;
        Some(to)
    }

    /// How many steps Ctrl+Z and Ctrl+Y can go (for the log).
    pub fn depth(&self) -> (usize, usize) {
        (self.undo.len(), self.redo.len())
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
    fn a_check_box_ticked_is_a_step_of_its_own_also_when_the_text_grows_by_one() {
        // `[]` ticked becomes `[x]`: one character more, as if typed. It was taken for the
        // next letter of the word before it, and Ctrl+Z took the tick and the word back.
        let mut history = EditHistory::default();
        let mut state: EditState = ("[] mil".into(), 6, 6);
        type_in(&mut history, &mut state, "k");
        history.record(&state, "[x] milk");
        let ticked: EditState = ("[x] milk".into(), 8, 8);
        assert_eq!(
            history.undo(ticked.clone()).map(|s| s.0).as_deref(),
            Some("[] milk")
        );
        // Unticked again (one character fewer) after Backspace: a step of its own too.
        let mut history = EditHistory::default();
        history.record(&("[x] milk".into(), 8, 8), "[x] mil");
        history.record(&("[x] mil".into(), 7, 7), "[] mil");
        let now: EditState = ("[] mil".into(), 6, 6);
        assert_eq!(history.undo(now).map(|s| s.0).as_deref(), Some("[x] mil"));
        // Typing in the middle of a text, Backspace and Delete still make runs.
        let mut history = EditHistory::default();
        history.record(&("ab".into(), 1, 1), "axb");
        history.record(&("axb".into(), 2, 2), "axyb");
        history.record(&("axyb".into(), 3, 3), "axb");
        history.record(&("axb".into(), 2, 2), "ab");
        history.record(&("ab".into(), 1, 1), "a");
        assert_eq!(history.depth(), (2, 0));
    }

    #[test]
    fn a_selection_does_not_end_inside_a_to_dos_marker() {
        // 「abc」 and a to-do 「milk」: the caret after 「abc」, Shift+Right twice. The second
        // press put the end one place into the unseen 「[ ] 」: nothing changed on screen for
        // three more presses, and a letter typed then left 「abcx ] milk」.
        let text = "abc\n[ ] milk";
        // After the line break (the line's start): a place that shows, left alone.
        assert_eq!(selection_end_out_of_marker(text, 3, 4), None);
        // Inside the marker going forward: on to the text of the to-do.
        assert_eq!(selection_end_out_of_marker(text, 3, 5), Some(8));
        assert_eq!(selection_end_out_of_marker(text, 3, 7), Some(8));
        assert_eq!(selection_end_out_of_marker(text, 3, 8), None);
        // Going backward from inside the to-do's text: back to the start of the line.
        assert_eq!(selection_end_out_of_marker(text, 10, 7), Some(4));
        assert_eq!(selection_end_out_of_marker(text, 10, 4), None);
        // A plain line has no marker.
        assert_eq!(selection_end_out_of_marker(text, 0, 2), None);
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

    #[test]
    fn move_lines_moves_only_those_lines() {
        let text = "[ ] 一\n[x] 二\n[ ] 三";
        // The last to-do to the top, the first one to the end, one step down and up.
        assert_eq!(
            move_lines(text, 2, 1, 0).as_deref(),
            Some("[ ] 三\n[ ] 一\n[x] 二")
        );
        assert_eq!(
            move_lines(text, 0, 1, 2).as_deref(),
            Some("[x] 二\n[ ] 三\n[ ] 一")
        );
        assert_eq!(
            move_lines(text, 0, 1, 1).as_deref(),
            Some("[x] 二\n[ ] 一\n[ ] 三")
        );
        assert_eq!(
            move_lines(text, 2, 1, 1).as_deref(),
            Some("[ ] 一\n[ ] 三\n[x] 二")
        );
        // A move back gives the text it was.
        let moved = move_lines(text, 2, 1, 0).unwrap();
        assert_eq!(move_lines(&moved, 0, 1, 2).as_deref(), Some(text));
        // Nothing moves: the place it has, a line that is not there, a place past the end,
        // no lines, a single to-do, an empty note.
        assert_eq!(move_lines(text, 1, 1, 1), None);
        assert_eq!(move_lines(text, 3, 1, 0), None);
        assert_eq!(move_lines(text, 0, 1, 3), None);
        assert_eq!(move_lines(text, 0, 0, 1), None);
        assert_eq!(move_lines("[ ] 一", 0, 1, 0), None);
        assert_eq!(move_lines("[ ] 一", 0, 1, 1), None);
        assert_eq!(move_lines("", 0, 1, 0), None);
        assert_eq!(move_lines("", 0, 1, 1), None);
        // A line break at the end makes no line and stays at the end.
        assert_eq!(
            move_lines("[ ] 一\n[ ] 二\n", 1, 1, 0).as_deref(),
            Some("[ ] 二\n[ ] 一\n")
        );
        assert_eq!(move_lines("[ ] 一\n[ ] 二\n", 0, 1, 2), None);
        // A long line (the note wraps it on screen) is one line and moves whole.
        let long = "[ ] 把上周的周报、下周的计划和三张报销单一起发给财务，抄送组长";
        let wrapped = format!("周会\n{long}\n[ ] 回邮件");
        assert_eq!(
            move_lines(&wrapped, 1, 1, 2),
            Some(format!("周会\n[ ] 回邮件\n{long}"))
        );
        // Plain lines between the to-dos keep their order; a to-do can land among them.
        let mixed = "今天\n[ ] 一\n备注\n明天\n[ ] 二";
        assert_eq!(
            move_lines(mixed, 4, 1, 1).as_deref(),
            Some("今天\n[ ] 二\n[ ] 一\n备注\n明天")
        );
        assert_eq!(
            move_lines(mixed, 1, 1, 2).as_deref(),
            Some("今天\n备注\n[ ] 一\n明天\n[ ] 二")
        );
        assert_eq!(
            move_lines(mixed, 4, 1, 0).as_deref(),
            Some("[ ] 二\n今天\n[ ] 一\n备注\n明天")
        );
        // Several lines together.
        assert_eq!(
            move_lines(mixed, 1, 2, 3).as_deref(),
            Some("今天\n明天\n[ ] 二\n[ ] 一\n备注")
        );
        assert_eq!(move_lines(mixed, 1, 2, 4), None);
        // The line breaks stay where they are, each as it was written.
        let crlf = "标题\r\n[ ] 一\r\n- [X] 二\n[] 三";
        assert_eq!(
            move_lines(crlf, 3, 1, 1).as_deref(),
            Some("标题\r\n[] 三\r\n[ ] 一\n- [X] 二")
        );
        assert_eq!(
            move_lines("[ ] 一\r\n[ ] 二\r\n", 0, 1, 1).as_deref(),
            Some("[ ] 二\r\n[ ] 一\r\n")
        );
        // The lines count as `lines` and `toggle` count them.
        assert_eq!(
            lines(&move_lines(crlf, 3, 1, 1).unwrap())[1],
            NoteLine::Todo {
                done: false,
                text: "三"
            }
        );
    }

    #[test]
    fn web_addresses_in_a_note_line() {
        let found = |text: &str| -> Vec<String> {
            let wide: Vec<u16> = text.encode_utf16().collect();
            links(text)
                .into_iter()
                .map(|(a, b)| String::from_utf16_lossy(&wide[a..b]))
                .collect()
        };
        assert_eq!(
            found("详见 https://example.com/wiki/weekly"),
            ["https://example.com/wiki/weekly"]
        );
        // The sentence's punctuation, Chinese text right after it, brackets around it.
        assert_eq!(
            found("看 https://a.example/x。然后"),
            ["https://a.example/x"]
        );
        assert_eq!(
            found("看https://a.example/x，然后"),
            ["https://a.example/x"]
        );
        assert_eq!(found("(see http://a.example/b.)"), ["http://a.example/b"]);
        assert_eq!(
            found("https://en.wikipedia.org/wiki/Fence_(disambiguation), HTTP://B.example"),
            [
                "https://en.wikipedia.org/wiki/Fence_(disambiguation)",
                "HTTP://B.example"
            ]
        );
        // Offsets count UTF-16 units: an emoji before the address is two.
        assert_eq!(links("😀 http://a.example"), [(3, 19)]);
        // No address: other schemes, a path, a scheme alone, one inside a word, one with a
        // quote in it (the browser's command line).
        for none in [
            "file:///C:/Windows/notepad.exe",
            "ms-settings:display",
            "C:\\Windows\\notepad.exe",
            "mailto:a@example.com",
            "https://",
            "xhttps://a.example",
            "javascript:alert(1)",
            "\\\\server\\share",
        ] {
            assert!(links(none).is_empty(), "{none}");
        }
        assert_eq!(found("http://a.example/\"b c"), ["http://a.example/"]);
    }
}
