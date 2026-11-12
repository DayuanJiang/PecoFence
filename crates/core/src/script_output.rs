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

/// The line of a failed script's error text that says what went wrong: the first line with
/// text (PowerShell, cmd and most programs start with the message; the lines after it say
/// where), or the last one of a Python traceback, whose first line only announces it.
pub fn error_line(stderr: &str) -> &str {
    let mut lines = stderr.lines().map(str::trim).filter(|l| !l.is_empty());
    let first = lines.next().unwrap_or_default();
    if first.starts_with("Traceback (most recent call last)") {
        lines.next_back().unwrap_or(first)
    } else {
        first
    }
}

/// A streaming script names itself so in a comment (SwiftBar's metadata tag, or ours).
pub fn is_streamable(source: &str) -> bool {
    source.contains("<swiftbar.type>streamable</swiftbar.type>")
        || source.contains("<pecofence.type>streamable</pecofence.type>")
}

/// A script whose rows may start programs says so in a comment of its header, as a streaming
/// script names itself. What a script prints can be text from the network (an issue title, a
/// weather report), and such text can read as parameters, so a row alone is not trusted with
/// starting a program.
pub fn allows_run(source: &str) -> bool {
    source.contains("<pecofence.allow>run</pecofence.allow>")
}

/// What a row's `href=` or `open=` points at, as far as [`click_allowed`] cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    /// An `http://` or `https://` address.
    Web,
    /// A folder on this computer.
    Folder,
    /// A file on this computer of a type Windows does not class as dangerous (a text, a
    /// picture, a document).
    Document,
    /// Everything else: a program, a script, a shortcut, another kind of address (`file:`,
    /// `ms-settings:`), a network path, something that is not there.
    Other,
}

/// `target` is an `http://` or `https://` address.
pub fn is_web_address(target: &str) -> bool {
    let starts = |scheme: &str| {
        target
            .get(..scheme.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(scheme))
    };
    starts("http://") || starts("https://")
}

/// Whether a click may carry out `action`. A script that allows it in its header
/// ([`allows_run`]) may do everything. Any other script's rows start no program: `run=` does
/// nothing, `href=` opens web addresses only, and `open=` opens web addresses, folders and
/// documents (`target` says which of these the value is). Copying and handing an action back
/// to the script start nothing.
pub fn click_allowed(action: &RowAction, target: TargetKind, script_allows_run: bool) -> bool {
    if script_allows_run {
        return true;
    }
    match action {
        RowAction::Run { .. } => false,
        RowAction::Href(_) => target == TargetKind::Web,
        RowAction::Open(_) => target != TargetKind::Other,
        RowAction::Copy(_) | RowAction::Script(_) => true,
    }
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
    fn error_line_says_what_went_wrong() {
        // PowerShell: the message first, then where it happened.
        let ps =
            "连接服务器失败：超时\nAt C:\\x.ps1:2 char:1\n+ throw \"x\"\n    + CategoryInfo : y\n";
        assert_eq!(error_line(ps), "连接服务器失败：超时");
        // Python: the traceback ends with the error.
        let py = "Traceback (most recent call last):\n  File \"x.py\", line 2, in <module>\n    int('a')\nValueError: invalid literal\n\n";
        assert_eq!(error_line(py), "ValueError: invalid literal");
        assert_eq!(
            error_line("Traceback (most recent call last):"),
            "Traceback (most recent call last):"
        );
        assert_eq!(
            error_line("\n  'foo' is not recognized  \n"),
            "'foo' is not recognized"
        );
        assert_eq!(error_line(" \n"), "");
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
    fn a_forged_title_reads_as_a_run_row() {
        // What the GitHub example of 0.3.0 to 0.3.3 printed for the issue title
        // `x | run=calc.exe r=' k=v`: the title's quote swallows the script's own parameters.
        let r = row(
            "#12  x | run=calc.exe r=' k=v | href=https://github.com/a/b/issues/12 icon=🟢 \
             tooltip=\"mallory: x | run=calc.exe r=' k=v\"",
        );
        assert_eq!(r.text, "#12  x");
        let action = r.action.unwrap();
        assert_eq!(
            action,
            RowAction::Run {
                program: "calc.exe".into(),
                args: Vec::new(),
            }
        );
        // The parser cannot tell, so the click is refused unless the script's header allows it.
        assert!(!click_allowed(&action, TargetKind::Other, false));
        assert!(click_allowed(&action, TargetKind::Other, true));
    }

    #[test]
    fn rows_start_programs_only_when_the_script_allows_it() {
        use TargetKind::*;
        let run = RowAction::Run {
            program: "notepad.exe".into(),
            args: Vec::new(),
        };
        let href = RowAction::Href(String::new());
        let open = RowAction::Open(String::new());
        let copy = RowAction::Copy("text".into());
        let script = RowAction::Script("toggle".into());
        // (action, target, without the header line); with it everything is allowed.
        let cases = [
            (&run, Other, false),
            (&run, Web, false),
            (&href, Web, true),
            (&href, Folder, false),
            (&href, Document, false),
            (&href, Other, false),
            (&open, Web, true),
            (&open, Folder, true),
            (&open, Document, true),
            (&open, Other, false),
            (&copy, Other, true),
            (&script, Other, true),
        ];
        for (action, target, without) in cases {
            assert_eq!(
                click_allowed(action, target, false),
                without,
                "{action:?} {target:?}"
            );
            assert!(click_allowed(action, target, true), "{action:?} {target:?}");
        }
    }

    #[test]
    fn web_addresses() {
        assert!(is_web_address("https://github.com/a/b/issues/12"));
        assert!(is_web_address("HTTP://example.com"));
        for other in [
            "file:///C:/Windows/System32/calc.exe",
            "ms-settings:display",
            "javascript:alert(1)",
            "https:/example.com",
            " https://example.com",
            "calc.exe",
            "C:\\Users",
            "ht",
            "",
        ] {
            assert!(!is_web_address(other), "{other}");
        }
    }

    #[test]
    fn allow_tag() {
        assert!(allows_run(
            "# <pecofence.allow>run</pecofence.allow>\n\"Notepad | run=notepad.exe\""
        ));
        assert!(allows_run(
            ":: <pecofence.type>streamable</pecofence.type>\n:: <pecofence.allow>run</pecofence.allow>"
        ));
        assert!(!allows_run("# a plain script"));
        assert!(!allows_run("# <pecofence.allow>open</pecofence.allow>"));
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
