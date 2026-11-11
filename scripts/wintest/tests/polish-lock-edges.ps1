# Fence polish round 2026-11-11, package lock-edges: corner resize zones (C4), the minimum width
# that keeps the start of the title (UX9), the lock glyph of a locked fence and its pulse when
# the lock refuses a drag (UX4 / MF-8), the rectangle that stays while 「＋ 新建栅栏」 is offered
# (MF-6a), the shadow that keeps up with a dragged edge (MF-9). Parts (run.sh --var Part=...):
# C corners and minimum width, L the lock glyph, M the rectangle on the desktop, S the shadow.
# Default: all parts. Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'CLMS' }
trap { PfTrap $_; continue }
Case 'setup'
CloseWindows
$PlNames = @('pl 季度报告.txt', 'pl 预算表 2026.txt', 'pl 会议记录与分工安排.txt', 'pl notes.txt')
function PlClean() { foreach ($n in $PlNames) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
PlClean
foreach ($n in $PlNames) { DeskFile $n | Out-Null }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
# Window contents while dragging (an RDP session starts with the outline only).
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
GreyWallpaper
. C:\pf\kit\rec.ps1
PfStart -Script @('sleep 3600000')
Start-Sleep -Seconds 5
ClearFences
function PlWin($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function PlRect($t) { return (Rect (PlWin $t)) }
function PlPark() { ParkMouse 1300 760; Start-Sleep -Milliseconds 500 }
# Most ink (how far a pixel departs from its row's usual brightness) in a screen rectangle.
function PlInk($x, $y, $w, $h) { $b = [Px]::Grab($x, $y, $w, $h); $m = [Px]::MaxInk($b, 0, $w - 1, 0, $h - 1); $b.Dispose(); return $m }

if ($Part -match 'C') {
  # ---- C4: a corner answers along 16 px of each of its edges (it was the 6 x 6 px square).
  Case 'C4 corner resize zones'
  Pf fence create --title Corner --rect '300,300,320,260' | Out-Null
  Start-Sleep -Seconds 1
  $h = PlWin 'Corner'; $r = Rect $h
  # 17 = HTBOTTOMRIGHT, 15 = HTBOTTOM, 11 = HTRIGHT, 13 = HTTOPLEFT, 16 = HTBOTTOMLEFT, 14 = HTTOPRIGHT.
  $along = 0; for ($d = 1; $d -le 60; $d++) { if ((NcHit $h ($r.R - $d) ($r.B - 2)) -eq 17) { $along = $d } else { break } }
  $up = 0; for ($d = 1; $d -le 60; $d++) { if ((NcHit $h ($r.R - 2) ($r.B - $d)) -eq 17) { $up = $d } else { break } }
  Check 'C4 the bottom-right corner reaches 16 px along the bottom edge' ($along -eq 16) "HTBOTTOMRIGHT for $along px from the corner"
  Check 'C4 the bottom-right corner reaches 16 px up the right edge' ($up -eq 16) "HTBOTTOMRIGHT for $up px from the corner"
  Check 'C4 the middle of the bottom and right edges stays one-way' ((NcHit $h ($r.L + 160) ($r.B - 2)) -eq 15 -and (NcHit $h ($r.R - 2) ($r.T + 130)) -eq 11) "bottom=$(NcHit $h ($r.L + 160) ($r.B - 2)) right=$(NcHit $h ($r.R - 2) ($r.T + 130))"
  Check 'C4 the other corners answer 12 px along their edges too' ((NcHit $h ($r.L + 12) ($r.T + 2)) -eq 13 -and (NcHit $h ($r.L + 2) ($r.B - 12)) -eq 16 -and (NcHit $h ($r.R - 2) ($r.T + 12)) -eq 14) "top-left=$(NcHit $h ($r.L + 12) ($r.T + 2)) bottom-left=$(NcHit $h ($r.L + 2) ($r.B - 12)) top-right=$(NcHit $h ($r.R - 2) ($r.T + 12))"
  Check 'C4 the edge is still 6 px deep at the corner' ((NcHit $h ($r.R - 12) ($r.B - 8)) -eq 1) "8 px above the bottom edge, 12 px from the right: $(NcHit $h ($r.R - 12) ($r.B - 8)) (1 = content)"
  # A real drag that starts on the bottom edge 12 px from the corner sizes both ways.
  PlPark
  RecStart 'corner-drag' -X ($r.L - 30) -Y ($r.T - 30) -W 560 -H 450 -Fps 30
  RecMark 'pointer on the bottom edge, 12 px from the corner'
  [G]::MoveTo($r.R - 12, $r.B - 3); Start-Sleep -Milliseconds 900
  RecMark 'drag down and to the right'
  DragTo ($r.R - 12) ($r.B - 3) ($r.R + 150) ($r.B + 110) 14
  RecMark 'released'
  Start-Sleep -Milliseconds 600
  RecStop
  $a = PlRect 'Corner'
  Check 'C4 a drag from the bottom edge 12 px from the corner sizes width and height' (($a.R - $a.L) -gt ($r.R - $r.L) + 40 -and ($a.B - $a.T) -gt ($r.B - $r.T) + 40) "from $($r.R - $r.L)x$($r.B - $r.T) to $($a.R - $a.L)x$($a.B - $a.T)"
  Shot 'C4-after-corner-drag'
  Pf fence delete Corner | Out-Null; Start-Sleep -Milliseconds 600

  # ---- UX9: the narrowest width keeps the start of the title (it was one icon column: only 「…」).
  Case 'UX9 minimum width'
  Pf fence create --title '季度报告与工作计划' --rect '500,300,320,260' | Out-Null
  Start-Sleep -Seconds 1
  $r = PlRect '季度报告与工作计划'
  PlPark
  DragTo ($r.R - 2) ($r.T + 150) ($r.L - 60) ($r.T + 150) 16
  PlPark
  $a = PlRect '季度报告与工作计划'
  $w = $a.R - $a.L
  Crop 'UX9-narrowest' ($a.L - 20) ($a.T - 10) 200 120 | Out-Null
  # The title's ink: more than the three dots of an ellipsis alone, which are a few px high.
  $b = [Px]::Grab($a.L + 12, $a.T + 6, [Math]::Max(8, $w - 52), 24)
  $tall = 0; foreach ($y in 0..23) { if ([Px]::MaxInk($b, 0, $b.Width - 1, $y, $y) -gt 60) { $tall++ } }
  $b.Dispose()
  Check 'UX9 the fence stops wider than one icon column' ($w -gt 88 -and $w -lt 140) "width $w px after dragging the right edge past the left one"
  Check 'UX9 the title shows letters, not only the ellipsis' ($tall -ge 7) "title ink spans $tall rows (an ellipsis alone: 2 or 3)"
  Check 'UX9 the log says the title holds the edge' ([bool](PfWaitLog 'sizing: the width stops where the title still shows' 3)) ''
  # A short name needs no more than its own width; a rolled fence keeps the same minimum.
  Pf fence rename '季度报告与工作计划' 'Docs' | Out-Null; Start-Sleep -Milliseconds 600
  $r = PlRect 'Docs'
  DragTo ($r.R - 2) ($r.T + 150) ($r.R + 200) ($r.T + 150) 10
  $r = PlRect 'Docs'
  DragTo ($r.R - 2) ($r.T + 150) ($r.L - 60) ($r.T + 150) 16
  PlPark
  $a = PlRect 'Docs'
  Check 'UX9 a short title is not cut either' (($a.R - $a.L) -ge 80 -and ($a.R - $a.L) -lt 140) "width $($a.R - $a.L) px"
  Shot 'UX9-short-title'
  Pf fence delete Docs | Out-Null; Start-Sleep -Milliseconds 600
}

if ($Part -match 'L') {
  # ---- UX4 / MF-8: a locked fence shows a lock glyph before the chevron while the pointer is
  # over it; a drag the lock refuses pulses it.
  Case 'UX4 lock glyph'
  Pf fence create --title 'Locked reports of the quarter' --rect '300,300,340,240' | Out-Null
  Pf fence create --title 'Free' --rect '700,300,340,240' | Out-Null
  Pf item move "$PfDesk\pl 季度报告.txt" "$PfDesk\pl 预算表 2026.txt" --to 'Locked reports of the quarter' | Out-Null
  Pf item move "$PfDesk\pl notes.txt" --to 'Free' | Out-Null
  Start-Sleep -Seconds 1
  Pf fence set 'Locked reports of the quarter' locked true | Out-Null
  Start-Sleep -Milliseconds 600
  $L = PlRect 'Locked reports of the quarter'; $F = PlRect 'Free'
  # The lock glyph's slot: 16 px wide, right before the chevron's 32 px box (4 px from the edge).
  function SlotInk($r) { return (PlInk ($r.R - 52) ($r.T + 9) 16 18) }
  PlPark
  $restL = SlotInk $L
  [G]::MoveTo($L.L + 170, $L.T + 150); Start-Sleep -Milliseconds 600
  $overL = SlotInk $L
  Crop 'UX4-locked-pointer-over' ($L.L - 10) ($L.T - 10) 360 120 | Out-Null
  [G]::MoveTo($F.L + 170, $F.T + 150); Start-Sleep -Milliseconds 600
  $overF = SlotInk $F
  Crop 'UX4-free-pointer-over' ($F.L - 10) ($F.T - 10) 360 120 | Out-Null
  Check 'UX4 the lock glyph shows while the pointer is over a locked fence' ($overL -ge 40 -and $restL -lt 25) "ink in the slot: pointer over the fence $overL, pointer away $restL"
  Check 'UX4 a fence that is not locked shows no glyph' ($overF -lt 25) "ink in the slot of the unlocked fence: $overF"
  Check 'UX4 the long title ends before the glyph (nothing in its slot at rest)' ($restL -lt 25) "ink at rest $restL"

  # A title drag: the fence stays, the glyph pulses (log line; the recording shows the dip).
  Case 'UX4 refused drags'
  $n0 = @(PfLogAll | Where-Object { $_ -match 'the lock refused a drag' }).Count
  RecStart 'lock-title-drag' -X ($L.L - 30) -Y ($L.T - 30) -W 520 -H 330 -Fps 30
  RecMark 'pointer over the locked fence'
  [G]::MoveTo($L.L + 120, $L.T + 150); Start-Sleep -Milliseconds 900
  RecMark 'drag the title'
  [G]::MoveTo($L.L + 120, $L.T + 18); Start-Sleep -Milliseconds 400
  MDown; Start-Sleep -Milliseconds 150
  foreach ($s in 1..10) { [G]::MoveTo($L.L + 120 + $s * 12, $L.T + 18 + $s * 2); Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 500
  MUp
  RecMark 'released'
  Start-Sleep -Milliseconds 700
  RecStop
  $a = PlRect 'Locked reports of the quarter'
  $n1 = @(PfLogAll | Where-Object { $_ -match 'the lock refused a drag' }).Count
  Check 'UX4 a title drag leaves the locked fence where it is' ($a.L -eq $L.L -and $a.T -eq $L.T) "at $($a.L),$($a.T), was $($L.L),$($L.T)"
  Check 'UX4 the refused title drag pulses the lock glyph' ($n1 -eq $n0 + 1) "pulse lines in the log: $n0 before, $n1 after"
  # An edge drag (where the left edge would resize).
  PlPark
  [G]::MoveTo($L.L + 2, $L.T + 150); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 150
  foreach ($s in 1..8) { [G]::MoveTo($L.L + 2 - $s * 8, $L.T + 150); Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 400
  MUp; Start-Sleep -Milliseconds 500
  $a = PlRect 'Locked reports of the quarter'
  $n2 = @(PfLogAll | Where-Object { $_ -match 'the lock refused a drag' }).Count
  Check 'UX4 an edge drag leaves the size and pulses the glyph too' ($a.L -eq $L.L -and $a.R -eq $L.R -and $n2 -eq $n1 + 1) "left $($a.L) right $($a.R); pulse lines $n1 -> $n2"
  # A plain click on the title is no drag: no pulse.
  PlPark
  [G]::Click($L.L + 120, $L.T + 18); Start-Sleep -Milliseconds 500
  $n3 = @(PfLogAll | Where-Object { $_ -match 'the lock refused a drag' }).Count
  Check 'UX4 a click on the title does not pulse' ($n3 -eq $n2) "pulse lines $n2 -> $n3"
  # The same drag on the unlocked fence moves it, with no pulse.
  DragTo ($F.L + 120) ($F.T + 18) ($F.L + 180) ($F.T + 60) 10
  $fa = PlRect 'Free'
  $n4 = @(PfLogAll | Where-Object { $_ -match 'the lock refused a drag' }).Count
  Check 'UX4 the unlocked fence still moves, without a pulse' ($fa.L -ne $F.L -and $n4 -eq $n3) "Free at $($fa.L),$($fa.T), was $($F.L),$($F.T); pulse lines $n3 -> $n4"

  Case 'UX4 infotip, rolled, tabs, unlock'
  # The glyph's infotip.
  PlPark
  [G]::MoveTo($L.L + 100, $L.T + 150); Start-Sleep -Milliseconds 500
  [G]::MoveTo($L.R - 44, $L.T + 18); Start-Sleep -Milliseconds 1800
  $tip = OcrText ($L.R - 260) ($L.T + 20) 560 70 'UX4-tip'
  Shot 'UX4-infotip'
  Check 'UX4 the glyph has an infotip that says how to unlock' ($tip -match 'locked' -and $tip -match 'nlock') "read: $tip"
  # Rolled: the item count moves aside, the glyph still shows.
  PlPark
  Pf fence roll 'Locked reports of the quarter' | Out-Null; Start-Sleep -Milliseconds 900
  $R = PlRect 'Locked reports of the quarter'
  # At rest: the item count ends before the glyph's slot, which is empty.
  $countInk = PlInk ($R.R - 112) ($R.T + 9) 58 18
  $restSlot = SlotInk $R
  Crop 'UX4-rolled-locked-rest' ($R.L - 10) ($R.T - 10) 360 60 | Out-Null
  Check 'UX4 rolled: the item count sits before the glyph slot' ($countInk -ge 30 -and $restSlot -lt 25) "ink of the count $countInk, in the slot $restSlot"
  # Pointer on the title, before the hover peek opens the fence.
  [G]::MoveTo($R.L + 60, $R.T + 18); Start-Sleep -Milliseconds 250
  $rolledInk = SlotInk $R
  Crop 'UX4-rolled-locked' ($R.L - 10) ($R.T - 10) 360 60 | Out-Null
  Check 'UX4 a rolled locked fence shows the glyph before the chevron' ($rolledInk -ge 40) "ink in the slot $rolledInk"
  PlPark
  Pf fence unroll 'Locked reports of the quarter' | Out-Null; Start-Sleep -Milliseconds 900
  # Unlocked again: no glyph, and the title gets the slot back.
  Pf fence set 'Locked reports of the quarter' locked false | Out-Null; Start-Sleep -Milliseconds 500
  [G]::MoveTo($L.L + 170, $L.T + 150); Start-Sleep -Milliseconds 600
  $unlocked = SlotInk $L
  Crop 'UX4-unlocked-again' ($L.L - 10) ($L.T - 10) 360 120 | Out-Null
  Check 'UX4 unlocking takes the glyph away' ($unlocked -lt 25 -or $unlocked -ne $overL) "ink in the slot $unlocked (locked: $overL)"
  # Tabs: the strip ends before the glyph's slot; a left tab column shows it by its chevron.
  PlPark
  Pf fence set 'Locked reports of the quarter' locked true | Out-Null
  Pf fence merge 'Free' --into 'Locked reports of the quarter' | Out-Null; Start-Sleep -Seconds 1
  $T = PlRect 'Locked reports of the quarter'
  if ($T.R -eq 0) { $T = PlRect 'Free' }
  [G]::MoveTo($T.L + 170, $T.T + 150); Start-Sleep -Milliseconds 700
  $tabsInk = SlotInk $T
  Crop 'UX4-tabs-locked' ($T.L - 10) ($T.T - 10) 360 120 | Out-Null
  Check 'UX4 a locked fence with tabs shows the glyph' ($tabsInk -ge 40) "ink in the slot $tabsInk"
  Pf settings set tabBar left | Out-Null; Start-Sleep -Seconds 1
  [G]::MoveTo($T.L + 220, $T.T + 150); Start-Sleep -Milliseconds 700
  Shot 'UX4-left-tabs-locked'
  Pf settings set tabBar top | Out-Null; Start-Sleep -Milliseconds 600
  PlPark
  ClearFences
}

if ($Part -match 'M') {
  # ---- MF-6a: the rectangle drawn on the desktop stays, fainter, while the buttons are offered.
  Case 'MF-6a rectangle with the offer'
  PlPark
  $p = FreeRect 320 220 500 250
  $x0 = $p[0]; $y0 = $p[1]
  RecStart 'marquee-offer' -X ($x0 - 40) -Y ($y0 - 40) -W 520 -H 380 -Fps 30
  RecMark 'draw a rectangle on the desktop'
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 150
  foreach ($s in 1..12) { [G]::MoveTo($x0 + $s * 25, $y0 + $s * 16); Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 300
  $during = PlInk ($x0 - 8) ($y0 + 60) 16 40
  MUp
  RecMark 'released: the buttons are offered'
  Start-Sleep -Milliseconds 700
  $offer = WaitWin 'PecoFence.MarqueeOffer' 3
  $after = PlInk ($x0 - 8) ($y0 + 60) 16 40
  Shot 'MF6a-offer'
  Start-Sleep -Milliseconds 4500
  RecMark 'the buttons have gone'
  $gone = PlInk ($x0 - 8) ($y0 + 60) 16 40
  Start-Sleep -Milliseconds 400
  RecStop
  Check 'MF-6a the buttons are offered after the release' ($offer -ne [IntPtr]::Zero) "offer window $offer"
  Check 'MF-6a the rectangle is still on screen while they are' ($after -ge 12) "ink on the left edge: while drawing $during, with the buttons $after"
  Check 'MF-6a it is fainter than while it was drawn' ($after -lt $during) "while drawing $during, with the buttons $after"
  Check 'MF-6a it goes when the buttons go' ($gone -lt 8) "ink on the left edge after the buttons went: $gone"
}

if ($Part -match 'S') {
  # ---- MF-9: while an edge is dragged the shadow gets a new size about as often as the window.
  Case 'MF-9 shadow during an edge drag'
  Add-Type -TypeDefinition @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices; using System.Threading;
public static class PlLag {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  static Thread t; static volatile bool on; static IntPtr body, shadow;
  public static int BodySizes, ShadowSizes, MaxLagMs, MaxBehindPx, Samples;
  static int margin, lastB, lastS;
  // The margin is read here, before the caller moves the pointer (the thread starts later).
  public static void Start(IntPtr b, IntPtr s) {
    body = b; shadow = s; RECT rb, rs; GetWindowRect(body, out rb); GetWindowRect(shadow, out rs);
    margin = (rs.R - rs.L) - (rb.R - rb.L); lastB = rb.R - rb.L; lastS = rs.R - rs.L;
    BodySizes = 0; ShadowSizes = 0; MaxLagMs = 0; MaxBehindPx = 0; Samples = 0;
    on = true; t = new Thread(Loop); t.IsBackground = true; t.Start();
  }
  // Every millisecond: the fence's width and its shadow's. The shadow is in step when it is
  // wider than the fence by the same margin as at the start.
  static void Loop() {
    RECT b, s;
    var sw = Stopwatch.StartNew(); long behindSince = -1;
    while (on) {
      GetWindowRect(body, out b); GetWindowRect(shadow, out s); Samples++;
      int wb = b.R - b.L, ws = s.R - s.L;
      if (wb != lastB) { BodySizes++; lastB = wb; }
      if (ws != lastS) { ShadowSizes++; lastS = ws; }
      int behind = Math.Abs(wb + margin - ws);
      if (behind > 0) {
        if (behindSince < 0) behindSince = sw.ElapsedMilliseconds;
        MaxLagMs = Math.Max(MaxLagMs, (int)(sw.ElapsedMilliseconds - behindSince));
        MaxBehindPx = Math.Max(MaxBehindPx, behind);
      } else behindSince = -1;
      Thread.Sleep(1);
    }
  }
  public static void Stop() { on = false; if (t != null) t.Join(500); }
}
'@
  Pf fence create --title Shade --rect '300,250,360,300' | Out-Null
  Pf item move "$PfDesk\pl 季度报告.txt" "$PfDesk\pl 预算表 2026.txt" "$PfDesk\pl notes.txt" --to Shade | Out-Null
  Start-Sleep -Seconds 1
  $h = PlWin 'Shade'; $r = Rect $h
  # Its shadow: the shadow window around it.
  $sh = @([W]::Tops('PecoFence.Shadow') | Where-Object { $q = Rect $_; $q.L -lt $r.L -and $q.R -gt $r.R -and $q.T -le $r.T -and $q.B -gt $r.B } | Sort-Object { $q = Rect $_; $q.R - $q.L } | Select-Object -First 1)[0]
  PlPark
  [G]::MoveTo($r.R - 2, $r.T + 150); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 200
  [PlLag]::Start($h, $sh)
  # Out 260 px and back in, a few px per step, as a hand does.
  foreach ($s in 1..65) { [G]::MoveTo($r.R - 2 + $s * 4, $r.T + 150); Start-Sleep -Milliseconds 16 }
  foreach ($s in 1..65) { [G]::MoveTo($r.R - 2 + 260 - $s * 4, $r.T + 150); Start-Sleep -Milliseconds 16 }
  Start-Sleep -Milliseconds 300
  [PlLag]::Stop()
  MUp; Start-Sleep -Milliseconds 600
  $detail = "window sizes $([PlLag]::BodySizes), shadow sizes $([PlLag]::ShadowSizes), longest time behind $([PlLag]::MaxLagMs) ms, most px behind $([PlLag]::MaxBehindPx), samples $([PlLag]::Samples)"
  Log "MF-9 $detail"
  Check 'MF-9 the shadow window was found and the edge moved' ($sh -ne $null -and [PlLag]::BodySizes -ge 40) $detail
  Check 'MF-9 the shadow takes at least three sizes for every four of the window' ([PlLag]::ShadowSizes * 4 -ge [PlLag]::BodySizes * 3) $detail
  Check 'MF-9 the shadow is never more than 60 ms behind the edge' ([PlLag]::MaxLagMs -le 60) $detail
  $line = @(PfLogAll | Where-Object { $_ -match 'size loop ended: how often the shadow followed' } | Select-Object -Last 1)
  Log "MF-9 log: $line"
  Pf fence delete Shade | Out-Null; Start-Sleep -Milliseconds 500
}

Case 'cleanup'
if (PfAlive) { ClearFences }
PlClean
[G]::SystemParametersInfoW(0x25, 0, [NullString]::Value, 3) | Out-Null
[Anim]::Set($AnimWas)
