# Fence polish 2026-11-12, package scroll. Parts (run.sh --var Part=...), default W:
#   W  the wheel: a wheel kept turning scrolls at an even speed (no kick at every notch), one
#      notch, a turn the other way, PageDown / End / Home and the scrollbar track as before,
#      the content rests on whole pixels
#   P  work per animation frame with 60 and with 6000 files (numbers into the job log)
#   F  the fills of the icon under the pointer and of selected icons show, go and are there
#      for icons that scroll into view (the per-frame sync touches only the icons that are on)
#   N  a Peek search scrolls a fence to its match; when it ends the fence goes back to where
#      it was, unless the user scrolled it during the search
#   R  recordings of the wheel for a before/after animation (no checks)
# The scroll offset is read from the app itself: the test script logs a `dump` on every timer
# tick (`pace 10`), each with the fence's `scroll=`; a frame is 16.7 ms, a tick 10 to 16 ms.
# `pace` is a debug build's line: part W needs `run.sh --bin target/wintest-bin-debug`.
if (-not $Part) { $Part = 'W' }
trap { PfTrap $_; continue }

Add-Type -Namespace PfPolish -Name Clock -MemberDefinition '[DllImport("kernel32.dll")] public static extern void GetSystemTimePreciseAsFileTime(out long ft);'
# Now, in ms (UTC file time), on the scale of the app log's timestamps.
function PsNow() { $ft = [long]0; [PfPolish.Clock]::GetSystemTimePreciseAsFileTime([ref]$ft); return $ft / 10000.0 }
function PsClean() { Get-ChildItem $PfDesk -Filter 'ps*.txt' -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue }
function PsFiles($n) {
  $words = '季度报告', '年度预算', '工作计划', '会议记录', '新年活动安排与分工', '文件整理', '图片素材', '报销单', '发票', '合同', '项目进度', '说明'
  for ($i = 1; $i -le $n; $i++) { Set-Content -LiteralPath (Join-Path $PfDesk ("ps{0:d4} {1}.txt" -f $i, $words[$i % 12])) 'x' }
}
# Test-script lines: a dump on every timer tick from $first ms on, for about $seconds.
function PsDumpScript($first, $seconds) {
  $n = [int]($seconds * 1000 / 14)
  return @('pace 10', "sleep $first") + (1..$n | ForEach-Object { 'dump s'; 'sleep 1' }) + @('sleep 3600000')
}
# The app's log as one string (the app keeps the file open).
function PsLogText() {
  $fs = [IO.File]::Open((PfLogPath), 'Open', 'Read', 'ReadWrite')
  try { return (New-Object IO.StreamReader($fs, [Text.Encoding]::UTF8)).ReadToEnd() } finally { $fs.Close() }
}
function PsLogMs($stamp) { return [DateTimeOffset]::Parse($stamp).UtcDateTime.ToFileTimeUtc() / 10000.0 }
# The fence's scroll offset over time from the dumps logged since $from: @(@(ms, offset), ...).
function PsSamples($title, $from) {
  $out = New-Object System.Collections.ArrayList
  $re = '(?m)^(\S+Z)\s[^\n]*\[s\] live [^\n]*title="' + [regex]::Escape($title) + '"[^\n]* scroll=(-?[\d.]+)'
  foreach ($m in [regex]::Matches((PsLogText), $re)) {
    $ms = PsLogMs $m.Groups[1].Value
    if ($ms -ge $from) { [void]$out.Add(@($ms, [double]$m.Groups[2].Value)) }
  }
  return , $out.ToArray()
}
# The offset drawn at time $t: the last sample at or before it.
function PsAt($samples, $t) {
  $v = $null
  foreach ($s in $samples) { if ($s[0] -le $t) { $v = $s[1] } else { break } }
  return $v
}
function PsEnd($samples) { return (PsAt $samples ((PsNow))) }
# Turns the wheel: $count notches $gap ms apart ($dir -1 scrolls down); the time of each notch.
function PsTurn($count, $gap, $dir = -1) {
  $times = @()
  for ($i = 0; $i -lt $count; $i++) {
    [CW]::Wheel($dir); $times += PsNow
    if ($i -lt $count - 1) { Start-Sleep -Milliseconds $gap }
  }
  return , $times
}
# Back to the top and at rest; the time after which samples belong to the next step.
function PsTop() {
  [CW]::Wheel(40); Start-Sleep -Milliseconds 700
  return (PsNow)
}
# For the notches after the first: the share of the distance still to go at the notch that the
# content covers in the next $ms. The decelerate curve started again covers more than half
# of it in 50 ms (a kick); a glide that carries its speed on covers about a third.
function PsKicks($samples, $times, $row, $ms = 50) {
  $shares = @()
  for ($k = 1; $k -lt $times.Count; $k++) {
    $at = PsAt $samples $times[$k]; $later = PsAt $samples ($times[$k] + $ms)
    $left = ($k + 1) * $row - $at
    if ($null -ne $at -and $null -ne $later -and $left -gt 1) { $shares += [math]::Abs($later - $at) / $left }
  }
  return , $shares
}
function PsMean($a) { if (@($a).Count -eq 0) { return -1 }; return [math]::Round((@($a) | Measure-Object -Average).Average, 3) }
function PsFmt($a) { return (@($a) | ForEach-Object { '{0:n2}' -f $_ }) -join ' ' }
function PsPeak($samples) { return (@($samples | ForEach-Object { $_[1] }) | Measure-Object -Maximum).Maximum }
# Largest step back between two samples (0 when the offset only ever grew).
function PsBack($samples) {
  $worst = 0.0
  for ($i = 1; $i -lt $samples.Count; $i++) { $d = $samples[$i - 1][1] - $samples[$i][1]; if ($d -gt $worst) { $worst = $d } }
  return $worst
}
# A fence of $n text files at a fixed place, sorted by name; its window rect.
function PsFence($n) {
  PsFiles $n
  ClearFences | Out-Null
  Pf fence create --title $PsTitle --rect '200,120,360,330' | Out-Null
  Pf item move --glob 'ps*.txt' --to $PsTitle | Out-Null
  Pf fence set $PsTitle sort name | Out-Null
  Start-Sleep -Seconds 2
  return (FenceGet $PsTitle).windowRect
}

