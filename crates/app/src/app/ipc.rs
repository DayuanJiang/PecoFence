//! UI-thread half of the CLI IPC (`pecofence-cli`): drains the requests the pipe threads queued
//! (`ipc_server.rs`), runs each `Method` against the `App` exactly like a queued `Command`,
//! saves the configuration, and hands the reply back to the waiting connection thread.
//!
//! Every message this module emits is English (the CLI's contract); the localized `i18n`
//! strings are for the GUI only.

use super::*;
use crate::ipc_server::Pending;
use fence_options::{layout_name, parse_fence_prop, sort_name};
use items::RenameError;
use pecofence_core::geometry;
use pecofence_core::rules::{self, Rule};
use pecofence_core::{AssignedBy, Fence, Item, ItemSourceSpec, Settings, Snapshot};
use pecofence_ipc::selector::{self, SelectorError};
use pecofence_ipc::{
    BIG_MOVE_SNAPSHOT_ITEMS, BackupDto, EVENT_HEARTBEAT_SECS, EVENT_NAMES, ErrorCode, EventDto,
    FenceDto, IpcError, ItemDto, MAX_SUBSCRIBERS, Method, MonitorDto, PROTOCOL_VERSION,
    PlannedMoveDto, PortalDto, Rect, Response, RuleEntry, RuleListDto, SnapshotDto, StatusDto,
    dotted_to_pointer, mutation,
};
use serde_json::{Value, json};
use std::sync::mpsc;
use uuid::Uuid;

/// Poll interval of the event stream while at least one `events.subscribe` client is connected.
const EVENT_TICK_MS: u32 = 500;
/// `FenceDto` fields left out of `fence.changed` detection: they follow the window during
/// animations and item events already report membership.
const EVENT_IGNORED_FENCE_FIELDS: &[&str] = &["windowRect", "itemCount"];

/// One `events.subscribe` client: the connection thread's reply channel plus its filter.
pub(super) struct IpcSubscriber {
    reply: mpsc::Sender<Response>,
    fence: Option<FenceId>,
    events: Option<Vec<String>>,
    seq: u64,
}

/// What an item looked like at the last tick (cheap to rebuild; the full [`ItemDto`] is only
/// produced for the items an event is about).
#[derive(Clone, Debug, PartialEq)]
pub(super) struct ItemSig {
    fence: FenceId,
    fence_title: String,
    name: String,
    path: Option<String>,
    is_folder: bool,
    is_namespace: bool,
}

/// The fences and items as of the last tick; diffed against the present to produce events.
#[derive(Default)]
pub(super) struct EventState {
    fences: HashMap<FenceId, FenceDto>,
    items: HashMap<ItemId, ItemSig>,
}

/// Newest `auto-cli-*` snapshots kept; older ones go when a new one is taken.
const AUTO_SNAPSHOTS_KEPT: usize = 3;
pub(super) const AUTO_SNAPSHOT_PREFIX: &str = "auto-cli-";
/// Longest fence title, rule name or snapshot name accepted over IPC (chars).
pub(super) const MAX_NAME_CHARS: usize = 256;
/// Warning attached when the snapshot list is full of user snapshots.
const SNAPSHOT_LIMIT_WARNING: &str = "snapshot limit reached; no automatic snapshot taken";
/// Shortest request lifetime honoured by `drain_ipc` (a `--timeout 0` request still runs once).
pub(super) const MIN_EXPIRY_MS: u64 = 100;
/// Fence selector alias for the inbox fence, independent of the UI language.
pub(super) const INBOX_ALIAS: &str = "inbox";

type IpcResult = std::result::Result<Value, IpcError>;

/// A trimmed absolute path, or `invalid_value` (relative paths would resolve against the app's
/// working directory, not the caller's).
fn absolute_path(path: &str) -> std::result::Result<std::path::PathBuf, IpcError> {
    let p = std::path::PathBuf::from(path.trim());
    if path.trim().is_empty() || !p.is_absolute() {
        return Err(IpcError::new(
            ErrorCode::InvalidValue,
            format!("{path:?} is not an absolute path"),
        )
        .hint("Pass a full path such as C:\\Users\\<you>\\Desktop\\pecofence.json"));
    }
    Ok(p)
}

/// Trimmed `value` as a title / rule name / snapshot name: `invalid_value` when blank or over
/// [`MAX_NAME_CHARS`].
pub(super) fn checked_name(what: &str, value: &str) -> std::result::Result<String, IpcError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(IpcError::invalid_value(
            format!("{what} must not be empty"),
            &["<non-empty string>"],
        ));
    }
    let len = trimmed.chars().count();
    if len > MAX_NAME_CHARS {
        return Err(IpcError::invalid_value(
            format!("{what} is {len} characters long; at most {MAX_NAME_CHARS} are allowed"),
            &[&format!("<string of 1 to {MAX_NAME_CHARS} characters>")],
        )
        .details(json!({ "max": MAX_NAME_CHARS, "length": len })));
    }
    Ok(trimmed.to_string())
}

fn work_area_json(w: &geometry::WorkArea) -> Value {
    json!({
        "monitor": w.device_path,
        "x": w.left,
        "y": w.top,
        "w": w.right - w.left,
        "h": w.bottom - w.top,
        "dpi": w.dpi,
    })
}

/// Pure: checks a `fences.create` / `fences.setBounds` rectangle and returns the work area its
/// centre falls in. Rejects (`invalid_value`) non-positive or overflowing sizes, a centre that
/// is on no connected monitor's work area, and a width/height under the fence minimum or over
/// the maximum (DIP limits scaled by that monitor's DPI).
pub(super) fn validate_rect(
    rect: Rect,
    work_areas: &[geometry::WorkArea],
) -> std::result::Result<geometry::WorkArea, IpcError> {
    if rect.w <= 0 || rect.h <= 0 {
        return Err(IpcError::invalid_value(
            "rect width and height must be positive",
            &["w > 0", "h > 0"],
        ));
    }
    let (Some(right), Some(bottom)) = (rect.x.checked_add(rect.w), rect.y.checked_add(rect.h))
    else {
        return Err(IpcError::invalid_value(
            "rect coordinates overflow: x + w and y + h must fit in a 32-bit integer",
            &["x + w <= 2147483647", "y + h <= 2147483647"],
        ));
    };
    // Midpoint without overflow: both ends are valid i32 now.
    let cx = rect.x + (right - rect.x) / 2;
    let cy = rect.y + (bottom - rect.y) / 2;
    let Some(work) = work_areas
        .iter()
        .find(|w| cx >= w.left && cx < w.right && cy >= w.top && cy < w.bottom)
    else {
        let areas: Vec<Value> = work_areas.iter().map(work_area_json).collect();
        let ids: Vec<String> = work_areas
            .iter()
            .map(|w| format!("{}: {}", w.device_path, work_area_json(w)))
            .collect();
        let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
        return Err(IpcError::invalid_value(
            "rect centre is outside every monitor's work area",
            &ids,
        )
        .hint("Run `pecofence-cli monitor list`; the rect is in physical pixels of the virtual screen")
        .details(json!({ "centre": { "x": cx, "y": cy }, "workAreas": areas })));
    };
    let scale = work.scale();
    let min_w = (geometry::MIN_W_DIP * scale).ceil() as i32;
    let min_h = (geometry::MIN_H_DIP * scale).ceil() as i32;
    let max = (geometry::MAX_DIP * scale).floor() as i32;
    if rect.w < min_w || rect.h < min_h || rect.w > max || rect.h > max {
        return Err(IpcError::invalid_value(
            format!(
                "rect size {}x{} is outside the allowed range on {} ({}x{} to {}x{} px at {} dpi)",
                rect.w, rect.h, work.device_path, min_w, min_h, max, max, work.dpi
            ),
            &[
                &format!("{min_w} <= w <= {max}"),
                &format!("{min_h} <= h <= {max}"),
            ],
        )
        .details(json!({
            "minW": min_w, "minH": min_h, "max": max, "dpi": work.dpi, "monitor": work.device_path
        })));
    }
    Ok(work.clone())
}

fn to_json<T: serde::Serialize>(v: &T) -> IpcResult {
    serde_json::to_value(v).map_err(|e| IpcError::internal(format!("serialization failed: {e}")))
}

fn rect_of(r: RECT) -> Rect {
    Rect {
        x: r.left,
        y: r.top,
        w: r.right - r.left,
        h: r.bottom - r.top,
    }
}

fn rect_to_win(r: Rect) -> RECT {
    RECT {
        left: r.x,
        top: r.y,
        right: r.x + r.w,
        bottom: r.y + r.h,
    }
}

fn unsupported(message: impl Into<String>) -> IpcError {
    IpcError::new(ErrorCode::Unsupported, message)
}

