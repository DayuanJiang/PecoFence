//! 脚本栅栏: what a script prints, line by line, in SwiftBar's format. A line is
//! `text | key=value key="value with spaces"`; the parameters make the row clickable (open a
//! URL, a file, run a program, copy text, hand an action back to the script, run it again)
//! and set its colour, icon and tooltip. `---` is a separator. A row whose text starts with
//! `[ ]` or `[x]` is drawn as a to-do. A line without parameters is plain text, so scripts
//! written before this format show as they did.
//!
//! A line that starts with `::` is a row of cells side by side, `:: cell :: cell :: cell`, each
//! cell written like a row (`text | key=value`): a cell with an action is a button (a glyph
//! alone makes an icon button, `tooltip=` names it), a cell with `slider=0.4` is a horizontal
//! slider that takes the rest of the width, any other cell is text. A click or drag on a slider
//! hands the script its action with the position, `action=seek` giving `seek:0.37`.

use crate::note::{NoteLine, parse_line};

/// One line of the output.
// A few dozen lines at most are held at a time: no need to box the row.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptLine {
    Separator,
    Row(ScriptRow),
    /// `:: cell :: cell`: rows side by side (buttons, sliders, text).
    Cells(Vec<ScriptRow>),
}

#[derive(Clone, Debug, Default, PartialEq)]
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
    /// `slider=0.4` (0 to 1): the cell is a slider at this position. A click or drag on it runs
    /// its `action=` with `:position` added.
    pub slider: Option<f32>,
    /// `width=44`: a cell of a row of cells is this many DIPs wide (at the normal text size),
    /// so cells of different rows line up.
    pub width: Option<f32>,
    /// `align=right` (1.0) or `align=center` (0.5): where the text sits in a cell wider than it.
    pub align: f32,
    /// `checked=true`: a button of a row of cells that is on (shuffle on, playing): drawn
    /// pressed in.
    pub checked: bool,
    /// `fill=true`: a cell of a row of cells that takes the width the other cells leave (as a slider
    /// does), so that the cells after it stay against the right edge of the fence.
    pub fill: bool,
    /// `under=true`: a cell of a row of cells that goes under the cell before it, in the same column
    /// (two small buttons one above the other).
    pub under: bool,
    /// `bold=true`: a cell of a row of cells in bold.
    pub bold: bool,
    /// `checkbox=true` or `checkbox=false`: a cell of a row of cells that starts with a check box,
    /// ticked or not, as a to-do has (a click on the cell is a click on the box).
    pub checkbox: Option<bool>,
    /// `gap=4`: the space after a cell of a row of cells, in DIPs at the normal text size, in
    /// place of the usual one between buttons (`gap=0` sets words side by side).
    pub gap: Option<f32>,
    /// `dblaction=name`: a double-click on the row (or cell) hands the script this action, as
    /// `action=` does for a click. The first click of the double-click already sent `action=`.
    pub dbl_action: Option<String>,
    /// `image=C:\\cover.jpg`: a row that shows this picture, centred, at most `height=` DIPs tall
    /// (96 by default) and as wide as the fence.
    pub image: Option<String>,
    /// `height=96`: how tall an `image=` is at most, in DIPs.
    pub height: Option<f32>,
    /// `hidden=true`: the row is not drawn and takes no room; its parameters still count (`pane=` and
    /// `follow=` below).
    pub hidden: bool,
    /// `pane=<key>` (on a `hidden=true` row): the rows above it stay in place and the rows below
    /// it scroll under them. While the key stays the same the scroll position is kept; a new key
    /// (another page of a list, another tab) brings the rows back to their top.
    pub pane: Option<String>,
    /// `follow=<key>` (on a `hidden=true` row): the row right after it is kept in the middle of
    /// the scrolling rows, each time the key changes (the line of a song being sung), unless the
    /// user scrolled by hand a few seconds ago.
    pub follow: Option<String>,
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
    if is_cells_line(line) {
        let cells: Vec<ScriptRow> = split_cells(&line.trim_start()[2..])
            .into_iter()
            .filter(|c| !c.trim().is_empty())
            .map(|c| parse_row(c.trim()))
            .collect();
        return ScriptLine::Cells(cells);
    }
    ScriptLine::Row(parse_row(line))
}

/// A line of cells: `::` alone or followed by a space or tab, so that text such as the IPv6
/// address `::1` stays text.
pub fn is_cells_line(line: &str) -> bool {
    line.trim_start()
        .strip_prefix("::")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '\t']))
}

