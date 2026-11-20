# Feedback #194: a fence moved onto the other monitor stays there when a monitor goes away and
# comes back. Two monitors as the reporter has them: A (2560x1440 at 150 %, primary, at 0,0)
# and B (125 %) to its right; B alone is at 0,0. A fence with auto height that the user moved
# from A onto B went back to A every time A came back: the auto height pass of the monitor
# change saved the window's place of the one-monitor layout (still where the window stood)
# into the two-monitor layout. And a fence gliding back to B over A (another scale) came out a
# sixth smaller: the glide took Windows' resize for A's scale. Also: a fence without auto
# height on B, an auto-height fence that stays on A, a scale change of B and back, B made the
# primary monitor and back, and items that come while an auto-height fence glides back (they
# are fitted when the glide ends).
#
# Needs a Hyper-V guest with a second monitor, from the virtual display driver on the PC:
#   scripts/wintest/hyperv/run-hyperv.sh --vm PF-Win11-2 --prep scripts/wintest/hyperv/vdd-prep.ps1 \
#     --extra <folder> scripts/wintest/tests/monitor-sets-move.ps1
# <folder>\vdd holds the PC's C:\VirtualDisplayDriver\MttVDD.inf, MttVDD.dll and mttvdd.cat, a
# vdd_settings.xml for one monitor with 2560x1440 as its first resolution, and signer.cer (the
# catalog's signer: (Get-AuthenticodeSignature mttvdd.cat).SignerCertificate.Export('Cert')).

Add-Type @'
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
}
'@

function MsMonitors() { return @((PfJson monitor list)) }
# The monitor (from `monitor list`) whose rect holds the middle of `r` ({x, y, w, h}).
function MsMonitorOf($r, $mons) {
  if (-not $r) { return $null }
  $cx = $r.x + $r.w / 2; $cy = $r.y + $r.h / 2
  foreach ($m in $mons) { if ($cx -ge $m.rect.x -and $cx -lt $m.rect.x + $m.rect.w -and $cy -ge $m.rect.y -and $cy -lt $m.rect.y + $m.rect.h) { return $m } }
  return $null
}
function MsRect($t) {
  $j = $null
  try { $j = (Pf fence get $t) | ConvertFrom-Json } catch { return $null }
  if ($j.fence) { $j = $j.fence }
  return $j.windowRect
}
function MsFmt($r) { if (-not $r) { return 'none' }; return "($($r.x),$($r.y) $($r.w)x$($r.h))" }
# Waits until PecoFence has logged the display change to `n` monitors after line `since`.
function MsWaitChange([int]$n, [int]$since, [int]$sec = 25) {
  for ($i = 0; $i -lt $sec * 4; $i++) {
    $lines = @(Get-Content (PfLogPath) -Encoding UTF8 -ErrorAction SilentlyContinue)
    $hit = @($lines | Select-Object -Skip $since | Where-Object { $_ -match "display changed monitors=$n " })
    if ($hit.Count) { Start-Sleep -Seconds 3; return $hit[-1] }
    Start-Sleep -Milliseconds 250
  }
  return $null
}
function MsLogCount() { return @(Get-Content (PfLogPath) -Encoding UTF8 -ErrorAction SilentlyContinue).Count }
function MsState($tag) {
  $mons = MsMonitors
  $s = foreach ($t in 'Moved', 'Plain', 'OnA') { $r = MsRect $t; $m = MsMonitorOf $r $mons; "$t=$(MsFmt $r) on $(if ($m) { $m.label + '@' + $m.dpi } else { 'none' })" }
  Log "state $tag | $($s -join ' | ') | monitors $(($mons | ForEach-Object { "$($_.label) $(MsFmt $_.rect) dpi $($_.dpi) primary $($_.primary)" }) -join '; ')"
}

