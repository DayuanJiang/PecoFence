//! Prints where the keyboard focus and the text caret are for a few seconds (the answers the
//! Magnifier follows): `cargo run -p pecofence-platform --example uia_probe -- [seconds]`. Click in
//! a text box of another program meanwhile and type. With `describe` as the second argument it
//! prints instead, once a second, what each way of finding the caret says for the focused element.

use pecofence_platform::uia::{FocusWatcher, describe_focus};
use std::time::{Duration, Instant};

fn main() {
    let seconds: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    if std::env::args().nth(2).as_deref() == Some("describe") {
        for _ in 0..seconds {
            println!("{}", describe_focus());
            std::thread::sleep(Duration::from_secs(1));
        }
        return;
    }
    let watcher = FocusWatcher::start();
    watcher.want(true, true);
    let end = Instant::now() + Duration::from_secs(seconds);
    let mut last = Default::default();
    while Instant::now() < end {
        let now = watcher.latest();
        if now != last {
            println!(
                "focus #{} {:?}   caret #{} {:?}",
                now.focus_seq, now.focus, now.caret_seq, now.caret
            );
            last = now;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
