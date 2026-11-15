# Issues round 2026-11-15, package hover. Every check states the fixed behaviour; on a build from
# before the fix (run.sh --bin <main's CI build>) a FAIL is the report showing.
#   run.sh --var Part=G   feedback #174: the title row of a Liquid Glass fence with 「鼠标悬停时显示」
#                         glides in and out under the pointer without the glass being rebuilt for
#                         every frame of the glide. Per-frame work by part from the app's
#                         render_timing trace lines, Liquid Glass against Fluent, and a hover
#                         expand of a rolled fence for comparison
#   run.sh --var Part=H   linux.do 2998063 #151: how long a rolled fence waits before it expands
#                         under a pointer that comes to rest on its title (two rolled fences, one
#                         8 px under the other, as in the report's picture). A pointer that came
#                         back while the fence was rolling up, and then rests, opens it too
#   run.sh --var Part=M   six such fences, each hovered once: afterwards no fence keeps the
#                         shadow bitmap of its glide (GDI objects and private bytes of the app)
#   run.sh --var Part=X   a rolled Liquid Glass fence with that title row opened by the CLI while
#                         the pointer is away: its title row folds while it opens, and the glass
#                         is never stretched far from the height drawn (a 1 px map stretched to
#                         the 48 px row showed a white glow); recorded for the report
#   run.sh --var Part=R   recordings of G's glide for the report (no checks)
$PfAllowLogErrors = $true
[Anim]::Set($true)
if (-not $Part) { $Part = 'GH' }

Add-Type @'
using System; using System.Diagnostics; using System.Runtime.InteropServices; using System.Threading;
public static class HoverWatch {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("winmm.dll")] static extern uint timeBeginPeriod(uint ms);
  [DllImport("winmm.dll")] static extern uint timeEndPeriod(uint ms);
  public static int Height(IntPtr h) { RECT r; GetWindowRect(h, out r); return r.B - r.T; }
  [DllImport("user32.dll")] static extern uint GetGuiResources(IntPtr process, uint flags);
  public static uint GdiObjects(IntPtr process) { return GetGuiResources(process, 0); }
  // Puts the pointer at (x, y) and returns the ms until the window is more than 2 px taller than
  // it was then (-1: not within timeoutMs). jitterMs > 0 moves the pointer 1 px back and forth
  // that often while waiting (a hand on the mouse).
  public static int MoveAndWait(IntPtr h, int x, int y, int timeoutMs, int jitterMs) {
    timeBeginPeriod(1);
    int start = Height(h);
    SetCursorPos(x, y);
    var clock = Stopwatch.StartNew(); long nextJitter = jitterMs; int side = 0;
    int result = -1;
    while (clock.ElapsedMilliseconds < timeoutMs) {
      if (Height(h) > start + 2) { result = (int)clock.ElapsedMilliseconds; break; }
      if (jitterMs > 0 && clock.ElapsedMilliseconds >= nextJitter) { side ^= 1; SetCursorPos(x + side, y); nextJitter += jitterMs; }
      Thread.Sleep(1);
    }
    timeEndPeriod(1);
    return result;
  }
  // Waits until window `shrinking` gets shorter than it is now (its roll-up started), then puts
  // the pointer at (x, y) and waits until window `watched` is more than 2 px taller than the
  // lowest it was seen after that. Returns "msToShrinkStart,msFromMoveToGrow" (-1 for a part
  // that did not happen within timeoutMs).
  public static string BackDuringRollUp(IntPtr shrinking, IntPtr watched, int x, int y, int timeoutMs) {
    timeBeginPeriod(1);
    int full = Height(shrinking);
    var clock = Stopwatch.StartNew();
    int shrinkAt = -1;
    while (clock.ElapsedMilliseconds < timeoutMs) {
      if (Height(shrinking) < full - 2) { shrinkAt = (int)clock.ElapsedMilliseconds; break; }
      Thread.Sleep(1);
    }
    int grow = -1;
    if (shrinkAt >= 0) {
      SetCursorPos(x, y);
      var since = Stopwatch.StartNew();
      int low = Height(watched);
      while (since.ElapsedMilliseconds < timeoutMs) {
        int now = Height(watched);
        if (now < low) low = now;
        if (now > low + 2) { grow = (int)since.ElapsedMilliseconds; break; }
        Thread.Sleep(1);
      }
    }
    timeEndPeriod(1);
    return shrinkAt + "," + grow;
  }
}
'@

