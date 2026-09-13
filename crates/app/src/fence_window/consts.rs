//! Timer ids, delays, geometry and motion constants shared by the fence window modules.

use super::*;

pub(super) const RESIZE_BORDER_DIP: f32 = 6.0;
pub(super) const TIMER_PEEK_OPEN: usize = 31;
pub(super) const TIMER_PEEK_CLOSE: usize = 32;
pub(super) const TIMER_SHADOW: usize = 33;
/// Fades an "inactive" scrollbar out 2 s after the last scroll / pointer activity (WinUI
/// ScrollView no-indicator countdown).
pub(super) const TIMER_SCROLLBAR: usize = 34;
/// Explorer's "slow double-click" rename: a second click on the label of the already-selected
/// item enters rename once the double-click interval has passed without a double-click.
pub(super) const TIMER_RENAME: usize = 35;
/// Infotip delay (SPI_GETMOUSEHOVERTIME) after the pointer settles on an item / tab / button.
pub(super) const TIMER_TIP: usize = 36;
/// Infotip auto-pop (SPI_GETMESSAGEDURATION, default 5 s), comctl32's TTDT_AUTOPOP equivalent;
/// TTF_TRACK tips do not pop on their own.
pub(super) const TIMER_TIP_HIDE: usize = 38;
/// Overlay scrollbar expand / contract begin delays (WinUI ScrollBarExpandBeginTime 0.4 s and
/// ScrollBarContractBeginTime 0.5 s): the thumb widens only after the pointer has rested on
/// it and stays wide for a moment after it leaves.
pub(super) const TIMER_SCROLLBAR_EXPAND: usize = 39;
pub(super) const TIMER_SCROLLBAR_CONTRACT: usize = 40;
/// Track-click paging auto-repeat while the button is held.
pub(super) const TIMER_SCROLL_REPEAT: usize = 41;
/// A drag resting on a rolled fence's title unrolls it after the hover delay (Explorer's
/// navigation pane spring-loads a collapsed folder under a drag the same way).
pub(super) const TIMER_DRAG_PEEK: usize = 42;
pub(super) const TIMER_VISIBILITY_FINISH: usize = 43;
pub(super) const TIMER_CONTENT_FINISH: usize = 44;
/// DWM can suspend completion callbacks for fully occluded windows. UI state must still
/// settle after the longest (250 ms) transition, independently of compositor visibility.
pub(super) const COMPOSITION_FINISH_MS: u32 = 400;
/// The selected (anchor) item's label unfolds to its full name like the desktop's focused
/// icon. Windows is uncapped; six lines keep a 255-character name from covering two rows in a
/// small fence.
pub(super) const MAX_UNFOLD_LINES: u32 = 6;
pub(super) const SCROLLBAR_EXPAND_BEGIN_MS: u32 = 400;
pub(super) const SCROLLBAR_CONTRACT_BEGIN_MS: u32 = 500;
/// Win32 scrollbar SCROLL_FIRST_DELAY / SCROLL_REPEAT_DELAY.
pub(super) const SCROLL_PAGE_FIRST_MS: u32 = 200;
pub(super) const SCROLL_PAGE_REPEAT_MS: u32 = 50;
/// Inactive-scrollbar linger after the last activity (WinUI ScrollView `s_noIndicatorCountdown`).
pub(super) const SCROLLBAR_LINGER_MS: u32 = 2000;
/// Marquee auto-scroll: a per-frame velocity proportional to how far the pointer is outside
/// the item area (DIP/s per DIP of overshoot, clamped), like Explorer's list view.
pub(super) const MARQUEE_SCROLL_GAIN: f32 = 12.0;
pub(super) const MARQUEE_SCROLL_MIN: f32 = 60.0;
pub(super) const MARQUEE_SCROLL_MAX: f32 = 1200.0;
/// OLE drag auto-scroll: a pointer inside the item area within this many DIPs of its top /
/// bottom edge scrolls, faster the closer it is to the edge (roughly one list row deep).
pub(super) const DRAG_SCROLL_ZONE_DIP: f32 = 20.0;
pub(super) const DRAG_SCROLL_MIN: f32 = 120.0;
pub(super) const DRAG_SCROLL_MAX: f32 = 720.0;
/// The drag must linger in the zone this long before the scroll starts (oleidl default).
pub(super) const DRAG_SCROLL_ARM: Duration = Duration::from_millis(50);
/// Explorer / ListView type-to-select: the typed prefix resets after this pause.
pub(super) const TYPE_AHEAD_TIMEOUT: Duration = Duration::from_millis(1000);
/// Pointer within this many DIPs of the right edge counts as "on the scrollbar".
pub(super) const SCROLLBAR_HOT_DIP: f32 = 12.0;
pub(super) const WM_NCLBUTTONUP: u32 = 0x00A2;
/// `SC_MOVE | HTCAPTION`: starts the caption drag loop as if the title had just been pressed.
pub const SC_MOVE_CAPTION: usize = 0xF012;

/// Shadow bitmaps are re-uploaded (`UpdateLayeredWindow`, several ms for a large fence) at most
/// this often while the body is being resized *interactively*; a trailing update catches the
/// final geometry. Animated (frame-clock) resizes bypass the throttle so the shadow stays
/// attached to the plate like a DWM shadow.
pub(super) const SHADOW_MIN_INTERVAL_MS: u128 = 40;
/// Hover-peek closes this long after the pointer left (no Fluent reference value).
pub(super) const PEEK_CLOSE_MS: u32 = 400;
pub(super) const SNAP_GAP_DIP: i32 = 8;
pub(super) const SNAP_DIST_DIP: i32 = 10;
/// Tab header geometry (DIPs).
pub(super) const TAB_MIN_W: f32 = 48.0;
pub(super) const TAB_MAX_W: f32 = 160.0;
pub(super) const TAB_GAP: f32 = 4.0;
pub(super) const TAB_LEFT: f32 = 8.0;
/// Right-hand title space reserved for the chevron / rolled item count.
pub(super) const TAB_RIGHT_RESERVE: f32 = 44.0;
/// Width of the insertion gap a tabbed strip opens for a fence dragged over it (WinUI TabView's
/// default tab width; the merged tab is laid out for real once it lands).
pub(super) const TAB_MERGE_GAP_W: f32 = 96.0;

/// In-place drag image limits (Explorer's DefView renders the selection in place only for a
/// small footprint; beyond it the shell's thumbnail stack + count badge is used): at most this
/// many items and this footprint in DIPs — a 2×2 block of 96 px icons or 3×2 of 48 px still
/// lifts off in place, any Rows-layout selection wider than that does not.
pub(super) const DRAG_IMAGE_MAX_ITEMS: usize = 4;
pub(super) const DRAG_IMAGE_MAX_DIP: f32 = 320.0;

/// Dragging a tab header this far (DIPs) out of the title row splits it into its own fence.
pub(super) const TAB_DETACH_DIP: f32 = 28.0;

/// WinUI ScrollBarVerticalThumbMinHeight.
pub(super) const SCROLLBAR_THUMB_MIN: f32 = 30.0;

/// Scale of a fence at the start of its entrance / end of its exit (Fluent Direct Entrance).
pub(super) const ENTRANCE_SCALE: f32 = 0.97;
/// Distance (DIPs) the incoming tab content slides from the side of the new tab. Fluent
/// publishes no figure for page-refresh slides; fences are small, so this stays short.
pub(super) const TAB_SLIDE_DIP: f32 = 24.0;
