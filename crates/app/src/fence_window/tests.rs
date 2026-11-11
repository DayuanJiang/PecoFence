use super::*;
use crate::drag_guides::GuideLine;

/// A 600 px fence grabbed 500 px from its left edge on a 200 % monitor is dragged onto a 100 %
/// one: WM_DPICHANGED halves it to 300 px, and the cursor must keep holding the same point of
/// it (250 px in), not sit 200 px to the right of the window.
#[test]
fn grab_offset_follows_a_dpi_change_mid_drag() {
    let offset = grab_offset_for_dpi((500, 18), 192, 96);
    let (cursor_x, width) = (3000, 300);
    let left = cursor_x - offset.0;
    assert!(
        cursor_x >= left && cursor_x < left + width,
        "cursor {cursor_x} outside the dragged window {left}..{}",
        left + width
    );
    assert_eq!(offset, (250, 9));
    assert_eq!(grab_offset_for_dpi((500, 18), 96, 96), (500, 18));
    assert_eq!(grab_offset_for_dpi((250, 9), 96, 144), (375, 14));
    assert_eq!(grab_offset_for_dpi((40, 12), 0, 144), (40, 12));
}

#[test]
fn window_drag_keeps_the_latest_sample_and_release_before_first_frame() {
    let mut drag = RemoteDrag {
        hwnd: HWND::default(),
        fence: FenceId::new_v4(),
        offset: (20, 10),
        offset_dpi: 96,
        merge_target: 0,
        merge_at: (i32::MIN, i32::MIN),
        moved_once: false,
        origin: WindowDragOrigin::Caption {
            rect: RECT::default(),
            click_expand: false,
        },
        group: Vec::new(),
        press: (-500, 200),
        last_pointer: (-500, 200),
        pending_pointer: None,
        started: false,
        requests: 0,
        applied: 0,
    };
    drag.track_pointer((-496, 204), (4, 4));
    assert!(
        !drag.started,
        "A click within the system threshold must not drag"
    );
    assert_eq!(drag.pending_pointer, None);
    for x in -495..500 {
        drag.track_pointer((x, 200), (4, 4));
    }
    assert_eq!(
        drag.pending_pointer,
        Some((499, 200)),
        "Do not queue stale positions"
    );
    assert_eq!(
        drag.applied, 0,
        "Mouse messages must not move the HWND themselves"
    );
    drag.track_pointer((550, 200), (4, 4)); // Mouse-up before the frame.
    assert_eq!(drag.pending_pointer.take(), Some((550, 200)));
    drag.track_pointer((550, 200), (4, 4));
    assert_eq!(
        drag.pending_pointer, None,
        "Identical points do not schedule extra frames"
    );
}

#[test]
fn tab_sweeps_cross_multiple_compressed_slots_without_bouncing() {
    for available in [70.0, 150.0, 500.0] {
        for initial in [vec![140.0, 48.0, 80.0, 140.0], vec![80.0; 4]] {
            for (from, right) in [(0, true), (3, false)] {
                let mut widths = initial.clone();
                let mut index = from;
                let first = tab_strip_rects(&widths, None, available);
                let grab = first[from].1 * 0.5;
                let pointer = if right {
                    TAB_LEFT + available + grab
                } else {
                    TAB_LEFT - grab
                };
                let mut steps = 0;
                loop {
                    let slots = tab_strip_rects(&widths, None, available);
                    let Some(to) = tab_reorder_at_pointer(index, pointer, grab, &slots) else {
                        break;
                    };
                    let width = widths.remove(index);
                    widths.insert(to, width);
                    index = to;
                    steps += 1;
                    assert!(steps < 4, "A held pointer must not oscillate");
                }
                assert_eq!(index, if right { 3 } else { 0 });
                assert_eq!(steps, 3);
            }
        }
    }
}

#[test]
fn auto_height_keeps_long_lists_above_the_taskbar() {
    assert_eq!(bounded_auto_height(2162, 72, 360, 2064), 1704);
    assert_eq!(bounded_auto_height(490, 72, 360, 2064), 490);
    assert_eq!(bounded_auto_height(1300, 36, -900, 0), 900);
    assert_eq!(bounded_auto_height(500, 72, 2030, 2064), 73);
}

#[test]
fn focus_only_navigation_does_not_select_and_shift_keeps_its_start() {
    let mut selected = HashSet::new();
    let (mut anchor, mut range_anchor) = (None, None);
    apply_cursor_selection(
        &mut selected,
        &mut anchor,
        &mut range_anchor,
        0,
        CursorMode::FocusOnly,
    );
    assert!(
        selected.is_empty(),
        "Ctrl navigation must not select an item"
    );
    for next in [1, 2, 3] {
        apply_cursor_selection(
            &mut selected,
            &mut anchor,
            &mut range_anchor,
            next,
            CursorMode::Extend,
        );
        assert_eq!(selected, (0..=next).collect());
        assert_eq!(range_anchor, Some(0), "Shift anchor must not advance");
    }
    apply_cursor_selection(
        &mut selected,
        &mut anchor,
        &mut range_anchor,
        1,
        CursorMode::Extend,
    );
    assert_eq!(
        selected,
        HashSet::from([0, 1]),
        "Reversing shrinks the same range"
    );
}

#[test]
fn ordinary_navigation_resets_the_range_without_changing_ctrl_selection() {
    let mut selected = HashSet::from([2, 5]);
    let (mut anchor, mut range_anchor) = (Some(5), Some(5));
    apply_cursor_selection(
        &mut selected,
        &mut anchor,
        &mut range_anchor,
        7,
        CursorMode::FocusOnly,
    );
    assert_eq!(selected, HashSet::from([2, 5]));
    assert_eq!(anchor, Some(7));
    apply_cursor_selection(
        &mut selected,
        &mut anchor,
        &mut range_anchor,
        4,
        CursorMode::Select,
    );
    assert_eq!(selected, HashSet::from([4]));
    assert_eq!(range_anchor, Some(4));
    apply_cursor_selection(
        &mut selected,
        &mut anchor,
        &mut range_anchor,
        2,
        CursorMode::Extend,
    );
    assert_eq!(selected, HashSet::from([2, 3, 4]));
}

/// WinUI GetTabInsertionIndex: before a pill's centre = its slot, after = the next; off the
/// ends = 0 / len; an empty strip appends at 0.
#[test]
fn merge_slot_follows_pill_centres() {
    let rects = [(8.0, 80.0), (92.0, 60.0), (156.0, 100.0)];
    assert_eq!(merge_slot_for(-10.0, &rects), 0);
    assert_eq!(merge_slot_for(8.0, &rects), 0);
    assert_eq!(merge_slot_for(47.0, &rects), 0);
    assert_eq!(merge_slot_for(48.0, &rects), 1);
    assert_eq!(merge_slot_for(90.0, &rects), 1, "between pills: next slot");
    assert_eq!(merge_slot_for(121.0, &rects), 1);
    assert_eq!(merge_slot_for(122.5, &rects), 2);
    assert_eq!(merge_slot_for(205.0, &rects), 2);
    assert_eq!(merge_slot_for(206.0, &rects), 3);
    assert_eq!(merge_slot_for(1000.0, &rects), 3);
    assert_eq!(merge_slot_for(50.0, &[]), 0);
}

/// Hit-testing the drawn strip (laid out around the open gap) is a fixed point: for every
/// pointer x the slot the rule picks, re-laid-out with the gap there, picks itself again —
/// no oscillation — and a pointer inside the gap keeps the slot. Holds with and without
/// the proportional shrink of a crowded strip.
#[test]
fn merge_slot_on_drawn_strip_is_stable() {
    let natural = [80.0, 60.0, 100.0, 48.0];
    for available in [1000.0, 300.0, 160.0] {
        let plain = tab_strip_rects(&natural, None, available);
        assert_eq!(plain.len(), natural.len());
        let mut x = 0.0;
        while x < 400.0 {
            let mut slot = merge_slot_for(x, &plain);
            for _ in 0..3 {
                let drawn = tab_strip_rects(&natural, Some(slot), available);
                let next = merge_slot_for(x, &drawn);
                assert_eq!(next, slot, "x={x} available={available}");
                slot = next;
            }
            // Inside the gap (between the neighbouring pills' centres) the slot holds.
            let drawn = tab_strip_rects(&natural, Some(slot), available);
            let lo = slot
                .checked_sub(1)
                .and_then(|i| drawn.get(i))
                .map_or(f32::MIN, |(tx, tw)| tx + tw / 2.0);
            let hi = drawn.get(slot).map_or(f32::MAX, |(tx, tw)| tx + tw / 2.0);
            for gx in [lo + 0.5, (lo.max(0.0) + hi.min(400.0)) / 2.0, hi - 0.5] {
                if gx > lo && gx < hi {
                    assert_eq!(merge_slot_for(gx, &drawn), slot);
                }
            }
            x += 0.5;
        }
    }
    // A gap past the end clamps to append; a single-pill strip still lays out.
    assert_eq!(
        tab_strip_rects(&[80.0], Some(9), 500.0),
        vec![(TAB_LEFT, 80.0)]
    );
}

