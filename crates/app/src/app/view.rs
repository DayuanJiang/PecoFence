//! 插件的轻量层: what a plugin fence shows and what the user does in it. A plugin's view
//! (`pecofence_plugin_host::view`) becomes script fence rows (`ScriptRowView`), which the fence
//! draws as a script fence's. Clicks, double-clicks, check boxes, to-dos, sliders and typing go
//! back to the plugin as events (`App::plugin_event`): `click` and `dblclick` with `{ id }`,
//! `change` with `{ id, value }`, `text` with `{ id, text }`.
//!
//! A view with a note (`input: { type: "note" }`) makes the fence a note: its text is kept in
//! `Config::notes` and edited and ticked as a note fence's. A one-line box opens the name box
//! (`rename`) over it. While either is open the plugin is neither restarted nor updated.

use super::*;
use crate::fence_window::{ScriptCellView, ScriptRowView, ScriptView};
use pecofence_plugin_host as host;

/// The restarts and stops held while a plugin fence's input is open: for each fence, the
/// plugin to start (its id and main.js) once it closes, or None to stop it. And the id of each
/// plugin fence's note, for its `text` events (also while a restarted plugin has no view yet).
#[derive(Default)]
pub(super) struct Held {
    starts: HashMap<FenceId, Option<(String, PathBuf)>>,
    notes: HashMap<FenceId, Option<String>>,
}

impl App {
    /// The rows fence `fence`'s plugin `view` turns into. A picture is a file in the plugin's
    /// data folder; a note makes the fence a note.
    pub(super) fn plugin_rows(&mut self, fence: FenceId, view: host::View) -> Vec<ScriptRowView> {
        let plugin = self.state.config.scripts.get(&fence);
        let data = plugin
            .and_then(|s| super::plugins::plugin_id(&s.path))
            .map(|id| self.plugins_data().join(id));
        let file =
            |name: Option<String>| Some(data.as_ref()?.join(name?).to_string_lossy().into_owned());
        let note = view
            .rows
            .iter()
            .filter_map(|r| r.input.as_ref())
            .find(|i| i.kind == "note")
            .map(|i| i.id.clone());
        match &note {
            Some(id) => self.held.notes.insert(fence, id.clone()),
            None => self.held.notes.remove(&fence),
        };
        let note = note.is_some();
        let was = if note {
            !self.state.note_inputs.insert(fence)
        } else {
            self.state.note_inputs.remove(&fence)
        };
        if note != was {
            tracing::info!(%fence, note, "plugin fence: its view holds a note now, or no more");
        }
        view.rows
            .into_iter()
            .map(|r| {
                let line = r.input.as_ref().is_some_and(|i| i.kind == "line");
                let canvas = r.canvas.map(|mut c| {
                    for step in &mut c.draw {
                        step.image = file(step.image.take());
                    }
                    Rc::new(c)
                });
                let cells = r.cells.into_iter().map(|c| ScriptCellView {
                    text: c.icon.unwrap_or(c.text),
                    color: c.color,
                    tip: c.tip,
                    clickable: c.id.is_some(),
                    slider: c.slider,
                    width: c.width,
                    align: match c.align.as_deref() {
                        Some("center") => 0.5,
                        Some("right") => 1.0,
                        _ => 0.0,
                    },
                    checked: c.checked,
                    bold: c.bold,
                    gap: None,
                    // A button's second click is a click too (rapid clicks on Next count) and
                    // also sends `dblclick`; a check box and a slider take it as a press.
                    dbl: c.id.is_some() && c.check.is_none() && c.slider.is_none(),
                    checkbox: c.check,
                    image: file(c.image),
                    image_height: c.height.unwrap_or(96.0),
                    fill: c.fill,
                    under: c.under,
                    id: c.id,
                    blur: c.blur,
                });
                ScriptRowView {
                    clickable: r.id.is_some() || line,
                    dbl: r.id.is_some(),
                    text: r.text,
                    color: r.color,
                    size: r.size,
                    short: r.short,
                    bold: r.bold,
                    icon: r.icon,
                    tip: r.tip,
                    todo: r.todo,
                    separator: r.separator,
                    image: file(r.image),
                    image_height: r.height.unwrap_or(96.0),
                    pane: r.pane,
                    follow: r.follow,
                    cells: cells.collect(),
                    plugin: true,
                    id: r.id,
                    spans: r.spans,
                    blur: r.blur,
                    progress: r.progress,
                    chart: r.chart,
                    canvas,
                    input: r.input,
                    ..ScriptRowView::default()
                }
            })
            .collect()
    }

