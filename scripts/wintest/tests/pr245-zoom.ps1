# PR #245 (GitHub #243): a double-click on a fence's empty background zooms it to the middle of
# its monitor (72 % of the work area, icons scaled with it); a second double-click, or a click on
# the desktop outside it, puts it back. Parts:
#   Z  zoom and back: the box, the exact way back, the saved rect untouched, a click outside,
#      a click on the taskbar, a second fence taking the zoom over, the double-click being off.
#   F  folder fences: in a subfolder a double-click on empty space goes up (no zoom); at the root
#      it zooms; with single-click open a double-click on a subfolder goes in and does not zoom.
#   G  what changes the fence while it is zoomed must not keep the zoomed size: auto height after
#      a new item, a roll and expand from the command line, a quit, a display change.
#   R  the review's cases: a double-click while a fence opens, 适应内容大小, auto height after the
#      zoom, the switch off while zoomed, a fence under another, a large icon size, our own menu.
#   T  a fence with tabs (a double-click on its tabs while zoomed keeps its size), animations off.
#   P  cost: the frames of the tween (WM_SIZE frame times), the icon extractions of one zoom, CPU
#      at rest while zoomed, memory after five zooms. Fluent and Liquid Glass.
# 1600x900 box at 100 % (the same checks hold at 150 %: run with WINTEST_SCALE=150).
if (-not (Get-Variable Part -ErrorAction SilentlyContinue) -or -not $Part) { $Part = 'ZTFGPR' }

Case 'setup'
CloseWindows
if ([Th]::SingleClick()) { [Th]::SetSingleClick($false) }
Set-Culture en-US
GreyWallpaper
# An RDP session starts with Windows' animations off; the zoom's tween needs them on.
[Anim]::Set($true)
$Lvl = 'pecofence=debug,pecofence::fence_window::handler::nc=trace'
PfStart -Script (DumpScript 300 25 3000) -LogLevel $Lvl
Start-Sleep -Seconds 2

function Wa() { return [Windows.Forms.Screen]::PrimaryScreen.WorkingArea }
function WinOf($id) { foreach ($w in @(PfDump 6)) { if ($w.active -eq $id) { return $w.hwnd } }; return [IntPtr]::Zero }
function R4($h) { $r = Rect $h; return @($r.L, $r.T, $r.R, $r.B) }
function Saved($id) { $s = (FenceGet $id).rect; return @([int]$s.x, [int]$s.y, [int]($s.x + $s.w), [int]($s.y + $s.h)) }
function Same($a, $b, $tol = 0) { for ($i = 0; $i -lt 4; $i++) { if ([math]::Abs($a[$i] - $b[$i]) -gt $tol) { return $false } }; return $true }
# Zoomed: 72 % of the work area high, centred on it, and at most 72 % wide (narrower when the
# icons reach their largest size first, so the columns stay).
function IsZoomed($r) {
  $wa = Wa
  $w = $r[2] - $r[0]; $h = $r[3] - $r[1]
  $cx = ($r[0] + $r[2]) / 2; $cy = ($r[1] + $r[3]) / 2
  return ($w -le $wa.Width * 0.72 + 6 -and $w -ge 300 -and [math]::Abs($h - $wa.Height * 0.72) -le 6 -and [math]::Abs($cx - ($wa.Left + $wa.Width / 2)) -le 4 -and [math]::Abs($cy - ($wa.Top + $wa.Height / 2)) -le 4)
}
# A point near the bottom right of the window, where a fence of a few items has no icon.
function Blank($h) { $r = Rect $h; return @(($r.R - 30), ($r.B - 24)) }
function DblBlank($h) { $p = Blank $h; DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 900 }
function NewPlain($title, $rect, $files) {
  $id = "$((PfJson fence create --title $title --rect $rect).fence.id)"
  foreach ($f in $files) { DeskFile $f | Out-Null }
  Start-Sleep -Milliseconds 800
  foreach ($f in $files) { Pf item move $f --to $id | Out-Null }
  Start-Sleep -Milliseconds 1200
  return $id
}

