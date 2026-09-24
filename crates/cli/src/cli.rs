//! The clap command tree. Help text is read by agents over and over, so it stays terse: one
//! line per option, one example per leaf.

use clap::{ArgGroup, Args, Parser, Subcommand};
use pecofence_ipc::{DEFAULT_TIMEOUT_MS, Rect};

const ABOUT: &str = "Control a running PecoFence (Windows desktop fences) from the terminal.\n\
Every command prints JSON (pretty on a terminal, one line otherwise); errors go to stderr as JSON.\n\
Agents: start with `pecofence-cli describe` (command catalog) and `pecofence-cli skill` (usage guide).";

const FENCE_HELP: &str =
    "Fence: id, unique id prefix (>=6 hex), or title (exact, then unique substring)";
const RULE_HELP: &str = "Rule: id, 0-based index, or name";
const RECT_HELP: &str = "x,y,w,h in physical pixels, virtual-screen coordinates";

#[derive(Parser, Debug)]
#[command(name = "pecofence-cli", version, about = ABOUT, long_about = None)]
#[command(propagate_version = true, disable_help_subcommand = true)]
pub struct Cli {
    /// Talk to a named test instance (PECOFENCE_INSTANCE)
    #[arg(long, global = true, value_name = "NAME")]
    pub instance: Option<String>,
    /// Milliseconds to wait for the app's reply
    #[arg(long, global = true, value_name = "MS", default_value_t = DEFAULT_TIMEOUT_MS)]
    pub timeout: u32,
    /// Indent the JSON output (default when stdout is a terminal)
    #[arg(long, global = true, conflicts_with = "compact")]
    pub pretty: bool,
    /// Single-line JSON output (default when stdout is not a terminal)
    #[arg(long, global = true)]
    pub compact: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// App version, pid, config path and counters (also the "is it running?" probe)
    #[command(after_help = "Example: pecofence-cli status")]
    Status,
    /// Monitors
    Monitor {
        #[command(subcommand)]
        cmd: MonitorCmd,
    },
    /// Machine-readable catalog of commands, exit codes and JSON Schemas (works offline)
    #[command(after_help = "Example: pecofence-cli describe --schema Settings")]
    Describe {
        /// Print the JSON Schema of one type instead (Settings, FenceDto, Cond, ...)
        #[arg(long, value_name = "NAME")]
        schema: Option<String>,
    },
    /// Print the bundled SKILL.md for coding agents (Claude Code, Codex)
    #[command(after_help = "Example: pecofence-cli skill > .claude/skills/pecofence-cli/SKILL.md")]
    Skill,
    /// Fences (virtual fences, folder portals, tabs)
    Fence {
        #[command(subcommand)]
        cmd: FenceCmd,
    },
    /// Desktop items (icons) inside fences
    Item {
        #[command(subcommand)]
        cmd: ItemCmd,
    },
    /// Global settings (see `describe --schema Settings` for the keys)
    Settings {
        #[command(subcommand)]
        cmd: SettingsCmd,
    },
    /// Auto-sorting rules
    Rule {
        #[command(subcommand)]
        cmd: RuleCmd,
    },
    /// Layout snapshots
    Snapshot {
        #[command(subcommand)]
        cmd: SnapshotCmd,
    },
    /// Configuration file export / import
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Daily configuration backups the app keeps
    Backup {
        #[command(subcommand)]
        cmd: BackupCmd,
    },
    /// Peek (float every fence above other windows)
    Peek {
        #[command(subcommand)]
        cmd: PeekCmd,
    },
    /// Stream desktop events as JSON lines (item added/removed/moved, fence created/deleted/changed) until Ctrl+C
    #[command(
        after_help = "Example: pecofence-cli watch --fence inbox --once\n         pecofence-cli watch --events item.added,item.moved\n\nEach line is an EventDto (`describe --schema EventDto`); a heartbeat arrives every 30 s on a quiet\nstream and is hidden unless --heartbeat. Typical use: a script waits with --once, then runs a batch."
    )]
    Watch {
        /// Only items entering or leaving this fence, and that fence's own events
        #[arg(long, value_name = "FENCE", help = FENCE_HELP)]
        fence: Option<String>,
        /// Event kinds, comma-separated: item.added,item.removed,item.moved,fence.created,fence.deleted,fence.changed
        #[arg(long, value_name = "EVENT,...", value_delimiter = ',')]
        events: Vec<String>,
        /// Exit after the first event (printed as the result)
        #[arg(long)]
        once: bool,
        /// Print heartbeat lines too
        #[arg(long)]
        heartbeat: bool,
    },
    /// Print the app's log file as text (not JSON); -f keeps following it until Ctrl+C
    #[command(
        after_help = "Example: pecofence-cli log -n 100\n         pecofence-cli log --follow"
    )]
    Log {
        /// Keep printing new lines as they are written
        #[arg(long, short = 'f')]
        follow: bool,
        /// Number of trailing lines to print first (0 = whole file)
        #[arg(long, short = 'n', value_name = "N", default_value_t = 50)]
        lines: usize,
    },
    /// Where PecoFence keeps its files: config, backups, log, crash dumps (works offline)
    #[command(after_help = "Example: pecofence-cli paths")]
    Paths,
}

