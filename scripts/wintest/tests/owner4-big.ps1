# Owner round 2026-11-12, package ux4-big: folder fences with many files (200 to 8000).
# Parts (run.sh --var Part=...):
#   M  measures one fence: --var N=1000 --var View=icons|list|details (--var Runs=5). One row per
#      action and run goes to C:\pf\run\big.tsv: how long the user waits and what the UI thread
#      spent it on, read from the app's debug log (`slow message`, `content draw`,
#      `animation run`). --var Lite=1 leaves out Peek, the file added and removed, and memory.
#   E  the same folder in an Explorer window (--var N=1000): open, sort, wheel.
#   B  one bound per fix (the default).
if (-not $Part) { $Part = 'B' }
if (-not $N) { $N = 1000 }
if (-not $View) { $View = 'icons' }
if (-not $Runs) { $Runs = 5 }
$N = [int]$N; $Runs = [int]$Runs
$BgBig = '大文件夹'; $BgSmall = '小文件夹'; $BgSmallN = 30
# The fence window: 600 px wide is 7 columns of icons.
$BgX = 60; $BgY = 80; $BgW = 600; $BgH = 560

Add-Type -Namespace PfBig -Name Clock -MemberDefinition '[DllImport("kernel32.dll")] public static extern void GetSystemTimePreciseAsFileTime(out long t);'
function BgNow() { $ft = [long]0; [PfBig.Clock]::GetSystemTimePreciseAsFileTime([ref]$ft); return $ft / 10000.0 }
function BgStamp($s) { return [DateTimeOffset]::Parse($s).UtcDateTime.ToFileTimeUtc() / 10000.0 }
function BgLogLen() {
  $p = PfLogPath
  if (-not (Test-Path $p)) { return 0 }
  $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $n = $fs.Length; $fs.Close()
  return $n
}
function BgLogFrom($pos) {
  $p = PfLogPath
  if (-not (Test-Path $p)) { return '' }
  $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite')
  try {
    if ($pos -gt 0 -and $pos -le $fs.Length) { $fs.Position = $pos }
    return (New-Object IO.StreamReader($fs, [Text.Encoding]::UTF8)).ReadToEnd()
  } finally { $fs.Close() }
}
# A folder of $n files under the job's run folder: Chinese and Latin names, short and long,
# six kinds of files, dates spread over two years.
function BgFolder($name, $n, [switch]$Plain) {
  $d = Join-Path "$PfRun\bigdata" $name
  New-Item -ItemType Directory -Force $d | Out-Null
  # The small folder's names share no letter with what the steps type into a search.
  if ($Plain) { for ($i = 1; $i -le $n; $i++) { [IO.File]::WriteAllText((Join-Path $d ("杂项 {0:D2}.txt" -f $i)), "small $i") }; return $d }
  $words = @('会议记录', '年度预算表', '设计稿 第二版', '合同扫描件', '周报', '项目计划书 最终版 已确认 可以发给客户的那一份',
    'Quarterly report', 'budget', 'Meeting notes from the long planning session in March', 'IMG', '发票', 'release notes draft')
  $exts = @('txt', 'md', 'log', 'csv', 'json', 'ini')
  $base = (Get-Date).AddDays(-700)
  for ($i = 1; $i -le $n; $i++) {
    $p = Join-Path $d ("{0} {1:D4}.{2}" -f $words[$i % $words.Count], $i, $exts[$i % $exts.Count])
    [IO.File]::WriteAllText($p, "big $i" + ('x' * ($i % 97)))
    [IO.File]::SetLastWriteTime($p, $base.AddMinutes(($i * 7919) % 1000000))
  }
  return $d
}
function BgMedian($a) { $v = @($a | Sort-Object); if ($v.Count -eq 0) { return -1 }; return $v[[int][math]::Floor(($v.Count - 1) / 2)] }
function BgControl() { return [G]::FindWindowW('PecoFence.Control', [NullString]::Value) }
# Returns when the app's UI thread has taken a message again (it is not inside a handler).
function BgIdle($ms = 30000) {
  $r = [IntPtr]::Zero
  [G]::SendMessageTimeoutW((BgControl), 0, [IntPtr]0, [IntPtr]0, 0, $ms, [ref]$r) | Out-Null
}
# Waits until the app has gone quiet: the UI thread answers and the log got no line for 500 ms,
# $min ms after $t0 at the earliest, $max at the latest.
function BgQuiet($t0, $min = 900, $max = 20000) {
  Start-Sleep -Milliseconds 60
  $last = BgLogLen; $lastAt = BgNow
  while ($true) {
    BgIdle
    Start-Sleep -Milliseconds 100
    $now = BgNow; $len = BgLogLen
    if ($len -ne $last) { $last = $len; $lastAt = $now }
    if (($now - $t0) -ge $min -and ($now - $lastAt) -ge 500) { break }
    if (($now - $t0) -ge $max) { break }
  }
}
# What the app logged from byte $pos on, as one row of big.tsv. `first` is the time from $t0
# to the end of the first content draw of the big fence (with $items items when not 0), `last`
# to the end of the last one; `busy` the sum of the outermost window messages of 4 ms or more.
function BgRow($action, $run, $pos, $t0, [int]$items = 0, $note = '') {
  $text = BgLogFrom $pos
  $first = -1; $last = -1; $draws = 0; $drawSum = 0.0; $labels = 0.0; $drawOnly = 0.0; $drawMax = 0.0; $drawn = 0; $scroll = ''
  foreach ($m in [regex]::Matches($text, '(?m)^(\S+Z)\s[^\n]*content draw fence=(\S+) items=(\d+) drawn=(\d+) scroll=(-?[\d.]+) ms=([\d.]+) labels_ms=([\d.]+) draw_ms=([\d.]+)')) {
    $at = (BgStamp $m.Groups[1].Value) - $t0
    if ($at -lt 0) { continue }
    if ($script:BgBigId -and $m.Groups[2].Value -ne $script:BgBigId) { continue }
    if ($items -gt 0 -and [int]$m.Groups[3].Value -ne $items) { continue }
    if ($first -lt 0) { $first = $at }
    $last = $at; $draws++
    $ms = [double]$m.Groups[6].Value
    $drawSum += $ms; $labels += [double]$m.Groups[7].Value; $drawOnly += [double]$m.Groups[8].Value
    if ($ms -gt $drawMax) { $drawMax = $ms }
    $drawn = [int]$m.Groups[4].Value; $scroll = $m.Groups[5].Value
  }
  $busy = 0.0; $worst = 0.0
  foreach ($m in [regex]::Matches($text, '(?m)^(\S+Z)\s[^\n]*slow message ms=([\d.]+)')) {
    if (((BgStamp $m.Groups[1].Value) - $t0) -lt 0) { continue }
    $ms = [double]$m.Groups[2].Value; $busy += $ms; if ($ms -gt $worst) { $worst = $ms }
  }
  $cmds = @()
  foreach ($m in [regex]::Matches($text, '(?m)^(\S+Z)\s[^\n]*slow command ms=([\d.]+) command=(\S+)')) {
    if (((BgStamp $m.Groups[1].Value) - $t0) -lt 0) { continue }
    $cmds += "$($m.Groups[3].Value):$($m.Groups[2].Value)"
  }
  $frames = 0; $work = 0.0; $maxFrame = 0.0
  foreach ($m in [regex]::Matches($text, '(?m)^(\S+Z)\s[^\n]*animation run(?: stuttered)? frames=(\d+) [^\n]*max_frame_ms=([\d.]+)[^\n]*work_ms=([\d.]+)')) {
    if (((BgStamp $m.Groups[1].Value) - $t0) -lt 0) { continue }
    $frames += [int]$m.Groups[2].Value; $work += [double]$m.Groups[4].Value
    if ([double]$m.Groups[3].Value -gt $maxFrame) { $maxFrame = [double]$m.Groups[3].Value }
  }
  $r = { param($v) [math]::Round($v, 1) }
  $row = @($N, $View, $action, $run, (& $r $first), (& $r $last), (& $r $busy), (& $r $worst), $draws, (& $r $drawSum), (& $r $labels), (& $r $drawOnly),
    (& $r $drawMax), $drawn, $scroll, $frames, (& $r $work), (& $r $maxFrame), ($cmds -join ' '), $note) -join "`t"
  $row | Add-Content "$PfRun\big.tsv" -Encoding UTF8
  Log "BIG $row"
  return @{ first = $first; last = $last; busy = $busy; worst = $worst; draws = $draws; labels = $labels; frames = $frames; work = $work; scroll = $scroll; drawMax = $drawMax; drawSum = $drawSum }
}
# One user action on the running app: the body, then quiet, then the row.
function BgAct($action, $run, [scriptblock]$body, [int]$items = 0, [int]$min = 900, [int]$max = 20000) {
  BgIdle
  $pos = BgLogLen
  $t0 = BgNow
  & $body
  BgQuiet $t0 $min $max
  return (BgRow $action $run $pos $t0 $items)
}
# Starts the build as PfStart does, without its waits; returns the time just before.
function BgStart() {
  if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
  $cur = PfLogPath
  if (Test-Path $cur) {
    New-Item -ItemType Directory -Force "$PfRun\logs" | Out-Null
    $k = @(Get-ChildItem "$PfRun\logs\app-*.log" -ErrorAction SilentlyContinue).Count + 1
    Copy-Item $cur "$PfRun\logs\app-$k.log"
    Remove-Item $cur -Force -ErrorAction SilentlyContinue
  }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  [IO.File]::WriteAllLines("$PfRun\script.txt", @('sleep 3600000'), (New-Object Text.UTF8Encoding($false)))
  $t0 = BgNow
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
  return $t0
}
# After BgStart: waits for the control window and for quiet; the row of the start.
function BgStarted($action, $run, $t0, $items) {
  WaitWin 'PecoFence.Control' 60 -Hidden | Out-Null
  BgQuiet $t0 2500 60000
  $text = BgLogFrom 0
  $m = [regex]::Match($text, '(?m)^(\S+Z)\s')
  $note = if ($m.Success) { "first log line at $([math]::Round((BgStamp $m.Groups[1].Value) - $t0, 1)) ms" } else { '' }
  return (BgRow $action $run 0 $t0 $items $note)
}
function BgCommit() { $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1; if ($p) { return [math]::Round($p.PrivateMemorySize64 / 1MB, 1) }; return -1 }
# Screen points of the fence window: a tab of the strip, the middle of the content, the thumb.
function BgTabPt($i) { return @([int]($BgX + $script:BgTabs[$i]), $BgY + 18) }
function BgMid() { return @(($BgX + 300), ($BgY + 300)) }
function BgClickTab($i) { $p = BgTabPt $i; [G]::Click($p[0], $p[1]) }
# The pointer goes to the tab before the clock starts; BgPress is the timed click.
function BgOverTab($i) { $p = BgTabPt $i; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 250 }
function BgPress() { MDown; MUp }

function PartM() {
  trap { PfTrap $_; continue }
  Case "M setup: $N files, $View"
  # A Remote Desktop session starts with Windows' animations off: a user's desktop has them on.
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  $big = BgFolder "big$N" $N
  $small = BgFolder 'small' $BgSmallN -Plain
  PfStart -Script (DumpScript 500 3) -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --portal $small --title $BgSmall --rect "$BgX,$BgY,$BgW,$BgH" | Out-Null
  Pf fence create --portal $big --title $BgBig --rect '700,80,600,560' | Out-Null
  Start-Sleep -Seconds 3
  Pf fence set $BgBig layout $View | Out-Null
  Pf fence set $BgBig sort name | Out-Null
  Pf fence merge $BgBig --into $BgSmall | Out-Null
  BgQuiet (BgNow) 3000 60000
  $d = DumpOf $BgSmall 8
  $script:BgTabs = @($d.rects | ForEach-Object { $_[0] + $_[1] / 2 })
  Log "tabs: $($d.tabs -join ' | ') at $($script:BgTabs -join ', ')"
  $iBig = [array]::IndexOf(@($d.tabs), $BgBig); $iSmall = [array]::IndexOf(@($d.tabs), $BgSmall)
  Check 'M the two folders are tabs of one window' ($iBig -ge 0 -and $iSmall -ge 0) "tabs $($d.tabs -join ' | ')"
  # The big fence's id as the content draw lines name it.
  $script:BgBigId = ''
  BgClickTab $iBig; BgQuiet (BgNow) 2000 60000
  $m = [regex]::Matches((BgLogFrom 0), "content draw fence=(\S+) items=$N ")
  if ($m.Count -gt 0) { $script:BgBigId = $m[$m.Count - 1].Groups[1].Value }
  Check 'M the big fence draws all its files' ($script:BgBigId -ne '') "fence id '$($script:BgBigId)'"
  BgClickTab $iSmall; BgQuiet (BgNow) 1000 20000
  ParkMouse
  PfQuit

  Case "M start and tab, $N files, $View"
  for ($r = 1; $r -le $Runs; $r++) {
    # The small tab was showing when the app quit: the big one is clicked for the first time.
    $t0 = BgStart
    BgStarted 'start, other tab showing' $r $t0 0 | Out-Null
    if ($r -eq 1) { $script:MemSmall = BgCommit }
    BgOverTab $iBig
    BgAct 'tab click, first time' $r { BgPress } $N 1500 60000 | Out-Null
    if ($r -eq 1) { Start-Sleep -Seconds 2; $script:MemBig = BgCommit }
    ParkMouse; Start-Sleep -Milliseconds 300
    BgOverTab $iSmall
    BgAct 'tab click away' $r { BgPress } 0 900 | Out-Null
    BgOverTab $iBig
    BgAct 'tab click, again' $r { BgPress } $N 900 | Out-Null
    ParkMouse; Start-Sleep -Milliseconds 300
    # The big tab is showing when the app quits: the next start draws it.
    PfQuit
    $t0 = BgStart
    BgStarted 'app start to fence drawn' $r $t0 $N | Out-Null
    if ($r -lt $Runs) { BgClickTab $iSmall; BgQuiet (BgNow) 900 20000; ParkMouse; PfQuit }
  }
  Check 'M starts and tab clicks were measured' (@(Get-Content "$PfRun\big.tsv" | Where-Object { $_ -match 'app start to fence drawn' }).Count -eq $Runs) ''

  Case "M view, sort, group, $N files, $View"
  $other = if ($View -eq 'icons') { 'list' } else { 'icons' }
  for ($r = 1; $r -le $Runs; $r++) {
    BgAct "view change to $other" $r { Pf fence set $BgBig layout $other | Out-Null } 0 | Out-Null
    BgAct "view change to $View" $r { Pf fence set $BgBig layout $View | Out-Null } 0 | Out-Null
  }
  for ($r = 1; $r -le $Runs; $r++) {
    BgAct 'sort by date' $r { Pf fence set $BgBig sort date | Out-Null } 0 | Out-Null
    BgAct 'sort by name' $r { Pf fence set $BgBig sort name | Out-Null } 0 | Out-Null
  }
  for ($r = 1; $r -le $Runs; $r++) {
    BgAct 'group by time on' $r { Pf fence set $BgBig groupByDate true | Out-Null } 0 | Out-Null
    BgAct 'group by time off' $r { Pf fence set $BgBig groupByDate false | Out-Null } 0 | Out-Null
  }

  Case "M scrolling, $N files, $View"
  $mid = BgMid
  [G]::MoveTo($mid[0], $mid[1]); Start-Sleep -Milliseconds 800
  for ($r = 1; $r -le $Runs; $r++) { BgAct 'wheel notch' $r { [CW]::Wheel(-1) } 0 700 | Out-Null }
  # Keyboard: a click on the first icon gives the fence the keys.
  [G]::Press(0x24)
  $tx = $BgX + $BgW - 7
  # The thumb at the top of its track (below the header row of the details view).
  $ty = if ($View -eq 'details') { $BgY + 80 } else { $BgY + 50 }
  for ($r = 1; $r -le $Runs; $r++) {
    [G]::Click($BgX + 40, $BgY + 36 + 40); Start-Sleep -Milliseconds 300
    BgAct 'End' $r { [G]::Press(0x23) } 0 900 | Out-Null
    BgAct 'Home' $r { [G]::Press(0x24) } 0 900 | Out-Null
    # The thumb, from the top of its track to the bottom.
    [G]::MoveTo($tx, $ty); Start-Sleep -Milliseconds 500
    $a = BgAct 'thumb dragged top to bottom' $r { MDown; Start-Sleep -Milliseconds 60; Glide $tx $ty $tx ($BgY + $BgH - 6) 30 25; Start-Sleep -Milliseconds 100; MUp } 0 1200
    [G]::MoveTo($mid[0], $mid[1]); Start-Sleep -Milliseconds 200
    [G]::Click($BgX + 40, $BgY + 36 + 40); Start-Sleep -Milliseconds 200
    [G]::Press(0x24); Start-Sleep -Milliseconds 900
  }
  if (-not $Lite) { $script:MemScrolled = BgCommit }

  Case "M width, $N files, $View"
  ParkMouse
  for ($r = 1; $r -le $Runs; $r++) {
    BgAct 'one column wider' $r { Pf fence resize $BgSmall --w ($BgW + 80) | Out-Null } 0 | Out-Null
    BgAct 'one column narrower' $r { Pf fence resize $BgSmall --w $BgW | Out-Null } 0 | Out-Null
  }
  $ex = $BgX + $BgW - 2; $ey = $BgY + 300
  Log "edge hit test: $(NcHit ([G]::FindWindowW('PecoFence.Fence', [NullString]::Value)) $ex $ey)"
  for ($r = 1; $r -le $Runs; $r++) {
    [G]::MoveTo($ex, $ey); Start-Sleep -Milliseconds 300
    BgAct 'edge dragged across five columns' $r { MDown; Start-Sleep -Milliseconds 80; Glide $ex $ey ($ex + 400) $ey 40 25; Start-Sleep -Milliseconds 150; MUp } 0 1200 | Out-Null
    [G]::MoveTo($ex + 400, $ey); Start-Sleep -Milliseconds 300
    BgAct 'edge dragged back' $r { MDown; Start-Sleep -Milliseconds 80; Glide ($ex + 400) $ey $ex $ey 40 25; Start-Sleep -Milliseconds 150; MUp } 0 1200 | Out-Null
  }
  Pf fence resize $BgSmall --w $BgW | Out-Null
  ParkMouse

  Case "M filter, $N files, $View"
  for ($r = 1; $r -le $Runs; $r++) {
    [G]::Click($BgX + 40, $BgY + 36 + 40); Start-Sleep -Milliseconds 300
    BgAct 'filter box opened (Ctrl+F)' $r { Chord 0x11 0x46 } 0 700 | Out-Null
    BgAct 'filter: a letter typed' $r { [G]::Type('b') } 0 900 | Out-Null
    BgAct 'filter: a second letter' $r { [G]::Type('u') } 0 900 | Out-Null
    BgAct 'filter cleared (Esc)' $r { [G]::Press(0x1B) } $N 900 | Out-Null
  }

  if (-not $Lite) {
    Case "M Peek search, a file added and removed, memory, $N files"
    for ($r = 1; $r -le $Runs; $r++) {
      BgAct 'Peek opened' $r { PeekKey } 0 1200 | Out-Null
      BgAct 'Peek search: a letter typed' $r { [G]::Type('b') } 0 1200 | Out-Null
      BgAct 'Peek closed (Esc twice)' $r { [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B) } 0 1200 | Out-Null
    }
    $new = Join-Path $big '000 新来的文件.txt'
    for ($r = 1; $r -le $Runs; $r++) {
      BgAct 'a file added in the folder' $r { [IO.File]::WriteAllText($new, 'new') } ($N + 1) 1500 | Out-Null
      BgAct 'a file removed from the folder' $r { Remove-Item -LiteralPath $new -Force } $N 1500 | Out-Null
    }
    $withBig = BgCommit
    Pf fence delete $BgBig | Out-Null
    Start-Sleep -Seconds 3
    $without = BgCommit
    $mem = "N=$N view=$View small tab showing=$($script:MemSmall) big tab shown=$($script:MemBig) after scrolling through all=$($script:MemScrolled) at the end=$withBig big fence deleted=$without MB"
    Log "BIGMEM $mem"
    "MEM`t$mem" | Add-Content "$PfRun\big.tsv" -Encoding UTF8
  }
  Shot "m-$N-$View-end"
  PfQuit
  [Anim]::Set($animWas)
  Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
}

if ($Part -match 'M') { PartM }
