#[inline]
pub unsafe fn AppendMenuW<P3>(
    hmenu: HMENU,
    uflags: u32,
    uidnewitem: usize,
    lpnewitem: P3,
) -> windows_core::BOOL
where
    P3: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn AppendMenuW(hmenu : HMENU, uflags : u32, uidnewitem : usize, lpnewitem : windows_core::PCWSTR) -> windows_core::BOOL);
    unsafe { AppendMenuW(hmenu, uflags, uidnewitem, lpnewitem.param().abi()) }
}
#[inline]
pub unsafe fn AssignProcessToJobObject(hjob: HANDLE, hprocess: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn AssignProcessToJobObject(hjob : HANDLE, hprocess : HANDLE) -> windows_core::BOOL);
    unsafe { AssignProcessToJobObject(hjob, hprocess) }
}
#[inline]
pub unsafe fn CallWindowProcW(
    lpprevwndfunc: WNDPROC,
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn CallWindowProcW(lpprevwndfunc : WNDPROC, hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
    unsafe { CallWindowProcW(lpprevwndfunc, hwnd, msg, wparam, lparam) }
}
#[inline]
pub unsafe fn CancelIoEx(
    hfile: HANDLE,
    lpoverlapped: Option<*const OVERLAPPED>,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn CancelIoEx(hfile : HANDLE, lpoverlapped : *const OVERLAPPED) -> windows_core::BOOL);
    unsafe { CancelIoEx(hfile, lpoverlapped.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn ClientToScreen(hwnd: HWND, lppoint: *mut POINT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ClientToScreen(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
    unsafe { ClientToScreen(hwnd, lppoint as _) }
}
#[inline]
pub unsafe fn CloseHandle(hobject: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn CloseHandle(hobject : HANDLE) -> windows_core::BOOL);
    unsafe { CloseHandle(hobject) }
}
#[inline]
pub unsafe fn CoCreateInstance<P1, T>(
    rclsid: *const windows_core::GUID,
    punkouter: P1,
    dwclscontext: u32,
) -> windows_core::Result<T>
where
    P1: windows_core::Param<windows_core::IUnknown>,
    T: windows_core::Interface,
{
    windows_core::link!("ole32.dll" "system" fn CoCreateInstance(rclsid : *const windows_core::GUID, punkouter : *mut core::ffi::c_void, dwclscontext : u32, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        CoCreateInstance(
            rclsid,
            punkouter.param().abi(),
            dwclscontext,
            &T::IID,
            &mut result__,
        )
        .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn CoInitializeEx(
    pvreserved: Option<*const core::ffi::c_void>,
    dwcoinit: u32,
) -> windows_core::HRESULT {
    windows_core::link!("ole32.dll" "system" fn CoInitializeEx(pvreserved : *const core::ffi::c_void, dwcoinit : u32) -> windows_core::HRESULT);
    unsafe { CoInitializeEx(pvreserved.unwrap_or(core::mem::zeroed()) as _, dwcoinit) }
}
#[inline]
pub unsafe fn CoTaskMemFree(pv: *mut core::ffi::c_void) {
    windows_core::link!("ole32.dll" "system" fn CoTaskMemFree(pv : *mut core::ffi::c_void));
    unsafe { CoTaskMemFree(pv as _) }
}
#[inline]
pub unsafe fn CoUninitialize() {
    windows_core::link!("ole32.dll" "system" fn CoUninitialize());
    unsafe { CoUninitialize() }
}
#[inline]
pub unsafe fn ConnectNamedPipe(
    hnamedpipe: HANDLE,
    lpoverlapped: Option<*mut OVERLAPPED>,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn ConnectNamedPipe(hnamedpipe : HANDLE, lpoverlapped : *mut OVERLAPPED) -> windows_core::BOOL);
    unsafe { ConnectNamedPipe(hnamedpipe, lpoverlapped.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn CreateBitmap(
    nwidth: i32,
    nheight: i32,
    nplanes: u32,
    nbitcount: u32,
    lpbits: Option<*const core::ffi::c_void>,
) -> HBITMAP {
    windows_core::link!("gdi32.dll" "system" fn CreateBitmap(nwidth : i32, nheight : i32, nplanes : u32, nbitcount : u32, lpbits : *const core::ffi::c_void) -> HBITMAP);
    unsafe {
        CreateBitmap(
            nwidth,
            nheight,
            nplanes,
            nbitcount,
            lpbits.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn CreateCompatibleDC(hdc: Option<HDC>) -> HDC {
    windows_core::link!("gdi32.dll" "system" fn CreateCompatibleDC(hdc : HDC) -> HDC);
    unsafe { CreateCompatibleDC(hdc.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn CreateDIBSection(
    hdc: Option<HDC>,
    pbmi: *const BITMAPINFO,
    usage: u32,
    ppvbits: *mut *mut core::ffi::c_void,
    hsection: Option<HANDLE>,
    offset: u32,
) -> HBITMAP {
    windows_core::link!("gdi32.dll" "system" fn CreateDIBSection(hdc : HDC, pbmi : *const BITMAPINFO, usage : u32, ppvbits : *mut *mut core::ffi::c_void, hsection : HANDLE, offset : u32) -> HBITMAP);
    unsafe {
        CreateDIBSection(
            hdc.unwrap_or(core::mem::zeroed()) as _,
            pbmi,
            usage,
            ppvbits as _,
            hsection.unwrap_or(core::mem::zeroed()) as _,
            offset,
        )
    }
}
#[inline]
pub unsafe fn CreateDispatcherQueueController(
    options: DispatcherQueueOptions,
) -> windows_core::Result<IDispatcherQueueController> {
    windows_core::link!("coremessaging.dll" "system" fn CreateDispatcherQueueController(options : DispatcherQueueOptions, dispatcherqueuecontroller : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        CreateDispatcherQueueController(options, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn CreateEventW<P3>(
    lpeventattributes: Option<*const SECURITY_ATTRIBUTES>,
    bmanualreset: bool,
    binitialstate: bool,
    lpname: P3,
) -> HANDLE
where
    P3: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateEventW(lpeventattributes : *const SECURITY_ATTRIBUTES, bmanualreset : windows_core::BOOL, binitialstate : windows_core::BOOL, lpname : windows_core::PCWSTR) -> HANDLE);
    unsafe {
        CreateEventW(
            lpeventattributes.unwrap_or(core::mem::zeroed()) as _,
            bmanualreset.into(),
            binitialstate.into(),
            lpname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn CreateFileW<P0>(
    lpfilename: P0,
    dwdesiredaccess: u32,
    dwsharemode: u32,
    lpsecurityattributes: Option<*const SECURITY_ATTRIBUTES>,
    dwcreationdisposition: u32,
    dwflagsandattributes: u32,
    htemplatefile: Option<HANDLE>,
) -> HANDLE
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateFileW(lpfilename : windows_core::PCWSTR, dwdesiredaccess : u32, dwsharemode : u32, lpsecurityattributes : *const SECURITY_ATTRIBUTES, dwcreationdisposition : u32, dwflagsandattributes : u32, htemplatefile : HANDLE) -> HANDLE);
    unsafe {
        CreateFileW(
            lpfilename.param().abi(),
            dwdesiredaccess,
            dwsharemode,
            lpsecurityattributes.unwrap_or(core::mem::zeroed()) as _,
            dwcreationdisposition,
            dwflagsandattributes,
            htemplatefile.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn CreateFontW<P13>(
    cheight: i32,
    cwidth: i32,
    cescapement: i32,
    corientation: i32,
    cweight: i32,
    bitalic: u32,
    bunderline: u32,
    bstrikeout: u32,
    icharset: u32,
    ioutprecision: u32,
    iclipprecision: u32,
    iquality: u32,
    ipitchandfamily: u32,
    pszfacename: P13,
) -> HFONT
where
    P13: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("gdi32.dll" "system" fn CreateFontW(cheight : i32, cwidth : i32, cescapement : i32, corientation : i32, cweight : i32, bitalic : u32, bunderline : u32, bstrikeout : u32, icharset : u32, ioutprecision : u32, iclipprecision : u32, iquality : u32, ipitchandfamily : u32, pszfacename : windows_core::PCWSTR) -> HFONT);
    unsafe {
        CreateFontW(
            cheight,
            cwidth,
            cescapement,
            corientation,
            cweight,
            bitalic,
            bunderline,
            bstrikeout,
            icharset,
            ioutprecision,
            iclipprecision,
            iquality,
            ipitchandfamily,
            pszfacename.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn CreateIconIndirect(piconinfo: *const ICONINFO) -> HICON {
    windows_core::link!("user32.dll" "system" fn CreateIconIndirect(piconinfo : *const ICONINFO) -> HICON);
    unsafe { CreateIconIndirect(piconinfo) }
}
#[inline]
pub unsafe fn CreateJobObjectW<P1>(
    lpjobattributes: Option<*const SECURITY_ATTRIBUTES>,
    lpname: P1,
) -> HANDLE
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateJobObjectW(lpjobattributes : *const SECURITY_ATTRIBUTES, lpname : windows_core::PCWSTR) -> HANDLE);
    unsafe {
        CreateJobObjectW(
            lpjobattributes.unwrap_or(core::mem::zeroed()) as _,
            lpname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn CreateMutexW<P2>(
    lpmutexattributes: Option<*const SECURITY_ATTRIBUTES>,
    binitialowner: bool,
    lpname: P2,
) -> HANDLE
where
    P2: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateMutexW(lpmutexattributes : *const SECURITY_ATTRIBUTES, binitialowner : windows_core::BOOL, lpname : windows_core::PCWSTR) -> HANDLE);
    unsafe {
        CreateMutexW(
            lpmutexattributes.unwrap_or(core::mem::zeroed()) as _,
            binitialowner.into(),
            lpname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn CreateNamedPipeW<P0>(
    lpname: P0,
    dwopenmode: u32,
    dwpipemode: u32,
    nmaxinstances: u32,
    noutbuffersize: u32,
    ninbuffersize: u32,
    ndefaulttimeout: u32,
    lpsecurityattributes: Option<*const SECURITY_ATTRIBUTES>,
) -> HANDLE
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateNamedPipeW(lpname : windows_core::PCWSTR, dwopenmode : u32, dwpipemode : u32, nmaxinstances : u32, noutbuffersize : u32, ninbuffersize : u32, ndefaulttimeout : u32, lpsecurityattributes : *const SECURITY_ATTRIBUTES) -> HANDLE);
    unsafe {
        CreateNamedPipeW(
            lpname.param().abi(),
            dwopenmode,
            dwpipemode,
            nmaxinstances,
            noutbuffersize,
            ninbuffersize,
            ndefaulttimeout,
            lpsecurityattributes.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn CreatePopupMenu() -> HMENU {
    windows_core::link!("user32.dll" "system" fn CreatePopupMenu() -> HMENU);
    unsafe { CreatePopupMenu() }
}
#[inline]
pub unsafe fn CreateProcessW<P0, P7>(
    lpapplicationname: P0,
    lpcommandline: Option<windows_core::PWSTR>,
    lpprocessattributes: Option<*const SECURITY_ATTRIBUTES>,
    lpthreadattributes: Option<*const SECURITY_ATTRIBUTES>,
    binherithandles: bool,
    dwcreationflags: u32,
    lpenvironment: Option<*const core::ffi::c_void>,
    lpcurrentdirectory: P7,
    lpstartupinfo: *const STARTUPINFOW,
    lpprocessinformation: *mut PROCESS_INFORMATION,
) -> windows_core::BOOL
where
    P0: windows_core::Param<windows_core::PCWSTR>,
    P7: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn CreateProcessW(lpapplicationname : windows_core::PCWSTR, lpcommandline : windows_core::PWSTR, lpprocessattributes : *const SECURITY_ATTRIBUTES, lpthreadattributes : *const SECURITY_ATTRIBUTES, binherithandles : windows_core::BOOL, dwcreationflags : u32, lpenvironment : *const core::ffi::c_void, lpcurrentdirectory : windows_core::PCWSTR, lpstartupinfo : *const STARTUPINFOW, lpprocessinformation : *mut PROCESS_INFORMATION) -> windows_core::BOOL);
    unsafe {
        CreateProcessW(
            lpapplicationname.param().abi(),
            lpcommandline.unwrap_or(core::mem::zeroed()) as _,
            lpprocessattributes.unwrap_or(core::mem::zeroed()) as _,
            lpthreadattributes.unwrap_or(core::mem::zeroed()) as _,
            binherithandles.into(),
            dwcreationflags,
            lpenvironment.unwrap_or(core::mem::zeroed()) as _,
            lpcurrentdirectory.param().abi(),
            lpstartupinfo,
            lpprocessinformation as _,
        )
    }
}
#[inline]
pub unsafe fn CreateSolidBrush(color: COLORREF) -> HBRUSH {
    windows_core::link!("gdi32.dll" "system" fn CreateSolidBrush(color : COLORREF) -> HBRUSH);
    unsafe { CreateSolidBrush(color) }
}
#[inline]
pub unsafe fn CreateWindowExW<P1, P2>(
    dwexstyle: u32,
    lpclassname: P1,
    lpwindowname: P2,
    dwstyle: u32,
    x: i32,
    y: i32,
    nwidth: i32,
    nheight: i32,
    hwndparent: Option<HWND>,
    hmenu: Option<HMENU>,
    hinstance: Option<HINSTANCE>,
    lpparam: Option<*const core::ffi::c_void>,
) -> HWND
where
    P1: windows_core::Param<windows_core::PCWSTR>,
    P2: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn CreateWindowExW(dwexstyle : u32, lpclassname : windows_core::PCWSTR, lpwindowname : windows_core::PCWSTR, dwstyle : u32, x : i32, y : i32, nwidth : i32, nheight : i32, hwndparent : HWND, hmenu : HMENU, hinstance : HINSTANCE, lpparam : *const core::ffi::c_void) -> HWND);
    unsafe {
        CreateWindowExW(
            dwexstyle,
            lpclassname.param().abi(),
            lpwindowname.param().abi(),
            dwstyle,
            x,
            y,
            nwidth,
            nheight,
            hwndparent.unwrap_or(core::mem::zeroed()) as _,
            hmenu.unwrap_or(core::mem::zeroed()) as _,
            hinstance.unwrap_or(core::mem::zeroed()) as _,
            lpparam.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn DCompositionWaitForCompositorClock(
    handles: Option<&[HANDLE]>,
    timeoutinms: u32,
) -> u32 {
    windows_core::link!("dcomp.dll" "system" fn DCompositionWaitForCompositorClock(count : u32, handles : *const HANDLE, timeoutinms : u32) -> u32);
    unsafe {
        DCompositionWaitForCompositorClock(
            handles.map_or(0, |slice| slice.len().try_into().unwrap()),
            handles.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            timeoutinms,
        )
    }
}
#[inline]
pub unsafe fn DefSubclassProc(hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    windows_core::link!("comctl32.dll" "system" fn DefSubclassProc(hwnd : HWND, umsg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
    unsafe { DefSubclassProc(hwnd, umsg, wparam, lparam) }
}
#[inline]
pub unsafe fn DefWindowProcW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn DefWindowProcW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
#[inline]
pub unsafe fn DeleteDC(hdc: HDC) -> windows_core::BOOL {
    windows_core::link!("gdi32.dll" "system" fn DeleteDC(hdc : HDC) -> windows_core::BOOL);
    unsafe { DeleteDC(hdc) }
}
#[inline]
pub unsafe fn DeleteObject(ho: HGDIOBJ) -> windows_core::BOOL {
    windows_core::link!("gdi32.dll" "system" fn DeleteObject(ho : HGDIOBJ) -> windows_core::BOOL);
    unsafe { DeleteObject(ho) }
}
#[inline]
pub unsafe fn DestroyIcon(hicon: HICON) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn DestroyIcon(hicon : HICON) -> windows_core::BOOL);
    unsafe { DestroyIcon(hicon) }
}
#[inline]
pub unsafe fn DestroyMenu(hmenu: HMENU) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn DestroyMenu(hmenu : HMENU) -> windows_core::BOOL);
    unsafe { DestroyMenu(hmenu) }
}
#[inline]
pub unsafe fn DestroyWindow(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn DestroyWindow(hwnd : HWND) -> windows_core::BOOL);
    unsafe { DestroyWindow(hwnd) }
}
#[inline]
pub unsafe fn DisconnectNamedPipe(hnamedpipe: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn DisconnectNamedPipe(hnamedpipe : HANDLE) -> windows_core::BOOL);
    unsafe { DisconnectNamedPipe(hnamedpipe) }
}
#[inline]
pub unsafe fn DispatchMessageW(lpmsg: *const MSG) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn DispatchMessageW(lpmsg : *const MSG) -> LRESULT);
    unsafe { DispatchMessageW(lpmsg) }
}
#[inline]
pub unsafe fn DragQueryFileW(
    hdrop: HDROP,
    ifile: u32,
    lpszfile: Option<windows_core::PWSTR>,
    cch: u32,
) -> u32 {
    windows_core::link!("shell32.dll" "system" fn DragQueryFileW(hdrop : HDROP, ifile : u32, lpszfile : windows_core::PWSTR, cch : u32) -> u32);
    unsafe {
        DragQueryFileW(
            hdrop,
            ifile,
            lpszfile.unwrap_or(core::mem::zeroed()) as _,
            cch,
        )
    }
}
#[inline]
pub unsafe fn DwmExtendFrameIntoClientArea(
    hwnd: HWND,
    pmarinset: *const MARGINS,
) -> windows_core::HRESULT {
    windows_core::link!("dwmapi.dll" "system" fn DwmExtendFrameIntoClientArea(hwnd : HWND, pmarinset : *const MARGINS) -> windows_core::HRESULT);
    unsafe { DwmExtendFrameIntoClientArea(hwnd, pmarinset) }
}
#[inline]
pub unsafe fn DwmFlush() -> windows_core::HRESULT {
    windows_core::link!("dwmapi.dll" "system" fn DwmFlush() -> windows_core::HRESULT);
    unsafe { DwmFlush() }
}
#[inline]
pub unsafe fn DwmGetWindowAttribute(
    hwnd: HWND,
    dwattribute: u32,
    pvattribute: *mut core::ffi::c_void,
    cbattribute: u32,
) -> windows_core::HRESULT {
    windows_core::link!("dwmapi.dll" "system" fn DwmGetWindowAttribute(hwnd : HWND, dwattribute : u32, pvattribute : *mut core::ffi::c_void, cbattribute : u32) -> windows_core::HRESULT);
    unsafe { DwmGetWindowAttribute(hwnd, dwattribute, pvattribute as _, cbattribute) }
}
#[inline]
pub unsafe fn DwmIsCompositionEnabled() -> windows_core::Result<windows_core::BOOL> {
    windows_core::link!("dwmapi.dll" "system" fn DwmIsCompositionEnabled(pfenabled : *mut windows_core::BOOL) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        DwmIsCompositionEnabled(&mut result__).map(|| result__)
    }
}
#[inline]
pub unsafe fn DwmSetWindowAttribute(
    hwnd: HWND,
    dwattribute: u32,
    pvattribute: *const core::ffi::c_void,
    cbattribute: u32,
) -> windows_core::HRESULT {
    windows_core::link!("dwmapi.dll" "system" fn DwmSetWindowAttribute(hwnd : HWND, dwattribute : u32, pvattribute : *const core::ffi::c_void, cbattribute : u32) -> windows_core::HRESULT);
    unsafe { DwmSetWindowAttribute(hwnd, dwattribute, pvattribute, cbattribute) }
}
#[inline]
pub unsafe fn EnumDisplayDevicesW<P0>(
    lpdevice: P0,
    idevnum: u32,
    lpdisplaydevice: *mut DISPLAY_DEVICEW,
    dwflags: u32,
) -> windows_core::BOOL
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn EnumDisplayDevicesW(lpdevice : windows_core::PCWSTR, idevnum : u32, lpdisplaydevice : *mut DISPLAY_DEVICEW, dwflags : u32) -> windows_core::BOOL);
    unsafe {
        EnumDisplayDevicesW(
            lpdevice.param().abi(),
            idevnum,
            lpdisplaydevice as _,
            dwflags,
        )
    }
}
#[inline]
pub unsafe fn EnumDisplayMonitors(
    hdc: Option<HDC>,
    lprcclip: Option<*const RECT>,
    lpfnenum: MONITORENUMPROC,
    dwdata: LPARAM,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn EnumDisplayMonitors(hdc : HDC, lprcclip : *const RECT, lpfnenum : MONITORENUMPROC, dwdata : LPARAM) -> windows_core::BOOL);
    unsafe {
        EnumDisplayMonitors(
            hdc.unwrap_or(core::mem::zeroed()) as _,
            lprcclip.unwrap_or(core::mem::zeroed()) as _,
            lpfnenum,
            dwdata,
        )
    }
}
#[inline]
pub unsafe fn EnumWindows(lpenumfunc: WNDENUMPROC, lparam: LPARAM) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn EnumWindows(lpenumfunc : WNDENUMPROC, lparam : LPARAM) -> windows_core::BOOL);
    unsafe { EnumWindows(lpenumfunc, lparam) }
}
#[inline]
pub unsafe fn FileTimeToSystemTime(
    lpfiletime: *const FILETIME,
    lpsystemtime: *mut SYSTEMTIME,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn FileTimeToSystemTime(lpfiletime : *const FILETIME, lpsystemtime : *mut SYSTEMTIME) -> windows_core::BOOL);
    unsafe { FileTimeToSystemTime(lpfiletime, lpsystemtime as _) }
}
#[inline]
pub unsafe fn FindWindowExW<P2, P3>(
    hwndparent: Option<HWND>,
    hwndchildafter: Option<HWND>,
    lpszclass: P2,
    lpszwindow: P3,
) -> HWND
where
    P2: windows_core::Param<windows_core::PCWSTR>,
    P3: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn FindWindowExW(hwndparent : HWND, hwndchildafter : HWND, lpszclass : windows_core::PCWSTR, lpszwindow : windows_core::PCWSTR) -> HWND);
    unsafe {
        FindWindowExW(
            hwndparent.unwrap_or(core::mem::zeroed()) as _,
            hwndchildafter.unwrap_or(core::mem::zeroed()) as _,
            lpszclass.param().abi(),
            lpszwindow.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn FindWindowW<P0, P1>(lpclassname: P0, lpwindowname: P1) -> HWND
where
    P0: windows_core::Param<windows_core::PCWSTR>,
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn FindWindowW(lpclassname : windows_core::PCWSTR, lpwindowname : windows_core::PCWSTR) -> HWND);
    unsafe { FindWindowW(lpclassname.param().abi(), lpwindowname.param().abi()) }
}
#[inline]
pub unsafe fn FlushFileBuffers(hfile: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn FlushFileBuffers(hfile : HANDLE) -> windows_core::BOOL);
    unsafe { FlushFileBuffers(hfile) }
}
#[inline]
pub unsafe fn GetAncestor(hwnd: HWND, gaflags: u32) -> HWND {
    windows_core::link!("user32.dll" "system" fn GetAncestor(hwnd : HWND, gaflags : u32) -> HWND);
    unsafe { GetAncestor(hwnd, gaflags) }
}
#[inline]
pub unsafe fn GetAsyncKeyState(vkey: i32) -> i16 {
    windows_core::link!("user32.dll" "system" fn GetAsyncKeyState(vkey : i32) -> i16);
    unsafe { GetAsyncKeyState(vkey) }
}
#[inline]
pub unsafe fn GetCapture() -> HWND {
    windows_core::link!("user32.dll" "system" fn GetCapture() -> HWND);
    unsafe { GetCapture() }
}
#[inline]
pub unsafe fn GetClassNameW(hwnd: HWND, lpclassname: windows_core::PWSTR, nmaxcount: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetClassNameW(hwnd : HWND, lpclassname : windows_core::PWSTR, nmaxcount : i32) -> i32);
    unsafe { GetClassNameW(hwnd, lpclassname, nmaxcount) }
}
#[inline]
pub unsafe fn GetClientRect(hwnd: HWND, lprect: *mut RECT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetClientRect(hwnd : HWND, lprect : *mut RECT) -> windows_core::BOOL);
    unsafe { GetClientRect(hwnd, lprect as _) }
}
#[inline]
pub unsafe fn GetClipboardSequenceNumber() -> u32 {
    windows_core::link!("user32.dll" "system" fn GetClipboardSequenceNumber() -> u32);
    unsafe { GetClipboardSequenceNumber() }
}
#[inline]
pub unsafe fn GetCurrentProcess() -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentProcess() -> HANDLE);
    unsafe { GetCurrentProcess() }
}
#[inline]
pub unsafe fn GetCurrentProcessId() -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentProcessId() -> u32);
    unsafe { GetCurrentProcessId() }
}
#[inline]
pub unsafe fn GetCurrentThread() -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentThread() -> HANDLE);
    unsafe { GetCurrentThread() }
}
#[inline]
pub unsafe fn GetCurrentThreadId() -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetCurrentThreadId() -> u32);
    unsafe { GetCurrentThreadId() }
}
#[inline]
pub unsafe fn GetCursorPos(lppoint: *mut POINT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetCursorPos(lppoint : *mut POINT) -> windows_core::BOOL);
    unsafe { GetCursorPos(lppoint as _) }
}
#[inline]
pub unsafe fn GetDC(hwnd: Option<HWND>) -> HDC {
    windows_core::link!("user32.dll" "system" fn GetDC(hwnd : HWND) -> HDC);
    unsafe { GetDC(hwnd.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GetDateFormatEx<P0, P3, P6>(
    lplocalename: P0,
    dwflags: u32,
    lpdate: Option<*const SYSTEMTIME>,
    lpformat: P3,
    lpdatestr: Option<windows_core::PWSTR>,
    cchdate: i32,
    lpcalendar: P6,
) -> i32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
    P3: windows_core::Param<windows_core::PCWSTR>,
    P6: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetDateFormatEx(lplocalename : windows_core::PCWSTR, dwflags : u32, lpdate : *const SYSTEMTIME, lpformat : windows_core::PCWSTR, lpdatestr : windows_core::PWSTR, cchdate : i32, lpcalendar : windows_core::PCWSTR) -> i32);
    unsafe {
        GetDateFormatEx(
            lplocalename.param().abi(),
            dwflags,
            lpdate.unwrap_or(core::mem::zeroed()) as _,
            lpformat.param().abi(),
            lpdatestr.unwrap_or(core::mem::zeroed()) as _,
            cchdate,
            lpcalendar.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn GetDesktopWindow() -> HWND {
    windows_core::link!("user32.dll" "system" fn GetDesktopWindow() -> HWND);
    unsafe { GetDesktopWindow() }
}
#[inline]
pub unsafe fn GetDoubleClickTime() -> u32 {
    windows_core::link!("user32.dll" "system" fn GetDoubleClickTime() -> u32);
    unsafe { GetDoubleClickTime() }
}
#[inline]
pub unsafe fn GetDpiForMonitor(
    hmonitor: HMONITOR,
    dpitype: MONITOR_DPI_TYPE,
    dpix: *mut u32,
    dpiy: *mut u32,
) -> windows_core::HRESULT {
    windows_core::link!("api-ms-win-shcore-scaling-l1-1-1.dll" "system" fn GetDpiForMonitor(hmonitor : HMONITOR, dpitype : MONITOR_DPI_TYPE, dpix : *mut u32, dpiy : *mut u32) -> windows_core::HRESULT);
    unsafe { GetDpiForMonitor(hmonitor, dpitype, dpix as _, dpiy as _) }
}
#[inline]
pub unsafe fn GetDpiForSystem() -> u32 {
    windows_core::link!("user32.dll" "system" fn GetDpiForSystem() -> u32);
    unsafe { GetDpiForSystem() }
}
#[inline]
pub unsafe fn GetDpiForWindow(hwnd: HWND) -> u32 {
    windows_core::link!("user32.dll" "system" fn GetDpiForWindow(hwnd : HWND) -> u32);
    unsafe { GetDpiForWindow(hwnd) }
}
#[inline]
pub unsafe fn GetDriveTypeW<P0>(lprootpathname: P0) -> u32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetDriveTypeW(lprootpathname : windows_core::PCWSTR) -> u32);
    unsafe { GetDriveTypeW(lprootpathname.param().abi()) }
}
#[inline]
pub unsafe fn GetExitCodeProcess(hprocess: HANDLE, lpexitcode: *mut u32) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn GetExitCodeProcess(hprocess : HANDLE, lpexitcode : *mut u32) -> windows_core::BOOL);
    unsafe { GetExitCodeProcess(hprocess, lpexitcode as _) }
}
#[inline]
pub unsafe fn GetFileAttributesW<P0>(lpfilename: P0) -> u32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetFileAttributesW(lpfilename : windows_core::PCWSTR) -> u32);
    unsafe { GetFileAttributesW(lpfilename.param().abi()) }
}
#[inline]
pub unsafe fn GetFocus() -> HWND {
    windows_core::link!("user32.dll" "system" fn GetFocus() -> HWND);
    unsafe { GetFocus() }
}
#[inline]
pub unsafe fn GetForegroundWindow() -> HWND {
    windows_core::link!("user32.dll" "system" fn GetForegroundWindow() -> HWND);
    unsafe { GetForegroundWindow() }
}
#[inline]
pub unsafe fn GetKeyState(nvirtkey: i32) -> i16 {
    windows_core::link!("user32.dll" "system" fn GetKeyState(nvirtkey : i32) -> i16);
    unsafe { GetKeyState(nvirtkey) }
}
#[inline]
pub unsafe fn GetKeyboardLayout(idthread: u32) -> HKL {
    windows_core::link!("user32.dll" "system" fn GetKeyboardLayout(idthread : u32) -> HKL);
    unsafe { GetKeyboardLayout(idthread) }
}
#[inline]
pub unsafe fn GetLastError() -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetLastError() -> u32);
    unsafe { GetLastError() }
}
#[inline]
pub unsafe fn GetLogicalDrives() -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetLogicalDrives() -> u32);
    unsafe { GetLogicalDrives() }
}
#[inline]
pub unsafe fn GetMenuItemCount(hmenu: Option<HMENU>) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetMenuItemCount(hmenu : HMENU) -> i32);
    unsafe { GetMenuItemCount(hmenu.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GetMenuItemID(hmenu: HMENU, npos: i32) -> u32 {
    windows_core::link!("user32.dll" "system" fn GetMenuItemID(hmenu : HMENU, npos : i32) -> u32);
    unsafe { GetMenuItemID(hmenu, npos) }
}
#[inline]
pub unsafe fn GetMessageW(
    lpmsg: *mut MSG,
    hwnd: Option<HWND>,
    wmsgfiltermin: u32,
    wmsgfiltermax: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetMessageW(lpmsg : *mut MSG, hwnd : HWND, wmsgfiltermin : u32, wmsgfiltermax : u32) -> windows_core::BOOL);
    unsafe {
        GetMessageW(
            lpmsg as _,
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            wmsgfiltermin,
            wmsgfiltermax,
        )
    }
}
#[inline]
pub unsafe fn GetModuleFileNameW(
    hmodule: Option<HMODULE>,
    lpfilename: windows_core::PWSTR,
    nsize: u32,
) -> u32 {
    windows_core::link!("kernel32.dll" "system" fn GetModuleFileNameW(hmodule : HMODULE, lpfilename : windows_core::PWSTR, nsize : u32) -> u32);
    unsafe {
        GetModuleFileNameW(
            hmodule.unwrap_or(core::mem::zeroed()) as _,
            lpfilename,
            nsize,
        )
    }
}
#[inline]
pub unsafe fn GetModuleHandleExW<P1>(
    dwflags: u32,
    lpmodulename: P1,
    phmodule: *mut HMODULE,
) -> windows_core::BOOL
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetModuleHandleExW(dwflags : u32, lpmodulename : windows_core::PCWSTR, phmodule : *mut HMODULE) -> windows_core::BOOL);
    unsafe { GetModuleHandleExW(dwflags, lpmodulename.param().abi(), phmodule as _) }
}
#[inline]
pub unsafe fn GetModuleHandleW<P0>(lpmodulename: P0) -> HMODULE
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetModuleHandleW(lpmodulename : windows_core::PCWSTR) -> HMODULE);
    unsafe { GetModuleHandleW(lpmodulename.param().abi()) }
}
#[inline]
pub unsafe fn GetMonitorInfoW(hmonitor: HMONITOR, lpmi: *mut MONITORINFO) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetMonitorInfoW(hmonitor : HMONITOR, lpmi : *mut MONITORINFO) -> windows_core::BOOL);
    unsafe { GetMonitorInfoW(hmonitor, lpmi as _) }
}
#[inline]
pub unsafe fn GetObjectW(h: HANDLE, c: i32, pv: Option<*mut core::ffi::c_void>) -> i32 {
    windows_core::link!("gdi32.dll" "system" fn GetObjectW(h : HANDLE, c : i32, pv : *mut core::ffi::c_void) -> i32);
    unsafe { GetObjectW(h, c, pv.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GetOverlappedResult(
    hfile: HANDLE,
    lpoverlapped: *const OVERLAPPED,
    lpnumberofbytestransferred: *mut u32,
    bwait: bool,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn GetOverlappedResult(hfile : HANDLE, lpoverlapped : *const OVERLAPPED, lpnumberofbytestransferred : *mut u32, bwait : windows_core::BOOL) -> windows_core::BOOL);
    unsafe {
        GetOverlappedResult(
            hfile,
            lpoverlapped,
            lpnumberofbytestransferred as _,
            bwait.into(),
        )
    }
}
#[inline]
pub unsafe fn GetProcAddress<P1>(hmodule: HMODULE, lpprocname: P1) -> FARPROC
where
    P1: windows_core::Param<windows_core::PCSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetProcAddress(hmodule : HMODULE, lpprocname : windows_core::PCSTR) -> FARPROC);
    unsafe { GetProcAddress(hmodule, lpprocname.param().abi()) }
}
#[inline]
pub unsafe fn GetProcessMemoryInfo(
    process: HANDLE,
    ppsmemcounters: *mut PROCESS_MEMORY_COUNTERS,
    cb: u32,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" "K32GetProcessMemoryInfo" fn GetProcessMemoryInfo(process : HANDLE, ppsmemcounters : *mut PROCESS_MEMORY_COUNTERS, cb : u32) -> windows_core::BOOL);
    unsafe { GetProcessMemoryInfo(process, ppsmemcounters as _, cb) }
}
#[inline]
pub unsafe fn GetRawInputData(
    hrawinput: HRAWINPUT,
    uicommand: u32,
    pdata: Option<*mut core::ffi::c_void>,
    pcbsize: *mut u32,
    cbsizeheader: u32,
) -> u32 {
    windows_core::link!("user32.dll" "system" fn GetRawInputData(hrawinput : HRAWINPUT, uicommand : u32, pdata : *mut core::ffi::c_void, pcbsize : *mut u32, cbsizeheader : u32) -> u32);
    unsafe {
        GetRawInputData(
            hrawinput,
            uicommand,
            pdata.unwrap_or(core::mem::zeroed()) as _,
            pcbsize as _,
            cbsizeheader,
        )
    }
}
#[inline]
pub unsafe fn GetShellWindow() -> HWND {
    windows_core::link!("user32.dll" "system" fn GetShellWindow() -> HWND);
    unsafe { GetShellWindow() }
}
#[inline]
pub unsafe fn GetSysColor(nindex: i32) -> u32 {
    windows_core::link!("user32.dll" "system" fn GetSysColor(nindex : i32) -> u32);
    unsafe { GetSysColor(nindex) }
}
#[inline]
pub unsafe fn GetSystemMetrics(nindex: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetSystemMetrics(nindex : i32) -> i32);
    unsafe { GetSystemMetrics(nindex) }
}
#[inline]
pub unsafe fn GetSystemMetricsForDpi(nindex: i32, dpi: u32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetSystemMetricsForDpi(nindex : i32, dpi : u32) -> i32);
    unsafe { GetSystemMetricsForDpi(nindex, dpi) }
}
#[inline]
pub unsafe fn GetTickCount64() -> u64 {
    windows_core::link!("kernel32.dll" "system" fn GetTickCount64() -> u64);
    unsafe { GetTickCount64() }
}
#[inline]
pub unsafe fn GetTimeFormatEx<P0, P3>(
    lplocalename: P0,
    dwflags: u32,
    lptime: Option<*const SYSTEMTIME>,
    lpformat: P3,
    lptimestr: Option<windows_core::PWSTR>,
    cchtime: i32,
) -> i32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
    P3: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn GetTimeFormatEx(lplocalename : windows_core::PCWSTR, dwflags : u32, lptime : *const SYSTEMTIME, lpformat : windows_core::PCWSTR, lptimestr : windows_core::PWSTR, cchtime : i32) -> i32);
    unsafe {
        GetTimeFormatEx(
            lplocalename.param().abi(),
            dwflags,
            lptime.unwrap_or(core::mem::zeroed()) as _,
            lpformat.param().abi(),
            lptimestr.unwrap_or(core::mem::zeroed()) as _,
            cchtime,
        )
    }
}
#[inline]
pub unsafe fn GetWindow(hwnd: HWND, ucmd: u32) -> HWND {
    windows_core::link!("user32.dll" "system" fn GetWindow(hwnd : HWND, ucmd : u32) -> HWND);
    unsafe { GetWindow(hwnd, ucmd) }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[inline]
pub unsafe fn GetWindowLongPtrW(hwnd: HWND, nindex: i32) -> isize {
    windows_core::link!("user32.dll" "system" fn GetWindowLongPtrW(hwnd : HWND, nindex : i32) -> isize);
    unsafe { GetWindowLongPtrW(hwnd, nindex) }
}
#[cfg(target_pointer_width = "32")]
pub use GetWindowLongW as GetWindowLongPtrW;
#[inline]
pub unsafe fn GetWindowLongW(hwnd: HWND, nindex: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetWindowLongW(hwnd : HWND, nindex : i32) -> i32);
    unsafe { GetWindowLongW(hwnd, nindex) }
}
#[inline]
pub unsafe fn GetWindowPlacement(hwnd: HWND, lpwndpl: *mut WINDOWPLACEMENT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetWindowPlacement(hwnd : HWND, lpwndpl : *mut WINDOWPLACEMENT) -> windows_core::BOOL);
    unsafe { GetWindowPlacement(hwnd, lpwndpl as _) }
}
#[inline]
pub unsafe fn GetWindowRect(hwnd: HWND, lprect: *mut RECT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn GetWindowRect(hwnd : HWND, lprect : *mut RECT) -> windows_core::BOOL);
    unsafe { GetWindowRect(hwnd, lprect as _) }
}
#[inline]
pub unsafe fn GetWindowTextLengthW(hwnd: HWND) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetWindowTextLengthW(hwnd : HWND) -> i32);
    unsafe { GetWindowTextLengthW(hwnd) }
}
#[inline]
pub unsafe fn GetWindowTextW(hwnd: HWND, lpstring: windows_core::PWSTR, nmaxcount: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn GetWindowTextW(hwnd : HWND, lpstring : windows_core::PWSTR, nmaxcount : i32) -> i32);
    unsafe { GetWindowTextW(hwnd, lpstring, nmaxcount) }
}
#[inline]
pub unsafe fn GetWindowThreadProcessId(hwnd: HWND, lpdwprocessid: Option<*mut u32>) -> u32 {
    windows_core::link!("user32.dll" "system" fn GetWindowThreadProcessId(hwnd : HWND, lpdwprocessid : *mut u32) -> u32);
    unsafe { GetWindowThreadProcessId(hwnd, lpdwprocessid.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn GlobalLock(hmem: HGLOBAL) -> *mut core::ffi::c_void {
    windows_core::link!("kernel32.dll" "system" fn GlobalLock(hmem : HGLOBAL) -> *mut core::ffi::c_void);
    unsafe { GlobalLock(hmem) }
}
#[inline]
pub unsafe fn GlobalSize(hmem: HGLOBAL) -> usize {
    windows_core::link!("kernel32.dll" "system" fn GlobalSize(hmem : HGLOBAL) -> usize);
    unsafe { GlobalSize(hmem) }
}
#[inline]
pub unsafe fn GlobalUnlock(hmem: HGLOBAL) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn GlobalUnlock(hmem : HGLOBAL) -> windows_core::BOOL);
    unsafe { GlobalUnlock(hmem) }
}
#[inline]
pub unsafe fn ILClone(pidl: *const ITEMIDLIST) -> LPITEMIDLIST {
    windows_core::link!("shell32.dll" "system" fn ILClone(pidl : *const ITEMIDLIST) -> LPITEMIDLIST);
    unsafe { ILClone(pidl) }
}
#[inline]
pub unsafe fn ILFindLastID(pidl: *const ITEMIDLIST) -> LPITEMIDLIST {
    windows_core::link!("shell32.dll" "system" fn ILFindLastID(pidl : *const ITEMIDLIST) -> LPITEMIDLIST);
    unsafe { ILFindLastID(pidl) }
}
#[inline]
pub unsafe fn ILFree(pidl: Option<*const ITEMIDLIST>) {
    windows_core::link!("shell32.dll" "system" fn ILFree(pidl : *const ITEMIDLIST));
    unsafe { ILFree(pidl.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> windows_core::BOOL {
    windows_core::link!("comctl32.dll" "system" fn InitCommonControlsEx(picce : *const INITCOMMONCONTROLSEX) -> windows_core::BOOL);
    unsafe { InitCommonControlsEx(picce) }
}
#[inline]
pub unsafe fn InsertMenuItemW(
    hmenu: HMENU,
    item: u32,
    fbyposition: bool,
    lpmi: *const MENUITEMINFOW,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn InsertMenuItemW(hmenu : HMENU, item : u32, fbyposition : windows_core::BOOL, lpmi : *const MENUITEMINFOW) -> windows_core::BOOL);
    unsafe { InsertMenuItemW(hmenu, item, fbyposition.into(), lpmi) }
}
#[inline]
pub unsafe fn IsIconic(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn IsIconic(hwnd : HWND) -> windows_core::BOOL);
    unsafe { IsIconic(hwnd) }
}
#[inline]
pub unsafe fn IsWindow(hwnd: Option<HWND>) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn IsWindow(hwnd : HWND) -> windows_core::BOOL);
    unsafe { IsWindow(hwnd.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn IsWindowEnabled(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn IsWindowEnabled(hwnd : HWND) -> windows_core::BOOL);
    unsafe { IsWindowEnabled(hwnd) }
}
#[inline]
pub unsafe fn IsWindowVisible(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn IsWindowVisible(hwnd : HWND) -> windows_core::BOOL);
    unsafe { IsWindowVisible(hwnd) }
}
#[inline]
pub unsafe fn KillTimer(hwnd: Option<HWND>, uidevent: usize) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn KillTimer(hwnd : HWND, uidevent : usize) -> windows_core::BOOL);
    unsafe { KillTimer(hwnd.unwrap_or(core::mem::zeroed()) as _, uidevent) }
}
#[inline]
pub unsafe fn LoadCursorW<P1>(hinstance: Option<HINSTANCE>, lpcursorname: P1) -> HCURSOR
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn LoadCursorW(hinstance : HINSTANCE, lpcursorname : windows_core::PCWSTR) -> HCURSOR);
    unsafe {
        LoadCursorW(
            hinstance.unwrap_or(core::mem::zeroed()) as _,
            lpcursorname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn LoadIconW<P1>(hinstance: Option<HINSTANCE>, lpiconname: P1) -> HICON
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn LoadIconW(hinstance : HINSTANCE, lpiconname : windows_core::PCWSTR) -> HICON);
    unsafe {
        LoadIconW(
            hinstance.unwrap_or(core::mem::zeroed()) as _,
            lpiconname.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn MapVirtualKeyExW(ucode: u32, umaptype: u32, dwhkl: Option<HKL>) -> u32 {
    windows_core::link!("user32.dll" "system" fn MapVirtualKeyExW(ucode : u32, umaptype : u32, dwhkl : HKL) -> u32);
    unsafe { MapVirtualKeyExW(ucode, umaptype, dwhkl.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn MessageBeep(utype: u32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn MessageBeep(utype : u32) -> windows_core::BOOL);
    unsafe { MessageBeep(utype) }
}
#[inline]
pub unsafe fn MessageBoxW<P1, P2>(hwnd: Option<HWND>, lptext: P1, lpcaption: P2, utype: u32) -> i32
where
    P1: windows_core::Param<windows_core::PCWSTR>,
    P2: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn MessageBoxW(hwnd : HWND, lptext : windows_core::PCWSTR, lpcaption : windows_core::PCWSTR, utype : u32) -> i32);
    unsafe {
        MessageBoxW(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            lptext.param().abi(),
            lpcaption.param().abi(),
            utype,
        )
    }
}
#[inline]
pub unsafe fn MiniDumpWriteDump(
    hprocess: HANDLE,
    processid: u32,
    hfile: HANDLE,
    dumptype: MINIDUMP_TYPE,
    exceptionparam: Option<*const MINIDUMP_EXCEPTION_INFORMATION>,
    userstreamparam: Option<*const MINIDUMP_USER_STREAM_INFORMATION>,
    callbackparam: Option<*const MINIDUMP_CALLBACK_INFORMATION>,
) -> windows_core::BOOL {
    windows_core::link!("dbghelp.dll" "system" fn MiniDumpWriteDump(hprocess : HANDLE, processid : u32, hfile : HANDLE, dumptype : MINIDUMP_TYPE, exceptionparam : *const MINIDUMP_EXCEPTION_INFORMATION, userstreamparam : *const MINIDUMP_USER_STREAM_INFORMATION, callbackparam : *const MINIDUMP_CALLBACK_INFORMATION) -> windows_core::BOOL);
    unsafe {
        MiniDumpWriteDump(
            hprocess,
            processid,
            hfile,
            dumptype,
            exceptionparam.unwrap_or(core::mem::zeroed()) as _,
            userstreamparam.unwrap_or(core::mem::zeroed()) as _,
            callbackparam.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn MonitorFromPoint(pt: POINT, dwflags: u32) -> HMONITOR {
    windows_core::link!("user32.dll" "system" fn MonitorFromPoint(pt : POINT, dwflags : u32) -> HMONITOR);
    unsafe { MonitorFromPoint(pt, dwflags) }
}
#[inline]
pub unsafe fn MonitorFromRect(lprc: *const RECT, dwflags: u32) -> HMONITOR {
    windows_core::link!("user32.dll" "system" fn MonitorFromRect(lprc : *const RECT, dwflags : u32) -> HMONITOR);
    unsafe { MonitorFromRect(lprc, dwflags) }
}
#[inline]
pub unsafe fn MonitorFromWindow(hwnd: HWND, dwflags: u32) -> HMONITOR {
    windows_core::link!("user32.dll" "system" fn MonitorFromWindow(hwnd : HWND, dwflags : u32) -> HMONITOR);
    unsafe { MonitorFromWindow(hwnd, dwflags) }
}
#[inline]
pub unsafe fn MoveWindow(
    hwnd: HWND,
    x: i32,
    y: i32,
    nwidth: i32,
    nheight: i32,
    brepaint: bool,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn MoveWindow(hwnd : HWND, x : i32, y : i32, nwidth : i32, nheight : i32, brepaint : windows_core::BOOL) -> windows_core::BOOL);
    unsafe { MoveWindow(hwnd, x, y, nwidth, nheight, brepaint.into()) }
}
#[inline]
pub unsafe fn OleGetClipboard() -> windows_core::Result<IDataObject> {
    windows_core::link!("ole32.dll" "system" fn OleGetClipboard(ppdataobj : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        OleGetClipboard(&mut result__).and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn OleInitialize(pvreserved: *const core::ffi::c_void) -> windows_core::HRESULT {
    windows_core::link!("ole32.dll" "system" fn OleInitialize(pvreserved : *const core::ffi::c_void) -> windows_core::HRESULT);
    unsafe { OleInitialize(pvreserved) }
}
#[inline]
pub unsafe fn OleSetClipboard<P0>(pdataobj: P0) -> windows_core::HRESULT
where
    P0: windows_core::Param<IDataObject>,
{
    windows_core::link!("ole32.dll" "system" fn OleSetClipboard(pdataobj : *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe { OleSetClipboard(pdataobj.param().abi()) }
}
#[inline]
pub unsafe fn OleUninitialize() {
    windows_core::link!("ole32.dll" "system" fn OleUninitialize());
    unsafe { OleUninitialize() }
}
#[inline]
pub unsafe fn OpenProcess(dwdesiredaccess: u32, binherithandle: bool, dwprocessid: u32) -> HANDLE {
    windows_core::link!("kernel32.dll" "system" fn OpenProcess(dwdesiredaccess : u32, binherithandle : windows_core::BOOL, dwprocessid : u32) -> HANDLE);
    unsafe { OpenProcess(dwdesiredaccess, binherithandle.into(), dwprocessid) }
}
#[inline]
pub unsafe fn OutputDebugStringW<P0>(lpoutputstring: P0)
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("kernel32.dll" "system" fn OutputDebugStringW(lpoutputstring : windows_core::PCWSTR));
    unsafe { OutputDebugStringW(lpoutputstring.param().abi()) }
}
#[inline]
pub unsafe fn PeekMessageW(
    lpmsg: *mut MSG,
    hwnd: Option<HWND>,
    wmsgfiltermin: u32,
    wmsgfiltermax: u32,
    wremovemsg: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn PeekMessageW(lpmsg : *mut MSG, hwnd : HWND, wmsgfiltermin : u32, wmsgfiltermax : u32, wremovemsg : u32) -> windows_core::BOOL);
    unsafe {
        PeekMessageW(
            lpmsg as _,
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            wmsgfiltermin,
            wmsgfiltermax,
            wremovemsg,
        )
    }
}
#[inline]
pub unsafe fn PostMessageW(
    hwnd: Option<HWND>,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn PostMessageW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> windows_core::BOOL);
    unsafe {
        PostMessageW(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            msg,
            wparam,
            lparam,
        )
    }
}
#[inline]
pub unsafe fn PostQuitMessage(nexitcode: i32) {
    windows_core::link!("user32.dll" "system" fn PostQuitMessage(nexitcode : i32));
    unsafe { PostQuitMessage(nexitcode) }
}
#[inline]
pub unsafe fn PostThreadMessageW(
    idthread: u32,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn PostThreadMessageW(idthread : u32, msg : u32, wparam : WPARAM, lparam : LPARAM) -> windows_core::BOOL);
    unsafe { PostThreadMessageW(idthread, msg, wparam, lparam) }
}
#[inline]
pub unsafe fn ReadDirectoryChangesW(
    hdirectory: HANDLE,
    lpbuffer: *mut core::ffi::c_void,
    nbufferlength: u32,
    bwatchsubtree: bool,
    dwnotifyfilter: u32,
    lpbytesreturned: Option<*mut u32>,
    lpoverlapped: Option<*mut OVERLAPPED>,
    lpcompletionroutine: LPOVERLAPPED_COMPLETION_ROUTINE,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn ReadDirectoryChangesW(hdirectory : HANDLE, lpbuffer : *mut core::ffi::c_void, nbufferlength : u32, bwatchsubtree : windows_core::BOOL, dwnotifyfilter : u32, lpbytesreturned : *mut u32, lpoverlapped : *mut OVERLAPPED, lpcompletionroutine : LPOVERLAPPED_COMPLETION_ROUTINE) -> windows_core::BOOL);
    unsafe {
        ReadDirectoryChangesW(
            hdirectory,
            lpbuffer as _,
            nbufferlength,
            bwatchsubtree.into(),
            dwnotifyfilter,
            lpbytesreturned.unwrap_or(core::mem::zeroed()) as _,
            lpoverlapped.unwrap_or(core::mem::zeroed()) as _,
            lpcompletionroutine,
        )
    }
}
#[inline]
pub unsafe fn RegCloseKey(hkey: HKEY) -> LSTATUS {
    windows_core::link!("advapi32.dll" "system" fn RegCloseKey(hkey : HKEY) -> LSTATUS);
    unsafe { RegCloseKey(hkey) }
}
#[inline]
pub unsafe fn RegCreateKeyExW<P1, P3>(
    hkey: HKEY,
    lpsubkey: P1,
    reserved: Option<u32>,
    lpclass: P3,
    dwoptions: u32,
    samdesired: REGSAM,
    lpsecurityattributes: Option<*const SECURITY_ATTRIBUTES>,
    phkresult: *mut HKEY,
    lpdwdisposition: Option<*mut u32>,
) -> LSTATUS
where
    P1: windows_core::Param<windows_core::PCWSTR>,
    P3: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn RegCreateKeyExW(hkey : HKEY, lpsubkey : windows_core::PCWSTR, reserved : u32, lpclass : windows_core::PCWSTR, dwoptions : u32, samdesired : REGSAM, lpsecurityattributes : *const SECURITY_ATTRIBUTES, phkresult : *mut HKEY, lpdwdisposition : *mut u32) -> LSTATUS);
    unsafe {
        RegCreateKeyExW(
            hkey,
            lpsubkey.param().abi(),
            reserved.unwrap_or(core::mem::zeroed()) as _,
            lpclass.param().abi(),
            dwoptions,
            samdesired,
            lpsecurityattributes.unwrap_or(core::mem::zeroed()) as _,
            phkresult as _,
            lpdwdisposition.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn RegDeleteValueW<P1>(hkey: HKEY, lpvaluename: P1) -> LSTATUS
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn RegDeleteValueW(hkey : HKEY, lpvaluename : windows_core::PCWSTR) -> LSTATUS);
    unsafe { RegDeleteValueW(hkey, lpvaluename.param().abi()) }
}
#[inline]
pub unsafe fn RegGetValueW<P1, P2>(
    hkey: HKEY,
    lpsubkey: P1,
    lpvalue: P2,
    dwflags: u32,
    pdwtype: Option<*mut u32>,
    pvdata: Option<*mut core::ffi::c_void>,
    pcbdata: Option<*mut u32>,
) -> LSTATUS
where
    P1: windows_core::Param<windows_core::PCWSTR>,
    P2: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn RegGetValueW(hkey : HKEY, lpsubkey : windows_core::PCWSTR, lpvalue : windows_core::PCWSTR, dwflags : u32, pdwtype : *mut u32, pvdata : *mut core::ffi::c_void, pcbdata : *mut u32) -> LSTATUS);
    unsafe {
        RegGetValueW(
            hkey,
            lpsubkey.param().abi(),
            lpvalue.param().abi(),
            dwflags,
            pdwtype.unwrap_or(core::mem::zeroed()) as _,
            pvdata.unwrap_or(core::mem::zeroed()) as _,
            pcbdata.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn RegNotifyChangeKeyValue(
    hkey: HKEY,
    bwatchsubtree: bool,
    dwnotifyfilter: u32,
    hevent: Option<HANDLE>,
    fasynchronous: bool,
) -> LSTATUS {
    windows_core::link!("advapi32.dll" "system" fn RegNotifyChangeKeyValue(hkey : HKEY, bwatchsubtree : windows_core::BOOL, dwnotifyfilter : u32, hevent : HANDLE, fasynchronous : windows_core::BOOL) -> LSTATUS);
    unsafe {
        RegNotifyChangeKeyValue(
            hkey,
            bwatchsubtree.into(),
            dwnotifyfilter,
            hevent.unwrap_or(core::mem::zeroed()) as _,
            fasynchronous.into(),
        )
    }
}
#[inline]
pub unsafe fn RegOpenKeyExW<P1>(
    hkey: HKEY,
    lpsubkey: P1,
    uloptions: Option<u32>,
    samdesired: REGSAM,
    phkresult: *mut HKEY,
) -> LSTATUS
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn RegOpenKeyExW(hkey : HKEY, lpsubkey : windows_core::PCWSTR, uloptions : u32, samdesired : REGSAM, phkresult : *mut HKEY) -> LSTATUS);
    unsafe {
        RegOpenKeyExW(
            hkey,
            lpsubkey.param().abi(),
            uloptions.unwrap_or(core::mem::zeroed()) as _,
            samdesired,
            phkresult as _,
        )
    }
}
#[inline]
pub unsafe fn RegSetValueExW<P1>(
    hkey: HKEY,
    lpvaluename: P1,
    reserved: Option<u32>,
    dwtype: u32,
    lpdata: Option<&[u8]>,
) -> LSTATUS
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("advapi32.dll" "system" fn RegSetValueExW(hkey : HKEY, lpvaluename : windows_core::PCWSTR, reserved : u32, dwtype : u32, lpdata : *const u8, cbdata : u32) -> LSTATUS);
    unsafe {
        RegSetValueExW(
            hkey,
            lpvaluename.param().abi(),
            reserved.unwrap_or(core::mem::zeroed()) as _,
            dwtype,
            lpdata.map_or(core::ptr::null(), |slice| slice.as_ptr()),
            lpdata.map_or(0, |slice| slice.len().try_into().unwrap()),
        )
    }
}
#[inline]
pub unsafe fn RegisterClassExW(param0: *const WNDCLASSEXW) -> ATOM {
    windows_core::link!("user32.dll" "system" fn RegisterClassExW(param0 : *const WNDCLASSEXW) -> ATOM);
    unsafe { RegisterClassExW(param0) }
}
#[inline]
pub unsafe fn RegisterClipboardFormatW<P0>(lpszformat: P0) -> u32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn RegisterClipboardFormatW(lpszformat : windows_core::PCWSTR) -> u32);
    unsafe { RegisterClipboardFormatW(lpszformat.param().abi()) }
}
#[inline]
pub unsafe fn RegisterDragDrop<P1>(hwnd: HWND, pdroptarget: P1) -> windows_core::HRESULT
where
    P1: windows_core::Param<IDropTarget>,
{
    windows_core::link!("ole32.dll" "system" fn RegisterDragDrop(hwnd : HWND, pdroptarget : *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe { RegisterDragDrop(hwnd, pdroptarget.param().abi()) }
}
#[inline]
pub unsafe fn RegisterHotKey(
    hwnd: Option<HWND>,
    id: i32,
    fsmodifiers: u32,
    vk: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn RegisterHotKey(hwnd : HWND, id : i32, fsmodifiers : u32, vk : u32) -> windows_core::BOOL);
    unsafe {
        RegisterHotKey(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            id,
            fsmodifiers,
            vk,
        )
    }
}
#[inline]
pub unsafe fn RegisterRawInputDevices(
    prawinputdevices: &[RAWINPUTDEVICE],
    cbsize: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn RegisterRawInputDevices(prawinputdevices : *const RAWINPUTDEVICE, uinumdevices : u32, cbsize : u32) -> windows_core::BOOL);
    unsafe {
        RegisterRawInputDevices(
            prawinputdevices.as_ptr(),
            prawinputdevices.len().try_into().unwrap(),
            cbsize,
        )
    }
}
#[inline]
pub unsafe fn RegisterWindowMessageW<P0>(lpstring: P0) -> u32
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn RegisterWindowMessageW(lpstring : windows_core::PCWSTR) -> u32);
    unsafe { RegisterWindowMessageW(lpstring.param().abi()) }
}
#[inline]
pub unsafe fn ReleaseCapture() -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ReleaseCapture() -> windows_core::BOOL);
    unsafe { ReleaseCapture() }
}
#[inline]
pub unsafe fn ReleaseDC(hwnd: Option<HWND>, hdc: HDC) -> i32 {
    windows_core::link!("user32.dll" "system" fn ReleaseDC(hwnd : HWND, hdc : HDC) -> i32);
    unsafe { ReleaseDC(hwnd.unwrap_or(core::mem::zeroed()) as _, hdc) }
}
#[inline]
pub unsafe fn ReleaseStgMedium(param0: *const STGMEDIUM) {
    windows_core::link!("ole32.dll" "system" fn ReleaseStgMedium(param0 : *const STGMEDIUM));
    unsafe { ReleaseStgMedium(param0) }
}
#[inline]
pub unsafe fn RemoveMenu(hmenu: HMENU, uposition: u32, uflags: u32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn RemoveMenu(hmenu : HMENU, uposition : u32, uflags : u32) -> windows_core::BOOL);
    unsafe { RemoveMenu(hmenu, uposition, uflags) }
}
#[inline]
pub unsafe fn RemoveWindowSubclass(
    hwnd: HWND,
    pfnsubclass: SUBCLASSPROC,
    uidsubclass: usize,
) -> windows_core::BOOL {
    windows_core::link!("comctl32.dll" "system" fn RemoveWindowSubclass(hwnd : HWND, pfnsubclass : SUBCLASSPROC, uidsubclass : usize) -> windows_core::BOOL);
    unsafe { RemoveWindowSubclass(hwnd, pfnsubclass, uidsubclass) }
}
#[inline]
pub unsafe fn RevokeDragDrop(hwnd: HWND) -> windows_core::HRESULT {
    windows_core::link!("ole32.dll" "system" fn RevokeDragDrop(hwnd : HWND) -> windows_core::HRESULT);
    unsafe { RevokeDragDrop(hwnd) }
}
#[inline]
pub unsafe fn RtlCaptureStackBackTrace(
    framestoskip: u32,
    framestocapture: u32,
    backtrace: *mut *mut core::ffi::c_void,
    backtracehash: Option<*mut u32>,
) -> u16 {
    windows_core::link!("kernel32.dll" "system" fn RtlCaptureStackBackTrace(framestoskip : u32, framestocapture : u32, backtrace : *mut *mut core::ffi::c_void, backtracehash : *mut u32) -> u16);
    unsafe {
        RtlCaptureStackBackTrace(
            framestoskip,
            framestocapture,
            backtrace as _,
            backtracehash.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn RtlGetVersion(lpversioninformation: *mut OSVERSIONINFOW) -> windows_core::NTSTATUS {
    windows_core::link!("ntdll.dll" "system" fn RtlGetVersion(lpversioninformation : *mut OSVERSIONINFOW) -> windows_core::NTSTATUS);
    unsafe { RtlGetVersion(lpversioninformation as _) }
}
#[inline]
pub unsafe fn SHBindToParent<T>(
    pidl: *const ITEMIDLIST,
    ppidllast: *mut LPCITEMIDLIST,
) -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    windows_core::link!("shell32.dll" "system" fn SHBindToParent(pidl : *const ITEMIDLIST, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void, ppidllast : *mut LPCITEMIDLIST) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        SHBindToParent(pidl, &T::IID, &mut result__, ppidllast as _)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHChangeNotification_Lock(
    hchange: HANDLE,
    dwprocid: u32,
    pppidl: *mut *mut LPITEMIDLIST,
    plevent: Option<*mut i32>,
) -> HANDLE {
    windows_core::link!("shell32.dll" "system" fn SHChangeNotification_Lock(hchange : HANDLE, dwprocid : u32, pppidl : *mut *mut LPITEMIDLIST, plevent : *mut i32) -> HANDLE);
    unsafe {
        SHChangeNotification_Lock(
            hchange,
            dwprocid,
            pppidl as _,
            plevent.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn SHChangeNotification_Unlock(hlock: HANDLE) -> windows_core::BOOL {
    windows_core::link!("shell32.dll" "system" fn SHChangeNotification_Unlock(hlock : HANDLE) -> windows_core::BOOL);
    unsafe { SHChangeNotification_Unlock(hlock) }
}
#[inline]
pub unsafe fn SHChangeNotifyDeregister(ulid: u32) -> windows_core::BOOL {
    windows_core::link!("shell32.dll" "system" fn SHChangeNotifyDeregister(ulid : u32) -> windows_core::BOOL);
    unsafe { SHChangeNotifyDeregister(ulid) }
}
#[inline]
pub unsafe fn SHChangeNotifyRegister(
    hwnd: HWND,
    fsources: i32,
    fevents: i32,
    wmsg: u32,
    centries: i32,
    pshcne: *const SHChangeNotifyEntry,
) -> u32 {
    windows_core::link!("shell32.dll" "system" fn SHChangeNotifyRegister(hwnd : HWND, fsources : i32, fevents : i32, wmsg : u32, centries : i32, pshcne : *const SHChangeNotifyEntry) -> u32);
    unsafe { SHChangeNotifyRegister(hwnd, fsources, fevents, wmsg, centries, pshcne) }
}
#[inline]
pub unsafe fn SHCreateDefaultContextMenu<T>(pdcm: *const DEFCONTEXTMENU) -> windows_core::Result<T>
where
    T: windows_core::Interface,
{
    windows_core::link!("shell32.dll" "system" fn SHCreateDefaultContextMenu(pdcm : *const DEFCONTEXTMENU, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        SHCreateDefaultContextMenu(pdcm, &T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHCreateItemFromParsingName<P0, P1, T>(
    pszpath: P0,
    pbc: P1,
) -> windows_core::Result<T>
where
    P0: windows_core::Param<windows_core::PCWSTR>,
    P1: windows_core::Param<IBindCtx>,
    T: windows_core::Interface,
{
    windows_core::link!("shell32.dll" "system" fn SHCreateItemFromParsingName(pszpath : windows_core::PCWSTR, pbc : *mut core::ffi::c_void, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        SHCreateItemFromParsingName(
            pszpath.param().abi(),
            pbc.param().abi(),
            &T::IID,
            &mut result__,
        )
        .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHCreateShellItemArrayFromIDLists(
    rgpidl: &[LPCITEMIDLIST],
) -> windows_core::Result<IShellItemArray> {
    windows_core::link!("shell32.dll" "system" fn SHCreateShellItemArrayFromIDLists(cidl : u32, rgpidl : *const LPCITEMIDLIST, ppsiitemarray : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SHCreateShellItemArrayFromIDLists(
            rgpidl.len().try_into().unwrap(),
            rgpidl.as_ptr(),
            &mut result__,
        )
        .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHCreateShellItemArrayFromShellItem<P0, T>(psi: P0) -> windows_core::Result<T>
where
    P0: windows_core::Param<IShellItem>,
    T: windows_core::Interface,
{
    windows_core::link!("shell32.dll" "system" fn SHCreateShellItemArrayFromShellItem(psi : *mut core::ffi::c_void, riid : *const windows_core::GUID, ppv : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    let mut result__ = core::ptr::null_mut();
    unsafe {
        SHCreateShellItemArrayFromShellItem(psi.param().abi(), &T::IID, &mut result__)
            .and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHDefExtractIconW<P0>(
    psziconfile: P0,
    iindex: i32,
    uflags: u32,
    phiconlarge: Option<*mut HICON>,
    phiconsmall: Option<*mut HICON>,
    niconsize: u32,
) -> windows_core::HRESULT
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("shell32.dll" "system" fn SHDefExtractIconW(psziconfile : windows_core::PCWSTR, iindex : i32, uflags : u32, phiconlarge : *mut HICON, phiconsmall : *mut HICON, niconsize : u32) -> windows_core::HRESULT);
    unsafe {
        SHDefExtractIconW(
            psziconfile.param().abi(),
            iindex,
            uflags,
            phiconlarge.unwrap_or(core::mem::zeroed()) as _,
            phiconsmall.unwrap_or(core::mem::zeroed()) as _,
            niconsize,
        )
    }
}
#[inline]
pub unsafe fn SHDoDragDrop<P1, P2>(
    hwnd: Option<HWND>,
    pdata: P1,
    pdsrc: P2,
    dweffect: u32,
) -> windows_core::Result<u32>
where
    P1: windows_core::Param<IDataObject>,
    P2: windows_core::Param<IDropSource>,
{
    windows_core::link!("shell32.dll" "system" fn SHDoDragDrop(hwnd : HWND, pdata : *mut core::ffi::c_void, pdsrc : *mut core::ffi::c_void, dweffect : u32, pdweffect : *mut u32) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SHDoDragDrop(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            pdata.param().abi(),
            pdsrc.param().abi(),
            dweffect,
            &mut result__,
        )
        .map(|| result__)
    }
}
#[inline]
pub unsafe fn SHGetDesktopFolder() -> windows_core::Result<IShellFolder> {
    windows_core::link!("shell32.dll" "system" fn SHGetDesktopFolder(ppshf : *mut *mut core::ffi::c_void) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SHGetDesktopFolder(&mut result__).and_then(|| windows_core::imp::Type::from_abi(result__))
    }
}
#[inline]
pub unsafe fn SHGetFileInfoW<P0>(
    pszpath: P0,
    dwfileattributes: u32,
    psfi: Option<*mut SHFILEINFOW>,
    cbfileinfo: u32,
    uflags: u32,
) -> usize
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("shell32.dll" "system" fn SHGetFileInfoW(pszpath : windows_core::PCWSTR, dwfileattributes : u32, psfi : *mut SHFILEINFOW, cbfileinfo : u32, uflags : u32) -> usize);
    unsafe {
        SHGetFileInfoW(
            pszpath.param().abi(),
            dwfileattributes,
            psfi.unwrap_or(core::mem::zeroed()) as _,
            cbfileinfo,
            uflags,
        )
    }
}
#[inline]
pub unsafe fn SHGetKnownFolderPath(
    rfid: *const KNOWNFOLDERID,
    dwflags: u32,
    htoken: Option<HANDLE>,
) -> windows_core::Result<windows_core::PWSTR> {
    windows_core::link!("shell32.dll" "system" fn SHGetKnownFolderPath(rfid : *const KNOWNFOLDERID, dwflags : u32, htoken : HANDLE, ppszpath : *mut windows_core::PWSTR) -> windows_core::HRESULT);
    unsafe {
        let mut result__ = core::mem::zeroed();
        SHGetKnownFolderPath(
            rfid,
            dwflags,
            htoken.unwrap_or(core::mem::zeroed()) as _,
            &mut result__,
        )
        .map(|| result__)
    }
}
#[inline]
pub unsafe fn SHGetSetSettings(lpss: Option<*mut SHELLSTATEA>, dwmask: u32, bset: bool) {
    windows_core::link!("shell32.dll" "system" fn SHGetSetSettings(lpss : *mut SHELLSTATEA, dwmask : u32, bset : windows_core::BOOL));
    unsafe {
        SHGetSetSettings(
            lpss.unwrap_or(core::mem::zeroed()) as _,
            dwmask,
            bset.into(),
        )
    }
}
#[inline]
pub unsafe fn SHParseDisplayName<P0, P1>(
    pszname: P0,
    pbc: P1,
    ppidl: *mut LPITEMIDLIST,
    sfgaoin: SFGAOF,
    psfgaoout: Option<*mut SFGAOF>,
) -> windows_core::HRESULT
where
    P0: windows_core::Param<windows_core::PCWSTR>,
    P1: windows_core::Param<IBindCtx>,
{
    windows_core::link!("shell32.dll" "system" fn SHParseDisplayName(pszname : windows_core::PCWSTR, pbc : *mut core::ffi::c_void, ppidl : *mut LPITEMIDLIST, sfgaoin : SFGAOF, psfgaoout : *mut SFGAOF) -> windows_core::HRESULT);
    unsafe {
        SHParseDisplayName(
            pszname.param().abi(),
            pbc.param().abi(),
            ppidl as _,
            sfgaoin,
            psfgaoout.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn SHQueryRecycleBinW<P0>(
    pszrootpath: P0,
    pshqueryrbinfo: *mut SHQUERYRBINFO,
) -> windows_core::HRESULT
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("shell32.dll" "system" fn SHQueryRecycleBinW(pszrootpath : windows_core::PCWSTR, pshqueryrbinfo : *mut SHQUERYRBINFO) -> windows_core::HRESULT);
    unsafe { SHQueryRecycleBinW(pszrootpath.param().abi(), pshqueryrbinfo as _) }
}
#[inline]
pub unsafe fn ScreenToClient(hwnd: HWND, lppoint: *mut POINT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ScreenToClient(hwnd : HWND, lppoint : *mut POINT) -> windows_core::BOOL);
    unsafe { ScreenToClient(hwnd, lppoint as _) }
}
#[inline]
pub unsafe fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ {
    windows_core::link!("gdi32.dll" "system" fn SelectObject(hdc : HDC, h : HGDIOBJ) -> HGDIOBJ);
    unsafe { SelectObject(hdc, h) }
}
#[inline]
pub unsafe fn SendMessageTimeoutW(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    fuflags: u32,
    utimeout: u32,
    lpdwresult: Option<*mut usize>,
) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn SendMessageTimeoutW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM, fuflags : u32, utimeout : u32, lpdwresult : *mut usize) -> LRESULT);
    unsafe {
        SendMessageTimeoutW(
            hwnd,
            msg,
            wparam,
            lparam,
            fuflags,
            utimeout,
            lpdwresult.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn SendMessageW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    windows_core::link!("user32.dll" "system" fn SendMessageW(hwnd : HWND, msg : u32, wparam : WPARAM, lparam : LPARAM) -> LRESULT);
    unsafe { SendMessageW(hwnd, msg, wparam, lparam) }
}
#[inline]
pub unsafe fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF {
    windows_core::link!("gdi32.dll" "system" fn SetBkColor(hdc : HDC, color : COLORREF) -> COLORREF);
    unsafe { SetBkColor(hdc, color) }
}
#[inline]
pub unsafe fn SetCapture(hwnd: HWND) -> HWND {
    windows_core::link!("user32.dll" "system" fn SetCapture(hwnd : HWND) -> HWND);
    unsafe { SetCapture(hwnd) }
}
#[inline]
pub unsafe fn SetCoalescableTimer(
    hwnd: Option<HWND>,
    nidevent: usize,
    uelapse: u32,
    lptimerfunc: TIMERPROC,
    utolerancedelay: u32,
) -> usize {
    windows_core::link!("user32.dll" "system" fn SetCoalescableTimer(hwnd : HWND, nidevent : usize, uelapse : u32, lptimerfunc : TIMERPROC, utolerancedelay : u32) -> usize);
    unsafe {
        SetCoalescableTimer(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            nidevent,
            uelapse,
            lptimerfunc,
            utolerancedelay,
        )
    }
}
#[inline]
pub unsafe fn SetCursor(hcursor: Option<HCURSOR>) -> HCURSOR {
    windows_core::link!("user32.dll" "system" fn SetCursor(hcursor : HCURSOR) -> HCURSOR);
    unsafe { SetCursor(hcursor.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn SetCursorPos(x: i32, y: i32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetCursorPos(x : i32, y : i32) -> windows_core::BOOL);
    unsafe { SetCursorPos(x, y) }
}
#[inline]
pub unsafe fn SetEvent(hevent: HANDLE) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn SetEvent(hevent : HANDLE) -> windows_core::BOOL);
    unsafe { SetEvent(hevent) }
}
#[inline]
pub unsafe fn SetFocus(hwnd: Option<HWND>) -> HWND {
    windows_core::link!("user32.dll" "system" fn SetFocus(hwnd : HWND) -> HWND);
    unsafe { SetFocus(hwnd.unwrap_or(core::mem::zeroed()) as _) }
}
#[inline]
pub unsafe fn SetForegroundWindow(hwnd: HWND) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetForegroundWindow(hwnd : HWND) -> windows_core::BOOL);
    unsafe { SetForegroundWindow(hwnd) }
}
#[inline]
pub unsafe fn SetInformationJobObject(
    hjob: HANDLE,
    jobobjectinformationclass: JOBOBJECTINFOCLASS,
    lpjobobjectinformation: *const core::ffi::c_void,
    cbjobobjectinformationlength: u32,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn SetInformationJobObject(hjob : HANDLE, jobobjectinformationclass : JOBOBJECTINFOCLASS, lpjobobjectinformation : *const core::ffi::c_void, cbjobobjectinformationlength : u32) -> windows_core::BOOL);
    unsafe {
        SetInformationJobObject(
            hjob,
            jobobjectinformationclass,
            lpjobobjectinformation,
            cbjobobjectinformationlength,
        )
    }
}
#[inline]
pub unsafe fn SetLastError(dwerrcode: u32) {
    windows_core::link!("kernel32.dll" "system" fn SetLastError(dwerrcode : u32));
    unsafe { SetLastError(dwerrcode) }
}
#[inline]
pub unsafe fn SetLayeredWindowAttributes(
    hwnd: HWND,
    crkey: COLORREF,
    balpha: u8,
    dwflags: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetLayeredWindowAttributes(hwnd : HWND, crkey : COLORREF, balpha : u8, dwflags : u32) -> windows_core::BOOL);
    unsafe { SetLayeredWindowAttributes(hwnd, crkey, balpha, dwflags) }
}
#[inline]
pub unsafe fn SetProcessDpiAwarenessContext(value: DPI_AWARENESS_CONTEXT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetProcessDpiAwarenessContext(value : DPI_AWARENESS_CONTEXT) -> windows_core::BOOL);
    unsafe { SetProcessDpiAwarenessContext(value) }
}
#[inline]
pub unsafe fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF {
    windows_core::link!("gdi32.dll" "system" fn SetTextColor(hdc : HDC, color : COLORREF) -> COLORREF);
    unsafe { SetTextColor(hdc, color) }
}
#[inline]
pub unsafe fn SetTimer(
    hwnd: Option<HWND>,
    nidevent: usize,
    uelapse: u32,
    lptimerfunc: TIMERPROC,
) -> usize {
    windows_core::link!("user32.dll" "system" fn SetTimer(hwnd : HWND, nidevent : usize, uelapse : u32, lptimerfunc : TIMERPROC) -> usize);
    unsafe {
        SetTimer(
            hwnd.unwrap_or(core::mem::zeroed()) as _,
            nidevent,
            uelapse,
            lptimerfunc,
        )
    }
}
#[inline]
pub unsafe fn SetUnhandledExceptionFilter(
    lptoplevelexceptionfilter: Option<LPTOP_LEVEL_EXCEPTION_FILTER>,
) -> LPTOP_LEVEL_EXCEPTION_FILTER {
    windows_core::link!("kernel32.dll" "system" fn SetUnhandledExceptionFilter(lptoplevelexceptionfilter : LPTOP_LEVEL_EXCEPTION_FILTER) -> LPTOP_LEVEL_EXCEPTION_FILTER);
    unsafe {
        SetUnhandledExceptionFilter(lptoplevelexceptionfilter.unwrap_or(core::mem::zeroed()) as _)
    }
}
#[inline]
pub unsafe fn SetWinEventHook(
    eventmin: u32,
    eventmax: u32,
    hmodwineventproc: Option<HMODULE>,
    pfnwineventproc: WINEVENTPROC,
    idprocess: u32,
    idthread: u32,
    dwflags: u32,
) -> HWINEVENTHOOK {
    windows_core::link!("user32.dll" "system" fn SetWinEventHook(eventmin : u32, eventmax : u32, hmodwineventproc : HMODULE, pfnwineventproc : WINEVENTPROC, idprocess : u32, idthread : u32, dwflags : u32) -> HWINEVENTHOOK);
    unsafe {
        SetWinEventHook(
            eventmin,
            eventmax,
            hmodwineventproc.unwrap_or(core::mem::zeroed()) as _,
            pfnwineventproc,
            idprocess,
            idthread,
            dwflags,
        )
    }
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[inline]
pub unsafe fn SetWindowLongPtrW(hwnd: HWND, nindex: i32, dwnewlong: isize) -> isize {
    windows_core::link!("user32.dll" "system" fn SetWindowLongPtrW(hwnd : HWND, nindex : i32, dwnewlong : isize) -> isize);
    unsafe { SetWindowLongPtrW(hwnd, nindex, dwnewlong) }
}
#[cfg(target_pointer_width = "32")]
pub use SetWindowLongW as SetWindowLongPtrW;
#[inline]
pub unsafe fn SetWindowLongW(hwnd: HWND, nindex: i32, dwnewlong: i32) -> i32 {
    windows_core::link!("user32.dll" "system" fn SetWindowLongW(hwnd : HWND, nindex : i32, dwnewlong : i32) -> i32);
    unsafe { SetWindowLongW(hwnd, nindex, dwnewlong) }
}
#[inline]
pub unsafe fn SetWindowPos(
    hwnd: HWND,
    hwndinsertafter: Option<HWND>,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    uflags: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SetWindowPos(hwnd : HWND, hwndinsertafter : HWND, x : i32, y : i32, cx : i32, cy : i32, uflags : u32) -> windows_core::BOOL);
    unsafe {
        SetWindowPos(
            hwnd,
            hwndinsertafter.unwrap_or(core::mem::zeroed()) as _,
            x,
            y,
            cx,
            cy,
            uflags,
        )
    }
}
#[inline]
pub unsafe fn SetWindowSubclass(
    hwnd: HWND,
    pfnsubclass: SUBCLASSPROC,
    uidsubclass: usize,
    dwrefdata: usize,
) -> windows_core::BOOL {
    windows_core::link!("comctl32.dll" "system" fn SetWindowSubclass(hwnd : HWND, pfnsubclass : SUBCLASSPROC, uidsubclass : usize, dwrefdata : usize) -> windows_core::BOOL);
    unsafe { SetWindowSubclass(hwnd, pfnsubclass, uidsubclass, dwrefdata) }
}
#[inline]
pub unsafe fn SetWindowTextW<P1>(hwnd: HWND, lpstring: P1) -> windows_core::BOOL
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn SetWindowTextW(hwnd : HWND, lpstring : windows_core::PCWSTR) -> windows_core::BOOL);
    unsafe { SetWindowTextW(hwnd, lpstring.param().abi()) }
}
#[inline]
pub unsafe fn SetWindowTheme<P1, P2>(
    hwnd: HWND,
    pszsubappname: P1,
    pszsubidlist: P2,
) -> windows_core::HRESULT
where
    P1: windows_core::Param<windows_core::PCWSTR>,
    P2: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("uxtheme.dll" "system" fn SetWindowTheme(hwnd : HWND, pszsubappname : windows_core::PCWSTR, pszsubidlist : windows_core::PCWSTR) -> windows_core::HRESULT);
    unsafe {
        SetWindowTheme(
            hwnd,
            pszsubappname.param().abi(),
            pszsubidlist.param().abi(),
        )
    }
}
#[inline]
pub unsafe fn ShellExecuteExW(pexecinfo: *mut SHELLEXECUTEINFOW) -> windows_core::BOOL {
    windows_core::link!("shell32.dll" "system" fn ShellExecuteExW(pexecinfo : *mut SHELLEXECUTEINFOW) -> windows_core::BOOL);
    unsafe { ShellExecuteExW(pexecinfo as _) }
}
#[inline]
pub unsafe fn Shell_NotifyIconW(
    dwmessage: u32,
    lpdata: *const NOTIFYICONDATAW,
) -> windows_core::BOOL {
    windows_core::link!("shell32.dll" "system" fn Shell_NotifyIconW(dwmessage : u32, lpdata : *const NOTIFYICONDATAW) -> windows_core::BOOL);
    unsafe { Shell_NotifyIconW(dwmessage, lpdata) }
}
#[inline]
pub unsafe fn ShowWindow(hwnd: HWND, ncmdshow: i32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn ShowWindow(hwnd : HWND, ncmdshow : i32) -> windows_core::BOOL);
    unsafe { ShowWindow(hwnd, ncmdshow) }
}
#[inline]
pub unsafe fn StackWalk64(
    machinetype: u32,
    hprocess: HANDLE,
    hthread: HANDLE,
    stackframe: *mut STACKFRAME64,
    contextrecord: *mut core::ffi::c_void,
    readmemoryroutine: PREAD_PROCESS_MEMORY_ROUTINE64,
    functiontableaccessroutine: PFUNCTION_TABLE_ACCESS_ROUTINE64,
    getmodulebaseroutine: PGET_MODULE_BASE_ROUTINE64,
    translateaddress: PTRANSLATE_ADDRESS_ROUTINE64,
) -> windows_core::BOOL {
    windows_core::link!("dbghelp.dll" "system" fn StackWalk64(machinetype : u32, hprocess : HANDLE, hthread : HANDLE, stackframe : *mut STACKFRAME64, contextrecord : *mut core::ffi::c_void, readmemoryroutine : PREAD_PROCESS_MEMORY_ROUTINE64, functiontableaccessroutine : PFUNCTION_TABLE_ACCESS_ROUTINE64, getmodulebaseroutine : PGET_MODULE_BASE_ROUTINE64, translateaddress : PTRANSLATE_ADDRESS_ROUTINE64) -> windows_core::BOOL);
    unsafe {
        StackWalk64(
            machinetype,
            hprocess,
            hthread,
            stackframe as _,
            contextrecord as _,
            readmemoryroutine,
            functiontableaccessroutine,
            getmodulebaseroutine,
            translateaddress,
        )
    }
}
#[inline]
pub unsafe fn SymFunctionTableAccess64(hprocess: HANDLE, addrbase: u64) -> *mut core::ffi::c_void {
    windows_core::link!("dbghelp.dll" "system" fn SymFunctionTableAccess64(hprocess : HANDLE, addrbase : u64) -> *mut core::ffi::c_void);
    unsafe { SymFunctionTableAccess64(hprocess, addrbase) }
}
#[inline]
pub unsafe fn SymGetModuleBase64(hprocess: HANDLE, qwaddr: u64) -> u64 {
    windows_core::link!("dbghelp.dll" "system" fn SymGetModuleBase64(hprocess : HANDLE, qwaddr : u64) -> u64);
    unsafe { SymGetModuleBase64(hprocess, qwaddr) }
}
#[inline]
pub unsafe fn SymInitializeW<P1>(
    hprocess: HANDLE,
    usersearchpath: P1,
    finvadeprocess: bool,
) -> windows_core::BOOL
where
    P1: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("dbghelp.dll" "system" fn SymInitializeW(hprocess : HANDLE, usersearchpath : windows_core::PCWSTR, finvadeprocess : windows_core::BOOL) -> windows_core::BOOL);
    unsafe {
        SymInitializeW(
            hprocess,
            usersearchpath.param().abi(),
            finvadeprocess.into(),
        )
    }
}
#[inline]
pub unsafe fn SystemParametersInfoForDpi(
    uiaction: u32,
    uiparam: u32,
    pvparam: *mut core::ffi::c_void,
    fwinini: u32,
    dpi: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SystemParametersInfoForDpi(uiaction : u32, uiparam : u32, pvparam : *mut core::ffi::c_void, fwinini : u32, dpi : u32) -> windows_core::BOOL);
    unsafe { SystemParametersInfoForDpi(uiaction, uiparam, pvparam as _, fwinini, dpi) }
}
#[inline]
pub unsafe fn SystemParametersInfoW(
    uiaction: u32,
    uiparam: u32,
    pvparam: *mut core::ffi::c_void,
    fwinini: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn SystemParametersInfoW(uiaction : u32, uiparam : u32, pvparam : *mut core::ffi::c_void, fwinini : u32) -> windows_core::BOOL);
    unsafe { SystemParametersInfoW(uiaction, uiparam, pvparam as _, fwinini) }
}
#[inline]
pub unsafe fn SystemTimeToTzSpecificLocalTime(
    lptimezoneinformation: Option<*const TIME_ZONE_INFORMATION>,
    lpuniversaltime: *const SYSTEMTIME,
    lplocaltime: *mut SYSTEMTIME,
) -> windows_core::BOOL {
    windows_core::link!("kernel32.dll" "system" fn SystemTimeToTzSpecificLocalTime(lptimezoneinformation : *const TIME_ZONE_INFORMATION, lpuniversaltime : *const SYSTEMTIME, lplocaltime : *mut SYSTEMTIME) -> windows_core::BOOL);
    unsafe {
        SystemTimeToTzSpecificLocalTime(
            lptimezoneinformation.unwrap_or(core::mem::zeroed()) as _,
            lpuniversaltime,
            lplocaltime as _,
        )
    }
}
#[inline]
pub unsafe fn TrackMouseEvent(lpeventtrack: *mut TRACKMOUSEEVENT) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn TrackMouseEvent(lpeventtrack : *mut TRACKMOUSEEVENT) -> windows_core::BOOL);
    unsafe { TrackMouseEvent(lpeventtrack as _) }
}
#[inline]
pub unsafe fn TrackPopupMenuEx(
    hmenu: HMENU,
    uflags: u32,
    x: i32,
    y: i32,
    hwnd: HWND,
    lptpm: Option<*const TPMPARAMS>,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn TrackPopupMenuEx(hmenu : HMENU, uflags : u32, x : i32, y : i32, hwnd : HWND, lptpm : *const TPMPARAMS) -> windows_core::BOOL);
    unsafe {
        TrackPopupMenuEx(
            hmenu,
            uflags,
            x,
            y,
            hwnd,
            lptpm.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn TranslateMessage(lpmsg: *const MSG) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn TranslateMessage(lpmsg : *const MSG) -> windows_core::BOOL);
    unsafe { TranslateMessage(lpmsg) }
}
#[inline]
pub unsafe fn UnhookWinEvent(hwineventhook: HWINEVENTHOOK) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn UnhookWinEvent(hwineventhook : HWINEVENTHOOK) -> windows_core::BOOL);
    unsafe { UnhookWinEvent(hwineventhook) }
}
#[inline]
pub unsafe fn UnregisterClassW<P0>(
    lpclassname: P0,
    hinstance: Option<HINSTANCE>,
) -> windows_core::BOOL
where
    P0: windows_core::Param<windows_core::PCWSTR>,
{
    windows_core::link!("user32.dll" "system" fn UnregisterClassW(lpclassname : windows_core::PCWSTR, hinstance : HINSTANCE) -> windows_core::BOOL);
    unsafe {
        UnregisterClassW(
            lpclassname.param().abi(),
            hinstance.unwrap_or(core::mem::zeroed()) as _,
        )
    }
}
#[inline]
pub unsafe fn UnregisterHotKey(hwnd: Option<HWND>, id: i32) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn UnregisterHotKey(hwnd : HWND, id : i32) -> windows_core::BOOL);
    unsafe { UnregisterHotKey(hwnd.unwrap_or(core::mem::zeroed()) as _, id) }
}
#[inline]
pub unsafe fn UpdateLayeredWindow(
    hwnd: HWND,
    hdcdst: Option<HDC>,
    pptdst: Option<*const POINT>,
    psize: Option<*const SIZE>,
    hdcsrc: Option<HDC>,
    pptsrc: Option<*const POINT>,
    crkey: COLORREF,
    pblend: Option<*const BLENDFUNCTION>,
    dwflags: u32,
) -> windows_core::BOOL {
    windows_core::link!("user32.dll" "system" fn UpdateLayeredWindow(hwnd : HWND, hdcdst : HDC, pptdst : *const POINT, psize : *const SIZE, hdcsrc : HDC, pptsrc : *const POINT, crkey : COLORREF, pblend : *const BLENDFUNCTION, dwflags : u32) -> windows_core::BOOL);
    unsafe {
        UpdateLayeredWindow(
            hwnd,
            hdcdst.unwrap_or(core::mem::zeroed()) as _,
            pptdst.unwrap_or(core::mem::zeroed()) as _,
            psize.unwrap_or(core::mem::zeroed()) as _,
            hdcsrc.unwrap_or(core::mem::zeroed()) as _,
            pptsrc.unwrap_or(core::mem::zeroed()) as _,
            crkey,
            pblend.unwrap_or(core::mem::zeroed()) as _,
            dwflags,
        )
    }
}
#[inline]
pub unsafe fn WaitForMultipleObjects(
    lphandles: &[HANDLE],
    bwaitall: bool,
    dwmilliseconds: u32,
) -> u32 {
    windows_core::link!("kernel32.dll" "system" fn WaitForMultipleObjects(ncount : u32, lphandles : *const HANDLE, bwaitall : windows_core::BOOL, dwmilliseconds : u32) -> u32);
    unsafe {
        WaitForMultipleObjects(
            lphandles.len().try_into().unwrap(),
            lphandles.as_ptr(),
            bwaitall.into(),
            dwmilliseconds,
        )
    }
}
#[inline]
pub unsafe fn WaitForSingleObject(hhandle: HANDLE, dwmilliseconds: u32) -> u32 {
    windows_core::link!("kernel32.dll" "system" fn WaitForSingleObject(hhandle : HANDLE, dwmilliseconds : u32) -> u32);
    unsafe { WaitForSingleObject(hhandle, dwmilliseconds) }
}
#[inline]
pub unsafe fn WindowFromPoint(point: POINT) -> HWND {
    windows_core::link!("user32.dll" "system" fn WindowFromPoint(point : POINT) -> HWND);
    unsafe { WindowFromPoint(point) }
}
#[inline]
pub unsafe fn keybd_event(bvk: u8, bscan: u8, dwflags: u32, dwextrainfo: usize) {
    windows_core::link!("user32.dll" "system" fn keybd_event(bvk : u8, bscan : u8, dwflags : u32, dwextrainfo : usize));
    unsafe { keybd_event(bvk, bscan, dwflags, dwextrainfo) }
}
#[inline]
pub unsafe fn mouse_event(dwflags: u32, dx: u32, dy: u32, dwdata: u32, dwextrainfo: usize) {
    windows_core::link!("user32.dll" "system" fn mouse_event(dwflags : u32, dx : u32, dy : u32, dwdata : u32, dwextrainfo : usize));
    unsafe { mouse_event(dwflags, dx, dy, dwdata, dwextrainfo) }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ACCESS_MASK(pub u32);
pub const AC_SRC_ALPHA: i32 = 1;
pub const AC_SRC_OVER: i32 = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ADDRESS64 {
    pub Offset: u64,
    pub Segment: u16,
    pub Mode: ADDRESS_MODE,
}
pub type ADDRESS_MODE = i32;
#[repr(C, align(16))]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct ARM64_NT_CONTEXT {
    pub ContextFlags: u32,
    pub Cpsr: u32,
    pub Anonymous: ARM64_NT_CONTEXT_0,
    pub Sp: u64,
    pub Pc: u64,
    pub V: [ARM64_NT_NEON128; 32],
    pub Fpcr: u32,
    pub Fpsr: u32,
    pub Bcr: [u32; 8],
    pub Bvr: [u64; 8],
    pub Wcr: [u32; 2],
    pub Wvr: [u64; 2],
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
impl Default for ARM64_NT_CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union ARM64_NT_CONTEXT_0 {
    pub Anonymous: ARM64_NT_CONTEXT_0_0,
    pub X: [u64; 31],
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
impl Default for ARM64_NT_CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_NT_CONTEXT_0_0 {
    pub X0: u64,
    pub X1: u64,
    pub X2: u64,
    pub X3: u64,
    pub X4: u64,
    pub X5: u64,
    pub X6: u64,
    pub X7: u64,
    pub X8: u64,
    pub X9: u64,
    pub X10: u64,
    pub X11: u64,
    pub X12: u64,
    pub X13: u64,
    pub X14: u64,
    pub X15: u64,
    pub X16: u64,
    pub X17: u64,
    pub X18: u64,
    pub X19: u64,
    pub X20: u64,
    pub X21: u64,
    pub X22: u64,
    pub X23: u64,
    pub X24: u64,
    pub X25: u64,
    pub X26: u64,
    pub X27: u64,
    pub X28: u64,
    pub Fp: u64,
    pub Lr: u64,
}
#[repr(C, align(16))]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub struct ARM64_NT_CONTEXT {
    pub ContextFlags: u32,
    pub Cpsr: u32,
    pub Anonymous: ARM64_NT_CONTEXT_0,
    pub Sp: u64,
    pub Pc: u64,
    pub V: [NEON128; 32],
    pub Fpcr: u32,
    pub Fpsr: u32,
    pub Bcr: [u32; 8],
    pub Bvr: [u64; 8],
    pub Wcr: [u32; 2],
    pub Wvr: [u64; 2],
}
#[cfg(target_arch = "aarch64")]
impl Default for ARM64_NT_CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub union ARM64_NT_CONTEXT_0 {
    pub Anonymous: ARM64_NT_CONTEXT_0_0,
    pub X: [u64; 31],
}
#[cfg(target_arch = "aarch64")]
impl Default for ARM64_NT_CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_NT_CONTEXT_0_0 {
    pub X0: u64,
    pub X1: u64,
    pub X2: u64,
    pub X3: u64,
    pub X4: u64,
    pub X5: u64,
    pub X6: u64,
    pub X7: u64,
    pub X8: u64,
    pub X9: u64,
    pub X10: u64,
    pub X11: u64,
    pub X12: u64,
    pub X13: u64,
    pub X14: u64,
    pub X15: u64,
    pub X16: u64,
    pub X17: u64,
    pub X18: u64,
    pub X19: u64,
    pub X20: u64,
    pub X21: u64,
    pub X22: u64,
    pub X23: u64,
    pub X24: u64,
    pub X25: u64,
    pub X26: u64,
    pub X27: u64,
    pub X28: u64,
    pub Fp: u64,
    pub Lr: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union ARM64_NT_NEON128 {
    pub Anonymous: ARM64_NT_NEON128_0,
    pub D: [f64; 2],
    pub S: [f32; 4],
    pub H: [u16; 8],
    pub B: [u8; 16],
}
impl Default for ARM64_NT_NEON128 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ARM64_NT_NEON128_0 {
    pub Low: u64,
    pub High: i64,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ATOM(pub u16);
pub const AddrMode1616: ADDRESS_MODE = 0;
pub const AddrMode1632: ADDRESS_MODE = 1;
pub const AddrModeFlat: ADDRESS_MODE = 3;
pub const AddrModeReal: ADDRESS_MODE = 2;
pub const BHID_DataObject: windows_core::GUID =
    windows_core::GUID::from_u128(0xb8c0bd9f_ed24_455c_83e6_d5390c4fe8c4);
pub const BHID_SFUIObject: windows_core::GUID =
    windows_core::GUID::from_u128(0x3981e225_f559_11d3_8e3a_00c04f6837d5);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BITMAP {
    pub bmType: i32,
    pub bmWidth: i32,
    pub bmHeight: i32,
    pub bmWidthBytes: i32,
    pub bmPlanes: u16,
    pub bmBitsPixel: u16,
    pub bmBits: *mut core::ffi::c_void,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}
impl Default for BITMAPINFO {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}
pub const BI_RGB: i32 = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BLENDFUNCTION {
    pub BlendOp: u8,
    pub BlendFlags: u8,
    pub SourceConstantAlpha: u8,
    pub AlphaFormat: u8,
}
pub const CF_HDROP: i32 = 15;
pub const CLEARTYPE_QUALITY: i32 = 5;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CLIPFORMAT(pub u16);
pub const CLIP_DEFAULT_PRECIS: i32 = 0;
pub type CLSCTX = u32;
pub const CLSCTX_ALL: i32 = 23;
pub const CLSCTX_INPROC_SERVER: CLSCTX = 1;
pub const CLSID_WICImagingFactory: windows_core::GUID =
    windows_core::GUID::from_u128(0xcacaf262_9370_4615_a13b_9f5539da4c0a);
pub const CMF_CANRENAME: i32 = 16;
pub const CMF_EXPLORE: i32 = 4;
pub const CMF_EXTENDEDVERBS: i32 = 256;
pub const CMF_NORMAL: i32 = 0;
pub const CMIC_MASK_PTINVOKE: i32 = 536870912;
pub const CMIC_MASK_SHIFT_DOWN: i32 = 268435456;
pub const CMIC_MASK_UNICODE: i32 = 16384;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CMINVOKECOMMANDINFO {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: HWND,
    pub lpVerb: windows_core::PCSTR,
    pub lpParameters: windows_core::PCSTR,
    pub lpDirectory: windows_core::PCSTR,
    pub nShow: i32,
    pub dwHotKey: u32,
    pub hIcon: HANDLE,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CMINVOKECOMMANDINFOEX {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: HWND,
    pub lpVerb: windows_core::PCSTR,
    pub lpParameters: windows_core::PCSTR,
    pub lpDirectory: windows_core::PCSTR,
    pub nShow: i32,
    pub dwHotKey: u32,
    pub hIcon: HANDLE,
    pub lpTitle: windows_core::PCSTR,
    pub lpVerbW: windows_core::PCWSTR,
    pub lpParametersW: windows_core::PCWSTR,
    pub lpDirectoryW: windows_core::PCWSTR,
    pub lpTitleW: windows_core::PCWSTR,
    pub ptInvoke: POINT,
}
pub type COINIT = i32;
pub const COINIT_APARTMENTTHREADED: COINIT = 2;
pub const COINIT_DISABLE_OLE1DDE: COINIT = 4;
pub const COINIT_MULTITHREADED: COINIT = 0;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct COLORREF(pub u32);
pub const COLOR_HIGHLIGHT: i32 = 13;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct COMDLG_FILTERSPEC {
    pub pszName: windows_core::PCWSTR,
    pub pszSpec: windows_core::PCWSTR,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CONTEXT {
    pub ContextFlags: u32,
    pub Dr0: u32,
    pub Dr1: u32,
    pub Dr2: u32,
    pub Dr3: u32,
    pub Dr6: u32,
    pub Dr7: u32,
    pub FloatSave: FLOATING_SAVE_AREA,
    pub SegGs: u32,
    pub SegFs: u32,
    pub SegEs: u32,
    pub SegDs: u32,
    pub Edi: u32,
    pub Esi: u32,
    pub Ebx: u32,
    pub Edx: u32,
    pub Ecx: u32,
    pub Eax: u32,
    pub Ebp: u32,
    pub Eip: u32,
    pub SegCs: u32,
    pub EFlags: u32,
    pub Esp: u32,
    pub SegSs: u32,
    pub ExtendedRegisters: [u8; 512],
}
#[cfg(target_arch = "x86")]
impl Default for CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct CONTEXT {
    pub P1Home: u64,
    pub P2Home: u64,
    pub P3Home: u64,
    pub P4Home: u64,
    pub P5Home: u64,
    pub P6Home: u64,
    pub ContextFlags: u32,
    pub MxCsr: u32,
    pub SegCs: u16,
    pub SegDs: u16,
    pub SegEs: u16,
    pub SegFs: u16,
    pub SegGs: u16,
    pub SegSs: u16,
    pub EFlags: u32,
    pub Dr0: u64,
    pub Dr1: u64,
    pub Dr2: u64,
    pub Dr3: u64,
    pub Dr6: u64,
    pub Dr7: u64,
    pub Rax: u64,
    pub Rcx: u64,
    pub Rdx: u64,
    pub Rbx: u64,
    pub Rsp: u64,
    pub Rbp: u64,
    pub Rsi: u64,
    pub Rdi: u64,
    pub R8: u64,
    pub R9: u64,
    pub R10: u64,
    pub R11: u64,
    pub R12: u64,
    pub R13: u64,
    pub R14: u64,
    pub R15: u64,
    pub Rip: u64,
    pub Anonymous: CONTEXT_0,
    pub VectorRegister: [M128A; 26],
    pub VectorControl: u64,
    pub DebugControl: u64,
    pub LastBranchToRip: u64,
    pub LastBranchFromRip: u64,
    pub LastExceptionToRip: u64,
    pub LastExceptionFromRip: u64,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for CONTEXT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub union CONTEXT_0 {
    pub FltSave: XMM_SAVE_AREA32,
    pub Anonymous: CONTEXT_0_0,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for CONTEXT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CONTEXT_0_0 {
    pub Header: [M128A; 2],
    pub Legacy: [M128A; 8],
    pub Xmm0: M128A,
    pub Xmm1: M128A,
    pub Xmm2: M128A,
    pub Xmm3: M128A,
    pub Xmm4: M128A,
    pub Xmm5: M128A,
    pub Xmm6: M128A,
    pub Xmm7: M128A,
    pub Xmm8: M128A,
    pub Xmm9: M128A,
    pub Xmm10: M128A,
    pub Xmm11: M128A,
    pub Xmm12: M128A,
    pub Xmm13: M128A,
    pub Xmm14: M128A,
    pub Xmm15: M128A,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for CONTEXT_0_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[cfg(target_arch = "aarch64")]
pub type CONTEXT = ARM64_NT_CONTEXT;
pub const CREATE_ALWAYS: i32 = 2;
pub const CREATE_NO_WINDOW: i32 = 134217728;
pub const CS_DBLCLKS: i32 = 8;
pub const CS_HREDRAW: i32 = 2;
pub const CS_VREDRAW: i32 = 1;
pub const CW_USEDEFAULT: i32 = -2147483648;
pub type D2D1_ALPHA_MODE = i32;
pub type D2D1_ANTIALIAS_MODE = i32;
pub const D2D1_ANTIALIAS_MODE_ALIASED: D2D1_ANTIALIAS_MODE = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_BITMAP_BRUSH_PROPERTIES {
    pub extendModeX: D2D1_EXTEND_MODE,
    pub extendModeY: D2D1_EXTEND_MODE,
    pub interpolationMode: D2D1_BITMAP_INTERPOLATION_MODE,
}
pub type D2D1_BITMAP_INTERPOLATION_MODE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_BITMAP_PROPERTIES {
    pub pixelFormat: D2D1_PIXEL_FORMAT,
    pub dpiX: f32,
    pub dpiY: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_BRUSH_PROPERTIES {
    pub opacity: f32,
    pub transform: windows_numerics::Matrix3x2,
}
pub type D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS = u32;
pub type D2D1_DRAW_TEXT_OPTIONS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_ELLIPSE {
    pub point: windows_numerics::Vector2,
    pub radiusX: f32,
    pub radiusY: f32,
}
pub type D2D1_EXTEND_MODE = i32;
pub type D2D1_FEATURE_LEVEL = i32;
pub type D2D1_GAMMA = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_GRADIENT_STOP {
    pub position: f32,
    pub color: D2D_COLOR_F,
}
pub type D2D1_LAYER_OPTIONS = u32;
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct D2D1_LAYER_PARAMETERS {
    pub contentBounds: D2D_RECT_F,
    pub geometricMask: core::mem::ManuallyDrop<Option<ID2D1Geometry>>,
    pub maskAntialiasMode: D2D1_ANTIALIAS_MODE,
    pub maskTransform: windows_numerics::Matrix3x2,
    pub opacity: f32,
    pub opacityBrush: core::mem::ManuallyDrop<Option<ID2D1Brush>>,
    pub layerOptions: D2D1_LAYER_OPTIONS,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
    pub startPoint: windows_numerics::Vector2,
    pub endPoint: windows_numerics::Vector2,
}
pub type D2D1_OPACITY_MASK_CONTENT = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D1_PIXEL_FORMAT {
    pub format: DXGI_FORMAT,
    pub alphaMode: D2D1_ALPHA_MODE,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES {
    pub center: windows_numerics::Vector2,
    pub gradientOriginOffset: windows_numerics::Vector2,
    pub radiusX: f32,
    pub radiusY: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_RENDER_TARGET_PROPERTIES {
    pub r#type: D2D1_RENDER_TARGET_TYPE,
    pub pixelFormat: D2D1_PIXEL_FORMAT,
    pub dpiX: f32,
    pub dpiY: f32,
    pub usage: D2D1_RENDER_TARGET_USAGE,
    pub minLevel: D2D1_FEATURE_LEVEL,
}
pub type D2D1_RENDER_TARGET_TYPE = i32;
pub type D2D1_RENDER_TARGET_USAGE = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D1_ROUNDED_RECT {
    pub rect: D2D_RECT_F,
    pub radiusX: f32,
    pub radiusY: f32,
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct D2D1_TAG(pub u64);
pub type D2D1_TEXT_ANTIALIAS_MODE = i32;
pub type D2D_COLOR_F = D3DCOLORVALUE;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D_RECT_F {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D2D_SIZE_F {
    pub width: f32,
    pub height: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct D2D_SIZE_U {
    pub width: u32,
    pub height: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct D3DCOLORVALUE {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
pub const DATE_SHORTDATE: i32 = 1;
pub const DEFAULT_CHARSET: i32 = 1;
pub const DEFAULT_PITCH: i32 = 0;
#[repr(C)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DEFCONTEXTMENU {
    pub hwnd: HWND,
    pub pcmcb: core::mem::ManuallyDrop<Option<IContextMenuCB>>,
    pub pidlFolder: LPCITEMIDLIST,
    pub psf: core::mem::ManuallyDrop<Option<IShellFolder>>,
    pub cidl: u32,
    pub apidl: *mut LPCITEMIDLIST,
    pub punkAssociationInfo: core::mem::ManuallyDrop<Option<windows_core::IUnknown>>,
    pub cKeys: u32,
    pub aKeys: *const HKEY,
}
pub type DESKTOP_SLIDESHOW_DIRECTION = i32;
pub type DESKTOP_SLIDESHOW_OPTIONS = u32;
pub type DESKTOP_SLIDESHOW_STATE = u32;
pub type DESKTOP_WALLPAPER_POSITION = i32;
pub const DIB_RGB_COLORS: i32 = 0;
pub type DISPATCHERQUEUE_THREAD_APARTMENTTYPE = i32;
pub type DISPATCHERQUEUE_THREAD_TYPE = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DISPLAY_DEVICEW {
    pub cb: u32,
    pub DeviceName: [u16; 32],
    pub DeviceString: [u16; 128],
    pub StateFlags: u32,
    pub DeviceID: [u16; 128],
    pub DeviceKey: [u16; 128],
}
impl Default for DISPLAY_DEVICEW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DPI_AWARENESS_CONTEXT(pub *mut core::ffi::c_void);
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: DPI_AWARENESS_CONTEXT =
    DPI_AWARENESS_CONTEXT(-4 as _);
pub const DQTAT_COM_ASTA: DISPATCHERQUEUE_THREAD_APARTMENTTYPE = 1;
pub const DQTAT_COM_NONE: DISPATCHERQUEUE_THREAD_APARTMENTTYPE = 0;
pub const DQTAT_COM_STA: DISPATCHERQUEUE_THREAD_APARTMENTTYPE = 2;
pub const DQTYPE_THREAD_CURRENT: DISPATCHERQUEUE_THREAD_TYPE = 2;
pub const DQTYPE_THREAD_DEDICATED: DISPATCHERQUEUE_THREAD_TYPE = 1;
pub const DRAGDROP_S_CANCEL: windows_core::HRESULT = windows_core::HRESULT(0x40101_u32 as _);
pub const DRAGDROP_S_DROP: windows_core::HRESULT = windows_core::HRESULT(0x40100_u32 as _);
pub const DRIVE_FIXED: i32 = 3;
pub const DROPEFFECT_COPY: i32 = 1;
pub const DROPEFFECT_LINK: i32 = 4;
pub const DROPEFFECT_MOVE: i32 = 2;
pub const DROPEFFECT_NONE: i32 = 0;
#[repr(C, packed(1))]
#[derive(Clone, Copy, Default)]
pub struct DROPFILES {
    pub pFiles: u32,
    pub pt: POINT,
    pub fNC: windows_core::BOOL,
    pub fWide: windows_core::BOOL,
}
pub type DVASPECT = i32;
pub const DVASPECT_CONTENT: DVASPECT = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DVTARGETDEVICE {
    pub tdSize: u32,
    pub tdDriverNameOffset: u16,
    pub tdDeviceNameOffset: u16,
    pub tdPortNameOffset: u16,
    pub tdExtDevmodeOffset: u16,
    pub tdData: [u8; 1],
}
impl Default for DVTARGETDEVICE {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type DWMNCRENDERINGPOLICY = i32;
pub const DWMNCRP_DISABLED: DWMNCRENDERINGPOLICY = 1;
pub const DWMNCRP_ENABLED: DWMNCRENDERINGPOLICY = 2;
pub const DWMSBT_AUTO: DWM_SYSTEMBACKDROP_TYPE = 0;
pub const DWMSBT_MAINWINDOW: DWM_SYSTEMBACKDROP_TYPE = 2;
pub const DWMSBT_NONE: DWM_SYSTEMBACKDROP_TYPE = 1;
pub const DWMSBT_TABBEDWINDOW: DWM_SYSTEMBACKDROP_TYPE = 4;
pub const DWMSBT_TRANSIENTWINDOW: DWM_SYSTEMBACKDROP_TYPE = 3;
pub const DWMWA_BORDER_COLOR: DWMWINDOWATTRIBUTE = 34;
pub const DWMWA_CAPTION_COLOR: DWMWINDOWATTRIBUTE = 35;
pub const DWMWA_CLOAK: DWMWINDOWATTRIBUTE = 13;
pub const DWMWA_CLOAKED: DWMWINDOWATTRIBUTE = 14;
pub const DWMWA_COLOR_DEFAULT: u32 = 4294967295;
pub const DWMWA_COLOR_NONE: u32 = 4294967294;
pub const DWMWA_EXCLUDED_FROM_PEEK: DWMWINDOWATTRIBUTE = 12;
pub const DWMWA_NCRENDERING_POLICY: DWMWINDOWATTRIBUTE = 2;
pub const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = 38;
pub const DWMWA_TEXT_COLOR: DWMWINDOWATTRIBUTE = 36;
pub const DWMWA_TRANSITIONS_FORCEDISABLED: DWMWINDOWATTRIBUTE = 3;
pub const DWMWA_USE_HOSTBACKDROPBRUSH: DWMWINDOWATTRIBUTE = 17;
pub const DWMWA_USE_IMMERSIVE_DARK_MODE: DWMWINDOWATTRIBUTE = 20;
pub const DWMWA_WINDOW_CORNER_PREFERENCE: DWMWINDOWATTRIBUTE = 33;
pub const DWMWCP_DEFAULT: DWM_WINDOW_CORNER_PREFERENCE = 0;
pub const DWMWCP_DONOTROUND: DWM_WINDOW_CORNER_PREFERENCE = 1;
pub const DWMWCP_ROUND: DWM_WINDOW_CORNER_PREFERENCE = 2;
pub const DWMWCP_ROUNDSMALL: DWM_WINDOW_CORNER_PREFERENCE = 3;
pub type DWMWINDOWATTRIBUTE = i32;
pub type DWM_SYSTEMBACKDROP_TYPE = i32;
pub type DWM_WINDOW_CORNER_PREFERENCE = i32;
pub const DWPOS_CENTER: DESKTOP_WALLPAPER_POSITION = 0;
pub const DWPOS_FILL: DESKTOP_WALLPAPER_POSITION = 4;
pub const DWPOS_FIT: DESKTOP_WALLPAPER_POSITION = 3;
pub const DWPOS_SPAN: DESKTOP_WALLPAPER_POSITION = 5;
pub const DWPOS_STRETCH: DESKTOP_WALLPAPER_POSITION = 2;
pub const DWPOS_TILE: DESKTOP_WALLPAPER_POSITION = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_OFFSET {
    pub advanceOffset: f32,
    pub ascenderOffset: f32,
}
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DWRITE_GLYPH_RUN {
    pub fontFace: core::mem::ManuallyDrop<Option<IDWriteFontFace>>,
    pub fontEmSize: f32,
    pub glyphCount: u32,
    pub glyphIndices: *const u16,
    pub glyphAdvances: *const f32,
    pub glyphOffsets: *const DWRITE_GLYPH_OFFSET,
    pub isSideways: windows_core::BOOL,
    pub bidiLevel: u32,
}
pub type DWRITE_MEASURING_MODE = i32;
pub type DXGI_FORMAT = i32;
pub const DesktopWallpaper: windows_core::GUID =
    windows_core::GUID::from_u128(0xc2cf3110_460e_4fc1_b9d0_8a1c0c9cc4bd);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DispatcherQueueOptions {
    pub dwSize: u32,
    pub threadType: DISPATCHERQUEUE_THREAD_TYPE,
    pub apartmentType: DISPATCHERQUEUE_THREAD_APARTMENTTYPE,
}
pub const EDD_GET_DEVICE_INTERFACE_NAME: i32 = 1;
pub const EM_SETSEL: i32 = 177;
pub const EN_CHANGE: i32 = 768;
pub const EN_KILLFOCUS: i32 = 512;
pub const ERROR_ALREADY_EXISTS: i32 = 183;
pub const ERROR_BROKEN_PIPE: i32 = 109;
pub const ERROR_NOTIFY_ENUM_DIR: i32 = 1022;
pub const ERROR_NO_DATA: i32 = 232;
pub const ERROR_PIPE_CONNECTED: i32 = 535;
pub const ES_AUTOHSCROLL: i32 = 128;
pub const EVENT_OBJECT_DESTROY: i32 = 32769;
pub const EVENT_SYSTEM_FOREGROUND: i32 = 3;
pub const EVENT_SYSTEM_MINIMIZEEND: i32 = 23;
pub const EVENT_SYSTEM_MINIMIZESTART: i32 = 22;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EXCEPTION_POINTERS {
    pub ExceptionRecord: PEXCEPTION_RECORD,
    pub ContextRecord: PCONTEXT,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EXCEPTION_RECORD {
    pub ExceptionCode: u32,
    pub ExceptionFlags: u32,
    pub ExceptionRecord: *mut Self,
    pub ExceptionAddress: *mut core::ffi::c_void,
    pub NumberParameters: u32,
    pub ExceptionInformation: [usize; 15],
}
impl Default for EXCEPTION_RECORD {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const E_FAIL: windows_core::HRESULT = windows_core::HRESULT(0x80004005_u32 as _);
pub const E_INVALIDARG: windows_core::HRESULT = windows_core::HRESULT(0x80070057_u32 as _);
pub type FARPROC = Option<unsafe extern "system" fn() -> isize>;
pub type FDAP = i32;
pub const FF_DONTCARE: i32 = 0;
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct FILEDESCRIPTORW {
    pub dwFlags: u32,
    pub clsid: windows_core::GUID,
    pub sizel: SIZEL,
    pub pointl: POINTL,
    pub dwFileAttributes: u32,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: u32,
    pub nFileSizeLow: u32,
    pub cFileName: [u16; 260],
}
impl Default for FILEDESCRIPTORW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct FILEGROUPDESCRIPTORW {
    pub cItems: u32,
    pub fgd: [FILEDESCRIPTORW; 1],
}
impl Default for FILEGROUPDESCRIPTORW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type FILEOPENDIALOGOPTIONS = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}
pub const FILE_ACTION_ADDED: i32 = 1;
pub const FILE_ACTION_MODIFIED: i32 = 3;
pub const FILE_ACTION_REMOVED: i32 = 2;
pub const FILE_ACTION_RENAMED_NEW_NAME: i32 = 5;
pub const FILE_ACTION_RENAMED_OLD_NAME: i32 = 4;
pub const FILE_ATTRIBUTE_DIRECTORY: i32 = 16;
pub const FILE_ATTRIBUTE_HIDDEN: i32 = 2;
pub const FILE_ATTRIBUTE_NORMAL: i32 = 128;
pub const FILE_ATTRIBUTE_REPARSE_POINT: i32 = 1024;
pub const FILE_ATTRIBUTE_SYSTEM: i32 = 4;
pub const FILE_FLAG_BACKUP_SEMANTICS: i32 = 33554432;
pub const FILE_FLAG_FIRST_PIPE_INSTANCE: i32 = 524288;
pub const FILE_FLAG_OVERLAPPED: i32 = 1073741824;
pub const FILE_LIST_DIRECTORY: i32 = 1;
pub const FILE_NOTIFY_CHANGE_ATTRIBUTES: i32 = 4;
pub const FILE_NOTIFY_CHANGE_DIR_NAME: i32 = 2;
pub const FILE_NOTIFY_CHANGE_FILE_NAME: i32 = 1;
pub const FILE_NOTIFY_CHANGE_LAST_WRITE: i32 = 16;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FILE_NOTIFY_INFORMATION {
    pub NextEntryOffset: u32,
    pub Action: u32,
    pub FileNameLength: u32,
    pub FileName: [u16; 1],
}
impl Default for FILE_NOTIFY_INFORMATION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const FILE_SHARE_DELETE: i32 = 4;
pub const FILE_SHARE_READ: i32 = 1;
pub const FILE_SHARE_WRITE: i32 = 2;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FLOATING_SAVE_AREA {
    pub ControlWord: u32,
    pub StatusWord: u32,
    pub TagWord: u32,
    pub ErrorOffset: u32,
    pub ErrorSelector: u32,
    pub DataOffset: u32,
    pub DataSelector: u32,
    pub RegisterArea: [u8; 80],
    pub Spare0: u32,
}
#[cfg(target_arch = "x86")]
impl Default for FLOATING_SAVE_AREA {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const FOFX_ADDUNDORECORD: i32 = 536870912;
pub const FOF_ALLOWUNDO: i32 = 64;
pub const FOF_NOCONFIRMATION: i32 = 16;
pub const FOF_NOCONFIRMMKDIR: i32 = 512;
pub const FOF_RENAMEONCOLLISION: i32 = 8;
pub const FOF_SILENT: i32 = 4;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FORMATETC {
    pub cfFormat: CLIPFORMAT,
    pub ptd: *mut DVTARGETDEVICE,
    pub dwAspect: u32,
    pub lindex: i32,
    pub tymed: u32,
}
pub const FOS_FILEMUSTEXIST: FILEOPENDIALOGOPTIONS = 4096;
pub const FOS_FORCEFILESYSTEM: FILEOPENDIALOGOPTIONS = 64;
pub const FOS_OVERWRITEPROMPT: FILEOPENDIALOGOPTIONS = 2;
pub const FOS_PATHMUSTEXIST: FILEOPENDIALOGOPTIONS = 2048;
pub const FW_NORMAL: i32 = 400;
pub const FW_SEMIBOLD: i32 = 600;
pub const FileOpenDialog: windows_core::GUID =
    windows_core::GUID::from_u128(0xdc1c5a9c_e88a_4dde_a5a1_60f82a20aef7);
pub const FileOperation: windows_core::GUID =
    windows_core::GUID::from_u128(0x3ad05575_8857_4850_9277_11b85bdb8e09);
pub const FileSaveDialog: windows_core::GUID =
    windows_core::GUID::from_u128(0xc0b4e2f3_ba21_4773_8dba_335ec946eb8b);
pub const GA_ROOT: i32 = 2;
pub const GA_ROOTOWNER: i32 = 3;
pub const GENERIC_READ: u32 = 2147483648;
pub const GENERIC_WRITE: i32 = 1073741824;
pub type GETPROPERTYSTOREFLAGS = u32;
pub const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: i32 = 4;
pub const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: i32 = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GROUP_AFFINITY {
    pub Mask: KAFFINITY,
    pub Group: u16,
    pub Reserved: [u16; 3],
}
impl Default for GROUP_AFFINITY {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const GUID_WICPixelFormat32bppBGRA: windows_core::GUID =
    windows_core::GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc90f);
pub const GUID_WICPixelFormat32bppPBGRA: windows_core::GUID =
    windows_core::GUID::from_u128(0x6fddc324_4e03_4bfe_b185_3d77768dc910);
pub const GWLP_USERDATA: i32 = -21;
pub const GWLP_WNDPROC: i32 = -4;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWL_STYLE: i32 = -16;
pub const GW_HWNDFIRST: i32 = 0;
pub const GW_HWNDLAST: i32 = 1;
pub const GW_HWNDNEXT: i32 = 2;
pub const GW_HWNDPREV: i32 = 3;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HANDLE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HBITMAP(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HBRUSH(pub *mut core::ffi::c_void);
pub type HCURSOR = HICON;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HDC(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HDROP(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HENHMETAFILE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HFONT(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HGDIOBJ(pub *mut core::ffi::c_void);
pub type HGLOBAL = HANDLE;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HICON(pub *mut core::ffi::c_void);
pub const HID_USAGE_GENERIC_MOUSE: USAGE = USAGE(2);
pub const HID_USAGE_PAGE_GENERIC: USAGE = USAGE(1);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HINSTANCE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HKEY(pub *mut core::ffi::c_void);
pub const HKEY_CURRENT_USER: HKEY = HKEY(-2147483647 as _);
pub const HKEY_LOCAL_MACHINE: HKEY = HKEY(-2147483646 as _);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HKL(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HMENU(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HMETAFILEPICT(pub *mut core::ffi::c_void);
pub type HMODULE = HINSTANCE;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HMONITOR(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HPALETTE(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HRAWINPUT(pub *mut core::ffi::c_void);
pub const HTBOTTOM: i32 = 15;
pub const HTBOTTOMLEFT: i32 = 16;
pub const HTBOTTOMRIGHT: i32 = 17;
pub const HTCAPTION: i32 = 2;
pub const HTCLIENT: i32 = 1;
pub const HTLEFT: i32 = 10;
pub const HTNOWHERE: i32 = 0;
pub const HTRIGHT: i32 = 11;
pub const HTTOP: i32 = 12;
pub const HTTOPLEFT: i32 = 13;
pub const HTTOPRIGHT: i32 = 14;
pub const HTTRANSPARENT: i32 = -1;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWINEVENTHOOK(pub *mut core::ffi::c_void);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct HWND(pub *mut core::ffi::c_void);
pub const HWND_BOTTOM: HWND = HWND(1 as _);
pub const HWND_MESSAGE: HWND = HWND(-3 as _);
pub const HWND_NOTOPMOST: HWND = HWND(-2 as _);
pub const HWND_TOP: HWND = HWND(0 as _);
pub const HWND_TOPMOST: HWND = HWND(-1 as _);
windows_core::imp::define_interface!(
    IAdviseSink,
    IAdviseSink_Vtbl,
    0x0000010f_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IAdviseSink, windows_core::IUnknown);
#[repr(C)]
pub struct IAdviseSink_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    OnDataChange: usize,
    OnViewChange: usize,
    OnRename: usize,
    OnSave: usize,
    OnClose: usize,
}
impl windows_core::RuntimeName for IAdviseSink {}
windows_core::imp::define_interface!(
    IBindCtx,
    IBindCtx_Vtbl,
    0x0000000e_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IBindCtx, windows_core::IUnknown);
#[repr(C)]
pub struct IBindCtx_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    RegisterObjectBound: usize,
    RevokeObjectBound: usize,
    ReleaseBoundObjects: usize,
    SetBindOptions: usize,
    GetBindOptions: usize,
    GetRunningObjectTable: usize,
    RegisterObjectParam: usize,
    GetObjectParam: usize,
    EnumObjectParam: usize,
    RevokeObjectParam: usize,
}
impl windows_core::RuntimeName for IBindCtx {}
pub const ICC_BAR_CLASSES: i32 = 4;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ICONINFO {
    pub fIcon: windows_core::BOOL,
    pub xHotspot: u32,
    pub yHotspot: u32,
    pub hbmMask: HBITMAP,
    pub hbmColor: HBITMAP,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ICONMETRICSW {
    pub cbSize: u32,
    pub iHorzSpacing: i32,
    pub iVertSpacing: i32,
    pub iTitleWrap: i32,
    pub lfFont: LOGFONTW,
}
windows_core::imp::define_interface!(
    IContextMenu,
    IContextMenu_Vtbl,
    0x000214e4_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IContextMenu, windows_core::IUnknown);
impl IContextMenu {
    pub(crate) unsafe fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        indexmenu: u32,
        idcmdfirst: u32,
        idcmdlast: u32,
        uflags: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).QueryContextMenu)(
                windows_core::Interface::as_raw(self),
                hmenu,
                indexmenu,
                idcmdfirst,
                idcmdlast,
                uflags,
            )
        }
    }
    pub(crate) unsafe fn InvokeCommand(
        &self,
        pici: *const CMINVOKECOMMANDINFO,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).InvokeCommand)(
                windows_core::Interface::as_raw(self),
                pici,
            )
        }
    }
    pub(crate) unsafe fn GetCommandString(
        &self,
        idcmd: usize,
        utype: u32,
        preserved: Option<*const u32>,
        pszname: *mut i8,
        cchmax: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetCommandString)(
                windows_core::Interface::as_raw(self),
                idcmd,
                utype,
                preserved.unwrap_or(core::mem::zeroed()) as _,
                pszname as _,
                cchmax,
            )
        }
    }
}
#[repr(C)]
pub struct IContextMenu_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub QueryContextMenu: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HMENU,
        u32,
        u32,
        u32,
        u32,
    ) -> windows_core::HRESULT,
    pub InvokeCommand: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const CMINVOKECOMMANDINFO,
    ) -> windows_core::HRESULT,
    pub GetCommandString: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        usize,
        u32,
        *const u32,
        *mut i8,
        u32,
    ) -> windows_core::HRESULT,
}
pub trait IContextMenu_Impl: windows_core::IUnknownImpl {
    fn QueryContextMenu(
        &self,
        hmenu: HMENU,
        indexmenu: u32,
        idcmdfirst: u32,
        idcmdlast: u32,
        uflags: u32,
    ) -> windows_core::Result<()>;
    fn InvokeCommand(&self, pici: *const CMINVOKECOMMANDINFO) -> windows_core::Result<()>;
    fn GetCommandString(
        &self,
        idcmd: usize,
        utype: u32,
        preserved: *const u32,
        pszname: *mut i8,
        cchmax: u32,
    ) -> windows_core::Result<()>;
}
impl IContextMenu_Vtbl {
    pub const fn new<Identity: IContextMenu_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn QueryContextMenu<
            Identity: IContextMenu_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hmenu: HMENU,
            indexmenu: u32,
            idcmdfirst: u32,
            idcmdlast: u32,
            uflags: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IContextMenu_Impl::QueryContextMenu(
                    this,
                    core::mem::transmute_copy(&hmenu),
                    core::mem::transmute_copy(&indexmenu),
                    core::mem::transmute_copy(&idcmdfirst),
                    core::mem::transmute_copy(&idcmdlast),
                    core::mem::transmute_copy(&uflags),
                )
                .into()
            }
        }
        unsafe extern "system" fn InvokeCommand<
            Identity: IContextMenu_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pici: *const CMINVOKECOMMANDINFO,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IContextMenu_Impl::InvokeCommand(this, core::mem::transmute_copy(&pici)).into()
            }
        }
        unsafe extern "system" fn GetCommandString<
            Identity: IContextMenu_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            idcmd: usize,
            utype: u32,
            preserved: *const u32,
            pszname: *mut i8,
            cchmax: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IContextMenu_Impl::GetCommandString(
                    this,
                    core::mem::transmute_copy(&idcmd),
                    core::mem::transmute_copy(&utype),
                    core::mem::transmute_copy(&preserved),
                    core::mem::transmute_copy(&pszname),
                    core::mem::transmute_copy(&cchmax),
                )
                .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            QueryContextMenu: QueryContextMenu::<Identity, OFFSET>,
            InvokeCommand: InvokeCommand::<Identity, OFFSET>,
            GetCommandString: GetCommandString::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IContextMenu as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IContextMenu {}
windows_core::imp::define_interface!(
    IContextMenu2,
    IContextMenu2_Vtbl,
    0x000214f4_0000_0000_c000_000000000046
);
impl core::ops::Deref for IContextMenu2 {
    type Target = IContextMenu;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IContextMenu2, windows_core::IUnknown, IContextMenu);
impl IContextMenu2 {
    pub(crate) unsafe fn HandleMenuMsg(
        &self,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).HandleMenuMsg)(
                windows_core::Interface::as_raw(self),
                umsg,
                wparam,
                lparam,
            )
        }
    }
}
#[repr(C)]
pub struct IContextMenu2_Vtbl {
    pub base__: IContextMenu_Vtbl,
    pub HandleMenuMsg: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        WPARAM,
        LPARAM,
    ) -> windows_core::HRESULT,
}
pub trait IContextMenu2_Impl: IContextMenu_Impl {
    fn HandleMenuMsg(&self, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> windows_core::Result<()>;
}
impl IContextMenu2_Vtbl {
    pub const fn new<Identity: IContextMenu2_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn HandleMenuMsg<
            Identity: IContextMenu2_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            umsg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IContextMenu2_Impl::HandleMenuMsg(
                    this,
                    core::mem::transmute_copy(&umsg),
                    core::mem::transmute_copy(&wparam),
                    core::mem::transmute_copy(&lparam),
                )
                .into()
            }
        }
        Self {
            base__: IContextMenu_Vtbl::new::<Identity, OFFSET>(),
            HandleMenuMsg: HandleMenuMsg::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IContextMenu2 as windows_core::Interface>::IID
            || iid == &<IContextMenu as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IContextMenu2 {}
windows_core::imp::define_interface!(
    IContextMenu3,
    IContextMenu3_Vtbl,
    0xbcfce0a0_ec17_11d0_8d10_00a0c90f2719
);
impl core::ops::Deref for IContextMenu3 {
    type Target = IContextMenu2;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IContextMenu3,
    windows_core::IUnknown,
    IContextMenu,
    IContextMenu2
);
impl IContextMenu3 {
    pub(crate) unsafe fn HandleMenuMsg2(
        &self,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        plresult: Option<*mut LRESULT>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).HandleMenuMsg2)(
                windows_core::Interface::as_raw(self),
                umsg,
                wparam,
                lparam,
                plresult.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
}
#[repr(C)]
pub struct IContextMenu3_Vtbl {
    pub base__: IContextMenu2_Vtbl,
    pub HandleMenuMsg2: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        WPARAM,
        LPARAM,
        *mut LRESULT,
    ) -> windows_core::HRESULT,
}
pub trait IContextMenu3_Impl: IContextMenu2_Impl {
    fn HandleMenuMsg2(
        &self,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        plresult: *mut LRESULT,
    ) -> windows_core::Result<()>;
}
impl IContextMenu3_Vtbl {
    pub const fn new<Identity: IContextMenu3_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn HandleMenuMsg2<
            Identity: IContextMenu3_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            umsg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
            plresult: *mut LRESULT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IContextMenu3_Impl::HandleMenuMsg2(
                    this,
                    core::mem::transmute_copy(&umsg),
                    core::mem::transmute_copy(&wparam),
                    core::mem::transmute_copy(&lparam),
                    core::mem::transmute_copy(&plresult),
                )
                .into()
            }
        }
        Self {
            base__: IContextMenu2_Vtbl::new::<Identity, OFFSET>(),
            HandleMenuMsg2: HandleMenuMsg2::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IContextMenu3 as windows_core::Interface>::IID
            || iid == &<IContextMenu as windows_core::Interface>::IID
            || iid == &<IContextMenu2 as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IContextMenu3 {}
windows_core::imp::define_interface!(
    IContextMenuCB,
    IContextMenuCB_Vtbl,
    0x3409e930_5a39_11d1_83fa_00a0c90dc849
);
windows_core::imp::interface_hierarchy!(IContextMenuCB, windows_core::IUnknown);
#[repr(C)]
pub struct IContextMenuCB_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    CallBack: usize,
}
impl windows_core::RuntimeName for IContextMenuCB {}
windows_core::imp::define_interface!(
    ID2D1Bitmap,
    ID2D1Bitmap_Vtbl,
    0xa2296057_ea42_4099_983b_539fb6505426
);
impl core::ops::Deref for ID2D1Bitmap {
    type Target = ID2D1Image;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1Bitmap,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Image
);
#[repr(C)]
pub struct ID2D1Bitmap_Vtbl {
    pub base__: ID2D1Image_Vtbl,
    GetSize: usize,
    GetPixelSize: usize,
    GetPixelFormat: usize,
    GetDpi: usize,
    CopyFromBitmap: usize,
    CopyFromRenderTarget: usize,
    CopyFromMemory: usize,
}
impl windows_core::RuntimeName for ID2D1Bitmap {}
windows_core::imp::define_interface!(
    ID2D1BitmapBrush,
    ID2D1BitmapBrush_Vtbl,
    0x2cd906aa_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1BitmapBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1BitmapBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1BitmapBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetExtendModeX: usize,
    SetExtendModeY: usize,
    SetInterpolationMode: usize,
    SetBitmap: usize,
    GetExtendModeX: usize,
    GetExtendModeY: usize,
    GetInterpolationMode: usize,
    GetBitmap: usize,
}
impl windows_core::RuntimeName for ID2D1BitmapBrush {}
windows_core::imp::define_interface!(
    ID2D1BitmapRenderTarget,
    ID2D1BitmapRenderTarget_Vtbl,
    0x2cd90695_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1BitmapRenderTarget {
    type Target = ID2D1RenderTarget;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1BitmapRenderTarget,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1RenderTarget
);
#[repr(C)]
pub struct ID2D1BitmapRenderTarget_Vtbl {
    pub base__: ID2D1RenderTarget_Vtbl,
    GetBitmap: usize,
}
impl windows_core::RuntimeName for ID2D1BitmapRenderTarget {}
windows_core::imp::define_interface!(
    ID2D1Brush,
    ID2D1Brush_Vtbl,
    0x2cd906a8_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Brush {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Brush, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Brush_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    SetOpacity: usize,
    SetTransform: usize,
    GetOpacity: usize,
    GetTransform: usize,
}
impl windows_core::RuntimeName for ID2D1Brush {}
windows_core::imp::define_interface!(
    ID2D1DrawingStateBlock,
    ID2D1DrawingStateBlock_Vtbl,
    0x28506e39_ebf6_46a1_bb47_fd85565ab957
);
impl core::ops::Deref for ID2D1DrawingStateBlock {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1DrawingStateBlock,
    windows_core::IUnknown,
    ID2D1Resource
);
#[repr(C)]
pub struct ID2D1DrawingStateBlock_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetDescription: usize,
    SetDescription: usize,
    SetTextRenderingParams: usize,
    GetTextRenderingParams: usize,
}
impl windows_core::RuntimeName for ID2D1DrawingStateBlock {}
windows_core::imp::define_interface!(
    ID2D1Geometry,
    ID2D1Geometry_Vtbl,
    0x2cd906a1_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Geometry {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Geometry, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Geometry_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetBounds: usize,
    GetWidenedBounds: usize,
    StrokeContainsPoint: usize,
    FillContainsPoint: usize,
    CompareWithGeometry: usize,
    Simplify: usize,
    Tessellate: usize,
    CombineWithGeometry: usize,
    Outline: usize,
    ComputeArea: usize,
    ComputeLength: usize,
    ComputePointAtLength: usize,
    Widen: usize,
}
impl windows_core::RuntimeName for ID2D1Geometry {}
windows_core::imp::define_interface!(
    ID2D1GradientStopCollection,
    ID2D1GradientStopCollection_Vtbl,
    0x2cd906a7_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1GradientStopCollection {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1GradientStopCollection,
    windows_core::IUnknown,
    ID2D1Resource
);
#[repr(C)]
pub struct ID2D1GradientStopCollection_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetGradientStopCount: usize,
    GetGradientStops: usize,
    GetColorInterpolationGamma: usize,
    GetExtendMode: usize,
}
impl windows_core::RuntimeName for ID2D1GradientStopCollection {}
windows_core::imp::define_interface!(
    ID2D1Image,
    ID2D1Image_Vtbl,
    0x65019f75_8da2_497c_b32c_dfa34e48ede6
);
impl core::ops::Deref for ID2D1Image {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Image, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Image_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
}
impl windows_core::RuntimeName for ID2D1Image {}
windows_core::imp::define_interface!(
    ID2D1Layer,
    ID2D1Layer_Vtbl,
    0x2cd9069b_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Layer {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Layer, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Layer_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetSize: usize,
}
impl windows_core::RuntimeName for ID2D1Layer {}
windows_core::imp::define_interface!(
    ID2D1LinearGradientBrush,
    ID2D1LinearGradientBrush_Vtbl,
    0x2cd906ab_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1LinearGradientBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1LinearGradientBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1LinearGradientBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetStartPoint: usize,
    SetEndPoint: usize,
    GetStartPoint: usize,
    GetEndPoint: usize,
    GetGradientStopCollection: usize,
}
impl windows_core::RuntimeName for ID2D1LinearGradientBrush {}
windows_core::imp::define_interface!(
    ID2D1Mesh,
    ID2D1Mesh_Vtbl,
    0x2cd906c2_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1Mesh {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1Mesh, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1Mesh_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    Open: usize,
}
impl windows_core::RuntimeName for ID2D1Mesh {}
windows_core::imp::define_interface!(
    ID2D1RadialGradientBrush,
    ID2D1RadialGradientBrush_Vtbl,
    0x2cd906ac_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1RadialGradientBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1RadialGradientBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1RadialGradientBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetCenter: usize,
    SetGradientOriginOffset: usize,
    SetRadiusX: usize,
    SetRadiusY: usize,
    GetCenter: usize,
    GetGradientOriginOffset: usize,
    GetRadiusX: usize,
    GetRadiusY: usize,
    GetGradientStopCollection: usize,
}
impl windows_core::RuntimeName for ID2D1RadialGradientBrush {}
windows_core::imp::define_interface!(
    ID2D1RenderTarget,
    ID2D1RenderTarget_Vtbl,
    0x2cd90694_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1RenderTarget {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1RenderTarget, windows_core::IUnknown, ID2D1Resource);
impl ID2D1RenderTarget {
    pub(crate) unsafe fn CreateBitmap(
        &self,
        size: D2D_SIZE_U,
        srcdata: Option<*const core::ffi::c_void>,
        pitch: u32,
        bitmapproperties: *const D2D1_BITMAP_PROPERTIES,
    ) -> windows_core::Result<ID2D1Bitmap> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmap)(
                windows_core::Interface::as_raw(self),
                size,
                srcdata.unwrap_or(core::mem::zeroed()) as _,
                pitch,
                bitmapproperties,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromWicBitmap<P0>(
        &self,
        wicbitmapsource: P0,
        bitmapproperties: Option<*const D2D1_BITMAP_PROPERTIES>,
    ) -> windows_core::Result<ID2D1Bitmap>
    where
        P0: windows_core::Param<IWICBitmapSource>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromWicBitmap)(
                windows_core::Interface::as_raw(self),
                wicbitmapsource.param().abi(),
                bitmapproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateSharedBitmap(
        &self,
        riid: *const windows_core::GUID,
        data: *mut core::ffi::c_void,
        bitmapproperties: Option<*const D2D1_BITMAP_PROPERTIES>,
        bitmap: *mut Option<ID2D1Bitmap>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CreateSharedBitmap)(
                windows_core::Interface::as_raw(self),
                riid,
                data as _,
                bitmapproperties.unwrap_or(core::mem::zeroed()) as _,
                core::mem::transmute(bitmap),
            )
        }
    }
    pub(crate) unsafe fn CreateBitmapBrush<P0>(
        &self,
        bitmap: P0,
        bitmapbrushproperties: Option<*const D2D1_BITMAP_BRUSH_PROPERTIES>,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
    ) -> windows_core::Result<ID2D1BitmapBrush>
    where
        P0: windows_core::Param<ID2D1Bitmap>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapBrush)(
                windows_core::Interface::as_raw(self),
                bitmap.param().abi(),
                bitmapbrushproperties.unwrap_or(core::mem::zeroed()) as _,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateSolidColorBrush(
        &self,
        color: *const D2D_COLOR_F,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
    ) -> windows_core::Result<ID2D1SolidColorBrush> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateSolidColorBrush)(
                windows_core::Interface::as_raw(self),
                color,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateGradientStopCollection(
        &self,
        gradientstops: &[D2D1_GRADIENT_STOP],
        colorinterpolationgamma: D2D1_GAMMA,
        extendmode: D2D1_EXTEND_MODE,
    ) -> windows_core::Result<ID2D1GradientStopCollection> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateGradientStopCollection)(
                windows_core::Interface::as_raw(self),
                gradientstops.as_ptr(),
                gradientstops.len().try_into().unwrap(),
                colorinterpolationgamma,
                extendmode,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateLinearGradientBrush<P2>(
        &self,
        lineargradientbrushproperties: *const D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
        gradientstopcollection: P2,
    ) -> windows_core::Result<ID2D1LinearGradientBrush>
    where
        P2: windows_core::Param<ID2D1GradientStopCollection>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateLinearGradientBrush)(
                windows_core::Interface::as_raw(self),
                lineargradientbrushproperties,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                gradientstopcollection.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateRadialGradientBrush<P2>(
        &self,
        radialgradientbrushproperties: *const D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
        brushproperties: Option<*const D2D1_BRUSH_PROPERTIES>,
        gradientstopcollection: P2,
    ) -> windows_core::Result<ID2D1RadialGradientBrush>
    where
        P2: windows_core::Param<ID2D1GradientStopCollection>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateRadialGradientBrush)(
                windows_core::Interface::as_raw(self),
                radialgradientbrushproperties,
                brushproperties.unwrap_or(core::mem::zeroed()) as _,
                gradientstopcollection.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateCompatibleRenderTarget(
        &self,
        desiredsize: Option<*const D2D_SIZE_F>,
        desiredpixelsize: Option<*const D2D_SIZE_U>,
        desiredformat: Option<*const D2D1_PIXEL_FORMAT>,
        options: D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS,
    ) -> windows_core::Result<ID2D1BitmapRenderTarget> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateCompatibleRenderTarget)(
                windows_core::Interface::as_raw(self),
                desiredsize.unwrap_or(core::mem::zeroed()) as _,
                desiredpixelsize.unwrap_or(core::mem::zeroed()) as _,
                desiredformat.unwrap_or(core::mem::zeroed()) as _,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateLayer(
        &self,
        size: Option<*const D2D_SIZE_F>,
    ) -> windows_core::Result<ID2D1Layer> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateLayer)(
                windows_core::Interface::as_raw(self),
                size.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateMesh(&self) -> windows_core::Result<ID2D1Mesh> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateMesh)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn DrawLine<P2, P4>(
        &self,
        point0: windows_numerics::Vector2,
        point1: windows_numerics::Vector2,
        brush: P2,
        strokewidth: f32,
        strokestyle: P4,
    ) where
        P2: windows_core::Param<ID2D1Brush>,
        P4: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawLine)(
                windows_core::Interface::as_raw(self),
                point0,
                point1,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawRectangle<P1, P3>(
        &self,
        rect: *const D2D_RECT_F,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawRectangle)(
                windows_core::Interface::as_raw(self),
                rect,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillRectangle<P1>(&self, rect: *const D2D_RECT_F, brush: P1)
    where
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillRectangle)(
                windows_core::Interface::as_raw(self),
                rect,
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawRoundedRectangle<P1, P3>(
        &self,
        roundedrect: *const D2D1_ROUNDED_RECT,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawRoundedRectangle)(
                windows_core::Interface::as_raw(self),
                roundedrect,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillRoundedRectangle<P1>(
        &self,
        roundedrect: *const D2D1_ROUNDED_RECT,
        brush: P1,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillRoundedRectangle)(
                windows_core::Interface::as_raw(self),
                roundedrect,
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawEllipse<P1, P3>(
        &self,
        ellipse: *const D2D1_ELLIPSE,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawEllipse)(
                windows_core::Interface::as_raw(self),
                ellipse,
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillEllipse<P1>(&self, ellipse: *const D2D1_ELLIPSE, brush: P1)
    where
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillEllipse)(
                windows_core::Interface::as_raw(self),
                ellipse,
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn DrawGeometry<P0, P1, P3>(
        &self,
        geometry: P0,
        brush: P1,
        strokewidth: f32,
        strokestyle: P3,
    ) where
        P0: windows_core::Param<ID2D1Geometry>,
        P1: windows_core::Param<ID2D1Brush>,
        P3: windows_core::Param<ID2D1StrokeStyle>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGeometry)(
                windows_core::Interface::as_raw(self),
                geometry.param().abi(),
                brush.param().abi(),
                strokewidth,
                strokestyle.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillGeometry<P0, P1, P2>(&self, geometry: P0, brush: P1, opacitybrush: P2)
    where
        P0: windows_core::Param<ID2D1Geometry>,
        P1: windows_core::Param<ID2D1Brush>,
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillGeometry)(
                windows_core::Interface::as_raw(self),
                geometry.param().abi(),
                brush.param().abi(),
                opacitybrush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillMesh<P0, P1>(&self, mesh: P0, brush: P1)
    where
        P0: windows_core::Param<ID2D1Mesh>,
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillMesh)(
                windows_core::Interface::as_raw(self),
                mesh.param().abi(),
                brush.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn FillOpacityMask<P0, P1>(
        &self,
        opacitymask: P0,
        brush: P1,
        content: D2D1_OPACITY_MASK_CONTENT,
        destinationrectangle: Option<*const D2D_RECT_F>,
        sourcerectangle: Option<*const D2D_RECT_F>,
    ) where
        P0: windows_core::Param<ID2D1Bitmap>,
        P1: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).FillOpacityMask)(
                windows_core::Interface::as_raw(self),
                opacitymask.param().abi(),
                brush.param().abi(),
                content,
                destinationrectangle.unwrap_or(core::mem::zeroed()) as _,
                sourcerectangle.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn DrawBitmap<P0>(
        &self,
        bitmap: P0,
        destinationrectangle: Option<*const D2D_RECT_F>,
        opacity: f32,
        interpolationmode: D2D1_BITMAP_INTERPOLATION_MODE,
        sourcerectangle: Option<*const D2D_RECT_F>,
    ) where
        P0: windows_core::Param<ID2D1Bitmap>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawBitmap)(
                windows_core::Interface::as_raw(self),
                bitmap.param().abi(),
                destinationrectangle.unwrap_or(core::mem::zeroed()) as _,
                opacity,
                interpolationmode,
                sourcerectangle.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn DrawText<P2, P4>(
        &self,
        string: &[u16],
        textformat: P2,
        layoutrect: *const D2D_RECT_F,
        defaultfillbrush: P4,
        options: D2D1_DRAW_TEXT_OPTIONS,
        measuringmode: DWRITE_MEASURING_MODE,
    ) where
        P2: windows_core::Param<IDWriteTextFormat>,
        P4: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawText)(
                windows_core::Interface::as_raw(self),
                string.as_ptr(),
                string.len().try_into().unwrap(),
                textformat.param().abi(),
                layoutrect,
                defaultfillbrush.param().abi(),
                options,
                measuringmode,
            );
        }
    }
    pub(crate) unsafe fn DrawTextLayout<P1, P2>(
        &self,
        origin: windows_numerics::Vector2,
        textlayout: P1,
        defaultfillbrush: P2,
        options: D2D1_DRAW_TEXT_OPTIONS,
    ) where
        P1: windows_core::Param<IDWriteTextLayout>,
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawTextLayout)(
                windows_core::Interface::as_raw(self),
                origin,
                textlayout.param().abi(),
                defaultfillbrush.param().abi(),
                options,
            );
        }
    }
    pub(crate) unsafe fn DrawGlyphRun<P2>(
        &self,
        baselineorigin: windows_numerics::Vector2,
        glyphrun: *const DWRITE_GLYPH_RUN,
        foregroundbrush: P2,
        measuringmode: DWRITE_MEASURING_MODE,
    ) where
        P2: windows_core::Param<ID2D1Brush>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DrawGlyphRun)(
                windows_core::Interface::as_raw(self),
                baselineorigin,
                glyphrun,
                foregroundbrush.param().abi(),
                measuringmode,
            );
        }
    }
    pub(crate) unsafe fn SetTransform(&self, transform: *const windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTransform)(
                windows_core::Interface::as_raw(self),
                transform,
            );
        }
    }
    pub(crate) unsafe fn GetTransform(&self, transform: *mut windows_numerics::Matrix3x2) {
        unsafe {
            (windows_core::Interface::vtable(self).GetTransform)(
                windows_core::Interface::as_raw(self),
                transform as _,
            );
        }
    }
    pub(crate) unsafe fn SetAntialiasMode(&self, antialiasmode: D2D1_ANTIALIAS_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetAntialiasMode)(
                windows_core::Interface::as_raw(self),
                antialiasmode,
            );
        }
    }
    pub(crate) unsafe fn GetAntialiasMode(&self) -> D2D1_ANTIALIAS_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetAntialiasMode)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn SetTextAntialiasMode(&self, textantialiasmode: D2D1_TEXT_ANTIALIAS_MODE) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextAntialiasMode)(
                windows_core::Interface::as_raw(self),
                textantialiasmode,
            );
        }
    }
    pub(crate) unsafe fn GetTextAntialiasMode(&self) -> D2D1_TEXT_ANTIALIAS_MODE {
        unsafe {
            (windows_core::Interface::vtable(self).GetTextAntialiasMode)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn SetTextRenderingParams<P0>(&self, textrenderingparams: P0)
    where
        P0: windows_core::Param<IDWriteRenderingParams>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTextRenderingParams)(
                windows_core::Interface::as_raw(self),
                textrenderingparams.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn GetTextRenderingParams(
        &self,
    ) -> windows_core::Result<IDWriteRenderingParams> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetTextRenderingParams)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            windows_core::imp::Type::from_abi(result__)
        }
    }
    pub(crate) unsafe fn SetTags(&self, tag1: D2D1_TAG, tag2: D2D1_TAG) {
        unsafe {
            (windows_core::Interface::vtable(self).SetTags)(
                windows_core::Interface::as_raw(self),
                tag1,
                tag2,
            );
        }
    }
    pub(crate) unsafe fn GetTags(&self, tag1: Option<*mut D2D1_TAG>, tag2: Option<*mut D2D1_TAG>) {
        unsafe {
            (windows_core::Interface::vtable(self).GetTags)(
                windows_core::Interface::as_raw(self),
                tag1.unwrap_or(core::mem::zeroed()) as _,
                tag2.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn PushLayer<P1>(
        &self,
        layerparameters: *const D2D1_LAYER_PARAMETERS,
        layer: P1,
    ) where
        P1: windows_core::Param<ID2D1Layer>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).PushLayer)(
                windows_core::Interface::as_raw(self),
                layerparameters,
                layer.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn PopLayer(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).PopLayer)(windows_core::Interface::as_raw(self));
        }
    }
    pub(crate) unsafe fn Flush(
        &self,
        tag1: Option<*mut D2D1_TAG>,
        tag2: Option<*mut D2D1_TAG>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Flush)(
                windows_core::Interface::as_raw(self),
                tag1.unwrap_or(core::mem::zeroed()) as _,
                tag2.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn SaveDrawingState(
        &self,
        drawingstateblock: &Option<ID2D1DrawingStateBlock>,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).SaveDrawingState)(
                windows_core::Interface::as_raw(self),
                core::mem::transmute_copy(drawingstateblock),
            );
        }
    }
    pub(crate) unsafe fn RestoreDrawingState<P0>(&self, drawingstateblock: P0)
    where
        P0: windows_core::Param<ID2D1DrawingStateBlock>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RestoreDrawingState)(
                windows_core::Interface::as_raw(self),
                drawingstateblock.param().abi(),
            );
        }
    }
    pub(crate) unsafe fn PushAxisAlignedClip(
        &self,
        cliprect: *const D2D_RECT_F,
        antialiasmode: D2D1_ANTIALIAS_MODE,
    ) {
        unsafe {
            (windows_core::Interface::vtable(self).PushAxisAlignedClip)(
                windows_core::Interface::as_raw(self),
                cliprect,
                antialiasmode,
            );
        }
    }
    pub(crate) unsafe fn PopAxisAlignedClip(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).PopAxisAlignedClip)(
                windows_core::Interface::as_raw(self),
            );
        }
    }
    pub(crate) unsafe fn Clear(&self, clearcolor: Option<*const D2D_COLOR_F>) {
        unsafe {
            (windows_core::Interface::vtable(self).Clear)(
                windows_core::Interface::as_raw(self),
                clearcolor.unwrap_or(core::mem::zeroed()) as _,
            );
        }
    }
    pub(crate) unsafe fn BeginDraw(&self) {
        unsafe {
            (windows_core::Interface::vtable(self).BeginDraw)(windows_core::Interface::as_raw(
                self,
            ));
        }
    }
    pub(crate) unsafe fn EndDraw(
        &self,
        tag1: Option<*mut D2D1_TAG>,
        tag2: Option<*mut D2D1_TAG>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).EndDraw)(
                windows_core::Interface::as_raw(self),
                tag1.unwrap_or(core::mem::zeroed()) as _,
                tag2.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn GetPixelFormat(&self) -> D2D1_PIXEL_FORMAT {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPixelFormat)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn SetDpi(&self, dpix: f32, dpiy: f32) {
        unsafe {
            (windows_core::Interface::vtable(self).SetDpi)(
                windows_core::Interface::as_raw(self),
                dpix,
                dpiy,
            );
        }
    }
    pub(crate) unsafe fn GetDpi(&self, dpix: *mut f32, dpiy: *mut f32) {
        unsafe {
            (windows_core::Interface::vtable(self).GetDpi)(
                windows_core::Interface::as_raw(self),
                dpix as _,
                dpiy as _,
            );
        }
    }
    pub(crate) unsafe fn GetSize(&self) -> D2D_SIZE_F {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn GetPixelSize(&self) -> D2D_SIZE_U {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPixelSize)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            );
            result__
        }
    }
    pub(crate) unsafe fn GetMaximumBitmapSize(&self) -> u32 {
        unsafe {
            (windows_core::Interface::vtable(self).GetMaximumBitmapSize)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn IsSupported(
        &self,
        rendertargetproperties: *const D2D1_RENDER_TARGET_PROPERTIES,
    ) -> windows_core::BOOL {
        unsafe {
            (windows_core::Interface::vtable(self).IsSupported)(
                windows_core::Interface::as_raw(self),
                rendertargetproperties,
            )
        }
    }
}
#[repr(C)]
pub struct ID2D1RenderTarget_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    pub CreateBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        D2D_SIZE_U,
        *const core::ffi::c_void,
        u32,
        *const D2D1_BITMAP_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromWicBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSharedBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D1_BITMAP_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateSolidColorBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_COLOR_F,
        *const D2D1_BRUSH_PROPERTIES,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateGradientStopCollection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_GRADIENT_STOP,
        u32,
        D2D1_GAMMA,
        D2D1_EXTEND_MODE,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLinearGradientBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateRadialGradientBrush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_RADIAL_GRADIENT_BRUSH_PROPERTIES,
        *const D2D1_BRUSH_PROPERTIES,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateCompatibleRenderTarget: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_SIZE_F,
        *const D2D_SIZE_U,
        *const D2D1_PIXEL_FORMAT,
        D2D1_COMPATIBLE_RENDER_TARGET_OPTIONS,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateLayer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_SIZE_F,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateMesh: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DrawLine: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        windows_numerics::Vector2,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub DrawRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *mut core::ffi::c_void,
    ),
    pub DrawRoundedRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ROUNDED_RECT,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillRoundedRectangle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ROUNDED_RECT,
        *mut core::ffi::c_void,
    ),
    pub DrawEllipse: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ELLIPSE,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillEllipse: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_ELLIPSE,
        *mut core::ffi::c_void,
    ),
    pub DrawGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        f32,
        *mut core::ffi::c_void,
    ),
    pub FillGeometry: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ),
    pub FillMesh: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ),
    pub FillOpacityMask: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D2D1_OPACITY_MASK_CONTENT,
        *const D2D_RECT_F,
        *const D2D_RECT_F,
    ),
    pub DrawBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        f32,
        D2D1_BITMAP_INTERPOLATION_MODE,
        *const D2D_RECT_F,
    ),
    pub DrawText: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const u16,
        u32,
        *mut core::ffi::c_void,
        *const D2D_RECT_F,
        *mut core::ffi::c_void,
        D2D1_DRAW_TEXT_OPTIONS,
        DWRITE_MEASURING_MODE,
    ),
    pub DrawTextLayout: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        D2D1_DRAW_TEXT_OPTIONS,
    ),
    pub DrawGlyphRun: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_numerics::Vector2,
        *const DWRITE_GLYPH_RUN,
        *mut core::ffi::c_void,
        DWRITE_MEASURING_MODE,
    ),
    pub SetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const windows_numerics::Matrix3x2),
    pub GetTransform:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut windows_numerics::Matrix3x2),
    pub SetAntialiasMode: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_ANTIALIAS_MODE),
    pub GetAntialiasMode: unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_ANTIALIAS_MODE,
    pub SetTextAntialiasMode:
        unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_TEXT_ANTIALIAS_MODE),
    pub GetTextAntialiasMode:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> D2D1_TEXT_ANTIALIAS_MODE,
    pub SetTextRenderingParams:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub GetTextRenderingParams:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut *mut core::ffi::c_void),
    pub SetTags: unsafe extern "system" fn(*mut core::ffi::c_void, D2D1_TAG, D2D1_TAG),
    pub GetTags: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D1_TAG, *mut D2D1_TAG),
    pub PushLayer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_LAYER_PARAMETERS,
        *mut core::ffi::c_void,
    ),
    pub PopLayer: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub Flush: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D2D1_TAG,
        *mut D2D1_TAG,
    ) -> windows_core::HRESULT,
    pub SaveDrawingState: unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub RestoreDrawingState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
    pub PushAxisAlignedClip:
        unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D_RECT_F, D2D1_ANTIALIAS_MODE),
    pub PopAxisAlignedClip: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub Clear: unsafe extern "system" fn(*mut core::ffi::c_void, *const D2D_COLOR_F),
    pub BeginDraw: unsafe extern "system" fn(*mut core::ffi::c_void),
    pub EndDraw: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut D2D1_TAG,
        *mut D2D1_TAG,
    ) -> windows_core::HRESULT,
    pub GetPixelFormat: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D1_PIXEL_FORMAT),
    pub SetDpi: unsafe extern "system" fn(*mut core::ffi::c_void, f32, f32),
    pub GetDpi: unsafe extern "system" fn(*mut core::ffi::c_void, *mut f32, *mut f32),
    pub GetSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D_SIZE_F),
    pub GetPixelSize: unsafe extern "system" fn(*mut core::ffi::c_void, *mut D2D_SIZE_U),
    pub GetMaximumBitmapSize: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    pub IsSupported: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const D2D1_RENDER_TARGET_PROPERTIES,
    ) -> windows_core::BOOL,
}
impl windows_core::RuntimeName for ID2D1RenderTarget {}
windows_core::imp::define_interface!(
    ID2D1Resource,
    ID2D1Resource_Vtbl,
    0x2cd90691_12e2_11dc_9fed_001143a055f9
);
windows_core::imp::interface_hierarchy!(ID2D1Resource, windows_core::IUnknown);
#[repr(C)]
pub struct ID2D1Resource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetFactory: usize,
}
impl windows_core::RuntimeName for ID2D1Resource {}
windows_core::imp::define_interface!(
    ID2D1SolidColorBrush,
    ID2D1SolidColorBrush_Vtbl,
    0x2cd906a9_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1SolidColorBrush {
    type Target = ID2D1Brush;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    ID2D1SolidColorBrush,
    windows_core::IUnknown,
    ID2D1Resource,
    ID2D1Brush
);
#[repr(C)]
pub struct ID2D1SolidColorBrush_Vtbl {
    pub base__: ID2D1Brush_Vtbl,
    SetColor: usize,
    GetColor: usize,
}
impl windows_core::RuntimeName for ID2D1SolidColorBrush {}
windows_core::imp::define_interface!(
    ID2D1StrokeStyle,
    ID2D1StrokeStyle_Vtbl,
    0x2cd9069d_12e2_11dc_9fed_001143a055f9
);
impl core::ops::Deref for ID2D1StrokeStyle {
    type Target = ID2D1Resource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(ID2D1StrokeStyle, windows_core::IUnknown, ID2D1Resource);
#[repr(C)]
pub struct ID2D1StrokeStyle_Vtbl {
    pub base__: ID2D1Resource_Vtbl,
    GetStartCap: usize,
    GetEndCap: usize,
    GetDashCap: usize,
    GetMiterLimit: usize,
    GetLineJoin: usize,
    GetDashOffset: usize,
    GetDashStyle: usize,
    GetDashesCount: usize,
    GetDashes: usize,
}
impl windows_core::RuntimeName for ID2D1StrokeStyle {}
pub const IDC_ARROW: windows_core::PCWSTR = windows_core::PCWSTR(32512 as _);
pub const IDC_HAND: windows_core::PCWSTR = windows_core::PCWSTR(32649 as _);
pub const IDC_IBEAM: windows_core::PCWSTR = windows_core::PCWSTR(32513 as _);
pub const IDC_SIZEALL: windows_core::PCWSTR = windows_core::PCWSTR(32646 as _);
pub const IDC_SIZENESW: windows_core::PCWSTR = windows_core::PCWSTR(32643 as _);
pub const IDC_SIZENS: windows_core::PCWSTR = windows_core::PCWSTR(32645 as _);
pub const IDC_SIZENWSE: windows_core::PCWSTR = windows_core::PCWSTR(32642 as _);
pub const IDC_SIZEWE: windows_core::PCWSTR = windows_core::PCWSTR(32644 as _);
windows_core::imp::define_interface!(
    IDWriteFontFace,
    IDWriteFontFace_Vtbl,
    0x5f49804d_7024_4d43_bfa9_d25984f53849
);
windows_core::imp::interface_hierarchy!(IDWriteFontFace, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteFontFace_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetType: usize,
    GetFiles: usize,
    GetIndex: usize,
    GetSimulations: usize,
    IsSymbolFont: usize,
    GetMetrics: usize,
    GetGlyphCount: usize,
    GetDesignGlyphMetrics: usize,
    GetGlyphIndices: usize,
    TryGetFontTable: usize,
    ReleaseFontTable: usize,
    GetGlyphRunOutline: usize,
    GetRecommendedRenderingMode: usize,
    GetGdiCompatibleMetrics: usize,
    GetGdiCompatibleGlyphMetrics: usize,
}
impl windows_core::RuntimeName for IDWriteFontFace {}
windows_core::imp::define_interface!(
    IDWriteRenderingParams,
    IDWriteRenderingParams_Vtbl,
    0x2f0da53a_2add_47cd_82ee_d9ec34688e75
);
windows_core::imp::interface_hierarchy!(IDWriteRenderingParams, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteRenderingParams_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetGamma: usize,
    GetEnhancedContrast: usize,
    GetClearTypeLevel: usize,
    GetPixelGeometry: usize,
    GetRenderingMode: usize,
}
impl windows_core::RuntimeName for IDWriteRenderingParams {}
windows_core::imp::define_interface!(
    IDWriteTextFormat,
    IDWriteTextFormat_Vtbl,
    0x9c906818_31d7_4fd3_a151_7c5e225db55a
);
windows_core::imp::interface_hierarchy!(IDWriteTextFormat, windows_core::IUnknown);
#[repr(C)]
pub struct IDWriteTextFormat_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    SetTextAlignment: usize,
    SetParagraphAlignment: usize,
    SetWordWrapping: usize,
    SetReadingDirection: usize,
    SetFlowDirection: usize,
    SetIncrementalTabStop: usize,
    SetTrimming: usize,
    SetLineSpacing: usize,
    GetTextAlignment: usize,
    GetParagraphAlignment: usize,
    GetWordWrapping: usize,
    GetReadingDirection: usize,
    GetFlowDirection: usize,
    GetIncrementalTabStop: usize,
    GetTrimming: usize,
    GetLineSpacing: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
}
impl windows_core::RuntimeName for IDWriteTextFormat {}
windows_core::imp::define_interface!(
    IDWriteTextLayout,
    IDWriteTextLayout_Vtbl,
    0x53737037_6d14_410b_9bfe_0b182bb70961
);
impl core::ops::Deref for IDWriteTextLayout {
    type Target = IDWriteTextFormat;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IDWriteTextLayout,
    windows_core::IUnknown,
    IDWriteTextFormat
);
#[repr(C)]
pub struct IDWriteTextLayout_Vtbl {
    pub base__: IDWriteTextFormat_Vtbl,
    SetMaxWidth: usize,
    SetMaxHeight: usize,
    SetFontCollection: usize,
    SetFontFamilyName: usize,
    SetFontWeight: usize,
    SetFontStyle: usize,
    SetFontStretch: usize,
    SetFontSize: usize,
    SetUnderline: usize,
    SetStrikethrough: usize,
    SetDrawingEffect: usize,
    SetInlineObject: usize,
    SetTypography: usize,
    SetLocaleName: usize,
    GetMaxWidth: usize,
    GetMaxHeight: usize,
    GetFontCollection: usize,
    GetFontFamilyNameLength: usize,
    GetFontFamilyName: usize,
    GetFontWeight: usize,
    GetFontStyle: usize,
    GetFontStretch: usize,
    GetFontSize: usize,
    GetUnderline: usize,
    GetStrikethrough: usize,
    GetDrawingEffect: usize,
    GetInlineObject: usize,
    GetTypography: usize,
    GetLocaleNameLength: usize,
    GetLocaleName: usize,
    Draw: usize,
    GetLineMetrics: usize,
    GetMetrics: usize,
    GetOverhangMetrics: usize,
    GetClusterMetrics: usize,
    DetermineMinWidth: usize,
    HitTestPoint: usize,
    HitTestTextPosition: usize,
    HitTestTextRange: usize,
}
impl windows_core::RuntimeName for IDWriteTextLayout {}
windows_core::imp::define_interface!(
    IDataObject,
    IDataObject_Vtbl,
    0x0000010e_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IDataObject, windows_core::IUnknown);
impl IDataObject {
    pub(crate) unsafe fn GetData(
        &self,
        pformatetcin: *const FORMATETC,
    ) -> windows_core::Result<STGMEDIUM> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetData)(
                windows_core::Interface::as_raw(self),
                pformatetcin,
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        }
    }
    pub(crate) unsafe fn GetDataHere(
        &self,
        pformatetc: *const FORMATETC,
        pmedium: *mut STGMEDIUM,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDataHere)(
                windows_core::Interface::as_raw(self),
                pformatetc,
                pmedium,
            )
        }
    }
    pub(crate) unsafe fn QueryGetData(
        &self,
        pformatetc: *const FORMATETC,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).QueryGetData)(
                windows_core::Interface::as_raw(self),
                pformatetc,
            )
        }
    }
    pub(crate) unsafe fn GetCanonicalFormatEtc(
        &self,
        pformatectin: *const FORMATETC,
        pformatetcout: *mut FORMATETC,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetCanonicalFormatEtc)(
                windows_core::Interface::as_raw(self),
                pformatectin,
                pformatetcout as _,
            )
        }
    }
    pub(crate) unsafe fn SetData(
        &self,
        pformatetc: *const FORMATETC,
        pmedium: *const STGMEDIUM,
        frelease: bool,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetData)(
                windows_core::Interface::as_raw(self),
                pformatetc,
                pmedium,
                frelease.into(),
            )
        }
    }
    pub(crate) unsafe fn EnumFormatEtc(
        &self,
        dwdirection: u32,
    ) -> windows_core::Result<IEnumFORMATETC> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).EnumFormatEtc)(
                windows_core::Interface::as_raw(self),
                dwdirection,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn DAdvise<P2>(
        &self,
        pformatetc: *const FORMATETC,
        advf: u32,
        padvsink: P2,
    ) -> windows_core::Result<u32>
    where
        P2: windows_core::Param<IAdviseSink>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).DAdvise)(
                windows_core::Interface::as_raw(self),
                pformatetc,
                advf,
                padvsink.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn DUnadvise(&self, dwconnection: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DUnadvise)(
                windows_core::Interface::as_raw(self),
                dwconnection,
            )
        }
    }
    pub(crate) unsafe fn EnumDAdvise(&self) -> windows_core::Result<IEnumSTATDATA> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).EnumDAdvise)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IDataObject_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetData: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const FORMATETC,
        *mut STGMEDIUM,
    ) -> windows_core::HRESULT,
    pub GetDataHere: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const FORMATETC,
        *mut STGMEDIUM,
    ) -> windows_core::HRESULT,
    pub QueryGetData: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const FORMATETC,
    ) -> windows_core::HRESULT,
    pub GetCanonicalFormatEtc: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const FORMATETC,
        *mut FORMATETC,
    ) -> windows_core::HRESULT,
    pub SetData: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const FORMATETC,
        *const STGMEDIUM,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub EnumFormatEtc: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DAdvise: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const FORMATETC,
        u32,
        *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub DUnadvise: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub EnumDAdvise: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IDataObject_Impl: windows_core::IUnknownImpl {
    fn GetData(&self, pformatetcin: *const FORMATETC) -> windows_core::Result<STGMEDIUM>;
    fn GetDataHere(
        &self,
        pformatetc: *const FORMATETC,
        pmedium: *mut STGMEDIUM,
    ) -> windows_core::Result<()>;
    fn QueryGetData(&self, pformatetc: *const FORMATETC) -> windows_core::Result<()>;
    fn GetCanonicalFormatEtc(
        &self,
        pformatectin: *const FORMATETC,
        pformatetcout: *mut FORMATETC,
    ) -> windows_core::Result<()>;
    fn SetData(
        &self,
        pformatetc: *const FORMATETC,
        pmedium: *const STGMEDIUM,
        frelease: windows_core::BOOL,
    ) -> windows_core::Result<()>;
    fn EnumFormatEtc(&self, dwdirection: u32) -> windows_core::Result<IEnumFORMATETC>;
    fn DAdvise(
        &self,
        pformatetc: *const FORMATETC,
        advf: u32,
        padvsink: windows_core::Ref<IAdviseSink>,
    ) -> windows_core::Result<u32>;
    fn DUnadvise(&self, dwconnection: u32) -> windows_core::Result<()>;
    fn EnumDAdvise(&self) -> windows_core::Result<IEnumSTATDATA>;
}
impl IDataObject_Vtbl {
    pub const fn new<Identity: IDataObject_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetData<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pformatetcin: *const FORMATETC,
            pmedium: *mut STGMEDIUM,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDataObject_Impl::GetData(this, core::mem::transmute_copy(&pformatetcin)) {
                    Ok(ok__) => {
                        pmedium.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetDataHere<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pformatetc: *const FORMATETC,
            pmedium: *mut STGMEDIUM,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDataObject_Impl::GetDataHere(
                    this,
                    core::mem::transmute_copy(&pformatetc),
                    core::mem::transmute_copy(&pmedium),
                )
                .into()
            }
        }
        unsafe extern "system" fn QueryGetData<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pformatetc: *const FORMATETC,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDataObject_Impl::QueryGetData(this, core::mem::transmute_copy(&pformatetc)).into()
            }
        }
        unsafe extern "system" fn GetCanonicalFormatEtc<
            Identity: IDataObject_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pformatectin: *const FORMATETC,
            pformatetcout: *mut FORMATETC,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDataObject_Impl::GetCanonicalFormatEtc(
                    this,
                    core::mem::transmute_copy(&pformatectin),
                    core::mem::transmute_copy(&pformatetcout),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetData<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pformatetc: *const FORMATETC,
            pmedium: *const STGMEDIUM,
            frelease: windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDataObject_Impl::SetData(
                    this,
                    core::mem::transmute_copy(&pformatetc),
                    core::mem::transmute_copy(&pmedium),
                    core::mem::transmute_copy(&frelease),
                )
                .into()
            }
        }
        unsafe extern "system" fn EnumFormatEtc<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            dwdirection: u32,
            ppenumformatetc: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDataObject_Impl::EnumFormatEtc(this, core::mem::transmute_copy(&dwdirection))
                {
                    Ok(ok__) => {
                        ppenumformatetc.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn DAdvise<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pformatetc: *const FORMATETC,
            advf: u32,
            padvsink: *mut core::ffi::c_void,
            pdwconnection: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDataObject_Impl::DAdvise(
                    this,
                    core::mem::transmute_copy(&pformatetc),
                    core::mem::transmute_copy(&advf),
                    core::mem::transmute_copy(&padvsink),
                ) {
                    Ok(ok__) => {
                        pdwconnection.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn DUnadvise<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            dwconnection: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDataObject_Impl::DUnadvise(this, core::mem::transmute_copy(&dwconnection)).into()
            }
        }
        unsafe extern "system" fn EnumDAdvise<Identity: IDataObject_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppenumadvise: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDataObject_Impl::EnumDAdvise(this) {
                    Ok(ok__) => {
                        ppenumadvise.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetData: GetData::<Identity, OFFSET>,
            GetDataHere: GetDataHere::<Identity, OFFSET>,
            QueryGetData: QueryGetData::<Identity, OFFSET>,
            GetCanonicalFormatEtc: GetCanonicalFormatEtc::<Identity, OFFSET>,
            SetData: SetData::<Identity, OFFSET>,
            EnumFormatEtc: EnumFormatEtc::<Identity, OFFSET>,
            DAdvise: DAdvise::<Identity, OFFSET>,
            DUnadvise: DUnadvise::<Identity, OFFSET>,
            EnumDAdvise: EnumDAdvise::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDataObject as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDataObject {}
windows_core::imp::define_interface!(
    IDesktopWallpaper,
    IDesktopWallpaper_Vtbl,
    0xb92b56a9_8b55_4e14_9a89_0199bbb6f93b
);
windows_core::imp::interface_hierarchy!(IDesktopWallpaper, windows_core::IUnknown);
impl IDesktopWallpaper {
    pub(crate) unsafe fn SetWallpaper<P0, P1>(
        &self,
        monitorid: P0,
        wallpaper: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
        P1: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetWallpaper)(
                windows_core::Interface::as_raw(self),
                monitorid.param().abi(),
                wallpaper.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetWallpaper<P0>(
        &self,
        monitorid: P0,
    ) -> windows_core::Result<windows_core::PWSTR>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetWallpaper)(
                windows_core::Interface::as_raw(self),
                monitorid.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetMonitorDevicePathAt(
        &self,
        monitorindex: u32,
    ) -> windows_core::Result<windows_core::PWSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetMonitorDevicePathAt)(
                windows_core::Interface::as_raw(self),
                monitorindex,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetMonitorDevicePathCount(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetMonitorDevicePathCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetMonitorRECT<P0>(&self, monitorid: P0) -> windows_core::Result<RECT>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetMonitorRECT)(
                windows_core::Interface::as_raw(self),
                monitorid.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetBackgroundColor(&self, color: COLORREF) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetBackgroundColor)(
                windows_core::Interface::as_raw(self),
                color,
            )
        }
    }
    pub(crate) unsafe fn GetBackgroundColor(&self) -> windows_core::Result<COLORREF> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetBackgroundColor)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetPosition(
        &self,
        position: DESKTOP_WALLPAPER_POSITION,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetPosition)(
                windows_core::Interface::as_raw(self),
                position,
            )
        }
    }
    pub(crate) unsafe fn GetPosition(&self) -> windows_core::Result<DESKTOP_WALLPAPER_POSITION> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPosition)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetSlideshow<P0>(&self, items: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItemArray>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetSlideshow)(
                windows_core::Interface::as_raw(self),
                items.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetSlideshow(&self) -> windows_core::Result<IShellItemArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSlideshow)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn SetSlideshowOptions(
        &self,
        options: DESKTOP_SLIDESHOW_OPTIONS,
        slideshowtick: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetSlideshowOptions)(
                windows_core::Interface::as_raw(self),
                options,
                slideshowtick,
            )
        }
    }
    pub(crate) unsafe fn GetSlideshowOptions(
        &self,
        options: *mut DESKTOP_SLIDESHOW_OPTIONS,
        slideshowtick: *mut u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetSlideshowOptions)(
                windows_core::Interface::as_raw(self),
                options as _,
                slideshowtick as _,
            )
        }
    }
    pub(crate) unsafe fn AdvanceSlideshow<P0>(
        &self,
        monitorid: P0,
        direction: DESKTOP_SLIDESHOW_DIRECTION,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AdvanceSlideshow)(
                windows_core::Interface::as_raw(self),
                monitorid.param().abi(),
                direction,
            )
        }
    }
    pub(crate) unsafe fn GetStatus(&self) -> windows_core::Result<DESKTOP_SLIDESHOW_STATE> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetStatus)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Enable(&self, enable: bool) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Enable)(
                windows_core::Interface::as_raw(self),
                enable.into(),
            )
        }
    }
}
#[repr(C)]
pub struct IDesktopWallpaper_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub SetWallpaper: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetWallpaper: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut windows_core::PWSTR,
    ) -> windows_core::HRESULT,
    pub GetMonitorDevicePathAt: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut windows_core::PWSTR,
    ) -> windows_core::HRESULT,
    pub GetMonitorDevicePathCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub GetMonitorRECT: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut RECT,
    ) -> windows_core::HRESULT,
    pub SetBackgroundColor:
        unsafe extern "system" fn(*mut core::ffi::c_void, COLORREF) -> windows_core::HRESULT,
    pub GetBackgroundColor:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut COLORREF) -> windows_core::HRESULT,
    pub SetPosition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DESKTOP_WALLPAPER_POSITION,
    ) -> windows_core::HRESULT,
    pub GetPosition: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DESKTOP_WALLPAPER_POSITION,
    ) -> windows_core::HRESULT,
    pub SetSlideshow: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetSlideshow: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetSlideshowOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        DESKTOP_SLIDESHOW_OPTIONS,
        u32,
    ) -> windows_core::HRESULT,
    pub GetSlideshowOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DESKTOP_SLIDESHOW_OPTIONS,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub AdvanceSlideshow: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        DESKTOP_SLIDESHOW_DIRECTION,
    ) -> windows_core::HRESULT,
    pub GetStatus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut DESKTOP_SLIDESHOW_STATE,
    ) -> windows_core::HRESULT,
    pub Enable: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IDesktopWallpaper_Impl: windows_core::IUnknownImpl {
    fn SetWallpaper(
        &self,
        monitorid: &windows_core::PCWSTR,
        wallpaper: &windows_core::PCWSTR,
    ) -> windows_core::Result<()>;
    fn GetWallpaper(
        &self,
        monitorid: &windows_core::PCWSTR,
    ) -> windows_core::Result<windows_core::PWSTR>;
    fn GetMonitorDevicePathAt(
        &self,
        monitorindex: u32,
    ) -> windows_core::Result<windows_core::PWSTR>;
    fn GetMonitorDevicePathCount(&self) -> windows_core::Result<u32>;
    fn GetMonitorRECT(&self, monitorid: &windows_core::PCWSTR) -> windows_core::Result<RECT>;
    fn SetBackgroundColor(&self, color: COLORREF) -> windows_core::Result<()>;
    fn GetBackgroundColor(&self) -> windows_core::Result<COLORREF>;
    fn SetPosition(&self, position: DESKTOP_WALLPAPER_POSITION) -> windows_core::Result<()>;
    fn GetPosition(&self) -> windows_core::Result<DESKTOP_WALLPAPER_POSITION>;
    fn SetSlideshow(&self, items: windows_core::Ref<IShellItemArray>) -> windows_core::Result<()>;
    fn GetSlideshow(&self) -> windows_core::Result<IShellItemArray>;
    fn SetSlideshowOptions(
        &self,
        options: DESKTOP_SLIDESHOW_OPTIONS,
        slideshowtick: u32,
    ) -> windows_core::Result<()>;
    fn GetSlideshowOptions(
        &self,
        options: *mut DESKTOP_SLIDESHOW_OPTIONS,
        slideshowtick: *mut u32,
    ) -> windows_core::Result<()>;
    fn AdvanceSlideshow(
        &self,
        monitorid: &windows_core::PCWSTR,
        direction: DESKTOP_SLIDESHOW_DIRECTION,
    ) -> windows_core::Result<()>;
    fn GetStatus(&self) -> windows_core::Result<DESKTOP_SLIDESHOW_STATE>;
    fn Enable(&self, enable: windows_core::BOOL) -> windows_core::Result<()>;
}
impl IDesktopWallpaper_Vtbl {
    pub const fn new<Identity: IDesktopWallpaper_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetWallpaper<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            monitorid: windows_core::PCWSTR,
            wallpaper: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::SetWallpaper(
                    this,
                    core::mem::transmute(&monitorid),
                    core::mem::transmute(&wallpaper),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetWallpaper<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            monitorid: windows_core::PCWSTR,
            wallpaper: *mut windows_core::PWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetWallpaper(this, core::mem::transmute(&monitorid)) {
                    Ok(ok__) => {
                        wallpaper.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetMonitorDevicePathAt<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            monitorindex: u32,
            monitorid: *mut windows_core::PWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetMonitorDevicePathAt(
                    this,
                    core::mem::transmute_copy(&monitorindex),
                ) {
                    Ok(ok__) => {
                        monitorid.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetMonitorDevicePathCount<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            count: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetMonitorDevicePathCount(this) {
                    Ok(ok__) => {
                        count.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetMonitorRECT<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            monitorid: windows_core::PCWSTR,
            displayrect: *mut RECT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetMonitorRECT(this, core::mem::transmute(&monitorid))
                {
                    Ok(ok__) => {
                        displayrect.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetBackgroundColor<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            color: COLORREF,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::SetBackgroundColor(this, core::mem::transmute_copy(&color))
                    .into()
            }
        }
        unsafe extern "system" fn GetBackgroundColor<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            color: *mut COLORREF,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetBackgroundColor(this) {
                    Ok(ok__) => {
                        color.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetPosition<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            position: DESKTOP_WALLPAPER_POSITION,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::SetPosition(this, core::mem::transmute_copy(&position))
                    .into()
            }
        }
        unsafe extern "system" fn GetPosition<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            position: *mut DESKTOP_WALLPAPER_POSITION,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetPosition(this) {
                    Ok(ok__) => {
                        position.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetSlideshow<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            items: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::SetSlideshow(this, core::mem::transmute_copy(&items)).into()
            }
        }
        unsafe extern "system" fn GetSlideshow<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            items: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetSlideshow(this) {
                    Ok(ok__) => {
                        items.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetSlideshowOptions<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            options: DESKTOP_SLIDESHOW_OPTIONS,
            slideshowtick: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::SetSlideshowOptions(
                    this,
                    core::mem::transmute_copy(&options),
                    core::mem::transmute_copy(&slideshowtick),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetSlideshowOptions<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            options: *mut DESKTOP_SLIDESHOW_OPTIONS,
            slideshowtick: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::GetSlideshowOptions(
                    this,
                    core::mem::transmute_copy(&options),
                    core::mem::transmute_copy(&slideshowtick),
                )
                .into()
            }
        }
        unsafe extern "system" fn AdvanceSlideshow<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            monitorid: windows_core::PCWSTR,
            direction: DESKTOP_SLIDESHOW_DIRECTION,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::AdvanceSlideshow(
                    this,
                    core::mem::transmute(&monitorid),
                    core::mem::transmute_copy(&direction),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetStatus<
            Identity: IDesktopWallpaper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            state: *mut DESKTOP_SLIDESHOW_STATE,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IDesktopWallpaper_Impl::GetStatus(this) {
                    Ok(ok__) => {
                        state.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Enable<Identity: IDesktopWallpaper_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            enable: windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDesktopWallpaper_Impl::Enable(this, core::mem::transmute_copy(&enable)).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            SetWallpaper: SetWallpaper::<Identity, OFFSET>,
            GetWallpaper: GetWallpaper::<Identity, OFFSET>,
            GetMonitorDevicePathAt: GetMonitorDevicePathAt::<Identity, OFFSET>,
            GetMonitorDevicePathCount: GetMonitorDevicePathCount::<Identity, OFFSET>,
            GetMonitorRECT: GetMonitorRECT::<Identity, OFFSET>,
            SetBackgroundColor: SetBackgroundColor::<Identity, OFFSET>,
            GetBackgroundColor: GetBackgroundColor::<Identity, OFFSET>,
            SetPosition: SetPosition::<Identity, OFFSET>,
            GetPosition: GetPosition::<Identity, OFFSET>,
            SetSlideshow: SetSlideshow::<Identity, OFFSET>,
            GetSlideshow: GetSlideshow::<Identity, OFFSET>,
            SetSlideshowOptions: SetSlideshowOptions::<Identity, OFFSET>,
            GetSlideshowOptions: GetSlideshowOptions::<Identity, OFFSET>,
            AdvanceSlideshow: AdvanceSlideshow::<Identity, OFFSET>,
            GetStatus: GetStatus::<Identity, OFFSET>,
            Enable: Enable::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDesktopWallpaper as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDesktopWallpaper {}
windows_core::imp::define_interface!(
    IDispatcherQueueController,
    IDispatcherQueueController_Vtbl,
    0x22f34e66_50db_4e36_a98d_61c01b384d20
);
impl windows_core::RuntimeType for IDispatcherQueueController {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_interface::<Self>();
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.System.IDispatcherQueueController");
}
#[repr(C)]
pub struct IDispatcherQueueController_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_interface!(
    IDropSource,
    IDropSource_Vtbl,
    0x00000121_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IDropSource, windows_core::IUnknown);
#[repr(C)]
pub struct IDropSource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    QueryContinueDrag: usize,
    GiveFeedback: usize,
}
impl windows_core::RuntimeName for IDropSource {}
windows_core::imp::define_interface!(
    IDropTarget,
    IDropTarget_Vtbl,
    0x00000122_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IDropTarget, windows_core::IUnknown);
impl IDropTarget {
    pub(crate) unsafe fn DragEnter<P0>(
        &self,
        pdataobj: P0,
        grfkeystate: u32,
        pt: POINTL,
        pdweffect: *mut u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IDataObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DragEnter)(
                windows_core::Interface::as_raw(self),
                pdataobj.param().abi(),
                grfkeystate,
                pt,
                pdweffect as _,
            )
        }
    }
    pub(crate) unsafe fn DragOver(
        &self,
        grfkeystate: u32,
        pt: POINTL,
        pdweffect: *mut u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DragOver)(
                windows_core::Interface::as_raw(self),
                grfkeystate,
                pt,
                pdweffect as _,
            )
        }
    }
    pub(crate) unsafe fn DragLeave(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DragLeave)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn Drop<P0>(
        &self,
        pdataobj: P0,
        grfkeystate: u32,
        pt: POINTL,
        pdweffect: *mut u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IDataObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Drop)(
                windows_core::Interface::as_raw(self),
                pdataobj.param().abi(),
                grfkeystate,
                pt,
                pdweffect as _,
            )
        }
    }
}
#[repr(C)]
pub struct IDropTarget_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub DragEnter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        POINTL,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub DragOver: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        POINTL,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub DragLeave: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Drop: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        POINTL,
        *mut u32,
    ) -> windows_core::HRESULT,
}
pub trait IDropTarget_Impl: windows_core::IUnknownImpl {
    fn DragEnter(
        &self,
        pdataobj: windows_core::Ref<IDataObject>,
        grfkeystate: u32,
        pt: &POINTL,
        pdweffect: *mut u32,
    ) -> windows_core::Result<()>;
    fn DragOver(
        &self,
        grfkeystate: u32,
        pt: &POINTL,
        pdweffect: *mut u32,
    ) -> windows_core::Result<()>;
    fn DragLeave(&self) -> windows_core::Result<()>;
    fn Drop(
        &self,
        pdataobj: windows_core::Ref<IDataObject>,
        grfkeystate: u32,
        pt: &POINTL,
        pdweffect: *mut u32,
    ) -> windows_core::Result<()>;
}
impl IDropTarget_Vtbl {
    pub const fn new<Identity: IDropTarget_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn DragEnter<Identity: IDropTarget_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pdataobj: *mut core::ffi::c_void,
            grfkeystate: u32,
            pt: POINTL,
            pdweffect: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTarget_Impl::DragEnter(
                    this,
                    core::mem::transmute_copy(&pdataobj),
                    core::mem::transmute_copy(&grfkeystate),
                    core::mem::transmute(&pt),
                    core::mem::transmute_copy(&pdweffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn DragOver<Identity: IDropTarget_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            grfkeystate: u32,
            pt: POINTL,
            pdweffect: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTarget_Impl::DragOver(
                    this,
                    core::mem::transmute_copy(&grfkeystate),
                    core::mem::transmute(&pt),
                    core::mem::transmute_copy(&pdweffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn DragLeave<Identity: IDropTarget_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTarget_Impl::DragLeave(this).into()
            }
        }
        unsafe extern "system" fn Drop<Identity: IDropTarget_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pdataobj: *mut core::ffi::c_void,
            grfkeystate: u32,
            pt: POINTL,
            pdweffect: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTarget_Impl::Drop(
                    this,
                    core::mem::transmute_copy(&pdataobj),
                    core::mem::transmute_copy(&grfkeystate),
                    core::mem::transmute(&pt),
                    core::mem::transmute_copy(&pdweffect),
                )
                .into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            DragEnter: DragEnter::<Identity, OFFSET>,
            DragOver: DragOver::<Identity, OFFSET>,
            DragLeave: DragLeave::<Identity, OFFSET>,
            Drop: Drop::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDropTarget as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDropTarget {}
windows_core::imp::define_interface!(
    IDropTargetHelper,
    IDropTargetHelper_Vtbl,
    0x4657278b_411b_11d2_839a_00c04fd918d0
);
windows_core::imp::interface_hierarchy!(IDropTargetHelper, windows_core::IUnknown);
impl IDropTargetHelper {
    pub(crate) unsafe fn DragEnter<P1>(
        &self,
        hwndtarget: HWND,
        pdataobject: P1,
        ppt: *const POINT,
        dweffect: u32,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IDataObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DragEnter)(
                windows_core::Interface::as_raw(self),
                hwndtarget,
                pdataobject.param().abi(),
                ppt,
                dweffect,
            )
        }
    }
    pub(crate) unsafe fn DragLeave(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DragLeave)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn DragOver(
        &self,
        ppt: *const POINT,
        dweffect: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).DragOver)(
                windows_core::Interface::as_raw(self),
                ppt,
                dweffect,
            )
        }
    }
    pub(crate) unsafe fn Drop<P0>(
        &self,
        pdataobject: P0,
        ppt: *const POINT,
        dweffect: u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IDataObject>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Drop)(
                windows_core::Interface::as_raw(self),
                pdataobject.param().abi(),
                ppt,
                dweffect,
            )
        }
    }
    pub(crate) unsafe fn Show(&self, fshow: bool) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Show)(
                windows_core::Interface::as_raw(self),
                fshow.into(),
            )
        }
    }
}
#[repr(C)]
pub struct IDropTargetHelper_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub DragEnter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        *mut core::ffi::c_void,
        *const POINT,
        u32,
    ) -> windows_core::HRESULT,
    pub DragLeave: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub DragOver: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const POINT,
        u32,
    ) -> windows_core::HRESULT,
    pub Drop: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const POINT,
        u32,
    ) -> windows_core::HRESULT,
    pub Show: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IDropTargetHelper_Impl: windows_core::IUnknownImpl {
    fn DragEnter(
        &self,
        hwndtarget: HWND,
        pdataobject: windows_core::Ref<IDataObject>,
        ppt: *const POINT,
        dweffect: u32,
    ) -> windows_core::Result<()>;
    fn DragLeave(&self) -> windows_core::Result<()>;
    fn DragOver(&self, ppt: *const POINT, dweffect: u32) -> windows_core::Result<()>;
    fn Drop(
        &self,
        pdataobject: windows_core::Ref<IDataObject>,
        ppt: *const POINT,
        dweffect: u32,
    ) -> windows_core::Result<()>;
    fn Show(&self, fshow: windows_core::BOOL) -> windows_core::Result<()>;
}
impl IDropTargetHelper_Vtbl {
    pub const fn new<Identity: IDropTargetHelper_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn DragEnter<
            Identity: IDropTargetHelper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwndtarget: HWND,
            pdataobject: *mut core::ffi::c_void,
            ppt: *const POINT,
            dweffect: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTargetHelper_Impl::DragEnter(
                    this,
                    core::mem::transmute_copy(&hwndtarget),
                    core::mem::transmute_copy(&pdataobject),
                    core::mem::transmute_copy(&ppt),
                    core::mem::transmute_copy(&dweffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn DragLeave<
            Identity: IDropTargetHelper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTargetHelper_Impl::DragLeave(this).into()
            }
        }
        unsafe extern "system" fn DragOver<
            Identity: IDropTargetHelper_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppt: *const POINT,
            dweffect: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTargetHelper_Impl::DragOver(
                    this,
                    core::mem::transmute_copy(&ppt),
                    core::mem::transmute_copy(&dweffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn Drop<Identity: IDropTargetHelper_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pdataobject: *mut core::ffi::c_void,
            ppt: *const POINT,
            dweffect: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTargetHelper_Impl::Drop(
                    this,
                    core::mem::transmute_copy(&pdataobject),
                    core::mem::transmute_copy(&ppt),
                    core::mem::transmute_copy(&dweffect),
                )
                .into()
            }
        }
        unsafe extern "system" fn Show<Identity: IDropTargetHelper_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            fshow: windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IDropTargetHelper_Impl::Show(this, core::mem::transmute_copy(&fshow)).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            DragEnter: DragEnter::<Identity, OFFSET>,
            DragLeave: DragLeave::<Identity, OFFSET>,
            DragOver: DragOver::<Identity, OFFSET>,
            Drop: Drop::<Identity, OFFSET>,
            Show: Show::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IDropTargetHelper as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IDropTargetHelper {}
windows_core::imp::define_interface!(
    IEnumFORMATETC,
    IEnumFORMATETC_Vtbl,
    0x00000103_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IEnumFORMATETC, windows_core::IUnknown);
impl IEnumFORMATETC {
    pub(crate) unsafe fn Next(
        &self,
        celt: u32,
        rgelt: *mut FORMATETC,
        pceltfetched: Option<*mut u32>,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Next)(
                windows_core::Interface::as_raw(self),
                celt,
                rgelt as _,
                pceltfetched.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
    pub(crate) unsafe fn Skip(&self, celt: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Skip)(
                windows_core::Interface::as_raw(self),
                celt,
            )
        }
    }
    pub(crate) unsafe fn Reset(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Reset)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn Clone(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Clone)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IEnumFORMATETC_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Next: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut FORMATETC,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub Skip: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub Reset: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Clone: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IEnumFORMATETC_Impl: windows_core::IUnknownImpl {
    fn Next(
        &self,
        celt: u32,
        rgelt: *mut FORMATETC,
        pceltfetched: *mut u32,
    ) -> windows_core::Result<()>;
    fn Skip(&self, celt: u32) -> windows_core::Result<()>;
    fn Reset(&self) -> windows_core::Result<()>;
    fn Clone(&self) -> windows_core::Result<IEnumFORMATETC>;
}
impl IEnumFORMATETC_Vtbl {
    pub const fn new<Identity: IEnumFORMATETC_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Next<Identity: IEnumFORMATETC_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            celt: u32,
            rgelt: *mut FORMATETC,
            pceltfetched: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IEnumFORMATETC_Impl::Next(
                    this,
                    core::mem::transmute_copy(&celt),
                    core::mem::transmute_copy(&rgelt),
                    core::mem::transmute_copy(&pceltfetched),
                )
                .into()
            }
        }
        unsafe extern "system" fn Skip<Identity: IEnumFORMATETC_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            celt: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IEnumFORMATETC_Impl::Skip(this, core::mem::transmute_copy(&celt)).into()
            }
        }
        unsafe extern "system" fn Reset<Identity: IEnumFORMATETC_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IEnumFORMATETC_Impl::Reset(this).into()
            }
        }
        unsafe extern "system" fn Clone<Identity: IEnumFORMATETC_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppenum: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IEnumFORMATETC_Impl::Clone(this) {
                    Ok(ok__) => {
                        ppenum.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Next: Next::<Identity, OFFSET>,
            Skip: Skip::<Identity, OFFSET>,
            Reset: Reset::<Identity, OFFSET>,
            Clone: Clone::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IEnumFORMATETC as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IEnumFORMATETC {}
windows_core::imp::define_interface!(
    IEnumIDList,
    IEnumIDList_Vtbl,
    0x000214f2_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IEnumIDList, windows_core::IUnknown);
#[repr(C)]
pub struct IEnumIDList_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Next: usize,
    Skip: usize,
    Reset: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IEnumIDList {}
windows_core::imp::define_interface!(
    IEnumSTATDATA,
    IEnumSTATDATA_Vtbl,
    0x00000105_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IEnumSTATDATA, windows_core::IUnknown);
#[repr(C)]
pub struct IEnumSTATDATA_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Next: usize,
    Skip: usize,
    Reset: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IEnumSTATDATA {}
windows_core::imp::define_interface!(
    IEnumShellItems,
    IEnumShellItems_Vtbl,
    0x70629033_e363_4a28_a567_0db78006e6d7
);
windows_core::imp::interface_hierarchy!(IEnumShellItems, windows_core::IUnknown);
#[repr(C)]
pub struct IEnumShellItems_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Next: usize,
    Skip: usize,
    Reset: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IEnumShellItems {}
windows_core::imp::define_interface!(
    IEnumUnknown,
    IEnumUnknown_Vtbl,
    0x00000100_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IEnumUnknown, windows_core::IUnknown);
#[repr(C)]
pub struct IEnumUnknown_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Next: usize,
    Skip: usize,
    Reset: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IEnumUnknown {}
windows_core::imp::define_interface!(
    IFileDialog,
    IFileDialog_Vtbl,
    0x42f85136_db7e_439c_85f1_e4075d135fc8
);
impl core::ops::Deref for IFileDialog {
    type Target = IModalWindow;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IFileDialog, windows_core::IUnknown, IModalWindow);
impl IFileDialog {
    pub(crate) unsafe fn SetFileTypes(
        &self,
        cfiletypes: u32,
        rgfilterspec: *const COMDLG_FILTERSPEC,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetFileTypes)(
                windows_core::Interface::as_raw(self),
                cfiletypes,
                rgfilterspec,
            )
        }
    }
    pub(crate) unsafe fn SetFileTypeIndex(&self, ifiletype: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetFileTypeIndex)(
                windows_core::Interface::as_raw(self),
                ifiletype,
            )
        }
    }
    pub(crate) unsafe fn GetFileTypeIndex(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFileTypeIndex)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Advise<P0>(&self, pfde: P0) -> windows_core::Result<u32>
    where
        P0: windows_core::Param<IFileDialogEvents>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Advise)(
                windows_core::Interface::as_raw(self),
                pfde.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Unadvise(&self, dwcookie: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Unadvise)(
                windows_core::Interface::as_raw(self),
                dwcookie,
            )
        }
    }
    pub(crate) unsafe fn SetOptions(&self, fos: FILEOPENDIALOGOPTIONS) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetOptions)(
                windows_core::Interface::as_raw(self),
                fos,
            )
        }
    }
    pub(crate) unsafe fn GetOptions(&self) -> windows_core::Result<FILEOPENDIALOGOPTIONS> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetOptions)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetDefaultFolder<P0>(&self, psi: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetDefaultFolder)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetFolder<P0>(&self, psi: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFolder)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetFolder(&self) -> windows_core::Result<IShellItem> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFolder)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetCurrentSelection(&self) -> windows_core::Result<IShellItem> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurrentSelection)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn SetFileName<P0>(&self, pszname: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFileName)(
                windows_core::Interface::as_raw(self),
                pszname.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetFileName(&self) -> windows_core::Result<windows_core::PWSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFileName)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetTitle<P0>(&self, psztitle: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetTitle)(
                windows_core::Interface::as_raw(self),
                psztitle.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetOkButtonLabel<P0>(&self, psztext: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetOkButtonLabel)(
                windows_core::Interface::as_raw(self),
                psztext.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetFileNameLabel<P0>(&self, pszlabel: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFileNameLabel)(
                windows_core::Interface::as_raw(self),
                pszlabel.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetResult(&self) -> windows_core::Result<IShellItem> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetResult)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn AddPlace<P0>(&self, psi: P0, fdap: FDAP) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).AddPlace)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
                fdap,
            )
        }
    }
    pub(crate) unsafe fn SetDefaultExtension<P0>(
        &self,
        pszdefaultextension: P0,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetDefaultExtension)(
                windows_core::Interface::as_raw(self),
                pszdefaultextension.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn Close(&self, hr: windows_core::HRESULT) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Close)(windows_core::Interface::as_raw(self), hr)
        }
    }
    pub(crate) unsafe fn SetClientGuid(
        &self,
        guid: *const windows_core::GUID,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetClientGuid)(
                windows_core::Interface::as_raw(self),
                guid,
            )
        }
    }
    pub(crate) unsafe fn ClearClientData(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).ClearClientData)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn SetFilter<P0>(&self, pfilter: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItemFilter>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetFilter)(
                windows_core::Interface::as_raw(self),
                pfilter.param().abi(),
            )
        }
    }
}
#[repr(C)]
pub struct IFileDialog_Vtbl {
    pub base__: IModalWindow_Vtbl,
    pub SetFileTypes: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const COMDLG_FILTERSPEC,
    ) -> windows_core::HRESULT,
    pub SetFileTypeIndex:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub GetFileTypeIndex:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub Advise: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub Unadvise: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub SetOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        FILEOPENDIALOGOPTIONS,
    ) -> windows_core::HRESULT,
    pub GetOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut FILEOPENDIALOGOPTIONS,
    ) -> windows_core::HRESULT,
    pub SetDefaultFolder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetFolder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFolder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetCurrentSelection: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetFileName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetFileName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::PWSTR,
    ) -> windows_core::HRESULT,
    pub SetTitle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub SetOkButtonLabel: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub SetFileNameLabel: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetResult: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub AddPlace: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        FDAP,
    ) -> windows_core::HRESULT,
    pub SetDefaultExtension: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub Close: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::HRESULT,
    ) -> windows_core::HRESULT,
    pub SetClientGuid: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
    ) -> windows_core::HRESULT,
    pub ClearClientData: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub SetFilter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IFileDialog_Impl: IModalWindow_Impl {
    fn SetFileTypes(
        &self,
        cfiletypes: u32,
        rgfilterspec: *const COMDLG_FILTERSPEC,
    ) -> windows_core::Result<()>;
    fn SetFileTypeIndex(&self, ifiletype: u32) -> windows_core::Result<()>;
    fn GetFileTypeIndex(&self) -> windows_core::Result<u32>;
    fn Advise(&self, pfde: windows_core::Ref<IFileDialogEvents>) -> windows_core::Result<u32>;
    fn Unadvise(&self, dwcookie: u32) -> windows_core::Result<()>;
    fn SetOptions(&self, fos: FILEOPENDIALOGOPTIONS) -> windows_core::Result<()>;
    fn GetOptions(&self) -> windows_core::Result<FILEOPENDIALOGOPTIONS>;
    fn SetDefaultFolder(&self, psi: windows_core::Ref<IShellItem>) -> windows_core::Result<()>;
    fn SetFolder(&self, psi: windows_core::Ref<IShellItem>) -> windows_core::Result<()>;
    fn GetFolder(&self) -> windows_core::Result<IShellItem>;
    fn GetCurrentSelection(&self) -> windows_core::Result<IShellItem>;
    fn SetFileName(&self, pszname: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn GetFileName(&self) -> windows_core::Result<windows_core::PWSTR>;
    fn SetTitle(&self, psztitle: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn SetOkButtonLabel(&self, psztext: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn SetFileNameLabel(&self, pszlabel: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn GetResult(&self) -> windows_core::Result<IShellItem>;
    fn AddPlace(&self, psi: windows_core::Ref<IShellItem>, fdap: FDAP) -> windows_core::Result<()>;
    fn SetDefaultExtension(
        &self,
        pszdefaultextension: &windows_core::PCWSTR,
    ) -> windows_core::Result<()>;
    fn Close(&self, hr: windows_core::HRESULT) -> windows_core::Result<()>;
    fn SetClientGuid(&self, guid: *const windows_core::GUID) -> windows_core::Result<()>;
    fn ClearClientData(&self) -> windows_core::Result<()>;
    fn SetFilter(&self, pfilter: windows_core::Ref<IShellItemFilter>) -> windows_core::Result<()>;
}
impl IFileDialog_Vtbl {
    pub const fn new<Identity: IFileDialog_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetFileTypes<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            cfiletypes: u32,
            rgfilterspec: *const COMDLG_FILTERSPEC,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetFileTypes(
                    this,
                    core::mem::transmute_copy(&cfiletypes),
                    core::mem::transmute_copy(&rgfilterspec),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetFileTypeIndex<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ifiletype: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetFileTypeIndex(this, core::mem::transmute_copy(&ifiletype))
                    .into()
            }
        }
        unsafe extern "system" fn GetFileTypeIndex<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pifiletype: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::GetFileTypeIndex(this) {
                    Ok(ok__) => {
                        pifiletype.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Advise<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pfde: *mut core::ffi::c_void,
            pdwcookie: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::Advise(this, core::mem::transmute_copy(&pfde)) {
                    Ok(ok__) => {
                        pdwcookie.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Unadvise<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            dwcookie: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::Unadvise(this, core::mem::transmute_copy(&dwcookie)).into()
            }
        }
        unsafe extern "system" fn SetOptions<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            fos: FILEOPENDIALOGOPTIONS,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetOptions(this, core::mem::transmute_copy(&fos)).into()
            }
        }
        unsafe extern "system" fn GetOptions<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pfos: *mut FILEOPENDIALOGOPTIONS,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::GetOptions(this) {
                    Ok(ok__) => {
                        pfos.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetDefaultFolder<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetDefaultFolder(this, core::mem::transmute_copy(&psi)).into()
            }
        }
        unsafe extern "system" fn SetFolder<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetFolder(this, core::mem::transmute_copy(&psi)).into()
            }
        }
        unsafe extern "system" fn GetFolder<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppsi: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::GetFolder(this) {
                    Ok(ok__) => {
                        ppsi.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCurrentSelection<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppsi: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::GetCurrentSelection(this) {
                    Ok(ok__) => {
                        ppsi.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetFileName<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pszname: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetFileName(this, core::mem::transmute(&pszname)).into()
            }
        }
        unsafe extern "system" fn GetFileName<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pszname: *mut windows_core::PWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::GetFileName(this) {
                    Ok(ok__) => {
                        pszname.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetTitle<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psztitle: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetTitle(this, core::mem::transmute(&psztitle)).into()
            }
        }
        unsafe extern "system" fn SetOkButtonLabel<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psztext: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetOkButtonLabel(this, core::mem::transmute(&psztext)).into()
            }
        }
        unsafe extern "system" fn SetFileNameLabel<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszlabel: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetFileNameLabel(this, core::mem::transmute(&pszlabel)).into()
            }
        }
        unsafe extern "system" fn GetResult<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppsi: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileDialog_Impl::GetResult(this) {
                    Ok(ok__) => {
                        ppsi.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn AddPlace<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
            fdap: FDAP,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::AddPlace(
                    this,
                    core::mem::transmute_copy(&psi),
                    core::mem::transmute_copy(&fdap),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetDefaultExtension<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszdefaultextension: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetDefaultExtension(
                    this,
                    core::mem::transmute(&pszdefaultextension),
                )
                .into()
            }
        }
        unsafe extern "system" fn Close<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            hr: windows_core::HRESULT,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::Close(this, core::mem::transmute_copy(&hr)).into()
            }
        }
        unsafe extern "system" fn SetClientGuid<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            guid: *const windows_core::GUID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetClientGuid(this, core::mem::transmute_copy(&guid)).into()
            }
        }
        unsafe extern "system" fn ClearClientData<
            Identity: IFileDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::ClearClientData(this).into()
            }
        }
        unsafe extern "system" fn SetFilter<Identity: IFileDialog_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pfilter: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileDialog_Impl::SetFilter(this, core::mem::transmute_copy(&pfilter)).into()
            }
        }
        Self {
            base__: IModalWindow_Vtbl::new::<Identity, OFFSET>(),
            SetFileTypes: SetFileTypes::<Identity, OFFSET>,
            SetFileTypeIndex: SetFileTypeIndex::<Identity, OFFSET>,
            GetFileTypeIndex: GetFileTypeIndex::<Identity, OFFSET>,
            Advise: Advise::<Identity, OFFSET>,
            Unadvise: Unadvise::<Identity, OFFSET>,
            SetOptions: SetOptions::<Identity, OFFSET>,
            GetOptions: GetOptions::<Identity, OFFSET>,
            SetDefaultFolder: SetDefaultFolder::<Identity, OFFSET>,
            SetFolder: SetFolder::<Identity, OFFSET>,
            GetFolder: GetFolder::<Identity, OFFSET>,
            GetCurrentSelection: GetCurrentSelection::<Identity, OFFSET>,
            SetFileName: SetFileName::<Identity, OFFSET>,
            GetFileName: GetFileName::<Identity, OFFSET>,
            SetTitle: SetTitle::<Identity, OFFSET>,
            SetOkButtonLabel: SetOkButtonLabel::<Identity, OFFSET>,
            SetFileNameLabel: SetFileNameLabel::<Identity, OFFSET>,
            GetResult: GetResult::<Identity, OFFSET>,
            AddPlace: AddPlace::<Identity, OFFSET>,
            SetDefaultExtension: SetDefaultExtension::<Identity, OFFSET>,
            Close: Close::<Identity, OFFSET>,
            SetClientGuid: SetClientGuid::<Identity, OFFSET>,
            ClearClientData: ClearClientData::<Identity, OFFSET>,
            SetFilter: SetFilter::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IFileDialog as windows_core::Interface>::IID
            || iid == &<IModalWindow as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IFileDialog {}
windows_core::imp::define_interface!(
    IFileDialogEvents,
    IFileDialogEvents_Vtbl,
    0x973510db_7d7f_452b_8975_74a85828d354
);
windows_core::imp::interface_hierarchy!(IFileDialogEvents, windows_core::IUnknown);
#[repr(C)]
pub struct IFileDialogEvents_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    OnFileOk: usize,
    OnFolderChanging: usize,
    OnFolderChange: usize,
    OnSelectionChange: usize,
    OnShareViolation: usize,
    OnTypeChange: usize,
    OnOverwrite: usize,
}
impl windows_core::RuntimeName for IFileDialogEvents {}
windows_core::imp::define_interface!(
    IFileOpenDialog,
    IFileOpenDialog_Vtbl,
    0xd57c7288_d4ad_4768_be02_9d969532d960
);
impl core::ops::Deref for IFileOpenDialog {
    type Target = IFileDialog;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IFileOpenDialog,
    windows_core::IUnknown,
    IModalWindow,
    IFileDialog
);
impl IFileOpenDialog {
    pub(crate) unsafe fn GetResults(&self) -> windows_core::Result<IShellItemArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetResults)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetSelectedItems(&self) -> windows_core::Result<IShellItemArray> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetSelectedItems)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IFileOpenDialog_Vtbl {
    pub base__: IFileDialog_Vtbl,
    pub GetResults: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetSelectedItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IFileOpenDialog_Impl: IFileDialog_Impl {
    fn GetResults(&self) -> windows_core::Result<IShellItemArray>;
    fn GetSelectedItems(&self) -> windows_core::Result<IShellItemArray>;
}
impl IFileOpenDialog_Vtbl {
    pub const fn new<Identity: IFileOpenDialog_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetResults<
            Identity: IFileOpenDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppenum: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileOpenDialog_Impl::GetResults(this) {
                    Ok(ok__) => {
                        ppenum.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetSelectedItems<
            Identity: IFileOpenDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppsai: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileOpenDialog_Impl::GetSelectedItems(this) {
                    Ok(ok__) => {
                        ppsai.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: IFileDialog_Vtbl::new::<Identity, OFFSET>(),
            GetResults: GetResults::<Identity, OFFSET>,
            GetSelectedItems: GetSelectedItems::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IFileOpenDialog as windows_core::Interface>::IID
            || iid == &<IModalWindow as windows_core::Interface>::IID
            || iid == &<IFileDialog as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IFileOpenDialog {}
windows_core::imp::define_interface!(
    IFileOperation,
    IFileOperation_Vtbl,
    0x947aab5f_0a5c_4c13_b4d6_4bf7836fc9f8
);
windows_core::imp::interface_hierarchy!(IFileOperation, windows_core::IUnknown);
impl IFileOperation {
    pub(crate) unsafe fn Advise<P0>(&self, pfops: P0) -> windows_core::Result<u32>
    where
        P0: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Advise)(
                windows_core::Interface::as_raw(self),
                pfops.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Unadvise(&self, dwcookie: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Unadvise)(
                windows_core::Interface::as_raw(self),
                dwcookie,
            )
        }
    }
    pub(crate) unsafe fn SetOperationFlags(&self, dwoperationflags: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetOperationFlags)(
                windows_core::Interface::as_raw(self),
                dwoperationflags,
            )
        }
    }
    pub(crate) unsafe fn SetProgressMessage<P0>(&self, pszmessage: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetProgressMessage)(
                windows_core::Interface::as_raw(self),
                pszmessage.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetProgressDialog<P0>(&self, popd: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IOperationsProgressDialog>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetProgressDialog)(
                windows_core::Interface::as_raw(self),
                popd.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetProperties<P0>(&self, pproparray: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IPropertyChangeArray>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetProperties)(
                windows_core::Interface::as_raw(self),
                pproparray.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetOwnerWindow(&self, hwndowner: HWND) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetOwnerWindow)(
                windows_core::Interface::as_raw(self),
                hwndowner,
            )
        }
    }
    pub(crate) unsafe fn ApplyPropertiesToItem<P0>(&self, psiitem: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ApplyPropertiesToItem)(
                windows_core::Interface::as_raw(self),
                psiitem.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn ApplyPropertiesToItems<P0>(&self, punkitems: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ApplyPropertiesToItems)(
                windows_core::Interface::as_raw(self),
                punkitems.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RenameItem<P0, P1, P2>(
        &self,
        psiitem: P0,
        psznewname: P1,
        pfopsitem: P2,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
        P1: windows_core::Param<windows_core::PCWSTR>,
        P2: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RenameItem)(
                windows_core::Interface::as_raw(self),
                psiitem.param().abi(),
                psznewname.param().abi(),
                pfopsitem.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn RenameItems<P0, P1>(
        &self,
        punkitems: P0,
        psznewname: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::IUnknown>,
        P1: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).RenameItems)(
                windows_core::Interface::as_raw(self),
                punkitems.param().abi(),
                psznewname.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn MoveItem<P0, P1, P2, P3>(
        &self,
        psiitem: P0,
        psidestinationfolder: P1,
        psznewname: P2,
        pfopsitem: P3,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
        P1: windows_core::Param<IShellItem>,
        P2: windows_core::Param<windows_core::PCWSTR>,
        P3: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).MoveItem)(
                windows_core::Interface::as_raw(self),
                psiitem.param().abi(),
                psidestinationfolder.param().abi(),
                psznewname.param().abi(),
                pfopsitem.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn MoveItems<P0, P1>(
        &self,
        punkitems: P0,
        psidestinationfolder: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::IUnknown>,
        P1: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).MoveItems)(
                windows_core::Interface::as_raw(self),
                punkitems.param().abi(),
                psidestinationfolder.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn CopyItem<P0, P1, P2, P3>(
        &self,
        psiitem: P0,
        psidestinationfolder: P1,
        pszcopyname: P2,
        pfopsitem: P3,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
        P1: windows_core::Param<IShellItem>,
        P2: windows_core::Param<windows_core::PCWSTR>,
        P3: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyItem)(
                windows_core::Interface::as_raw(self),
                psiitem.param().abi(),
                psidestinationfolder.param().abi(),
                pszcopyname.param().abi(),
                pfopsitem.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn CopyItems<P0, P1>(
        &self,
        punkitems: P0,
        psidestinationfolder: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::IUnknown>,
        P1: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyItems)(
                windows_core::Interface::as_raw(self),
                punkitems.param().abi(),
                psidestinationfolder.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn DeleteItem<P0, P1>(
        &self,
        psiitem: P0,
        pfopsitem: P1,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
        P1: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DeleteItem)(
                windows_core::Interface::as_raw(self),
                psiitem.param().abi(),
                pfopsitem.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn DeleteItems<P0>(&self, punkitems: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::IUnknown>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).DeleteItems)(
                windows_core::Interface::as_raw(self),
                punkitems.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn NewItem<P0, P2, P3, P4>(
        &self,
        psidestinationfolder: P0,
        dwfileattributes: u32,
        pszname: P2,
        psztemplatename: P3,
        pfopsitem: P4,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
        P2: windows_core::Param<windows_core::PCWSTR>,
        P3: windows_core::Param<windows_core::PCWSTR>,
        P4: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).NewItem)(
                windows_core::Interface::as_raw(self),
                psidestinationfolder.param().abi(),
                dwfileattributes,
                pszname.param().abi(),
                psztemplatename.param().abi(),
                pfopsitem.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn PerformOperations(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).PerformOperations)(
                windows_core::Interface::as_raw(self),
            )
        }
    }
    pub(crate) unsafe fn GetAnyOperationsAborted(
        &self,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAnyOperationsAborted)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IFileOperation_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Advise: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub Unadvise: unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub SetOperationFlags:
        unsafe extern "system" fn(*mut core::ffi::c_void, u32) -> windows_core::HRESULT,
    pub SetProgressMessage: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub SetProgressDialog: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetOwnerWindow:
        unsafe extern "system" fn(*mut core::ffi::c_void, HWND) -> windows_core::HRESULT,
    pub ApplyPropertiesToItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ApplyPropertiesToItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RenameItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub RenameItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub MoveItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub MoveItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CopyItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CopyItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DeleteItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub DeleteItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub NewItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        windows_core::PCWSTR,
        windows_core::PCWSTR,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub PerformOperations:
        unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub GetAnyOperationsAborted: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IFileOperation_Impl: windows_core::IUnknownImpl {
    fn Advise(
        &self,
        pfops: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<u32>;
    fn Unadvise(&self, dwcookie: u32) -> windows_core::Result<()>;
    fn SetOperationFlags(&self, dwoperationflags: u32) -> windows_core::Result<()>;
    fn SetProgressMessage(&self, pszmessage: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn SetProgressDialog(
        &self,
        popd: windows_core::Ref<IOperationsProgressDialog>,
    ) -> windows_core::Result<()>;
    fn SetProperties(
        &self,
        pproparray: windows_core::Ref<IPropertyChangeArray>,
    ) -> windows_core::Result<()>;
    fn SetOwnerWindow(&self, hwndowner: HWND) -> windows_core::Result<()>;
    fn ApplyPropertiesToItem(
        &self,
        psiitem: windows_core::Ref<IShellItem>,
    ) -> windows_core::Result<()>;
    fn ApplyPropertiesToItems(
        &self,
        punkitems: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
    fn RenameItem(
        &self,
        psiitem: windows_core::Ref<IShellItem>,
        psznewname: &windows_core::PCWSTR,
        pfopsitem: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<()>;
    fn RenameItems(
        &self,
        punkitems: windows_core::Ref<windows_core::IUnknown>,
        psznewname: &windows_core::PCWSTR,
    ) -> windows_core::Result<()>;
    fn MoveItem(
        &self,
        psiitem: windows_core::Ref<IShellItem>,
        psidestinationfolder: windows_core::Ref<IShellItem>,
        psznewname: &windows_core::PCWSTR,
        pfopsitem: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<()>;
    fn MoveItems(
        &self,
        punkitems: windows_core::Ref<windows_core::IUnknown>,
        psidestinationfolder: windows_core::Ref<IShellItem>,
    ) -> windows_core::Result<()>;
    fn CopyItem(
        &self,
        psiitem: windows_core::Ref<IShellItem>,
        psidestinationfolder: windows_core::Ref<IShellItem>,
        pszcopyname: &windows_core::PCWSTR,
        pfopsitem: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<()>;
    fn CopyItems(
        &self,
        punkitems: windows_core::Ref<windows_core::IUnknown>,
        psidestinationfolder: windows_core::Ref<IShellItem>,
    ) -> windows_core::Result<()>;
    fn DeleteItem(
        &self,
        psiitem: windows_core::Ref<IShellItem>,
        pfopsitem: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<()>;
    fn DeleteItems(
        &self,
        punkitems: windows_core::Ref<windows_core::IUnknown>,
    ) -> windows_core::Result<()>;
    fn NewItem(
        &self,
        psidestinationfolder: windows_core::Ref<IShellItem>,
        dwfileattributes: u32,
        pszname: &windows_core::PCWSTR,
        psztemplatename: &windows_core::PCWSTR,
        pfopsitem: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<()>;
    fn PerformOperations(&self) -> windows_core::Result<()>;
    fn GetAnyOperationsAborted(&self) -> windows_core::Result<windows_core::BOOL>;
}
impl IFileOperation_Vtbl {
    pub const fn new<Identity: IFileOperation_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Advise<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pfops: *mut core::ffi::c_void,
            pdwcookie: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileOperation_Impl::Advise(this, core::mem::transmute_copy(&pfops)) {
                    Ok(ok__) => {
                        pdwcookie.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Unadvise<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            dwcookie: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::Unadvise(this, core::mem::transmute_copy(&dwcookie)).into()
            }
        }
        unsafe extern "system" fn SetOperationFlags<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            dwoperationflags: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::SetOperationFlags(
                    this,
                    core::mem::transmute_copy(&dwoperationflags),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetProgressMessage<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszmessage: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::SetProgressMessage(this, core::mem::transmute(&pszmessage))
                    .into()
            }
        }
        unsafe extern "system" fn SetProgressDialog<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            popd: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::SetProgressDialog(this, core::mem::transmute_copy(&popd))
                    .into()
            }
        }
        unsafe extern "system" fn SetProperties<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pproparray: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::SetProperties(this, core::mem::transmute_copy(&pproparray))
                    .into()
            }
        }
        unsafe extern "system" fn SetOwnerWindow<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwndowner: HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::SetOwnerWindow(this, core::mem::transmute_copy(&hwndowner))
                    .into()
            }
        }
        unsafe extern "system" fn ApplyPropertiesToItem<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psiitem: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::ApplyPropertiesToItem(
                    this,
                    core::mem::transmute_copy(&psiitem),
                )
                .into()
            }
        }
        unsafe extern "system" fn ApplyPropertiesToItems<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            punkitems: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::ApplyPropertiesToItems(
                    this,
                    core::mem::transmute_copy(&punkitems),
                )
                .into()
            }
        }
        unsafe extern "system" fn RenameItem<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psiitem: *mut core::ffi::c_void,
            psznewname: windows_core::PCWSTR,
            pfopsitem: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::RenameItem(
                    this,
                    core::mem::transmute_copy(&psiitem),
                    core::mem::transmute(&psznewname),
                    core::mem::transmute_copy(&pfopsitem),
                )
                .into()
            }
        }
        unsafe extern "system" fn RenameItems<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            punkitems: *mut core::ffi::c_void,
            psznewname: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::RenameItems(
                    this,
                    core::mem::transmute_copy(&punkitems),
                    core::mem::transmute(&psznewname),
                )
                .into()
            }
        }
        unsafe extern "system" fn MoveItem<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psiitem: *mut core::ffi::c_void,
            psidestinationfolder: *mut core::ffi::c_void,
            psznewname: windows_core::PCWSTR,
            pfopsitem: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::MoveItem(
                    this,
                    core::mem::transmute_copy(&psiitem),
                    core::mem::transmute_copy(&psidestinationfolder),
                    core::mem::transmute(&psznewname),
                    core::mem::transmute_copy(&pfopsitem),
                )
                .into()
            }
        }
        unsafe extern "system" fn MoveItems<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            punkitems: *mut core::ffi::c_void,
            psidestinationfolder: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::MoveItems(
                    this,
                    core::mem::transmute_copy(&punkitems),
                    core::mem::transmute_copy(&psidestinationfolder),
                )
                .into()
            }
        }
        unsafe extern "system" fn CopyItem<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psiitem: *mut core::ffi::c_void,
            psidestinationfolder: *mut core::ffi::c_void,
            pszcopyname: windows_core::PCWSTR,
            pfopsitem: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::CopyItem(
                    this,
                    core::mem::transmute_copy(&psiitem),
                    core::mem::transmute_copy(&psidestinationfolder),
                    core::mem::transmute(&pszcopyname),
                    core::mem::transmute_copy(&pfopsitem),
                )
                .into()
            }
        }
        unsafe extern "system" fn CopyItems<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            punkitems: *mut core::ffi::c_void,
            psidestinationfolder: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::CopyItems(
                    this,
                    core::mem::transmute_copy(&punkitems),
                    core::mem::transmute_copy(&psidestinationfolder),
                )
                .into()
            }
        }
        unsafe extern "system" fn DeleteItem<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psiitem: *mut core::ffi::c_void,
            pfopsitem: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::DeleteItem(
                    this,
                    core::mem::transmute_copy(&psiitem),
                    core::mem::transmute_copy(&pfopsitem),
                )
                .into()
            }
        }
        unsafe extern "system" fn DeleteItems<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            punkitems: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::DeleteItems(this, core::mem::transmute_copy(&punkitems)).into()
            }
        }
        unsafe extern "system" fn NewItem<Identity: IFileOperation_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psidestinationfolder: *mut core::ffi::c_void,
            dwfileattributes: u32,
            pszname: windows_core::PCWSTR,
            psztemplatename: windows_core::PCWSTR,
            pfopsitem: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::NewItem(
                    this,
                    core::mem::transmute_copy(&psidestinationfolder),
                    core::mem::transmute_copy(&dwfileattributes),
                    core::mem::transmute(&pszname),
                    core::mem::transmute(&psztemplatename),
                    core::mem::transmute_copy(&pfopsitem),
                )
                .into()
            }
        }
        unsafe extern "system" fn PerformOperations<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileOperation_Impl::PerformOperations(this).into()
            }
        }
        unsafe extern "system" fn GetAnyOperationsAborted<
            Identity: IFileOperation_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pfanyoperationsaborted: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileOperation_Impl::GetAnyOperationsAborted(this) {
                    Ok(ok__) => {
                        pfanyoperationsaborted.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Advise: Advise::<Identity, OFFSET>,
            Unadvise: Unadvise::<Identity, OFFSET>,
            SetOperationFlags: SetOperationFlags::<Identity, OFFSET>,
            SetProgressMessage: SetProgressMessage::<Identity, OFFSET>,
            SetProgressDialog: SetProgressDialog::<Identity, OFFSET>,
            SetProperties: SetProperties::<Identity, OFFSET>,
            SetOwnerWindow: SetOwnerWindow::<Identity, OFFSET>,
            ApplyPropertiesToItem: ApplyPropertiesToItem::<Identity, OFFSET>,
            ApplyPropertiesToItems: ApplyPropertiesToItems::<Identity, OFFSET>,
            RenameItem: RenameItem::<Identity, OFFSET>,
            RenameItems: RenameItems::<Identity, OFFSET>,
            MoveItem: MoveItem::<Identity, OFFSET>,
            MoveItems: MoveItems::<Identity, OFFSET>,
            CopyItem: CopyItem::<Identity, OFFSET>,
            CopyItems: CopyItems::<Identity, OFFSET>,
            DeleteItem: DeleteItem::<Identity, OFFSET>,
            DeleteItems: DeleteItems::<Identity, OFFSET>,
            NewItem: NewItem::<Identity, OFFSET>,
            PerformOperations: PerformOperations::<Identity, OFFSET>,
            GetAnyOperationsAborted: GetAnyOperationsAborted::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IFileOperation as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IFileOperation {}
windows_core::imp::define_interface!(
    IFileOperationProgressSink,
    IFileOperationProgressSink_Vtbl,
    0x04b0f1a7_9490_44bc_96e1_4296a31252e2
);
windows_core::imp::interface_hierarchy!(IFileOperationProgressSink, windows_core::IUnknown);
#[repr(C)]
pub struct IFileOperationProgressSink_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    StartOperations: usize,
    FinishOperations: usize,
    PreRenameItem: usize,
    PostRenameItem: usize,
    PreMoveItem: usize,
    PostMoveItem: usize,
    PreCopyItem: usize,
    PostCopyItem: usize,
    PreDeleteItem: usize,
    PostDeleteItem: usize,
    PreNewItem: usize,
    PostNewItem: usize,
    UpdateProgress: usize,
    ResetTimer: usize,
    PauseTimer: usize,
    ResumeTimer: usize,
}
impl windows_core::RuntimeName for IFileOperationProgressSink {}
windows_core::imp::define_interface!(
    IFileSaveDialog,
    IFileSaveDialog_Vtbl,
    0x84bccd23_5fde_4cdb_aea4_af64b83d78ab
);
impl core::ops::Deref for IFileSaveDialog {
    type Target = IFileDialog;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IFileSaveDialog,
    windows_core::IUnknown,
    IModalWindow,
    IFileDialog
);
impl IFileSaveDialog {
    pub(crate) unsafe fn SetSaveAsItem<P0>(&self, psi: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetSaveAsItem)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetProperties<P0>(&self, pstore: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IPropertyStore>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetProperties)(
                windows_core::Interface::as_raw(self),
                pstore.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn SetCollectedProperties<P0>(
        &self,
        plist: P0,
        fappenddefault: bool,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IPropertyDescriptionList>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetCollectedProperties)(
                windows_core::Interface::as_raw(self),
                plist.param().abi(),
                fappenddefault.into(),
            )
        }
    }
    pub(crate) unsafe fn GetProperties(&self) -> windows_core::Result<IPropertyStore> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetProperties)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn ApplyProperties<P0, P1, P3>(
        &self,
        psi: P0,
        pstore: P1,
        hwnd: HWND,
        psink: P3,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IShellItem>,
        P1: windows_core::Param<IPropertyStore>,
        P3: windows_core::Param<IFileOperationProgressSink>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ApplyProperties)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
                pstore.param().abi(),
                hwnd,
                psink.param().abi(),
            )
        }
    }
}
#[repr(C)]
pub struct IFileSaveDialog_Vtbl {
    pub base__: IFileDialog_Vtbl,
    pub SetSaveAsItem: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub SetCollectedProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub GetProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub ApplyProperties: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        HWND,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IFileSaveDialog_Impl: IFileDialog_Impl {
    fn SetSaveAsItem(&self, psi: windows_core::Ref<IShellItem>) -> windows_core::Result<()>;
    fn SetProperties(&self, pstore: windows_core::Ref<IPropertyStore>) -> windows_core::Result<()>;
    fn SetCollectedProperties(
        &self,
        plist: windows_core::Ref<IPropertyDescriptionList>,
        fappenddefault: windows_core::BOOL,
    ) -> windows_core::Result<()>;
    fn GetProperties(&self) -> windows_core::Result<IPropertyStore>;
    fn ApplyProperties(
        &self,
        psi: windows_core::Ref<IShellItem>,
        pstore: windows_core::Ref<IPropertyStore>,
        hwnd: HWND,
        psink: windows_core::Ref<IFileOperationProgressSink>,
    ) -> windows_core::Result<()>;
}
impl IFileSaveDialog_Vtbl {
    pub const fn new<Identity: IFileSaveDialog_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn SetSaveAsItem<
            Identity: IFileSaveDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileSaveDialog_Impl::SetSaveAsItem(this, core::mem::transmute_copy(&psi)).into()
            }
        }
        unsafe extern "system" fn SetProperties<
            Identity: IFileSaveDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pstore: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileSaveDialog_Impl::SetProperties(this, core::mem::transmute_copy(&pstore)).into()
            }
        }
        unsafe extern "system" fn SetCollectedProperties<
            Identity: IFileSaveDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            plist: *mut core::ffi::c_void,
            fappenddefault: windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileSaveDialog_Impl::SetCollectedProperties(
                    this,
                    core::mem::transmute_copy(&plist),
                    core::mem::transmute_copy(&fappenddefault),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetProperties<
            Identity: IFileSaveDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppstore: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IFileSaveDialog_Impl::GetProperties(this) {
                    Ok(ok__) => {
                        ppstore.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn ApplyProperties<
            Identity: IFileSaveDialog_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
            pstore: *mut core::ffi::c_void,
            hwnd: HWND,
            psink: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IFileSaveDialog_Impl::ApplyProperties(
                    this,
                    core::mem::transmute_copy(&psi),
                    core::mem::transmute_copy(&pstore),
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&psink),
                )
                .into()
            }
        }
        Self {
            base__: IFileDialog_Vtbl::new::<Identity, OFFSET>(),
            SetSaveAsItem: SetSaveAsItem::<Identity, OFFSET>,
            SetProperties: SetProperties::<Identity, OFFSET>,
            SetCollectedProperties: SetCollectedProperties::<Identity, OFFSET>,
            GetProperties: GetProperties::<Identity, OFFSET>,
            ApplyProperties: ApplyProperties::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IFileSaveDialog as windows_core::Interface>::IID
            || iid == &<IModalWindow as windows_core::Interface>::IID
            || iid == &<IFileDialog as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IFileSaveDialog {}
pub const IMAGE_FILE_MACHINE_AMD64: i32 = 34404;
windows_core::imp::define_interface!(
    IModalWindow,
    IModalWindow_Vtbl,
    0xb4db1657_70d7_485e_8e3e_6fcb5a5c1802
);
windows_core::imp::interface_hierarchy!(IModalWindow, windows_core::IUnknown);
impl IModalWindow {
    pub(crate) unsafe fn Show(&self, hwndowner: Option<HWND>) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Show)(
                windows_core::Interface::as_raw(self),
                hwndowner.unwrap_or(core::mem::zeroed()) as _,
            )
        }
    }
}
#[repr(C)]
pub struct IModalWindow_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub Show: unsafe extern "system" fn(*mut core::ffi::c_void, HWND) -> windows_core::HRESULT,
}
pub trait IModalWindow_Impl: windows_core::IUnknownImpl {
    fn Show(&self, hwndowner: HWND) -> windows_core::Result<()>;
}
impl IModalWindow_Vtbl {
    pub const fn new<Identity: IModalWindow_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Show<Identity: IModalWindow_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            hwndowner: HWND,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IModalWindow_Impl::Show(this, core::mem::transmute_copy(&hwndowner)).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            Show: Show::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IModalWindow as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IModalWindow {}
pub const INFINITE: u32 = 4294967295;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: u32,
    pub dwICC: u32,
}
pub const INVALID_FILE_ATTRIBUTES: u32 = 4294967295;
pub const INVALID_HANDLE_VALUE: HANDLE = HANDLE(-1 as _);
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IO_COUNTERS {
    pub ReadOperationCount: u64,
    pub WriteOperationCount: u64,
    pub OtherOperationCount: u64,
    pub ReadTransferCount: u64,
    pub WriteTransferCount: u64,
    pub OtherTransferCount: u64,
}
windows_core::imp::define_interface!(
    IOperationsProgressDialog,
    IOperationsProgressDialog_Vtbl,
    0x0c9fb851_e5c9_43eb_a370_f0677b13874c
);
windows_core::imp::interface_hierarchy!(IOperationsProgressDialog, windows_core::IUnknown);
#[repr(C)]
pub struct IOperationsProgressDialog_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    StartProgressDialog: usize,
    StopProgressDialog: usize,
    SetOperation: usize,
    SetMode: usize,
    UpdateProgress: usize,
    UpdateLocations: usize,
    ResetTimer: usize,
    PauseTimer: usize,
    ResumeTimer: usize,
    GetMilliseconds: usize,
    GetOperationStatus: usize,
}
impl windows_core::RuntimeName for IOperationsProgressDialog {}
windows_core::imp::define_interface!(
    IPersist,
    IPersist_Vtbl,
    0x0000010c_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IPersist, windows_core::IUnknown);
#[repr(C)]
pub struct IPersist_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetClassID: usize,
}
impl windows_core::RuntimeName for IPersist {}
windows_core::imp::define_interface!(
    IPersistFile,
    IPersistFile_Vtbl,
    0x0000010b_0000_0000_c000_000000000046
);
impl core::ops::Deref for IPersistFile {
    type Target = IPersist;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IPersistFile, windows_core::IUnknown, IPersist);
impl IPersistFile {
    pub(crate) unsafe fn IsDirty(&self) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).IsDirty)(windows_core::Interface::as_raw(self))
        }
    }
    pub(crate) unsafe fn Load<P0>(&self, pszfilename: P0, dwmode: u32) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Load)(
                windows_core::Interface::as_raw(self),
                pszfilename.param().abi(),
                dwmode,
            )
        }
    }
    pub(crate) unsafe fn Save<P0>(&self, pszfilename: P0, fremember: bool) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Save)(
                windows_core::Interface::as_raw(self),
                pszfilename.param().abi(),
                fremember.into(),
            )
        }
    }
    pub(crate) unsafe fn SaveCompleted<P0>(&self, pszfilename: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SaveCompleted)(
                windows_core::Interface::as_raw(self),
                pszfilename.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetCurFile(&self) -> windows_core::Result<windows_core::PWSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCurFile)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IPersistFile_Vtbl {
    pub base__: IPersist_Vtbl,
    pub IsDirty: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    pub Load: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        u32,
    ) -> windows_core::HRESULT,
    pub Save: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        windows_core::BOOL,
    ) -> windows_core::HRESULT,
    pub SaveCompleted: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetCurFile: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::PWSTR,
    ) -> windows_core::HRESULT,
}
impl windows_core::RuntimeName for IPersistFile {}
windows_core::imp::define_interface!(
    IPropertyChangeArray,
    IPropertyChangeArray_Vtbl,
    0x380f5cad_1b5e_42f2_805d_637fd392d31e
);
windows_core::imp::interface_hierarchy!(IPropertyChangeArray, windows_core::IUnknown);
#[repr(C)]
pub struct IPropertyChangeArray_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetCount: usize,
    GetAt: usize,
    InsertAt: usize,
    Append: usize,
    AppendOrReplace: usize,
    RemoveAt: usize,
    IsKeyInArray: usize,
}
impl windows_core::RuntimeName for IPropertyChangeArray {}
windows_core::imp::define_interface!(
    IPropertyDescriptionList,
    IPropertyDescriptionList_Vtbl,
    0x1f9fc1d0_c39b_4b26_817f_011967d3440e
);
windows_core::imp::interface_hierarchy!(IPropertyDescriptionList, windows_core::IUnknown);
#[repr(C)]
pub struct IPropertyDescriptionList_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetCount: usize,
    GetAt: usize,
}
impl windows_core::RuntimeName for IPropertyDescriptionList {}
windows_core::imp::define_interface!(
    IPropertyStore,
    IPropertyStore_Vtbl,
    0x886d8eeb_8cf2_4446_8d02_cdba1dbdcf99
);
windows_core::imp::interface_hierarchy!(IPropertyStore, windows_core::IUnknown);
#[repr(C)]
pub struct IPropertyStore_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetCount: usize,
    GetAt: usize,
    GetValue: usize,
    SetValue: usize,
    Commit: usize,
}
impl windows_core::RuntimeName for IPropertyStore {}
windows_core::imp::define_interface!(
    ISequentialStream,
    ISequentialStream_Vtbl,
    0x0c733a30_2a1c_11ce_ade5_00aa0044773d
);
windows_core::imp::interface_hierarchy!(ISequentialStream, windows_core::IUnknown);
#[repr(C)]
pub struct ISequentialStream_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Read: usize,
    Write: usize,
}
impl windows_core::RuntimeName for ISequentialStream {}
windows_core::imp::define_interface!(
    IShellFolder,
    IShellFolder_Vtbl,
    0x000214e6_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IShellFolder, windows_core::IUnknown);
impl IShellFolder {
    pub(crate) unsafe fn ParseDisplayName<P1, P2>(
        &self,
        hwnd: HWND,
        pbc: P1,
        pszdisplayname: P2,
        pcheaten: Option<*const u32>,
        ppidl: *mut LPITEMIDLIST,
        pdwattributes: *mut u32,
    ) -> windows_core::HRESULT
    where
        P1: windows_core::Param<IBindCtx>,
        P2: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).ParseDisplayName)(
                windows_core::Interface::as_raw(self),
                hwnd,
                pbc.param().abi(),
                pszdisplayname.param().abi(),
                pcheaten.unwrap_or(core::mem::zeroed()) as _,
                ppidl as _,
                pdwattributes as _,
            )
        }
    }
    pub(crate) unsafe fn EnumObjects(
        &self,
        hwnd: HWND,
        grfflags: SHCONTF,
    ) -> windows_core::Result<IEnumIDList> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).EnumObjects)(
                windows_core::Interface::as_raw(self),
                hwnd,
                grfflags,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn BindToObject<P1, T>(
        &self,
        pidl: *const ITEMIDLIST,
        pbc: P1,
    ) -> windows_core::Result<T>
    where
        P1: windows_core::Param<IBindCtx>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).BindToObject)(
                windows_core::Interface::as_raw(self),
                pidl,
                pbc.param().abi(),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn BindToStorage<P1, T>(
        &self,
        pidl: *const ITEMIDLIST,
        pbc: P1,
    ) -> windows_core::Result<T>
    where
        P1: windows_core::Param<IBindCtx>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).BindToStorage)(
                windows_core::Interface::as_raw(self),
                pidl,
                pbc.param().abi(),
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CompareIDs(
        &self,
        lparam: LPARAM,
        pidl1: *const ITEMIDLIST,
        pidl2: *const ITEMIDLIST,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).CompareIDs)(
                windows_core::Interface::as_raw(self),
                lparam,
                pidl1,
                pidl2,
            )
        }
    }
    pub(crate) unsafe fn CreateViewObject<T>(&self, hwndowner: HWND) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).CreateViewObject)(
                windows_core::Interface::as_raw(self),
                hwndowner,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetAttributesOf(
        &self,
        cidl: u32,
        apidl: *const LPCITEMIDLIST,
        rgfinout: *mut SFGAOF,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetAttributesOf)(
                windows_core::Interface::as_raw(self),
                cidl,
                apidl,
                rgfinout as _,
            )
        }
    }
    pub(crate) unsafe fn GetUIObjectOf<T>(
        &self,
        hwndowner: HWND,
        cidl: u32,
        apidl: *const LPCITEMIDLIST,
        rgfreserved: Option<*const u32>,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetUIObjectOf)(
                windows_core::Interface::as_raw(self),
                hwndowner,
                cidl,
                apidl,
                &T::IID,
                rgfreserved.unwrap_or(core::mem::zeroed()) as _,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetDisplayNameOf(
        &self,
        pidl: *const ITEMIDLIST,
        uflags: SHGDNF,
        pname: *mut STRRET,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDisplayNameOf)(
                windows_core::Interface::as_raw(self),
                pidl,
                uflags,
                pname as _,
            )
        }
    }
    pub(crate) unsafe fn SetNameOf<P2>(
        &self,
        hwnd: Option<HWND>,
        pidl: *const ITEMIDLIST,
        pszname: P2,
        uflags: SHGDNF,
    ) -> windows_core::Result<LPITEMIDLIST>
    where
        P2: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).SetNameOf)(
                windows_core::Interface::as_raw(self),
                hwnd.unwrap_or(core::mem::zeroed()) as _,
                pidl,
                pszname.param().abi(),
                uflags,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IShellFolder_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub ParseDisplayName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *const u32,
        *mut LPITEMIDLIST,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub EnumObjects: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        SHCONTF,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub BindToObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const ITEMIDLIST,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub BindToStorage: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const ITEMIDLIST,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CompareIDs: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        LPARAM,
        *const ITEMIDLIST,
        *const ITEMIDLIST,
    ) -> windows_core::HRESULT,
    pub CreateViewObject: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetAttributesOf: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *const LPCITEMIDLIST,
        *mut SFGAOF,
    ) -> windows_core::HRESULT,
    pub GetUIObjectOf: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        u32,
        *const LPCITEMIDLIST,
        *const windows_core::GUID,
        *const u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetDisplayNameOf: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const ITEMIDLIST,
        SHGDNF,
        *mut STRRET,
    ) -> windows_core::HRESULT,
    pub SetNameOf: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HWND,
        *const ITEMIDLIST,
        windows_core::PCWSTR,
        SHGDNF,
        *mut LPITEMIDLIST,
    ) -> windows_core::HRESULT,
}
pub trait IShellFolder_Impl: windows_core::IUnknownImpl {
    fn ParseDisplayName(
        &self,
        hwnd: HWND,
        pbc: windows_core::Ref<IBindCtx>,
        pszdisplayname: &windows_core::PCWSTR,
        pcheaten: *const u32,
        ppidl: *mut LPITEMIDLIST,
        pdwattributes: *mut u32,
    ) -> windows_core::Result<()>;
    fn EnumObjects(&self, hwnd: HWND, grfflags: SHCONTF) -> windows_core::Result<IEnumIDList>;
    fn BindToObject(
        &self,
        pidl: *const ITEMIDLIST,
        pbc: windows_core::Ref<IBindCtx>,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn BindToStorage(
        &self,
        pidl: *const ITEMIDLIST,
        pbc: windows_core::Ref<IBindCtx>,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn CompareIDs(
        &self,
        lparam: LPARAM,
        pidl1: *const ITEMIDLIST,
        pidl2: *const ITEMIDLIST,
    ) -> windows_core::Result<()>;
    fn CreateViewObject(
        &self,
        hwndowner: HWND,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetAttributesOf(
        &self,
        cidl: u32,
        apidl: *const LPCITEMIDLIST,
        rgfinout: *mut SFGAOF,
    ) -> windows_core::Result<()>;
    fn GetUIObjectOf(
        &self,
        hwndowner: HWND,
        cidl: u32,
        apidl: *const LPCITEMIDLIST,
        riid: *const windows_core::GUID,
        rgfreserved: *const u32,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetDisplayNameOf(
        &self,
        pidl: *const ITEMIDLIST,
        uflags: SHGDNF,
        pname: *mut STRRET,
    ) -> windows_core::Result<()>;
    fn SetNameOf(
        &self,
        hwnd: HWND,
        pidl: *const ITEMIDLIST,
        pszname: &windows_core::PCWSTR,
        uflags: SHGDNF,
    ) -> windows_core::Result<LPITEMIDLIST>;
}
impl IShellFolder_Vtbl {
    pub const fn new<Identity: IShellFolder_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn ParseDisplayName<
            Identity: IShellFolder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwnd: HWND,
            pbc: *mut core::ffi::c_void,
            pszdisplayname: windows_core::PCWSTR,
            pcheaten: *const u32,
            ppidl: *mut LPITEMIDLIST,
            pdwattributes: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::ParseDisplayName(
                    this,
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&pbc),
                    core::mem::transmute(&pszdisplayname),
                    core::mem::transmute_copy(&pcheaten),
                    core::mem::transmute_copy(&ppidl),
                    core::mem::transmute_copy(&pdwattributes),
                )
                .into()
            }
        }
        unsafe extern "system" fn EnumObjects<Identity: IShellFolder_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            hwnd: HWND,
            grfflags: SHCONTF,
            ppenumidlist: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellFolder_Impl::EnumObjects(
                    this,
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&grfflags),
                ) {
                    Ok(ok__) => {
                        ppenumidlist.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn BindToObject<Identity: IShellFolder_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pidl: *const ITEMIDLIST,
            pbc: *mut core::ffi::c_void,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::BindToObject(
                    this,
                    core::mem::transmute_copy(&pidl),
                    core::mem::transmute_copy(&pbc),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn BindToStorage<
            Identity: IShellFolder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pidl: *const ITEMIDLIST,
            pbc: *mut core::ffi::c_void,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::BindToStorage(
                    this,
                    core::mem::transmute_copy(&pidl),
                    core::mem::transmute_copy(&pbc),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn CompareIDs<Identity: IShellFolder_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            lparam: LPARAM,
            pidl1: *const ITEMIDLIST,
            pidl2: *const ITEMIDLIST,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::CompareIDs(
                    this,
                    core::mem::transmute_copy(&lparam),
                    core::mem::transmute_copy(&pidl1),
                    core::mem::transmute_copy(&pidl2),
                )
                .into()
            }
        }
        unsafe extern "system" fn CreateViewObject<
            Identity: IShellFolder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwndowner: HWND,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::CreateViewObject(
                    this,
                    core::mem::transmute_copy(&hwndowner),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetAttributesOf<
            Identity: IShellFolder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            cidl: u32,
            apidl: *const LPCITEMIDLIST,
            rgfinout: *mut SFGAOF,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::GetAttributesOf(
                    this,
                    core::mem::transmute_copy(&cidl),
                    core::mem::transmute_copy(&apidl),
                    core::mem::transmute_copy(&rgfinout),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetUIObjectOf<
            Identity: IShellFolder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hwndowner: HWND,
            cidl: u32,
            apidl: *const LPCITEMIDLIST,
            riid: *const windows_core::GUID,
            rgfreserved: *const u32,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::GetUIObjectOf(
                    this,
                    core::mem::transmute_copy(&hwndowner),
                    core::mem::transmute_copy(&cidl),
                    core::mem::transmute_copy(&apidl),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&rgfreserved),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetDisplayNameOf<
            Identity: IShellFolder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pidl: *const ITEMIDLIST,
            uflags: SHGDNF,
            pname: *mut STRRET,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellFolder_Impl::GetDisplayNameOf(
                    this,
                    core::mem::transmute_copy(&pidl),
                    core::mem::transmute_copy(&uflags),
                    core::mem::transmute_copy(&pname),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetNameOf<Identity: IShellFolder_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            hwnd: HWND,
            pidl: *const ITEMIDLIST,
            pszname: windows_core::PCWSTR,
            uflags: SHGDNF,
            ppidlout: *mut LPITEMIDLIST,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellFolder_Impl::SetNameOf(
                    this,
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&pidl),
                    core::mem::transmute(&pszname),
                    core::mem::transmute_copy(&uflags),
                ) {
                    Ok(ok__) => {
                        ppidlout.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            ParseDisplayName: ParseDisplayName::<Identity, OFFSET>,
            EnumObjects: EnumObjects::<Identity, OFFSET>,
            BindToObject: BindToObject::<Identity, OFFSET>,
            BindToStorage: BindToStorage::<Identity, OFFSET>,
            CompareIDs: CompareIDs::<Identity, OFFSET>,
            CreateViewObject: CreateViewObject::<Identity, OFFSET>,
            GetAttributesOf: GetAttributesOf::<Identity, OFFSET>,
            GetUIObjectOf: GetUIObjectOf::<Identity, OFFSET>,
            GetDisplayNameOf: GetDisplayNameOf::<Identity, OFFSET>,
            SetNameOf: SetNameOf::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellFolder as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellFolder {}
windows_core::imp::define_interface!(
    IShellItem,
    IShellItem_Vtbl,
    0x43826d1e_e718_42ee_bc55_a1e261c37bfe
);
windows_core::imp::interface_hierarchy!(IShellItem, windows_core::IUnknown);
impl IShellItem {
    pub(crate) unsafe fn BindToHandler<P0, T>(
        &self,
        pbc: P0,
        bhid: *const windows_core::GUID,
    ) -> windows_core::Result<T>
    where
        P0: windows_core::Param<IBindCtx>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).BindToHandler)(
                windows_core::Interface::as_raw(self),
                pbc.param().abi(),
                bhid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetParent(&self) -> windows_core::Result<Self> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetParent)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetDisplayName(
        &self,
        sigdnname: SIGDN,
    ) -> windows_core::Result<windows_core::PWSTR> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDisplayName)(
                windows_core::Interface::as_raw(self),
                sigdnname,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetAttributes(&self, sfgaomask: SFGAOF) -> windows_core::Result<SFGAOF> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAttributes)(
                windows_core::Interface::as_raw(self),
                sfgaomask,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Compare<P0>(&self, psi: P0, hint: SICHINTF) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<Self>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Compare)(
                windows_core::Interface::as_raw(self),
                psi.param().abi(),
                hint,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IShellItem_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub BindToHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetParent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetDisplayName: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SIGDN,
        *mut windows_core::PWSTR,
    ) -> windows_core::HRESULT,
    pub GetAttributes: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SFGAOF,
        *mut SFGAOF,
    ) -> windows_core::HRESULT,
    pub Compare: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        SICHINTF,
        *mut i32,
    ) -> windows_core::HRESULT,
}
pub trait IShellItem_Impl: windows_core::IUnknownImpl {
    fn BindToHandler(
        &self,
        pbc: windows_core::Ref<IBindCtx>,
        bhid: *const windows_core::GUID,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetParent(&self) -> windows_core::Result<IShellItem>;
    fn GetDisplayName(&self, sigdnname: SIGDN) -> windows_core::Result<windows_core::PWSTR>;
    fn GetAttributes(&self, sfgaomask: SFGAOF) -> windows_core::Result<SFGAOF>;
    fn Compare(
        &self,
        psi: windows_core::Ref<IShellItem>,
        hint: SICHINTF,
    ) -> windows_core::Result<i32>;
}
impl IShellItem_Vtbl {
    pub const fn new<Identity: IShellItem_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn BindToHandler<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pbc: *mut core::ffi::c_void,
            bhid: *const windows_core::GUID,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellItem_Impl::BindToHandler(
                    this,
                    core::mem::transmute_copy(&pbc),
                    core::mem::transmute_copy(&bhid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetParent<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppsi: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::GetParent(this) {
                    Ok(ok__) => {
                        ppsi.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetDisplayName<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            sigdnname: SIGDN,
            ppszname: *mut windows_core::PWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::GetDisplayName(this, core::mem::transmute_copy(&sigdnname)) {
                    Ok(ok__) => {
                        ppszname.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetAttributes<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            sfgaomask: SFGAOF,
            psfgaoattribs: *mut SFGAOF,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::GetAttributes(this, core::mem::transmute_copy(&sfgaomask)) {
                    Ok(ok__) => {
                        psfgaoattribs.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Compare<Identity: IShellItem_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            psi: *mut core::ffi::c_void,
            hint: SICHINTF,
            piorder: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItem_Impl::Compare(
                    this,
                    core::mem::transmute_copy(&psi),
                    core::mem::transmute_copy(&hint),
                ) {
                    Ok(ok__) => {
                        piorder.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            BindToHandler: BindToHandler::<Identity, OFFSET>,
            GetParent: GetParent::<Identity, OFFSET>,
            GetDisplayName: GetDisplayName::<Identity, OFFSET>,
            GetAttributes: GetAttributes::<Identity, OFFSET>,
            Compare: Compare::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellItem as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellItem {}
windows_core::imp::define_interface!(
    IShellItemArray,
    IShellItemArray_Vtbl,
    0xb63ea76d_1f85_456f_a19c_48159efa858b
);
windows_core::imp::interface_hierarchy!(IShellItemArray, windows_core::IUnknown);
impl IShellItemArray {
    pub(crate) unsafe fn BindToHandler<P0, T>(
        &self,
        pbc: P0,
        bhid: *const windows_core::GUID,
    ) -> windows_core::Result<T>
    where
        P0: windows_core::Param<IBindCtx>,
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).BindToHandler)(
                windows_core::Interface::as_raw(self),
                pbc.param().abi(),
                bhid,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetPropertyStore<T>(
        &self,
        flags: GETPROPERTYSTOREFLAGS,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetPropertyStore)(
                windows_core::Interface::as_raw(self),
                flags,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetPropertyDescriptionList<T>(
        &self,
        keytype: *const PROPERTYKEY,
    ) -> windows_core::Result<T>
    where
        T: windows_core::Interface,
    {
        let mut result__ = core::ptr::null_mut();
        unsafe {
            (windows_core::Interface::vtable(self).GetPropertyDescriptionList)(
                windows_core::Interface::as_raw(self),
                keytype,
                &T::IID,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetAttributes(
        &self,
        attribflags: SIATTRIBFLAGS,
        sfgaomask: SFGAOF,
    ) -> windows_core::Result<SFGAOF> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetAttributes)(
                windows_core::Interface::as_raw(self),
                attribflags,
                sfgaomask,
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetCount(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetItemAt(&self, dwindex: u32) -> windows_core::Result<IShellItem> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetItemAt)(
                windows_core::Interface::as_raw(self),
                dwindex,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn EnumItems(&self) -> windows_core::Result<IEnumShellItems> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).EnumItems)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IShellItemArray_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub BindToHandler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPropertyStore: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        GETPROPERTYSTOREFLAGS,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPropertyDescriptionList: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const PROPERTYKEY,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetAttributes: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SIATTRIBFLAGS,
        SFGAOF,
        *mut SFGAOF,
    ) -> windows_core::HRESULT,
    pub GetCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub GetItemAt: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub EnumItems: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IShellItemArray_Impl: windows_core::IUnknownImpl {
    fn BindToHandler(
        &self,
        pbc: windows_core::Ref<IBindCtx>,
        bhid: *const windows_core::GUID,
        riid: *const windows_core::GUID,
        ppvout: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetPropertyStore(
        &self,
        flags: GETPROPERTYSTOREFLAGS,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetPropertyDescriptionList(
        &self,
        keytype: *const PROPERTYKEY,
        riid: *const windows_core::GUID,
        ppv: *mut *mut core::ffi::c_void,
    ) -> windows_core::Result<()>;
    fn GetAttributes(
        &self,
        attribflags: SIATTRIBFLAGS,
        sfgaomask: SFGAOF,
    ) -> windows_core::Result<SFGAOF>;
    fn GetCount(&self) -> windows_core::Result<u32>;
    fn GetItemAt(&self, dwindex: u32) -> windows_core::Result<IShellItem>;
    fn EnumItems(&self) -> windows_core::Result<IEnumShellItems>;
}
impl IShellItemArray_Vtbl {
    pub const fn new<Identity: IShellItemArray_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn BindToHandler<
            Identity: IShellItemArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pbc: *mut core::ffi::c_void,
            bhid: *const windows_core::GUID,
            riid: *const windows_core::GUID,
            ppvout: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellItemArray_Impl::BindToHandler(
                    this,
                    core::mem::transmute_copy(&pbc),
                    core::mem::transmute_copy(&bhid),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppvout),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetPropertyStore<
            Identity: IShellItemArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            flags: GETPROPERTYSTOREFLAGS,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellItemArray_Impl::GetPropertyStore(
                    this,
                    core::mem::transmute_copy(&flags),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetPropertyDescriptionList<
            Identity: IShellItemArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            keytype: *const PROPERTYKEY,
            riid: *const windows_core::GUID,
            ppv: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellItemArray_Impl::GetPropertyDescriptionList(
                    this,
                    core::mem::transmute_copy(&keytype),
                    core::mem::transmute_copy(&riid),
                    core::mem::transmute_copy(&ppv),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetAttributes<
            Identity: IShellItemArray_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            attribflags: SIATTRIBFLAGS,
            sfgaomask: SFGAOF,
            psfgaoattribs: *mut SFGAOF,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItemArray_Impl::GetAttributes(
                    this,
                    core::mem::transmute_copy(&attribflags),
                    core::mem::transmute_copy(&sfgaomask),
                ) {
                    Ok(ok__) => {
                        psfgaoattribs.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetCount<Identity: IShellItemArray_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pdwnumitems: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItemArray_Impl::GetCount(this) {
                    Ok(ok__) => {
                        pdwnumitems.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetItemAt<Identity: IShellItemArray_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            dwindex: u32,
            ppsi: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItemArray_Impl::GetItemAt(this, core::mem::transmute_copy(&dwindex)) {
                    Ok(ok__) => {
                        ppsi.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn EnumItems<Identity: IShellItemArray_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppenumshellitems: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItemArray_Impl::EnumItems(this) {
                    Ok(ok__) => {
                        ppenumshellitems.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            BindToHandler: BindToHandler::<Identity, OFFSET>,
            GetPropertyStore: GetPropertyStore::<Identity, OFFSET>,
            GetPropertyDescriptionList: GetPropertyDescriptionList::<Identity, OFFSET>,
            GetAttributes: GetAttributes::<Identity, OFFSET>,
            GetCount: GetCount::<Identity, OFFSET>,
            GetItemAt: GetItemAt::<Identity, OFFSET>,
            EnumItems: EnumItems::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellItemArray as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellItemArray {}
windows_core::imp::define_interface!(
    IShellItemFilter,
    IShellItemFilter_Vtbl,
    0x2659b475_eeb8_48b7_8f07_b378810f48cf
);
windows_core::imp::interface_hierarchy!(IShellItemFilter, windows_core::IUnknown);
#[repr(C)]
pub struct IShellItemFilter_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    IncludeItem: usize,
    GetEnumFlagsForItem: usize,
}
impl windows_core::RuntimeName for IShellItemFilter {}
windows_core::imp::define_interface!(
    IShellItemImageFactory,
    IShellItemImageFactory_Vtbl,
    0xbcc18b79_ba16_442f_80c4_8a59c30c463b
);
windows_core::imp::interface_hierarchy!(IShellItemImageFactory, windows_core::IUnknown);
impl IShellItemImageFactory {
    pub(crate) unsafe fn GetImage(
        &self,
        size: SIZE,
        flags: SIIGBF,
    ) -> windows_core::Result<HBITMAP> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetImage)(
                windows_core::Interface::as_raw(self),
                size,
                flags,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IShellItemImageFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetImage: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        SIZE,
        SIIGBF,
        *mut HBITMAP,
    ) -> windows_core::HRESULT,
}
pub trait IShellItemImageFactory_Impl: windows_core::IUnknownImpl {
    fn GetImage(&self, size: &SIZE, flags: SIIGBF) -> windows_core::Result<HBITMAP>;
}
impl IShellItemImageFactory_Vtbl {
    pub const fn new<Identity: IShellItemImageFactory_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetImage<
            Identity: IShellItemImageFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            size: SIZE,
            flags: SIIGBF,
            phbm: *mut HBITMAP,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellItemImageFactory_Impl::GetImage(
                    this,
                    core::mem::transmute(&size),
                    core::mem::transmute_copy(&flags),
                ) {
                    Ok(ok__) => {
                        phbm.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetImage: GetImage::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellItemImageFactory as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellItemImageFactory {}
windows_core::imp::define_interface!(
    IShellLinkW,
    IShellLinkW_Vtbl,
    0x000214f9_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IShellLinkW, windows_core::IUnknown);
impl IShellLinkW {
    pub(crate) unsafe fn GetPath(
        &self,
        pszfile: windows_core::PWSTR,
        cch: i32,
        pfd: *mut WIN32_FIND_DATAW,
        fflags: u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetPath)(
                windows_core::Interface::as_raw(self),
                pszfile,
                cch,
                pfd as _,
                fflags,
            )
        }
    }
    pub(crate) unsafe fn GetIDList(&self) -> windows_core::Result<LPITEMIDLIST> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetIDList)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetIDList(&self, pidl: *const ITEMIDLIST) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetIDList)(
                windows_core::Interface::as_raw(self),
                pidl,
            )
        }
    }
    pub(crate) unsafe fn GetDescription(
        &self,
        pszname: windows_core::PWSTR,
        cch: i32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetDescription)(
                windows_core::Interface::as_raw(self),
                pszname,
                cch,
            )
        }
    }
    pub(crate) unsafe fn SetDescription<P0>(&self, pszname: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetDescription)(
                windows_core::Interface::as_raw(self),
                pszname.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetWorkingDirectory(
        &self,
        pszdir: windows_core::PWSTR,
        cch: i32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetWorkingDirectory)(
                windows_core::Interface::as_raw(self),
                pszdir,
                cch,
            )
        }
    }
    pub(crate) unsafe fn SetWorkingDirectory<P0>(&self, pszdir: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetWorkingDirectory)(
                windows_core::Interface::as_raw(self),
                pszdir.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetArguments(
        &self,
        pszargs: windows_core::PWSTR,
        cch: i32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetArguments)(
                windows_core::Interface::as_raw(self),
                pszargs,
                cch,
            )
        }
    }
    pub(crate) unsafe fn SetArguments<P0>(&self, pszargs: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetArguments)(
                windows_core::Interface::as_raw(self),
                pszargs.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetHotkey(&self) -> windows_core::Result<u16> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetHotkey)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetHotkey(&self, whotkey: u16) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetHotkey)(
                windows_core::Interface::as_raw(self),
                whotkey,
            )
        }
    }
    pub(crate) unsafe fn GetShowCmd(&self) -> windows_core::Result<i32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetShowCmd)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn SetShowCmd(&self, ishowcmd: i32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).SetShowCmd)(
                windows_core::Interface::as_raw(self),
                ishowcmd,
            )
        }
    }
    pub(crate) unsafe fn GetIconLocation(
        &self,
        psziconpath: windows_core::PWSTR,
        cch: i32,
        piicon: *mut i32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetIconLocation)(
                windows_core::Interface::as_raw(self),
                psziconpath,
                cch,
                piicon as _,
            )
        }
    }
    pub(crate) unsafe fn SetIconLocation<P0>(
        &self,
        psziconpath: P0,
        iicon: i32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetIconLocation)(
                windows_core::Interface::as_raw(self),
                psziconpath.param().abi(),
                iicon,
            )
        }
    }
    pub(crate) unsafe fn SetRelativePath<P0>(
        &self,
        pszpathrel: P0,
        dwreserved: u32,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetRelativePath)(
                windows_core::Interface::as_raw(self),
                pszpathrel.param().abi(),
                dwreserved,
            )
        }
    }
    pub(crate) unsafe fn Resolve(&self, hwnd: HWND, fflags: u32) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).Resolve)(
                windows_core::Interface::as_raw(self),
                hwnd,
                fflags,
            )
        }
    }
    pub(crate) unsafe fn SetPath<P0>(&self, pszfile: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).SetPath)(
                windows_core::Interface::as_raw(self),
                pszfile.param().abi(),
            )
        }
    }
}
#[repr(C)]
pub struct IShellLinkW_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetPath: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PWSTR,
        i32,
        *mut WIN32_FIND_DATAW,
        u32,
    ) -> windows_core::HRESULT,
    pub GetIDList: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut LPITEMIDLIST,
    ) -> windows_core::HRESULT,
    pub SetIDList: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const ITEMIDLIST,
    ) -> windows_core::HRESULT,
    pub GetDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PWSTR,
        i32,
    ) -> windows_core::HRESULT,
    pub SetDescription: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetWorkingDirectory: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PWSTR,
        i32,
    ) -> windows_core::HRESULT,
    pub SetWorkingDirectory: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetArguments: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PWSTR,
        i32,
    ) -> windows_core::HRESULT,
    pub SetArguments: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
    pub GetHotkey:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u16) -> windows_core::HRESULT,
    pub SetHotkey: unsafe extern "system" fn(*mut core::ffi::c_void, u16) -> windows_core::HRESULT,
    pub GetShowCmd:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut i32) -> windows_core::HRESULT,
    pub SetShowCmd: unsafe extern "system" fn(*mut core::ffi::c_void, i32) -> windows_core::HRESULT,
    pub GetIconLocation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PWSTR,
        i32,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub SetIconLocation: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        i32,
    ) -> windows_core::HRESULT,
    pub SetRelativePath: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        u32,
    ) -> windows_core::HRESULT,
    pub Resolve:
        unsafe extern "system" fn(*mut core::ffi::c_void, HWND, u32) -> windows_core::HRESULT,
    pub SetPath: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
    ) -> windows_core::HRESULT,
}
pub trait IShellLinkW_Impl: windows_core::IUnknownImpl {
    fn GetPath(
        &self,
        pszfile: windows_core::PWSTR,
        cch: i32,
        pfd: *mut WIN32_FIND_DATAW,
        fflags: u32,
    ) -> windows_core::Result<()>;
    fn GetIDList(&self) -> windows_core::Result<LPITEMIDLIST>;
    fn SetIDList(&self, pidl: *const ITEMIDLIST) -> windows_core::Result<()>;
    fn GetDescription(&self, pszname: windows_core::PWSTR, cch: i32) -> windows_core::Result<()>;
    fn SetDescription(&self, pszname: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn GetWorkingDirectory(
        &self,
        pszdir: windows_core::PWSTR,
        cch: i32,
    ) -> windows_core::Result<()>;
    fn SetWorkingDirectory(&self, pszdir: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn GetArguments(&self, pszargs: windows_core::PWSTR, cch: i32) -> windows_core::Result<()>;
    fn SetArguments(&self, pszargs: &windows_core::PCWSTR) -> windows_core::Result<()>;
    fn GetHotkey(&self) -> windows_core::Result<u16>;
    fn SetHotkey(&self, whotkey: u16) -> windows_core::Result<()>;
    fn GetShowCmd(&self) -> windows_core::Result<i32>;
    fn SetShowCmd(&self, ishowcmd: i32) -> windows_core::Result<()>;
    fn GetIconLocation(
        &self,
        psziconpath: windows_core::PWSTR,
        cch: i32,
        piicon: *mut i32,
    ) -> windows_core::Result<()>;
    fn SetIconLocation(
        &self,
        psziconpath: &windows_core::PCWSTR,
        iicon: i32,
    ) -> windows_core::Result<()>;
    fn SetRelativePath(
        &self,
        pszpathrel: &windows_core::PCWSTR,
        dwreserved: u32,
    ) -> windows_core::Result<()>;
    fn Resolve(&self, hwnd: HWND, fflags: u32) -> windows_core::Result<()>;
    fn SetPath(&self, pszfile: &windows_core::PCWSTR) -> windows_core::Result<()>;
}
impl IShellLinkW_Vtbl {
    pub const fn new<Identity: IShellLinkW_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetPath<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pszfile: windows_core::PWSTR,
            cch: i32,
            pfd: *mut WIN32_FIND_DATAW,
            fflags: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::GetPath(
                    this,
                    core::mem::transmute_copy(&pszfile),
                    core::mem::transmute_copy(&cch),
                    core::mem::transmute_copy(&pfd),
                    core::mem::transmute_copy(&fflags),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetIDList<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ppidl: *mut LPITEMIDLIST,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellLinkW_Impl::GetIDList(this) {
                    Ok(ok__) => {
                        ppidl.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetIDList<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pidl: *const ITEMIDLIST,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetIDList(this, core::mem::transmute_copy(&pidl)).into()
            }
        }
        unsafe extern "system" fn GetDescription<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszname: windows_core::PWSTR,
            cch: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::GetDescription(
                    this,
                    core::mem::transmute_copy(&pszname),
                    core::mem::transmute_copy(&cch),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetDescription<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszname: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetDescription(this, core::mem::transmute(&pszname)).into()
            }
        }
        unsafe extern "system" fn GetWorkingDirectory<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszdir: windows_core::PWSTR,
            cch: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::GetWorkingDirectory(
                    this,
                    core::mem::transmute_copy(&pszdir),
                    core::mem::transmute_copy(&cch),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetWorkingDirectory<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszdir: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetWorkingDirectory(this, core::mem::transmute(&pszdir)).into()
            }
        }
        unsafe extern "system" fn GetArguments<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pszargs: windows_core::PWSTR,
            cch: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::GetArguments(
                    this,
                    core::mem::transmute_copy(&pszargs),
                    core::mem::transmute_copy(&cch),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetArguments<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pszargs: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetArguments(this, core::mem::transmute(&pszargs)).into()
            }
        }
        unsafe extern "system" fn GetHotkey<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pwhotkey: *mut u16,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellLinkW_Impl::GetHotkey(this) {
                    Ok(ok__) => {
                        pwhotkey.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetHotkey<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            whotkey: u16,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetHotkey(this, core::mem::transmute_copy(&whotkey)).into()
            }
        }
        unsafe extern "system" fn GetShowCmd<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pishowcmd: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IShellLinkW_Impl::GetShowCmd(this) {
                    Ok(ok__) => {
                        pishowcmd.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn SetShowCmd<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            ishowcmd: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetShowCmd(this, core::mem::transmute_copy(&ishowcmd)).into()
            }
        }
        unsafe extern "system" fn GetIconLocation<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psziconpath: windows_core::PWSTR,
            cch: i32,
            piicon: *mut i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::GetIconLocation(
                    this,
                    core::mem::transmute_copy(&psziconpath),
                    core::mem::transmute_copy(&cch),
                    core::mem::transmute_copy(&piicon),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetIconLocation<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            psziconpath: windows_core::PCWSTR,
            iicon: i32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetIconLocation(
                    this,
                    core::mem::transmute(&psziconpath),
                    core::mem::transmute_copy(&iicon),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetRelativePath<
            Identity: IShellLinkW_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pszpathrel: windows_core::PCWSTR,
            dwreserved: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetRelativePath(
                    this,
                    core::mem::transmute(&pszpathrel),
                    core::mem::transmute_copy(&dwreserved),
                )
                .into()
            }
        }
        unsafe extern "system" fn Resolve<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            hwnd: HWND,
            fflags: u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::Resolve(
                    this,
                    core::mem::transmute_copy(&hwnd),
                    core::mem::transmute_copy(&fflags),
                )
                .into()
            }
        }
        unsafe extern "system" fn SetPath<Identity: IShellLinkW_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            pszfile: windows_core::PCWSTR,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IShellLinkW_Impl::SetPath(this, core::mem::transmute(&pszfile)).into()
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetPath: GetPath::<Identity, OFFSET>,
            GetIDList: GetIDList::<Identity, OFFSET>,
            SetIDList: SetIDList::<Identity, OFFSET>,
            GetDescription: GetDescription::<Identity, OFFSET>,
            SetDescription: SetDescription::<Identity, OFFSET>,
            GetWorkingDirectory: GetWorkingDirectory::<Identity, OFFSET>,
            SetWorkingDirectory: SetWorkingDirectory::<Identity, OFFSET>,
            GetArguments: GetArguments::<Identity, OFFSET>,
            SetArguments: SetArguments::<Identity, OFFSET>,
            GetHotkey: GetHotkey::<Identity, OFFSET>,
            SetHotkey: SetHotkey::<Identity, OFFSET>,
            GetShowCmd: GetShowCmd::<Identity, OFFSET>,
            SetShowCmd: SetShowCmd::<Identity, OFFSET>,
            GetIconLocation: GetIconLocation::<Identity, OFFSET>,
            SetIconLocation: SetIconLocation::<Identity, OFFSET>,
            SetRelativePath: SetRelativePath::<Identity, OFFSET>,
            Resolve: Resolve::<Identity, OFFSET>,
            SetPath: SetPath::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IShellLinkW as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IShellLinkW {}
windows_core::imp::define_interface!(
    IStorage,
    IStorage_Vtbl,
    0x0000000b_0000_0000_c000_000000000046
);
windows_core::imp::interface_hierarchy!(IStorage, windows_core::IUnknown);
#[repr(C)]
pub struct IStorage_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    CreateStream: usize,
    OpenStream: usize,
    CreateStorage: usize,
    OpenStorage: usize,
    CopyTo: usize,
    MoveElementTo: usize,
    Commit: usize,
    Revert: usize,
    EnumElements: usize,
    DestroyElement: usize,
    RenameElement: usize,
    SetElementTimes: usize,
    SetClass: usize,
    SetStateBits: usize,
    Stat: usize,
}
impl windows_core::RuntimeName for IStorage {}
windows_core::imp::define_interface!(
    IStream,
    IStream_Vtbl,
    0x0000000c_0000_0000_c000_000000000046
);
impl core::ops::Deref for IStream {
    type Target = ISequentialStream;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IStream, windows_core::IUnknown, ISequentialStream);
#[repr(C)]
pub struct IStream_Vtbl {
    pub base__: ISequentialStream_Vtbl,
    Seek: usize,
    SetSize: usize,
    CopyTo: usize,
    Commit: usize,
    Revert: usize,
    LockRegion: usize,
    UnlockRegion: usize,
    Stat: usize,
    Clone: usize,
}
impl windows_core::RuntimeName for IStream {}
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ITEMIDLIST {
    pub mkid: SHITEMID,
}
windows_core::imp::define_interface!(
    IWICBitmap,
    IWICBitmap_Vtbl,
    0x00000121_a8f2_4877_ba0a_fd2b6645fb94
);
impl core::ops::Deref for IWICBitmap {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IWICBitmap, windows_core::IUnknown, IWICBitmapSource);
#[repr(C)]
pub struct IWICBitmap_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    Lock: usize,
    SetPalette: usize,
    SetResolution: usize,
}
impl windows_core::RuntimeName for IWICBitmap {}
windows_core::imp::define_interface!(
    IWICBitmapClipper,
    IWICBitmapClipper_Vtbl,
    0xe4fbcf03_223d_4e81_9333_d635556dd1b5
);
impl core::ops::Deref for IWICBitmapClipper {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICBitmapClipper,
    windows_core::IUnknown,
    IWICBitmapSource
);
#[repr(C)]
pub struct IWICBitmapClipper_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    Initialize: usize,
}
impl windows_core::RuntimeName for IWICBitmapClipper {}
windows_core::imp::define_interface!(
    IWICBitmapCodecInfo,
    IWICBitmapCodecInfo_Vtbl,
    0xe87a44c4_b76e_4c47_8b09_298eb12a2714
);
impl core::ops::Deref for IWICBitmapCodecInfo {
    type Target = IWICComponentInfo;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICBitmapCodecInfo,
    windows_core::IUnknown,
    IWICComponentInfo
);
#[repr(C)]
pub struct IWICBitmapCodecInfo_Vtbl {
    pub base__: IWICComponentInfo_Vtbl,
    GetContainerFormat: usize,
    GetPixelFormats: usize,
    GetColorManagementVersion: usize,
    GetDeviceManufacturer: usize,
    GetDeviceModels: usize,
    GetMimeTypes: usize,
    GetFileExtensions: usize,
    DoesSupportAnimation: usize,
    DoesSupportChromakey: usize,
    DoesSupportLossless: usize,
    DoesSupportMultiframe: usize,
    MatchesMimeType: usize,
}
impl windows_core::RuntimeName for IWICBitmapCodecInfo {}
windows_core::imp::define_interface!(
    IWICBitmapDecoder,
    IWICBitmapDecoder_Vtbl,
    0x9edde9e7_8dee_47ea_99df_e6faf2ed44bf
);
windows_core::imp::interface_hierarchy!(IWICBitmapDecoder, windows_core::IUnknown);
impl IWICBitmapDecoder {
    pub(crate) unsafe fn QueryCapability<P0>(&self, pistream: P0) -> windows_core::Result<u32>
    where
        P0: windows_core::Param<IStream>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).QueryCapability)(
                windows_core::Interface::as_raw(self),
                pistream.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn Initialize<P0>(
        &self,
        pistream: P0,
        cacheoptions: WICDecodeOptions,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IStream>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Initialize)(
                windows_core::Interface::as_raw(self),
                pistream.param().abi(),
                cacheoptions,
            )
        }
    }
    pub(crate) unsafe fn GetContainerFormat(&self) -> windows_core::Result<windows_core::GUID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetContainerFormat)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetDecoderInfo(&self) -> windows_core::Result<IWICBitmapDecoderInfo> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetDecoderInfo)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CopyPalette<P0>(&self, pipalette: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IWICPalette>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyPalette)(
                windows_core::Interface::as_raw(self),
                pipalette.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn GetMetadataQueryReader(
        &self,
    ) -> windows_core::Result<IWICMetadataQueryReader> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetMetadataQueryReader)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetPreview(&self) -> windows_core::Result<IWICBitmapSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPreview)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetColorContexts(
        &self,
        ccount: u32,
        ppicolorcontexts: *mut Option<IWICColorContext>,
        pcactualcount: *mut u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetColorContexts)(
                windows_core::Interface::as_raw(self),
                ccount,
                core::mem::transmute(ppicolorcontexts),
                pcactualcount as _,
            )
        }
    }
    pub(crate) unsafe fn GetThumbnail(&self) -> windows_core::Result<IWICBitmapSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetThumbnail)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetFrameCount(&self) -> windows_core::Result<u32> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFrameCount)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetFrame(
        &self,
        index: u32,
    ) -> windows_core::Result<IWICBitmapFrameDecode> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetFrame)(
                windows_core::Interface::as_raw(self),
                index,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IWICBitmapDecoder_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub QueryCapability: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub Initialize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        WICDecodeOptions,
    ) -> windows_core::HRESULT,
    pub GetContainerFormat: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut windows_core::GUID,
    ) -> windows_core::HRESULT,
    pub GetDecoderInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CopyPalette: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetMetadataQueryReader: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetPreview: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetColorContexts: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub GetThumbnail: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetFrameCount:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut u32) -> windows_core::HRESULT,
    pub GetFrame: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IWICBitmapDecoder_Impl: windows_core::IUnknownImpl {
    fn QueryCapability(&self, pistream: windows_core::Ref<IStream>) -> windows_core::Result<u32>;
    fn Initialize(
        &self,
        pistream: windows_core::Ref<IStream>,
        cacheoptions: WICDecodeOptions,
    ) -> windows_core::Result<()>;
    fn GetContainerFormat(&self) -> windows_core::Result<windows_core::GUID>;
    fn GetDecoderInfo(&self) -> windows_core::Result<IWICBitmapDecoderInfo>;
    fn CopyPalette(&self, pipalette: windows_core::Ref<IWICPalette>) -> windows_core::Result<()>;
    fn GetMetadataQueryReader(&self) -> windows_core::Result<IWICMetadataQueryReader>;
    fn GetPreview(&self) -> windows_core::Result<IWICBitmapSource>;
    fn GetColorContexts(
        &self,
        ccount: u32,
        ppicolorcontexts: windows_core::OutRef<IWICColorContext>,
        pcactualcount: *mut u32,
    ) -> windows_core::Result<()>;
    fn GetThumbnail(&self) -> windows_core::Result<IWICBitmapSource>;
    fn GetFrameCount(&self) -> windows_core::Result<u32>;
    fn GetFrame(&self, index: u32) -> windows_core::Result<IWICBitmapFrameDecode>;
}
impl IWICBitmapDecoder_Vtbl {
    pub const fn new<Identity: IWICBitmapDecoder_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn QueryCapability<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pistream: *mut core::ffi::c_void,
            pdwcapability: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::QueryCapability(
                    this,
                    core::mem::transmute_copy(&pistream),
                ) {
                    Ok(ok__) => {
                        pdwcapability.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn Initialize<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pistream: *mut core::ffi::c_void,
            cacheoptions: WICDecodeOptions,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapDecoder_Impl::Initialize(
                    this,
                    core::mem::transmute_copy(&pistream),
                    core::mem::transmute_copy(&cacheoptions),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetContainerFormat<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pguidcontainerformat: *mut windows_core::GUID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetContainerFormat(this) {
                    Ok(ok__) => {
                        pguidcontainerformat.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetDecoderInfo<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppidecoderinfo: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetDecoderInfo(this) {
                    Ok(ok__) => {
                        ppidecoderinfo.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CopyPalette<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pipalette: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapDecoder_Impl::CopyPalette(this, core::mem::transmute_copy(&pipalette))
                    .into()
            }
        }
        unsafe extern "system" fn GetMetadataQueryReader<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppimetadataqueryreader: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetMetadataQueryReader(this) {
                    Ok(ok__) => {
                        ppimetadataqueryreader.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetPreview<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppibitmapsource: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetPreview(this) {
                    Ok(ok__) => {
                        ppibitmapsource.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetColorContexts<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ccount: u32,
            ppicolorcontexts: *mut *mut core::ffi::c_void,
            pcactualcount: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapDecoder_Impl::GetColorContexts(
                    this,
                    core::mem::transmute_copy(&ccount),
                    core::mem::transmute_copy(&ppicolorcontexts),
                    core::mem::transmute_copy(&pcactualcount),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetThumbnail<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppithumbnail: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetThumbnail(this) {
                    Ok(ok__) => {
                        ppithumbnail.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetFrameCount<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pcount: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetFrameCount(this) {
                    Ok(ok__) => {
                        pcount.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetFrame<
            Identity: IWICBitmapDecoder_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            index: u32,
            ppibitmapframe: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapDecoder_Impl::GetFrame(this, core::mem::transmute_copy(&index)) {
                    Ok(ok__) => {
                        ppibitmapframe.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            QueryCapability: QueryCapability::<Identity, OFFSET>,
            Initialize: Initialize::<Identity, OFFSET>,
            GetContainerFormat: GetContainerFormat::<Identity, OFFSET>,
            GetDecoderInfo: GetDecoderInfo::<Identity, OFFSET>,
            CopyPalette: CopyPalette::<Identity, OFFSET>,
            GetMetadataQueryReader: GetMetadataQueryReader::<Identity, OFFSET>,
            GetPreview: GetPreview::<Identity, OFFSET>,
            GetColorContexts: GetColorContexts::<Identity, OFFSET>,
            GetThumbnail: GetThumbnail::<Identity, OFFSET>,
            GetFrameCount: GetFrameCount::<Identity, OFFSET>,
            GetFrame: GetFrame::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IWICBitmapDecoder as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IWICBitmapDecoder {}
windows_core::imp::define_interface!(
    IWICBitmapDecoderInfo,
    IWICBitmapDecoderInfo_Vtbl,
    0xd8cd007f_d08f_4191_9bfc_236ea7f0e4b5
);
impl core::ops::Deref for IWICBitmapDecoderInfo {
    type Target = IWICBitmapCodecInfo;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICBitmapDecoderInfo,
    windows_core::IUnknown,
    IWICComponentInfo,
    IWICBitmapCodecInfo
);
#[repr(C)]
pub struct IWICBitmapDecoderInfo_Vtbl {
    pub base__: IWICBitmapCodecInfo_Vtbl,
    GetPatterns: usize,
    MatchesPattern: usize,
    CreateInstance: usize,
}
impl windows_core::RuntimeName for IWICBitmapDecoderInfo {}
windows_core::imp::define_interface!(
    IWICBitmapEncoder,
    IWICBitmapEncoder_Vtbl,
    0x00000103_a8f2_4877_ba0a_fd2b6645fb94
);
windows_core::imp::interface_hierarchy!(IWICBitmapEncoder, windows_core::IUnknown);
#[repr(C)]
pub struct IWICBitmapEncoder_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Initialize: usize,
    GetContainerFormat: usize,
    GetEncoderInfo: usize,
    SetColorContexts: usize,
    SetPalette: usize,
    SetThumbnail: usize,
    SetPreview: usize,
    CreateNewFrame: usize,
    Commit: usize,
    GetMetadataQueryWriter: usize,
}
impl windows_core::RuntimeName for IWICBitmapEncoder {}
windows_core::imp::define_interface!(
    IWICBitmapFlipRotator,
    IWICBitmapFlipRotator_Vtbl,
    0x5009834f_2d6a_41ce_9e1b_17c5aff7a782
);
impl core::ops::Deref for IWICBitmapFlipRotator {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICBitmapFlipRotator,
    windows_core::IUnknown,
    IWICBitmapSource
);
#[repr(C)]
pub struct IWICBitmapFlipRotator_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    Initialize: usize,
}
impl windows_core::RuntimeName for IWICBitmapFlipRotator {}
windows_core::imp::define_interface!(
    IWICBitmapFrameDecode,
    IWICBitmapFrameDecode_Vtbl,
    0x3b16811b_6a43_4ec9_a813_3d930c13b940
);
impl core::ops::Deref for IWICBitmapFrameDecode {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICBitmapFrameDecode,
    windows_core::IUnknown,
    IWICBitmapSource
);
impl IWICBitmapFrameDecode {
    pub(crate) unsafe fn GetMetadataQueryReader(
        &self,
    ) -> windows_core::Result<IWICMetadataQueryReader> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetMetadataQueryReader)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn GetColorContexts(
        &self,
        ccount: u32,
        ppicolorcontexts: *mut Option<IWICColorContext>,
        pcactualcount: *mut u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetColorContexts)(
                windows_core::Interface::as_raw(self),
                ccount,
                core::mem::transmute(ppicolorcontexts),
                pcactualcount as _,
            )
        }
    }
    pub(crate) unsafe fn GetThumbnail(&self) -> windows_core::Result<IWICBitmapSource> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetThumbnail)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IWICBitmapFrameDecode_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    pub GetMetadataQueryReader: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetColorContexts: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        *mut *mut core::ffi::c_void,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub GetThumbnail: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IWICBitmapFrameDecode_Impl: IWICBitmapSource_Impl {
    fn GetMetadataQueryReader(&self) -> windows_core::Result<IWICMetadataQueryReader>;
    fn GetColorContexts(
        &self,
        ccount: u32,
        ppicolorcontexts: windows_core::OutRef<IWICColorContext>,
        pcactualcount: *mut u32,
    ) -> windows_core::Result<()>;
    fn GetThumbnail(&self) -> windows_core::Result<IWICBitmapSource>;
}
impl IWICBitmapFrameDecode_Vtbl {
    pub const fn new<Identity: IWICBitmapFrameDecode_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetMetadataQueryReader<
            Identity: IWICBitmapFrameDecode_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppimetadataqueryreader: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapFrameDecode_Impl::GetMetadataQueryReader(this) {
                    Ok(ok__) => {
                        ppimetadataqueryreader.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetColorContexts<
            Identity: IWICBitmapFrameDecode_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ccount: u32,
            ppicolorcontexts: *mut *mut core::ffi::c_void,
            pcactualcount: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapFrameDecode_Impl::GetColorContexts(
                    this,
                    core::mem::transmute_copy(&ccount),
                    core::mem::transmute_copy(&ppicolorcontexts),
                    core::mem::transmute_copy(&pcactualcount),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetThumbnail<
            Identity: IWICBitmapFrameDecode_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppithumbnail: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapFrameDecode_Impl::GetThumbnail(this) {
                    Ok(ok__) => {
                        ppithumbnail.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: IWICBitmapSource_Vtbl::new::<Identity, OFFSET>(),
            GetMetadataQueryReader: GetMetadataQueryReader::<Identity, OFFSET>,
            GetColorContexts: GetColorContexts::<Identity, OFFSET>,
            GetThumbnail: GetThumbnail::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IWICBitmapFrameDecode as windows_core::Interface>::IID
            || iid == &<IWICBitmapSource as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IWICBitmapFrameDecode {}
windows_core::imp::define_interface!(
    IWICBitmapScaler,
    IWICBitmapScaler_Vtbl,
    0x00000302_a8f2_4877_ba0a_fd2b6645fb94
);
impl core::ops::Deref for IWICBitmapScaler {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(IWICBitmapScaler, windows_core::IUnknown, IWICBitmapSource);
impl IWICBitmapScaler {
    pub(crate) unsafe fn Initialize<P0>(
        &self,
        pisource: P0,
        uiwidth: u32,
        uiheight: u32,
        mode: WICBitmapInterpolationMode,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IWICBitmapSource>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Initialize)(
                windows_core::Interface::as_raw(self),
                pisource.param().abi(),
                uiwidth,
                uiheight,
                mode,
            )
        }
    }
}
#[repr(C)]
pub struct IWICBitmapScaler_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    pub Initialize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        u32,
        WICBitmapInterpolationMode,
    ) -> windows_core::HRESULT,
}
pub trait IWICBitmapScaler_Impl: IWICBitmapSource_Impl {
    fn Initialize(
        &self,
        pisource: windows_core::Ref<IWICBitmapSource>,
        uiwidth: u32,
        uiheight: u32,
        mode: WICBitmapInterpolationMode,
    ) -> windows_core::Result<()>;
}
impl IWICBitmapScaler_Vtbl {
    pub const fn new<Identity: IWICBitmapScaler_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Initialize<
            Identity: IWICBitmapScaler_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pisource: *mut core::ffi::c_void,
            uiwidth: u32,
            uiheight: u32,
            mode: WICBitmapInterpolationMode,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapScaler_Impl::Initialize(
                    this,
                    core::mem::transmute_copy(&pisource),
                    core::mem::transmute_copy(&uiwidth),
                    core::mem::transmute_copy(&uiheight),
                    core::mem::transmute_copy(&mode),
                )
                .into()
            }
        }
        Self {
            base__: IWICBitmapSource_Vtbl::new::<Identity, OFFSET>(),
            Initialize: Initialize::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IWICBitmapScaler as windows_core::Interface>::IID
            || iid == &<IWICBitmapSource as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IWICBitmapScaler {}
windows_core::imp::define_interface!(
    IWICBitmapSource,
    IWICBitmapSource_Vtbl,
    0x00000120_a8f2_4877_ba0a_fd2b6645fb94
);
windows_core::imp::interface_hierarchy!(IWICBitmapSource, windows_core::IUnknown);
impl IWICBitmapSource {
    pub(crate) unsafe fn GetSize(
        &self,
        puiwidth: *mut u32,
        puiheight: *mut u32,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetSize)(
                windows_core::Interface::as_raw(self),
                puiwidth as _,
                puiheight as _,
            )
        }
    }
    pub(crate) unsafe fn GetPixelFormat(&self) -> windows_core::Result<WICPixelFormatGUID> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).GetPixelFormat)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub(crate) unsafe fn GetResolution(
        &self,
        pdpix: *mut f64,
        pdpiy: *mut f64,
    ) -> windows_core::HRESULT {
        unsafe {
            (windows_core::Interface::vtable(self).GetResolution)(
                windows_core::Interface::as_raw(self),
                pdpix as _,
                pdpiy as _,
            )
        }
    }
    pub(crate) unsafe fn CopyPalette<P0>(&self, pipalette: P0) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IWICPalette>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).CopyPalette)(
                windows_core::Interface::as_raw(self),
                pipalette.param().abi(),
            )
        }
    }
    pub(crate) unsafe fn CopyPixels(
        &self,
        prc: *const WICRect,
        cbstride: u32,
        cbbuffersize: u32,
    ) -> windows_core::Result<u8> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CopyPixels)(
                windows_core::Interface::as_raw(self),
                prc,
                cbstride,
                cbbuffersize,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IWICBitmapSource_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub GetSize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut u32,
        *mut u32,
    ) -> windows_core::HRESULT,
    pub GetPixelFormat: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut WICPixelFormatGUID,
    ) -> windows_core::HRESULT,
    pub GetResolution: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut f64,
        *mut f64,
    ) -> windows_core::HRESULT,
    pub CopyPalette: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CopyPixels: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const WICRect,
        u32,
        u32,
        *mut u8,
    ) -> windows_core::HRESULT,
}
pub trait IWICBitmapSource_Impl: windows_core::IUnknownImpl {
    fn GetSize(&self, puiwidth: *mut u32, puiheight: *mut u32) -> windows_core::Result<()>;
    fn GetPixelFormat(&self) -> windows_core::Result<WICPixelFormatGUID>;
    fn GetResolution(&self, pdpix: *mut f64, pdpiy: *mut f64) -> windows_core::Result<()>;
    fn CopyPalette(&self, pipalette: windows_core::Ref<IWICPalette>) -> windows_core::Result<()>;
    fn CopyPixels(
        &self,
        prc: *const WICRect,
        cbstride: u32,
        cbbuffersize: u32,
    ) -> windows_core::Result<u8>;
}
impl IWICBitmapSource_Vtbl {
    pub const fn new<Identity: IWICBitmapSource_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn GetSize<Identity: IWICBitmapSource_Impl, const OFFSET: isize>(
            this: *mut core::ffi::c_void,
            puiwidth: *mut u32,
            puiheight: *mut u32,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapSource_Impl::GetSize(
                    this,
                    core::mem::transmute_copy(&puiwidth),
                    core::mem::transmute_copy(&puiheight),
                )
                .into()
            }
        }
        unsafe extern "system" fn GetPixelFormat<
            Identity: IWICBitmapSource_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppixelformat: *mut WICPixelFormatGUID,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapSource_Impl::GetPixelFormat(this) {
                    Ok(ok__) => {
                        ppixelformat.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn GetResolution<
            Identity: IWICBitmapSource_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pdpix: *mut f64,
            pdpiy: *mut f64,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapSource_Impl::GetResolution(
                    this,
                    core::mem::transmute_copy(&pdpix),
                    core::mem::transmute_copy(&pdpiy),
                )
                .into()
            }
        }
        unsafe extern "system" fn CopyPalette<
            Identity: IWICBitmapSource_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pipalette: *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICBitmapSource_Impl::CopyPalette(this, core::mem::transmute_copy(&pipalette))
                    .into()
            }
        }
        unsafe extern "system" fn CopyPixels<
            Identity: IWICBitmapSource_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            prc: *const WICRect,
            cbstride: u32,
            cbbuffersize: u32,
            pbbuffer: *mut u8,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICBitmapSource_Impl::CopyPixels(
                    this,
                    core::mem::transmute_copy(&prc),
                    core::mem::transmute_copy(&cbstride),
                    core::mem::transmute_copy(&cbbuffersize),
                ) {
                    Ok(ok__) => {
                        pbbuffer.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            GetSize: GetSize::<Identity, OFFSET>,
            GetPixelFormat: GetPixelFormat::<Identity, OFFSET>,
            GetResolution: GetResolution::<Identity, OFFSET>,
            CopyPalette: CopyPalette::<Identity, OFFSET>,
            CopyPixels: CopyPixels::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IWICBitmapSource as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IWICBitmapSource {}
windows_core::imp::define_interface!(
    IWICColorContext,
    IWICColorContext_Vtbl,
    0x3c613a02_34b2_44ea_9a7c_45aea9c6fd6d
);
windows_core::imp::interface_hierarchy!(IWICColorContext, windows_core::IUnknown);
#[repr(C)]
pub struct IWICColorContext_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    InitializeFromFilename: usize,
    InitializeFromMemory: usize,
    InitializeFromExifColorSpace: usize,
    GetType: usize,
    GetProfileBytes: usize,
    GetExifColorSpace: usize,
}
impl windows_core::RuntimeName for IWICColorContext {}
windows_core::imp::define_interface!(
    IWICColorTransform,
    IWICColorTransform_Vtbl,
    0xb66f034f_d0e2_40ab_b436_6de39e321a94
);
impl core::ops::Deref for IWICColorTransform {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICColorTransform,
    windows_core::IUnknown,
    IWICBitmapSource
);
#[repr(C)]
pub struct IWICColorTransform_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    Initialize: usize,
}
impl windows_core::RuntimeName for IWICColorTransform {}
windows_core::imp::define_interface!(
    IWICComponentInfo,
    IWICComponentInfo_Vtbl,
    0x23bc3f0a_698b_4357_886b_f24d50671334
);
windows_core::imp::interface_hierarchy!(IWICComponentInfo, windows_core::IUnknown);
#[repr(C)]
pub struct IWICComponentInfo_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetComponentType: usize,
    GetCLSID: usize,
    GetSigningStatus: usize,
    GetAuthor: usize,
    GetVendorGUID: usize,
    GetVersion: usize,
    GetSpecVersion: usize,
    GetFriendlyName: usize,
}
impl windows_core::RuntimeName for IWICComponentInfo {}
windows_core::imp::define_interface!(
    IWICFastMetadataEncoder,
    IWICFastMetadataEncoder_Vtbl,
    0xb84e2c09_78c9_4ac4_8bd3_524ae1663a2f
);
windows_core::imp::interface_hierarchy!(IWICFastMetadataEncoder, windows_core::IUnknown);
#[repr(C)]
pub struct IWICFastMetadataEncoder_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    Commit: usize,
    GetMetadataQueryWriter: usize,
}
impl windows_core::RuntimeName for IWICFastMetadataEncoder {}
windows_core::imp::define_interface!(
    IWICFormatConverter,
    IWICFormatConverter_Vtbl,
    0x00000301_a8f2_4877_ba0a_fd2b6645fb94
);
impl core::ops::Deref for IWICFormatConverter {
    type Target = IWICBitmapSource;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICFormatConverter,
    windows_core::IUnknown,
    IWICBitmapSource
);
impl IWICFormatConverter {
    pub(crate) unsafe fn Initialize<P0, P3>(
        &self,
        pisource: P0,
        dstformat: REFWICPixelFormatGUID,
        dither: WICBitmapDitherType,
        pipalette: P3,
        alphathresholdpercent: f64,
        palettetranslate: WICBitmapPaletteType,
    ) -> windows_core::HRESULT
    where
        P0: windows_core::Param<IWICBitmapSource>,
        P3: windows_core::Param<IWICPalette>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Initialize)(
                windows_core::Interface::as_raw(self),
                pisource.param().abi(),
                dstformat,
                dither,
                pipalette.param().abi(),
                alphathresholdpercent,
                palettetranslate,
            )
        }
    }
    pub(crate) unsafe fn CanConvert(
        &self,
        srcpixelformat: REFWICPixelFormatGUID,
        dstpixelformat: REFWICPixelFormatGUID,
    ) -> windows_core::Result<windows_core::BOOL> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CanConvert)(
                windows_core::Interface::as_raw(self),
                srcpixelformat,
                dstpixelformat,
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
#[repr(C)]
pub struct IWICFormatConverter_Vtbl {
    pub base__: IWICBitmapSource_Vtbl,
    pub Initialize: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        REFWICPixelFormatGUID,
        WICBitmapDitherType,
        *mut core::ffi::c_void,
        f64,
        WICBitmapPaletteType,
    ) -> windows_core::HRESULT,
    pub CanConvert: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        REFWICPixelFormatGUID,
        REFWICPixelFormatGUID,
        *mut windows_core::BOOL,
    ) -> windows_core::HRESULT,
}
pub trait IWICFormatConverter_Impl: IWICBitmapSource_Impl {
    fn Initialize(
        &self,
        pisource: windows_core::Ref<IWICBitmapSource>,
        dstformat: REFWICPixelFormatGUID,
        dither: WICBitmapDitherType,
        pipalette: windows_core::Ref<IWICPalette>,
        alphathresholdpercent: f64,
        palettetranslate: WICBitmapPaletteType,
    ) -> windows_core::Result<()>;
    fn CanConvert(
        &self,
        srcpixelformat: REFWICPixelFormatGUID,
        dstpixelformat: REFWICPixelFormatGUID,
    ) -> windows_core::Result<windows_core::BOOL>;
}
impl IWICFormatConverter_Vtbl {
    pub const fn new<Identity: IWICFormatConverter_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn Initialize<
            Identity: IWICFormatConverter_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pisource: *mut core::ffi::c_void,
            dstformat: REFWICPixelFormatGUID,
            dither: WICBitmapDitherType,
            pipalette: *mut core::ffi::c_void,
            alphathresholdpercent: f64,
            palettetranslate: WICBitmapPaletteType,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                IWICFormatConverter_Impl::Initialize(
                    this,
                    core::mem::transmute_copy(&pisource),
                    core::mem::transmute_copy(&dstformat),
                    core::mem::transmute_copy(&dither),
                    core::mem::transmute_copy(&pipalette),
                    core::mem::transmute_copy(&alphathresholdpercent),
                    core::mem::transmute_copy(&palettetranslate),
                )
                .into()
            }
        }
        unsafe extern "system" fn CanConvert<
            Identity: IWICFormatConverter_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            srcpixelformat: REFWICPixelFormatGUID,
            dstpixelformat: REFWICPixelFormatGUID,
            pfcanconvert: *mut windows_core::BOOL,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICFormatConverter_Impl::CanConvert(
                    this,
                    core::mem::transmute_copy(&srcpixelformat),
                    core::mem::transmute_copy(&dstpixelformat),
                ) {
                    Ok(ok__) => {
                        pfcanconvert.write(ok__);
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: IWICBitmapSource_Vtbl::new::<Identity, OFFSET>(),
            Initialize: Initialize::<Identity, OFFSET>,
            CanConvert: CanConvert::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IWICFormatConverter as windows_core::Interface>::IID
            || iid == &<IWICBitmapSource as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IWICFormatConverter {}
windows_core::imp::define_interface!(
    IWICImagingFactory,
    IWICImagingFactory_Vtbl,
    0xec5ec8a9_c395_4314_9c77_54d7a935ff70
);
windows_core::imp::interface_hierarchy!(IWICImagingFactory, windows_core::IUnknown);
impl IWICImagingFactory {
    pub(crate) unsafe fn CreateDecoderFromFilename<P0>(
        &self,
        wzfilename: P0,
        pguidvendor: *const windows_core::GUID,
        dwdesiredaccess: u32,
        metadataoptions: WICDecodeOptions,
    ) -> windows_core::Result<IWICBitmapDecoder>
    where
        P0: windows_core::Param<windows_core::PCWSTR>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateDecoderFromFilename)(
                windows_core::Interface::as_raw(self),
                wzfilename.param().abi(),
                pguidvendor,
                dwdesiredaccess,
                metadataoptions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateDecoderFromStream<P0>(
        &self,
        pistream: P0,
        pguidvendor: *const windows_core::GUID,
        metadataoptions: WICDecodeOptions,
    ) -> windows_core::Result<IWICBitmapDecoder>
    where
        P0: windows_core::Param<IStream>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateDecoderFromStream)(
                windows_core::Interface::as_raw(self),
                pistream.param().abi(),
                pguidvendor,
                metadataoptions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateDecoderFromFileHandle(
        &self,
        hfile: usize,
        pguidvendor: *const windows_core::GUID,
        metadataoptions: WICDecodeOptions,
    ) -> windows_core::Result<IWICBitmapDecoder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateDecoderFromFileHandle)(
                windows_core::Interface::as_raw(self),
                hfile,
                pguidvendor,
                metadataoptions,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateComponentInfo(
        &self,
        clsidcomponent: *const windows_core::GUID,
    ) -> windows_core::Result<IWICComponentInfo> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateComponentInfo)(
                windows_core::Interface::as_raw(self),
                clsidcomponent,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateDecoder(
        &self,
        guidcontainerformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICBitmapDecoder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateDecoder)(
                windows_core::Interface::as_raw(self),
                guidcontainerformat,
                pguidvendor,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateEncoder(
        &self,
        guidcontainerformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICBitmapEncoder> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateEncoder)(
                windows_core::Interface::as_raw(self),
                guidcontainerformat,
                pguidvendor,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreatePalette(&self) -> windows_core::Result<IWICPalette> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreatePalette)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateFormatConverter(&self) -> windows_core::Result<IWICFormatConverter> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFormatConverter)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapScaler(&self) -> windows_core::Result<IWICBitmapScaler> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapScaler)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapClipper(&self) -> windows_core::Result<IWICBitmapClipper> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapClipper)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFlipRotator(
        &self,
    ) -> windows_core::Result<IWICBitmapFlipRotator> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFlipRotator)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateStream(&self) -> windows_core::Result<IWICStream> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateStream)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateColorContext(&self) -> windows_core::Result<IWICColorContext> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorContext)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateColorTransformer(&self) -> windows_core::Result<IWICColorTransform> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateColorTransformer)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmap(
        &self,
        uiwidth: u32,
        uiheight: u32,
        pixelformat: REFWICPixelFormatGUID,
        option: WICBitmapCreateCacheOption,
    ) -> windows_core::Result<IWICBitmap> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmap)(
                windows_core::Interface::as_raw(self),
                uiwidth,
                uiheight,
                pixelformat,
                option,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromSource<P0>(
        &self,
        pibitmapsource: P0,
        option: WICBitmapCreateCacheOption,
    ) -> windows_core::Result<IWICBitmap>
    where
        P0: windows_core::Param<IWICBitmapSource>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromSource)(
                windows_core::Interface::as_raw(self),
                pibitmapsource.param().abi(),
                option,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromSourceRect<P0>(
        &self,
        pibitmapsource: P0,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> windows_core::Result<IWICBitmap>
    where
        P0: windows_core::Param<IWICBitmapSource>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromSourceRect)(
                windows_core::Interface::as_raw(self),
                pibitmapsource.param().abi(),
                x,
                y,
                width,
                height,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromMemory(
        &self,
        uiwidth: u32,
        uiheight: u32,
        pixelformat: REFWICPixelFormatGUID,
        cbstride: u32,
        cbbuffersize: u32,
        pbbuffer: *const u8,
    ) -> windows_core::Result<IWICBitmap> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromMemory)(
                windows_core::Interface::as_raw(self),
                uiwidth,
                uiheight,
                pixelformat,
                cbstride,
                cbbuffersize,
                pbbuffer,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromHBITMAP(
        &self,
        hbitmap: HBITMAP,
        hpalette: HPALETTE,
        options: WICBitmapAlphaChannelOption,
    ) -> windows_core::Result<IWICBitmap> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromHBITMAP)(
                windows_core::Interface::as_raw(self),
                hbitmap,
                hpalette,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateBitmapFromHICON(
        &self,
        hicon: HICON,
    ) -> windows_core::Result<IWICBitmap> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateBitmapFromHICON)(
                windows_core::Interface::as_raw(self),
                hicon,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateComponentEnumerator(
        &self,
        componenttypes: u32,
        options: u32,
    ) -> windows_core::Result<IEnumUnknown> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateComponentEnumerator)(
                windows_core::Interface::as_raw(self),
                componenttypes,
                options,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateFastMetadataEncoderFromDecoder<P0>(
        &self,
        pidecoder: P0,
    ) -> windows_core::Result<IWICFastMetadataEncoder>
    where
        P0: windows_core::Param<IWICBitmapDecoder>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFastMetadataEncoderFromDecoder)(
                windows_core::Interface::as_raw(self),
                pidecoder.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateFastMetadataEncoderFromFrameDecode<P0>(
        &self,
        piframedecoder: P0,
    ) -> windows_core::Result<IWICFastMetadataEncoder>
    where
        P0: windows_core::Param<IWICBitmapFrameDecode>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateFastMetadataEncoderFromFrameDecode)(
                windows_core::Interface::as_raw(self),
                piframedecoder.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateQueryWriter(
        &self,
        guidmetadataformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICMetadataQueryWriter> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateQueryWriter)(
                windows_core::Interface::as_raw(self),
                guidmetadataformat,
                pguidvendor,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
    pub(crate) unsafe fn CreateQueryWriterFromReader<P0>(
        &self,
        piqueryreader: P0,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICMetadataQueryWriter>
    where
        P0: windows_core::Param<IWICMetadataQueryReader>,
    {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).CreateQueryWriterFromReader)(
                windows_core::Interface::as_raw(self),
                piqueryreader.param().abi(),
                pguidvendor,
                &mut result__,
            )
            .and_then(|| windows_core::imp::Type::from_abi(result__))
        }
    }
}
#[repr(C)]
pub struct IWICImagingFactory_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    pub CreateDecoderFromFilename: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        windows_core::PCWSTR,
        *const windows_core::GUID,
        u32,
        WICDecodeOptions,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDecoderFromStream: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        WICDecodeOptions,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDecoderFromFileHandle: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        usize,
        *const windows_core::GUID,
        WICDecodeOptions,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateComponentInfo: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateDecoder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateEncoder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreatePalette: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFormatConverter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapScaler: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapClipper: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFlipRotator: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateStream: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorContext: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateColorTransformer: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmap: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        REFWICPixelFormatGUID,
        WICBitmapCreateCacheOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromSource: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        WICBitmapCreateCacheOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromSourceRect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        u32,
        u32,
        u32,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromMemory: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        REFWICPixelFormatGUID,
        u32,
        u32,
        *const u8,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromHBITMAP: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HBITMAP,
        HPALETTE,
        WICBitmapAlphaChannelOption,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateBitmapFromHICON: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        HICON,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateComponentEnumerator: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        u32,
        u32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateFastMetadataEncoderFromDecoder: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    )
        -> windows_core::HRESULT,
    pub CreateFastMetadataEncoderFromFrameDecode:
        unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *mut core::ffi::c_void,
            *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT,
    pub CreateQueryWriter: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub CreateQueryWriterFromReader: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *const windows_core::GUID,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
pub trait IWICImagingFactory_Impl: windows_core::IUnknownImpl {
    fn CreateDecoderFromFilename(
        &self,
        wzfilename: &windows_core::PCWSTR,
        pguidvendor: *const windows_core::GUID,
        dwdesiredaccess: u32,
        metadataoptions: WICDecodeOptions,
    ) -> windows_core::Result<IWICBitmapDecoder>;
    fn CreateDecoderFromStream(
        &self,
        pistream: windows_core::Ref<IStream>,
        pguidvendor: *const windows_core::GUID,
        metadataoptions: WICDecodeOptions,
    ) -> windows_core::Result<IWICBitmapDecoder>;
    fn CreateDecoderFromFileHandle(
        &self,
        hfile: usize,
        pguidvendor: *const windows_core::GUID,
        metadataoptions: WICDecodeOptions,
    ) -> windows_core::Result<IWICBitmapDecoder>;
    fn CreateComponentInfo(
        &self,
        clsidcomponent: *const windows_core::GUID,
    ) -> windows_core::Result<IWICComponentInfo>;
    fn CreateDecoder(
        &self,
        guidcontainerformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICBitmapDecoder>;
    fn CreateEncoder(
        &self,
        guidcontainerformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICBitmapEncoder>;
    fn CreatePalette(&self) -> windows_core::Result<IWICPalette>;
    fn CreateFormatConverter(&self) -> windows_core::Result<IWICFormatConverter>;
    fn CreateBitmapScaler(&self) -> windows_core::Result<IWICBitmapScaler>;
    fn CreateBitmapClipper(&self) -> windows_core::Result<IWICBitmapClipper>;
    fn CreateBitmapFlipRotator(&self) -> windows_core::Result<IWICBitmapFlipRotator>;
    fn CreateStream(&self) -> windows_core::Result<IWICStream>;
    fn CreateColorContext(&self) -> windows_core::Result<IWICColorContext>;
    fn CreateColorTransformer(&self) -> windows_core::Result<IWICColorTransform>;
    fn CreateBitmap(
        &self,
        uiwidth: u32,
        uiheight: u32,
        pixelformat: REFWICPixelFormatGUID,
        option: WICBitmapCreateCacheOption,
    ) -> windows_core::Result<IWICBitmap>;
    fn CreateBitmapFromSource(
        &self,
        pibitmapsource: windows_core::Ref<IWICBitmapSource>,
        option: WICBitmapCreateCacheOption,
    ) -> windows_core::Result<IWICBitmap>;
    fn CreateBitmapFromSourceRect(
        &self,
        pibitmapsource: windows_core::Ref<IWICBitmapSource>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> windows_core::Result<IWICBitmap>;
    fn CreateBitmapFromMemory(
        &self,
        uiwidth: u32,
        uiheight: u32,
        pixelformat: REFWICPixelFormatGUID,
        cbstride: u32,
        cbbuffersize: u32,
        pbbuffer: *const u8,
    ) -> windows_core::Result<IWICBitmap>;
    fn CreateBitmapFromHBITMAP(
        &self,
        hbitmap: HBITMAP,
        hpalette: HPALETTE,
        options: WICBitmapAlphaChannelOption,
    ) -> windows_core::Result<IWICBitmap>;
    fn CreateBitmapFromHICON(&self, hicon: HICON) -> windows_core::Result<IWICBitmap>;
    fn CreateComponentEnumerator(
        &self,
        componenttypes: u32,
        options: u32,
    ) -> windows_core::Result<IEnumUnknown>;
    fn CreateFastMetadataEncoderFromDecoder(
        &self,
        pidecoder: windows_core::Ref<IWICBitmapDecoder>,
    ) -> windows_core::Result<IWICFastMetadataEncoder>;
    fn CreateFastMetadataEncoderFromFrameDecode(
        &self,
        piframedecoder: windows_core::Ref<IWICBitmapFrameDecode>,
    ) -> windows_core::Result<IWICFastMetadataEncoder>;
    fn CreateQueryWriter(
        &self,
        guidmetadataformat: *const windows_core::GUID,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICMetadataQueryWriter>;
    fn CreateQueryWriterFromReader(
        &self,
        piqueryreader: windows_core::Ref<IWICMetadataQueryReader>,
        pguidvendor: *const windows_core::GUID,
    ) -> windows_core::Result<IWICMetadataQueryWriter>;
}
impl IWICImagingFactory_Vtbl {
    pub const fn new<Identity: IWICImagingFactory_Impl, const OFFSET: isize>() -> Self {
        unsafe extern "system" fn CreateDecoderFromFilename<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            wzfilename: windows_core::PCWSTR,
            pguidvendor: *const windows_core::GUID,
            dwdesiredaccess: u32,
            metadataoptions: WICDecodeOptions,
            ppidecoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateDecoderFromFilename(
                    this,
                    core::mem::transmute(&wzfilename),
                    core::mem::transmute_copy(&pguidvendor),
                    core::mem::transmute_copy(&dwdesiredaccess),
                    core::mem::transmute_copy(&metadataoptions),
                ) {
                    Ok(ok__) => {
                        ppidecoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateDecoderFromStream<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pistream: *mut core::ffi::c_void,
            pguidvendor: *const windows_core::GUID,
            metadataoptions: WICDecodeOptions,
            ppidecoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateDecoderFromStream(
                    this,
                    core::mem::transmute_copy(&pistream),
                    core::mem::transmute_copy(&pguidvendor),
                    core::mem::transmute_copy(&metadataoptions),
                ) {
                    Ok(ok__) => {
                        ppidecoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateDecoderFromFileHandle<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hfile: usize,
            pguidvendor: *const windows_core::GUID,
            metadataoptions: WICDecodeOptions,
            ppidecoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateDecoderFromFileHandle(
                    this,
                    core::mem::transmute_copy(&hfile),
                    core::mem::transmute_copy(&pguidvendor),
                    core::mem::transmute_copy(&metadataoptions),
                ) {
                    Ok(ok__) => {
                        ppidecoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateComponentInfo<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            clsidcomponent: *const windows_core::GUID,
            ppiinfo: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateComponentInfo(
                    this,
                    core::mem::transmute_copy(&clsidcomponent),
                ) {
                    Ok(ok__) => {
                        ppiinfo.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateDecoder<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            guidcontainerformat: *const windows_core::GUID,
            pguidvendor: *const windows_core::GUID,
            ppidecoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateDecoder(
                    this,
                    core::mem::transmute_copy(&guidcontainerformat),
                    core::mem::transmute_copy(&pguidvendor),
                ) {
                    Ok(ok__) => {
                        ppidecoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateEncoder<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            guidcontainerformat: *const windows_core::GUID,
            pguidvendor: *const windows_core::GUID,
            ppiencoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateEncoder(
                    this,
                    core::mem::transmute_copy(&guidcontainerformat),
                    core::mem::transmute_copy(&pguidvendor),
                ) {
                    Ok(ok__) => {
                        ppiencoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreatePalette<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppipalette: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreatePalette(this) {
                    Ok(ok__) => {
                        ppipalette.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateFormatConverter<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppiformatconverter: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateFormatConverter(this) {
                    Ok(ok__) => {
                        ppiformatconverter.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapScaler<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppibitmapscaler: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapScaler(this) {
                    Ok(ok__) => {
                        ppibitmapscaler.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapClipper<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppibitmapclipper: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapClipper(this) {
                    Ok(ok__) => {
                        ppibitmapclipper.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapFlipRotator<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppibitmapfliprotator: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapFlipRotator(this) {
                    Ok(ok__) => {
                        ppibitmapfliprotator.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateStream<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppiwicstream: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateStream(this) {
                    Ok(ok__) => {
                        ppiwicstream.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateColorContext<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppiwiccolorcontext: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateColorContext(this) {
                    Ok(ok__) => {
                        ppiwiccolorcontext.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateColorTransformer<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            ppiwiccolortransform: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateColorTransformer(this) {
                    Ok(ok__) => {
                        ppiwiccolortransform.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmap<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            uiwidth: u32,
            uiheight: u32,
            pixelformat: REFWICPixelFormatGUID,
            option: WICBitmapCreateCacheOption,
            ppibitmap: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmap(
                    this,
                    core::mem::transmute_copy(&uiwidth),
                    core::mem::transmute_copy(&uiheight),
                    core::mem::transmute_copy(&pixelformat),
                    core::mem::transmute_copy(&option),
                ) {
                    Ok(ok__) => {
                        ppibitmap.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapFromSource<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pibitmapsource: *mut core::ffi::c_void,
            option: WICBitmapCreateCacheOption,
            ppibitmap: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapFromSource(
                    this,
                    core::mem::transmute_copy(&pibitmapsource),
                    core::mem::transmute_copy(&option),
                ) {
                    Ok(ok__) => {
                        ppibitmap.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapFromSourceRect<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pibitmapsource: *mut core::ffi::c_void,
            x: u32,
            y: u32,
            width: u32,
            height: u32,
            ppibitmap: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapFromSourceRect(
                    this,
                    core::mem::transmute_copy(&pibitmapsource),
                    core::mem::transmute_copy(&x),
                    core::mem::transmute_copy(&y),
                    core::mem::transmute_copy(&width),
                    core::mem::transmute_copy(&height),
                ) {
                    Ok(ok__) => {
                        ppibitmap.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapFromMemory<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            uiwidth: u32,
            uiheight: u32,
            pixelformat: REFWICPixelFormatGUID,
            cbstride: u32,
            cbbuffersize: u32,
            pbbuffer: *const u8,
            ppibitmap: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapFromMemory(
                    this,
                    core::mem::transmute_copy(&uiwidth),
                    core::mem::transmute_copy(&uiheight),
                    core::mem::transmute_copy(&pixelformat),
                    core::mem::transmute_copy(&cbstride),
                    core::mem::transmute_copy(&cbbuffersize),
                    core::mem::transmute_copy(&pbbuffer),
                ) {
                    Ok(ok__) => {
                        ppibitmap.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapFromHBITMAP<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hbitmap: HBITMAP,
            hpalette: HPALETTE,
            options: WICBitmapAlphaChannelOption,
            ppibitmap: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapFromHBITMAP(
                    this,
                    core::mem::transmute_copy(&hbitmap),
                    core::mem::transmute_copy(&hpalette),
                    core::mem::transmute_copy(&options),
                ) {
                    Ok(ok__) => {
                        ppibitmap.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateBitmapFromHICON<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            hicon: HICON,
            ppibitmap: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateBitmapFromHICON(
                    this,
                    core::mem::transmute_copy(&hicon),
                ) {
                    Ok(ok__) => {
                        ppibitmap.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateComponentEnumerator<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            componenttypes: u32,
            options: u32,
            ppienumunknown: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateComponentEnumerator(
                    this,
                    core::mem::transmute_copy(&componenttypes),
                    core::mem::transmute_copy(&options),
                ) {
                    Ok(ok__) => {
                        ppienumunknown.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateFastMetadataEncoderFromDecoder<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            pidecoder: *mut core::ffi::c_void,
            ppifastencoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateFastMetadataEncoderFromDecoder(
                    this,
                    core::mem::transmute_copy(&pidecoder),
                ) {
                    Ok(ok__) => {
                        ppifastencoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateFastMetadataEncoderFromFrameDecode<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            piframedecoder: *mut core::ffi::c_void,
            ppifastencoder: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateFastMetadataEncoderFromFrameDecode(
                    this,
                    core::mem::transmute_copy(&piframedecoder),
                ) {
                    Ok(ok__) => {
                        ppifastencoder.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateQueryWriter<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            guidmetadataformat: *const windows_core::GUID,
            pguidvendor: *const windows_core::GUID,
            ppiquerywriter: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateQueryWriter(
                    this,
                    core::mem::transmute_copy(&guidmetadataformat),
                    core::mem::transmute_copy(&pguidvendor),
                ) {
                    Ok(ok__) => {
                        ppiquerywriter.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        unsafe extern "system" fn CreateQueryWriterFromReader<
            Identity: IWICImagingFactory_Impl,
            const OFFSET: isize,
        >(
            this: *mut core::ffi::c_void,
            piqueryreader: *mut core::ffi::c_void,
            pguidvendor: *const windows_core::GUID,
            ppiquerywriter: *mut *mut core::ffi::c_void,
        ) -> windows_core::HRESULT {
            unsafe {
                let this: &Identity =
                    &*((this as *const *const ()).offset(OFFSET) as *const Identity);
                match IWICImagingFactory_Impl::CreateQueryWriterFromReader(
                    this,
                    core::mem::transmute_copy(&piqueryreader),
                    core::mem::transmute_copy(&pguidvendor),
                ) {
                    Ok(ok__) => {
                        ppiquerywriter.write(core::mem::transmute(ok__));
                        windows_core::HRESULT(0)
                    }
                    Err(err) => err.into(),
                }
            }
        }
        Self {
            base__: windows_core::IUnknown_Vtbl::new::<Identity, OFFSET>(),
            CreateDecoderFromFilename: CreateDecoderFromFilename::<Identity, OFFSET>,
            CreateDecoderFromStream: CreateDecoderFromStream::<Identity, OFFSET>,
            CreateDecoderFromFileHandle: CreateDecoderFromFileHandle::<Identity, OFFSET>,
            CreateComponentInfo: CreateComponentInfo::<Identity, OFFSET>,
            CreateDecoder: CreateDecoder::<Identity, OFFSET>,
            CreateEncoder: CreateEncoder::<Identity, OFFSET>,
            CreatePalette: CreatePalette::<Identity, OFFSET>,
            CreateFormatConverter: CreateFormatConverter::<Identity, OFFSET>,
            CreateBitmapScaler: CreateBitmapScaler::<Identity, OFFSET>,
            CreateBitmapClipper: CreateBitmapClipper::<Identity, OFFSET>,
            CreateBitmapFlipRotator: CreateBitmapFlipRotator::<Identity, OFFSET>,
            CreateStream: CreateStream::<Identity, OFFSET>,
            CreateColorContext: CreateColorContext::<Identity, OFFSET>,
            CreateColorTransformer: CreateColorTransformer::<Identity, OFFSET>,
            CreateBitmap: CreateBitmap::<Identity, OFFSET>,
            CreateBitmapFromSource: CreateBitmapFromSource::<Identity, OFFSET>,
            CreateBitmapFromSourceRect: CreateBitmapFromSourceRect::<Identity, OFFSET>,
            CreateBitmapFromMemory: CreateBitmapFromMemory::<Identity, OFFSET>,
            CreateBitmapFromHBITMAP: CreateBitmapFromHBITMAP::<Identity, OFFSET>,
            CreateBitmapFromHICON: CreateBitmapFromHICON::<Identity, OFFSET>,
            CreateComponentEnumerator: CreateComponentEnumerator::<Identity, OFFSET>,
            CreateFastMetadataEncoderFromDecoder: CreateFastMetadataEncoderFromDecoder::<
                Identity,
                OFFSET,
            >,
            CreateFastMetadataEncoderFromFrameDecode: CreateFastMetadataEncoderFromFrameDecode::<
                Identity,
                OFFSET,
            >,
            CreateQueryWriter: CreateQueryWriter::<Identity, OFFSET>,
            CreateQueryWriterFromReader: CreateQueryWriterFromReader::<Identity, OFFSET>,
        }
    }
    pub fn matches(iid: &windows_core::GUID) -> bool {
        iid == &<IWICImagingFactory as windows_core::Interface>::IID
    }
}
impl windows_core::RuntimeName for IWICImagingFactory {}
windows_core::imp::define_interface!(
    IWICMetadataQueryReader,
    IWICMetadataQueryReader_Vtbl,
    0x30989668_e1c9_4597_b395_458eedb808df
);
windows_core::imp::interface_hierarchy!(IWICMetadataQueryReader, windows_core::IUnknown);
#[repr(C)]
pub struct IWICMetadataQueryReader_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    GetContainerFormat: usize,
    GetLocation: usize,
    GetMetadataByName: usize,
    GetEnumerator: usize,
}
impl windows_core::RuntimeName for IWICMetadataQueryReader {}
windows_core::imp::define_interface!(
    IWICMetadataQueryWriter,
    IWICMetadataQueryWriter_Vtbl,
    0xa721791a_0def_4d06_bd91_2118bf1db10b
);
impl core::ops::Deref for IWICMetadataQueryWriter {
    type Target = IWICMetadataQueryReader;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICMetadataQueryWriter,
    windows_core::IUnknown,
    IWICMetadataQueryReader
);
#[repr(C)]
pub struct IWICMetadataQueryWriter_Vtbl {
    pub base__: IWICMetadataQueryReader_Vtbl,
    SetMetadataByName: usize,
    RemoveMetadataByName: usize,
}
impl windows_core::RuntimeName for IWICMetadataQueryWriter {}
windows_core::imp::define_interface!(
    IWICPalette,
    IWICPalette_Vtbl,
    0x00000040_a8f2_4877_ba0a_fd2b6645fb94
);
windows_core::imp::interface_hierarchy!(IWICPalette, windows_core::IUnknown);
#[repr(C)]
pub struct IWICPalette_Vtbl {
    pub base__: windows_core::IUnknown_Vtbl,
    InitializePredefined: usize,
    InitializeCustom: usize,
    InitializeFromBitmap: usize,
    InitializeFromPalette: usize,
    GetType: usize,
    GetColorCount: usize,
    GetColors: usize,
    IsBlackWhite: usize,
    IsGrayscale: usize,
    HasAlpha: usize,
}
impl windows_core::RuntimeName for IWICPalette {}
windows_core::imp::define_interface!(
    IWICStream,
    IWICStream_Vtbl,
    0x135ff860_22b7_4ddf_b0f6_218f4f299a43
);
impl core::ops::Deref for IWICStream {
    type Target = IStream;
    fn deref(&self) -> &Self::Target {
        unsafe { core::mem::transmute(self) }
    }
}
windows_core::imp::interface_hierarchy!(
    IWICStream,
    windows_core::IUnknown,
    ISequentialStream,
    IStream
);
#[repr(C)]
pub struct IWICStream_Vtbl {
    pub base__: IStream_Vtbl,
    InitializeFromIStream: usize,
    InitializeFromFilename: usize,
    InitializeFromMemory: usize,
    InitializeFromIStreamRegion: usize,
}
impl windows_core::RuntimeName for IWICStream {}
pub type JOBOBJECTINFOCLASS = i32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
    pub PerProcessUserTimeLimit: i64,
    pub PerJobUserTimeLimit: i64,
    pub LimitFlags: u32,
    pub MinimumWorkingSetSize: usize,
    pub MaximumWorkingSetSize: usize,
    pub ActiveProcessLimit: u32,
    pub Affinity: usize,
    pub PriorityClass: u32,
    pub SchedulingClass: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
    pub BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION,
    pub IoInfo: IO_COUNTERS,
    pub ProcessMemoryLimit: usize,
    pub JobMemoryLimit: usize,
    pub PeakProcessMemoryUsed: usize,
    pub PeakJobMemoryUsed: usize,
}
pub const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: i32 = 8192;
pub const JobObjectExtendedLimitInformation: JOBOBJECTINFOCLASS = 9;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct KAFFINITY(pub usize);
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KDHELP64 {
    pub Thread: u64,
    pub ThCallbackStack: u32,
    pub ThCallbackBStore: u32,
    pub NextCallback: u32,
    pub FramePointer: u32,
    pub KiCallUserMode: u64,
    pub KeUserCallbackDispatcher: u64,
    pub SystemRangeStart: u64,
    pub KiUserExceptionDispatcher: u64,
    pub StackBase: u64,
    pub StackLimit: u64,
    pub BuildVersion: u32,
    pub RetpolineStubFunctionTableSize: u32,
    pub RetpolineStubFunctionTable: u64,
    pub RetpolineStubOffset: u32,
    pub RetpolineStubSize: u32,
    pub Reserved0: [u64; 2],
}
impl Default for KDHELP64 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const KEYEVENTF_KEYUP: i32 = 2;
pub const KEY_NOTIFY: i32 = 16;
pub const KEY_QUERY_VALUE: i32 = 1;
pub const KEY_READ: i32 = 131097;
pub const KEY_SET_VALUE: i32 = 2;
pub const KF_FLAG_DEFAULT: KNOWN_FOLDER_FLAG = 0;
pub type KNOWNFOLDERID = windows_core::GUID;
pub type KNOWN_FOLDER_FLAG = u32;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LOGFONTW {
    pub lfHeight: i32,
    pub lfWidth: i32,
    pub lfEscapement: i32,
    pub lfOrientation: i32,
    pub lfWeight: i32,
    pub lfItalic: u8,
    pub lfUnderline: u8,
    pub lfStrikeOut: u8,
    pub lfCharSet: u8,
    pub lfOutPrecision: u8,
    pub lfClipPrecision: u8,
    pub lfQuality: u8,
    pub lfPitchAndFamily: u8,
    pub lfFaceName: [u16; 32],
}
impl Default for LOGFONTW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LPARAM(pub isize);
pub type LPBYTE = *mut u8;
pub type LPCITEMIDLIST = *const ITEMIDLIST;
pub type LPITEMIDLIST = *mut ITEMIDLIST;
pub type LPOVERLAPPED_COMPLETION_ROUTINE = Option<
    unsafe extern "system" fn(
        dwerrorcode: u32,
        dwnumberofbytestransfered: u32,
        lpoverlapped: *mut OVERLAPPED,
    ),
>;
pub type LPTOP_LEVEL_EXCEPTION_FILTER = PTOP_LEVEL_EXCEPTION_FILTER;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LRESULT(pub isize);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct LSTATUS(pub i32);
pub const LWA_ALPHA: i32 = 2;
pub const LWA_COLORKEY: i32 = 1;
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct M128A {
    pub Low: u64,
    pub High: i64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MARGINS {
    pub cxLeftWidth: i32,
    pub cxRightWidth: i32,
    pub cyTopHeight: i32,
    pub cyBottomHeight: i32,
}
pub const MA_ACTIVATE: i32 = 1;
pub const MA_NOACTIVATE: i32 = 3;
pub const MA_NOACTIVATEANDEAT: i32 = 4;
pub const MB_ICONERROR: i32 = 16;
pub const MB_OK: i32 = 0;
pub const MDT_EFFECTIVE_DPI: MONITOR_DPI_TYPE = 0;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MENUITEMINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub fType: u32,
    pub fState: u32,
    pub wID: u32,
    pub hSubMenu: HMENU,
    pub hbmpChecked: HBITMAP,
    pub hbmpUnchecked: HBITMAP,
    pub dwItemData: usize,
    pub dwTypeData: windows_core::PWSTR,
    pub cch: u32,
    pub hbmpItem: HBITMAP,
}
pub const MFS_CHECKED: i32 = 8;
pub const MFS_DEFAULT: i32 = 4096;
pub const MFT_SEPARATOR: i32 = 2048;
pub const MFT_STRING: i32 = 0;
pub const MF_BYCOMMAND: i32 = 0;
pub const MF_BYPOSITION: i32 = 1024;
pub const MF_CHECKED: i32 = 8;
pub const MF_DISABLED: i32 = 2;
pub const MF_GRAYED: i32 = 1;
pub const MF_POPUP: i32 = 16;
pub const MF_SEPARATOR: i32 = 2048;
pub const MF_STRING: i32 = 0;
pub const MF_UNCHECKED: i32 = 0;
pub const MIIM_FTYPE: i32 = 256;
pub const MIIM_ID: i32 = 2;
pub const MIIM_STATE: i32 = 1;
pub const MIIM_STRING: i32 = 64;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default)]
pub struct MINIDUMP_CALLBACK_INFORMATION {
    pub CallbackRoutine: MINIDUMP_CALLBACK_ROUTINE,
    pub CallbackParam: *mut core::ffi::c_void,
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_INFORMATION {
    pub CallbackRoutine: MINIDUMP_CALLBACK_ROUTINE,
    pub CallbackParam: *mut core::ffi::c_void,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MINIDUMP_CALLBACK_INPUT {
    pub ProcessId: u32,
    pub ProcessHandle: HANDLE,
    pub CallbackType: u32,
    pub Anonymous: MINIDUMP_CALLBACK_INPUT_0,
}
impl Default for MINIDUMP_CALLBACK_INPUT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union MINIDUMP_CALLBACK_INPUT_0 {
    pub Status: windows_core::HRESULT,
    pub Thread: MINIDUMP_THREAD_CALLBACK,
    pub ThreadEx: MINIDUMP_THREAD_EX_CALLBACK,
    pub Module: MINIDUMP_MODULE_CALLBACK,
    pub IncludeThread: MINIDUMP_INCLUDE_THREAD_CALLBACK,
    pub IncludeModule: MINIDUMP_INCLUDE_MODULE_CALLBACK,
    pub Io: MINIDUMP_IO_CALLBACK,
    pub ReadMemoryFailure: MINIDUMP_READ_MEMORY_FAILURE_CALLBACK,
    pub SecondaryFlags: u32,
    pub VmQuery: MINIDUMP_VM_QUERY_CALLBACK,
    pub VmPreRead: MINIDUMP_VM_PRE_READ_CALLBACK,
    pub VmPostRead: MINIDUMP_VM_POST_READ_CALLBACK,
    pub CompressedMemoryStreamFinish: MINIDUMP_COMPRESSED_MEMORY_STREAM_FINISH_CALLBACK,
}
impl Default for MINIDUMP_CALLBACK_INPUT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct MINIDUMP_CALLBACK_OUTPUT {
    pub Anonymous: MINIDUMP_CALLBACK_OUTPUT_0,
}
#[cfg(target_arch = "x86")]
impl Default for MINIDUMP_CALLBACK_OUTPUT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub union MINIDUMP_CALLBACK_OUTPUT_0 {
    pub ModuleWriteFlags: u32,
    pub ThreadWriteFlags: u32,
    pub SecondaryFlags: u32,
    pub Anonymous: MINIDUMP_CALLBACK_OUTPUT_0_0,
    pub Anonymous2: MINIDUMP_CALLBACK_OUTPUT_0_1,
    pub Handle: HANDLE,
    pub Anonymous3: MINIDUMP_CALLBACK_OUTPUT_0_2,
    pub Anonymous4: MINIDUMP_CALLBACK_OUTPUT_0_3,
    pub Anonymous5: MINIDUMP_CALLBACK_OUTPUT_0_4,
    pub CompressedMemoryStreamStart: MINIDUMP_COMPRESSED_MEMORY_STREAM_START_CALLBACK,
    pub Status: windows_core::HRESULT,
}
#[cfg(target_arch = "x86")]
impl Default for MINIDUMP_CALLBACK_OUTPUT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(4))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_0 {
    pub MemoryBase: u64,
    pub MemorySize: u32,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_1 {
    pub CheckCancel: windows_core::BOOL,
    pub Cancel: windows_core::BOOL,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_2 {
    pub VmRegion: MINIDUMP_MEMORY_INFO,
    pub Continue: windows_core::BOOL,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_3 {
    pub VmQueryStatus: windows_core::HRESULT,
    pub VmQueryResult: MINIDUMP_MEMORY_INFO,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_4 {
    pub VmReadStatus: windows_core::HRESULT,
    pub VmReadBytesCompleted: u32,
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy)]
pub struct MINIDUMP_CALLBACK_OUTPUT {
    pub Anonymous: MINIDUMP_CALLBACK_OUTPUT_0,
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for MINIDUMP_CALLBACK_OUTPUT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy)]
pub union MINIDUMP_CALLBACK_OUTPUT_0 {
    pub ModuleWriteFlags: u32,
    pub ThreadWriteFlags: u32,
    pub SecondaryFlags: u32,
    pub Anonymous: MINIDUMP_CALLBACK_OUTPUT_0_0,
    pub Anonymous2: MINIDUMP_CALLBACK_OUTPUT_0_1,
    pub Handle: HANDLE,
    pub Anonymous3: MINIDUMP_CALLBACK_OUTPUT_0_2,
    pub Anonymous4: MINIDUMP_CALLBACK_OUTPUT_0_3,
    pub Anonymous5: MINIDUMP_CALLBACK_OUTPUT_0_4,
    pub CompressedMemoryStreamStart: MINIDUMP_COMPRESSED_MEMORY_STREAM_START_CALLBACK,
    pub Status: windows_core::HRESULT,
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for MINIDUMP_CALLBACK_OUTPUT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_0 {
    pub MemoryBase: u64,
    pub MemorySize: u32,
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_1 {
    pub CheckCancel: windows_core::BOOL,
    pub Cancel: windows_core::BOOL,
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_2 {
    pub VmRegion: MINIDUMP_MEMORY_INFO,
    pub Continue: windows_core::BOOL,
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_3 {
    pub VmQueryStatus: windows_core::HRESULT,
    pub VmQueryResult: MINIDUMP_MEMORY_INFO,
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_CALLBACK_OUTPUT_0_4 {
    pub VmReadStatus: windows_core::HRESULT,
    pub VmReadBytesCompleted: u32,
}
pub type MINIDUMP_CALLBACK_ROUTINE = Option<
    unsafe extern "system" fn(
        callbackparam: *mut core::ffi::c_void,
        callbackinput: *const MINIDUMP_CALLBACK_INPUT,
        callbackoutput: *mut MINIDUMP_CALLBACK_OUTPUT,
    ) -> windows_core::BOOL,
>;
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_COMPRESSED_MEMORY_STREAM_FINISH_CALLBACK {
    pub CompressedStreamSize: u64,
    pub StreamCompressionRate: u32,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_COMPRESSED_MEMORY_STREAM_START_CALLBACK {
    pub MaxParallelism: u32,
    pub MaxTransferSize: u32,
    pub NumaNodeInfoArray: PNUMA_NODE_RELATIONSHIP,
    pub NumaNodeInfoArraySize: u32,
    pub Reserved: u32,
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_COMPRESSED_MEMORY_STREAM_START_CALLBACK {
    pub MaxParallelism: u32,
    pub MaxTransferSize: u32,
    pub NumaNodeInfoArray: PNUMA_NODE_RELATIONSHIP,
    pub NumaNodeInfoArraySize: u32,
    pub Reserved: u32,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_EXCEPTION_INFORMATION {
    pub ThreadId: u32,
    pub ExceptionPointers: PEXCEPTION_POINTERS,
    pub ClientPointers: windows_core::BOOL,
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_EXCEPTION_INFORMATION {
    pub ThreadId: u32,
    pub ExceptionPointers: PEXCEPTION_POINTERS,
    pub ClientPointers: windows_core::BOOL,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_INCLUDE_MODULE_CALLBACK {
    pub BaseOfImage: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_INCLUDE_THREAD_CALLBACK {
    pub ThreadId: u32,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_IO_CALLBACK {
    pub Handle: HANDLE,
    pub Offset: u64,
    pub Buffer: *mut core::ffi::c_void,
    pub BufferBytes: u32,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_MEMORY_INFO {
    pub BaseAddress: u64,
    pub AllocationBase: u64,
    pub AllocationProtect: u32,
    pub __alignment1: u32,
    pub RegionSize: u64,
    pub State: u32,
    pub Protect: u32,
    pub Type: u32,
    pub __alignment2: u32,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_MODULE_CALLBACK {
    pub FullPath: PWCHAR,
    pub BaseOfImage: u64,
    pub SizeOfImage: u32,
    pub CheckSum: u32,
    pub TimeDateStamp: u32,
    pub VersionInfo: VS_FIXEDFILEINFO,
    pub CvRecord: *mut core::ffi::c_void,
    pub SizeOfCvRecord: u32,
    pub MiscRecord: *mut core::ffi::c_void,
    pub SizeOfMiscRecord: u32,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_READ_MEMORY_FAILURE_CALLBACK {
    pub Offset: u64,
    pub Bytes: u32,
    pub FailureStatus: windows_core::HRESULT,
}
#[repr(C, packed(4))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct MINIDUMP_THREAD_CALLBACK {
    pub ThreadId: u32,
    pub ThreadHandle: HANDLE,
    pub Context: CONTEXT,
    pub SizeOfContext: u32,
    pub StackBase: u64,
    pub StackEnd: u64,
}
#[cfg(target_arch = "x86")]
impl Default for MINIDUMP_THREAD_CALLBACK {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct MINIDUMP_THREAD_CALLBACK {
    pub ThreadId: u32,
    pub ThreadHandle: HANDLE,
    pub Context: CONTEXT,
    pub SizeOfContext: u32,
    pub StackBase: u64,
    pub StackEnd: u64,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for MINIDUMP_THREAD_CALLBACK {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub struct MINIDUMP_THREAD_CALLBACK {
    pub ThreadId: u32,
    pub ThreadHandle: HANDLE,
    pub Pad: u32,
    pub Context: CONTEXT,
    pub SizeOfContext: u32,
    pub StackBase: u64,
    pub StackEnd: u64,
}
#[cfg(target_arch = "aarch64")]
impl Default for MINIDUMP_THREAD_CALLBACK {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(4))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct MINIDUMP_THREAD_EX_CALLBACK {
    pub ThreadId: u32,
    pub ThreadHandle: HANDLE,
    pub Context: CONTEXT,
    pub SizeOfContext: u32,
    pub StackBase: u64,
    pub StackEnd: u64,
    pub BackingStoreBase: u64,
    pub BackingStoreEnd: u64,
}
#[cfg(target_arch = "x86")]
impl Default for MINIDUMP_THREAD_EX_CALLBACK {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
#[derive(Clone, Copy)]
pub struct MINIDUMP_THREAD_EX_CALLBACK {
    pub ThreadId: u32,
    pub ThreadHandle: HANDLE,
    pub Context: CONTEXT,
    pub SizeOfContext: u32,
    pub StackBase: u64,
    pub StackEnd: u64,
    pub BackingStoreBase: u64,
    pub BackingStoreEnd: u64,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
impl Default for MINIDUMP_THREAD_EX_CALLBACK {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(target_arch = "aarch64")]
#[derive(Clone, Copy)]
pub struct MINIDUMP_THREAD_EX_CALLBACK {
    pub ThreadId: u32,
    pub ThreadHandle: HANDLE,
    pub Pad: u32,
    pub Context: CONTEXT,
    pub SizeOfContext: u32,
    pub StackBase: u64,
    pub StackEnd: u64,
    pub BackingStoreBase: u64,
    pub BackingStoreEnd: u64,
}
#[cfg(target_arch = "aarch64")]
impl Default for MINIDUMP_THREAD_EX_CALLBACK {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type MINIDUMP_TYPE = i32;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_USER_STREAM {
    pub Type: u32,
    pub BufferSize: u32,
    pub Buffer: *mut core::ffi::c_void,
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_USER_STREAM {
    pub Type: u32,
    pub BufferSize: u32,
    pub Buffer: *mut core::ffi::c_void,
}
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINIDUMP_USER_STREAM_INFORMATION {
    pub UserStreamCount: u32,
    pub UserStreamArray: PMINIDUMP_USER_STREAM,
}
#[repr(C, packed(4))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_USER_STREAM_INFORMATION {
    pub UserStreamCount: u32,
    pub UserStreamArray: PMINIDUMP_USER_STREAM,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_VM_POST_READ_CALLBACK {
    pub Offset: u64,
    pub Buffer: *mut core::ffi::c_void,
    pub Size: u32,
    pub Completed: u32,
    pub Status: windows_core::HRESULT,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_VM_PRE_READ_CALLBACK {
    pub Offset: u64,
    pub Buffer: *mut core::ffi::c_void,
    pub Size: u32,
}
#[repr(C, packed(4))]
#[derive(Clone, Copy, Default)]
pub struct MINIDUMP_VM_QUERY_CALLBACK {
    pub Offset: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MINMAXINFO {
    pub ptReserved: POINT,
    pub ptMaxSize: POINT,
    pub ptMaxPosition: POINT,
    pub ptMinTrackSize: POINT,
    pub ptMaxTrackSize: POINT,
}
pub const MK_CONTROL: i32 = 8;
pub const MK_LBUTTON: i32 = 1;
pub const MK_SHIFT: i32 = 4;
pub const MOD_ALT: i32 = 1;
pub const MOD_CONTROL: i32 = 2;
pub const MOD_NOREPEAT: i32 = 16384;
pub const MOD_SHIFT: i32 = 4;
pub const MOD_WIN: i32 = 8;
pub type MONITORENUMPROC = Option<
    unsafe extern "system" fn(
        param0: HMONITOR,
        param1: HDC,
        param2: *mut RECT,
        param3: LPARAM,
    ) -> windows_core::BOOL,
>;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MONITORINFOEXW {
    pub Base: MONITORINFO,
    pub szDevice: [u16; 32],
}
impl Default for MONITORINFOEXW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const MONITORINFOF_PRIMARY: i32 = 1;
pub const MONITOR_DEFAULTTONEAREST: i32 = 2;
pub const MONITOR_DEFAULTTONULL: i32 = 0;
pub const MONITOR_DEFAULTTOPRIMARY: i32 = 1;
pub type MONITOR_DPI_TYPE = i32;
pub const MOUSEEVENTF_ABSOLUTE: i32 = 32768;
pub const MOUSEEVENTF_LEFTDOWN: i32 = 2;
pub const MOUSEEVENTF_LEFTUP: i32 = 4;
pub const MOUSEEVENTF_MOVE: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}
pub const MiniDumpScanMemory: MINIDUMP_TYPE = 16;
pub const MiniDumpWithDataSegs: MINIDUMP_TYPE = 1;
pub const MiniDumpWithIndirectlyReferencedMemory: MINIDUMP_TYPE = 64;
pub const MiniDumpWithThreadInfo: MINIDUMP_TYPE = 4096;
#[cfg(target_arch = "aarch64")]
pub type NEON128 = ARM64_NT_NEON128;
pub const NIF_ICON: i32 = 2;
pub const NIF_INFO: i32 = 16;
pub const NIF_MESSAGE: i32 = 1;
pub const NIF_SHOWTIP: i32 = 128;
pub const NIF_TIP: i32 = 4;
pub const NIIF_INFO: i32 = 1;
pub const NIIF_WARNING: i32 = 2;
pub const NIM_ADD: i32 = 0;
pub const NIM_DELETE: i32 = 2;
pub const NIM_MODIFY: i32 = 1;
pub const NIM_SETVERSION: i32 = 4;
pub const NIN_BALLOONUSERCLICK: i32 = 1029;
pub const NIN_KEYSELECT: i32 = 1025;
pub const NIN_SELECT: i32 = 1024;
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub Anonymous: NOTIFYICONDATAW_0,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: windows_core::GUID,
    pub hBalloonIcon: HICON,
}
#[cfg(target_arch = "x86")]
impl Default for NOTIFYICONDATAW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub union NOTIFYICONDATAW_0 {
    pub uTimeout: u32,
    pub uVersion: u32,
}
#[cfg(target_arch = "x86")]
impl Default for NOTIFYICONDATAW_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub Anonymous: NOTIFYICONDATAW_0,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: windows_core::GUID,
    pub hBalloonIcon: HICON,
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for NOTIFYICONDATAW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy)]
pub union NOTIFYICONDATAW_0 {
    pub uTimeout: u32,
    pub uVersion: u32,
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for NOTIFYICONDATAW_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const NOTIFYICON_VERSION_4: i32 = 4;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NUMA_NODE_RELATIONSHIP {
    pub NodeNumber: u32,
    pub Reserved: [u8; 18],
    pub GroupCount: u16,
    pub Anonymous: NUMA_NODE_RELATIONSHIP_0,
}
impl Default for NUMA_NODE_RELATIONSHIP {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union NUMA_NODE_RELATIONSHIP_0 {
    pub GroupMask: GROUP_AFFINITY,
    pub GroupMasks: [GROUP_AFFINITY; 1],
}
impl Default for NUMA_NODE_RELATIONSHIP_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const OPEN_EXISTING: i32 = 3;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OSVERSIONINFOW {
    pub dwOSVersionInfoSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u32,
    pub szCSDVersion: [u16; 128],
}
impl Default for OSVERSIONINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const OUT_DEFAULT_PRECIS: i32 = 0;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OVERLAPPED {
    pub Internal: usize,
    pub InternalHigh: usize,
    pub Anonymous: OVERLAPPED_0,
    pub hEvent: HANDLE,
}
impl Default for OVERLAPPED {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union OVERLAPPED_0 {
    pub Anonymous: OVERLAPPED_0_0,
    pub Pointer: *mut core::ffi::c_void,
}
impl Default for OVERLAPPED_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OVERLAPPED_0_0 {
    pub Offset: u32,
    pub OffsetHigh: u32,
}
#[cfg(any(target_arch = "arm64ec", target_arch = "x86", target_arch = "x86_64"))]
pub type PCONTEXT = *mut CONTEXT;
#[cfg(target_arch = "aarch64")]
pub type PCONTEXT = *mut ARM64_NT_CONTEXT;
pub type PEXCEPTION_POINTERS = *mut EXCEPTION_POINTERS;
pub type PEXCEPTION_RECORD = *mut EXCEPTION_RECORD;
pub type PFUNCTION_TABLE_ACCESS_ROUTINE64 =
    Option<unsafe extern "system" fn(ahprocess: HANDLE, addrbase: u64) -> *mut core::ffi::c_void>;
pub type PGET_MODULE_BASE_ROUTINE64 =
    Option<unsafe extern "system" fn(hprocess: HANDLE, address: u64) -> u64>;
pub const PIPE_ACCESS_DUPLEX: i32 = 3;
pub const PIPE_READMODE_BYTE: i32 = 0;
pub const PIPE_REJECT_REMOTE_CLIENTS: i32 = 8;
pub const PIPE_TYPE_BYTE: i32 = 0;
pub const PIPE_UNLIMITED_INSTANCES: i32 = 255;
pub const PIPE_WAIT: i32 = 0;
pub type PMINIDUMP_USER_STREAM = *mut MINIDUMP_USER_STREAM;
pub const PM_REMOVE: i32 = 1;
pub type PNUMA_NODE_RELATIONSHIP = *mut NUMA_NODE_RELATIONSHIP;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct POINTL {
    pub x: i32,
    pub y: i32,
}
pub type PREAD_PROCESS_MEMORY_ROUTINE64 = Option<
    unsafe extern "system" fn(
        hprocess: HANDLE,
        qwbaseaddress: u64,
        lpbuffer: *mut core::ffi::c_void,
        nsize: u32,
        lpnumberofbytesread: *mut u32,
    ) -> windows_core::BOOL,
>;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_INFORMATION {
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    pub dwProcessId: u32,
    pub dwThreadId: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_MEMORY_COUNTERS {
    pub cb: u32,
    pub PageFaultCount: u32,
    pub PeakWorkingSetSize: usize,
    pub WorkingSetSize: usize,
    pub QuotaPeakPagedPoolUsage: usize,
    pub QuotaPagedPoolUsage: usize,
    pub QuotaPeakNonPagedPoolUsage: usize,
    pub QuotaNonPagedPoolUsage: usize,
    pub PagefileUsage: usize,
    pub PeakPagefileUsage: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_MEMORY_COUNTERS_EX {
    pub cb: u32,
    pub PageFaultCount: u32,
    pub PeakWorkingSetSize: usize,
    pub WorkingSetSize: usize,
    pub QuotaPeakPagedPoolUsage: usize,
    pub QuotaPagedPoolUsage: usize,
    pub QuotaPeakNonPagedPoolUsage: usize,
    pub QuotaNonPagedPoolUsage: usize,
    pub PagefileUsage: usize,
    pub PeakPagefileUsage: usize,
    pub PrivateUsage: usize,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROCESS_MEMORY_COUNTERS_EX2 {
    pub cb: u32,
    pub PageFaultCount: u32,
    pub PeakWorkingSetSize: usize,
    pub WorkingSetSize: usize,
    pub QuotaPeakPagedPoolUsage: usize,
    pub QuotaPagedPoolUsage: usize,
    pub QuotaPeakNonPagedPoolUsage: usize,
    pub QuotaNonPagedPoolUsage: usize,
    pub PagefileUsage: usize,
    pub PeakPagefileUsage: usize,
    pub PrivateUsage: usize,
    pub PrivateWorkingSetSize: usize,
    pub SharedCommitUsage: u64,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PROPERTYKEY {
    pub fmtid: windows_core::GUID,
    pub pid: u32,
}
pub type PTOP_LEVEL_EXCEPTION_FILTER =
    Option<unsafe extern "system" fn(exceptioninfo: *const EXCEPTION_POINTERS) -> i32>;
pub type PTRANSLATE_ADDRESS_ROUTINE64 = Option<
    unsafe extern "system" fn(hprocess: HANDLE, hthread: HANDLE, lpaddr: *const ADDRESS64) -> u64,
>;
pub type PWCHAR = *mut u16;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RAWHID {
    pub dwSizeHid: u32,
    pub dwCount: u32,
    pub bRawData: [u8; 1],
}
impl Default for RAWHID {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RAWINPUT {
    pub header: RAWINPUTHEADER,
    pub data: RAWINPUT_0,
}
impl Default for RAWINPUT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union RAWINPUT_0 {
    pub mouse: RAWMOUSE,
    pub keyboard: RAWKEYBOARD,
    pub hid: RAWHID,
}
impl Default for RAWINPUT_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RAWINPUTDEVICE {
    pub usUsagePage: u16,
    pub usUsage: u16,
    pub dwFlags: u32,
    pub hwndTarget: HWND,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RAWINPUTHEADER {
    pub dwType: u32,
    pub dwSize: u32,
    pub hDevice: HANDLE,
    pub wParam: WPARAM,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RAWKEYBOARD {
    pub MakeCode: u16,
    pub Flags: u16,
    pub Reserved: u16,
    pub VKey: u16,
    pub Message: u32,
    pub ExtraInformation: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct RAWMOUSE {
    pub usFlags: u16,
    pub Anonymous: RAWMOUSE_0,
    pub ulRawButtons: u32,
    pub lLastX: i32,
    pub lLastY: i32,
    pub ulExtraInformation: u32,
}
impl Default for RAWMOUSE {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union RAWMOUSE_0 {
    pub ulButtons: u32,
    pub Anonymous: RAWMOUSE_0_0,
}
impl Default for RAWMOUSE_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RAWMOUSE_0_0 {
    pub usButtonFlags: u16,
    pub usButtonData: u16,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}
pub type REFWICPixelFormatGUID = *const windows_core::GUID;
pub type REGSAM = ACCESS_MASK;
pub const REG_NOTIFY_CHANGE_LAST_SET: i32 = 4;
pub const REG_SZ: u32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RGBQUAD {
    pub rgbBlue: u8,
    pub rgbGreen: u8,
    pub rgbRed: u8,
    pub rgbReserved: u8,
}
pub const RIDEV_INPUTSINK: i32 = 256;
pub const RIDEV_REMOVE: i32 = 1;
pub const RID_INPUT: i32 = 268435459;
pub const RIM_TYPEMOUSE: i32 = 0;
pub const RI_MOUSE_LEFT_BUTTON_DOWN: i32 = 1;
pub const RI_MOUSE_LEFT_BUTTON_UP: i32 = 2;
pub const RRF_RT_REG_DWORD: i32 = 16;
pub const SC_MOVE: i32 = 61456;
pub const SC_SIZE: i32 = 61440;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: u32,
    pub lpSecurityDescriptor: *mut core::ffi::c_void,
    pub bInheritHandle: windows_core::BOOL,
}
pub const SEE_MASK_FLAG_NO_UI: i32 = 1024;
pub const SEE_MASK_INVOKEIDLIST: i32 = 12;
pub const SEE_MASK_NOASYNC: i32 = 256;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct SFGAOF(pub u32);
pub const SHCNE_ASSOCCHANGED: i32 = 134217728;
pub const SHCNE_ATTRIBUTES: i32 = 2048;
pub const SHCNE_CREATE: i32 = 2;
pub const SHCNE_DELETE: i32 = 4;
pub const SHCNE_MKDIR: i32 = 8;
pub const SHCNE_RENAMEFOLDER: i32 = 131072;
pub const SHCNE_RENAMEITEM: i32 = 1;
pub const SHCNE_RMDIR: i32 = 16;
pub const SHCNE_UPDATEDIR: i32 = 4096;
pub const SHCNE_UPDATEIMAGE: i32 = 32768;
pub const SHCNE_UPDATEITEM: i32 = 8192;
pub const SHCNRF_NewDelivery: i32 = 32768;
pub const SHCNRF_ShellLevel: i32 = 2;
pub type SHCONTF = u32;
#[repr(C, packed(1))]
#[derive(Clone, Copy, Default)]
pub struct SHChangeNotifyEntry {
    pub pidl: LPCITEMIDLIST,
    pub fRecursive: windows_core::BOOL,
}
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: HWND,
    pub lpVerb: windows_core::PCWSTR,
    pub lpFile: windows_core::PCWSTR,
    pub lpParameters: windows_core::PCWSTR,
    pub lpDirectory: windows_core::PCWSTR,
    pub nShow: i32,
    pub hInstApp: HINSTANCE,
    pub lpIDList: *mut core::ffi::c_void,
    pub lpClass: windows_core::PCWSTR,
    pub hkeyClass: HKEY,
    pub dwHotKey: u32,
    pub Anonymous: SHELLEXECUTEINFOW_0,
    pub hProcess: HANDLE,
}
#[cfg(target_arch = "x86")]
impl Default for SHELLEXECUTEINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub union SHELLEXECUTEINFOW_0 {
    pub hIcon: HANDLE,
    pub hMonitor: HANDLE,
}
#[cfg(target_arch = "x86")]
impl Default for SHELLEXECUTEINFOW_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy)]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: HWND,
    pub lpVerb: windows_core::PCWSTR,
    pub lpFile: windows_core::PCWSTR,
    pub lpParameters: windows_core::PCWSTR,
    pub lpDirectory: windows_core::PCWSTR,
    pub nShow: i32,
    pub hInstApp: HINSTANCE,
    pub lpIDList: *mut core::ffi::c_void,
    pub lpClass: windows_core::PCWSTR,
    pub hkeyClass: HKEY,
    pub dwHotKey: u32,
    pub Anonymous: SHELLEXECUTEINFOW_0,
    pub hProcess: HANDLE,
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for SHELLEXECUTEINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy)]
pub union SHELLEXECUTEINFOW_0 {
    pub hIcon: HANDLE,
    pub hMonitor: HANDLE,
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for SHELLEXECUTEINFOW_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(1))]
#[derive(Clone, Copy, Default)]
pub struct SHELLSTATEA {
    pub _bitfield1: windows_core::BOOL,
    pub dwWin95Unused: u32,
    pub uWin95Unused: u32,
    pub lParamSort: i32,
    pub iSortDirection: i32,
    pub version: u32,
    pub uNotUsed: u32,
    pub _bitfield2: windows_core::BOOL,
}
#[repr(C, packed(1))]
#[derive(Clone, Copy, Default)]
pub struct SHELLSTATEW {
    pub _bitfield1: windows_core::BOOL,
    pub dwWin95Unused: u32,
    pub uWin95Unused: u32,
    pub lParamSort: i32,
    pub iSortDirection: i32,
    pub version: u32,
    pub uNotUsed: u32,
    pub _bitfield2: windows_core::BOOL,
}
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy)]
pub struct SHFILEINFOW {
    pub hIcon: HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}
#[cfg(target_arch = "x86")]
impl Default for SHFILEINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SHFILEINFOW {
    pub hIcon: HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for SHFILEINFOW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type SHGDNF = u32;
pub const SHGFI_PIDL: i32 = 8;
pub const SHGFI_TYPENAME: i32 = 1024;
pub const SHGFI_USEFILEATTRIBUTES: i32 = 16;
#[repr(C, packed(1))]
#[derive(Clone, Copy)]
pub struct SHITEMID {
    pub cb: u16,
    pub abID: [u8; 1],
}
impl Default for SHITEMID {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C, packed(1))]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Default)]
pub struct SHQUERYRBINFO {
    pub cbSize: u32,
    pub i64Size: i64,
    pub i64NumItems: i64,
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SHQUERYRBINFO {
    pub cbSize: u32,
    pub i64Size: i64,
    pub i64NumItems: i64,
}
pub type SIATTRIBFLAGS = u32;
pub type SICHINTF = u32;
pub type SIGDN = i32;
pub const SIGDN_FILESYSPATH: SIGDN = -2147123200;
pub const SIGDN_NORMALDISPLAY: SIGDN = 0;
pub const SIGDN_PARENTRELATIVEEDITING: SIGDN = -2147282943;
pub type SIIGBF = i32;
pub const SIIGBF_BIGGERSIZEOK: SIIGBF = 1;
pub const SIIGBF_ICONONLY: SIIGBF = 4;
pub const SIIGBF_INCACHEONLY: SIIGBF = 16;
pub const SIIGBF_RESIZETOFIT: SIIGBF = 0;
pub const SIIGBF_THUMBNAILONLY: SIIGBF = 8;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}
pub type SIZEL = SIZE;
pub type SLGP_FLAGS = u32;
pub const SLGP_RAWPATH: SLGP_FLAGS = 4;
pub const SLGP_UNCPRIORITY: SLGP_FLAGS = 2;
pub const SMTO_ABORTIFHUNG: i32 = 2;
pub const SMTO_BLOCK: i32 = 1;
pub const SM_CXDOUBLECLK: i32 = 36;
pub const SM_CXDRAG: i32 = 68;
pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYDOUBLECLK: i32 = 37;
pub const SM_CYDRAG: i32 = 69;
pub const SM_CYSCREEN: i32 = 1;
pub const SPI_GETCLIENTAREAANIMATION: i32 = 4162;
pub const SPI_GETICONMETRICS: i32 = 45;
pub const SPI_GETICONTITLELOGFONT: i32 = 31;
pub const SPI_GETMESSAGEDURATION: i32 = 8214;
pub const SPI_GETMOUSEHOVERHEIGHT: i32 = 100;
pub const SPI_GETMOUSEHOVERTIME: i32 = 102;
pub const SPI_GETMOUSEHOVERWIDTH: i32 = 98;
pub const SPI_GETWHEELSCROLLLINES: i32 = 104;
pub const SPI_SETICONTITLELOGFONT: i32 = 34;
pub const SSF_HIDEICONS: i32 = 16384;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct STACKFRAME64 {
    pub AddrPC: ADDRESS64,
    pub AddrReturn: ADDRESS64,
    pub AddrFrame: ADDRESS64,
    pub AddrStack: ADDRESS64,
    pub AddrBStore: ADDRESS64,
    pub FuncTableEntry: *mut core::ffi::c_void,
    pub Params: [u64; 4],
    pub Far: windows_core::BOOL,
    pub Virtual: windows_core::BOOL,
    pub Reserved: [u64; 3],
    pub KdHelp: KDHELP64,
}
impl Default for STACKFRAME64 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct STARTUPINFOW {
    pub cb: u32,
    pub lpReserved: windows_core::PWSTR,
    pub lpDesktop: windows_core::PWSTR,
    pub lpTitle: windows_core::PWSTR,
    pub dwX: u32,
    pub dwY: u32,
    pub dwXSize: u32,
    pub dwYSize: u32,
    pub dwXCountChars: u32,
    pub dwYCountChars: u32,
    pub dwFillAttribute: u32,
    pub dwFlags: u32,
    pub wShowWindow: u16,
    pub cbReserved2: u16,
    pub lpReserved2: LPBYTE,
    pub hStdInput: HANDLE,
    pub hStdOutput: HANDLE,
    pub hStdError: HANDLE,
}
pub type STGMEDIUM = uSTGMEDIUM;
pub const STGM_READ: i32 = 0;
pub const STILL_ACTIVE: i32 = 259;
#[repr(C)]
#[derive(Clone, Copy)]
pub struct STRRET {
    pub uType: u32,
    pub Anonymous: STRRET_0,
}
impl Default for STRRET {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub union STRRET_0 {
    pub pOleStr: windows_core::PWSTR,
    pub uOffset: u32,
    pub cStr: [i8; 260],
}
impl Default for STRRET_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub type SUBCLASSPROC = Option<
    unsafe extern "system" fn(
        hwnd: HWND,
        umsg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        uidsubclass: usize,
        dwrefdata: usize,
    ) -> LRESULT,
>;
pub const SWP_ASYNCWINDOWPOS: i32 = 16384;
pub const SWP_FRAMECHANGED: i32 = 32;
pub const SWP_HIDEWINDOW: i32 = 128;
pub const SWP_NOACTIVATE: i32 = 16;
pub const SWP_NOMOVE: i32 = 2;
pub const SWP_NOOWNERZORDER: i32 = 512;
pub const SWP_NOREDRAW: i32 = 8;
pub const SWP_NOSENDCHANGING: i32 = 1024;
pub const SWP_NOSIZE: i32 = 1;
pub const SWP_NOZORDER: i32 = 4;
pub const SWP_SHOWWINDOW: i32 = 64;
pub const SW_HIDE: i32 = 0;
pub const SW_SHOWNA: i32 = 8;
pub const SW_SHOWNOACTIVATE: i32 = 4;
pub const SW_SHOWNORMAL: i32 = 1;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}
pub const S_FALSE: windows_core::HRESULT = windows_core::HRESULT(0x1_u32 as _);
pub const S_OK: windows_core::HRESULT = windows_core::HRESULT(0x0_u32 as _);
pub const ShellLink: windows_core::GUID =
    windows_core::GUID::from_u128(0x00021401_0000_0000_c000_000000000046);
pub type TIMERPROC =
    Option<unsafe extern "system" fn(param0: HWND, param1: u32, param2: usize, param3: u32)>;
pub const TIMERV_DEFAULT_COALESCING: i32 = 0;
pub const TIME_NOSECONDS: i32 = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TIME_ZONE_INFORMATION {
    pub Bias: i32,
    pub StandardName: [u16; 32],
    pub StandardDate: SYSTEMTIME,
    pub StandardBias: i32,
    pub DaylightName: [u16; 32],
    pub DaylightDate: SYSTEMTIME,
    pub DaylightBias: i32,
}
impl Default for TIME_ZONE_INFORMATION {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
pub const TME_LEAVE: i32 = 2;
pub const TME_NONCLIENT: i32 = 16;
pub const TOOLTIPS_CLASSW: windows_core::PCWSTR = windows_core::w!("tooltips_class32");
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TPMPARAMS {
    pub cbSize: u32,
    pub rcExclude: RECT,
}
pub const TPM_BOTTOMALIGN: i32 = 32;
pub const TPM_LEFTALIGN: i32 = 0;
pub const TPM_NONOTIFY: i32 = 128;
pub const TPM_RETURNCMD: i32 = 256;
pub const TPM_RIGHTBUTTON: i32 = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TRACKMOUSEEVENT {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub hwndTrack: HWND,
    pub dwHoverTime: u32,
}
pub const TTF_TRACK: i32 = 32;
pub const TTM_ADDTOOLW: i32 = 1074;
pub const TTM_DELTOOLW: i32 = 1075;
pub const TTM_SETMAXTIPWIDTH: i32 = 1048;
pub const TTM_TRACKACTIVATE: i32 = 1041;
pub const TTM_TRACKPOSITION: i32 = 1042;
pub const TTM_UPDATETIPTEXTW: i32 = 1081;
pub const TTS_ALWAYSTIP: i32 = 1;
pub const TTS_NOPREFIX: i32 = 2;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TTTOOLINFOW {
    pub cbSize: u32,
    pub uFlags: u32,
    pub hwnd: HWND,
    pub uId: usize,
    pub rect: RECT,
    pub hinst: HINSTANCE,
    pub lpszText: windows_core::PWSTR,
    pub lParam: LPARAM,
    pub lpReserved: *mut core::ffi::c_void,
}
pub type TYMED = i32;
pub const TYMED_HGLOBAL: TYMED = 1;
pub const ULW_ALPHA: i32 = 2;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct USAGE(pub u16);
pub const USER_DEFAULT_SCREEN_DPI: i32 = 96;
pub const VK_APPS: i32 = 93;
pub const VK_BACK: i32 = 8;
pub const VK_CONTROL: i32 = 17;
pub const VK_DELETE: i32 = 46;
pub const VK_ESCAPE: i32 = 27;
pub const VK_F10: i32 = 121;
pub const VK_F2: i32 = 113;
pub const VK_F5: i32 = 116;
pub const VK_LBUTTON: i32 = 1;
pub const VK_LWIN: i32 = 91;
pub const VK_MENU: i32 = 18;
pub const VK_NEXT: i32 = 34;
pub const VK_PRIOR: i32 = 33;
pub const VK_RETURN: i32 = 13;
pub const VK_SHIFT: i32 = 16;
pub const VK_SPACE: i32 = 32;
pub const VK_TAB: i32 = 9;
pub const VK_UP: i32 = 38;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VS_FIXEDFILEINFO {
    pub dwSignature: u32,
    pub dwStrucVersion: u32,
    pub dwFileVersionMS: u32,
    pub dwFileVersionLS: u32,
    pub dwProductVersionMS: u32,
    pub dwProductVersionLS: u32,
    pub dwFileFlagsMask: u32,
    pub dwFileFlags: u32,
    pub dwFileOS: u32,
    pub dwFileType: u32,
    pub dwFileSubtype: u32,
    pub dwFileDateMS: u32,
    pub dwFileDateLS: u32,
}
pub const WAIT_FAILED: u32 = 4294967295;
pub const WAIT_OBJECT_0: i32 = 0;
pub const WHEEL_DELTA: i32 = 120;
pub type WICBitmapAlphaChannelOption = i32;
pub type WICBitmapCreateCacheOption = i32;
pub type WICBitmapDitherType = i32;
pub const WICBitmapDitherTypeNone: WICBitmapDitherType = 0;
pub const WICBitmapIgnoreAlpha: WICBitmapAlphaChannelOption = 2;
pub type WICBitmapInterpolationMode = i32;
pub const WICBitmapInterpolationModeFant: WICBitmapInterpolationMode = 3;
pub const WICBitmapInterpolationModeHighQualityCubic: WICBitmapInterpolationMode = 4;
pub type WICBitmapPaletteType = i32;
pub const WICBitmapPaletteTypeMedianCut: WICBitmapPaletteType = 1;
pub const WICBitmapUseAlpha: WICBitmapAlphaChannelOption = 0;
pub const WICBitmapUsePremultipliedAlpha: WICBitmapAlphaChannelOption = 1;
pub const WICDecodeMetadataCacheOnDemand: WICDecodeOptions = 0;
pub type WICDecodeOptions = i32;
pub type WICPixelFormatGUID = windows_core::GUID;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WICRect {
    pub X: i32,
    pub Y: i32,
    pub Width: i32,
    pub Height: i32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WIN32_FIND_DATAW {
    pub dwFileAttributes: u32,
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nFileSizeHigh: u32,
    pub nFileSizeLow: u32,
    pub dwReserved0: u32,
    pub dwReserved1: u32,
    pub cFileName: [u16; 260],
    pub cAlternateFileName: [u16; 14],
}
impl Default for WIN32_FIND_DATAW {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WINDOWPLACEMENT {
    pub length: u32,
    pub flags: u32,
    pub showCmd: u32,
    pub ptMinPosition: POINT,
    pub ptMaxPosition: POINT,
    pub rcNormalPosition: RECT,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WINDOWPOS {
    pub hwnd: HWND,
    pub hwndInsertAfter: HWND,
    pub x: i32,
    pub y: i32,
    pub cx: i32,
    pub cy: i32,
    pub flags: u32,
}
pub type WINEVENTPROC = Option<
    unsafe extern "system" fn(
        hwineventhook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        idobject: i32,
        idchild: i32,
        ideventthread: u32,
        dwmseventtime: u32,
    ),
>;
pub const WINEVENT_OUTOFCONTEXT: i32 = 0;
pub const WINEVENT_SKIPOWNPROCESS: i32 = 2;
pub const WM_ACTIVATE: i32 = 6;
pub const WM_ACTIVATEAPP: i32 = 28;
pub const WM_APP: i32 = 32768;
pub const WM_CAPTURECHANGED: i32 = 533;
pub const WM_CHAR: i32 = 258;
pub const WM_CLOSE: i32 = 16;
pub const WM_COMMAND: i32 = 273;
pub const WM_CONTEXTMENU: i32 = 123;
pub const WM_CREATE: i32 = 1;
pub const WM_CTLCOLOREDIT: i32 = 307;
pub const WM_DESTROY: i32 = 2;
pub const WM_DISPLAYCHANGE: i32 = 126;
pub const WM_DPICHANGED: i32 = 736;
pub const WM_DRAWITEM: i32 = 43;
pub const WM_DWMCOLORIZATIONCOLORCHANGED: i32 = 800;
pub const WM_DWMCOMPOSITIONCHANGED: i32 = 798;
pub const WM_ENDSESSION: i32 = 22;
pub const WM_ENTERMENULOOP: i32 = 529;
pub const WM_ENTERSIZEMOVE: i32 = 561;
pub const WM_ERASEBKGND: i32 = 20;
pub const WM_EXITMENULOOP: i32 = 530;
pub const WM_EXITSIZEMOVE: i32 = 562;
pub const WM_GETDLGCODE: i32 = 135;
pub const WM_GETMINMAXINFO: i32 = 36;
pub const WM_HOTKEY: i32 = 786;
pub const WM_INITMENUPOPUP: i32 = 279;
pub const WM_INPUT: i32 = 255;
pub const WM_KEYDOWN: i32 = 256;
pub const WM_KEYUP: i32 = 257;
pub const WM_KILLFOCUS: i32 = 8;
pub const WM_LBUTTONDBLCLK: i32 = 515;
pub const WM_LBUTTONDOWN: i32 = 513;
pub const WM_LBUTTONUP: i32 = 514;
pub const WM_MBUTTONDOWN: i32 = 519;
pub const WM_MEASUREITEM: i32 = 44;
pub const WM_MENUSELECT: i32 = 287;
pub const WM_MOUSEACTIVATE: i32 = 33;
pub const WM_MOUSELEAVE: i32 = 675;
pub const WM_MOUSEMOVE: i32 = 512;
pub const WM_MOUSEWHEEL: i32 = 522;
pub const WM_MOVE: i32 = 3;
pub const WM_MOVING: i32 = 534;
pub const WM_NCACTIVATE: i32 = 134;
pub const WM_NCCALCSIZE: i32 = 131;
pub const WM_NCDESTROY: i32 = 130;
pub const WM_NCHITTEST: i32 = 132;
pub const WM_NCLBUTTONDBLCLK: i32 = 163;
pub const WM_NCLBUTTONDOWN: i32 = 161;
pub const WM_NCMOUSELEAVE: i32 = 674;
pub const WM_NCMOUSEMOVE: i32 = 160;
pub const WM_NCPAINT: i32 = 133;
pub const WM_NCRBUTTONUP: i32 = 165;
pub const WM_NULL: i32 = 0;
pub const WM_PAINT: i32 = 15;
pub const WM_QUERYENDSESSION: i32 = 17;
pub const WM_QUIT: i32 = 18;
pub const WM_RBUTTONDOWN: i32 = 516;
pub const WM_RBUTTONUP: i32 = 517;
pub const WM_SETCURSOR: i32 = 32;
pub const WM_SETFOCUS: i32 = 7;
pub const WM_SETFONT: i32 = 48;
pub const WM_SETTINGCHANGE: i32 = 26;
pub const WM_SHOWWINDOW: i32 = 24;
pub const WM_SIZE: i32 = 5;
pub const WM_SIZING: i32 = 532;
pub const WM_SYSCOMMAND: i32 = 274;
pub const WM_SYSKEYDOWN: i32 = 260;
pub const WM_SYSKEYUP: i32 = 261;
pub const WM_THEMECHANGED: i32 = 794;
pub const WM_TIMER: i32 = 275;
pub const WM_USER: i32 = 1024;
pub const WM_WINDOWPOSCHANGED: i32 = 71;
pub const WM_WINDOWPOSCHANGING: i32 = 70;
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: windows_core::PCWSTR,
    pub lpszClassName: windows_core::PCWSTR,
    pub hIconSm: HICON,
}
pub type WNDENUMPROC =
    Option<unsafe extern "system" fn(param0: HWND, param1: LPARAM) -> windows_core::BOOL>;
pub type WNDPROC = Option<
    unsafe extern "system" fn(param0: HWND, param1: u32, param2: WPARAM, param3: LPARAM) -> LRESULT,
>;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct WPARAM(pub usize);
pub const WS_BORDER: i32 = 8388608;
pub const WS_CAPTION: i32 = 12582912;
pub const WS_CHILD: i32 = 1073741824;
pub const WS_CLIPCHILDREN: i32 = 33554432;
pub const WS_CLIPSIBLINGS: i32 = 67108864;
pub const WS_EX_ACCEPTFILES: i32 = 16;
pub const WS_EX_LAYERED: i32 = 524288;
pub const WS_EX_NOACTIVATE: i32 = 134217728;
pub const WS_EX_NOREDIRECTIONBITMAP: i32 = 2097152;
pub const WS_EX_TOOLWINDOW: i32 = 128;
pub const WS_EX_TOPMOST: i32 = 8;
pub const WS_EX_TRANSPARENT: i32 = 32;
pub const WS_MAXIMIZEBOX: i32 = 65536;
pub const WS_MINIMIZEBOX: i32 = 131072;
pub const WS_OVERLAPPEDWINDOW: i32 = 13565952;
pub const WS_POPUP: u32 = 2147483648;
pub const WS_SYSMENU: i32 = 524288;
pub const WS_THICKFRAME: i32 = 262144;
pub const WS_VISIBLE: i32 = 268435456;
#[cfg(any(target_arch = "arm64ec", target_arch = "x86_64"))]
pub type XMM_SAVE_AREA32 = XSAVE_FORMAT;
#[repr(C)]
#[cfg(target_arch = "x86")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XSAVE_FORMAT {
    pub ControlWord: u16,
    pub StatusWord: u16,
    pub TagWord: u8,
    pub Reserved1: u8,
    pub ErrorOpcode: u16,
    pub ErrorOffset: u32,
    pub ErrorSelector: u16,
    pub Reserved2: u16,
    pub DataOffset: u32,
    pub DataSelector: u16,
    pub Reserved3: u16,
    pub MxCsr: u32,
    pub MxCsr_Mask: u32,
    pub FloatRegisters: [M128A; 8],
    pub XmmRegisters: [M128A; 8],
    pub Reserved4: [u8; 224],
}
#[cfg(target_arch = "x86")]
impl Default for XSAVE_FORMAT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XSAVE_FORMAT {
    pub ControlWord: u16,
    pub StatusWord: u16,
    pub TagWord: u8,
    pub Reserved1: u8,
    pub ErrorOpcode: u16,
    pub ErrorOffset: u32,
    pub ErrorSelector: u16,
    pub Reserved2: u16,
    pub DataOffset: u32,
    pub DataSelector: u16,
    pub Reserved3: u16,
    pub MxCsr: u32,
    pub MxCsr_Mask: u32,
    pub FloatRegisters: [M128A; 8],
    pub XmmRegisters: [M128A; 16],
    pub Reserved4: [u8; 96],
}
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm64ec",
    target_arch = "x86_64"
))]
impl Default for XSAVE_FORMAT {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub struct uSTGMEDIUM {
    pub tymed: u32,
    pub Anonymous: uSTGMEDIUM_0,
    pub pUnkForRelease: core::mem::ManuallyDrop<Option<windows_core::IUnknown>>,
}
impl Clone for uSTGMEDIUM {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for uSTGMEDIUM {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
#[repr(C)]
pub union uSTGMEDIUM_0 {
    pub hBitmap: HBITMAP,
    pub hMetaFilePict: HMETAFILEPICT,
    pub hEnhMetaFile: HENHMETAFILE,
    pub hGlobal: HGLOBAL,
    pub lpszFileName: windows_core::PWSTR,
    pub pstm: core::mem::ManuallyDrop<Option<IStream>>,
    pub pstg: core::mem::ManuallyDrop<Option<IStorage>>,
}
impl Clone for uSTGMEDIUM_0 {
    fn clone(&self) -> Self {
        unsafe { core::mem::transmute_copy(self) }
    }
}
impl Default for uSTGMEDIUM_0 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