# The switch is off by default: a double-click on empty space changes nothing until it is on.
Case 'setup: off by default'
ClearFences | Out-Null
$d = NewPlain '默认' '300,180,360,240' @('zd1.txt')
$hd = WinOf $d; $rd = R4 $hd
DblBlank $hd
Check 'a double-click on empty space does not zoom with the default settings' (Same (R4 $hd) $rd) "now=$((R4 $hd) -join ',') was $($rd -join ',') zoom=$((PfJson settings get).zoom.enabled)"
Pf settings set zoom.enabled true | Out-Null; Start-Sleep -Milliseconds 600

function PartZ() {
  trap { PfTrap $_; continue }
  Case 'Z zoom and back'
  ClearFences | Out-Null
  $a = NewPlain '放大A' '200,150,360,240' @('za1.txt', 'za2.txt', 'za3.txt', 'za4.txt')
  $b = NewPlain '放大B' '1360,150,220,240' @('zb1.txt')
  $ha = WinOf $a; $hb = WinOf $b
  Check 'Z two fences made' ($ha -ne [IntPtr]::Zero -and $hb -ne [IntPtr]::Zero) "a=$ha b=$hb"
  $r0 = R4 $ha; $s0 = Saved $a
  DblBlank $ha
  $r1 = R4 $ha
  Check 'Z double-click on empty space zooms to the middle of the screen' (IsZoomed $r1) "before=$($r0 -join ',') after=$($r1 -join ',') work=$(Wa)"
  $settled = PfWaitLog 'zoom settled.*opening=true' 3
  Check 'Z the icons grow with it' ($settled -match 'icon_dip=([\d.]+)' -and [double]$Matches[1] -gt 60) "$settled"
  Check 'Z the saved rect stays the one it had' (Same (Saved $a) $s0) "saved=$((Saved $a) -join ',') was $($s0 -join ',')"
  Shot 'Z-zoomed'
  DblBlank $ha
  $r2 = R4 $ha
  Check 'Z a second double-click puts it back exactly' (Same $r2 $r0) "back=$($r2 -join ',') was $($r0 -join ',')"

  DblBlank $ha
  DeskClick 1000 820 | Out-Null; Start-Sleep -Milliseconds 900
  Check 'Z a click on the desktop outside puts it back' (Same (R4 $ha) $r0) "now=$((R4 $ha) -join ',')"

  # A touchpad tap: down and up at once, between two looks at the button state.
  DblBlank $ha
  $q = FreeSpot 1000 820
  [G]::MoveTo($q[0], $q[1]); Start-Sleep -Milliseconds 150; MDown; MUp; Start-Sleep -Milliseconds 900
  Check 'Z a quick tap on the desktop outside puts it back too' (Same (R4 $ha) $r0) "now=$((R4 $ha) -join ',') at $($q -join ',')"

  DblBlank $ha
  [G]::Click(700, ((Wa).Bottom + 20)); Start-Sleep -Milliseconds 900
  Check 'Z a click on the taskbar leaves it zoomed' (IsZoomed (R4 $ha)) "now=$((R4 $ha) -join ',')"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  DblBlank $hb
  Start-Sleep -Milliseconds 400
  Check 'Z a double-click on another fence puts the first back' (Same (R4 $ha) $r0) "a=$((R4 $ha) -join ',')"
  Check 'Z and zooms that one' (IsZoomed (R4 $hb)) "b=$((R4 $hb) -join ',')"
  DblBlank $hb
  Check 'Z the second one goes back too' (Same (R4 $hb) @(1360, 150, 1580, 390) 2) "b=$((R4 $hb) -join ',')"

  Pf settings set zoom.enabled false | Out-Null; Start-Sleep -Milliseconds 600
  DblBlank $ha
  Check 'Z switched off: a double-click on empty space changes nothing' (Same (R4 $ha) $r0) "now=$((R4 $ha) -join ',')"
  Pf settings set zoom.enabled true | Out-Null; Start-Sleep -Milliseconds 600
}

