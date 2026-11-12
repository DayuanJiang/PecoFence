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
/// A folder watcher queued events; `wparam != 0` asks for a desktop sync whatever the batch
/// holds (a menu verb that changed a namespace item).
pub const WM_APP_FS_CHANGED: u32 = msg::WM_APP + 2;
pub const WM_APP_TRAY: u32 = msg::WM_APP + 3;
/// The wallpaper registry key changed (worker thread → UI thread). wparam 1: a foreground
/// change (lparam the window); 2: the foreground window is on another virtual desktop.
pub const WM_APP_WALLPAPER: u32 = msg::WM_APP + 6;
/// The wallpaper reader answered (`WallpaperReader`, worker thread → UI thread).
pub const WM_APP_WALLPAPER_READ: u32 = msg::WM_APP + 30;
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
/// Posted by the shell (`SHChangeNotifyRegister`) when the Recycle Bin's contents or the
/// desktop namespace changed; `wparam`/`lparam` carry the notification to release.
pub const WM_APP_SHELL_CHANGED: u32 = msg::WM_APP + 12;
/// Posted by the worker that decoded another virtual desktop's wallpaper in advance.
pub const WM_APP_PREWARM_DONE: u32 = msg::WM_APP + 13;
/// wparam 0: the live wallpaper capture worker finished; 1: the desktop came to the front.
pub const WM_APP_LIVE_WALLPAPER: u32 = msg::WM_APP + 14;
/// Sent to a fence window while another fence is dragged over it: returns the width in device
/// px of its left tab column (0 without one). Such a fence has no title row; its column takes
/// the merge drop instead, also while the fence is collapsed to it.
pub const WM_APP_TAB_RAIL: u32 = msg::WM_APP + 15;
/// The startup wallpaper worker finished after the fences went up (worker thread → UI thread).
pub const WM_APP_STARTUP_BACKDROPS: u32 = msg::WM_APP + 16;
/// The Store edition's startup switch was read on a worker (worker thread → UI thread).
pub const WM_APP_STARTUP_TASK: u32 = msg::WM_APP + 17;
/// A file in the custom themes folder changed (watcher thread → UI thread).
pub const WM_APP_THEMES_CHANGED: u32 = msg::WM_APP + 18;
/// 像桌面一样排列: the fence window takes its new shape (posted, so SetWindowRgn's own
/// messages never arrive while the view is borrowed).
pub const WM_APP_DESKTOP_SHAPE: u32 = msg::WM_APP + 19;
/// 脚本栅栏: a script run ended on its worker thread; the App collects the result.
pub const WM_APP_SCRIPT_DONE: u32 = msg::WM_APP + 20;
/// 搜索整台电脑: the Everything install ended (wParam: 1 done, 0 failed, 2 no winget).
pub const WM_APP_EVERYTHING_SETUP: u32 = msg::WM_APP + 21;
/// 对话框跳转 offer: a file dialog and an Explorer folder to offer (`dialog_jump::take_offer`).
pub const WM_APP_DIALOG_OFFER: u32 = msg::WM_APP + 22;
/// 智能栅栏: a folder read without Everything finished on its worker.
pub const WM_APP_SMART_READ: u32 = msg::WM_APP + 32;
/// 智能栅栏: a smart fence's own folder watcher saw a change (the fences wait in
/// `SmartState::changed`).
pub const WM_APP_SMART_FS: u32 = msg::WM_APP + 33;
/// A save of the configuration failed on the saver thread (`state::saver_for`); the App takes
/// the failure and says so on the tray.
pub const WM_APP_SAVE_FAILED: u32 = msg::WM_APP + 23;

/// What a file drop does with the dropped paths (Explorer's Move / Copy / Create shortcut, the
/// effect the drop target reported).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferMode {
    Move,
    Copy,
    /// Create `.lnk` shortcuts to the paths at the destination; the originals stay.
    Link,
}