/// A merge into a left tab column collapsed to fit its rows opens one row: the rows shrink by a
/// quarter for three tabs, where a title-row-wide gap would halve them.
#[test]
fn merge_gap_in_left_column_is_one_row() {
    let rows = [RAIL_ROW_H; 3];
    let fit = 3.0 * RAIL_ROW_H + 2.0 * TAB_GAP;
    let drawn = tab_strip_rects_with(&rows, Some((1, RAIL_ROW_H)), fit);
    assert_eq!(drawn.len(), 3);
    for (_, h) in &drawn {
        assert!(*h > RAIL_ROW_H * 0.7, "row squeezed to {h}");
    }
    // The second row moves down past the gap.
    assert!(drawn[1].0 > drawn[0].0 + drawn[0].1 + RAIL_ROW_H * 0.7);
}

/// The shell's stack + count badge takes over for large footprints; small ones lift off in
/// place.
#[test]
fn drag_image_footprint_limit() {
    assert!(drag_image_fits(110.0, 140.0));
    assert!(drag_image_fits(320.0, 320.0));
    assert!(!drag_image_fits(321.0, 100.0));
    assert!(!drag_image_fits(100.0, 400.0));
    assert_eq!(DRAG_IMAGE_MAX_ITEMS, 4);
}

/// A fence collapsed to its left tab column keeps no room for the chevron under the tabs: two
/// tabs take 84 DIP instead of the expanded column's 120.
#[test]
fn collapsed_tab_column_has_no_chevron_room() {
    assert_eq!(rail_len_for(2, RAIL_BOTTOM), 120.0);
    assert_eq!(rail_len_for(2, RAIL_COLLAPSED_BOTTOM), 84.0);
    assert_eq!(rail_len_for(3, RAIL_COLLAPSED_BOTTOM), 120.0);
}

#[test]
fn narrow_tab_strips_cannot_cover_the_chevron_or_count() {
    for count in 2..=10 {
        let natural = vec![120.0; count];
        for available in [24.0, 48.0, 84.0, 140.0, 320.0] {
            for gap in [None, Some(0), Some(count / 2), Some(count)] {
                let rects = tab_strip_rects(&natural, gap, available);
                assert_eq!(rects.len(), count);
                for &(x, w) in &rects {
                    assert!(x >= TAB_LEFT && w >= 0.0);
                    assert!(x + w <= TAB_LEFT + available + 0.001);
                }
                assert!(rects.windows(2).all(|r| r[0].0 + r[0].1 <= r[1].0 + 0.001));
            }
        }
    }
}

/// A scrolled title-row strip keeps every pill at its natural width, `TAB_GAP` apart, and the
/// merge gap opens a slot in it.
#[test]
fn scrolled_strip_keeps_natural_widths() {
    let natural = [80.0, 60.0, 100.0];
    assert_eq!(tab_strip_span(&natural, None), 240.0 + 2.0 * TAB_GAP);
    assert_eq!(
        tab_strip_span(&natural, Some((1, TAB_MERGE_GAP_W))),
        240.0 + TAB_MERGE_GAP_W + 3.0 * TAB_GAP
    );
    let rects = tab_strip_scrolled(&natural, None, 30.0);
    assert_eq!(
        rects,
        vec![
            (TAB_LEFT - 30.0, 80.0),
            (TAB_LEFT + 54.0, 60.0),
            (TAB_LEFT + 118.0, 100.0)
        ]
    );
    let gapped = tab_strip_scrolled(&natural, Some((1, TAB_MERGE_GAP_W)), 0.0);
    assert_eq!(gapped[1].0, TAB_LEFT + 84.0 + TAB_MERGE_GAP_W + TAB_GAP);
}

/// The strip scrolls only as far as needed to show the active pill clear of the fades, and
/// never past its ends.
#[test]
fn strip_scrolls_just_enough_to_show_the_active_tab() {
    let m = TAB_SCROLL_MARGIN;
    // 200 DIPs of room over a 500 DIP strip: at most 300 of scroll.
    assert_eq!(scroll_to_show(0.0, 50.0, 80.0, 200.0, 300.0), 0.0);
    assert_eq!(scroll_to_show(0.0, 250.0, 80.0, 200.0, 300.0), 130.0 + m);
    assert_eq!(scroll_to_show(170.0, 160.0, 80.0, 200.0, 300.0), 160.0 - m);
    // At the strip's ends there is nothing to keep room for.
    assert_eq!(scroll_to_show(0.0, 10.0, 80.0, 200.0, 300.0), 0.0);
    assert_eq!(scroll_to_show(130.0, 420.0, 80.0, 200.0, 300.0), 300.0);
    // A room too narrow for both margins centres the pill.
    assert_eq!(scroll_to_show(0.0, 100.0, 80.0, 120.0, 300.0), 80.0);
    assert_eq!(scroll_to_show(0.0, 100.0, 250.0, 200.0, 300.0), 125.0);
}
/// Right-drag menu offers only what both the source and this target can do, in Explorer's
/// order.
/// Explorer: Ctrl+dragging one item of a multi-selection copies the whole selection; the
/// Ctrl-press only deselects the item when the button comes up without a drag.
#[test]
fn ctrl_drag_carries_the_whole_selection() {
    let mut selected: HashSet<usize> = [0, 1, 2].into_iter().collect();
    let mut anchor = Some(0);
    press_item(&mut selected, &mut anchor, 1, false, true);
    drag_selection(&mut selected, 1);
    let mut dragged: Vec<usize> = selected.iter().copied().collect();
    dragged.sort();
    assert_eq!(
        dragged,
        vec![0, 1, 2],
        "Ctrl+drag lost the rest of the selection"
    );
    // A Ctrl-click without a drag still toggles the item off.
    let mut selected: HashSet<usize> = [0, 1, 2].into_iter().collect();
    let press2 = press_item(&mut selected, &mut anchor, 1, false, true);
    release_item(&mut selected, 1, press2);
    let mut left: Vec<usize> = selected.iter().copied().collect();
    left.sort();
    assert_eq!(left, vec![0, 2]);
    // Ctrl on an unselected item adds it at once (it is dragged along too).
    let mut selected: HashSet<usize> = [0].into_iter().collect();
    press_item(&mut selected, &mut anchor, 3, false, true);
    assert!(selected.contains(&3) && selected.contains(&0));
}

/// Explorer's single-click open on the desktop: resting on an item selects it alone, Ctrl
/// toggles it (adds, then removes on the next rest), Shift selects the range from the anchor.
#[test]
fn hover_select_follows_the_desktop() {
    let sorted = |s: &HashSet<usize>| {
        let mut v: Vec<usize> = s.iter().copied().collect();
        v.sort();
        v
    };
    let mut selected = HashSet::new();
    let mut anchor = None;
    assert!(hover_item(&mut selected, &mut anchor, 2, false, false));
    assert!(hover_item(&mut selected, &mut anchor, 4, false, false));
    assert_eq!(sorted(&selected), vec![4]);
    assert_eq!(anchor, Some(4));
    // Resting on it again changes nothing.
    assert!(!hover_item(&mut selected, &mut anchor, 4, false, false));
    assert!(hover_item(&mut selected, &mut anchor, 1, false, true));
    assert!(hover_item(&mut selected, &mut anchor, 6, false, true));
    assert_eq!(sorted(&selected), vec![1, 4, 6]);
    assert!(hover_item(&mut selected, &mut anchor, 1, false, true));
    assert_eq!(sorted(&selected), vec![4, 6]);
    // A plain rest on an item of a multi-selection keeps it, as a plain press does.
    assert!(!hover_item(&mut selected, &mut anchor, 4, false, false));
    assert_eq!(sorted(&selected), vec![4, 6]);
    assert!(hover_item(&mut selected, &mut anchor, 3, true, false));
    assert_eq!(sorted(&selected), vec![1, 2, 3]);
    assert_eq!(anchor, Some(1));
}

/// Real file paths stay ours; Outlook mail, a Start-menu app (its `CF_HDROP` holds a
/// known-folder-relative "path") and other shell-only items go to the shell; our own fences'
/// drags and browser links keep their paths.
#[test]
fn drop_kind_routes_non_file_items_to_the_shell() {
    let kind = |internal, hdrop: Option<&[PathBuf]>, url, shell| {
        classify_drop(internal, hdrop, || url, || shell)
    };
    let files = [PathBuf::from(r"C:\Users\me\Desktop\a.txt")];
    let unc = [PathBuf::from(r"\\server\share\b.txt")];
    let start_menu = [PathBuf::from(
        r"{7C5A40EF-A0FB-4BFC-874A-C0F2E0B9FA8E}\Steam\steam.exe",
    )];
    let bin = [PathBuf::from("::{645FF040-5081-101B-9F08-00AA002F954E}")];
    assert_eq!(kind(false, Some(&files), false, true), DropKind::Files);
    // A plain Explorer file drag never queries the data object any further.
    assert_eq!(
        classify_drop(false, Some(&files), || unreachable!(), || unreachable!()),
        DropKind::Files
    );
    assert_eq!(kind(false, Some(&unc), false, true), DropKind::Files);
    assert_eq!(kind(false, Some(&start_menu), false, true), DropKind::Shell);
    // Outlook: virtual files only.
    assert_eq!(kind(false, None, false, true), DropKind::Shell);
    // Nothing the shell could take apart either: the paths are all we have.
    assert_eq!(
        kind(false, Some(&start_menu), false, false),
        DropKind::Files
    );
    // Our own drag of the Recycle Bin stays a membership move.
    assert_eq!(kind(true, Some(&bin), false, true), DropKind::Files);
    assert_eq!(kind(true, None, false, true), DropKind::None);
    // A browser link keeps becoming a .url even though it describes a virtual file.
    assert_eq!(kind(false, None, true, true), DropKind::Url);
    assert_eq!(kind(false, None, false, false), DropKind::None);
    // Shown effect: Copy for a plain drag, Link when that is all the Start menu offers.
    assert_eq!(shell_effect(0, dragdrop::ALL_EFFECTS), DropEffect::Copy);
    assert_eq!(
        shell_effect(dragdrop::MK_SHIFT, dragdrop::ALL_EFFECTS),
        DropEffect::Move
    );
    assert_eq!(shell_effect(0, DropEffect::Link.to_raw()), DropEffect::Link);
}

