// A stand-in for a live wallpaper player, for tests of the fences' live wallpaper backdrop.
//   live-wallpaper-standin.exe [--gdi] [--still] [--scene N] [--log FILE]
// Like Wallpaper Engine's scene player it asks Progman for the wallpaper layer (message
// 0x052C), creates a window with WS_EX_NOREDIRECTIONBITMAP, puts it into that layer and
// presents a picture through a Direct3D 11 flip-model swap chain (WARP where there is no GPU).
// PrintWindow has nothing to read in such a window on a real GPU.
//   --gdi     a normal window painted with GDI instead: a player PrintWindow can read
//   --still   presents one frame per scene and then nothing (a paused player)
//   --scene N the scene to start with
// The picture: four vertical bands whose colours depend on the scene, and a white bar that moves
// with every frame (10 a second). Signals: Local\PfLiveNext shows the next scene,
// Local\PfLiveHang makes the program stop answering messages (a player that hangs),
// Local\PfLiveQuit ends it. The log file gets one line per fact (window, parent, driver).
using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;

public static class LiveStandIn {
  delegate IntPtr WndProc(IntPtr h, uint m, IntPtr w, IntPtr l);
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
  struct WNDCLASSEX {
    public uint cbSize, style; public WndProc lpfnWndProc; public int cbClsExtra, cbWndExtra;
    public IntPtr hInstance, hIcon, hCursor, hbrBackground; public string lpszMenuName, lpszClassName; public IntPtr hIconSm;
  }
  [StructLayout(LayoutKind.Sequential)] struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam, lParam; public uint time; public int x, y; }
  [StructLayout(LayoutKind.Sequential)]
  struct SWAPDESC {
    public uint Width, Height; public int Format, Stereo; public uint SampleCount, SampleQuality, BufferUsage, BufferCount;
    public int Scaling, SwapEffect, AlphaMode; public uint Flags;
  }
  [StructLayout(LayoutKind.Sequential)]
  struct BITMAPINFOHEADER {
    public uint biSize; public int biWidth, biHeight; public ushort biPlanes, biBitCount; public uint biCompression, biSizeImage;
    public int biXPelsPerMeter, biYPelsPerMeter; public uint biClrUsed, biClrImportant;
  }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern ushort RegisterClassExW(ref WNDCLASSEX c);
  [DllImport("user32.dll", CharSet = CharSet.Unicode, SetLastError = true)] static extern IntPtr CreateWindowExW(uint ex, string cls, string title, uint style, int x, int y, int w, int h, IntPtr parent, IntPtr menu, IntPtr inst, IntPtr param);
  [DllImport("user32.dll")] static extern IntPtr DefWindowProcW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowW(string cls, string title);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint flags, uint ms, out IntPtr r);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll", SetLastError = true)] static extern IntPtr SetParent(IntPtr child, IntPtr parent);
  [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int w, int cy, uint flags);
  [DllImport("user32.dll")] static extern bool SetLayeredWindowAttributes(IntPtr h, uint key, byte alpha, uint flags);
  [DllImport("user32.dll")] static extern bool PeekMessageW(out MSG m, IntPtr h, uint min, uint max, uint remove);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessageW(ref MSG m);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll")] static extern long GetWindowLongPtrW(IntPtr h, int i);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint flags);
  [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr h, IntPtr dc);
  [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  [DllImport("gdi32.dll")] static extern int SetDIBitsToDevice(IntPtr dc, int x, int y, uint w, uint h, int sx, int sy, uint start, uint lines, IntPtr bits, ref BITMAPINFOHEADER info, uint use);
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern IntPtr GetModuleHandleW(string n);
  [DllImport("d3d11.dll")] static extern int D3D11CreateDevice(IntPtr adapter, int driver, IntPtr software, uint flags, IntPtr levels, uint nLevels, uint sdk, out IntPtr device, out int level, out IntPtr context);

  // COM calls by vtable slot: the .NET Framework compiler on the test boxes has no D3D wrappers.
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate int QueryInterfaceFn(IntPtr self, ref Guid iid, out IntPtr obj);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate int GetAdapterFn(IntPtr self, out IntPtr adapter);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate int GetParentFn(IntPtr self, ref Guid iid, out IntPtr parent);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate int CreateSwapChainForHwndFn(IntPtr self, IntPtr device, IntPtr hwnd, ref SWAPDESC desc, IntPtr fullscreen, IntPtr output, out IntPtr swap);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate int GetBufferFn(IntPtr self, uint index, ref Guid iid, out IntPtr surface);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate int PresentFn(IntPtr self, uint sync, uint flags);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate void UpdateSubresourceFn(IntPtr self, IntPtr resource, uint sub, IntPtr box, IntPtr data, uint rowPitch, uint depthPitch);

  static Delegate Slot(IntPtr obj, int slot, Type t) {
    IntPtr vtbl = Marshal.ReadIntPtr(obj);
    return Marshal.GetDelegateForFunctionPointer(Marshal.ReadIntPtr(vtbl, slot * IntPtr.Size), t);
  }
  static string Cls(IntPtr h) { var s = new System.Text.StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }

  static string logPath;
  static void Log(string line) {
    if (logPath != null) File.AppendAllText(logPath, DateTime.Now.ToString("HH:mm:ss.fff") + " " + line + "\r\n");
  }
  static void Check(int hr, string what) { if (hr < 0) { Log("failed: " + what + " 0x" + hr.ToString("X8")); Environment.Exit(3); } }

  // Scene colours as B, G, R (red, green, blue, purple; no yellow, the colour of the capture
  // border): every scene shifts the four bands by one.
  static readonly byte[][] Bands = {
    new byte[] { 40, 40, 220 }, new byte[] { 60, 180, 40 }, new byte[] { 220, 90, 40 }, new byte[] { 200, 60, 160 },
  };
  static void Paint(byte[] px, int w, int h, int scene, int frame) {
    for (int x = 0; x < w; x++) {
      byte[] c = Bands[(x * 4 / w + scene) % 4];
      px[x * 4] = c[0]; px[x * 4 + 1] = c[1]; px[x * 4 + 2] = c[2]; px[x * 4 + 3] = 255;
    }
    for (int y = 1; y < h; y++) Buffer.BlockCopy(px, 0, px, y * w * 4, w * 4);
    int bar = (frame * 8) % Math.Max(1, h - 12);
    for (int i = bar * w * 4; i < (bar + 12) * w * 4; i++) px[i] = 255;
  }

  static WndProc proc = (h, m, w, l) => DefWindowProcW(h, m, w, l);

  public static int Main(string[] args) {
    bool gdi = false, still = false; int scene = 0;
    for (int i = 0; i < args.Length; i++) {
      if (args[i] == "--gdi") gdi = true;
      else if (args[i] == "--still") still = true;
      else if (args[i] == "--scene" && i + 1 < args.Length) scene = int.Parse(args[++i]);
      else if (args[i] == "--log" && i + 1 < args.Length) logPath = args[++i];
    }
    SetProcessDpiAwarenessContext((IntPtr)(-4));
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1);

    // The wallpaper layer. Windows 11 24H2 and later: Progman holds the icons and a WorkerW
    // with the Windows wallpaper, and a player goes between them. Before: the top-level WorkerW
    // behind the window that holds the icons.
    IntPtr progman = FindWindowW("Progman", null), r;
    SendMessageTimeoutW(progman, 0x052C, (IntPtr)0xD, (IntPtr)1, 0, 1000, out r);
    IntPtr icons = FindWindowExW(progman, IntPtr.Zero, "SHELLDLL_DefView", null);
    IntPtr parent = progman, after = icons;
    if (FindWindowExW(progman, IntPtr.Zero, "WorkerW", null) == IntPtr.Zero) {
      IntPtr layer = IntPtr.Zero;
      EnumWindows((top, l) => {
        if (FindWindowExW(top, IntPtr.Zero, "SHELLDLL_DefView", null) != IntPtr.Zero)
          layer = FindWindowExW(IntPtr.Zero, top, "WorkerW", null);
        return true;
      }, IntPtr.Zero);
      if (layer != IntPtr.Zero) { parent = layer; after = IntPtr.Zero; }
    }

    var wc = new WNDCLASSEX();
    wc.cbSize = (uint)Marshal.SizeOf(typeof(WNDCLASSEX));
    wc.lpfnWndProc = proc; wc.hInstance = GetModuleHandleW(null); wc.lpszClassName = "PfLiveStandIn";
    RegisterClassExW(ref wc);
    // The Direct3D window: a popup with WS_EX_NOREDIRECTIONBITMAP, moved into the layer. The
    // GDI window: a layered child (WS_CHILD | WS_EX_LAYERED), or it would have nothing to
    // paint into inside a parent without a redirection bitmap (Progman since 24H2).
    IntPtr hwnd = gdi
      ? CreateWindowExW(0x00080000u, "PfLiveStandIn", "PfLiveStandIn", 0x40000000u, 0, 0, w, h, parent, IntPtr.Zero, wc.hInstance, IntPtr.Zero)
      : CreateWindowExW(0x00200000u, "PfLiveStandIn", "PfLiveStandIn", 0x80000000u, 0, 0, w, h, IntPtr.Zero, IntPtr.Zero, wc.hInstance, IntPtr.Zero);
    if (hwnd == IntPtr.Zero) { Log("failed: CreateWindowExW " + Marshal.GetLastWin32Error()); return 2; }
    if (gdi) SetLayeredWindowAttributes(hwnd, 0, 255, 2); else SetParent(hwnd, parent);
    // SWP_NOACTIVATE | SWP_SHOWWINDOW, below the icons.
    SetWindowPos(hwnd, after, 0, 0, w, h, 0x0010 | 0x0040);
    Log(string.Format("window=0x{0:X} parent={1} 0x{2:X} root={3} size={4}x{5} exstyle=0x{6:X8} mode={7}{8}",
      hwnd.ToInt64(), Cls(parent), parent.ToInt64(), Cls(GetAncestor(hwnd, 2)), w, h, GetWindowLongPtrW(hwnd, -20), gdi ? "gdi" : "d3d11-flip", still ? " still" : ""));

    IntPtr device = IntPtr.Zero, context = IntPtr.Zero, swap = IntPtr.Zero, back = IntPtr.Zero;
    if (!gdi) {
      int level, hr = D3D11CreateDevice(IntPtr.Zero, 1, IntPtr.Zero, 0x20, IntPtr.Zero, 0, 7, out device, out level, out context);
      string driver = "hardware";
      if (hr < 0) { driver = "warp"; hr = D3D11CreateDevice(IntPtr.Zero, 5, IntPtr.Zero, 0x20, IntPtr.Zero, 0, 7, out device, out level, out context); }
      Check(hr, "D3D11CreateDevice");
      Guid dxgiDevice = new Guid("54ec77fa-1377-44e6-8c32-88fd5f44c84c"), factory2 = new Guid("50c83a1c-e072-4c48-87b0-3630fa36a6d0"), texture2d = new Guid("6f15aaf2-d208-4e89-9ab4-489535d34f9c");
      IntPtr dxgi, adapter, factory;
      Check(((QueryInterfaceFn)Slot(device, 0, typeof(QueryInterfaceFn)))(device, ref dxgiDevice, out dxgi), "IDXGIDevice");
      Check(((GetAdapterFn)Slot(dxgi, 7, typeof(GetAdapterFn)))(dxgi, out adapter), "GetAdapter");
      Check(((GetParentFn)Slot(adapter, 6, typeof(GetParentFn)))(adapter, ref factory2, out factory), "IDXGIFactory2");
      var desc = new SWAPDESC();
      desc.Width = (uint)w; desc.Height = (uint)h; desc.Format = 87; // DXGI_FORMAT_B8G8R8A8_UNORM
      desc.SampleCount = 1; desc.BufferUsage = 0x20; desc.BufferCount = 2; // DXGI_USAGE_RENDER_TARGET_OUTPUT
      desc.SwapEffect = 3; // DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL
      Check(((CreateSwapChainForHwndFn)Slot(factory, 15, typeof(CreateSwapChainForHwndFn)))(factory, device, hwnd, ref desc, IntPtr.Zero, IntPtr.Zero, out swap), "CreateSwapChainForHwnd");
      Check(((GetBufferFn)Slot(swap, 9, typeof(GetBufferFn)))(swap, 0, ref texture2d, out back), "GetBuffer");
      Log("driver=" + driver + " level=0x" + level.ToString("X") + " swap chain: flip sequential, 2 buffers");
    }
    var update = gdi ? null : (UpdateSubresourceFn)Slot(context, 48, typeof(UpdateSubresourceFn));
    var present = gdi ? null : (PresentFn)Slot(swap, 8, typeof(PresentFn));

    var next = new EventWaitHandle(false, EventResetMode.AutoReset, "Local\\PfLiveNext");
    var quit = new EventWaitHandle(false, EventResetMode.AutoReset, "Local\\PfLiveQuit");
    var hang = new EventWaitHandle(false, EventResetMode.AutoReset, "Local\\PfLiveHang");
    var px = new byte[w * h * 4];
    var pin = GCHandle.Alloc(px, GCHandleType.Pinned);
    var info = new BITMAPINFOHEADER();
    info.biSize = 40; info.biWidth = w; info.biHeight = -h; info.biPlanes = 1; info.biBitCount = 32;
    int frame = 0, shown = -1;
    Log("ready scene=" + scene);
    while (!quit.WaitOne(0)) {
      MSG m;
      while (PeekMessageW(out m, IntPtr.Zero, 0, 0, 1)) DispatchMessageW(ref m);
      if (next.WaitOne(0)) { scene++; Log("scene=" + scene); }
      // A player that hangs: its window stays, its thread answers no message.
      if (hang.WaitOne(0)) { Log("hanging"); quit.WaitOne(); break; }
      if (!still || shown != scene) {
        Paint(px, w, h, scene, still ? 0 : frame);
        if (gdi) {
          IntPtr dc = GetDC(hwnd);
          SetDIBitsToDevice(dc, 0, 0, (uint)w, (uint)h, 0, 0, 0, (uint)h, pin.AddrOfPinnedObject(), ref info, 0);
          ReleaseDC(hwnd, dc);
        } else {
          update(context, back, 0, IntPtr.Zero, pin.AddrOfPinnedObject(), (uint)(w * 4), 0);
          int hr = present(swap, 0, 0);
          if (hr < 0 && frame % 50 == 0) Log("Present 0x" + hr.ToString("X8"));
        }
        shown = scene; frame++;
      }
      Thread.Sleep(100);
    }
    Log("quit after " + frame + " frames");
    return 0;
  }
}
