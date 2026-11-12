# Owner round 2026-11-12, third batch, package ux3-peek: the Peek, its search box, notices.
# Runs on a 1600x900 box at 100 %.
#   run.sh --var Part=S ...   the Peek search box when a letter makes the list shorter: no frame
#                             shows the old rows cut at the new height, the height changes over
#                             several steps, and the new rows come no later than before
#   run.sh --var Part=H ...   a Peek over hidden fences (隐藏所有栅栏) ends in one motion: the
#                             fences fade with the dimmer, not after it
#   run.sh --var Part=C ...   a smart fence's count with a plus (「3+」): the pointer resting on it
#                             says of how many files the fence shows the newest
#   run.sh --var Part=R ...   a smart fence whose searched folder was renamed: the pointer coming
#                             onto the fence makes it look again at once (it kept the old files
#                             for up to a minute)
# Without --var every part runs.
if (-not $Part) { $Part = 'SHCR' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1
$U3Files = @('报告.txt', '宝贝计划.txt', '报价单 客户甲.txt', '报销单 10月.txt', '预算表 2026.xlsx', '年度报告 2025 董事会审阅版（含附录）.pdf', '备忘 周会.md', '笔记 读书.txt')
function U3Clean() {
  foreach ($n in $U3Files) { $p = Join-Path $PfDesk $n; if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Force -ErrorAction SilentlyContinue } }
}
function U3PeekOn() { return ([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0 }
function U3WaitPeek($on, $sec = 4) { for ($i = 0; $i -lt $sec * 20 -and (U3PeekOn) -ne $on; $i++) { Start-Sleep -Milliseconds 50 }; return ((U3PeekOn) -eq $on) }
function U3Box() { $b = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value); if ($b -ne [IntPtr]::Zero -and [G]::IsWindowVisible($b)) { return $b }; return [IntPtr]::Zero }
function U3Lum($c) { return [int](0.299 * $c.R + 0.587 * $c.G + 0.114 * $c.B) }
function U3Frames($name) {
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
function U3Marks($name) { $m = @{}; foreach ($line in (Get-Content "$PfShots\rec-$name\marks.tsv" -Encoding UTF8)) { $p = $line -split "`t", 2; if ($p.Count -eq 2) { $m[$p[1]] = [int]$p[0] } }; return $m }
# The box's height in a frame of a recording that starts at the box's top left corner: how far
# down its bright left padding goes (the dimmed desktop under it is dark).
function U3BoxH($bmp) { $y = 0; while ($y -lt $bmp.Height -and (U3Lum $bmp.GetPixel(6, $y)) -gt 170) { $y += 2 }; return $y }
# How many sample points of the list's text differ between two frames (rows from `top` to `bottom`).
function U3Diff($a, $b, $top, $bottom) {
  $n = 0
  for ($y = $top; $y -lt $bottom; $y += 3) { for ($x = 44; $x -lt 300; $x += 2) { if ([Math]::Abs((U3Lum $a.GetPixel($x, $y)) - (U3Lum $b.GetPixel($x, $y))) -gt 70) { $n++ } } }
  return $n
}

# How many sample points of the fence's rectangle (20,20 to 380,420 in the recording) differ from
# the bare desktop in `bare` by more than `step`.
function U3Fence($a, $bare, $step) {
  $n = 0
  for ($y = 24; $y -lt 416; $y += 6) { for ($x = 24; $x -lt 376; $x += 4) { if ([Math]::Abs((U3Lum $a.GetPixel($x, $y)) - (U3Lum $bare.GetPixel($x, $y))) -gt $step) { $n++ } } }
  return $n
}

function U3PartS() {
  trap { PfTrap $_; continue }
  Case 'search box: a shorter list'
  U3Clean
  foreach ($n in $U3Files) { DeskFile $n | Out-Null }
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '资料' --rect '60,120,360,400' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move @($U3Files | ForEach-Object { "$PfDesk\$_" }) --to '资料' | Out-Null
  Start-Sleep -Seconds 2
  [G]::MoveTo(800, 860)
  $bx = 480; $by = 227
  # A first search warms the fonts and the icons; the measured one is the second.
  PeekKey; U3WaitPeek $true | Out-Null; Start-Sleep -Milliseconds 900
  [G]::Type('b'); Start-Sleep -Milliseconds 1200; [G]::Type('a'); Start-Sleep -Milliseconds 900
  [G]::Press(0x1B); U3WaitPeek $false | Out-Null; Start-Sleep -Milliseconds 900
  PeekKey; U3WaitPeek $true | Out-Null; Start-Sleep -Milliseconds 900
  RecStart 'S-shorter' -X $bx -Y $by -W 640 -H 560 -Fps 60
  Start-Sleep -Milliseconds 300
  RecMark 'first letter'
  [G]::Type('b'); Start-Sleep -Milliseconds 1100
  $box = U3Box; $r1 = Rect $box; $h1 = $r1.B - $r1.T
  RecMark 'second letter'
  [G]::Type('a')
  # The window's height, asked every few ms for half a second.
  $hs = @(); $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 500) { $r = Rect $box; $hs += ($r.B - $r.T); Start-Sleep -Milliseconds 4 }
  $r2 = Rect $box; $h2 = $r2.B - $r2.T
  Start-Sleep -Milliseconds 500
  RecMark 'third letter'
  [G]::Type('x'); Start-Sleep -Milliseconds 900
  $r3 = Rect $box; $h3 = $r3.B - $r3.T
  RecMark 'Backspace'
  [G]::Press(0x08)
  $hg = @(); $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 500) { $r = Rect $box; $hg += ($r.B - $r.T); Start-Sleep -Milliseconds 4 }
  Start-Sleep -Milliseconds 500
  $r4 = Rect $box; $h4 = $r4.B - $r4.T
  RecStop
  Check 'the box opened at its usual place' ($r1.L -eq $bx -and $r1.T -eq $by) "box=$($r1.L),$($r1.T),$($r1.R),$($r1.B)"
  Check 'the second letter makes the list shorter' ($h2 -lt $h1 - 20) "height $h1 -> $h2 -> $h3 -> $h4"
  $between = @($hs | Where-Object { $_ -lt $h1 -and $_ -gt $h2 } | Select-Object -Unique)
  Check 'a shorter list: the height changes over several steps' ($between.Count -ge 2) "heights between $h1 and ${h2}: $($between -join ' ')"
  $grow = @($hg | Where-Object { $_ -gt $h3 -and $_ -lt $h4 } | Select-Object -Unique)
  Check 'a longer list: the height changes over several steps' ($h4 -gt $h3 -and $grow.Count -ge 1) "heights between $h3 and ${h4}: $($grow -join ' ')"
  Check 'the height comes to rest at the list' ($h4 -eq $h2) "after Backspace $h4, after the second letter $h2"
  # The frames after the second letter: the box's height and whether its rows are the new ones
  # (compared with the settled frame just before the third letter).
  $fr = U3Frames 'S-shorter'; $mk = U3Marks 'S-shorter'
  $t2 = $mk['second letter']; $t3 = $mk['third letter']
  # A frame's time is when its grab began and a grab takes about 30 ms: the frames compared
  # with keep 150 ms clear of the keys, and the frames looked at start a grab before the key.
  $settled = $fr | Where-Object { $_.T -lt $t3 - 150 } | Select-Object -Last 1
  $old = $fr | Where-Object { $_.T -lt $t2 - 150 } | Select-Object -Last 1
  $top = 100; $bottom = $h2 - 44
  $oldDiff = U3Diff $old.B $settled.B $top $bottom
  $wrong = 0; $wrongAt = @(); $newAt = -999; $series = @()
  foreach ($f in ($fr | Where-Object { $_.T -ge $t2 - 40 -and $_.T -lt $t2 + 600 })) {
    $h = U3BoxH $f.B; $d = U3Diff $f.B $settled.B $top $bottom
    $series += "$($f.T - $t2):$h/$d"
    $isOld = $d -gt [Math]::Max(40, $oldDiff / 3)
    if ($isOld -and $h -lt $h1 - 6) { $wrong++; $wrongAt += ($f.T - $t2) }
    if (-not $isOld -and $newAt -eq -999) { $newAt = $f.T - $t2 }
  }
  Log "frames after the second letter (ms:height/points that differ from the new rows): $($series -join ' ')"
  Log "new rows first seen $newAt ms after the key"
  Check 'the old and the new list can be told apart in the frames' ($oldDiff -gt 120) "points of the old list that differ from the new one: $oldDiff"
  Check 'no frame shows the old rows in a window already shorter' ($wrong -eq 0) "frames: $wrong (at $($wrongAt -join ', ') ms after the key); new rows first seen after $newAt ms"
  Check 'the new rows are on screen within 150 ms of the key' ($newAt -ne -999 -and $newAt -le 150) "new rows first seen in the frame grabbed from $newAt ms after the key"
  foreach ($f in $fr) { $f.B.Dispose() }
  [G]::Press(0x1B); U3WaitPeek $false | Out-Null; Start-Sleep -Milliseconds 700
  # With Windows' animations off the height changes in one step.
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  PeekKey; U3WaitPeek $true | Out-Null; Start-Sleep -Milliseconds 700
  [G]::Type('b'); Start-Sleep -Milliseconds 900
  $box = U3Box; $r1 = Rect $box; $h1 = $r1.B - $r1.T
  [G]::Type('a')
  $hs = @(); $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 400) { $r = Rect $box; $hs += ($r.B - $r.T); Start-Sleep -Milliseconds 4 }
  $r2 = Rect $box; $h2 = $r2.B - $r2.T
  $between = @($hs | Where-Object { $_ -lt $h1 -and $_ -gt $h2 } | Select-Object -Unique)
  Check 'animations off: the height changes in one step' ($h2 -lt $h1 -and $between.Count -eq 0) "height $h1 -> $h2, between: $($between -join ' ')"
  [G]::Press(0x1B); U3WaitPeek $false | Out-Null; Start-Sleep -Milliseconds 300
  [Anim]::Set($anim)
  PfQuit
  U3Clean
}

