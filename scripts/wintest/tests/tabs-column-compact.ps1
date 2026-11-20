# Issues round 2026-11-20: 标题对齐 in a left tab column (feedback #192) and 收起时缩到标题的宽度
# (rollUp.compactRolled, GitHub #242). Every check states the new behaviour: on main the
# 192 checks fail (the rows stay at the top) and the 242 checks fail (no such setting).
# Parts (run.sh --var Part=...): A the tab column, B compact rolled fences. Default: both.
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }
Add-Type @"
using System; using System.Runtime.InteropServices;
public class Dpi { [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h); }
public class Widths {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  // The window's widths, each time it changes, for `ms` milliseconds.
  public static int[] Sample(IntPtr h, int ms) {
    var list = new System.Collections.Generic.List<int>(); var sw = System.Diagnostics.Stopwatch.StartNew();
    while (sw.ElapsedMilliseconds < ms) { RECT r; GetWindowRect(h, out r); int w = r.R - r.L; if (list.Count == 0 || list[list.Count - 1] != w) list.Add(w); System.Threading.Thread.Sleep(1); }
    return list.ToArray();
  }
}
"@
Case 'setup'
CloseWindows
foreach ($i in 1..9) { Set-Content "$PfDesk\tc-$i.txt" "tc$i" }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
PfStart -Script (DumpScript 250 20)
ClearFences
$wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
function Park() { ParkMouse ($wa.Left + 60) ($wa.Top + 300); Start-Sleep -Milliseconds 500 }
function Win($t) { return (Rect ([G]::FindWindowW('PecoFence.Fence', $t))) }
function K($t) { return ([Dpi]::GetDpiForWindow([G]::FindWindowW('PecoFence.Fence', $t)) / 96.0) }
$x0 = $wa.Left + [int]($wa.Width * 0.35)

