# Owner round 2026-11-12, second batch, package ux-popups: what pops up over the desktop.
# Runs on a 1600x900 box at 100 %.
#   run.sh --var Part=N ...   the undo card: a second delete puts its own name on the card and
#                             its 撤销 brings that fence back; a click on 撤销 fades the card out
#                             (it vanished in one frame)
#   run.sh --var Part=S ...   the Peek search box: it never shows as a bare text field before its
#                             frame is drawn (the first search after a sign-in), and it fades in
#                             and out with the dimmer
#   run.sh --var Part=T ...   a fence title cut with an ellipsis shows all of it in a tooltip,
#                             and the arrow at its right says 收起 / 展开
#   run.sh --var Part=R ...   a name no file can have: the reason shows under the name box
#   run.sh --var Part=M ...   the icon menu's own entries have an icon each
#   run.sh --var Part=C ...   「添加规则」 on the card that offers a sorting rule: the card for the
#                             added rule takes its place where it stands (the app runs without
#                             a test script here: the offer does not show under one)
#   run.sh --var Part=K ...   the Menu key on an icon: the menu opens clear of the icon
# Without --var every part runs. Record=1 also records the clips for the report.
if (-not $Part) { $Part = 'NSTRMCK' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1
$UpFiles = @('预算表 2026.xlsx', '会议纪要 11月.txt', '年度报告 2025 董事会审阅版（含附录）.pdf', '报告.txt', 'a.md')
function UpClean() {
  foreach ($n in $UpFiles) { $p = Join-Path $PfDesk $n; if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue } }
}
function UpNotice() { $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value); if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }; return [IntPtr]::Zero }
function UpWaitNotice($sec = 6) { for ($i = 0; $i -lt $sec * 10; $i++) { $n = UpNotice; if ($n -ne [IntPtr]::Zero) { return $n }; Start-Sleep -Milliseconds 100 }; return [IntPtr]::Zero }
function UpFence($title) { return (@(PfJson fence list) | Where-Object { $_ -and $_.title -eq $title } | Select-Object -First 1) }
# 删除栅栏 from the fence's own menu (a right-click on its title).
function UpMenuDelete($title) {
  $f = UpFence $title
  [G]::RClick($f.windowRect.x + 120, $f.windowRect.y + 14); Start-Sleep -Milliseconds 900
  $m = MenuWin 4
  $p = MenuPick '^(Delete fence|删除栅栏)' $m
  Start-Sleep -Milliseconds 400
  return $p
}
# How far two colours are apart (the largest channel difference).
function UpFar($a, $b) { return [Math]::Max([Math]::Abs($a.R - $b.R), [Math]::Max([Math]::Abs($a.G - $b.G), [Math]::Abs($a.B - $b.B))) }

function UpPartN() {
  trap { PfTrap $_; continue }
  Case 'undo card: a second delete, and how the card leaves'
  UpClean
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '临时甲' --rect '200,120,300,200' | Out-Null
  Pf fence create --title '临时乙' --rect '520,120,300,200' | Out-Null
  Start-Sleep -Seconds 2
  [G]::MoveTo(900, 500)
  if ($Record) { RecStart 'N-card' -X 1180 -Y 600 -W 420 -H 260 -Fps 30 }
  if ($Record) { RecMark '删除栅栏“临时甲”' }
  UpMenuDelete '临时甲' | Out-Null
  $n = UpWaitNotice
  Start-Sleep -Milliseconds 700
  $t1 = [L]::Text($n)
  Check 'the undo card names the deleted fence' ($n -ne [IntPtr]::Zero -and $t1 -match '临时甲') "title='$t1'"
  if ($Record) { RecMark '接着删除栅栏“临时乙”' }
  UpMenuDelete '临时乙' | Out-Null
  Start-Sleep -Milliseconds 1200
  $n2 = UpNotice
  $t2 = [L]::Text($n2)
  Check 'a second delete puts its own name on the card' ($n2 -ne [IntPtr]::Zero -and $t2 -match '临时乙') "title='$t2' (was '$t1')"
  Check 'the card is the same window (no second card, no blink)' ($n2 -eq $n) "first=$n second=$n2"
  Shot 'n-second-delete'
  # The pointer rests on 撤销; a patch of the card's plate near its bottom-left corner is read
  # as fast as the screen allows while the button is clicked.
  $r = Rect $n2
  [G]::MoveTo($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 500
  # The button while the mouse button is held on it: another shade than under the resting pointer.
  # (Its left part, clear of the label; the press is let go off the card, so nothing is clicked.)
  $g = [Px]::Grab($r.R - 130, $r.B - 34, 4, 4); $hov = $g.GetPixel(1, 1); $g.Dispose()
  MDown; Start-Sleep -Milliseconds 250
  $g = [Px]::Grab($r.R - 130, $r.B - 34, 4, 4); $down = $g.GetPixel(1, 1); $g.Dispose()
  Crop 'n-button-pressed' ($r.L - 10) ($r.T - 10) ($r.R - $r.L + 20) ($r.B - $r.T + 20) | Out-Null
  [G]::MoveTo($r.L - 120, $r.T - 120); Start-Sleep -Milliseconds 150; MUp; Start-Sleep -Milliseconds 300
  Check 'the button shows pressed while the mouse button is held on it' ((UpFar $hov $down) -ge 8) "under the pointer $($hov.R),$($hov.G),$($hov.B); pressed $($down.R),$($down.G),$($down.B)"
  Check 'a press let go off the card clicks nothing' ((UpNotice) -eq $n2 -and $null -eq (UpFence '临时乙')) "notice=$(UpNotice)"
  [G]::MoveTo($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 500
  if ($Record) { RecMark '点“撤销”' }
  [Px]::Burst($r.L + 10, $r.B - 14, 6, 6, 600, 1)
  $first = [Px]::Frames[0].GetPixel(3, 3)
  $last = [Px]::Frames[[Px]::Frames.Count - 1].GetPixel(3, 3)
  $mid = 0; $midAt = @()
  for ($i = 1; $i -lt [Px]::Frames.Count; $i++) {
    $c = [Px]::Frames[$i].GetPixel(3, 3)
    if ((UpFar $c $first) -gt 12 -and (UpFar $c $last) -gt 12) { $mid++; $midAt += [Px]::Times[$i] }
  }
  $row = "frames=$([Px]::Frames.Count) card=$($first.R),$($first.G),$($first.B) after=$($last.R),$($last.G),$($last.B) in between: $mid frames ($(($midAt | Select-Object -First 1)) to $(($midAt | Select-Object -Last 1)) ms)"
  Log "undo click: $row"
  Check 'the card and the wallpaper under it differ enough to see a fade' ((UpFar $first $last) -gt 40) $row
  Check 'a click on 撤销 fades the card out (frames between card and wallpaper)' ($mid -ge 2) $row
  Start-Sleep -Milliseconds 900
  if ($Record) { RecStop }
  Check 'the card is gone after the click' ((UpNotice) -eq [IntPtr]::Zero) "notice=$(UpNotice)"
  Check '撤销 brought back the fence the card named (临时乙), not the older one' (($null -ne (UpFence '临时乙')) -and ($null -eq (UpFence '临时甲'))) "临时乙=$($null -ne (UpFence '临时乙')) 临时甲=$($null -ne (UpFence '临时甲'))"
  # Ctrl+Z still reaches the older delete.
  $f = UpFence '临时乙'
  [G]::Click($f.windowRect.x + 150, $f.windowRect.y + 100); Start-Sleep -Milliseconds 400
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 1200
  Check 'Ctrl+Z then brings back the older fence too' ($null -ne (UpFence '临时甲')) "临时甲=$($null -ne (UpFence '临时甲'))"
  # A new card after one that left by its button: it shows, and x still fades it out.
  UpMenuDelete '临时甲' | Out-Null
  $n3 = UpWaitNotice
  Start-Sleep -Milliseconds 700
  Check 'the next delete shows its card again' ($n3 -ne [IntPtr]::Zero -and [L]::Text($n3) -match '临时甲') "title='$([L]::Text($n3))'"
  $r = Rect $n3
  [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 900
  Check 'x closes the card' ((UpNotice) -eq [IntPtr]::Zero) "notice=$(UpNotice)"
  # With Windows' animations off the card goes at once.
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  UpMenuDelete '临时乙' | Out-Null
  $n4 = UpWaitNotice
  Start-Sleep -Milliseconds 700
  $r = Rect $n4
  [G]::MoveTo($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 400
  [Px]::Burst($r.L + 10, $r.B - 14, 6, 6, 500, 1)
  $first = [Px]::Frames[0].GetPixel(3, 3); $last = [Px]::Frames[[Px]::Frames.Count - 1].GetPixel(3, 3); $mid = 0
  for ($i = 1; $i -lt [Px]::Frames.Count; $i++) { $c = [Px]::Frames[$i].GetPixel(3, 3); if ((UpFar $c $first) -gt 12 -and (UpFar $c $last) -gt 12) { $mid++ } }
  Check 'animations off: the card goes at once' ($n4 -ne [IntPtr]::Zero -and $mid -eq 0 -and (UpFar $first $last) -gt 40) "frames=$([Px]::Frames.Count) in between: $mid"
  [Anim]::Set($anim)
  PfQuit
  UpClean
}

function UpPeekOn() { return ([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0 }
function UpWaitPeek($on, $sec = 4) { for ($i = 0; $i -lt $sec * 20 -and (UpPeekOn) -ne $on; $i++) { Start-Sleep -Milliseconds 50 }; return ((UpPeekOn) -eq $on) }
function UpBox() { $b = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value); if ($b -ne [IntPtr]::Zero -and [G]::IsWindowVisible($b)) { return $b }; return [IntPtr]::Zero }
function UpLum($c) { return [int](0.299 * $c.R + 0.587 * $c.G + 0.114 * $c.B) }
# The frames of a recording as bitmaps with their times: @{ T = ms; B = bitmap }.
function UpFrames($name) {
  $dir = "$PfShots\rec-$name"
  $out = @()
  foreach ($line in (Get-Content "$dir\frames.tsv")) {
    $p = $line -split "`t"
    if ($p.Count -lt 2) { continue }
    $f = Join-Path $dir ('{0:D5}.jpg' -f [int]$p[0])
    if (Test-Path $f) { $out += @{ T = [int]$p[1]; B = (New-Object Drawing.Bitmap $f) } }
  }
  return $out
}
function UpMarks($name) { $m = @{}; foreach ($line in (Get-Content "$PfShots\rec-$name\marks.tsv" -Encoding UTF8)) { $p = $line -split "`t", 2; if ($p.Count -eq 2) { $m[$p[1]] = [int]$p[0] } }; return $m }

function UpPartS() {
  trap { PfTrap $_; continue }
  Case 'search box: whole from its first frame, fades in and out'
  UpClean
  foreach ($n in $UpFiles) { DeskFile $n | Out-Null }
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '资料' --rect '60,120,360,300' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move @($UpFiles | ForEach-Object { "$PfDesk\$_" }) --to '资料' | Out-Null
  Start-Sleep -Seconds 2
  [G]::MoveTo(800, 800)
  # The box opens in the upper middle of the 1600x900 screen: 480,227, 640 wide. Two patches
  # per frame: A inside the text field, B in the box's left padding (drawn by the box itself).
  $bx = 480; $by = 227
  PeekKey; UpWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 900
  RecStart 'S-first' -X $bx -Y $by -W 640 -H 540 -Fps 30
  Start-Sleep -Milliseconds 400
  RecMark 'first letter'
  [G]::Type('b'); Start-Sleep -Milliseconds 1100
  $box = UpBox; $r1 = Rect $box
  RecMark 'second letter'
  [G]::Type('a'); Start-Sleep -Milliseconds 900
  $r2 = Rect $box
  RecMark 'Esc'
  [G]::Press(0x1B); UpWaitPeek $false | Out-Null; Start-Sleep -Milliseconds 800
  RecStop
  Check 'the box opened at its usual place' ($r1.L -eq $bx -and $r1.T -eq $by) "box=$($r1.L),$($r1.T),$($r1.R),$($r1.B)"
  $fr = UpFrames 'S-first'; $mk = UpMarks 'S-first'
  $a0 = UpLum $fr[0].B.GetPixel(300, 28); $b0 = UpLum $fr[0].B.GetPixel(8, 28)
  $t1 = $mk['second letter']; $t2 = $mk['Esc']
  $shown = $fr | Where-Object { $_.T -lt $t1 } | Select-Object -Last 1
  $a1 = UpLum $shown.B.GetPixel(300, 28); $b1 = UpLum $shown.B.GetPixel(8, 28)
  # C: a patch of desktop under the box's place that the box never covers (it tells how far
  # the dimmer has faded); at the end B and C are both bare desktop.
  $last = $fr[$fr.Count - 1]
  $bEnd = UpLum $last.B.GetPixel(8, 28); $cEnd = [Math]::Max(1, (UpLum $last.B.GetPixel(300, 520)))
  $bare = 0; $bareAt = @(); $in = 0; $out = 0; $series = @()
  foreach ($f in $fr) {
    $a = UpLum $f.B.GetPixel(300, 28); $b = UpLum $f.B.GetPixel(8, 28); $c = UpLum $f.B.GetPixel(300, 520)
    $series += "$($f.T):$a/$b/$c"
    # The text field is there (A far from the desktop) and the box around it is not (B still the desktop).
    if ($f.T -lt $t1 -and [Math]::Abs($a - $a0) -gt 40 -and [Math]::Abs($b - $b0) -lt 15) { $bare++; $bareAt += $f.T }
    if ($f.T -lt $t1 -and [Math]::Abs($b - $b0) -gt 15 -and [Math]::Abs($b - $b1) -gt 15) { $in++ }
    # Fading out: brighter than the desktop there is under the dimmer at this moment, and no
    # longer the box's full colour.
    $desk = $bEnd * $c / $cEnd
    if ($f.T -ge $t2 -and ($b - $desk) -gt 20 -and ($b1 - $b) -gt 15) { $out++ }
  }
  Log "search box frames (ms:field/padding/desktop): $($series -join ' ')"
  $row = "desktop field/padding=$a0/$b0, box=$a1/$b1; frames with the text field alone: $bare ($(($bareAt | Select-Object -First 1)) to $(($bareAt | Select-Object -Last 1)) ms); frames half faded: in $in, out $out"
  Check 'the box and the desktop under it differ enough to tell them apart' ([Math]::Abs($b1 - $b0) -gt 40) $row
  Check 'first search after a start: no frame shows the text field without its box' ($bare -eq 0) $row
  Check 'the box fades out when the search ends (a frame between box and desktop)' ($out -ge 1) $row
  Check 'the second letter makes the list shorter (the box follows it)' (($r2.B - $r2.T) -lt ($r1.B - $r1.T)) "height $($r1.B - $r1.T) -> $($r2.B - $r2.T)"
  foreach ($f in $fr) { $f.B.Dispose() }
  # A second search (everything is warm): the fade in.
  PeekKey; UpWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 900
  RecStart 'S-second' -X $bx -Y $by -W 640 -H 120 -Fps 30
  Start-Sleep -Milliseconds 400
  RecMark 'second search: a letter'
  [G]::Type('b'); Start-Sleep -Milliseconds 900
  RecStop
  $fr = UpFrames 'S-second'
  $b0 = UpLum $fr[0].B.GetPixel(8, 28); $b1 = UpLum $fr[$fr.Count - 1].B.GetPixel(8, 28); $in = 0; $series = @()
  foreach ($f in $fr) { $b = UpLum $f.B.GetPixel(8, 28); $series += "$($f.T):$b"; if ([Math]::Abs($b - $b0) -gt 15 -and [Math]::Abs($b - $b1) -gt 15) { $in++ } }
  foreach ($f in $fr) { $f.B.Dispose() }
  Log "second search frames (ms:padding): $($series -join ' ')"
  Check 'the box fades in (a frame between desktop and box)' ($in -ge 1) "desktop=$b0 box=$b1 frames half faded: $in"
  Check 'the box has the keyboard and the typed letter' ([M]::BoxText() -eq 'b') "text='$([M]::BoxText())'"
  [G]::Press(0x1B); UpWaitPeek $false | Out-Null; Start-Sleep -Milliseconds 700
  Check 'after Esc no search window is left' ([G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value) -eq [IntPtr]::Zero) "window=$([G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value))"
  # With Windows' animations off the box shows and goes at once.
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  PeekKey; UpWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 700
  [G]::Type('ba'); Start-Sleep -Milliseconds 800
  Check 'animations off: the box shows with its text' ((UpBox) -ne [IntPtr]::Zero -and [M]::BoxText() -eq 'ba') "box=$(UpBox) text='$([M]::BoxText())'"
  [G]::Press(0x1B); UpWaitPeek $false | Out-Null; Start-Sleep -Milliseconds 300
  Check 'animations off: no search window is left after Esc' ([G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value) -eq [IntPtr]::Zero) ''
  [Anim]::Set($anim)
  PfQuit
  UpClean
}

function UpPartT() {
  trap { PfTrap $_; continue }
  Case 'title tooltip'
  UpClean
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  $long = '二〇二六年度第一季度销售数据汇总与分析报告'
  Pf fence create --title $long --rect '500,300,220,180' | Out-Null
  Pf fence create --title '短名' --rect '800,300,220,180' | Out-Null
  Start-Sleep -Seconds 2
  $tipOf = { @(ProcTops 'tooltips_class32' 'pecofence' | ForEach-Object { try { [Windows.Automation.AutomationElement]::FromHandle($_).Current.Name } catch { '?' } }) -join ' / ' }
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 300
  [G]::MoveTo(560, 314); Start-Sleep -Milliseconds 1500
  $t = & $tipOf
  Crop 't-cut-title' 480 280 420 220 | Out-Null
  Check 'a title cut with an ellipsis shows all of it in a tooltip' ($t -eq $long) "tip='$t'"
  [G]::MoveTo(860, 314); Start-Sleep -Milliseconds 1500
  $t = & $tipOf
  Check 'a title that fits shows no tooltip' ($t -eq '') "tip='$t'"
  [G]::MoveTo(560, 314); Start-Sleep -Milliseconds 1500
  [G]::MoveTo(560, 600); Start-Sleep -Milliseconds 500
  $t = & $tipOf
  Check 'the tooltip goes when the pointer leaves the title' ($t -eq '') "tip='$t'"
  # The arrow at the right of the title (it shows while the pointer is in the fence).
  $f = UpFence '短名'
  [G]::MoveTo($f.windowRect.x + 100, $f.windowRect.y + 100); Start-Sleep -Milliseconds 400
  [G]::MoveTo($f.windowRect.x + $f.windowRect.w - 20, $f.windowRect.y + 14); Start-Sleep -Milliseconds 1500
  $t = & $tipOf
  Crop 't-arrow' 780 280 300 220 | Out-Null
  Check 'the arrow of an open fence says 收起' ($t -eq '收起') "tip='$t'"
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 400
  Pf fence roll '短名' | Out-Null
  Pf settings set rollUp.clickToExpand true | Out-Null
  Start-Sleep -Milliseconds 1200
  # Not the very point of before: a move back to the same screen point arms no tip.
  [G]::MoveTo($f.windowRect.x + $f.windowRect.w - 17, $f.windowRect.y + 16); Start-Sleep -Milliseconds 1500
  $t = & $tipOf
  Check 'the arrow of a rolled-up fence says 展开' ($t -eq '展开') "tip='$t'"
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 400
  [Anim]::Set($anim)
  PfQuit
  UpClean
}

function UpTips() {
  return @(ProcTops 'tooltips_class32' 'pecofence' | ForEach-Object {
    $n = try { [Windows.Automation.AutomationElement]::FromHandle($_).Current.Name } catch { '?' }
    [pscustomobject]@{ H = $_; Text = $n; R = (Rect $_) }
  })
}

function UpPartR() {
  trap { PfTrap $_; continue }
  Case 'name box: why Enter did not take the name'
  UpClean
  DeskFile '报告.txt' | Out-Null
  DeskFile '会议纪要 11月.txt' | Out-Null
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '资料' --rect '300,200,360,260' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move "$PfDesk\报告.txt" "$PfDesk\会议纪要 11月.txt" --to '资料' | Out-Null
  Start-Sleep -Seconds 2
  $p = ItemPt '资料' '报告'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  if ($Record) { RecStart 'R-hint' -X 0 -Y 0 -W 1600 -H 900 -Fps 10 }
  if ($Record) { RecMark '按 F2，输入带冒号的名字' }
  [G]::Press(0x71); Start-Sleep -Milliseconds 800
  $box = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
  [G]::Type('a:b'); Start-Sleep -Milliseconds 600
  if ($Record) { RecMark '按 Enter' }
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
  $br = Rect $box
  $tip = UpTips | Select-Object -First 1
  $at = if ($tip) { "$($tip.R.L),$($tip.R.T) '$($tip.Text)'" } else { 'none' }
  Shot 'r-hint'
  Check 'the name box stays open after Enter' ([G]::IsWindowVisible($box)) "box=$box $($br.L),$($br.T),$($br.R),$($br.B)"
  Check 'the reason shows under the name box' ($tip -and $tip.Text -match '[\\/:*?]' -and ($tip.R.T - $br.B) -ge 0 -and ($tip.R.T - $br.B) -le 12 -and [Math]::Abs($tip.R.L - $br.L) -le 6) "name box bottom-left $($br.L),$($br.B); tip $at"
  Start-Sleep -Milliseconds 1500
  if ($Record) { RecMark '改掉冒号' }
  [G]::Press(0x08); [G]::Press(0x08); Start-Sleep -Milliseconds 600
  Check 'the reason goes when the text changes' (@(UpTips).Count -eq 0) "tips=$(@(UpTips).Count)"
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1200
  if ($Record) { RecStop }
  Check 'a name without the colon is taken' (Test-Path -LiteralPath (Join-Path $PfDesk 'a.txt')) "a.txt exists: $(Test-Path -LiteralPath (Join-Path $PfDesk 'a.txt'))"
  Remove-Item -LiteralPath (Join-Path $PfDesk 'a.txt') -Force -ErrorAction SilentlyContinue
  [Anim]::Set($anim)
  PfQuit
  UpClean
}

function UpPartM() {
  trap { PfTrap $_; continue }
  Case 'icon menu: an icon on each of its own entries'
  UpClean
  DeskFile '报告.txt' | Out-Null
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  Pf settings set theme light | Out-Null
  ClearFences
  Pf fence create --title '资料' --rect '300,120,360,220' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move "$PfDesk\报告.txt" --to '资料' | Out-Null
  Start-Sleep -Seconds 2
  $m = RClickItem '资料' '报告'
  Start-Sleep -Milliseconds 1200
  $mr = Rect $m
  Crop 'm-item-menu' ($mr.L - 20) ($mr.T - 20) ($mr.R - $mr.L + 40) 260 | Out-Null
  $own = '^(打开|打开文件所在位置|重命名|删除|移动到栅栏|以后把 .* 文件都放到这里|移出栅栏)'
  $rows = @(MenuRows $m | Where-Object { $_.text -match $own })
  $bare = @(); $ink = @()
  foreach ($r in $rows) {
    # The icon column: the first 28 px of the row, against the row's own background.
    $b = [Px]::Grab($r.L, $r.T, 28, $r.B - $r.T)
    $bg = UpLum $b.GetPixel(1, 1); $n = 0
    for ($x = 2; $x -lt 28; $x++) { for ($y = 2; $y -lt $b.Height - 2; $y++) { if ([Math]::Abs((UpLum $b.GetPixel($x, $y)) - $bg) -gt 60) { $n++ } } }
    $b.Dispose()
    $ink += "$($r.text -replace '\t.*', ''):$n"
    if ($n -lt 8) { $bare += $r.text }
  }
  Check 'the icon menu has its seven own entries' ($rows.Count -eq 7) "rows=$($rows.Count): $(($rows | ForEach-Object { $_.text }) -join ' | ')"
  Check 'each of them has an icon' ($rows.Count -gt 0 -and $bare.Count -eq 0) "ink pixels in the icon column: $($ink -join ', ')"
  CloseMenus
  # The menu of a Peek search result (Right arrow on the highlighted row): its three own entries.
  [G]::MoveTo(900, 700); Start-Sleep -Milliseconds 300
  PeekKey; UpWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 800
  [G]::Type('bao'); Start-Sleep -Milliseconds 1000
  [G]::Press(0x27); Start-Sleep -Milliseconds 1500
  $m = MenuWin 4
  $mr = Rect $m
  Crop 'm-search-row-menu' ($mr.L - 20) ($mr.T - 20) ($mr.R - $mr.L + 40) 140 | Out-Null
  $rows = @(MenuRows $m | Where-Object { $_.text -match '^(打开|打开所在文件夹|复制路径)' })
  $bare = @(); $ink = @()
  foreach ($r in $rows) {
    $b = [Px]::Grab($r.L, $r.T, 28, $r.B - $r.T)
    $bg = UpLum $b.GetPixel(1, 1); $n = 0
    for ($x = 2; $x -lt 28; $x++) { for ($y = 2; $y -lt $b.Height - 2; $y++) { if ([Math]::Abs((UpLum $b.GetPixel($x, $y)) - $bg) -gt 60) { $n++ } } }
    $b.Dispose()
    $ink += "$($r.text -replace '\t.*', ''):$n"
    if ($n -lt 8) { $bare += $r.text }
  }
  Check 'the menu of a search result has an icon on its three own entries' ($rows.Count -eq 3 -and $bare.Count -eq 0) "rows=$($rows.Count); ink pixels in the icon column: $($ink -join ', ')"
  CloseMenus
  [G]::Press(0x1B); UpWaitPeek $false | Out-Null; Start-Sleep -Milliseconds 400
  PfQuit
  UpClean
}

function UpPartC() {
  trap { PfTrap $_; continue }
  Case 'rule offer card: the next card takes its place'
  UpClean
  $names = @('会议纪要 11月', '报告', '合同 草稿')
  foreach ($n in $names) { DeskFile "$n.txt" | Out-Null }
  $anim = [Anim]::Get(); [Anim]::Set($true)
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  [G]::MoveTo(700, 450)
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
  UpWaitNotice 30 | Out-Null
  Start-Sleep -Seconds 3
  Pf settings set language zh-CN | Out-Null
  Start-Sleep -Seconds 2
  # The first-start tip goes (its close mark), then three .txt files are dragged into a new fence.
  $n = UpNotice; $r = Rect $n
  [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 1200
  Pf fence create --title '文档' --rect '500,200,360,240' | Out-Null
  Start-Sleep -Seconds 2
  $g = UpFence '文档'
  foreach ($name in $names) {
    $p = ItemPt (ItemFence $name) $name
    DragTo $p[0] $p[1] ($g.windowRect.x + 180) ($g.windowRect.y + 150)
    Start-Sleep -Milliseconds 1200
  }
  $n = UpWaitNotice 8
  Start-Sleep -Milliseconds 800
  Check 'three files of one kind dragged into a fence bring up the rule offer' ($n -ne [IntPtr]::Zero -and [L]::Text($n) -match '让新文件自动进来') "title='$([L]::Text($n))'"
  $r = Rect $n
  if ($Record) { RecStart 'C-rule-card' -X 1180 -Y 600 -W 420 -H 260 -Fps 30 }
  # The pointer rests on 添加规则; a patch of the card's plate near its bottom-left corner is
  # read as fast as the screen allows while the button is clicked.
  [G]::MoveTo($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 600
  if ($Record) { RecMark '点“添加规则”' }
  [Px]::Burst($r.L + 10, $r.B - 14, 6, 6, 800, 1)
  $first = [Px]::Frames[0].GetPixel(3, 3)
  $gone = 0; $goneAt = @()
  for ($i = 1; $i -lt [Px]::Frames.Count; $i++) {
    if ((UpFar ([Px]::Frames[$i].GetPixel(3, 3)) $first) -gt 10) { $gone++; $goneAt += [Px]::Times[$i] }
  }
  Start-Sleep -Milliseconds 700
  if ($Record) { RecStop }
  $n1 = UpNotice
  $row = "window $n -> $n1, frames $([Px]::Frames.Count), plate changed in $gone ($(($goneAt | Select-Object -First 1)) to $(($goneAt | Select-Object -Last 1)) ms)"
  Shot 'c-rule-added'
  Check 'the card for the added rule shows' ($n1 -ne [IntPtr]::Zero -and [L]::Text($n1) -match '已添加整理规则') "title='$([L]::Text($n1))'"
  Check 'it is the card that was there (no second card)' ($n1 -eq $n) $row
  Check 'the card''s plate never leaves the screen between the two' ($gone -eq 0) $row
  Check 'the rule was added' (@(Rules | Where-Object { "$($_ | ConvertTo-Json -Depth 6 -Compress)" -match '\.txt' }).Count -ge 1) "rules=$(@(Rules).Count)"
  # The new card works: x closes it.
  $r = Rect $n1
  [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Milliseconds 1000
  Check 'x closes the new card' ((UpNotice) -eq [IntPtr]::Zero) "notice=$(UpNotice)"
  [Anim]::Set($anim)
  PfQuit
  UpClean
  foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk "$n.txt") -Force -ErrorAction SilentlyContinue }
}

function UpPartK() {
  trap { PfTrap $_; continue }
  Case 'menu key: the menu keeps clear of the icon'
  UpClean
  DeskFile '报告.txt' | Out-Null
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  # In the lower half of the screen: the menu (about 500 px tall) has no room under the name.
  Pf fence create --title '资料' --rect '200,360,360,260' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move "$PfDesk\报告.txt" --to '资料' | Out-Null
  Start-Sleep -Seconds 2
  $p = ItemPt '资料' '报告'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  [G]::Press(0x5D); Start-Sleep -Milliseconds 1500
  $m = MenuWin 4; $mr = Rect $m
  Crop 'k-menu-key' 150 300 700 560 | Out-Null
  $covers = ($mr.L -le $p[0] -and $p[0] -lt $mr.R -and $mr.T -le $p[1] -and $p[1] -lt $mr.B)
  $row = "menu $($mr.L),$($mr.T),$($mr.R),$($mr.B); icon centre $($p -join ',')"
  Check 'the Menu key opens the icon menu' ($m -ne [IntPtr]::Zero -and (MenuTexts $m) -match '打开文件所在位置') $row
  Check 'the menu does not lie over the icon it is for' (-not $covers) $row
  CloseMenus; Start-Sleep -Milliseconds 400
  # With room under the name the menu opens there, as before.
  Pf fence move '资料' --x 200 --y 40 | Out-Null
  Start-Sleep -Milliseconds 1200
  $p = ItemPt '资料' '报告'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  [G]::Press(0x5D); Start-Sleep -Milliseconds 1500
  $m = MenuWin 4; $mr = Rect $m
  Check 'with room under the name the menu opens under it' ($m -ne [IntPtr]::Zero -and ($mr.T - $p[1]) -ge 20 -and ($mr.T - $p[1]) -le 90 -and [Math]::Abs($mr.L - $p[0]) -le 60) "menu $($mr.L),$($mr.T),$($mr.R),$($mr.B); icon centre $($p -join ',')"
  CloseMenus
  # A right-click still opens the menu at the pointer.
  [G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 1500
  $m = MenuWin 4; $mr = Rect $m
  Check 'a right-click opens the menu at the pointer' ($m -ne [IntPtr]::Zero -and [Math]::Abs($mr.L - $p[0]) -le 3 -and [Math]::Abs($mr.T - $p[1]) -le 3) "menu $($mr.L),$($mr.T); pointer $($p -join ',')"
  CloseMenus
  PfQuit
  UpClean
}

if ($Part -match 'N') { UpPartN }
if ($Part -match 'S') { UpPartS }
if ($Part -match 'T') { UpPartT }
if ($Part -match 'R') { UpPartR }
if ($Part -match 'M') { UpPartM }
if ($Part -match 'C') { UpPartC }
if ($Part -match 'K') { UpPartK }
