# Owner round 2026-11-12, latency: how long PecoFence takes to answer what the user does.
# One part per call:
#   --var Part=TAB    the hover tab switch (「鼠标悬停时切换标签页」): from the pointer on another tab to
#                     the first picture of the new tab and to the complete one, with 默认 and 快, on a
#                     tab shown before and on one never shown in this run, with 5 and with 60 icons
#   --var Part=NEXT   the tab after a hover switch follows the pointer after a short wait (the check
#                     that fails before the change), and sweeps along five tabs
#   --var Part=REC --var Rec=before|after   the sweep recorded for the before/after clip
# How the times are taken: [Lat]::Watch grabs a small screen region as fast as the box allows
# (`per` ms per grab, in every line), sends the input and notes the first grab that differs from
# the picture before the input (`first`), the first grab that equals the picture of the finished
# answer where one was taken beforehand (`shown`), and the last grab that differs from the one
# before it (`last`). A time is the end of its grab, so it is at most one grab late. The app's
# debug log lines of the same moment (`+ms` after the input) are written beside them.
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'TAB' }

Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
public static class Lat {
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  public static int X, Y, W, H;
  public static List<int[]> Refs = new List<int[]>();
  public static List<int[]> Kept = new List<int[]>();
  static int[] Grab() {
    using (var b = new Bitmap(W, H, PixelFormat.Format32bppRgb)) {
      using (var g = Graphics.FromImage(b)) g.CopyFromScreen(X, Y, 0, 0, new Size(W, H));
      var d = b.LockBits(new Rectangle(0, 0, W, H), ImageLockMode.ReadOnly, PixelFormat.Format32bppRgb);
      var a = new int[W * H]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
    }
  }
  // Pixels that differ by more than 6 in a channel.
  static int Differ(int[] a, int[] b) {
    int n = 0;
    for (int i = 0; i < a.Length; i++) {
      int p = a[i], q = b[i];
      if (p == q) continue;
      if (Math.Abs((p & 255) - (q & 255)) > 6 || Math.Abs(((p >> 8) & 255) - ((q >> 8) & 255)) > 6 || Math.Abs(((p >> 16) & 255) - ((q >> 16) & 255)) > 6) n++;
    }
    return n;
  }
  public static void Region(int x, int y, int w, int h) { X = x; Y = y; W = w; H = h; Refs.Clear(); }
  // The region as it is now, as the picture of a finished answer; returns its number.
  public static int AddRef() { Refs.Add(Grab()); return Refs.Count - 1; }
  static void MoveTo(int x, int y) {
    int w = GetSystemMetrics(0), h = GetSystemMetrics(1);
    mouse_event(0x8001, x * 65535 / (w - 1), y * 65535 / (h - 1), 0, UIntPtr.Zero);
  }
  // Plays pointer moves (triples: at ms, x, y) without grabbing, for a recording.
  public static void Play(int[] moves, int ms) {
    var sw = Stopwatch.StartNew(); int m = 0;
    while (sw.ElapsedMilliseconds < ms) {
      while (m + 2 < moves.Length && sw.ElapsedMilliseconds >= moves[m]) { MoveTo(moves[m + 1], moves[m + 2]); m += 3; }
      System.Threading.Thread.Sleep(1);
    }
  }
  // Grabs the region, runs `act` (may be null) and the pointer moves (triples: at ms, x, y; the
  // clock starts just before the first input), runs `later` at laterMs, and watches for ms.
  // "utc=<ticks at the input> first= last= grabs= per= seen=<ms:ref ...> changes=<ms:pixels ...>"
  public static string Watch(int ms, Action act, int[] moves, Action later, int laterMs, int min) {
    Kept.Clear();
    var baseline = Grab(); var prev = baseline; Kept.Add(baseline);
    long first = -1, last = -1; int n = 0, m = 0, seen = -2;
    var changes = new StringBuilder(); var seenAt = new StringBuilder();
    long utc = DateTime.UtcNow.Ticks;
    var sw = Stopwatch.StartNew();
    if (act != null) act();
    while (sw.ElapsedMilliseconds < ms) {
      while (moves != null && m + 2 < moves.Length && sw.ElapsedMilliseconds >= moves[m]) { MoveTo(moves[m + 1], moves[m + 2]); m += 3; }
      if (later != null && sw.ElapsedMilliseconds >= laterMs) { later(); later = null; }
      var cur = Grab(); long at = sw.ElapsedMilliseconds; n++;
      int dPrev = Differ(cur, prev);
      if (first < 0 && Differ(cur, baseline) >= min) first = at;
      if (dPrev >= min) { last = at; changes.Append(at + ":" + dPrev + " "); if (Kept.Count < 40) Kept.Add(cur); }
      if (Refs.Count > 0 && (dPrev >= min || seen == -2)) {
        int now = -1;
        for (int r = 0; r < Refs.Count; r++) if (Differ(cur, Refs[r]) < min) { now = r; break; }
        if (now != seen) { seen = now; seenAt.Append(at + ":" + now + " "); }
      }
      prev = cur;
    }
    return "utc=" + utc + " first=" + first + " last=" + last + " grabs=" + n + " per=" + (ms / Math.Max(1, n))
      + " seen=" + seenAt.ToString().Trim().Replace(' ', ',') + " changes=" + changes.ToString().Trim().Replace(' ', ',');
  }
  // The pictures that changed during the last Watch (the first is the one before the input), in a row.
  public static void Strip(string path, int max) {
    int k = Math.Min(max, Kept.Count); if (k == 0) return;
    using (var o = new Bitmap(k * (W + 2), H, PixelFormat.Format32bppRgb)) {
      using (var g = Graphics.FromImage(o)) g.Clear(Color.Magenta);
      for (int i = 0; i < k; i++) {
        var d = o.LockBits(new Rectangle(i * (W + 2), 0, W, H), ImageLockMode.WriteOnly, PixelFormat.Format32bppRgb);
        for (int y = 0; y < H; y++) Marshal.Copy(Kept[i], y * W, d.Scan0 + y * d.Stride, W);
        o.UnlockBits(d);
      }
      o.Save(path, ImageFormat.Png);
    }
  }
}
'@

