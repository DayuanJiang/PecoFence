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