#[derive(Subcommand, Debug)]
pub enum MonitorCmd {
    /// Connected monitors with id, rect, work area, DPI
    #[command(after_help = "Example: pecofence-cli monitor list")]
    List,
}

#[derive(Subcommand, Debug)]
pub enum FenceCmd {
    /// Fences of the active layout with geometry and options
    #[command(after_help = "Example: pecofence-cli fence list")]
    List,
    /// One fence
    #[command(after_help = "Example: pecofence-cli fence get Work")]
    Get {
        #[arg(help = FENCE_HELP)]
        fence: String,
    },
    /// New virtual fence, or a folder portal with --portal
    #[command(
        after_help = "Example: pecofence-cli fence create --title Work --rect 100,100,600,400"
    )]
    Create {
        /// Title of the new fence (defaults to the folder name with --portal)
        #[arg(long, value_name = "TITLE", required_unless_present = "portal")]
        title: Option<String>,
        #[arg(long, value_name = "X,Y,W,H", value_parser = Rect::parse, allow_hyphen_values = true, help = RECT_HELP)]
        rect: Option<Rect>,
        /// Monitor id (see `monitor list`) to place the fence on when --rect is omitted
        #[arg(long, value_name = "ID", conflicts_with = "rect")]
        monitor: Option<String>,
        /// Folder path: show this folder as a portal instead of a virtual fence
        #[arg(long, value_name = "DIR")]
        portal: Option<String>,
    },
    /// Delete a fence; its items return to the desktop (inbox) fence, rules targeting it are removed
    #[command(after_help = "Example: pecofence-cli fence delete Temp")]
    Delete {
        #[arg(help = FENCE_HELP)]
        fence: String,
    },
    /// Rename a fence
    #[command(after_help = "Example: pecofence-cli fence rename Work Projects")]
    Rename {
        #[arg(help = FENCE_HELP)]
        fence: String,
        /// New title
        title: String,
    },
    /// Move a fence: absolute --rect, new --x/--y, or another --monitor
    #[command(after_help = "Example: pecofence-cli fence move Work --x 1200 --y 80")]
    #[command(group(ArgGroup::new("where").required(true).multiple(true)))]
    Move {
        #[arg(help = FENCE_HELP)]
        fence: String,
        #[arg(long, value_name = "X,Y,W,H", value_parser = Rect::parse, allow_hyphen_values = true, group = "where", conflicts_with_all = ["x", "y", "monitor"], help = RECT_HELP)]
        rect: Option<Rect>,
        /// New left edge (physical px, virtual screen); keeps size
        #[arg(
            long,
            group = "where",
            conflicts_with = "monitor",
            allow_negative_numbers = true
        )]
        x: Option<i32>,
        /// New top edge (physical px, virtual screen); keeps size
        #[arg(
            long,
            group = "where",
            conflicts_with = "monitor",
            allow_negative_numbers = true
        )]
        y: Option<i32>,
        /// Move to this monitor id, keeping the relative position
        #[arg(long, value_name = "ID", group = "where")]
        monitor: Option<String>,
    },
    /// Resize a fence (physical px); position is kept
    #[command(after_help = "Example: pecofence-cli fence resize Work --w 800")]
    #[command(group(ArgGroup::new("size").required(true).multiple(true)))]
    Resize {
        #[arg(help = FENCE_HELP)]
        fence: String,
        /// New width
        #[arg(long, group = "size")]
        w: Option<i32>,
        /// New height
        #[arg(long, group = "size")]
        h: Option<i32>,
    },
    /// Set a per-fence option (title, iconSize, spacing, autoHeight, locked, excludeFromQuickHide, opacity, tint, titleColor, titleSize, layout, sort, reverse, groupByDate, labelLines, portalNavigate, portalTitleIcon); `fence rename` is the friendlier way to set title
    #[command(
        override_usage = "pecofence-cli fence set [OPTIONS] <FENCE> <PROP> <VALUE>\n       pecofence-cli fence set [OPTIONS] --all <PROP> <VALUE>",
        after_help = "Arguments:\n  <FENCE>  Fence: id, unique id prefix (>=6 hex), or title (exact, then unique substring)\n  <PROP>   Option name as shown by `fence get` (title: prefer `fence rename`)\n  <VALUE>  JSON value or bare text: 48, true, list, null; quote # colours: \"#ff8800\" or '\"#ff8800\"'\n\nExample: pecofence-cli fence set Work layout list\n         pecofence-cli fence set Work tint \"#ff8800\"\n         pecofence-cli fence set --all opacity clear"
    )]
    Set {
        /// FENCE PROP VALUE, or PROP VALUE with --all
        #[arg(value_name = "ARGS", num_args = 2..=3, required = true, hide = true, allow_negative_numbers = true)]
        args: Vec<String>,
        /// Apply to every fence (one call per fence; hosted tabs are skipped)
        #[arg(long)]
        all: bool,
        /// Treat VALUE as a string even if it parses as JSON (e.g. title 2024)
        #[arg(long)]
        string: bool,
    },
    /// Roll a fence up to its title bar
    #[command(after_help = "Example: pecofence-cli fence roll Work")]
    Roll {
        #[arg(help = FENCE_HELP, required_unless_present = "all", conflicts_with = "all")]
        fence: Option<String>,
        /// Every fence
        #[arg(long)]
        all: bool,
    },
    /// Expand a rolled-up fence
    #[command(after_help = "Example: pecofence-cli fence unroll --all")]
    Unroll {
        #[arg(help = FENCE_HELP, required_unless_present = "all", conflicts_with = "all")]
        fence: Option<String>,
        /// Every fence
        #[arg(long)]
        all: bool,
    },
    /// Dock a fence to the top edge of its monitor (rolled up, expands on hover)
    #[command(after_help = "Example: pecofence-cli fence dock-top Work")]
    DockTop {
        #[arg(help = FENCE_HELP)]
        fence: String,
    },
    /// Merge a fence into another one as a tab
    #[command(after_help = "Example: pecofence-cli fence merge Games --into Work")]
    Merge {
        #[arg(help = FENCE_HELP)]
        fence: String,
        /// Host fence that receives the tab
        #[arg(long, value_name = "FENCE")]
        into: String,
    },
    /// Split a tab out into its own window
    #[command(after_help = "Example: pecofence-cli fence detach Games")]
    Detach {
        /// Tab (fence selector)
        #[arg(help = FENCE_HELP)]
        fence: String,
    },
    /// Quick-hide every fence
    #[command(after_help = "Example: pecofence-cli fence hide-all")]
    HideAll,
    /// Show every fence again
    #[command(after_help = "Example: pecofence-cli fence show-all")]
    ShowAll,
    /// Open the Settings window on this fence's options page
    #[command(after_help = "Example: pecofence-cli fence open-options Work")]
    OpenOptions {
        #[arg(help = FENCE_HELP)]
        fence: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ItemCmd {
    /// Items of every fence, or of one fence, with kind, ext, size, modified, created, openCount, lastOpened, shortcutTarget
    #[command(
        after_help = "Example: pecofence-cli item list --fence Work\n         pecofence-cli item list --fence inbox --kind documents --ext pdf"
    )]
    List {
        #[arg(long, value_name = "FENCE", help = FENCE_HELP)]
        fence: Option<String>,
        /// Only items of this kind: folders, programs, installers, shortcuts, documents, images, music, video, archives, namespace, other
        #[arg(long, value_name = "KIND")]
        kind: Option<String>,
        /// Only items with this extension (pdf or .pdf; case-insensitive)
        #[arg(long, value_name = "EXT")]
        ext: Option<String>,
    },
    /// Rename the file behind an item (a real rename on disk, like F2)
    #[command(
        after_help = "Example: pecofence-cli item rename \"C:\\Users\\me\\Desktop\\IMG_2031.pdf\" \"2026-09 electricity bill\"\n         pecofence-cli item rename notes notes.md --keep-ext false\n\nThe current extension is kept unless NAME already ends with it; --keep-ext false renames verbatim."
    )]
    Rename {
        /// Item id, full path, or unique display name
        item: String,
        /// New name (extension added automatically unless --keep-ext false)
        name: String,
        /// Keep the current extension
        #[arg(long, value_name = "BOOL", default_value_t = true, action = clap::ArgAction::Set)]
        keep_ext: bool,
    },
    /// Move items into a fence (into or out of a folder portal moves the real files)
    #[command(
        after_help = "Example: pecofence-cli item move --glob \"*.pdf\" --to Docs\n         pecofence-cli item move \"C:\\Users\\me\\Desktop\\report.pdf\" Steam --to Work"
    )]
    #[command(group(ArgGroup::new("which").required(true)))]
    Move {
        /// Item id, full path, or unique display name
        #[arg(value_name = "ITEM", group = "which", num_args = 1..)]
        items: Vec<String>,
        /// Select items whose file name matches this pattern (* and ?, case-insensitive)
        #[arg(long, value_name = "PATTERN", group = "which")]
        glob: Option<String>,
        /// Only consider items currently in this fence (with --glob)
        #[arg(long, value_name = "FENCE", conflicts_with = "items")]
        from: Option<String>,
        /// Destination fence
        #[arg(
            long,
            value_name = "FENCE",
            help = "Destination fence (id, id prefix, or title)"
        )]
        to: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SettingsCmd {
    /// The whole Settings object, or one value at a dotted camelCase path
    #[command(after_help = "Example: pecofence-cli settings get peek.enabled")]
    Get {
        /// Dotted path such as peek.enabled, quickHide.delayMs, iconSize
        path: Option<String>,
    },
    /// Set one value; the app applies it immediately
    #[command(after_help = "Example: pecofence-cli settings set peek.enabled false")]
    Set {
        /// Dotted camelCase path (see `describe --schema Settings`)
        path: String,
        /// JSON value or bare text: 48, true, dark, null, -4
        #[arg(allow_negative_numbers = true)]
        value: String,
        /// Treat VALUE as a string even if it parses as JSON
        #[arg(long)]
        string: bool,
    },
    /// Open the Settings window
    #[command(after_help = "Example: pecofence-cli settings open-ui")]
    OpenUi,
}

