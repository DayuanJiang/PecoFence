//! `--test-script <file>`: drives the running app from a plain text script so UI behaviour can
//! be exercised and inspected without a human at the mouse (a second instance with
//! `PECOFENCE_INSTANCE=test --portable --no-hide-icons` is the intended host).
//!
//! One command per line, `#` comments, blank lines ignored:
//!
//! ```text
//! sleep <ms>                 wait
//! dump <tag>                 log one `pecofence::test` line per fence window (+ dying ones)
//! message <json>             send the same JSON command as the settings WebView
//! quick-hide | quick-show    the desktop double-click toggle
//! peek | end-peek            Peek overlay
//! pin-test-windows           keep debug audit windows above other apps without an overlay
//! roll <title> | unroll <title>
//! activate <title>           switch to a tab through the native command path
//! housekeeping               run the same periodic maintenance as the one-minute timer
//! bounds <title> <x> <y> <w> <h>  set a test window's physical rectangle
//! input <title> <action> <x> <y>  debug-only native mouse/cancel regression input
//! pace <ms>                  debug-only script timer interval (default 50 ms)
//! reorder <title> <index>    same command as the tab menu
//! detach <title>             tear the tab out into its own fence (menu path)
//! merge <title> <into-title> merge a fence into another one's tab strip
//! delete <title>
//! new-fence <x> <y> <w> <h>  physical px
//! drop-desktop <title>       first item of the fence dropped on the bare desktop (inbox + rules)
//! move <title> <into-title>  first item of the fence moved into another fence (drop minus OLE)
//! transfer <path> <title>    shell-move a file into the portal fence's folder (worker thread)
//! create-file <path>         write a small file (a watched folder gains an item, like a shell move)
//! delete-file <path>         remove it again
//! new-folder <title>         the fence's New Folder command (uses its portal or desktop)
//! new-text <title>           the fence's New Text Document command
//! cancel-rename              cancel the active inline editor
//! rename-active <name>       submit a supplied name for the item being edited
//! edit-item <title> <name>   open rename for an item whose display name contains <name>
//! crash                      force an access violation (tests the crash logger)
//! exit                       quit the process
//! exit-if-file <path>        quit when a test harness creates a stop marker
//! ```
//!
//! `<title>` matches the first fence whose title contains it. Commands run on the control
//! window's timer between message-loop iterations, exactly like user-driven commands.

use super::App;
use crate::commands::Command;
use pecofence_core::FenceId;
use pecofence_platform::{RECT, desktop, window};
use std::time::{Duration, Instant};

pub(super) struct TestScript {
    lines: Vec<String>,
    pos: usize,
    wait_until: Option<Instant>,
}

impl TestScript {
    pub(super) fn load(path: &str) -> Option<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(path, error = %e, "test script unreadable");
                return None;
            }
        };
        Some(Self {
            lines: text.lines().map(|l| l.trim().to_string()).collect(),
            pos: 0,
            wait_until: None,
        })
    }
}

