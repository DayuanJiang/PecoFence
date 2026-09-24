//! Maps each subcommand to one or more protocol calls. Client-side batching (`--all`,
//! `--glob`) lives here too; the server has no bulk methods.

use std::io::Read;

use pecofence_core::rules::{Cond, RuleSet, StrOp, TypeCategory, glob_match};
use pecofence_ipc::{ErrorCode, IpcError, Method, Rect, Response};
use serde_json::{Value, json};

use crate::cli::split_set_args;
use crate::cli::{
    Command, FenceCmd, ItemCmd, MonitorCmd, PeekCmd, RuleAddArgs, RuleCmd, SettingsCmd, SnapshotCmd,
};
use crate::output::Reply;
use crate::{client, describe};

const SET_USAGE: &str =
    "Usage: pecofence-cli fence set <FENCE> <PROP> <VALUE>  |  fence set --all <PROP> <VALUE>";

pub struct Ctx {
    pub instance: Option<String>,
    pub timeout_ms: u32,
}

impl Ctx {
    fn call(&self, method: Method) -> Result<Reply, IpcError> {
        let response = client::send(self.instance.as_deref(), method, self.timeout_ms)?;
        reply_from(response)
    }
}

fn reply_from(response: Response) -> Result<Reply, IpcError> {
    if !response.ok {
        return Err(response.error.unwrap_or_else(|| {
            IpcError::internal("PecoFence reported a failure without details")
        }));
    }
    Ok(Reply {
        result: response.result.unwrap_or(Value::Null),
        warning: response.warning,
        partial_failure: false,
    })
}

pub fn run(ctx: &Ctx, command: Command) -> Result<Reply, IpcError> {
    match command {
        Command::Status => ctx.call(Method::StatusGet),
        Command::Monitor {
            cmd: MonitorCmd::List,
        } => ctx.call(Method::MonitorsList),
        Command::Describe { schema } => describe::run(schema.as_deref()).map(Reply::new),
        Command::Skill => {
            // Verbatim text, not JSON: the file is meant to be redirected into a skills folder.
            crate::output::print_text(
                include_str!("../../../skills/pecofence-cli/SKILL.md"),
                false,
            );
            std::process::exit(0);
        }
        Command::Fence { cmd } => run_fence(ctx, cmd),
        Command::Item { cmd } => run_item(ctx, cmd),
        Command::Settings { cmd } => run_settings(ctx, cmd),
        Command::Rule { cmd } => run_rule(ctx, cmd),
        Command::Snapshot { cmd } => run_snapshot(ctx, cmd),
        Command::Peek {
            cmd: PeekCmd::Start,
        } => ctx.call(Method::PeekStart),
        Command::Peek { cmd: PeekCmd::End } => ctx.call(Method::PeekEnd),
    }
}

// ---- fences ------------------------------------------------------------------------------

fn run_fence(ctx: &Ctx, cmd: FenceCmd) -> Result<Reply, IpcError> {
    match cmd {
        FenceCmd::List => ctx.call(Method::FencesList),
        FenceCmd::Get { fence } => ctx.call(Method::FencesGet { fence }),
        FenceCmd::Create {
            title,
            rect,
            monitor,
            portal,
        } => ctx.call(Method::FencesCreate {
            title: Some(title),
            rect,
            monitor,
            portal,
        }),
        FenceCmd::Delete { fence } => ctx.call(Method::FencesDelete { fence }),
        FenceCmd::Rename { fence, title } => ctx.call(Method::FencesRename { fence, title }),
        FenceCmd::Move {
            fence,
            rect,
            x,
            y,
            monitor,
        } => {
            if let Some(monitor) = monitor {
                return ctx.call(Method::FencesMoveToMonitor { fence, monitor });
            }
            if let Some(rect) = rect {
                return ctx.call(Method::FencesSetBounds { fence, rect });
            }
            let (id, current) = current_rect(ctx, &fence)?;
            let rect = Rect {
                x: x.unwrap_or(current.x),
                y: y.unwrap_or(current.y),
                ..current
            };
            ctx.call(Method::FencesSetBounds { fence: id, rect })
        }
        FenceCmd::Resize { fence, w, h } => {
            let (id, current) = current_rect(ctx, &fence)?;
            let rect = Rect {
                w: w.unwrap_or(current.w),
                h: h.unwrap_or(current.h),
                ..current
            };
            if rect.w <= 0 || rect.h <= 0 {
                return Err(IpcError::new(
                    ErrorCode::InvalidValue,
                    "width and height must be positive",
                ));
            }
            ctx.call(Method::FencesSetBounds { fence: id, rect })
        }
        FenceCmd::Set { args, all } => {
            let (fence, prop, value) =
                split_set_args(&args, all).map_err(|m| IpcError::usage(m).hint(SET_USAGE))?;
            let value = parse_value(value, false);
            let prop = prop.to_string();
            if all {
                for_each_fence(ctx, |fence| Method::FencesSetOption {
                    fence,
                    prop: prop.clone(),
                    value: value.clone(),
                })
            } else {
                ctx.call(Method::FencesSetOption {
                    fence: fence.unwrap_or_default().to_string(),
                    prop,
                    value,
                })
            }
        }
        FenceCmd::Roll { fence, all } => roll(ctx, fence, all, true),
        FenceCmd::Unroll { fence, all } => roll(ctx, fence, all, false),
        FenceCmd::DockTop { fence } => ctx.call(Method::FencesDockTop { fence }),
        FenceCmd::Merge { fence, into } => ctx.call(Method::FencesMerge { fence, into }),
        FenceCmd::Detach { fence } => ctx.call(Method::FencesDetach { tab: fence }),
        FenceCmd::HideAll => ctx.call(Method::FencesSetVisible { visible: false }),
        FenceCmd::ShowAll => ctx.call(Method::FencesSetVisible { visible: true }),
        FenceCmd::OpenOptions { fence } => ctx.call(Method::FencesOpenOptions { fence }),
    }
}

