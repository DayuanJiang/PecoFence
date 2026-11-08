# scripts/wintest/lib.ps1: helpers for desktop test steps on a Windows test box.
# run.sh copies it to C:\pf\kit and its job.ps1 dot-sources it before the steps file, in pf's
# desktop session (scheduled task PFJob). See README.md.
#
# Layout on the box:
#   C:\pf\bin    the build under test (synced by run.sh, only when it changed)
#   C:\pf\extra  extra files given with run.sh --extra (installers, test exes, ...)
#   C:\pf\run    this job: app\ (portable copy), shots\, job.log, results.tsv, done.txt
#
# Steps call Case and Check; every Check is one row of results.tsv, which run.sh prints as the
# pass/fail table. A failed Check also saves a screenshot. An error in a step line becomes a
# failed check of the current case and the next line still runs.
#
# Names: helper functions must not share a name with a PowerShell alias (`kill`, `measure`,
# `cli`, `where`, `ls`, ... win over a function and the job silently does something else).
# run.sh checks every steps file for that and for parse errors before it starts a job.

$ErrorActionPreference = 'Continue'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$PfBin = 'C:\pf\bin'
$PfExtra = 'C:\pf\extra'
$PfRun = 'C:\pf\run'
$PfShots = "$PfRun\shots"
$PfApp = "$PfRun\app"
$PfInst = 'wt' + (Get-Random -Minimum 10000 -Maximum 99999)
$PfDesk = [Environment]::GetFolderPath('Desktop')
New-Item -ItemType Directory -Force $PfShots, $PfApp | Out-Null
$script:PfCase = 'setup'
$script:PfFails = 0

function Log($m) { "$(Get-Date -Format HH:mm:ss.fff) $m" | Add-Content "$PfRun\job.log" -Encoding UTF8 }

