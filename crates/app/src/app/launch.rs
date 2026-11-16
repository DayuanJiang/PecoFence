//! Plugin interface 2 in the app: the entries a plugin puts in its fence menu (`view.menu`),
//! the shortcuts the user gives its commands (`__keys` in the fence's settings: a press shows
//! the fence the way the Peek does, for it alone, then the plugin gets `command`), files dropped
//! on its fence (`drop`), and the launcher's items (`__launch`): the plugin knows them by id and
//! name only, PecoFence keeps the paths and opens them as a fence icon opens.

use super::*;
use pecofence_core::i18n;
use serde_json::{Value, json};

/// The fence menu's plugin entries: this id and the `MAX_MENU - 1` after it.
pub(super) const CMD_PLUGIN_MENU: u32 = 1940;
pub(super) const CMD_PLUGIN_MENU_END: u32 =
    CMD_PLUGIN_MENU + pecofence_plugin_host::view::MAX_MENU as u32;
/// `WM_HOTKEY` ids of plugin commands: from here up.
pub(super) const HOTKEY_PLUGIN_FIRST: i32 = 0x100;
const KEYS: &str = "__keys";
const LAUNCH: &str = "__launch";
/// The most items of one launcher fence.
const MAX_ITEMS: usize = 200;
/// The drops kept for `launch.add` (a call of one may come after the next drop).
const DROPS_KEPT: usize = 8;

/// A launcher item's picture in its plugin's data folder (named after its fence: it goes with it).
fn icon_name(fence: FenceId, item: &str) -> String {
    format!("{fence}-launch-{item}.png")
}

impl App {
    /// The plugin's own entries at the end of its fence's menu (from its last `pf.show`), in a
    /// submenu named after it when there are more than three.
    pub(super) fn add_plugin_menu(&self, menu: &PopupMenu, fence: FenceId, id: &str) {
        let Some(items) = self.light.menus.get(&fence).filter(|m| !m.is_empty()) else {
            return;
        };
        // The ids as shown: a choice means these even when the plugin shows another menu meanwhile.
        *self.light.menu_shown.borrow_mut() = items.iter().map(|m| m.id.clone()).collect();
        let sub = PopupMenu::new();
        let many = items.iter().filter(|m| !m.separator).count() > 3;
        let to = if many { &sub } else { menu.separator() };
        for (i, m) in items.iter().enumerate() {
            if m.separator {
                to.separator();
            } else {
                to.item(CMD_PLUGIN_MENU + i as u32, &m.title, m.checked, m.disabled);
            }
        }
        if many {
            menu.separator().submenu(&self.plugin_name(id), sub);
        }
    }

    pub(super) fn plugin_menu_chosen(&mut self, fence: FenceId, i: usize) {
        let id = self.light.menu_shown.borrow().get(i).cloned();
        let Some(id) = id.filter(|id| !id.is_empty()) else {
            return;
        };
        tracing::info!(%fence, id, "plugin fence: its menu entry chosen");
        self.plugin_event(fence, "command", json!({ "id": id, "via": "menu" }));
    }

    /// The shortcuts the fences give their plugins' commands: of fences whose plugin runs (or
    /// starts again by itself) and still declares the command.
    fn plugin_keys_wanted(&self) -> Vec<(FenceId, String, Hotkey)> {
        let mut want = Vec::new();
        for (fence, spec) in &self.state.config.scripts {
            let m = super::plugins::plugin_id(&spec.path).and_then(|id| self.plugin(id));
            let keys = spec.settings.get(KEYS).and_then(Value::as_object);
            let (true, Some(m), Some(keys)) = (self.light_keeps_going(*fence), m, keys) else {
                continue;
            };
            let commands = &m.manifest.permissions.commands;
            for (command, text) in keys {
                let h = text.as_str().and_then(|t| Hotkey::parse(t).ok());
                if let Some(h) = h.filter(|_| commands.iter().any(|c| c.id == *command)) {
                    want.push((*fence, command.clone(), h));
                }
            }
        }
        want
    }