function PartF() {
  trap { PfTrap $_; continue }
  Case 'F folder fences'
  ClearFences | Out-Null
  $root = "$PfRun\放大文件夹"
  New-Item -ItemType Directory -Force "$root\a子文件夹" | Out-Null
  Set-Content -LiteralPath "$root\b.txt" 'b'
  Set-Content -LiteralPath "$root\a子文件夹\c.txt" 'c'
  $id = "$((PfJson fence create --portal $root --rect '300,180,440,320').fence.id)"
  Start-Sleep -Seconds 2
  $h = WinOf $id; $r0 = R4 $h
  $p = ItemAt $id 'a子文件夹'
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 1500
  $f = FenceGet $id
  Check 'F a double-click on the subfolder opens it in the fence' ("$($f.portal.current)" -like '*a子文件夹') "current=$($f.portal.current)"
  DblBlank $h
  $f = FenceGet $id
  Check 'F in the subfolder a double-click on empty space goes up' (-not $f.portal.current -or "$($f.portal.current)" -eq $root) "current=$($f.portal.current)"
  Check 'F and does not zoom' (Same (R4 $h) $r0) "now=$((R4 $h) -join ',')"
  DblBlank $h
  Check 'F at the root it zooms' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  DblBlank $h
  Check 'F and goes back' (Same (R4 $h) $r0) "now=$((R4 $h) -join ',')"

  Case 'F single-click open: a double-click on a subfolder goes in and does not zoom'
  $r2 = "$PfRun\单击放大"
  New-Item -ItemType Directory -Force "$r2\c子文件夹" | Out-Null
  Set-Content -LiteralPath "$r2\a1.txt" '1'
  Set-Content -LiteralPath "$r2\a2.txt" '2'
  Set-Content -LiteralPath "$r2\c子文件夹\唯一的文件.txt" '3'
  $sid = "$((PfJson fence create --portal $r2 --rect '820,180,440,240').fence.id)"
  Pf fence set $sid foldersFirst false | Out-Null
  Start-Sleep -Seconds 2
  $hs = WinOf $sid; $rs = R4 $hs
  [Th]::SetSingleClick($true); Start-Sleep -Seconds 2
  $p = ItemAt $sid 'c子文件夹'
  DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 1500
  $f = FenceGet $sid
  Check 'F single-click open: in the subfolder' ("$($f.portal.current)" -like '*c子文件夹') "current=$($f.portal.current)"
  Check 'F single-click open: the second click of that double-click does not zoom' (Same (R4 $hs) $rs) "now=$((R4 $hs) -join ',')"
  [Th]::SetSingleClick($false); Start-Sleep -Seconds 1
  if (IsZoomed (R4 $hs)) { DblBlank $hs }
}

