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
    use super::super::ipc::{AUTO_SNAPSHOT_PREFIX, patch_settings_path, prune_auto_snapshots};
    use pecofence_core::{Settings, Snapshot, ViewLayout};
    use pecofence_ipc::ErrorCode;
    use serde_json::json;

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
}