    /// A light plugin fence's view: its rows (正在运行… until the first), or why it stopped and
    /// a 重新启动 button.
    pub(super) fn light_view(&self, fence: FenceId) -> Option<ScriptView> {
        let stopped = self.light_stopped_rows(fence);
        let rows = stopped.or_else(|| self.light.running.get(&fence)?.0.clone());
        let quiet = rows.is_none() && self.light.running.contains_key(&fence);
        // A plugin that shows nothing shows nothing (no script's "printed no text" hint).
        let rows = rows.map(|mut rows| {
            if rows.is_empty() {
                rows.push(ScriptRowView {
                    plugin: true,
                    ..ScriptRowView::default()
                });
            }
            rows
        });
        (quiet || rows.is_some()).then(|| ScriptView {
            rows: rows.unwrap_or_default(),
            quiet,
            ..ScriptView::default()
        })
    }

    /// A click on row `index` (cell `cell`) of plugin fence `fence`: what it means for the
    /// plugin, which then shows its new view. A check box, a to-do and a slider show their
    /// new state at once.
    pub(super) fn plugin_row_clicked(
        &mut self,
        fence: FenceId,
        index: usize,
        cell: usize,
        fraction: Option<f32>,
        dragging: bool,
        double: bool,
    ) {
        use serde_json::json;
        if self.light.stopped.contains_key(&fence) {
            // The 重新启动 button under why it stopped.
            tracing::info!(%fence, "plugin fence: 重新启动 clicked");
            return self.refresh_script(fence);
        }
        let Some(row) = self
            .light
            .running
            .get_mut(&fence)
            .and_then(|(rows, ..)| rows.as_mut()?.get_mut(index))
        else {
            return;
        };
        let target = row.cells.get_mut(cell);
        let (name, id, value) = match target {
            Some(c) => {
                let Some(id) = c.id.clone() else { return };
                if let Some(at) = fraction.filter(|_| c.slider.is_some()) {
                    c.slider = Some(at);
                    ("change", id, Some(json!(at)))
                } else if let Some(on) = c.checkbox {
                    c.checkbox = Some(!on);
                    ("change", id, Some(json!(!on)))
                } else {
                    ("click", id, None)
                }
            }
            None if row.input.as_ref().is_some_and(|i| i.kind == "line") && !double => {
                return self.open_plugin_line(fence, index);
            }
            None => {
                let Some(id) = row.id.clone() else { return };
                match row.todo {
                    Some(done) if !double => {
                        row.todo = Some(!done);
                        ("change", id, Some(json!(!done)))
                    }
                    _ if double => ("dblclick", id, None),
                    _ => ("click", id, None),
                }
            }
        };
        if !dragging {
            tracing::info!(%fence, index, cell, name, double, "plugin fence: row clicked");
        }
        if name == "change" {
            self.push_tool_view(self.state.host_of(fence));
        }
        match value {
            Some(value) => self.plugin_event(fence, name, json!({ "id": id, "value": value })),
            None => {
                self.plugin_event(fence, name, json!({ "id": id }));
                // The second click of a double-click on a button: a click, then `dblclick`.
                if double && name == "click" {
                    self.plugin_event(fence, "dblclick", json!({ "id": id }));
                }
            }
        }
    }