function PartW() {
  trap { PfTrap $_; continue }
  Check 'W a debug build runs (the test script needs its pace line)' $PfDebugBuild "debug=$PfDebugBuild"
  $env:PECOFENCE_UI_TEST_WINDOWS = '1'
  $t0 = PsNow
  PfStart -Script (PsDumpScript 14000 75) -Lang 'zh-CN'
  $r = PsFence 60
  $cx = $r.x + 140; $cy = $r.y + 180
  [G]::MoveTo($cx, $cy)
  $wait = 17000 - ((PsNow) - $t0); if ($wait -gt 0) { Start-Sleep -Milliseconds $wait }
  Log "fence $($r.x),$($r.y),$($r.w),$($r.h); dumps run from about 15 s to 90 s after the start; now $([int]((PsNow) - $t0)) ms"

  Case 'W1 one notch from rest'
  $from = PsTop
  $t = PsTurn 1 0
  Start-Sleep -Milliseconds 700
  $s = PsSamples $PsTitle $from
  $row = PsEnd $s
  $first = (PsAt $s ($t[0] + 50)) / [math]::Max(1, $row)
  Log "W1 row=$row share after 50 ms=$('{0:n2}' -f $first) samples=$($s.Count)"
  Check 'W1 one notch scrolls one row of icons' ($row -ge 60 -and $row -le 140) "offset $row"
  Check 'W1 one notch answers at once, as before (over 30 % of the row within 50 ms)' ($first -ge 0.3) "share $('{0:n2}' -f $first)"
  Check 'W1 at rest on the row 320 ms after the notch' ((PsAt $s ($t[0] + 320)) -eq $row) "offset at 320 ms $(PsAt $s ($t[0] + 320))"

  Case 'W2 three notches 70 ms apart'
  $from = PsTop
  $t = PsTurn 3 60
  Start-Sleep -Milliseconds 800
  $s = PsSamples $PsTitle $from
  $k = PsKicks $s $t $row
  Log "W2 gaps $([int]($t[1] - $t[0])) $([int]($t[2] - $t[1])) ms; shares $(PsFmt $k); end $(PsEnd $s)"
  Check 'W2 no kick at the second and third notch (mean share within 50 ms under 0.42)' ((PsMean $k) -ge 0 -and (PsMean $k) -lt 0.42) "shares $(PsFmt $k), mean $(PsMean $k)"
  Check 'W2 ends three rows down' ((PsEnd $s) -eq 3 * $row) "offset $(PsEnd $s)"
  Check 'W2 never a step back' ((PsBack $s) -eq 0) "largest step back $(PsBack $s)"

  Case 'W3 eight notches 70 ms apart'
  $from = PsTop
  $t = PsTurn 8 60
  Start-Sleep -Milliseconds 800
  $s = PsSamples $PsTitle $from
  $k = PsKicks $s $t $row
  $end = PsEnd $s
  Log "W3 took $([int]($t[7] - $t[0])) ms; shares $(PsFmt $k); end $end"
  Check 'W3 a wheel kept turning has no kick per notch (mean share within 50 ms under 0.42)' ((PsMean $k) -ge 0 -and (PsMean $k) -lt 0.42) "shares $(PsFmt $k), mean $(PsMean $k)"
  Check 'W3 ends eight rows down' ($end -eq 8 * $row) "offset $end"
  Check 'W3 never a step back and never past the end' ((PsBack $s) -eq 0 -and (PsPeak $s) -le $end) "largest step back $(PsBack $s), peak $(PsPeak $s)"

  Case 'W4 ten notches 35 ms apart'
  $from = PsTop
  $t = PsTurn 10 30
  Start-Sleep -Milliseconds 800
  $s = PsSamples $PsTitle $from
  $end = PsEnd $s
  Log "W4 took $([int]($t[9] - $t[0])) ms for ten notches; end $end; at the last notch $(PsAt $s $t[9]); 320 ms later $(PsAt $s ($t[9] + 320))"
  Check 'W4 ends ten rows down' ($end -eq 10 * $row) "offset $end"
  Check 'W4 never a step back' ((PsBack $s) -eq 0) "largest step back $(PsBack $s)"
  Check 'W4 at rest 320 ms after the last notch, as before' ((PsAt $s ($t[9] + 320)) -eq $end) "offset then $(PsAt $s ($t[9] + 320))"

  Case 'W5 three notches down, then two up'
  $from = PsTop
  PsTurn 3 60 | Out-Null
  Start-Sleep -Milliseconds 90
  $u = PsTurn 2 60 1
  Start-Sleep -Milliseconds 800
  $s = PsSamples $PsTitle $from
  $end = PsEnd $s
  # The first notch up takes a row off the way still to go (the content may still be short of
  # the new end); after the second one the end lies behind the content, which turns at once.
  $turn = (PsAt $s $u[1]) - (PsAt $s ($u[1] + 60))
  Log "W5 peak $(PsPeak $s) end $end; at the notches up $(PsAt $s $u[0]) $(PsAt $s $u[1]); moved back $turn within 60 ms of the second"
  Check 'W5 ends one row down' ($end -eq $row) "offset $end"
  Check 'W5 never past three rows' ((PsPeak $s) -le 3 * $row) "peak $(PsPeak $s)"
  Check 'W5 a notch the other way turns the content back at once' ($turn -gt 0.15 * $row) "moved back $turn of a $row row"

  Case 'W6 keyboard paging and the scrollbar track, as before'
  $from = PsTop
  $p = CellPt $PsTitle 1
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
  $keys0 = PsNow
  [G]::Press(0x22); Start-Sleep -Milliseconds 80; [G]::Press(0x22); $second = PsNow
  Start-Sleep -Milliseconds 900
  $s = PsSamples $PsTitle $from
  $page2 = PsEnd $s
  $at = PsAt $s $second; $share = ((PsAt $s ($second + 50)) - $at) / [math]::Max(1, $page2 - $at)
  Log "W6 two PageDown: end $page2; at the second press $at; share within 50 ms of it $('{0:n2}' -f $share)"
  Check 'W6 PageDown twice ends more than three rows down' ($page2 -gt 3 * $row) "offset $page2"
  [G]::Press(0x23); Start-Sleep -Milliseconds 900
  $endAt = PsEnd (PsSamples $PsTitle $from)
  [G]::Press(0x24); Start-Sleep -Milliseconds 900
  $homeAt = PsEnd (PsSamples $PsTitle $from)
  Check 'W6 End goes to the last row, Home to the top' ($endAt -gt $page2 -and $homeAt -eq 0) "End $endAt, Home $homeAt"
  # One press on the track below the thumb: a page.
  [G]::MoveTo($r.x + $r.w - 5, $r.y + $r.h - 40); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 60; MUp; Start-Sleep -Milliseconds 900
  $paged = PsEnd (PsSamples $PsTitle $from)
  Check 'W6 a press on the scrollbar track pages down' ($paged -gt 2 * $row) "offset $paged"
  $keys1 = PsNow
  [G]::MoveTo($cx, $cy); Start-Sleep -Milliseconds 300
  # Keys and the scrollbar never take the wheel's path (the line exists only in builds that carry the speed on).
  $wheelLines = @([regex]::Matches((PsLogText), '(?m)^(\S+Z)\s[^\n]*wheel notch on a running glide[^\n]*') | ForEach-Object { , @((PsLogMs $_.Groups[1].Value), $_.Value) })
  $inKeys = @($wheelLines | Where-Object { $_[0] -ge $keys0 -and $_[0] -le $keys1 })
  Check 'W6 keys and the scrollbar keep the decelerate curve (no wheel carry in the log)' ($inKeys.Count -eq 0) "lines $($inKeys.Count)"

  Case 'W7 every rest is on a whole pixel'
  $rests = @($row, $page2, $endAt, $paged)
  Check 'W7 whole pixels' (@($rests | Where-Object { $_ -ne [math]::Round($_) }).Count -eq 0) "rests $($rests -join ' ')"
  Log "W7 notches on a running glide in the log: $($wheelLines.Count), of them carried: $(@($wheelLines | Where-Object { $_[1] -match 'carried=[1-9]' }).Count)"
  PfQuit
}