function U3PartH() {
  trap { PfTrap $_; continue }
  Case 'a Peek over hidden fences ends in one motion'
  U3Clean
  foreach ($n in $U3Files) { DeskFile $n | Out-Null }
  $anim = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '资料' --rect '60,120,360,400' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move @($U3Files | ForEach-Object { "$PfDesk\$_" }) --to '资料' | Out-Null
  Start-Sleep -Seconds 2
  [G]::MoveTo(800, 860)
  Pf fence hide-all | Out-Null
  Start-Sleep -Milliseconds 1500
  Check 'every fence is hidden' ((VisFences) -eq 0) "visible fences=$(VisFences)"
  # The recording holds the fence (at 60,120) and bare desktop right of it.
  RecStart 'H-hidden' -X 40 -Y 100 -W 700 -H 440 -Fps 60
  Start-Sleep -Milliseconds 300
  RecMark 'Peek'
  PeekKey; U3WaitPeek $true | Out-Null; Start-Sleep -Milliseconds 1200
  Check 'the Peek shows the hidden fence' ((VisFences) -ge 1) "visible fences=$(VisFences)"
  RecMark 'Esc'
  [G]::Press(0x1B); U3WaitPeek $false | Out-Null; Start-Sleep -Milliseconds 1200
  RecStop
  Check 'after the Peek the fence is hidden again' ((VisFences) -eq 0) "visible fences=$(VisFences)"
  # Per frame: F, a point of the fence's title bar (40,14 inside the fence), and D, a point of
  # bare desktop (the dimmer). At the end both are the plain desktop.
  $fr = U3Frames 'H-hidden'; $mk = U3Marks 'H-hidden'
  $tEsc = $mk['Esc']
  $last = $fr[$fr.Count - 1]
  $during = $fr | Where-Object { $_.T -lt $tEsc } | Select-Object -Last 1
  $dx = 560; $dy = 300
  $dEnd = U3Lum $last.B.GetPixel($dx, $dy); $dOn = U3Lum $during.B.GetPixel($dx, $dy)
  # The dim darkens the desktop inside the fence's rectangle too: a point counts for the fence
  # when it differs from the bare desktop by more than the dim alone does.
  $dimStep = [Math]::Abs($dOn - $dEnd) + 12
  $fOn = U3Fence $during.B $last.B $dimStep; $fEnd = 0
  $alone = 0; $aloneAt = @(); $series = @(); $firstMove = -1; $lastMove = -1
  foreach ($f in ($fr | Where-Object { $_.T -ge $tEsc })) {
    $fv = U3Fence $f.B $last.B $dimStep; $dv = U3Lum $f.B.GetPixel($dx, $dy)
    $series += "$($f.T - $tEsc):$fv/$dv"
    $dimGone = [Math]::Abs($dv - $dEnd) -le 4
    $fenceThere = $fv -gt [Math]::Max(6, $fOn / 20)
    if ($dimGone -and $fenceThere) { $alone++; $aloneAt += ($f.T - $tEsc) }
    if ($fenceThere -or -not $dimGone) { $lastMove = $f.T - $tEsc }
    if ($firstMove -lt 0 -and ($fv -lt $fOn * 0.9 -or [Math]::Abs($dv - $dOn) -gt 4)) { $firstMove = $f.T - $tEsc }
  }
  Log "frames after Esc (ms:fence/desktop): $($series -join ' ')"
  $row = "during the Peek: $fOn points of the fence show, desktop brightness $dOn; at the end 0 points, brightness $dEnd; frames with the dim gone and the fence still there: $alone ($($aloneAt -join ', ') ms); the motion ran from $firstMove to $lastMove ms after Esc"
  Check 'the fence and the dim can be told from the desktop' ($fOn -gt 40 -and [Math]::Abs($dOn - $dEnd) -gt 8) $row
  Check 'the fence fades with the dim, not after it (at most 2 frames of the fence alone)' ($alone -le 2) $row
  foreach ($f in $fr) { $f.B.Dispose() }
  Pf fence show-all | Out-Null
  [Anim]::Set($anim)
  PfQuit
  U3Clean
}

