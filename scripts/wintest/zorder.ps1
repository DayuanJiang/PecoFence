# scripts/wintest/zorder.ps1: a z-order sampler for show desktop (Win+D) and similar shell moves.
# [Zs]::Run presses Win+<key> and records, as fast as EnumWindows allows, which of the fences, the
# desktop windows and the test's app windows (Notepad, Explorer) are above which, with frames of
# the screen at the same time; ZsStats sums up a run. Steps dot-source it: . C:\pf\kit\zorder.ps1
Add-Type -ReferencedAssemblies System.Drawing @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
public static class Zs {
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", EntryPoint = "GetWindowLongW")] static extern int GetWindowLong(IntPtr h, int i);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern IntPtr GetShellWindow();
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr p, IntPtr a, string c, string t);
  [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, int a, out int v, int s);
  [DllImport("user32.dll")] static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  public static HashSet<string> Apps = new HashSet<string> { "Notepad", "CabinetWClass" };

  // Top of the z-order first: f/F a fence (F topmost, runs as f7), P Progman, W the WorkerW with
  // the icons, w another WorkerW, A an app window of the test (m when minimized), then m<n> the
  // count of minimized ones and the foreground class.
  public static string Sig() {
    var sb = new StringBuilder();
    uint shellPid; GetWindowThreadProcessId(GetShellWindow(), out shellPid);
    string run = null; int n = 0; int minimized = 0;
    EnumProc cb = (h, l) => {
      if (!IsWindowVisible(h)) return true;
      string cls = Cls(h);
      int c; bool cloaked = DwmGetWindowAttribute(h, 14, out c, 4) == 0 && c != 0;
      if (cloaked) return true;
      uint pid; GetWindowThreadProcessId(h, out pid);
      bool top = (GetWindowLong(h, -20) & 8) != 0;
      string tok = null;
      if (cls == "PecoFence.Fence") tok = top ? "F" : "f";
      else if (pid == shellPid && cls == "Progman") tok = top ? "P^" : "P";
      else if (pid == shellPid && cls == "WorkerW") tok = (FindWindowExW(h, IntPtr.Zero, "SHELLDLL_DefView", null) != IntPtr.Zero ? "W" : "w") + (top ? "^" : "");
      else if (Apps.Contains(cls)) { if (IsIconic(h)) { minimized++; tok = "m"; } else tok = top ? "a" : "A"; }
      if (tok == null) return true;
      if (tok == run && (tok == "f" || tok == "F")) { n++; return true; }
      if (run != null) sb.Append(run).Append(n > 1 ? n.ToString() : "").Append(' ');
      run = tok; n = 1;
      return true;
    };
    EnumWindows(cb, IntPtr.Zero);
    GC.KeepAlive(cb);
    if (run != null) sb.Append(run).Append(n > 1 ? n.ToString() : "").Append(' ');
    sb.Append("m").Append(minimized).Append(" fg=").Append(Cls(GetForegroundWindow()));
    return sb.ToString();
  }

  // Presses Win+<vk> after delayMs and samples for ms: "t sig" lines on every change, frames
  // (half size) every ~20 ms saved as <prefix>-<t>.jpg when prefix is not null.
  delegate void WinEventProc(IntPtr hook, uint ev, IntPtr hwnd, int obj, int child, uint thread, uint time);
  [DllImport("user32.dll")] static extern IntPtr SetWinEventHook(uint min, uint max, IntPtr mod, WinEventProc f, uint pid, uint tid, uint flags);
  [DllImport("user32.dll")] static extern bool UnhookWinEvent(IntPtr h);
  [DllImport("user32.dll")] static extern bool PeekMessageW(out MSG m, IntPtr h, uint a, uint b, uint r);
  [DllImport("user32.dll")] static extern bool TranslateMessage(ref MSG m);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessageW(ref MSG m);
  [DllImport("user32.dll")] static extern uint MsgWaitForMultipleObjects(uint n, IntPtr[] h, bool all, uint ms, uint mask);
  [StructLayout(LayoutKind.Sequential)] struct MSG { public IntPtr h; public uint m; public IntPtr w, l; public uint t; public int x, y; }
  // With Hook set, Run also logs these WinEvents (out of context): "t EV <name> <class>".
  public static bool Hook = false;
  public static string[] Run(int ms, int delayMs, byte vk, string prefix, int maxFrames) {
    var lines = new List<string>();
    var sw = Stopwatch.StartNew();
    bool stop = false;
    Thread hk = null;
    if (Hook) {
      hk = new Thread(() => {
        WinEventProc cb = (hook, ev, hwnd, obj, child, thread, time) => {
          if (obj != 0 && ev != 0x8004) return;
          string name = ev == 3 ? "foreground" : ev == 0x16 ? "minstart" : ev == 0x17 ? "minend" : ev == 0x8004 ? "reorder" : ev.ToString("x");
          uint pid; GetWindowThreadProcessId(hwnd, out pid);
          lock (lines) lines.Add(sw.Elapsed.TotalMilliseconds.ToString("0.0") + " EV " + name + " " + Cls(hwnd) + " obj=" + obj + " pid=" + pid + " tid=" + thread);
        };
        var h1 = SetWinEventHook(3, 3, IntPtr.Zero, cb, 0, 0, 2);
        var h2 = SetWinEventHook(0x16, 0x17, IntPtr.Zero, cb, 0, 0, 2);
        var h3 = SetWinEventHook(0x8004, 0x8004, IntPtr.Zero, cb, 0, 0, 2);
        MSG m;
        while (!stop) {
          MsgWaitForMultipleObjects(0, null, false, 5, 0x4FF);
          while (PeekMessageW(out m, IntPtr.Zero, 0, 0, 1)) { TranslateMessage(ref m); DispatchMessageW(ref m); }
        }
        UnhookWinEvent(h1); UnhookWinEvent(h2); UnhookWinEvent(h3);
        GC.KeepAlive(cb);
      });
      hk.Start();
      Thread.Sleep(50);
    }
    long pressedAt = -1;
    var key = new Thread(() => {
      Thread.Sleep(delayMs);
      pressedAt = sw.ElapsedMilliseconds;
      keybd_event(0x5B, 0, 0, UIntPtr.Zero); Thread.Sleep(20);
      keybd_event(vk, 0, 0, UIntPtr.Zero); Thread.Sleep(20);
      keybd_event(vk, 0, 2, UIntPtr.Zero); Thread.Sleep(20);
      keybd_event(0x5B, 0, 2, UIntPtr.Zero);
    });
    var z = new Thread(() => {
      string last = null;
      while (!stop) {
        string s = Sig();
        if (s != last) { lock (lines) lines.Add(sw.Elapsed.TotalMilliseconds.ToString("0.0") + " " + s); last = s; }
        Thread.Sleep(1);
      }
    });
    z.Priority = ThreadPriority.AboveNormal;
    z.Start(); key.Start();
    var frames = new List<KeyValuePair<long, Bitmap>>();
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1);
    while (sw.ElapsedMilliseconds < ms) {
      if (prefix != null && frames.Count < maxFrames) {
        var full = new Bitmap(w, h, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(full)) g.CopyFromScreen(0, 0, 0, 0, new Size(w, h));
        long t = sw.ElapsedMilliseconds;
        var half = new Bitmap(w / 2, h / 2, PixelFormat.Format24bppRgb);
        using (var g = Graphics.FromImage(half)) { g.InterpolationMode = System.Drawing.Drawing2D.InterpolationMode.Bilinear; g.DrawImage(full, 0, 0, w / 2, h / 2); }
        full.Dispose();
        frames.Add(new KeyValuePair<long, Bitmap>(t, half));
      } else Thread.Sleep(5);
    }
    stop = true; z.Join(); key.Join(); if (hk != null) hk.Join();
    foreach (var f in frames) { f.Value.Save(prefix + "-" + f.Key.ToString("0000") + ".jpg", ImageFormat.Jpeg); f.Value.Dispose(); }
    lock (lines) { lines.Insert(0, "pressed " + pressedAt); return lines.ToArray(); }
  }
}
'@

