//! A shell extension for tests only: a context-menu handler and a copy hook that record every
//! call the shell makes into them and, on request, fault there, the way a broken extension of
//! another program does inside PecoFence (GitHub #167, #178).
//!
//! Beside the DLL:
//!   calls.log  one line per call: time, host program, process and thread id, entry point
//!   fault.txt  `<entry> [<wait ms>]`: that entry point waits and then reads from an address
//!              that is never mapped (an access violation), only inside pecofence.exe;
//!              `<entry> hang`: that entry point never returns
//! Entry points: DllGetClassObject, Initialize, QueryContextMenu, InvokeCommand, CopyCallback.
//!
//! Registered per user by the steps that use it (HKCU\Software\Classes\CLSID\{CLSID} with
//! InprocServer32, and a `shellex\ContextMenuHandlers` / `shellex\CopyHookHandlers` entry),
//! and removed by them again.

#![allow(non_snake_case, clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, Ordering};

type Hresult = i32;
const S_OK: Hresult = 0;
const S_FALSE: Hresult = 1;
const E_NOTIMPL: Hresult = 0x8000_4001u32 as i32;
const E_NOINTERFACE: Hresult = 0x8000_4002u32 as i32;
const E_FAIL: Hresult = 0x8000_4005u32 as i32;
const CLASS_E_CLASSNOTAVAILABLE: Hresult = 0x8004_0111u32 as i32;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Guid(u32, u16, u16, [u8; 8]);

/// The shell's own interface ids all end in C000-000000000046.
const fn ole(first: u32) -> Guid {
    Guid(first, 0, 0, [0xC0, 0, 0, 0, 0, 0, 0, 0x46])
}
const IID_UNKNOWN: Guid = ole(0x0000_0000);
const IID_CLASS_FACTORY: Guid = ole(0x0000_0001);
const IID_SHELL_EXT_INIT: Guid = ole(0x0002_14E8);
const IID_CONTEXT_MENU: Guid = ole(0x0002_14E4);
const IID_COPY_HOOK_W: Guid = ole(0x0002_14FC);
/// {7C0F1E5A-51B6-4E0C-9C55-70F5C0F3A167}
const CLSID_FAULT: Guid = Guid(
    0x7C0F_1E5A,
    0x51B6,
    0x4E0C,
    [0x9C, 0x55, 0x70, 0xF5, 0xC0, 0xF3, 0xA1, 0x67],
);

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleFileNameW(module: isize, buffer: *mut u16, size: u32) -> u32;
    fn GetCurrentThreadId() -> u32;
}
#[link(name = "user32")]
unsafe extern "system" {
    fn InsertMenuW(menu: isize, position: u32, flags: u32, id: usize, text: *const u16) -> i32;
}

/// This DLL's module handle, from `DllMain`.
static MODULE: AtomicIsize = AtomicIsize::new(0);

/// The folder of this DLL: `calls.log` and `fault.txt` are there.
fn folder() -> PathBuf {
    let mut name = [0u16; 520];
    // SAFETY: buffer of the stated length.
    let n = unsafe { GetModuleFileNameW(MODULE.load(Ordering::Relaxed), name.as_mut_ptr(), 520) };
    let path = PathBuf::from(String::from_utf16_lossy(&name[..n as usize]));
    path.parent().map(PathBuf::from).unwrap_or_default()
}

/// The program this DLL is loaded into, lower case (`pecofence.exe`, `explorer.exe`).
fn host() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .unwrap_or_default()
}

fn log(entry: &str, detail: &str) {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    // SAFETY: plain FFI call.
    let thread = unsafe { GetCurrentThreadId() };
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(folder().join("calls.log"))
    {
        let _ = writeln!(
            file,
            "ms={ms} host={} pid={} tid={thread} {entry} {detail}",
            host(),
            std::process::id()
        );
    }
}

/// Records the call; faults here when `fault.txt` names this entry point and the host is
/// PecoFence (Explorer and every other program on the test box stay unharmed).
fn enter(entry: &str, detail: &str) {
    log(entry, detail);
    let Ok(text) = std::fs::read_to_string(folder().join("fault.txt")) else {
        return;
    };
    let mut words = text.split_whitespace();
    if words.next() != Some(entry) || host() != "pecofence.exe" {
        return;
    }
    let wait = words.next();
    if wait == Some("hang") {
        log(entry, "hanging now");
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    }
    if let Some(ms) = wait.and_then(|w| w.parse::<u64>().ok()) {
        log(entry, &format!("waiting {ms} ms before the fault"));
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
    log(entry, "faulting now");
    // SAFETY: none, on purpose: the first 64 KB of the address space are never mapped, so
    // this read is an access violation, like the one in the report.
    unsafe { std::ptr::read_volatile(0x10 as *const u8) };
}

type This = *mut c_void;
type QueryFn = unsafe extern "system" fn(This, *const Guid, *mut *mut c_void) -> Hresult;
type CountFn = unsafe extern "system" fn(This) -> u32;

#[repr(C)]
struct UnknownVtbl {
    query: QueryFn,
    add_ref: CountFn,
    release: CountFn,
}
#[repr(C)]
struct FactoryVtbl {
    base: UnknownVtbl,
    create: unsafe extern "system" fn(This, This, *const Guid, *mut *mut c_void) -> Hresult,
    lock: unsafe extern "system" fn(This, i32) -> Hresult,
}
#[repr(C)]
struct InitVtbl {
    base: UnknownVtbl,
    initialize: unsafe extern "system" fn(This, *const c_void, This, isize) -> Hresult,
}
#[repr(C)]
struct MenuVtbl {
    base: UnknownVtbl,
    query_menu: unsafe extern "system" fn(This, isize, u32, u32, u32, u32) -> Hresult,
    invoke: unsafe extern "system" fn(This, *const InvokeInfo) -> Hresult,
    command_string: unsafe extern "system" fn(This, usize, u32, *mut u32, *mut u8, u32) -> Hresult,
}
#[repr(C)]
struct HookVtbl {
    base: UnknownVtbl,
    copy: unsafe extern "system" fn(This, isize, u32, u32, *const u16, u32, *const u16, u32) -> u32,
}
/// The start of CMINVOKECOMMANDINFO.
#[repr(C)]
struct InvokeInfo {
    size: u32,
    mask: u32,
    window: isize,
    verb: *const u8,
}

/// A COM object without state: only its vtable. They live for the whole process, so the
/// reference counts are constants.
#[repr(C)]
struct Object<V: 'static>(&'static V);
// SAFETY: the vtables hold function pointers only and never change.
unsafe impl<V> Sync for Object<V> {}