# The texts of the app's tooltips on screen.
function U3Tips() { return (@(ProcTops 'tooltips_class32' 'pecofence' | ForEach-Object { try { [Windows.Automation.AutomationElement]::FromHandle($_).Current.Name } catch { '?' } }) -join ' / ') }
# Rests the pointer on points of the title row left of the arrow, one after the other, and
# returns the tips that came up. `shot`: a picture of the title row's right end with the pointer
# at rest there (with the tip, when one came up).
function U3CapsuleTips($title, $shot) {
  $f = @(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq $title })[0]
  $right = $f.windowRect.x + $f.windowRect.w; $y = $f.windowRect.y + 16
  $tips = @()
  for ($x = $right - 84; $x -le $right - 40; $x += 11) {
    [G]::MoveTo($right - 200, $y + 400); Start-Sleep -Milliseconds 300
    [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 1400
    $t = U3Tips
    if ($t) { $tips += $t }
    if ($shot -and ($t -or $tips.Count -eq 0)) { Crop $shot ($right - 240) ($f.windowRect.y - 10) 300 110 | Out-Null }
  }
  [G]::MoveTo($right - 200, $y + 400); Start-Sleep -Milliseconds 300
  return $tips
}

function U3PartC() {
  trap { PfTrap $_; continue }
  Case 'a smart fence count with a plus says what it means'
  $dir = "$PfDocs\智能栅栏提示检查"
  Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
  foreach ($i in 1..8) { AgedFile $dir "报告 第 $i 期.txt" $i | Out-Null }
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '最近报告' --smart-folder $dir --rect '500,300,360,240' | Out-Null
  Check 'the fence shows the eight files' (WaitSmart '最近报告' { $s.shown -eq 8 } 15) (SmartLine '最近报告')
  $all = @(U3CapsuleTips '最近报告')
  Check 'a count without a plus has no tip about more files' (@($all | Where-Object { $_ -match '共找到' }).Count -eq 0) "tips: $($all -join ' | ')"
  Pf fence set '最近报告' smartLimit 3 | Out-Null
  Check 'with a limit of 3 the fence shows three files' (WaitSmart '最近报告' { $s.shown -eq 3 } 15) (SmartLine '最近报告')
  Start-Sleep -Milliseconds 600
  $cut = @(U3CapsuleTips '最近报告' 'C-capsule')
  Check 'the count 「3+」 says of how many files it shows the newest' (@($cut | Where-Object { $_ -eq '显示最新的 3 个，共找到 8 个' }).Count -ge 1) "tips: $($cut -join ' | ')"
  PfQuit
  Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
}

