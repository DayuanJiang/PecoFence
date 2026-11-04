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