# ZsStats: from Run's lines, the time after the key press (ms) during which the fences were
# covered (a desktop window above a fence), topmost, or drawn over an app window that is itself
# above the desktop (not minimized, not hidden by the raised desktop). $RunMs = Run's length.
function ZsStats($lines, $RunMs = 1600) {
  $pressed = [double](($lines[0] -split ' ')[1])
  $rows = @($lines | Select-Object -Skip 1 | Where-Object { $_ -notmatch ' EV ' } | ForEach-Object { $t, $rest = $_ -split ' ', 2; [pscustomobject]@{ t = [double]$t; sig = $rest } })
  $covered = 0.0; $topmost = 0.0; $over = 0.0
  for ($i = 0; $i -lt $rows.Count; $i++) {
    $a = [Math]::Max($rows[$i].t, $pressed)
    $b = if ($i + 1 -lt $rows.Count) { $rows[$i + 1].t } else { [double]$RunMs }
    if ($b -le $a) { continue }
    $kinds = @(($rows[$i].sig -split ' fg=')[0] -split ' ' | Where-Object { $_ -and $_ -notmatch '^m\d+$' } | ForEach-Object { $_.Substring(0, 1) })
    $desk = $kinds.Count; for ($k = $kinds.Count - 1; $k -ge 0; $k--) { if ($kinds[$k] -ceq 'P' -or $kinds[$k] -ceq 'W') { $desk = $k } }
    $firstFence = -1; $lastFence = -1; $lastApp = -1
    for ($k = 0; $k -lt $kinds.Count; $k++) {
      if ($kinds[$k] -ceq 'f' -or $kinds[$k] -ceq 'F') { if ($firstFence -lt 0) { $firstFence = $k }; $lastFence = $k }
      if (($kinds[$k] -ceq 'A' -or $kinds[$k] -ceq 'a') -and $k -lt $desk) { $lastApp = $k }
    }
    if ($lastFence -gt $desk) { $covered += $b - $a }
    if ($kinds -ccontains 'F') { $topmost += $b - $a }
    if ($firstFence -ge 0 -and $lastApp -gt $firstFence) { $over += $b - $a }
  }
  return [pscustomobject]@{ covered = [int]$covered; topmost = [int]$topmost; over = [int]$over; changes = $rows.Count }
}
