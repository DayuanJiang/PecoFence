//! 脚本栅栏: what a script prints, line by line, in SwiftBar's format. A line is
//! `text | key=value key="value with spaces"`; the parameters make the row clickable (open a
//! URL, a file, run a program, copy text, hand an action back to the script, run it again)
//! and set its colour, icon and tooltip. `---` is a separator. A row whose text starts with
//! `[ ]` or `[x]` is drawn as a to-do. A line without parameters is plain text, so scripts
//! written before this format show as they did.

use crate::note::{NoteLine, parse_line};

/// One line of the output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptLine {
    Separator,
    Row(ScriptRow),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScriptRow {
    pub text: String,
    /// `[ ]` (false) or `[x]` (true) before the text.
    pub todo: Option<bool>,
    /// `color=`: a name (red, green, ...) or `#rrggbb`, as written.
    pub color: Option<String>,
    /// `icon=`: text drawn before the row (an emoji).
    pub icon: Option<String>,
    pub tooltip: Option<String>,
    pub action: Option<RowAction>,
    /// `refresh=true`: run the script again after the click (and after the action).
    pub refresh: bool,
}

/// What a click on a row does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowAction {
    /// `href=`: the default browser (or whatever handles the URL).
    Href(String),
    /// `open=`: a file or folder, as a double-click on it does.
    Open(String),
    /// `run=` (or SwiftBar's `bash=`) with `param1=`, `param2=`, ...: a program, no window.
    Run { program: String, args: Vec<String> },
    /// `copy=`: text to the clipboard.
    Copy(String),
    /// `action=`: the script runs again with `PECOFENCE_ACTION` set to this (a streaming
    /// script gets it as a line on its standard input).
    Script(String),
}

impl ScriptRow {
    /// A click does something.
    pub fn clickable(&self) -> bool {
        self.action.is_some() || self.refresh
    }

    /// What the click does, for the tooltip when the script gave none.
    pub fn target(&self) -> Option<&str> {
        match self.action.as_ref()? {
            RowAction::Href(s) | RowAction::Open(s) | RowAction::Copy(s) => Some(s),
            RowAction::Run { program, .. } => Some(program),
            RowAction::Script(_) => None,
        }
    }
}

/// The lines of `output` (a script's standard output, already cleaned up).
pub fn parse(output: &str) -> Vec<ScriptLine> {
    output.lines().map(parse_one).collect()
}

fn parse_one(line: &str) -> ScriptLine {
    if line.trim() == "---" {
        return ScriptLine::Separator;
    }
    // The parameters start at the first `|` whose rest is all key=value pairs, so the text
    // may hold a `|` itself; `a | b` without any key=value stays text.
    let (text, params) = line
        .match_indices('|')
        .find_map(|(at, _)| params(&line[at + 1..]).map(|p| (line[..at].trim_end(), p)))
        .unwrap_or((line, Vec::new()));
    let get = |key: &str| {
        params
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.clone())
    };
    let (todo, text) = match parse_line(text) {
        NoteLine::Todo { done, text } => (Some(done), text.to_string()),
        NoteLine::Text(t) => (None, t.to_string()),
    };
    let program = get("run").or_else(|| get("bash"));
    let action = if let Some(url) = get("href") {
        Some(RowAction::Href(url))
    } else if let Some(path) = get("open") {
        Some(RowAction::Open(path))
    } else if let Some(program) = program {
        let mut args = Vec::new();
        for n in 1.. {
            match get(&format!("param{n}")) {
                Some(a) => args.push(a),
                None => break,
            }
        }
        Some(RowAction::Run { program, args })
    } else if let Some(text) = get("copy") {
        Some(RowAction::Copy(text))
    } else {
        get("action").map(RowAction::Script)
    };
    let truthy = |v: Option<String>| v.is_some_and(|v| v.eq_ignore_ascii_case("true"));
    ScriptLine::Row(ScriptRow {
        text,
        todo,
        color: get("color").filter(|c| !c.is_empty()),
        icon: get("icon").filter(|c| !c.is_empty()),
        tooltip: get("tooltip").filter(|c| !c.is_empty()),
        action,
        refresh: truthy(get("refresh")),
    })
}

/// `key=value` pairs separated by spaces; a value may be quoted with `"` or `'`. None when
/// the text holds no pair at all (then the `|` was part of the text).
fn params(text: &str) -> Option<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut chars = text.trim().chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let mut key = String::new();
        while let Some(&c) = chars.peek() {
            if c == '=' || c.is_whitespace() {
                break;
            }
            key.push(c);
            chars.next();
        }
        if key.is_empty() {
            break;
        }
        if chars.peek() != Some(&'=') {
            // A word without `=`: not a parameter list.
            return None;
        }
        chars.next();
        let mut value = String::new();
        match chars.peek().copied() {
            Some(q @ ('"' | '\'')) => {
                chars.next();
                for c in chars.by_ref() {
                    if c == q {
                        break;
                    }
                    value.push(c);
                }
            }
            _ => {
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() {
                        break;
                    }
                    value.push(c);
                    chars.next();
                }
            }
        }
        out.push((key, value));
    }
    (!out.is_empty()).then_some(out)
}