    /// Opens the one-line box of row `index` of plugin fence `fence` over the row (the name
    /// box, which takes the input method as a title's does).
    fn open_plugin_line(&mut self, fence: FenceId, index: usize) {
        let Some(line) = self
            .light
            .running
            .get(&fence)
            .and_then(|(rows, ..)| rows.as_ref()?.get(index)?.input.clone())
        else {
            return;
        };
        let Some(w) = self.window_for(fence) else {
            return;
        };
        let Some(rect) = w.script_row_screen_rect(index, 0) else {
            return;
        };
        tracing::info!(%fence, index, chars = line.value.chars().count(), "plugin fence: its one-line box opens");
        crate::rename::begin_line_input(
            w.hwnd(),
            rect,
            (fence, index),
            line.value,
            line.placeholder,
            self.queue.clone(),
            w.label_ink(self.theme_mode),
        );
    }

    /// The one-line box of row `index` closed: with Enter or a click elsewhere its text goes
    /// to the plugin (`text`) and shows in the row; Esc keeps the old one.
    pub(super) fn plugin_line_closed(
        &mut self,
        fence: FenceId,
        index: usize,
        text: Option<String>,
    ) {
        if let Some(text) = text {
            let row = self
                .light
                .running
                .get_mut(&fence)
                .and_then(|(rows, ..)| rows.as_mut()?.get_mut(index));
            if let Some(row) = row
                && let Some(line) = &mut row.input
            {
                line.value = text.clone();
                let id = line.id.clone();
                tracing::info!(%fence, index, chars = text.chars().count(), "plugin fence: one-line box sends its text");
                self.push_tool_view(self.state.host_of(fence));
                // A restart held while the box was open first: the new runtime gets the text.
                self.release_held(fence);
                self.plugin_event(fence, "text", serde_json::json!({ "id": id, "text": text }));
                return;
            }
        }
        self.release_held(fence);
    }

    /// A note of a plugin fence changed (edited, ticked, a to-do moved): the plugin gets its
    /// new text (`text`, with the note's id). The editor that closed lets go of what it held.
    pub(super) fn plugin_note_changed(&mut self, fence: FenceId, before: Option<String>) {
        if !self.state.note_inputs.contains(&fence) {
            return;
        }
        // A restart held while the note was edited first: the new runtime gets the text.
        if crate::note_edit::open_for() != Some(fence) {
            self.release_held(fence);
        }
        let text = self.state.note_text(fence).map(str::to_string);
        if text != before
            && let Some(text) = text
        {
            let id = self.held.notes.get(&fence).cloned().flatten();
            tracing::info!(%fence, chars = text.chars().count(), "plugin fence: its note changed, the plugin is told");
            self.plugin_event(fence, "text", serde_json::json!({ "id": id, "text": text }));
        }
    }

    /// Whether plugin fence `fence` has an input open: its note in the editor, or its one-line
    /// box.
    fn plugin_input_open(&self, fence: FenceId) -> bool {
        crate::note_edit::open_for() == Some(fence)
            || matches!(crate::rename::active_target(), Some(crate::rename::RenameTarget::PluginLine(f, _)) if f == fence)
    }

    /// `start_light` and `stop_light` ask first: while the fence's input is open, the plugin
    /// is neither restarted (an update, 立即运行) nor stopped; what was asked last is done when
    /// it closes. `start` is the plugin to start, None a stop.
    pub(super) fn hold_light(&mut self, fence: FenceId, start: Option<(&str, &Path)>) -> bool {
        if !self.plugin_input_open(fence) {
            return false;
        }
        let start = start.map(|(id, main)| (id.to_string(), main.to_path_buf()));
        tracing::info!(%fence, start = start.is_some(), "plugin fence: restart or stop held while its input is open");
        self.held.starts.insert(fence, start);
        true
    }

    fn release_held(&mut self, fence: FenceId) {
        match self.held.starts.remove(&fence) {
            Some(Some((id, main))) => {
                tracing::info!(%fence, id, "plugin fence: the held restart runs now");
                self.start_light(fence, &id, &main);
                self.push_tool_view(self.state.host_of(fence));
            }
            Some(None) => self.stop_light(fence),
            None => {}
        }
    }
}
