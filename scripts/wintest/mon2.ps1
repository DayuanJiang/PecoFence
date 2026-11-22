# Two monitors in a Hyper-V guest (the Hyper-V display and the virtual display driver of
# hyperv/vdd-prep.ps1): scale, resolution and arrangement per monitor, clicks and shots on any
# monitor. Dot-source it from a steps file: . C:\pf\kit\mon2.ps1

Add-Type -ReferencedAssemblies System.Drawing @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class Mon2 {
    [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr ctx);
    [StructLayout(LayoutKind.Sequential)] public struct LUID { public uint Low; public int High; }
    [StructLayout(LayoutKind.Sequential)] struct PATH_SOURCE { public LUID adapterId; public uint id; public uint modeInfoIdx; public uint statusFlags; }
    [StructLayout(LayoutKind.Sequential)] struct PATH_TARGET { public LUID adapterId; public uint id; public uint modeInfoIdx; public uint outputTechnology; public uint rotation; public uint scaling; public uint refreshNum; public uint refreshDen; public uint scanLineOrdering; public int targetAvailable; public uint statusFlags; }
    [StructLayout(LayoutKind.Sequential)] struct PATH_INFO { public PATH_SOURCE src; public PATH_TARGET tgt; public uint flags; }
    [StructLayout(LayoutKind.Sequential, Size = 64)] struct MODE_INFO { public uint infoType; public uint id; public LUID adapterId; }
    [StructLayout(LayoutKind.Sequential)] struct HEADER { public int type; public uint size; public LUID adapterId; public uint id; }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] struct SOURCE_NAME { public HEADER h; [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string name; }
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)] struct TARGET_NAME { public HEADER h; public uint flags; public uint outputTechnology; public ushort edidMan; public ushort edidProduct; public uint connector; [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 64)] public string friendly; [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)] public string path; }
    [StructLayout(LayoutKind.Sequential)] struct DPI_GET { public HEADER h; public int min; public int cur; public int max; }
    [StructLayout(LayoutKind.Sequential)] struct DPI_SET { public HEADER h; public int rel; }
    [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint numPaths, out uint numModes);
    [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint numPaths, [Out] PATH_INFO[] paths, ref uint numModes, [Out] MODE_INFO[] modes, IntPtr topologyId);
    [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref SOURCE_NAME p);
    [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref TARGET_NAME p);
    [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref DPI_GET p);
    [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref DPI_SET p);
    [DllImport("user32.dll")] static extern int SetDisplayConfig(uint numPaths, IntPtr paths, uint numModes, IntPtr modes, uint flags);
    static readonly int[] Vals = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

    /// Active sources: "gdi|friendly name|scale %" per line; with `gdi` and `percent` > 0 sets that one's scale.
    public static string Dpi(string gdi, int percent) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        uint np, nm;
        GetDisplayConfigBufferSizes(2, out np, out nm);
        var paths = new PATH_INFO[np]; var modes = new MODE_INFO[nm];
        if (QueryDisplayConfig(2, ref np, paths, ref nm, modes, IntPtr.Zero) != 0) return "query failed";
        var sb = new StringBuilder();
        for (int i = 0; i < np; i++) {
            var p = paths[i];
            var sn = new SOURCE_NAME(); sn.h.type = 1; sn.h.size = (uint)Marshal.SizeOf(typeof(SOURCE_NAME)); sn.h.adapterId = p.src.adapterId; sn.h.id = p.src.id;
            DisplayConfigGetDeviceInfo(ref sn);
            var tn = new TARGET_NAME(); tn.h.type = 2; tn.h.size = (uint)Marshal.SizeOf(typeof(TARGET_NAME)); tn.h.adapterId = p.tgt.adapterId; tn.h.id = p.tgt.id;
            DisplayConfigGetDeviceInfo(ref tn);
            var g = new DPI_GET(); g.h.type = -3; g.h.size = (uint)Marshal.SizeOf(typeof(DPI_GET)); g.h.adapterId = p.src.adapterId; g.h.id = p.src.id;
            int rc = DisplayConfigGetDeviceInfo(ref g);
            int rec = Math.Abs(g.min);
            string cur = rc == 0 && rec + g.cur >= 0 && rec + g.cur < Vals.Length ? Vals[rec + g.cur].ToString() : "?";
            sb.AppendFormat("{0}|{1}|{2}\n", sn.name, tn.friendly, cur);
            if (percent > 0 && string.Equals(sn.name, gdi, StringComparison.OrdinalIgnoreCase)) {
                var s = new DPI_SET(); s.h.type = -4; s.h.size = (uint)Marshal.SizeOf(typeof(DPI_SET)); s.h.adapterId = p.src.adapterId; s.h.id = p.src.id;
                s.rel = Array.IndexOf(Vals, percent) - rec;
                DisplayConfigSetDeviceInfo(ref s);
            }
        }
        return sb.ToString();
    }

    /// SDC_TOPOLOGY_INTERNAL 1, EXTEND 4, EXTERNAL 8 (with SDC_APPLY).
    public static int Topology(uint t) { return SetDisplayConfig(0, IntPtr.Zero, 0, IntPtr.Zero, 0x80 | t); }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    struct DEVMODE {
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmDeviceName;
        public short dmSpecVersion, dmDriverVersion, dmSize, dmDriverExtra;
        public int dmFields, dmPositionX, dmPositionY, dmDisplayOrientation, dmDisplayFixedOutput;
        public short dmColor, dmDuplex, dmYResolution, dmTTOption, dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 32)] public string dmFormName;
        public short dmLogPixels;
        public int dmBitsPerPel, dmPelsWidth, dmPelsHeight, dmDisplayFlags, dmDisplayFrequency, dmICMMethod, dmICMIntent, dmMediaType, dmDitherType, dmReserved1, dmReserved2, dmPanningWidth, dmPanningHeight;
    }
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern bool EnumDisplaySettingsW(string dev, int mode, ref DEVMODE dm);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int ChangeDisplaySettingsExW(string dev, ref DEVMODE dm, IntPtr hwnd, uint flags, IntPtr param);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int ChangeDisplaySettingsExW(string dev, IntPtr dm, IntPtr hwnd, uint flags, IntPtr param);

    public static int Res(string gdi, int w, int h) {
        var dm = new DEVMODE(); dm.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
        if (!EnumDisplaySettingsW(gdi, -1, ref dm)) return -100;
        dm.dmPelsWidth = w; dm.dmPelsHeight = h; dm.dmFields = 0x80000 | 0x100000;
        return ChangeDisplaySettingsExW(gdi, ref dm, IntPtr.Zero, 1, IntPtr.Zero);
    }

    /// `first` becomes the primary monitor at (0, 0) and `second` stands to its right.
    public static string Arrange(string first, string second) {
        var a = new DEVMODE(); a.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
        var b = new DEVMODE(); b.dmSize = (short)Marshal.SizeOf(typeof(DEVMODE));
        if (!EnumDisplaySettingsW(first, -1, ref a) || !EnumDisplaySettingsW(second, -1, ref b)) return "enum failed";
        a.dmPositionX = 0; a.dmPositionY = 0; a.dmFields = 0x20;
        b.dmPositionX = a.dmPelsWidth; b.dmPositionY = 0; b.dmFields = 0x20;
        // CDS_UPDATEREGISTRY | CDS_NORESET (| CDS_SET_PRIMARY), then one apply.
        int r1 = ChangeDisplaySettingsExW(first, ref a, IntPtr.Zero, 0x10000000 | 0x10 | 1, IntPtr.Zero);
        int r2 = ChangeDisplaySettingsExW(second, ref b, IntPtr.Zero, 0x10000000 | 1, IntPtr.Zero);
        int r3 = ChangeDisplaySettingsExW(null, IntPtr.Zero, IntPtr.Zero, 0, IntPtr.Zero);
        return r1 + "," + r2 + "," + r3;
    }

    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
    /// A click at a point of the virtual screen (physical px, any monitor): `right` for the
    /// right button. The kit's own clicks reach the primary monitor only.
    public static void Click(int x, int y, bool right) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        SetCursorPos(x, y); System.Threading.Thread.Sleep(80);
        mouse_event(right ? 8u : 2u, 0, 0, 0, UIntPtr.Zero); System.Threading.Thread.Sleep(40);
        mouse_event(right ? 16u : 4u, 0, 0, 0, UIntPtr.Zero);
    }
    public static void MoveTo(int x, int y) { SetThreadDpiAwarenessContext(new IntPtr(-4)); SetCursorPos(x, y); }
    public static void Button(bool down) { mouse_event(down ? 2u : 4u, 0, 0, 0, UIntPtr.Zero); }
    /// The whole virtual screen (every monitor) as a PNG.
    public static void ShotAll(string path) {
        SetThreadDpiAwarenessContext(new IntPtr(-4));
        int x = GetSystemMetrics(76), y = GetSystemMetrics(77), w = GetSystemMetrics(78), h = GetSystemMetrics(79);
        using (var b = new System.Drawing.Bitmap(w, h)) {
            using (var g = System.Drawing.Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, b.Size);
            b.Save(path);
        }
    }
}
'@