/// A shell drop onto a virtual fence files the new desktop entries plus the described virtual
/// files that were not there before (still copying), each once, never a pre-existing entry.
#[test]
fn shell_drop_landed_lists_new_desktop_entries() {
    let dir = Path::new(r"C:\Desk");
    let before: HashSet<PathBuf> = [dir.join("old.txt"), dir.join("Report.msg")]
        .into_iter()
        .collect();
    let mut after = before.clone();
    after.insert(dir.join("Steam.lnk"));
    let names = vec![
        "Report.msg".to_string(),
        "Budget.msg".to_string(),
        r"Photos\a.jpg".to_string(),
        r"Photos\b.jpg".to_string(),
    ];
    assert_eq!(
        shell_drop_landed(dir, &before, &after, &names),
        vec![
            dir.join("Steam.lnk"),
            dir.join("Budget.msg"),
            dir.join("Photos")
        ]
    );
    assert!(shell_drop_landed(dir, &before, &before, &[]).is_empty());
}

#[test]
fn right_drag_choices_follow_source_and_target() {
    let all = [DropEffect::Move, DropEffect::Copy, DropEffect::Link];
    assert_eq!(
        right_drag_choices(&all, dragdrop::ALL_EFFECTS),
        all.to_vec()
    );
    assert_eq!(
        right_drag_choices(&all, DropEffect::Copy.to_raw() | DropEffect::Link.to_raw()),
        vec![DropEffect::Copy, DropEffect::Link]
    );
    assert_eq!(
        right_drag_choices(&[DropEffect::Link], dragdrop::ALL_EFFECTS),
        vec![DropEffect::Link]
    );
    assert!(right_drag_choices(&all, 0).is_empty());
    assert_eq!(transfer_mode(DropEffect::Link), Some(TransferMode::Link));
    assert_eq!(transfer_mode(DropEffect::None), None);
    // Modifier table clipped to the source mask.
    assert_eq!(
        effect_for(dragdrop::MK_ALT, DropEffect::Copy.to_raw(), true),
        DropEffect::Copy
    );
    assert_eq!(effect_for(0, dragdrop::ALL_EFFECTS, true), DropEffect::Move);
    // From another drive a plain drag copies; Shift still moves.
    assert_eq!(
        effect_for(0, dragdrop::ALL_EFFECTS, false),
        DropEffect::Copy
    );
    assert_eq!(
        effect_for(dragdrop::MK_SHIFT, dragdrop::ALL_EFFECTS, false),
        DropEffect::Move
    );
    assert_eq!(
        effect_for(
            dragdrop::MK_CONTROL | dragdrop::MK_SHIFT,
            dragdrop::ALL_EFFECTS,
            true
        ),
        DropEffect::Link
    );
}

/// Wheel distance follows SPI_GETWHEELSCROLLLINES: N rows per notch in List / Details, one
/// grid row in Icons, a viewport for WHEEL_PAGESCROLL, nothing when the wheel is disabled;
/// touchpad fractions scale linearly.
#[test]
fn wheel_step_follows_scroll_lines_setting() {
    let (row, view_h) = (28.0, 300.0);
    assert_eq!(wheel_step_dip(3, 1.0, row, view_h, false), 84.0);
    assert_eq!(wheel_step_dip(1, 1.0, row, view_h, false), 28.0);
    assert_eq!(wheel_step_dip(3, 1.0, 96.0, view_h, true), 96.0);
    assert_eq!(wheel_step_dip(u32::MAX, 1.0, row, view_h, false), view_h);
    assert_eq!(
        wheel_step_dip(u32::MAX, -2.0, row, view_h, true),
        -2.0 * view_h
    );
    assert_eq!(wheel_step_dip(0, 1.0, row, view_h, false), 0.0);
    assert_eq!(wheel_step_dip(3, 0.5, row, view_h, false), 42.0);
}

/// WinUI's 5 ms/px rule with the per-input cap: a 96 DIP notch would be 480 ms but wheel
/// input caps at 250 ms; tiny moves floor at 50 ms; page jumps cap at 333 ms.
#[test]
fn scroll_duration_is_capped_per_input() {
    assert_eq!(
        scroll_anim_duration(96.0, motion::NORMAL),
        Duration::from_millis(250)
    );
    assert_eq!(
        scroll_anim_duration(20.0, motion::NORMAL),
        Duration::from_millis(100)
    );
    assert_eq!(
        scroll_anim_duration(2.0, motion::NORMAL),
        Duration::from_millis(50)
    );
    assert_eq!(
        scroll_anim_duration(600.0, motion::SLOW),
        Duration::from_millis(333)
    );
    // Track / arrow auto-repeat: each tick's glide lands before the next 50 ms tick.
    assert_eq!(
        scroll_anim_duration(600.0, motion::FASTER),
        Duration::from_millis(83)
    );
    // Precision-touchpad sub-notch deltas: no glide, the value lands on the next frame.
    assert_eq!(scroll_anim_duration(5.0, Duration::ZERO), Duration::ZERO);
    let t0 = Instant::now();
    let t = Tween::new(0.0, 10.0, Duration::ZERO, Curve::Decelerate, t0);
    assert_eq!(t.value_at(t0), 10.0);
    assert!(t.is_done(t0));
}

/// While a roll is in flight the chrome shows its end state (chevron direction, count,
/// tab reserve) so those animate with the height instead of flipping when it lands; the
/// progress tween is phase-locked with the height (167 ms collapse curve reaches 1 exactly
/// at the end).
/// A height change keeps the top edge, or the bottom edge of a fence that expands upward
/// (issue #125): a title row resting on the taskbar grows away from it.
#[test]
fn expand_upward_keeps_the_bottom_edge() {
    let r = RECT {
        left: 20,
        top: 968,
        right: 380,
        bottom: 1000,
    };
    assert_eq!(top_for_height(false, &r, 400), 968);
    assert_eq!(top_for_height(true, &r, 400), 600);
    // Rolling back up lands on the same row.
    let expanded = RECT { top: 600, ..r };
    assert_eq!(top_for_height(true, &expanded, 32), 968);
}

#[test]
fn roll_target_leads_the_committed_state() {
    assert!(roll_target_of(Some(true), false));
    assert!(!roll_target_of(Some(false), true));
    assert!(roll_target_of(None, true));
    assert!(!roll_target_of(None, false));
    let t0 = Instant::now();
    let mut roll_t = Tween::at(0.0, t0);
    roll_t.retarget(1.0, motion::FAST, Curve::Collapse, t0);
    let mid = roll_t.value_at(t0 + Duration::from_millis(80));
    assert!(mid > 0.0 && mid < 1.0, "{mid}");
    assert_eq!(roll_t.value_at(t0 + motion::FAST), 1.0);
    assert!(roll_t.is_done(t0 + motion::FAST));
    // Reversing halfway continues from the current angle, not from 1.
    let half = t0 + Duration::from_millis(80);
    roll_t.retarget(0.0, motion::SLOW, Curve::Decelerate, half);
    assert!((roll_t.value_at(half) - mid).abs() < 1e-5);
}

/// Successive notches retarget the running glide from its current value, so the motion is
/// continuous and the pending target accumulates.
#[test]
fn scroll_glide_retargets_continuously() {
    let t0 = Instant::now();
    let mut t = Tween::new(0.0, 84.0, Duration::from_millis(250), Curve::Decelerate, t0);
    let mid = t0 + Duration::from_millis(100);
    let v_mid = t.value_at(mid);
    assert!(v_mid > 0.0 && v_mid < 84.0);
    // Second notch: new target from the anticipated end, start from the current value.
    let target = t.target() + 84.0;
    t.retarget(target, Duration::from_millis(250), Curve::Decelerate, mid);
    assert_eq!(t.value_at(mid), v_mid);
    assert_eq!(t.target(), 168.0);
    assert!(!t.is_done(mid));
    assert!(t.is_done(mid + Duration::from_millis(250)));
    assert_eq!(t.value_at(mid + Duration::from_millis(250)), 168.0);
    // Zero-length (animations off) is done at once.
    let snap = Tween::new(0.0, 84.0, Duration::ZERO, Curve::Decelerate, t0);
    assert!(snap.is_done(t0));
    assert_eq!(snap.value_at(t0), 84.0);
}

