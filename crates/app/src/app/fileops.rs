//! Shell file operations off the UI thread. `IFileOperation::PerformOperations` blocks for
//! hundreds of milliseconds on a OneDrive folder; run on the UI thread it froze every fence's
//! animation and the shell drag image at the very moment of a drop. Explorer performs file
//! operations on its own worker threads and lets the folder-change notifications update the
//! views; this does the same: a worker STA thread runs the operation, the result is queued and
//! the control window is poked, and the folder watchers (plus a targeted portal refresh on
//! completion) bring the items in.

use super::*;
use pecofence_platform::com::OleGuard;

/// What to do on the UI thread once the operation has finished.
pub(super) enum FileOpThen {
    /// Files placed into a portal's folder or a folder item (drop / 移动到栅栏): re-read the
    /// destination and the source folders' portals right away.
    IntoFolder {
        dest: PathBuf,
        copy: bool,
        sources: Vec<PathBuf>,
    },
    /// Files moved / copied onto the desktop with `PendingRoute`s for where they should land:
    /// a cancelled or failed operation withdraws those routes.
    ToDesktop {
        routed: Vec<PathBuf>,
        copy: bool,
        what: &'static str,
    },
    /// Paste into the folder the files already live in ("xxx - 副本").
    Duplicate,
    /// A rename the filesystem refused, retried through the shell for its permission prompt.
    /// The folder watcher pairs the rename, so the item keeps its fence.
    Rename,
    /// The shell's own delete; the folder watchers take the items away.
    Delete { count: usize, permanent: bool },
}

pub(super) struct FileOp {
    pub(super) paths: Vec<PathBuf>,
    pub(super) dest: PathBuf,
    pub(super) copy: bool,
    pub(super) rename_on_collision: bool,
    pub(super) owner: Option<HWND>,
    pub(super) then: FileOpThen,
}

/// Finished operations waiting for the UI thread (`Err` carries the error text: COM errors are
/// not `Send`).
pub(super) type FileOpResults = Arc<Mutex<Vec<(FileOpThen, std::result::Result<bool, String>)>>>;

impl App {
    /// Starts `op` on a worker thread; `drain_fileops` handles the outcome.
    pub(super) fn start_fileop(&mut self, op: FileOp) {
        let FileOp {
            paths,
            dest,
            copy,
            rename_on_collision,
            owner,
            then,
        } = op;
        self.run_fileop(owner, then, move |owner| {
            shell::transfer_to_folder(&paths, &dest, owner, copy, rename_on_collision)
        });
    }

    /// Renames `path` to `new_name` through the shell on a worker thread (see
    /// [`FileOpThen::Rename`]).
    pub(super) fn start_shell_rename(&mut self, path: PathBuf, new_name: String, owner: HWND) {
        self.run_fileop(Some(owner), FileOpThen::Rename, move |owner| {
            shell::rename_with_prompt(&path, &new_name, owner)
        });
    }

    /// The shell's `delete` verb on `paths` (Recycle Bin, or permanent with Shift, with
    /// Explorer's own dialogs) on a worker thread. It can return many seconds after the files
    /// are gone (a full Recycle Bin, an antivirus checking each file), and on the UI thread
    /// every fence stopped responding until then (#92).
    pub(super) fn start_shell_delete(&mut self, paths: Vec<PathBuf>, owner: HWND, permanent: bool) {
        let count = paths.len();
        let pt = window::cursor_pos();
        tracing::info!(count, permanent, "shell delete started");
        self.run_fileop(
            Some(owner),
            FileOpThen::Delete { count, permanent },
            move |owner| {
                let refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
                ShellContextMenu::for_paths(&refs).and_then(|mut m| {
                    m.invoke_verb("delete", owner.unwrap_or_default(), pt, permanent)
                })
            },
        );
    }

    fn run_fileop(
        &mut self,
        owner: Option<HWND>,
        then: FileOpThen,
        job: impl FnOnce(Option<HWND>) -> pecofence_platform::Result<bool> + Send + 'static,
    ) {
        let owner = owner.map(|h| h.0 as isize);
        let results = self.fileops_done.clone();
        let control = self.control.hwnd().0 as isize;
        let spawned = std::thread::Builder::new()
            .name("pecofence-fileop".into())
            .spawn(move || {
                // The shell's copy engine wants an STA; its progress / conflict dialogs run on
                // this thread, parented to the fence window.
                let _sta = OleGuard::init();
                let owner = owner.map(|h| HWND(h as *mut core::ffi::c_void));
                let result = job(owner).map_err(|e| e.to_string());
                if let Ok(mut r) = results.lock() {
                    r.push((then, result));
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_COMMAND,
                    0,
                    0,
                );
            });
        match spawned {
            Ok(_) => self.fileops_running += 1,
            Err(e) => tracing::warn!(error = %e, "file operation thread failed to start"),
        }
    }