function U3PartR() {
  trap { PfTrap $_; continue }
  Case 'a smart fence whose folder was renamed'
  $dir = "$PfDocs\智能栅栏改名检查"; $moved = "$PfDocs\智能栅栏改名检查 旧"
  Remove-Item -Recurse -Force $dir, $moved -ErrorAction SilentlyContinue
  foreach ($i in 1..5) { AgedFile $dir "合同 第 $i 份.txt" $i | Out-Null }
  PfStart -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  ClearFences
  Pf fence create --title '合同' --smart-folder $dir --rect '500,300,360,240' | Out-Null
  Check 'the fence shows the five files' (WaitSmart '合同' { $s.shown -eq 5 } 15) (SmartLine '合同')
  [G]::MoveTo(200, 800); Start-Sleep -Milliseconds 500
  Rename-Item -LiteralPath $dir -NewName (Split-Path $moved -Leaf)
  Start-Sleep -Milliseconds 1500
  Log "after the rename, pointer away: $(SmartLine '合同')"
  # The pointer comes onto the fence, as before a double-click on one of its icons.
  [G]::MoveTo(640, 480); Start-Sleep -Milliseconds 200; [G]::MoveTo(650, 470)
  Check 'the pointer on the fence: its files are gone within 4 s' (WaitSmart '合同' { $s.shown -eq 0 } 4) (SmartLine '合同')
  $f = @(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq '合同' })[0]
  Crop 'R-renamed' ($f.windowRect.x - 10) ($f.windowRect.y - 10) 380 260 | Out-Null
  PfQuit
  Remove-Item -Recurse -Force $dir, $moved -ErrorAction SilentlyContinue
}

if ($Part -match 'S') { U3PartS }
if ($Part -match 'H') { U3PartH }
if ($Part -match 'C') { U3PartC }
if ($Part -match 'R') { U3PartR }