/// `invalid_path` naming the keys that do exist at the deepest valid prefix of `path`.
fn invalid_path(root: &Value, path: &str) -> IpcError {
    let mut node = root;
    let mut valid = String::new();
    for seg in path.trim().trim_matches('.').split('.') {
        match node.get(seg) {
            Some(next) => {
                node = next;
                if !valid.is_empty() {
                    valid.push('.');
                }
                valid.push_str(seg);
            }
            None => break,
        }
    }
    let keys: Vec<&str> = node
        .as_object()
        .map(|o| o.keys().map(String::as_str).collect())
        .unwrap_or_default();
    let scope = if valid.is_empty() {
        "Top-level keys".to_string()
    } else {
        format!("Keys under {valid:?}")
    };
    IpcError::new(ErrorCode::InvalidPath, format!("no setting at {path:?}"))
        .hint(format!("{scope}: {}", keys.join(", ")))
        .details(json!({ "allowed": keys }))
}

/// Pure: `current` with the value at the dotted camelCase `path` replaced (`""` = the whole
/// object), re-validated as a `Settings`.
pub(super) fn patch_settings_path(
    current: &Settings,
    path: &str,
    value: Value,
) -> std::result::Result<Settings, IpcError> {
    let mut root = to_json(current)?;
    let pointer = dotted_to_pointer(path);
    if pointer.is_empty() {
        if !value.is_object() {
            return Err(IpcError::new(
                ErrorCode::ValidationFailed,
                "the whole settings object must be a JSON object",
            )
            .hint("Run `pecofence-cli settings get` to see the shape"));
        }
        root = value;
    } else {
        let Some(slot) = root.pointer_mut(&pointer) else {
            return Err(invalid_path(&root, path));
        };
        *slot = value;
    }
    let settings: Settings = serde_json::from_value(root).map_err(|e| {
        IpcError::new(
            ErrorCode::ValidationFailed,
            format!("settings rejected: {e}"),
        )
        .details(json!({ "expected": e.to_string() }))
    })?;
    if !matches!(settings.icon_size, 32 | 48 | 64 | 96) {
        return Err(IpcError::new(
            ErrorCode::ValidationFailed,
            format!(
                "iconSize {} is not one of 32, 48, 64, 96",
                settings.icon_size
            ),
        )
        .details(json!({ "expected": [32, 48, 64, 96] })));
    }
    Ok(settings)
}

/// Keeps the newest `keep` `auto-cli-*` snapshots (the list is chronological); returns the ids
/// removed.
pub(super) fn prune_auto_snapshots(snapshots: &mut Vec<Snapshot>, keep: usize) -> Vec<Uuid> {
    let auto: Vec<Uuid> = snapshots
        .iter()
        .filter(|s| s.name.starts_with(AUTO_SNAPSHOT_PREFIX))
        .map(|s| s.id)
        .collect();
    let excess = auto.len().saturating_sub(keep);
    let evict: Vec<Uuid> = auto.into_iter().take(excess).collect();
    if !evict.is_empty() {
        snapshots.retain(|s| !evict.contains(&s.id));
    }
    evict
}

fn auto_snapshot_name() -> String {
    format!(
        "{AUTO_SNAPSHOT_PREFIX}{}",
        pecofence_platform::fileinfo::format_local_timestamp(pecofence_core::now_unix())
    )
}

/// Pure half of [`App::auto_snapshot`]: prunes `auto-cli-*` entries down to
/// `AUTO_SNAPSHOTS_KEPT - 1` (making room for the one about to be taken) and reports whether
/// a slot under `MAX_SNAPSHOTS` is free. `false` means the list is full of snapshots the user
/// saved, which an automatic one must never evict.
pub(super) fn auto_snapshot_slot(snapshots: &mut Vec<Snapshot>) -> bool {
    prune_auto_snapshots(snapshots, AUTO_SNAPSHOTS_KEPT.saturating_sub(1));
    snapshots.len() < pecofence_core::MAX_SNAPSHOTS
}

/// How long a queued request stays runnable: the client's timeout, but never under
/// [`MIN_EXPIRY_MS`] (`--timeout 0` must still run once; the pipe round trip alone takes a
/// few milliseconds).
pub(super) fn request_expiry(timeout_ms: u32) -> Duration {
    Duration::from_millis(u64::from(timeout_ms).max(MIN_EXPIRY_MS))
}

/// Whether a fence selector is the language-independent inbox alias.
pub(super) fn is_inbox_alias(sel: &str) -> bool {
    sel.trim().eq_ignore_ascii_case(INBOX_ALIAS)
}

/// `rules.add` needs at least one condition; an empty `allOf` would match everything.
pub(super) fn check_rule_conditions(
    all_of: &[pecofence_core::Cond],
) -> std::result::Result<(), IpcError> {
    if all_of.is_empty() {
        return Err(IpcError::invalid_value(
            "a rule needs at least one condition in allOf",
            &["allOf: [<condition>, ...]"],
        )
        .hint("Run `pecofence-cli describe rules.add` for the condition shapes"));
    }
    Ok(())
}

/// Response with `warning` appended (several are joined with `; `).
fn add_warning(mut response: Response, warning: &str) -> Response {
    response.warning = Some(match response.warning.take() {
        Some(existing) => format!("{existing}; {warning}"),
        None => warning.to_string(),
    });
    response
}

/// Top-level camelCase keys whose value in `applied` differs from `requested` (the settings
/// `apply_settings` kept at their old value).
pub(super) fn rolled_back_settings(requested: &Settings, applied: &Settings) -> Vec<String> {
    let (Ok(Value::Object(want)), Ok(Value::Object(got))) = (
        serde_json::to_value(requested),
        serde_json::to_value(applied),
    ) else {
        return Vec::new();
    };
    want.iter()
        .filter(|(k, v)| got.get(*k) != Some(v))
        .map(|(k, _)| k.clone())
        .collect()
}

fn snapshot_dto(s: &Snapshot) -> SnapshotDto {
    SnapshotDto {
        id: s.id,
        name: s.name.clone(),
        ts: s.ts,
        fence_count: s.layouts.iter().map(|l| l.fences.len()).sum(),
    }
}

impl App {
    /// Runs every queued CLI request (called from the command pump). A request older than the
    /// client's own timeout is dropped unexecuted so an agent's retry never runs twice; the
    /// check happens only before execution.
    pub(super) fn drain_ipc(&mut self) {
        let batch: Vec<Pending> = self
            .ipc_pending
            .lock()
            .map(|mut q| q.drain(..).collect())
            .unwrap_or_default();
        for Pending {
            request,
            reply,
            arrived,
        } in batch
        {
            let method = request.method.name();
            if arrived.elapsed() > request_expiry(request.timeout_ms) {
                tracing::debug!(
                    target: "pecofence::ipc",
                    method,
                    waited_ms = arrived.elapsed().as_millis() as u64,
                    "request expired before it could run; dropped"
                );
                continue;
            }
            let started = Instant::now();
            let mut response = match &request.method {
                // Streams need the connection's channel; every other method is stateless.
                Method::EventsSubscribe { fence, events } => {
                    self.ipc_warnings.clear();
                    self.ipc_subscribe(fence.as_deref(), events.as_deref(), reply.clone())
                        .into()
                }
                other => self.handle_ipc(other),
            };
            if request.method.is_mutation() && response.ok {
                // Persist before replying so the caller can rely on the file; the deferred
                // save the mutation scheduled is redundant now.
                window::kill_timer(self.control.hwnd(), TIMER_SAVE);
                if self.state.is_dirty() && !self.state.save_if_dirty() {
                    response = add_warning(response, "applied but not saved: see the log");
                }
            }
            let spent = started.elapsed();
            if spent > Duration::from_millis(8) {
                tracing::info!(
                    target: "pecofence::ipc",
                    ms = spent.as_secs_f32() * 1000.0,
                    method,
                    "slow request"
                );
            }
            let _ = reply.send(response);
            if request.method.is_mutation() && !self.ipc_subscribers.is_empty() {
                // Push what this command changed right away instead of at the next tick.
                self.ipc_events_tick();
            }
        }
    }

    /// Runs one method; warnings the handler queued (`ipc_warnings`) ride along on success.
    pub(super) fn handle_ipc(&mut self, m: &Method) -> Response {
        self.ipc_warnings.clear();
        let mut response: Response = self.ipc_dispatch(m).into();
        if response.ok {
            for w in std::mem::take(&mut self.ipc_warnings) {
                response = add_warning(response, &w);
            }
        }
        self.ipc_warnings.clear();
        response
    }

    fn ipc_dispatch(&mut self, m: &Method) -> IpcResult {
        match m {
            Method::StatusGet => to_json(&self.status_dto()),
            Method::MonitorsList => to_json(&self.monitor_dtos()),
            Method::FencesList => {
                let list: Vec<FenceDto> = self
                    .state
                    .fences()
                    .iter()
                    .map(|f| self.fence_dto(f))
                    .collect();
                to_json(&list)
            }
            Method::FencesGet { fence } => {
                let id = self.resolve_fence(fence)?;
                to_json(&self.fence_dto(self.fence_or_err(id)?))
            }
            Method::ItemsList { fence } => {
                let only = fence
                    .as_deref()
                    .map(|sel| self.resolve_fence(sel))
                    .transpose()?;
                to_json(&self.item_dtos(only))
            }
            Method::SettingsGet { path } => {
                let root = to_json(&self.state.config.settings)?;
                let path = path.as_deref().unwrap_or("");
                let pointer = dotted_to_pointer(path);
                if pointer.is_empty() {
                    return Ok(root);
                }
                root.pointer(&pointer)
                    .cloned()
                    .ok_or_else(|| invalid_path(&root, path))
            }
            Method::RulesGet => to_json(&self.rule_list_dto()),
            Method::SnapshotsList => {
                let list: Vec<SnapshotDto> = self
                    .state
                    .config
                    .snapshots
                    .iter()
                    .map(snapshot_dto)
                    .collect();
                to_json(&list)
            }

            Method::FencesCreate {
                title,
                rect,
                monitor,
                portal,
            } => self.ipc_create_fence(
                title.as_deref(),
                *rect,
                monitor.as_deref(),
                portal.as_deref(),
            ),
            Method::FencesDelete { fence } => self.ipc_delete_fence(fence),
            Method::FencesRename { fence, title } => {
                let id = self.resolve_fence(fence)?;
                let prop = parse_fence_prop("title", &Value::String(title.clone()))?;
                let changed = self.apply_fence_prop(id, prop);
                self.push_settings_state();
                Ok(mutation(changed, None, self.fence_extra(id)?))
            }
            Method::FencesSetBounds { fence, rect } => self.ipc_set_bounds(fence, *rect),
            Method::FencesMoveToMonitor { fence, monitor } => {
                self.ipc_move_to_monitor(fence, monitor)
            }
            Method::FencesSetOption { fence, prop, value } => {
                let id = self.resolve_fence(fence)?;
                let prop = parse_fence_prop(prop, value)?;
                let changed = self.apply_fence_prop(id, prop);
                self.push_settings_state();
                Ok(mutation(changed, None, self.fence_extra(id)?))
            }
            Method::FencesRoll { fence, rolled } => {
                let id = self.resolve_fence(fence)?;
                let host = self.state.host_of(id);
                let current = self.fence_or_err(host)?.rolled_up;
                let changed = current != *rolled;
                if changed {
                    self.set_roll(host, *rolled);
                    self.push_settings_state();
                }
                Ok(mutation(changed, None, self.fence_extra(id)?))
            }
            Method::FencesDockTop { fence } => {
                let id = self.resolve_fence(fence)?;
                let host = self.state.host_of(id);
                let before = self.fence_or_err(host)?.clone();
                self.dock_to_top(host);
                let changed = self.state.fence(host) != Some(&before);
                self.push_settings_state();
                Ok(mutation(changed, None, self.fence_extra(id)?))
            }
            Method::FencesMerge { fence, into } => self.ipc_merge(fence, into),
            Method::FencesDetach { tab } => self.ipc_detach(tab),
            Method::FencesSetVisible { visible } => {
                let Some(hidden) = self.fences_hidden() else {
                    return Err(unsupported(
                        "fences cannot be hidden or shown right now: the desktop anchor is not ready",
                    )
                    .hint("Retry in a moment"));
                };
                if hidden == *visible {
                    self.toggle_all_fences();
                }
                // Only what the anchor actually did counts.
                let changed = self.fences_hidden() != Some(hidden);
                Ok(mutation(changed, None, json!({ "visible": visible })))
            }
            Method::FencesOpenOptions { fence } => {
                let id = self.resolve_fence(fence)?;
                self.handle(Command::OpenOptionsForFence(id));
                Ok(json!({ "changed": true }))
            }

            Method::ItemsMove { items, to } => self.ipc_move_items(items, to),
            Method::ItemsRename {
                item,
                name,
                keep_ext,
            } => self.ipc_rename_item(item, name, *keep_ext),
            Method::EventsSubscribe { .. } => Err(IpcError::internal(
                "events.subscribe must be handled by the request pump",
            )),

            Method::SettingsPatch { path, value } => {
                // No auto snapshot: snapshots hold layouts only and could not undo this.
                let before = self.state.config.settings.clone();
                let requested = patch_settings_path(&before, path, value.clone())?;
                self.apply_settings(requested.clone());
                let after = &self.state.config.settings;
                let changed = *after != before;
                // `apply_settings` keeps the old value when a side effect fails (e.g. Explorer
                // refused to hide the desktop icons); say so instead of silently not changing.
                let rolled_back = rolled_back_settings(&requested, after);
                if !rolled_back.is_empty() {
                    self.ipc_warnings.push(format!(
                        "PecoFence rolled back {}: the change could not be applied (see the log)",
                        rolled_back.join(", ")
                    ));
                }
                Ok(mutation(
                    changed,
                    None,
                    json!({ "settings": to_json(&self.state.config.settings)? }),
                ))
            }
            Method::RulesSet { rules } => {
                // No auto snapshot: snapshots hold layouts, not rules.
                let before = self.state.config.rules.clone();
                self.set_rules(rules.clone());
                let changed = self.state.config.rules != before;
                self.push_settings_state();
                Ok(mutation(
                    changed,
                    None,
                    json!({ "rules": to_json(&self.rule_list_dto())? }),
                ))
            }
            Method::RulesAdd {
                name,
                target,
                all_of,
                index,
            } => self.ipc_add_rule(name, target, all_of, *index),
            Method::RulesRemove { rule } => {
                let at = self.resolve_rule(rule)?;
                let removed = self.state.config.rules.list.remove(at);
                self.rules_mutated();
                Ok(mutation(true, None, json!({ "removed": removed.id })))
            }
            Method::RulesEnable { rule, enabled } => {
                let at = self.resolve_rule(rule)?;
                let changed = self.state.config.rules.list[at].enabled != *enabled;
                if changed {
                    self.state.config.rules.list[at].enabled = *enabled;
                    self.rules_mutated();
                }
                let entry = self.rule_entry(at, &self.state.config.rules.list[at]);
                Ok(mutation(changed, None, json!({ "rule": to_json(&entry)? })))
            }
            Method::RulesMove { rule, to } => {
                let from = self.resolve_rule(rule)?;
                let list = &mut self.state.config.rules.list;
                let to = (*to).min(list.len().saturating_sub(1));
                let changed = from != to;
                if changed {
                    let rule = list.remove(from);
                    list.insert(to, rule);
                    self.rules_mutated();
                }
                Ok(mutation(
                    changed,
                    None,
                    json!({ "rules": to_json(&self.rule_list_dto())? }),
                ))
            }
            Method::RulesApply { dry_run } => {
                let entries = shell::enumerate_desktop();
                let plan = self.state.plan_rules(&entries, false);
                let moves: Vec<PlannedMoveDto> =
                    plan.iter().map(|m| self.planned_move_dto(m)).collect();
                if *dry_run {
                    return Ok(json!({
                        "changed": false,
                        "dryRun": true,
                        "moved": moves.len(),
                        "moves": to_json(&moves)?,
                    }));
                }
                // Re-filing every desktop item rewrites memberships: keep a way back, but only
                // when something is going to move.
                let snapshot = (!plan.is_empty()).then(|| self.auto_snapshot()).flatten();
                let moved = self.state.apply_rules_all(&entries);
                self.refresh_all();
                self.schedule_save();
                self.push_settings_state();
                Ok(mutation(
                    moved > 0,
                    snapshot,
                    json!({ "dryRun": false, "moved": moved, "moves": to_json(&moves)? }),
                ))
            }

            Method::SnapshotsSave { name } => {
                let name = checked_name("snapshot name", name)?;
                if self.state.config.snapshots.len() >= pecofence_core::MAX_SNAPSHOTS {
                    return Err(IpcError::new(
                        ErrorCode::LimitReached,
                        format!(
                            "{} snapshots already exist (the maximum)",
                            pecofence_core::MAX_SNAPSHOTS
                        ),
                    )
                    .hint("Delete one with `pecofence-cli snapshot delete <id>` first"));
                }
                let id = self.state.save_snapshot(&name);
                self.schedule_save();
                self.push_settings_state();
                let snap = self
                    .state
                    .config
                    .snapshots
                    .iter()
                    .find(|s| s.id == id)
                    .map(snapshot_dto)
                    .ok_or_else(|| IpcError::internal("snapshot vanished after saving"))?;
                Ok(mutation(true, None, json!({ "snapshot": to_json(&snap)? })))
            }
            Method::SnapshotsRestore { id } => {
                let target = self.resolve_snapshot(id)?;
                // The backup the existing mechanism takes is this call's auto snapshot; when
                // the list is full of the user's snapshots the layout is restored without one
                // (warned) rather than evicting theirs.
                let backup = if auto_snapshot_slot(&mut self.state.config.snapshots) {
                    Some(
                        self.state
                            .restore_snapshot_with_backup(target, &auto_snapshot_name())
                            .ok_or_else(|| self.snapshot_not_found(id))?,
                    )
                } else {
                    if !self.state.restore_snapshot(target) {
                        return Err(self.snapshot_not_found(id));
                    }
                    self.ipc_warnings.push(SNAPSHOT_LIMIT_WARNING.to_string());
                    None
                };
                self.end_peek_now();
                self.relayout_from_state();
                // Portals restored with the snapshot need enumerating; stale runtime state of
                // the replaced ones is pruned on the way.
                self.refresh_portals();
                self.schedule_save();
                self.push_settings_state();
                Ok(mutation(true, backup, json!({ "restored": target })))
            }
            Method::SnapshotsDelete { id } => {
                let target = self.resolve_snapshot(id)?;
                let changed = self.state.delete_snapshot(target);
                if changed {
                    self.schedule_save();
                    self.push_settings_state();
                }
                Ok(mutation(changed, None, json!({ "deleted": target })))
            }

            Method::ConfigExport { path } => self.ipc_config_export(path),
            Method::ConfigImport { path } => self.ipc_config_import(path),
            Method::BackupsList => {
                let list: Vec<BackupDto> = self
                    .state
                    .backup_files()
                    .iter()
                    .map(|p| BackupDto {
                        path: p.to_string_lossy().into_owned(),
                        name: p
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                    })
                    .collect();
                to_json(&list)
            }
            Method::BackupsRestore { path } => self.ipc_backup_restore(path),

            Method::PeekStart => {
                let idle = self.peek.as_ref().is_none_or(|p| p.is_closing());
                if idle {
                    self.toggle_peek();
                }
                // toggle_peek declines without fence windows.
                let changed = idle && self.peek.as_ref().is_some_and(|p| !p.is_closing());
                Ok(mutation(changed, None, Value::Null))
            }
            Method::PeekEnd => {
                let active = self.peek.as_ref().is_some_and(|p| !p.is_closing());
                if active {
                    self.end_peek();
                }
                Ok(mutation(active, None, Value::Null))
            }
            Method::SettingsOpenUi => {
                self.handle(Command::OpenSettings);
                Ok(json!({ "changed": true }))
            }
        }
    }

    // ---- selectors -----------------------------------------------------------------------

    /// Fence selector; the alias `inbox` (any case) names the inbox fence whatever its
    /// localized title.
    fn resolve_fence(&self, sel: &str) -> std::result::Result<FenceId, IpcError> {
        if is_inbox_alias(sel)
            && let Some(id) = self.state.inbox_id()
        {
            return Ok(id);
        }
        selector::resolve(
            self.state.fences().iter().map(|f| (f.id, f.title.as_str())),
            sel,
        )
        .map_err(|e| IpcError::fence(sel, e))
    }

    fn fence_or_err(&self, id: FenceId) -> std::result::Result<&Fence, IpcError> {
        self.state
            .fence(id)
            .ok_or_else(|| IpcError::internal(format!("fence {id} vanished")))
    }

    fn resolve_rule(&self, sel: &str) -> std::result::Result<usize, IpcError> {
        selector::resolve_indexed(
            self.state
                .config
                .rules
                .list
                .iter()
                .map(|r| (r.id, r.name.as_str())),
            sel,
        )
        .map_err(|e| match e {
            SelectorError::NotFound => {
                IpcError::new(ErrorCode::RuleNotFound, format!("no rule matches {sel:?}"))
                    .hint("Run `pecofence-cli rule list` and use the id, the index or the exact name")
            }
            SelectorError::Ambiguous(c) => IpcError::new(
                ErrorCode::RuleNotFound,
                format!("{} rules match {sel:?}", c.len()),
            )
            .hint("Use the rule id or its index")
            .details(json!({
                "candidates": c.iter().map(|(id, name)| json!({ "id": id, "name": name })).collect::<Vec<_>>()
            })),
        })
    }

    fn snapshot_not_found(&self, sel: &str) -> IpcError {
        IpcError::new(
            ErrorCode::SnapshotNotFound,
            format!("no snapshot matches {sel:?}"),
        )
        .hint("Run `pecofence-cli snapshot list` and use the id or the exact name")
    }

    /// `config.export`: the complete configuration as pretty JSON at an absolute path.
    fn ipc_config_export(&mut self, path: &str) -> IpcResult {
        let target = absolute_path(path)?;
        if target.is_dir() {
            return Err(IpcError::new(
                ErrorCode::InvalidValue,
                format!("{} is a directory; pass a file path", target.display()),
            ));
        }
        if !target.parent().is_some_and(|dir| dir.is_dir()) {
            return Err(IpcError::new(
                ErrorCode::InvalidValue,
                format!("the folder of {} does not exist", target.display()),
            ));
        }
        self.state.save_if_dirty();
        pecofence_core::ConfigStore::export_to(&self.state.config, &target).map_err(|e| {
            IpcError::internal(format!("could not write {}: {e}", target.display()))
        })?;
        let bytes = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        Ok(json!({ "path": target.to_string_lossy(), "bytes": bytes }))
    }

    /// `config.import`: replace the configuration with a file (same path as the GUI's import).
    fn ipc_config_import(&mut self, path: &str) -> IpcResult {
        let source = absolute_path(path)?;
        if !source.is_file() {
            return Err(IpcError::new(
                ErrorCode::InvalidValue,
                format!("{} is not a file", source.display()),
            ));
        }
        self.ipc_adopt_file(&source, "imported", "导入配置")
    }

    /// `backups.restore`: only files the app itself wrote (the same guard as the settings page).
    fn ipc_backup_restore(&mut self, path: &str) -> IpcResult {
        let wanted = path.trim().to_lowercase();
        let backups = self.state.backup_files();
        let Some(source) = backups
            .iter()
            .find(|p| p.to_string_lossy().to_lowercase() == wanted)
            .cloned()
        else {
            return Err(IpcError::new(
                ErrorCode::InvalidValue,
                format!("{path:?} is not one of PecoFence's backups"),
            )
            .hint("Run `pecofence-cli backup list` and pass one of its paths verbatim")
            .details(json!({
                "allowed": backups.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>()
            })));
        };
        self.ipc_adopt_file(&source, "restored", "恢复备份")
    }

    /// Shared tail of import and backup restore: validate the file, snapshot the current
    /// layout (never evicting a user snapshot), then adopt it exactly like the GUI does.
    fn ipc_adopt_file(
        &mut self,
        source: &std::path::Path,
        key: &str,
        what_key: &'static str,
    ) -> IpcResult {
        let cfg = pecofence_core::ConfigStore::parse_file(source).map_err(|e| {
            IpcError::new(
                ErrorCode::ValidationFailed,
                format!(
                    "{} is not a usable PecoFence configuration: {e}",
                    source.display()
                ),
            )
        })?;
        let snapshot = self.auto_snapshot();
        self.adopt_config(cfg, pecofence_core::i18n::text(what_key));
        Ok(mutation(
            true,
            snapshot,
            json!({ key: source.to_string_lossy() }),
        ))
    }

    fn resolve_snapshot(&self, sel: &str) -> std::result::Result<Uuid, IpcError> {
        selector::resolve(
            self.state
                .config
                .snapshots
                .iter()
                .map(|s| (s.id, s.name.as_str())),
            sel,
        )
        .map_err(|e| match e {
            SelectorError::NotFound => self.snapshot_not_found(sel),
            SelectorError::Ambiguous(c) => IpcError::new(
                ErrorCode::SnapshotNotFound,
                format!("{} snapshots match {sel:?}", c.len()),
            )
            .hint("Use the snapshot id")
            .details(json!({
                "candidates": c.iter().map(|(id, name)| json!({ "id": id, "name": name })).collect::<Vec<_>>()
            })),
        })
    }

    /// Item by id, by full path (case-insensitive), or by a unique display / file name.
    fn resolve_item(&self, sel: &str) -> std::result::Result<ItemId, IpcError> {
        let sel = sel.trim();
        let mut all: Vec<(&Item, &Fence)> = Vec::new();
        for f in self.state.fences() {
            all.extend(self.state.items_of(f).into_iter().map(|it| (it, f)));
        }
        let not_found = || {
            IpcError::new(ErrorCode::ItemNotFound, format!("no item matches {sel:?}"))
                .hint("Run `pecofence-cli item list` and use the id or the full path")
        };
        if sel.is_empty() {
            return Err(not_found());
        }
        if let Ok(id) = Uuid::parse_str(sel) {
            return all
                .iter()
                .find(|(it, _)| it.id == id)
                .map(|(it, _)| it.id)
                .ok_or_else(not_found);
        }
        let key = ItemKey::from_path(sel);
        if let Some((it, _)) = all.iter().find(|(it, _)| it.key == key) {
            return Ok(it.id);
        }
        let lower = sel.to_lowercase();
        let hits: Vec<&(&Item, &Fence)> = all
            .iter()
            .filter(|(it, _)| {
                it.display_name.to_lowercase() == lower
                    || it
                        .key
                        .as_path()
                        .and_then(|p| p.rsplit('\\').next())
                        .is_some_and(|name| name == lower)
            })
            .collect();
        match hits.len() {
            0 => Err(not_found()),
            1 => Ok(hits[0].0.id),
            n => {
                let candidates: Vec<Value> = hits
                    .iter()
                    .map(|(it, f)| {
                        json!({
                            "id": it.id,
                            "name": it.display_name,
                            "path": it.key.as_path(),
                            "fence": f.id,
                            "fenceTitle": f.title,
                        })
                    })
                    .collect();
                Err(
                    IpcError::new(ErrorCode::AmbiguousItem, format!("{n} items match {sel:?}"))
                        .hint("Use the item id or its full path")
                        .details(json!({ "candidates": candidates })),
                )
            }
        }
    }