Add-Type -AssemblyName System.Drawing, System.Windows.Forms, UIAutomationClient, UIAutomationTypes
# Input, screenshots, window rects (from the 2026-11-05 EC2 jobs).
Add-Type -ReferencedAssemblies System.Drawing @'
using System;
using System.Runtime.InteropServices;
using System.Drawing;
public static class G {
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr v);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, INPUT[] i, int size);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern bool SystemParametersInfoW(uint a, uint b, string c, uint d);
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct DEVMODE {
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dmDeviceName;
    public short dmSpecVersion, dmDriverVersion, dmSize, dmDriverExtra; public int dmFields;
    public int dmPositionX, dmPositionY, dmDisplayOrientation, dmDisplayFixedOutput;
    public short dmColor, dmDuplex, dmYResolution, dmTTOption, dmCollate;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dmFormName;
    public short dmLogPixels; public int dmBitsPerPel, dmPelsWidth, dmPelsHeight, dmDisplayFlags, dmDisplayFrequency;
    public int dmICMMethod, dmICMIntent, dmMediaType, dmDitherType, dmReserved1, dmReserved2, dmPanningWidth, dmPanningHeight; }
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int EnumDisplaySettingsW(string dev, int mode, ref DEVMODE dm);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int ChangeDisplaySettingsW(ref DEVMODE dm, int flags);
  public static int SetRes(int w, int h) {
    var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
    EnumDisplaySettingsW(null, -1, ref dm);
    dm.dmPelsWidth = w; dm.dmPelsHeight = h; dm.dmFields = 0x80000 | 0x100000;
    return ChangeDisplaySettingsW(ref dm, 0);
  }
  // The largest mode the display offers up to maxW wide; "WxH" list for the log.
  public static string Modes() {
    var s = new System.Text.StringBuilder();
    var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
    for (int i = 0; EnumDisplaySettingsW(null, i, ref dm) != 0 && i < 400; i++) s.Append(dm.dmPelsWidth + "x" + dm.dmPelsHeight + " ");
    return s.ToString();
  }
  public static int Best(int maxW, out int bw, out int bh) {
    bw = 0; bh = 0;
    var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
    for (int i = 0; EnumDisplaySettingsW(null, i, ref dm) != 0 && i < 400; i++)
      if (dm.dmPelsWidth <= maxW && dm.dmPelsWidth * dm.dmPelsHeight > bw * bh) { bw = dm.dmPelsWidth; bh = dm.dmPelsHeight; }
    return bw > 0 ? SetRes(bw, bh) : -100;
  }
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct KEYBDINPUT { public ushort vk; public ushort scan; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] public struct MOUSEINPUT { public int dx, dy; public uint data, flags, time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] public struct U { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] public struct INPUT { public uint type; public U u; }
  public static string Cls(IntPtr h) { var s = new System.Text.StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  static void Key(ushort vk, ushort scan, uint flags) {
    var i = new INPUT[1]; i[0].type = 1; i[0].u.ki.vk = vk; i[0].u.ki.scan = scan; i[0].u.ki.flags = flags;
    SendInput(1, i, Marshal.SizeOf(typeof(INPUT)));
  }
  public static void Type(string text) {
    foreach (char c in text) {
      if (c == '\n') { Key(0x0D, 0, 0); Key(0x0D, 0, 2); }
      else { Key(0, c, 4); Key(0, c, 4 | 2); }
      System.Threading.Thread.Sleep(15);
    }
  }
  public static void Press(ushort vk) { Key(vk, 0, 0); Key(vk, 0, 2); }
  public static void KeyDown(ushort vk) { Key(vk, 0, 0); }
  public static void KeyUp(ushort vk) { Key(vk, 0, 2); }
  public static void MoveTo(int x, int y) {
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1);
    mouse_event(0x8001, x * 65535 / (w - 1), y * 65535 / (h - 1), 0, UIntPtr.Zero);
  }
  public static void Click(int x, int y) { MoveTo(x, y); System.Threading.Thread.Sleep(80); mouse_event(2, 0, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(40); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void RClick(int x, int y) { MoveTo(x, y); System.Threading.Thread.Sleep(80); mouse_event(8, 0, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(40); mouse_event(16, 0, 0, 0, UIntPtr.Zero); }
  public static void Shot(string path) {
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1);
    using (var b = new Bitmap(w, h)) { using (var g = Graphics.FromImage(b)) g.CopyFromScreen(0, 0, 0, 0, b.Size); b.Save(path); }
  }
}
'@
# Window text, class, enumeration, moving windows.
Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class L {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr p, EnumProc f, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, R2, B; }
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool MoveWindow(IntPtr h, int x, int y, int w, int hh, bool r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindowW(string c, string t);
  [StructLayout(LayoutKind.Explicit, Size=40)] public struct KI { [FieldOffset(0)] public uint type; [FieldOffset(8)] public ushort vk; [FieldOffset(10)] public ushort scan; [FieldOffset(12)] public uint flags; [FieldOffset(16)] public uint time; [FieldOffset(24)] public IntPtr extra; }
  [DllImport("user32.dll")] public static extern uint SendInput(uint n, KI[] i, int size);
  public static void ScanKey(ushort vk, ushort scan, bool up) {
    var i = new KI[1]; i[0].type = 1; i[0].vk = vk; i[0].scan = scan; i[0].flags = up ? 2u : 0u;
    SendInput(1, i, Marshal.SizeOf(typeof(KI)));
  }
  public static string Text(IntPtr h) { var s = new StringBuilder(1024); GetWindowTextW(h, s, 1024); return s.ToString(); }
  public static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  // "Address: C:\..." of the breadcrumb toolbar inside a dialog or Explorer window.
  public static string Address(IntPtr top) {
    string found = null;
    EnumChildWindows(top, (h, l) => {
      if (Cls(h) == "ToolbarWindow32") { var t = Text(h); if (t.Contains(": ")) { found = t; return false; } }
      return true; }, IntPtr.Zero);
    return found;
  }
  // Visible top-level windows of processes whose name matches, "class | title | rect".
  public static string Tops(string procLike) {
    var sb = new StringBuilder();
    EnumWindows((h, l) => {
      if (!IsWindowVisible(h)) return true;
      uint pid; GetWindowThreadProcessId(h, out pid);
      string name = "";
      try { name = System.Diagnostics.Process.GetProcessById((int)pid).ProcessName; } catch {}
      if (name.IndexOf(procLike, StringComparison.OrdinalIgnoreCase) >= 0) {
        R r; GetWindowRect(h, out r);
        sb.Append(name + " | " + Cls(h) + " | " + Text(h) + " | " + r.L + "," + r.T + "," + r.R2 + "," + r.B + "\n");
      }
      return true; }, IntPtr.Zero);
    return sb.ToString();
  }
}
'@
# File dialogs and Explorer windows (address bar, file name box, active tab).
Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
[ComImport, Guid("6D5140C1-7436-11CE-8034-00AA006009FA"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IServiceProviderX { [PreserveSig] int QueryService(ref Guid s, ref Guid i, out IntPtr o); }
[ComImport, Guid("000214E2-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IShellBrowserX { [PreserveSig] int GetWindow(out IntPtr h); }
public static class QS {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr p, EnumProc f, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, string l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, StringBuilder l);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern IntPtr GetDlgItem(IntPtr h, int id);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  public static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  public static string Text(IntPtr h) { var s = new StringBuilder(2048); SendMessageW(h, 0x0D, (IntPtr)2048, s); return s.ToString(); }
  static IntPtr Find(IntPtr parent, Func<IntPtr, bool> pred) {
    IntPtr found = IntPtr.Zero;
    EnumChildWindows(parent, (h, l) => { if (pred(h)) { found = h; return false; } return true; }, IntPtr.Zero);
    return found;
  }
  public static string Address(IntPtr dlg) {
    var tb = Find(dlg, h => Cls(h) == "ToolbarWindow32" && Text(h).Contains(": "));
    return tb == IntPtr.Zero ? null : Text(tb);
  }
  public static IntPtr AddressEdit(IntPtr dlg) {
    var band = Find(dlg, h => Cls(h) == "Address Band Root");
    return band == IntPtr.Zero ? IntPtr.Zero : Find(band, h => Cls(h) == "Edit");
  }
  // Open: ComboBoxEx32 1148 > ComboBox > Edit. Save: FloatNotifySink > ComboBox > Edit 1001.
  // Folder picker: Edit 1152 ("Folder:"), returned only when picker is true.
  public static IntPtr FileNameEdit(IntPtr dlg, bool picker = false) {
    var c = GetDlgItem(dlg, 1148);
    if (c != IntPtr.Zero) return Cls(c) == "Edit" ? c : Find(c, h => Cls(h) == "Edit");
    var e = Find(dlg, h => Cls(h) == "Edit" && GetDlgCtrlID(h) == 1001 && Cls(GetParent(h)) == "ComboBox");
    if (e != IntPtr.Zero) return e;
    return picker ? GetDlgItem(dlg, 1152) : IntPtr.Zero;
  }
  public static bool IsPicker(IntPtr dlg) { return GetDlgItem(dlg, 1152) != IntPtr.Zero && GetDlgItem(dlg, 1148) == IntPtr.Zero; }
  // Address bar: click the empty end of the breadcrumb (posted, no real mouse) so the edit shows,
  // then put the path in and press Enter there.
  [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr h, out RC r);
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  public static string ViaAddressClick(IntPtr dlg, string path) {
    var band = Find(dlg, h => Cls(h) == "Address Band Root");
    if (band == IntPtr.Zero) return "no band";
    var crumbs = Find(band, h => Cls(h) == "ToolbarWindow32" && GetDlgCtrlID(h) == 1001);
    if (crumbs == IntPtr.Zero) return "no crumbs";
    RC r; GetClientRect(crumbs, out r);
    IntPtr xy = (IntPtr)(((r.B / 2) << 16) | ((r.R - 4) & 0xFFFF));
    PostMessageW(crumbs, 0x201, (IntPtr)1, xy); PostMessageW(crumbs, 0x202, IntPtr.Zero, xy);
    System.Threading.Thread.Sleep(400);
    var e = Find(band, h => Cls(h) == "Edit");
    if (e == IntPtr.Zero) return "no edit";
    bool vis = IsWindowVisible(e);
    SendMessageW(e, 0x0C, IntPtr.Zero, path);
    PostMessageW(e, 0x100, (IntPtr)0x0D, (IntPtr)0x001C0001);
    PostMessageW(e, 0x101, (IntPtr)0x0D, unchecked((IntPtr)0xC01C0001));
    return "edit visible after click=" + vis;
  }
  [DllImport("user32.dll")] static extern int GetDlgCtrlID(IntPtr h);
  [DllImport("user32.dll")] static extern IntPtr GetParent(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr after, string cls, string title);
  public static IntPtr ActiveTab(IntPtr top) { return FindWindowExW(top, IntPtr.Zero, "ShellTabWindowClass", null); }
  public static string Tree(IntPtr dlg) {
    var sb = new StringBuilder();
    EnumChildWindows(dlg, (h, l) => {
      int depth = 0; for (var p = GetParent(h); p != IntPtr.Zero && p != dlg && depth < 30; p = GetParent(p)) depth++;
      var c = Cls(h);
      string t = (c == "Edit" || c == "Button" || c == "Static" || c == "ToolbarWindow32") ? Text(h) : "";
      sb.Append(new string(' ', depth * 2) + c + " id=" + GetDlgCtrlID(h) + (IsWindowVisible(h) ? "" : " hidden") + (t.Length > 0 ? " '" + t + "'" : "") + "\n");
      return true; }, IntPtr.Zero);
    return sb.ToString();
  }
  public static string FileName(IntPtr dlg) { var e = FileNameEdit(dlg, true); return e == IntPtr.Zero ? null : Text(e); }
  public static bool HasTypeCombo(IntPtr dlg) { return GetDlgItem(dlg, 1136) != IntPtr.Zero; }
  public static string OkText(IntPtr dlg) { var b = GetDlgItem(dlg, 1); return b == IntPtr.Zero ? null : Text(b); }
  // Address bar: put the path into the (hidden) address edit and press Enter there.
  public static bool ViaAddress(IntPtr dlg, string path) {
    var e = AddressEdit(dlg);
    if (e == IntPtr.Zero) return false;
    SendMessageW(e, 0x0C, IntPtr.Zero, path);
    PostMessageW(e, 0x100, (IntPtr)0x0D, (IntPtr)0x001C0001);
    PostMessageW(e, 0x101, (IntPtr)0x0D, unchecked((IntPtr)0xC01C0001));
    return true;
  }
  // File name box: put the path in, press the OK button, the caller restores the old text.
  public static string ViaFileName(IntPtr dlg, string path) {
    var e = FileNameEdit(dlg);
    if (e == IntPtr.Zero) return null;
    var old = Text(e);
    SendMessageW(e, 0x0C, IntPtr.Zero, path);
    PostMessageW(dlg, 0x111, (IntPtr)1, IntPtr.Zero);
    return old;
  }
  public static void SetFileName(IntPtr dlg, string text) { var e = FileNameEdit(dlg); if (e != IntPtr.Zero) SendMessageW(e, 0x0C, IntPtr.Zero, text); }
  // The tab window (ShellTabWindowClass) of one ShellWindows item.
  public static IntPtr TabWindow(object item) {
    var sp = item as IServiceProviderX;
    if (sp == null) return IntPtr.Zero;
    Guid sid = new Guid("4C96BE40-915C-11CF-99D3-00AA004AE837"), iid = new Guid("000214E2-0000-0000-C000-000000000046");
    IntPtr p;
    if (sp.QueryService(ref sid, ref iid, out p) != 0 || p == IntPtr.Zero) return IntPtr.Zero;
    var sb = (IShellBrowserX)Marshal.GetObjectForIUnknown(p);
    Marshal.Release(p);
    IntPtr h; sb.GetWindow(out h);
    return h;
  }
}
'@
# The Peek search box's text.
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class M {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, StringBuilder l);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  public static string BoxText() {
    IntPtr box = FindWindowExW(IntPtr.Zero, IntPtr.Zero, "PecoFence.PeekSearch", null);
    if (box == IntPtr.Zero) return null;
    IntPtr edit = FindWindowExW(box, IntPtr.Zero, "Edit", null);
    if (edit == IntPtr.Zero) return "(no edit)";
    var sb = new StringBuilder(512); SendMessageW(edit, 0x000D, (IntPtr)512, sb); return sb.ToString();
  }
}
'@
[G]::SetProcessDpiAwarenessContext([IntPtr](-4)) | Out-Null

# ---- results

# Starts a group of checks. The steps after it are plain script lines (variables stay shared
# between cases); an error in a line becomes a failed check of the current case (the trap in
# job.ps1) and the next line runs.
# A popup menu left open by the case before is closed first: while one is open the CLI and the
# test script wait, and every later check fails after a timeout.
function Case($name) {
  if ($script:PfCaseClock) { Log "== case $script:PfCase done in $([int]$script:PfCaseClock.Elapsed.TotalSeconds) s" }
  CloseMenus
  $script:PfCase = $name
  $script:PfCaseClock = [Diagnostics.Stopwatch]::StartNew()
  Log "== case $name"
}

# Esc until no popup menu (#32768) is open.
function CloseMenus() {
  for ($i = 0; $i -lt 4; $i++) {
    $m = [G]::FindWindowW('#32768', [NullString]::Value)
    if ($m -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($m)) { return }
    [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  }
}

# run.sh puts `trap { PfTrap $_; continue }` in front of every steps file. An Add-Type that
# does not compile only says "Compilation errors occurred": the compiler's messages are the
# errors just before it, so they go into the table too.
function PfTrap($e) {
  $where = "$(Split-Path -Leaf $e.InvocationInfo.ScriptName):$($e.InvocationInfo.ScriptLineNumber)"
  $msg = $e.Exception.Message
  if ($msg -match 'Cannot add type') {
    $msg += ' ' + ((@($Error) | Select-Object -Skip 1 -First 4 | ForEach-Object { "$_" }) -join ' | ')
  }
  Check 'no error' $false "$msg at $where"
}

function Check($name, $ok, $detail = '') {
  $r = if ($ok) { 'PASS' } else { 'FAIL' }
  $d = "$detail" -replace '[\t\r\n]+', ' '
  if ($d.Length -gt 400) { $d = $d.Substring(0, 400) + '...' }
  "$script:PfCase`t$name`t$r`t$d" | Add-Content "$PfRun\results.tsv" -Encoding UTF8
  Log "CHECK $r $name $d"
  if (-not $ok) {
    $script:PfFails++
    Shot ('FAIL-{0:D2}' -f $script:PfFails)
  }
}

function Shot($name) {
  Start-Sleep -Milliseconds 400
  $safe = $name -replace '[^\w.-]', '_'
  [G]::Shot("$PfShots\$safe.png")
  Log "shot $safe"
}

# ---- PecoFence

# Starts the build under test as a portable instance of its own (config in $PfApp\config, log
# in %LOCALAPPDATA%\PecoFence\pecofence.<instance>.log). -Script gives test-script lines
# (written without a BOM: a BOM eats the first line); the default keeps it running an hour.
# A second PfStart in the same job keeps the config, so it tests a restart.
# Variables set in $env: before PfStart reach the app (PECOFENCE_WEBVIEW_ARGS,
# PECOFENCE_UI_TEST_WINDOWS for a debug build).
function PfStart([string[]]$Script = @('sleep 3600000'), [string]$Lang = 'en', [string]$LogLevel = 'pecofence=debug') {
  if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
  # A start renames the last log to .prev.log and a third start drops it: keep a copy of every
  # earlier run's log for PfLogAll and the end-of-job check.
  $cur = PfLogPath
  if (Test-Path $cur) {
    New-Item -ItemType Directory -Force "$PfRun\logs" | Out-Null
    $n = @(Get-ChildItem "$PfRun\logs\app-*.log" -ErrorAction SilentlyContinue).Count + 1
    Copy-Item $cur "$PfRun\logs\app-$n.log"
  }
  if (-not (Test-Path "$PfApp\pecofence.exe")) { Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = $LogLevel
  [IO.File]::WriteAllLines("$PfRun\script.txt", $Script, (New-Object Text.UTF8Encoding($false)))
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
  $h = WaitWin 'PecoFence.Control' 20 -Hidden
  Log "started $PfInst control=$h"
  Start-Sleep -Seconds 3
  if ($Lang) { Pf settings set language $Lang | Out-Null }
}

# Ends PecoFence the way a sign-out does (it saves, restores the desktop icons and quits).
# Killing it instead makes the watchdog report a crash.
function PfQuit() {
  $h = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  $r = [IntPtr]::Zero
  if ($h -ne [IntPtr]::Zero) {
    [G]::SendMessageTimeoutW($h, 0x11, [IntPtr]0, [IntPtr]1, 2, 5000, [ref]$r) | Out-Null
    [G]::SendMessageTimeoutW($h, 0x16, [IntPtr]1, [IntPtr]1, 2, 5000, [ref]$r) | Out-Null
  }
  for ($i = 0; $i -lt 50 -and (Get-Process pecofence -ErrorAction SilentlyContinue); $i++) { Start-Sleep -Milliseconds 200 }
  $left = [bool](Get-Process pecofence -ErrorAction SilentlyContinue)
  Log "quit; still running: $left"
  if ($left) { Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue | Stop-Process -Force }
}
function Quit() { PfQuit }

function PfAlive() { return [bool](Get-Process pecofence -ErrorAction SilentlyContinue) }

# pecofence-cli against the running test instance; returns its output as one string.
function Pf {
  $a = @($args | ForEach-Object { $_ })
  $r = & "$PfApp\pecofence-cli.exe" @a 2>&1 | Out-String
  $t = $r.Trim(); if ($t.Length -gt 300) { $t = $t.Substring(0, 300) + "... ($($t.Length) chars)" }
  Log ("cli " + ($a -join ' ') + " -> " + $t)
  return $r
}
# `fence list`, `item list` and `rule list` print plain arrays (no wrapper object).
function PfJson { return ((Pf @args) | ConvertFrom-Json) }

function PfLogPath() { return "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.log" }

# Every app log of this job: the test instance's earlier runs (kept by PfStart) and the logs in
# %LOCALAPPDATA%\PecoFence, which run.sh empties before each job, so an installed or portable
# PecoFence started by the steps is covered too.
function PfLogFiles() {
  return @(@(Get-ChildItem "$PfRun\logs\app-*.log" -ErrorAction SilentlyContinue) +
    @(Get-ChildItem "$env:LOCALAPPDATA\PecoFence\*.log" -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne "pecofence.$PfInst.prev.log" }))
}
function PfLogAll() { return @(PfLogFiles | ForEach-Object { Get-Content $_.FullName -Encoding UTF8 }) }

# ERROR / panic lines the app logged since the job began, in any of its logs.
# Case-sensitive: a logged text may say "error" (the feedback excerpt's heading does).
function PfLogErrors() { return @(PfLogAll | Where-Object { $_ -cmatch ' ERROR |panicked' }) }

# Waits for a log line matching the regex (for example a `>> dump <tag>` line of the test
# script); returns the line or $null.
function PfWaitLog($regex, $sec = 15) {
  for ($i = 0; $i -lt $sec * 5; $i++) {
    $p = PfLogPath
    if (Test-Path $p) {
      $m = @(Get-Content $p -Encoding UTF8 | Where-Object { $_ -match $regex }) | Select-Object -Last 1
      if ($m) { return $m }
    }
    Start-Sleep -Milliseconds 200
  }
  return $null
}

function PfFence($title) {
  $j = PfJson fence list
  foreach ($f in @($j.fences) + @($j)) { if ($f -and $f.title -eq $title) { return $f } }
  return $null
}

# ---- windows and input

function WaitWin($cls, $sec = 10, [switch]$Hidden) {
  for ($i = 0; $i -lt $sec * 10; $i++) {
    $h = [G]::FindWindowW($cls, [NullString]::Value)
    if ($h -ne [IntPtr]::Zero -and ($Hidden -or [G]::IsWindowVisible($h))) { return $h }
    Start-Sleep -Milliseconds 100
  }
  return [IntPtr]::Zero
}
function Rect($h) { $r = New-Object G+RECT; [G]::GetWindowRect($h, [ref]$r) | Out-Null; return $r }
function Fg() { $h = [L]::GetForegroundWindow(); return "$([L]::Cls($h)) '$([L]::Text($h))'" }
function Chord($mod, $vk) { [G]::KeyDown($mod); Start-Sleep -Milliseconds 30; [G]::Press($vk); Start-Sleep -Milliseconds 30; [G]::KeyUp($mod) }
function PeekKey() { [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x20); [G]::KeyUp(0x12); [G]::KeyUp(0x11) }
function DoubleCtrl() { foreach ($k in 1, 2) { [L]::ScanKey(0xA2, 0x1D, $false); Start-Sleep -Milliseconds 60; [L]::ScanKey(0xA2, 0x1D, $true); Start-Sleep -Milliseconds 110 } }
function TitleClick($h) { $r = Rect $h; [G]::Click($r.L + 260, $r.T + 14); Start-Sleep -Milliseconds 600 }
function NotepadTitles() { (Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.MainWindowTitle }) -join ' | ' }

# Closes Explorer folder windows and Notepads (left over from earlier cases).
function CloseWindows() {
  (New-Object -ComObject Shell.Application).Windows() | ForEach-Object { $_.Quit() }
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 400
}

function Explorers() {
  $lines = @()
  foreach ($w in @((New-Object -ComObject Shell.Application).Windows())) {
    $tab = [QS]::TabWindow($w)
    $path = try { $w.Document.Folder.Self.Path } catch { '?' }
    $lines += "active=$([QS]::ActiveTab([IntPtr]$w.HWND) -eq $tab) top=$($w.HWND) tab=$tab path=$path"
  }
  return ($lines -join "`n")
}
function WaitAddr($dlg, $want) {
  for ($i = 0; $i -lt 30; $i++) { $a = [QS]::Address($dlg); if ($a -and $a.EndsWith($want)) { return $a }; Start-Sleep -Milliseconds 100 }
  return [QS]::Address($dlg)
}
function DlgState($name, $dlg) {
  "$name visible=$([QS]::IsWindowVisible($dlg)) address=$([QS]::Address($dlg)) filename='$([QS]::FileName($dlg))' ok='$([QS]::OkText($dlg))'"
}

# A UI Automation element by name under a window (settings switches and the like).
function UiaFind($hwnd, $name) {
  $root = [Windows.Automation.AutomationElement]::FromHandle($hwnd)
  $cond = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $name)
  return $root.FindFirst([Windows.Automation.TreeScope]::Descendants, $cond)
}

# ---- which copy the generic helpers talk to
# Pf / PfJson always mean the test instance (PfStart), Ipf / IpfJson the user's installed or
# portable copy (user.ps1). ItemFence, ItemPt, RClickItem and StateText use the test instance
# until PfUseUser switches them to the user's copy.
$script:PfTarget = 'test'
function PfUseUser() { $script:PfTarget = 'user' }
function TCli { if ($script:PfTarget -eq 'user') { return (Ipf @args) }; return (Pf @args) }
function TJson { if ($script:PfTarget -eq 'user') { return (IpfJson @args) }; return (PfJson @args) }


# ---- more input (from the hold-out tests of 2026-11-06)

# Client origin of a window (the settings page's (0,0) is the client area's top left).
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class CW {
  [StructLayout(LayoutKind.Sequential)] public struct P { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref P p);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, int d, UIntPtr e);
  public static void Wheel(int notches) { mouse_event(0x0800, 0, 0, notches * 120, UIntPtr.Zero); }
}
'@
function ClientOrigin($h) { $p = New-Object CW+P; [CW]::ClientToScreen($h, [ref]$p) | Out-Null; return $p }
# Mouse wheel over a client point of a window (negative notches scroll down).
function WheelAt($h, $cx, $cy, $notches) { $o = ClientOrigin $h; [G]::MoveTo($o.X + $cx, $o.Y + $cy); Start-Sleep -Milliseconds 100; [CW]::Wheel($notches); Start-Sleep -Milliseconds 900 }

# Real mouse: button down / up where the pointer is, a glide in steps (raw input, so drags and
# the Windows move loop see it), the wheel (notches < 0 = towards the user).
function MDown() { [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero) }
function MUp() { [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero) }
function Glide($x0, $y0, $x1, $y1, $steps = 12, $ms = 25) {
  for ($i = 1; $i -le $steps; $i++) { [G]::MoveTo([int]($x0 + ($x1 - $x0) * $i / $steps), [int]($y0 + ($y1 - $y0) * $i / $steps)); Start-Sleep -Milliseconds $ms }
}
function Wheel($x, $y, $notches) {
  [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 80
  $d = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int](120 * $notches)), 0)
  [G]::mouse_event(0x800, 0, 0, $d, [UIntPtr]::Zero)
}
function DoubleClick($x, $y) { [G]::Click($x, $y); Start-Sleep -Milliseconds 90; MDown; Start-Sleep -Milliseconds 30; MUp }