/// Where dragged icons landed on the desktop (screen px): the cursor, and the centre of each
/// icon (as far from the cursor as it was when the drag started).
#[derive(Clone, Debug, PartialEq)]
pub struct DesktopDrop {
    pub cursor: (i32, i32),
    pub centres: Vec<(ItemId, (i32, i32))>,
}

#[derive(Debug)]
pub enum Command {
    /// A fence window finished a move/resize; persist its geometry.
    FenceBoundsChanged {
        fence: FenceId,
        rect: RECT,
    },
    /// The user let go of a fence they were moving (not resized, not cancelled, not merged):
    /// if it now covers another fence, it moves to the nearest free spot.
    FenceDropped(FenceId),
    ToggleRollUp(FenceId),
    /// A double-click on the bottom edge: the height that shows every icon (适应内容大小).
    FitToContent(FenceId),
    /// A hover-peeked fence was double-clicked: keep it expanded.
    CommitExpanded(FenceId),
    /// Open `item` from fence `from` (a folder opens in place when `from` is a navigating portal).
    /// `inside` is false when several items are opened at once: folders open in Explorer then.
    LaunchItem {
        item: ItemId,
        from: FenceId,
        inside: bool,
    },
    /// Move `items` into `to` (internal drag or menu). `at`: dropped between the icons of a
    /// 手动 fence, they go before that display index (counted before they arrived); None = after
    /// the others.
    MoveItems {
        items: Vec<ItemId>,
        to: FenceId,
        at: Option<usize>,
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
    /// The mouse's forward button: back into the folder the last up (or back button) left.
    PortalForward(FenceId),
    /// A click on a folder fence's title glyph at its root folder: Explorer opens the folder.
    OpenPortalFolder(FenceId),
    /// Tab / Shift+Tab in fence window `from`: the keyboard moves to the next / previous
    /// fence in reading order. A null `from` is the Peek's first Tab or arrow key, which no
    /// fence received: the first fence (the last one for Shift+Tab).
    FocusNextFence {
        from: HWND,
        back: bool,
    },
    /// A fence window is being dragged over `target`'s title or left tab column (null = none):
    /// highlight it and, on a tabbed target, open the insertion gap at pointer screen point
    /// `at`.
    MergeHint {
        target: HWND,
        at: (i32, i32),
    },
    /// A fence window was dropped on another fence's title or tab column at pointer screen
    /// point `at`: merge it there as a tab, inserted at the slot under the pointer
    /// (`i32::MIN` = no pointer: appended).
    MergeFence {
        fence: FenceId,
        into: HWND,
        at: (i32, i32),
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
    /// Internal drag dropped on the bare desktop: back to the inbox. `at`: where the icons
    /// landed, for 自由摆放 (None from a test script).
    MoveItemsToInbox {
        items: Vec<ItemId>,
        at: Option<DesktopDrop>,
    },
    /// 自由摆放: the 「桌面」 fence gave icons other cells than their saved ones (new icons, or
    /// ones moved off a fence or into a smaller work area): save what its latest pass placed
    /// for this monitor setup.
    DesktopCellsPlaced {
        fence: FenceId,
    },
    /// Files dropped from Explorer onto a fence with the effect the drop reported (Explorer's
    /// modifier table: Ctrl copy, Alt / Ctrl+Shift shortcut, otherwise move).
    ExternalDrop {
        paths: Vec<PathBuf>,
        to: FenceId,
        mode: TransferMode,
    },
    /// A link dragged from a browser onto a fence: create an Internet Shortcut (.url) like
    /// Explorer does, and file it into `to` (or write it into `folder`, the folder item it was
    /// dropped on).
    ExternalUrlDrop {
        url: String,
        name: Option<String>,
        to: FenceId,
        folder: Option<PathBuf>,
    },
    /// A drop onto `to` was handed to the shell's folder drop target (Outlook mail, a
    /// Start-menu app): the portals show what landed in their folders; `landed` are the desktop
    /// entries a virtual fence's drop created, to be filed into `to`. `on_desktop`: the drop
    /// went to the desktop folder (a virtual fence), so what the shell writes there a moment
    /// later goes into `to` too.
    ShellDrop {
        to: FenceId,
        landed: Vec<PathBuf>,
        on_desktop: bool,
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
    /// A fence just made has faded in: its title opens for editing
    /// (`FenceWindow::rename_after_fade_in`).
    BeginFenceRename(FenceId),
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
    /// Enter in the rename popup with a name holding a reserved character: the editor stays
    /// open, the App says why.
    RenameItemInvalid,
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
    /// The name box over this fence's title, or the editor of this note, closed while it had
    /// the keyboard (Enter or Esc): the fence takes the keyboard, so the next key reaches it
    /// and not the taskbar.
    FocusFence(FenceId),
    DeleteFence(FenceId),
    NewFence {
        x: i32,
        y: i32,
    },
    /// 便签与待办: a new note fence near the screen point (tray / fence menu, settings page).
    NewNote {
        x: i32,
        y: i32,
    },
    /// 新建小工具 › an example: its script fence near the screen point.
    NewScriptExample {
        example: crate::scripts::Example,
        x: i32,
        y: i32,
    },
    /// 时钟: a new clock fence near the screen point (none while the desktop has one).
    NewClock {
        x: i32,
        y: i32,
    },
    /// 脚本栅栏: pick a script, then a new script fence near the screen point.
    NewScript {
        x: i32,
        y: i32,
    },
    /// 新建智能栅栏… (`fence` None; the new fence goes near the screen point) or 编辑条件… of a
    /// smart fence: the smart fence editor.
    OpenSmartEditor {
        fence: Option<FenceId>,
        x: i32,
        y: i32,
        by: &'static str,
    },
    /// Everything answered a smart fence's search (`WM_COPYDATA` to the control window).
    SmartResults {
        kind: usize,
        bytes: Vec<u8>,
    },
    /// Ctrl+F or 筛选… in a folder fence or a smart fence: open its filter box.
    OpenFenceFilter(FenceId),
    /// The filter box's text changed (empty: no filter).
    FenceFilterText {
        fence: FenceId,
        text: String,
    },
    /// The filter box closed (the filter stays as typed).
    FenceFilterClosed(FenceId),
    /// Esc in a filtered fence, or 清除筛选 in its menu: the fence shows everything again.
    ClearFenceFilter(FenceId),
    /// A click on a note's text: open its editor, the caret at that place of the text
    /// (UTF-16 units).
    EditNote {
        fence: FenceId,
        caret: Option<usize>,
    },
    /// A click on a to-do's check box in a note.
    ToggleNoteLine {
        fence: FenceId,
        line: usize,
    },
    /// 脚本栅栏: a click on a clickable row of the script's output.
    ScriptRowClicked {
        fence: FenceId,
        row: usize,
    },
    /// A click on the row under a fence's empty-state text (a smart fence that needs
    /// Everything: 「启动 Everything」 / 「安装 Everything」).
    EmptyActionClicked {
        fence: FenceId,
    },
    /// The note editor holds this text and selection now (UTF-16 units, line breaks `\n`;
    /// the caret is at `active`): the note's fence draws them.
    NoteEditState {
        fence: FenceId,
        text: String,
        anchor: usize,
        active: usize,
    },
    /// The caret of the note being edited shows or hides (its blink).
    NoteCaretBlink {
        fence: FenceId,
        on: bool,
    },
    /// The mouse in the text of the note being edited: the caret goes to `offset` (`extend`
    /// keeps the other end of the selection, `word` takes the word there).
    NoteCaret {
        fence: FenceId,
        offset: usize,
        extend: bool,
        word: bool,
    },
    /// Up or Down in the note editor: the caret goes one line on screen up (`dir` < 0) or
    /// down, as the fence wraps the text.
    NoteCaretLine {
        fence: FenceId,
        dir: i8,
        extend: bool,
    },
    /// A right-click in the text of the note being edited: the editor's own text menu opens
    /// at this screen point.
    NoteEditMenu {
        x: i32,
        y: i32,
    },
    /// A press in a fence window while a note editor is open: the editor saves and closes
    /// (a tool fence takes no focus, so the editor would not notice the click).
    EndNoteEdit,
    /// The note editor closed with this text.
    NoteEdited {
        fence: FenceId,
        text: String,
    },
    /// Offer a fence with these screen bounds (a desktop marquee that touched no fence).
    NewFenceRect(RECT),
    /// The buttons that offer were clicked, or went away.
    MarqueeOffer(crate::marquee_offer::OfferEvent),
    /// A press / drag / release on the empty desktop: selects the fences the marquee touches.
    DesktopMarquee(crate::anchor::MarqueeEvent),
    /// A click elsewhere ended the desktop fence selection.
    ClearFenceSelection,
    /// A press in the fence window `except` ends the icon selection of every other fence.
    EndItemSelection {
        except: HWND,
    },
    /// Peek hotkey: float all fences above the current windows (again = end).
    TogglePeek,
    /// End the peek; the reason (a fixed text) goes to the log.
    EndPeek(&'static str),
    /// A file drag out of a fence started: during a Peek the dimmer fades away while it runs.
    PeekDragStarted,
    /// That drag ended; `elsewhere`: it was dropped on another program's window, which ends a
    /// Peek. Otherwise (cancelled, or dropped on a fence or the desktop) the dimmer comes back.
    PeekDragEnded {
        elsewhere: bool,
    },
    /// Text typed during a Peek while the search box did not have the keyboard yet: opens the
    /// box with it, or adds it to the box.
    PeekSearchType(String),
    /// The Peek search box text changed.
    PeekSearchChanged,
    /// Up / Down in the search box: highlight the previous (-1) or next (+1) match.
    PeekSearchStep(i32),
    /// Enter in the search box: open the highlighted match.
    PeekSearchOpen,
    /// A click on a row of the search list, or its Ctrl+digit: open that row.
    PeekSearchOpenRow(usize),
    /// Ctrl+Enter in the search box: open the folder holding the highlighted match.
    PeekSearchOpenFolder,
    /// Right (caret at the end), Ctrl+O, the menu key or Shift+F10 in the search box: the
    /// highlighted match's actions, under its row.
    PeekSearchActions,
    /// A right-click on a row of the search list: highlight it and show its actions at the
    /// pointer (`x`, `y`: screen pixels).
    PeekSearchRowMenu {
        row: usize,
        x: i32,
        y: i32,
    },
    /// Everything answered a search query (`WM_COPYDATA` to the search box).
    EverythingResults {
        kind: usize,
        bytes: Vec<u8>,
    },
    ApplyRulesNow,
    OpenSettings,
    /// Settings window on the 「栅栏」 page with this fence selected.
    OpenOptionsForFence(FenceId),
    RedrawAll,
    /// Leave; the reason (a fixed text) goes to the log.
    Quit(&'static str),
    /// A window was destroyed (unregister).
    WindowGone(HWND),
    /// The settings window became the active window (back from Windows' Startup page, say).
    SettingsActivated,
    /// A retired fence window (deleted / merged) finished fading out: destroy it now.
    FadeOutDone(HWND),
    /// The user clicked a fence: bring it above the other fences.
    RaiseFence(HWND),
    /// A hover peek (or a file drag resting on its title) opened a rolled fence: above the
    /// other fences while it is open (#197).
    RaiseForHover(HWND),
    /// That peek rolled it up again: back to its place among the other fences.
    EndHoverRaise(HWND),
    /// Details-view header click: sort by that column (again = reverse).
    SortColumn {
        fence: FenceId,
        sort: pecofence_core::SortMode,
    },
    /// JSON message from the settings page.
    SettingsMessage(String),
    /// JSON message from the smart fence editor's page.
    EditorMessage(String),
    /// The rating notice was clicked or went away (see `notice.rs`).
    Notice(crate::notice::NoticeEvent),
    /// Ctrl+Z in a fence (see `app/undo.rs`).
    Undo,
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
