use super::dnd::url_shortcut_base_name;

#[test]
fn url_shortcut_names_prefer_the_browser_title() {
    assert_eq!(
        url_shortcut_base_name(Some("Rust: fast?.url"), "https://www.rust-lang.org/"),
        "Rust- fast-"
    );
    assert_eq!(
        url_shortcut_base_name(None, "https://example.com/docs/page.html?x=1"),
        "example.com - page.html"
    );
    assert_eq!(
        url_shortcut_base_name(None, "https://example.com"),
        "example.com"
    );
    assert_eq!(
        url_shortcut_base_name(Some("   "), "https://"),
        "新建 Internet 快捷方式"
    );
}

mod ipc {
    use super::super::fence_options::{FenceProp, parse_fence_prop};
    use super::super::ipc::{
        AUTO_SNAPSHOT_PREFIX, MAX_NAME_CHARS, MIN_EXPIRY_MS, auto_snapshot_slot,
        check_rule_conditions, checked_name, is_inbox_alias, patch_settings_path,
        prune_auto_snapshots, request_expiry, rolled_back_settings, validate_rect,
    };
    use pecofence_core::geometry::WorkArea;
    use pecofence_core::{Cond, MAX_SNAPSHOTS, Settings, Snapshot, ViewLayout};
    use pecofence_ipc::{ErrorCode, Rect};
    use serde_json::json;
    use std::time::Duration;

    fn work(device: &str, left: i32, top: i32, w: i32, h: i32, dpi: u32) -> WorkArea {
        WorkArea {
            device_path: device.into(),
            left,
            top,
            right: left + w,
            bottom: top + h,
            dpi,
            mon_left: left,
            mon_top: top,
            mon_right: left + w,
            mon_bottom: top + h + 40,
        }
    }

    fn two_monitors() -> Vec<WorkArea> {
        vec![
            work("primary", 0, 0, 1920, 1040, 96),
            // A 150 % monitor to the left, taller than the primary.
            work("left", -2560, -200, 2560, 1400, 144),
        ]
    }

    fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn rect_validation_accepts_rects_centred_on_a_monitor() {
        let areas = two_monitors();
        assert_eq!(
            validate_rect(rect(100, 100, 300, 200), &areas)
                .unwrap()
                .device_path,
            "primary"
        );
        // Hanging off the edge is fine as long as the centre is on screen.
        assert_eq!(
            validate_rect(rect(-2700, 0, 400, 300), &areas)
                .unwrap()
                .device_path,
            "left"
        );
        // Minimum size scales with the monitor's DPI: 64x36 DIP is 96x54 px at 150 %.
        assert!(validate_rect(rect(-1000, 500, 64, 36), &areas).is_err());
        assert!(validate_rect(rect(-1000, 500, 96, 54), &areas).is_ok());
        assert!(validate_rect(rect(500, 500, 64, 36), &areas).is_ok());
    }

    #[test]
    fn rect_validation_rejects_overflow_offscreen_and_bad_sizes() {
        let areas = two_monitors();
        let err = validate_rect(rect(i32::MAX - 10, 0, 300, 200), &areas).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(err.message.contains("overflow"), "{}", err.message);
        let err = validate_rect(rect(0, i32::MAX, 300, 200), &areas).unwrap_err();
        assert!(err.message.contains("overflow"), "{}", err.message);

        let err = validate_rect(rect(0, 0, 0, 200), &areas).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(err.message.contains("positive"), "{}", err.message);
        assert!(validate_rect(rect(0, 0, 300, -1), &areas).is_err());

        // Centre below both monitors' work areas (on the primary's taskbar strip).
        let err = validate_rect(rect(100, 1030, 300, 200), &areas).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert_eq!(
            err.message,
            "rect centre is outside every monitor's work area"
        );
        let details = err.details.unwrap();
        assert_eq!(details["workAreas"].as_array().unwrap().len(), 2);
        assert_eq!(details["centre"], json!({ "x": 250, "y": 1130 }));
        // Far to the right of everything.
        assert!(validate_rect(rect(5000, 0, 300, 200), &areas).is_err());
        // Nothing connected.
        assert!(validate_rect(rect(0, 0, 300, 200), &[]).is_err());

        // Too small on the primary (36 px wide) and too large (over 8192 DIP).
        let err = validate_rect(rect(500, 500, 36, 100), &areas).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(
            err.message.contains("outside the allowed range"),
            "{}",
            err.message
        );
        assert!(validate_rect(rect(-4000, 500, 9000, 9000), &areas).is_err());
        // 8192 DIP at 150 % is 12288 px: allowed on the left monitor, whose work area holds
        // the centre (-1280, 500); the same size is over the limit on the 96 dpi primary.
        assert!(validate_rect(rect(-7424, -5644, 12288, 12288), &areas).is_ok());
        assert!(validate_rect(rect(-5184, -5624, 12288, 12288), &areas).is_err());
    }