# NcHit: WM_NCHITTEST of a window at a screen point (2 caption, 1 client, 11 right edge ...).
function NcHit($h, $x, $y) {
  $res = [IntPtr]::Zero
  [G]::SendMessageTimeoutW($h, 0x84, [IntPtr]::Zero, [IntPtr](($y -shl 16) -bor ($x -band 0xFFFF)), 2, 1000, [ref]$res) | Out-Null
  return [int64]$res
}

# Real mouse: double-click, drag with the button held.
function DblClick($x, $y) {
  [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 80
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30; [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
  Start-Sleep -Milliseconds 90
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 30; [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero)
}
function DragTo($x1, $y1, $x2, $y2, $steps = 15) {
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 150
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  for ($s = 1; $s -le $steps; $s++) {
    [G]::MoveTo([int]($x1 + ($x2 - $x1) * $s / $steps), [int]($y1 + ($y2 - $y1) * $s / $steps)); Start-Sleep -Milliseconds 40
  }
  Start-Sleep -Milliseconds 400
  [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 800
}

# ---- popup menus and windows

# The open popup menu (#32768): its item texts, and a click on the item with that text.
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class Menu {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowW(string c, string t);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern int GetMenuItemCount(IntPtr m);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetMenuStringW(IntPtr m, uint id, StringBuilder s, int n, uint f);
  [DllImport("user32.dll")] static extern bool GetMenuItemRect(IntPtr w, IntPtr m, uint i, out RC r);
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  public static IntPtr Handle() { var w = FindWindowW("#32768", null); return w == IntPtr.Zero ? IntPtr.Zero : SendMessageW(w, 0x01E1, IntPtr.Zero, IntPtr.Zero); }
  public static string Items() {
    var m = Handle(); if (m == IntPtr.Zero) return null;
    var sb = new StringBuilder();
    for (int i = 0; i < GetMenuItemCount(m); i++) { var s = new StringBuilder(256); GetMenuStringW(m, (uint)i, s, 256, 0x400); sb.Append(s.ToString().Replace("&", "") + "|"); }
    return sb.ToString();
  }
  // Centre (x, y) of the first item whose text starts with `text` (screen px), or null.
  public static int[] ItemPoint(string text) {
    var m = Handle(); if (m == IntPtr.Zero) return null;
    for (int i = 0; i < GetMenuItemCount(m); i++) {
      var s = new StringBuilder(256); GetMenuStringW(m, (uint)i, s, 256, 0x400);
      if (s.ToString().Replace("&", "").StartsWith(text)) { RC r; GetMenuItemRect(IntPtr.Zero, m, (uint)i, out r); return new int[] { (r.L + r.R) / 2, (r.T + r.B) / 2 }; }
    }
    return null;
  }
}
'@
function MenuClick($text) {
  $p = [Menu]::ItemPoint($text)
  if ($null -eq $p) { return $false }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600; return $true
}

# Windows of a class, popup menus (owner-drawn: text and rect through MN_GETHMENU), the window
# under a point, the desktop's own icon view.
Add-Type @'
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class W {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern int GetMenuItemCount(IntPtr m);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetMenuStringW(IntPtr m, uint id, StringBuilder s, int n, uint flags);
  [DllImport("user32.dll")] static extern uint GetMenuState(IntPtr m, uint id, uint flags);
  [DllImport("user32.dll")] static extern bool GetMenuItemRect(IntPtr h, IntPtr m, uint i, out RECT r);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowW(string c, string t);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  public static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  public static string Title(IntPtr h) { var s = new StringBuilder(512); GetWindowTextW(h, s, 512); return s.ToString(); }
  public static string ProcOf(IntPtr h) { uint pid; GetWindowThreadProcessId(h, out pid); try { return System.Diagnostics.Process.GetProcessById((int)pid).ProcessName; } catch { return "?"; } }
  // Visible top-level windows of a class, topmost first.
  public static IntPtr[] Tops(string cls) {
    var list = new List<IntPtr>();
    EnumWindows((h, l) => { if (IsWindowVisible(h) && Cls(h) == cls) list.Add(h); return true; }, IntPtr.Zero);
    return list.ToArray();
  }
  [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr p, EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowEnabled(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, StringBuilder l);
  public static string Text(IntPtr h) { var s = new StringBuilder(2048); SendMessageW(h, 0x0D, (IntPtr)2048, s); return s.ToString(); }
  // Visible child buttons: "hwnd TAB caption without & TAB enabled TAB L,T,R,B" lines.
  public static string Buttons(IntPtr top) {
    var sb = new StringBuilder();
    EnumChildWindows(top, (h, l) => {
      if (Cls(h).IndexOf("Button", StringComparison.OrdinalIgnoreCase) >= 0 && IsWindowVisible(h)) {
        RECT r; GetWindowRect(h, out r);
        sb.Append((long)h + "\t" + Text(h).Replace("&", "") + "\t" + IsWindowEnabled(h) + "\t" + r.L + "," + r.T + "," + r.R + "," + r.B + "\n");
      }
      return true; }, IntPtr.Zero);
    return sb.ToString();
  }
  // The texts of a dialog's static controls (a message box's message).
  public static string Statics(IntPtr top) {
    var sb = new StringBuilder();
    EnumChildWindows(top, (h, l) => { var c = Cls(h); if (c == "Static" || c.StartsWith("TNewStaticText")) sb.Append(Text(h) + " "); return true; }, IntPtr.Zero);
    return sb.ToString();
  }
  public static string Info(IntPtr h) { RECT r; GetWindowRect(h, out r); return ProcOf(h) + " '" + Title(h) + "' " + r.L + "," + r.T + "," + r.R + "," + r.B; }
  // Items of an open popup menu window: "index TAB text TAB state TAB L,T,R,B" lines.
  public static string MenuItems(IntPtr menuWnd) {
    IntPtr m = SendMessageW(menuWnd, 0x01E1, IntPtr.Zero, IntPtr.Zero);
    if (m == IntPtr.Zero) return "";
    var sb = new StringBuilder();
    int n = GetMenuItemCount(m);
    for (int i = 0; i < n; i++) {
      var t = new StringBuilder(512);
      GetMenuStringW(m, (uint)i, t, 512, 0x400);
      uint st = GetMenuState(m, (uint)i, 0x400);
      RECT r; GetMenuItemRect(IntPtr.Zero, m, (uint)i, out r);
      sb.Append(i + "\t" + t.ToString().Replace("\t", " ~ ") + "\t" + st + "\t" + r.L + "," + r.T + "," + r.R + "," + r.B + "\n");
    }
    return sb.ToString();
  }
  public static string RootClassAt(int x, int y) {
    var p = new POINT(); p.X = x; p.Y = y;
    var h = WindowFromPoint(p);
    return h == IntPtr.Zero ? "" : Cls(GetAncestor(h, 2));
  }
  public static IntPtr DefView() {
    IntPtr found = FindWindowExW(FindWindowW("Progman", null), IntPtr.Zero, "SHELLDLL_DefView", null);
    if (found != IntPtr.Zero) return found;
    EnumWindows((h, l) => { if (Cls(h) == "WorkerW") { var d = FindWindowExW(h, IntPtr.Zero, "SHELLDLL_DefView", null); if (d != IntPtr.Zero) { found = d; return false; } } return true; }, IntPtr.Zero);
    return found;
  }
  // Windows' own desktop icons on screen (the FolderView list under SHELLDLL_DefView).
  public static bool IconsVisible() { var d = DefView(); if (d == IntPtr.Zero) return false; var lv = FindWindowExW(d, IntPtr.Zero, "SysListView32", null); return lv != IntPtr.Zero && IsWindowVisible(lv); }
  // Explorer's View > Show desktop icons.
  public static void ToggleIcons() { var d = DefView(); if (d != IntPtr.Zero) PostMessageW(d, 0x111, (IntPtr)0x7402, IntPtr.Zero); }
}
'@

# Popup menus: the topmost open one, its rows, a click on the row whose text matches.
function MenuWin($sec = 6) {
  for ($i = 0; $i -lt $sec * 10; $i++) { $m = @([W]::Tops('#32768')); if ($m.Count) { return $m[0] }; Start-Sleep -Milliseconds 100 }
  return [IntPtr]::Zero
}
function MenuRows($h) {
  $rows = @()
  foreach ($line in ([W]::MenuItems($h) -split "`n")) {
    if (-not $line) { continue }
    $p = $line -split "`t"; $r = $p[3] -split ','
    $rows += [pscustomobject]@{ i = [int]$p[0]; text = ($p[1] -replace '&', ''); state = [int64]$p[2]; L = [int]$r[0]; T = [int]$r[1]; R = [int]$r[2]; B = [int]$r[3] }
  }
  return $rows
}
# Entry texts, a greyed one marked (off).
function MenuTexts($h) { return ((@(MenuRows $h) | Where-Object { $_.text } | ForEach-Object { if ($_.state -band 3) { "$($_.text) (off)" } else { $_.text } }) -join ' | ') }
function MenuDupes($h) { return ((@(MenuRows $h) | Where-Object { $_.text } | Group-Object text | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name }) -join ', ') }
function MenuPoint($h, $regex) {
  $row = @(MenuRows $h) | Where-Object { $_.text -match $regex } | Select-Object -First 1
  if (-not $row) { return $null }
  return @([int](($row.L + $row.R) / 2), [int](($row.T + $row.B) / 2))
}
function MenuPick($regex, $h = [IntPtr]::Zero) {
  if ($h -eq [IntPtr]::Zero) { $h = MenuWin 5 }
  if ($h -eq [IntPtr]::Zero) { Log "MenuPick '$regex' found no menu"; return $null }
  $p = MenuPoint $h $regex
  if (-not $p) { Log "MenuPick '$regex' not among: $(MenuTexts $h)"; return $null }
  $t = (@(MenuRows $h) | Where-Object { $_.text -match $regex } | Select-Object -First 1).text
  [G]::Click($p[0], $p[1])
  Log "menu pick '$t'"
  return $t
}

function WaitGoneWin($cls, $sec) {
  for ($i = 0; $i -lt $sec * 5; $i++) { $h = [G]::FindWindowW($cls, [NullString]::Value); if ($h -eq [IntPtr]::Zero -or -not [G]::IsWindowVisible($h)) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}

function VisFences() { return @([W]::Tops('PecoFence.Fence')).Count }
function BinNames() { return ((@((New-Object -ComObject Shell.Application).NameSpace(10).Items()) | ForEach-Object { $_.Name }) -join ', ') }
function LogTime($line) { if ($line -match '^(\S+Z)\s') { return [DateTimeOffset]::Parse($Matches[1]).UtcDateTime }; return $null }

# Windows' client-area animations (SPI_GETCLIENTAREAANIMATION): an RDP session starts with them
# off, which makes PecoFence snap where a normal desktop animates (tab slides, fades).
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class Anim {
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, ref int v, uint f);
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, IntPtr v, uint f);
  public static bool Get() { int v = 0; SystemParametersInfoW(0x1042, 0, ref v, 0); return v != 0; }
  public static void Set(bool on) { SystemParametersInfoW(0x1043, 0, (IntPtr)(on ? 1 : 0), 3); }
}
'@

# ---- fence and item state

# DumpScript: test-script lines that log a `dump dN` every $ms ms for $minutes, so a step can
# read the fence windows' state at any moment with PfDump.
function DumpScript($ms = 250, $minutes = 25, $first = 4000) {
  $n = [int]($minutes * 60000 / $ms)
  return @("sleep $first") + (1..$n | ForEach-Object { "dump d$_"; "sleep $ms" }) + @('sleep 3600000')
}

# ParseDumpLine: one `[dN] live ...` dump line as an object: hwnd, l t r b (window rect, px),
# ox oy (client offset), cw ch (chrome size px), title, ntabs, tabs (strip order of tab
# titles), rects ((x, w) DIPs per tab, window x; (y, h) in a left column), active (fence id),
# rolled, tabDrag, winDrag, contentX.
function ParseDumpLine($s) {
  $o = [ordered]@{ raw = $s }
  if ($s -match 'hwnd=0x([0-9a-f]+)') { $o.hwnd = [IntPtr][Convert]::ToInt64($Matches[1], 16) }
  if ($s -match ' rect=\((-?\d+),(-?\d+),(-?\d+),(-?\d+)\)') { $o.l = [int]$Matches[1]; $o.t = [int]$Matches[2]; $o.r = [int]$Matches[3]; $o.b = [int]$Matches[4] }
  if ($s -match ' title="((?:[^"\\]|\\.)*)"') { $o.title = $Matches[1] }
  if ($s -match ' rolled=(\w+)') { $o.rolled = $Matches[1] -eq 'true' }
  if ($s -match ' tabs=(\d+)') { $o.ntabs = [int]$Matches[1] }
  if ($s -match ' active=([0-9a-f-]+)') { $o.active = $Matches[1] }
  if ($s -match ' chrome=\((\d+), (\d+)\)') { $o.cw = [int]$Matches[1]; $o.ch = [int]$Matches[2] }
  if ($s -match ' client_offset=\((-?\d+),(-?\d+)\)') { $o.ox = [int]$Matches[1]; $o.oy = [int]$Matches[2] }
  if ($s -match ' content_x=(-?\d+)') { $o.contentX = [int]$Matches[1] }
  if ($s -match ' tab_fonts=\[(.*?)\] tab_rects') { $o.tabs = @([regex]::Matches($Matches[1], '\("((?:[^"\\]|\\.)*)", \d+\)') | ForEach-Object { $_.Groups[1].Value }) }
  if ($s -match ' tab_rects=\[(.*?)\]') { $o.rects = @([regex]::Matches($Matches[1], '\((-?[\d.]+), (-?[\d.]+)\)') | ForEach-Object { , @([double]$_.Groups[1].Value, [double]$_.Groups[2].Value) }) }
  if ($s -match ' tab_drag=(\w+)') { $o.tabDrag = $Matches[1] -eq 'true' }
  if ($s -match ' window_drag=(\w+)') { $o.winDrag = $Matches[1] -eq 'true' }
  return [pscustomobject]$o
}

# PfDump: the first complete dump logged after this call (one object per live fence window),
# read from the end of the log, so it stays fast however big the log grows.
function PfDump($sec = 5) {
  $p = PfLogPath
  if (-not (Test-Path $p)) { return @() }
  $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $start = $fs.Length; $fs.Close()
  $deadline = (Get-Date).AddSeconds($sec)
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 100
    $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $fs.Position = $start
    $sr = New-Object IO.StreamReader($fs, [Text.Encoding]::UTF8); $buf = $sr.ReadToEnd(); $sr.Close()
    foreach ($m in [regex]::Matches($buf, '\[(d\d+)\] fences=')) {
      $tag = $m.Groups[1].Value
      if ($buf -notmatch ">> dump $tag\b") { continue }
      return @([regex]::Matches($buf, "\[$tag\] live ([^\r\n]*)") | ForEach-Object { ParseDumpLine $_.Groups[1].Value })
    }
  }
  return @()
}
# DumpOf: the window of that fence (its title or one of its tabs) in the next dump.
function DumpOf($name, $sec = 5) {
  foreach ($w in @(PfDump $sec)) { if ($w.title -eq $name -or (@($w.tabs) -contains $name)) { return $w } }
  return $null
}


# fence list as title -> id and id -> title.
function FenceIds() {
  $j = PfJson fence list
  $script:Id = @{}; $script:IdTitle = @{}
  foreach ($f in @($j.fences) + @($j)) { if ($f -and $f.title) { $script:Id[$f.title] = "$($f.id)"; $script:IdTitle["$($f.id)"] = $f.title } }
}
function FenceGet($t) { $j = PfJson fence get $t; if ($j.fence) { return $j.fence }; return $j }

# Desktop files: one with that name (and a line of text), and the fence title of a desktop item.
function DeskFile($name) { $p = Join-Path $PfDesk $name; Set-Content -LiteralPath $p -Value "hold-out $name" -Encoding ASCII; return $p }
# The fence an item is in (by display name or file name), read from the test instance or, after
# PfUseUser, from the user's copy.
function ItemFence($name) {
  foreach ($it in @(TJson item list)) { if ($it -and ($it.fileName -eq $name -or $it.name -eq $name)) { return $it.fenceTitle } }
  return $null
}
function WaitItemFence($file, $want, $sec = 10) {
  for ($k = 0; $k -lt $sec; $k++) { $f = ItemFence $file; if ($f -eq $want) { return $f }; Start-Sleep -Seconds 1 }
  return $f
}

# Rules from `rule list`; one rule as a comparable string; a rule by id.
function Rules() { return @((PfJson rule list).list) }
function RuleSig($r) { return ($r | Select-Object id, name, enabled, target, allOf, priorityClass, template | ConvertTo-Json -Depth 8 -Compress) }
function RuleById($id) { return (Rules | Where-Object { $_.id -eq $id } | Select-Object -First 1) }
function WaitRule($id, [scriptblock]$ok, $sec = 8) {
  for ($k = 0; $k -lt $sec * 2; $k++) { $r = RuleById $id; if ($r -and (& $ok $r)) { return $r }; Start-Sleep -Milliseconds 500 }
  return $r
}

# Where an item's icon is on screen: icons layout at 100 %, sorted by name (other sorts: CellPt
# with the index), 36 px title row, cells 80 x 96 at icon size 48, from `fence get` (windowRect, fit).
function ItemPt($fence, $name) {
  $f = TJson fence get $fence
  $items = @(TJson item list --fence $fence)
  # As a fence sorted by name shows them: arranged like the desktop (desktopLayout), This PC and
  # the Recycle Bin come first as on the desktop; any other fence sorts them among the files.
  $ns = if ($f.desktopLayout) { @($items | Where-Object { $_.kind -eq 'namespace' } | Sort-Object name) } else { @() }
  $order = @(@($ns) + @($items | Where-Object { $ns -notcontains $_ } | Sort-Object name) | ForEach-Object { $_.name })
  $i = [array]::IndexOf($order, $name)
  if ($i -lt 0 -or -not $f.windowRect) { Log "ItemPt: '$name' not in '$fence' ($($order -join ', '))"; return $null }
  $wr = $f.windowRect
  $cols = if ($f.fit -and $f.fit.columns) { [int]$f.fit.columns } else { [math]::Max(1, [math]::Floor($wr.w / 80)) }
  $pitch = $wr.w / $cols
  $x = [int]($wr.x + ($i % $cols) * $pitch + $pitch / 2)
  $y = [int]($wr.y + 36 + 8 + [math]::Floor($i / $cols) * 96 + 32)
  Log "ItemPt '$name' in '$fence': index $i of $($order.Count), cols $cols -> $x,$y"
  return @($x, $y)
}
function RClickItem($fence, $name) {
  $p = ItemPt $fence $name
  if (-not $p) { return [IntPtr]::Zero }
  [G]::RClick($p[0], $p[1])
  return (MenuWin 8)
}

# Fences, rules, item memberships and the theme as sorted text lines (compared across an upgrade).
function StateText() {
  $lines = @()
  foreach ($f in @(TJson fence list) | Sort-Object title) { $r = $f.rect; $lines += "fence $($f.title) | $($r.x),$($r.y),$($r.w),$($r.h) | tint $($f.tint)" }
  foreach ($r in @((TJson rule list).list)) { $lines += "rule $($r.name) -> $($r.targetTitle) enabled=$($r.enabled)" }
  foreach ($it in @(TJson item list) | Sort-Object name) { $lines += "item $($it.name) in $($it.fenceTitle)" }
  $lines += "theme $((TCli settings get theme).Trim())"
  return $lines
}

# ---- more from the second hold-out test (2026-11-06)

Add-Type -AssemblyName WindowsBase, Microsoft.VisualBasic

# PfFresh: the test instance as a first run (no config), with a test script.
function PfFresh([string[]]$Script = @('sleep 3600000')) {
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config" -ErrorAction SilentlyContinue
  PfStart -Script $Script
}

# One field of a dump line the kit's ParseDumpLine leaves out (selected, focused, visible,
# peeking, rolling, items ...), as text.
function DumpVal($w, $name) {
  if ($w -and $w.raw -match "(?:^| )$name=(\S+)") { return $Matches[1].Trim(',') }
  return $null
}

# The shareable log of the test instance (what 「反馈」 sends).
function ShareLog() { return "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.share.log" }

# What the 「反馈」 form would send besides the user's text: the JSON the `feedback-info` line of
# the test script logs (it must be in the script given to PfStart).
function FeedbackInfo($sec = 30) {
  $l = PfWaitLog 'feedback-info \{' $sec
  if (-not $l) { return $null }
  return ($l.Substring($l.IndexOf('feedback-info {') + 14) | ConvertFrom-Json)
}

# Snapshots from `snapshot list`. (A missing property of an array's items enumerates as one $null
# per item in PowerShell 5.1, so `$list.snapshots` of a plain array is a list of $nulls.)
function Snaps() { return @(@(PfJson snapshot list) | Where-Object { $_ -and $_.id }) }

# Where the i-th icon of a fence is (the kit's ItemPt by position instead of by name: two items
# with the same display name), and a crop of its 48 px icon.
function CellPt($fence, $i) {
  $f = FenceGet $fence
  $wr = $f.windowRect
  $cols = if ($f.fit -and $f.fit.columns) { [int]$f.fit.columns } else { [math]::Max(1, [math]::Floor($wr.w / 80)) }
  $pitch = $wr.w / $cols
  return @([int]($wr.x + ($i % $cols) * $pitch + $pitch / 2), [int]($wr.y + 36 + 8 + [math]::Floor($i / $cols) * 96 + 32))
}
function IconCrop($fence, $i, $name) { $p = CellPt $fence $i; return (Crop $name ($p[0] - 24) ($p[1] - 24) 48 48) }
# The kit's ItemPt for a fence that holds This PC or the Recycle Bin: ItemPt puts those first,
# but a fence shows them among the files. In the order the fence shows them, which `item list`
# keeps: by name, or in 手动 order with the icons that came later after the others.
function ItemAt($fence, $name) {
  $order = @(@(TJson item list --fence $fence) | ForEach-Object { $_.name })
  $i = [array]::IndexOf($order, $name)
  if ($i -lt 0) { Log "ItemAt: '$name' not in '$fence' ($($order -join ', '))"; return $null }
  $p = CellPt $fence $i
  Log "ItemAt '$name' in '$fence': index $i of $($order.Count) -> $($p -join ',')"
  return $p
}

# Threads, processes, windows: stop a process's UI thread for a while (a hang), end a process
# with an exit code (a crash's 0xC0000005, Task Manager's 1), a layered window's alpha, how long
# a window takes to answer, Explorer's click-to-open setting, many Recycle Bin items at once.
Add-Type @'
using System; using System.IO; using System.Text; using System.Diagnostics; using System.Runtime.InteropServices;
public static class Th {
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("kernel32.dll")] static extern IntPtr OpenThread(uint access, bool inherit, uint id);
  [DllImport("kernel32.dll")] static extern uint SuspendThread(IntPtr h);
  [DllImport("kernel32.dll")] static extern uint ResumeThread(IntPtr h);
  [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(uint access, bool inherit, uint id);
  [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr h, uint code);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetLayeredWindowAttributes(IntPtr h, out uint key, out byte alpha, out uint flags);
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, string l, uint f, uint t, out IntPtr r);
  [DllImport("shell32.dll")] static extern void SHGetSetSettings(byte[] s, uint mask, bool set);
  public static uint UiThread(IntPtr hwnd) { uint pid; return GetWindowThreadProcessId(hwnd, out pid); }
  public static bool Freeze(uint tid, int ms) {
    var h = OpenThread(0x0002, false, tid); if (h == IntPtr.Zero) return false;
    SuspendThread(h); System.Threading.Thread.Sleep(ms); ResumeThread(h); CloseHandle(h); return true;
  }
  public static IntPtr Suspend(uint tid) { var h = OpenThread(0x0002, false, tid); if (h != IntPtr.Zero) SuspendThread(h); return h; }
  public static int Terminate(int pid, uint code) {
    var h = OpenProcess(0x0001, false, (uint)pid); if (h == IntPtr.Zero) return 0;
    bool ok = TerminateProcess(h, code); CloseHandle(h); return ok ? 1 : 0;
  }
  public static int Alpha(IntPtr h) { uint k, f; byte a; return GetLayeredWindowAttributes(h, out k, out a, out f) ? a : -1; }
  // The longest a window took to answer WM_NULL during `ms` (asked again and again).
  public static int MaxPing(IntPtr h, int ms) {
    var all = Stopwatch.StartNew(); int max = 0;
    while (all.ElapsedMilliseconds < ms) {
      var sw = Stopwatch.StartNew(); IntPtr r; SendMessageTimeoutW(h, 0, IntPtr.Zero, IntPtr.Zero, 0, 20000, out r);
      max = Math.Max(max, (int)sw.ElapsedMilliseconds); System.Threading.Thread.Sleep(20);
    }
    return max;
  }
  public static bool SingleClick() { var s = new byte[64]; SHGetSetSettings(s, 0x80, false); return (s[0] & 0x20) == 0; }
  public static void SetSingleClick(bool on) {
    var s = new byte[64]; SHGetSetSettings(s, 0x80, false);
    s[0] = on ? (byte)(s[0] & ~0x20) : (byte)(s[0] | 0x20);
    SHGetSetSettings(s, 0x80, true);
    IntPtr r; SendMessageTimeoutW((IntPtr)0xFFFF, 0x1A, IntPtr.Zero, "ShellState", 2, 3000, out r);
  }
  // n deleted files in the user's Recycle Bin folder ($I record + $R file, Vista format 2).
  public static int FillBin(string dir, string desk, int n) {
    long ft = DateTime.Now.ToFileTime();
    for (int i = 0; i < n; i++) {
      string id = "H2" + i.ToString("D5") + ".txt", orig = desk + "\\h2-full-" + i + ".txt";
      File.WriteAllText(dir + "\\$R" + id, "x");
      using (var w = new BinaryWriter(File.Create(dir + "\\$I" + id))) {
        w.Write((long)2); w.Write((long)1); w.Write(ft); w.Write(orig.Length + 1);
        w.Write(Encoding.Unicode.GetBytes(orig + "\0"));
      }
    }
    return n;
  }
  public static int EmptyFill(string dir) {
    int n = 0;
    foreach (var f in Directory.GetFiles(dir, "$?H2?????.txt")) { File.Delete(f); n++; }
    return n;
  }
}
'@
function RecycleDir() { return "C:\`$Recycle.Bin\$([Security.Principal.WindowsIdentity]::GetCurrent().User.Value)" }