/// The cells of a `::` line: split at each `::` that is not inside a quoted value (a quote
/// opens right after `=`, so an apostrophe in a text is only a letter).
fn split_cells(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut quote: Option<char> = None;
    let mut prev = ' ';
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None if (c == '"' || c == '\'') && prev == '=' => quote = Some(c),
            None if c == ':' && matches!(chars.peek(), Some((_, ':'))) => {
                out.push(&text[start..at]);
                chars.next();
                start = at + 2;
                prev = ' ';
                continue;
            }
            None => {}
        }
        prev = c;
    }
    out.push(&text[start..]);
    out
}

fn parse_row(line: &str) -> ScriptRow {
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
    ScriptRow {
        text,
        todo,
        color: get("color").filter(|c| !c.is_empty()),
        icon: get("icon").filter(|c| !c.is_empty()),
        // `\n` (a backslash and an n) breaks the line: a script prints one line per row.
        tooltip: get("tooltip")
            .filter(|c| !c.is_empty())
            .map(|t| t.replace("\\n", "\n")),
        action,
        refresh: truthy(get("refresh")),
        slider: get("slider")
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(0.0, 1.0)),
        width: get("width")
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
            .map(|v| v.min(1000.0)),
        checked: truthy(get("checked")),
        fill: truthy(get("fill")),
        under: truthy(get("under")),
        bold: truthy(get("bold")),
        checkbox: get("checkbox").map(|v| v.eq_ignore_ascii_case("true")),
        dbl_action: get("dblaction").filter(|a| !a.is_empty()),
        image: get("image").filter(|p| !p.is_empty()),
        hidden: truthy(get("hidden")),
        pane: get("pane").filter(|k| !k.is_empty()),
        follow: get("follow").filter(|k| !k.is_empty()),
        height: get("height")
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
            .map(|v| v.min(1000.0)),
        gap: get("gap")
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v >= 0.0)
            .map(|v| v.min(1000.0)),
        align: match get("align").map(|a| a.to_ascii_lowercase()).as_deref() {
            Some("right") => 1.0,
            Some("center") => 0.5,
            _ => 0.0,
        },
    }
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

