# Owner round 2026-11-12, package hunt-scale: what the fences do when the screen changes.
#   run.sh --var Part=G tests/owner2-hunt-scale.ps1     the taskbar hides itself: fences glide
#   run.sh --var Part=N tests/owner2-hunt-scale.ps1     a fence's name box sits on its title
#   tests/free-placement.sh --part D --steps tests/owner2-hunt-scale.ps1
#        the first-run column over 1280x720, 150 %, 125 %, 1920x1080: gaps kept, edge gap kept
# Part D asks free-placement.sh for another RDP size or scale (C:\pf\resize-want.txt).
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'G' }
trap { PfTrap $_; continue }
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class HsScr {
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential)] struct ABD { public int cb; public IntPtr hWnd; public uint msg; public uint edge; public int l, t, r, b; public IntPtr lParam; }
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, ref RC r, uint f);
  [DllImport("shell32.dll")] static extern IntPtr SHAppBarMessage(uint m, ref ABD d);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindowW(string c, string t);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  public static string Size() { return GetSystemMetrics(0) + "x" + GetSystemMetrics(1); }
  public static int[] Work() { var r = new RC(); SystemParametersInfoW(0x30, 0, ref r, 0); return new int[] { r.L, r.T, r.R, r.B }; }
  // The taskbar hides itself (ABM_SETSTATE with ABS_AUTOHIDE) or stays (ABS_ALWAYSONTOP).
  public static void TaskbarAutoHide(bool on) { var d = new ABD(); d.cb = Marshal.SizeOf(typeof(ABD)); d.hWnd = FindWindowW("Shell_TrayWnd", null); d.lParam = (IntPtr)(on ? 1 : 2); SHAppBarMessage(10, ref d); }
  public static IntPtr Fence(string title) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => {
      var c = new StringBuilder(128); GetClassNameW(h, c, 128);
      if (c.ToString() != "PecoFence.Fence" || !IsWindowVisible(h)) return true;
      var t = new StringBuilder(512); GetWindowTextW(h, t, 512);
      if (t.ToString() == title) { found = h; return false; }
      return true; }, IntPtr.Zero);
    return found;
  }
}
'@
# Asks free-placement.sh for a size (and scale) and waits until the session has it.
function WantSize($w, $h, $scale = '') {
  $want = if ($scale) { "${w}x${h}@$scale" } else { "${w}x${h}" }
  Set-Content C:\pf\resize-want.txt $want -Encoding ASCII
  for ($k = 0; $k -lt 150; $k++) {
    Start-Sleep -Seconds 1
    if (-not (Test-Path C:\pf\resize-want.txt) -and [HsScr]::Size() -eq "${w}x${h}") { break }
  }
  Start-Sleep -Seconds 7
  Log "screen now $([HsScr]::Size()), work area $([HsScr]::Work() -join ',') asked $want"
}
function HsFence($title) { return @(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq $title })[0] }
function HsWin($title) { $f = HsFence $title; return $f.windowRect }
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue

if ($Part -eq 'G') {
  # The work area changes (the taskbar hides itself and comes back): a fence that rests on the
  # taskbar goes to its new place over several frames. It jumped the 48 px in one step.
  Case 'the taskbar hides itself: the fences glide to their new places'
  PfStart
  Start-Sleep -Seconds 6
  ClearFences | Out-Null
  Pf fence create --title '底部栅栏' --rect '1292,580,300,264' | Out-Null
  Pf fence create --title '顶部栅栏' --rect '8,8,300,240' | Out-Null
  Start-Sleep -Seconds 1
  [Anim]::Set($true)
  ParkMouse 800 300
  $h = [HsScr]::Fence('底部栅栏')
  $top0 = (Rect $h).T
  function Watch($sec) {
    $seen = @(); $t0 = Get-Date
    while (((Get-Date) - $t0).TotalSeconds -lt $sec) { $t = (Rect $h).T; if ($seen.Count -eq 0 -or $seen[-1] -ne $t) { $seen += $t }; Start-Sleep -Milliseconds 5 }
    return $seen
  }
  [HsScr]::TaskbarAutoHide($true)
  $down = @(Watch 3)
  Log "taskbar hidden: tops seen $($down -join ' ')"
  $top1 = (Rect $h).T
  Check 'the fence on the taskbar follows the work area down' ($top1 - $top0 -eq 48) "top $top0 -> $top1, work area $([HsScr]::Work() -join ',')"
  Check 'it glides there: places in between are seen' ($down.Count -ge 4) "tops seen: $($down -join ' ')"
  $steps = @(for ($i = 1; $i -lt $down.Count; $i++) { $down[$i] - $down[$i - 1] })
  Check 'no step of the glide goes back' (@($steps | Where-Object { $_ -lt 0 }).Count -eq 0) "steps: $($steps -join ' ')"
  Check 'the fence at the top stays' ((Rect ([HsScr]::Fence('顶部栅栏'))).T -eq 8) "top $((Rect ([HsScr]::Fence('顶部栅栏'))).T)"
  [HsScr]::TaskbarAutoHide($false)
  $up = @(Watch 3)
  Log "taskbar back: tops seen $($up -join ' ')"
  Check 'with the taskbar back the fence is where it was' ((Rect $h).T -eq $top0) "top $((Rect $h).T), was $top0"
  Check 'it glides back as well' ($up.Count -ge 4) "tops seen: $($up -join ' ')"
  $g = HsFence '底部栅栏'
  Check 'the saved place is the one on screen' ($g.windowRect.y -eq $top0 -and $g.rect.y -eq $top0) "window $($g.windowRect.y) saved $($g.rect.y)"
  # With Windows' animations off it goes there at once.
  [Anim]::Set($false)
  Start-Sleep -Milliseconds 500
  [HsScr]::TaskbarAutoHide($true)
  $snap = @(Watch 3)
  Check 'with animations off the fence is at its new place in one step' ($snap.Count -le 2 -and (Rect $h).T -eq $top1) "tops seen: $($snap -join ' ')"
  [HsScr]::TaskbarAutoHide($false)
  Start-Sleep -Seconds 2
  PfQuit
}

if ($Part -eq 'N') {
  # A fence's name box: its text starts where the title's does and its box is in the middle of
  # the title row. It stood 8 px from the fence's left edge and 4 px under its top.
  Case 'the name box of a fence sits on its title'
  PfStart -Script (DumpScript)
  Start-Sleep -Seconds 6
  ClearFences | Out-Null
  Pf fence create --title '季度报告' --rect '200,200,320,240' | Out-Null
  Start-Sleep -Seconds 1
  $f = Rect ([HsScr]::Fence('季度报告'))
  Shot 'N-0-title'
  [G]::RClick($f.L + 200, $f.T + 16); Start-Sleep -Milliseconds 900
  MenuPick '^Rename' (MenuWin 5) | Out-Null
  $box = WaitWin 'PecoFence.Rename' 6
  Start-Sleep -Milliseconds 500
  Shot 'N-1-namebox'
  $b = Rect $box
  Log "fence $($f.L),$($f.T),$($f.R),$($f.B) name box $($b.L),$($b.T),$($b.R),$($b.B)"
  # The title row is 36 px at 100 %; the title's text starts 16 px from the fence's left edge
  # and the box's text 4 px inside the box.
  Check 'the box is in the middle of the title row' (($b.T - $f.T) -eq ($f.T + 36 - $b.B)) "above $($b.T - $f.T) px, below $($f.T + 36 - $b.B) px"
  Check 'its text starts where the title text starts' (($b.L + 4) -eq ($f.L + 16)) "box left $($b.L), fence left $($f.L)"
  [G]::Type('新名字'); Start-Sleep -Milliseconds 300
  [G]::Press(0x0D); Start-Sleep -Milliseconds 800
  Check 'Enter still renames the fence' ((HsFence '新名字') -ne $null) "titles: $((@(PfJson fence list) | ForEach-Object { $_.title }) -join ', ')"

  # A tab in the title row: its box opens on its own caption. It opened at the left end of
  # the row, over the first tab.
  Case 'the name box of a tab opens on that tab'
  Pf fence create --title '甲甲甲甲甲甲' --rect '600,200,360,240' | Out-Null
  Pf fence create --title '乙' --rect '1000,200,300,240' | Out-Null
  Pf fence merge '乙' --into '甲甲甲甲甲甲' | Out-Null
  Start-Sleep -Seconds 1
  $d = DumpOf '甲甲甲甲甲甲'
  $t2 = $d.rects[1]
  Log "tab rects: $(($d.rects | ForEach-Object { "$($_[0]),$($_[1])" }) -join ' ') window $($d.l),$($d.t)"
  [G]::RClick([int]($d.l + $t2[0] + $t2[1] / 2), $d.t + 16); Start-Sleep -Milliseconds 900
  MenuPick '^Rename' (MenuWin 5) | Out-Null
  $box = WaitWin 'PecoFence.Rename' 6
  Start-Sleep -Milliseconds 500
  Shot 'N-2-tab-namebox'
  $b = Rect $box
  Check 'the box starts on the second tab, not over the first' ($b.L -ge ($d.l + $t2[0]) -and $b.L -lt ($d.l + $t2[0] + $t2[1])) "box left $($b.L), second tab $($d.l + $t2[0]) to $($d.l + $t2[0] + $t2[1])"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  PfQuit
}

