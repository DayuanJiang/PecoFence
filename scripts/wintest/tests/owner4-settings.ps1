# How soon the page of the settings window and of the smart fence editor arrives (owner round
# 2026-11-12, fourth batch). Each opening is timed from the moment the app takes the command:
#   the app's log      frame shown, WebView2 environment (first opening of a run), controller
#                      ready, navigation submitted, the page's `ready`, its `drawn`, the
#                      browser's window moved into the frame
#   the screen         a small region inside the window, looked at about every 30 ms (one look
#                      takes that long on a box): the first look that is not the desktop (the
#                      frame), the first look after the plain frame that shows something of the
#                      page, the last look that changed
#   the page (CDP)     a debug build only: when the document began, the first byte parsed,
#                      DOMContentLoaded, load, first paint, and the marks the page sets
# Parts ($Part, default FAE; $N openings each, default 10):
#   F  the first opening after the app started (the app is restarted for each)
#   A  opened again four seconds after closing, in one run of the app
#   E  the smart fence editor, 新建智能栅栏… from a fence's menu (real mouse)
#   I  opened again after ten minutes without a window (one opening; takes 11 minutes)
# and, on a debug build, looks into one load:
#   P  the page loaded again in the running browser, three times, with the browser's own
#      counters (script, style, layout) and a CPU profile of the last one
#   T  a trace of one such load (shots\reload-trace.jsonl)
#   S  the browser's own trace of one opening from its start (shots\startup-trace.json), and
#      the check that it sets up no font in search of a glyph: an icon that the icon font of
#      this Windows lacks, and the code point the page drew to find such icons, made it set up
#      Segoe MDL2 Assets, Noto Sans SC, Arial and Times New Roman in every new browser (14 ms
#      on an EC2 box, and 4 to 7 ms for the page's check); the host now puts the stand-ins
#      into the document (`icon_stand_ins`)
#   N  the fonts the page's text is drawn with, and which icons got their stand-in
# The rows go to shots\openings.tsv (the notes of the round have table.py for them). The one
# bound: the median of part A's `drawn` (the app's log) is at most $Bound ms (default 900: 669 ms
# before the change and 650 ms after it on an EC2 box, 758 ms the worst of ten).
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'FAE' }
if (-not (Get-Variable N -ErrorAction SilentlyContinue)) { $N = 10 }
if (-not (Get-Variable Bound -ErrorAction SilentlyContinue)) { $Bound = 900 }
$N = [int]$N; $Bound = [int]$Bound

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Text; using System.Drawing; using System.Drawing.Imaging;
using System.Diagnostics; using System.Runtime.InteropServices;
public static class Ow {
  [DllImport("kernel32.dll")] static extern void GetSystemTimePreciseAsFileTime(out long ft);
  // Milliseconds since 1970 (UTC), as the app's log and the page's performance.timeOrigin count.
  public static double NowMs() { long ft; GetSystemTimePreciseAsFileTime(out ft); return (ft - 116444736000000000L) / 10000.0; }
  static int Off(int[] a, int[] b, int colour, int thr) {
    int n = 0;
    for (int i = 0; i < a.Length; i++) {
      int p = a[i], q = b != null ? b[i] : colour;
      if (Math.Abs(((p >> 16) & 255) - ((q >> 16) & 255)) > thr || Math.Abs(((p >> 8) & 255) - ((q >> 8) & 255)) > thr || Math.Abs((p & 255) - (q & 255)) > thr) n++;
    }
    return n;
  }
  // Looks at a region for `ms` ms, with `every` ms of sleep between looks. One line per change
  // of what it shows: "<time> desk" (as the first look), "<time> plain" (all of it in the colour
  // `sheet`: the window's frame before its page), "<time> other"; then "<time> settled", the
  // last look that differed from the one before, and "<n> looks".
  public static string Watch(int x, int y, int w, int h, int ms, int every, int sheet) {
    var sb = new StringBuilder(); int[] first = null, prev = null; string kind = ""; double last = 0; int looks = 0;
    var sw = Stopwatch.StartNew();
    using (var b = new Bitmap(w, h, PixelFormat.Format32bppArgb)) using (var g = Graphics.FromImage(b)) {
      while (sw.ElapsedMilliseconds < ms) {
        g.CopyFromScreen(x, y, 0, 0, new Size(w, h));
        double now = NowMs(); looks++;
        var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        var a = new int[w * h]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d);
        if (first == null) first = a;
        string k = Off(a, first, 0, 6) <= a.Length / 200 ? "desk" : Off(a, null, sheet, 6) <= a.Length / 200 ? "plain" : "other";
        if (k != kind) { sb.Append(now.ToString("F1")).Append(' ').Append(k).Append('\n'); kind = k; }
        if (prev != null && Off(a, prev, 0, 2) > 0) last = now;
        prev = a;
        System.Threading.Thread.Sleep(every);
      }
    }
    sb.Append(last.ToString("F1")).Append(" settled\n").Append(looks).Append(" looks\n");
    return sb.ToString();
  }
}
'@