/// Marquee auto-scroll speed grows with the overshoot between a crawl and a cap.
#[test]
fn marquee_speed_is_proportional_and_clamped() {
    assert_eq!(marquee_scroll_speed(1.0), MARQUEE_SCROLL_MIN);
    assert_eq!(marquee_scroll_speed(-1.0), MARQUEE_SCROLL_MIN);
    assert_eq!(marquee_scroll_speed(10.0), 120.0);
    assert_eq!(marquee_scroll_speed(500.0), MARQUEE_SCROLL_MAX);
}

/// OLE drag edge zones: inside the band only, depth 0 at the inner boundary and 1 at the
/// edge, up above / down below, nothing in the middle or outside the item area.
#[test]
fn drag_scroll_zone_geometry() {
    let (top, bottom, zone) = (100, 400, 20);
    assert_eq!(drag_scroll_zone(99, top, bottom, zone), None);
    assert_eq!(drag_scroll_zone(401, top, bottom, zone), None);
    assert_eq!(drag_scroll_zone(250, top, bottom, zone), None);
    assert_eq!(drag_scroll_zone(100, top, bottom, zone), Some((-1, 1.0)));
    assert_eq!(drag_scroll_zone(110, top, bottom, zone), Some((-1, 0.5)));
    assert_eq!(drag_scroll_zone(120, top, bottom, zone), None);
    assert_eq!(drag_scroll_zone(400, top, bottom, zone), Some((1, 1.0)));
    assert_eq!(drag_scroll_zone(390, top, bottom, zone), Some((1, 0.5)));
    assert_eq!(drag_scroll_zone(380, top, bottom, zone), None);
    // Speed: crawl at the boundary, about one row per 50 ms at the edge, clamped.
    assert_eq!(drag_scroll_speed(0.0, 28.0), DRAG_SCROLL_MIN);
    assert_eq!(drag_scroll_speed(1.0, 28.0), 560.0);
    assert_eq!(drag_scroll_speed(1.0, 96.0), DRAG_SCROLL_MAX);
    assert_eq!(drag_scroll_speed(1.0, 4.0), DRAG_SCROLL_MIN);
}

/// The dragged tab pill rides at pointer - grab offset, clamped so it never leaves the strip.
#[test]
fn dragged_tab_stays_inside_the_strip() {
    // Slots: [8, 108), [112, 172); dragged pill 100 wide grabbed 30 DIP into it.
    let (first, end) = (8.0, 172.0);
    assert_eq!(dragged_tab_x(60.0, 30.0, 100.0, first, end), 30.0);
    assert_eq!(dragged_tab_x(0.0, 30.0, 100.0, first, end), first);
    assert_eq!(dragged_tab_x(500.0, 30.0, 100.0, first, end), end - 100.0);
    // A pill wider than the strip parks at the first slot.
    assert_eq!(dragged_tab_x(50.0, 0.0, 400.0, first, end), first);
}

#[test]
fn two_tabs_swap_both_directions_even_when_the_painted_pill_is_clipped() {
    for widths in [[80.0, 80.0], [140.0, 48.0], [48.0, 140.0]] {
        let slots = tab_strip_rects(&widths, None, 500.0);
        for from in 0..2 {
            let to = 1 - from;
            for fraction in [0.1, 0.5, 0.9] {
                let grab = widths[from] * fraction;
                let target_centre = slots[to].0 + slots[to].1 * 0.5;
                let pointer = target_centre + grab - widths[from] * 0.5;
                assert_eq!(
                    tab_reorder_at_pointer(from, pointer, grab, &slots),
                    Some(to)
                );
                let swapped = [widths[1], widths[0]];
                let new_slots = tab_strip_rects(&swapped, None, 500.0);
                assert_eq!(
                    tab_reorder_at_pointer(to, pointer, grab, &new_slots),
                    None,
                    "holding the pointer after a swap must not swap back"
                );
            }
        }
    }
    // This is the old failure: the painted equal-width pill stops exactly on its
    // neighbour's slot. A later pointer position must still produce a swap.
    let slots = [(8.0, 80.0), (92.0, 80.0)];
    assert_eq!(dragged_tab_x(160.0, 40.0, 80.0, 8.0, 172.0), 92.0);
    assert_eq!(tab_reorder_at_pointer(0, 160.0, 40.0, &slots), Some(1));
    // A wide tab grabbed by its leading edge must swap before leaving the
    // window. Otherwise dragging far enough to swap would tear it off instead.
    let slots = [(8.0, 48.0), (60.0, 160.0)];
    assert_eq!(tab_reorder_at_pointer(1, 30.0, 4.0, &slots), Some(0));
    let swapped = [(8.0, 160.0), (172.0, 48.0)];
    assert_eq!(tab_reorder_at_pointer(0, 30.0, 4.0, &swapped), None);
    assert_eq!(tab_reorder_at_pointer(0, 198.0, 156.0, &swapped), Some(1));
    assert_eq!(tab_reorder_at_pointer(1, 198.0, 156.0, &slots), None);
}
/// Reorder rule: the dragged pill swaps with a neighbour when its centre passes that
/// neighbour's centre, one slot per step, and never past either end.
#[test]
fn dragged_tab_swaps_when_its_centre_passes_a_neighbours_centre() {
    // Slots [8, 108), [112, 172), [176, 256): centres 58, 142, 216.
    let slots = [(8.0, 100.0), (112.0, 60.0), (176.0, 80.0)];
    // Pill 0 (100 wide) at rest, then with its centre just short of / past 142.
    assert_eq!(tab_reorder_step(0, 8.0, 100.0, &slots), None);
    assert_eq!(tab_reorder_step(0, 91.9, 100.0, &slots), None);
    assert_eq!(tab_reorder_step(0, 92.1, 100.0, &slots), Some(1));
    // Pill 1 moving left past 58, or right past 216.
    assert_eq!(tab_reorder_step(1, 27.9, 60.0, &slots), Some(0));
    assert_eq!(tab_reorder_step(1, 28.1, 60.0, &slots), None);
    assert_eq!(tab_reorder_step(1, 186.1, 60.0, &slots), Some(2));
    // The ends never step off the strip, however far the pill hangs over.
    assert_eq!(tab_reorder_step(0, -300.0, 100.0, &slots), None);
    assert_eq!(tab_reorder_step(2, 900.0, 80.0, &slots), None);
    assert_eq!(tab_reorder_step(0, 0.0, 10.0, &[]), None);
}

/// A narrow pill crossing a wide neighbour must not oscillate: after the swap the same
/// pointer position must not swap back. Slots are re-derived from the new order (widths
/// travel with their tabs), exactly as `tab_rects` does after a reorder.
#[test]
fn narrow_tab_crossing_a_wide_one_is_stable() {
    let strip = |widths: &[f32]| -> Vec<(f32, f32)> {
        let mut x = 8.0;
        widths
            .iter()
            .map(|w| {
                let slot = (x, *w);
                x += w + 4.0;
                slot
            })
            .collect()
    };
    // A (40) then B (100): B's centre is 102. Pointer parks the pill's centre at 103.
    let (mut widths, mut index) = (vec![40.0, 100.0], 0usize);
    let centre_x = 103.0;
    for _ in 0..4 {
        let slots = strip(&widths);
        let w = widths[index];
        match tab_reorder_step(index, centre_x - w / 2.0, w, &slots) {
            Some(to) => {
                let w = widths.remove(index);
                widths.insert(to, w);
                index = to;
            }
            None => break,
        }
    }
    // Exactly one swap: A now sits right of B and stays there.
    assert_eq!((index, widths.clone()), (1, vec![100.0, 40.0]));
    let slots = strip(&widths);
    assert_eq!(tab_reorder_step(1, centre_x - 20.0, 40.0, &slots), None);
    // And the old rule's failure case, a centre inside B's original slot, does nothing.
    let slots = strip(&[40.0, 100.0]);
    assert_eq!(tab_reorder_step(0, 49.0 - 20.0, 40.0, &slots), None);
}

/// Layout motion is a rearrangement effect: it runs for sorts / reorders / a few adds or
/// removes and snaps for refills (tab switch, portal navigation); the first items of an empty
/// fence fade in.
#[test]
fn layout_motion_applies_only_to_rearrangements() {
    assert!(layout_motion_applies(0, 10, 10));
    assert!(layout_motion_applies(1, 10, 11));
    assert!(layout_motion_applies(5, 10, 10));
    assert!(!layout_motion_applies(6, 10, 10));
    assert!(!layout_motion_applies(20, 10, 10));
    assert!(layout_motion_applies(5, 0, 5));
    // Removing everything still snaps.
    assert!(!layout_motion_applies(10, 10, 0));
}