fn roll(ctx: &Ctx, fence: Option<String>, all: bool, rolled: bool) -> Result<Reply, IpcError> {
    if all {
        for_each_fence(ctx, |fence| Method::FencesRoll { fence, rolled })
    } else {
        ctx.call(Method::FencesRoll {
            fence: fence.unwrap_or_default(),
            rolled,
        })
    }
}

/// `fences.get`, returning the fence's id (so later calls skip selector resolution) and its
/// expanded rectangle.
fn current_rect(ctx: &Ctx, fence: &str) -> Result<(String, Rect), IpcError> {
    let dto = ctx
        .call(Method::FencesGet {
            fence: fence.to_string(),
        })?
        .result;
    let id = dto["id"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| fence.to_string());
    let title = dto["title"].as_str().unwrap_or(fence);
    match serde_json::from_value::<Option<Rect>>(dto["rect"].clone()) {
        Ok(Some(rect)) => Ok((id, rect)),
        _ => {
            let hint = match dto["tabHost"].as_str() {
                Some(host) => format!(
                    "The fence is a tab hosted by fence {host}; move or resize the host instead"
                ),
                None => "Its monitor is disconnected; reconnect it or use --monitor".to_string(),
            };
            Err(IpcError::new(
                ErrorCode::Unsupported,
                format!("fence {title:?} has no geometry of its own"),
            )
            .hint(hint))
        }
    }
}

/// `fences.list`, then one call per fence. Never stops early; the summary says what failed.
fn for_each_fence(ctx: &Ctx, make: impl Fn(String) -> Method) -> Result<Reply, IpcError> {
    let list = ctx.call(Method::FencesList)?.result;
    let fences = list.as_array().cloned().unwrap_or_default();
    let targets = fences.iter().map(|f| {
        (
            f["id"].as_str().unwrap_or_default().to_string(),
            f["title"].as_str().unwrap_or_default().to_string(),
        )
    });
    let mut results = Vec::new();
    let mut changed = false;
    let mut failed = false;
    let mut warnings = Vec::new();
    for (id, title) in targets {
        match ctx.call(make(id.clone())) {
            Ok(reply) => {
                let this_changed = reply.result["changed"].as_bool().unwrap_or(false);
                changed |= this_changed;
                if let Some(w) = reply.warning {
                    warnings.push(format!("{title}: {w}"));
                }
                results.push(json!({ "fence": id, "title": title, "changed": this_changed }));
            }
            Err(error) => {
                failed = true;
                results.push(json!({ "fence": id, "title": title, "error": error }));
            }
        }
    }
    Ok(Reply {
        result: json!({ "changed": changed, "results": results }),
        warning: (!warnings.is_empty()).then(|| warnings.join("; ")),
        partial_failure: failed,
    })
}

// ---- items -------------------------------------------------------------------------------

