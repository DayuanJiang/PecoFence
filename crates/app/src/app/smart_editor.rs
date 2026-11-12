//! 智能栅栏编辑器: the smart fence editor, a small window of its own like the Notes app's smart
//! folder dialog. It is the settings page in an editor-only mode (`HostKind::SmartEditor`):
//! the name, the templates for a new fence, 全部 / 任一, one row per condition, the live count
//! and 取消 / 创建 (好 for a fence that exists). Nothing changes until 创建 / 好; 取消, Esc and
//! the close box change nothing. One editor at a time.
//!
//! Page messages (`Command::EditorMessage`): `ready`, `editorDirty {dirty}`,
//! `editorPickFolder {row}`, `editorSave {title, spec}`, `editorCancel`, `editorCount` (see
//! `smart.rs`) and `action` `pageError`. The host sends `state` (language, theme), `editor`
//! (the fence and the templates), `editorFolder {row, path}` and `editorError {text}`.

use super::*;
use crate::settings_host::HostKind;
use pecofence_core::smart::{SmartSpec, Template};

pub(super) struct SmartEditor {
    host: SettingsHost,
    /// The fence edited; None while a new one is made.
    pub fence: Option<FenceId>,
    /// Where a new fence goes: the point its menu opened at.
    at: (i32, i32),
    /// The page has changes it has not saved (its `editorDirty`).
    dirty: bool,
}

impl SmartEditor {
    pub fn hwnd(&self) -> HWND {
        self.host.hwnd()
    }

    pub fn post_json(&self, json: &str) {
        self.host.post_json(json);
    }
}

/// The window's caption: 新建智能栅栏 or 编辑智能栅栏.
fn editor_title(fence: Option<FenceId>) -> &'static str {
    match fence {
        None => pecofence_core::i18n::text("新建智能栅栏"),
        Some(_) => pecofence_core::i18n::text("编辑智能栅栏"),
    }
}

impl App {
    /// Opens the editor for smart fence `fence`, or for a new one placed near `at`. An editor
    /// already open comes to the front: it takes the other fence only when nothing in it is
    /// unsaved, and asks nothing.
    pub(super) fn open_smart_editor(&mut self, fence: Option<FenceId>, at: (i32, i32), by: &str) {
        if let Some(id) = fence
            && self.state.smart_spec(id).is_none()
        {
            tracing::warn!(%id, by, "smart fence editor: not a smart fence");
            return;
        }
        if let Some(ed) = self.smart_editor.as_mut() {
            if ed.fence == fence || ed.dirty {
                tracing::info!(by, fence = ?fence, editing = ?ed.fence, unsaved = ed.dirty, "smart fence editor: already open, brought to the front");
            } else {
                tracing::info!(by, fence = ?fence, was = ?ed.fence, "smart fence editor: takes the other fence (nothing unsaved)");
                ed.fence = fence;
                ed.at = at;
                ed.host.set_title(editor_title(fence));
                if ed.host.ready {
                    let json = self.smart_editor_json();
                    if let Some(ed) = &self.smart_editor {
                        ed.post_json(&json);
                    }
                }
            }
            if let Some(ed) = &self.smart_editor {
                window::show_normal(ed.hwnd());
                window::bring_to_front(ed.hwnd());
            }
            return;
        }
        if !self.ensure_web_env() {
            return;
        }
        match SettingsHost::open(
            &self.editor_class,
            self.web_env.as_ref().unwrap(),
            self.theme_mode,
            self.ctx.theme.borrow().liquid_glass,
            self.queue.clone(),
            HostKind::SmartEditor,
            editor_title(fence),
            Some(at),
            None,
        ) {
            Ok(mut host) => {
                self.set_host_icons(&mut host);
                tracing::info!(by, fence = ?fence, "smart fence editor opened");
                self.smart_editor = Some(SmartEditor {
                    host,
                    fence,
                    at,
                    dirty: false,
                });
            }
            Err(_) if window::quit_posted() => window::post_quit(0),
            Err(e) => tracing::error!(error = %e, "smart fence editor window failed"),
        }
    }

    /// What the editor shows: the fence's name and search (a new fence: no name, every drive),
    /// whether it is new, the templates with this computer's folders (null when the template's
    /// folder is not here) and Everything's state.
    fn smart_editor_json(&self) -> String {
        let fence = self.smart_editor.as_ref().and_then(|e| e.fence);
        let edited = fence.and_then(|id| self.state.fence(id));
        let (title, spec) = match edited {
            Some(f) => (f.title.clone(), f.smart.clone().unwrap_or_default()),
            None => (String::new(), SmartSpec::default()),
        };
        let known = super::smart::known_folders();
        let templates: Vec<serde_json::Value> = Template::ALL
            .iter()
            .map(|t| serde_json::json!({ "key": t.key(), "title": t.title(), "spec": t.spec(&known) }))
            .collect();
        serde_json::json!({
            "type": "editor",
            "fence": fence,
            "isNew": edited.is_none(),
            "title": title,
            "spec": spec,
            "templates": templates,
            "everything": self.everything_json(),
        })
        .to_string()
    }

