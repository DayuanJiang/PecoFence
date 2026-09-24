//! Wire protocol between `pecofence-cli` and the running PecoFence instance.
//!
//! Transport: the app listens on the named pipe [`pecofence_core::brand::ipc_pipe_name`]. One
//! connection carries exactly one [`Request`] and one [`Response`], each a single line of JSON
//! terminated by `\n`. Both binaries depend on this crate only, so the CLI never links the
//! GUI (or WebView2).
//!
//! Compatibility: [`PROTOCOL_VERSION`] is bumped whenever a method's params or a DTO changes in
//! a way an older peer would misread; the server answers `version_mismatch` for anything else.

pub mod dto;
#[cfg(feature = "describe")]
pub mod schema;
pub mod selector;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use dto::*;
pub use pecofence_core::brand::ipc_pipe_name;

/// Bump when the wire format changes incompatibly.
pub const PROTOCOL_VERSION: u32 = 1;

/// The CLI's default wait for a reply. The UI thread may be inside a modal loop (a context
/// menu, a file dialog); the server drops requests that outlive this instead of running them
/// late, so an agent's retry never executes a command twice.
pub const DEFAULT_TIMEOUT_MS: u32 = 15_000;

/// Longest request line the server reads; anything longer is rejected as `usage`.
pub const MAX_REQUEST_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub protocol: u32,
    /// How long the client will wait; the server discards the request once this has elapsed
    /// without executing it.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u32,
    #[serde(flatten)]
    pub method: Method,
}

fn default_timeout() -> u32 {
    DEFAULT_TIMEOUT_MS
}

