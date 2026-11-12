//! Settings host window + page protocol: state JSON, toasts, message handling, config adoption,
//! import / export / restore, settings diff application.

use super::*;

/// How to write a theme file (Settings › 常规 › 主题风格 › 自定义 · 主题文件).
const THEME_GUIDE_URL: &str = "https://github.com/DayuanJiang/PecoFence/blob/main/docs/THEMES.md";

/// Largest snapping gap honoured, in DIPs; `snapping.gapPx` is clamped to `0..=` this.
pub(super) const MAX_SNAP_GAP_DIP: i32 = 64;

/// `snapping.gapPx` as the gap (DIPs) fence windows keep to each other and the work-area edges
/// while snapping.
pub(super) fn snap_gap_dip(snapping: &pecofence_core::SnappingSettings) -> i32 {
    snapping.gap_px.clamp(0, MAX_SNAP_GAP_DIP)
}

/// A script error on the settings page (`pageError`): its JavaScript error type and position
/// go to the log as fixed names and numbers; the page never sends the message, which can
/// quote what the user typed.
pub(super) fn log_page_error(v: &serde_json::Value) {
    const KINDS: &[&str] = &[
        "Error",
        "TypeError",
        "ReferenceError",
        "RangeError",
        "SyntaxError",
        "URIError",
        "EvalError",
        "AggregateError",
    ];
    let text = |key: &str| v.get(key).and_then(|k| k.as_str()).unwrap_or("");
    let kind = KINDS
        .iter()
        .find(|&&k| k == text("kind"))
        .copied()
        .unwrap_or("other");
    let file = text("file");
    let script = if file.ends_with("i18n.js") {
        "i18n.js"
    } else if file.is_empty() {
        "none"
    } else {
        "settings page"
    };
    let number = |key: &str| v.get(key).and_then(|n| n.as_u64()).unwrap_or(0);
    tracing::warn!(
        what = kind,
        at = script,
        line = number("line"),
        col = number("col"),
        "settings page script error"
    );
}

/// Where a wholesale configuration came from: names its toast and the undo snapshot.
#[derive(Clone, Copy)]
pub(super) enum ConfigSource {
    Import,
    Backup,
    /// The saved files, read once another program stopped holding them open.
    Unlocked,
}

impl ConfigSource {
    fn done(self) -> &'static str {
        match self {
            Self::Import => pecofence_core::i18n::text("已导入配置"),
            Self::Backup => pecofence_core::i18n::text("已恢复备份"),
            Self::Unlocked => pecofence_core::i18n::text("已读到配置文件，你的栅栏已恢复。"),
        }
    }

    /// The name of the snapshot taken before a restore from a file (an import or a backup;
    /// reading the unlocked files takes none).
    fn before(self) -> &'static str {
        match self {
            Self::Import => pecofence_core::i18n::text("导入配置前"),
            Self::Backup | Self::Unlocked => pecofence_core::i18n::text("恢复备份前"),
        }
    }
}

/// Room for one more snapshot: one the user saves (page or CLI) or one of the page's automatic
/// "…前" snapshots. Like the CLI's `auto_snapshot_slot`, it never evicts a snapshot the user
/// saved: a full list loses its oldest automatic one (the page's or the CLI's) instead, and
/// `false` means none is left to drop, so none is taken. `protect` (the snapshot about to be
/// restored) always stays. Automatic names saved in another UI language count as the user's.
pub(super) fn snapshot_room(
    snapshots: &mut Vec<pecofence_core::Snapshot>,
    protect: Option<uuid::Uuid>,
) -> bool {
    if snapshots.len() >= pecofence_core::MAX_SNAPSHOTS
        && let Some(i) = snapshots
            .iter()
            .position(|s| Some(s.id) != protect && is_auto_snapshot(&s.name))
    {
        snapshots.remove(i);
    }
    snapshots.len() < pecofence_core::MAX_SNAPSHOTS
}

/// A snapshot the app saved by itself (the page's 「…前」 ones in the current language, the
/// CLI's `auto-cli-…`): the ones a full list gives up, and the page marks them 「自动保存」.
pub(super) fn is_auto_snapshot(name: &str) -> bool {
    name.starts_with(super::ipc::AUTO_SNAPSHOT_PREFIX)
        || [
            pecofence_core::i18n::text("恢复快照前"),
            pecofence_core::i18n::text("删除栅栏前"),
            ConfigSource::Import.before(),
            ConfigSource::Backup.before(),
        ]
        .contains(&name)
}

/// What the settings window leaves for the next one of this run.
#[derive(Default)]
pub(super) struct SettingsKept {
    /// Where the window was closed: its rectangle (screen px) and the scale of its monitor.
    pub closed_at: Option<(RECT, f32)>,
    /// The page it showed (a `data-page` of the navigation) and how far it was scrolled.
    pub view: Option<(String, f64)>,
    /// The snapshot the page deleted last and its place in the list, for the notice's 撤销.
    /// Gone when the window closes.
    pub deleted_snapshot: Option<(usize, pecofence_core::Snapshot)>,
    /// The 「恢复快照前」 snapshot of the page's last restore, for the notice's 撤销.
    pub restore_undo: Option<uuid::Uuid>,
}

fn snapshot_limit_text() -> String {
    pecofence_core::i18n::format(
        "快照已达上限（{0} 个），未保存新快照。请删除不需要的快照。",
        &[pecofence_core::MAX_SNAPSHOTS.to_string()],
    )
}

/// A restore or an import that did not run: the snapshot of the layout it replaces had no
/// room. Says so, where [`snapshot_limit_text`] only speaks of the snapshot.
fn not_done_for_snapshot_limit_text() -> String {
    pecofence_core::i18n::format(
        "快照已达上限（{0} 个），无法先保存当前布局，所以什么都没有改。请删除不需要的快照后再试。",
        &[pecofence_core::MAX_SNAPSHOTS.to_string()],
    )
}

/// While another program holds the configuration files open, the fences on show are a
/// stand-in that is never saved: an import would be gone once the files are read, and an
/// export would write the stand-in.
pub(super) fn config_locked_text() -> &'static str {
    pecofence_core::i18n::text(
        "配置文件正被其他程序占用，现在不能导入、导出或恢复备份。等 PecoFence 读到配置文件后再试。",
    )
}