# Waits for both monitors, gives A (the virtual one) `$W` x `$H` at `$ScaleA` % as the primary
# monitor at 0,0 and B (the Hyper-V display) `$ScaleB` % to its right. Returns @(gdiA, gdiB).
function Mon2Pair([int]$W = 2560, [int]$H = 1440, [int]$ScaleA = 150, [int]$ScaleB = 125) {
  $ms = ''
  for ($i = 0; $i -lt 30; $i++) { $ms = [Mon2]::Dpi('', 0); if (@($ms.Trim() -split "`n").Count -ge 2) { break }; Start-Sleep -Seconds 1 }
  $lines = @($ms.Trim() -split "`n" | ForEach-Object { $_.Trim() })
  $vdd = @($lines | Where-Object { $_ -notmatch 'DISPLAY1\|' }) | Select-Object -First 1
  $hv = @($lines | Where-Object { $_ -match 'DISPLAY1\|' }) | Select-Object -First 1
  Check 'the guest has two monitors (the Hyper-V display and the virtual one)' ($lines.Count -ge 2 -and $vdd -and $hv) ($lines -join ' / ')
  $a = ($vdd -split '\|')[0]; $b = ($hv -split '\|')[0]
  Log "res A: $([Mon2]::Res($a, $W, $H))"; Start-Sleep -Seconds 3
  [Mon2]::Dpi($a, $ScaleA) | Out-Null; [Mon2]::Dpi($b, $ScaleB) | Out-Null; Start-Sleep -Seconds 3
  Log "arrange: $([Mon2]::Arrange($a, $b))"; Start-Sleep -Seconds 4
  Log "monitors: $(([Mon2]::Dpi('', 0)).Trim() -replace "`n", ' / ')"
  return @($a, $b)
}
# A shot of every monitor (the kit's Shot takes the primary one only).
function ShotAll($name) {
  Start-Sleep -Milliseconds 400
  $safe = $name -replace '[^\w.-]', '_'
  [Mon2]::ShotAll("$PfShots\$safe.png")
  Log "shot $safe"
}