impl App {
    /// Runs script lines until one asks to wait; called from `TIMER_TEST`.
    pub(super) fn test_step(&mut self) {
        loop {
            let Some(t) = self.test.as_mut() else {
                return;
            };
            if let Some(until) = t.wait_until {
                if Instant::now() < until {
                    return;
                }
                t.wait_until = None;
            }
            let Some(line) = t.lines.get(t.pos).cloned() else {
                tracing::info!(target: "pecofence::test", "script finished");
                self.test = None;
                return;
            };
            t.pos += 1;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            tracing::info!(target: "pecofence::test", ">> {line}");
            let words: Vec<&str> = line.split_whitespace().collect();
            match words.as_slice() {
                ["sleep", ms] => {
                    let ms: u64 = ms.parse().unwrap_or(0);
                    if let Some(t) = self.test.as_mut() {
                        t.wait_until = Some(Instant::now() + Duration::from_millis(ms));
                    }
                }
                ["dump", tag] => self.test_dump(tag),
                #[cfg(debug_assertions)]
                ["pace", ms] => {
                    if pecofence_core::brand::var_os("PECOFENCE_UI_TEST_WINDOWS").is_some()
                        && let Ok(ms) = ms.parse::<u32>()
                    {
                        window::set_timer(
                            self.control.hwnd(),
                            super::TIMER_TEST,
                            ms.clamp(1, 1000),
                        );
                    }
                }
                #[cfg(debug_assertions)]
                ["input", title, action, x, y] => {
                    if let (Some(id), Ok(x), Ok(y)) = (self.test_fence(title), x.parse(), y.parse())
                        && let Some(w) = self.fences.get(&self.state.host_of(id))
                    {
                        w.test_input(action, x, y);
                    }
                }
                ["reorder", title, index] => {
                    if let (Some(tab), Ok(to)) = (self.test_fence(title), index.parse()) {
                        self.queue.push(Command::ReorderTab {
                            host: self.state.host_of(tab),
                            tab,
                            to,
                        });
                    }
                }
                ["housekeeping"] => self.housekeeping(),
                ["bounds", title, x, y, w, h] => {
                    if let (Some(id), Ok(x), Ok(y), Ok(width), Ok(height)) = (
                        self.test_fence(title),
                        x.parse::<i32>(),
                        y.parse::<i32>(),
                        w.parse::<i32>(),
                        h.parse::<i32>(),
                    ) && width > 0
                        && height > 0
                        && let Some(window) = self.fences.get(&self.state.host_of(id))
                    {
                        window.set_bounds(RECT {
                            left: x,
                            top: y,
                            right: x + width,
                            bottom: y + height,
                        });
                    }
                }
                ["message", ..] => {
                    if let Some(json) = line.strip_prefix("message ") {
                        self.queue.push(Command::SettingsMessage(json.to_string()));
                    }
                }
                ["quick-hide"] | ["quick-show"] => {
                    let want_hidden = words[0] == "quick-hide";
                    let hidden = self
                        .anchor
                        .borrow()
                        .as_ref()
                        .map(|a| a.fences_hidden())
                        .unwrap_or(false);
                    if hidden != want_hidden {
                        self.toggle_all_fences();
                    }
                }
                ["peek"] => self.queue.push(Command::TogglePeek),
                ["pin-test-windows"] | ["pin-test-windows", "raw"] => {
                    if cfg!(debug_assertions)
                        && pecofence_core::brand::var_os("PECOFENCE_UI_TEST_WINDOWS").is_some()
                        && let Some(a) = self.anchor.borrow_mut().as_mut()
                    {
                        a.set_peek(true, &[]);
                        self.ctx.behavior.floating.set(words.len() == 1);
                        for window in self.fences.values() {
                            window.redraw();
                        }
                    }
                }
                ["end-peek"] => self.queue.push(Command::EndPeek),
                ["roll", title] | ["unroll", title] => {
                    let want = words[0] == "roll";
                    if let Some(id) = self.test_fence(title) {
                        let host = self.state.host_of(id);
                        if let Some(w) = self.fences.get(&host)
                            && w.is_rolled() != want
                        {
                            self.queue.push(Command::ToggleRollUp(host));
                        }
                    }
                }
                ["detach", title] => {
                    if let Some(id) = self.test_fence(title) {
                        let (x, y) = self.test_free_point();
                        self.detach_tab(id, x, y, false);
                    }
                }
                ["activate", title] => {
                    if let Some(tab) = self.test_fence(title) {
                        self.queue.push(Command::SwitchTab {
                            host: self.state.host_of(tab),
                            tab,
                        });
                    }
                }
                ["merge", title, into] => {
                    if let (Some(a), Some(b)) = (self.test_fence(title), self.test_fence(into)) {
                        let host = self.state.host_of(b);
                        if let Some(w) = self.fences.get(&host) {
                            self.queue.push(Command::MergeFence {
                                fence: a,
                                into: w.hwnd(),
                                x: i32::MIN,
                            });
                        }
                    }
                }
                ["delete", title] => {
                    if let Some(id) = self.test_fence(title) {
                        self.queue.push(Command::DeleteFence(id));
                    }
                }
                ["new-fence", x, y, w, h] => {
                    let p = |s: &str| s.parse::<i32>().unwrap_or(0);
                    let (x, y, w, h) = (p(x), p(y), p(w), p(h));
                    self.queue.push(Command::NewFenceRect(RECT {
                        left: x,
                        top: y,
                        right: x + w,
                        bottom: y + h,
                    }));
                }
                ["drop-desktop", title] => {
                    // What a drag-out onto the bare desktop ends in (minus the OLE round
                    // trip): the fence's first item goes back to the inbox and through the rules.
                    if let Some(id) = self.test_fence(title)
                        && let Some(item) = self
                            .state
                            .fence(id)
                            .and_then(|f| f.items.first().map(|r| r.item_id))
                    {
                        self.queue
                            .push(Command::MoveItemsToInbox { items: vec![item] });
                    }
                }
                ["move", title, into] => {
                    // A drop from one fence onto another, minus the OLE round trip: the first
                    // item of `title` moves into `into` (layout glide on both sides).
                    if let (Some(from), Some(to)) = (self.test_fence(title), self.test_fence(into))
                        && let Some(item) = self
                            .state
                            .fence(from)
                            .and_then(|f| f.items.first().map(|r| r.item_id))
                    {
                        self.queue.push(Command::MoveItems {
                            items: vec![item],
                            to,
                        });
                    }
                }
                ["transfer", path, into] => {
                    // A file dropped onto a portal fence: the shell moves it into the portal's
                    // folder on a worker thread (`fileops.rs`); the watcher / completion refresh
                    // bring it in.
                    if let Some(to) = self.test_fence(into) {
                        if let Some(dir) = self.state.portal_path(to) {
                            self.transfer_files_into_folder(
                                vec![std::path::PathBuf::from(path)],
                                dir,
                                to,
                                crate::commands::TransferMode::Move,
                            );
                        } else {
                            tracing::warn!(target: "pecofence::test", "transfer: {into:?} is not a portal");
                        }
                    }
                }
                ["icon-probe", path, px] | ["icon-probe", path, px, _] => {
                    // Synchronous extraction on the UI thread, result in the log: what does the
                    // shell hand back for this file at this size? An optional 4th word names a
                    // file the raw premultiplied BGRA (icon_only variant) is dumped into.
                    let px: u32 = px.parse().unwrap_or(96);
                    let dump = words.get(3).map(|s| s.to_string());
                    let p = std::path::Path::new(path);
                    for icon_only in [true, false] {
                        match pecofence_platform::shell::shell_image(p, px, icon_only) {
                            Ok(img) => {
                                let opaque = img
                                    .bgra
                                    .as_chunks::<4>()
                                    .0
                                    .iter()
                                    .filter(|c| c[3] > 0)
                                    .count();
                                tracing::info!(target: "pecofence::test", path, px, icon_only, w = img.width, h = img.height, opaque_px = opaque, "icon-probe ok");
                                if icon_only && let Some(d) = &dump {
                                    let _ = std::fs::write(d, &img.bgra);
                                }
                            }
                            Err(e) => {
                                tracing::info!(target: "pecofence::test", path, px, icon_only, error = %e, "icon-probe FAILED")
                            }
                        }
                    }
                }
                ["create-file", path] => {
                    // What the shell does at the end of a drag-out from a portal: a new file
                    // appears in a watched folder. Timestamp here vs. "portal refreshed" /
                    // "desktop resynced" = watcher-to-screen latency.
                    match std::fs::write(path, b"pecofence test\n") {
                        Ok(()) => tracing::info!(target: "pecofence::test", path, "file created"),
                        Err(e) => {
                            tracing::warn!(target: "pecofence::test", path, error = %e, "create-file failed")
                        }
                    }
                }
                ["delete-file", path] => match std::fs::remove_file(path) {
                    Ok(()) => tracing::info!(target: "pecofence::test", path, "file deleted"),
                    Err(e) => {
                        tracing::warn!(target: "pecofence::test", path, error = %e, "delete-file failed")
                    }
                },
                ["new-folder", title] | ["new-text", title] => {
                    if let Some(id) = self.test_fence(title) {
                        self.create_desktop_item(id, words[0] == "new-folder");
                    }
                }
                ["cancel-rename"] => {
                    self.queue.push(Command::EndItemRename { commit: false });
                }
                ["rename-active", name] => {
                    if let Some(crate::rename::RenameTarget::Item(item)) =
                        crate::rename::active_target()
                    {
                        self.queue.push(Command::EndItemRename { commit: false });
                        self.queue.push(Command::RenameItemCommit {
                            item,
                            name: (*name).to_string(),
                        });
                    }
                }
                ["edit-item", title, name] => {
                    if let Some(fence) = self.test_fence(title)
                        && let Some(item) = self.state.fence(fence).and_then(|f| {
                            self.state
                                .items_of(f)
                                .into_iter()
                                .find(|it| it.display_name.contains(name))
                                .map(|it| it.id)
                        })
                    {
                        self.queue.push(Command::RenameItem { fence, item });
                    }
                }
                ["crash"] => {
                    // Exercises platform::crashlog: an access violation from the UI thread must
                    // leave a CRASH line + minidump behind.
                    tracing::info!(target: "pecofence::test", "forcing an access violation");
                    // SAFETY: deliberately not — this is the crash under test.
                    unsafe {
                        std::ptr::null_mut::<u32>().write_volatile(1);
                    }
                }
                ["exit-if-file", path] => {
                    if std::path::Path::new(path).is_file() {
                        self.test = None;
                        window::quit_after(50);
                        return;
                    }
                }
                ["exit"] => {
                    self.test = None;
                    window::quit_after(50);
                    return;
                }
                _ => tracing::warn!(target: "pecofence::test", "unknown script line: {line}"),
            }
        }
    }