    /// A message from the editor's page.
    pub(super) fn on_editor_message(&mut self, json: &str) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else {
            return;
        };
        let Some(ed) = self.smart_editor.as_mut() else {
            return;
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("ready") => {
                let first = !ed.host.ready;
                ed.host.ready = true;
                let ms = first.then(|| ed.host.opened.elapsed().as_millis() as u64);
                tracing::info!(ms, fence = ?ed.fence, "smart fence editor: page ready");
                // The page's language and theme, then the fence.
                let state = self.settings_state_json();
                let editor = self.smart_editor_json();
                if let Some(ed) = &self.smart_editor {
                    ed.post_json(&state);
                    ed.post_json(&editor);
                    // The window shows once the page has drawn the two (`drawn`).
                    if first {
                        ed.host.show_soon();
                    }
                }
            }
            Some("drawn") => {
                if ed.host.show() {
                    let ms = ed.host.opened.elapsed().as_millis() as u64;
                    tracing::info!(ms, "smart fence editor: page drawn, window shown");
                }
            }
            Some("editorDirty") => {
                ed.dirty = v.get("dirty").and_then(|d| d.as_bool()).unwrap_or(false);
                tracing::debug!(dirty = ed.dirty, "smart fence editor: unsaved changes");
            }
            Some("editorPickFolder") => {
                let row = v.get("row").cloned().unwrap_or_default();
                let owner = Some(ed.hwnd());
                tracing::info!("smart fence editor: picking a folder");
                let path = match pecofence_platform::filedialog::pick_folder(
                    owner,
                    pecofence_core::i18n::text("选择智能栅栏要搜索的文件夹"),
                ) {
                    Ok(Some(dir)) => {
                        tracing::info!("smart fence editor: folder picked");
                        Some(
                            crate::state::canonical_path(dir)
                                .to_string_lossy()
                                .into_owned(),
                        )
                    }
                    Ok(None) => {
                        tracing::info!("smart fence editor: folder picker cancelled");
                        None
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "smart fence editor: folder picker failed");
                        None
                    }
                };
                if let Some(ed) = &self.smart_editor {
                    ed.post_json(
                        &serde_json::json!({ "type": "editorFolder", "row": row, "path": path })
                            .to_string(),
                    );
                }
            }
            Some("editorSave") => self.save_smart_editor(&v),
            Some("editorCancel") => self.close_smart_editor("cancelled"),
            Some("editorCount") => self.smart_editor_count(&v),
            Some("action") => match v.get("name").and_then(|n| n.as_str()) {
                Some("pageError") => super::settings::log_page_error(&v),
                Some("startEverything") => self.start_everything(),
                Some("installEverything") => self.install_everything(),
                _ => {}
            },
            _ => {}
        }
    }

    /// 创建 / 好: checks the search, then saves it (a new fence near the menu's point, or the
    /// edited fence's search and name) and closes the editor. A search the host cannot take
    /// leaves the editor open with the reason. An edited fence deleted meanwhile is made anew,
    /// so the work in the editor is kept.
    fn save_smart_editor(&mut self, v: &serde_json::Value) {
        let Some(ed) = &self.smart_editor else {
            return;
        };
        let (fence, at) = (ed.fence, ed.at);
        let refuse = |app: &Self, why: &str, text: &str| {
            tracing::info!(fence = ?fence, why, "smart fence editor: not saved");
            if let Some(ed) = &app.smart_editor {
                ed.post_json(
                    &serde_json::json!({ "type": "editorError", "text": text }).to_string(),
                );
            }
        };
        let mut spec: SmartSpec = match v.get("spec").cloned().map(serde_json::from_value) {
            Some(Ok(spec)) => spec,
            Some(Err(e)) => {
                tracing::warn!(error = %e, "smart fence editor: the page sent a search the app cannot read");
                return refuse(
                    self,
                    "unreadable",
                    pecofence_core::i18n::text("没能保存，请检查条件"),
                );
            }
            None => return,
        };
        if let Some(problem) = spec.problem() {
            return refuse(
                self,
                &problem,
                pecofence_core::i18n::text("没能保存，请检查条件"),
            );
        }
        if let Some(missing) = spec.folders.iter().find(|f| !Path::new(f).is_dir()) {
            let text =
                pecofence_core::i18n::format("找不到文件夹 {0}", std::slice::from_ref(missing));
            return refuse(self, "a folder is missing", &text);
        }
        let title = v
            .get("title")
            .and_then(|t| t.as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| super::ipc::checked_name("title", t));
        let title = match title.transpose() {
            Ok(t) => t,
            Err(e) => {
                return refuse(self, &e.message, pecofence_core::i18n::text("名称太长了"));
            }
        };
        match fence.filter(|id| self.state.smart_spec(*id).is_some()) {
            Some(id) => {
                // Closed first: its own changes are not changes made elsewhere.
                self.close_smart_editor("saved");
                let changed = self.replace_smart_spec(id, spec);
                let renamed = title.is_some_and(|t| {
                    self.apply_fence_prop(id, super::fence_options::FenceProp::Title(t))
                });
                tracing::info!(%id, changed, renamed, "smart fence editor: saved");
            }
            None => {
                // A template stays only while its folders are this computer's.
                let known = super::smart::known_folders();
                if let Some(key) = spec.template.clone()
                    && Template::parse(&key)
                        .and_then(|t| t.spec(&known))
                        .is_none_or(|t| t.folders != spec.folders)
                {
                    spec.template = None;
                }
                let template = spec.template.clone();
                if fence.is_some() {
                    tracing::info!(fence = ?fence, "smart fence editor: the fence was deleted meanwhile; a new one is made");
                }
                let Some(id) = self.create_smart_fence(spec, title.as_deref(), None, at.0, at.1)
                else {
                    // No room for another fence: the editor stays open with its rows.
                    let text = super::fences::fence_limit_text();
                    return refuse(self, "no room for another fence", &text);
                };
                tracing::info!(%id, template = ?template, "smart fence editor: new smart fence created");
            }
        }
        self.close_smart_editor("saved");
        self.push_settings_state();
    }

    /// Closes the editor at once (saved or cancelled; the close box goes through
    /// `smart_editor_gone`).
    pub(super) fn close_smart_editor(&mut self, why: &str) {
        if let Some(ed) = self.smart_editor.take() {
            tracing::info!(why, fence = ?ed.fence, unsaved = ed.dirty, "smart fence editor closed");
            self.smart_count_cancel();
            ed.host.close();
        }
    }

    /// `WindowGone`: the editor's window was closed by its close box (or Alt+F4): nothing is
    /// saved.
    pub(super) fn smart_editor_gone(&mut self, hwnd: HWND) {
        if self.smart_editor.as_ref().is_some_and(|e| e.hwnd() == hwnd)
            && let Some(ed) = self.smart_editor.take()
        {
            tracing::info!(fence = ?ed.fence, unsaved = ed.dirty, "smart fence editor closed by its window");
            self.smart_count_cancel();
        }
    }

    /// Fence `fence`'s search or name changed elsewhere (`fence set`, `fence rename`, the
    /// settings page, a template's folder found again) while the editor shows it: with nothing unsaved the editor shows the new search, else it
    /// says that 好 would put its own search in place of that change.
    pub(super) fn smart_editor_fence_changed(&mut self, fence: FenceId) {
        let Some(ed) = self
            .smart_editor
            .as_ref()
            .filter(|e| e.fence == Some(fence) && e.host.ready)
        else {
            return;
        };
        if ed.dirty {
            tracing::info!(%fence, "smart fence editor: the search changed elsewhere; OK would replace it");
            ed.post_json(
                &serde_json::json!({
                    "type": "editorNotice",
                    "text": pecofence_core::i18n::text("这个栅栏的条件刚在别处改过，按“好”会换成这里的条件"),
                })
                .to_string(),
            );
        } else {
            tracing::info!(%fence, "smart fence editor: the search changed elsewhere; shown again");
            let json = self.smart_editor_json();
            self.post_to_smart_editor(&json);
        }
    }

    /// The interface language changed: the caption and the page follow.
    pub(super) fn smart_editor_language_changed(&mut self) {
        if let Some(ed) = &self.smart_editor {
            ed.host.set_title(editor_title(ed.fence));
            if ed.host.ready {
                ed.post_json(&self.settings_state_json());
            }
        }
    }

    /// The theme changed (`Some` when light / dark or the material changed): the caption and
    /// the page follow.
    pub(super) fn smart_editor_theme_changed(&self, mode: Option<(ThemeMode, bool)>) {
        if let Some(ed) = &self.smart_editor {
            if let Some((mode, liquid_glass)) = mode {
                ed.host.set_theme(mode, liquid_glass);
            }
            if ed.host.ready {
                ed.post_json(&self.settings_state_json());
            }
        }
    }
}

impl App {
    /// `editorCount {seq, spec}`: the live count while editing (`smart.rs` counts it). A search
    /// the app cannot read is not counted.
    fn smart_editor_count(&mut self, v: &serde_json::Value) {
        let seq = v.get("seq").and_then(|s| s.as_u64()).unwrap_or(0);
        match v
            .get("spec")
            .cloned()
            .map(serde_json::from_value::<SmartSpec>)
        {
            Some(Ok(spec)) if spec.problem().is_none() => self.count_editor_search(spec, seq),
            Some(Ok(spec)) => {
                tracing::info!(seq, problem = ?spec.problem(), "smart fence editor: not counted")
            }
            Some(Err(e)) => {
                tracing::warn!(seq, error = %e, "smart fence editor: a search to count the app cannot read")
            }
            None => {}
        }
    }

    /// The editor closed: a count it waits for is no longer wanted.
    fn smart_count_cancel(&mut self) {
        self.stop_editor_count();
    }

    /// A message for the editor's page, when it is open and ready.
    pub(super) fn post_to_smart_editor(&self, json: &str) {
        if let Some(ed) = self.smart_editor.as_ref().filter(|e| e.host.ready) {
            ed.post_json(json);
        }
    }
}