# An entry of the shell's own popup menu by its name: the classic menu (#32768) or the
# Windows 11 one (UI Automation), centre point or $null.
function ShellMenuPoint($name, $sec = 5) {
  $any = [Windows.Automation.Condition]::TrueCondition
  $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $name)
  $ex = @(Get-Process explorer -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  for ($i = 0; $i -lt $sec * 4; $i++) {
    $m = @([W]::Tops('#32768'))
    if ($m.Count) { $p = MenuPoint $m[0] "^$name"; if ($p) { return $p } }
    foreach ($w in @([Windows.Automation.AutomationElement]::RootElement.FindAll([Windows.Automation.TreeScope]::Children, $any))) {
      $c = $w.Current
      if ($ex -notcontains $c.ProcessId -or $c.ClassName -match '^(Progman|WorkerW|Shell_TrayWnd|Shell_SecondaryTrayWnd|CabinetWClass)$') { continue }
      $e = $w.FindFirst([Windows.Automation.TreeScope]::Descendants, $byName)
      if ($e) { $r = $e.Current.BoundingRectangle; Log "menu item '$name' in $($c.ClassName) at $r"; return @([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2)) }
    }
    Start-Sleep -Milliseconds 250
  }
  return $null
}

# The clipboard's files and its Preferred DropEffect (1 copy, 2 move, -1 none).
function ClipFiles() { return @([Windows.Forms.Clipboard]::GetFileDropList()) }
function ClipEffect() {
  $d = [Windows.Forms.Clipboard]::GetData('Preferred DropEffect')
  if (-not $d) { return -1 }
  $b = New-Object byte[] 4; $d.Read($b, 0, 4) | Out-Null; return [BitConverter]::ToInt32($b, 0)
}

