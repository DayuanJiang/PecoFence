//! Settings host window + page protocol: state JSON, toasts, message handling, config adoption,
//! import / export / restore, settings diff application.

use super::*;

impl App {
    /// Replaces the configuration wholesale (import / backup) and rebuilds everything.
    pub(super) fn adopt_config(&mut self, cfg: pecofence_core::Config, what: &str) {
        self.end_peek_now();
        let old_settings = self.state.config.settings.clone();
        self.state.replace_config(cfg);
        if let Some(desk) = shell::user_desktop() {
            self.state.migrate_desktop_path(&desk);
        }
        // What the file (plus the desktop-path migration) wants the settings to be.
        let new_settings = self.state.config.settings.clone();
        self.sync_desktop_if_available("config adopted");
        self.relayout_from_state();
        // Re-apply the settings through the diff path so the anchor / Run-key side effects
        // (hide_real_icons, quick_hide.enabled, show_desktop, autostart) and the ctx.behavior
        // cells update exactly as if the user had changed them on the page.
        self.state.config.settings = old_settings;
        self.apply_settings(new_settings);
        // apply_settings only refreshes on a visual / icon diff; a wholesale swap (per-fence
        // tint, layouts) needs these regardless.
        self.sync_peek_hotkey();
        self.apply_icon_variant();
        self.refresh_visuals(true);
        self.state.save_if_dirty();
        self.push_settings_state();
        self.settings_toast(&pecofence_core::i18n::format("已{0}", &[what.to_string()]));
    }

    pub(super) fn export_config(&mut self) {
        let owner = self.settings.as_ref().map(|h| h.hwnd());
        let name = format!(
            "pecofence-{}.json",
            pecofence_core::config_store::today_yyyy_mm_dd()
        );
        match pecofence_platform::filedialog::save_json(
            owner,
            pecofence_core::i18n::text("导出 PecoFence 配置"),
            &name,
        ) {
            Ok(Some(path)) => {
                self.state.save_if_dirty();
                match pecofence_core::ConfigStore::export_to(&self.state.config, &path) {
                    Ok(()) => self.settings_toast(&pecofence_core::i18n::format(
                        "已导出到 {0}",
                        &[format!("{}", path.display())],
                    )),
                    Err(e) => self.settings_error(&pecofence_core::i18n::format(
                        "导出失败：{0}",
                        &[e.to_string()],
                    )),
                }
            }
            Ok(None) => {}
            Err(e) => self.settings_error(&pecofence_core::i18n::format(
                "无法打开保存对话框：{0}",
                &[e.to_string()],
            )),
        }
    }

    pub(super) fn import_config(&mut self) {
        let owner = self.settings.as_ref().map(|h| h.hwnd());
        match pecofence_platform::filedialog::open_json(
            owner,
            pecofence_core::i18n::text("导入 PecoFence 配置"),
        ) {
            Ok(Some(path)) => self.restore_from_file(&path, pecofence_core::i18n::text("导入配置")),
            Ok(None) => {}
            Err(e) => self.settings_error(&pecofence_core::i18n::format(
                "无法打开文件对话框：{0}",
                &[e.to_string()],
            )),
        }
    }

    pub(super) fn restore_from_file(&mut self, path: &std::path::Path, what: &str) {
        match pecofence_core::ConfigStore::parse_file(path) {
            Ok(cfg) => {
                // Keep a snapshot of what we are replacing so the step can be undone.
                self.state
                    .save_snapshot(&pecofence_core::i18n::format("{0}前", &[what.to_string()]));
                self.adopt_config(cfg, what);
            }
            Err(e) => self.settings_error(&pecofence_core::i18n::format(
                "文件无法使用：{0}",
                std::slice::from_ref(&e),
            )),
        }
    }