Case 'monitor sets: two monitors in the guest'
$ms = ''
for ($i = 0; $i -lt 30; $i++) { $ms = [Mon2]::Dpi('', 0); if (@($ms.Trim() -split "`n").Count -ge 2) { break }; Start-Sleep -Seconds 1 }
$lines = @($ms.Trim() -split "`n" | ForEach-Object { $_.Trim() })
$vdd = @($lines | Where-Object { $_ -notmatch 'DISPLAY1\|' }) | Select-Object -First 1
$hv = @($lines | Where-Object { $_ -match 'DISPLAY1\|' }) | Select-Object -First 1
Check 'the guest has two monitors (the Hyper-V display and the virtual one)' ($lines.Count -ge 2 -and $vdd -and $hv) ($lines -join ' / ')
$GdiA = ($vdd -split '\|')[0]; $GdiB = ($hv -split '\|')[0]
Log "res A: $([Mon2]::Res($GdiA, 2560, 1440))"; Start-Sleep -Seconds 3
[Mon2]::Dpi($GdiA, 150) | Out-Null; [Mon2]::Dpi($GdiB, 125) | Out-Null; Start-Sleep -Seconds 3
Log "arrange: $([Mon2]::Arrange($GdiA, $GdiB))"; Start-Sleep -Seconds 4
Log "monitors: $(([Mon2]::Dpi('', 0)).Trim() -replace "`n", ' / ')"

PfStart
ClearFences | Out-Null
$mons = MsMonitors
$MA = @($mons | Where-Object { $_.primary })[0]; $MB = @($mons | Where-Object { -not $_.primary })[0]
Check 'A is the primary monitor at 0,0 at 150 %, B stands to its right at 125 %' ($MA -and $MB -and $MA.rect.x -eq 0 -and $MA.dpi -eq 144 -and $MB.dpi -eq 120 -and $MB.rect.x -ge $MA.rect.w) (($mons | ForEach-Object { "$($_.label) $(MsFmt $_.rect) dpi $($_.dpi) primary $($_.primary)" }) -join '; ')
# Forty files; Moved gets 31 of them and is narrow: taller than B's work area at 125 %, so
# its auto height stops at B's bottom edge there and at A's (lower) one on A.
$MsFiles = 1..40 | ForEach-Object { DeskFile ('Ms file {0:D2}.txt' -f $_) }
Start-Sleep -Seconds 3
$ax = $MA.workArea.x; $ay = $MA.workArea.y; $bx = $MB.workArea.x; $by = $MB.workArea.y
# --rect is x,y,w,h in physical px.
Pf fence create --title Moved --rect "$($ax + 90),$($ay + 60),450,640" | Out-Null
Pf fence create --title Plain --rect "$($bx + 800),$($by + 60),400,300" | Out-Null
Pf fence create --title OnA --rect "$($ax + 1500),$($ay + 60),600,440" | Out-Null
Start-Sleep -Seconds 1
Pf item move --glob 'Ms file 0*' --to OnA | Out-Null
foreach ($d in 1..4) { Pf item move --glob "Ms file $d*" --to Moved | Out-Null }
Pf fence set Moved autoHeight true | Out-Null
Pf fence set OnA autoHeight true | Out-Null
Start-Sleep -Seconds 2
# The user moves Moved from A onto B (the reporter's fence on the laptop's own screen).
Pf fence move Moved --x ($bx + 60) --y ($by + 40) | Out-Null
Start-Sleep -Seconds 2
MsState 'start'
$mons = MsMonitors
$Want = @{}
foreach ($t in 'Moved', 'Plain', 'OnA') { $Want[$t] = MsRect $t }
Check 'start: Moved and Plain are on B, OnA on A' ((MsMonitorOf $Want['Moved'] $mons).primary -eq $false -and (MsMonitorOf $Want['Plain'] $mons).primary -eq $false -and (MsMonitorOf $Want['OnA'] $mons).primary -eq $true) "Moved $(MsFmt $Want['Moved']), Plain $(MsFmt $Want['Plain']), OnA $(MsFmt $Want['OnA'])"
Shot 'ms-start'

function MsBack($tag) {
  trap { PfTrap $_; continue }
  $mons = MsMonitors
  foreach ($t in 'Moved', 'Plain', 'OnA') {
    $r = MsRect $t; $w = $Want[$t]
    $same = $r -and $w -and [math]::Abs($r.x - $w.x) -le 4 -and [math]::Abs($r.y - $w.y) -le 4 -and [math]::Abs($r.w - $w.w) -le 4 -and [math]::Abs($r.h - $w.h) -le 4
    $m = MsMonitorOf $r $mons; $mw = MsMonitorOf $w $mons
    Check "${tag}: $t is back at its place and size on its own monitor" ($same -and $m -and $mw -and $m.label -eq $mw.label) "now $(MsFmt $r) on $(if ($m) { $m.label }), was $(MsFmt $w) on $(if ($mw) { $mw.label })"
  }
}

function MsCycle([int]$k) {
  trap { PfTrap $_; continue }
  $n0 = MsLogCount
  Log "topology B only: $([Mon2]::Topology(8))"
  $l1 = MsWaitChange 1 $n0
  Check "cycle ${k}: PecoFence saw one monitor" ([bool]$l1) "$l1"
  MsState "cycle $k, B alone"
  if ($k -eq 1) { Shot 'ms-b-alone' }
  $n1 = MsLogCount
  Log "topology both: $([Mon2]::Topology(4))"
  $l2 = MsWaitChange 2 $n1
  Check "cycle ${k}: PecoFence saw two monitors again" ([bool]$l2) "$l2"
  MsState "cycle $k, both again"
  MsBack "cycle $k"
  if ($k -eq 1) { Shot 'ms-back-1' }
}

Case 'monitor sets: A goes away and comes back (three times)'
foreach ($k in 1..3) { MsCycle $k }
$saved = @(PfLogAll | Where-Object { $_ -match 'fence rect saved .*by=auto height' })
Log "auto height saves: $($saved.Count)"; $saved | Select-Object -Last 8 | ForEach-Object { Log "  $_" }

Case 'monitor sets: B changes its scale and back'
$n0 = MsLogCount
[Mon2]::Dpi($GdiB, 150) | Out-Null
$l = MsWaitChange 2 $n0
MsState 'B at 150 %'
$mons = MsMonitors
$r = MsRect 'Moved'; $m = MsMonitorOf $r $mons
Check 'B at 150 %: Moved stays on B' ($m -and -not $m.primary) "Moved $(MsFmt $r) on $(if ($m) { $m.label + '@' + $m.dpi }); $l"
$n0 = MsLogCount
[Mon2]::Dpi($GdiB, 125) | Out-Null
$l = MsWaitChange 2 $n0
MsState 'B at 125 % again'
MsBack 'B back at 125 %'

Case 'monitor sets: B becomes the primary monitor and back'
$n0 = MsLogCount
Log "arrange B first: $([Mon2]::Arrange($GdiB, $GdiA))"
$l = MsWaitChange 2 $n0
MsState 'B primary'
$mons = MsMonitors
foreach ($t in 'Moved', 'Plain') {
  $r = MsRect $t; $m = MsMonitorOf $r $mons
  Check "B primary: $t stays on B" ($m -and $m.primary) "$t $(MsFmt $r) on $(if ($m) { $m.label + ' primary ' + $m.primary })"
}
$r = MsRect 'OnA'; $m = MsMonitorOf $r $mons
Check 'B primary: OnA stays on A' ($m -and -not $m.primary) "OnA $(MsFmt $r) on $(if ($m) { $m.label })"
Shot 'ms-b-primary'
$n0 = MsLogCount
Log "arrange A first: $([Mon2]::Arrange($GdiA, $GdiB))"
$l = MsWaitChange 2 $n0
MsState 'A primary again'
MsBack 'A primary again'

Case 'monitor sets: items that come while a fence glides back'
# Grow (auto height, on B) only moves when A comes back, so it glides there (250 ms), and auto
# height waits while it moves. Items that come during the glide are fitted when it ends: the
# fence kept its old height and cut them off before (the glide's end sent nothing).
$GrFiles = @(1..4 | ForEach-Object { DeskFile "Gr a$_.txt" }) + @(1..8 | ForEach-Object { DeskFile "Gr b$_.txt" })
Start-Sleep -Seconds 2
$mons = MsMonitors; $MB = @($mons | Where-Object { -not $_.primary })[0]
Pf fence create --title Grow --rect "$($MB.workArea.x + 820),$($MB.workArea.y + 380),600,200" | Out-Null
Start-Sleep -Milliseconds 500
Pf item move --glob 'Gr a*' --to Grow | Out-Null
Pf fence set Grow autoHeight true | Out-Null
Start-Sleep -Seconds 1
$gj = (Pf fence get Grow) | ConvertFrom-Json; if ($gj.fence) { $gj = $gj.fence }
$GrowId = $gj.id
$Inbox = @(@(PfJson fence list) | Where-Object { $_.kind -eq 'inbox' })[0].id
Log "Grow $GrowId at $(MsFmt $gj.windowRect), fit $($gj.fit.fittingHeight)"
# Lines the app logged after byte $from of its log.
function MsNewLines([long]$from) {
  $fs = [IO.File]::Open((PfLogPath), 'Open', 'Read', 'ReadWrite')
  try { $null = $fs.Seek($from, 'Begin'); $sr = New-Object IO.StreamReader($fs); return @{ text = $sr.ReadToEnd(); end = $fs.Length } } finally { $fs.Dispose() }
}
function MsGrowTry([int]$k) {
  trap { PfTrap $_; continue }
  $n0 = MsLogCount
  [Mon2]::Topology(8) | Out-Null
  MsWaitChange 1 $n0 | Out-Null
  $from = (Get-Item (PfLogPath)).Length
  [Mon2]::Topology(4) | Out-Null
  $seen = $false; $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 25) {
    $r = MsNewLines $from
    if ($r.text -match "fence goes to its landing place fence=$GrowId .*glide=true") { $seen = $true; break }
    Start-Sleep -Milliseconds 10
  }
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $out = & "$PfApp\pecofence-cli.exe" item move --glob 'Gr b*' --to Grow 2>&1 | Out-String
  $ms = $sw.ElapsedMilliseconds
  Log "try ${k}: glide seen $seen, items moved $ms ms later: $($out.Trim())"
  Start-Sleep -Seconds 3
  $f = (Pf fence get Grow) | ConvertFrom-Json; if ($f.fence) { $f = $f.fence }
  return @{ inTime = ($seen -and $ms -lt 200); ms = $ms; f = $f }
}
$res = $null
foreach ($k in 1..3) {
  $res = MsGrowTry $k
  if ($res.inTime) { break }
  # Too late for the glide: the items go back and the next try begins.
  Pf item move --glob 'Gr b*' --to $Inbox | Out-Null
  Start-Sleep -Seconds 2
}
Check 'the items came while Grow glided back' ($res -and $res.inTime) "$($res.ms) ms after the glide began"
$f = $res.f
Check 'after the glide, Grow is as tall as its twelve items' ($f -and -not $f.fit.overflow -and [math]::Abs($f.windowRect.h - $f.fit.fittingHeight) -le 4) "window $(MsFmt $f.windowRect), fits $($f.fit.fittingHeight), overflow $($f.fit.overflow)"
Shot 'ms-grow-after-glide'

PfQuit
foreach ($p in @($MsFiles) + @($GrFiles)) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue }
