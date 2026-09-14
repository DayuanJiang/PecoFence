//! Drop handling that moves real files: transfers into portals / folder items, .lnk and .url creation.

use super::*;

/// File name (without `.url`) for a dropped link: the browser's suggested title, else the URL's
/// host and last path segment, else a generic name. Illegal characters become `-`.
pub fn url_shortcut_base_name(name: Option<&str>, url: &str) -> String {
    let sanitize = |s: &str| -> String {
        let s: String = s
            .chars()
            .map(|c| match c {
                '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
                c if c.is_control() => ' ',
                c => c,
            })
            .collect();
        s.trim().trim_end_matches('.').chars().take(120).collect()
    };
    if let Some(n) = name {
        let n = n.trim();
        let n = n
            .strip_suffix(".url")
            .or_else(|| n.strip_suffix(".URL"))
            .unwrap_or(n);
        let n = sanitize(n);
        if !n.is_empty() {
            return n;
        }
    }
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let rest = rest.split(['?', '#']).next().unwrap_or("");
    let mut parts = rest.split('/').filter(|p| !p.is_empty());
    let host = parts.next().unwrap_or("");
    let last = parts.next_back();
    let n = match (host.is_empty(), last) {
        (false, Some(seg)) => sanitize(&format!("{host} - {seg}")),
        (false, None) => sanitize(host),
        _ => String::new(),
    };
    if n.is_empty() {
        pecofence_core::i18n::text("新建 Internet 快捷方式").to_string()
    } else {
        n
    }
}

impl App {
    /// Moves real files into a portal's folder (drop from Explorer, drag from another fence,
    /// menu "移动到栅栏"). Uses the shell so collisions/undo behave like Explorer.
    pub(super) fn move_files_into_portal(&mut self, paths: Vec<PathBuf>, to: FenceId) {
        self.transfer_files_into_portal(paths, to, TransferMode::Move);
    }

    pub(super) fn transfer_files_into_portal(
        &mut self,
        paths: Vec<PathBuf>,
        to: FenceId,
        mode: TransferMode,
    ) {
        let Some(dest) = self.state.portal_path(to) else {
            return;
        };
        self.transfer_files_into_folder(paths, dest, to, mode);
    }

    /// Creates Explorer-style `.lnk` shortcuts to `paths` in `dir` (the Link drop effect: Alt or
    /// Ctrl+Shift, or 创建快捷方式 from the right-drag menu). Returns the shortcuts written.
    pub(super) fn create_shortcuts_in(&self, paths: &[PathBuf], dir: &Path) -> Vec<PathBuf> {
        let mut made = Vec::new();
        // Failures are logged per path but reported once: a multi-selection dropped on a
        // read-only folder must not raise one balloon per file.
        let mut failed = 0usize;
        let mut first_error = None;
        for p in paths {
            match shell::create_shortcut(p, dir) {
                Ok(lnk) => made.push(lnk),
                Err(e) => {
                    tracing::warn!(error = %e, path = %p.display(), "creating shortcut failed");
                    failed += 1;
                    first_error.get_or_insert(e);
                }
            }
        }
        if let (Some(t), Some(e)) = (&self.tray, first_error) {
            let text = if failed == 1 {
                pecofence_core::i18n::format("无法创建快捷方式：{0}", &[e.to_string()])
            } else {
                pecofence_core::i18n::format(
                    "无法创建 {0} 个快捷方式：{1}",
                    &[failed.to_string(), e.to_string()],
                )
            };
            t.show_info("PecoFence", &text, true);
        }
        made
    }

    /// Moves (Ctrl: copies; Alt / Ctrl+Shift: creates shortcuts to) real files into `dest` with
    /// the shell's file operation — a portal's folder, or a folder item that was the drop target
    /// inside `owner_fence`. Files already in `dest`, the folder itself and its ancestors are
    /// skipped.
    pub(super) fn transfer_files_into_folder(
        &mut self,
        paths: Vec<PathBuf>,
        dest: PathBuf,
        owner_fence: FenceId,
        mode: TransferMode,
    ) {
        let paths = crate::fence_window::filter_folder_paths(paths, &dest);
        if paths.is_empty() {
            return;
        }
        if shell::is_recycle_bin_path(&dest) {
            // Only a move recycles; the fence window already clamps the effect to Move.
            if mode == TransferMode::Move {
                self.recycle_paths(paths, owner_fence);
            }
            return;
        }
        if mode == TransferMode::Link {
            let made = self.create_shortcuts_in(&paths, &dest);
            tracing::info!(count = made.len(), dest = %dest.display(), "shortcuts placed into folder");
            self.refresh_portals();
            return;
        }
        let copy = mode == TransferMode::Copy;
        let owner = self.window_for(owner_fence).map(|w| w.hwnd());
        // Off the UI thread: the operation takes hundreds of ms on a OneDrive folder and the
        // drop's layout motion is running right now. The outcome lands in `on_fileop_done`.
        self.start_fileop(FileOp {
            paths: paths.clone(),
            dest: dest.clone(),
            copy,
            rename_on_collision: false,
            owner,
            then: FileOpThen::IntoFolder {
                dest,
                copy,
                sources: paths,
            },
        });
    }

    /// Files dropped on the Recycle Bin item: the shell's `delete` verb, as when dropping on
    /// Explorer's own bin (its confirmation dialog; Shift held = permanent).
    fn recycle_paths(&mut self, paths: Vec<PathBuf>, owner_fence: FenceId) {
        let owner = self
            .window_for(owner_fence)
            .map(|w| w.hwnd())
            .unwrap_or(self.control.hwnd());
        let refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
        let permanent = window::key_down(msg::VK_SHIFT);
        match ShellContextMenu::for_paths(&refs)
            .and_then(|mut m| m.invoke_verb("delete", owner, window::cursor_pos(), permanent))
        {
            Ok(true) => {
                tracing::info!(count = paths.len(), permanent, "dropped on the Recycle Bin")
            }
            Ok(false) => tracing::warn!("the shell offers no delete verb for the dropped files"),
            Err(e) => tracing::warn!(error = %e, "recycling dropped files failed"),
        }
    }

    /// Browser link dropped on a fence: write the Internet Shortcut Explorer would write and file
    /// it into `to` (desktop items route through a PendingRoute; a portal's folder shows it).
    pub(super) fn create_url_shortcut(&mut self, url: String, name: Option<String>, to: FenceId) {
        let portal_dir = self.state.portal_path(to);
        let Some(dir) = portal_dir.clone().or_else(shell::user_desktop) else {
            return;
        };
        let base = url_shortcut_base_name(name.as_deref(), &url);
        let mut path = dir.join(format!("{base}.url"));
        let mut n = 2;
        while path.exists() {
            path = dir.join(format!("{base} ({n}).url"));
            n += 1;
        }
        match std::fs::write(&path, format!("[InternetShortcut]\r\nURL={url}\r\n")) {
            Ok(()) => {
                tracing::info!(path = %path.display(), %to, "internet shortcut created from a link drop");
                if portal_dir.is_some() {
                    self.refresh_portals();
                    return;
                }
                self.pending_routes.push(PendingRoute {
                    fence: to,
                    path,
                    since: Instant::now(),
                    rename: false,
                });
            }
            Err(e) => {
                tracing::warn!(error = %e, "creating internet shortcut failed");
                if let Some(t) = &self.tray {
                    t.show_info(
                        "PecoFence",
                        &pecofence_core::i18n::format(
                            "无法创建 Internet 快捷方式：{0}",
                            &[e.to_string()],
                        ),
                        true,
                    );
                }
            }
        }
    }
}