    #[test]
    fn names_are_trimmed_and_length_limited() {
        assert_eq!(checked_name("title", "  Work  ").unwrap(), "Work");
        let err = checked_name("title", "   ").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(err.message.contains("title"), "{}", err.message);
        let just_fits = "字".repeat(MAX_NAME_CHARS);
        assert_eq!(checked_name("rule name", &just_fits).unwrap(), just_fits);
        let too_long = "x".repeat(MAX_NAME_CHARS + 1);
        let err = checked_name("snapshot name", &too_long).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(err.message.contains("257"), "{}", err.message);
        assert_eq!(err.details.unwrap()["max"], json!(MAX_NAME_CHARS));
        // Surrounding whitespace does not count.
        let padded = format!("  {}  ", "y".repeat(MAX_NAME_CHARS));
        assert!(checked_name("title", &padded).is_ok());
        // parse_fence_prop("title") goes through the same check.
        let err = parse_fence_prop("title", &json!(too_long)).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
    }

    #[test]
    fn tiny_timeouts_still_run_once() {
        assert_eq!(request_expiry(0), Duration::from_millis(MIN_EXPIRY_MS));
        assert_eq!(request_expiry(50), Duration::from_millis(MIN_EXPIRY_MS));
        assert_eq!(request_expiry(15_000), Duration::from_millis(15_000));
    }

    #[test]
    fn rules_need_a_condition() {
        let err = check_rule_conditions(&[]).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert!(
            err.message.contains("at least one condition"),
            "{}",
            err.message
        );
        assert!(check_rule_conditions(&[Cond::Ext(vec![".pdf".into()])]).is_ok());
    }

    #[test]
    fn inbox_alias_is_case_insensitive() {
        assert!(is_inbox_alias("inbox"));
        assert!(is_inbox_alias(" INBOX "));
        assert!(is_inbox_alias("Inbox"));
        assert!(!is_inbox_alias("inbox2"));
        assert!(!is_inbox_alias("桌面"));
    }

    #[test]
    fn rolled_back_settings_names_the_kept_keys() {
        // Asked to show the real icons (default: hidden), but Explorer refused.
        let requested = Settings {
            hide_real_icons: false,
            icon_size: 64,
            ..Default::default()
        };
        let applied = Settings {
            icon_size: 64,
            ..Default::default()
        };
        assert!(applied.hide_real_icons);
        assert_eq!(
            rolled_back_settings(&requested, &applied),
            vec!["hideRealIcons".to_string()]
        );
        assert!(rolled_back_settings(&applied, &applied).is_empty());
    }

    #[test]
    fn settings_patch_sets_a_nested_bool() {
        let current = Settings::default();
        assert!(current.peek.enabled);
        let patched = patch_settings_path(&current, "peek.enabled", json!(false)).unwrap();
        assert!(!patched.peek.enabled);
        // Everything else untouched.
        assert_eq!(patched.peek.dim, current.peek.dim);
        assert_eq!(patched.icon_size, current.icon_size);
    }

    #[test]
    fn settings_patch_rejects_unknown_keys_with_the_valid_ones() {
        let err = patch_settings_path(&Settings::default(), "peek.nope", json!(1)).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidPath);
        let allowed = err.details.unwrap()["allowed"].clone();
        assert!(
            allowed.as_array().unwrap().contains(&json!("enabled")),
            "{allowed}"
        );
        assert!(err.hint.unwrap().contains("peek"));