    fn test_fence(&self, title: &str) -> Option<FenceId> {
        let id = self
            .state
            .fences()
            .iter()
            .find(|f| f.title.contains(title))
            .map(|f| f.id);
        if id.is_none() {
            tracing::warn!(target: "pecofence::test", "no fence titled like {title:?}");
        }
        id
    }

    /// A point in the first work area's lower-right quarter, away from the default fences.
    fn test_free_point(&self) -> (i32, i32) {
        match self.state.work_areas.first() {
            Some(w) => (
                w.left + (w.right - w.left) * 3 / 4,
                w.top + (w.bottom - w.top) * 3 / 4,
            ),
            None => (900, 700),
        }
    }

    fn test_dump(&self, tag: &str) {
        for (kind, w) in self
            .fences
            .values()
            .map(|w| ("live", w))
            .chain(self.dying.iter().map(|w| ("dying", w)))
        {
            let hwnd = w.hwnd();
            let r = w.rect();
            tracing::info!(
                target: "pecofence::test",
                "[{tag}] {kind} hwnd={:#x} visible={} rect=({},{},{},{}) {}",
                hwnd.0 as isize,
                desktop::is_visible(hwnd),
                r.left,
                r.top,
                r.right,
                r.bottom,
                w.debug_state()
            );
        }
        let peek = self.peek.is_some();
        let rename = crate::rename::active_target().is_some();
        tracing::info!(target: "pecofence::test", "[{tag}] fences={} dying={} peek={peek} rename={rename} hide_setting={} icons_hidden={}",
            self.fences.len(), self.dying.len(), self.state.config.settings.hide_real_icons,
            pecofence_platform::shell_icons::desktop_icons_hidden());
    }
}