# The app's "animation run" lines logged between $from and $to (ms): frames, the slowest
# frame's work (ms) and, in builds that log it, the work of all frames together (ms).
function PsRuns($from, $to) {
  $runs = @()
  foreach ($m in [regex]::Matches((PsLogText), '(?m)^(\S+Z)\s[^\n]*animation run(?: stuttered)?[^\n]* frames=(\d+) [^\n]*max_frame_ms=([\d.]+)(?:[^\n]*work_ms=([\d.]+))?')) {
    $ms = PsLogMs $m.Groups[1].Value
    if ($ms -lt $from -or $ms -gt $to -or [int]$m.Groups[2].Value -lt 3) { continue }
    $work = if ($m.Groups[4].Success) { [double]$m.Groups[4].Value } else { -1 }
    $runs += , @([int]$m.Groups[2].Value, [double]$m.Groups[3].Value, $work)
  }
  return , $runs
}
function PsMedian($a) { $v = @($a | Sort-Object); if ($v.Count -eq 0) { return -1 }; return $v[[int][math]::Floor(($v.Count - 1) / 2)] }
# One phase of pointer or wheel work on a fence: the median of the slowest frame per run, and
# the mean work per frame over all runs (-1 when the build does not log the sum).
function PsPhase($name, [scriptblock]$body) {
  $from = PsNow
  & $body
  Start-Sleep -Milliseconds 700
  $runs = PsRuns $from (PsNow)
  $frames = (@($runs | ForEach-Object { $_[0] }) | Measure-Object -Sum).Sum
  $sum = (@($runs | Where-Object { $_[2] -ge 0 } | ForEach-Object { $_[2] }) | Measure-Object -Sum).Sum
  $mean = if ($runs.Count -gt 0 -and $runs[0][2] -ge 0 -and $frames -gt 0) { [math]::Round($sum / $frames, 3) } else { -1 }
  $slowest = PsMedian @($runs | ForEach-Object { $_[1] })
  Log "P $name runs=$($runs.Count) frames=$frames slowest frame per run (median)=$slowest ms, mean work per frame=$mean ms"
  return @{ runs = $runs.Count; slowest = $slowest; mean = $mean }
}
function PsFolder($name, $n) {
  $d = Join-Path "$PfRun\psdata" $name
  New-Item -ItemType Directory -Force $d | Out-Null
  $kinds = @('会议记录', '预算表', '设计稿 第二版', '合同扫描件', '周报', '项目计划书 最终版')
  for ($i = 1; $i -le $n; $i++) { [IO.File]::WriteAllText((Join-Path $d ("项目文档 {0:D4} {1}.txt" -f $i, $kinds[$i % $kinds.Count])), "ps $i") }
  return $d
}
# The pointer goes between two icons of the first row 14 times: a hover fade each.
function PsHover($x, $y) { for ($k = 0; $k -lt 14; $k++) { [G]::MoveTo($x + 40 + 84 * ($k % 2), $y + 36 + 40); Start-Sleep -Milliseconds 350 } }
# One notch down, then up, 8 times: a glide each.
function PsWheel($x, $y) {
  [G]::MoveTo($x + 250, $y + 200); Start-Sleep -Milliseconds 300
  for ($k = 0; $k -lt 8; $k++) { [CW]::Wheel(-1); Start-Sleep -Milliseconds 500; [CW]::Wheel(1); Start-Sleep -Milliseconds 500 }
}