        let err = patch_settings_path(&Settings::default(), "nope", json!(1)).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidPath);
        assert!(err.hint.unwrap().starts_with("Top-level keys"));
    }

    #[test]
    fn settings_patch_rejects_wrong_types_and_ranges() {
        let err =
            patch_settings_path(&Settings::default(), "peek.enabled", json!("yes")).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
        assert!(
            err.details.unwrap()["expected"]
                .as_str()
                .unwrap()
                .contains("bool")
        );

        let err = patch_settings_path(&Settings::default(), "iconSize", json!(50)).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
        assert_eq!(err.details.unwrap()["expected"], json!([32, 48, 64, 96]));

        // Enum paths: the serde error names the variants.
        let err = patch_settings_path(&Settings::default(), "theme", json!("neon")).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
        assert!(err.message.contains("expected one of"), "{}", err.message);
    }

    #[test]
    fn settings_patch_replaces_the_whole_object() {
        let wanted = Settings {
            icon_size: 64,
            peek: pecofence_core::PeekSettings {
                dim: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let value = serde_json::to_value(&wanted).unwrap();
        let patched = patch_settings_path(&Settings::default(), "", value).unwrap();
        assert_eq!(patched, wanted);

        let err = patch_settings_path(&Settings::default(), "", json!(true)).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
        let err = patch_settings_path(&Settings::default(), "", json!({})).unwrap_err();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
    }

    #[test]
    fn fence_props_are_validated() {
        let err = parse_fence_prop("nope", &json!(1)).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        let allowed = err.details.unwrap()["allowed"].clone();
        for p in [
            "title",
            "iconSize",
            "layout",
            "labelLines",
            "portalTitleIcon",
        ] {
            assert!(allowed.as_array().unwrap().contains(&json!(p)), "{allowed}");
        }

        let err = parse_fence_prop("iconSize", &json!(50)).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert_eq!(
            err.details.unwrap()["allowed"],
            json!(["32", "48", "64", "96"])
        );
        assert_eq!(
            parse_fence_prop("iconSize", &json!(64)).unwrap(),
            FenceProp::IconSize(64)
        );

        assert_eq!(
            parse_fence_prop("layout", &json!("list")).unwrap(),
            FenceProp::Layout(ViewLayout::List)
        );
        assert_eq!(
            parse_fence_prop("tint", &json!("#ABCDEF")).unwrap(),
            FenceProp::Tint(Some([0xAB, 0xCD, 0xEF]))
        );
        assert_eq!(
            parse_fence_prop("tint", &serde_json::Value::Null).unwrap(),
            FenceProp::Tint(None)
        );
        let err = parse_fence_prop("tint", &json!("red")).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidValue);
        assert_eq!(
            parse_fence_prop("labelLines", &json!(4)).unwrap_err().code,
            ErrorCode::InvalidValue
        );
        assert_eq!(
            parse_fence_prop("locked", &json!("true")).unwrap_err().code,
            ErrorCode::InvalidValue
        );
        assert_eq!(
            parse_fence_prop("title", &json!("  ")).unwrap_err().code,
            ErrorCode::InvalidValue
        );
    }

    fn snap(name: &str, ts: i64) -> Snapshot {
        Snapshot {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            ts,
            layouts: Vec::new(),
        }
    }

    #[test]
    fn auto_snapshots_rotate_and_leave_user_snapshots_alone() {
        let mut list = vec![
            snap("mine", 1),
            snap(&format!("{AUTO_SNAPSHOT_PREFIX}a"), 2),
            snap(&format!("{AUTO_SNAPSHOT_PREFIX}b"), 3),
            snap("yours", 4),
            snap(&format!("{AUTO_SNAPSHOT_PREFIX}c"), 5),
            snap(&format!("{AUTO_SNAPSHOT_PREFIX}d"), 6),
        ];
        let oldest_auto = list[1].id;
        let evicted = prune_auto_snapshots(&mut list, 3);
        assert_eq!(evicted, vec![oldest_auto]);
        let names: Vec<&str> = list.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            ["mine", "auto-cli-b", "yours", "auto-cli-c", "auto-cli-d"]
        );
        assert!(prune_auto_snapshots(&mut list, 3).is_empty());
    }

    #[test]
    fn auto_snapshot_makes_room_by_pruning_its_own_kind_only() {
        // Full list: MAX-2 user snapshots plus two auto ones. Pruning to KEPT-1 (= 2) removes
        // nothing, so there is no room: no snapshot, no eviction.
        let mut list: Vec<Snapshot> = (0..MAX_SNAPSHOTS - 2)
            .map(|i| snap(&format!("mine {i}"), i as i64))
            .collect();
        list.push(snap(&format!("{AUTO_SNAPSHOT_PREFIX}a"), 100));
        list.push(snap(&format!("{AUTO_SNAPSHOT_PREFIX}b"), 101));
        let before: Vec<uuid::Uuid> = list.iter().map(|s| s.id).collect();
        assert!(!auto_snapshot_slot(&mut list));
        let after: Vec<uuid::Uuid> = list.iter().map(|s| s.id).collect();
        assert_eq!(after, before, "nothing may be evicted when no slot opens");

        // Three auto snapshots in a full list: the oldest goes, a slot opens, every user
        // snapshot survives.
        let mut list: Vec<Snapshot> = (0..MAX_SNAPSHOTS - 3)
            .map(|i| snap(&format!("mine {i}"), i as i64))
            .collect();
        let users: Vec<uuid::Uuid> = list.iter().map(|s| s.id).collect();
        list.push(snap(&format!("{AUTO_SNAPSHOT_PREFIX}a"), 100));
        let oldest_auto = list.last().unwrap().id;
        list.push(snap(&format!("{AUTO_SNAPSHOT_PREFIX}b"), 101));
        list.push(snap(&format!("{AUTO_SNAPSHOT_PREFIX}c"), 102));
        assert!(auto_snapshot_slot(&mut list));
        assert_eq!(list.len(), MAX_SNAPSHOTS - 1);
        assert!(list.iter().all(|s| s.id != oldest_auto));
        assert!(users.iter().all(|u| list.iter().any(|s| s.id == *u)));

        // Full list of user snapshots only: refused, untouched.
        let mut list: Vec<Snapshot> = (0..MAX_SNAPSHOTS)
            .map(|i| snap(&format!("mine {i}"), i as i64))
            .collect();
        let before: Vec<uuid::Uuid> = list.iter().map(|s| s.id).collect();
        assert!(!auto_snapshot_slot(&mut list));
        let after: Vec<uuid::Uuid> = list.iter().map(|s| s.id).collect();
        assert_eq!(after, before);

        // Plenty of room: nothing pruned beyond the rotation, slot granted.
        let mut list = vec![snap("mine", 1)];
        assert!(auto_snapshot_slot(&mut list));
        assert_eq!(list.len(), 1);
    }
}