static FACTORY: Object<FactoryVtbl> = Object(&FactoryVtbl {
    base: UnknownVtbl {
        query: factory_query,
        add_ref: one,
        release: one,
    },
    create,
    lock,
});
static INIT: Object<InitVtbl> = Object(&InitVtbl {
    base: BASE,
    initialize,
});
static MENU: Object<MenuVtbl> = Object(&MenuVtbl {
    base: BASE,
    query_menu,
    invoke,
    command_string,
});
static HOOK: Object<HookVtbl> = Object(&HookVtbl { base: BASE, copy });
const BASE: UnknownVtbl = UnknownVtbl {
    query,
    add_ref: one,
    release: one,
};

unsafe extern "system" fn one(_: This) -> u32 {
    1
}

unsafe extern "system" fn factory_query(
    _: This,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> Hresult {
    // SAFETY: COM hands valid pointers.
    unsafe {
        let known = *iid == IID_UNKNOWN || *iid == IID_CLASS_FACTORY;
        *out = if known {
            &FACTORY as *const _ as *mut c_void
        } else {
            std::ptr::null_mut()
        };
        if known { S_OK } else { E_NOINTERFACE }
    }
}

unsafe extern "system" fn query(_: This, iid: *const Guid, out: *mut *mut c_void) -> Hresult {
    // SAFETY: COM hands valid pointers.
    unsafe {
        let iid = *iid;
        *out = if iid == IID_UNKNOWN || iid == IID_SHELL_EXT_INIT {
            &INIT as *const _ as *mut c_void
        } else if iid == IID_CONTEXT_MENU {
            &MENU as *const _ as *mut c_void
        } else if iid == IID_COPY_HOOK_W {
            &HOOK as *const _ as *mut c_void
        } else {
            std::ptr::null_mut()
        };
        if (*out).is_null() {
            E_NOINTERFACE
        } else {
            S_OK
        }
    }
}

unsafe extern "system" fn create(
    _: This,
    _outer: This,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> Hresult {
    // SAFETY: forwards COM's pointers.
    unsafe { query(std::ptr::null_mut(), iid, out) }
}

unsafe extern "system" fn lock(_: This, _: i32) -> Hresult {
    S_OK
}

unsafe extern "system" fn initialize(
    _: This,
    _folder: *const c_void,
    _data: This,
    _key: isize,
) -> Hresult {
    enter("Initialize", "");
    S_OK
}

unsafe extern "system" fn query_menu(
    _: This,
    menu: isize,
    index: u32,
    first: u32,
    _last: u32,
    flags: u32,
) -> Hresult {
    enter("QueryContextMenu", &format!("flags={flags:#x}"));
    const CMF_DEFAULTONLY: u32 = 1;
    const MF_BYPOSITION: u32 = 0x400;
    if flags & CMF_DEFAULTONLY != 0 {
        return 0;
    }
    let text: Vec<u16> = "PF fault test\0".encode_utf16().collect();
    // SAFETY: the shell's menu; the text outlives the call.
    unsafe { InsertMenuW(menu, index, MF_BYPOSITION, first as usize, text.as_ptr()) };
    // One command id used.
    1
}

unsafe extern "system" fn invoke(_: This, info: *const InvokeInfo) -> Hresult {
    // SAFETY: the shell hands a valid CMINVOKECOMMANDINFO.
    let verb = unsafe { (*info).verb } as usize;
    // A verb given by name is another handler's (the shell asks every handler in turn).
    if verb >> 16 != 0 {
        enter("InvokeCommand", "a verb by name");
        return E_FAIL;
    }
    enter("InvokeCommand", &format!("command {verb}"));
    S_OK
}

unsafe extern "system" fn command_string(
    _: This,
    _: usize,
    _: u32,
    _: *mut u32,
    _: *mut u8,
    _: u32,
) -> Hresult {
    E_NOTIMPL
}

unsafe extern "system" fn copy(
    _: This,
    _window: isize,
    func: u32,
    _flags: u32,
    _src: *const u16,
    _: u32,
    _dest: *const u16,
    _: u32,
) -> u32 {
    enter("CopyCallback", &format!("func={func}"));
    const IDYES: u32 = 6;
    IDYES
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(module: isize, reason: u32, _: *mut c_void) -> i32 {
    const DLL_PROCESS_ATTACH: u32 = 1;
    if reason == DLL_PROCESS_ATTACH {
        MODULE.store(module, Ordering::Relaxed);
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    clsid: *const Guid,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> Hresult {
    // SAFETY: COM hands valid pointers.
    unsafe {
        if *clsid != CLSID_FAULT {
            *out = std::ptr::null_mut();
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        enter("DllGetClassObject", "");
        factory_query(std::ptr::null_mut(), iid, out)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllCanUnloadNow() -> Hresult {
    S_FALSE
}
