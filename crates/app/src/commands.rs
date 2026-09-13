//! UI-thread command queue: window handlers push commands and poke the control window; the
//! `App` drains them outside any handler, so no `RefCell` is ever borrowed re-entrantly.

use pecofence_core::{FenceId, ItemId};
use pecofence_platform::window::Window;
use pecofence_platform::{HWND, RECT, msg};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::rc::Rc;

pub const WM_APP_COMMAND: u32 = msg::WM_APP + 1;
pub const WM_APP_FS_CHANGED: u32 = msg::WM_APP + 2;
pub const WM_APP_TRAY: u32 = msg::WM_APP + 3;
/// The wallpaper registry key changed (worker thread → UI thread).
pub const WM_APP_WALLPAPER: u32 = msg::WM_APP + 6;
/// Posted by the compositor frame clock (`FrameClock`) when a fence asked for a frame.
pub const WM_APP_FRAME: u32 = msg::WM_APP + 7;
/// Posted to a fence window (from the compositor's callback thread) when its whole-window
/// fade finished; wparam = the fade generation it belongs to.
pub const WM_APP_FADE_DONE: u32 = msg::WM_APP + 8;
/// Posted to a fence window when a tab-switch cross-fade finished; wparam = generation.
pub const WM_APP_TAB_SWAP_DONE: u32 = msg::WM_APP + 9;
/// Sent to a fence window by the desktop anchor: wparam 1 = show (fade in), 0 = hide (fade
/// out, then `ShowWindow(SW_HIDE)`). The fence manages its shadow itself; returns 1.
pub const WM_APP_SET_VISIBLE: u32 = msg::WM_APP + 10;
/// Posted to the control window after Peek's background focus handoff completes. `wparam` is
/// the dimmer HWND, so a stale completion from an earlier Peek can be ignored.
pub const WM_APP_PEEK_FOCUSED: u32 = msg::WM_APP + 11;