    /// Registers the shortcuts wanted and drops the others (a fence deleted, its plugin
    /// uninstalled, taken down, stopped for good, or a command an update took away). One another
    /// program held is tried again at each sync.
    pub(super) fn sync_plugin_keys(&mut self) {
        if self.hotkeys_paused {
            return;
        }
        let want = self.plugin_keys_wanted();
        let hwnd = self.control.hwnd();
        self.light.keys.retain(|id, k| {
            let keep = want.contains(k);
            if !keep {
                hotkey::unregister(hwnd, *id);
                tracing::info!(fence = %k.0, command = k.1, hotkey = %k.2, "plugin shortcut unregistered");
            }
            keep
        });
        self.light.keys_taken.clear();
        for k in want {
            if self.light.keys.values().any(|r| *r == k) {
                continue;
            }
            let id = (HOTKEY_PLUGIN_FIRST..).find(|i| !self.light.keys.contains_key(i));
            let id = id.unwrap_or(HOTKEY_PLUGIN_FIRST);
            let (fence, command, h) = (k.0, k.1.clone(), k.2);
            if hotkey::register(hwnd, id, h.mods | hotkey::mods::NOREPEAT, h.vk) {
                tracing::info!(%fence, command, hotkey = %h, id, "plugin shortcut registered");
                self.light.keys.insert(id, k);
            } else {
                tracing::warn!(%fence, command, hotkey = %h, "plugin shortcut taken by Windows or another program");
                self.light.keys_taken.push(k);
            }
        }
    }

    /// The settings page records a shortcut: the plugins' go until it ends.
    pub(super) fn drop_plugin_keys(&mut self) {
        let hwnd = self.control.hwnd();
        for id in self.light.keys.keys() {
            hotkey::unregister(hwnd, *id);
        }
        self.light.keys.clear();
        self.light.keys_taken.clear();
    }

    /// `WM_HOTKEY` of a plugin command: its fence shows like a Peek of it alone and its plugin
    /// wakes, then gets `command`; pressed again while that Peek is open, the Peek ends. While
    /// another fence is up alone, this one takes its place in the same Peek.
    pub(super) fn plugin_key_pressed(&mut self, id: i32) {
        let Some((fence, command, h)) = self.light.keys.get(&id).cloned() else {
            return;
        };
        let host = self.state.host_of(fence);
        if self
            .window_for(fence)
            .is_some_and(|w| w.active_fence() != fence)
        {
            self.switch_tab(host, fence);
        }
        let Some(hwnd) = self.window_for(fence).map(|w| w.hwnd()) else {
            return;
        };
        let ours = self.peek.is_some() && self.peek_only == Some(hwnd);
        tracing::info!(%fence, command, hotkey = %h, peek = self.peek.is_some(), ours, "plugin shortcut pressed");
        if ours {
            return self.end_peek("hotkey or tray menu");
        }
        if self.peek.is_some() && self.peek_only.is_some() {
            // In the same Peek: one that ends and opens again loses the keyboard to the window
            // under it and ends at once.
            tracing::info!(%fence, "plugin shortcut: the fence up alone gives its place to this one");
            let over = self.peek.as_ref().map(|p| p.hwnds()).unwrap_or_default();
            if let Some(a) = self.anchor.borrow_mut().as_mut() {
                a.set_peek(false, &[]);
                a.peek_only = Some(hwnd);
                a.set_peek(true, &over);
            }
            self.peek_only = Some(hwnd);
        } else if self.peek.is_none() {
            self.peek_only = Some(hwnd);
            self.toggle_peek();
            if self.peek.is_none() {
                tracing::info!(%fence, "plugin shortcut: no Peek opened, the fence shows as it is");
                self.peek_only = None;
            }
        }
        self.sync_light();
        self.plugin_event(fence, "command", json!({ "id": command, "via": "key" }));
    }

