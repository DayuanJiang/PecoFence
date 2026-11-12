# Owner round 2026-11-12, package ux4-big: folder fences with many files (200 to 8000).
# Parts (run.sh --var Part=...):
#   M  measures one fence: --var N=1000 --var View=icons|list|details (--var Runs=5). One row per
#      action and run goes to C:\pf\run\big.tsv: how long the user waits and what the UI thread
#      spent it on, read from the app's debug log (`slow message`, `content draw`,
#      `animation run`). --var Lite=1 leaves out Peek, the file added and removed, and memory.
#      --var Only=scroll or Only=peek measures those rows alone, right after a start.
#   R  memory: --var N=1000 (--var Runs=3). Private commit with the small folder alone, with the
#      big folder's fence open, after scrolling through all of it, and 40 s later.
#   P  pictures of a fence of 1000 files at rest (three views, three scroll positions, a
#      filter) in shots\p-*.png, to compare two builds pixel by pixel on the dev box.
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
  # The same dates in every run (the pictures of two builds are compared pixel by pixel),
  # none of them after today: a file made by the steps is the newest.
  $base = [datetime]'2024-06-01T09:00:00'
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
  return @{ first = $first; last = $last; busy = $busy; worst = $worst; draws = $draws; labels = $labels; frames = $frames; work = $work; maxFrame = $maxFrame; scroll = $scroll; drawMax = $drawMax; drawSum = $drawSum }
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

  if (-not $Only) {
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

  } else {
    # --var Only=scroll: the scrolling rows alone. The app starts and the big tab is shown
    # for the first time, so no name below the first screen is fitted yet.
    $t0 = BgStart
    BgStarted 'start, other tab showing' 1 $t0 0 | Out-Null
    BgOverTab $iBig
    BgAct 'tab click, first time' 1 { BgPress } $N 1500 60000 | Out-Null
    ParkMouse; Start-Sleep -Milliseconds 300
    if ($Only -eq 'peek') {
      # --var Only=peek: the Peek search rows alone.
      Case "M Peek search, $N files"
      for ($r = 1; $r -le $Runs; $r++) {
        BgAct 'Peek opened' $r { PeekKey } 0 1200 | Out-Null
        BgAct 'Peek search: a letter typed' $r { [G]::Type('b') } 0 1200 | Out-Null
        BgAct 'Peek closed (Esc twice)' $r { [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B) } 0 1200 | Out-Null
      }
      PfQuit
      [Anim]::Set($animWas)
      Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
      return
    }
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

  if ($Only -eq 'scroll') {
    PfQuit
    [Anim]::Set($animWas)
    Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
    return
  }

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
    # By name the new file is the first: every icon moves one cell on.
    Pf fence set $BgBig sort name | Out-Null
    BgQuiet (BgNow) 1200 20000
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

# Wheel notches towards the user over the fence, then the pointer away and the fence at rest.
function BgNotches([int]$n) {
  $mid = BgMid
  [G]::MoveTo($mid[0], $mid[1]); Start-Sleep -Milliseconds 300
  for ($k = 0; $k -lt [math]::Abs($n); $k++) { [CW]::Wheel($(if ($n -lt 0) { -1 } else { 1 })); Start-Sleep -Milliseconds 60 }
  Start-Sleep -Milliseconds 900
  ParkMouse
  BgQuiet (BgNow) 1200 8000
}
function BgPicture($name) { $b = Crop $name $BgX $BgY $BgW $BgH; $b.Dispose() }

function PartP() {
  trap { PfTrap $_; continue }
  Case 'P pictures at rest'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  $big = BgFolder 'pictures' 1000
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --portal $big --title $BgBig --rect "$BgX,$BgY,$BgW,$BgH" | Out-Null
  Start-Sleep -Seconds 3
  foreach ($v in 'icons', 'list', 'details') {
    Pf fence set $BgBig layout $v | Out-Null
    ParkMouse
    BgQuiet (BgNow) 2000 10000
    BgPicture "p-$v-1-top"
    BgNotches -5
    BgPicture "p-$v-2-five-notches"
    BgNotches -40
    BgPicture "p-$v-3-forty-five-notches"
    BgNotches 60
  }
  Pf fence set $BgBig layout icons | Out-Null
  Pf fence set $BgBig groupByDate true | Out-Null
  ParkMouse
  BgQuiet (BgNow) 2000 10000
  BgPicture 'p-grouped-1-top'
  BgNotches -12
  BgPicture 'p-grouped-2-twelve-notches'
  BgNotches 20
  Pf fence set $BgBig groupByDate false | Out-Null
  BgQuiet (BgNow) 1500 10000
  # A filter: the first icon clicked (it stays selected), Ctrl+F, two letters, Enter.
  [G]::Click($BgX + 40, $BgY + 36 + 40); Start-Sleep -Milliseconds 400
  Chord 0x11 0x46; Start-Sleep -Milliseconds 700
  [G]::Type('bu'); Start-Sleep -Milliseconds 900
  [G]::Press(0x0D); Start-Sleep -Milliseconds 600
  ParkMouse
  BgQuiet (BgNow) 2000 10000
  BgPicture 'p-filter-bu'
  $shown = (FenceGet $BgBig).filter.shown
  Check 'P the pictures were taken' (@(Get-ChildItem "$PfShots\p-*.png").Count -eq 12) "pictures $(@(Get-ChildItem "$PfShots\p-*.png").Count), the filter shows $shown"
  PfQuit
  [Anim]::Set($animWas)
  Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
}

function PartR() {
  trap { PfTrap $_; continue }
  Case "R memory, $N files"
  $big = BgFolder "big$N" $N
  $small = BgFolder 'small' $BgSmallN -Plain
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --portal $small --title $BgSmall --rect '700,80,600,560' | Out-Null
  Start-Sleep -Seconds 2
  PfQuit
  $tx = $BgX + $BgW - 7; $ty = $BgY + 50
  for ($r = 1; $r -le $Runs; $r++) {
    $t0 = BgStart
    WaitWin 'PecoFence.Control' 60 -Hidden | Out-Null
    BgQuiet $t0 6000 30000
    $alone = BgCommit
    Pf fence create --portal $big --title $BgBig --rect "$BgX,$BgY,$BgW,$BgH" | Out-Null
    BgQuiet (BgNow) 5000 30000
    $open = BgCommit
    # Through all of it: the thumb from the top to the bottom in 80 steps, then back.
    [G]::MoveTo($tx, $ty); Start-Sleep -Milliseconds 500
    MDown; Start-Sleep -Milliseconds 60; Glide $tx $ty $tx ($BgY + $BgH - 6) 80 40; Start-Sleep -Milliseconds 200
    Glide $tx ($BgY + $BgH - 6) $tx $ty 80 40; Start-Sleep -Milliseconds 100; MUp
    ParkMouse
    BgQuiet (BgNow) 3000 30000
    $scrolled = BgCommit
    $later = -1
    if ($r -eq $Runs) { Start-Sleep -Seconds 40; $later = BgCommit }
    Pf fence delete $BgBig | Out-Null
    Start-Sleep -Seconds 2
    $row = "N=$N run=$r small folder alone=$alone big fence open=$open after scrolling through all=$scrolled 40 s later=$later MB; per 1000 files open=$([math]::Round(($open - $alone) * 1000 / $N, 2)) scrolled=$([math]::Round(($scrolled - $alone) * 1000 / $N, 2)) MB"
    Log "BIGMEM $row"
    "MEMR`t$row" | Add-Content "$PfRun\big.tsv" -Encoding UTF8
    PfQuit
  }
  Check 'R memory was read' (@(Get-Content "$PfRun\big.tsv" | Where-Object { $_ -match '^MEMR' }).Count -eq $Runs) ''
  Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
}

# A part of the screen as numbers, and whether two such grabs differ.
function BgGrab($x, $y) { return [Px]::Grab($x, $y, 360, 240) }
function BgChanged($a, $b) { return ([Px]::Diff($a, $b, 0, 359, 0, 239) -gt 0.02) }
# After an action at $t0: when the picture at ($x, $y) first changed and when it stopped
# changing (three equal grabs in a row), in ms; grabs come about every 25 ms.
function BgWatch($t0, $x, $y, $before, $max = 15000) {
  $first = -1; $settled = -1; $same = 0; $prev = $before; $lastChange = 0
  while (((BgNow) - $t0) -lt $max) {
    $g = BgGrab $x $y
    $at = (BgNow) - $t0
    if (BgChanged $g $prev) { if ($first -lt 0) { $first = $at }; $lastChange = $at; $same = 0 } else { $same++ }
    if ($prev -ne $before) { $prev.Dispose() }
    $prev = $g
    if ($first -ge 0 -and $same -ge 3) { $settled = $lastChange; break }
    if ($first -lt 0 -and $at -gt 4000) { break }
  }
  if ($prev -ne $before) { $prev.Dispose() }
  return @([math]::Round($first), [math]::Round($settled))
}

function PartE() {
  trap { PfTrap $_; continue }
  Case "E Explorer, $N files"
  $big = BgFolder "big$N" $N
  CloseWindows
  $sh = New-Object -ComObject Shell.Application
  for ($r = 1; $r -le $Runs; $r++) {
    # Where the window will be is not known before it opens: the open is timed by the window
    # appearing and by its file list coming to rest.
    $t0 = BgNow
    Start-Process explorer.exe $big
    $w = $null
    for ($k = 0; $k -lt 600 -and -not $w; $k++) {
      Start-Sleep -Milliseconds 20
      $w = @($sh.Windows() | Where-Object { $_.LocationURL -and $_.Document -and $_.Document.Folder.Self.Path -eq $big }) | Select-Object -First 1
    }
    $shown = [math]::Round((BgNow) - $t0)
    $rc = Rect ([IntPtr][int64]$w.HWND)
    # The file list: right of the navigation pane, below the bars.
    $gx = $rc.L + 330; $gy = $rc.T + 190
    $blank = BgGrab $gx $gy
    $o = BgWatch $t0 $gx $gy $blank 20000
    Start-Sleep -Milliseconds 1500
    $view = $w.Document.CurrentViewMode
    $before = BgGrab $gx $gy
    $t1 = BgNow
    $w.Document.SortColumns = 'prop:-System.DateModified;'
    $sd = BgWatch $t1 $gx $gy $before
    Start-Sleep -Milliseconds 800
    $before2 = BgGrab $gx $gy
    $t2 = BgNow
    $w.Document.SortColumns = 'prop:System.ItemNameDisplay;'
    $sn = BgWatch $t2 $gx $gy $before2
    Start-Sleep -Milliseconds 800
    [G]::MoveTo($gx + 180, $gy + 120); Start-Sleep -Milliseconds 400
    $before3 = BgGrab $gx $gy
    $t3 = BgNow
    [CW]::Wheel(-1)
    $wh = BgWatch $t3 $gx $gy $before3
    $row = "N=$N run=$r view mode=$view window found=$shown ms, list first drawn=$($o[0]) settled=$($o[1]) ms; sort by date first=$($sd[0]) settled=$($sd[1]) ms; sort by name first=$($sn[0]) settled=$($sn[1]) ms; wheel notch first=$($wh[0]) settled=$($wh[1]) ms"
    Log "BIGEXP $row"
    "EXP`t$row" | Add-Content "$PfRun\big.tsv" -Encoding UTF8
    if ($r -eq 1) { Shot "e-$N-explorer" }
    foreach ($b in $blank, $before, $before2, $before3) { $b.Dispose() }
    ParkMouse
    $w.Quit()
    Start-Sleep -Milliseconds 1500
  }
  Check 'E Explorer was timed' (@(Get-Content "$PfRun\big.tsv" | Where-Object { $_ -match '^EXP' }).Count -eq $Runs) ''
  CloseWindows
  Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
}

# One bound per fix, on a folder of 4000 files: loose enough for a box without a graphics
# card, tight enough to fail on the build before the fix (the numbers of both are in the
# details). Medians of three.
function PartB() {
  trap { PfTrap $_; continue }
  Case 'B setup: 4000 files'
  $script:N = 4000; $script:View = 'icons'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  $big = BgFolder 'big4000' 4000
  $small = BgFolder 'small' $BgSmallN -Plain
  PfStart -Script (DumpScript 500 2) -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --portal $small --title $BgSmall --rect "$BgX,$BgY,$BgW,$BgH" | Out-Null
  Pf fence create --portal $big --title $BgBig --rect '700,80,600,560' | Out-Null
  Start-Sleep -Seconds 3
  Pf fence set $BgBig sort name | Out-Null
  Pf fence merge $BgBig --into $BgSmall | Out-Null
  BgQuiet (BgNow) 3000 60000
  $d = DumpOf $BgSmall 8
  $script:BgTabs = @($d.rects | ForEach-Object { $_[0] + $_[1] / 2 })
  $iBig = [array]::IndexOf(@($d.tabs), $BgBig); $iSmall = [array]::IndexOf(@($d.tabs), $BgSmall)
  $script:BgBigId = ''
  BgClickTab $iBig; BgQuiet (BgNow) 2000 60000
  $m = [regex]::Matches((BgLogFrom 0), 'content draw fence=(\S+) items=4000 ')
  if ($m.Count -gt 0) { $script:BgBigId = $m[$m.Count - 1].Groups[1].Value }
  Check 'B the folder of 4000 files is a tab and draws' ($iBig -ge 0 -and $script:BgBigId -ne '') "tabs $($d.tabs -join ' | ') id '$($script:BgBigId)'"

  Case 'B1 a name is fitted when its icon is first drawn'
  $tab = @(); $names = @()
  for ($r = 1; $r -le 3; $r++) {
    BgOverTab $iSmall; BgAct 'B tab click away' $r { BgPress } 0 900 | Out-Null
    BgOverTab $iBig
    $a = BgAct 'B tab click, again' $r { BgPress } 4000 900
    $tab += $a.first; $names += $a.labels
  }
  ParkMouse
  $t = BgMedian $tab
  Check 'B1 a click on the tab of a folder of 4000 files shows it within 120 ms (before: about 280 ms, of which 270 ms fit 4000 names)' ($t -ge 0 -and $t -lt 120) "first picture after $([math]::Round($t)) ms (runs $(@($tab | ForEach-Object { [math]::Round($_) }) -join ', ')); names and icon requests $(@($names | ForEach-Object { [math]::Round($_, 1) }) -join ', ') ms"
  # The first End and Home after the tab is shown again: the rows on the way have no
  # fitted name yet, and every frame of the glide fits the names it shows.
  $endMax = @(); $homeMax = @(); $endWork = @()
  for ($r = 1; $r -le 3; $r++) {
    BgOverTab $iSmall; BgAct 'B tab click away' $r { BgPress } 0 900 | Out-Null
    BgOverTab $iBig; BgAct 'B tab click, again' $r { BgPress } 4000 900 | Out-Null
    [G]::Click($BgX + 40, $BgY + 36 + 40); Start-Sleep -Milliseconds 400
    $a = BgAct 'B first End' $r { [G]::Press(0x23) } 0 900
    $endMax += $a.maxFrame
    if ($a.frames -gt 0) { $endWork += $a.work / $a.frames }
    $a = BgAct 'B first Home' $r { [G]::Press(0x24) } 0 900
    $homeMax += $a.maxFrame
  }
  ParkMouse
  $em = BgMedian $endMax; $hm = BgMedian $homeMax
  # No fix yet: the wrapped names of the icons still take a layout per halving. The bound
  # only keeps the slowest frame from getting worse than it is.
  Check 'B4 the first End and Home through 4000 files: the slowest frame stays under 14 ms of work (it is 8 to 10 ms; under 5 ms before the names were fitted when drawn)' ($em -gt 0 -and $em -lt 14 -and $hm -lt 14) "slowest frame: End $($endMax -join ', ') ms, Home $($homeMax -join ', ') ms; mean work per frame of End $(@($endWork | ForEach-Object { [math]::Round($_, 2) }) -join ', ') ms"
  Pf fence set $BgBig layout details | Out-Null
  BgQuiet (BgNow) 1500 30000
  $wide = @()
  for ($r = 1; $r -le 3; $r++) {
    $a = BgAct 'B details, one column wider' $r { Pf fence resize $BgSmall --w ($BgW + 80) | Out-Null } 0
    $wide += $a.worst
    BgAct 'B details, one column narrower' $r { Pf fence resize $BgSmall --w $BgW | Out-Null } 0 | Out-Null
  }
  $w = BgMedian $wide
  Check 'B1 详细信息 with 4000 files: 80 px wider keeps the fence under 100 ms (before: about 250 ms, every name fitted again)' ($w -lt 100) "longest message $([math]::Round($w, 1)) ms (runs $(@($wide | ForEach-Object { [math]::Round($_, 1) }) -join ', '))"
  Pf fence set $BgBig layout icons | Out-Null
  BgQuiet (BgNow) 1500 30000

  Case 'B2 a list change moves only the icons near the visible part'
  $new = Join-Path $big '000 新来的文件.txt'
  $work = @(); $far = @(); $moving = @()
  for ($r = 1; $r -le 3; $r++) {
    $pos = BgLogLen
    $a = BgAct 'B a file added' $r { [IO.File]::WriteAllText($new, 'new') } 4001 1500
    if ($a.frames -gt 0) { $work += $a.work / $a.frames }
    $line = [regex]::Match((BgLogFrom $pos), 'icons out of sight take their places at once moving=(\d+) far=(\d+)')
    if ($line.Success) { $moving += [int]$line.Groups[1].Value; $far += [int]$line.Groups[2].Value }
    BgAct 'B a file removed' $r { Remove-Item -LiteralPath $new -Force } 4000 1500 | Out-Null
  }
  Check 'B2 a file added at the top of 4000: at most 150 icons glide, the others take their places at once (before: 4000 glides)' ($moving.Count -eq 3 -and (BgMedian $moving) -le 150 -and (BgMedian $far) -ge 3800) "gliding $($moving -join ', '), at once $($far -join ', ')"
  $wf = BgMedian $work
  Check 'B2 the frames of that glide take under 2 ms of work each (before: about 2 ms with 4000 files, 3.3 ms with 8000)' ($wf -ge 0 -and $wf -lt 2.0) "mean work per frame $(@($work | ForEach-Object { [math]::Round($_, 2) }) -join ', ') ms"

  Case 'B3 a letter in the filter lists the folder once'
  $lists = @(); $sorts = @(); $busy = @()
  for ($r = 1; $r -le 3; $r++) {
    [G]::Click($BgX + 40, $BgY + 36 + 40); Start-Sleep -Milliseconds 300
    Chord 0x11 0x46; BgQuiet (BgNow) 700 8000
    $pos = BgLogLen
    $a = BgAct 'B filter: a letter typed' $r { [G]::Type('b') } 0 900
    $busy += $a.busy
    $text = BgLogFrom $pos
    $l = [regex]::Matches($text, 'item list built for the window[^\n]* sort_ms=([\d.]+)')
    $lists += $l.Count
    if ($l.Count -gt 0) { $sorts += [double]$l[0].Groups[1].Value }
    [G]::Press(0x1B); BgQuiet (BgNow) 900 8000
  }
  Check 'B3 a letter typed into the filter of 4000 files lists the folder once and keeps the fence under 28 ms (before: listed and matched three times, 39 to 47 ms)' ((BgMedian $lists) -eq 1 -and (BgMedian $busy) -lt 28) "listings per letter $($lists -join ', '); UI thread $(@($busy | ForEach-Object { [math]::Round($_, 1) }) -join ', ') ms"
  $sm = BgMedian $sorts
  Check 'B3 the name sort of 4000 files takes under 12 ms' ($sm -ge 0 -and $sm -lt 12) "sort $($sorts -join ', ') ms"

  Case 'B5 the search fades are set when the search changes, not at every draw'
  ParkMouse
  $fade = @(); $sets = @()
  for ($r = 1; $r -le 3; $r++) {
    BgAct 'B Peek opened' $r { PeekKey } 0 1200 | Out-Null
    $pos = BgLogLen
    $a = BgAct 'B Peek search: a letter typed' $r { [G]::Type('b') } 0 1200
    if ($a.frames -gt 0) { $fade += $a.work / $a.frames }
    $sets += [regex]::Matches((BgLogFrom $pos), 'search fades set for every item').Count
    BgAct 'B Peek closed' $r { [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B) } 0 1200 | Out-Null
  }
  $fm = BgMedian $fade
  Check 'B5 a Peek search letter with a fence of 4000 files on screen: the fades of the matches take under 2.6 ms of work per frame, and every item gets its fade set a few times per letter, not once per frame' ($fm -ge 0 -and $fm -lt 2.6 -and (BgMedian $sets) -ge 1 -and (BgMedian $sets) -le 4) "mean work per frame $(@($fade | ForEach-Object { [math]::Round($_, 2) }) -join ', ') ms; sets per letter $($sets -join ', ')"
  PfQuit
  [Anim]::Set($animWas)
  Remove-Item -Recurse -Force "$PfRun\bigdata" -ErrorAction SilentlyContinue
}

if ($Part -match 'M') { PartM }
if ($Part -match 'E') { PartE }
if ($Part -match 'B') { PartB }
if ($Part -match 'P') { PartP }
if ($Part -match 'R') { PartR }