    pub(super) fn open_settings(&mut self) {
        if let Some(h) = &self.settings {
            window::show_normal(h.hwnd());
            window::bring_to_front(h.hwnd());
            return;
        }
        if self.web_env.is_none() {
            match WebEnvironment::create() {
                Ok(env) => self.web_env = Some(env),
                Err(e) => {
                    tracing::error!(error = %e, "WebView2 environment failed");
                    if let Some(t) = &self.tray {
                        t.show_info(
                            "PecoFence",
                            pecofence_core::i18n::text("无法创建 WebView2 环境，请确认已安装 Microsoft Edge WebView2 运行时。"),
                            true,
                        );
                    }
                    return;
                }
            }
        }
        match SettingsHost::open(
            &self.settings_class,
            self.web_env.as_ref().unwrap(),
            self.theme_mode,
            self.ctx.theme.borrow().liquid_glass,
            self.queue.clone(),
        ) {
            Ok(mut host) => {
                // Caption icon: the same drawn fence glyph as the tray, at 16/32 DIP.
                let scale = monitors::dpi_for_window(host.hwnd()).max(96) as f32 / 96.0;
                let accent = self.ctx.theme.borrow().accent_rgb8();
                let dark = self.theme_mode == ThemeMode::Dark;
                let small = (16.0 * scale).round() as i32;
                let big = (32.0 * scale).round() as i32;
                host.set_icons(
                    (small, tray_icon_image(small, accent, dark)),
                    (big, tray_icon_image(big, accent, dark)),
                );
                self.settings = Some(host);
            }
            Err(e) => {
                tracing::error!(error = %e, "settings window failed");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text("无法打开设置窗口（WebView2 初始化失败），请检查 Microsoft Edge WebView2 运行时。"),
                        true,
                    );
                }
            }
        }
    }

    pub(super) fn settings_state_json(&self) -> String {
        let localization = pecofence_core::i18n::ui_payload();
        let fences: Vec<serde_json::Value> = self
            .state
            .fences()
            .iter()
            .map(|f| self.fence_options_json(f))
            .collect();
        let mem_mb = pecofence_platform::memstats::MemoryStats::current()
            .map(|m| m.private_working_set as f64 / (1024.0 * 1024.0))
            .unwrap_or(0.0);
        let snapshots: Vec<serde_json::Value> = self
            .state
            .config
            .snapshots
            .iter()
            .map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "name": s.name,
                    "ts": s.ts,
                    "date": pecofence_platform::fileinfo::format_local_datetime(s.ts),
                    "fenceCount": s.layouts.iter().map(|l| l.fences.len()).sum::<usize>(),
                })
            })
            .collect();
        let backups: Vec<serde_json::Value> = self
            .state
            .backup_files()
            .iter()
            .map(|p| {
                serde_json::json!({
                    "path": p.to_string_lossy(),
                    "name": p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                })
            })
            .collect();
        let monitors: Vec<serde_json::Value> = self
            .monitor_labels()
            .into_iter()
            .map(|(id, label)| serde_json::json!({ "id": id, "label": label }))
            .collect();
        let accent = {
            let [r, g, b] = self.ctx.theme.borrow().accent_rgb8();
            format!("#{r:02X}{g:02X}{b:02X}")
        };
        serde_json::json!({
            "type": "state",
            "locale": localization["locale"],
            "translations": localization["translations"],
            "settings": self.state.config.settings,
            "desktopIconsHidden": pecofence_platform::shell_icons::desktop_icons_hidden(),
            "rules": self.state.config.rules,
            "fences": fences,
            "snapshots": snapshots,
            "backups": backups,
            "monitors": monitors,
            "tintPalette": fence_options::tint_palette_json(),
            "version": env!("CARGO_PKG_VERSION"),
            "configPath": self.state.config_path().to_string_lossy(),
            "memoryMb": mem_mb,
            "itemCount": self.state.workspace_item_count(),
            "themeMode": if self.theme_mode == ThemeMode::Dark { "dark" } else { "light" },
            "accent": accent,
        })
        .to_string()
    }

    pub(super) fn push_settings_state(&self) {
        if let Some(h) = &self.settings {
            h.post_json(&self.settings_state_json());
        }
    }

    /// File watcher/navigation updates must not rebuild controls while the user is
    /// editing a name or has a select popup open.
    pub(super) fn push_workspace_summary(&self) {
        if let Some(h) = &self.settings {
            h.post_json(
                &serde_json::json!({
                    "type": "workspaceSummary",
                    "fenceCount": self.state.fences().len(),
                    "itemCount": self.state.workspace_item_count(),
                })
                .to_string(),
            );
        }
    }

    /// Call after mutating `self.state.config.settings` anywhere other than apply_settings:
    /// persists the change and keeps an open settings page in sync so its next
    /// whole-object patchSettings does not revert it.
    pub(super) fn settings_mutated(&mut self) {
        self.state.mark_dirty();
        self.schedule_save();
        self.push_settings_state();
    }

    pub(super) fn settings_toast(&self, text: &str) {
        if let Some(h) = &self.settings {
            h.post_json(&serde_json::json!({ "type": "toast", "text": text }).to_string());
        }
    }

    fn settings_error(&self, text: &str) {
        if let Some(h) = &self.settings {
            h.post_json(
                &serde_json::json!({ "type": "toast", "text": text, "error": true }).to_string(),
            );
        }
    }

    pub(super) fn on_settings_message(&mut self, json: &str) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else {
            return;
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("ready") => {
                tracing::info!("settings: page ready");
                self.push_settings_state();
                if let Some(fence) = self.settings_focus_fence.take() {
                    self.post_show_fence(fence);
                }
            }
            Some("setFence") => self.on_set_fence(&v),
            Some("patchSettings") => {
                let Some(new_settings) = v.get("settings").cloned() else {
                    return;
                };
                match serde_json::from_value::<pecofence_core::Settings>(new_settings) {
                    Ok(new_settings) => self.apply_settings(new_settings),
                    Err(e) => tracing::warn!(error = %e, "bad settings from page"),
                }
            }
            Some("setRules") => {
                if let Some(rules) = v.get("rules").cloned()
                    && let Ok(rules) = serde_json::from_value::<pecofence_core::RuleSet>(rules)
                {
                    self.set_rules(rules);
                }
            }
            Some("action") => match v.get("name").and_then(|n| n.as_str()) {
                Some("applyRules") => {
                    let entries = shell::enumerate_desktop();
                    let moved = self.state.apply_rules_all(&entries);
                    self.refresh_all();
                    self.schedule_save();
                    self.settings_toast(&pecofence_core::i18n::format(
                        "已按规则整理 {0} 个项目",
                        &[moved.to_string()],
                    ));
                }
                Some("addTemplate") => {
                    if let Some(template) = v
                        .get("template")
                        .and_then(|t| t.as_str())
                        .and_then(pecofence_core::rules::Template::parse)
                    {
                        self.add_template(template);
                    }
                }
                Some("openConfigFolder") => {
                    let dir = self.state.config_path();
                    if let Some(dir) = dir.parent() {
                        let _ = std::process::Command::new("explorer.exe").arg(dir).spawn();
                    }
                }
                Some("saveSnapshot") => {
                    let name = v.get("snapshotName").and_then(|n| n.as_str()).unwrap_or("");
                    self.state.save_snapshot(name);
                    self.schedule_save();
                    self.push_settings_state();
                    self.settings_toast(pecofence_core::i18n::text("已保存快照"));
                }
                Some("restoreSnapshot") => {
                    if let Some(id) = v
                        .get("id")
                        .and_then(|i| i.as_str())
                        .and_then(|i| uuid::Uuid::parse_str(i).ok())
                    {
                        // Snapshot of the current state first, so "restore" is reversible
                        // (the target is looked up before the backup can evict it).
                        if self
                            .state
                            .restore_snapshot_with_backup(id, pecofence_core::i18n::text("恢复前"))
                            .is_some()
                        {
                            self.end_peek_now();
                            self.relayout_from_state();
                            // Portals restored with the snapshot need enumerating; stale
                            // runtime state of the replaced ones is pruned on the way.
                            self.refresh_portals();
                            self.schedule_save();
                            self.push_settings_state();
                            self.settings_toast(pecofence_core::i18n::text("已恢复快照"));
                        }
                    }
                }
                Some("deleteSnapshot") => {
                    if let Some(id) = v
                        .get("id")
                        .and_then(|i| i.as_str())
                        .and_then(|i| uuid::Uuid::parse_str(i).ok())
                        && self.state.delete_snapshot(id)
                    {
                        self.schedule_save();
                        self.push_settings_state();
                    }
                }
                Some("swapMonitors") => {
                    let a = v
                        .get("a")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    let b = v
                        .get("b")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    if !a.is_empty() && !b.is_empty() {
                        self.swap_monitors(&a, &b);
                    }
                }
                Some("exportConfig") => self.export_config(),
                Some("importConfig") => self.import_config(),
                Some("restoreBackup") => {
                    if let Some(p) = v.get("path").and_then(|p| p.as_str()) {
                        let path = PathBuf::from(p);
                        // Only files from our own backups folder.
                        if self.state.backup_files().contains(&path) {
                            self.restore_from_file(&path, pecofence_core::i18n::text("恢复备份"));
                        }
                    }
                }
                Some("repairIcons") => self.set_desktop_icons_hidden(false),
                Some("hideDesktopIcons") => self.set_desktop_icons_hidden(true),
                _ => {}
            },
            _ => {}
        }
    }

    /// Replaces the rule set (settings page / CLI). A folder portal never shows `fence.items`:
    /// a rule routing into one would make matched items vanish from every fence, so such
    /// targets fall back to the inbox.
    pub(super) fn set_rules(&mut self, mut rules: pecofence_core::RuleSet) {
        let targets = std::iter::once(&mut rules.default_target)
            .chain(rules.list.iter_mut().map(|r| &mut r.target));
        for t in targets {
            if let Target::Fence(id) = *t
                && self
                    .state
                    .fence(id)
                    .is_some_and(|f| f.kind == FenceKind::FolderPortal)
            {
                *t = Target::Inbox;
            }
        }
        self.state.config.rules = rules;
        self.state.mark_dirty();
        self.schedule_save();
    }

    /// Both rescue buttons and the tray use the same idempotent operation. A second
    /// request must still repair Explorer's state if it changed outside PecoFence.
    pub(super) fn set_desktop_icons_hidden(&mut self, hidden: bool) {
        if self.apply_desktop_icons_hidden(hidden) {
            self.state.config.settings.hide_real_icons = hidden;
            self.settings_mutated();
            self.desktop_icons_setting_changed();
            self.settings_toast(if hidden {
                pecofence_core::i18n::text("Windows 桌面图标已重新隐藏。")
            } else {
                pecofence_core::i18n::text("桌面图标已显示，可点击“重新隐藏桌面图标”恢复隐藏。")
            });
        } else {
            self.push_settings_state();
            self.settings_error(pecofence_core::i18n::text(
                "桌面图标状态未能更改，请等待资源管理器恢复后重试。",
            ));
        }
    }

    /// The special desktop items (Recycle Bin, ...) enter or leave the inbox fence together
    /// with the hide-real-icons setting: resync now rather than at the next folder event.
    pub(super) fn desktop_icons_setting_changed(&mut self) {
        self.sync_desktop_if_available("hide-real-icons toggled");
        self.refresh_all();
        self.push_workspace_summary();
    }

    fn apply_desktop_icons_hidden(&self, hidden: bool) -> bool {
        let mut guard = self.anchor.borrow_mut();
        let Some(a) = guard.as_mut() else {
            return false;
        };
        if hidden {
            return a.hide_desktop_icons();
        }
        a.restore_desktop_icons();
        if pecofence_platform::shell_icons::desktop_icons_hidden()
            && let Some(h) = desktop::resolve_icon_host(a.generation)
        {
            let _ = pecofence_platform::shell_icons::set_desktop_icons_hidden(
                false, h.def_view, h.host,
            );
            // Clear our recovery marker only after Explorer actually restored the icons.
            a.restore_desktop_icons();
        }
        !pecofence_platform::shell_icons::desktop_icons_hidden()
    }

    /// Applies a full settings object from the page, reacting to what changed.
    pub(super) fn apply_settings(&mut self, mut new: pecofence_core::Settings) {
        let old = self.state.config.settings.clone();
        if new == old {
            return;
        }
        if new.autostart != old.autostart {
            let _ = pecofence_platform::autostart::set_product_enabled(new.autostart);
            if pecofence_platform::process::is_packaged() {
                // Store installs: Windows owns the startup task, so hand the user the switch.
                let _ = shell::shell_execute(
                    std::path::Path::new("ms-settings:startupapps"),
                    None,
                    None,
                );
            }
        }
        if new.hide_real_icons != old.hide_real_icons
            && !self.apply_desktop_icons_hidden(new.hide_real_icons)
        {
            new.hide_real_icons = old.hide_real_icons;
            self.settings_error(pecofence_core::i18n::text(
                "桌面图标状态未能更改，请稍后重试。",
            ));
        }
        let icons_toggled = new.hide_real_icons != old.hide_real_icons;
        if new.quick_hide.enabled != old.quick_hide.enabled
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.quick_hide_enabled = new.quick_hide.enabled;
        }
        if new.show_desktop != old.show_desktop
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.behavior = match new.show_desktop {
                ShowDesktopSetting::KeepVisible => ShowDesktopBehavior::KeepVisible,
                ShowDesktopSetting::HideWithDesktop => ShowDesktopBehavior::HideWithDesktop,
            };
        }
        self.ctx.behavior.hover_peek.set(new.roll_up.hover_peek);
        self.ctx.behavior.snapping.set(new.snapping.enabled);
        self.ctx
            .behavior
            .click_to_expand
            .set(new.roll_up.click_to_expand);
        self.ctx
            .behavior
            .title_on_hover
            .set(new.roll_up.title_on_hover);
        self.ctx
            .behavior
            .hide_inactive_scrollbar
            .set(new.roll_up.hide_inactive_scrollbar);
        let chrome_changed = new.roll_up.title_on_hover != old.roll_up.title_on_hover
            || new.roll_up.hide_inactive_scrollbar != old.roll_up.hide_inactive_scrollbar;
        if chrome_changed {
            for w in self.fences.values() {
                w.redraw();
            }
        }
        self.ctx
            .behavior
            .backdrop
            .set(backdrop_mode_for(new.backdrop));
        let visual_changed = new.theme != old.theme
            || new.theme_style != old.theme_style
            || new.backdrop != old.backdrop;
        let icons_changed = new.icons != old.icons;
        self.state.config.settings = new;
        self.refresh_language();
        self.state.mark_dirty();
        self.schedule_save();
        self.sync_peek_hotkey();
        if icons_toggled {
            self.desktop_icons_setting_changed();
        }
        if icons_changed {
            self.apply_icon_variant();
        }
        if visual_changed {
            self.refresh_visuals(true);
        }
        self.push_settings_state();
    }

    pub(super) fn refresh_language(&mut self) {
        let language = self
            .state
            .config
            .settings
            .language
            .resolve(pecofence_platform::locale::ui_language());
        if language == pecofence_core::i18n::language() {
            return;
        }
        pecofence_core::i18n::set_language(language);
        tracing::info!(language = language.tag(), "interface language changed");
        if let Some(host) = &self.settings {
            host.update_language();
        }
        for window in self.fences.values() {
            window.redraw();
        }
        self.push_settings_state();
    }
}
