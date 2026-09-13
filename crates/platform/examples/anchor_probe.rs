//! Read-only diagnostics for the windows immediately above Explorer's icon host.
use pecofence_platform::{desktop, window};

fn main() {
    let Some(host) = desktop::resolve_icon_host(desktop::detect_generation()) else {
        println!("No desktop icon host");
        return;
    };
    println!("host={host:?}");
    let mut current = host.host;
    let mut nearby = Vec::new();
    for _ in 0..16 {
        if current.0.is_null() {
            break;
        }
        nearby.push(current);
        println!(
            "window={:#x} class={} pid={} visible={} ex_style={:#x} rect={:?}",
            current.0 as usize,
            desktop::class_name(current),
            desktop::window_pid(current),
            desktop::is_visible(current),
            desktop::ex_style(current),
            window::window_rect(current),
        );
        current = desktop::window_above(current);
    }
    println!("{}", desktop::describe_zorder(&nearby, false));
}