/// An edge dragged from five columns to four: the fifth icon glides from the end of the first
/// row to the start of the second, an icon already gliding continues from where it is drawn,
/// icons more than a viewport away are left out, and a scroll offset the shorter content
/// clamps moves the start with it (the icon starts from its place on screen).
#[test]
fn reflow_glides_start_where_the_icons_are_drawn() {
    let m = GridMetrics::for_icon_size(48, 2);
    let (old, new) = (Grid::new(m, 408.0, 300), Grid::new(m, 380.0, 300));
    assert_eq!((old.columns, new.columns), (5, 4));
    let in_flight = |i: usize| (i == 0).then_some((7.0, 9.0));
    let glides = reflow_glides(&old, &new, 300, in_flight, (0.0, 0.0), 300.0);
    let of = |g: &[(usize, (f32, f32), (f32, f32))], i: usize| g.iter().find(|x| x.0 == i).copied();
    let (_, from, to) = of(&glides, 4).unwrap();
    assert_eq!(from, (old.cell(4).x, old.cell(4).y));
    assert_eq!(to, (new.cell(4).x, new.cell(4).y));
    assert!(to.0 < from.0 && to.1 > from.1, "{from:?} -> {to:?}");
    assert_eq!(of(&glides, 0).unwrap().1, (7.0, 9.0));
    assert!(of(&glides, 299).is_none());
    // Every icon listed is in or near the 300 DIP viewport before or after.
    assert!(glides.len() < 60, "{}", glides.len());
    let glides = reflow_glides(&old, &new, 300, |_| None, (400.0, 350.0), 300.0);
    assert_eq!(of(&glides, 20).unwrap().1.1, old.cell(20).y - 50.0);
}

/// A dropped icon starts with its centre where its drag picture's was, and glides from there.
#[test]
fn a_dropped_icon_starts_under_its_drag_picture() {
    let cell = CellRect {
        x: 160.0,
        y: 96.0,
        w: 80.0,
        h: 96.0,
    };
    // Let go with its centre at (300, 400): the cell origin is half a cell up and left.
    assert_eq!(landing_origin(300.0, 400.0, &cell), (260.0, 352.0));
    // Let go exactly over its cell: no distance to cover.
    assert_eq!(landing_origin(200.0, 144.0, &cell), (cell.x, cell.y));
}

/// A new icon whose cell another icon is still sliding out of fades in 83 ms later; elsewhere
/// at once. The delayed fade stays at 0 until then and takes its 167 ms from there.
#[test]
fn a_new_icon_waits_for_the_icon_leaving_its_cell() {
    let now = Instant::now();
    assert_eq!(fade_in_start(false, now), now);
    let start = fade_in_start(true, now);
    assert_eq!(start, now + motion::FASTER);
    let fade = Tween::new(0.0, 1.0, motion::FAST, Curve::Linear, start);
    assert_eq!(fade.value_at(now), 0.0);
    assert_eq!(fade.value_at(now + Duration::from_millis(80)), 0.0);
    assert!(!fade.is_done(now + Duration::from_millis(80)));
    assert!(fade.value_at(now + Duration::from_millis(170)) > 0.4);
    assert!(fade.is_done(now + motion::FASTER + motion::FAST));
    // Cells sit on whole DIPs: the key of a cell is the same however it was computed.
    assert_eq!(cell_key(159.999, 96.001), cell_key(160.0, 96.0));
    assert_ne!(cell_key(160.0, 96.0), cell_key(240.0, 96.0));
}

/// Selection / focus survive a refresh by item identity: kept ids follow their items to
/// the new slots, vanished ids drop out, an anchor whose item is gone becomes None.
#[test]
fn index_state_remaps_by_item_id_across_a_refresh() {
    let ids: Vec<ItemId> = (0..5).map(|_| ItemId::new_v4()).collect();
    // New order: reversed, with ids[1] removed and a new item appended.
    let new_list = [ids[4], ids[3], ids[2], ids[0], ItemId::new_v4()];
    let new: HashMap<ItemId, usize> = new_list
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect();
    assert_eq!(remap_index(&ids, &new, 0), Some(3));
    assert_eq!(remap_index(&ids, &new, 4), Some(0));
    assert_eq!(remap_index(&ids, &new, 1), None, "removed item");
    assert_eq!(remap_index(&ids, &new, 9), None, "index past the old list");
    let sel: HashSet<usize> = [0, 1, 2].into_iter().collect();
    let remapped = remap_indices(&ids, &new, &sel);
    assert_eq!(remapped, [3usize, 2].into_iter().collect::<HashSet<_>>());
    // A wholesale replacement (tab switch, portal navigation) keeps nothing.
    let other: HashMap<ItemId, usize> = [(ItemId::new_v4(), 0)].into_iter().collect();
    assert!(remap_indices(&ids, &other, &sel).is_empty());
}

/// After the user's own Delete the focus goes to the item that followed the focused one (the
/// one before it at the end of the list), never to another item of the same Delete.
#[test]
fn focus_moves_to_the_neighbour_after_a_delete() {
    let ids: Vec<ItemId> = (0..5).map(|_| ItemId::new_v4()).collect();
    let new_of = |left: &[usize]| -> HashMap<ItemId, usize> {
        left.iter().enumerate().map(|(n, &i)| (ids[i], n)).collect()
    };
    let del = |gone: &[usize]| -> HashSet<ItemId> { gone.iter().map(|&i| ids[i]).collect() };
    // One item in the middle: the next one moves into its slot and takes the focus.
    assert_eq!(
        focus_after_delete(&ids, 2, &new_of(&[0, 1, 3, 4]), &del(&[2])),
        Some(2)
    );
    // The last item: the one before it.
    assert_eq!(
        focus_after_delete(&ids, 4, &new_of(&[0, 1, 2, 3]), &del(&[4])),
        Some(3)
    );
    // Three selected, the focused one leaves first: the others of the same Delete are skipped.
    assert_eq!(
        focus_after_delete(&ids, 2, &new_of(&[0, 1, 3, 4]), &del(&[1, 2, 3])),
        Some(3),
        "ids[4], now at index 3"
    );
    // Everything after it is being deleted too: back to the one before.
    assert_eq!(
        focus_after_delete(&ids, 3, &new_of(&[0, 1, 2, 4]), &del(&[3, 4])),
        Some(2)
    );
    // The only item: nothing to focus.
    assert_eq!(
        focus_after_delete(&ids[..1], 0, &new_of(&[]), &del(&[0])),
        None
    );
}

/// Scroll offsets rest on whole device pixels at every common scale (round, not floor,
/// so a thumb drag cannot drift).
#[test]
fn scroll_offset_snaps_to_device_pixels() {
    for scale in [1.0f32, 1.25, 1.5, 1.75, 2.0] {
        let snapped = snap_offset(123.456, scale);
        let px = snapped * scale;
        assert!((px - px.round()).abs() < 1e-3, "scale {scale}: {px}");
        assert!((snapped - 123.456).abs() <= 0.5 / scale + 1e-3);
    }
    assert_eq!(snap_offset(10.3, 1.0), 10.0);
    assert_eq!(snap_offset(10.6, 1.0), 11.0);
    assert_eq!(snap_offset(10.3, 2.0), 10.5);
    assert_eq!(snap_offset(0.0, 1.5), 0.0);
}

/// TME_HOVER box: the infotip delay restarts only once the pointer leaves the
/// SM_C[XY]MOUSEHOVER rectangle centred on where it was armed.
#[test]
fn tip_rest_box_is_centred_on_the_anchor() {
    let rect = (4, 4);
    assert!(!tip_hover_moved((10, 10), (10, 10), rect));
    assert!(!tip_hover_moved((10, 10), (11, 11), rect));
    assert!(!tip_hover_moved((10, 10), (9, 9), rect));
    assert!(tip_hover_moved((10, 10), (12, 10), rect));
    assert!(tip_hover_moved((10, 10), (10, 8), rect));
    assert!(tip_hover_moved((10, 10), (8, 10), rect));
    // A degenerate metric still needs a real move.
    assert!(!tip_hover_moved((10, 10), (10, 10), (1, 1)));
    assert!(tip_hover_moved((10, 10), (11, 10), (1, 1)));
}

#[test]
fn folder_drop_filter_rejects_self_parent_and_ancestors() {
    let folder = Path::new(r"C:\Users\Me\Desktop\Docs");
    let paths = vec![
        PathBuf::from(r"C:\Users\Me\Desktop\Docs\inside.txt"), // already in the folder
        PathBuf::from(r"C:\Users\Me\Desktop\docs"),            // the folder itself (case)
        PathBuf::from(r"C:\Users\Me\Desktop"),                 // an ancestor of the folder
        PathBuf::from(r"C:\Users\Me\Desktop\other.txt"),       // a sibling: allowed
        PathBuf::from(r"C:\Users\Me\Desktop\Documents"),       // prefix but not ancestor
    ];
    let kept = filter_folder_paths(paths, folder);
    assert_eq!(
        kept,
        vec![
            PathBuf::from(r"C:\Users\Me\Desktop\other.txt"),
            PathBuf::from(r"C:\Users\Me\Desktop\Documents"),
        ]
    );
}