if ($Part -eq 'D') {
  # The first-run fences (a column at the right, snapped) over other sizes and scales.
  $files = 1..40 | ForEach-Object { "列文件 $_.txt" }
  foreach ($n in $files) { DeskFile $n | Out-Null }
  PfStart
  Start-Sleep -Seconds 7
  $d0 = "$((HsWin 'Programs').y) $((HsWin 'Folders').y) $((HsWin 'Files and documents').y)"
  Log "first-run column at 1600x900: tops $d0"
  function ColumnChecks($what, $gapPx) {
    $p = HsWin 'Programs'; $fo = HsWin 'Folders'; $fi = HsWin 'Files and documents'
    $wa = [HsScr]::Work()
    Log "$what Programs $($p.x),$($p.y),$($p.w),$($p.h) Folders $($fo.x),$($fo.y),$($fo.w),$($fo.h) Files $($fi.x),$($fi.y),$($fi.w),$($fi.h) work $($wa -join ',')"
    ParkMouse 400 400
    Shot "D $what"
    Check "$what the second fence of the column is one gap under the first" (($fo.y - ($p.y + $p.h)) -eq $gapPx) "gap $($fo.y - ($p.y + $p.h)) px, wanted $gapPx"
    Check "$what the column is one gap from the top and the right edge" ($p.y -eq $gapPx -and ($wa[2] - ($p.x + $p.w)) -eq $gapPx) "top $($p.y), right gap $($wa[2] - ($p.x + $p.w))"
    Check "$what the tall fence keeps the gap to the top and stays off the taskbar" ($fi.y -eq $gapPx -and ($fi.y + $fi.h) -le ($wa[3] - $gapPx)) "top $($fi.y), bottom $($fi.y + $fi.h), work bottom $($wa[3])"
    Check "$what the tall fence is one gap left of the column" (($p.x - ($fi.x + $fi.w)) -eq $gapPx) "gap $($p.x - ($fi.x + $fi.w))"
  }
  Case 'first-run column at 1280x720'
  WantSize 1280 720
  ColumnChecks '1280x720:' 8
  Case 'first-run column at 150 %'
  WantSize 1600 900 150
  ColumnChecks '150 %:' 12
  Case 'first-run column at 125 % after 150 %'
  WantSize 1600 900 125
  ColumnChecks '125 % after 150 %:' 10
  Case 'first-run column at 1920x1080 after 125 %'
  WantSize 1920 1080
  ColumnChecks '1920x1080 after 125 %:' 8
  Case 'back at 1600x900'
  WantSize 1600 900
  $d1 = "$((HsWin 'Programs').y) $((HsWin 'Folders').y) $((HsWin 'Files and documents').y)"
  Check 'back at 1600x900 the column is where it was' ($d1 -eq $d0) "before $d0 | now $d1"
  PfQuit
  foreach ($n in $files) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
}

Case 'end'
# The box as the next job expects it: the taskbar stays, animations off (the RDP default).
if ($Part -eq 'G') { [HsScr]::TaskbarAutoHide($false); [Anim]::Set($false) }
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue
