//! `--test-script <file>`: drives the running app from a plain text script so UI behaviour can
//! be exercised and inspected without a human at the mouse (a second instance with
//! `PECOFENCE_INSTANCE=test --portable --no-hide-icons` is the intended host).
//!
//! One command per line, `#` comments, blank lines ignored:
//!
//! ```text
//! sleep <ms>                 wait
//! dump <tag>                 log one `pecofence::test` line per fence window (+ dying ones)
//! feedback-info              log what the 「反馈」 form would send besides the user's text
//! message <json>             send the same JSON command as the settings WebView
//! quick-hide | quick-show    the desktop double-click toggle
//! peek | end-peek            Peek overlay
//! pin-test-windows           keep debug audit windows above other apps without an overlay
//! roll <title> | unroll <title>
//! activate <title>           switch to a tab through the native command path
//! housekeeping               run the same periodic maintenance as the one-minute timer
//! open-settings | close-settings  the settings window (as the tray opens it / its close box)
//! monitors <id>@<x>,<y>,<w>,<h>@<dpi> …  debug-only: these monitors (work areas in physical
//!                            px) stand in for the connected ones, then a display change
//! bounds <title> <x> <y> <w> <h>  set a test window's physical rectangle
//! size <title> <edge> <x> <y>     drag an edge (left, top-right, bottom …) to x / y through
//!                            WM_SIZING, as the size loop does; `size-end <title>` releases it
//! input <title> <action> <x> <y>  debug-only native mouse/cancel regression input
//!                            (`hover 1 0` / `hover 0 0`: the pointer enters / leaves)
//! pace <ms>                  debug-only script timer interval (default 50 ms)
//! reorder <title> <index>    same command as the tab menu
//! detach <title>             tear the tab out into its own fence (menu path)
//! merge <title> <into-title> merge a fence into another one's tab strip
//! delete <title>
//! menu-delete <title>        the fence menu's 删除栅栏 (the same path, including its prompt)
//! new-fence <x> <y> <w> <h>  open the 「在此新建栅栏」 menu for that rectangle (physical px), as
//!                            a desktop marquee over no fence does; the script waits until
//!                            the menu is closed
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
//! drop-files <title> <0|1> <path>  debug-only: the fence's OLE drop handler for a file from
//!                            another program, no modifier keys; logs the effect DragEnter
//!                            reports and (1) performs the drop; a trailing `<name>` drops
//!                            onto the item whose name contains it instead of the centre;
//!                            `<path>` may be any shell parsing name (`shell:AppsFolder\…`)
//!                            (`allowed=link`, `allowed=copy+link`: the source's effects)
//! drop-marshaled <title> <0|1> <file> [<name>]  debug-only: the same for a data object another
//!                            process marshaled into <file> (.cache/vdrop: an Outlook-like
//!                            virtual-file source)
//! item-menu <title>          log the right-click menu of the fence's first item (not shown)
//! device-lost <n>            debug-only: every fence window reports a lost GPU device and the
//!                            next <n> device rebuilds fail (a driver reset in progress)
//! device-state               log how many fence windows still wait for a working device
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
                ["feedback-info"] => {
                    let info = self.feedback_info_json();
                    tracing::info!(target: "pecofence::test", "feedback-info {info}");
                }
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
                #[cfg(debug_assertions)]
                ["drop-files", title, perform, path, at @ ..] => {
                    if let Some(id) = self.test_fence(title)
                        && let Some(w) = self.fences.get(&self.state.host_of(id))
                    {
                        let path = std::path::PathBuf::from(path);
                        match pecofence_platform::dragdrop::data_object_for_paths(&[path.as_path()])
                        {
                            Ok(data) => {
                                let (at, allowed) = test_drop_options(at);
                                let (entered, dropped) =
                                    w.test_drop(&self.ctx, &data, 0, *perform == "1", at, allowed);
                                tracing::info!(
                                    target: "pecofence::test",
                                    "drop-files {}: enter={entered:?} drop={dropped:?}",
                                    path.display()
                                );
                            }
                            Err(e) => tracing::warn!(
                                target: "pecofence::test",
                                error = %e,
                                "drop-files: no data object for {}",
                                path.display()
                            ),
                        }
                    }
                }
                #[cfg(debug_assertions)]
                ["drop-marshaled", title, perform, file, at @ ..] => {
                    if let Some(id) = self.test_fence(title)
                        && let Some(w) = self.fences.get(&self.state.host_of(id))
                    {
                        match std::fs::read(file)
                            .map_err(|e| e.to_string())
                            .and_then(|b| {
                                pecofence_platform::dragdrop::unmarshal_data_object(&b)
                                    .map_err(|e| e.to_string())
                            }) {
                            Ok(data) => {
                                let (at, allowed) = test_drop_options(at);
                                let (entered, dropped) =
                                    w.test_drop(&self.ctx, &data, 0, *perform == "1", at, allowed);
                                tracing::info!(
                                    target: "pecofence::test",
                                    "drop-marshaled {file}: enter={entered:?} drop={dropped:?}"
                                );
                            }
                            Err(e) => tracing::warn!(
                                target: "pecofence::test",
                                error = %e,
                                "drop-marshaled: no data object in {file}"
                            ),
                        }
                    }
                }
                ["item-menu", title] => {
                    if let Some(id) = self.test_fence(title)
                        && let Some(f) = self.state.fence(id)
                        && let Some(item) = self.state.items_of(f).first().map(|it| it.id)
                    {
                        let name = self.state.item(item).map(|it| it.display_name.clone());
                        let labels = self.item_menu_labels(id, item);
                        tracing::info!(
                            target: "pecofence::test",
                            "item-menu {name:?}: {}",
                            labels.join(" | ")
                        );
                    }
                }
                #[cfg(debug_assertions)]
                ["device-lost", n] => {
                    super::motion::TEST_FAILING_RECOVERIES
                        .store(n.parse().unwrap_or(0), std::sync::atomic::Ordering::Relaxed);
                    for w in self.fences.values() {
                        w.test_lose_device();
                    }
                }
                ["device-state"] => {
                    let lost = self.fences.values().filter(|w| w.device_lost()).count();
                    tracing::info!(
                        target: "pecofence::test",
                        "device-state lost={lost}/{}",
                        self.fences.len()
                    );
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
                #[cfg(debug_assertions)]
                ["monitors", specs @ ..] => {
                    let areas: Vec<_> = specs.iter().filter_map(|s| test_monitor(s)).collect();
                    super::fences::TEST_MONITORS.with_borrow_mut(|m| *m = Some(areas));
                    let display_change = pecofence_platform::msg::WM_DISPLAYCHANGE;
                    window::post_message(self.control.hwnd(), display_change, 0, 0);
                }
                ["housekeeping"] => self.housekeeping(),
                ["open-settings"] => self.open_settings(),
                ["close-settings"] => {
                    // The close box's path: WM_DESTROY closes the controller, WindowGone
                    // drops the host.
                    if let Some(host) = &self.settings {
                        window::post_message(host.hwnd(), pecofence_platform::msg::WM_CLOSE, 0, 0);
                    }
                }
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
                ["size", title, edge, x, y] => {
                    const EDGES: [&str; 8] = [
                        "left",
                        "right",
                        "top",
                        "top-left",
                        "top-right",
                        "bottom",
                        "bottom-left",
                        "bottom-right",
                    ];
                    if let (Some(wmsz), Some(id), Ok(x), Ok(y)) = (
                        EDGES.iter().position(|e| e == edge),
                        self.test_fence(title),
                        x.parse::<i32>(),
                        y.parse::<i32>(),
                    ) && let Some(window) = self.fences.get(&self.state.host_of(id))
                    {
                        let r = window.test_size(wmsz + 1, x, y);
                        tracing::info!(
                            target: "pecofence::test",
                            "sized {title:?} {edge} to ({x},{y}): rect=({},{},{},{})",
                            r.left,
                            r.top,
                            r.right,
                            r.bottom
                        );
                    }
                }
                ["size-end", title] => {
                    if let Some(id) = self.test_fence(title)
                        && let Some(window) = self.fences.get(&self.state.host_of(id))
                    {
                        window.test_size_end();
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
                ["end-peek"] => self.queue.push(Command::EndPeek("test script")),
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
                                at: (i32::MIN, i32::MIN),
                            });
                        }
                    }
                }
                ["delete", title] => {
                    if let Some(id) = self.test_fence(title) {
                        self.queue.push(Command::DeleteFence(id));
                    }
                }
                ["menu-delete", title] => {
                    if let Some(id) = self.test_fence(title) {
                        self.delete_fence_from_menu(id);
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
                // What the anchor reports for a drag over the bare desktop from (x, y) to
                // (x + w, y + h); `add` = Ctrl / Shift held. `marquee` presses, drags and
                // releases; `marquee-drag` stops before the release, `marquee-end` releases.
                // Releasing over no fence opens the new-fence menu, so scripts draw over fences.
                [
                    verb @ ("marquee" | "marquee-drag" | "marquee-end"),
                    x,
                    y,
                    w,
                    h,
                    rest @ ..,
                ] if rest.is_empty() || rest == ["add"] => {
                    use crate::anchor::MarqueeEvent;
                    let p = |s: &str| s.parse::<i32>().unwrap_or(0);
                    let (x, y, w, h) = (p(x), p(y), p(w), p(h));
                    let rect = RECT {
                        left: x.min(x + w),
                        top: y.min(y + h),
                        right: x.max(x + w),
                        bottom: y.max(y + h),
                    };
                    if *verb != "marquee-end" {
                        self.on_desktop_marquee(MarqueeEvent::Pressed {
                            additive: !rest.is_empty(),
                        });
                        self.on_desktop_marquee(MarqueeEvent::Moved(rect));
                    }
                    if *verb != "marquee-drag" {
                        self.on_desktop_marquee(MarqueeEvent::Released(rect));
                    }
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
        let rect = |r: RECT| (r.left, r.top, r.right, r.bottom);
        let outline = self
            .ctx
            .behavior
            .drop_preview
            .try_borrow()
            .ok()
            .and_then(|p| p.as_ref().and_then(|p| p.shown()).map(rect));
        let guides: Vec<_> = self
            .ctx
            .behavior
            .drag_guides
            .try_borrow()
            .ok()
            .and_then(|g| {
                g.as_ref()
                    .map(|g| g.shown().into_iter().map(rect).collect())
            })
            .unwrap_or_default();
        tracing::info!(target: "pecofence::test", "[{tag}] outline={outline:?} guides={guides:?}");
        let selected: Vec<String> = self
            .ctx
            .behavior
            .selection
            .borrow()
            .iter()
            .filter_map(|(_, id)| self.state.fence(*id).map(|f| f.title.clone()))
            .collect();
        let ringed: Vec<String> = self
            .fences
            .iter()
            .filter(|(_, w)| w.group_selected())
            .filter_map(|(id, _)| self.state.fence(*id).map(|f| f.title.clone()))
            .collect();
        let band = self.marquee_band_shown().map(rect);
        tracing::info!(target: "pecofence::test", "[{tag}] selection={selected:?} ringed={ringed:?} band={band:?}");
        let peek = self.peek.is_some();
        let rename = crate::rename::active_target().is_some();
        tracing::info!(target: "pecofence::test", "[{tag}] fences={} dying={} peek={peek} rename={rename} hide_setting={} icons_hidden={}",
            self.fences.len(), self.dying.len(), self.state.config.settings.hide_real_icons,
            pecofence_platform::shell_icons::desktop_icons_hidden());
    }
}

/// `monitors` spec `<id>@<x>,<y>,<w>,<h>@<dpi>`: a monitor whose work area is that rectangle.
#[cfg(debug_assertions)]
fn test_monitor(spec: &str) -> Option<pecofence_core::geometry::WorkArea> {
    let mut parts = spec.split('@');
    let (id, rect, dpi) = (parts.next()?, parts.next()?, parts.next()?.parse().ok()?);
    let n: Vec<i32> = rect.split(',').filter_map(|v| v.parse().ok()).collect();
    let [x, y, w, h] = n[..] else {
        return None;
    };
    Some(pecofence_core::geometry::WorkArea {
        device_path: id.to_string(),
        gdi_name: String::new(),
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
        dpi,
        mon_left: x,
        mon_top: y,
        mon_right: x + w,
        mon_bottom: y + h,
    })
}

/// The trailing options of `drop-files` / `drop-marshaled`: an item name to drop onto, and
/// `allowed=link` / `allowed=copy+link` … for a source that offers only those effects (the
/// Start menu offers Link alone).
#[cfg(debug_assertions)]
fn test_drop_options<'a>(rest: &[&'a str]) -> (Option<&'a str>, Option<u32>) {
    use pecofence_platform::dragdrop::DropEffect;
    let mut at = None;
    let mut allowed = None;
    for t in rest {
        if let Some(list) = t.strip_prefix("allowed=") {
            allowed = Some(list.split('+').fold(0, |mask, e| {
                mask | match e {
                    "copy" => DropEffect::Copy.to_raw(),
                    "move" => DropEffect::Move.to_raw(),
                    "link" => DropEffect::Link.to_raw(),
                    _ => 0,
                }
            }));
        } else {
            at = Some(*t);
        }
    }
    (at, allowed)
}