# A desktop shortcut to Notepad.
function DeskLink($name) { $p = Join-Path $PfDesk $name; $s = (New-Object -ComObject WScript.Shell).CreateShortcut($p); $s.TargetPath = "$env:WINDIR\notepad.exe"; $s.Save(); return $p }

# Visible windows of a class owned by a process (Properties dialogs of PecoFence, Open With).
function ProcTops($cls, $proc) { return @([W]::Tops($cls) | Where-Object { [W]::ProcOf($_) -eq $proc }) }

# ---- modules: the settings page over CDP, pixel reads, the user's copy and setup
. "$PSScriptRoot\page.ps1"
. "$PSScriptRoot\pixels.ps1"
. "$PSScriptRoot\user.ps1"

# ---- job start: a known desktop

[G]::Press(0x1B)
CloseWindows
$wp = @('C:\Windows\Web\Wallpaper\Windows\img0.jpg', 'C:\Windows\Web\Screen\img100.jpg') | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($wp) { [G]::SystemParametersInfoW(0x14, 0, $wp, 3) | Out-Null }
# A job before may have ended PecoFence the hard way (ctl.ps1 reset), which leaves Windows'
# desktop icons hidden: show them again, so the build under test meets a user's desktop.
if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Milliseconds 600; Log "desktop icons were hidden by an earlier job; shown again" }
Log ("job start: screen " + [G]::GetSystemMetrics(0) + "x" + [G]::GetSystemMetrics(1) + ", instance $PfInst, build " + (Get-FileHash "$PfBin\pecofence.exe").Hash.Substring(0, 12))