if ($Part -match 'A') {
  Case '192 setup'
  Pf fence create --title Host --rect "$x0,$($wa.Top + 80),640,760" | Out-Null
  Pf fence create --title Second --rect "$($wa.Left + 40),$($wa.Top + 80),300,300" | Out-Null
  Pf fence create --title Third --rect "$($wa.Left + 40),$($wa.Top + 420),300,300" | Out-Null
  Pf item move "$PfDesk\tc-1.txt" "$PfDesk\tc-2.txt" "$PfDesk\tc-3.txt" "$PfDesk\tc-4.txt" "$PfDesk\tc-5.txt" --to Third | Out-Null
  Pf fence merge Second --into Host | Out-Null
  Pf fence merge Third --into Host | Out-Null
  Pf settings set tabBar left | Out-Null
  Start-Sleep -Seconds 1; Park
  foreach ($style in 'fluent', 'liquidGlass') {
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Milliseconds 700
    $pos = @{}
    foreach ($al in 'left', 'center', 'right') {
      Case "192 $style 标题对齐 $al"
      Pf settings set titleAlign $al | Out-Null
      Pf fence set Third iconAlign $al | Out-Null
      Start-Sleep -Milliseconds 900; Park
      $d = DumpOf 'Host'
      $k = K 'Host'
      $hDip = ($d.b - $d.t) / $k
      $first = $d.rects[0]; $last = $d.rects[-1]
      $mid = ($first[0] + $last[0] + $last[1]) / 2
      $pos[$al] = @($first[0], $mid, ($last[0] + $last[1]))
      Log "192 $style $al rects=$(($d.rects | ForEach-Object { '(' + $_[0] + ',' + $_[1] + ')' }) -join ' ') window_h_dip=$hDip"
      Shot "192-$style-left-$al"
    }
    $h = ((DumpOf 'Host').b - (DumpOf 'Host').t) / (K 'Host')
    Check "192 ${style}: 左对齐 rows start at the top of the column" ($pos['left'][0] -le 16) "first row at $($pos['left'][0]) DIP"
    Check "192 ${style}: 居中 rows are centred on the column" ([math]::Abs($pos['center'][1] - $h / 2) -le 2) "rows' middle $($pos['center'][1]), column middle $($h / 2)"
    Check "192 ${style}: 右对齐 rows end lower than 居中" ($pos['right'][2] -gt $pos['center'][2] + 20 -and $pos['right'][2] -le $h) "ends: left $($pos['left'][2]), centre $($pos['center'][2]), right $($pos['right'][2]), column $h"
    # Rolled, the fence collapses to its column, which is just as tall as its rows: they are at
    # its top; opened again they are centred again.
    Case "192 $style 居中, rolled and opened"
    Pf settings set titleAlign center | Out-Null; Start-Sleep -Milliseconds 500
    Pf fence roll Host | Out-Null; Park; Start-Sleep -Milliseconds 900
    $d = DumpOf 'Host'
    Check "192 ${style}: rolled, the rows sit at the top of the column" ($d.rolled -and $d.rects[0][0] -le 16) "rolled=$($d.rolled) first row at $($d.rects[0][0])"
    Shot "192-$style-left-center-rolled"
    Pf fence unroll Host | Out-Null; Park; Start-Sleep -Milliseconds 900
    $d = DumpOf 'Host'
    $mid = ($d.rects[0][0] + $d.rects[-1][0] + $d.rects[-1][1]) / 2
    Check "192 ${style}: opened again, centred again" ([math]::Abs($mid - (($d.b - $d.t) / (K 'Host')) / 2) -le 2) "middle $mid"
    # A fence dragged onto a column with 右对齐 opens a gap row there: the rows move up so the
    # last tab stays inside the column, above the chevron's room (44 DIP).
    Case "192 $style 右对齐, a fence dragged onto the column"
    Pf settings set titleAlign right | Out-Null; Start-Sleep -Milliseconds 700
    Pf fence create --title Extra --rect "$($wa.Left + 40),$($wa.Top + 80),260,200" | Out-Null; Start-Sleep -Milliseconds 900; Park
    $d0 = DumpOf 'Host'; $k = K 'Host'; $hh = Win 'Host'; $e = Win 'Extra'
    $ty = $hh.T + [int](($d0.rects[-1][0] + 16) * $k)
    $sx = $e.L + 60; $sy = $e.T + [int](14 * $k)
    [G]::MoveTo($sx, $sy); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 200
    Glide $sx $sy ($hh.L + [int](40 * $k)) $ty 20 30
    Start-Sleep -Milliseconds 700
    $d = DumpOf 'Host'
    Glide ($hh.L + [int](40 * $k)) $ty $sx $sy 20 30
    Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 900
    $h = ($d.b - $d.t) / $k
    $end = $d.rects[-1][0] + $d.rects[-1][1]
    $rest = ($d0.rects | ForEach-Object { '(' + $_[0] + ',' + $_[1] + ')' }) -join ' '
    $drag = ($d.rects | ForEach-Object { '(' + $_[0] + ',' + $_[1] + ')' }) -join ' '
    Log "192 $style merge drag rects=$drag resting=$rest h=$h"
    Check "192 ${style}: a merge drag opened a gap row in the column" ($drag -ne $rest) "resting $rest, during the drag $drag"
    Check "192 ${style}: with the gap open the last tab stays above the chevron" ($d.rects.Count -eq 3 -and $end -le $h - 44 + 0.5) "last tab ends at $end, the room ends at $($h - 44)"
    foreach ($f in @(PfJson fence list)) { if ($f -and $f.title -eq 'Extra') { Pf fence delete "$($f.id)" | Out-Null } }
    Start-Sleep -Milliseconds 500
    # The top tab bar still centres the strip in the title row.
    Case "192 $style top bar unchanged"
    Pf settings set titleAlign center | Out-Null; Pf settings set tabBar top | Out-Null; Start-Sleep -Milliseconds 900
    $d = DumpOf 'Host'
    $w = ($d.r - $d.l) / (K 'Host')
    $smid = ($d.rects[0][0] + $d.rects[-1][0] + $d.rects[-1][1]) / 2
    Check "192 ${style}: top bar 居中 still centred on the fence" ([math]::Abs($smid - $w / 2) -le 3) "strip middle $smid of $w"
    Shot "192-$style-top-center"
    Pf settings set tabBar left | Out-Null; Start-Sleep -Milliseconds 600
  }
  Pf settings set titleAlign left | Out-Null; Pf settings set tabBar top | Out-Null
  Pf settings set themeStyle fluent | Out-Null
  ClearFences
}