function PartG() {
  trap { PfTrap $_; continue }
  Case 'G auto height while zoomed'
  ClearFences | Out-Null
  $a = NewPlain '放大高度' '200,150,360,240' @('zg1.txt', 'zg2.txt')
  Pf fence set $a autoHeight true | Out-Null; Start-Sleep -Milliseconds 1500
  $h = WinOf $a; $r0 = R4 $h; $s0 = Saved $a
  DblBlank $h
  Check 'G zoomed' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  DeskFile 'zg3.txt' | Out-Null; Start-Sleep -Milliseconds 800
  Pf item move 'zg3.txt' --to $a | Out-Null; Start-Sleep -Milliseconds 2000
  $sz = Saved $a
  Check 'G a new item while zoomed does not save the zoomed size' (($sz[2] - $sz[0]) -eq ($s0[2] - $s0[0]) -and ($sz[3] - $sz[1]) -lt 400) "saved=$($sz -join ',') was $($s0 -join ',')"
  if (IsZoomed (R4 $h)) { DblBlank $h }
  $r1 = R4 $h; $s1 = Saved $a
  Check 'G after the zoom the window is the saved rect' (Same $r1 $s1 2) "window=$($r1 -join ',') saved=$($s1 -join ',')"

  Case 'G roll and expand while zoomed'
  Pf fence set $a autoHeight false | Out-Null; Start-Sleep -Milliseconds 800
  # Room under the one row of icons, for the double-clicks below at any scale.
  Pf fence move $a --rect '200,150,360,330' | Out-Null; Start-Sleep -Milliseconds 800
  $r0 = R4 $h; $s0 = Saved $a
  DblBlank $h
  Pf fence roll $a | Out-Null; Start-Sleep -Milliseconds 1500
  Pf fence unroll $a | Out-Null; Start-Sleep -Milliseconds 1500
  if (IsZoomed (R4 $h)) { DblBlank $h }
  $r1 = R4 $h; $s1 = Saved $a
  Check 'G after roll and expand the fence has its own size' ((($r1[2] - $r1[0]) -eq ($r0[2] - $r0[0])) -and (($r1[3] - $r1[1]) -eq ($r0[3] - $r0[1]))) "window=$($r1 -join ',') was $($r0 -join ',')"
  Check 'G and the saved rect is the one it had' (Same $s1 $s0 2) "saved=$($s1 -join ',') was $($s0 -join ',')"

  Case 'G quit while zoomed'
  $r0 = R4 $h; $s0 = Saved $a
  DblBlank $h
  PfQuit
  PfStart -Script (DumpScript 300 25 3000) -LogLevel $Lvl
  Start-Sleep -Seconds 2
  $h = WinOf $a
  Check 'G after a quit while zoomed: the saved rect is the one it had' (Same (Saved $a) $s0 2) "saved=$((Saved $a) -join ',') was $($s0 -join ',')"
  Check 'G and the window is back there' (Same (R4 $h) $s0 2) "window=$((R4 $h) -join ',')"

  Case 'G display change while zoomed'
  # The box's RDP display has one mode, so the app is told of a display change as Windows does
  # (WM_DISPLAYCHANGE to its window): the pass that follows places every fence again.
  $r0 = R4 $h; $s0 = Saved $a
  DblBlank $h
  Check 'G zoomed before the display change' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value); $res = [IntPtr]::Zero
  [G]::SendMessageTimeoutW($ctl, 0x7E, [IntPtr]32, [IntPtr]((900 -shl 16) -bor 1600), 2, 3000, [ref]$res) | Out-Null
  Start-Sleep -Seconds 3
  $seen = PfWaitLog 'display changed' 3
  $r1 = R4 $h; $s1 = Saved $a
  Check 'G after a display change while zoomed the fence is not zoomed' (-not (IsZoomed $r1)) "window=$($r1 -join ',') pass seen=$([bool]$seen)"
  Check 'G and its window is its saved rect' (Same $r1 $s1 2) "window=$($r1 -join ',') saved=$($s1 -join ',') was $($s0 -join ',')"
  Shot 'G-after-display-change'
  # Over and done: the next double-click zooms again (a zoom left half open would close instead).
  DblBlank $h
  Check 'G after the display change a double-click zooms afresh' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  if (IsZoomed (R4 $h)) { DblBlank $h }
}

function FrameStats($since) {
  $us = @(PfLogAll | Where-Object { $_ -match 'WM_SIZE frame' -and (LogTime $_) -gt $since } | ForEach-Object { if ($_ -match ' us=(\d+)') { [int]$Matches[1] } })
  if ($us.Count -eq 0) { return 'no frames' }
  $s = @($us | Sort-Object)
  return "frames=$($s.Count) median_ms=$([math]::Round($s[[int]($s.Count / 2)] / 1000, 1)) max_ms=$([math]::Round($s[-1] / 1000, 1))"
}
function Extracted($since) {
  $n = 0
  foreach ($l in @(PfLogAll | Where-Object { $_ -match 'icons extracted' -and (LogTime $_) -gt $since })) { if ($l -match ' ok=(\d+)') { $n += [int]$Matches[1] } }
  return $n
}