/// The title a script asks for its fence in a comment of its header
/// (`<pecofence.title>Clipboard History</pecofence.title>`): what a fence made from it is called
/// instead of the file name. None for no tag, an empty one or one of more than one line.
pub fn script_title(source: &str) -> Option<String> {
    const OPEN: &str = "<pecofence.title>";
    const CLOSE: &str = "</pecofence.title>";
    let start = source.find(OPEN)? + OPEN.len();
    let len = source[start..].find(CLOSE)?;
    let title = source[start..start + len].trim();
    (!title.is_empty() && !title.contains(['\n', '\r']) && title.chars().count() <= 80)
        .then(|| title.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(line: &str) -> ScriptRow {
        match parse_one(line) {
            ScriptLine::Row(r) => r,
            _ => panic!("not a row"),
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
    fn cells_side_by_side() {
        let ScriptLine::Cells(cells) = parse_one(
            ":: ⏮ | action=prev tooltip=\"Previous track\" :: ▶ | action=play :: 0:42 | color=gray",
        ) else {
            panic!("cells");
        };
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0].text, "⏮");
        assert_eq!(cells[0].tooltip.as_deref(), Some("Previous track"));
        assert_eq!(cells[1].action, Some(RowAction::Script("play".into())));
        assert!(!cells[2].clickable());
        // A `::` inside a quoted value, an apostrophe in a text and empty cells.
        let ScriptLine::Cells(cells) =
            parse_one(":: Don't :: B | tooltip='a :: b' action=x ::  :: ")
        else {
            panic!("cells");
        };
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].text, "Don't");
        assert_eq!(cells[1].tooltip.as_deref(), Some("a :: b"));
        // The line of a text that only holds a colon is no row of cells.
        assert!(matches!(parse_one("a :: b"), ScriptLine::Row(_)));
    }

    #[test]
    fn sliders() {
        let ScriptLine::Cells(cells) =
            parse_one(":: 🔊 | action=mute :: | slider=0.25 action=volume")
        else {
            panic!("cells");
        };
        assert_eq!(cells[0].slider, None);
        assert_eq!(cells[1].slider, Some(0.25));
        assert_eq!(cells[1].action, Some(RowAction::Script("volume".into())));
        assert_eq!(row("x | slider=3").slider, Some(1.0));
        assert_eq!(row("x | slider=-1").slider, Some(0.0));
        assert_eq!(row("x | slider=abc").slider, None);
    }

    #[test]
    fn cell_width_and_alignment() {
        let r = row("27% | width=44 align=right");
        assert_eq!((r.width, r.align), (Some(44.0), 1.0));
        let r = row("x | width=-3 align=center");
        assert_eq!((r.width, r.align), (None, 0.5));
        let r = row("x | align=up");
        assert_eq!((r.width, r.align), (None, 0.0));
        assert!(row("x | checked=true").checked);
        assert!(!row("x | checked=false").checked);
        let r = row("word | bold=true gap=4");
        assert!(r.bold);
        assert_eq!(r.gap, Some(4.0));
        assert_eq!(row("x | gap=-1").gap, None);
        assert!(!row("x").bold);
        assert_eq!(row("Online | checkbox=true").checkbox, Some(true));
        assert_eq!(row("Online | checkbox=false").checkbox, Some(false));
        assert_eq!(row("Online").checkbox, None);
        let r = row("Song | action=sel:3 dblaction=playitem:3");
        assert_eq!(r.action, Some(RowAction::Script("sel:3".into())));
        assert_eq!(r.dbl_action.as_deref(), Some("playitem:3"));
        assert_eq!(row("x | dblaction=").dbl_action, None);
    }

    #[test]
    fn tooltips_break_lines_at_backslash_n() {
        let r = row("x | action=a tooltip=\"Next track\\nSong - Title.mp3\"");
        assert_eq!(r.tooltip.as_deref(), Some("Next track\nSong - Title.mp3"));
        assert_eq!(row("x | tooltip=plain").tooltip.as_deref(), Some("plain"));
    }

    #[test]
    fn rows_that_split_the_pinned_part_from_the_scrolling_one() {
        let r = row(" | pane=playlist:50 hidden=true");
        assert_eq!(r.pane.as_deref(), Some("playlist:50"));
        assert!(r.hidden && r.follow.is_none());
        let r = row(" | follow=12 hidden=true");
        assert_eq!(r.follow.as_deref(), Some("12"));
        assert_eq!(row("x | pane=").pane, None, "an empty key is no key");
        assert_eq!(row("Plain row").pane, None);
    }

    #[test]
    fn images_and_the_prefix_of_cells() {
        let r = row(" | image=\"C:\\My Music\\cover.jpg\" height=120");
        assert_eq!(r.text, "");
        assert_eq!(r.image.as_deref(), Some("C:\\My Music\\cover.jpg"));
        assert_eq!(r.height, Some(120.0));
        assert_eq!(row("x | height=0").height, None);
        assert_eq!(row("x").image, None);
        // Only `::` followed by a space (or alone) makes a row of cells.
        assert!(is_cells_line(":: a | action=x"));
        assert!(is_cells_line("  ::\ta"));
        assert!(!is_cells_line("::1"));
        assert!(!is_cells_line("::ffff:192.168.0.1 | copy=x"));
        assert!(matches!(parse_one("::1  localhost"), ScriptLine::Row(_)));
    }

    #[test]
    fn a_cell_may_hold_a_picture() {
        let ScriptLine::Cells(cells) = parse_one(
            ":: | image=\"C:\\a b\\p.png\" height=80 width=280 action=copy:3 :: X | action=del:3",
        ) else {
            panic!("a row of cells");
        };
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].image.as_deref(), Some("C:\\a b\\p.png"));
        assert_eq!(cells[0].height, Some(80.0));
        assert_eq!(cells[0].width, Some(280.0));
        assert_eq!(cells[0].action, Some(RowAction::Script("copy:3".into())));
        assert_eq!(cells[1].image, None);
    }

    #[test]
    fn cells_that_fill_and_cells_that_go_under() {
        let ScriptLine::Cells(cells) = parse_one(
            ":: Long text | fill=true action=copy:1 :: T | action=del:1 :: P | under=true action=pin:1",
        ) else {
            panic!("a row of cells");
        };
        assert!(cells[0].fill && !cells[0].under);
        assert!(!cells[1].fill && !cells[1].under);
        assert!(cells[2].under && !cells[2].fill);
        assert!(!row("x").fill && !row("x").under);
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
    fn a_script_names_its_fence() {
        assert_eq!(
            script_title("# <pecofence.title>Clipboard History</pecofence.title>\n# more")
                .as_deref(),
            Some("Clipboard History")
        );
        assert_eq!(
            script_title("// <pecofence.title>  Spaces  </pecofence.title>").as_deref(),
            Some("Spaces")
        );
        assert_eq!(script_title("# no tag"), None);
        assert_eq!(script_title("# <pecofence.title></pecofence.title>"), None);
        assert_eq!(script_title("# <pecofence.title>never closed"), None);
        assert_eq!(
            script_title("<pecofence.title>two\nlines</pecofence.title>"),
            None
        );
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