function PartP() {
  trap { PfTrap $_; continue }
  Case 'P work per frame with 6000 files and with 60'
  $big = PsFolder 'big' 6000
  $small = PsFolder 'small' 60
  PfStart
  Start-Sleep -Seconds 4
  ClearFences | Out-Null
  Pf fence create --portal $big --rect '80,120,504,420' | Out-Null
  Pf fence create --portal $small --rect '700,120,504,420' | Out-Null
  Start-Sleep -Seconds 12
  ParkMouse; Start-Sleep -Milliseconds 800
  $hb = PsPhase 'hover, 6000 files' { PsHover 80 120 }
  $hs = PsPhase 'hover, 60 files' { PsHover 700 120 }
  $wb = PsPhase 'wheel, 6000 files' { PsWheel 80 120 }
  $ws = PsPhase 'wheel, 60 files' { PsWheel 700 120 }
  $hb2 = PsPhase 'hover again, 6000 files' { PsHover 80 120 }
  $hs2 = PsPhase 'hover again, 60 files' { PsHover 700 120 }
  ParkMouse
  Check 'P every phase had animation runs' (@($hb, $hs, $wb, $ws, $hb2, $hs2 | Where-Object { $_.runs -lt 5 }).Count -eq 0) "runs $($hb.runs) $($hs.runs) $($wb.runs) $($ws.runs) $($hb2.runs) $($hs2.runs)"
  Check 'P the animation run lines carry the work of all frames (work_ms), so the mean per frame can be read' ($wb.mean -ge 0 -and $ws.mean -ge 0) "mean work per frame: wheel $($wb.mean) / $($ws.mean) ms, hover $($hb.mean) / $($hs.mean) ms (6000 / 60 files)"
  $hover = [math]::Max($hb.slowest, $hb2.slowest) - [math]::Min($hs.slowest, $hs2.slowest)
  $wheel = $wb.slowest - $ws.slowest
  Check 'P a hover fade costs under 0.3 ms more per frame with 6000 files than with 60' ($hover -lt 0.3) "slowest frame per run: $($hb.slowest) and $($hb2.slowest) ms against $($hs.slowest) and $($hs2.slowest) ms; mean work per frame $($hb.mean) against $($hs.mean) ms"
  Check 'P a wheel glide costs under 0.3 ms more per frame with 6000 files than with 60' ($wheel -lt 0.3) "slowest frame per run: $($wb.slowest) ms against $($ws.slowest) ms; mean work per frame $($wb.mean) against $($ws.mean) ms"
  PfQuit
}