if ($Part -match 'B') {
  Case '242 setup'
  Pf fence create --title 'Notes' --rect "$x0,$($wa.Top + 80),600,450" | Out-Null
  Pf item move "$PfDesk\tc-6.txt" "$PfDesk\tc-7.txt" --to 'Notes' | Out-Null
  Pf fence create --title 'Work' --rect "$($x0 + 700),$($wa.Top + 80),600,450" | Out-Null
  Pf fence create --title 'Games' --rect "$($x0 + 700),$($wa.Top + 600),600,300" | Out-Null
  Pf item move "$PfDesk\tc-8.txt" --to 'Games' | Out-Null
  Pf fence merge 'Games' --into 'Work' | Out-Null
  Start-Sleep -Seconds 1; Park
  $wOpen = (Win 'Notes'); $openW = $wOpen.R - $wOpen.L
  Pf fence roll 'Notes' | Out-Null; Park; Start-Sleep -Milliseconds 800
  $r = Win 'Notes'
  Check '242 off: a rolled fence keeps its width' (($r.R - $r.L) -eq $openW) "width $($r.R - $r.L) of $openW"
  Pf fence unroll 'Notes' | Out-Null; Start-Sleep -Milliseconds 700

  foreach ($style in 'fluent', 'liquidGlass') {
    Pf settings set themeStyle $style | Out-Null; Start-Sleep -Milliseconds 700
    Pf settings set rollUp.compactRolled true | Out-Null; Start-Sleep -Milliseconds 500
    Case "242 $style roll and open"
    Pf fence roll 'Notes' | Out-Null; Park; Start-Sleep -Milliseconds 900
    $r = Win 'Notes'; $k = K 'Notes'
    $cw = ($r.R - $r.L) / $k
    $f = FenceGet 'Notes'
    Check "242 ${style}: rolled, as wide as its title row" ($cw -ge 120 -and $cw -le 260 -and $r.L -eq $wOpen.L) "width $cw DIP, left $($r.L) (open $($wOpen.L))"
    Check "242 ${style}: the saved rect keeps the open width" ($f.rect.w -eq $openW) "saved w $($f.rect.w), open $openW"
    Shot "242-$style-rolled"
    # A hover peek opens it at its own width, and it narrows again when the pointer leaves.
    [G]::MoveTo($r.L + 30, $r.T + 14); Start-Sleep -Milliseconds 1400
    $p = Win 'Notes'
    Check "242 ${style}: a hover peek opens it at its own width" (($p.R - $p.L) -eq $openW -and ($p.B - $p.T) -gt 300) "peek $($p.L),$($p.T),$($p.R),$($p.B)"
    Shot "242-$style-peek"
    Park; Start-Sleep -Milliseconds 1200
    $q = Win 'Notes'
    Check "242 ${style}: narrow again after the peek" (($q.R - $q.L) -eq ($r.R - $r.L)) "width $($q.R - $q.L), before $($r.R - $r.L)"
    # A longer name widens the rolled row.
    Pf fence rename 'Notes' 'Notes and drafts for the week' | Out-Null; Start-Sleep -Milliseconds 900
    $n = Win 'Notes and drafts for the week'
    Check "242 ${style}: a longer name widens the rolled row" (($n.R - $n.L) -gt ($q.R - $q.L) + 40 -and ($n.R - $n.L) -le $openW) "width $($n.R - $n.L), before $($q.R - $q.L)"
    Shot "242-$style-renamed"
    Pf fence rename 'Notes and drafts for the week' 'Notes' | Out-Null; Start-Sleep -Milliseconds 700
    # A tabbed fence: as wide as its tabs.
    Pf fence roll 'Work' | Out-Null; Park; Start-Sleep -Milliseconds 900
    $d = DumpOf 'Work'; $k = K 'Work'
    $tw = ($d.r - $d.l) / $k
    $end = $d.rects[-1][0] + $d.rects[-1][1]
    Check "242 ${style}: rolled tabs keep every tab whole" ($tw -lt 600 / $k -and $end -le $tw - 40) "width $tw DIP, last tab ends at $end"
    Shot "242-$style-tabs-rolled"
    # Opened, the fence is as wide as before.
    Pf fence unroll 'Notes' | Out-Null; Pf fence unroll 'Work' | Out-Null; Park; Start-Sleep -Milliseconds 900
    $o = Win 'Notes'; $ow = Win 'Work'
    Check "242 ${style}: opened at its own width again" (($o.R - $o.L) -eq $openW -and ($ow.R - $ow.L) -eq $openW) "Notes $($o.R - $o.L), Work $($ow.R - $ow.L), want $openW"
  }
  # Locked, the rolled row makes room for the lock glyph; its edges only move it.
  Case '242 lock and edges'
  Pf fence roll 'Notes' | Out-Null; Park; Start-Sleep -Milliseconds 900
  $r0 = Win 'Notes'; $k = K 'Notes'
  Pf fence set 'Notes' locked true | Out-Null; Start-Sleep -Milliseconds 900
  $r1 = Win 'Notes'
  Check '242: locked, the rolled row is wider by the lock glyph' ((($r1.R - $r1.L) - ($r0.R - $r0.L)) / $k -ge 12) "width $($r0.R - $r0.L) -> $($r1.R - $r1.L) px"
  Pf fence set 'Notes' locked false | Out-Null; Start-Sleep -Milliseconds 900
  $r2 = Win 'Notes'
  Check '242: unlocked, back to its width' (($r2.R - $r2.L) -eq ($r0.R - $r0.L)) "width $($r2.R - $r2.L), before $($r0.R - $r0.L)"
  $hw = [G]::FindWindowW('PecoFence.Fence', 'Notes')
  $hitR = NcHit $hw ($r2.R - 2) ($r2.T + [int](14 * $k)); $hitL = NcHit $hw ($r2.L + 1) ($r2.T + [int](14 * $k))
  Check '242: the left and right edges of a compact rolled row only move it (HTCAPTION)' ($hitR -eq 2 -and $hitL -eq 2) "right edge $hitR, left edge $hitL"
  # A clock rolled up is as wide as its time from the start: the width it lands at is the one a
  # later settle gives it (it measured its name before the roll began).
  Case '242 rolled clock'
  Pf fence create --tool clock --title Clock --rect "$($wa.Left + 40),$($wa.Top + 560),260,160" | Out-Null; Start-Sleep -Seconds 2
  $ch = [G]::FindWindowW('PecoFence.Fence', 'Clock')
  Pf fence roll Clock | Out-Null
  $ws = [Widths]::Sample($ch, 1500)
  $grew = @(for ($i = 1; $i -lt $ws.Count; $i++) { if ($ws[$i] -gt $ws[$i - 1]) { "$($ws[$i - 1])->$($ws[$i])" } })
  Check '242: a rolled clock narrows straight to the width of its time (no step back at the end)' ($grew.Count -eq 0) "widths $($ws -join ' ')"
  Park; Start-Sleep -Milliseconds 600
  $c1 = Win 'Clock'
  Pf settings set rollUp.compactRolled false | Out-Null; Start-Sleep -Milliseconds 700
  Pf settings set rollUp.compactRolled true | Out-Null; Start-Sleep -Milliseconds 900
  $c2 = Win 'Clock'
  Check '242: a rolled clock lands at the width of its time' ([math]::Abs(($c1.R - $c1.L) - ($c2.R - $c2.L)) -le 1 -and ($c2.R - $c2.L) -lt 260) "landed $($c1.R - $c1.L) px, settled $($c2.R - $c2.L) px"
  Shot '242-clock-rolled'
  foreach ($f in @(PfJson fence list)) { if ($f -and $f.title -eq 'Clock') { Pf fence delete "$($f.id)" | Out-Null } }
  Pf fence unroll 'Notes' | Out-Null; Start-Sleep -Milliseconds 700
  # A restart keeps a compact rolled fence compact and its saved rect; the switch off widens it.
  Case '242 restart and switch off'
  Pf fence roll 'Notes' | Out-Null; Park; Start-Sleep -Milliseconds 900
  $before = Win 'Notes'; $savedBefore = (FenceGet 'Notes').rect
  PfQuit; PfStart -Script (DumpScript 250 10); Start-Sleep -Seconds 2
  $after = Win 'Notes'; $savedAfter = (FenceGet 'Notes').rect
  Check '242: after a restart, still compact at the same place' ($after.L -eq $before.L -and ($after.R - $after.L) -eq ($before.R - $before.L)) "before $($before.L)..$($before.R), after $($after.L)..$($after.R)"
  Check '242: after a restart, the saved rect is the same' ($savedAfter.w -eq $savedBefore.w -and $savedAfter.x -eq $savedBefore.x) "before w $($savedBefore.w), after w $($savedAfter.w)"
  Pf settings set rollUp.compactRolled false | Out-Null; Start-Sleep -Milliseconds 900
  $off = Win 'Notes'
  Check '242: the switch off gives the rolled fence its width back' (($off.R - $off.L) -eq $openW -and (FenceGet 'Notes').rolledUp) "width $($off.R - $off.L), want $openW"
  Pf settings set themeStyle fluent | Out-Null
}
[Anim]::Set($AnimWas)
PfQuit