    /// Applies the outcome of every finished file operation (called from the command pump).
    pub(super) fn drain_fileops(&mut self) {
        let done: Vec<(FileOpThen, std::result::Result<bool, String>)> = self
            .fileops_done
            .lock()
            .map(|mut d| d.drain(..).collect())
            .unwrap_or_default();
        for (then, result) in done {
            self.fileops_running = self.fileops_running.saturating_sub(1);
            self.on_fileop_done(then, result);
        }
    }

    fn toast(&self, text: String) {
        if let Some(t) = &self.tray {
            t.show_info("PecoFence", &text, true);
        }
    }

    /// A move that may have been a paste of cut files finished (`paste_into`): the clipboard
    /// is emptied once the files moved; after a cancel or failure they stay on it.
    fn settle_cut_paste(&mut self, moved: bool) {
        if std::mem::take(&mut self.cut_paste_pending) && moved {
            self.finish_cut_paste();
        }
    }

    fn on_fileop_done(&mut self, then: FileOpThen, result: std::result::Result<bool, String>) {
        match then {
            FileOpThen::IntoFolder {
                dest,
                copy,
                sources,
            } => {
                self.settle_cut_paste(matches!(result, Ok(true)));
                match &result {
                    Ok(true) => {
                        tracing::info!(count = sources.len(), copy, dest = %dest.display(), "placed into folder")
                    }
                    Ok(false) => tracing::info!("move into folder cancelled by user"),
                    Err(e) => {
                        tracing::warn!(error = %e, "move into folder failed");
                        self.toast(pecofence_core::i18n::format(
                            "移动到文件夹失败：{0}",
                            std::slice::from_ref(e),
                        ));
                    }
                }
                // The watchers fire too; re-reading the folders involved now makes the drop
                // land without waiting for the batch.
                let mut dirs = vec![dest];
                for p in &sources {
                    if let Some(d) = p.parent()
                        && !dirs.iter().any(|x| x == d)
                    {
                        dirs.push(d.to_path_buf());
                    }
                }
                self.refresh_portals_in(&dirs);
            }
            FileOpThen::ToDesktop { routed, copy, what } => {
                self.settle_cut_paste(matches!(result, Ok(true)));
                match result {
                    Ok(true) => {
                        tracing::info!(count = routed.len(), copy, what, "files transferred");
                        // The expiry counts from the end of the copy for files still to appear.
                        for r in self.pending_routes.iter_mut() {
                            if routed.contains(&r.path) {
                                r.since = Instant::now();
                            }
                        }
                    }
                    Ok(false) => {
                        tracing::info!("move to desktop cancelled by user");
                        self.pending_routes.retain(|r| !routed.contains(&r.path));
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "move to desktop failed");
                        self.pending_routes.retain(|r| !routed.contains(&r.path));
                        self.toast(pecofence_core::i18n::format(
                            "移动到桌面失败：{0}",
                            std::slice::from_ref(&e),
                        ));
                    }
                }
            }
            FileOpThen::Duplicate => {
                match &result {
                    Ok(true) => tracing::info!("pasted duplicates"),
                    Ok(false) => tracing::info!("paste cancelled by user"),
                    Err(e) => {
                        tracing::warn!(error = %e, "paste failed");
                        self.toast(pecofence_core::i18n::format(
                            "粘贴失败：{0}",
                            std::slice::from_ref(e),
                        ));
                    }
                }
                self.refresh_portals();
            }
            FileOpThen::Delete { count, permanent } => match &result {
                Ok(true) => tracing::info!(count, permanent, "shell delete finished"),
                Ok(false) => tracing::warn!("shell menu offers no delete verb"),
                Err(e) => tracing::warn!(error = %e, "shell delete failed"),
            },
            FileOpThen::Rename => match &result {
                Ok(true) => tracing::info!("item renamed through the shell"),
                Ok(false) => tracing::info!("rename cancelled by user"),
                Err(e) => {
                    tracing::warn!(error = %e, "rename through the shell failed");
                    self.toast(pecofence_core::i18n::format(
                        "重命名失败：{0}",
                        std::slice::from_ref(e),
                    ));
                }
            },
        }
    }
}