# Mean colour of the i-th cell of the fence as it is on screen now (its icon and name; the
# pointer must be elsewhere): "r g b".
function PsCell($i) {
  $p = CellPt $PsTitle $i
  $b = [Px]::Grab($p[0] - 36, $p[1] - 30, 72, 76)
  $m = (MeanRgb $b) -join ' '; $b.Dispose()
  return $m
}
# How far two "r g b" colours are apart (largest channel difference).
function PsFar($a, $b) {
  $x = @($a -split ' ' | ForEach-Object { [double]$_ }); $y = @($b -split ' ' | ForEach-Object { [double]$_ })
  return [math]::Round((@(0, 1, 2 | ForEach-Object { [math]::Abs($x[$_] - $y[$_]) }) | Measure-Object -Maximum).Maximum, 1)
}

# The fills an icon gets (pointer over it, selected) follow the same rules whether the fence
# syncs every icon per frame or only those that are on: they show, they go, and an icon that
# scrolls into view already has the state it should have.
function PartF() {
  trap { PfTrap $_; continue }
  PfStart -Lang 'zh-CN'
  $r = PsFence 60
  $park = @(($r.x + $r.w + 60), ($r.y + 200))
  [G]::MoveTo($park[0], $park[1]); Start-Sleep -Milliseconds 600
  $plain = @(0..7 | ForEach-Object { PsCell $_ })

  Case 'F1 the pointer over an icon'
  $p1 = CellPt $PsTitle 1
  [G]::MoveTo($p1[0], $p1[1]); Start-Sleep -Milliseconds 250
  $over = PsCell 1; $other = PsCell 2
  [G]::MoveTo($park[0], $park[1]); Start-Sleep -Milliseconds 500
  $left = PsCell 1
  Check 'F1 the icon under the pointer gets its fill, its neighbour none' ((PsFar $over $plain[1]) -gt 3 -and (PsFar $other $plain[2]) -lt 1) "under the pointer $(PsFar $over $plain[1]), neighbour $(PsFar $other $plain[2])"
  Check 'F1 the fill goes when the pointer leaves' ((PsFar $left $plain[1]) -lt 1) "left over $(PsFar $left $plain[1])"

  Case 'F2 selecting one icon, then another'
  [G]::Click($p1[0], $p1[1]); [G]::MoveTo($park[0], $park[1]); Start-Sleep -Milliseconds 500
  $sel1 = PsCell 1
  $p5 = CellPt $PsTitle 5
  [G]::Click($p5[0], $p5[1]); [G]::MoveTo($park[0], $park[1]); Start-Sleep -Milliseconds 500
  $was1 = PsCell 1; $sel5 = PsCell 5
  Check 'F2 the selected icon gets its fill' ((PsFar $sel1 $plain[1]) -gt 6) "selected $(PsFar $sel1 $plain[1])"
  Check 'F2 selecting another icon moves the fill there' ((PsFar $was1 $plain[1]) -lt 1 -and (PsFar $sel5 $plain[5]) -gt 6) "first icon $(PsFar $was1 $plain[1]), second $(PsFar $sel5 $plain[5])"

  Case 'F3 everything selected, then scrolled'
  [G]::KeyDown(0x11); [G]::Press(0x41); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 500
  $all = @(0..7 | ForEach-Object { PsFar (PsCell $_) $plain[$_] })
  [G]::MoveTo($p1[0], $p1[1]); Start-Sleep -Milliseconds 100
  [CW]::Wheel(-4); [G]::MoveTo($park[0], $park[1]); Start-Sleep -Milliseconds 120
  # Rows 4 and 5 of the list now stand where rows 0 and 1 stood: seen 120 ms into the glide and
  # at rest, a selected icon that scrolls in is selected from its first frame, not fading in.
  $mid = [Px]::Grab($r.x + 8, $r.y + 44, $r.w - 24, $r.h - 52)
  Start-Sleep -Milliseconds 700
  $rest = @(0..7 | ForEach-Object { PsFar (PsCell $_) $plain[$_] })
  $blue = (MeanRgb $mid) -join ' '; $mid.Dispose()
  $restArea = [Px]::Grab($r.x + 8, $r.y + 44, $r.w - 24, $r.h - 52); $restBlue = (MeanRgb $restArea) -join ' '; $restArea.Dispose()
  Check 'F3 Ctrl+A fills every icon in view' (@($all | Where-Object { $_ -le 6 }).Count -eq 0) "change per icon $($all -join ' ')"
  Check 'F3 the icons that scrolled in are selected' (@($rest | Where-Object { $_ -le 4 }).Count -eq 0) "change per place $($rest -join ' ')"
  Check 'F3 they are selected while they scroll in, not fading in' ((PsFar $blue $restBlue) -lt 4) "the icon area 120 ms into the glide $blue, at rest $restBlue"

  Case 'F4 the selection dropped, scrolled back'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  [G]::MoveTo($p1[0], $p1[1]); Start-Sleep -Milliseconds 100
  [CW]::Wheel(40); [G]::MoveTo($park[0], $park[1]); Start-Sleep -Milliseconds 900
  $back = @(0..7 | ForEach-Object { PsFar (PsCell $_) $plain[$_] })
  # The icon clicked last keeps the keyboard's focus ring after Esc (a change of about 5), no fill (over 20).
  $fills = @(0..7 | Where-Object { $back[$_] -ge $(if ($_ -eq 5) { 10 } else { 1 }) })
  Check 'F4 no fill is left on the icons that were out of view' ($fills.Count -eq 0) "change per icon $($back -join ' ')"
  PfQuit
}

