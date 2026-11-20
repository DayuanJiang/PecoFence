// A stand-in for Bing Wallpaper's desktop widget (GitHub #247): two windows of another program
// in the desktop's wallpaper layer that draw nothing of the wallpaper.
//   wallpaper-overlay-standin.exe [--prefix NAME] [--log FILE]
// Bing Wallpaper 1.1.473 puts them into Progman, above Explorer's wallpaper WorkerW:
//   <prefix>Visual    over the whole monitor, WS_EX_NOREDIRECTIONBITMAP | WS_EX_CONTROLPARENT |
//                     WS_EX_TRANSPARENT (0x210020), drawn through DirectComposition
//   <prefix>Position  over the work area, WS_EX_CONTROLPARENT (0x10000)
// The default prefix is Bing's own, BingWallpaper_Widget_. Neither window draws anything here
// (Bing's draw a small caption only), and both let clicks through. They go right below the
// icons, where a player goes. Local\PfOverlayQuit ends the program (also after 10 minutes).
// The log file gets one line per window (handle, parent, size, extended style) and "ready".
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;

public static class OverlayStandIn {
  delegate IntPtr WndProc(IntPtr h, uint m, IntPtr w, IntPtr l);
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct WNDCLASSEX {
    public uint cbSize, style; public WndProc lpfnWndProc; public int cbClsExtra, cbWndExtra;
    public IntPtr hInstance, hIcon, hCursor, hbrBackground; public string lpszMenuName, lpszClassName; public IntPtr hIconSm;
  }
  [StructLayout(LayoutKind.Sequential)] struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam, lParam; public uint time; public int x, y; }
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern ushort RegisterClassExW(ref WNDCLASSEX c);
  [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)] static extern IntPtr CreateWindowExW(uint ex, string cls, string title, uint style, int x, int y, int w, int h, IntPtr parent, IntPtr menu, IntPtr inst, IntPtr param);
  [DllImport("user32.dll")] static extern IntPtr DefWindowProcW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
  [DllImport("user32.dll", SetLastError = true)] static extern IntPtr SetParent(IntPtr child, IntPtr parent);
  [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int cy, uint flags);
  [DllImport("user32.dll")] static extern bool PeekMessageW(out MSG m, IntPtr h, uint min, uint max, uint remove);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessageW(ref MSG m);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint action, uint param, ref RECT r, uint ini);
  [DllImport("user32.dll")] static extern long GetWindowLongPtrW(IntPtr h, int i);
  [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern IntPtr GetModuleHandleW(string n);

  static string logPath;
  static void Log(string line) {
    if (logPath != null) File.AppendAllText(logPath, DateTime.Now.ToString("HH:mm:ss.fff") + " " + line + "\r\n");
  }
  // Clicks go through to what is below (WM_NCHITTEST answers HTTRANSPARENT); nothing is painted.
  static WndProc proc = (h, m, w, l) => m == 0x0084 ? (IntPtr)(-1) : m == 0x0014 ? (IntPtr)1 : DefWindowProcW(h, m, w, l);

  static IntPtr Make(string cls, uint ex, int x, int y, int w, int h, IntPtr progman, IntPtr after) {
    var wc = new WNDCLASSEX();
    wc.cbSize = (uint)Marshal.SizeOf(typeof(WNDCLASSEX));
    wc.lpfnWndProc = proc; wc.hInstance = GetModuleHandleW(null); wc.lpszClassName = cls;
    RegisterClassExW(ref wc);
    // A popup moved into Progman, as the players and Bing's widget do.
    IntPtr hwnd = CreateWindowExW(ex, cls, "", 0x80000000u, x, y, w, h, IntPtr.Zero, IntPtr.Zero, wc.hInstance, IntPtr.Zero);
    if (hwnd == IntPtr.Zero) { Log("failed: CreateWindowExW " + cls + " " + Marshal.GetLastWin32Error()); Environment.Exit(2); }
    SetParent(hwnd, progman);
    // SWP_NOACTIVATE | SWP_SHOWWINDOW, right below the icons.
    SetWindowPos(hwnd, after, x, y, w, h, 0x0010 | 0x0040);
    Log(string.Format("window=0x{0:X} class={1} parent=0x{2:X} rect={3},{4},{5}x{6} exstyle=0x{7:X8}",
      hwnd.ToInt64(), cls, progman.ToInt64(), x, y, w, h, GetWindowLongPtrW(hwnd, -20)));
    return hwnd;
  }

  public static int Main(string[] args) {
    string prefix = "BingWallpaper_Widget_";
    for (int i = 0; i < args.Length; i++) {
      if (args[i] == "--prefix" && i + 1 < args.Length) prefix = args[++i];
      else if (args[i] == "--log" && i + 1 < args.Length) logPath = args[++i];
    }
    SetProcessDpiAwarenessContext((IntPtr)(-4));
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1);
    var work = new RECT();
    SystemParametersInfoW(0x0030, 0, ref work, 0); // SPI_GETWORKAREA
    IntPtr progman = FindWindowW("Progman", null);
    IntPtr icons = FindWindowExW(progman, IntPtr.Zero, "SHELLDLL_DefView", null);
    // Made in this order, the visual ends up first, as in Bing's.
    Make(prefix + "Position", 0x00010000u, work.Left, work.Top, work.Right - work.Left, work.Bottom - work.Top, progman, icons);
    Make(prefix + "Visual", 0x00210020u, 0, 0, w, h, progman, icons);
    var quit = new EventWaitHandle(false, EventResetMode.AutoReset, "Local\\PfOverlayQuit");
    Log("ready");
    var until = DateTime.Now.AddMinutes(10);
    while (!quit.WaitOne(0) && DateTime.Now < until) {
      MSG m;
      while (PeekMessageW(out m, IntPtr.Zero, 0, 0, 1)) DispatchMessageW(ref m);
      Thread.Sleep(50);
    }
    Log("quit");
    return 0;
  }
}
