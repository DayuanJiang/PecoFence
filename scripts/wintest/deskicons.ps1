# scripts/wintest/deskicons.ps1: Windows' own desktop icons through the desktop's folder view
# (IShellWindows.FindWindowSW SWC_DESKTOP, the top-level browser's active view, IFolderView2).
# Dot-source it in a steps file: `. C:\pf\kit\deskicons.ps1`.
#   [DeskIcons]::List()               one line per icon: parsing name | x | y (listview px)
#   [DeskIcons]::Place(name, x, y)    moves the icon whose parsing name or file name is `name`
#   [DeskIcons]::Spacing()            Windows' icon grid spacing "cx,cy" (px)
#   [DeskIcons]::Flags() / SetFlags   the view's FWF_ flags (1 自动排列图标, 4 将图标与网格对齐)
#   [DeskIcons]::Origin()             the listview's client origin on the screen "x,y"
#   [DeskIcons]::ListSpacing()        the list view's own spacing (LVM_GETITEMSPACING) "cx,cy"
#   [DeskIcons]::ExplorerDpi()        the scale Explorer started at: Spacing() stays in it after a
#                                     scale change until Explorer restarts (the places do not)
# Positions are the listview's client coordinates, as IFolderView reports them; Origin() turns
# them into screen pixels.

Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class DeskIcons {
  [StructLayout(LayoutKind.Sequential)] public struct PT { public int X; public int Y; }

  [ComImport, Guid("85CB6900-4D95-11CF-960C-0080C7F4EE85"), InterfaceType(ComInterfaceType.InterfaceIsDual)]
  interface IShellWindows {
    [PreserveSig] int get_Count(out int c);
    [PreserveSig] int Item([MarshalAs(UnmanagedType.Struct)] object index, [MarshalAs(UnmanagedType.IDispatch)] out object folder);
    [PreserveSig] int _NewEnum([MarshalAs(UnmanagedType.IUnknown)] out object e);
    [PreserveSig] int Register([MarshalAs(UnmanagedType.IDispatch)] object pid, int hwnd, int swClass, out int cookie);
    [PreserveSig] int RegisterPending(int tid, ref object loc, ref object root, int swClass, out int cookie);
    [PreserveSig] int Revoke(int cookie);
    [PreserveSig] int OnNavigate(int cookie, ref object loc);
    [PreserveSig] int OnActivated(int cookie, bool active);
    [PreserveSig] int FindWindowSW(ref object loc, ref object root, int swClass, out int hwnd, int options, [MarshalAs(UnmanagedType.IDispatch)] out object disp);
  }
  [ComImport, Guid("6D5140C1-7436-11CE-8034-00AA006009FA"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IServiceProvider { [PreserveSig] int QueryService(ref Guid service, ref Guid riid, out IntPtr ppv); }
  [ComImport, Guid("000214E2-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IShellBrowser {
    [PreserveSig] int GetWindow(out IntPtr hwnd);
    [PreserveSig] int ContextSensitiveHelp(bool enter);
    [PreserveSig] int InsertMenusSB(IntPtr a, IntPtr b);
    [PreserveSig] int SetMenuSB(IntPtr a, IntPtr b, IntPtr c);
    [PreserveSig] int RemoveMenusSB(IntPtr a);
    [PreserveSig] int SetStatusTextSB(IntPtr a);
    [PreserveSig] int EnableModelessSB(bool e);
    [PreserveSig] int TranslateAcceleratorSB(IntPtr msg, short id);
    [PreserveSig] int BrowseObject(IntPtr pidl, uint flags);
    [PreserveSig] int GetViewStateStream(uint mode, out IntPtr strm);
    [PreserveSig] int GetControlWindow(uint id, out IntPtr hwnd);
    [PreserveSig] int SendControlMsg(uint id, uint msg, IntPtr w, IntPtr l, out IntPtr ret);
    [PreserveSig] int QueryActiveShellView(out IntPtr view);
  }
  // IShellView: its first method is IOleWindow's GetWindow (no separate query for IOleWindow:
  // Explorer's proxy refuses that).
  [ComImport, Guid("000214E3-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IShellView { [PreserveSig] int GetWindow(out IntPtr hwnd); }
  [ComImport, Guid("1af3a467-214f-4298-908e-06b03e0b39f9"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IFolderView2 {
    [PreserveSig] int GetCurrentViewMode(out uint mode);
    [PreserveSig] int SetCurrentViewMode(uint mode);
    [PreserveSig] int GetFolder(ref Guid riid, out IntPtr ppv);
    [PreserveSig] int Item(int i, out IntPtr pidl);
    [PreserveSig] int ItemCount(uint flags, out int count);
    [PreserveSig] int Items(uint flags, ref Guid riid, out IntPtr ppv);
    [PreserveSig] int GetSelectionMarkedItem(out int i);
    [PreserveSig] int GetFocusedItem(out int i);
    [PreserveSig] int GetItemPosition(IntPtr pidl, out PT pt);
    [PreserveSig] int GetSpacing(ref PT pt);
    [PreserveSig] int GetDefaultSpacing(out PT pt);
    [PreserveSig] int GetAutoArrange();
    [PreserveSig] int SelectItem(int i, uint flags);
    [PreserveSig] int SelectAndPositionItems(uint cidl, [In, MarshalAs(UnmanagedType.LPArray)] IntPtr[] apidl, [In, MarshalAs(UnmanagedType.LPArray)] PT[] apt, uint flags);
    [PreserveSig] int SetGroupBy(IntPtr key, bool asc);
    [PreserveSig] int GetGroupBy(IntPtr key, out bool asc);
    [PreserveSig] int SetViewProperty(IntPtr pidl, IntPtr key, IntPtr v);
    [PreserveSig] int GetViewProperty(IntPtr pidl, IntPtr key, IntPtr v);
    [PreserveSig] int SetTileViewProperties(IntPtr pidl, IntPtr props);
    [PreserveSig] int SetExtendedTileViewProperties(IntPtr pidl, IntPtr props);
    [PreserveSig] int SetText(int type, IntPtr text);
    [PreserveSig] int SetCurrentFolderFlags(uint mask, uint flags);
    [PreserveSig] int GetCurrentFolderFlags(out uint flags);
    [PreserveSig] int GetSortColumnCount(out int c);
    [PreserveSig] int SetSortColumns(IntPtr cols, int n);
    [PreserveSig] int GetSortColumns(IntPtr cols, int n);
    [PreserveSig] int GetItem(int i, ref Guid riid, out IntPtr ppv);
    [PreserveSig] int GetVisibleItem(int start, bool previous, out int i);
    [PreserveSig] int GetSelectedItem(int start, out int i);
    [PreserveSig] int GetSelection(bool none, out IntPtr items);
    [PreserveSig] int GetSelectionState(IntPtr pidl, out uint flags);
    [PreserveSig] int InvokeVerbOnSelection(IntPtr verb);
    [PreserveSig] int SetViewModeAndIconSize(int mode, int size);
    [PreserveSig] int GetViewModeAndIconSize(out int mode, out int size);
  }

  [DllImport("shell32.dll")] static extern int SHGetNameFromIDList(IntPtr pidl, uint sigdn, out IntPtr name);
  [DllImport("ole32.dll")] static extern void CoTaskMemFree(IntPtr p);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowExW(IntPtr parent, IntPtr after, string cls, string title);
  [DllImport("user32.dll")] static extern bool ClientToScreen(IntPtr h, ref PT p);

  static IFolderView2 View(out IntPtr defView) {
    defView = IntPtr.Zero;
    var sw = (IShellWindows)Activator.CreateInstance(Type.GetTypeFromCLSID(new Guid("9BA05972-F6A8-11CF-A442-00A0C90A8F39")));
    object loc = 0, root = null, disp; int hwnd;
    int hr = sw.FindWindowSW(ref loc, ref root, 8, out hwnd, 1, out disp);
    if (hr != 0 || disp == null) throw new Exception("FindWindowSW 0x" + hr.ToString("X"));
    var sp = (IServiceProvider)disp;
    Guid sid = new Guid("4C96BE40-915C-11CF-99D3-00AA004AE837"), iid = new Guid("000214E2-0000-0000-C000-000000000046");
    IntPtr pb;
    hr = sp.QueryService(ref sid, ref iid, out pb);
    if (hr != 0) throw new Exception("QueryService 0x" + hr.ToString("X"));
    var browser = (IShellBrowser)Marshal.GetObjectForIUnknown(pb); Marshal.Release(pb);
    IntPtr pv;
    hr = browser.QueryActiveShellView(out pv);
    if (hr != 0) throw new Exception("QueryActiveShellView 0x" + hr.ToString("X"));
    object view = Marshal.GetObjectForIUnknown(pv); Marshal.Release(pv);
    ((IShellView)view).GetWindow(out defView);
    return (IFolderView2)view;
  }

  static string Name(IntPtr pidl, uint sigdn) {
    IntPtr p;
    if (SHGetNameFromIDList(pidl, sigdn, out p) != 0) return "";
    string s = Marshal.PtrToStringUni(p); CoTaskMemFree(p); return s;
  }

  public static string[] List() {
    IntPtr dv; var v = View(out dv);
    int n; v.ItemCount(2, out n);
    var o = new List<string>();
    for (int i = 0; i < n; i++) {
      IntPtr pidl;
      if (v.Item(i, out pidl) != 0) continue;
      PT pt; v.GetItemPosition(pidl, out pt);
      o.Add(Name(pidl, 0x80028000) + "|" + pt.X + "|" + pt.Y + "|" + Name(pidl, 0));
      CoTaskMemFree(pidl);
    }
    return o.ToArray();
  }

  // The icon whose parsing name ends with `name` (a file name, or ::{CLSID}) goes to (x, y).
  public static string Place(string name, int x, int y) {
    IntPtr dv; var v = View(out dv);
    int n; v.ItemCount(2, out n);
    for (int i = 0; i < n; i++) {
      IntPtr pidl;
      if (v.Item(i, out pidl) != 0) continue;
      string pn = Name(pidl, 0x80028000);
      if (pn.EndsWith("\\" + name, StringComparison.OrdinalIgnoreCase) || string.Equals(pn, name, StringComparison.OrdinalIgnoreCase)) {
        var pt = new PT { X = x, Y = y };
        int hr = v.SelectAndPositionItems(1, new[] { pidl }, new[] { pt }, 0x80);
        PT got; v.GetItemPosition(pidl, out got);
        CoTaskMemFree(pidl);
        return "hr=0x" + hr.ToString("X") + " at " + got.X + "," + got.Y;
      }
      CoTaskMemFree(pidl);
    }
    return "not found";
  }

  public static string Spacing() {
    IntPtr dv; var v = View(out dv);
    var pt = new PT(); v.GetSpacing(ref pt);
    int mode, size; v.GetViewModeAndIconSize(out mode, out size);
    return pt.X + "," + pt.Y + " icon " + size + " mode " + mode + " autoArrange " + (v.GetAutoArrange() == 0);
  }

  public static uint Flags() { IntPtr dv; var v = View(out dv); uint f; v.GetCurrentFolderFlags(out f); return f; }
  public static int SetFlags(uint mask, uint flags) { IntPtr dv; var v = View(out dv); return v.SetCurrentFolderFlags(mask, flags); }

  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  // The list view's own icon spacing (LVM_GETITEMSPACING) "cx,cy".
  public static string ListSpacing() {
    IntPtr dv; View(out dv);
    var lv = FindWindowExW(dv, IntPtr.Zero, "SysListView32", null);
    IntPtr r; SendMessageTimeoutW(lv, 0x1033, IntPtr.Zero, IntPtr.Zero, 2, 2000, out r);
    long v = r.ToInt64(); return (v & 0xFFFF) + "," + ((v >> 16) & 0xFFFF);
  }

  [DllImport("user32.dll")] static extern IntPtr GetShellWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("kernel32.dll")] static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetSystemDpiForProcess(IntPtr p);
  public static uint ExplorerDpi() {
    uint pid; GetWindowThreadProcessId(GetShellWindow(), out pid);
    var p = OpenProcess(0x1000, false, pid); if (p == IntPtr.Zero) return 0;
    uint d = GetSystemDpiForProcess(p); CloseHandle(p); return d;
  }

  public static string Origin() {
    IntPtr dv; View(out dv);
    var lv = FindWindowExW(dv, IntPtr.Zero, "SysListView32", null);
    var p = new PT(); ClientToScreen(lv, ref p);
    return p.X + "," + p.Y;
  }
}
'@