function HvP($n) { return (Join-Path $PfDesk $n) }
function HvFence($title, $rect, $names) {
  foreach ($n in $names) { Set-Content -LiteralPath (HvP $n) "issues-hover $n" -Encoding UTF8 }
  Start-Sleep -Seconds 2
  Pf fence create --title $title --rect $rect | Out-Null
  Pf item move @($names | ForEach-Object { HvP $_ }) --to $title | Out-Null
  Start-Sleep -Seconds 2
}
function HvClean() {
  Get-ChildItem -LiteralPath $PfDesk -Filter 'hv *' -ErrorAction SilentlyContinue | Remove-Item -Force -ErrorAction SilentlyContinue
}
function HvRect($t) { return (FenceGet $t).windowRect }
function HvStart() {
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config" -ErrorAction SilentlyContinue
  PfStart -LogLevel 'pecofence=debug,pecofence::render_timing=trace'
  ClearFences | Out-Null
}

# The app's log lines between two times (UTC), as objects with the fields G needs.
function HvLines($from, $to) {
  $out = @()
  foreach ($line in @(PfLogAll)) {
    $t = LogTime $line
    if ($t -and $t -ge $from -and $t -le $to) { $out += $line }
  }
  return $out
}
function HvSum($lines, $regex, $field) {
  $s = 0.0; $n = 0
  foreach ($l in $lines) { if ($l -match $regex -and $l -match " $field=([\d.]+)") { $s += [double]$Matches[1]; $n++ } }
  return @($s, $n)
}
# Per-frame work of the animation runs between two times: frames, the UI work per frame (ms), the
# slowest frame, and from the trace lines per frame: chrome draw (ms), glass map builds, shadow (ms).
function HvWork($from, $to, $label) {
  $all = @(HvLines $from $to)
  $frames = 0; $work = 0.0; $max = 0.0; $runs = 0; $per = @()
  foreach ($l in $all) {
    if ($l -match 'animation run( stuttered)? frames=(\d+) total_ms=([\d.]+) max_gap_ms=([\d.]+) max_frame_ms=([\d.]+) timer_frames=(\d+) work_ms=([\d.]+)') {
      $runs++; $frames += [int]$Matches[2]; $work += [double]$Matches[7]; $max = [math]::Max($max, [double]$Matches[5])
      $per += [double]$Matches[7] / [math]::Max(1, [int]$Matches[2])
    }
  }
  # The middle run's work per frame: on a box without a GPU some runs wait 20 ms per frame for
  # the software rasterizer (BeginDraw), with either build and either style.
  $per = @($per | Sort-Object)
  $median = if ($per.Count) { [math]::Round($per[[int][math]::Floor($per.Count / 2)], 2) } else { -1 }
  $chrome = 0.0
  foreach ($l in $all) {
    if ($l -match 'surface draw' -and $l -match ' calls_us=(\d+)' ) { $c = [double]$Matches[1]; if ($l -match ' end_us=(\d+)') { $c += [double]$Matches[1] }; $chrome += $c }
  }
  $maps = @($all | Where-Object { $_ -match 'glass map built' })
  $mapUs = (HvSum $maps 'glass map built' 'us')[0]
  $shadowUs = 0.0; $shadows = 0; $bitmapUs = 0.0
  foreach ($l in $all) {
    if ($l -match 'shadow stages' -and $l -match ' make_us=(\d+) copy_us=(\d+) present_us=(\d+)') { $shadowUs += [double]$Matches[1] + [double]$Matches[2] + [double]$Matches[3]; $bitmapUs += [double]$Matches[1] + [double]$Matches[2]; $shadows++ }
  }
  $f = [math]::Max(1, $frames)
  $r = [pscustomobject]@{
    label = $label; runs = $runs; frames = $frames
    perFrame = [math]::Round($work / $f, 2); maxFrame = $max; median = $median
    chromePerFrame = [math]::Round($chrome / 1000 / $f, 2)
    maps = $maps.Count; mapMsPerBuild = $(if ($maps.Count) { [math]::Round($mapUs / 1000 / $maps.Count, 2) } else { 0 })
    shadows = $shadows; shadowPerFrame = [math]::Round($shadowUs / 1000 / $f, 2)
    shadowBitmapPerFrame = [math]::Round($bitmapUs / 1000 / $f, 2)
  }
  Log ("G {0}: {1} runs, {2} frames, UI work {3} ms per frame (middle run {10} ms, slowest frame {4} ms); surface draws {5} ms per frame; glass map built {6} times ({7} ms each); shadow updates {8}, {9} ms per frame, of it making and copying the bitmap {11} ms" -f $r.label, $r.runs, $r.frames, $r.perFrame, $r.maxFrame, $r.chromePerFrame, $r.maps, $r.mapMsPerBuild, $r.shadows, $r.shadowPerFrame, $r.median, $r.shadowBitmapPerFrame)
  return $r
}

