//! Last-chance crash reporting: an unhandled-exception filter that writes a minidump next to
//! the log and appends the exception code, faulting address and a `module+rva` stack walk to
//! the log itself, so an access violation in a release build can be symbolized afterwards
//! against the build's `.pdb` (`dbghelp SymFromAddr` at `module base + rva`).
//!
//! Rust panics are handled separately by the app's panic hook; this covers what that hook
//! cannot see (AVs in `unsafe` / FFI code, stack overflows, fail-fasts from foreign frames).

use crate::bindings::*;
use crate::wide::to_wide;
use std::path::PathBuf;
use std::sync::OnceLock;
use windows_core::PCWSTR;

static DUMP_DIR: OnceLock<PathBuf> = OnceLock::new();
static INSTANCE: OnceLock<String> = OnceLock::new();
/// Debug builds: the module the crash line names as the faulting one instead of the real one.
#[cfg(debug_assertions)]
static FAKE_MODULE: OnceLock<String> = OnceLock::new();

/// Debug builds: the next crash line names `module` (`ExplorerPatcher.amd64.dll`) as the
/// faulting module, as when another program's DLL crashes inside PecoFence (test scripts'
/// `crash <module>`).
#[cfg(debug_assertions)]
pub fn fake_faulting_module(module: &str) {
    let _ = FAKE_MODULE.set(module.to_string());
}

/// Installs the filter. `dump_dir` receives `crash-<instance>-<ticks>.dmp`; `instance` tags the
/// files of a second (test) instance.
pub fn install(dump_dir: PathBuf, instance: &str) {
    let _ = DUMP_DIR.set(dump_dir);
    let _ = INSTANCE.set(instance.to_string());
    // SAFETY: registering a plain extern "system" callback.
    unsafe {
        // The generated wrapper takes `Option<LPTOP_LEVEL_EXCEPTION_FILTER>`, itself an Option.
        SetUnhandledExceptionFilter(Some(Some(
            filter as unsafe extern "system" fn(*const EXCEPTION_POINTERS) -> i32,
        )));
    }
}

/// The loaded module containing `addr`, if any. It stays loaded until the process ends:
/// another thread cannot unload it while the filter reads its version resource.
fn module_at(addr: u64) -> Option<HMODULE> {
    let mut module = HMODULE::default();
    // SAFETY: querying the module containing `addr`; pinning it is harmless in a process that
    // is crashing.
    let found = unsafe {
        GetModuleHandleExW(
            (GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_PIN) as u32,
            PCWSTR(addr as usize as *const u16),
            &mut module,
        )
    };
    (found.as_bool() && !module.0.is_null()).then_some(module)
}

/// The loaded module containing `addr`, if any, for a process that goes on running: the
/// module is not pinned, so it can still be unloaded.
pub(crate) fn module_holding(addr: u64) -> Option<HMODULE> {
    let mut module = HMODULE::default();
    // SAFETY: querying the module containing `addr` without changing its reference count.
    let found = unsafe {
        GetModuleHandleExW(
            (GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT)
                as u32,
            PCWSTR(addr as usize as *const u16),
            &mut module,
        )
    };
    (found.as_bool() && !module.0.is_null()).then_some(module)
}

/// `module+rva` for an address in this process (module = file name only).
fn locate(addr: u64) -> String {
    describe(addr, module_at(addr))
}

/// As [`locate`], without pinning the module (see [`module_holding`]).
pub(crate) fn locate_unpinned(addr: u64) -> String {
    describe(addr, module_holding(addr))
}

fn describe(addr: u64, module: Option<HMODULE>) -> String {
    let Some(module) = module else {
        return format!("{addr:#x}");
    };
    let path = module_path(Some(module));
    let file = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
    format!("{file}+{:#x}", addr.wrapping_sub(module.0 as usize as u64))
}

/// The full path of `module` (`None`: the program itself).
pub(crate) fn module_path(module: Option<HMODULE>) -> String {
    let mut name = [0u16; 260];
    // SAFETY: buffer of the stated length.
    let n = unsafe { GetModuleFileNameW(module, windows_core::PWSTR(name.as_mut_ptr()), 260) };
    String::from_utf16_lossy(&name[..(n as usize).min(name.len())])
}

