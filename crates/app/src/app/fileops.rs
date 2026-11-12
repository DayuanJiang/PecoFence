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
    /// The shell's own delete of `paths`; the folder watchers take the items away.
    /// `recycled`: set by the worker when the Recycle Bin gained items, so the delete can be
    /// undone (a USB stick, a network folder and 「不将文件移到回收站」 delete for good).
    Delete {
        paths: Vec<PathBuf>,
        permanent: bool,
        started: Instant,
        recycled: Arc<std::sync::atomic::AtomicBool>,
    },
    /// 清空回收站: the bin is asked again for its icon.
    EmptyBin,
    /// Ctrl+Z of items made from a fence (`UndoStep::Created`): they went to the Recycle Bin.
    UndoCreated,
}

pub(super) struct FileOp {
    pub(super) paths: Vec<PathBuf>,
    pub(super) dest: PathBuf,
    pub(super) copy: bool,
    pub(super) rename_on_collision: bool,
    pub(super) owner: Option<HWND>,
    pub(super) then: FileOpThen,
}

/// Finished operations waiting for the UI thread, each with its serial (`Err` carries the error
/// text: COM errors are not `Send`).
pub(super) type FileOpResults =
    Arc<Mutex<Vec<(u64, FileOpThen, std::result::Result<bool, String>)>>>;

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
    /// every fence stopped responding until then (#92). Files a delete still running was
    /// given are left out: a second press of Delete before the first one finished started
    /// another delete of the same files (feedback #108).
    pub(super) fn start_shell_delete(
        &mut self,
        mut paths: Vec<PathBuf>,
        owner: HWND,
        permanent: bool,
    ) {
        let asked = paths.len();
        paths.retain(|p| !self.deleting.contains(p));
        let count = paths.len();
        if count < asked {
            tracing::info!(
                asked,
                left = count,
                "shell delete: files already being deleted are left out"
            );
        }
        if count == 0 {
            return;
        }
        self.deleting.extend(paths.iter().cloned());
        let pt = window::cursor_pos();
        tracing::info!(count, permanent, "shell delete started");
        let recycled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let then = FileOpThen::Delete {
            paths: paths.clone(),
            permanent,
            started: Instant::now(),
            recycled: recycled.clone(),
        };
        let job_paths = paths.clone();
        let spawned = self.run_fileop(Some(owner), then, move |owner| {
            let refs: Vec<&Path> = job_paths.iter().map(|p| p.as_path()).collect();
            // The bin of the files' drive, counted before and after: only a delete that added
            // to it is in Windows' undo list.
            let bin = || {
                (!permanent)
                    .then(|| job_paths.first())
                    .flatten()
                    .and_then(|p| shell::recycle_bin_count_for(p))
            };
            let before = bin();
            let done = ShellContextMenu::for_paths(&refs)
                .and_then(|m| m.invoke_verb("delete", owner.unwrap_or_default(), pt, permanent));
            let after = bin();
            let gained = matches!((before, after), (Some(b), Some(a)) if a > b);
            recycled.store(gained, std::sync::atomic::Ordering::Relaxed);
            tracing::debug!(
                ?before,
                ?after,
                gained,
                permanent,
                "shell delete and the Recycle Bin"
            );
            // True when a file is gone: the delete was not declined in Windows' own dialog.
            done.map(|()| job_paths.iter().any(|p| !p.exists()))
        });
        if !spawned {
            self.settle_delete(&paths);
        }
    }

    /// Ctrl+Z of items made from a fence: each goes to the Recycle Bin on a worker thread,
    /// without a record in Windows' undo list and without a dialog. One that is gone already
    /// is left out, and so is a path that holds another item now (`born`: the creation time
    /// of the item that was made).
    pub(super) fn start_undo_created(
        &mut self,
        items: Vec<(PathBuf, Option<std::time::SystemTime>)>,
    ) {
        self.run_fileop(None, FileOpThen::UndoCreated, move |_| {
            let mut any = false;
            for (path, born) in &items {
                let now = std::fs::metadata(path).and_then(|m| m.created()).ok();
                if born.is_none() || now != *born {
                    tracing::info!(
                        path = %path.display(),
                        ?born,
                        ?now,
                        "new item not recycled: the item that was made is not at this path"
                    );
                    continue;
                }
                match shell::recycle_unrecorded(path) {
                    Ok(done) => any |= done,
                    Err(error) => {
                        tracing::info!(%error, path = %path.display(), "new item not recycled")
                    }
                }
            }
            Ok(any)
        });
    }

    /// 清空回收站 from the bin's menu, with Windows' confirmation and progress, on a worker
    /// thread: a big bin takes a while to empty.
    pub(super) fn start_empty_recycle_bin(&mut self, owner: HWND) {
        tracing::info!("emptying the Recycle Bin");
        self.run_fileop(Some(owner), FileOpThen::EmptyBin, shell::empty_recycle_bin);
    }

    fn run_fileop(
        &mut self,
        owner: Option<HWND>,
        then: FileOpThen,
        job: impl FnOnce(Option<HWND>) -> pecofence_platform::Result<bool> + Send + 'static,
    ) -> bool {
        let owner = owner.map(|h| h.0 as isize);
        let results = self.fileops_done.clone();
        let control = self.control.hwnd().0 as isize;
        let serial = self.fileop_serial + 1;
        let spawned = std::thread::Builder::new()
            .name("pecofence-fileop".into())
            .spawn(move || {
                // The shell's copy engine wants an STA; its progress / conflict dialogs run on
                // this thread, parented to the fence window.
                let _sta = OleGuard::init();
                let owner = owner.map(|h| HWND(h as *mut core::ffi::c_void));
                let result = job(owner).map_err(|e| e.to_string());
                if let Ok(mut r) = results.lock() {
                    r.push((serial, then, result));
                }
                window::post_message(
                    HWND(control as *mut core::ffi::c_void),
                    WM_APP_COMMAND,
                    0,
                    0,
                );
            });
        match spawned {
            Ok(_) => {
                self.fileops_running += 1;
                self.fileop_serial = serial;
                true
            }
            Err(e) => {
                tracing::warn!(error = %e, "file operation thread failed to start");
                false
            }
        }
    }

    /// The delete of `paths` is over (or never started): another Delete may ask for them again.
    fn settle_delete(&mut self, paths: &[PathBuf]) {
        for p in paths {
            self.deleting.remove(p);
        }
    }

    /// Applies the outcome of every finished file operation (called from the command pump).
    pub(super) fn drain_fileops(&mut self) {
        let done: Vec<(u64, FileOpThen, std::result::Result<bool, String>)> = self
            .fileops_done
            .lock()
            .map(|mut d| d.drain(..).collect())
            .unwrap_or_default();
        let any = !done.is_empty();
        for (serial, then, result) in done {
            self.fileops_running = self.fileops_running.saturating_sub(1);
            self.on_fileop_done(serial, then, result);
        }
        // A Ctrl+Z pressed while an operation was running (`undo_last`): its step is noted now.
        if self.undo_waiting && self.fileops_running == 0 {
            self.undo_waiting = false;
            self.undo_last("Ctrl+Z, after the file operation");
        }
        if any {
            // A paste's files may be in their fence now; those the folder watcher still has
            // to report count as just pasted from here, however long the copy took.
            if let Some(p) = self.pasted.as_mut() {
                p.since = Instant::now();
            }
            self.select_pasted();
        }
    }

    pub(super) fn toast(&self, text: String) {
        if let Some(t) = &self.tray {
            t.show_info("PecoFence", &text, true);
        }
    }

    /// Operation `serial` finished; if it was the move of a paste of cut files (`paste_into`),
    /// the clipboard is emptied once the files moved; after a cancel or failure they stay on
    /// it. Something copied or cut meanwhile (the move can take minutes) is left alone.
    fn settle_cut_paste(&mut self, serial: u64, moved: bool) {
        if let Some((op, seq)) = self.cut_paste
            && op == serial
        {
            self.cut_paste = None;
            if moved && clipboard::sequence() == seq {
                self.finish_cut_paste();
            }
        }
    }

    fn on_fileop_done(
        &mut self,
        serial: u64,
        then: FileOpThen,
        result: std::result::Result<bool, String>,
    ) {
        // An operation that went through and that the shell recorded in Windows' undo list is a
        // step of Ctrl+Z (`undo.rs`), whoever started it (the command line's file moves too:
        // their records sit in the same list), so the stack here and that list stay in step.
        let went_through = matches!(result, Ok(true));
        if !went_through {
            tracing::debug!(
                serial,
                "file operation is no step of Ctrl+Z: it did not go through"
            );
        }
        if went_through {
            match &then {
                FileOpThen::IntoFolder { .. } => {
                    self.note_fileop_for_undo("files into a folder", true)
                }
                FileOpThen::ToDesktop { .. } => {
                    self.note_fileop_for_undo("files onto the desktop", true)
                }
                FileOpThen::Duplicate => self.note_fileop_for_undo("pasted copies", true),
                FileOpThen::Rename => self.note_fileop_for_undo("rename through the shell", true),
                // Shift+Delete, and a delete that put nothing into the Recycle Bin (a drive
                // without one, the bin switched off, a delete inside the bin), leave nothing
                // to undo.
                FileOpThen::Delete {
                    permanent,
                    recycled,
                    ..
                } => {
                    let lost = *permanent || !recycled.load(std::sync::atomic::Ordering::Relaxed);
                    if lost {
                        self.note_fileop_for_undo("permanent delete", false);
                    } else {
                        self.note_fileop_for_undo("delete", true);
                    }
                }
                FileOpThen::EmptyBin | FileOpThen::UndoCreated => {}
            }
        }
        match then {
            FileOpThen::IntoFolder {
                dest,
                copy,
                sources,
            } => {
                self.settle_cut_paste(serial, matches!(result, Ok(true)));
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
                self.settle_cut_paste(serial, matches!(result, Ok(true)));
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
                        self.route_places.retain(|p, _| !routed.contains(p));
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "move to desktop failed");
                        self.pending_routes.retain(|r| !routed.contains(&r.path));
                        self.route_places.retain(|p, _| !routed.contains(p));
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
                // The copies made (also by a paste cancelled halfway) go into the fence pasted
                // into: routes for the desktop ones, taken at once so they never show where
                // the rules would put them.
                let routes = self.pending_routes.len();
                self.claim_pasted_duplicates(Some(serial));
                if self.pending_routes.len() > routes {
                    let report = self.sync_desktop_if_available("paste");
                    self.route_pending_creation();
                    if report.changed() {
                        self.refresh_all();
                        self.schedule_save();
                    }
                }
                self.refresh_portals();
            }
            FileOpThen::Delete {
                paths,
                permanent,
                started,
                ..
            } => {
                self.settle_delete(&paths);
                // The fences hear how it went: a delete that removed nothing (declined in
                // Windows' question, failed) must not leave one waiting for its icons to go.
                let gone = matches!(result, Ok(true));
                for w in self.fences.values() {
                    w.settle_user_delete(gone);
                }
                let count = paths.len();
                let ms = crate::share_log::millis(started.elapsed());
                match &result {
                    Ok(_) => tracing::info!(count, permanent, ms, "shell delete finished"),
                    Err(e) => tracing::warn!(error = %e, count, ms, "shell delete failed"),
                }
            }
            FileOpThen::UndoCreated => {
                match &result {
                    Ok(done) => tracing::info!(done, "new items moved to the Recycle Bin"),
                    Err(e) => tracing::warn!(error = %e, "undo of new items failed"),
                }
                shell::recycle_bin_changed();
            }
            FileOpThen::EmptyBin => {
                match &result {
                    Ok(_) => tracing::info!("Recycle Bin emptied"),
                    Err(e) => tracing::warn!(error = %e, "emptying the Recycle Bin failed"),
                }
                shell::recycle_bin_changed();
                window::post_message(self.control.hwnd(), WM_APP_FS_CHANGED, 1, 0);
            }
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