impl Request {
    pub fn new(method: Method) -> Self {
        Self {
            protocol: PROTOCOL_VERSION,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            method,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

/// Every operation the app exposes. Serialized as `{"method": "fences.create", "params": {…}}`;
/// unit variants omit `params`.
///
/// Fence selectors (`fence`, `into`, `tab`, `to`, rule targets) accept a full UUID, a unique
/// UUID prefix of at least six hex digits, or a title (exact match first, then a unique
/// case-insensitive substring); see [`selector::resolve`]. The alias `inbox` (any case)
/// always names the inbox fence, whatever its localized title.
///
/// Titles, rule names and snapshot names must be 1 to 256 characters after trimming.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(
    tag = "method",
    content = "params",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Method {
    // ---- read-only -------------------------------------------------------------------
    /// Version, pid, config path, counters. Result: [`StatusDto`].
    #[serde(rename = "status.get")]
    StatusGet,
    /// Connected monitors with virtual-screen rectangles. Result: `[MonitorDto]`.
    #[serde(rename = "monitors.list")]
    MonitorsList,
    /// Fences of the active layout (the current monitor set). Result: `[FenceDto]`.
    #[serde(rename = "fences.list")]
    FencesList,
    /// Result: [`FenceDto`].
    #[serde(rename = "fences.get")]
    FencesGet { fence: String },
    /// Items of one fence, or of every fence. Result: `[ItemDto]`.
    #[serde(rename = "items.list")]
    ItemsList {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fence: Option<String>,
    },
    /// The global `Settings` object, or one value at a dotted camelCase path such as
    /// `peek.enabled`. Result: the JSON value.
    #[serde(rename = "settings.get")]
    SettingsGet {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    /// Result: [`RuleListDto`].
    #[serde(rename = "rules.get")]
    RulesGet,
    /// Result: `[SnapshotDto]`.
    #[serde(rename = "snapshots.list")]
    SnapshotsList,

    // ---- fences ---------------------------------------------------------------------
    /// New virtual fence (or folder portal when `portal` is set). `rect` is physical pixels in
    /// virtual-screen coordinates; omitted = free spot near the pointer / first monitor.
    /// Result: `{changed, fence: FenceDto}`.
    #[serde(rename = "fences.create")]
    FencesCreate {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        rect: Option<Rect>,
        /// Monitor id (see `monitors.list`) to place the fence on when `rect` is omitted.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        monitor: Option<String>,
        /// Folder path: create a folder portal showing this folder instead of a virtual fence.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        portal: Option<String>,
    },
    /// Items return to the 「桌面」 (inbox) fence; rules targeting the fence are removed. The
    /// inbox itself cannot be deleted. Result: `{changed, deleted: uuid, snapshotId?}`.
    #[serde(rename = "fences.delete")]
    FencesDelete { fence: String },
    /// Result: `{changed, fence: FenceDto}`.
    #[serde(rename = "fences.rename")]
    FencesRename { fence: String, title: String },
    /// Move and/or resize (physical px). Tabs hosted in another fence's window have no
    /// geometry of their own (`unsupported`; address the host). Result: `{changed, fence}`.
    #[serde(rename = "fences.setBounds")]
    FencesSetBounds { fence: String, rect: Rect },
    /// Keep the relative position, place the fence on another monitor. Result: `{changed, fence}`.
    #[serde(rename = "fences.moveToMonitor")]
    FencesMoveToMonitor { fence: String, monitor: String },
    /// Per-fence option; `prop` and `value` use the same names and values as [`FenceDto`]
    /// (`iconSize` 32|48|64|96, `spacing`, `autoHeight`, `locked`, `excludeFromQuickHide`,
    /// `opacity` default|clear|solid, `tint` "#RRGGBB"|null, `titleColor`, `titleSize`,
    /// `layout` icons|list|details, `sort`, `reverse`, `groupByDate`, `labelLines`,
    /// `portalNavigate`, `portalTitleIcon`). Result: `{changed, fence}`.
    #[serde(rename = "fences.setOption")]
    FencesSetOption {
        fence: String,
        prop: String,
        value: Value,
    },
    /// Roll up (`true`) or expand (`false`). Result: `{changed, fence}`.
    #[serde(rename = "fences.roll")]
    FencesRoll { fence: String, rolled: bool },
    /// Dock to the top edge of its monitor (rolled up, expands on hover). Result: `{changed, fence}`.
    #[serde(rename = "fences.dockTop")]
    FencesDockTop { fence: String },
    /// Merge `fence` into `into` as a tab. Result: `{changed, host: FenceDto}`.
    #[serde(rename = "fences.merge")]
    FencesMerge { fence: String, into: String },
    /// Split a tab out into its own window. Result: `{changed, fence: FenceDto}`.
    #[serde(rename = "fences.detach")]
    FencesDetach { tab: String },
    /// Quick-hide (`false`) or show (`true`) every fence. Result: `{changed, visible}`.
    #[serde(rename = "fences.setVisible")]
    FencesSetVisible { visible: bool },
    /// Open the Settings window on this fence's options page. Result: `{changed: true}`.
    #[serde(rename = "fences.openOptions")]
    FencesOpenOptions { fence: String },

    // ---- items ----------------------------------------------------------------------
    /// Move items (by id, full path, or unique display name) into a fence. Moving into or out of
    /// a folder portal moves the real files. A batch of [`BIG_MOVE_SNAPSHOT_ITEMS`] or more
    /// desktop items into a virtual fence is preceded by an automatic layout snapshot
    /// (`snapshotId`). Result: `{changed, moved: n, to: uuid, snapshotId?}`.
    #[serde(rename = "items.move")]
    ItemsMove { items: Vec<String>, to: String },
    /// Rename the file behind an item (a real rename on disk, like F2 in a fence). With
    /// `keepExt` (default) the current extension is kept unless `name` already ends with it;
    /// `keepExt: false` uses `name` verbatim. Folders are always renamed verbatim. Namespace
    /// items (This PC, Recycle Bin) are `unsupported`; a name that exists already is
    /// `invalid_value`. Result: `{changed, item: ItemDto}`.
    #[serde(rename = "items.rename")]
    ItemsRename {
        item: String,
        name: String,
        #[serde(default = "default_true")]
        keep_ext: bool,
    },

    // ---- settings / rules / snapshots ------------------------------------------------
    /// Set one value at a dotted camelCase path (`peek.enabled`, `quickHide.enabled`,
    /// `iconSize`); `path: ""` replaces the whole `Settings` object. No snapshot is taken
    /// (snapshots hold layouts, not settings); keep the old `settings.get` output to undo. A
    /// value the app had to keep unchanged (e.g. `hideRealIcons` when Explorer refused) is
    /// reported in `warning`. Result: `{changed, settings: Settings}`.
    #[serde(rename = "settings.patch")]
    SettingsPatch { path: String, value: Value },
    /// Replace the whole rule set. No snapshot is taken (snapshots hold layouts, not rules);
    /// keep the old `rules.get` output to undo. Result: `{changed, rules: RuleListDto}`.
    #[serde(rename = "rules.set")]
    RulesSet {
        rules: pecofence_core::rules::RuleSet,
    },
    /// Append (or insert at `index`) a rule; `target` is a fence selector or `inbox`; `allOf`
    /// needs at least one condition. Result: `{changed, rule: RuleEntry}`.
    #[serde(rename = "rules.add")]
    RulesAdd {
        name: String,
        target: String,
        all_of: Vec<pecofence_core::rules::Cond>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
    },
    /// `rule` = rule id, 0-based index, or name. Result: `{changed, removed: uuid}`.
    #[serde(rename = "rules.remove")]
    RulesRemove { rule: String },
    /// Result: `{changed, rule: RuleEntry}`.
    #[serde(rename = "rules.enable")]
    RulesEnable { rule: String, enabled: bool },
    /// Reorder: place `rule` at 0-based `to`. Result: `{changed, rules: RuleListDto}`.
    #[serde(rename = "rules.move")]
    RulesMove { rule: String, to: usize },
    /// Re-file every desktop item through the rules now. With `dryRun` nothing moves and no
    /// snapshot is taken; `moves` lists what would happen. Result:
    /// `{changed, dryRun, moved: n, moves: [PlannedMoveDto], snapshotId?}`.
    #[serde(rename = "rules.apply")]
    RulesApply {
        #[serde(default)]
        dry_run: bool,
    },
    /// Result: `{changed, snapshot: SnapshotDto}`.
    #[serde(rename = "snapshots.save")]
    SnapshotsSave { name: String },
    /// `id` = snapshot id (or unique prefix) or name. A snapshot of the current layout is
    /// taken first unless the list is full of user snapshots (then `warning` says so and
    /// `snapshotId` is absent). Result: `{changed, restored: uuid, snapshotId?}`.
    #[serde(rename = "snapshots.restore")]
    SnapshotsRestore { id: String },
    /// Result: `{changed, deleted: uuid}`.
    #[serde(rename = "snapshots.delete")]
    SnapshotsDelete { id: String },

    // ---- configuration files -----------------------------------------------------------
    /// Write the complete configuration (settings, rules, layouts, items, snapshots) as
    /// pretty JSON to an absolute `path` (overwritten). Result: `{path, bytes}`.
    #[serde(rename = "config.export")]
    ConfigExport { path: String },
    /// Replace the configuration with the file at `path` (a `config export` file or a
    /// `config.json`); the current layout is snapshotted first. Result:
    /// `{changed, imported: path, snapshotId?}`.
    #[serde(rename = "config.import")]
    ConfigImport { path: String },
    /// Daily backups the app keeps beside `config.json` (newest first). Result: `[BackupDto]`.
    #[serde(rename = "backups.list")]
    BackupsList,
    /// Restore one of the files from `backups.list` (its exact `path`); the current layout is
    /// snapshotted first. Result: `{changed, restored: path, snapshotId?}`.
    #[serde(rename = "backups.restore")]
    BackupsRestore { path: String },

    // ---- other ----------------------------------------------------------------------
    /// Float every fence above other windows (Peek). Result: `{changed}`.
    #[serde(rename = "peek.start")]
    PeekStart,
    #[serde(rename = "peek.end")]
    PeekEnd,
    /// Open the Settings window. Result: `{changed: true}`.
    #[serde(rename = "settings.openUi")]
    SettingsOpenUi,

    // ---- events ---------------------------------------------------------------------
    /// Turn the connection into an event stream: the first reply is
    /// `{subscribed: true, fence?, events}`; every later line is an `ok` response whose
    /// `result` is an [`EventDto`], until the client closes the pipe. `fence` limits item events
    /// to items entering or leaving that fence and fence events to that fence; `events` limits
    /// the kinds (see [`EVENT_NAMES`]; `heartbeat` is always sent). At most
    /// [`MAX_SUBSCRIBERS`] streams at a time (`limit_reached`).
    #[serde(rename = "events.subscribe")]
    EventsSubscribe {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fence: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        events: Option<Vec<String>>,
    },
}

fn default_true() -> bool {
    true
}

/// `items.move` batches of at least this many desktop items get an automatic layout snapshot.
pub const BIG_MOVE_SNAPSHOT_ITEMS: usize = 20;

/// Concurrent `events.subscribe` streams the app serves.
pub const MAX_SUBSCRIBERS: usize = 4;

impl Method {
    /// Wire name (`fences.create`).
    pub fn name(&self) -> String {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            _ => String::new(),
        }
    }

    /// Whether a successful call may have changed persistent state (the server saves the
    /// configuration before replying to these).
    pub fn is_mutation(&self) -> bool {
        !matches!(
            self,
            Method::StatusGet
                | Method::MonitorsList
                | Method::FencesList
                | Method::FencesGet { .. }
                | Method::ItemsList { .. }
                | Method::SettingsGet { .. }
                | Method::RulesGet
                | Method::RulesApply { dry_run: true }
                | Method::SnapshotsList
                | Method::BackupsList
                | Method::ConfigExport { .. }
                | Method::FencesOpenOptions { .. }
                | Method::SettingsOpenUi
                | Method::PeekStart
                | Method::PeekEnd
                | Method::EventsSubscribe { .. }
        )
    }

    /// Whether a successful reply is followed by a stream of further lines on the same
    /// connection (see [`Method::EventsSubscribe`]).
    pub fn is_stream(&self) -> bool {
        matches!(self, Method::EventsSubscribe { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
    /// The command was applied but something secondary failed (e.g. the config file could not
    /// be written); the CLI prints it alongside the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

impl Response {
    pub fn ok(result: Value) -> Self {
        Self {
            ok: true,
            result: Some(result),
            error: None,
            warning: None,
        }
    }

    pub fn err(error: IpcError) -> Self {
        Self {
            ok: false,
            result: None,
            error: Some(error),
            warning: None,
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warning = Some(warning.into());
        self
    }
}

impl From<Result<Value, IpcError>> for Response {
    fn from(r: Result<Value, IpcError>) -> Self {
        match r {
            Ok(v) => Response::ok(v),
            Err(e) => Response::err(e),
        }
    }
}

/// Stable, machine-readable failure classes. The CLI maps them to exit codes (see
/// [`ErrorCode::exit_code`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// No PecoFence instance is listening (CLI-side).
    NotRunning,
    /// No reply within `--timeout` (CLI-side); the app is probably inside a menu or dialog.
    Timeout,
    /// Too many concurrent CLI connections.
    Busy,
    /// Malformed request, unknown method, missing or mistyped parameter.
    Usage,
    /// CLI and app speak different protocol versions.
    VersionMismatch,
    FenceNotFound,
    /// Several fences match the selector; `details.candidates` lists them.
    AmbiguousFence,
    ItemNotFound,
    AmbiguousItem,
    RuleNotFound,
    SnapshotNotFound,
    /// A parameter is out of range or not one of the allowed values (`details.allowed`).
    InvalidValue,
    /// `settings.*` path does not exist in `Settings`.
    InvalidPath,
    /// The patched object did not deserialize / validate (`details.expected`).
    ValidationFailed,
    /// e.g. 64 fences or 20 snapshots already exist.
    LimitReached,
    /// The operation does not apply to this target (inbox fence, hosted tab…).
    Unsupported,
    Internal,
}

impl ErrorCode {
    /// Process exit code the CLI uses: 3 not running, 4 timeout, 2 usage, otherwise 1.
    pub fn exit_code(self) -> i32 {
        match self {
            ErrorCode::NotRunning => 3,
            ErrorCode::Timeout => 4,
            ErrorCode::Usage => 2,
            _ => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "describe", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct IpcError {
    pub code: ErrorCode,
    pub message: String,
    /// What to do next, phrased for the caller (often a command to run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// Machine-readable extras: `candidates`, `allowed`, `expected`, …
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl IpcError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
            details: None,
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn usage(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Usage, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    pub fn invalid_value(message: impl Into<String>, allowed: &[&str]) -> Self {
        Self::new(ErrorCode::InvalidValue, message)
            .details(serde_json::json!({ "allowed": allowed }))
    }

    /// From a selector failure over fences.
    pub fn fence(sel: &str, err: selector::SelectorError) -> Self {
        match err {
            selector::SelectorError::NotFound => {
                Self::new(ErrorCode::FenceNotFound, format!("no fence matches {sel:?}"))
                    .hint("Run `pecofence-cli fence list` and use the id or exact title")
            }
            selector::SelectorError::Ambiguous(c) => Self::new(
                ErrorCode::AmbiguousFence,
                format!("{} fences match {sel:?}", c.len()),
            )
            .hint("Use the full id or the exact title")
            .details(serde_json::json!({
                "candidates": c.iter().map(|(id, title)| serde_json::json!({"id": id, "title": title})).collect::<Vec<_>>()
            })),
        }
    }
}

impl std::fmt::Display for IpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(h) = &self.hint {
            write!(f, " ({h})")?;
        }
        Ok(())
    }
}

impl std::error::Error for IpcError {}

/// `{"changed": …, "snapshotId": …, …extra}` — the shape every mutating method returns.
pub fn mutation(changed: bool, snapshot_id: Option<uuid::Uuid>, extra: Value) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("changed".into(), Value::Bool(changed));
    if let Some(id) = snapshot_id {
        map.insert("snapshotId".into(), Value::String(id.to_string()));
    }
    if let Value::Object(extra) = extra {
        map.extend(extra);
    }
    Value::Object(map)
}

/// `peek.enabled` → `/peek/enabled`; `""` → `""` (the root). Keys are used verbatim, so they
/// must be the camelCase names of the JSON (`quickHide`, not `quick_hide`).
pub fn dotted_to_pointer(path: &str) -> String {
    let path = path.trim().trim_matches('.');
    if path.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for seg in path.split('.') {
        out.push('/');
        out.push_str(&seg.replace('~', "~0").replace('/', "~1"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pecofence_core::rules::{Cond, RuleSet, TypeCategory};

    fn every_method() -> Vec<Method> {
        vec![
            Method::StatusGet,
            Method::MonitorsList,
            Method::FencesList,
            Method::FencesGet {
                fence: "Work".into(),
            },
            Method::ItemsList { fence: None },
            Method::ItemsList {
                fence: Some("Work".into()),
            },
            Method::SettingsGet { path: None },
            Method::SettingsGet {
                path: Some("peek.enabled".into()),
            },
            Method::RulesGet,
            Method::SnapshotsList,
            Method::FencesCreate {
                title: Some("Work".into()),
                rect: Some(Rect {
                    x: 10,
                    y: 20,
                    w: 300,
                    h: 200,
                }),
                monitor: None,
                portal: None,
            },
            Method::FencesDelete {
                fence: "Work".into(),
            },
            Method::FencesRename {
                fence: "Work".into(),
                title: "Play".into(),
            },
            Method::FencesSetBounds {
                fence: "Work".into(),
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: 1,
                    h: 1,
                },
            },
            Method::FencesMoveToMonitor {
                fence: "Work".into(),
                monitor: "m".into(),
            },
            Method::FencesSetOption {
                fence: "Work".into(),
                prop: "iconSize".into(),
                value: serde_json::json!(48),
            },
            Method::FencesRoll {
                fence: "Work".into(),
                rolled: true,
            },
            Method::FencesDockTop {
                fence: "Work".into(),
            },
            Method::FencesMerge {
                fence: "a".into(),
                into: "b".into(),
            },
            Method::FencesDetach { tab: "a".into() },
            Method::FencesSetVisible { visible: false },
            Method::FencesOpenOptions {
                fence: "Work".into(),
            },
            Method::ItemsMove {
                items: vec!["C:\\x.pdf".into()],
                to: "Docs".into(),
            },
            Method::ItemsRename {
                item: "C:\\x.pdf".into(),
                name: "report".into(),
                keep_ext: true,
            },
            Method::ItemsRename {
                item: "a".into(),
                name: "b.txt".into(),
                keep_ext: false,
            },
            Method::EventsSubscribe {
                fence: None,
                events: None,
            },
            Method::EventsSubscribe {
                fence: Some("inbox".into()),
                events: Some(vec!["item.added".into()]),
            },
            Method::SettingsPatch {
                path: "peek.enabled".into(),
                value: Value::Bool(false),
            },
            Method::RulesSet {
                rules: RuleSet::default(),
            },
            Method::RulesAdd {
                name: "PDFs".into(),
                target: "Docs".into(),
                all_of: vec![
                    Cond::Ext(vec![".pdf".into()]),
                    Cond::Type(vec![TypeCategory::Documents]),
                ],
                index: None,
            },
            Method::RulesRemove { rule: "0".into() },
            Method::RulesEnable {
                rule: "PDFs".into(),
                enabled: false,
            },
            Method::RulesMove {
                rule: "PDFs".into(),
                to: 2,
            },
            Method::RulesApply { dry_run: false },
            Method::RulesApply { dry_run: true },
            Method::SnapshotsSave { name: "x".into() },
            Method::SnapshotsRestore { id: "x".into() },
            Method::SnapshotsDelete { id: "x".into() },
            Method::ConfigExport {
                path: "C:\\tmp\\x.json".into(),
            },
            Method::ConfigImport {
                path: "C:\\tmp\\x.json".into(),
            },
            Method::BackupsList,
            Method::BackupsRestore {
                path: "C:\\tmp\\x.json".into(),
            },
            Method::PeekStart,
            Method::PeekEnd,
            Method::SettingsOpenUi,
        ]
    }

    #[test]
    fn every_method_round_trips_inside_a_request() {
        for m in every_method() {
            let req = Request::new(m.clone()).with_timeout(1234);
            let text = serde_json::to_string(&req).unwrap();
            assert!(!text.contains('\n'));
            let back: Request = serde_json::from_str(&text).unwrap();
            assert_eq!(back.method, m, "{text}");
            assert_eq!(back.timeout_ms, 1234);
            assert_eq!(back.protocol, PROTOCOL_VERSION);
        }
    }

    #[test]
    fn unit_variants_omit_params_and_accept_their_absence() {
        let text = serde_json::to_string(&Request::new(Method::StatusGet)).unwrap();
        assert_eq!(
            text,
            r#"{"protocol":1,"timeoutMs":15000,"method":"status.get"}"#
        );
        let back: Request = serde_json::from_str(r#"{"protocol":1,"method":"peek.end"}"#).unwrap();
        assert_eq!(back.method, Method::PeekEnd);
        assert_eq!(back.timeout_ms, DEFAULT_TIMEOUT_MS);
    }

    #[test]
    fn method_names_are_dotted() {
        assert_eq!(
            Method::FencesCreate {
                title: None,
                rect: None,
                monitor: None,
                portal: None
            }
            .name(),
            "fences.create"
        );
        assert_eq!(Method::StatusGet.name(), "status.get");
        let names: std::collections::HashSet<String> =
            every_method().iter().map(Method::name).collect();
        assert!(names.iter().all(|n| n.contains('.')), "{names:?}");
    }

    #[test]
    fn rules_apply_and_rename_defaults_are_optional_on_the_wire() {
        // An older CLI sends `rules.apply` without params (the server adds `{}`).
        let back: Request =
            serde_json::from_str(r#"{"protocol":1,"method":"rules.apply","params":{}}"#).unwrap();
        assert_eq!(back.method, Method::RulesApply { dry_run: false });
        let back: Request = serde_json::from_str(
            r#"{"protocol":1,"method":"items.rename","params":{"item":"a","name":"b"}}"#,
        )
        .unwrap();
        assert_eq!(
            back.method,
            Method::ItemsRename {
                item: "a".into(),
                name: "b".into(),
                keep_ext: true
            }
        );
        let text = serde_json::to_string(&Method::RulesApply { dry_run: true }).unwrap();
        assert!(text.contains(r#""dryRun":true"#), "{text}");
    }

    #[test]
    fn only_subscribe_streams() {
        for m in every_method() {
            assert_eq!(
                m.is_stream(),
                matches!(m, Method::EventsSubscribe { .. }),
                "{m:?}"
            );
            if m.is_stream() {
                assert!(!m.is_mutation());
            }
        }
        assert!(EVENT_NAMES.contains(&"heartbeat"));
        assert!(EVENT_NAMES.contains(&"item.added"));
    }

    #[test]
    fn read_only_methods_are_not_mutations() {
        assert!(!Method::FencesList.is_mutation());
        assert!(!Method::SettingsOpenUi.is_mutation());
        assert!(Method::RulesApply { dry_run: false }.is_mutation());
        assert!(!Method::RulesApply { dry_run: true }.is_mutation());
        assert!(
            Method::FencesSetOption {
                fence: "a".into(),
                prop: "locked".into(),
                value: Value::Bool(true)
            }
            .is_mutation()
        );
    }

    #[test]
    fn response_envelopes_round_trip() {
        let ok = Response::ok(serde_json::json!({"changed": true})).with_warning("not saved");
        let text = serde_json::to_string(&ok).unwrap();
        assert!(!text.contains("error"));
        assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), ok);

        let err = Response::err(
            IpcError::new(ErrorCode::FenceNotFound, "no fence matches \"x\"")
                .hint("fence list")
                .details(serde_json::json!({"candidates": []})),
        );
        let text = serde_json::to_string(&err).unwrap();
        assert!(text.contains(r#""code":"fence_not_found""#));
        assert_eq!(serde_json::from_str::<Response>(&text).unwrap(), err);
    }

    #[test]
    fn exit_codes() {
        assert_eq!(ErrorCode::NotRunning.exit_code(), 3);
        assert_eq!(ErrorCode::Timeout.exit_code(), 4);
        assert_eq!(ErrorCode::Usage.exit_code(), 2);
        assert_eq!(ErrorCode::Internal.exit_code(), 1);
        assert_eq!(ErrorCode::FenceNotFound.exit_code(), 1);
    }

    #[test]
    fn mutation_merges_extras() {
        let id = uuid::Uuid::nil();
        let v = mutation(true, Some(id), serde_json::json!({"moved": 3}));
        assert_eq!(
            v,
            serde_json::json!({"changed": true, "snapshotId": id.to_string(), "moved": 3})
        );
        assert_eq!(
            mutation(false, None, Value::Null),
            serde_json::json!({"changed": false})
        );
    }

    #[test]
    fn dotted_paths_become_json_pointers() {
        assert_eq!(dotted_to_pointer(""), "");
        assert_eq!(dotted_to_pointer("iconSize"), "/iconSize");
        assert_eq!(dotted_to_pointer("peek.enabled"), "/peek/enabled");
        assert_eq!(dotted_to_pointer(".peek.enabled."), "/peek/enabled");
        assert_eq!(dotted_to_pointer("a/b"), "/a~1b");
    }
}