#[derive(Subcommand, Debug)]
pub enum RuleCmd {
    /// Rules in evaluation order, with defaultTarget and keepUpdated
    #[command(after_help = "Example: pecofence-cli rule list")]
    List,
    /// Add a rule; every given condition must match (AND)
    #[command(after_help = "Example: pecofence-cli rule add --name PDFs --ext pdf --to Docs")]
    Add(Box<RuleAddArgs>),
    /// Remove a rule
    #[command(after_help = "Example: pecofence-cli rule remove PDFs")]
    Remove {
        #[arg(help = RULE_HELP)]
        rule: String,
    },
    /// Enable a rule
    #[command(after_help = "Example: pecofence-cli rule enable 0")]
    Enable {
        #[arg(help = RULE_HELP)]
        rule: String,
    },
    /// Disable a rule (kept in the list)
    #[command(after_help = "Example: pecofence-cli rule disable PDFs")]
    Disable {
        #[arg(help = RULE_HELP)]
        rule: String,
    },
    /// Reorder: place a rule at a 0-based index
    #[command(after_help = "Example: pecofence-cli rule move PDFs --to 0")]
    Move {
        #[arg(help = RULE_HELP)]
        rule: String,
        /// New 0-based position
        #[arg(long, value_name = "INDEX")]
        to: usize,
    },
    /// Replace the whole rule set from a RuleSet JSON file (or stdin with -)
    #[command(
        after_help = "Example: pecofence-cli rule list | jq '.keepUpdated=false' | pecofence-cli rule import -"
    )]
    Import {
        /// Path to a RuleSet JSON file, or - for stdin
        file: String,
    },
    /// Re-file every desktop item through the rules now; --dry-run only lists what would move
    #[command(
        after_help = "Example: pecofence-cli rule apply --dry-run\n         pecofence-cli rule apply"
    )]
    Apply {
        /// Report the moves without making them (no snapshot, nothing changes)
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Args, Debug)]
#[command(group(ArgGroup::new("cond").required(true).multiple(true)))]
pub struct RuleAddArgs {
    /// Rule name
    #[arg(long, value_name = "NAME")]
    pub name: String,
    /// Target fence selector, or `inbox` for the desktop fence
    #[arg(long, value_name = "FENCE|inbox")]
    pub to: String,
    /// Extensions, comma-separated (pdf,docx or .pdf,.docx)
    #[arg(long, value_name = "EXT,...", value_delimiter = ',', group = "cond")]
    pub ext: Vec<String>,
    /// Type categories, comma-separated: programs,folders,documents,images,music,video,archives,shortcuts,installers
    #[arg(
        long = "type",
        value_name = "TYPE,...",
        value_delimiter = ',',
        group = "cond"
    )]
    pub types: Vec<String>,
    /// File name contains this text (case-insensitive)
    #[arg(long, value_name = "TEXT", group = "cond")]
    pub name_contains: Option<String>,
    /// File name does not contain this text
    #[arg(long, value_name = "TEXT", group = "cond")]
    pub name_not_contains: Option<String>,
    /// File name starts with this text
    #[arg(long, value_name = "TEXT", group = "cond")]
    pub starts_with: Option<String>,
    /// File name ends with this text
    #[arg(long, value_name = "TEXT", group = "cond")]
    pub ends_with: Option<String>,
    /// File name (with extension) equals this text
    #[arg(long, value_name = "TEXT", group = "cond")]
    pub name_is: Option<String>,
    /// File name matches this pattern (* and ?)
    #[arg(long, value_name = "PATTERN", group = "cond")]
    pub glob: Option<String>,
    /// Only folders
    #[arg(long, group = "cond", conflicts_with = "files_only")]
    pub folders_only: bool,
    /// Only files
    #[arg(long, group = "cond")]
    pub files_only: bool,
    /// Extra conditions as a JSON array of Cond objects (see `describe --schema Cond`)
    #[arg(long, value_name = "COND-ARRAY-JSON", group = "cond")]
    pub json: Option<String>,
    /// Insert at this 0-based position instead of appending
    #[arg(long, value_name = "N")]
    pub index: Option<usize>,
}

