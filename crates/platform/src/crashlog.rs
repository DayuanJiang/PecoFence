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

/// `module+rva` for an address in this process (module = file name only).
pub(crate) fn locate(addr: u64) -> String {
    let Some(module) = module_at(addr) else {
        return format!("{addr:#x}");
    };
    let mut name = [0u16; 260];
    // SAFETY: buffer of the stated length.
    let n =
        unsafe { GetModuleFileNameW(Some(module), windows_core::PWSTR(name.as_mut_ptr()), 260) };
    let path = String::from_utf16_lossy(&name[..n as usize]);
    let file = path.rsplit(['\\', '/']).next().unwrap_or(&path).to_string();
    format!("{file}+{:#x}", addr.wrapping_sub(module.0 as usize as u64))
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
    tracing::error!(
        code = format_args!("{code:#010x}"),
        at = %locate(address),
        module_version = at_version.as_deref(),
        access,
        address = accessed.as_deref(),
        exe_version = exe_version.as_deref(),
        dump = ?dump,
        stack = %frames.join("\n"),
        "CRASH (unhandled exception)"
    );
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