/// A rolled-up fence ignores the keys that act on its (hidden) items and keeps the others.
#[test]
fn rolled_fence_ignores_only_the_item_keys() {
    // Arrows, Home, End, PageUp, PageDown, Enter, F2, Delete: with or without Ctrl.
    for vk in [0x25, 0x26, 0x27, 0x28, 0x24, 0x23, 0x21, 0x22] {
        assert!(acts_on_items(vk, false), "{vk:#x}");
        assert!(acts_on_items(vk, true), "{vk:#x} with Ctrl");
    }
    for vk in [msg::VK_RETURN, msg::VK_F2, msg::VK_DELETE] {
        assert!(acts_on_items(vk, false), "{vk:#x}");
    }
    // Ctrl+A, Ctrl+C, Ctrl+D, Ctrl+X, Ctrl+Space; the bare letters are type-ahead.
    for vk in [0x41, 0x43, 0x44, 0x58, 0x20] {
        assert!(acts_on_items(vk, true), "Ctrl+{vk:#x}");
        assert!(!acts_on_items(vk, false), "{vk:#x}");
    }
    // Tab, Ctrl+Tab, Ctrl+Z, Ctrl+V, Ctrl+F, Ctrl+N, F5, Esc, Backspace stay.
    for (vk, ctrl) in [
        (0x09, false),
        (0x09, true),
        (0x5A, true),
        (0x56, true),
        (0x46, true),
        (0x4E, true),
        (0x74, false),
        (msg::VK_ESCAPE, false),
        (0x08, false),
    ] {
        assert!(!acts_on_items(vk, ctrl), "{vk:#x} ctrl={ctrl}");
    }
}

#[test]
fn cursor_mode_and_tip_delay_tables() {
    assert_eq!(cursor_mode(false, false), CursorMode::Select);
    assert_eq!(cursor_mode(true, false), CursorMode::Extend);
    assert_eq!(cursor_mode(false, true), CursorMode::FocusOnly);
    // Shift wins over Ctrl (Ctrl+Shift+Arrow extends).
    assert_eq!(cursor_mode(true, true), CursorMode::Extend);

    // Infotip: full delay at first, TTDT_RESHOW when a tip was hidden within one hover
    // delay, full delay again after that window.
    assert_eq!(tip_delay_ms(400, 100, None), 400);
    assert_eq!(tip_delay_ms(400, 100, Some(Duration::from_millis(50))), 100);
    assert_eq!(
        tip_delay_ms(400, 100, Some(Duration::from_millis(399))),
        100
    );
    assert_eq!(
        tip_delay_ms(400, 100, Some(Duration::from_millis(400))),
        400
    );
    assert_eq!(tip_delay_ms(400, 100, Some(Duration::from_secs(3))), 400);
}

/// Track-repeat stop rule: the thumb placed for the glide's *destination* covers the
/// pointer, so the same hit test that started paging now reports Thumb and paging stops
/// even though the animated thumb has not arrived yet.
#[test]
fn track_repeat_stops_when_destination_thumb_reaches_pointer() {
    let (view_h, content_h) = (200.0f32, 1000.0f32);
    let max_scroll = content_h - view_h;
    let (track_top, track_h) = scrollbar_track(0.0, view_h);
    let thumb_h = scrollbar_thumb_h(track_h, view_h, content_h);
    let thumb_y = |scroll: f32| track_top + (track_h - thumb_h) * (scroll / max_scroll);
    let hit = |y: f32, scroll: f32| {
        if y < thumb_y(scroll) {
            ScrollHit::TrackAbove
        } else if y > thumb_y(scroll) + thumb_h {
            ScrollHit::TrackBelow
        } else {
            ScrollHit::Thumb
        }
    };
    let pointer_y = 150.0;
    assert_eq!(hit(pointer_y, 0.0), ScrollHit::TrackBelow);
    // Animated thumb still near the top, destination one page down: keep paging.
    assert_eq!(hit(pointer_y, view_h), ScrollHit::TrackBelow);
    // Destination three pages down: the thumb will sit under the pointer, stop.
    assert_eq!(hit(pointer_y, 3.0 * view_h), ScrollHit::Thumb);
    // Overshooting the pointer flips the side: stop as well.
    assert_eq!(hit(pointer_y, max_scroll), ScrollHit::TrackAbove);
}

/// WinUI ScrollBar geometry: a 12 DIP RepeatButton at each end of the band, the thumb
/// travelling between them and never shorter than 30 DIP; the buttons step a row, the
/// track a viewport.
#[test]
fn scrollbar_reserves_arrow_buttons_and_min_thumb() {
    let (track_top, track_h) = scrollbar_track(20.0, 200.0);
    assert_eq!(track_top, 32.0);
    assert_eq!(track_h, 176.0);
    assert_eq!(scrollbar_thumb_h(176.0, 200.0, 1000.0), 35.2);
    assert_eq!(scrollbar_thumb_h(176.0, 200.0, 5000.0), SCROLLBAR_THUMB_MIN);
    assert_eq!(scrollbar_thumb_h(20.0, 200.0, 5000.0), 20.0);
    // A band too short for two buttons still yields a usable track.
    assert_eq!(scrollbar_track(0.0, 10.0).1, 1.0);
    assert_eq!(scroll_step_for(ScrollHit::ArrowUp, 200.0, 76.0), -76.0);
    assert_eq!(scroll_step_for(ScrollHit::ArrowDown, 200.0, 76.0), 76.0);
    assert_eq!(scroll_step_for(ScrollHit::TrackAbove, 200.0, 76.0), -200.0);
    assert_eq!(scroll_step_for(ScrollHit::TrackBelow, 200.0, 76.0), 200.0);
    assert_eq!(scroll_step_for(ScrollHit::Thumb, 200.0, 76.0), 0.0);
}

/// Dragging the thumb by the free track length must scroll exactly to the end (the inverse
/// of the thumb placement in `scrollbar_geometry`).
#[test]
fn scrollbar_drag_inverse_reaches_max_scroll() {
    let (view_h, content_h) = (200.0f32, 1000.0f32);
    let max_scroll = content_h - view_h;
    let (track_top, track_h) = scrollbar_track(0.0, view_h);
    let thumb_h = scrollbar_thumb_h(track_h, view_h, content_h);
    let thumb_y = |scroll: f32| track_top + (track_h - thumb_h) * (scroll / max_scroll);
    let dy = thumb_y(max_scroll) - thumb_y(0.0);
    let scrolled = dy * (content_h - view_h) / (track_h - thumb_h);
    assert!((scrolled - max_scroll).abs() < 1e-3);
}

fn rc(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
    RECT {
        left,
        top,
        right,
        bottom,
    }
}

/// A fence dragged near the middle of the work area snaps to be centred on it, and the centre
/// guides span the work area; far from the middle it stays where the pointer put it.
#[test]
fn drag_snaps_to_the_work_area_centre_lines() {
    let work = rc(0, 0, 3840, 2064);
    let mut r = rc(1600, 900, 2241, 1161); // 641 x 261: centred across, 2 px above the middle
    snap_among(&mut r, &[], Some(&work), 16, 20);
    assert_eq!((r.left, r.top), (1920 - 641 / 2, 1032 - 261 / 2));
    let guides = alignment_guides(&r, &[], Some(&work));
    assert_eq!(
        guides,
        vec![
            GuideLine {
                vertical: true,
                at: 1920,
                from: 0,
                to: 2064,
            },
            GuideLine {
                vertical: false,
                at: 1032,
                from: 0,
                to: 3840,
            },
        ]
    );
    let mut far = rc(1000, 300, 1641, 561);
    snap_among(&mut far, &[], Some(&work), 16, 20);
    assert_eq!((far.left, far.top), (1000, 300));
    assert!(alignment_guides(&far, &[], Some(&work)).is_empty());
}

/// Edges level with another fence get one guide spanning both fences (a second fence on the
/// same line extends it), however far apart they are (feedback #100); a gap-apart neighbour's
/// facing edge is no guide.
#[test]
fn guides_join_level_edges_of_nearby_and_far_fences() {
    let above = rc(100, 100, 500, 400);
    let below_far = rc(100, 1500, 300, 1600);
    let beside = rc(708, 416, 900, 700);
    // Stacked a gap below `above`, same left edge; `beside` is a gap to the right, same top.
    let me = rc(100, 416, 692, 700);
    let guides = alignment_guides(&me, &[above, below_far, beside], None);
    assert!(guides.contains(&GuideLine {
        vertical: true,
        at: 100,
        from: 100,
        to: 1600,
    }));
    assert!(guides.contains(&GuideLine {
        vertical: false,
        at: 416,
        from: 100,
        to: 900,
    }));
    assert!(guides.contains(&GuideLine {
        vertical: false,
        at: 700,
        from: 100,
        to: 900,
    }));
    assert_eq!(guides.len(), 3, "{guides:?}");
    // A second stacked fence on the same left edge lengthens the same guide.
    let under = rc(100, 716, 400, 900);
    let guides = alignment_guides(&me, &[above, under], None);
    assert_eq!(
        guides
            .iter()
            .filter(|g| g.vertical && g.at == 100)
            .collect::<Vec<_>>(),
        vec![&GuideLine {
            vertical: true,
            at: 100,
            from: 100,
            to: 900,
        }]
    );
}