# A Peek search for $text (the desktop in front first), and the end of it.
function PsSearch($text) {
  DeskClick | Out-Null
  PeekKey; Start-Sleep -Milliseconds 800
  [G]::Type($text); Start-Sleep -Milliseconds 1200
}
function PsEndSearch() {
  for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }
  Start-Sleep -Milliseconds 800
}

# The fence window of $title as the dumps name it ("0x..."), and what its latest dump says:
# rolled, how many items the tab on show has, the scroll offset.
function PsHwnd($title) {
  $m = [regex]::Matches((PsLogText), '(?m)^\S+Z\s[^\n]*\[s\] live hwnd=(0x[0-9a-f]+) [^\n]*title="' + [regex]::Escape($title) + '"')
  if ($m.Count -eq 0) { return $null }
  return $m[$m.Count - 1].Groups[1].Value
}
function PsState($hwnd) {
  $m = [regex]::Matches((PsLogText), '(?m)^\S+Z\s[^\n]*\[s\] live hwnd=' + $hwnd + ' [^\n]* rolled=(\w+) [^\n]* items=(\d+) [^\n]* scroll=(-?[\d.]+)')
  if ($m.Count -eq 0) { return @{ rolled = $null; items = -1; scroll = -1 } }
  $g = $m[$m.Count - 1].Groups
  return @{ rolled = ($g[1].Value -eq 'true'); items = [int]$g[2].Value; scroll = [double]$g[3].Value }
}

