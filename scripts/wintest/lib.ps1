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
function PfLogErrors() { return @(PfLogAll | Where-Object { $_ -match ' ERROR |panicked' }) }

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

# ---- job start: a known desktop

[G]::Press(0x1B)
CloseWindows
$wp = @('C:\Windows\Web\Wallpaper\Windows\img0.jpg', 'C:\Windows\Web\Screen\img100.jpg') | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($wp) { [G]::SystemParametersInfoW(0x14, 0, $wp, 3) | Out-Null }
Log ("job start: screen " + [G]::GetSystemMetrics(0) + "x" + [G]::GetSystemMetrics(1) + ", instance $PfInst, build " + (Get-FileHash "$PfBin\pecofence.exe").Hash.Substring(0, 12))