/// What a file drop does with the dropped paths (Explorer's Move / Copy / Create shortcut, the
/// effect the drop target reported).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferMode {
    Move,
    Copy,
    /// Create `.lnk` shortcuts to the paths at the destination; the originals stay.
    Link,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Command {
    /// A fence window finished a move/resize; persist its geometry.
    FenceBoundsChanged {
        fence: FenceId,
        rect: RECT,
    },
    ToggleRollUp(FenceId),
    /// A hover-peeked fence was double-clicked: keep it expanded.
    CommitExpanded(FenceId),
    LaunchItem(ItemId),
    /// Move `items` into `to` (internal drag or menu).
    MoveItems {
        items: Vec<ItemId>,
        to: FenceId,
    },
    /// Internal drag dropped inside its own fence (手动 sort): place `items` before display
    /// index `index`.
    ReorderItems {
        fence: FenceId,
        items: Vec<ItemId>,
        index: usize,
    },
    /// Files dropped onto a folder item shown inside `fence` (Explorer moves them into the
    /// folder; Ctrl = copy, Alt / Ctrl+Shift = shortcut).
    DropIntoFolder {
        paths: Vec<PathBuf>,
        folder: PathBuf,
        fence: FenceId,
        mode: TransferMode,
    },
    /// An OLE drag out of a fence ended with a Move/Copy performed by a foreign target: resync
    /// the desktop right away instead of waiting for the watcher.
    DragOutFinished,
    /// Folder portal: open a subfolder inside the portal / go one level up.
    PortalEnter {
        fence: FenceId,
        path: PathBuf,
    },
    PortalUp(FenceId),
    /// A fence window is being dragged over `target`'s title (null = none): highlight it and,
    /// on a tabbed target, open the insertion gap at pointer screen `x`.
    MergeHint {
        target: HWND,
        x: i32,
    },
    /// A fence window was dropped on another fence's title at pointer screen `x`: merge it
    /// there as a tab, inserted at the slot under the pointer.
    MergeFence {
        fence: FenceId,
        into: HWND,
        x: i32,
    },
    /// Details header divider dragged: persist the (修改日期, 类型, 大小) widths.
    SetColumnWidths {
        fence: FenceId,
        widths: [f32; 3],
    },
    /// Tab header clicked in `host`'s window.
    SwitchTab {
        host: FenceId,
        tab: FenceId,
    },
    /// A tab header was dragged out of its window (or the menu asked): split it into its own
    /// fence near (x, y). `from_drag` = torn off by dragging; the new window follows the pointer.
    DetachTab {
        tab: FenceId,
        x: i32,
        y: i32,
        from_drag: bool,
    },
    /// Esc / right button: restore the group's original ownership, order and geometry.
    CancelDetach {
        change: Box<pecofence_core::TabDetach>,
    },
    /// Tab header dragged along the strip (or menu 左移 / 右移): place it at index `to`.
    ReorderTab {
        host: FenceId,
        tab: FenceId,
        to: usize,
    },
    /// Internal drag dropped on the bare desktop: back to the inbox.
    MoveItemsToInbox {
        items: Vec<ItemId>,
    },
    /// Files dropped from Explorer onto a fence with the effect the drop reported (Explorer's
    /// modifier table: Ctrl copy, Alt / Ctrl+Shift shortcut, otherwise move).
    ExternalDrop {
        paths: Vec<PathBuf>,
        to: FenceId,
        mode: TransferMode,
    },
    /// A link dragged from a browser onto a fence: create an Internet Shortcut (.url) like
    /// Explorer does, and file it into `to`.
    ExternalUrlDrop {
        url: String,
        name: Option<String>,
        to: FenceId,
    },
    /// Right-click on the Details header: column visibility menu at screen coordinates.
    HeaderMenu {
        fence: FenceId,
        x: i32,
        y: i32,
    },
    /// Ctrl+C / Ctrl+X: the shell's copy / cut verb on the selection (cut items draw dimmed).
    ClipboardVerb {
        fence: FenceId,
        items: Vec<ItemId>,
        cut: bool,
    },
    /// Ctrl+V / menu 粘贴: the clipboard's files land in `fence` (its folder for a portal).
    Paste {
        fence: FenceId,
    },
    /// Ctrl+Shift+N: 新建文件夹 in `fence`.
    NewFolder {
        fence: FenceId,
    },
    /// Ctrl+wheel over the item area: step the active tab's icon size (Explorer/Fences habit).
    StepIconSize {
        fence: FenceId,
        larger: bool,
    },
    /// Alt+Enter / menu: open the shell Properties sheet for the selection.
    ItemProperties {
        fence: FenceId,
        items: Vec<ItemId>,
    },
    /// F5: re-read this fence's contents (desktop or portal folder) and its icons.
    RefreshFence(FenceId),
    /// Context menu on a fence's title/background at screen coordinates.
    FenceMenu {
        fence: FenceId,
        x: i32,
        y: i32,
    },
    /// Context menu on an item at screen coordinates.
    ItemMenu {
        fence: FenceId,
        items: Vec<ItemId>,
        x: i32,
        y: i32,
    },
    RenameFence {
        fence: FenceId,
        title: String,
    },
    /// F2 / menu on a single selected item: open the inline rename popup.
    RenameItem {
        fence: FenceId,
        item: ItemId,
    },
    /// The rename popup committed a new display name for an item (file gets renamed).
    RenameItemCommit {
        item: ItemId,
        name: String,
    },
    /// Close the open inline rename popup (the view scrolled under it): commit or cancel.
    EndItemRename {
        commit: bool,
    },
    /// Delete key: hand the selection to the shell's `delete` verb (Recycle Bin, confirmations).
    /// `permanent` = Shift held (Explorer's Shift+Delete skips the Recycle Bin).
    DeleteItems {
        fence: FenceId,
        items: Vec<ItemId>,
        permanent: bool,
    },
    /// A composition surface reported `DXGI_ERROR_DEVICE_REMOVED`: rebuild the render stack.
    DeviceLost,
    DeleteFence(FenceId),
    NewFence {
        x: i32,
        y: i32,
    },
    /// The user drew a marquee on the empty desktop: offer a fence with these screen bounds.
    NewFenceRect(RECT),
    ToggleAllFences,
    /// Peek hotkey: float all fences above the current windows (again = end).
    TogglePeek,
    EndPeek,
    ApplyRulesNow,
    OpenSettings,
    /// Settings window on the 「栅栏」 page with this fence selected.
    OpenOptionsForFence(FenceId),
    RedrawAll,
    Quit,
    /// A window was destroyed (unregister).
    WindowGone(HWND),
    /// A retired fence window (deleted / merged) finished fading out: destroy it now.
    FadeOutDone(HWND),
    /// The user clicked a fence: bring it above the other fences.
    RaiseFence(HWND),
    /// Details-view header click: sort by that column (again = reverse).
    SortColumn {
        fence: FenceId,
        sort: pecofence_core::SortMode,
    },
    /// JSON message from the settings page.
    SettingsMessage(String),
}

#[derive(Clone)]
pub struct CommandQueue {
    queue: Rc<RefCell<VecDeque<Command>>>,
    control: Rc<RefCell<Option<HWND>>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            queue: Rc::new(RefCell::new(VecDeque::new())),
            control: Rc::new(RefCell::new(None)),
        }
    }

    pub fn attach(&self, control: &Window) {
        *self.control.borrow_mut() = Some(control.hwnd());
    }

    pub fn push(&self, cmd: Command) {
        self.queue.borrow_mut().push_back(cmd);
        if let Some(h) = *self.control.borrow() {
            // SAFETY-free: PostMessage through the platform wrapper.
            pecofence_platform::window::post_message(h, WM_APP_COMMAND, 0, 0);
        }
    }

    pub fn drain(&self) -> Vec<Command> {
        self.queue.borrow_mut().drain(..).collect()
    }
}
