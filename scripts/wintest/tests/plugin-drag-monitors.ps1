# Row 5 of plan v6 on two monitors at different scales: the Hyper-V display at 100 % and a virtual
# 1920x1080 monitor at 150 %, with the fences of plugin-drag.ps1's test plugins on the 150 % one.
# A drop on the web page finds the zone under the pointer (its point), a drop on a light plugin's
# row finds that row, and the drag pictures (a file from File Explorer over the page, a row's
# file, the page's picture) sit at the pointer at the monitor's size: shots, and the changed
# pixels around the pointer. Manual: a Hyper-V guest with the virtual display driver, debug build:
#   run-hyperv.sh --vm PF-Win11-2 --bin target/wintest-bin-debug --extra <folder with
#   prepare-guest.ps1, mon.cs and VirtualDisplayDriver\> scripts/wintest/tests/plugin-drag-monitors.ps1
Add-Type -Path C:\pf\extra\mon.cs
$Dr = 'C:\pf\run\drag'
$script:Config = "$PfApp\config"
# Input and pictures in physical pixels of the virtual screen (the thread per-monitor aware).
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Runtime.InteropServices; using System.Threading;
public static class Px2 {
  [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint data, UIntPtr extra);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  static void Aware() { SetThreadDpiAwarenessContext(new IntPtr(-4)); }
  public static int[] Rect(IntPtr h) { Aware(); RECT r; GetWindowRect(h, out r); return new[] { r.L, r.T, r.R, r.B }; }
  public static Bitmap Grab(int x, int y, int w, int h) {
    Aware(); var b = new Bitmap(w, h); using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, b.Size); return b;
  }
  // Press at (x1, y1), glide to (x2, y2), rest; `shot` (a file) is the square of `side` px
  // around the pointer then; with `cancel` Esc ends the drag there before the release.
  public static void Drag(int x1, int y1, int x2, int y2, string shot, int side, bool cancel) {
    Aware(); SetCursorPos(x1, y1); Thread.Sleep(200);
    mouse_event(2, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(200);
    for (int i = 1; i <= 24; i++) { SetCursorPos(x1 + (x2 - x1) * i / 24, y1 + (y2 - y1) * i / 24); mouse_event(1, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(35); }
    Thread.Sleep(600); SetCursorPos(x2 + 1, y2 + 1); Thread.Sleep(600);
    if (shot != "") using (var b = Grab(x2 - side / 2, y2 - side / 2, side, side)) b.Save(shot);
    if (cancel) { keybd_event(0x1B, 0, 0, UIntPtr.Zero); keybd_event(0x1B, 0, 2, UIntPtr.Zero); Thread.Sleep(300); }
    mouse_event(4, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(1500);
  }
  // The box of the pixels that differ between two pictures of the same size (-1s: none).
  public static int[] Changed(string a, string b) {
    using (var p = new Bitmap(a)) using (var q = new Bitmap(b)) {
      int l = int.MaxValue, t = int.MaxValue, r = -1, d = -1;
      for (int y = 0; y < p.Height; y += 2) for (int x = 0; x < p.Width; x += 2) {
        Color c = p.GetPixel(x, y), e = q.GetPixel(x, y);
        if (Math.Abs(c.R - e.R) + Math.Abs(c.G - e.G) + Math.Abs(c.B - e.B) > 60) { l = Math.Min(l, x); t = Math.Min(t, y); r = Math.Max(r, x); d = Math.Max(d, y); }
      }
      return r < 0 ? new[] { -1, -1, -1, -1 } : new[] { l, t, r, d };
    }
  }
}
'@
function FenceWin([string]$title) { return @([W]::Tops('PecoFence.Fence') | Where-Object { [L]::Text($_) -eq $title }) | Select-Object -First 1 }
function Out() { return (FenceOut $script:Fid) }
function WaitOut($regex, $sec = 8) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((Out) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
function PageOut() { return "$(Js "document.getElementById('out').textContent")" }
function WaitPage($regex, $sec = 8) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((PageOut) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
# An element's middle in physical px (the page fills the fence window to its bottom right).
function ElPt([string]$sel) {
  $q = $sel | ConvertTo-Json
  $e = Js "(() => { const b = document.querySelector($q).getBoundingClientRect(), d = devicePixelRatio; return { x: (b.left + b.width / 2) * d, y: (b.top + b.height / 2) * d, pw: innerWidth * d, ph: innerHeight * d }; })()"
  $r = [Px2]::Rect((FenceWin 'Web'))
  return @([int]($r[2] - $e.pw + $e.x), [int]($r[3] - $e.ph + $e.y))
}
function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
# Row `row` of the Drag fence in physical px: its window rect, content offset (px) and hit areas (DIPs).
function RowPt($row) {
  $w = DumpOf 'Drag' 5
  $r = @(CellRects $w | Where-Object { $_.i -eq $row * 100 }) | Select-Object -First 1
  if (-not $r) { Log "no hit area of row $row"; return $null }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  $win = [Px2]::Rect((FenceWin 'Drag'))
  return @([int]($win[0] + [double](DumpVal $w 'content_x') + ($r.l + 40) * $sc), [int]($win[1] + [double](DumpVal $w 'content_y') + ($r.t + $r.b) / 2 * $sc))
}
# A drag picture: the square around the pointer with and without the drag, and where they differ.
function DragShot($name, $x1, $y1, $x2, $y2, [switch]$Cancel) {
  $side = 360
  $before = "$PfShots\$name-rest.png"
  $b = [Px2]::Grab($x2 - $side / 2, $y2 - $side / 2, $side, $side); $b.Save($before); $b.Dispose()
  [Px2]::Drag($x1, $y1, $x2, $y2, "$PfShots\$name.png", $side, [bool]$Cancel)
  $c = [Px2]::Changed($before, "$PfShots\$name.png")
  Log "drag picture $name changed box $($c -join ',') in a $side px square around the pointer"
  return $c
}

Case 'two monitors: 100 % and 150 %'
CloseWindows
[Mon]::Topology(4) | Out-Null; Start-Sleep -Seconds 4
$dpi = [Mon]::Dpi('', 0); Log "displays: $dpi"
$vdd = ([regex]::Matches($dpi, '(\\\\\.\\DISPLAY\d+) target=\S*(MTT|Virtual|VDD)\S*') | Select-Object -First 1)
$gdi = if ($vdd) { $vdd.Groups[1].Value } else { '' }
Check 'a second monitor is there' ($gdi -ne '') $dpi
[Mon]::Res($gdi, 1920, 1080) | Out-Null; Start-Sleep -Seconds 3
Log ([Mon]::Dpi($gdi, 150)); Start-Sleep -Seconds 4
$mons2 = [Mon]::List()
Log "after: $([Mon]::Dpi('', 0)) $mons2"
# Bare desktop near the 150 % monitor's bottom right: where the pictures out are held.
$m2 = [regex]::Match($mons2, 'mon=\((-?\d+),(-?\d+),(-?\d+),(-?\d+)\) work=\S+ dpi=144')
$hold = @(([int]$m2.Groups[3].Value - 300), ([int]$m2.Groups[4].Value - 300))
Log "held over the 150 % monitor at $($hold -join ',')"
New-Item -ItemType Directory -Force $Dr | Out-Null
[IO.File]::WriteAllText("$Dr\dropped.txt", 'dropped on the page')
$data = "$script:Config\plugins-data\drag-demo"
New-Item -ItemType Directory -Force $data | Out-Null
$bm = New-Object Drawing.Bitmap 32, 32; $g = [Drawing.Graphics]::FromImage($bm); $g.Clear([Drawing.Color]::FromArgb(40, 140, 220)); $g.Dispose()
$bm.Save("$data\pic.png", [Drawing.Imaging.ImageFormat]::Png)
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfPlacePlugin 'C:\pf\kit\assets\drag-demo' 'drag-demo'
PfPlacePlugin 'C:\pf\kit\assets\webdrag' 'webdrag'
$bm.Save("$script:Config\plugins\webdrag\1.0.0\pic.png", [Drawing.Imaging.ImageFormat]::Png); $bm.Dispose()
PfStart -Lang en -Script (DumpScript 400 30 3000)
$mons = PfJson monitor list
$list = @(@($mons.monitors) + @($mons) | Where-Object { $_ -and $_.id })
$prim = $list | Where-Object { $_.primary } | Select-Object -First 1
$other = $list | Where-Object { -not $_.primary } | Select-Object -First 1
Check 'PecoFence sees two monitors with different scales' ($prim -and $other -and $prim.scale -ne $other.scale) "primary=$($prim.scale) other=$($other.scale) $($mons | ConvertTo-Json -Compress -Depth 5)"
$script:Fid = PfPluginFence 'Drag' '80,60,380,420' 'drag-demo'
$script:Web = PfPluginFence 'Web' '500,60,460,380' 'webdrag'
Check 'the page loads' ([bool](PfWaitLog 'web plugin: page loading' 40)) ''
Pf fence move $script:Fid --monitor $other.id | Out-Null
Pf fence move $script:Web --monitor $other.id | Out-Null
Start-Sleep -Seconds 3
$ok = CdpOpen -Url 'pecofence-plugin://webdrag/*' -sec 30
$dpr = Js 'devicePixelRatio'
Check 'both fences are on the 150 % monitor: the page has devicePixelRatio 1.5' ($dpr -eq 1.5) "dpr=$dpr drag=$([Px2]::Rect((FenceWin 'Drag')) -join ',') web=$([Px2]::Rect((FenceWin 'Web')) -join ',')"
Check 'the plugin shows its rows' (WaitOut 'dragend none' 15) (Out)
ParkMouse; Start-Sleep -Seconds 1

Case 'on the 150 % monitor: the drop point and the row under a drop'
$ex = [IntPtr]::Zero
Start-Process explorer.exe "`"$Dr`""
for ($i = 0; $i -lt 40 -and $ex -eq [IntPtr]::Zero; $i++) {
  foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) { if ((try { $win.Document.Folder.Self.Path } catch { '' }) -eq $Dr) { $ex = [IntPtr]$win.HWND } }
  Start-Sleep -Milliseconds 250
}
[L]::MoveWindow($ex, 40, 300, 700, 500, $true) | Out-Null; Start-Sleep -Seconds 2
$root = [Windows.Automation.AutomationElement]::FromHandle($ex)
$by = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, 'dropped.txt')
$el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, $by)
if (-not $el) { $el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, (New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, 'dropped'))) }
$er = $el.Current.BoundingRectangle
$file = @([int]($er.Left + 40), [int](($er.Top + $er.Bottom) / 2))
Log "file in Explorer at $($file -join ',') (UIA, physical in a per-monitor aware Explorer)"
foreach ($zone in 'z1', 'z2') {
  $z = ElPt "#$zone"
  $c = DragShot "M-file-over-$zone" $file[0] $file[1] $z[0] $z[1]
  Check "a file dropped on zone $zone of the page is found there (its point at 150 %)" (WaitPage "files=dropped.txt:19 .*zone=$zone") (PageOut)
  Check "the file's drag picture is at the pointer ($zone)" ($c[0] -ge 0 -and $c[0] -lt 250 -and $c[1] -lt 250) "changed $($c -join ',')"
}
foreach ($row in @(@(0, 'pic'), @(2, 'link'))) {
  $p = RowPt $row[0]
  [Px2]::Drag($file[0], $file[1], $p[0], $p[1], '', 0, $false)
  Check "a file dropped on row $($row[1]) on the 150 % monitor names that row" (WaitOut "drop row=$($row[1]) text=none files=1") (Out)
}

Case 'on the 150 % monitor: the drag pictures out of a row and out of the page'
# Held over the 150 % monitor's bare desktop, then Esc: nothing lands anywhere.
$p = RowPt 0
$c = DragShot 'M-row-file-out' $p[0] $p[1] $hold[0] $hold[1] -Cancel
Check 'a row''s file: its drag picture is at the pointer on the 150 % monitor' ($c[0] -ge 0 -and $c[0] -lt 250 -and $c[1] -lt 250) "changed $($c -join ',')"
Check 'Esc there: dragend says not dropped' (WaitOut 'dragend row=pic dropped=false') (Out)
$p = ElPt '#pic'
$c = DragShot 'M-page-pic-out' $p[0] $p[1] $hold[0] $hold[1] -Cancel
Check 'the page''s picture: its drag picture is at the pointer on the 150 % monitor' ($c[0] -ge 0 -and $c[0] -lt 250 -and $c[1] -lt 250) "changed $($c -join ',')"
$p = ElPt '#pic'
[Px2]::Drag($p[0], $p[1], $file[0] + 120, $file[1] + 120, '', 0, $false)
Check 'dropped into File Explorer on the 100 % monitor, the picture lands as a png' ((Get-ChildItem $Dr -Name) -match '\.png$') "$((Get-ChildItem $Dr -Name) -join ',')"
CloseWindows
