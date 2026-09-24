//! UI-thread half of the CLI IPC (`pecofence-cli`): drains the requests the pipe threads queued
//! (`ipc_server.rs`), runs each `Method` against the `App` exactly like a queued `Command`,
//! saves the configuration, and hands the reply back to the waiting connection thread.
//!
//! Every message this module emits is English (the CLI's contract); the localized `i18n`
//! strings are for the GUI only.

use super::*;
use crate::ipc_server::Pending;
use fence_options::{layout_name, parse_fence_prop, sort_name};
use pecofence_core::geometry;
use pecofence_core::rules::Rule;
use pecofence_core::{AssignedBy, Fence, Item, ItemSourceSpec, Settings, Snapshot};
use pecofence_ipc::selector::{self, SelectorError};
use pecofence_ipc::{
    ErrorCode, FenceDto, IpcError, ItemDto, Method, MonitorDto, PROTOCOL_VERSION, PortalDto, Rect,
    Response, RuleEntry, RuleListDto, SnapshotDto, StatusDto, dotted_to_pointer, mutation,
};
use serde_json::{Value, json};
use uuid::Uuid;

/// Newest `auto-cli-*` snapshots kept; older ones go when a new one is taken.
const AUTO_SNAPSHOTS_KEPT: usize = 3;
pub(super) const AUTO_SNAPSHOT_PREFIX: &str = "auto-cli-";

