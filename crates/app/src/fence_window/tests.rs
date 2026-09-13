use super::*;

#[test]
fn window_drag_keeps_the_latest_sample_and_release_before_first_frame() {
    let mut drag = RemoteDrag {
        hwnd: HWND::default(),
        fence: FenceId::new_v4(),
        offset: (20, 10),
        merge_target: 0,
        merge_x: i32::MIN,
        moved_once: false,
        origin: WindowDragOrigin::Caption {
            rect: RECT::default(),
            click_expand: false,
        },
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
/// Right-drag menu offers only what both the source and this target can do, in Explorer's
/// order.
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
        effect_for(dragdrop::MK_ALT, DropEffect::Copy.to_raw()),
        DropEffect::Copy
    );
    assert_eq!(effect_for(0, dragdrop::ALL_EFFECTS), DropEffect::Move);
    assert_eq!(
        effect_for(
            dragdrop::MK_CONTROL | dragdrop::MK_SHIFT,
            dragdrop::ALL_EFFECTS
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
/// removes and snaps for refills (tab switch, portal navigation) or an empty start.
#[test]
fn layout_motion_applies_only_to_rearrangements() {
    assert!(layout_motion_applies(0, 10, 10));
    assert!(layout_motion_applies(1, 10, 11));
    assert!(layout_motion_applies(5, 10, 10));
    assert!(!layout_motion_applies(6, 10, 10));
    assert!(!layout_motion_applies(20, 10, 10));
    assert!(!layout_motion_applies(0, 0, 5));
    // Removing everything is a refill too.
    assert!(!layout_motion_applies(10, 10, 0));
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