    /// What `fence`'s plugin takes dropped (`drop`; folders only for a launcher), if anything.
    pub(super) fn plugin_takes_drops(
        &self,
        fence: FenceId,
    ) -> Option<crate::fence_window::PluginDrops> {
        let spec = self.state.config.scripts.get(&fence)?;
        let p = &self.plugin(super::plugins::plugin_id(&spec.path)?)?;
        let p = &p.manifest.permissions;
        let has = |kind: &str| p.drop.iter().any(|d| d == kind);
        let files = has("files");
        let takes = crate::fence_window::PluginDrops {
            files,
            folders: files && p.launch,
            text: has("text"),
            links: has("links"),
        };
        (files || takes.text || takes.links).then_some(takes)
    }

    /// Files, a text or a link dropped on plugin fence `fence`: its plugin gets `drop` with the
    /// files' names and sizes (folders only by name), the text, the link and the drop's number,
    /// and may add the files as launcher items while it handles it (the plugin host adds the
    /// number to that `launch.add`) or read them (`pf.drop.*`, the last drop's).
    pub(super) fn plugin_dropped(
        &mut self,
        fence: FenceId,
        paths: Vec<PathBuf>,
        text: Option<String>,
        url: Option<String>,
    ) {
        let takes = self.plugin_takes_drops(fence).unwrap_or_default();
        let (text, url) = (text.filter(|_| takes.text), url.filter(|_| takes.links));
        let paths = if takes.files { paths } else { Vec::new() };
        let nothing = paths.is_empty() && text.is_none() && url.is_none();
        if nothing || !self.light.running.contains_key(&fence) {
            return tracing::info!(%fence, files = paths.len(), "drop on a plugin fence refused: its plugin takes none of it or does not run");
        }
        let files: Vec<Value> = (paths.iter())
            .map(|p| {
                let meta = std::fs::metadata(p).ok();
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                let folder = meta.as_ref().is_some_and(|m| m.is_dir());
                let size = meta.filter(|m| m.is_file()).map_or(0, |m| m.len());
                json!({ "name": name, "size": size, "folder": folder })
            })
            .collect();
        self.light.next_drop += 1;
        let drop = self.light.next_drop;
        let chars = text.as_ref().map_or(0, |t| t.chars().count());
        tracing::info!(%fence, files = files.len(), chars, link = url.is_some(), drop, "dropped on a plugin fence: its plugin is told");
        self.light.drops.push((drop, fence, paths));
        if self.light.drops.len() > DROPS_KEPT {
            self.light.drops.remove(0);
        }
        let data = json!({ "files": files, "text": text, "url": url, "drop": drop });
        self.plugin_event(fence, "drop", data);
    }

    /// File `i` of drop `drop` on `fence` for `pf.drop.*` (a folder is for a launcher only).
    pub(super) fn dropped_file(
        &self,
        fence: FenceId,
        drop: Option<u64>,
        i: usize,
    ) -> std::result::Result<PathBuf, String> {
        let found = (self.light.drops.iter()).find(|d| Some(d.0) == drop && d.1 == fence);
        let files = &found.ok_or("pf.drop: that drop is gone")?.2;
        let path = files
            .get(i)
            .ok_or_else(|| format!("pf.drop: the drop has {} files", files.len()))?;
        if !path.is_file() {
            return Err("pf.drop: that is a folder or no longer there".into());
        }
        Ok(path.clone())
    }

    /// The launcher items of `fence`: `{id, path, name}` each.
    fn launch_items(&self, fence: FenceId) -> Vec<Value> {
        let spec = self.state.config.scripts.get(&fence);
        let items = spec.and_then(|s| s.settings.get(LAUNCH)?.as_array().cloned());
        items.unwrap_or_default()
    }

    /// What the plugin and the page see of an item: its id, name and picture (no path).
    fn shown_item(&self, fence: FenceId, item: &Value) -> Value {
        let id = item["id"].as_str().unwrap_or_default();
        let data = self.plugin_data_dir(fence);
        let icon = data
            .map(|d| d.join(icon_name(fence, id)))
            .filter(|f| f.is_file());
        json!({ "id": id, "name": item["name"], "icon": icon.map(|_| icon_name(fence, id)) })
    }

