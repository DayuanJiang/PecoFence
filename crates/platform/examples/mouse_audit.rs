//! End-to-end mouse driver for an explicitly named portable PecoFence audit process.
//! Uses SendInput, never sends window messages or calls the application's handlers.
//! mouse_audit PID EXE TITLE FROM_X FROM_Y TO_X TO_Y DURATION_MS [escape|right]
//! Coordinates are physical virtual-desktop pixels; Esc aborts a running gesture.
use pecofence_platform::{desktop, window};
use std::{
    env,
    error::Error,
    mem::size_of,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    },
    UI::{
        HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::{
            GUITHREADINFO, GetGUIThreadInfo, GetSystemMetrics, GetWindowTextW,
            GetWindowThreadProcessId, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        },
    },
};

fn send(input: INPUT) -> Result<(), Box<dyn Error>> {
    // SAFETY: one correctly tagged Win32 INPUT, with its exact ABI size.
    if unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) } != 1 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

fn mouse(flags: u32, x: i32, y: i32) -> Result<(), Box<dyn Error>> {
    send(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: x,
                dy: y,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    })
}

fn escape() -> Result<(), Box<dyn Error>> {
    for flags in [0, KEYEVENTF_KEYUP] {
        send(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_ESCAPE,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        })?;
    }
    Ok(())
}

struct LeftButton;
impl Drop for LeftButton {
    fn drop(&mut self) {
        // Always release our own synthetic press, including a failed/aborted gesture.
        let _ = mouse(MOUSEEVENTF_LEFTUP, 0, 0);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    if !(8..=9).contains(&args.len()) {
        return Err("PID EXE TITLE FROM_X FROM_Y TO_X TO_Y DURATION_MS [escape|right]".into());
    }
    let pid: u32 = args[0].parse()?;
    let requested = PathBuf::from(&args[1]).canonicalize()?;
    // This helper cannot target the user's installed application or arbitrary programs.
    let path = requested
        .to_string_lossy()
        .to_lowercase()
        .replace('/', "\\");
    if !path.contains("\\.cache\\") || !path.ends_with("\\pecofence.exe") {
        return Err(
            "Only a portable pecofence.exe inside a .cache audit directory is allowed".into(),
        );
    }
    // SAFETY: read-only process query, local output buffer, handle closed on both paths.
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return Err("Cannot query audit process".into());
        }
        let mut buffer = [0u16; 32768];
        let mut length = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(process);
        if ok == 0
            || PathBuf::from(String::from_utf16_lossy(&buffer[..length as usize])).canonicalize()?
                != requested
        {
            return Err("Audit PID/executable mismatch".into());
        }
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let number = |i: usize| args[i].parse::<i32>();
    let from = (number(3)?, number(4)?);
    let to = (number(5)?, number(6)?);
    let duration = Duration::from_millis(args[7].parse::<u64>()?.clamp(16, 30_000));
    let cancel = args.get(8).map(String::as_str);
    if cancel.is_some_and(|c| c != "escape" && c != "right") {
        return Err("Unsupported cancellation".into());
    }
    let target = desktop::root_ancestor(desktop::window_from_point(from.0, from.1));
    let class = desktop::class_name(target);
    if desktop::window_pid(target) != pid || class != "PecoFence.Fence" {
        return Err(format!(
            "Start point is covered by another window ({class}); refresh the observation"
        )
        .into());
    }
    let mut title = [0u16; 512];
    // SAFETY: valid HWND from WindowFromPoint and a local UTF-16 buffer.
    let n = unsafe { GetWindowTextW(target.0, title.as_mut_ptr(), title.len() as i32) };
    if String::from_utf16_lossy(&title[..n.max(0) as usize]) != args[2] {
        return Err("The observed fence title changed".into());
    }
    let before = window::window_rect(target);
    // SAFETY: a live target HWND; no process-id output is requested.
    let thread_id = unsafe { GetWindowThreadProcessId(target.0, std::ptr::null_mut()) };
    let ensure_focus = || -> Result<(), Box<dyn Error>> {
        if desktop::window_pid(desktop::foreground_window()) != pid {
            return Err("Audit lost foreground; no further pointer movement was sent".into());
        }
        Ok(())
    };
    ensure_focus()?;
    // SAFETY: asynchronous read of physical button state, no mutation.
    if unsafe { GetAsyncKeyState(VK_LBUTTON as i32) } < 0 {
        return Err("A mouse gesture is already in progress".into());
    }
    // SAFETY: scalar screen metrics from user32.
    let bounds = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    };
    let pointer = |(x, y): (i32, i32)| {
        let norm = |p: i32, origin: i32, span: i32| {
            (((p - origin) as i64 * 65536 + 32768) / span.max(1) as i64).clamp(0, 65535) as i32
        };
        mouse(
            MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
            norm(x, bounds.0, bounds.2),
            norm(y, bounds.1, bounds.3),
        )
    };
    pointer(from)?;
    mouse(MOUSEEVENTF_LEFTDOWN, 0, 0)?;
    let pressed = LeftButton;
    let start = Instant::now();
    let mut samples = 0;
    loop {
        ensure_focus()?;
        // SAFETY: read-only emergency cancellation.
        if unsafe { GetAsyncKeyState(VK_ESCAPE as i32) } < 0 {
            return Err("Mouse audit interrupted by Escape".into());
        }
        let t = (start.elapsed().as_secs_f64() / duration.as_secs_f64()).min(1.0);
        pointer((
            (from.0 as f64 + (to.0 - from.0) as f64 * t).round() as i32,
            (from.1 as f64 + (to.1 - from.1) as f64 * t).round() as i32,
        ))?;
        samples += 1;
        if t >= 1.0 {
            break;
        }
        thread::sleep(Duration::from_millis(4));
    }
    // Let the final pointer reach the real window before a cancellation key/button.
    thread::sleep(Duration::from_millis(40));
    match cancel {
        Some("escape") => escape()?,
        Some("right") => {
            mouse(MOUSEEVENTF_RIGHTDOWN, 0, 0)?;
            mouse(MOUSEEVENTF_RIGHTUP, 0, 0)?;
        }
        _ => {}
    }
    drop(pressed);
    thread::sleep(Duration::from_millis(350));
    let at = window::cursor_pos();
    let after = desktop::is_window(target).then(|| window::window_rect(target));
    let mut gui = GUITHREADINFO {
        cbSize: size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    // SAFETY: valid GUI thread id and correctly sized output structure.
    if unsafe { GetGUIThreadInfo(thread_id, &mut gui) } == 0 || !gui.hwndCapture.is_null() {
        return Err("Mouse capture was not released after the gesture".into());
    }
    if cancel.is_some() && after != Some(before) {
        return Err("Cancellation did not restore the source window geometry".into());
    }
    let rect = |r: pecofence_platform::RECT| (r.left, r.top, r.right, r.bottom);
    println!(
        "SendInput pid={pid} title={:?} from={from:?} to={to:?} samples={samples} duration_ms={} cancel={cancel:?} final_pointer=({},{}) source_before={:?} source_after={:?} capture_released=true",
        args[2],
        duration.as_millis(),
        at.x,
        at.y,
        rect(before),
        after.map(rect)
    );
    Ok(())
}