# Pointer in and out of the fence $n times; returns the time span (UTC) it took.
function HvHoverInOut($t, $n) {
  $wr = HvRect $t
  $in = @([int]($wr.x + $wr.w / 2), [int]($wr.y + $wr.h / 2))
  ParkMouse 1300 760; Start-Sleep -Milliseconds 900
  $from = (Get-Date).ToUniversalTime()
  for ($i = 0; $i -lt $n; $i++) {
    Glide 1300 760 $in[0] $in[1] 8 15; Start-Sleep -Milliseconds 700
    Glide $in[0] $in[1] 1300 760 8 15; Start-Sleep -Milliseconds 700
  }
  $to = (Get-Date).ToUniversalTime()
  return @($from, $to)
}

if ($Part -match 'G') {
  Case 'G: setup, a fence with a title row that shows only under the pointer'
  HvClean
  HvStart
  Pf settings set theme light | Out-Null
  $names = @(1..12 | ForEach-Object { "hv $_ 会议纪要" })
  HvFence 'HvGlass' '360,160,440,380' $names
  Pf fence set HvGlass titleOnHover hover | Out-Null
  $res = @{}
  foreach ($style in 'liquidGlass', 'fluent') {
    Pf settings set themeStyle $style | Out-Null
    Start-Sleep -Seconds 3
    $span = HvHoverInOut 'HvGlass' 10
    # The app writes its log in blocks: wait for them before reading.
    Start-Sleep -Seconds 2
    $res["$style title"] = $span
    # A rolled fence that the resting pointer expands (its title row always shows).
    Pf fence set HvGlass titleOnHover always | Out-Null
    Pf fence roll HvGlass | Out-Null; Start-Sleep -Milliseconds 1200
    $wr = HvRect 'HvGlass'
    $from = (Get-Date).ToUniversalTime()
    for ($i = 0; $i -lt 6; $i++) {
      ParkMouse 1300 760; Start-Sleep -Milliseconds 1500
      [G]::MoveTo([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 1300
    }
    ParkMouse 1300 760; Start-Sleep -Milliseconds 1500
    $res["$style roll"] = @($from, (Get-Date).ToUniversalTime())
    Pf fence unroll HvGlass | Out-Null; Start-Sleep -Milliseconds 1000
    Pf fence set HvGlass titleOnHover hover | Out-Null; Start-Sleep -Milliseconds 1000
  }
  PfQuit
  $w = @{}
  foreach ($k in 'liquidGlass title', 'fluent title', 'liquidGlass roll', 'fluent roll') { $w[$k] = HvWork $res[$k][0] $res[$k][1] $k }
  $g = $w['liquidGlass title']; $f = $w['fluent title']; $gr = $w['liquidGlass roll']
  $d = "Liquid Glass: $($g.frames) frames in $($g.runs) runs, $($g.perFrame) ms of work per frame (middle run $($g.median) ms, slowest frame $($g.maxFrame) ms), surface draws $($g.chromePerFrame) ms per frame, shadow $($g.shadowPerFrame) ms per frame (bitmap $($g.shadowBitmapPerFrame) ms), glass map built $($g.maps) times ($($g.mapMsPerBuild) ms each); Fluent: $($f.perFrame) ms per frame (middle run $($f.median) ms, slowest $($f.maxFrame) ms), surface draws $($f.chromePerFrame), shadow $($f.shadowPerFrame) (bitmap $($f.shadowBitmapPerFrame)); rolled expand with Liquid Glass: $($gr.perFrame) ms per frame (middle run $($gr.median) ms), map built $($gr.maps) times in $($gr.frames) frames"
  Check 'G: the steps saw the title row glide in and out 20 times with Liquid Glass' ($g.runs -ge 15 -and $g.frames -ge 100) $d
  Check 'G: the glass of the plate is built once per glide (for the height it ends at), not for every frame' ($g.runs -ge 15 -and $g.maps -le [math]::Ceiling($g.frames / 3)) $d
  Check 'G: a frame of the glide costs about what it costs with Fluent (middle run at most 1.5 times Fluent''s, and under 6.9 ms, a frame at 144 Hz)' ($g.runs -ge 15 -and $f.median -gt 0 -and $g.median -le 1.5 * $f.median -and $g.median -lt 6.9) $d
  Check 'G: the shadow bitmap is not made again for every frame of the glide (under 0.15 ms per frame to make and copy it)' ($g.runs -ge 15 -and $g.shadowBitmapPerFrame -lt 0.15) $d
  HvClean
}

if ($Part -match 'H') {
  Case 'H: setup, two rolled fences, one 8 px under the other'
  HvClean
  HvStart
  # --var Speed=fast / Push=off: the same cases with 「展开快慢」 快 and without the push.
  if ($Speed) { Pf settings set rollUp.hoverSpeed $Speed | Out-Null }
  if ($Push -eq 'off') { Pf settings set rollUp.pushNeighbors false | Out-Null }
  $hoverMs = if ($Speed -eq 'fast') { 150 } else { 400 }
  $runs = if ($Runs) { [int]$Runs } else { 10 }
  HvFence 'HvTop' '520,140,320,260' @('hv a 报告', 'hv b 预算表', 'hv c 会议纪要', 'hv d 分工')
  Pf fence roll HvTop | Out-Null; Start-Sleep -Milliseconds 1200
  $topH = [G]::FindWindowW('PecoFence.Fence', 'HvTop')
  $tr = Rect $topH
  HvFence 'HvLow' "520,$($tr.B + 8),320,220" @('hv e 照片', 'hv f 文档', 'hv g 下载')
  Pf fence roll HvLow | Out-Null; Start-Sleep -Milliseconds 1200
  $lowH = [G]::FindWindowW('PecoFence.Fence', 'HvLow')
  $lr = Rect $lowH
  $rolledH = $tr.B - $tr.T
  Check 'H: both fences are rolled up, the second one 8 px under the first' ($topH -ne [IntPtr]::Zero -and $lowH -ne [IntPtr]::Zero -and $lr.T -eq $tr.B + 8 -and ($lr.B - $lr.T) -eq $rolledH) "top $($tr.T)..$($tr.B), low $($lr.T)..$($lr.B)"
  $topPt = @([int](($tr.L + $tr.R) / 2), [int]($tr.T + $rolledH / 2))
  $lowPt = @([int](($lr.L + $lr.R) / 2), [int]($lr.T + $rolledH / 2))
  # Rests the pointer away from both until the short wait is over and both are rolled up again.
  function HvAway() {
    [G]::MoveTo(1300, 760)
    for ($k = 0; $k -lt 40 -and ((([HoverWatch]::Height($topH)) -gt $rolledH) -or (([HoverWatch]::Height($lowH)) -gt $rolledH)); $k++) { Start-Sleep -Milliseconds 100 }
    Start-Sleep -Milliseconds 1600
  }
  $cases = [ordered]@{ c1 = @(); c2 = @(); c3 = @(); c4 = @(); c5 = @(); c6 = @() }
  for ($n = 1; $n -le $runs; $n++) {
    # 1: from far away onto the first one's title, the pointer rests there.
    HvAway
    $cases['c1'] += [HoverWatch]::MoveAndWait($topH, $topPt[0], $topPt[1], 3000, 0)
    # 2: the first one is open; within 0.3 s onto the second one's title (pushed down now).
    Start-Sleep -Milliseconds 250
    $lowNow = Rect $lowH
    $cases['c2'] += [HoverWatch]::MoveAndWait($lowH, $lowPt[0], [int]($lowNow.T + $rolledH / 2), 3000, 0)
    # 3: away for 2 s, then onto the second one's title.
    HvAway
    $cases['c3'] += [HoverWatch]::MoveAndWait($lowH, $lowPt[0], $lowPt[1], 3000, 0)
    # 4: the first one open under the pointer; the pointer leaves, and while it rolls up the
    # pointer comes back to its title and rests there.
    HvAway
    if ([HoverWatch]::MoveAndWait($topH, $topPt[0], $topPt[1], 3000, 0) -ge 0) {
      Start-Sleep -Milliseconds 500
      [G]::MoveTo(1300, 760)
      $r4 = [HoverWatch]::BackDuringRollUp($topH, $topH, $topPt[0], $topPt[1], 3000)
      $cases['c4'] += [int]($r4 -split ',')[1]
      Log "H case 4 run ${n}: $r4"
    } else { $cases['c4'] += -2 }
    # 5: the same, but the pointer comes to rest on the second one's title.
    HvAway
    if ([HoverWatch]::MoveAndWait($topH, $topPt[0], $topPt[1], 3000, 0) -ge 0) {
      Start-Sleep -Milliseconds 500
      [G]::MoveTo(1300, 760)
      $r5 = [HoverWatch]::BackDuringRollUp($topH, $lowH, $lowPt[0], $lowPt[1], 3000)
      $cases['c5'] += [int]($r5 -split ',')[1]
      Log "H case 5 run ${n}: $r5"
    } else { $cases['c5'] += -2 }
    # 6: from far away onto the first one's title, then the hand trembles 1 px every 50 ms.
    HvAway
    $cases['c6'] += [HoverWatch]::MoveAndWait($topH, $topPt[0], $topPt[1], 3000, 50)
  }
  HvAway
  $txt = @(foreach ($k in $cases.Keys) { "case $($k.Substring(1)): $($cases[$k] -join ' ')" }) -join '; '
  Log "H waits in ms (-1: did not expand within 3 s): $txt"
  PfQuit
  $armed = @(PfLogAll | Where-Object { $_ -match 'hover expand armed' }).Count
  function Ok($list, $lo, $hi) { return (@($list | Where-Object { $_ -lt $lo -or $_ -gt $hi }).Count -eq 0) }
  $lo = $hoverMs - 100; $hi = $hoverMs + 300
  Check "H 1: from far away, the first rolled fence opens after the hover time ($lo to $hi ms)" (Ok $cases['c1'] $lo $hi) $txt
  Check 'H 2: right after it, the rolled fence below opens after the short wait (under 0.25 s)' (Ok $cases['c2'] 0 250) $txt
  Check "H 3: after 2 s away, the rolled fence below waits the hover time again ($lo to $hi ms)" (Ok $cases['c3'] $lo $hi) $txt
  Check 'H 4: a pointer that came back to the title while the fence rolled up, and rests there, opens it again (within 0.9 s)' (Ok $cases['c4'] 0 900) "$txt; hover expand armed lines: $armed"
  Check 'H 5: a pointer that rests on the title below while the fence above rolls up opens that one (within 0.9 s)' (Ok $cases['c5'] 0 900) "$txt; hover expand armed lines: $armed"
  Check "H 6: a trembling hand does not restart the wait ($lo to $hi ms)" (Ok $cases['c6'] $lo $hi) $txt
  HvClean
}

if ($Part -match 'L') {
  # The glass at rest, for a pixel comparison between two builds on the dev box (no checks):
  # the title row folded away, and shown under the pointer.
  Case 'L: the plate at rest, title row folded and shown'
  HvClean
  HvStart
  Pf settings set theme light | Out-Null
  Pf settings set themeStyle liquidGlass | Out-Null
  $names = @(1..12 | ForEach-Object { "hv $_ 会议纪要" })
  HvFence 'HvGlass' '360,160,440,380' $names
  Pf fence set HvGlass titleOnHover hover | Out-Null
  Start-Sleep -Seconds 3
  $wr = HvRect 'HvGlass'
  ParkMouse 1300 760; Start-Sleep -Milliseconds 1500
  [Px]::Grab($wr.x - 20, $wr.y - 20, $wr.w + 40, $wr.h + 40).Save("$PfShots\L-folded.png")
  # The pointer on the content's empty corner, clear of every icon.
  [G]::MoveTo([int]($wr.x + $wr.w - 12), [int]($wr.y + $wr.h - 12)); Start-Sleep -Milliseconds 1500
  [Px]::Grab($wr.x - 20, $wr.y - 20, $wr.w + 40, $wr.h + 40).Save("$PfShots\L-shown.png")
  ParkMouse 1300 760
  PfQuit
  HvClean
}

if ($Part -match 'M') {
  Case 'M: six fences with the title row on hover, each hovered once'
  HvClean
  HvStart
  Pf settings set theme light | Out-Null
  Pf settings set themeStyle liquidGlass | Out-Null
  $k = 0
  foreach ($row in 0, 1) {
    foreach ($col in 0, 1, 2) {
      $k++
      HvFence "HvMem$k" "$(60 + $col * 500),$(60 + $row * 400),440,340" @("hv m$k 报告", "hv m$k 预算表")
    }
  }
  ParkMouse 1560 860; Start-Sleep -Seconds 4
  function HvMem() {
    $p = Get-Process pecofence | Select-Object -First 1
    $p.Refresh()
    return @([int][HoverWatch]::GdiObjects($p.Handle), [math]::Round($p.PrivateMemorySize64 / 1MB, 2))
  }
  # Before any title row glided: the title rows show, nothing has been kept for a glide yet.
  $m0 = HvMem
  # Each title row folds away (one glide each), then comes back under the pointer and folds again.
  for ($i = 1; $i -le 6; $i++) { Pf fence set "HvMem$i" titleOnHover hover | Out-Null }
  Start-Sleep -Seconds 1
  for ($i = 1; $i -le 6; $i++) {
    $wr = HvRect "HvMem$i"
    [G]::MoveTo([int]($wr.x + $wr.w - 20), [int]($wr.y + $wr.h - 20)); Start-Sleep -Milliseconds 800
    ParkMouse 1560 860; Start-Sleep -Milliseconds 800
  }
  Start-Sleep -Seconds 3
  $m1 = HvMem
  $d = "GDI objects $($m0[0]) before the title rows glided, $($m1[0]) after; private bytes $($m0[1]) MB before, $($m1[1]) MB after"
  Log "M $d"
  Check 'M: after the glides the fences keep no shadow bitmaps of them (at most 4 more GDI objects, 12 with one bitmap and DC per fence)' ($m1[0] -le $m0[0] + 4) $d
  Check 'M: and private bytes grow by under 1.5 MB (about 0.8 MB per fence with the bitmaps kept)' ($m1[1] - $m0[1] -lt 1.5) $d
  PfQuit
  HvClean
}

if ($Part -match 'X') {
  . C:\pf\kit\rec.ps1
  Case 'X: a rolled Liquid Glass fence with the title row on hover, opened by the CLI'
  HvClean
  HvStart
  Pf settings set theme light | Out-Null
  Pf settings set themeStyle liquidGlass | Out-Null
  HvFence 'HvOpen' '420,200,440,340' @(1..8 | ForEach-Object { "hv x$_ 会议纪要" })
  Pf fence set HvOpen titleOnHover hover | Out-Null
  Pf fence roll HvOpen | Out-Null
  ParkMouse 1500 840; Start-Sleep -Seconds 3
  $wr = HvRect 'HvOpen'
  $from = (Get-Date).ToUniversalTime()
  for ($n = 1; $n -le 3; $n++) {
    RecStart "open-$n" -X ($wr.x - 30) -Y ($wr.y - 30) -W ($wr.w + 60) -H 440 -Fps 30 -InMemory
    Start-Sleep -Milliseconds 300
    RecMark '用命令行展开收起的栅栏（鼠标不在上面）'
    Pf fence unroll HvOpen | Out-Null
    Start-Sleep -Milliseconds 900
    RecStop
    Pf fence roll HvOpen | Out-Null; Start-Sleep -Milliseconds 1500
  }
  $to = (Get-Date).ToUniversalTime()
  PfQuit
  $maps = @(HvLines $from $to | Where-Object { $_ -match 'glass map built w=\d+ h=(\d+) drawn_h=(\d+)' })
  $far = @($maps | Where-Object { $_ -match ' h=(\d+) drawn_h=(\d+)' -and ([double]$Matches[2] -gt 1.4 * [double]$Matches[1] -or [double]$Matches[1] -gt 1.4 * [double]$Matches[2]) })
  $d = "$($maps.Count) glass builds while the fence opened 3 times; built far from the height drawn: $($far.Count)$(if ($far.Count) { '; e.g. ' + (($far | Select-Object -First 3 | ForEach-Object { $_ -replace '^.*glass map built ', '' }) -join ' / ') })"
  Log "X $d"
  Check 'X: the glass is never built for a height far from the one drawn (no 1 px map stretched over the title row)' ($maps.Count -ge 3 -and $far.Count -eq 0) $d
  HvClean
}

if ($Part -match 'R') {
  . C:\pf\kit\rec.ps1
  Case 'R: recordings of the title row glide with Liquid Glass'
  HvClean
  HvStart
  Pf settings set theme light | Out-Null
  Pf settings set themeStyle liquidGlass | Out-Null
  $names = @(1..12 | ForEach-Object { "hv $_ 会议纪要" })
  HvFence 'HvGlass' '360,160,440,380' $names
  Pf fence set HvGlass titleOnHover hover | Out-Null
  Start-Sleep -Seconds 3
  $wr = HvRect 'HvGlass'
  $in = @([int]($wr.x + $wr.w / 2), [int]($wr.y + $wr.h / 2))
  ParkMouse 1300 760; Start-Sleep -Milliseconds 900
  RecStart 'glide' -X ($wr.x - 40) -Y ($wr.y - 40) -W ($wr.w + 80) -H ($wr.h + 80) -Fps 30 -InMemory
  for ($i = 0; $i -lt 3; $i++) {
    RecMark '鼠标移进栅栏'
    Glide 1300 760 $in[0] $in[1] 8 15; Start-Sleep -Milliseconds 800
    RecMark '鼠标移出栅栏'
    Glide $in[0] $in[1] 1300 760 8 15; Start-Sleep -Milliseconds 800
  }
  RecStop
  PfQuit
  HvClean
}