    fn plugin_data_dir(&self, fence: FenceId) -> Option<PathBuf> {
        let spec = self.state.config.scripts.get(&fence)?;
        Some(
            self.plugins_data()
                .join(super::plugins::plugin_id(&spec.path)?),
        )
    }

    /// Adds `path` as an item of launcher fence `fence` and returns its id (its picture comes
    /// later, [`Self::make_launch_pictures`]). Nothing of PecoFence's own folder (its
    /// configuration, plugins and their data), nor a shortcut to it; a path once; at most
    /// [`MAX_ITEMS`]. The errors name no path (the plugin gets them).
    fn add_launch_item(
        &mut self,
        fence: FenceId,
        path: &Path,
    ) -> std::result::Result<String, String> {
        let config = self.state.config_path_on_disk();
        let own = config.parent().and_then(|d| std::fs::canonicalize(d).ok());
        let real = std::fs::canonicalize(path).map_err(|e| format!("launch: {e}"))?;
        let lnk = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"));
        let target = lnk
            .then(|| pecofence_platform::shell::shortcut_target_expanded(path))
            .flatten()
            .and_then(|t| std::fs::canonicalize(t).ok());
        if own.is_some_and(|own| {
            real.starts_with(&own) || target.is_some_and(|t| t.starts_with(&own))
        }) {
            tracing::info!(%fence, lnk, "launcher item refused: a file of PecoFence's own folder or a shortcut to one");
            return Err("a file of PecoFence's own folder cannot be an item".into());
        }
        let items = self.launch_items(fence);
        let text = path.to_string_lossy();
        if items.iter().any(|i| {
            i["path"]
                .as_str()
                .is_some_and(|p| p.eq_ignore_ascii_case(&text))
        }) {
            tracing::info!(%fence, "launcher item refused: the fence has it already");
            return Err("the fence has this item already".into());
        }
        if items.len() >= MAX_ITEMS {
            tracing::info!(%fence, items = items.len(), "launcher item refused: the fence is full");
            return Err(format!("a launcher fence holds at most {MAX_ITEMS} items"));
        }
        let next = items
            .iter()
            .filter_map(|i| i["id"].as_str()?.parse::<u32>().ok());
        let id = (next.max().unwrap_or(0) + 1).to_string();
        let name = pecofence_platform::shell::display_name(path).unwrap_or_else(|| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into()
        });
        let item = json!({ "id": id, "path": text, "name": name });
        self.set_launch_items(fence, items.into_iter().chain([item]).collect());
        tracing::info!(%fence, id, kind = pecofence_platform::shell::log_ext(path), "launcher item added");
        Ok(id)
    }

    /// Adds `paths` as items of `fence` (refused ones are left out, each with a log line) and
    /// makes their pictures; what the plugin sees of the ones added.
    fn add_launch_items(&mut self, fence: FenceId, paths: &[PathBuf]) -> Vec<Value> {
        let mut added = Vec::new();
        for path in paths {
            match self.add_launch_item(fence, path) {
                Ok(id) => added.push((id, path.clone())),
                Err(e) => tracing::info!(%fence, error = e, "launcher item not added"),
            }
        }
        let shown = (added.iter())
            .map(|(id, _)| {
                self.shown_item(
                    fence,
                    &json!({ "id": id, "name": self.launch_name(fence, id) }),
                )
            })
            .collect();
        self.make_launch_pictures(fence, added);
        shown
    }

    fn launch_name(&self, fence: FenceId, id: &str) -> Value {
        let items = self.launch_items(fence);
        let item = items.iter().find(|i| i["id"].as_str() == Some(id));
        item.map(|i| i["name"].clone()).unwrap_or_default()
    }

    /// The pictures of new items, made on a thread of their own the way fence icons are (shell
    /// extensions and network drives take their time there) and saved in the plugin's data
    /// folder while the plugin is installed; then the plugin hears `settings` and draws them.
    fn make_launch_pictures(&mut self, fence: FenceId, items: Vec<(String, PathBuf)>) {
        let Some(dir) = self.plugin_data_dir(fence).filter(|_| !items.is_empty()) else {
            return;
        };
        let (api, jobs, control) = (
            self.light_host(),
            self.light.jobs.clone(),
            self.control.hwnd().0 as isize,
        );
        let spawned = std::thread::Builder::new()
            .name("pecofence-launch-pictures".into())
            .spawn(move || {
                let _sta = pecofence_platform::com::OleGuard::init();
                let mut made = 0;
                for (id, path) in &items {
                    let saved = (pecofence_platform::shell::shell_image(path, 48, true))
                        .map_err(|e| e.to_string())
                        .and_then(|img| {
                            let file = dir.join(icon_name(fence, id));
                            api.write_data(fence, &file, img.bgra.len() as u64, |_| {
                                pecofence_platform::clipboard::save_png(
                                    &file, img.width, img.height, &img.bgra,
                                )
                                .map_err(|e| e.to_string())
                            })
                        });
                    match saved {
                        Ok(()) => made += 1,
                        Err(e) => {
                            tracing::info!(%fence, id, error = e, "launcher item: no picture saved")
                        }
                    }
                }
                tracing::debug!(%fence, made, items = items.len(), "launcher item pictures made");
                let job: super::light_host::UiJob = Box::new(move |app: &mut App| {
                    if made > 0 {
                        app.launch_redraw(fence);
                    }
                });
                Self::post_job(&jobs, control, job);
            });
        if let Err(e) = spawned {
            tracing::warn!(%fence, error = %e, "launcher item pictures: no thread");
        }
    }

    /// The plugin draws its items again, as after a change of its settings.
    fn launch_redraw(&mut self, fence: FenceId) {
        let settings = self.light_manifest(fence).map(|m| {
            let saved = &self.state.config.scripts[&fence].settings;
            super::plugins::plugin_settings(&m, saved)
        });
        if let Some(settings) = settings {
            self.plugin_event(fence, "settings", settings);
        }
    }

    fn set_launch_items(&mut self, fence: FenceId, items: Vec<Value>) {
        if let Some(spec) = self.state.config.scripts.get_mut(&fence) {
            spec.settings.insert(LAUNCH.into(), Value::Array(items));
            self.state.mark_dirty();
            self.schedule_save();
        }
    }

    /// Removes launcher item `id` of `fence` and its picture.
    fn remove_launch_item(&mut self, fence: FenceId, id: &str) -> bool {
        let mut items = self.launch_items(fence);
        let before = items.len();
        items.retain(|i| i["id"].as_str() != Some(id));
        if items.len() == before {
            return false;
        }
        if let Some(dir) = self.plugin_data_dir(fence) {
            let _ = std::fs::remove_file(dir.join(icon_name(fence, id)));
        }
        self.set_launch_items(fence, items);
        tracing::info!(%fence, id, "launcher item removed");
        true
    }

    /// `pf.launch.*` of fence `fence` (checked by the plugin host already), on this thread.
    pub(super) fn launch_call(
        &mut self,
        fence: FenceId,
        name: &str,
        args: &Value,
    ) -> std::result::Result<Value, String> {
        let id = args[0].as_str().unwrap_or_default().to_string();
        let item = || {
            self.launch_items(fence)
                .into_iter()
                .find(|i| i["id"].as_str() == Some(&id))
        };
        match name {
            "launch.list" => {
                let items = self.launch_items(fence);
                Ok(json!(
                    items
                        .iter()
                        .map(|i| self.shown_item(fence, i))
                        .collect::<Vec<_>>()
                ))
            }
            "launch.add" => {
                // [indexes, the drop's number (from the plugin host)]: that drop's files.
                let drop = (self.light.drops.iter())
                    .find(|d| Some(d.0) == args[1].as_u64() && d.1 == fence);
                let files = drop.ok_or("launch.add: that drop is gone")?.2.clone();
                let wanted = args[0]
                    .as_array()
                    .ok_or("launch.add takes a list of the drop's file numbers")?;
                let paths: Option<Vec<PathBuf>> = (wanted.iter())
                    .map(|i| files.get(i.as_u64()? as usize).cloned())
                    .collect();
                let paths = paths
                    .ok_or_else(|| format!("launch.add: the drop has {} files", files.len()))?;
                Ok(json!(self.add_launch_items(fence, &paths)))
            }
            "launch.pick" => {
                let owner = self.window_for(fence).map(|w| w.hwnd());
                let title = if id == "program" {
                    i18n::text("添加程序")
                } else {
                    i18n::text("添加文件")
                };
                match pecofence_platform::filedialog::pick_file(owner, title, id == "program") {
                    Ok(Some(path)) => {
                        let id = self.add_launch_item(fence, &path)?;
                        let name = self.launch_name(fence, &id);
                        let shown = self.shown_item(fence, &json!({ "id": id, "name": name }));
                        self.make_launch_pictures(fence, vec![(id, path)]);
                        Ok(shown)
                    }
                    _ => Ok(Value::Null),
                }
            }
            "launch.remove" => Ok(json!(self.remove_launch_item(fence, &id))),
            _ => {
                let path = item().and_then(|i| Some(PathBuf::from(i["path"].as_str()?)));
                let path = path.ok_or_else(|| format!("launch.open: no item {id}"))?;
                let _loads = pecofence_platform::shell_menu::LoadWatch::new("Launch");
                let opened = pecofence_platform::shell::shell_execute(&path, None, None);
                tracing::info!(%fence, id, ok = opened.is_ok(), kind = pecofence_platform::shell::log_ext(&path), "launcher item opened");
                opened.map(|_| Value::Null).map_err(|e| e.to_string())
            }
        }
    }

    /// What the settings page shows of fence `fence` of plugin `m`: its items (移除) and its
    /// commands with their shortcuts.
    pub(super) fn launch_json(
        &self,
        fence: FenceId,
        m: &pecofence_core::plugins::Manifest,
    ) -> (Value, Value) {
        let items: Vec<Value> = (self.launch_items(fence).iter())
            .map(|i| self.shown_item(fence, i))
            .collect();
        let saved = self.state.config.scripts.get(&fence);
        let saved = saved
            .and_then(|s| s.settings.get(KEYS)?.as_object().cloned())
            .unwrap_or_default();
        let lang = i18n::language().tag();
        let keys: Vec<Value> = (m.permissions.commands.iter())
            .map(|c| {
                let h = saved
                    .get(&c.id)
                    .and_then(|t| Hotkey::parse(t.as_str()?).ok());
                let k = h.map(|h| (fence, c.id.clone(), h));
                let taken = k
                    .as_ref()
                    .is_some_and(|k| self.light.keys_taken.contains(k));
                json!({
                    "id": c.id, "title": pecofence_core::plugins::pick(&c.title, lang),
                    "key": h.map(super::peek::hotkey_json), "taken": taken,
                })
            })
            .collect();
        (json!(items), json!(keys))
    }

    /// The page's 移除 of an item and a command's shortcut (recorded, or none for 清除).
    pub(super) fn launch_page_action(&mut self, name: &str, v: &Value) {
        let fence = v["fence"]
            .as_str()
            .and_then(|f| uuid::Uuid::parse_str(f).ok());
        let Some(fence) = fence.filter(|f| self.state.config.scripts.contains_key(f)) else {
            return;
        };
        if name == "pluginItemRemove" {
            if self.remove_launch_item(fence, v["item"].as_str().unwrap_or_default()) {
                self.launch_redraw(fence);
            }
            return;
        }
        let command = v["command"].as_str().unwrap_or_default().to_string();
        let declared = self.light_manifest(fence);
        if !declared.is_some_and(|m| m.permissions.commands.iter().any(|c| c.id == command)) {
            return tracing::warn!(%fence, "plugin shortcut for a command its plugin does not have");
        }
        let number = |key: &str| v.get(key).and_then(Value::as_u64);
        let h = number("vk").map(|vk| Hotkey::new(number("mods").unwrap_or(0) as u32, vk as u32));
        let recording = self.hotkeys_paused;
        self.resume_hotkeys();
        let refused = h.and_then(|h| self.plugin_key_problem(fence, &command, h));
        match &refused {
            Some(_) if recording => self.pause_hotkeys(),
            Some(_) => {}
            None => {
                if let Some(spec) = self.state.config.scripts.get_mut(&fence) {
                    let keys = spec.settings.entry(KEYS).or_insert_with(|| json!({}));
                    if let Some(keys) = keys.as_object_mut() {
                        match h {
                            Some(h) => keys.insert(command.clone(), h.to_string().into()),
                            None => keys.remove(&command),
                        };
                    }
                    self.state.mark_dirty();
                    self.schedule_save();
                }
                tracing::info!(%fence, command, hotkey = ?h.map(|h| h.to_string()), "plugin shortcut set");
                self.sync_plugin_keys();
            }
        }
        if let Some(host) = &self.settings {
            let box_id = v.get("box").cloned().unwrap_or_default();
            let result = json!({ "type": "pluginKeyResult", "box": box_id, "ok": refused.is_none(), "text": refused });
            host.post_json(&result.to_string());
        }
    }

    /// Why `h` cannot be the shortcut of `command` of `fence` (None = it can).
    fn plugin_key_problem(&self, fence: FenceId, command: &str, h: Hotkey) -> Option<String> {
        if let Err(problem) = h.check() {
            return Some(super::peek::hotkey_problem_text(problem, h));
        }
        let mine = self
            .light
            .keys
            .values()
            .any(|k| k.0 == fence && k.1 == command && k.2 == h);
        if mine {
            return None;
        }
        // PecoFence's own shortcuts by name: the Peek, the Windows desktop, dialog jump's Ctrl+G
        // (while it is on), another plugin command.
        let other = (self.plugin_keys_wanted().into_iter())
            .find(|k| k.2 == h && (k.0 != fence || k.1 != command));
        let lang = i18n::language().tag();
        let title = |(f, c, _): &(FenceId, String, Hotkey)| {
            let spec = self.state.config.scripts.get(f)?;
            let m = self.plugin(super::plugins::plugin_id(&spec.path)?)?;
            let c = m
                .manifest
                .permissions
                .commands
                .iter()
                .find(|x| x.id == *c)?;
            Some(pecofence_core::plugins::pick(&c.title, lang).to_string())
        };
        let dialog_jump = self.state.config.settings.tools.dialog_jump
            && h == Hotkey::new(pecofence_core::hotkey::CTRL, 0x47);
        let ours = if self.peek_hotkey == Some(h) {
            Some(i18n::text("速览").to_string())
        } else if self.windows_desktop_hotkey == Some(h) {
            Some(i18n::text("临时显示 Windows 桌面").to_string())
        } else if dialog_jump {
            Some(i18n::text("对话框跳转").to_string())
        } else {
            other.map(|k| title(&k).unwrap_or(k.1))
        };
        let label = super::peek::peek_hotkey_label(h);
        if let Some(name) = ours {
            tracing::info!(hotkey = %h, "plugin shortcut refused: one of PecoFence's own");
            return Some(i18n::format(
                "{0} 已经是“{1}”的快捷键，请换一个",
                &[label, name],
            ));
        }
        if !self.hotkey_free(h) {
            tracing::info!(hotkey = %h, "plugin shortcut refused: taken by Windows or another program");
            return Some(i18n::format(
                "{0} 已被 Windows 或其他程序占用，请换一个",
                &[label],
            ));
        }
        None
    }
}
