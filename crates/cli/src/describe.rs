//! `describe`: a self-contained catalog for agents, generated without talking to the app.

use pecofence_ipc::schema::{SCHEMA_NAMES, schema};
use pecofence_ipc::{ErrorCode, IpcError, PROTOCOL_VERSION};
use serde_json::{Value, json};

#[cfg(test)]
use pecofence_ipc::Method;

/// One CLI leaf: command, wire method (`None` for client-only commands), summary, example.
pub struct Entry {
    pub command: &'static str,
    pub method: Option<&'static str>,
    pub summary: &'static str,
    pub example: &'static str,
}

macro_rules! entry {
    ($command:literal, $method:expr, $summary:literal, $example:literal) => {
        Entry {
            command: $command,
            method: $method,
            summary: $summary,
            example: $example,
        }
    };
}

pub const COMMANDS: &[Entry] = &[
    entry!(
        "status",
        Some("status.get"),
        "App version, pid, config path, counters; the is-it-running probe",
        "pecofence-cli status"
    ),
    entry!(
        "monitor list",
        Some("monitors.list"),
        "Connected monitors: id, rect, workArea, dpi, primary",
        "pecofence-cli monitor list"
    ),
    entry!(
        "describe",
        None,
        "This catalog; --schema <Name> prints a JSON Schema (offline)",
        "pecofence-cli describe --schema Settings"
    ),
    entry!(
        "skill",
        None,
        "Print the bundled SKILL.md for coding agents (offline)",
        "pecofence-cli skill"
    ),
    entry!(
        "fence list",
        Some("fences.list"),
        "Fences of the active layout with geometry and options",
        "pecofence-cli fence list"
    ),
    entry!(
        "fence get",
        Some("fences.get"),
        "One fence",
        "pecofence-cli fence get Work"
    ),
    entry!(
        "fence create",
        Some("fences.create"),
        "New virtual fence (--rect/--monitor) or folder portal (--portal DIR, --title optional)",
        "pecofence-cli fence create --title Work --rect 100,100,600,400"
    ),
    entry!(
        "fence delete",
        Some("fences.delete"),
        "Delete; items go back to the desktop fence, rules targeting it are removed",
        "pecofence-cli fence delete Temp"
    ),
    entry!(
        "fence rename",
        Some("fences.rename"),
        "Rename",
        "pecofence-cli fence rename Work Projects"
    ),
    entry!(
        "fence move",
        Some("fences.setBounds"),
        "--rect or --x/--y (fences.setBounds), or --monitor (fences.moveToMonitor)",
        "pecofence-cli fence move Work --x 1200 --y 80"
    ),
    entry!(
        "fence resize",
        Some("fences.setBounds"),
        "--w and/or --h, position kept",
        "pecofence-cli fence resize Work --w 800"
    ),
    entry!(
        "fence set",
        Some("fences.setOption"),
        "Per-fence option (--string forces text); --all applies to every fence one by one, skipping hosted tabs",
        "pecofence-cli fence set Work layout list"
    ),
    entry!(
        "fence roll",
        Some("fences.roll"),
        "Roll up to the title bar (rolled=true); --all",
        "pecofence-cli fence roll Work"
    ),
    entry!(
        "fence unroll",
        Some("fences.roll"),
        "Expand (rolled=false); --all",
        "pecofence-cli fence unroll --all"
    ),
    entry!(
        "fence dock-top",
        Some("fences.dockTop"),
        "Dock to the top edge of its monitor",
        "pecofence-cli fence dock-top Work"
    ),
    entry!(
        "fence merge",
        Some("fences.merge"),
        "Merge into another fence as a tab",
        "pecofence-cli fence merge Games --into Work"
    ),
    entry!(
        "fence detach",
        Some("fences.detach"),
        "Split a tab into its own window",
        "pecofence-cli fence detach Games"
    ),
    entry!(
        "fence hide-all",
        Some("fences.setVisible"),
        "Quick-hide every fence (visible=false)",
        "pecofence-cli fence hide-all"
    ),
    entry!(
        "fence show-all",
        Some("fences.setVisible"),
        "Show every fence (visible=true)",
        "pecofence-cli fence show-all"
    ),
    entry!(
        "fence open-options",
        Some("fences.openOptions"),
        "Open the Settings window on the fence's options page",
        "pecofence-cli fence open-options Work"
    ),
    entry!(
        "item list",
        Some("items.list"),
        "Items of every fence or of --fence",
        "pecofence-cli item list --fence Work"
    ),
    entry!(
        "item move",
        Some("items.move"),
        "Move items (id, path, or name) or --glob matches into --to; into/out of a folder portal moves real files (--glob skips portal items unless --from names the portal)",
        "pecofence-cli item move --glob \"*.pdf\" --to Docs"
    ),
    entry!(
        "settings get",
        Some("settings.get"),
        "Whole Settings or one dotted camelCase path",
        "pecofence-cli settings get peek.enabled"
    ),
    entry!(
        "settings set",
        Some("settings.patch"),
        "Set one value (JSON or text; --string forces text)",
        "pecofence-cli settings set peek.enabled false"
    ),
    entry!(
        "settings open-ui",
        Some("settings.openUi"),
        "Open the Settings window",
        "pecofence-cli settings open-ui"
    ),
    entry!(
        "rule list",
        Some("rules.get"),
        "Rules in order with defaultTarget and keepUpdated",
        "pecofence-cli rule list"
    ),
    entry!(
        "rule add",
        Some("rules.add"),
        "Add a rule from --ext/--type/--name-*/--glob/--folders-only/--files-only/--json",
        "pecofence-cli rule add --name PDFs --ext pdf --to Docs"
    ),
    entry!(
        "rule remove",
        Some("rules.remove"),
        "Remove by id, index, or name",
        "pecofence-cli rule remove PDFs"
    ),
    entry!(
        "rule enable",
        Some("rules.enable"),
        "Enable (enabled=true)",
        "pecofence-cli rule enable 0"
    ),
    entry!(
        "rule disable",
        Some("rules.enable"),
        "Disable (enabled=false)",
        "pecofence-cli rule disable PDFs"
    ),
    entry!(
        "rule move",
        Some("rules.move"),
        "Reorder to a 0-based index",
        "pecofence-cli rule move PDFs --to 0"
    ),
    entry!(
        "rule import",
        Some("rules.set"),
        "Replace the whole rule set from a RuleSet JSON file or stdin (-)",
        "pecofence-cli rule import rules.json"
    ),
    entry!(
        "rule apply",
        Some("rules.apply"),
        "Re-file every desktop item through the rules now",
        "pecofence-cli rule apply"
    ),
    entry!(
        "snapshot list",
        Some("snapshots.list"),
        "Saved layout snapshots",
        "pecofence-cli snapshot list"
    ),
    entry!(
        "snapshot save",
        Some("snapshots.save"),
        "Save the current fence layout; keep the returned snapshot.id for restore",
        "pecofence-cli snapshot save before-cleanup"
    ),
    entry!(
        "snapshot restore",
        Some("snapshots.restore"),
        "Restore by id (names only while unique); the current layout is snapshotted first",
        "pecofence-cli snapshot restore 3f9c2a1e"
    ),
    entry!(
        "snapshot delete",
        Some("snapshots.delete"),
        "Delete a snapshot by id (or unique name)",
        "pecofence-cli snapshot delete 3f9c2a1e"
    ),
    entry!(
        "config export",
        Some("config.export"),
        "Write the whole configuration to an absolute JSON path (overwrites)",
        "pecofence-cli config export C:\\Users\\me\\Desktop\\pecofence.json"
    ),
    entry!(
        "config import",
        Some("config.import"),
        "Replace the configuration from a JSON file; the current layout is snapshotted first",
        "pecofence-cli config import C:\\Users\\me\\Desktop\\pecofence.json"
    ),
    entry!(
        "backup list",
        Some("backups.list"),
        "Daily config backups the app keeps (path, name), newest first",
        "pecofence-cli backup list"
    ),
    entry!(
        "backup restore",
        Some("backups.restore"),
        "Restore a backup by the exact path from `backup list`; the current layout is snapshotted first",
        "pecofence-cli backup restore C:\\Users\\me\\AppData\\Roaming\\PecoFence\\backups\\2026-09-23.json"
    ),
    entry!(
        "peek start",
        Some("peek.start"),
        "Float every fence above other windows",
        "pecofence-cli peek start"
    ),
    entry!(
        "peek end",
        Some("peek.end"),
        "End Peek",
        "pecofence-cli peek end"
    ),
];