windows_core::link!("kernel32.dll" "system" fn GetSystemWindowsDirectoryW(buffer: windows_core::PWSTR, size: u32) -> u32);
windows_core::link!("kernel32.dll" "system" fn TerminateProcess(process: HANDLE, code: u32) -> windows_core::BOOL);

/// Who ships the module at `path`: `windows`, `pecofence` (next to the program) or `other`,
/// another program's DLL that loaded into PecoFence (a shell extension, an input method, a
/// hook). The next start names an `other` module in its notice (feedback #97, GitHub #167).
pub(crate) fn module_owner(path: &str) -> &'static str {
    let mut dir = [0u16; 260];
    // SAFETY: buffer of the stated length.
    let n = unsafe { GetSystemWindowsDirectoryW(windows_core::PWSTR(dir.as_mut_ptr()), 260) };
    let windows = String::from_utf16_lossy(&dir[..(n as usize).min(dir.len())]);
    let exe = module_path(None);
    let program = exe.rsplit_once(['\\', '/']).map_or("", |(dir, _)| dir);
    owner_of(path, &windows, program)
}

/// [`module_owner`] for a module path, the Windows folder and the program's folder. The
/// WebView2 Runtime (the settings page) comes with Windows, outside its folder.
fn owner_of(path: &str, windows: &str, program: &str) -> &'static str {
    let path = path.to_lowercase().replace('/', "\\");
    let inside = |dir: &str| {
        let dir = dir.trim_end_matches(['\\', '/']).to_lowercase();
        !dir.is_empty() && path.starts_with(&format!("{dir}\\"))
    };
    if inside(windows) || path.contains("\\microsoft\\edgewebview\\") {
        "windows"
    } else if inside(program) {
        "pecofence"
    } else {
        "other"
    }
}

/// The file version (`32.0.21025.10016`) in the version resource of the module containing
/// `addr`, read from the module's mapped image: no file access, no allocation but the result.
fn module_version(addr: u64) -> Option<String> {
    const VS_VERSION_INFO: usize = 1;
    const VS_FFI_SIGNATURE: u32 = 0xFEEF_04BD;
    let module = module_at(addr)?;
    // SAFETY: the resource lies in the image of a loaded module, and every read stays within
    // the size Windows reports for it.
    unsafe {
        let res = FindResourceW(
            Some(module),
            PCWSTR(VS_VERSION_INFO as *const u16),
            RT_VERSION,
        );
        if res.0.is_null() {
            return None;
        }
        let size = SizeofResource(Some(module), res) as usize;
        let data = LockResource(LoadResource(Some(module), res)) as *const u8;
        if data.is_null() {
            return None;
        }
        // VS_FIXEDFILEINFO follows the VS_VERSIONINFO header and its key, 32-bit aligned.
        let fixed = std::mem::size_of::<VS_FIXEDFILEINFO>();
        let mut at = 0;
        while at + fixed <= size.min(256) {
            let info = std::ptr::read_unaligned(data.add(at).cast::<VS_FIXEDFILEINFO>());
            if info.dwSignature == VS_FFI_SIGNATURE {
                let (ms, ls) = (info.dwFileVersionMS, info.dwFileVersionLS);
                return Some(format!(
                    "{}.{}.{}.{}",
                    ms >> 16,
                    ms & 0xFFFF,
                    ls >> 16,
                    ls & 0xFFFF
                ));
            }
            at += 4;
        }
        None
    }
}

