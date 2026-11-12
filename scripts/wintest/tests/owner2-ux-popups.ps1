# Owner round 2026-11-12, second batch, package ux-popups: what pops up over the desktop.
# Runs on a 1600x900 box at 100 %.
#   run.sh --var Part=N ...   the undo card: a second delete puts its own name on the card and
#                             its 撤销 brings that fence back; a click on 撤销 fades the card out
#                             (it vanished in one frame)
# Without --var every part runs. Record=1 also records the clips for the report.
if (-not $Part) { $Part = 'N' }
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

if ($Part -match 'N') { UpPartN }