    // ---- DTOs ----------------------------------------------------------------------------

    fn status_dto(&self) -> StatusDto {
        let memory_mb = pecofence_platform::memstats::MemoryStats::current()
            .map(|m| m.private_working_set as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);
        StatusDto {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol: PROTOCOL_VERSION,
            pid: std::process::id(),
            instance: self.instance.clone(),
            config_path: self.state.config_path().to_string_lossy().into_owned(),
            fence_count: self.state.fences().len(),
            item_count: self.state.workspace_item_count(),
            memory_mb,
            theme_mode: if self.theme_mode == ThemeMode::Dark {
                "dark"
            } else {
                "light"
            }
            .to_string(),
            desktop_icons_hidden: pecofence_platform::shell_icons::desktop_icons_hidden(),
        }
    }

    fn monitor_dtos(&self) -> Vec<MonitorDto> {
        let labels = self.monitor_labels();
        self.state
            .work_areas
            .iter()
            .map(|w| MonitorDto {
                id: w.device_path.clone(),
                label: labels
                    .iter()
                    .find(|(id, _)| *id == w.device_path)
                    .map(|(_, label)| label.clone())
                    .unwrap_or_else(|| w.device_path.clone()),
                rect: Rect {
                    x: w.mon_left,
                    y: w.mon_top,
                    w: w.mon_right - w.mon_left,
                    h: w.mon_bottom - w.mon_top,
                },
                work_area: Rect {
                    x: w.left,
                    y: w.top,
                    w: w.right - w.left,
                    h: w.bottom - w.top,
                },
                dpi: w.dpi,
                primary: w.mon_left == 0 && w.mon_top == 0,
            })
            .collect()
    }

    /// The CLI's view of a fence: the settings page's option strings plus geometry.
    pub(super) fn fence_dto(&self, f: &Fence) -> FenceDto {
        let host_id = self.state.host_of(f.id);
        let host = self.state.fence(host_id).unwrap_or(f);
        let connected = self
            .state
            .work_areas
            .iter()
            .any(|w| w.device_path == f.geometry.monitor);
        let opts = self.fence_options_json(f);
        let text = |key: &str| {
            opts.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let flag = |key: &str| opts.get(key).and_then(Value::as_bool).unwrap_or(false);
        let with_hash = |s: String| {
            if s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                format!("#{s}")
            } else {
                s
            }
        };
        let portal = (f.kind == FenceKind::FolderPortal).then(|| PortalDto {
            path: self
                .state
                .portal_root(f.id)
                .map(|p| p.to_string_lossy().into_owned())
                .or_else(|| match &f.source {
                    ItemSourceSpec::Folder { path, .. } => Some(path.clone()),
                    ItemSourceSpec::Desktop => None,
                })
                .unwrap_or_default(),
            navigate: f.portal_navigate,
            title_icon: !f.hide_title_icon,
        });
        FenceDto {
            id: f.id,
            title: f.title.clone(),
            kind: match f.kind {
                FenceKind::Inbox => "inbox",
                FenceKind::Virtual => "virtual",
                FenceKind::FolderPortal => "portal",
            }
            .to_string(),
            rect: connected.then(|| Rect::from(self.state.fence_px_rect(f))),
            window_rect: (connected && f.tab_host.is_none())
                .then(|| self.fences.get(&f.id).map(|w| rect_of(w.rect())))
                .flatten(),
            monitor: f.geometry.monitor.clone(),
            monitor_connected: connected,
            geometry: f.geometry.clone(),
            rolled_up: host.rolled_up,
            locked: host.locked,
            item_count: self.state.items_of(f).len(),
            tab_host: f.tab_host,
            tabs: if f.tab_host.is_none() {
                self.state
                    .tabs_of(f.id)
                    .into_iter()
                    .filter(|id| *id != f.id)
                    .collect()
            } else {
                Vec::new()
            },
            icon_size: f.view.icon_size,
            spacing: text("spacing"),
            auto_height: flag("autoHeight"),
            exclude_from_quick_hide: flag("excludeFromQuickHide"),
            opacity: text("opacity"),
            tint: opts
                .get("tint")
                .and_then(Value::as_str)
                .map(|t| with_hash(t.to_string())),
            title_color: with_hash(text("titleColor")),
            title_size: text("titleSize"),
            layout: layout_name(f.view.layout).to_string(),
            sort: sort_name(f.view.sort).to_string(),
            reverse: f.view.reverse,
            group_by_date: f.view.group_by_date,
            label_lines: f.view.label_lines,
            portal,
        }
    }

    /// `{"fence": FenceDto}` for a mutation result.
    fn fence_extra(&self, id: FenceId) -> IpcResult {
        Ok(json!({ "fence": to_json(&self.fence_dto(self.fence_or_err(id)?))? }))
    }

    fn item_dtos(&self, only: Option<FenceId>) -> Vec<ItemDto> {
        let mut out = Vec::new();
        for f in self.state.fences() {
            if only.is_some_and(|id| id != f.id) {
                continue;
            }
            for it in self.state.items_of(f) {
                out.push(self.item_dto(f, it));
            }
        }
        out
    }

    /// `assignedBy` and the filing rule's name for an item of `f`.
    fn assignment_of(&self, f: &Fence, item: ItemId) -> (&'static str, Option<String>) {
        if f.kind == FenceKind::FolderPortal {
            return ("portal", None);
        }
        match f
            .items
            .iter()
            .find(|r| r.item_id == item)
            .map(|r| &r.assigned_by)
        {
            Some(AssignedBy::User) => ("user", None),
            Some(AssignedBy::Rule(id)) => (
                "rule",
                Some(
                    self.state
                        .config
                        .rules
                        .list
                        .iter()
                        .find(|r| r.id == *id)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| id.to_string()),
                ),
            ),
            Some(AssignedBy::Migration) | None => ("migration", None),
        }
    }

    /// The CLI's view of one item: identity, category, file metadata and where it is filed.
    /// Reads the file's creation time and resolves shortcut targets, so it is built per call,
    /// not cached.
    pub(super) fn item_dto(&self, f: &Fence, it: &Item) -> ItemDto {
        let (assigned_by, rule) = self.assignment_of(f, it.id);
        let path = (!it.is_namespace())
            .then(|| it.key.as_path().map(str::to_string))
            .flatten();
        let key_file_name = path
            .as_deref()
            .and_then(|p| p.rsplit('\\').next())
            .unwrap_or_default();
        // The key is lower-cased; the display name has the real spelling minus the hidden
        // extension, which the suffix puts back.
        let file_name = if path.is_some() {
            format!(
                "{}{}",
                it.display_name,
                crate::rename::rename_hidden_suffix(&it.display_name, key_file_name, it.is_folder)
            )
        } else {
            it.display_name.clone()
        };
        let ext = if it.is_folder || it.is_namespace() {
            String::new()
        } else {
            rules::ext_of(&file_name)
        };
        let shortcut_target = match (path.as_deref(), ext.as_str()) {
            (Some(p), ".lnk") => shell::shortcut_target(Path::new(p)),
            (Some(p), ".url") => shell::url_shortcut_target(Path::new(p)),
            _ => None,
        };
        let kind = if it.is_namespace() {
            "namespace".to_string()
        } else {
            let facts = pecofence_core::ItemFacts {
                file_name: file_name.clone(),
                is_folder: it.is_folder,
                size_bytes: it.size,
                shortcut_target: shortcut_target.clone(),
                shortcut_target_ext: match ext.as_str() {
                    ".lnk" => shortcut_target.as_deref().map(rules::ext_of),
                    ".url" => Some(".url".into()),
                    _ => None,
                },
                ..Default::default()
            };
            rules::classify(&facts)
                .map(rules::category_name)
                .unwrap_or("other")
                .to_string()
        };
        let created = path
            .as_deref()
            .and_then(|p| std::fs::metadata(p).ok())
            .and_then(|m| m.created().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        ItemDto {
            id: it.id,
            name: it.display_name.clone(),
            file_name,
            path,
            is_folder: it.is_folder,
            kind,
            ext,
            size: it.size,
            modified: (it.mtime > 0).then_some(it.mtime),
            created,
            open_count: it.open_count,
            last_opened: it.last_opened,
            shortcut_target,
            fence: f.id,
            fence_title: f.title.clone(),
            assigned_by: assigned_by.to_string(),
            rule,
        }
    }

    /// `{"item": ItemDto}` for a mutation result.
    fn item_extra(&self, id: ItemId) -> IpcResult {
        let Some((f, it)) = self.locate_item(id) else {
            return Err(IpcError::internal(format!("item {id} vanished")));
        };
        Ok(json!({ "item": to_json(&self.item_dto(f, it))? }))
    }

    /// The fence an item is listed in, with the item.
    fn locate_item(&self, id: ItemId) -> Option<(&Fence, &Item)> {
        self.state.fences().iter().find_map(|f| {
            self.state
                .items_of(f)
                .into_iter()
                .find(|it| it.id == id)
                .map(|it| (f, it))
        })
    }

    fn planned_move_dto(&self, m: &crate::state::PlannedRuleMove) -> PlannedMoveDto {
        let title = |id: FenceId| {
            self.state
                .fence(id)
                .map(|f| f.title.clone())
                .unwrap_or_default()
        };
        let item = self.state.item(m.item);
        let (rule, rule_name) = match m.by {
            AssignedBy::Rule(id) => (
                Some(id),
                self.state
                    .config
                    .rules
                    .list
                    .iter()
                    .find(|r| r.id == id)
                    .map(|r| r.name.clone()),
            ),
            _ => (None, None),
        };
        let from = m.from.or_else(|| self.state.inbox_id()).unwrap_or_default();
        PlannedMoveDto {
            item: m.item,
            name: item.map(|it| it.display_name.clone()).unwrap_or_default(),
            path: item.and_then(|it| {
                (!it.is_namespace())
                    .then(|| it.key.as_path().map(str::to_string))
                    .flatten()
            }),
            from,
            from_title: title(from),
            to: m.to,
            to_title: title(m.to),
            rule,
            rule_name,
        }
    }

    fn rule_entry(&self, index: usize, rule: &Rule) -> RuleEntry {
        RuleEntry {
            index,
            rule: rule.clone(),
            target_title: match rule.target {
                Target::Fence(id) => self.state.fence(id).map(|f| f.title.clone()),
                Target::Inbox => None,
            },
        }
    }

    fn rule_list_dto(&self) -> RuleListDto {
        let rules = &self.state.config.rules;
        RuleListDto {
            default_target: rules.default_target,
            keep_updated: rules.keep_updated,
            list: rules
                .list
                .iter()
                .enumerate()
                .map(|(i, r)| self.rule_entry(i, r))
                .collect(),
        }
    }

    // ---- helpers -------------------------------------------------------------------------

    /// `auto-cli-<yyyy-mm-dd HH:MM:SS>` snapshot of the current layouts, keeping only the newest
    /// few. Older auto snapshots make room first; when the list is still full of the user's own
    /// snapshots none is taken (a warning is queued) rather than evicting one of theirs.
    fn auto_snapshot(&mut self) -> Option<Uuid> {
        if !auto_snapshot_slot(&mut self.state.config.snapshots) {
            self.ipc_warnings.push(SNAPSHOT_LIMIT_WARNING.to_string());
            return None;
        }
        // Room is guaranteed now, so `save_snapshot`'s eviction cannot trigger.
        Some(self.state.save_snapshot(&auto_snapshot_name()))
    }

    /// Quick-hide state; `None` before the desktop anchor exists.
    fn fences_hidden(&self) -> Option<bool> {
        self.anchor.borrow().as_ref().map(|a| a.fences_hidden())
    }

    fn rules_mutated(&mut self) {
        self.state.mark_dirty();
        self.schedule_save();
        self.push_settings_state();
    }

    fn tab_unsupported(&self, f: &Fence, host: FenceId) -> IpcError {
        let title = self
            .state
            .fence(host)
            .map(|h| h.title.clone())
            .unwrap_or_default();
        unsupported(format!(
            "{:?} is a tab inside {title:?} and has no window of its own",
            f.title
        ))
        .hint(format!("use the host fence {title:?}"))
    }

    // ---- fence mutations -----------------------------------------------------------------

    fn ipc_create_fence(
        &mut self,
        title: Option<&str>,
        rect: Option<Rect>,
        monitor: Option<&str>,
        portal: Option<&str>,
    ) -> IpcResult {
        let title = title.map(|t| checked_name("title", t)).transpose()?;
        let title = title.as_deref();
        // Centre to place the fence around when no rect was given.
        let (cx, cy) = match rect {
            Some(r) => {
                validate_rect(r, &self.state.work_areas)?;
                (r.x + r.w / 2, r.y + r.h / 2)
            }
            None => {
                let ids: Vec<&str> = self
                    .state
                    .work_areas
                    .iter()
                    .map(|w| w.device_path.as_str())
                    .collect();
                let area = match monitor {
                    Some(m) => self
                        .state
                        .work_areas
                        .iter()
                        .find(|w| w.device_path == m)
                        .ok_or_else(|| {
                            IpcError::invalid_value(format!("unknown monitor {m:?}"), &ids)
                                .hint("Run `pecofence-cli monitor list`")
                        })?,
                    None => self
                        .state
                        .work_areas
                        .first()
                        .ok_or_else(|| unsupported("no monitor is connected"))?,
                };
                ((area.left + area.right) / 2, (area.top + area.bottom) / 2)
            }
        };
        if let Some(folder) = portal {
            return self.ipc_create_portal(folder, title, rect, cx, cy);
        }
        if self.state.fences().len() >= pecofence_core::config_store::MAX_FENCES {
            return Err(IpcError::new(
                ErrorCode::LimitReached,
                format!(
                    "{} fences already exist (the maximum)",
                    pecofence_core::config_store::MAX_FENCES
                ),
            )
            .hint("Delete or merge a fence first"));
        }
        let rect = match rect {
            Some(r) => rect_to_win(r),
            None => self.place_new_fence(3, 200.0, cx, cy, None),
        };
        let id = self
            .create_fence_at(rect, title)
            .ok_or_else(|| unsupported("no monitor to place the fence on"))?;
        self.push_settings_state();
        Ok(mutation(true, None, self.fence_extra(id)?))
    }

    fn ipc_create_portal(
        &mut self,
        folder: &str,
        title: Option<&str>,
        rect: Option<Rect>,
        cx: i32,
        cy: i32,
    ) -> IpcResult {
        let folder = PathBuf::from(folder.trim());
        if !folder.is_dir() {
            return Err(IpcError::new(
                ErrorCode::InvalidValue,
                format!("{} is not an existing folder", folder.display()),
            )
            .hint("Pass the full path of a folder for --portal"));
        }
        let canonical = std::fs::canonicalize(&folder)
            .map(|c| {
                let s = c.to_string_lossy().to_string();
                PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s))
            })
            .unwrap_or(folder);
        let wanted = canonical.to_string_lossy().to_lowercase();
        let existing_portal = |app: &App| {
            app.state
                .fences()
                .iter()
                .find(|f| {
                    app.state
                        .portal_root(f.id)
                        .is_some_and(|p| p.to_string_lossy().to_lowercase() == wanted)
                })
                .map(|f| f.id)
        };
        if let Some(id) = existing_portal(self) {
            return Ok(mutation(false, None, self.fence_extra(id)?));
        }
        if self.state.fences().len() >= pecofence_core::config_store::MAX_FENCES {
            return Err(IpcError::new(
                ErrorCode::LimitReached,
                format!(
                    "{} fences already exist (the maximum)",
                    pecofence_core::config_store::MAX_FENCES
                ),
            )
            .hint("Delete or merge a fence first"));
        }
        self.create_portal(canonical, cx, cy, None);
        let id = existing_portal(self)
            .ok_or_else(|| IpcError::internal("the portal fence was not created; see the log"))?;
        if let Some(r) = rect {
            self.ipc_set_bounds(&id.to_string(), r)?;
        }
        if let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) {
            self.apply_fence_prop(id, fence_options::FenceProp::Title(title.to_string()));
        }
        self.push_settings_state();
        Ok(mutation(true, None, self.fence_extra(id)?))
    }

    fn ipc_delete_fence(&mut self, sel: &str) -> IpcResult {
        let id = self.resolve_fence(sel)?;
        let f = self.fence_or_err(id)?.clone();
        if f.kind == FenceKind::Inbox {
            return Err(unsupported(format!(
                "{:?} is the inbox fence (the desktop itself) and cannot be deleted",
                f.title
            )));
        }
        // Memberships are about to move back to the inbox: keep a way back.
        let snapshot = (f.kind != FenceKind::FolderPortal && !self.state.items_of(&f).is_empty())
            .then(|| self.auto_snapshot())
            .flatten();
        self.delete_fence(id);
        if self.state.fence(id).is_some() {
            return Err(IpcError::internal(
                "the fence could not be deleted; see the log",
            ));
        }
        self.push_settings_state();
        Ok(mutation(true, snapshot, json!({ "deleted": id })))
    }

    fn ipc_set_bounds(&mut self, sel: &str, rect: Rect) -> IpcResult {
        let id = self.resolve_fence(sel)?;
        let before = self.fence_or_err(id)?.clone();
        if let Some(host) = before.tab_host {
            return Err(self.tab_unsupported(&before, host));
        }
        validate_rect(rect, &self.state.work_areas)?;
        let Some(w) = self.fences.get(&id) else {
            return Err(IpcError::internal("the fence has no window"));
        };
        // Same steps as a user move/resize (`FenceBoundsChanged`) and the column snap: the
        // window first, then the state with the expanded height. Rolled up, only the title bar
        // shows; `h` becomes the height it unrolls to.
        let full = rect_to_win(rect);
        let rolled = w.is_rolled();
        if rolled {
            w.restore_geometry(full, true, rect.h);
        } else {
            w.set_bounds(full);
            w.apply_height(rect.h, false);
        }
        self.state.set_fence_bounds(id, full, rolled, rect.h);
        self.apply_auto_height(id);
        self.schedule_save();
        let changed = self.state.fence(id) != Some(&before);
        self.push_settings_state();
        Ok(mutation(changed, None, self.fence_extra(id)?))
    }

    fn ipc_move_to_monitor(&mut self, sel: &str, monitor: &str) -> IpcResult {
        let id = self.resolve_fence(sel)?;
        let f = self.fence_or_err(id)?.clone();
        if let Some(host) = f.tab_host {
            return Err(self.tab_unsupported(&f, host));
        }
        let ids: Vec<&str> = self
            .state
            .work_areas
            .iter()
            .map(|w| w.device_path.as_str())
            .collect();
        let Some(work) = self
            .state
            .work_areas
            .iter()
            .find(|w| w.device_path == monitor)
            .cloned()
        else {
            return Err(
                IpcError::invalid_value(format!("unknown monitor {monitor:?}"), &ids)
                    .hint("Run `pecofence-cli monitor list`"),
            );
        };
        if f.geometry.monitor == monitor {
            return Ok(mutation(false, None, self.fence_extra(id)?));
        }
        // Like `swap_monitors`: map onto the other work area keeping the anchored gaps, then
        // store the geometry re-normalised there. The tabs this window hosts share its place
        // on screen, so their saved geometry travels too (they have no window of their own to
        // report a move).
        for member in self.state.tabs_of(id) {
            let Some(geo) = self
                .state
                .fence(member)
                .map(|m| geometry::normalize(geometry::denormalize(&m.geometry, &work), &work))
            else {
                continue;
            };
            if let Some(fm) = self.state.fence_mut(member) {
                fm.geometry = geo;
            }
        }
        self.state.mark_dirty();
        self.end_peek_now();
        self.relayout_from_state();
        self.schedule_save();
        Ok(mutation(true, None, self.fence_extra(id)?))
    }

    fn ipc_merge(&mut self, fence: &str, into: &str) -> IpcResult {
        let a = self.resolve_fence(fence)?;
        let host = self.state.host_of(self.resolve_fence(into)?);
        let extra = |app: &App| -> IpcResult {
            Ok(json!({ "host": to_json(&app.fence_dto(app.fence_or_err(host)?))? }))
        };
        if a == host || self.state.host_of(a) == host {
            return Ok(mutation(false, None, extra(self)?));
        }
        let Some(w) = self.fences.get(&host) else {
            return Err(IpcError::internal("the target fence has no window"));
        };
        let hwnd = w.hwnd();
        // i32::MIN = no pointer position: append to the strip (see `merge_slot_at`).
        self.handle(Command::MergeFence {
            fence: a,
            into: hwnd,
            x: i32::MIN,
        });
        let changed = self.state.host_of(a) == host;
        self.push_settings_state();
        Ok(mutation(changed, None, extra(self)?))
    }

    fn ipc_detach(&mut self, tab: &str) -> IpcResult {
        let id = self.resolve_fence(tab)?;
        let f = self.fence_or_err(id)?.clone();
        let Some(host) = f.tab_host else {
            return Ok(mutation(false, None, self.fence_extra(id)?));
        };
        let (x, y) = self
            .fences
            .get(&host)
            .map(|w| {
                let r = w.rect();
                ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
            })
            .or_else(|| {
                self.state
                    .work_areas
                    .first()
                    .map(|w| ((w.left + w.right) / 2, (w.top + w.bottom) / 2))
            })
            .unwrap_or((400, 300));
        self.detach_tab(id, x, y, false);
        let changed = self.state.fence(id).is_some_and(|f| f.tab_host.is_none());
        self.push_settings_state();
        Ok(mutation(changed, None, self.fence_extra(id)?))
    }

    fn ipc_move_items(&mut self, items: &[String], to: &str) -> IpcResult {
        let to = self.resolve_fence(to)?;
        let mut ids: Vec<ItemId> = Vec::new();
        for sel in items {
            let id = self.resolve_item(sel)?;
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        // Where each item lives now; items already in `to` do not count as moved.
        let mut located: HashMap<ItemId, FenceId> = HashMap::new();
        for f in self.state.fences() {
            for it in self.state.items_of(f) {
                located.insert(it.id, f.id);
            }
        }
        let moving: Vec<ItemId> = ids
            .into_iter()
            .filter(|id| located.get(id) != Some(&to))
            .collect();
        // What `move_items` will actually act on: into a portal only real files go (the
        // Recycle Bin and friends are not files); into a virtual fence, desktop items change
        // membership and portal items are moved out as files.
        let into_portal = self.state.portal_path(to).is_some();
        let (accepted, skipped): (Vec<ItemId>, Vec<ItemId>) = moving.iter().partition(|id| {
            let Some(it) = self.state.item(**id) else {
                return false;
            };
            if into_portal {
                !it.is_namespace() && it.key.as_path().is_some()
            } else {
                self.state.is_portal_item(**id) || self.state.config.items.contains_key(id)
            }
        });
        if !skipped.is_empty() {
            let names: Vec<String> = skipped
                .iter()
                .filter_map(|id| self.state.item(*id))
                .map(|it| it.display_name.clone())
                .collect();
            self.ipc_warnings.push(format!(
                "{} item(s) cannot go into a folder and stayed where they are: {}",
                skipped.len(),
                names.join(", ")
            ));
        }
        if accepted.is_empty() {
            return Ok(mutation(false, None, json!({ "moved": 0, "to": to })));
        }
        // A large re-filing of desktop items is a layout change worth a way back. Real file
        // moves (a portal on either side) are not: a snapshot could not undo them.
        let membership_only =
            !into_portal && accepted.iter().all(|id| !self.state.is_portal_item(*id));
        let snapshot = (membership_only && accepted.len() >= BIG_MOVE_SNAPSHOT_ITEMS)
            .then(|| self.auto_snapshot())
            .flatten();
        // Portal sources / targets move real files on a worker thread; the fences update when
        // the shell reports back.
        self.move_items(&accepted, to);
        self.push_settings_state();
        Ok(mutation(
            true,
            snapshot,
            json!({ "moved": accepted.len(), "to": to }),
        ))
    }

    /// `items.rename`: a real rename inside the item's folder (see [`App::rename_item_to`]).
    fn ipc_rename_item(&mut self, sel: &str, name: &str, keep_ext: bool) -> IpcResult {
        let id = self.resolve_item(sel)?;
        let Some((_, it)) = self.locate_item(id) else {
            return Err(IpcError::internal(format!("item {id} vanished")));
        };
        if it.is_namespace() {
            return Err(unsupported(format!(
                "{:?} is a shell namespace item (This PC, Recycle Bin, …) and has no file to rename",
                it.display_name
            )));
        }
        let Some(old_path) = it.key.as_path() else {
            return Err(unsupported(format!("{:?} has no path", it.display_name)));
        };
        let old_file_name = old_path.rsplit('\\').next().unwrap_or(old_path).to_string();
        let is_folder = it.is_folder;
        let name = name.trim();
        let target = if is_folder || !keep_ext {
            name.to_string()
        } else {
            let old_ext = rules::ext_of(&old_file_name);
            if old_ext.is_empty() || rules::ext_of(name) == old_ext {
                name.to_string()
            } else {
                format!("{name}{old_ext}")
            }
        };
        if target.eq_ignore_ascii_case(&old_file_name) && target == old_file_name {
            return Ok(mutation(false, None, self.item_extra(id)?));
        }
        let new_path = self.rename_item_to(id, &target).map_err(|e| match e {
            RenameError::Unsupported => unsupported(format!("{sel:?} cannot be renamed")),
            RenameError::InvalidName => IpcError::invalid_value(
                format!("{target:?} is not a valid file name"),
                &["a non-empty name without \\ / : * ? \" < > |"],
            ),
            RenameError::Exists => IpcError::new(
                ErrorCode::InvalidValue,
                format!("{target:?} already exists in that folder"),
            )
            .hint("Pick another name, or move the existing item first"),
            RenameError::Io(msg) => IpcError::internal(format!("rename failed: {msg}")),
        })?;
        // Portal ids are path hashes and change with the name; desktop ids are stable.
        let new_id = if self.state.item(id).is_some() {
            id
        } else {
            crate::state::portal_item_id(&new_path.to_string_lossy())
        };
        self.push_settings_state();
        Ok(mutation(
            true,
            None,
            json!({
                "item": self.item_extra(new_id).ok().and_then(|v| v.get("item").cloned()).unwrap_or(Value::Null),
                "path": new_path.to_string_lossy(),
            }),
        ))
    }

    // ---- event stream --------------------------------------------------------------------

    /// `events.subscribe`: registers the connection as a stream. The first tick's baseline is
    /// taken now, so the client sees only what changes after it subscribed.
    fn ipc_subscribe(
        &mut self,
        fence: Option<&str>,
        events: Option<&[String]>,
        reply: mpsc::Sender<Response>,
    ) -> IpcResult {
        if self.ipc_subscribers.len() >= MAX_SUBSCRIBERS {
            return Err(IpcError::new(
                ErrorCode::LimitReached,
                format!("{MAX_SUBSCRIBERS} event streams are already open"),
            )
            .hint("Stop one of the running `pecofence-cli watch` processes"));
        }
        let fence_id = fence.map(|sel| self.resolve_fence(sel)).transpose()?;
        let events: Option<Vec<String>> = match events {
            None => None,
            Some(list) => {
                let mut names = Vec::new();
                for name in list {
                    let name = name.trim().to_ascii_lowercase();
                    if !EVENT_NAMES.contains(&name.as_str()) {
                        return Err(IpcError::invalid_value(
                            format!("unknown event {name:?}"),
                            EVENT_NAMES,
                        ));
                    }
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                (!names.is_empty()).then_some(names)
            }
        };
        if self.ipc_subscribers.is_empty() {
            self.ipc_event_state = Some(self.event_state());
            self.ipc_last_event = Instant::now();
            window::set_timer(self.control.hwnd(), TIMER_IPC_EVENTS, EVENT_TICK_MS);
        }
        self.ipc_subscribers.push(IpcSubscriber {
            reply,
            fence: fence_id,
            events: events.clone(),
            seq: 0,
        });
        tracing::info!(target: "pecofence::ipc", subscribers = self.ipc_subscribers.len(), "event stream opened");
        Ok(json!({
            "subscribed": true,
            "fence": fence_id,
            "events": events.unwrap_or_else(|| EVENT_NAMES.iter().map(|s| s.to_string()).collect()),
            "heartbeatSecs": EVENT_HEARTBEAT_SECS,
        }))
    }

    /// The present fences and items in the shape the diff needs.
    fn event_state(&self) -> EventState {
        let mut state = EventState::default();
        for f in self.state.fences() {
            state.fences.insert(f.id, self.fence_dto(f));
            for it in self.state.items_of(f) {
                state.items.insert(
                    it.id,
                    ItemSig {
                        fence: f.id,
                        fence_title: f.title.clone(),
                        name: it.display_name.clone(),
                        path: (!it.is_namespace())
                            .then(|| it.key.as_path().map(str::to_string))
                            .flatten(),
                        is_folder: it.is_folder,
                        is_namespace: it.is_namespace(),
                    },
                );
            }
        }
        state
    }

    /// An `ItemDto` for an item that is gone: what the last tick knew, no file metadata.
    fn removed_item_dto(id: ItemId, sig: &ItemSig) -> ItemDto {
        let file_name = sig
            .path
            .as_deref()
            .and_then(|p| p.rsplit('\\').next())
            .map(str::to_string)
            .unwrap_or_else(|| sig.name.clone());
        let ext = if sig.is_folder || sig.is_namespace {
            String::new()
        } else {
            rules::ext_of(&file_name)
        };
        ItemDto {
            id,
            name: sig.name.clone(),
            file_name,
            path: sig.path.clone(),
            is_folder: sig.is_folder,
            kind: if sig.is_namespace {
                "namespace".into()
            } else if sig.is_folder {
                "folders".into()
            } else {
                "other".into()
            },
            ext,
            size: 0,
            modified: None,
            created: None,
            open_count: 0,
            last_opened: None,
            shortcut_target: None,
            fence: sig.fence,
            fence_title: sig.fence_title.clone(),
            assigned_by: "removed".into(),
            rule: None,
        }
    }

    /// `FenceDto` fields (camelCase) whose value differs, ignoring the animated ones.
    fn fence_changed_fields(old: &FenceDto, new: &FenceDto) -> Vec<String> {
        let (Ok(Value::Object(a)), Ok(Value::Object(b))) =
            (serde_json::to_value(old), serde_json::to_value(new))
        else {
            return Vec::new();
        };
        a.iter()
            .filter(|(k, v)| {
                !EVENT_IGNORED_FENCE_FIELDS.contains(&k.as_str()) && b.get(*k) != Some(v)
            })
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Diffs the present against the last tick, sends the resulting events to every subscriber
    /// whose filter admits them, and drops subscribers whose connection has gone. A quiet
    /// subscription gets a `heartbeat` every [`EVENT_HEARTBEAT_SECS`] so a vanished client is
    /// noticed within that time. Runs from the tick timer and right after CLI mutations.
    pub(super) fn ipc_events_tick(&mut self) {
        if self.ipc_subscribers.is_empty() {
            window::kill_timer(self.control.hwnd(), TIMER_IPC_EVENTS);
            self.ipc_event_state = None;
            return;
        }
        let now_state = self.event_state();
        let Some(prev) = self.ipc_event_state.take() else {
            self.ipc_event_state = Some(now_state);
            return;
        };
        let ts = pecofence_core::now_unix();
        let blank = |event: &str| EventDto {
            seq: 0,
            ts,
            event: event.to_string(),
            item: None,
            from: None,
            from_title: None,
            fence: None,
            changed: Vec::new(),
        };
        let mut events: Vec<EventDto> = Vec::new();
        for (id, cur) in &now_state.fences {
            match prev.fences.get(id) {
                None => events.push(EventDto {
                    fence: Some(cur.clone()),
                    ..blank("fence.created")
                }),
                Some(old) => {
                    let changed = Self::fence_changed_fields(old, cur);
                    if !changed.is_empty() {
                        events.push(EventDto {
                            fence: Some(cur.clone()),
                            changed,
                            ..blank("fence.changed")
                        });
                    }
                }
            }
        }
        for (id, old) in &prev.fences {
            if !now_state.fences.contains_key(id) {
                events.push(EventDto {
                    fence: Some(old.clone()),
                    ..blank("fence.deleted")
                });
            }
        }
        for (id, sig) in &now_state.items {
            let dto = || {
                self.locate_item(*id)
                    .map(|(f, it)| self.item_dto(f, it))
                    .unwrap_or_else(|| Self::removed_item_dto(*id, sig))
            };
            match prev.items.get(id) {
                None => events.push(EventDto {
                    item: Some(dto()),
                    ..blank("item.added")
                }),
                Some(old) if old.fence != sig.fence => events.push(EventDto {
                    item: Some(dto()),
                    from: Some(old.fence),
                    from_title: Some(old.fence_title.clone()),
                    ..blank("item.moved")
                }),
                Some(_) => {}
            }
        }
        for (id, old) in &prev.items {
            if !now_state.items.contains_key(id) {
                events.push(EventDto {
                    item: Some(Self::removed_item_dto(*id, old)),
                    ..blank("item.removed")
                });
            }
        }
        self.ipc_event_state = Some(now_state);
        if events.is_empty() {
            if self.ipc_last_event.elapsed() < Duration::from_secs(EVENT_HEARTBEAT_SECS) {
                return;
            }
            events.push(blank("heartbeat"));
        }
        self.ipc_last_event = Instant::now();
        let mut gone = Vec::new();
        for (i, sub) in self.ipc_subscribers.iter_mut().enumerate() {
            for ev in &events {
                if !Self::event_admitted(sub, ev) {
                    continue;
                }
                sub.seq += 1;
                let mut ev = ev.clone();
                ev.seq = sub.seq;
                let line = match serde_json::to_value(&ev) {
                    Ok(v) => Response::ok(v),
                    Err(e) => Response::err(IpcError::internal(format!("event: {e}"))),
                };
                if sub.reply.send(line).is_err() {
                    gone.push(i);
                    break;
                }
            }
        }
        for i in gone.into_iter().rev() {
            self.ipc_subscribers.remove(i);
            tracing::info!(target: "pecofence::ipc", subscribers = self.ipc_subscribers.len(), "event stream closed");
        }
        if self.ipc_subscribers.is_empty() {
            window::kill_timer(self.control.hwnd(), TIMER_IPC_EVENTS);
            self.ipc_event_state = None;
        }
    }

    /// Whether `ev` passes a subscriber's `fence` / `events` filter (heartbeats always do).
    fn event_admitted(sub: &IpcSubscriber, ev: &EventDto) -> bool {
        if ev.event == "heartbeat" {
            return true;
        }
        if let Some(list) = &sub.events
            && !list.contains(&ev.event)
        {
            return false;
        }
        match sub.fence {
            None => true,
            Some(f) => {
                ev.fence.as_ref().is_some_and(|d| d.id == f)
                    || ev.item.as_ref().is_some_and(|i| i.fence == f)
                    || ev.from == Some(f)
            }
        }
    }

    fn ipc_add_rule(
        &mut self,
        name: &str,
        target: &str,
        all_of: &[pecofence_core::Cond],
        index: Option<usize>,
    ) -> IpcResult {
        let name = checked_name("rule name", name)?;
        check_rule_conditions(all_of)?;
        let target = if is_inbox_alias(target) {
            Target::Inbox
        } else {
            let id = self.resolve_fence(target)?;
            if self.fence_or_err(id)?.kind == FenceKind::FolderPortal {
                return Err(unsupported(
                    "a folder portal cannot be a rule target (it shows a folder, not desktop items)",
                )
                .hint("Target a virtual fence or `inbox`"));
            }
            Target::Fence(id)
        };
        let rule = Rule::new(&name, target, all_of.to_vec());
        let list = &mut self.state.config.rules.list;
        let at = index.map_or(list.len(), |i| i.min(list.len()));
        list.insert(at, rule.clone());
        self.rules_mutated();
        let entry = self.rule_entry(at, &rule);
        Ok(mutation(true, None, json!({ "rule": to_json(&entry)? })))
    }
}