unsafe extern "system" fn filter(info: *const EXCEPTION_POINTERS) -> i32 {
    // Everything here must tolerate a corrupted process: no allocation-heavy work beyond a
    // few strings, no locks we may already hold (tracing's writer is a mutex — accept the
    // small risk; the dump is written first so it survives a hang).
    const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    const EXCEPTION_ACCESS_VIOLATION: u32 = 0xC000_0005;
    const EXCEPTION_IN_PAGE_ERROR: u32 = 0xC000_0006;
    // SAFETY: the OS hands a valid EXCEPTION_POINTERS to the filter.
    let (code, address, ctx, fault) = unsafe {
        if info.is_null() || (*info).ExceptionRecord.is_null() {
            return EXCEPTION_CONTINUE_SEARCH;
        }
        let rec = &*(*info).ExceptionRecord;
        // An access violation also says what the instruction tried and at which address.
        let fault = (matches!(
            rec.ExceptionCode,
            EXCEPTION_ACCESS_VIOLATION | EXCEPTION_IN_PAGE_ERROR
        ) && rec.NumberParameters >= 2)
            .then(|| (rec.ExceptionInformation[0], rec.ExceptionInformation[1]));
        (
            rec.ExceptionCode,
            rec.ExceptionAddress as usize as u64,
            (*info).ContextRecord,
            fault,
        )
    };
    let dump = write_dump(info);
    let frames = walk(ctx);
    let access = fault.map(|(kind, _)| match kind {
        0 => "read",
        1 => "write",
        8 => "execute",
        _ => "other",
    });
    let accessed = fault.map(|(_, accessed)| format!("{accessed:#x}"));
    let at_version = module_version(address);
    // This function is part of pecofence.exe: the build the exe's frames in the stack are from.
    let exe_version = module_version(filter as *const () as usize as u64);
    let at = locate(address);
    let owner = module_at(address).map(|m| module_owner(&module_path(Some(m))));
    // A test's module stands for one at that path; a bare name for another program's.
    #[cfg(debug_assertions)]
    let (at, owner) = match (FAKE_MODULE.get(), at.split_once('+')) {
        (Some(module), Some((_, rva))) => {
            let file = module.rsplit(['\\', '/']).next().unwrap_or(module);
            (format!("{file}+{rva}"), Some(module_owner(module)))
        }
        _ => (at, owner),
    };
    // What the UI thread was doing, when it was running a command (another thread may be the
    // one that crashed).
    let command = crate::hangwatch::running_command();
    tracing::error!(
        code = format_args!("{code:#010x}"),
        at = %at,
        command = command.as_deref(),
        module_version = at_version.as_deref(),
        module_owner = owner,
        access,
        address = accessed.as_deref(),
        exe_version = exe_version.as_deref(),
        dump = ?dump,
        stack = %frames.join("\n"),
        "CRASH (unhandled exception)"
    );
    // The process ends when this returns: the line has to be in the file by then.
    pecofence_core::log_file::written(pecofence_core::log_file::EXIT_WAIT);
    // A crash inside another program's module ends the process here, with the exception's code
    // as its exit code: the watchdog reads that code and starts PecoFence again. Returning to
    // Windows instead hands the crash to Windows Error Reporting, and the dead process stayed,
    // with its fences on screen, until that report was done: 1 to 2 s on a test box, about
    // 50 s on a user's PC (GitHub #167). A crash in PecoFence's own code or in a Windows file
    // still goes that way, so Windows records and reports it.
    if owner == Some("other") {
        // SAFETY: plain FFI call on this process.
        unsafe {
            let _ = TerminateProcess(GetCurrentProcess(), code);
        }
    }
    EXCEPTION_CONTINUE_SEARCH
}

fn write_dump(info: *const EXCEPTION_POINTERS) -> Option<PathBuf> {
    let dir = DUMP_DIR.get()?;
    let instance = INSTANCE.get().map(String::as_str).unwrap_or("main");
    // SAFETY: plain FFI calls with valid handles / buffers; the exception pointers come from
    // the filter's own argument.
    unsafe {
        let ticks = GetTickCount64();
        let path = dir.join(format!("crash-{instance}-{ticks}.dmp"));
        let wide = to_wide(&path.to_string_lossy());
        let file = CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_WRITE as u32,
            0,
            None,
            CREATE_ALWAYS as u32,
            FILE_ATTRIBUTE_NORMAL as u32,
            None,
        );
        if file.0.is_null() || file.0 as isize == -1 {
            return None;
        }
        let exception = MINIDUMP_EXCEPTION_INFORMATION {
            ThreadId: GetCurrentThreadId(),
            ExceptionPointers: info as *mut EXCEPTION_POINTERS,
            ClientPointers: windows_core::BOOL(0),
        };
        let ok = MiniDumpWriteDump(
            GetCurrentProcess(),
            GetCurrentProcessId(),
            file,
            MiniDumpWithIndirectlyReferencedMemory
                | MiniDumpScanMemory
                | MiniDumpWithThreadInfo
                | MiniDumpWithDataSegs,
            Some(&exception),
            None,
            None,
        );
        let _ = CloseHandle(file);
        ok.as_bool().then_some(path)
    }
}