function PartP() {
  trap { PfTrap $_; continue }
  foreach ($style in 'fluent', 'liquidGlass') {
    Case "P cost ($style)"
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Seconds 2
    ClearFences | Out-Null
    # Twelve kinds of file: twelve pictures to fetch at each icon size.
    $files = 'txt', 'bmp', 'zip', 'ps1', 'bat', 'ini', 'log', 'xml', 'htm', 'csv', 'json', 'rtf' | ForEach-Object { "zp.$_" }
    $a = NewPlain "放大$style" '200,150,420,330' $files
    $h = WinOf $a
    Start-Sleep -Seconds 3
    $t0 = [DateTime]::UtcNow
    DblBlank $h
    Start-Sleep -Seconds 3
    Log "P $style zoom in: $(FrameStats $t0) extracted=$(Extracted $t0)"
    Check "P $style zoom in: per frame and icons" $true "$(FrameStats $t0) icon extractions=$(Extracted $t0)"
    Shot "P-$style-zoomed"
    $p = Get-Process pecofence
    $cpu0 = $p.TotalProcessorTime.TotalMilliseconds
    Start-Sleep -Seconds 10
    $p.Refresh()
    $cpu = $p.TotalProcessorTime.TotalMilliseconds - $cpu0
    Check "P $style zoomed at rest: CPU over 10 s" ($cpu -lt 300) "cpu_ms=$([int]$cpu)"
    $t1 = [DateTime]::UtcNow
    DblBlank $h
    Start-Sleep -Seconds 2
    Log "P $style zoom out: $(FrameStats $t1)"
    $p.Refresh(); $m0 = $p.PrivateMemorySize64
    for ($i = 0; $i -lt 5; $i++) { DblBlank $h; DblBlank $h }
    Start-Sleep -Seconds 8
    $p.Refresh(); $m1 = $p.PrivateMemorySize64
    Check "P $style five zooms: private memory" (($m1 - $m0) -lt 20MB) "before=$([int]($m0 / 1MB)) MB after=$([int]($m1 / 1MB)) MB"
  }
  Pf settings set themeStyle fluent | Out-Null
}

function PartT() {
  trap { PfTrap $_; continue }
  Case 'T tabs and animations off'
  ClearFences | Out-Null
  $a = NewPlain '标签甲' '300,180,420,300' @('zt1.txt', 'zt2.txt')
  $b = NewPlain '标签乙' '900,180,300,240' @('zt3.txt')
  Pf fence merge $b --into $a | Out-Null; Start-Sleep -Milliseconds 1500
  $h = (DumpOf '标签甲').hwnd; $r0 = R4 $h; $s0 = Saved $a
  DblBlank $h
  Check 'T a fence with tabs zooms' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  Shot 'T-tabs-zoomed'
  # The title row (the tabs) of the zoomed fence: a double-click there rolls or does nothing,
  # and whatever it does, the fence keeps its own size.
  $z = R4 $h
  DoubleClick ($z[0] + 60) ($z[1] + 16); Start-Sleep -Milliseconds 1200
  $f = FenceGet $a
  if ($f.rolledUp) { Pf fence unroll $a | Out-Null; Start-Sleep -Milliseconds 1200 }
  if (IsZoomed (R4 $h)) { DblBlank $h }
  Check 'T after a double-click on the tabs of the zoomed fence it has its own size' (Same (R4 $h) $r0 2) "window=$((R4 $h) -join ',') was $($r0 -join ',') rolled=$($f.rolledUp)"
  Check 'T and its saved rect' (Same (Saved $a) $s0 2) "saved=$((Saved $a) -join ',') was $($s0 -join ',')"
  [Anim]::Set($false); Start-Sleep -Milliseconds 500
  $p = Blank $h; DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 150
  Check 'T animations off: zoomed at once' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  $p = Blank $h; DoubleClick $p[0] $p[1]; Start-Sleep -Milliseconds 150
  Check 'T animations off: back at once' (Same (R4 $h) $r0) "now=$((R4 $h) -join ',')"
  [Anim]::Set($true)
}

# R: the review's cases (2026-11-20, five reviewers): a double-click during the expand of a
# rolled fence, 适应内容大小 from the command line, auto height done after the zoom, the switch
# turned off while zoomed, a fence under another one, a large icon size, a press on our own menu.
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class ZoomWin {
  [StructLayout(LayoutKind.Sequential)] public struct P { public int X, Y; }
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
  public static IntPtr RootAt(int x, int y) { var p = new P(); p.X = x; p.Y = y; return GetAncestor(WindowFromPoint(p), 2); }
}
'@
function Fit($id) { return [int](FenceGet $id).fit.fittingHeight }
function Cols($id) { return [int](FenceGet $id).fit.columns }

function PartR() {
  trap { PfTrap $_; continue }
  Case 'R1 a double-click while a rolled fence opens'
  CloseWindows
  ClearFences | Out-Null
  $a = NewPlain '展开中' '200,150,360,300' @('zr1.txt')
  $h = WinOf $a; $r0 = R4 $h; $s0 = Saved $a
  Pf fence roll $a | Out-Null; Start-Sleep -Milliseconds 1200
  # The expand takes 333 ms: the double-click lands while it runs, on the open fence's lower right.
  Pf fence unroll $a | Out-Null
  DoubleClick ($r0[2] - 30) ($r0[3] - 24); Start-Sleep -Milliseconds 1500
  if (IsZoomed (R4 $h)) { DblBlank $h }
  Start-Sleep -Milliseconds 600
  Check 'R1 the saved rect is the one it had' (Same (Saved $a) $s0 2) "saved=$((Saved $a) -join ',') was $($s0 -join ',')"
  Check 'R1 and the window is back there' (Same (R4 $h) $s0 2) "window=$((R4 $h) -join ',')"

  Case 'R2 适应内容大小 from the command line while zoomed'
  $h = WinOf $a; $s0 = Saved $a
  DblBlank $h
  Pf fence fit $a | Out-Null; Start-Sleep -Milliseconds 1500
  if (IsZoomed (R4 $h)) { DblBlank $h }
  $s1 = Saved $a; $f = Fit $a
  Check 'R2 the fence fits its own content, not the zoomed one' ((Same (R4 $h) $s1 2) -and (($s1[3] - $s1[1]) -eq $f) -and (($s1[2] - $s1[0]) -eq ($s0[2] - $s0[0]))) "saved=$($s1 -join ',') fitting=$f was $($s0 -join ',')"

  Case 'R3 auto height is done once the zoom is over'
  CloseWindows
  ClearFences | Out-Null
  # Three icons in four columns: the fourth place of the row is empty space to double-click.
  $a = NewPlain '自动高度' '200,150,360,240' @('zh1.txt', 'zh2.txt', 'zh3.txt')
  Pf fence set $a autoHeight true | Out-Null; Start-Sleep -Milliseconds 1500
  $h = WinOf $a; $one = (R4 $h)
  DblBlank $h
  Check 'R3 zoomed' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  foreach ($n in 'zh4.txt', 'zh5.txt') { DeskFile $n | Out-Null }
  Start-Sleep -Milliseconds 800
  foreach ($n in 'zh4.txt', 'zh5.txt') { Pf item move $n --to $a | Out-Null }
  Start-Sleep -Milliseconds 1500
  DblBlank $h; Start-Sleep -Milliseconds 1200
  $r1 = R4 $h; $f = Fit $a
  Check 'R3 after the zoom the fence has the height of its five icons' ((($r1[3] - $r1[1]) -eq $f) -and $f -gt ($one[3] - $one[1])) "height=$($r1[3] - $r1[1]) fitting=$f before=$($one[3] - $one[1])"

  Case 'R4 the switch turned off while zoomed'
  Pf fence set $a autoHeight false | Out-Null
  Pf fence move $a --rect '200,150,360,330' | Out-Null; Start-Sleep -Milliseconds 800
  CloseWindows
  $r0 = R4 $h
  DblBlank $h
  Check 'R4 zoomed' (IsZoomed (R4 $h)) "now=$((R4 $h) -join ',')"
  Pf settings set zoom.enabled false | Out-Null; Start-Sleep -Milliseconds 1200
  Check 'R4 switching it off puts the zoomed fence back' (Same (R4 $h) $r0) "now=$((R4 $h) -join ',') was $($r0 -join ',')"
  if (IsZoomed (R4 $h)) { DblBlank $h }
  Pf settings set zoom.enabled true | Out-Null; Start-Sleep -Milliseconds 600

  Case 'R5 a fence inside the zoom box is under the zoomed fence'
  CloseWindows
  ClearFences | Out-Null
  # PecoFence keeps fences apart: the other fence lies inside the box the zoom grows to, and it
  # was made last, so it is above the first one until the zoom.
  $a = NewPlain '放大的' '200,150,360,240' @('zu1.txt')
  $b = NewPlain '旁边的' '700,300,300,200' @('zu2.txt')
  $ha = WinOf $a; $hb = WinOf $b
  $before = [ZoomWin]::RootAt(850, 450)
  DblBlank $ha
  $at = [ZoomWin]::RootAt(850, 450)
  Check 'R5 zoomed, the fence is over the other one' ((IsZoomed (R4 $ha)) -and $at -eq $ha -and $before -eq $hb) "zoomed=$(IsZoomed (R4 $ha)) at 850,450: $at (zoomed $ha other $hb, before $before)"
  if (IsZoomed (R4 $ha)) { DblBlank $ha }

  Case 'R6 a large icon size keeps its columns'
  CloseWindows
  ClearFences | Out-Null
  $a = NewPlain '大图标' '200,150,200,420' @('zc1.txt', 'zc2.txt', 'zc3.txt')
  Pf fence set $a iconSize 128 | Out-Null; Start-Sleep -Milliseconds 1500
  # Taller than its three rows: empty space under them.
  Pf fence move $a --rect '200,120,200,700' | Out-Null; Start-Sleep -Milliseconds 1000
  $h = WinOf $a; $c0 = Cols $a; $rb = R4 $h
  DblBlank $h
  Check 'R6 zoomed' (-not (Same (R4 $h) $rb)) "now=$((R4 $h) -join ',') was $($rb -join ',')"
  $c1 = Cols $a; $rz = R4 $h
  Check 'R6 zoomed, the fence keeps its column count' ($c1 -eq $c0 -and $c0 -ge 1) "columns $c0 -> $c1 zoomed=$($rz -join ',')"
  DblBlank $h

  Case 'R7 a press on our own menu outside the zoomed fence'
  CloseWindows
  ClearFences | Out-Null
  $a = NewPlain '菜单' '200,150,360,240' @('zm1.txt')
  $h = WinOf $a
  DblBlank $h
  $z = R4 $h
  [G]::RClick(($z[2] - 40), ($z[3] - 40)); Start-Sleep -Milliseconds 900
  $m = MenuWin 3
  # A row that only opens a submenu, pressed where the menu lies outside the fence.
  $row = @(MenuRows $m) | Where-Object { $_.text -match '^(View|Sort by|视图|排序方式)$' } | Select-Object -First 1
  Log "R7 menu: $(MenuTexts $m); picked '$($row.text)' at $($row.L),$($row.T)"
  $px = [math]::Max([int]($z[2] + 12), [int](($row.L + $row.R) / 2))
  if ($row -and $px -lt $row.R) { [G]::Click($px, [int](($row.T + $row.B) / 2)); Start-Sleep -Milliseconds 900 } else { $row = $null }
  $still = IsZoomed (R4 $h)
  CloseMenus; Start-Sleep -Milliseconds 400
  Check 'R7 a press on its menu does not put the zoomed fence back' ($row -and $still) "row='$($row.text)' zoomed=$still"
  if (IsZoomed (R4 $h)) { DblBlank $h }
}

# A: a recording for the report (no checks): zoom a fence with tabs, double-click its tabs, open
# it again. --var Side=before|after names the recording.
function PartA() {
  trap { PfTrap $_; continue }
  Case 'A recording'
  . C:\pf\kit\rec.ps1
  ClearFences | Out-Null
  $a = NewPlain '标签甲' '300,180,420,300' @('zt1.txt', 'zt2.txt')
  $b = NewPlain '标签乙' '900,180,300,240' @('zt3.txt')
  Pf fence merge $b --into $a | Out-Null; Start-Sleep -Milliseconds 1500
  $h = (DumpOf '标签甲').hwnd
  ParkMouse 1500 820
  RecStart "A-$Side" -X 160 -Y 60 -W 1280 -H 720 -Fps 12
  RecMark '双击空白处：放大'
  DblBlank $h; Start-Sleep -Milliseconds 600
  RecMark '双击放大后的标签行：收起'
  $z = R4 $h
  DoubleClick ($z[0] + 60) ($z[1] + 16); Start-Sleep -Milliseconds 1400
  RecMark '再展开'
  Pf fence unroll $a | Out-Null; Start-Sleep -Milliseconds 1600
  RecStop
}

if ($Part -match 'A') { PartA }
if ($Part -match 'R') { PartR }
if ($Part -match 'Z') { PartZ }
if ($Part -match 'T') { PartT }
if ($Part -match 'F') { PartF }
if ($Part -match 'G') { PartG }
if ($Part -match 'P') { PartP }
CloseWindows
[Anim]::Set($false)