/// Methods reachable only through a flag of a command listed above.
#[cfg(test)]
const EXTRA_METHODS: &[&str] = &["fences.moveToMonitor"];

pub const ERROR_CODES: &[(ErrorCode, &str)] = &[
    (
        ErrorCode::NotRunning,
        "No PecoFence instance is listening (exit 3)",
    ),
    (
        ErrorCode::Timeout,
        "No reply within --timeout; the app is probably inside a menu or dialog (exit 4)",
    ),
    (
        ErrorCode::Busy,
        "Too many concurrent CLI connections; retry",
    ),
    (
        ErrorCode::Usage,
        "Malformed request, unknown method, missing or mistyped parameter (exit 2)",
    ),
    (
        ErrorCode::VersionMismatch,
        "CLI and app speak different protocol versions; update both",
    ),
    (ErrorCode::FenceNotFound, "No fence matches the selector"),
    (
        ErrorCode::AmbiguousFence,
        "Several fences match; details.candidates lists them",
    ),
    (
        ErrorCode::ItemNotFound,
        "No item matches the id, path, name, or glob",
    ),
    (
        ErrorCode::AmbiguousItem,
        "Several items share that display name; use the path or id",
    ),
    (
        ErrorCode::RuleNotFound,
        "No rule matches the id, index, or name",
    ),
    (
        ErrorCode::SnapshotNotFound,
        "No snapshot matches the id or name",
    ),
    (
        ErrorCode::InvalidValue,
        "Parameter out of range or not one of details.allowed",
    ),
    (
        ErrorCode::InvalidPath,
        "settings path does not exist in Settings",
    ),
    (
        ErrorCode::ValidationFailed,
        "The patched object did not deserialize; details.expected",
    ),
    (
        ErrorCode::LimitReached,
        "Maximum number of fences or snapshots reached",
    ),
    (
        ErrorCode::Unsupported,
        "Operation does not apply to this target (inbox fence, hosted tab)",
    ),
    (
        ErrorCode::Internal,
        "Unexpected failure in the app or the transport",
    ),
];