fn run_item(ctx: &Ctx, cmd: ItemCmd) -> Result<Reply, IpcError> {
    match cmd {
        ItemCmd::List { fence } => ctx.call(Method::ItemsList { fence }),
        ItemCmd::Move {
            items,
            glob,
            from,
            to,
        } => {
            let items = match glob {
                None => items,
                Some(pattern) => {
                    let list = ctx.call(Method::ItemsList { fence: from })?.result;
                    let ids = glob_item_ids(&pattern, &list);
                    if ids.is_empty() {
                        return Err(IpcError::new(
                            ErrorCode::ItemNotFound,
                            format!("no item matches {pattern:?}"),
                        )
                        .hint("Run `pecofence-cli item list` to see names and paths"));
                    }
                    ids
                }
            };
            ctx.call(Method::ItemsMove { items, to })
        }
    }
}

/// Ids of the items whose file name (last path component, with extension) or display name
/// matches `pattern`.
pub fn glob_item_ids(pattern: &str, list: &Value) -> Vec<String> {
    list.as_array()
        .map(|items| {
            items
                .iter()
                .filter(|item| {
                    let file_name = item["path"]
                        .as_str()
                        .map(|p| p.rsplit(['\\', '/']).next().unwrap_or(p))
                        .unwrap_or_default();
                    let name = item["name"].as_str().unwrap_or_default();
                    (!file_name.is_empty() && glob_match(pattern, file_name))
                        || (!name.is_empty() && glob_match(pattern, name))
                })
                .filter_map(|item| item["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

// ---- settings ----------------------------------------------------------------------------

fn run_settings(ctx: &Ctx, cmd: SettingsCmd) -> Result<Reply, IpcError> {
    match cmd {
        SettingsCmd::Get { path } => ctx.call(Method::SettingsGet { path }),
        SettingsCmd::Set {
            path,
            value,
            string,
        } => ctx.call(Method::SettingsPatch {
            path,
            value: parse_value(&value, string),
        }),
        SettingsCmd::OpenUi => ctx.call(Method::SettingsOpenUi),
    }
}

/// `true` → bool, `48` → number, `null` → null, `{"a":1}` → object; anything that is not JSON
/// (or everything, with `force_string`) is a string.
pub fn parse_value(text: &str, force_string: bool) -> Value {
    if force_string {
        return Value::String(text.to_string());
    }
    serde_json::from_str::<Value>(text).unwrap_or_else(|_| Value::String(text.to_string()))
}

// ---- rules -------------------------------------------------------------------------------

fn run_rule(ctx: &Ctx, cmd: RuleCmd) -> Result<Reply, IpcError> {
    match cmd {
        RuleCmd::List => ctx.call(Method::RulesGet),
        RuleCmd::Add(args) => {
            let args = *args;
            let all_of = build_conds(&args)?;
            ctx.call(Method::RulesAdd {
                name: args.name,
                target: args.to,
                all_of,
                index: args.index,
            })
        }
        RuleCmd::Remove { rule } => ctx.call(Method::RulesRemove { rule }),
        RuleCmd::Enable { rule } => ctx.call(Method::RulesEnable {
            rule,
            enabled: true,
        }),
        RuleCmd::Disable { rule } => ctx.call(Method::RulesEnable {
            rule,
            enabled: false,
        }),
        RuleCmd::Move { rule, to } => ctx.call(Method::RulesMove { rule, to }),
        RuleCmd::Import { file } => {
            let text = read_file_or_stdin(&file)?;
            let rules: RuleSet = serde_json::from_str(&text).map_err(|e| {
                IpcError::new(
                    ErrorCode::ValidationFailed,
                    format!("{file} is not a RuleSet: {e}"),
                )
                .hint(
                    "Expected the shape of `pecofence-cli rule list` / `describe --schema RuleSet`",
                )
            })?;
            ctx.call(Method::RulesSet { rules })
        }
        RuleCmd::Apply => ctx.call(Method::RulesApply),
    }
}

fn read_file_or_stdin(file: &str) -> Result<String, IpcError> {
    if file == "-" {
        let mut text = String::new();
        std::io::stdin()
            .read_to_string(&mut text)
            .map_err(|e| IpcError::internal(format!("cannot read stdin: {e}")))?;
        Ok(text)
    } else {
        std::fs::read_to_string(file)
            .map_err(|e| IpcError::new(ErrorCode::InvalidValue, format!("cannot read {file}: {e}")))
    }
}

pub const TYPE_NAMES: &[(&str, TypeCategory)] = &[
    ("programs", TypeCategory::Programs),
    ("folders", TypeCategory::Folders),
    ("documents", TypeCategory::Documents),
    ("images", TypeCategory::Images),
    ("music", TypeCategory::Music),
    ("video", TypeCategory::Video),
    ("archives", TypeCategory::Archives),
    ("shortcuts", TypeCategory::Shortcuts),
    ("installers", TypeCategory::Installers),
];

fn parse_type(name: &str) -> Result<TypeCategory, IpcError> {
    let wanted = name.trim().to_ascii_lowercase();
    TYPE_NAMES
        .iter()
        .find(|(n, _)| *n == wanted)
        .map(|(_, t)| *t)
        .ok_or_else(|| {
            let allowed: Vec<&str> = TYPE_NAMES.iter().map(|(n, _)| *n).collect();
            IpcError::invalid_value(format!("unknown --type {name:?}"), &allowed)
        })
}

/// The `Cond` list for `rule add`, in flag order; `--json` conditions come last.
pub fn build_conds(args: &RuleAddArgs) -> Result<Vec<Cond>, IpcError> {
    let mut conds = Vec::new();
    let exts: Vec<String> = args
        .ext
        .iter()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty())
        .map(|e| {
            if e.starts_with('.') {
                e
            } else {
                format!(".{e}")
            }
        })
        .collect();
    if !exts.is_empty() {
        conds.push(Cond::Ext(exts));
    }
    if !args.types.is_empty() {
        let types = args
            .types
            .iter()
            .map(|t| parse_type(t))
            .collect::<Result<Vec<_>, _>>()?;
        conds.push(Cond::Type(types));
    }
    let names = [
        (StrOp::Contains, &args.name_contains),
        (StrOp::NotContains, &args.name_not_contains),
        (StrOp::StartsWith, &args.starts_with),
        (StrOp::EndsWith, &args.ends_with),
        (StrOp::Is, &args.name_is),
    ];
    for (op, value) in names {
        if let Some(value) = value {
            conds.push(Cond::Name {
                op,
                value: value.clone(),
            });
        }
    }
    if let Some(glob) = &args.glob {
        conds.push(Cond::Glob(glob.clone()));
    }
    if args.folders_only {
        conds.push(Cond::FoldersOnly);
    }
    if args.files_only {
        conds.push(Cond::FilesOnly);
    }
    if let Some(json) = &args.json {
        let extra: Vec<Cond> = serde_json::from_str(json).map_err(|e| {
            IpcError::new(
                ErrorCode::InvalidValue,
                format!("--json is not an array of Cond objects: {e}"),
            )
            .hint("See `pecofence-cli describe --schema Cond`")
        })?;
        conds.extend(extra);
    }
    if conds.is_empty() {
        return Err(IpcError::usage("rule add needs at least one condition"));
    }
    Ok(conds)
}

// ---- snapshots ---------------------------------------------------------------------------

fn run_snapshot(ctx: &Ctx, cmd: SnapshotCmd) -> Result<Reply, IpcError> {
    match cmd {
        SnapshotCmd::List => ctx.call(Method::SnapshotsList),
        SnapshotCmd::Save { name } => ctx.call(Method::SnapshotsSave { name }),
        SnapshotCmd::Restore { id } => ctx.call(Method::SnapshotsRestore { id }),
        SnapshotCmd::Delete { id } => ctx.call(Method::SnapshotsDelete { id }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_parse_as_json_first_then_text() {
        assert_eq!(parse_value("true", false), Value::Bool(true));
        assert_eq!(parse_value("48", false), json!(48));
        assert_eq!(parse_value("-1.5", false), json!(-1.5));
        assert_eq!(parse_value("null", false), Value::Null);
        assert_eq!(parse_value("a b", false), json!("a b"));
        assert_eq!(parse_value("list", false), json!("list"));
        assert_eq!(parse_value("#ff8800", false), json!("#ff8800"));
        assert_eq!(parse_value("\"quoted\"", false), json!("quoted"));
        assert_eq!(parse_value("[1,2]", false), json!([1, 2]));
        assert_eq!(parse_value("48", true), json!("48"));
        assert_eq!(parse_value("true", true), json!("true"));
    }

    #[test]
    fn glob_matches_file_name_or_display_name_case_insensitively() {
        let list = json!([
            {"id": "a", "name": "Report", "path": "C:\\Users\\me\\Desktop\\Report.PDF"},
            {"id": "b", "name": "notes", "path": "C:/Users/me/Desktop/notes.txt"},
            {"id": "c", "name": "This PC", "path": null},
            {"id": "d", "name": "photo", "path": "C:\\Users\\me\\Desktop\\photo.jpeg"},
        ]);
        assert_eq!(glob_item_ids("*.pdf", &list), vec!["a"]);
        assert_eq!(glob_item_ids("*.jp?g", &list), vec!["d"]);
        assert_eq!(glob_item_ids("this pc", &list), vec!["c"]);
        assert_eq!(glob_item_ids("n*", &list), vec!["b"]);
        assert_eq!(glob_item_ids("*", &list).len(), 4);
        assert!(glob_item_ids("*.exe", &list).is_empty());
        assert!(glob_item_ids("*", &json!(null)).is_empty());
    }

    fn add_args() -> RuleAddArgs {
        RuleAddArgs {
            name: "r".into(),
            to: "Docs".into(),
            ext: vec![],
            types: vec![],
            name_contains: None,
            name_not_contains: None,
            starts_with: None,
            ends_with: None,
            name_is: None,
            glob: None,
            folders_only: false,
            files_only: false,
            json: None,
            index: None,
        }
    }

    #[test]
    fn rule_conditions_are_built_from_flags() {
        let mut a = add_args();
        a.ext = vec!["PDF".into(), ".docx".into(), " ".into()];
        a.types = vec!["Images".into(), "video".into()];
        a.name_contains = Some("draft".into());
        a.name_not_contains = Some("final".into());
        a.starts_with = Some("20".into());
        a.ends_with = Some("_v2".into());
        a.name_is = Some("todo.txt".into());
        a.glob = Some("*.bak".into());
        a.files_only = true;
        a.json = Some(r#"[{"cond":"idleDays","value":{"min":30}}]"#.into());
        let conds = build_conds(&a).unwrap();
        assert_eq!(
            conds,
            vec![
                Cond::Ext(vec![".pdf".into(), ".docx".into()]),
                Cond::Type(vec![TypeCategory::Images, TypeCategory::Video]),
                Cond::Name {
                    op: StrOp::Contains,
                    value: "draft".into()
                },
                Cond::Name {
                    op: StrOp::NotContains,
                    value: "final".into()
                },
                Cond::Name {
                    op: StrOp::StartsWith,
                    value: "20".into()
                },
                Cond::Name {
                    op: StrOp::EndsWith,
                    value: "_v2".into()
                },
                Cond::Name {
                    op: StrOp::Is,
                    value: "todo.txt".into()
                },
                Cond::Glob("*.bak".into()),
                Cond::FilesOnly,
                Cond::IdleDays { min: 30 },
            ]
        );

        let mut a = add_args();
        a.folders_only = true;
        assert_eq!(build_conds(&a).unwrap(), vec![Cond::FoldersOnly]);
    }

    #[test]
    fn rule_conditions_reject_unknown_types_and_bad_json() {
        let mut a = add_args();
        a.types = vec!["apps".into()];
        let err = build_conds(&a).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(err.details.unwrap()["allowed"].as_array().unwrap().len() == 9);

        let mut a = add_args();
        a.json = Some("{}".into());
        assert_eq!(build_conds(&a).unwrap_err().code, ErrorCode::InvalidValue);

        assert_eq!(build_conds(&add_args()).unwrap_err().code, ErrorCode::Usage);
    }

    #[test]
    fn every_type_category_has_a_name() {
        let all = [
            TypeCategory::Programs,
            TypeCategory::Folders,
            TypeCategory::Documents,
            TypeCategory::Images,
            TypeCategory::Music,
            TypeCategory::Video,
            TypeCategory::Archives,
            TypeCategory::Shortcuts,
            TypeCategory::Installers,
        ];
        for t in all {
            let name = serde_json::to_value(t).unwrap();
            assert_eq!(parse_type(name.as_str().unwrap()).unwrap(), t);
        }
    }

    #[test]
    fn server_errors_pass_through_and_missing_results_are_null() {
        let err = IpcError::new(ErrorCode::FenceNotFound, "nope");
        assert_eq!(reply_from(Response::err(err.clone())).unwrap_err(), err);
        let ok = reply_from(Response::ok(json!({"changed": false})).with_warning("w")).unwrap();
        assert_eq!(ok.result, json!({"changed": false}));
        assert_eq!(ok.warning.as_deref(), Some("w"));
        let bare = Response {
            ok: true,
            result: None,
            error: None,
            warning: None,
        };
        assert_eq!(reply_from(bare).unwrap().result, Value::Null);
    }
}