# A Peek search scrolls a fence to its match; when the search ends the fence goes back to where
# it was (as the tabs it switched and the fences it expanded do), unless the user scrolled it.
# That holds for a fence the search opened and let roll back up, and for every tab it scrolled.
function PartN() {
  trap { PfTrap $_; continue }
  PfStart -Script (PsDumpScript 9000 120) -Lang 'zh-CN'
  $r = PsFence 60
  $cx = $r.x + 140; $cy = $r.y + 180
  [G]::MoveTo($cx, $cy); Start-Sleep -Milliseconds 300
  [CW]::Wheel(-1); Start-Sleep -Milliseconds 3000
  $from = PsNow
  Start-Sleep -Milliseconds 300
  $row = PsEnd (PsSamples $PsTitle $from)
  Check 'N the fence rests one row down before the search' ($row -ge 60 -and $row -le 140) "offset $row"

  Case 'N1 the search ends: back to where the fence was'
  PsSearch 'ps0058'
  $box = [M]::BoxText()
  $found = PsEnd (PsSamples $PsTitle $from)
  PsEndSearch
  $after = PsEnd (PsSamples $PsTitle $from)
  Log "N1 box='$box' offset during the search $found, after it $after"
  Check 'N1 the search scrolled the fence to its match' ($box -eq 'ps0058' -and $found -gt 5 * $row) "box='$box' offset $found"
  Check 'N1 after the search the fence is back where it was' ($after -eq $row) "offset $after, before the search $row"

  Case 'N2 the user scrolled during the search: that place stays'
  PsSearch 'ps0058'
  [G]::MoveTo($cx, $cy); Start-Sleep -Milliseconds 200
  [CW]::Wheel(2); Start-Sleep -Milliseconds 700
  $moved = PsEnd (PsSamples $PsTitle $from)
  PsEndSearch
  $kept = PsEnd (PsSamples $PsTitle $from)
  Log "N2 offset after two notches up during the search $moved, after the search $kept"
  Check 'N2 the wheel moved the fence during the search' ($moved -eq $found - 2 * $row) "offset $moved"
  Check 'N2 the fence keeps the place the user scrolled to' ($kept -eq $moved) "offset $kept"

  Case 'N3 the next search goes back to the place the user chose'
  PsSearch 'ps0046'
  $third = PsEnd (PsSamples $PsTitle $from)
  PsEndSearch
  $still = PsEnd (PsSamples $PsTitle $from)
  Log "N3 offset during the third search $third, after it $still"
  Check 'N3 the search scrolled the fence again' ($third -ne $kept) "offset $third"
  Check 'N3 after it the fence is back at the place the user scrolled to' ($still -eq $kept) "offset $still, before the search $kept"

  Case 'N4 a rolled fence the search opened, scrolled and let roll back up'
  $hw = PsHwnd $PsTitle
  ParkMouse | Out-Null; Start-Sleep -Milliseconds 400
  $base = (PsState $hw).scroll
  Pf fence roll $PsTitle | Out-Null; Start-Sleep -Milliseconds 900
  $shut0 = PsState $hw
  PsSearch 'ps0001'
  $open = PsState $hw
  # One more letter leaves the fence without a match (no name holds a q; an x would match the
  # x of .txt): it rolls back up while the search goes on.
  [G]::Type('q'); Start-Sleep -Milliseconds 1700
  $shut = PsState $hw
  PsEndSearch
  $after = PsState $hw
  Pf fence unroll $PsTitle | Out-Null; Start-Sleep -Milliseconds 900
  $opened = PsState $hw
  Log "N4 before $base; rolled $($shut0.rolled); the search opened it (rolled $($open.rolled)) at $($open.scroll); without a match rolled $($shut.rolled) at $($shut.scroll); after the search rolled $($after.rolled) at $($after.scroll); opened again at $($opened.scroll)"
  Check 'N4 the search opened the rolled fence and scrolled it to its match' ($shut0.rolled -eq $true -and $open.rolled -eq $false -and $base -gt 0 -and $open.scroll -ne $base) "before $base, during the search $($open.scroll), rolled $($shut0.rolled) then $($open.rolled)"
  Check 'N4 left without a match, the fence rolled back up during the search' ($shut.rolled -eq $true) "rolled $($shut.rolled)"
  Check 'N4 after the search the rolled fence has its place from before the search' ($after.rolled -eq $true -and $after.scroll -eq $base) "offset $($after.scroll), before the search $base"
  Check 'N4 opened again, the fence shows where it was before the search' ($opened.rolled -eq $false -and $opened.scroll -eq $base) "offset $($opened.scroll), before the search $base"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  PfQuit
}