/// `#rrggbb` or a colour name as RGB, with a darker shade for light backgrounds and a lighter
/// one for dark ones (`dark` = the fence is dark). None for what it does not know.
pub fn color_rgb(color: &str, dark: bool) -> Option<[u8; 3]> {
    let c = color.trim();
    if let Some(hex) = c.strip_prefix('#') {
        if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
            let v = u32::from_str_radix(hex, 16).ok()?;
            return Some([(v >> 16) as u8, (v >> 8) as u8, v as u8]);
        }
        return None;
    }
    // Windows' own status colours (Fluent "critical", "success", "caution", ...).
    let (light, on_dark) = match c.to_ascii_lowercase().as_str() {
        "red" => ([0xC4, 0x2B, 0x1C], [0xFF, 0x99, 0xA4]),
        "green" => ([0x0F, 0x7B, 0x0F], [0x6C, 0xCB, 0x5F]),
        "orange" | "amber" => ([0x9D, 0x5D, 0x00], [0xFC, 0xE1, 0x00]),
        "yellow" => ([0x8A, 0x6D, 0x00], [0xFF, 0xE4, 0x6B]),
        "blue" => ([0x00, 0x5F, 0xB8], [0x60, 0xCD, 0xFF]),
        "purple" => ([0x88, 0x17, 0x98], [0xD6, 0x9C, 0xF0]),
        "gray" | "grey" => ([0x5F, 0x5F, 0x5F], [0xA0, 0xA0, 0xA0]),
        _ => return None,
    };
    Some(if dark { on_dark } else { light })
}

/// A streaming script names itself so in a comment (SwiftBar's metadata tag, or ours).
pub fn is_streamable(source: &str) -> bool {
    source.contains("<swiftbar.type>streamable</swiftbar.type>")
        || source.contains("<pecofence.type>streamable</pecofence.type>")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(line: &str) -> ScriptRow {
        match parse_one(line) {
            ScriptLine::Row(r) => r,
            ScriptLine::Separator => panic!("separator"),
        }
    }

    #[test]
    fn plain_lines_stay_text() {
        let r = row("Tokyo  20°C  Light rain");
        assert_eq!(r.text, "Tokyo  20°C  Light rain");
        assert!(!r.clickable());
        // A bar without key=value is part of the text.
        assert_eq!(row("a | b").text, "a | b");
        let r = row("#12  Crash | Win10 | href=https://x/12 tooltip='a | b'");
        assert_eq!(r.text, "#12  Crash | Win10");
        assert_eq!(r.action, Some(RowAction::Href("https://x/12".into())));
        assert_eq!(r.tooltip.as_deref(), Some("a | b"));
        assert_eq!(parse_one(" --- "), ScriptLine::Separator);
    }

    #[test]
    fn actions_and_looks() {
        let r =
            row("PR #186 | href=https://github.com/x/y/pull/186 tooltip=\"open it\" color=green");
        assert_eq!(r.text, "PR #186");
        assert_eq!(
            r.action,
            Some(RowAction::Href("https://github.com/x/y/pull/186".into()))
        );
        assert_eq!(r.tooltip.as_deref(), Some("open it"));
        assert_eq!(r.color.as_deref(), Some("green"));
        let r =
            row("Backup | bash='C:\\scripts\\backup now.cmd' param1=--full param2=D: refresh=true");
        assert_eq!(
            r.action,
            Some(RowAction::Run {
                program: "C:\\scripts\\backup now.cmd".into(),
                args: vec!["--full".into(), "D:".into()],
            })
        );
        assert!(r.refresh);
        assert_eq!(row("Refresh | refresh=true").action, None);
        assert!(row("Refresh | refresh=true").clickable());
        assert_eq!(
            row("IP | copy=10.0.0.2 icon=📋").action,
            Some(RowAction::Copy("10.0.0.2".into()))
        );
        assert_eq!(
            row("IP | copy=10.0.0.2 icon=📋").icon.as_deref(),
            Some("📋")
        );
    }

    #[test]
    fn todos_hand_actions_back() {
        let r = row("[x] Pay rent | action=toggle:3");
        assert_eq!(r.todo, Some(true));
        assert_eq!(r.text, "Pay rent");
        assert_eq!(r.action, Some(RowAction::Script("toggle:3".into())));
        assert_eq!(row("[ ] Call Anna").todo, Some(false));
    }

    #[test]
    fn colours() {
        assert_eq!(color_rgb("#102030", false), Some([0x10, 0x20, 0x30]));
        assert_eq!(color_rgb("RED", false), Some([0xC4, 0x2B, 0x1C]));
        assert_ne!(color_rgb("red", true), color_rgb("red", false));
        assert_eq!(color_rgb("#12", false), None);
        assert_eq!(color_rgb("chartreuse", false), None);
    }

    #[test]
    fn streamable_tag() {
        assert!(is_streamable(
            "# <swiftbar.type>streamable</swiftbar.type>\nwhile ..."
        ));
        assert!(is_streamable(
            "# <pecofence.type>streamable</pecofence.type>"
        ));
        assert!(!is_streamable("# a plain script"));
    }
}