unsafe extern "system" fn function_table_access(h: HANDLE, addr: u64) -> *mut core::ffi::c_void {
    // SAFETY: forwards to dbghelp with the caller's arguments.
    unsafe { SymFunctionTableAccess64(h, addr) }
}

unsafe extern "system" fn module_base(h: HANDLE, addr: u64) -> u64 {
    // SAFETY: forwards to dbghelp with the caller's arguments.
    unsafe { SymGetModuleBase64(h, addr) }
}

/// Frames of the faulting thread as `module+rva`, from the exception context (the filter's own
/// frames are not part of it).
fn walk(ctx: *mut CONTEXT) -> Vec<String> {
    let mut out = Vec::new();
    if ctx.is_null() {
        return out;
    }
    // SAFETY: dbghelp stack walk over a copy of the faulting context; all pointers are to
    // locals or to the context the OS passed in.
    unsafe {
        let process = GetCurrentProcess();
        let _ = SymInitializeW(process, PCWSTR::null(), true);
        let mut context: CONTEXT = *ctx;
        let mut frame = STACKFRAME64::default();
        frame.AddrPC.Offset = context.Rip;
        frame.AddrPC.Mode = AddrModeFlat;
        frame.AddrFrame.Offset = context.Rbp;
        frame.AddrFrame.Mode = AddrModeFlat;
        frame.AddrStack.Offset = context.Rsp;
        frame.AddrStack.Mode = AddrModeFlat;
        for _ in 0..48 {
            let ok = StackWalk64(
                IMAGE_FILE_MACHINE_AMD64 as u32,
                process,
                GetCurrentThread(),
                &mut frame,
                (&mut context as *mut CONTEXT).cast(),
                None,
                Some(function_table_access),
                Some(module_base),
                None,
            );
            if !ok.as_bool() || frame.AddrPC.Offset == 0 {
                break;
            }
            out.push(format!("  {}", locate(frame.AddrPC.Offset)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::owner_of;

    #[test]
    fn faulting_modules_are_told_apart_by_folder() {
        let (windows, program) = (r"C:\WINDOWS", r"C:\Program Files\PecoFence");
        let owner = |path| owner_of(path, windows, program);
        assert_eq!(owner(r"C:\Windows\System32\combase.dll"), "windows");
        // The GPU driver's DLLs are in the driver store, inside the Windows folder.
        assert_eq!(
            owner(r"C:\WINDOWS\System32\DriverStore\FileRepository\u0.inf_amd64\B1\amdxx64.dll"),
            "windows"
        );
        assert_eq!(
            owner(
                r"C:\Program Files (x86)\Microsoft\EdgeWebView\Application\142.0\EBWebView\x64\EmbeddedBrowserWebView.dll"
            ),
            "windows"
        );
        assert_eq!(
            owner(r"C:\Program Files\PecoFence\pecofence.exe"),
            "pecofence"
        );
        assert_eq!(
            owner(r"C:\Program Files\PecoFence\WebView2Loader.dll"),
            "pecofence"
        );
        assert_eq!(
            owner(
                r"C:\Users\a\AppData\Local\Kingsoft\WPS Office\12.1.0.29162\office6\qingshellext64.dll"
            ),
            "other"
        );
        assert_eq!(
            owner(r"C:\Program Files\ExplorerPatcher\ExplorerPatcher.amd64.dll"),
            "other"
        );
        // A folder whose name only starts like the program's is another one.
        assert_eq!(owner(r"C:\Program Files\PecoFence2\x.dll"), "other");
        assert_eq!(owner(r"C:\Windows.old\System32\x.dll"), "other");
        // A bare name (a test's module) is in neither folder.
        assert_eq!(owner("qingshellext64.dll"), "other");
        assert_eq!(owner_of("x.dll", "", ""), "other");
    }
}