# ---- measuring
# One watch as an object: first, last, per, grabs, utc, seen (the string), raw. $moves: pointer
# moves as triples (at ms, x, y); $act / $later: script blocks for other input.
function LatWatch([int]$ms, $moves = $null, [scriptblock]$act = $null, [scriptblock]$later = $null, [int]$laterMs = 0, [int]$min = 6) {
  $a = $null; if ($act) { $a = [Action]$act }
  $l = $null; if ($later) { $l = [Action]$later }
  $mv = $null; if ($moves) { $mv = [int[]]$moves }
  $s = [Lat]::Watch($ms, $a, $mv, $l, $laterMs, $min)
  $o = @{ raw = $s; seen = '' }
  foreach ($k in 'first', 'last', 'grabs', 'per') { if ($s -match "\b$k=(-?\d+)") { $o[$k] = [int]$Matches[1] } }
  if ($s -match '\butc=(\d+)') { $o.utc = [long]$Matches[1] }
  if ($s -match '\bseen=(\S*)') { $o.seen = $Matches[1] }
  Log "lat: $s"
  return $o
}
# The ms at which the picture of answer $ref was first seen whole in a watch (-1: never).
function SeenAt($r, $ref) { foreach ($p in @($r.seen -split ',')) { $q = $p -split ':'; if ($q.Count -eq 2 -and [int]$q[1] -eq $ref) { return [int]$q[0] } }; return -1 }
# The answers seen whole during a watch, in order ("1 2 4").
function SeenList($r) { return (@($r.seen -split ',' | ForEach-Object { ($_ -split ':')[1] } | Where-Object { $_ -ne $null -and $_ -ne '-1' -and $_ -ne '' }) -join ' ') }
function Med($a) { $s = @($a | Sort-Object); if ($s.Count -eq 0) { return -1 }; return $s[[int][math]::Floor(($s.Count - 1) / 2)] }
function Worst($a) { $s = @($a | Sort-Object); if ($s.Count -eq 0) { return -1 }; return $s[-1] }
# The app log: where it ends now, and the lines after that point matching $regex as
# "+<ms after the input at $utc> text".
function LogPos() { $p = PfLogPath; if (-not (Test-Path $p)) { return 0 }; $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $n = $fs.Length; $fs.Close(); return $n }
function LogRel($pos, $utc, $regex) {
  $p = PfLogPath; if (-not (Test-Path $p)) { return @() }
  $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $fs.Position = $pos
  $sr = New-Object IO.StreamReader($fs, [Text.Encoding]::UTF8); $t = $sr.ReadToEnd(); $sr.Close()
  $out = @()
  foreach ($l in @($t -split "`r?`n")) {
    if ($l -notmatch $regex) { continue }
    if ($l -match '^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z\s+\w+\s+(.*)$') {
      $ts = [DateTime]::Parse($Matches[1] + 'Z', [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AdjustToUniversal)
      $out += "+$([int](($ts.Ticks - $utc) / 10000)) $($Matches[2] -replace '^pecofence::\S+ ', '')"
    }
  }
  return $out
}
# One line of the result table (shots\latency.tsv comes back with the job).
function LatRow($action, $case, $first, $done, $per, $note) {
  New-Item -ItemType Directory -Force $PfShots | Out-Null
  "$Part`t$action`t$case`t$first`t$done`t$per`t$note" | Add-Content "$PfShots\latency.tsv" -Encoding UTF8
  Log "row: $action | $case | first=$first done=$done per=$per | $note"
}

# ---- the fence with five tabs: 0 常用 (5 icons), 1 项目资料与归档 (5), 2 全部程序 (60), 3 图 (5), 4 下载的文件 (5)
$LtTabs = '常用', '项目资料与归档', '全部程序', '图', '下载的文件'
$LtCount = 5, 5, 60, 5, 5
function HtX($d, $i) { return [int]($d.l + $d.ox + $d.rects[$i][0] + $d.rects[$i][1] / 2) }
function HtY($d) { return [int]($d.t + $d.oy + 18) }
function LtClean() { Remove-Item "$PfDesk\L4 *" -Force -ErrorAction SilentlyContinue }
# Shortcuts to 80 programs of Windows (each with its own icon to read), spread over five fences
# that are then merged into one. Returns the dump of the merged fence.
function LtBuild() {
  LtClean
  ClearFences
  $exes = @(Get-ChildItem "$env:WINDIR\System32\*.exe" | Sort-Object Length -Descending | Select-Object -First 80)
  $sh = New-Object -ComObject WScript.Shell
  $names = '报告', '季度销售数据汇总表', '会议纪要', '图', '项目计划书（最终版）', '发票', '报价单 第二版'
  $paths = @(); $i = 0
  # WScript.Shell cannot save a shortcut under a Chinese name: saved in ASCII, then renamed.
  $tmp = Join-Path $PfRun 'l4.lnk'
  foreach ($e in $exes) { $i++; $p = Join-Path $PfDesk "L4 $($names[$i % 7]) $i.lnk"; $s = $sh.CreateShortcut($tmp); $s.TargetPath = $e.FullName; $s.Save(); Move-Item -LiteralPath $tmp -Destination $p -Force; $paths += $p }
  $host0 = $LtTabs[0]
  Pf fence create --title $host0 --rect '300,160,640,420' | Out-Null
  foreach ($k in 1..4) { $t = $LtTabs[$k]; Pf fence create --title $t --rect "1000,$(60 + 150 * $k),300,140" | Out-Null }
  for ($w = 0; $w -lt 30 -and @(PfJson item list | Where-Object { $_ -and $_.fileName -like 'L4 *' }).Count -lt 80; $w++) { Start-Sleep -Milliseconds 500 }
  $at = 0
  foreach ($k in 0..4) {
    $part = @($paths[$at..($at + $LtCount[$k] - 1)]); $at += $LtCount[$k]
    $t = $LtTabs[$k]
    for ($c = 0; $c -lt $part.Count; $c += 20) { $some = @($part[$c..([math]::Min($part.Count, $c + 20) - 1)]); Pf item move @some --to $t | Out-Null }
  }
  Start-Sleep -Seconds 2
  foreach ($k in 1..4) { $t = $LtTabs[$k]; Pf fence merge $t --into $host0 | Out-Null; Start-Sleep -Milliseconds 400 }
  Start-Sleep -Seconds 2
  ParkMouse 1300 820
  $d = DumpOf $host0
  Log "dump: $($d.raw)"
  return $d
}
# The content below the tabs (without the scrollbar's edge) as the watched region, and the
# picture of every tab's content at rest as answer 0..4. Ends on tab 0.
function LtRefs($d) {
  [Lat]::Region($d.l + 12, $d.t + 56, $d.r - $d.l - 44, [math]::Min($d.b - $d.t - 68, 230))
  foreach ($k in 4, 3, 2, 1, 0) {
    [G]::Click((HtX $d $k), (HtY $d)); Start-Sleep -Milliseconds 300
    ParkMouse 1300 820; Start-Sleep -Milliseconds 900
  }
  foreach ($k in 0..4) {
    [G]::Click((HtX $d $k), (HtY $d)); Start-Sleep -Milliseconds 300
    ParkMouse 1300 820; Start-Sleep -Milliseconds 900
    $n = [Lat]::AddRef()
    if ($k -eq 2) { [Lat]::Kept.Clear(); [Lat]::Kept.Add([Lat]::Refs[$n]); [Lat]::Strip("$PfShots\ref-tab2.png", 1) }
  }
  [G]::Click((HtX $d 0), (HtY $d)); Start-Sleep -Milliseconds 300
  ParkMouse 1300 820; Start-Sleep -Milliseconds 900
}
# The pointer jumps from where it is onto tab $i and rests there. Returns the watch with
# .shown (the tab's whole picture), and writes the log's lines of the switch.
function LtHover($d, $i, $ms = 1200) {
  $pos = LogPos
  $r = LatWatch $ms @(0, (HtX $d $i), (HtY $d))
  $r.shown = SeenAt $r $i
  Start-Sleep -Milliseconds 250
  $r.log = (LogRel $pos $r.utc 'tab hover switch|animation run|slow command') -join ' ; '
  Log "hover tab ${i}: first=$($r.first) shown=$($r.shown) last=$($r.last) per=$($r.per) | $($r.log)"
  return $r
}
# Away from the fence long enough that the next hover is a first one again.
function LtRest() { ParkMouse 1300 820; Start-Sleep -Milliseconds 1300 }
# Pointer moves along the tab centres, from tab $from: legs of @(tab, travel ms, dwell ms), as
# triples for [Lat] and the total ms.
function LtPath($d, $from, $legs) {
  $y = HtY $d; $x = HtX $d $from; $t = 0; $out = New-Object 'Collections.Generic.List[int]'
  $out.Add(0); $out.Add($x); $out.Add($y)
  foreach ($leg in $legs) {
    $x1 = HtX $d $leg[0]; $n = [math]::Max(1, [int]($leg[1] / 10))
    for ($k = 1; $k -le $n; $k++) { $out.Add([int]($t + $leg[1] * $k / $n)); $out.Add([int]($x + ($x1 - $x) * $k / $n)); $out.Add($y) }
    $t += $leg[1] + $leg[2]; $x = $x1
  }
  $script:LtPathMs = $t
  return $out.ToArray()
}
function LtStart($speed, $lines) {
  if ($Next) { $env:PECOFENCE_TAB_NEXT_MS = "$Next" }
  PfStart -Script $lines
  Start-Sleep -Seconds 2
  Pf settings set rollUp.switchTabOnHover true | Out-Null
  Pf settings set rollUp.hoverSpeed $speed | Out-Null
  Start-Sleep -Seconds 1
}

# ---------------------------------------------------------------------------------------------
function PartTAB {
  trap { PfTrap $_; continue }
  Case 'TAB setup: five tabs, 5 / 5 / 60 / 5 / 5 icons'
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtStart 'normal' (DumpScript 250 6)
  $d = LtBuild
  LtRefs $d
  $counts = @($LtTabs | ForEach-Object { @(PfJson item list --fence $_ | Where-Object { $_ -and $_.fileName }).Count }) -join ' '
  Check 'TAB five tabs with 5 5 60 5 5 icons, a picture of each' ($d.ntabs -eq 5 -and $counts -eq '5 5 60 5 5' -and [Lat]::Refs.Count -eq 5) "tabs=$($d.tabs -join ' | ') icons=$counts region=$([Lat]::W)x$([Lat]::H)"

  foreach ($speed in 'normal', 'fast') {
    Case "TAB hover switch with 展开快慢 = $speed"
    # A new run of the app: tabs 1 and 2 were never shown in it.
    PfQuit
    LtStart $speed @('sleep 3600000')
    LtRest
    $never60 = LtHover $d 2
    [Lat]::Strip("$PfShots\tab-never60-$speed.png", 8)
    LtRest
    $never5 = LtHover $d 1
    LtRest
    LatRow "hover tab, $speed" 'never shown, 60 icons' $never60.first $never60.shown $never60.per $never60.log
    LatRow "hover tab, $speed" 'never shown, 5 icons' $never5.first $never5.shown $never5.per $never5.log
    $f60 = @(); $s60 = @(); $f5 = @(); $s5 = @(); $logs = @()
    foreach ($k in 1..5) {
      $a = LtHover $d 2; LtRest
      $b = LtHover $d 1; LtRest
      $f60 += $a.first; $s60 += $a.shown; $f5 += $b.first; $s5 += $b.shown
      if ($k -eq 1) { $logs = @($a.log, $b.log); [Lat]::Strip("$PfShots\tab-shown5-$speed.png", 8) }
    }
    LatRow "hover tab, $speed" 'shown before, 60 icons' "$(Med $f60) (worst $(Worst $f60))" "$(Med $s60) (worst $(Worst $s60))" $a.per "first: $($f60 -join ' ') shown: $($s60 -join ' ') | $($logs[0])"
    LatRow "hover tab, $speed" 'shown before, 5 icons' "$(Med $f5) (worst $(Worst $f5))" "$(Med $s5) (worst $(Worst $s5))" $b.per "first: $($f5 -join ' ') shown: $($s5 -join ' ') | $($logs[1])"
    $full = if ($speed -eq 'fast') { 150 } else { 400 }
    Check "TAB ${speed}: the first tab under a resting pointer shows after the full wait ($full ms) and is whole within 300 ms more" ((Med $f5) -ge $full - 40 -and (Med $s5) -gt 0 -and (Med $s5) -le $full + 300) "5 icons: first $($f5 -join ' ') whole $($s5 -join ' '); 60 icons: first $($f60 -join ' ') whole $($s60 -join ' '); never shown: 60 icons first $($never60.first) whole $($never60.shown), 5 icons first $($never5.first) whole $($never5.shown); one grab $($a.per) ms"
  }
  LtEnd
}

# The tab after a hover switch, and sweeps.
function PartNEXT {
  trap { PfTrap $_; continue }
  Case 'NEXT setup'
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtStart 'normal' (DumpScript 250 6)
  $d = LtBuild
  LtRefs $d
  PfQuit
  LtStart 'normal' @('sleep 3600000')
  foreach ($k in 1, 2, 3, 4, 0) { [G]::Click((HtX $d $k), (HtY $d)); Start-Sleep -Milliseconds 500 }
  LtRest

  Case 'NEXT the tab after a hover switch'
  # The pointer rests on tab 1 until it shows (a first hover: the full wait), then goes to the
  # next tab without leaving the strip.
  $firsts = @(); $shown = @(); $log = ''
  foreach ($k in 1..5) {
    LtRest
    $null = LtHover $d 1 900
    $r = LtHover $d 3 900
    $firsts += $r.first; $shown += $r.shown; if ($k -eq 1) { $log = $r.log; [Lat]::Strip("$PfShots\next-1.png", 8) }
    $r2 = LtHover $d 4 900
    $firsts += $r2.first; $shown += $r2.shown
  }
  LatRow 'hover the next tab after a hover switch, normal' '5 icons' "$(Med $firsts) (worst $(Worst $firsts))" "$(Med $shown) (worst $(Worst $shown))" $r.per "first: $($firsts -join ' ') shown: $($shown -join ' ') | $log"
  Check 'NEXT after a hover switch the next tab under the pointer is whole within 200 ms (默认)' ((Worst $shown) -le 200 -and (Med $shown) -gt 0) "first $($firsts -join ' ') whole $($shown -join ' ') ms; one grab $($r.per) ms"

  Case 'NEXT what ends the short wait'
  # Off the tabs for 1.3 s: the full wait again.
  LtRest
  $null = LtHover $d 1 900
  LtRest
  $r = LtHover $d 3 900
  Check 'NEXT after 1.3 s off the tabs the next hover waits the full time again' ($r.first -ge 360) "first=$($r.first) whole=$($r.shown)"
  # A pointer that only crosses the strip from outside switches nothing.
  LtRest
  $x = HtX $d 1; $y = HtY $d
  $r = LatWatch 900 @(0, $x, ($y - 60), 20, $x, ($y - 20), 40, $x, $y, 100, $x, ($y + 20), 120, $x, ($y + 200), 140, 1300, 820)
  Check 'NEXT a pointer that crosses a tab in 60 ms switches nothing' ($r.first -lt 0) "first=$($r.first)"
  # A click on a tab ends the short wait.
  LtRest
  $null = LtHover $d 1 900
  [G]::Click((HtX $d 3), (HtY $d)); Start-Sleep -Milliseconds 700
  $r = LtHover $d 4 900
  Check 'NEXT after a click the next hover waits the full time again' ($r.first -ge 360) "first=$($r.first) whole=$($r.shown)"

  Case 'NEXT sweeps along the strip'
  # The pointer rests on tab 1 (600 ms), then goes on over tabs 2, 3 and 4 at a steady speed and
  # rests on tab 4. Which tabs were shown whole on the way?
  foreach ($per in 40, 70, 110, 160, 250) {
    [G]::Click((HtX $d 0), (HtY $d)); Start-Sleep -Milliseconds 400
    LtRest
    [G]::MoveTo((HtX $d 0), (HtY $d) - 40); Start-Sleep -Milliseconds 100
    $path = LtPath $d 1 @(@(1, 0, 650), @(2, $per, 0), @(3, $per, 0), @(4, $per, 600))
    $r = LatWatch ($script:LtPathMs + 200) $path
    $seen = SeenList $r
    LatRow "sweep over tabs 2 3 4, $per ms a tab" 'after a hover switch to tab 1' '' $seen $r.per "seen=$($r.seen)"
    Check "NEXT sweep at $per ms a tab ends on the last tab" ($seen -match '4$') "tabs shown whole: $seen ; $($r.seen)"
  }
  LtEnd
}

# The sweep for the before/after clip: rest on tab 1, on over 2, 3 and 4 with 220 ms on each, back to 0.
function PartREC {
  trap { PfTrap $_; continue }
  Case "REC sweep ($Rec)"
  CloseWindows
  . C:\pf\kit\rec.ps1
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtStart 'normal' (DumpScript 250 6)
  $d = LtBuild
  LtRefs $d
  PfQuit
  LtStart 'normal' @('sleep 3600000')
  foreach ($k in 1, 2, 3, 4, 0) { [G]::Click((HtX $d $k), (HtY $d)); Start-Sleep -Milliseconds 500 }
  LtRest
  [G]::MoveTo((HtX $d 0), (HtY $d) - 40); Start-Sleep -Milliseconds 300
  $path = LtPath $d 1 @(@(1, 0, 700), @(2, 60, 200), @(3, 60, 200), @(4, 60, 200), @(0, 240, 900))
  RecStart "SWEEP-$Rec" -X ($d.l - 20) -Y ($d.t - 50) -W ($d.r - $d.l + 40) -H 330 -Fps 30
  RecMark '鼠标先停在第二个标签上，再划过后面三个标签，每个停 0.2 秒，最后回到第一个'
  Start-Sleep -Milliseconds 300
  [Lat]::Play([int[]]$path, $script:LtPathMs)
  RecMark '停下'
  Start-Sleep -Milliseconds 600
  RecStop
  # The same path once more, watched: which tabs were shown whole.
  [G]::Click((HtX $d 0), (HtY $d)); Start-Sleep -Milliseconds 400
  LtRest
  [G]::MoveTo((HtX $d 0), (HtY $d) - 40); Start-Sleep -Milliseconds 300
  $r = LatWatch ($script:LtPathMs + 200) $path
  $seen = SeenList $r
  LatRow 'sweep of the clip' "$Rec" '' $seen $r.per "seen=$($r.seen)"
  Check "REC recorded the sweep ($Rec)" ($seen -ne '') "tabs shown whole: $seen ; $($r.seen)"
  LtEnd
}

# The last case of every part: the test files, the animations switch and the pointer as they were.
function LtEnd {
  Case 'cleanup'
  ParkMouse 1300 820
  PfQuit
  LtClean
  [Anim]::Set([bool]$script:AnimWas)
  $env:PECOFENCE_TAB_NEXT_MS = $null
  Check 'the test files are gone from the desktop' (@(Get-ChildItem "$PfDesk\L4 *" -ErrorAction SilentlyContinue).Count -eq 0) ''
}

if ($Part -eq 'TAB') { PartTAB }
if ($Part -eq 'NEXT') { PartNEXT }
if ($Part -eq 'REC') { PartREC }