type IpcResult = std::result::Result<Value, IpcError>;

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
            if arrived.elapsed() > Duration::from_millis(u64::from(request.timeout_ms)) {
                tracing::debug!(
                    target: "pecofence::ipc",
                    method,
                    waited_ms = arrived.elapsed().as_millis() as u64,
                    "request expired before it could run; dropped"
                );
                continue;
            }
            let started = Instant::now();
            let mut response = self.handle_ipc(&request.method);
            if request.method.is_mutation() && response.ok {
                // Persist before replying so the caller can rely on the file; the deferred
                // save the mutation scheduled is redundant now.
                window::kill_timer(self.control.hwnd(), TIMER_SAVE);
                if self.state.is_dirty() && !self.state.save_if_dirty() {
                    response = response.with_warning("applied but not saved: see the log");
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
        }
    }

    pub(super) fn handle_ipc(&mut self, m: &Method) -> Response {
        self.ipc_dispatch(m).into()
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
                let hidden = self
                    .anchor
                    .borrow()
                    .as_ref()
                    .map(|a| a.fences_hidden())
                    .unwrap_or(false);
                let changed = hidden == *visible;
                if changed {
                    self.toggle_all_fences();
                }
                Ok(mutation(changed, None, json!({ "visible": visible })))
            }
            Method::FencesOpenOptions { fence } => {
                let id = self.resolve_fence(fence)?;
                self.handle(Command::OpenOptionsForFence(id));
                Ok(json!({ "changed": true }))
            }

            Method::ItemsMove { items, to } => self.ipc_move_items(items, to),

            Method::SettingsPatch { path, value } => {
                let before = self.state.config.settings.clone();
                let new = patch_settings_path(&before, path, value.clone())?;
                let whole = dotted_to_pointer(path).is_empty();
                let snapshot = (whole && new != before).then(|| self.auto_snapshot());
                self.apply_settings(new);
                let changed = self.state.config.settings != before;
                Ok(mutation(
                    changed,
                    snapshot,
                    json!({ "settings": to_json(&self.state.config.settings)? }),
                ))
            }
            Method::RulesSet { rules } => {
                let before = self.state.config.rules.clone();
                let snapshot = (*rules != before).then(|| self.auto_snapshot());
                self.set_rules(rules.clone());
                let changed = self.state.config.rules != before;
                self.push_settings_state();
                Ok(mutation(
                    changed,
                    snapshot,
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
            Method::RulesApply => {
                // Re-filing every desktop item rewrites memberships: keep a way back, but not
                // when nothing moved.
                let snapshot = self.auto_snapshot();
                let entries = shell::enumerate_desktop();
                let moved = self.state.apply_rules_all(&entries);
                self.refresh_all();
                self.schedule_save();
                self.push_settings_state();
                let snapshot = if moved > 0 {
                    Some(snapshot)
                } else {
                    self.state.delete_snapshot(snapshot);
                    None
                };
                Ok(mutation(moved > 0, snapshot, json!({ "moved": moved })))
            }

            Method::SnapshotsSave { name } => {
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
                let id = self.state.save_snapshot(name);
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
                // The backup the existing mechanism takes is this call's auto snapshot.
                let backup = self
                    .state
                    .restore_snapshot_with_backup(target, &auto_snapshot_name())
                    .ok_or_else(|| self.snapshot_not_found(id))?;
                prune_auto_snapshots(&mut self.state.config.snapshots, AUTO_SNAPSHOTS_KEPT);
                self.end_peek_now();
                self.relayout_from_state();
                // Portals restored with the snapshot need enumerating; stale runtime state of
                // the replaced ones is pruned on the way.
                self.refresh_portals();
                self.schedule_save();
                self.push_settings_state();
                Ok(mutation(true, Some(backup), json!({ "restored": target })))
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

    fn resolve_fence(&self, sel: &str) -> std::result::Result<FenceId, IpcError> {
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
            let is_portal = f.kind == FenceKind::FolderPortal;
            for it in self.state.items_of(f) {
                let assigned_by = if is_portal {
                    "portal"
                } else {
                    match f
                        .items
                        .iter()
                        .find(|r| r.item_id == it.id)
                        .map(|r| &r.assigned_by)
                    {
                        Some(AssignedBy::User) => "user",
                        Some(AssignedBy::Rule(_)) => "rule",
                        Some(AssignedBy::Migration) | None => "migration",
                    }
                };
                out.push(ItemDto {
                    id: it.id,
                    name: it.display_name.clone(),
                    path: (!it.is_namespace())
                        .then(|| it.key.as_path().map(str::to_string))
                        .flatten(),
                    is_folder: it.is_folder,
                    fence: f.id,
                    fence_title: f.title.clone(),
                    assigned_by: assigned_by.to_string(),
                });
            }
        }
        out
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

    /// `auto-cli-<yyyy-mm-dd HH:MM:SS>` snapshot of the current layouts, keeping only the newest few.
    fn auto_snapshot(&mut self) -> Uuid {
        let id = self.state.save_snapshot(&auto_snapshot_name());
        prune_auto_snapshots(&mut self.state.config.snapshots, AUTO_SNAPSHOTS_KEPT);
        id
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
        if let Some(r) = rect
            && (r.w <= 0 || r.h <= 0)
        {
            return Err(IpcError::invalid_value(
                "rect width and height must be positive",
                &["w > 0", "h > 0"],
            ));
        }
        // Centre to place the fence around when no rect was given.
        let (cx, cy) = match rect {
            Some(r) => (r.x + r.w / 2, r.y + r.h / 2),
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
            .then(|| self.auto_snapshot());
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
        if rect.w <= 0 || rect.h <= 0 {
            return Err(IpcError::invalid_value(
                "rect width and height must be positive",
                &["w > 0", "h > 0"],
            ));
        }
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
        // store the geometry re-normalised there.
        let px = geometry::denormalize(&f.geometry, &work);
        let geo = geometry::normalize(px, &work);
        if let Some(fm) = self.state.fence_mut(id) {
            fm.geometry = geo;
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
        if moving.is_empty() {
            return Ok(mutation(false, None, json!({ "moved": 0, "to": to })));
        }
        // Portal sources / targets move real files on a worker thread; the fences update when
        // the shell reports back.
        self.move_items(&moving, to);
        self.push_settings_state();
        Ok(mutation(
            true,
            None,
            json!({ "moved": moving.len(), "to": to }),
        ))
    }

    fn ipc_add_rule(
        &mut self,
        name: &str,
        target: &str,
        all_of: &[pecofence_core::Cond],
        index: Option<usize>,
    ) -> IpcResult {
        let name = name.trim();
        if name.is_empty() {
            return Err(IpcError::invalid_value(
                "rule name must not be empty",
                &["<non-empty string>"],
            ));
        }
        let target = if target.trim().eq_ignore_ascii_case("inbox") {
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
        let rule = Rule::new(name, target, all_of.to_vec());
        let list = &mut self.state.config.rules.list;
        let at = index.map_or(list.len(), |i| i.min(list.len()));
        list.insert(at, rule.clone());
        self.rules_mutated();
        let entry = self.rule_entry(at, &rule);
        Ok(mutation(true, None, json!({ "rule": to_json(&entry)? })))
    }
}