function PartR() {
  trap { PfTrap $_; continue }
  . C:\pf\kit\rec.ps1
  PfStart -Lang 'zh-CN'
  $r = PsFence 60
  [G]::MoveTo($r.x + 140, $r.y + 180); Start-Sleep -Milliseconds 500
  Case 'R wheel recordings'
  foreach ($run in @(@('wheel8', 8, 80, '把滚轮一格一格连续转 8 格'), @('wheel10', 10, 30, '把滚轮快速转 10 格'))) {
    [CW]::Wheel(40); Start-Sleep -Milliseconds 900
    RecStart $run[0] -X ($r.x - 10) -Y ($r.y - 10) -W ($r.w + 20) -H ($r.h + 20) -Fps 60
    Start-Sleep -Milliseconds 400
    RecMark $run[3]
    PsTurn $run[1] $run[2] | Out-Null
    Start-Sleep -Milliseconds 900
    RecMark '停下'
    Start-Sleep -Milliseconds 300
    RecStop
  }
  # A Peek search that scrolls the fence to its match, then the end of the search. The fence
  # stands at the left, clear of the search box.
  [CW]::Wheel(40); Start-Sleep -Milliseconds 900
  Pf fence move $PsTitle --rect '40,120,360,330' | Out-Null; Start-Sleep -Milliseconds 800
  RecStart 'search' -X 30 -Y 110 -W 380 -H 350 -Fps 60
  Start-Sleep -Milliseconds 500
  RecMark '打开速览，输入 ps0058 搜索'
  PsSearch 'ps0058'
  Start-Sleep -Milliseconds 500
  RecMark '按 Esc 结束搜索'
  PsEndSearch
  Start-Sleep -Milliseconds 700
  RecStop
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  Check 'R recorded' $true (Get-Content "$PfShots\rec-wheel10\stats.txt")
  PfQuit
}

Case 'setup'
CloseWindows
PsClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
$PsTitle = '很多文件'
if ($Part -match 'W') { PartW }
if ($Part -match 'P') { PartP }
if ($Part -match 'F') { PartF }
if ($Part -match 'N') { PartN }
if ($Part -match 'R') { PartR }
Case 'cleanup'
[Anim]::Set($AnimWas)
PsClean