pub const NOTES: &[&str] = &[
    "Output is JSON only: the result on stdout, {\"error\":{code,message,hint?,details?}} on stderr. Pretty on a terminal, one line otherwise (--pretty/--compact).",
    "Exit codes: 0 ok, 1 the app returned an error (or part of a --all/--glob batch failed), 2 usage, 3 not running, 4 timeout.",
    "Coordinates are physical pixels in virtual-screen space (primary monitor top-left = 0,0; monitors to the left are negative).",
    "Mutations return {changed:bool, ...}; changed:false means the state was already as requested and is not an error. fence delete (with items), rules.apply (when it moves something) and snapshot restore take an automatic layout snapshot first and add snapshotId; settings and rules are not covered by snapshots.",
    "Fence selectors: id, unique id prefix (>=6 hex), or title (exact, then unique substring); `inbox` always means the desktop fence (kind == \"inbox\" in fence list), whatever its title. Rules: id, 0-based index, or name. Snapshots: id, unique id prefix, or name (fails with snapshot_not_found when several share it; use snapshot.id from snapshot save).",
    "Folder portals show real files: item move into or out of a portal is an Explorer file move (undo only via Explorer Ctrl+Z; snapshots do not revert it). item move --glob without --from skips portal items; pass --from <portal> to include them.",
    "Shell quoting: values starting with #, [ or * or containing spaces need quotes (Git Bash treats #ff8800 as a comment): fence set Work tint \"#ff8800\" or tint '\"#ff8800\"'. Negative numbers work bare: settings set snapping.gapPx -4.",
    "Settings paths are dotted camelCase (peek.enabled, quickHide.delayMs, rollUp.hoverPeek, snapping.gapPx, iconSize, theme, themeStyle, hideRealIcons, autostart, icons.chameleon); see describe --schema Settings.",
    "fence set props: title (or fence rename), iconSize 32|48|64|96, spacing compact|normal|loose, autoHeight, locked, excludeFromQuickHide, opacity default|clear|solid, tint \"#RRGGBB\"|null, titleColor theme|tint|white|black|\"#RRGGBB\", titleSize small|normal|large, layout icons|list|details, sort manual|name|type|date|size|openCount, reverse, groupByDate, labelLines, portalNavigate, portalTitleIcon. --string keeps numeric-looking text (title 2024) a string.",
    "The app must be running; the CLI never edits config.json. --instance <name> only addresses a test instance started with PECOFENCE_INSTANCE=<name>.",
];

pub fn catalog() -> Value {
    let commands: Vec<Value> = COMMANDS
        .iter()
        .map(|e| {
            json!({
                "command": e.command,
                "method": e.method,
                "summary": e.summary,
                "example": e.example,
            })
        })
        .collect();
    let error_codes: Vec<Value> = ERROR_CODES
        .iter()
        .map(|(code, meaning)| json!({ "code": code, "exitCode": code.exit_code(), "meaning": meaning }))
        .collect();
    json!({
        "cli": env!("CARGO_PKG_VERSION"),
        "protocol": PROTOCOL_VERSION,
        "exitCodes": {
            "0": "success",
            "1": "the app returned an error, or part of a batch failed",
            "2": "usage error (clap message on stderr, or code usage)",
            "3": "PecoFence is not running",
            "4": "timeout waiting for the app",
        },
        "errorCodes": error_codes,
        "commands": commands,
        "schemas": SCHEMA_NAMES,
        "notes": NOTES,
    })
}

pub fn run(schema_name: Option<&str>) -> Result<Value, IpcError> {
    match schema_name {
        None => Ok(catalog()),
        Some(name) => {
            let wanted = name.trim();
            let canonical = SCHEMA_NAMES
                .iter()
                .find(|n| n.eq_ignore_ascii_case(wanted))
                .and_then(|n| schema(n));
            canonical.ok_or_else(|| {
                IpcError::usage(format!("unknown schema {name:?}"))
                    .hint("Run `pecofence-cli describe` and pick one of .schemas")
                    .details(json!({ "allowed": SCHEMA_NAMES }))
            })
        }
    }
}

/// Pretty layout tuned for reading: one line per command / error code / note, so the whole
/// catalog stays around a hundred lines instead of several hundred.
pub fn render_pretty(catalog: &Value) -> String {
    let Value::Object(map) = catalog else {
        return serde_json::to_string_pretty(catalog).unwrap_or_default();
    };
    // serde_json sorts object keys; readers want the short scalars first and the catalog last.
    const KEY_ORDER: &[&str] = &[
        "cli",
        "protocol",
        "exitCodes",
        "errorCodes",
        "commands",
        "schemas",
        "notes",
    ];
    let ordered: Vec<(&String, &Value)> = KEY_ORDER
        .iter()
        .filter_map(|k| map.get_key_value(*k))
        .chain(map.iter().filter(|(k, _)| !KEY_ORDER.contains(&k.as_str())))
        .collect();
    let mut out = String::from("{\n");
    let last = ordered.len().saturating_sub(1);
    for (i, (key, value)) in ordered.into_iter().enumerate() {
        out.push_str(&format!("  {}: ", json!(key)));
        match value {
            Value::Array(items) if items.iter().all(Value::is_string) => {
                out.push_str(&serde_json::to_string(value).unwrap_or_default());
            }
            Value::Array(items) => {
                out.push_str("[\n");
                for (j, item) in items.iter().enumerate() {
                    out.push_str("    ");
                    out.push_str(&serde_json::to_string(item).unwrap_or_default());
                    out.push_str(if j + 1 == items.len() { "\n" } else { ",\n" });
                }
                out.push_str("  ]");
            }
            Value::Object(_) => {
                let nested = serde_json::to_string_pretty(value).unwrap_or_default();
                out.push_str(&nested.replace('\n', "\n  "));
            }
            other => out.push_str(&serde_json::to_string(other).unwrap_or_default()),
        }
        out.push_str(if i == last { "\n" } else { ",\n" });
    }
    out.push('}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every `const`/`enum` string in a schema that looks like a dotted method name.
    fn collect_dotted(value: &Value, out: &mut BTreeSet<String>) {
        match value {
            Value::Object(map) => {
                for (k, v) in map {
                    match (k.as_str(), v) {
                        ("const", Value::String(s)) if s.contains('.') => {
                            out.insert(s.clone());
                        }
                        ("enum", Value::Array(items)) => {
                            for s in items.iter().filter_map(Value::as_str) {
                                if s.contains('.') {
                                    out.insert(s.to_string());
                                }
                            }
                        }
                        _ => collect_dotted(v, out),
                    }
                }
            }
            Value::Array(items) => items.iter().for_each(|v| collect_dotted(v, out)),
            _ => {}
        }
    }

    #[test]
    fn catalog_covers_every_protocol_method() {
        let mut from_schema = BTreeSet::new();
        collect_dotted(&schema("Request").unwrap(), &mut from_schema);
        assert!(from_schema.contains("fences.create"), "{from_schema:?}");
        assert!(from_schema.len() >= 30, "{from_schema:?}");

        let mut from_catalog: BTreeSet<String> = COMMANDS
            .iter()
            .filter_map(|e| e.method.map(str::to_string))
            .collect();
        from_catalog.extend(EXTRA_METHODS.iter().map(|s| s.to_string()));
        assert_eq!(from_catalog, from_schema);
    }

    #[test]
    fn catalog_covers_every_error_code() {
        let mut from_schema = BTreeSet::new();
        fn collect_enum(value: &Value, out: &mut BTreeSet<String>) {
            match value {
                Value::Object(map) => {
                    // schemars emits documented variants as `const` and folds the undocumented
                    // ones into one `enum` array.
                    if let Some(Value::Array(items)) = map.get("enum") {
                        for s in items.iter().filter_map(Value::as_str) {
                            out.insert(s.to_string());
                        }
                    }
                    if let Some(Value::String(s)) = map.get("const") {
                        out.insert(s.clone());
                    }
                    map.values().for_each(|v| collect_enum(v, out));
                }
                Value::Array(items) => items.iter().for_each(|v| collect_enum(v, out)),
                _ => {}
            }
        }
        let response = schema("Response").unwrap();
        collect_enum(&response["$defs"]["ErrorCode"], &mut from_schema);
        assert!(from_schema.contains("not_running"), "{response}");
        let listed: BTreeSet<String> = ERROR_CODES
            .iter()
            .map(|(c, _)| {
                serde_json::to_value(c)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string()
            })
            .collect();
        assert_eq!(listed, from_schema);
    }

    #[test]
    fn every_command_has_a_distinct_example_starting_with_the_binary() {
        let mut seen = BTreeSet::new();
        for e in COMMANDS {
            assert!(e.example.starts_with("pecofence-cli "), "{}", e.command);
            assert!(
                e.example.contains(e.command),
                "{} vs {}",
                e.command,
                e.example
            );
            assert!(seen.insert(e.command), "duplicate {}", e.command);
        }
    }

    #[test]
    fn pretty_catalog_is_valid_json_and_compact_in_height() {
        let cat = catalog();
        let text = render_pretty(&cat);
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), cat);
        let lines = text.lines().count();
        assert!(lines < 120, "{lines} lines");
        assert!(cat["schemas"].as_array().unwrap().len() == SCHEMA_NAMES.len());
    }

    #[test]
    fn unknown_schema_is_a_usage_error_and_names_are_case_insensitive() {
        let err = run(Some("Nope")).unwrap_err();
        assert_eq!(err.code, ErrorCode::Usage);
        assert_eq!(err.code.exit_code(), 2);
        assert_eq!(err.details.unwrap()["allowed"], json!(SCHEMA_NAMES));
        let canonical = run(Some("Settings")).unwrap();
        assert!(canonical.is_object());
        assert_eq!(run(Some("settings")).unwrap(), canonical);
        assert_eq!(run(Some("SETTINGS")).unwrap(), canonical);
        assert_eq!(
            run(Some(" fencedto ")).unwrap(),
            run(Some("FenceDto")).unwrap()
        );
    }

    /// Minimal shell-words: whitespace separates, double or single quotes group (and are
    /// dropped). Enough for the examples, which never nest quotes.
    fn shell_words(line: &str) -> Vec<String> {
        let mut words = Vec::new();
        let mut cur = String::new();
        let mut quote: Option<char> = None;
        let mut in_word = false;
        for c in line.chars() {
            match (quote, c) {
                (Some(q), c) if c == q => quote = None,
                (Some(_), c) => cur.push(c),
                (None, '"' | '\'') => {
                    quote = Some(c);
                    in_word = true;
                }
                (None, c) if c.is_whitespace() => {
                    if in_word {
                        words.push(std::mem::take(&mut cur));
                        in_word = false;
                    }
                }
                (None, c) => {
                    cur.push(c);
                    in_word = true;
                }
            }
        }
        assert!(quote.is_none(), "unbalanced quotes in {line:?}");
        if in_word {
            words.push(cur);
        }
        words
    }

    #[test]
    fn every_example_parses() {
        use clap::Parser;
        for e in COMMANDS {
            let words = shell_words(e.example);
            assert_eq!(words[0], "pecofence-cli", "{}", e.example);
            crate::cli::Cli::try_parse_from(&words)
                .unwrap_or_else(|err| panic!("{}: {err}", e.example));
        }
        assert_eq!(shell_words(r#"a "b c" 'd' e"#), vec!["a", "b c", "d", "e"]);
    }

    /// Leaves of the clap tree as space-joined paths (`fence set`, `peek start`, ...).
    fn clap_leaves(cmd: &clap::Command, prefix: &str, out: &mut BTreeSet<String>) {
        let subs: Vec<&clap::Command> = cmd.get_subcommands().collect();
        if subs.is_empty() {
            out.insert(prefix.to_string());
            return;
        }
        for sub in subs {
            let path = if prefix.is_empty() {
                sub.get_name().to_string()
            } else {
                format!("{prefix} {}", sub.get_name())
            };
            clap_leaves(sub, &path, out);
        }
    }

    #[test]
    fn catalog_lists_every_clap_leaf_and_nothing_else() {
        use clap::CommandFactory;
        let mut leaves = BTreeSet::new();
        clap_leaves(&crate::cli::Cli::command(), "", &mut leaves);
        let listed: BTreeSet<String> = COMMANDS.iter().map(|e| e.command.to_string()).collect();
        assert_eq!(listed, leaves);
    }

    #[test]
    fn catalog_methods_are_real_wire_names() {
        // Every method string in the catalog must be one the protocol enum serialises.
        let wire: BTreeSet<String> = {
            let mut set = BTreeSet::new();
            collect_dotted(&schema("Request").unwrap(), &mut set);
            set
        };
        for e in COMMANDS {
            if let Some(m) = e.method {
                assert!(wire.contains(m), "{m} is not a Method");
            }
        }
        assert_eq!(Method::FencesList.name(), "fences.list");
    }
}
