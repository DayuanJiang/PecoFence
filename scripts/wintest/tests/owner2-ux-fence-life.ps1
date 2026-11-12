# Owner round 2026-11-12, UI / UX / motion hunt, package ux-fence-life: the life of a fence as a
# window. Parts (run.sh --var Part=...): D the icons of 「桌面」 keep their cells while a deleted
# fence's icons come back (FL-1), N the name box of a fence (FL-2), K 停靠到屏幕顶部 glides
# (FL-5), U Ctrl+Z after a new fence (FL-13), X a start without an empty desktop (FL-3).
# Default: all parts. Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'DNKUX' }
trap { PfTrap $_; continue }
Case 'setup'
CloseWindows
$FlNames = @('fl 季度报告.txt', 'fl 预算表 2026.txt', 'fl 会议记录与分工安排.txt', 'fl notes.txt', 'fl 清单 a.txt', 'fl 清单 b.txt')
function FlClean() { foreach ($n in $FlNames) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
FlClean
foreach ($n in $FlNames) { DeskFile $n | Out-Null }
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
GreyWallpaper
. C:\pf\kit\rec.ps1
PfStart -Lang zh-CN -Script @('sleep 3600000')
Start-Sleep -Seconds 4
ClearFences
Start-Sleep -Seconds 1
function FlWin($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function FlRect($t) { return (Rect (FlWin $t)) }
function FlPark() { ParkMouse 1300 760; Start-Sleep -Milliseconds 400 }
# pecofence-cli without waiting for it (one argument string, so no title with a space).
function FlCliAsync($argLine) { Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList $argLine -WindowStyle Hidden | Out-Null }
# The first ink column of a screen rectangle and the first and last ink rows of the 12 px
# after it (the first character): @(x, top, bottom) in screen px, -1 when there is no ink.
function FlTextAt($x, $y, $w, $h) {
  $b = [Px]::Grab($x, $y, $w, $h)
  $left = [Px]::FirstInk($b, 0, $w - 1, 0, $h - 1, 60)
  $top = -1; $bot = -1
  if ($left -ge 0) {
    foreach ($r in 0..($h - 1)) { if ([Px]::MaxInk($b, $left, [Math]::Min($w - 1, $left + 12), $r, $r) -ge 60) { if ($top -lt 0) { $top = $r }; $bot = $r } }
  }
  $b.Dispose()
  if ($left -lt 0) { return @(-1, -1, -1) }
  return @(($x + $left), ($y + $top), ($y + $bot))
}

if ($Part -match 'D') {
  # ---- FL-1: 「桌面」 arranged like the desktop. A fence with three icons is deleted: they come
  # back to 「桌面」 and the icons that were there stay in their cells. They used to glide to the
  # cells the old list had at their index and jump back 250 ms later.
  Case 'FL-1 desktop icons keep their cells'
  Pf fence create --title '工作文件' --rect '560,240,340,220' | Out-Null
  Pf item move "$PfDesk\fl 季度报告.txt" "$PfDesk\fl 预算表 2026.txt" "$PfDesk\fl 会议记录与分工安排.txt" --to '工作文件' | Out-Null
  Start-Sleep -Seconds 2
  FlPark
  $inbox = (@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' } | Select-Object -First 1)
  Check 'FL-1 「桌面」 is arranged like the desktop' ([bool]$inbox.desktopLayout) "desktopLayout=$($inbox.desktopLayout) freePlacement=$($inbox.freePlacement)"
  # The Recycle Bin stands in the first cell (top-left of the work area) before and after.
  FlCliAsync 'fence delete 工作文件'
  [Px]::Burst(8, 8, 72, 84, 1100, 0)
  $n = [Px]::Frames.Count
  $away = 0; $worst = 0.0
  for ($i = 1; $i -lt $n; $i++) { $d = [Px]::Diff([Px]::Frames[0], [Px]::Frames[$i], 0, 71, 0, 83); if ($d -gt $worst) { $worst = $d }; if ($d -gt 4) { $away++ } }
  $back = WaitItemFence 'fl 季度报告.txt' '桌面' 5
  Check 'FL-1 the fence was deleted and its icons are back in 「桌面」' ($back -eq '桌面' -and (FlWin '工作文件') -eq [IntPtr]::Zero) "fl 季度报告 is in '$back'"
  Check 'FL-1 the Recycle Bin stays in its cell meanwhile' ($away -eq 0) "$away of $n frames show something else in its cell (largest difference $([Math]::Round($worst, 1)))"
  # The same once more for the report.
  Pf fence create --title '工作文件' --rect '560,240,340,220' | Out-Null
  Pf item move "$PfDesk\fl 季度报告.txt" "$PfDesk\fl 预算表 2026.txt" "$PfDesk\fl 会议记录与分工安排.txt" --to '工作文件' | Out-Null
  Start-Sleep -Seconds 2
  FlPark
  RecStart 'fl1-delete' -X 0 -Y 0 -W 960 -H 720 -Fps 30
  RecMark '删除一个装着 3 个图标的栅栏'
  Start-Sleep -Milliseconds 500
  Pf fence delete '工作文件' | Out-Null
  Start-Sleep -Milliseconds 1500
  RecStop
}

if ($Part -match 'N') {
  # ---- FL-2: the name box of a fence opens with its text where the title's was, in a frame.
  Case 'FL-2 fence name box'
  ClearFences
  Pf fence create --title '工作文件 2026' --rect '560,240,340,220' | Out-Null
  Start-Sleep -Seconds 1
  FlPark
  $r = FlRect '工作文件 2026'
  $title = FlTextAt ($r.L + 4) ($r.T + 4) 220 28
  Crop 'fl2-title' ($r.L - 8) ($r.T - 8) 360 60 | Out-Null
  [G]::RClick($r.L + 200, $r.T + 14)
  $m = MenuWin 5; Start-Sleep -Milliseconds 300
  MenuPick '^重命名' $m | Out-Null
  $box = WaitWin 'PecoFence.Rename' 5
  Start-Sleep -Milliseconds 600
  # The caret to the end: the name is no longer selected (white on blue) and can be measured.
  [G]::Press(0x23); Start-Sleep -Milliseconds 400
  $b = Rect $box
  $inside = FlTextAt ($b.L + 2) ($b.T + 2) ($b.R - $b.L - 4) ($b.B - $b.T - 5)
  Crop 'fl2-box' ($r.L - 8) ($r.T - 8) 360 60 | Out-Null
  # The box's edge against its field, at mid height: a frame is darker than the field.
  $edge = [Px]::Grab($b.L, $b.T + [int](($b.B - $b.T) / 2), 6, 1)
  $pe = [Px]::Pixels($edge); $edge.Dispose()
  $edgeLum = [int]((($pe[0] -shr 16) -band 255) * 0.299 + (($pe[0] -shr 8) -band 255) * 0.587 + ($pe[0] -band 255) * 0.114)
  $fieldLum = [int]((($pe[4] -shr 16) -band 255) * 0.299 + (($pe[4] -shr 8) -band 255) * 0.587 + ($pe[4] -band 255) * 0.114)
  Check 'FL-2 the name box opened over the title' ($box -ne [IntPtr]::Zero -and $title[0] -gt 0 -and $inside[0] -gt 0) "title text at $($title -join ','), box $($b.L),$($b.T),$($b.R),$($b.B), its text at $($inside -join ',')"
  Check 'FL-2 the name starts at the same x in the box' ([Math]::Abs($inside[0] - $title[0]) -le 1) "title x $($title[0]), in the box x $($inside[0])"
  Check 'FL-2 the name stays at the same height in the box' ([Math]::Abs($inside[1] - $title[1]) -le 1 -and [Math]::Abs($inside[2] - $title[2]) -le 1) "title rows $($title[1]) to $($title[2]), in the box $($inside[1]) to $($inside[2])"
  Check 'FL-2 the box has an outline' ($fieldLum - $edgeLum -ge 20) "edge brightness $edgeLum, field $fieldLum"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  FlPark
}

if ($Part -match 'K') {
  # ---- FL-5: 停靠到屏幕顶部 moves the fence to the top edge in steps (a 167 ms glide), not in one.
  Case 'FL-5 dock to top glides'
  ClearFences
  Pf fence create --title '抽屉' --rect '700,300,320,240' | Out-Null
  Pf item move "$PfDesk\fl notes.txt" "$PfDesk\fl 清单 a.txt" --to '抽屉' | Out-Null
  Start-Sleep -Seconds 1
  FlPark
  $h = FlWin '抽屉'
  $startTop = (Rect $h).T
  RecStart 'fl5-dock' -X 640 -Y 0 -W 440 -H 600 -Fps 30
  RecMark '停靠到屏幕顶部'
  Start-Sleep -Milliseconds 400
  FlCliAsync 'fence dock-top 抽屉'
  $tops = New-Object System.Collections.Generic.List[int]
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 1500) { $t = (Rect $h).T; if ($tops.Count -eq 0 -or $tops[$tops.Count - 1] -ne $t) { $tops.Add($t) }; Start-Sleep -Milliseconds 4 }
  RecStop
  $between = @($tops | Where-Object { $_ -gt 0 -and $_ -lt $startTop }).Count
  $end = Rect $h
  Check 'FL-5 the fence ends rolled up at the top edge' ($end.T -eq 0 -and ($end.B - $end.T) -lt 60) "top $($end.T), height $($end.B - $end.T)"
  Check 'FL-5 it passes places in between on its way' ($between -ge 3) "tops seen: $($tops -join ' ') ($between between $startTop and 0)"
  Pf fence delete '抽屉' | Out-Null
}

if ($Part -match 'U') {
  # ---- FL-13: Ctrl+Z right after a new fence takes that fence away again. It had no undo step
  # of its own, so Ctrl+Z undid what came before it (here: put another fence back).
  Case 'FL-13 undo a new fence'
  ClearFences
  Pf fence create --title '旧的' --rect '300,420,300,200' | Out-Null
  Start-Sleep -Seconds 1
  FlPark
  $o = FlRect '旧的'
  DragTo ($o.L + 150) ($o.T + 14) ($o.L + 210) ($o.T + 94) 10
  $moved = FlRect '旧的'
  FlPark
  $p = FreeRect 320 200 760 120
  $x0 = $p[0]; $y0 = $p[1]
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 150
  foreach ($s in 1..12) { [G]::MoveTo($x0 + $s * 25, $y0 + $s * 15); Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 300
  MUp
  $offer = WaitWin 'PecoFence.MarqueeOffer' 3
  Start-Sleep -Milliseconds 500
  $or = Rect $offer
  # 「＋ 新建栅栏」 is the right one of the two buttons.
  $sw = [Diagnostics.Stopwatch]::StartNew()
  [G]::Click($or.R - 50, [int](($or.T + $or.B) / 2))
  $boxMs = -1
  while ($sw.ElapsedMilliseconds -lt 3000) { $bx = [G]::FindWindowW('PecoFence.Rename', 'rename'); if ($bx -ne [IntPtr]::Zero -and [G]::IsWindowVisible($bx)) { $boxMs = $sw.ElapsedMilliseconds; break }; Start-Sleep -Milliseconds 5 }
  Start-Sleep -Milliseconds 500
  # FL-14: the name box opens when the fence has faded in (167 ms). It waited for the whole
  # entrance to be reported (the 250 ms scale and the compositor's answer: about 0.3 s), and
  # what was typed meanwhile went nowhere.
  $nb = @(PfLogAll | Where-Object { $_ -match 'name box.*waited_ms=\d+' } | Select-Object -Last 1)
  $waited = -1; if ($nb.Count -and $nb[0] -match 'waited_ms=(\d+)') { $waited = [int]$Matches[1] }
  Check 'FL-14 the name box of a new fence opens when the fence has faded in' ($waited -ge 0 -and $waited -le 230) "the app waited $waited ms for the fence; the box was on screen $boxMs ms after the click"
  # Esc: the fence keeps the name 新栅栏 and takes the keyboard.
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  $made = (FlWin '新栅栏') -ne [IntPtr]::Zero
  $fg = Fg
  Chord 0x11 0x5A
  Start-Sleep -Milliseconds 1200
  $after = FlRect '旧的'
  Check 'FL-13 the rectangle made a fence and it has the keyboard' ($made -and $fg -match '新栅栏') "made=$made fg=$fg"
  Check 'FL-13 Ctrl+Z takes the new fence away' ((FlWin '新栅栏') -eq [IntPtr]::Zero) "新栅栏 still there: $((FlWin '新栅栏') -ne [IntPtr]::Zero)"
  Check 'FL-13 the fence moved before it stays where it was put' ($moved.L -ne $o.L -and $after.L -eq $moved.L -and $after.T -eq $moved.T) "started at $($o.L),$($o.T), dragged to $($moved.L),$($moved.T), after Ctrl+Z at $($after.L),$($after.T)"
  FlPark
}

if ($Part -match 'X') {
  # ---- FL-3: at a start the Windows desktop icons go while the fences fade in. They used to
  # go about 0.1 s before the first fence showed: an empty desktop in between.
  Case 'FL-3 start without an empty desktop'
  PfFresh
  Pf settings set language zh-CN | Out-Null
  Start-Sleep -Seconds 3
  # A first-run fence at the right that holds shortcuts, and the second cell of Windows' own
  # icon column at the left (the Recycle Bin's, the first, also shows in 「桌面」).
  $prog = (@(PfJson fence list) | Where-Object { $_ -and $_.title -eq '程序' } | Select-Object -First 1)
  $fr = FlRect '程序'
  Check 'FL-3 the first run made the 程序 fence with icons in it' ($fr.R -gt $fr.L -and $prog.itemCount -ge 2) "rect $($fr.L),$($fr.T),$($fr.R),$($fr.B) items $($prog.itemCount)"
  PfQuit
  for ($i = 0; $i -lt 40 -and -not [W]::IconsVisible(); $i++) { Start-Sleep -Milliseconds 100 }
  Start-Sleep -Milliseconds 1500
  FlPark
  # One strip across the screen per sample (a grab costs about 25 ms whatever its size): the
  # rows of the Windows icon's cell, which are the rows of the fence's first icon names. Ink
  # is how far a pixel is from the wallpaper's grey: the Windows icon's white name at the
  # left, the fence's plate and names at the right.
  $sy = 104; $sh = 60; $fx = $fr.L + 10
  $b0 = [Px]::Grab(0, $sy, 1600, $sh); $w0 = [Px]::MaxInk($b0, 8, 80, 0, $sh - 1); $f0 = [Px]::MaxInk($b0, $fx, $fx + 200, 0, $sh - 1); $b0.Dispose()
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
  # Every sample: ms, ink of the Windows icon, ink of the fence. (No recording meanwhile.)
  $samples = New-Object System.Collections.Generic.List[object]
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 3000) {
    $t = $sw.ElapsedMilliseconds
    $bs = [Px]::Grab(0, $sy, 1600, $sh)
    $samples.Add(@($t, [Px]::MaxInk($bs, 8, 80, 0, $sh - 1), [Px]::MaxInk($bs, $fx, $fx + 200, 0, $sh - 1)))
    $bs.Dispose()
  }
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
  # Empty: from the first sample without the Windows icon to the first one with the fence.
  $gone = @($samples | Where-Object { $_[1] -lt 25 } | Select-Object -First 1)
  $came = @($samples | Where-Object { $_[2] -ge 25 } | Select-Object -First 1)
  $tGone = -1; if ($gone.Count) { $tGone = $gone[0][0] }
  $tCame = -1; if ($came.Count) { $tCame = $came[0][0] }
  $emptyMs = $tCame - $tGone
  Log ("FL-3 samples: " + ((@($samples | Where-Object { $_[0] -lt 900 }) | ForEach-Object { "$($_[0]):$($_[1])/$($_[2])" }) -join ' '))
  Check 'FL-3 the Windows icon showed before the start and went, the fence came' ($w0 -ge 40 -and $f0 -lt 25 -and $tGone -gt 0 -and $tCame -gt 0) "ink before: icon $w0, fence $f0; Windows icon gone at $tGone ms, fence there at $tCame ms ($($samples.Count) samples)"
  Check 'FL-3 the desktop is not empty in between' ($emptyMs -le 40) "the fence shows $emptyMs ms after the Windows icon went (negative: before it went)"
  Check 'FL-3 the Windows desktop icons are hidden after the start' (-not [W]::IconsVisible()) "visible=$([W]::IconsVisible())"
  Check 'FL-3 the log says when they went' ([bool](PfWaitLog 'start: Windows desktop icons hidden as the fences come in' 5)) ''
  # The same start once more for the report.
  PfQuit
  for ($i = 0; $i -lt 40 -and -not [W]::IconsVisible(); $i++) { Start-Sleep -Milliseconds 100 }
  Start-Sleep -Milliseconds 1500
  RecStart 'fl3-start' -X 0 -Y 0 -W 1600 -H 420 -Fps 30
  RecMark '启动 PecoFence'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
  Start-Sleep -Milliseconds 3500
  RecStop
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
FlClean