#[derive(Subcommand, Debug)]
pub enum SnapshotCmd {
    /// Saved snapshots (newest first)
    #[command(after_help = "Example: pecofence-cli snapshot list")]
    List,
    /// Save the current layout as a snapshot
    #[command(after_help = "Example: pecofence-cli snapshot save before-cleanup")]
    Save {
        /// Snapshot name
        name: String,
    },
    /// Restore a snapshot (the current layout is snapshotted first)
    #[command(after_help = "Example: pecofence-cli snapshot restore 3f9c2a1e
         (snapshot.id or a unique id prefix from `snapshot save` / `snapshot list`; a name works only while it is unique)")]
    Restore {
        /// Snapshot id, unique id prefix, or (unique) name
        id: String,
    },
    /// Delete a snapshot
    #[command(after_help = "Example: pecofence-cli snapshot delete 3f9c2a1e")]
    Delete {
        /// Snapshot id, unique id prefix, or (unique) name
        id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCmd {
    /// Write the complete configuration (settings, rules, layouts, snapshots) to a JSON file
    #[command(
        after_help = "Example: pecofence-cli config export C:\\Users\\me\\Desktop\\pecofence.json
         (absolute path; an existing file is overwritten)"
    )]
    Export {
        /// Absolute path of the file to write
        file: String,
    },
    /// Replace the configuration with a JSON file (the current layout is snapshotted first)
    #[command(
        after_help = "Example: pecofence-cli config import C:\\Users\\me\\Desktop\\pecofence.json"
    )]
    Import {
        /// Absolute path of a `config export` file or a config.json
        file: String,
    },
    /// Validate a config.json or export file without the app: parse, then cross-check rule targets, portal folders, tabs, $schema
    #[command(
        after_help = "Example: pecofence-cli config check C:\\Users\\me\\Desktop\\pecofence.json\n         pecofence-cli config check            (the running or default config.json)\n\nExit 0 = loads and no errors (warnings may be listed); exit 1 = the app would reject it or a problem was found."
    )]
    Check {
        /// File to check (default: the config.json in use, see `paths`)
        file: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum BackupCmd {
    /// Daily backups beside config.json, newest first
    #[command(after_help = "Example: pecofence-cli backup list")]
    List,
    /// Restore one of the listed backups (the current layout is snapshotted first)
    #[command(
        after_help = "Example: pecofence-cli backup restore \"C:\\Users\\me\\AppData\\Roaming\\PecoFence\\backups\\2026-09-23.json\"
         (a path exactly as printed by `backup list`)"
    )]
    Restore {
        /// A path from `backup list`
        file: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PeekCmd {
    /// Float every fence above the current windows
    #[command(after_help = "Example: pecofence-cli peek start")]
    Start,
    /// End Peek
    #[command(after_help = "Example: pecofence-cli peek end")]
    End,
}

/// `fence set` positionals: `FENCE PROP VALUE`, or `PROP VALUE` together with `--all`.
pub fn split_set_args(args: &[String], all: bool) -> Result<(Option<&str>, &str, &str), String> {
    match (args, all) {
        ([prop, value], true) => Ok((None, prop, value)),
        ([fence, prop, value], false) => Ok((Some(fence), prop, value)),
        ([_, _], false) => Err(
            "missing FENCE: pass a fence selector or --all; values starting with # or containing spaces must be quoted, e.g. tint '\"#ff8800\"' or tint \"#ff8800\""
                .into(),
        ),
        ([_, _, _], true) => Err("--all cannot be combined with a FENCE".into()),
        _ => Err("expected FENCE PROP VALUE".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn command_tree_is_well_formed() {
        Cli::command().debug_assert();
    }

    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("pecofence-cli").chain(args.iter().copied()))
    }

    #[test]
    fn rect_flag_uses_rect_parse() {
        let cli = parse(&[
            "fence",
            "create",
            "--title",
            "Work",
            "--rect",
            "100,100,600,400",
        ])
        .unwrap();
        match cli.command {
            Command::Fence {
                cmd: FenceCmd::Create { rect: Some(r), .. },
            } => assert_eq!((r.x, r.y, r.w, r.h), (100, 100, 600, 400)),
            other => panic!("{other:?}"),
        }
        assert!(parse(&["fence", "create", "--title", "W", "--rect", "1,2,3"]).is_err());
        assert!(parse(&["fence", "create", "--title", "W", "--rect", "1,2,0,4"]).is_err());
    }

    #[test]
    fn fence_set_takes_fence_prop_value_or_all_prop_value() {
        let cli = parse(&["fence", "set", "--all", "iconSize", "48"]).unwrap();
        match cli.command {
            Command::Fence {
                cmd: FenceCmd::Set { args, all, .. },
            } => {
                assert!(all);
                assert_eq!(
                    split_set_args(&args, all).unwrap(),
                    (None, "iconSize", "48")
                );
            }
            other => panic!("{other:?}"),
        }
        let cli = parse(&["fence", "set", "Work", "layout", "list"]).unwrap();
        match cli.command {
            Command::Fence {
                cmd: FenceCmd::Set { args, all, .. },
            } => {
                assert!(!all);
                assert_eq!(
                    split_set_args(&args, all).unwrap(),
                    (Some("Work"), "layout", "list")
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(parse(&["fence", "set", "iconSize"]).is_err());
        assert!(parse(&["fence", "set", "a", "b", "c", "d"]).is_err());
        let two: Vec<String> = vec!["iconSize".into(), "48".into()];
        let three: Vec<String> = vec!["Work".into(), "iconSize".into(), "48".into()];
        assert!(split_set_args(&two, false).is_err());
        assert!(split_set_args(&three, true).is_err());
        assert!(parse(&["fence", "roll"]).is_err());
        assert!(parse(&["fence", "roll", "--all"]).is_ok());
        assert!(parse(&["fence", "roll", "Work", "--all"]).is_err());
    }

    #[test]
    fn missing_fence_error_explains_shell_quoting() {
        // Git Bash turned `tint #ff8800` into `tint` alone; the error must say why.
        let two: Vec<String> = vec!["tint".into(), "titleColor".into()];
        let msg = split_set_args(&two, false).unwrap_err();
        assert!(msg.starts_with("missing FENCE"), "{msg}");
        assert!(msg.contains("must be quoted"), "{msg}");
        assert!(msg.contains(r##"'"#ff8800"'"##), "{msg}");
        assert!(msg.contains(r##""#ff8800""##), "{msg}");
    }

    #[test]
    fn negative_values_and_string_flag_are_accepted() {
        let cli = parse(&["settings", "set", "snapping.gapPx", "-4"]).unwrap();
        match cli.command {
            Command::Settings {
                cmd:
                    SettingsCmd::Set {
                        path,
                        value,
                        string,
                    },
            } => {
                assert_eq!(
                    (path.as_str(), value.as_str(), string),
                    ("snapping.gapPx", "-4", false)
                );
            }
            other => panic!("{other:?}"),
        }
        let cli = parse(&["fence", "set", "Work", "labelLines", "-1"]).unwrap();
        match cli.command {
            Command::Fence {
                cmd: FenceCmd::Set { args, all, string },
            } => {
                assert_eq!(args, vec!["Work", "labelLines", "-1"]);
                assert!(!all && !string);
            }
            other => panic!("{other:?}"),
        }
        let cli = parse(&["fence", "set", "Work", "title", "2024", "--string"]).unwrap();
        match cli.command {
            Command::Fence {
                cmd: FenceCmd::Set { string, .. },
            } => assert!(string),
            other => panic!("{other:?}"),
        }
        assert!(parse(&["fence", "set", "--string", "--all", "title", "x"]).is_ok());
    }

    #[test]
    fn portal_create_does_not_need_a_title() {
        let cli = parse(&["fence", "create", "--portal", "C:\\Users\\me\\Downloads"]).unwrap();
        match cli.command {
            Command::Fence {
                cmd: FenceCmd::Create { title, portal, .. },
            } => {
                assert_eq!(title, None);
                assert_eq!(portal.as_deref(), Some("C:\\Users\\me\\Downloads"));
            }
            other => panic!("{other:?}"),
        }
        assert!(parse(&["fence", "create", "--rect", "1,2,3,4"]).is_err());
        assert!(parse(&["fence", "create", "--title", "Dl", "--portal", "D:\\x"]).is_ok());
    }

    #[test]
    fn fence_move_needs_one_way_of_saying_where() {
        assert!(parse(&["fence", "move", "Work"]).is_err());
        assert!(parse(&["fence", "move", "Work", "--x", "10"]).is_ok());
        assert!(parse(&["fence", "move", "Work", "--x", "10", "--y", "-20"]).is_ok());
        assert!(parse(&["fence", "move", "Work", "--rect", "-100,0,300,200"]).is_ok());
        assert!(parse(&["fence", "move", "Work", "--rect", "1,2,3,4", "--x", "1"]).is_err());
        assert!(parse(&["fence", "move", "Work", "--monitor", "m", "--x", "1"]).is_err());
        assert!(parse(&["fence", "resize", "Work"]).is_err());
        assert!(parse(&["fence", "resize", "Work", "--h", "300"]).is_ok());
    }

    #[test]
    fn new_leaves_parse() {
        assert!(parse(&["watch"]).is_ok());
        assert!(parse(&["watch", "--fence", "inbox", "--once", "--heartbeat"]).is_ok());
        let cli = parse(&["watch", "--events", "item.added,item.moved"]).unwrap();
        match cli.command {
            Command::Watch { events, .. } => assert_eq!(events, vec!["item.added", "item.moved"]),
            other => panic!("{other:?}"),
        }
        assert!(parse(&["log"]).is_ok());
        assert!(parse(&["log", "-f", "-n", "10"]).is_ok());
        assert!(parse(&["paths"]).is_ok());
        assert!(parse(&["config", "check"]).is_ok());
        assert!(parse(&["config", "check", "C:\\x.json"]).is_ok());
        assert!(parse(&["rule", "apply", "--dry-run"]).is_ok());
        let cli = parse(&["item", "rename", "a", "b"]).unwrap();
        match cli.command {
            Command::Item {
                cmd: ItemCmd::Rename { keep_ext, .. },
            } => assert!(keep_ext),
            other => panic!("{other:?}"),
        }
        let cli = parse(&["item", "rename", "a", "b.txt", "--keep-ext", "false"]).unwrap();
        match cli.command {
            Command::Item {
                cmd: ItemCmd::Rename { keep_ext, .. },
            } => assert!(!keep_ext),
            other => panic!("{other:?}"),
        }
        assert!(parse(&["item", "list", "--kind", "documents", "--ext", "pdf"]).is_ok());
    }

    #[test]
    fn item_move_takes_items_or_a_glob() {
        assert!(parse(&["item", "move", "--to", "Work"]).is_err());
        assert!(parse(&["item", "move", "a", "b", "--to", "Work"]).is_ok());
        assert!(parse(&["item", "move", "--glob", "*.pdf", "--to", "Work"]).is_ok());
        assert!(
            parse(&[
                "item", "move", "--glob", "*.pdf", "--from", "Inbox", "--to", "Work"
            ])
            .is_ok()
        );
        assert!(parse(&["item", "move", "a", "--glob", "*.pdf", "--to", "Work"]).is_err());
        assert!(parse(&["item", "move", "a", "--from", "Inbox", "--to", "Work"]).is_err());
    }

    #[test]
    fn rule_add_requires_a_condition() {
        assert!(parse(&["rule", "add", "--name", "x", "--to", "Docs"]).is_err());
        assert!(
            parse(&[
                "rule", "add", "--name", "x", "--to", "Docs", "--ext", "pdf,docx"
            ])
            .is_ok()
        );
        assert!(
            parse(&[
                "rule",
                "add",
                "--name",
                "x",
                "--to",
                "Docs",
                "--folders-only",
                "--files-only"
            ])
            .is_err()
        );
        let cli = parse(&[
            "rule",
            "add",
            "--name",
            "x",
            "--to",
            "Docs",
            "--type",
            "images,Video",
        ])
        .unwrap();
        match cli.command {
            Command::Rule {
                cmd: RuleCmd::Add(a),
            } => assert_eq!(a.types, vec!["images", "Video"]),
            other => panic!("{other:?}"),
        }
    }
}
