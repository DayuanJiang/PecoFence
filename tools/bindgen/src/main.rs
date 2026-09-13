//! Regenerates the platform, Composition and GPU glass bindings from Windows metadata.
//!
//! Run from the workspace root: `cargo run -p tool_bindgen`.

fn main() {
    let start = std::time::Instant::now();
    // Optional arguments name the `--etc` files to run (one-off probes).
    let etcs: Vec<String> = match std::env::args().skip(1).collect::<Vec<_>>() {
        v if v.is_empty() => vec![
            "tools/bindgen/platform.txt".to_string(),
            "tools/bindgen/composition.txt".to_string(),
            "tools/bindgen/gpu-glass.txt".to_string(),
        ],
        v => v,
    };
    for etc in &etcs {
        windows_bindgen::bindgen(["--etc", etc.as_str()]);
        println!(
            "regenerated bindings from {etc} in {:.2}s",
            start.elapsed().as_secs_f32()
        );
    }
}
