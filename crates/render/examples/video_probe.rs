//! Plays a video file with the native player on the GPU for a few seconds and reports what
//! came out: the state, how many pictures were taken, and the average colour of what Direct2D
//! drew from the last one (a black picture means nothing was decoded).
//!
//! `cargo run -p pecofence-render --example video_probe -- <file> [seconds]`
use pecofence_platform::video::VideoPlayer;
use std::time::{Duration, Instant};
use windows_canvas::{ColorF, GpuDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("usage: video_probe <file> [seconds]")?;
    let seconds: f64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(4.0);
    let _ole = pecofence_platform::com::OleGuard::init()?;
    let gpu = GpuDevice::new_or_warp()?;
    let mut player = VideoPlayer::open(gpu.d3d_device(), &path, true)?;
    player.set_volume(0.0);
    let target = gpu.create_render_target(480, 270)?;
    let began = Instant::now();
    let (mut pictures, mut last_state) = (0u32, None);
    let mut average = None;
    while began.elapsed() < Duration::from_secs_f64(seconds) {
        if player.update(true)? {
            pictures += 1;
            if let Some(frame) = player.frame() {
                target.draw(|s| {
                    s.clear(ColorF::TRANSPARENT);
                    frame.draw(s.raw(), [0.0, 0.0, 480.0, 270.0])
                })?;
                let pixels = target.read_pixels()?;
                let (mut sum, mut n) = ([0u64; 3], 0u64);
                for px in pixels.as_chunks::<4>().0 {
                    sum[0] += u64::from(px[2]);
                    sum[1] += u64::from(px[1]);
                    sum[2] += u64::from(px[0]);
                    n += 1;
                }
                average = Some([sum[0] / n, sum[1] / n, sum[2] / n]);
            }
        }
        let state = player.state();
        if last_state != Some(state.loaded) {
            println!("{:>6.2}s state {state:?}", began.elapsed().as_secs_f32());
            last_state = Some(state.loaded);
        }
        std::thread::sleep(Duration::from_millis(8));
    }
    let state = player.state();
    println!("final state {state:?}");
    println!(
        "{pictures} pictures in {:.1}s ({:.1}/s), frame {:?}, average colour {average:?}",
        began.elapsed().as_secs_f32(),
        f64::from(pictures) / began.elapsed().as_secs_f64(),
        player.frame().map(|f| (f.width, f.height)),
    );
    Ok(())
}