/// The names of the top-level settings that differ (`hideRealIcons,quickHide`): field names of
/// `Settings`, never their values (a path, a hotkey).
fn changed_settings(old: &pecofence_core::Settings, new: &pecofence_core::Settings) -> String {
    let (Ok(serde_json::Value::Object(old)), Ok(serde_json::Value::Object(new))) =
        (serde_json::to_value(old), serde_json::to_value(new))
    else {
        return String::new();
    };
    new.iter()
        .filter(|(key, value)| old.get(key.as_str()) != Some(value))
        .map(|(key, _)| key.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

impl App {
    /// Replaces the configuration wholesale (import / backup) and rebuilds everything. Returns
    /// how many scripts of an imported file are held off (the settings page says so in its
    /// notice, the CLI in its reply).
    pub(super) fn adopt_config(
        &mut self,
        mut cfg: pecofence_core::Config,
        source: ConfigSource,
    ) -> usize {
        self.end_peek_now();
        let old_settings = self.state.config.settings.clone();
        let imported = matches!(source, ConfigSource::Import);
        // A snapshot of an imported file brings its fences, never scripts: restoring it later
        // would start them with no word of warning. That goes for the scripts a snapshot keeps
        // itself and for those the file lists for fences that are in none of its layouts (only
        // a snapshot can show them).
        let snapshot_scripts: usize = if imported {
            let shown: std::collections::HashSet<FenceId> = cfg
                .layouts
                .iter()
                .flat_map(|l| &l.fences)
                .map(|f| f.id)
                .collect();
            let listed = cfg.scripts.len();
            cfg.scripts.retain(|id, _| shown.contains(id));
            let kept: usize = cfg
                .snapshots
                .iter_mut()
                .map(|s| std::mem::take(&mut s.scripts).len())
                .sum();
            listed - cfg.scripts.len() + kept
        } else {
            0
        };
        // The scripts the file itself brings (the user's own stay in the configuration for the
        // undo snapshot and are counted apart).
        let file_scripts = cfg.scripts.values().filter(|s| !s.path.is_empty()).count();
        self.state.replace_config(cfg);
        if let Some(desk) = shell::user_desktop() {
            self.state.migrate_desktop_path(&desk);
        }
        // What the file (plus the desktop-path migration) wants the settings to be. The Store
        // edition's startup switch is Windows': a file does not change it (nor open its page).
        let mut new_settings = self.state.config.settings.clone();
        // An imported file may come from someone else: its script fences do not run until the
        // user switches 小工具 on (a backup is the user's own and keeps the switch). A file that
        // has the switch off already gets the same words: its scripts start with the switch too.
        let scripts_held = imported && file_scripts > 0;
        if scripts_held || snapshot_scripts > 0 {
            tracing::info!(
                scripts = file_scripts,
                snapshot_scripts,
                switch_was_on = new_settings.tools.enabled,
                "imported script fences held off"
            );
        }
        if scripts_held {
            new_settings.tools.enabled = false;
        }
        if pecofence_platform::process::is_packaged() {
            new_settings.autostart = old_settings.autostart;
        }
        self.sync_desktop_if_available("config adopted");
        self.relayout_from_state_gliding();
        // The file's portals need enumerating; runtime state of the replaced ones is pruned.
        self.refresh_portals();
        // Re-apply the settings through the diff path so the anchor / Run-key side effects
        // (hide_real_icons, quick_hide.enabled, show_desktop, autostart) and the ctx.behavior
        // cells update exactly as if the user had changed them on the page.
        self.state.config.settings = old_settings;
        self.apply_settings(new_settings, "import");
        // apply_settings only refreshes on a visual / icon diff; a wholesale swap (per-fence
        // tint, layouts) needs these regardless.
        self.sync_peek_hotkey();
        self.sync_windows_desktop_hotkey();
        self.apply_icon_variant();
        self.refresh_visuals(true);
        // Written before the page lists the backups: the first save of a day adds one.
        self.state.save_and_wait("import");
        self.push_settings_state();
        if scripts_held {
            self.settings_toast(pecofence_core::i18n::text(
                "已导入配置。其中的脚本还没有运行。确认这些脚本可信后，在“小工具”页打开“小工具”开关。",
            ));
            file_scripts
        } else {
            self.settings_toast(source.done());
            0
        }
    }

    pub(super) fn export_config(&mut self) {
        if self.state.locked_config.is_some() {
            tracing::info!("export refused: the configuration files are locked");
            self.settings_error(config_locked_text());
            return;
        }
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

    /// 「自定义颜色」 swatch: the system colour dialog, opened on the colour the swatch shows.
    /// The page applies the answer to the swatch row named by `target`, like a swatch click.
    fn pick_color(&mut self, v: &serde_json::Value) {
        let Some(target) = v.get("target").and_then(|t| t.as_str()) else {
            return;
        };
        let owner = self.settings.as_ref().map(|h| h.hwnd());
        let initial = v
            .get("color")
            .and_then(|c| c.as_str())
            .and_then(fence_options::parse_hex);
        // The Windows accent itself: the theme's accent token is a lighter or darker step of it
        // (or a custom theme's own colour).
        let accent = self
            .accent
            .as_ref()
            .map_or_else(|| self.ctx.theme.borrow().accent_rgb8(), |p| p.accent);
        let Some(rgb) = pecofence_platform::colordialog::pick(owner, initial, accent) else {
            return;
        };
        let hex = fence_options::hex(rgb);
        if let Some(h) = &self.settings {
            h.post_json(
                &serde_json::json!({ "type": "pickedColor", "target": target, "hex": hex })
                    .to_string(),
            );
        }
        self.picked_colors.insert(target.to_string(), hex);
    }

    /// `TIMER_CONFIG_LOCKED`: the config file another program held open at start is read as
    /// soon as it opens, and replaces what showed meanwhile (which was never saved).
    pub(super) fn retry_locked_config(&mut self) {
        self.config_lock_tries += 1;
        let Some((outcome, report)) = self.state.load_after_lock() else {
            if let Some(path) = &self.state.locked_config {
                if self.config_lock_tries.is_multiple_of(30) {
                    tracing::info!(path = %path.display(), tries = self.config_lock_tries, "config file still locked by another program; saves held");
                }
                window::set_timer(
                    self.control.hwnd(),
                    super::TIMER_CONFIG_LOCKED,
                    super::CONFIG_LOCKED_RETRY_MS,
                );
            }
            return;
        };
        for (what, path, error) in &report.failures {
            let what = *what;
            tracing::warn!(what, path = %path.display(), error = %error, "config file unusable");
        }
        let source = match &outcome {
            pecofence_core::LoadOutcome::Primary(_) => "config.json",
            pecofence_core::LoadOutcome::Recovered(_, from)
                if from.extension().is_some_and(|e| e == "bak") =>
            {
                "config.bak"
            }
            pecofence_core::LoadOutcome::Recovered(..) => "daily backup",
            pecofence_core::LoadOutcome::Fresh(_) => "new: files unusable",
        };
        // What the tray says: the user's file was read, or (it opened but could not be used,
        // and was kept aside) what shows in its place, as at a start with such a file.
        let told = match &outcome {
            pecofence_core::LoadOutcome::Recovered(_, from) if report.moved_to.is_some() => {
                Some(pecofence_core::i18n::format(
                    "配置文件读不出来，现在用的是备份 {0}。原来的文件已改名，仍在配置文件夹里。",
                    &[from
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()],
                ))
            }
            pecofence_core::LoadOutcome::Fresh(_) if report.moved_to.is_some() => Some(
                pecofence_core::i18n::text(
                    "配置文件读不出来，也没有可用的备份，现在是默认的栅栏。原来的文件已改名，仍在配置文件夹里。",
                )
                .to_string(),
            ),
            _ => None,
        };
        tracing::info!(
            source,
            tries = self.config_lock_tries,
            moved_aside = report.moved_to.is_some(),
            told_unreadable = told.is_some(),
            "config files readable again; loaded"
        );
        self.adopt_config(outcome.into_config(), ConfigSource::Unlocked);
        if let Some(t) = &self.tray {
            match &told {
                Some(text) => t.show_info("PecoFence", text, true),
                None => t.show_info("PecoFence", ConfigSource::Unlocked.done(), false),
            }
        }
    }

    pub(super) fn import_config(&mut self) {
        let owner = self.settings.as_ref().map(|h| h.hwnd());
        match pecofence_platform::filedialog::open_json(
            owner,
            pecofence_core::i18n::text("导入 PecoFence 配置"),
        ) {
            Ok(Some(path)) => self.restore_from_file(&path, ConfigSource::Import),
            Ok(None) => {}
            Err(e) => self.settings_error(&pecofence_core::i18n::format(
                "无法打开文件对话框：{0}",
                &[e.to_string()],
            )),
        }
    }

    /// After the page restored a snapshot's layouts (or took the restore back): the windows
    /// follow, and the page lists the snapshots as they are now.
    fn show_restored_layout(&mut self) {
        self.end_peek_now();
        // Files that came to the desktop after the snapshot are in none of its fences: they
        // are filed now, not at the next desktop change.
        self.sync_desktop_if_available("snapshot restored");
        self.relayout_from_state_gliding();
        // Portals restored with the snapshot need enumerating; stale runtime state of the
        // replaced ones is pruned on the way.
        self.refresh_portals();
        self.schedule_save();
        self.push_settings_state();
    }

    pub(super) fn restore_from_file(&mut self, path: &std::path::Path, source: ConfigSource) {
        if self.state.locked_config.is_some() {
            tracing::info!("import or backup restore refused: the configuration files are locked");
            self.settings_error(config_locked_text());
            return;
        }
        match pecofence_core::ConfigStore::parse_file(path) {
            Ok(cfg) => {
                // Keep a snapshot of what we are replacing so the step can be undone; a list
                // full of the user's own snapshots has no room for it, so nothing is replaced.
                if !snapshot_room(&mut self.state.config.snapshots, None) {
                    self.settings_error(&not_done_for_snapshot_limit_text());
                    return;
                }
                let undo = self.state.save_snapshot(source.before());
                self.state.keep_tool_data_in(undo);
                self.adopt_config(cfg, source);
            }
            Err(e) => {
                tracing::warn!(error = %e, "config file to restore unusable");
                let mut text =
                    pecofence_core::i18n::format("无法使用此文件：{0}", &[e.to_string()]);
                if e.not_encrypted() {
                    text = format!("{text} {}", not_encrypted_hint());
                }
                self.settings_error(&text);
            }
        }
    }

    /// Opens the settings window on `page` (a `data-page` of the navigation), scrolled to the
    /// element with id `focus`.
    pub(super) fn open_settings_page(&mut self, page: &'static str, focus: Option<&'static str>) {
        self.open_settings();
        match &self.settings {
            Some(host) if host.ready => self.post_show_page(page, focus),
            Some(_) => self.settings_focus_page = Some((page, focus)),
            None => {}
        }
    }

    fn post_show_page(&self, page: &str, focus: Option<&str>) {
        tracing::info!(page, focus, "settings page asked to show a page");
        if let Some(h) = &self.settings {
            h.post_json(
                &serde_json::json!({ "type": "showPage", "page": page, "focus": focus })
                    .to_string(),
            );
        }
    }

    pub(super) fn open_settings(&mut self) {
        self.query_store_startup();
        if let Some(h) = &self.settings {
            window::show_normal(h.hwnd());
            window::bring_to_front(h.hwnd());
            return;
        }
        if !self.ensure_web_env() {
            return;
        }
        match SettingsHost::open(
            &self.settings_class,
            self.web_env.as_ref().unwrap(),
            self.theme_mode,
            self.ctx.theme.borrow().liquid_glass,
            self.queue.clone(),
            HostKind::Settings,
            pecofence_core::i18n::text("PecoFence 设置"),
            None,
            self.settings_kept.closed_at,
        ) {
            Ok(mut host) => {
                self.set_host_icons(&mut host);
                self.settings = Some(host);
            }
            Err(_) if window::quit_posted() => window::post_quit(0),
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

    /// The WebView2 environment the settings and the smart fence editor share, made on first
    /// use. False when it cannot be made (the tray says why).
    pub(super) fn ensure_web_env(&mut self) -> bool {
        if self.web_env.is_some() {
            return true;
        }
        match WebEnvironment::create() {
            Ok(env) => {
                self.web_env = Some(env);
                true
            }
            // WebView2's wait for its callback pumps messages and ends with an error when it
            // takes a WM_QUIT (the session ends): quit after all.
            Err(_) if window::quit_posted() => {
                window::post_quit(0);
                false
            }
            Err(e) => {
                tracing::error!(error = %e, "WebView2 environment failed");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        pecofence_core::i18n::text(
                            "无法创建 WebView2 环境，请确认已安装 Microsoft Edge WebView2 运行时。",
                        ),
                        true,
                    );
                }
                false
            }
        }
    }

    /// Caption icon of a settings or editor window: the same drawn fence glyph as the tray,
    /// at 16/32 DIP.
    pub(super) fn set_host_icons(&self, host: &mut SettingsHost) {
        let scale = monitors::dpi_for_window(host.hwnd()).max(96) as f32 / 96.0;
        let accent = self.ctx.theme.borrow().accent_rgb8();
        let dark = self.theme_mode == ThemeMode::Dark;
        let small = (16.0 * scale).round() as i32;
        let big = (32.0 * scale).round() as i32;
        host.set_icons(
            (small, tray_icon_image(small, accent, dark)),
            (big, tray_icon_image(big, accent, dark)),
        );
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
                    "fenceCount": super::ipc::snapshot_dto(s, &self.state.work_areas).fence_count,
                    "auto": is_auto_snapshot(&s.name),
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
            // 在哪个虚拟桌面显示: the desktops in Task View order.
            "virtualDesktops": self
                .desktop_labels()
                .into_iter()
                .map(|(id, name, current)| serde_json::json!({ "id": id, "name": name, "current": current }))
                .collect::<Vec<_>>(),
            "pickedColors": self.picked_colors,
            "version": env!("CARGO_PKG_VERSION"),
            "storeEdition": pecofence_platform::process::is_packaged(),
            // Store edition: Windows' state of the startup switch (`StartupState` names).
            "startupState": self.store_startup_state.map(|s| format!("{s:?}")),
            "cliCommand": cli_command(),
            "configPath": self.state.config_path_on_disk().to_string_lossy(),
            "memoryMb": mem_mb,
            "itemCount": self.state.workspace_item_count(),
            "themeMode": if self.theme_mode == ThemeMode::Dark { "dark" } else { "light" },
            // The style the fences show, which a custom theme's base decides.
            "styleInUse": if self.ctx.theme.borrow().liquid_glass { "liquidGlass" } else { "fluent" },
            "customThemes": custom_theme_names(&self.state),
            "customThemeError": self.custom_theme_error,
            "accent": accent,
            "update": self.update_json(),
            "peekHotkey": self.peek_hotkey_json(),
            "windowsDesktopHotkey": self.windows_desktop_hotkey_json(),
            "everything": self.everything_json(),
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

    /// Store edition: reads Windows' startup switch for the package on a worker; the result comes
    /// back through `WM_APP_STARTUP_TASK` to [`Self::finish_store_startup`].
    pub(super) fn query_store_startup(&self) {
        if !pecofence_platform::process::is_packaged() {
            return;
        }
        let result = self.store_startup.clone();
        let control = self.control.hwnd().0 as isize;
        let _ = std::thread::Builder::new()
            .name("pecofence-startup-task".into())
            .spawn(
                move || match pecofence_platform::store::startup_task_state() {
                    Ok(state) => {
                        result.store(state as u8, std::sync::atomic::Ordering::Relaxed);
                        window::post_message(
                            HWND(control as *mut core::ffi::c_void),
                            WM_APP_STARTUP_TASK,
                            0,
                            0,
                        );
                    }
                    Err(e) => tracing::warn!(error = %e, "reading the startup task failed"),
                },
            );
    }

    /// Store edition: a click on 「随 Windows 启动」 switches Windows' startup task for the package
    /// on a worker, where the app may (`store::set_startup_task`). The state comes back as a read
    /// does; when Windows kept it as it was, [`Self::finish_store_startup`] opens Windows'
    /// Startup settings, where the user can change it.
    fn switch_store_startup(&mut self, on: bool) {
        let result = self.store_startup.clone();
        let refused = self.store_startup_refused.clone();
        let control = self.control.hwnd().0 as isize;
        let (done, done_receiver) = std::sync::mpsc::channel();
        self.store_startup_done = Some(done_receiver);
        let _ = std::thread::Builder::new()
            .name("pecofence-startup-task".into())
            .spawn(move || {
                let state = match pecofence_platform::store::set_startup_task(on) {
                    Ok((before, after)) => {
                        tracing::info!(on, old = ?before, new = ?after, "startup task set");
                        Some(after)
                    }
                    Err(e) => {
                        tracing::warn!(on, error = %e, "setting the startup task failed");
                        pecofence_platform::store::startup_task_state().ok()
                    }
                };
                if state.is_none_or(|s| s.is_on() != on) {
                    refused.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                if let Some(state) = state {
                    result.store(state as u8, std::sync::atomic::Ordering::Relaxed);
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_STARTUP_TASK,
                    0,
                    0,
                );
                let _ = done.send(());
            });
    }

    /// Store edition: waits briefly for the startup task switch a CLI call started, so the reply
    /// shows Windows' state (and the rollback warning when Windows kept it).
    pub(super) fn wait_store_startup_switch(&mut self) {
        let Some(done) = self.store_startup_done.take() else {
            return;
        };
        match done.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(()) => self.finish_store_startup(),
            Err(_) => tracing::info!("startup task not set in time; reply as asked"),
        }
    }

    /// Store edition: reads Windows' startup switch on a worker and waits for it briefly, so a
    /// `pecofence-cli settings get` reply shows a change made in Windows Settings meanwhile.
    pub(super) fn follow_store_startup_now(&mut self) {
        if !pecofence_platform::process::is_packaged() {
            return;
        }
        let (send, receive) = std::sync::mpsc::channel();
        let _ = std::thread::Builder::new()
            .name("pecofence-startup-task".into())
            .spawn(move || {
                let _ = send.send(pecofence_platform::store::startup_task_state());
            });
        match receive.recv_timeout(std::time::Duration::from_millis(800)) {
            Ok(Ok(state)) => {
                self.store_startup
                    .store(state as u8, std::sync::atomic::Ordering::Relaxed);
                self.finish_store_startup();
            }
            Ok(Err(e)) => tracing::warn!(error = %e, "reading the startup task failed"),
            Err(_) => tracing::info!("startup task state not read in time; reply as saved"),
        }
    }

    /// 「随 Windows 启动」 follows the switch Windows keeps for the Store package. After a click
    /// the app could not carry out, Windows' Startup settings open.
    pub(super) fn finish_store_startup(&mut self) {
        let refused = self
            .store_startup_refused
            .swap(false, std::sync::atomic::Ordering::Relaxed);
        let read = pecofence_platform::store::StartupState::from_code(
            self.store_startup
                .load(std::sync::atomic::Ordering::Relaxed),
        );
        if let Some(state) = read {
            let changed = self.store_startup_state != Some(state);
            if changed {
                tracing::info!(old = ?self.store_startup_state, new = ?state, "startup task state");
                self.store_startup_state = Some(state);
            }
            let on = state.is_on();
            if self.state.config.settings.autostart != on {
                tracing::info!(on, "startup switch follows Windows");
                self.state.config.settings.autostart = on;
                self.settings_mutated();
            } else if changed {
                self.push_settings_state();
            }
        }
        if refused {
            tracing::info!("startup switch kept by Windows; opening Windows' Startup settings");
            let _ =
                shell::shell_execute(std::path::Path::new("ms-settings:startupapps"), None, None);
        }
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
                // The first `ready` of a window: how long WebView2 took to show the page.
                let ms = self
                    .settings
                    .as_ref()
                    .filter(|h| !h.ready)
                    .map(|h| h.opened.elapsed().as_millis() as u64);
                tracing::info!(ms, "settings: page ready");
                if let Some(h) = self.settings.as_mut() {
                    h.ready = true;
                }
                // A reload in the middle of recording a shortcut never says it ended.
                self.resume_hotkeys();
                self.push_settings_state();
                // The window of a first `ready` shows once the page has drawn this state
                // (`drawn`); a reload's stays as the user left it.
                if let Some(h) = self
                    .settings
                    .as_ref()
                    .filter(|h| !desktop::is_visible(h.hwnd()))
                {
                    h.show_soon();
                }
                // Nothing asked for a page (栅栏选项…, a tip's 打开设置, the feedback form):
                // the page this run's last window was closed on, scrolled as it was.
                let asked = self.settings_focus_page.is_some()
                    || self.settings_focus_fence.is_some()
                    || self.feedback.focus.is_some();
                if !asked
                    && let Some((page, scroll)) = &self.settings_kept.view
                    && let Some(h) = &self.settings
                {
                    tracing::debug!(page, scroll, "settings: page shown as it was closed");
                    h.post_json(
                        &serde_json::json!({ "type": "showPage", "page": page, "scroll": scroll })
                            .to_string(),
                    );
                }
                if let Some((page, focus)) = self.settings_focus_page.take() {
                    self.post_show_page(page, focus);
                }
                if let Some(fence) = self.settings_focus_fence.take() {
                    self.post_show_fence(fence);
                }
                self.show_pending_feedback();
            }
            // The page has drawn the state (and the page or fence it was asked for): the
            // window appears with the right colours and the real values.
            Some("drawn") => {
                if let Some(h) = &self.settings
                    && h.show()
                {
                    let ms = h.opened.elapsed().as_millis() as u64;
                    tracing::info!(ms, "settings: page drawn, window shown");
                }
            }
            // The page shown and how far it is scrolled, kept for the next open in this run.
            Some("view") => {
                if let Some(page) = v.get("page").and_then(|p| p.as_str()) {
                    let scroll = v.get("scroll").and_then(|s| s.as_f64()).unwrap_or(0.0);
                    self.settings_kept.view = Some((page.to_string(), scroll));
                }
            }
            Some("setFence") => self.on_set_fence(&v),
            Some("ringFence") => {
                let host = v
                    .get("id")
                    .and_then(|i| i.as_str())
                    .and_then(|i| uuid::Uuid::parse_str(i).ok())
                    .map(|id| self.state.host_of(id));
                self.set_settings_ring(host);
            }
            Some("patchSettings") => {
                let Some(new_settings) = v.get("settings").cloned() else {
                    return;
                };
                match serde_json::from_value::<pecofence_core::Settings>(new_settings) {
                    Ok(new_settings) => self.apply_settings(new_settings, "settings"),
                    Err(e) => tracing::warn!(error = %e, "bad settings from page"),
                }
            }
            Some("setRules") => {
                // A rule list the core cannot read would drop the user's edit without a trace.
                match v
                    .get("rules")
                    .cloned()
                    .map(serde_json::from_value::<pecofence_core::RuleSet>)
                {
                    Some(Ok(rules)) => {
                        tracing::info!(rules = rules.list.len(), "rules saved from page");
                        self.set_rules(rules);
                    }
                    Some(Err(e)) => {
                        // The page shows the stored rules again and says the edit was not saved.
                        tracing::warn!(error = %e, "bad rules from page: not saved, stored rules sent back");
                        self.push_settings_state();
                        self.settings_error(pecofence_core::i18n::text(
                            "没能保存这次改动，规则已恢复原样",
                        ));
                    }
                    None => tracing::warn!("setRules from page without rules"),
                }
            }
            Some("action") => match v.get("name").and_then(|n| n.as_str()) {
                Some("applyRules") => {
                    // The notice with 撤销 says how many moved; without it, the page does.
                    let (moved, step) = self.apply_rules_now();
                    if moved == 0 {
                        // Nothing moved: say that, not "0 items sorted".
                        self.settings_toast(pecofence_core::i18n::text(
                            "没有需要整理的项目，图标都已按规则放好。",
                        ));
                    } else if !step.is_some_and(|id| self.offer_undo_refiled(id, moved)) {
                        self.settings_toast(&pecofence_core::i18n::format(
                            "已按规则整理 {0} 个项目",
                            &[moved.to_string()],
                        ));
                    }
                }
                Some(name @ ("newNote" | "newClock" | "newScript" | "newScriptExample")) => {
                    let example = v
                        .get("example")
                        .and_then(|e| e.as_str())
                        .and_then(crate::scripts::Example::parse);
                    // Clear of this window, which would cover it (`place_new_tool`). Only a
                    // maximized window leaves no such spot: then beside the middle of the
                    // primary monitor's work area, where a free spot is looked for.
                    let at = self
                        .state
                        .work_areas
                        .first()
                        .map(|w| ((w.left + w.right) / 2, (w.top + w.bottom) / 2));
                    if let Some((x, y)) = at {
                        self.tool_clear_of = self
                            .settings
                            .as_ref()
                            .map(|h| window::window_rect(h.hwnd()));
                        match name {
                            "newNote" => self.new_note_near(x, y),
                            "newClock" => self.new_clock_near(x, y),
                            "newScript" => self.new_script_near(x, y, None),
                            _ => match example {
                                Some(kind) => self.new_script_example_of(kind, x, y),
                                None => tracing::warn!("unknown script example on the page"),
                            },
                        }
                        self.tool_clear_of = None;
                    }
                }
                Some(name @ ("editSmart" | "refreshSmart")) => {
                    let fence = v
                        .get("id")
                        .and_then(|i| i.as_str())
                        .and_then(|i| uuid::Uuid::parse_str(i).ok())
                        .filter(|id| self.state.smart_spec(*id).is_some());
                    match (name, fence) {
                        // 编辑条件…: the editor opens on the settings window's monitor.
                        ("editSmart", Some(id)) => {
                            let at = self
                                .settings
                                .as_ref()
                                .map(|h| window::window_rect(h.hwnd()))
                                .map_or((0, 0), |r| {
                                    ((r.left + r.right) / 2, (r.top + r.bottom) / 2)
                                });
                            self.open_smart_editor(Some(id), at, "settings page");
                        }
                        (_, Some(id)) => {
                            self.run_smart(id, "settings page");
                            self.push_settings_state();
                        }
                        (_, None) => tracing::warn!(name, "smart fence action for no smart fence"),
                    }
                }
                Some("installEverything") => self.install_everything(),
                Some("startEverything") => self.start_everything(),
                Some("useEverything15") => self.use_everything_15(),
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
                    // Explorer opens Documents for a path it cannot see: hand it the real one.
                    use std::os::windows::process::CommandExt;
                    let path = self.state.config_path_on_disk();
                    let mut explorer = std::process::Command::new("explorer.exe");
                    if path.is_file() {
                        // Quoted as a whole (what `arg` does once a space appears), the
                        // `/select` argument opens Documents too.
                        explorer.raw_arg(format!("/select,\"{}\"", path.display()));
                    } else {
                        explorer.arg(path.parent().unwrap_or(&path));
                    }
                    let _ = explorer.spawn();
                }
                Some("openThemesFolder") => {
                    // `quiet`: 自定义 was chosen with no theme file yet. The examples are
                    // written as for the button, without an Explorer window.
                    let quiet = v.get("quiet").and_then(|q| q.as_bool()).unwrap_or(false);
                    let dir = self.state.themes_dir();
                    tracing::debug!(quiet, exists = dir.exists(), "settings: themes folder");
                    if !dir.exists() {
                        match std::fs::create_dir_all(&dir) {
                            Ok(()) => write_example_themes(&dir),
                            Err(e) => {
                                tracing::warn!(dir = %dir.display(), error = %e, "themes folder not made")
                            }
                        }
                    }
                    self.sync_theme_watcher();
                    // The examples were written before the watcher started: list them now.
                    self.push_settings_state();
                    if !quiet {
                        // The Store package's copy of %APPDATA%, where Explorer can find it.
                        let _ = std::process::Command::new("explorer.exe")
                            .arg(crate::state::canonical_path(dir))
                            .spawn();
                    }
                }
                Some("openThemeGuide") => {
                    let _ = shell::shell_execute(Path::new(THEME_GUIDE_URL), None, None);
                }
                // 查找设置 in a Chinese interface: the pinyin of the page's Chinese characters.
                Some("pinyin") => {
                    let chars = v.get("chars").and_then(|c| c.as_str()).unwrap_or("");
                    let map: serde_json::Map<String, serde_json::Value> = chars
                        .chars()
                        .filter_map(|c| {
                            let syl = pecofence_core::matcher::syllable(c)?;
                            Some((c.to_string(), serde_json::Value::from(syl)))
                        })
                        .collect();
                    if let Some(host) = &self.settings {
                        host.post_json(
                            &serde_json::json!({ "type": "pinyin", "map": map }).to_string(),
                        );
                    }
                }
                Some("saveSnapshot") => {
                    // A full list drops its oldest automatic snapshot for it (as the CLI
                    // does); only a list of the user's own snapshots refuses it.
                    if !snapshot_room(&mut self.state.config.snapshots, None) {
                        self.settings_error(&snapshot_limit_text());
                    } else {
                        let name = v.get("snapshotName").and_then(|n| n.as_str()).unwrap_or("");
                        self.state.save_snapshot(name);
                        self.schedule_save();
                        self.push_settings_state();
                        self.settings_toast(pecofence_core::i18n::text("已保存快照"));
                    }
                }
                Some("restoreSnapshot") => {
                    if let Some(id) = v
                        .get("id")
                        .and_then(|i| i.as_str())
                        .and_then(|i| uuid::Uuid::parse_str(i).ok())
                        && self.state.config.snapshots.iter().any(|s| s.id == id)
                    {
                        // Snapshot of the current state first, so "restore" is reversible;
                        // a list full of the user's own snapshots has no room for it, so
                        // nothing is restored.
                        if !snapshot_room(&mut self.state.config.snapshots, Some(id)) {
                            self.settings_error(&not_done_for_snapshot_limit_text());
                            return;
                        }
                        let before = self.state.restore_snapshot_with_backup(
                            id,
                            pecofence_core::i18n::text("恢复快照前"),
                        );
                        self.show_restored_layout();
                        // The notice offers 撤销 for a few seconds, as after a delete: the
                        // layout from before is in the snapshot just taken.
                        self.settings_kept.restore_undo = before;
                        if let Some(h) = &self.settings {
                            h.post_json(
                                &serde_json::json!({
                                    "type": "toast",
                                    "text": pecofence_core::i18n::text("已恢复快照"),
                                    "undo": "undoRestoreSnapshot",
                                })
                                .to_string(),
                            );
                        }
                    }
                }
                // 撤销 on the restore notice: the layout from before the restore comes back
                // and the 「恢复快照前」 snapshot that held it goes, its work done.
                Some("undoRestoreSnapshot") => {
                    if let Some(before) = self.settings_kept.restore_undo.take()
                        && self.state.restore_snapshot(before)
                    {
                        self.state.delete_snapshot(before);
                        tracing::info!("snapshot restore undone from the page's notice");
                        self.show_restored_layout();
                    }
                }
                Some("deleteSnapshot") => {
                    if let Some(id) = v
                        .get("id")
                        .and_then(|i| i.as_str())
                        .and_then(|i| uuid::Uuid::parse_str(i).ok())
                        && let Some(at) =
                            self.state.config.snapshots.iter().position(|s| s.id == id)
                    {
                        // One click deletes. The page's notice offers 撤销 for a few seconds:
                        // the snapshot is kept for it.
                        let gone = self.state.config.snapshots[at].clone();
                        self.state.delete_snapshot(id);
                        self.schedule_save();
                        self.push_settings_state();
                        tracing::debug!(
                            index = at,
                            "snapshot deleted on the page, kept for its undo"
                        );
                        if let Some(h) = &self.settings {
                            let text = pecofence_core::i18n::format(
                                "已删除快照“{0}”",
                                std::slice::from_ref(&gone.name),
                            );
                            let undo = "restoreDeletedSnapshot";
                            h.post_json(
                                &serde_json::json!({ "type": "toast", "text": text, "undo": undo })
                                    .to_string(),
                            );
                        }
                        self.settings_kept.deleted_snapshot = Some((at, gone));
                    }
                }
                // 撤销 on the delete notice: the snapshot goes back to its place in the list,
                // with its name and content.
                Some("restoreDeletedSnapshot") => {
                    if let Some((at, snapshot)) = self.settings_kept.deleted_snapshot.take() {
                        if !snapshot_room(&mut self.state.config.snapshots, None) {
                            self.settings_error(&snapshot_limit_text());
                            return;
                        }
                        let list = &mut self.state.config.snapshots;
                        let at = at.min(list.len());
                        list.insert(at, snapshot);
                        self.state.mark_dirty();
                        self.schedule_save();
                        self.push_settings_state();
                        tracing::info!(index = at, "deleted snapshot put back from the page");
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
                Some("pickColor") => self.pick_color(&v),
                Some("exportConfig") => self.export_config(),
                Some("importConfig") => self.import_config(),
                Some("restoreBackup") => {
                    if let Some(p) = v.get("path").and_then(|p| p.as_str()) {
                        let path = PathBuf::from(p);
                        // Only files from our own backups folder.
                        if self.state.backup_files().contains(&path) {
                            self.restore_from_file(&path, ConfigSource::Backup);
                        }
                    }
                }
                Some(name @ ("feedbackInfo" | "feedbackResult")) => {
                    self.on_feedback_action(name, &v);
                }
                Some("pageError") => log_page_error(&v),
                Some(
                    name @ ("checkUpdate" | "installUpdate" | "openReleasePage"
                    | "openDownloadPage"),
                ) => {
                    self.on_update_action(name);
                }
                Some("setPeekHotkey") => {
                    let number = |key: &str| v.get(key).and_then(|n| n.as_u64()).unwrap_or(0);
                    self.set_peek_hotkey_from_page(number("mods") as u32, number("vk") as u32);
                }
                Some("peekHotkeyRecording" | "windowsDesktopHotkeyRecording") => {
                    if v.get("on").and_then(|o| o.as_bool()).unwrap_or(false) {
                        self.pause_hotkeys();
                    } else {
                        self.resume_hotkeys();
                    }
                }
                Some("setWindowsDesktopHotkey") => {
                    // Without a key: 清除.
                    let number = |key: &str| v.get(key).and_then(|n| n.as_u64());
                    let h = number("vk")
                        .map(|vk| Hotkey::new(number("mods").unwrap_or(0) as u32, vk as u32));
                    self.set_windows_desktop_hotkey_from_page(h);
                }
                Some("repairIcons") => self.set_desktop_icons_hidden(false),
                Some("hideDesktopIcons") => self.set_desktop_icons_hidden(true),
                Some("rateInStore") => self.open_store_rating(),
                Some("openManual") => self.open_manual(),
                Some("scriptGuide") => self.open_manual_at("#write-scripts"),
                Some("openRepository") => {
                    let url = pecofence_core::brand::REPOSITORY_URL;
                    let _ = shell::shell_execute(std::path::Path::new(url), None, None);
                }
                Some("openCliDocs") => {
                    let url = pecofence_core::brand::CLI_DOCS_URL;
                    let _ = shell::shell_execute(std::path::Path::new(url), None, None);
                }
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
                && self.state.fence(id).is_some_and(|f| !f.holds_items())
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
    /// `by`: where the change came from (`settings`, `cli`, `import`), for the log.
    pub(super) fn apply_settings(&mut self, mut new: pecofence_core::Settings, by: &'static str) {
        let old = self.state.config.settings.clone();
        if new == old {
            return;
        }
        let what = changed_settings(&old, &new);
        tracing::info!(what = what.as_str(), by, "settings changed");
        if new.autostart != old.autostart {
            if let Err(e) =
                pecofence_platform::autostart::set_product_enabled(new.autostart, self.portable)
            {
                tracing::warn!(error = %e, "autostart change failed");
                new.autostart = old.autostart;
                self.settings_error(pecofence_core::i18n::text(
                    "未能更改“随 Windows 启动”，请稍后重试。",
                ));
            } else if pecofence_platform::process::is_packaged() {
                // Store installs: Windows keeps the startup task; a worker switches it where the
                // app may. The switch shows the change at once and Windows' state once the
                // worker is back.
                self.switch_store_startup(new.autostart);
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
        if new.keep_icons_hidden != old.keep_icons_hidden
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.keep_icons_hidden = new.keep_icons_hidden;
        }
        if new.quick_hide.enabled != old.quick_hide.enabled
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.quick_hide_enabled = new.quick_hide.enabled;
            // Switched off while the fences are hidden: a double-click can no longer show them.
            if !a.quick_hide_enabled && a.fences_hidden() {
                a.toggle_fences_hidden_by("quick hide turned off");
            }
        }
        if new.show_desktop != old.show_desktop
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.behavior = match new.show_desktop {
                ShowDesktopSetting::KeepVisible => ShowDesktopBehavior::KeepVisible,
                ShowDesktopSetting::HideWithDesktop => ShowDesktopBehavior::HideWithDesktop,
            };
        }
        if new.zorder != old.zorder
            && let Some(a) = self.anchor.borrow_mut().as_mut()
        {
            a.mode = match new.zorder {
                ZOrderSetting::InsertAboveHost => ZMode::InsertAboveHost,
                ZOrderSetting::HwndBottom => ZMode::HwndBottom,
            };
            a.reanchor("zorder changed");
        }
        self.ctx.behavior.hover_peek.set(new.roll_up.hover_peek);
        self.ctx.behavior.snapping.set(new.snapping.enabled);
        self.ctx
            .behavior
            .snap_gap_dip
            .set(snap_gap_dip(&new.snapping));
        self.ctx
            .behavior
            .size_to_cells
            .set(new.snapping.size_to_cells);
        self.ctx.behavior.guide_lines.set(new.snapping.guide_lines);
        self.ctx
            .behavior
            .click_to_expand
            .set(new.roll_up.click_to_expand);
        self.ctx.behavior.hover_speed.set(new.roll_up.hover_speed);
        self.ctx
            .behavior
            .title_on_hover
            .set(new.roll_up.title_on_hover);
        self.ctx.behavior.title_align.set(new.title_align);
        self.ctx.behavior.tab_bar.set(new.tab_bar);
        self.ctx.behavior.label_color.set(new.label_color);
        self.ctx
            .behavior
            .hide_inactive_scrollbar
            .set(new.roll_up.hide_inactive_scrollbar);
        self.ctx
            .behavior
            .switch_tab_on_hover
            .set(new.roll_up.switch_tab_on_hover);
        self.ctx
            .behavior
            .hide_count_and_chevron
            .set(new.roll_up.hide_count_and_chevron);
        // Only whole rows show with 「调整大小时按整行整列对齐」 on: redraw.
        let chrome_changed = new.roll_up.title_on_hover != old.roll_up.title_on_hover
            || new.title_align != old.title_align
            || new.label_color != old.label_color
            || new.roll_up.hide_inactive_scrollbar != old.roll_up.hide_inactive_scrollbar
            || new.roll_up.hide_count_and_chevron != old.roll_up.hide_count_and_chevron
            || new.snapping.size_to_cells != old.snapping.size_to_cells;
        if chrome_changed {
            for w in self.fences.values() {
                w.redraw();
            }
        }
        if new.tab_bar != old.tab_bar {
            // The tab column takes (or gives back) width beside the items, and an auto-height
            // fence grows to show every tab in it.
            let hosts: Vec<FenceId> = self.fences.keys().copied().collect();
            for host in hosts {
                if let Some(w) = self.fences.get(&host) {
                    w.sync_tab_bar();
                }
                self.apply_auto_height(host);
            }
        }
        self.ctx
            .behavior
            .backdrop
            .set(backdrop_mode_for(new.backdrop));
        let visual_changed = new.theme != old.theme
            || new.theme_style != old.theme_style
            || new.custom_theme != old.custom_theme
            || new.backdrop != old.backdrop;
        let icons_changed = new.icons != old.icons;
        let push_changed = new.roll_up.push_neighbors != old.roll_up.push_neighbors;
        let inbox_hiding_changed =
            new.hide_inbox_when_icons_shown != old.hide_inbox_when_icons_shown;
        // A tool switched on brings its fences back; switched off, they go.
        let tools_changed = new.tools != old.tools;
        let per_desktop_changed = new.per_desktop_fences != old.per_desktop_fences;
        if per_desktop_changed {
            tracing::info!(
                on = new.per_desktop_fences,
                desktop = ?self.state.current_desktop,
                "per-desktop fences switched"
            );
        }
        // 「调整大小时按整行整列对齐」 switched on keeps every rectangle: the rows fill any
        // height with whole icons (redrawn above), and rounding would undo fences lined up.
        self.state.config.settings = new;
        self.refresh_language();
        self.state.mark_dirty();
        self.schedule_save();
        self.sync_peek_hotkey();
        self.sync_windows_desktop_hotkey();
        self.sync_dialog_jump();
        if icons_toggled {
            self.desktop_icons_setting_changed();
        }
        if icons_changed {
            self.apply_icon_variant();
        }
        if visual_changed {
            self.refresh_visuals(true);
        }
        if push_changed {
            // Pushed fences go back at once when it is switched off.
            self.reflow_pushed();
        }
        if inbox_hiding_changed || tools_changed {
            self.sync_hidden_fences();
        }
        if tools_changed {
            self.refresh_tool_views();
            self.run_due_scripts();
        }
        if per_desktop_changed {
            // New fences follow the current desktop from now on: it is followed while on.
            self.apply_desktop_visibility("per-desktop fences switched");
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
        self.localize_auto_names();
        if let Some(host) = &self.settings {
            host.set_title(pecofence_core::i18n::text("PecoFence 设置"));
        }
        // Texts a window keeps as text: a script fence's status line, a smart fence's empty
        // state, the filter tag, the tip of the virtual desktop mark.
        let hosts: Vec<FenceId> = self.fences.keys().copied().collect();
        for host in &hosts {
            self.push_tool_view(*host);
            self.apply_portal_deco(*host);
        }
        self.apply_desktop_visibility("interface language changed");
        tracing::debug!(
            windows = hosts.len(),
            "fence windows got their kept texts in the new language"
        );
        for window in self.fences.values() {
            window.redraw();
        }
        self.push_settings_state();
        self.smart_editor_language_changed();
    }
}

impl App {
    /// The first start's fences (程序, 文件夹, 文件与文档, 桌面) and rules keep their names in the
    /// interface language until the user renames them: a name that is PecoFence's own in some
    /// language becomes the same name in the new one. Every saved layout, not only this one.
    fn localize_auto_names(&mut self) {
        const FENCES: &[&str] = &["程序", "文件夹", "文件与文档"];
        const INBOX: &[&str] = &["桌面"];
        const RULES: &[&str] = &["程序与快捷方式", "文件夹", "文件与文档"];
        let shown: Vec<FenceId> = self.state.fences().iter().map(|f| f.id).collect();
        let mut renamed = Vec::new();
        for layout in &mut self.state.config.layouts {
            for f in &mut layout.fences {
                let keys = match f.kind {
                    FenceKind::Inbox => INBOX,
                    FenceKind::Virtual if f.tool.is_none() => FENCES,
                    _ => continue,
                };
                if let Some(title) = pecofence_core::i18n::retranslated(&f.title, keys) {
                    f.title = title.to_string();
                    renamed.push(f.id);
                }
            }
        }
        let mut rules = 0;
        for r in &mut self.state.config.rules.list {
            if let Some(name) = pecofence_core::i18n::retranslated(&r.name, RULES) {
                r.name = name.to_string();
                rules += 1;
            }
        }
        tracing::info!(
            fences = renamed.len(),
            rules,
            "first-start names follow the interface language"
        );
        if renamed.is_empty() && rules == 0 {
            return;
        }
        for id in renamed.iter().filter(|id| shown.contains(id)) {
            self.refresh_fence(*id);
        }
        self.state.mark_dirty();
        self.schedule_save();
    }
}

/// How an AI agent in a terminal runs pecofence-cli (the 整理规则 prompt names it): the Store
/// edition puts `pecofence-cli` on PATH through an App Execution Alias; setup and the ZIP
/// leave it next to `pecofence.exe`, so the prompt gets its full path.
fn cli_command() -> String {
    if !pecofence_platform::process::is_packaged()
        && let Some(cli) = std::env::current_exe()
            .ok()
            .and_then(|exe| Some(exe.parent()?.join("pecofence-cli.exe")))
            .filter(|cli| cli.is_file())
    {
        return cli.to_string_lossy().into_owned();
    }
    "pecofence-cli".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_settings_are_named_not_shown() {
        let old = pecofence_core::Settings::default();
        let mut new = old.clone();
        new.hide_real_icons = !old.hide_real_icons;
        new.custom_theme = Some(r"C:\Users\jane\theme.json".to_string());
        new.quick_hide.enabled = !old.quick_hide.enabled;
        assert_eq!(
            changed_settings(&old, &new),
            "customTheme,hideRealIcons,quickHide"
        );
        assert_eq!(changed_settings(&old, &old), "");
    }
}