/// Of two pulls within reach the nearer wins: a neighbour's edge or the work-area centre line.
#[test]
fn nearest_pull_wins_between_fences_and_centre_lines() {
    let work = rc(0, 0, 2000, 1400);
    let other = rc(100, 100, 1153, 400);
    // Stacked just below `other`, 301 wide: same right is at 852, centred at 850.
    let mut r = rc(853, 405, 1154, 600);
    snap_among(&mut r, &[other], Some(&work), 8, 20);
    assert_eq!(
        (r.left, r.right, r.top),
        (852, 1153, 408),
        "same right beats the centre line; the gap below wins vertically"
    );
    let mut r = rc(849, 405, 1150, 600);
    snap_among(&mut r, &[other], Some(&work), 8, 20);
    assert_eq!(
        (r.left, r.top),
        (850, 408),
        "the centre line beats same right"
    );
}

/// A fence dropped beside (or under) another, 3 to 15 px off the 8 px gap and 5 px off level,
/// gets the gap and the same top (left) edge together: the gap counts towards being across
/// (the review's sweep: 12 and 15 px stayed 5 px off).
#[test]
fn a_fence_dropped_a_gap_beside_another_lines_up_with_it() {
    let work = rc(0, 0, 1600, 860);
    let other = rc(420, 120, 720, 370);
    for pre_gap in [3, 6, 9, 12, 15] {
        let left = 720 + pre_gap;
        let mut beside = rc(left, 125, left + 240, 325);
        snap_among(&mut beside, &[other], Some(&work), 8, 10);
        assert_eq!(
            (beside.left, beside.top),
            (728, 120),
            "beside, pre-gap {pre_gap}"
        );
        let top = 370 + pre_gap;
        let mut under = rc(425, top, 665, top + 200);
        snap_among(&mut under, &[other], Some(&work), 8, 10);
        assert_eq!(
            (under.left, under.top),
            (420, 378),
            "under, pre-gap {pre_gap}"
        );
    }
}

/// Two fences at the two ends of the screen: a top within the snap distance of the other's
/// becomes level with it and a guide joins them; farther off nothing pulls, and no gap is
/// kept to a fence that far away (feedback #100).
#[test]
fn far_fences_in_one_row_line_up_and_show_a_guide() {
    let far = rc(100, 120, 400, 370);
    for off in [2, 4, 6, -6] {
        let mut r = rc(1000, 120 + off, 1240, 320 + off);
        snap_among(&mut r, &[far], None, 8, 10);
        assert_eq!((r.left, r.top), (1000, 120), "{off} px off");
        assert_eq!(
            alignment_guides(&r, &[far], None),
            vec![GuideLine {
                vertical: false,
                at: 120,
                from: 100,
                to: 1240,
            }]
        );
    }
    let mut r = rc(1000, 150, 1240, 350);
    snap_among(&mut r, &[far], None, 8, 10);
    assert_eq!((r.left, r.top), (1000, 150), "30 px off stays");
    assert!(alignment_guides(&r, &[far], None).is_empty());
    // Its left edge 4 px right of the far fence's right + gap: no pull from across the screen.
    let mut r = rc(412, 600, 652, 800);
    snap_among(&mut r, &[far], None, 8, 10);
    assert_eq!(r.left, 412);
}

/// 48 px icons (80 x 96 DIP cells) at 100 %: 36 DIP title, 8 DIP pads, whole columns and rows.
fn cell_rules() -> SizeRules {
    SizeRules {
        cols: Some(Cells {
            fixed: 0.0,
            step: 80.0,
        }),
        rows: Some(Cells {
            fixed: 54.0,
            step: 96.0,
        }),
        min_w: 80,
        min_h: 150,
        rolled_h: None,
        fit_h: None,
    }
}

fn neighbours<'a>(others: &'a [RECT], work: Option<&'a RECT>, align: bool) -> Neighbours<'a> {
    Neighbours {
        others,
        work,
        gap: 8,
        dist: 10,
        align,
    }
}

const BOTTOM: SizingEdges = SizingEdges {
    left: false,
    right: false,
    top: false,
    bottom: true,
};

/// Resizing with whole cells: a bottom edge near another fence's bottom lines up with it even
/// though that fence's rows are another height (its guide shows); away from every fence, or
/// with Alt held, the height takes whole rows.
#[test]
fn resized_bottom_lines_up_with_fences_of_other_icon_sizes() {
    let cur = rc(100, 100, 420, 442); // 3 rows: 54 + 3 * 96
    let beside = rc(428, 100, 700, 531); // a fence with other rows: 531 is not on our ladder
    let rules = cell_rules();
    let mut r = rc(100, 100, 420, 526);
    let guides = size_among(
        &mut r,
        &cur,
        BOTTOM,
        &rules,
        Some(&neighbours(&[beside], None, true)),
    );
    assert_eq!(r.bottom, 531);
    assert_eq!(
        guides,
        vec![GuideLine {
            vertical: false,
            at: 531,
            from: 100,
            to: 700,
        }]
    );
    // Alt held: whole rows (4 rows end at 538), no guide.
    let mut r = rc(100, 100, 420, 526);
    let guides = size_among(
        &mut r,
        &cur,
        BOTTOM,
        &rules,
        Some(&neighbours(&[beside], None, false)),
    );
    assert_eq!(r.bottom, 538);
    assert!(guides.is_empty());
    // Out of reach: whole rows too.
    let mut r = rc(100, 100, 420, 515);
    size_among(
        &mut r,
        &cur,
        BOTTOM,
        &rules,
        Some(&neighbours(&[beside], None, true)),
    );
    assert_eq!(r.bottom, 538);
}

/// Towards a fence below, the edge stops exactly a gap above it (a target within reach, the
/// limit beyond), not a whole row short; the work area's edge a gap inside is a target too.
/// Snapping off, only whole rows apply.
#[test]
fn resized_bottom_stops_a_gap_above_the_fence_below() {
    let cur = rc(100, 100, 420, 442);
    let below = rc(100, 700, 420, 900);
    let work = rc(0, 0, 1920, 1040);
    let rules = cell_rules();
    for pointer in [686, 850] {
        let mut r = rc(100, 100, 420, pointer);
        size_among(
            &mut r,
            &cur,
            BOTTOM,
            &rules,
            Some(&neighbours(&[below], Some(&work), true)),
        );
        assert_eq!(r.bottom, 692, "pointer at {pointer}");
    }
    let mut r = rc(100, 100, 420, 1025);
    size_among(
        &mut r,
        &cur,
        BOTTOM,
        &rules,
        Some(&neighbours(&[], Some(&work), true)),
    );
    assert_eq!(r.bottom, 1032);
    let mut r = rc(100, 100, 420, 850);
    assert!(size_among(&mut r, &cur, BOTTOM, &rules, None).is_empty());
    assert_eq!(r.bottom, 826, "7 whole rows, no neighbour stop");
}

/// A side edge lines up with a stacked fence's side edge (guide along it) and otherwise takes
/// whole columns; the height it does not touch stays as it is, whole rows or not.
#[test]
fn resized_side_edges_line_up_and_leave_the_height() {
    let cur = rc(100, 100, 420, 431); // not whole rows: kept
    let stacked = rc(150, 600, 437, 680);
    let under_left = rc(40, 500, 300, 560);
    let right = SizingEdges {
        right: true,
        ..SizingEdges::default()
    };
    let rules = cell_rules();
    let mut r = rc(100, 100, 433, 431);
    let guides = size_among(
        &mut r,
        &cur,
        right,
        &rules,
        Some(&neighbours(&[stacked], None, true)),
    );
    assert_eq!((r.right, r.bottom), (437, 431));
    assert_eq!(
        guides,
        vec![GuideLine {
            vertical: true,
            at: 437,
            from: 100,
            to: 680,
        }]
    );
    let mut r = rc(100, 100, 470, 431);
    size_among(
        &mut r,
        &cur,
        right,
        &rules,
        Some(&neighbours(&[stacked], None, true)),
    );
    assert_eq!(r.right, 500, "5 whole columns");
    let left = SizingEdges {
        left: true,
        ..SizingEdges::default()
    };
    let mut r = rc(45, 100, 420, 431);
    size_among(
        &mut r,
        &cur,
        left,
        &rules,
        Some(&neighbours(&[under_left], None, true)),
    );
    assert_eq!(r.left, 40);
}

/// The top edge lines up a gap below a fence above; a target leaving less than one row is
/// ignored; a rolled fence stays one title bar tall whatever edge moves.
#[test]
fn resized_top_edge_minimum_and_rolled_fences() {
    let cur = rc(100, 100, 420, 442);
    let above = rc(100, 0, 420, 60);
    let top = SizingEdges {
        top: true,
        ..SizingEdges::default()
    };
    let rules = cell_rules();
    let mut r = rc(100, 72, 420, 442);
    size_among(
        &mut r,
        &cur,
        top,
        &rules,
        Some(&neighbours(&[above], None, true)),
    );
    assert_eq!(r.top, 68);
    let short = rc(500, 150, 700, 245);
    let mut r = rc(100, 100, 420, 248);
    size_among(
        &mut r,
        &cur,
        BOTTOM,
        &rules,
        Some(&neighbours(&[short], None, true)),
    );
    assert_eq!(r.bottom, 250, "245 would leave less than one row");
    let rolled = SizeRules {
        rolled_h: Some(36),
        ..cell_rules()
    };
    let corner = SizingEdges {
        right: true,
        bottom: true,
        ..SizingEdges::default()
    };
    let mut r = rc(100, 100, 470, 500);
    size_among(&mut r, &rc(100, 100, 420, 136), corner, &rolled, None);
    assert_eq!((r.right, r.bottom), (500, 136));
    assert_eq!(SizingEdges::from_wmsz(8), corner);
    assert_eq!(
        SizingEdges::from_wmsz(4),
        SizingEdges {
            left: true,
            top: true,
            ..SizingEdges::default()
        }
    );
    assert_eq!(SizingEdges::from_wmsz(3), top);
}