function SettingsWin() { return [G]::FindWindowW('PecoFence.Settings', [NullString]::Value) }
function EditorWinNow() { return [G]::FindWindowW('PecoFence.SmartEditor', [NullString]::Value) }
function CloseHost($cls) {
  $h = [G]::FindWindowW($cls, [NullString]::Value); $r = [IntPtr]::Zero
  if ($h -ne [IntPtr]::Zero) { [G]::SendMessageTimeoutW($h, 0x10, [IntPtr]0, [IntPtr]0, 2, 5000, [ref]$r) | Out-Null }
  for ($i = 0; $i -lt 50 -and [G]::FindWindowW($cls, [NullString]::Value) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 100 }
}
function LogLines() { $p = PfLogPath; if (Test-Path $p) { return @(Get-Content $p -Encoding UTF8) }; return @() }
# When a log line was written, in ms since 1970.
function LineMs($line) {
  if ("$line" -match '^(\d{4}-\d\d-\d\dT[\d:.]+Z)') {
    return ([DateTimeOffset]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture).UtcTicks - 621355968000000000) / 10000.0
  }
  return -1
}
function First($lines, $re) { return ($lines | Where-Object { $_ -match $re } | Select-Object -First 1) }
function MsOf($line) { if ("$line" -match ' ms=(?:Some\()?(\d+)') { return [int]$Matches[1] }; return -1 }
function Fences() { return @(@(PfJson fence list) | Where-Object { $_ -and $_.id -and $_.kind -ne 'inbox' }) }
function LastRow($m) { return (@(MenuRows $m) | Where-Object { $_.text } | Select-Object -Last 1) }
function Rel($t, $opened) { if ($t -gt 0 -and $opened -gt 0) { return [int][math]::Round($t - $opened) }; return -1 }

# What the page says about its own load, in ms after the document began (`origin` is that
# moment, in ms since 1970).
$script:PageTimes = @'
(() => {
  const t = performance.timing, s = t.navigationStart, p = {};
  for (const e of performance.getEntriesByType('paint')) p[e.name] = Math.round(e.startTime);
  const marks = {};
  for (const m of performance.getEntriesByType('mark')) marks[m.name] = Math.round(m.startTime);
  return { origin: performance.timeOrigin, parse: t.domLoading - s, interactive: t.domInteractive - s, dcl: t.domContentLoadedEventStart - s,
    dclEnd: t.domContentLoadedEventEnd - s, load: t.loadEventEnd - s, fp: p['first-paint'] ?? -1, fcp: p['first-contentful-paint'] ?? -1, marks };
})()
'@

$script:Cols = 'case', 'n', 'cmd', 'frame', 'env', 'controller', 'nav', 'ready', 'drawn', 'moved', 'scr_frame', 'scr_page', 'scr_settled', 'doc', 'parse', 'interactive', 'dcl', 'dclEnd', 'load', 'fp', 'fcp', 'marks'
$script:Rows = @()
# Reads one opening: the log lines after line $from, what the watch of the screen saw, the
# page's own times. $tCmd: when the command was given (ms since 1970).
function ReadOpening($case, $i, $from, $tCmd, $watch, [switch]$Editor) {
  $who = if ($Editor) { 'smart fence editor' } else { 'settings' }
  $lines = @()
  for ($k = 0; $k -lt 150; $k++) {
    $lines = @(LogLines | Select-Object -Skip $from)
    if (First $lines "${who}: page drawn") { break }
    Start-Sleep -Milliseconds 100
  }
  $frameLine = First $lines 'settings: window frame shown'
  $frame = MsOf $frameLine
  $opened = if ($frame -ge 0) { (LineMs $frameLine) - $frame } else { -1 }
  $envA = First $lines 'creating WebView2 environment'; $envB = First $lines 'WebView2 environment ready'
  $envMs = if ($envA -and $envB) { [int][math]::Round((LineMs $envB) - (LineMs $envA)) } else { 0 }
  $row = [ordered]@{ case = $case; n = $i; cmd = (Rel $opened $tCmd); frame = $frame; env = $envMs
    controller = (MsOf (First $lines 'WebView2 controller ready')); nav = (MsOf (First $lines 'page navigation submitted'))
    ready = (MsOf (First $lines "${who}: page ready")); drawn = (MsOf (First $lines "${who}: page drawn"))
    moved = (Rel (LineMs (First $lines 'moved into the frame')) $opened)
    scr_frame = -1; scr_page = -1; scr_settled = -1; doc = -1; parse = -1; interactive = -1; dcl = -1; dclEnd = -1; load = -1; fp = -1; fcp = -1; marks = '' }
  # The screen: the first look that is not the desktop, the first that shows something of the
  # page after the plain frame, the last that changed.
  $plain = $false
  foreach ($l in ($watch -split "`n")) {
    $t, $k = $l -split ' ', 2
    if ($k -eq 'looks') { continue }
    $ms = Rel ([double]$t) $opened
    if ($k -ne 'desk' -and $k -ne 'settled' -and $k -and $row.scr_frame -lt 0) { $row.scr_frame = $ms }
    if ($k -eq 'plain') { $plain = $true }
    if ($k -eq 'other' -and $plain -and $row.scr_page -lt 0) { $row.scr_page = $ms }
    if ($k -eq 'settled') { $row.scr_settled = $ms }
  }
  if ($PfDebugBuild -and $env:PECOFENCE_WEBVIEW_ARGS) {
    $ok = if ($Editor) { CdpOpen 9352 10 -Editor } else { CdpOpen 9352 10 }
    if ($ok) {
      $p = Js $script:PageTimes
      $row.doc = Rel $p.origin $opened
      foreach ($k in 'parse', 'interactive', 'dcl', 'dclEnd', 'load', 'fp', 'fcp') { $row[$k] = if ($p.$k -ge 0) { $row.doc + [int]$p.$k } else { -1 } }
      $row.marks = (@($p.marks.PSObject.Properties | ForEach-Object { "$($_.Name)=$($row.doc + [int]$_.Value)" }) -join ' ')
      try { $script:Cdp.Dispose() } catch { }; $script:Cdp = $null
    }
  }
  $script:Rows += [pscustomobject]$row
  ($script:Cols | ForEach-Object { $row[$_] }) -join "`t" | Add-Content "$PfShots\openings.tsv" -Encoding UTF8
  Log "opening $case ${i}: $(($script:Cols | ForEach-Object { "$_=$($row[$_])" }) -join ' '); screen: $(($watch -split "`n") -join ', ')"
  return $row
}
# Median and worst of a column over the rows of a case, as "median (worst)".
function Stat($case, $col) {
  $v = @($script:Rows | Where-Object { $_.case -eq $case -and $_.$col -ge 0 } | ForEach-Object { [int]$_.$col } | Sort-Object)
  if (-not $v.Count) { return '-' }
  return "$($v[[int][math]::Floor(($v.Count - 1) / 2)]) ($($v[-1]))"
}
function Median($case, $col) {
  $v = @($script:Rows | Where-Object { $_.case -eq $case -and $_.$col -ge 0 } | ForEach-Object { [int]$_.$col } | Sort-Object)
  if (-not $v.Count) { return -1 }
  return $v[[int][math]::Floor(($v.Count - 1) / 2)]
}
function Summary($case) {
  return "ms after the app took the command, median (worst) of $(@($script:Rows | Where-Object { $_.case -eq $case }).Count): " + (('frame', 'controller', 'nav', 'ready', 'drawn', 'moved', 'scr_frame', 'scr_page', 'scr_settled', 'doc', 'parse', 'dcl', 'fp') | ForEach-Object { "$_ $(Stat $case $_)" }) -join '; '
}
# Opens the settings with the command line and reads the opening.
function OpenSettingsOnce($case, $i, $watchMs) {
  $from = (LogLines).Count
  $tCmd = [Ow]::NowMs()
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'settings', 'open-ui' -WindowStyle Hidden | Out-Null
  $watch = [Ow]::Watch(700, 420, 240, 160, $watchMs, 5, 0x202020)
  $row = ReadOpening $case $i $from $tCmd $watch
  CloseHost 'PecoFence.Settings'
  return $row
}

Case 'setup'
CloseWindows
$script:Cols -join "`t" | Set-Content "$PfShots\openings.tsv" -Encoding UTF8
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
GreyWallpaper
PfStart -Lang zh-CN
Pf settings set theme dark | Out-Null
Start-Sleep -Seconds 2
ParkMouse
Check 'the app runs' (PfAlive) "debug build: $PfDebugBuild"

# ---- F: the first opening after the app started
function PartF() {
  trap { PfTrap $_; continue }
  Case 'F: the first opening after the app started'
  for ($i = 1; $i -le $N; $i++) {
    PfQuit; Start-Sleep -Seconds 2
    PfStart -Lang zh-CN
    Start-Sleep -Seconds 2
    ParkMouse
    # (The very first opening after the box started reads the browser from the disk.)
    $ms = if ($i -eq 1) { 9000 } else { 3500 }
    OpenSettingsOnce 'first' $i $ms | Out-Null
  }
  Check "F: $N first openings read" (@($script:Rows | Where-Object { $_.case -eq 'first' -and $_.drawn -ge 0 }).Count -eq $N) (Summary 'first')
}
if ($Part -match 'F') { PartF }

# ---- A: opened again a few seconds after closing
function PartA() {
  trap { PfTrap $_; continue }
  Case 'A: opened again four seconds after closing'
  # One opening that does not count: the run's first.
  Pf settings open-ui | Out-Null; Start-Sleep -Seconds 4; CloseHost 'PecoFence.Settings'
  for ($i = 1; $i -le $N; $i++) {
    Start-Sleep -Seconds 4
    OpenSettingsOnce 'again' $i 3000 | Out-Null
  }
  Check "A: $N openings read" (@($script:Rows | Where-Object { $_.case -eq 'again' -and $_.drawn -ge 0 }).Count -eq $N) (Summary 'again')
  $m = Median 'again' 'drawn'
  Check "A: the page has drawn within $Bound ms of the command (median)" ($m -ge 0 -and $m -le $Bound) "median $m ms; $(Summary 'again')"
  $f = Median 'again' 'frame'
  Check 'A: the frame is shown within 300 ms of the command (median, the app''s log)' ($f -ge 0 -and $f -le 300) "median $f ms; on screen $(Stat 'again' 'scr_frame')"
}
if ($Part -match 'A') { PartA }

# ---- E: the smart fence editor
function PartE() {
  trap { PfTrap $_; continue }
  Case 'E: the smart fence editor, 新建智能栅栏… from a fence''s menu'
  CloseHost 'PecoFence.Settings'
  if ((Fences).Count -lt 1) { Pf fence create --title '工作文档' --rect '1180,80,360,220' | Out-Null; Start-Sleep -Seconds 1 }
  for ($i = 1; $i -le $N; $i++) {
    Start-Sleep -Seconds 4
    $wr = (FenceGet (Fences)[0].id).windowRect
    [G]::RClick([int]($wr.x + $wr.w / 2), [int]($wr.y + 14)); Start-Sleep -Milliseconds 800
    $m = MenuWin 5
    $row = LastRow $m
    [G]::Click([int](($row.L + $row.R) / 2), [int](($row.T + $row.B) / 2)); Start-Sleep -Milliseconds 800
    $sub = @([W]::Tops('#32768')) | Where-Object { $_ -ne $m } | Select-Object -First 1
    $s = LastRow $sub
    [G]::MoveTo([int](($s.L + $s.R) / 2), [int](($s.T + $s.B) / 2)); Start-Sleep -Milliseconds 500
    $from = (LogLines).Count
    $tCmd = [Ow]::NowMs()
    [G]::Click([int](($s.L + $s.R) / 2), [int](($s.T + $s.B) / 2))
    $watch = [Ow]::Watch(400, 150, 300, 120, 3000, 5, 0x202020)
    ReadOpening 'editor' $i $from $tCmd $watch -Editor | Out-Null
    CloseHost 'PecoFence.SmartEditor'
  }
  Check "E: $N openings read" (@($script:Rows | Where-Object { $_.case -eq 'editor' -and $_.drawn -ge 0 }).Count -eq $N) (Summary 'editor')
}
if ($Part -match 'E') { PartE }

# ---- I: opened again after ten minutes without a window
function PartI() {
  trap { PfTrap $_; continue }
  Case 'I: opened again after ten minutes'
  Pf settings open-ui | Out-Null; Start-Sleep -Seconds 4; CloseHost 'PecoFence.Settings'
  Start-Sleep -Seconds 600
  OpenSettingsOnce 'idle' 1 5000 | Out-Null
  Check 'I: the opening after ten minutes read' (@($script:Rows | Where-Object { $_.case -eq 'idle' -and $_.drawn -ge 0 }).Count -eq 1) (Summary 'idle')
}
if ($Part -match 'I') { PartI }

# ---- P: the page loaded again in the running browser (debug build)
# One CDP call; the result object.
function CdpCall([string]$method, $params = @{}) {
  $script:CdpN++
  $id = $script:CdpN
  $body = @{ id = $id; method = $method; params = $params } | ConvertTo-Json -Depth 6 -Compress
  $bytes = [Text.Encoding]::UTF8.GetBytes($body)
  $none = [Threading.CancellationToken]::None
  $script:Cdp.SendAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $bytes)), [Net.WebSockets.WebSocketMessageType]::Text, $true, $none).Wait()
  while ($true) {
    $ms = New-Object IO.MemoryStream
    $buf = New-Object byte[] 65536
    do {
      $r = $script:Cdp.ReceiveAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $buf)), $none).Result
      $ms.Write($buf, 0, $r.Count)
    } while (-not $r.EndOfMessage)
    $text = [Text.Encoding]::UTF8.GetString($ms.ToArray())
    if ($text -match "^\{`"id`":$id,") { return $text }
  }
}
function Metrics() {
  $m = @{}
  foreach ($e in ((CdpCall 'Performance.getMetrics') | ConvertFrom-Json).result.metrics) { $m[$e.name] = $e.value }
  return $m
}
function PartP() {
  trap { PfTrap $_; continue }
  Case 'P: the page loaded again in the running browser'
  Check 'P: a debug build' $PfDebugBuild
  if (-not $PfDebugBuild) { return }
  Check 'P: the settings page is reached' (PfSettings)
  Start-Sleep -Seconds 2
  CdpCall 'Performance.enable' | Out-Null
  CdpCall 'Profiler.enable' | Out-Null
  CdpCall 'Profiler.setSamplingInterval' @{ interval = 200 } | Out-Null
  for ($i = 1; $i -le 3; $i++) {
    $a = Metrics
    if ($i -eq 3) { CdpCall 'Profiler.start' | Out-Null }
    CdpCall 'Page.reload' | Out-Null
    Start-Sleep -Milliseconds 300
    JsWait "!!document.documentElement && document.documentElement.classList.contains('ready')" 20 | Out-Null
    Start-Sleep -Milliseconds 600
    if ($i -eq 3) { CdpCall 'Profiler.stop' | Set-Content "$PfShots\reload-profile.json" -Encoding UTF8 }
    $b = Metrics
    $p = Js $script:PageTimes
    $d = ('ScriptDuration', 'V8CompileDuration', 'RecalcStyleDuration', 'LayoutDuration', 'TaskDuration') | ForEach-Object { "$_ $([int][math]::Round(1000 * ($b[$_] - $a[$_])))" }
    $line = "reload ${i}: parse $($p.parse) interactive $($p.interactive) dcl $($p.dcl) dclEnd $($p.dclEnd) load $($p.load) fp $($p.fp) fcp $($p.fcp) marks $(($p.marks.PSObject.Properties | ForEach-Object { "$($_.Name)=$($_.Value)" }) -join ' '); $($d -join ', '); nodes $($b['Nodes']) listeners $($b['JSEventListeners']) heap $([int]($b['JSHeapUsedSize'] / 1024)) KB"
    Log $line
    $line | Add-Content "$PfShots\reloads.txt" -Encoding UTF8
  }
  Check 'P: three reloads read' (Test-Path "$PfShots\reload-profile.json") (Get-Content "$PfShots\reloads.txt" -Raw)
  CloseHost 'PecoFence.Settings'
}
if ($Part -match 'P') { PartP }

# ---- T: a trace of the page loading again (debug build): what the browser's main thread does
# One message of the CDP connection, as text.
function CdpRead() {
  $none = [Threading.CancellationToken]::None
  $ms = New-Object IO.MemoryStream
  $buf = New-Object byte[] 65536
  do {
    $r = $script:Cdp.ReceiveAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $buf)), $none).Result
    $ms.Write($buf, 0, $r.Count)
  } while (-not $r.EndOfMessage)
  return [Text.Encoding]::UTF8.GetString($ms.ToArray())
}
function PartT() {
  trap { PfTrap $_; continue }
  Case 'T: a trace of the page loading again'
  Check 'T: a debug build' $PfDebugBuild
  if (-not $PfDebugBuild) { return }
  Check 'T: the settings page is reached' (PfSettings)
  Start-Sleep -Seconds 2
  CdpCall 'Tracing.start' @{ transferMode = 'ReportEvents'; traceConfig = @{ includedCategories = @('devtools.timeline', 'loading', 'blink.user_timing', 'v8.execute', 'disabled-by-default-devtools.timeline') } } | Out-Null
  CdpCall 'Page.reload' | Out-Null
  Start-Sleep -Milliseconds 1500
  CdpCall 'Tracing.end' | Out-Null
  $out = New-Object IO.StreamWriter("$PfShots\reload-trace.jsonl", $false, (New-Object Text.UTF8Encoding($false)))
  $n = 0
  for ($k = 0; $k -lt 5000; $k++) {
    $text = CdpRead
    if ($text -match '"method":"Tracing.dataCollected"') { $out.WriteLine($text); $n++ }
    if ($text -match '"method":"Tracing.tracingComplete"') { break }
  }
  $out.Close()
  Check 'T: the trace is saved' ($n -gt 0) "$n chunks"
  CloseHost 'PecoFence.Settings'
}
if ($Part -match 'T') { PartT }

# ---- S: the browser's own trace of an opening from its start (debug build): the browser is
# started with Chromium's startup tracing for three seconds and writes shots\startup-trace.json.
function PartS() {
  trap { PfTrap $_; continue }
  Case 'S: the browser''s trace of an opening'
  Check 'S: a debug build' $PfDebugBuild
  if (-not $PfDebugBuild) { return }
  $was = $env:PECOFENCE_WEBVIEW_ARGS
  $env:PECOFENCE_WEBVIEW_ARGS = "--remote-debugging-port=9352 --trace-startup=devtools.timeline,loading,blink,v8,navigation,fonts,disabled-by-default-devtools.timeline --trace-startup-file=$PfShots\startup-trace.json --trace-startup-duration=3 --trace-startup-format=json"
  PfQuit; Start-Sleep -Seconds 2
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 2
  # One opening before, so that the traced one is not the run's first.
  Pf settings open-ui | Out-Null; Start-Sleep -Seconds 6; CloseHost 'PecoFence.Settings'; Start-Sleep -Seconds 4
  Remove-Item "$PfShots\startup-trace.json" -ErrorAction SilentlyContinue
  OpenSettingsOnce 'traced' 1 5000 | Out-Null
  Start-Sleep -Seconds 4
  Check 'S: the trace is written' (Test-Path "$PfShots\startup-trace.json") "$((Get-Item "$PfShots\startup-trace.json" -ErrorAction SilentlyContinue).Length) bytes"
  # The font families the browser asked Windows for in that opening. For a glyph that no font
  # of the page's list has (an icon this Windows' icon font lacks, the code point the page drew
  # to find such icons) it went on through the system's fonts: Noto Sans SC, Arial and Times
  # New Roman on the box, 14 ms of setting up fonts nothing is drawn with.
  $trace = Get-Content "$PfShots\startup-trace.json" -Raw
  $asked = @([regex]::Matches($trace, '"family_name":\s*"([^"]*)"') | ForEach-Object { $_.Groups[1].Value } | Sort-Object -Unique)
  $named = 'Segoe UI Variable Text', 'Segoe UI Variable Display', 'Segoe UI', 'Microsoft YaHei UI', 'Microsoft YaHei', 'Segoe Fluent Icons', 'Segoe MDL2 Assets', 'Cascadia Mono', 'Consolas'
  $other = @($asked | Where-Object { $named -notcontains $_ })
  Check 'S: the browser sets up no font in search of a glyph (it asks Windows only for families the page names)' ($asked.Count -gt 0 -and $other.Count -eq 0) "asked for: $(($asked | ForEach-Object { "'$_'" }) -join ', '); not named by the page: $(($other | ForEach-Object { "'$_'" }) -join ', ')"
  $env:PECOFENCE_WEBVIEW_ARGS = $was
  PfQuit; Start-Sleep -Seconds 2
  PfStart -Lang zh-CN
}
if ($Part -match 'S') { PartS }

# ---- N: which fonts the page's text is drawn with (debug build): every font the first layout
# needs is read from disk and set up in each new browser, about 1 to 12 ms a font on the box.
function PartN() {
  trap { PfTrap $_; continue }
  Case 'N: the fonts of the page''s text'
  Check 'N: a debug build' $PfDebugBuild
  if (-not $PfDebugBuild) { return }
  Check 'N: the settings page is reached' (PfSettings)
  Start-Sleep -Seconds 1
  # The icons with a stand-in (data-alt): which of them this Windows' icon font lacks.
  $alts = Js "[...document.querySelectorAll('.ic[data-alt]')].map(el => (el.textContent === el.dataset.alt ? 'replaced ' : 'kept ') + el.dataset.alt.codePointAt(0).toString(16) + ' on ' + (el.closest('.page') ? el.closest('.page').id : 'nav'))"
  Log "icons with a stand-in: $($alts -join '; ')"
  $alts -join "`r`n" | Set-Content "$PfShots\icons.txt" -Encoding UTF8
  CdpCall 'DOM.enable' | Out-Null
  CdpCall 'CSS.enable' | Out-Null
  $root = ((CdpCall 'DOM.getDocument' @{ depth = 0 }) | ConvertFrom-Json).result.root.nodeId
  $ids = ((CdpCall 'DOM.querySelectorAll' @{ nodeId = $root; selector = 'body *' }) | ConvertFrom-Json).result.nodeIds
  $by = @{}
  foreach ($id in $ids) {
    $fonts = ((CdpCall 'CSS.getPlatformFontsForNode' @{ nodeId = $id }) | ConvertFrom-Json).result.fonts
    foreach ($f in @($fonts)) {
      if (-not $f) { continue }
      if (-not $by.ContainsKey($f.familyName)) { $by[$f.familyName] = @{ nodes = 0; glyphs = 0; first = '' } }
      $e = $by[$f.familyName]; $e.nodes++; $e.glyphs += $f.glyphCount
      if ($e.nodes -le 3) {
        $html = ((CdpCall 'DOM.getOuterHTML' @{ nodeId = $id }) | ConvertFrom-Json).result.outerHTML
        $e.first += " [$($f.glyphCount) glyphs] " + ($html.Substring(0, [math]::Min(160, $html.Length)) -replace '\s+', ' ')
      }
    }
  }
  $lines = @($by.Keys | Sort-Object | ForEach-Object { "$_`t$($by[$_].nodes) nodes`t$($by[$_].glyphs) glyphs`t$($by[$_].first)" })
  $lines | Set-Content "$PfShots\fonts.tsv" -Encoding UTF8
  Check 'N: the fonts are read' ($lines.Count -gt 0) "$(@($ids).Count) elements; $($by.Keys -join ', ')"
  CloseHost 'PecoFence.Settings'
}
if ($Part -match 'N') { PartN }

Case 'end'
CloseHost 'PecoFence.Settings'
CloseHost 'PecoFence.SmartEditor'
foreach ($f in @(Fences | Where-Object { $_.title -eq '工作文档' })) { Pf fence delete $f.id | Out-Null }
Check 'no window left' ((SettingsWin) -eq [IntPtr]::Zero -and (EditorWinNow) -eq [IntPtr]::Zero) ''