/// A group drag keeps every member's offset to the dragged fence; with snapping the group's
/// bounding box (not the dragged fence alone) lines up with a fence outside the group.
#[test]
fn group_drag_keeps_offsets_and_snaps_its_bounding_box() {
    let start = rc(100, 100, 300, 300);
    let member = GroupMember {
        hwnd: HWND::default(),
        fence: FenceId::new_v4(),
        start: rc(320, 100, 520, 260),
    };
    let work = rc(-5000, -5000, 5000, 5000);
    let mut r = rc(150, 150, 350, 350);
    let (placed, guides) = place_group(&mut r, &start, &[member], &[], |_| Some(work), None);
    assert_eq!(r, rc(150, 150, 350, 350));
    assert_eq!(placed, vec![rc(370, 150, 570, 310)]);
    assert!(guides.is_empty());
    // The box (170..590) ends 10 px short of a gap before the fence at 600: it moves 2 px
    // right as a whole, and its top joins that fence's top.
    let other = rc(600, 0, 800, 400);
    let mut r = rc(170, 5, 370, 205);
    let (placed, guides) = place_group(
        &mut r,
        &start,
        &[member],
        &[other],
        |_| Some(work),
        Some((8, 20)),
    );
    assert_eq!(r, rc(172, 0, 372, 200));
    assert_eq!(placed, vec![rc(392, 0, 592, 160)]);
    assert!(guides.iter().any(|g| !g.vertical && g.at == 0));
}

/// One tab per notch over the strip: towards the user = the tab to the right, away = the tab
/// to the left, several notches at once move several tabs, and the ends hold (no wrap).
#[test]
fn wheel_over_tabs_steps_and_stops_at_the_ends() {
    assert_eq!(wheel_tab_target(1, 4, -1), 2);
    assert_eq!(wheel_tab_target(1, 4, 1), 0);
    assert_eq!(wheel_tab_target(0, 4, -3), 3);
    assert_eq!(wheel_tab_target(0, 4, 1), 0);
    assert_eq!(wheel_tab_target(3, 4, -1), 3);
    assert_eq!(wheel_tab_target(3, 4, -9), 3);
    assert_eq!(wheel_tab_target(2, 4, 0), 2);
}

/// Hover switching aims at another tab only: not the active one, not with the setting off,
/// a single tab, a title rename open, or no tab under the pointer.
#[test]
fn hover_switches_only_to_another_tab() {
    assert_eq!(hover_switch_target(Some(2), 0, 3, true, false), Some(2));
    assert_eq!(hover_switch_target(Some(0), 0, 3, true, false), None);
    assert_eq!(hover_switch_target(Some(2), 0, 3, false, false), None);
    assert_eq!(hover_switch_target(Some(2), 0, 3, true, true), None);
    assert_eq!(hover_switch_target(Some(0), 1, 1, true, false), None);
    assert_eq!(hover_switch_target(Some(5), 0, 3, true, false), None);
    assert_eq!(hover_switch_target(None, 0, 3, true, false), None);
}

/// A 300 x 400 px window at 100 %: 6 px edges, corners that reach 16 px along each edge.
fn edge(left: i32, top: i32) -> Option<isize> {
    resize_edge_at((left, top, 300 - left, 400 - top), (300, 400), 6, 6, 16)
}

/// The diagonal resize answers along 16 px of both edges of a corner, as a normal window's
/// does, not only in the 6 x 6 square where the two edges cross.
#[test]
fn corners_reach_sixteen_px_along_each_edge() {
    // Bottom edge, walking in from the right corner (`right` = 300 - x, corner while <= 16).
    assert_eq!(edge(297, 397), Some(msg::HTBOTTOMRIGHT));
    assert_eq!(edge(290, 397), Some(msg::HTBOTTOMRIGHT));
    assert_eq!(edge(284, 397), Some(msg::HTBOTTOMRIGHT));
    assert_eq!(edge(283, 397), Some(msg::HTBOTTOM));
    // Right edge, walking up from the bottom corner.
    assert_eq!(edge(297, 384), Some(msg::HTBOTTOMRIGHT));
    assert_eq!(edge(297, 383), Some(msg::HTRIGHT));
    // The other three corners.
    assert_eq!(edge(2, 15), Some(msg::HTTOPLEFT));
    assert_eq!(edge(15, 2), Some(msg::HTTOPLEFT));
    assert_eq!(edge(2, 16), Some(msg::HTLEFT));
    assert_eq!(edge(16, 2), Some(msg::HTTOP));
    assert_eq!(edge(297, 10), Some(msg::HTTOPRIGHT));
    assert_eq!(edge(10, 397), Some(msg::HTBOTTOMLEFT));
    // Inside the corner's square but off both edges: no resize.
    assert_eq!(edge(290, 390), None);
    assert_eq!(edge(150, 200), None);
}

/// Beside a scrollbar the right edge is 3 px deep: the bar keeps its strip right down to the
/// bottom edge, whose own depth still gives the corner.
#[test]
fn corner_beside_a_scrollbar_keeps_the_bars_strip() {
    let at = |left: i32, top: i32| {
        resize_edge_at((left, top, 300 - left, 400 - top), (300, 400), 6, 3, 16)
    };
    assert_eq!(at(295, 390), None);
    assert_eq!(at(298, 390), Some(msg::HTBOTTOMRIGHT));
    assert_eq!(at(298, 300), Some(msg::HTRIGHT));
    assert_eq!(at(295, 396), Some(msg::HTBOTTOMRIGHT));
}

/// A corner takes at most a third of a short edge, so a one-column fence keeps a straight
/// bottom edge between its corners.
#[test]
fn corners_of_a_narrow_window_leave_the_edge_between_them() {
    let at = |left: i32| resize_edge_at((left, 397, 80 - left, 3), (80, 400), 6, 6, 16);
    assert_eq!(at(10), Some(msg::HTBOTTOMLEFT));
    assert_eq!(at(40), Some(msg::HTBOTTOM));
    assert_eq!(at(70), Some(msg::HTBOTTOMRIGHT));
    let tiny = |left: i32| resize_edge_at((left, 397, 30 - left, 3), (30, 400), 6, 6, 16);
    assert_eq!(tiny(15), Some(msg::HTBOTTOM));
}

/// The snap gap is rounded the same way wherever fences are lined up: a gap of 3 DIPs at 150 %
/// is 5 px (4.5 rounded), where a move used to cut it to 4 and a resize used 5.
#[test]
fn snap_gap_is_rounded_at_fractional_scales() {
    assert_eq!(snap_gap_px(8, 1.0), 8);
    assert_eq!(snap_gap_px(8, 1.25), 10);
    assert_eq!(snap_gap_px(3, 1.5), 5);
    assert_eq!(snap_gap_px(2, 1.25), 3);
    assert_eq!(snap_gap_px(6, 1.75), 11);
    assert_eq!(snap_gap_px(0, 1.5), 0);
}

/// The lock glyph follows its hover fade, except while the lock has just refused a drag: then
/// it shows at full strength, dips to 0.4 halfway through the 250 ms pulse and comes back.
#[test]
fn lock_glyph_pulses_once_when_the_lock_refuses_a_drag() {
    let ms = |n: u64| Some(Duration::from_millis(n));
    assert_eq!(lock_alpha(0.0, None), 0.0);
    assert_eq!(lock_alpha(0.7, None), 0.7);
    // The pulse wins over a hover fade that has not finished.
    assert!((lock_alpha(0.2, ms(0)) - 1.0).abs() < 1e-4);
    assert!((lock_alpha(1.0, ms(125)) - LOCK_PULSE_LOW).abs() < 1e-3);
    assert!(lock_alpha(1.0, ms(60)) < 1.0 && lock_alpha(1.0, ms(60)) > LOCK_PULSE_LOW);
    assert!(lock_alpha(1.0, ms(200)) > lock_alpha(1.0, ms(125)));
    // Over: back on the hover fade.
    assert_eq!(lock_alpha(1.0, ms(250)), 1.0);
    assert_eq!(lock_alpha(0.0, ms(400)), 0.0);
}

/// While an edge is dragged the shadow gets a new bitmap as often as its cost allows: every
/// frame for a cheap one, the old 40 ms for a very large fence, never more than a sixth of the
/// time.
#[test]
fn shadow_keeps_up_with_a_dragged_edge_when_it_is_cheap() {
    let us = Duration::from_micros;
    assert_eq!(shadow_interval_ms(us(300)), 15);
    assert_eq!(shadow_interval_ms(us(2_500)), 15);
    assert_eq!(shadow_interval_ms(us(4_000)), 24);
    assert_eq!(shadow_interval_ms(us(7_000)), 40);
    assert_eq!(shadow_interval_ms(Duration::from_millis(40)), 40);
    assert_eq!(shadow_interval_ms(Duration::ZERO), 15);
}
