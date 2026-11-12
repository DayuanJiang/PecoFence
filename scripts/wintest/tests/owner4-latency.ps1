# Owner round 2026-11-12, latency: how long PecoFence takes to answer what the user does.
# One part per call:
#   --var Part=TAB    the hover tab switch (「鼠标悬停时切换标签页」): from the pointer on another tab to
#                     the first picture of the new tab and to the complete one, with 默认 and 快, on a
#                     tab shown before and on one never shown in this run, with 5 and with 60 icons
#   --var Part=NEXT   the tab after a hover switch follows the pointer after a short wait (the check
#                     that fails before the change), and sweeps along five tabs
#   --var Part=REC --var Rec=before|after   the sweep recorded for the before/after clip
#   --var Part=ACT    everyday actions on icons and fences: pointer, click, right-click, drag,
#                     roll, chevron, tab, icon size, wheel, F2, Delete, double-click, quick hide
#   --var Part=ACT2   notes, a watched folder and its filter, the tray menu, 设置…, Peek and its search
#   --var Part=WIN    the same pointer and key actions on Windows' own desktop, PecoFence not running
#   --var Part=SRCH   the Peek's search box at the first letter, with the app's log times beside the grabs
#   --var Part=ROLL   a row of four rolled fences: the first under the pointer, the next ones after it
#                     (--var Rec=before|after records the sweep along the row); Part=COL: the same in a column
#   --var Part=FILL   the hover and selection fills in the first draws after the input, from the app's log
#   --var Part=MORE   two file changes within 0.4 s, a new match in a smart fence, a page change in the
#                     settings window, the whole-computer rows of the Peek search (needs Everything)
#   --var Part=MORE2  the first right-click menu and the first Peek after the app's start
#   --var Part=CLEAN  removes what an interrupted run left (test files on the desktop and in the Recycle Bin)
# Every part writes shots\latency.tsv: part, action, case, first answer ms (median and worst of
# five), complete ms, ms per grab, the single runs and the app's log lines.
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
  // How far a channel must differ for a pixel to count as changed (1: any change the eye could see).
  public static int Tol = 6;
  public static List<int[]> Refs = new List<int[]>();
  public static List<int[]> Kept = new List<int[]>();
  static int[] Grab() {
    using (var b = new Bitmap(W, H, PixelFormat.Format32bppRgb)) {
      using (var g = Graphics.FromImage(b)) g.CopyFromScreen(X, Y, 0, 0, new Size(W, H));
      var d = b.LockBits(new Rectangle(0, 0, W, H), ImageLockMode.ReadOnly, PixelFormat.Format32bppRgb);
      var a = new int[W * H]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
    }
  }
  // Pixels that differ by more than Tol in a channel.
  static int Differ(int[] a, int[] b) {
    int n = 0;
    for (int i = 0; i < a.Length; i++) {
      int p = a[i], q = b[i];
      if (p == q) continue;
      if (Math.Abs((p & 255) - (q & 255)) > Tol || Math.Abs(((p >> 8) & 255) - ((q >> 8) & 255)) > Tol || Math.Abs(((p >> 16) & 255) - ((q >> 16) & 255)) > Tol) n++;
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
# The shortcuts the Delete measures sent to the Recycle Bin leave it too (entries named "L4 ..." only).
function LtBinClean() {
  $dir = RecycleDir
  foreach ($it in @((New-Object -ComObject Shell.Application).NameSpace(10).Items())) {
    if ($it.Name -notlike 'L4 *') { continue }
    $r = "$($it.Path)"
    if ($r.StartsWith($dir, [StringComparison]::OrdinalIgnoreCase)) {
      $i = Join-Path $dir ('$I' + (Split-Path $r -Leaf).Substring(2))
      Remove-Item -LiteralPath $r, $i -Force -ErrorAction SilentlyContinue
    }
  }
}
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
  LtBinClean
  [Anim]::Set([bool]$script:AnimWas)
  Check 'the test files are gone from the desktop and the Recycle Bin' (@(Get-ChildItem "$PfDesk\L4 *" -ErrorAction SilentlyContinue).Count -eq 0 -and (BinNames) -notmatch '(^|, )L4 ') "bin: $(BinNames)"
}

if ($Part -eq 'TAB') { PartTAB }
if ($Part -eq 'NEXT') { PartNEXT }
if ($Part -eq 'REC') { PartREC }

# ---------------------------------------------------------------------------------------------
# Part 2: everyday actions. ACT: icons and fences. ACT2: Peek, tray, settings, notes, folders.
# WIN: the same actions on Windows' own desktop (PecoFence not running).
function Chg($r) { $o = @(); if ($r.raw -match '\bchanges=(\S+)') { foreach ($p in ($Matches[1] -split ',')) { $q = $p -split ':'; $o += , @([int]$q[0], [int]$q[1]) } }; return , $o }
# First and last change at or after $z ms of a watch, counted from $z (-1: none). $minPx: the
# first answer is the first change of at least so many pixels (a menu or a drag picture, not the
# fades still running from the press).
function After($r, $z = 0, $minPx = 0) { $f = -1; $l = -1; foreach ($c in (Chg $r)) { if ($c[0] -ge $z) { if ($f -lt 0 -and $c[1] -ge $minPx) { $f = $c[0] - $z }; if ($f -ge 0) { $l = $c[0] - $z } } }; return @{ first = $f; done = $l; per = $r.per; utc = $r.utc } }
$script:Ms = @{}
function Add1($key, $m) { if (-not $script:Ms.ContainsKey($key)) { $script:Ms[$key] = @() }; $script:Ms[$key] += , $m; if (@($script:Ms[$key]).Count -eq 1) { [Lat]::Strip("$PfShots\act-$key.png", 10) } }
function Row($action, $case, $key, $note = '') {
  $all = @($script:Ms[$key]); if ($all.Count -eq 0) { LatRow $action $case 'not measured' '' '' $note; return }
  $f = @($all | ForEach-Object { $_.first }); $d = @($all | ForEach-Object { $_.done })
  LatRow $action $case "$(Med $f) (worst $(Worst $f))" "$(Med $d) (worst $(Worst $d))" $all[0].per "first: $($f -join ' ') done: $($d -join ' ') | $note"
  Check "$action ($case): an answer was seen" ((Med $f) -ge 0) "first: $($f -join ' ') done: $($d -join ' ') ms, one grab $($all[0].per) ms"
}
function CellBox($p) { [Lat]::Region($p[0] - 38, $p[1] - 34, 76, 92) }
function RDown() { [G]::mouse_event(8, 0, 0, 0, [UIntPtr]::Zero) }
function RUp() { [G]::mouse_event(16, 0, 0, 0, [UIntPtr]::Zero) }
function L4Links($n) {
  $exes = @(Get-ChildItem "$env:WINDIR\System32\*.exe" | Sort-Object Length -Descending | Select-Object -First $n)
  $sh = New-Object -ComObject WScript.Shell; $tmp = Join-Path $PfRun 'l4.lnk'
  $names = '报告', '季度销售数据汇总表', '会议纪要', '图', '项目计划书（最终版）', '发票', '报价单 第二版'
  $out = @(); $i = 0
  foreach ($e in $exes) { $i++; $p = Join-Path $PfDesk "L4 $($names[$i % 7]) $i.lnk"; $s = $sh.CreateShortcut($tmp); $s.TargetPath = $e.FullName; $s.Save(); Move-Item -LiteralPath $tmp -Destination $p -Force; $out += $p }
  return $out
}
function WaitL4($n) { for ($w = 0; $w -lt 30 -and @(PfJson item list | Where-Object { $_ -and $_.fileName -like 'L4 *' }).Count -lt $n; $w++) { Start-Sleep -Milliseconds 500 } }
function SlowLog($pos, $utc) { return ((LogRel $pos $utc '^(?!.*\] live ).*(slow command|launch|settings: window frame|settings: WebView2 controller|peek)') -join ' ; ') }

function PartACT {
  trap { PfTrap $_; continue }
  Case 'ACT setup'
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  PfStart -Script (DumpScript 250 2)
  Start-Sleep -Seconds 2
  ClearFences
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  Pf settings set quickHide.enabled true | Out-Null
  $links = L4Links 41
  $dbl = Join-Path $PfDesk 'L4 双击打开.txt'; Set-Content -LiteralPath $dbl -Value 'latency' -Encoding ASCII
  $TI = 'L4 图标'; $TS = 'L4 滚动'; $TR = 'L4 收起'; $TO = 'L4 打开'; $TA = 'L4 标签甲'; $TB = 'L4 标签乙'
  Pf fence create --title $TI --rect '300,160,640,330' | Out-Null
  Pf fence create --title $TS --rect '980,160,300,220' | Out-Null
  Pf fence create --title $TR --rect '300,560,400,200' | Out-Null
  Pf fence create --title $TO --rect '760,560,180,160' | Out-Null
  Pf fence create --title $TA --rect '980,440,300,200' | Out-Null
  Pf fence create --title $TB --rect '1300,440,200,150' | Out-Null
  WaitL4 42
  $a = @($links[0..11]); Pf item move @a --to $TI | Out-Null
  $a = @($links[12..31]); Pf item move @a --to $TS | Out-Null
  $a = @($links[32..34]); Pf item move @a --to $TR | Out-Null
  $a = @($links[35..37]); Pf item move @a --to $TA | Out-Null
  $a = @($links[38..40]); Pf item move @a --to $TB | Out-Null
  Pf item move $dbl --to $TO | Out-Null
  Start-Sleep -Seconds 2
  Pf fence merge $TB --into $TA | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 820
  $dt = DumpOf $TA
  $wi = (FenceGet $TI).windowRect; $ws = (FenceGet $TS).windowRect; $wr = (FenceGet $TR).windowRect
  Check 'ACT fences are there' ($dt.ntabs -eq 2 -and $wi.w -eq 640) "tabs=$($dt.tabs -join ' | ') icons fence $($wi.x),$($wi.y),$($wi.w),$($wi.h)"

  Case 'ACT pointer onto an icon'
  foreach ($k in 1..5) {
    ParkMouse 1300 820; Start-Sleep -Milliseconds 400
    $p = CellPt $TI $k; CellBox $p
    $r = LatWatch 350 @(0, $p[0], $p[1])
    Add1 'hover' (After $r 0)
  }
  Row 'pointer onto an icon (hover fill)' 'fence' 'hover'

  Case 'ACT click on an icon'
  foreach ($k in 1..5) {
    $p = CellPt $TI $k; CellBox $p
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 250
    $r = LatWatch 500 $null { MDown } { MUp } 40
    Add1 'click' (After $r 0)
    $null = DeskClick 1300 780
  }
  Row 'click on an icon (selection)' 'fence' 'click'

  Case 'ACT right-click on an icon'
  foreach ($k in 1..5) {
    $p = CellPt $TI 2
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 500
    [Lat]::Region($p[0] + 60, $p[1] + 10, 160, 240)
    $pos = LogPos
    $r = LatWatch 1500 $null { RDown } { RUp } 40
    $m = After $r 40 12000; Add1 'rclick' $m
    if ($k -eq 1) { $note = SlowLog $pos $r.utc }
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
    $null = DeskClick 1300 780
  }
  Row 'right-click on an icon (menu)' 'fence' 'rclick' $note

  Case 'ACT right-click on a bare part of a fence'
  foreach ($k in 1..5) {
    $x = $wi.x + $wi.w - 260; $y = $wi.y + $wi.h - 60
    [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 400
    [Lat]::Region($x + 20, $y + 20, 160, 120)
    $r = LatWatch 1200 $null { RDown } { RUp } 40
    Add1 'rbare' (After $r 40 1000)
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Row 'right-click on a bare part of a fence (menu)' 'fence' 'rbare'

  Case 'ACT press and drag of an icon'
  foreach ($k in 1..5) {
    $p = CellPt $TI 3
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 500
    [Lat]::Region($p[0] - 60, $p[1] - 50, 200, 170)
    $x = $p[0]; $y = $p[1]
    $r = LatWatch 900 @(150, ($x + 2), ($y + 2), 200, ($x + 12), ($y + 12), 230, ($x + 24), ($y + 24), 260, ($x + 40), ($y + 40)) { MDown } { [G]::Press(0x1B) } 700
    MUp
    Add1 'drag' (After $r 200 6000)
    Start-Sleep -Milliseconds 400
    $null = DeskClick 1300 780
  }
  Row 'press and drag of an icon (drag picture)' 'fence; from the move that passes the drag threshold' 'drag'

  Case 'ACT press and drag of a fence title'
  foreach ($k in 1..5) {
    $x = $wi.x + $wi.w - 160; $y = $wi.y + 18
    [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 500
    [Lat]::Region($wi.x - 30, $wi.y + 80, 60, 60)
    $r = LatWatch 900 @(150, ($x + 2), $y, 200, ($x + 14), $y, 230, ($x + 30), $y, 260, ($x + 50), $y) { MDown } { MUp } 600
    Add1 'title' (After $r 200)
    Start-Sleep -Milliseconds 300
    Pf fence move $TI --rect '300,160,640,330' | Out-Null
    Start-Sleep -Milliseconds 600
  }
  Row 'press and drag of a fence title (first move)' 'fence; from the move that passes the drag threshold' 'title'

  Case 'ACT a rolled fence under the pointer, and left'
  Pf fence roll $TR | Out-Null
  ParkMouse 1300 820; Start-Sleep -Milliseconds 800
  foreach ($k in 1..5) {
    [Lat]::Region($wr.x + 20, $wr.y + 44, 200, $wr.h - 50)
    $r = LatWatch 1300 @(0, ($wr.x + 120), ($wr.y + 18))
    Add1 'expand' (After $r 0)
    Start-Sleep -Milliseconds 300
    $r = LatWatch 1300 @(0, 1300, 820)
    Add1 'rollup' (After $r 0)
    Start-Sleep -Milliseconds 400
  }
  Row 'a rolled fence under the pointer (expands)' 'fence, 展开快慢 默认' 'expand'
  Row 'the pointer leaves an expanded fence (rolls up)' 'fence' 'rollup'

  Case 'ACT click on the chevron'
  foreach ($k in 1..5) {
    ParkMouse 1300 820; Start-Sleep -Milliseconds 500
    [Lat]::Region($wr.x + 20, $wr.y + 44, 200, $wr.h - 50)
    [G]::MoveTo($wr.x + $wr.w - 13, $wr.y + 18); Start-Sleep -Milliseconds 80
    $r = LatWatch 800 $null { MDown } { MUp } 40
    Add1 'chevopen' (After $r 40)
    Start-Sleep -Milliseconds 300
    $r = LatWatch 800 $null { MDown } { MUp } 40
    Add1 'chevclose' (After $r 40)
  }
  Row 'click on the chevron of a rolled fence (expands)' 'fence' 'chevopen'
  Row 'click on the chevron of an open fence (rolls up)' 'fence' 'chevclose'
  ParkMouse 1300 820

  Case 'ACT a tab clicked'
  [Lat]::Region($dt.l + 12, $dt.t + 56, $dt.r - $dt.l - 44, 100)
  foreach ($k in 1..6) {
    $i = $k % 2
    [G]::MoveTo((HtX $dt $i), (HtY $dt)); Start-Sleep -Milliseconds 500
    $r = LatWatch 700 $null { MDown } { MUp } 40
    if ($k -gt 1) { Add1 'tab' (After $r 0) }
  }
  Row 'a tab clicked' 'fence, 3 icons a tab' 'tab'
  $tf = @($script:Ms['tab'] | ForEach-Object { $_.first })
  Check 'ACT a clicked tab shows its page in the first picture after the click (48 ms at most)' ((Med $tf) -ge 0 -and (Med $tf) -le 48) "first: $($tf -join ' ') ms"
  ParkMouse 1300 820

  Case 'ACT the icon size changed with Ctrl and the wheel'
  [Lat]::Region($wi.x + 12, $wi.y + 50, $wi.w - 44, 200)
  [G]::MoveTo($wi.x + 300, $wi.y + 250); Start-Sleep -Milliseconds 400
  foreach ($k in 1..6) {
    $n = if ($k % 2) { 1 } else { -1 }
    $d = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int](120 * $n)), 0)
    $r = LatWatch 900 $null { [G]::KeyDown(0x11); [G]::mouse_event(0x800, 0, 0, $d, [UIntPtr]::Zero); [G]::KeyUp(0x11) }
    if ($k -gt 1) { Add1 'zoom' (After $r 0) }
    Start-Sleep -Milliseconds 300
  }
  Row 'the view changed (icon size, Ctrl and the wheel)' 'fence, 12 icons' 'zoom'

  Case 'ACT wheel scrolling'
  [Lat]::Region($ws.x + 12, $ws.y + 50, $ws.w - 44, $ws.h - 70)
  [G]::MoveTo($ws.x + 150, $ws.y + 120); Start-Sleep -Milliseconds 400
  foreach ($k in 1..6) {
    $n = if ($k % 2) { -1 } else { 1 }
    $d = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int](120 * $n)), 0)
    $r = LatWatch 800 $null { [G]::mouse_event(0x800, 0, 0, $d, [UIntPtr]::Zero) }
    if ($k -gt 1) { Add1 'wheel' (After $r 0) }
    Start-Sleep -Milliseconds 300
  }
  Row 'wheel scrolling (one notch)' 'fence, 20 icons' 'wheel'

  Case 'ACT F2 and Delete'
  foreach ($k in 1..5) {
    $p = CellPt $TI $k; CellBox $p
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
    $r = LatWatch 450 $null { [G]::Press(0x71) }
    Add1 'f2' (After $r 0)
    [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  }
  Row 'F2 (name box)' 'fence' 'f2'
  foreach ($k in 1..5) {
    $p = CellPt $TS 0; CellBox $p
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
    $pos = LogPos
    $r = LatWatch 2500 $null { [G]::Press(0x2E) }
    Add1 'delete' (After $r 0)
    if ($k -eq 1) { $note = SlowLog $pos $r.utc }
    Start-Sleep -Milliseconds 800
  }
  Row 'Delete (icon gone)' 'fence, a shortcut to the Recycle Bin' 'delete' $note
  ParkMouse 1300 820

  Case 'ACT double-click on an icon'
  $t = @(); $note = ''
  foreach ($k in 1..3) {
    $p = CellPt $TO 0
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 400
    $pos = LogPos
    MDown; MUp; Start-Sleep -Milliseconds 70
    $utc = [DateTime]::UtcNow.Ticks; $sw = [Diagnostics.Stopwatch]::StartNew()
    MDown; MUp
    $ms = -1
    while ($sw.ElapsedMilliseconds -lt 6000) { $h = [G]::FindWindowW('Notepad', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $ms = [int]$sw.ElapsedMilliseconds; break } }
    $t += $ms
    Start-Sleep -Milliseconds 600
    if ($k -eq 1) { $note = SlowLog $pos $utc }
    Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 700
    $null = DeskClick 1300 780
  }
  LatRow 'double-click on an icon (Notepad window on screen)' 'fence, a text file' "$(Med $t) (worst $(Worst $t))" '' 1 "window after: $($t -join ' ') ms | $note"
  Check 'ACT double-click opened Notepad' ((Med $t) -gt 0) "$($t -join ' ') ms"

  Case 'ACT quick hide by a double-click on the desktop, and back'
  $s = FreeSpot 1250 760
  [Lat]::Region($wi.x + 20, $wi.y + 60, 200, 120)
  foreach ($k in 1..6) {
    [G]::MoveTo($s[0], $s[1]); Start-Sleep -Milliseconds 700
    MDown; MUp; Start-Sleep -Milliseconds 70
    $r = LatWatch 1100 $null { MDown } { MUp } 30
    $key = if ($k % 2) { 'hide' } else { 'show' }
    Add1 $key (After $r 30)
  }
  Row 'quick hide: double-click on the desktop (fences fade out)' 'from the second release' 'hide'
  Row 'quick hide: double-click again (fences back)' 'from the second release' 'show'
  LtEnd
}

function PartACT2 {
  trap { PfTrap $_; continue }
  Case 'ACT2 setup'
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  $dir = Join-Path $PfRun 'L4文件夹'
  New-Item -ItemType Directory -Force $dir | Out-Null
  foreach ($k in 1..20) { Set-Content -LiteralPath (Join-Path $dir ("a{0:d2} 报告.txt" -f $k)) -Value 'x' -Encoding ASCII }
  PfStart -Script (DumpScript 250 2)
  Start-Sleep -Seconds 2
  ClearFences
  $links = L4Links 12
  $TI = 'L4 图标'; $TN = 'L4 便签'; $TP = 'L4 文件夹'
  Pf fence create --title $TI --rect '300,160,640,330' | Out-Null
  Pf fence create --tool note --title $TN --text "[ ] 买牛奶`n[ ] 寄快递`n明天上午开会" --rect '980,160,300,220' | Out-Null
  Pf fence create --portal $dir --title $TP --rect '980,420,420,300' | Out-Null
  WaitL4 12
  Pf item move @links --to $TI | Out-Null
  Start-Sleep -Seconds 3
  ParkMouse 1300 860
  $dn = DumpOf $TN
  $wi = (FenceGet $TI).windowRect; $wp = (FenceGet $TP).windowRect; $wn = (FenceGet $TN).windowRect
  Check 'ACT2 fences are there' ($wi.w -eq 640 -and $wp.w -eq 420 -and $dn) "note dump: $($dn.raw)"

  Case 'ACT2 a to-do ticked, a note clicked'
  $cx = [int](DumpVal $dn 'content_x'); $cy = [int](DumpVal $dn 'content_y')
  $bx = -1; $v = DumpVal $dn 'note_boxes'
  if ($v -and $v -ne '-') { $q = (($v -split ';')[0] -split ':')[1] -split ','; $bx = $dn.l + $cx + [int](([double]$q[0] + [double]$q[2]) / 2); $by = $dn.t + $cy + [int](([double]$q[1] + [double]$q[3]) / 2) }
  Log "to-do box at $bx,$by (note_boxes=$v)"
  [Lat]::Region($wn.x + 8, $wn.y + 40, $wn.w - 30, 120)
  if ($bx -gt 0) {
    foreach ($k in 1..5) {
      [G]::MoveTo($bx, $by); Start-Sleep -Milliseconds 600
      $r = LatWatch 600 $null { MDown } { MUp } 40
      Add1 'todo' (After $r 40)
    }
  }
  Row 'a to-do ticked' 'note; from the release' 'todo'
  foreach ($k in 1..5) {
    [G]::MoveTo($wn.x + 150, $wn.y + 150); Start-Sleep -Milliseconds 500
    $r = LatWatch 450 $null { MDown } { MUp } 40
    Add1 'note' (After $r 0)
    $null = DeskClick 1300 780; Start-Sleep -Milliseconds 400
  }
  Row 'a note clicked (caret)' 'note' 'note'

  Case 'ACT2 a file created in a watched folder, a letter in its filter'
  [Lat]::Region($wp.x + 12, $wp.y + 50, $wp.w - 44, $wp.h - 70)
  foreach ($k in 1..5) {
    $f = Join-Path $dir "a00 新文件 $k.txt"
    $pos = LogPos
    $r = LatWatch 1500 $null { Set-Content -LiteralPath $f -Value 'x' -Encoding ASCII }
    Add1 'watch' (After $r 0)
    if ($k -eq 1) { $note = SlowLog $pos $r.utc }
    Start-Sleep -Milliseconds 700
  }
  Row 'a file created in a watched folder (icon there)' 'folder fence, 20 files' 'watch' $note
  [G]::Click($wp.x + $wp.w - 80, $wp.y + $wp.h - 30); Start-Sleep -Milliseconds 400
  foreach ($k in 1..5) {
    Chord 0x11 0x46; Start-Sleep -Milliseconds 700
    $r = LatWatch 700 $null { [G]::Type('1') }
    Add1 'filter1' (After $r 0)
    $r = LatWatch 700 $null { [G]::Type('5') }
    Add1 'filter2' (After $r 0)
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Row 'a letter in a folder fence filter (first letter)' 'folder fence, 25 files' 'filter1'
  Row 'a letter in a folder fence filter (second letter)' 'folder fence, 25 files' 'filter2'
  $ff = @($script:Ms['filter1'] | ForEach-Object { $_.first })
  Check 'ACT2 a letter in the filter shows the filtered list in the first picture after the key (48 ms at most)' ((Med $ff) -ge 0 -and (Med $ff) -le 48) "first: $($ff -join ' ') ms"
  ParkMouse 1300 860

  Case 'ACT2 the tray menu and the settings window'
  $tr = TrayIconRect
  if ($tr) {
    $tx = [int]($tr.X + $tr.Width / 2); $ty = [int]($tr.Y + $tr.Height / 2)
    foreach ($k in 1..5) {
      [G]::MoveTo($tx, $ty); Start-Sleep -Milliseconds 500
      [Lat]::Region($tx - 200, $ty - 330, 300, 300)
      $r = LatWatch 1200 $null { RDown } { RUp } 40
      Add1 'tray' (After $r 40 12000)
      [G]::Press(0x1B); Start-Sleep -Milliseconds 600
    }
    Row 'the tray menu' 'from the release' 'tray'
    foreach ($k in 1..4) {
      [G]::MoveTo($tx, $ty); Start-Sleep -Milliseconds 500
      [Lat]::Region(500, 250, 600, 400)
      $pos = LogPos
      $r = LatWatch 4000 $null { MDown } { MUp } 40
      $m = After $r 40
      if ($k -eq 1) { $cold = $m; $coldNote = SlowLog $pos $r.utc; [Lat]::Strip("$PfShots\act-settings-cold.png", 10) } else { Add1 'settings' $m; $note = SlowLog $pos $r.utc }
      $hs = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
      $rr = Rect $hs; Log "settings window $hs at $($rr.L),$($rr.T),$($rr.R),$($rr.B)"
      $res = [IntPtr]::Zero
      [G]::SendMessageTimeoutW($hs, 0x10, [IntPtr]0, [IntPtr]0, 2, 5000, [ref]$res) | Out-Null
      Start-Sleep -Milliseconds 1200
    }
    LatRow '设置… the first time (frame, then page)' 'tray icon clicked; from the release' $cold.first $cold.done $cold.per $coldNote
    Row '设置… opened again (frame, then page)' 'tray icon clicked; from the release' 'settings' $note
  }

  Case 'ACT2 the Peek hotkey and typing in it'
  Start-Process notepad
  $np = WaitWin 'Notepad' 10
  [L]::MoveWindow($np, 250, 120, 800, 450, $true) | Out-Null
  Start-Sleep -Seconds 1
  [G]::Click(700, 400); Start-Sleep -Milliseconds 500
  ParkMouse 1300 860
  foreach ($k in 1..5) {
    [Lat]::Region(340, 240, 300, 150)
    $pos = LogPos
    $r = LatWatch 1000 $null { PeekKey }
    Add1 'peek' (After $r 0)
    if ($k -eq 1) { $note = SlowLog $pos $r.utc }
    Start-Sleep -Milliseconds 500
    $r = LatWatch 1000 $null { PeekKey }
    Add1 'peekend' (After $r 0)
    Start-Sleep -Milliseconds 700
  }
  Row 'the Peek hotkey (fences over the window)' 'a Notepad window over the fence' 'peek' $note
  Row 'the Peek hotkey again (fences gone)' 'a Notepad window over the fence' 'peekend'
  # Where the search box comes.
  PeekKey; Start-Sleep -Milliseconds 800
  [G]::Type('l'); Start-Sleep -Milliseconds 900
  $sb = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value)
  $br = Rect $sb; Log "search box $sb at $($br.L),$($br.T),$($br.R),$($br.B)"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  if (([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0) { PeekKey; Start-Sleep -Milliseconds 600 }
  if ($br.R -gt $br.L) {
    foreach ($k in 1..5) {
      PeekKey; Start-Sleep -Milliseconds 900
      [Lat]::Region($br.L, $br.T, [math]::Min(700, $br.R - $br.L), 320)
      $r = LatWatch 900 $null { [G]::Type('l') }
      Add1 'search1' (After $r 0)
      $r = LatWatch 700 $null { [G]::Type('4') }
      Add1 'search2' (After $r 0)
      [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 600
      if (([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0) { PeekKey; Start-Sleep -Milliseconds 600 }
    }
  }
  Row 'the first letter typed in a Peek (search box, results)' 'Peek open' 'search1'
  Row 'the second letter typed in a Peek (results)' 'Peek open' 'search2'
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue
  LtEnd
}

# Windows' own desktop, without PecoFence: the same pointer and key actions on Explorer's icons.
function PartWIN {
  trap { PfTrap $_; continue }
  Case 'WIN setup: files on Windows own desktop'
  [Lat]::Tol = 1
  CloseWindows
  . C:\pf\kit\deskicons.ps1
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 2 }
  $null = L4Links 8
  $dbl = Join-Path $PfDesk 'L4 双击打开.txt'; Set-Content -LiteralPath $dbl -Value 'latency' -Encoding ASCII
  Start-Sleep -Seconds 3
  $o = [DeskIcons]::Origin() -split ','
  $pts = @(); $dp = $null
  foreach ($l in [DeskIcons]::List()) {
    $q = $l -split '\|'; if ($q.Count -lt 3 -or $q[0] -notmatch 'L4 ') { continue }
    $pt = @(([int]$o[0] + [int]$q[1].Trim() + 38), ([int]$o[1] + [int]$q[2].Trim() + 30))
    if ($q[0] -match '双击') { $dp = $pt } else { $pts += , $pt }
  }
  $sp = [DeskIcons]::Spacing()
  Check 'WIN the test files are icons of the desktop' ($pts.Count -ge 6 -and $dp) "icons=$($pts.Count) spacing=$sp first=$($pts[0] -join ',')"
  ParkMouse 1300 780

  Case 'WIN pointer onto an icon, click, right-click, drag, F2, Delete'
  foreach ($k in 1..5) {
    ParkMouse 1300 780; Start-Sleep -Milliseconds 400
    $p = $pts[$k]; CellBox $p
    $r = LatWatch 350 @(0, $p[0], $p[1])
    Add1 'whover' (After $r 0)
  }
  Row 'pointer onto an icon (hover fill)' 'Windows desktop' 'whover'
  foreach ($k in 1..5) {
    $p = $pts[$k]; CellBox $p
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 250
    $r = LatWatch 500 $null { MDown } { MUp } 40
    Add1 'wclick' (After $r 0)
    $null = DeskClick 1300 780
  }
  Row 'click on an icon (selection)' 'Windows desktop' 'wclick'
  foreach ($k in 1..5) {
    $p = $pts[2]
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 500
    [Lat]::Region($p[0] + 60, $p[1] + 10, 160, 240)
    $r = LatWatch 1500 $null { RDown } { RUp } 40
    Add1 'wrclick' (After $r 40 12000)
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
    $null = DeskClick 1300 780
  }
  Row 'right-click on an icon (menu)' 'Windows desktop' 'wrclick'
  $s = FreeSpot 900 500
  foreach ($k in 1..5) {
    [G]::MoveTo($s[0], $s[1]); Start-Sleep -Milliseconds 400
    [Lat]::Region($s[0] + 20, $s[1] + 20, 160, 120)
    $r = LatWatch 1200 $null { RDown } { RUp } 40
    Add1 'wrbare' (After $r 40 1000)
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Row 'right-click on the bare desktop (menu)' 'Windows desktop' 'wrbare'
  foreach ($k in 1..5) {
    $p = $pts[3]
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 500
    [Lat]::Region($p[0] - 60, $p[1] - 50, 200, 170)
    $x = $p[0]; $y = $p[1]
    $r = LatWatch 900 @(150, ($x + 2), ($y + 2), 200, ($x + 12), ($y + 12), 230, ($x + 24), ($y + 24), 260, ($x + 40), ($y + 40)) { MDown } { [G]::Press(0x1B) } 700
    MUp
    Add1 'wdrag' (After $r 200 6000)
    Start-Sleep -Milliseconds 400
    $null = DeskClick 1300 780
  }
  Row 'press and drag of an icon (drag picture)' 'Windows desktop; from the move that passes the drag threshold' 'wdrag'
  foreach ($k in 1..5) {
    $p = $pts[$k]; CellBox $p
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
    $r = LatWatch 450 $null { [G]::Press(0x71) }
    Add1 'wf2' (After $r 0)
    [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  }
  Row 'F2 (name box)' 'Windows desktop' 'wf2'
  $t = @()
  foreach ($k in 1..3) {
    [G]::MoveTo($dp[0], $dp[1]); Start-Sleep -Milliseconds 400
    MDown; MUp; Start-Sleep -Milliseconds 70
    $sw = [Diagnostics.Stopwatch]::StartNew()
    MDown; MUp
    $ms = -1
    while ($sw.ElapsedMilliseconds -lt 6000) { $h = [G]::FindWindowW('Notepad', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $ms = [int]$sw.ElapsedMilliseconds; break } }
    $t += $ms
    Start-Sleep -Milliseconds 600
    Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep -Milliseconds 700
    $null = DeskClick 1300 780
  }
  LatRow 'double-click on an icon (Notepad window on screen)' 'Windows desktop, a text file' "$(Med $t) (worst $(Worst $t))" '' 1 "window after: $($t -join ' ') ms"
  Start-Process notepad
  $np = WaitWin 'Notepad' 10
  [L]::MoveWindow($np, 500, 300, 500, 300, $true) | Out-Null
  Start-Sleep -Seconds 1
  foreach ($k in 1..5) {
    $x = 760; $y = 315
    [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 500
    [Lat]::Region(470, 380, 60, 60)
    $r = LatWatch 900 @(150, ($x + 2), $y, 200, ($x + 14), $y, 230, ($x + 30), $y, 260, ($x + 50), $y) { MDown } { MUp } 600
    Add1 'wtitle' (After $r 200)
    Start-Sleep -Milliseconds 300
    [L]::MoveWindow($np, 500, 300, 500, 300, $true) | Out-Null
    Start-Sleep -Milliseconds 500
  }
  Row 'press and drag of a window title (first move)' 'a Notepad window; from the move that passes the drag threshold' 'wtitle'
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 500
  # A new file: where the next icon comes is not known beforehand, so the whole first columns are watched.
  [Lat]::Region([int]$o[0], [int]$o[1], 240, 700)
  foreach ($k in 1..5) {
    $f = Join-Path $PfDesk "L4 新文件 $k.txt"
    $r = LatWatch 2500 $null { Set-Content -LiteralPath $f -Value 'x' -Encoding ASCII }
    Add1 'wwatch' (After $r 0)
    Start-Sleep -Milliseconds 500
  }
  Row 'a file created on the desktop (icon there)' 'Windows desktop' 'wwatch'
  foreach ($k in 1..5) {
    $p = $pts[$k]; CellBox $p
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
    [Lat]::Region([int]$o[0], [int]$o[1], 240, 700)
    $r = LatWatch 2500 $null { [G]::Press(0x2E) }
    Add1 'wdelete' (After $r 0)
    Start-Sleep -Milliseconds 800
  }
  Row 'Delete (icon gone)' 'Windows desktop, a shortcut to the Recycle Bin' 'wdelete'
  Case 'cleanup'
  ParkMouse 1300 820
  LtClean
  LtBinClean
  [Anim]::Set([bool]$script:AnimWas)
  Check 'the test files are gone from the desktop and the Recycle Bin' (@(Get-ChildItem "$PfDesk\L4 *" -ErrorAction SilentlyContinue).Count -eq 0 -and (BinNames) -notmatch '(^|, )L4 ') "bin: $(BinNames)"
}

if ($Part -eq 'ACT') { PartACT }
if ($Part -eq 'ACT2') { PartACT2 }
if ($Part -eq 'WIN') { PartWIN }
# What an interrupted run of this file left on the desktop and in the Recycle Bin.
if ($Part -eq 'CLEAN') { Case 'cleanup'; LtClean; LtBinClean; Check 'the test files are gone from the desktop and the Recycle Bin' (@(Get-ChildItem "$PfDesk\L4 *" -ErrorAction SilentlyContinue).Count -eq 0 -and (BinNames) -notmatch '(^|, )L4 ') "bin: $(BinNames)" }

# ---------------------------------------------------------------------------------------------
# Second round. SRCH: the Peek's search box at the first letter, with the app's own log times.
# ROLL: a row of rolled fences under the pointer. MORE: what the first round did not measure.
function LogLines($pos, $utc, $regex) { return ((LogRel $pos $utc ('^(?!.*\] live ).*(' + $regex + ')')) -join ' ; ') }
# The ms after the input of the first log line matching $regex in LogLines' text (-1: none).
function LogAt($text, $regex) { foreach ($l in ($text -split ' ; ')) { if ($l -match "^\+(-?\d+) .*($regex)") { return [int]$Matches[1] } }; return -1 }
function PeekOff() { [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 600; if (([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0) { PeekKey; Start-Sleep -Milliseconds 600 } }

function PartSRCH {
  trap { PfTrap $_; continue }
  Case 'SRCH setup'
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  PfStart -Script @('sleep 3600000')
  Start-Sleep -Seconds 2
  ClearFences
  $links = L4Links 12
  $TI = 'L4 图标'
  Pf fence create --title $TI --rect '300,160,640,330' | Out-Null
  WaitL4 12
  Pf item move @links --to $TI | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 860
  # Where the box comes (and the first search of a run, whose fonts are cold).
  PeekKey; Start-Sleep -Milliseconds 900
  [G]::Type('l'); Start-Sleep -Milliseconds 1200
  $sb = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value)
  $br = Rect $sb; Log "search box $sb at $($br.L),$($br.T),$($br.R),$($br.B)"
  PeekOff
  Check 'SRCH the search box opened' ($br.R -gt $br.L) "box $($br.L),$($br.T),$($br.R),$($br.B)"

  Case 'SRCH the first letter typed in a Peek'
  $drawn = @(); $shows = @(); $notes = @()
  foreach ($k in 1..7) {
    PeekKey; Start-Sleep -Milliseconds 900
    [Lat]::Region($br.L, $br.T, [math]::Min(700, $br.R - $br.L), 320)
    # The key is sent a little after a screen refresh, at another point of the refresh each run.
    Start-Sleep -Milliseconds (4 * $k)
    $pos = LogPos
    $r = LatWatch 800 $null { [G]::Type('l') }
    $m = After $r 0
    Add1 'srch1' $m
    Start-Sleep -Milliseconds 200
    $t = LogLines $pos $r.utc 'peek key kept|search box|search list laid out|slow command'
    $drawn += (LogAt $t 'drawn once'); $shows += (LogAt $t 'search box: shows')
    if ($k -le 2) { $notes += $t }
    $r = LatWatch 600 $null { [G]::Type('4') }
    Add1 'srch2' (After $r 0)
    PeekOff
  }
  foreach ($k in 1..3) {
    PeekKey; Start-Sleep -Milliseconds 900
    [Lat]::Region(312, 210, 596, 200)
    $r = LatWatch 600 $null { [G]::Type('l') }
    Add1 'srchf' (After $r 0)
    PeekOff
  }
  Row 'the first letter typed in a Peek (matches marked in the fence)' 'Peek open' 'srchf'
  Row 'the first letter typed in a Peek (search box, results)' 'Peek open' 'srch1' "log: box drawn at +$($drawn -join ' ') ms, first visible alpha at +$($shows -join ' ') ms | $($notes[0])"
  Row 'the second letter typed in a Peek (results)' 'Peek open' 'srch2'
  $f = @($script:Ms['srch1'] | ForEach-Object { $_.first })
  # A new window is on screen two refreshes after the key on these boxes (0.3.3: 63 to 65 ms; its
  # 34 ms are the marks in the fence its box covered). With the fade started by a timer it was three.
  Check 'SRCH the search box is on screen two refreshes after the first letter, as in 0.3.3 (75 ms at most, 6 of 7 runs)' (@($f | Where-Object { $_ -ge 0 -and $_ -le 75 }).Count -ge 6) "first: $($f -join ' ') ms; the log has the box drawn at +$($drawn -join ' ') ms and visible at +$($shows -join ' ') ms"
  LtEnd
}

if ($Part -eq 'SRCH') { PartSRCH }

# Four rolled fences side by side: 0..3. The point on a title, the region below it.
$RlX = 200, 520, 840, 1160
function RlTitle($k) { return @(($RlX[$k] + 120), 218) }
function RlRegion($k) { [Lat]::Region($RlX[$k] + 20, 244, 200, 190) }
function RlSetup() {
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  PfStart -Script @('sleep 3600000')
  Start-Sleep -Seconds 2
  ClearFences
  Pf settings set rollUp.hoverSpeed normal | Out-Null
  $links = L4Links 12
  WaitL4 12
  foreach ($k in 0..3) {
    $t = "L4 收起 $($k + 1)"
    Pf fence create --title $t --rect "$($RlX[$k]),200,300,240" | Out-Null
    $a = @($links[(3 * $k)..(3 * $k + 2)]); Pf item move @a --to $t | Out-Null
  }
  Start-Sleep -Seconds 2
  foreach ($k in 0..3) { $t = "L4 收起 $($k + 1)"; Pf fence roll $t | Out-Null }
  ParkMouse 1300 860; Start-Sleep -Milliseconds 1500
}
# The pointer jumps onto the title of fence $k; the watch of the region below that title.
function RlHover($k, $ms = 1000) {
  RlRegion $k
  $p = RlTitle $k
  $pos = LogPos
  $r = LatWatch $ms @(0, $p[0], $p[1])
  $m = After $r 0
  Start-Sleep -Milliseconds 150
  $m.log = LogLines $pos $r.utc 'hover expand'
  Log "hover rolled fence ${k}: first=$($m.first) done=$($m.done) | $($m.log)"
  return $m
}
function RlRest() { ParkMouse 1300 860; Start-Sleep -Milliseconds 1500 }

function PartROLL {
  trap { PfTrap $_; continue }
  Case 'ROLL setup: four rolled fences in a row'
  RlSetup
  $rolled = @(0..3 | ForEach-Object { (FenceGet "L4 收起 $($_ + 1)").rolledUp }) -join ' '
  Check 'ROLL four rolled fences' ($rolled -eq 'True True True True') "rolled: $rolled"

  Case 'ROLL the first rolled fence, and the next one after it'
  $log = ''
  foreach ($k in 1..5) {
    RlRest
    $a = (2 * $k) % 4; $b = ($a + 1) % 4
    $m = RlHover $a; Add1 'roll1' $m
    Start-Sleep -Milliseconds 200
    $m = RlHover $b; Add1 'roll2' $m
    if ($k -eq 1) { $log = $m.log }
  }
  Row 'a rolled fence under the pointer (expands)' 'the first one, 展开快慢 默认' 'roll1'
  Row 'the next rolled fence after one expanded under the pointer' 'four rolled fences in a row' 'roll2' $log
  $f = @($script:Ms['roll2'] | ForEach-Object { $_.first })
  Check 'ROLL after a rolled fence expanded under the pointer, the next one starts to expand within 200 ms (默认)' ((Worst $f) -le 200 -and (Med $f) -gt 0) "first: $($f -join ' ') ms"

  Case 'ROLL what ends the short wait, and what must not change'
  RlRest
  $null = RlHover 0
  RlRest
  $m = RlHover 1
  Check 'ROLL after 1.8 s off the rolled titles the next one waits the full time again' ($m.first -ge 360) "first=$($m.first)"
  RlRest
  $null = RlHover 0
  # A click in the fence that has just expanded (a press on the bare desktop is not seen by the fences).
  [G]::Click($RlX[0] + 200, 400); Start-Sleep -Milliseconds 200
  $m = RlHover 2
  Check 'ROLL after a click in a fence the next rolled fence waits the full time again' ($m.first -ge 360) "first=$($m.first)"
  RlRest
  # From outside, across a rolled title in 60 ms: nothing expands.
  RlRegion 1
  $p = RlTitle 1
  $r = LatWatch 900 @(0, $p[0], 150, 20, $p[0], 190, 40, $p[0], 218, 100, $p[0], 260, 120, $p[0], 400, 140, 1300, 860)
  $m = After $r 0
  Check 'ROLL a pointer that crosses a rolled title in 60 ms expands nothing' ($m.first -lt 0) "first=$($m.first)"
  # The roll-up after the pointer left keeps its wait.
  RlRest
  $null = RlHover 0
  RlRegion 0
  $r = LatWatch 1300 @(0, 1300, 860)
  # From the first change of 1000 pixels: a few pixels of the title's hover fade lie in the region.
  $m = After $r 0 1000
  LatRow 'the pointer leaves an expanded fence (rolls up)' 'after a hover expand' $m.first $m.done $r.per ''
  Check 'ROLL the fence rolls up 400 ms after the pointer left, as before' ($m.first -ge 360 -and $m.first -le 560) "first=$($m.first) done=$($m.done)"

  if ($Rec) {
    Case "ROLL sweep recorded ($Rec)"
    . C:\pf\kit\rec.ps1
    RlRest
    $moves = New-Object 'Collections.Generic.List[int]'
    $t = 0
    foreach ($k in 0..3) {
      $p = RlTitle $k
      if ($k -gt 0) { $q = RlTitle ($k - 1); foreach ($s in 1..6) { $moves.Add($t + 10 * $s); $moves.Add([int]($q[0] + ($p[0] - $q[0]) * $s / 6)); $moves.Add(218) }; $t += 60 }
      else { $moves.Add(0); $moves.Add($p[0]); $moves.Add($p[1]) }
      $t += $(if ($k -eq 0) { 800 } else { 300 })
    }
    $t += 700
    RecStart "ROLL-$Rec" -X 180 -Y 170 -W 1300 -H 290 -Fps 30
    RecMark '鼠标先停在第一个收起的栅栏上，再依次移到后面三个上，每个停 0.3 秒'
    Start-Sleep -Milliseconds 300
    [Lat]::Play($moves.ToArray(), $t)
    RecMark '停下'
    Start-Sleep -Milliseconds 500
    RecStop
    Check "ROLL recorded the sweep ($Rec)" $true ''
  }
  LtEnd
}

# FILL: what the first draws after a hover and after a click show of the fill (the app's log).
function FillLines($pos, $utc) {
  $out = @()
  foreach ($l in @(LogRel $pos $utc 'fill drawn')) {
    if ($l -match '^\+(-?\d+) fill drawn hover=([\d.]+) selection=([\d.]+) next_refresh=Some\(\(([\d.]+), ([\d.]+)\)\)') {
      $out += , @([int]$Matches[1], [double]$Matches[2], [double]$Matches[3], [double]$Matches[4], [double]$Matches[5])
    }
  }
  return , $out
}
# From the fill lines of one input: the first draw (ms, level), the first draw with a visible
# fill (ms) and the refresh that shows it (ms after the input). $col: 1 hover, 2 selection.
function FillSum($lines, $col, $from = 0) {
  $o = @{ first = -1; level = -1; visible = -1; onscreen = -1; refresh = 0 }
  foreach ($l in $lines) {
    if ($l[0] -lt $from) { continue }
    if ($o.first -lt 0) { $o.first = $l[0]; $o.level = $l[$col] }
    if ($l[$col] -gt 0.001) { $o.visible = $l[0]; $o.onscreen = [int]($l[0] + $l[3]); $o.refresh = $l[4]; break }
  }
  return $o
}
function PartFILL {
  trap { PfTrap $_; continue }
  Case 'FILL setup'
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  PfStart -Script @('sleep 3600000')
  Start-Sleep -Seconds 2
  ClearFences
  $links = L4Links 12
  $TI = 'L4 图标'
  Pf fence create --title $TI --rect '300,160,640,330' | Out-Null
  WaitL4 12
  Pf item move @links --to $TI | Out-Null
  Start-Sleep -Seconds 2
  Check 'FILL a fence with 12 icons' (@(PfJson item list --fence $TI | Where-Object { $_ -and $_.fileName }).Count -eq 12) ''

  Case 'FILL the hover fill and the selection fill in the first draws after the input'
  $hl = @(); $hv = @(); $hs = @(); $sl = @(); $sv = @(); $ss = @(); $hgrab = @(); $sgrab = @(); $refresh = 0
  foreach ($k in 1..8) {
    ParkMouse 1300 820; Start-Sleep -Milliseconds 500
    $p = CellPt $TI $k; CellBox $p
    Start-Sleep -Milliseconds (4 * $k)
    $pos = LogPos
    $r = LatWatch 300 @(0, $p[0], $p[1])
    $hgrab += (After $r 0).first
    Start-Sleep -Milliseconds 100
    $s = FillSum (FillLines $pos $r.utc) 1
    $hl += $s.level; $hv += $s.visible; $hs += $s.onscreen; if ($s.refresh) { $refresh = $s.refresh }
    Start-Sleep -Milliseconds (3 * $k)
    $pos = LogPos
    $r = LatWatch 300 $null { MDown } { MUp } 40
    $sgrab += (After $r 0).first
    Start-Sleep -Milliseconds 100
    $s = FillSum (FillLines $pos $r.utc) 2
    $sl += $s.level; $sv += $s.visible; $ss += $s.onscreen
    $null = DeskClick 1300 780
  }
  LatRow 'hover fill: the first draw after the pointer came, the first draw with a visible fill, the refresh that shows it' 'app log, 8 runs' "$(Med $hv) (worst $(Worst $hv))" "$(Med $hs) (worst $(Worst $hs))" $refresh "level of the first draw: $($hl -join ' ') | first visible draw at +$($hv -join ' ') ms | on screen at +$($hs -join ' ') ms | screen grabs: $($hgrab -join ' ')"
  LatRow 'selection fill: the first draw after the press, the first draw with a visible fill, the refresh that shows it' 'app log, 8 runs' "$(Med $sv) (worst $(Worst $sv))" "$(Med $ss) (worst $(Worst $ss))" $refresh "level of the first draw: $($sl -join ' ') | first visible draw at +$($sv -join ' ') ms | on screen at +$($ss -join ' ') ms | screen grabs: $($sgrab -join ' ')"
  Check 'FILL the first draw after the pointer came shows the hover fill' (@($hl | Where-Object { $_ -gt 0.001 }).Count -ge 7) "level of the first draw: $($hl -join ' '); first visible draw at +$($hv -join ' ') ms"
  Check 'FILL the first draw after the press shows the selection fill' (@($sl | Where-Object { $_ -gt 0.001 }).Count -ge 7) "level of the first draw: $($sl -join ' '); first visible draw at +$($sv -join ' ') ms"
  LtEnd
}

if ($Part -eq 'ROLL') { PartROLL }
if ($Part -eq 'FILL') { PartFILL }

# MORE: what the first round left unmeasured. Two file changes within 0.4 s, a new match in a
# smart fence, the whole-computer rows of the Peek search (with Everything), a page change in
# the settings window; MORE2: the first right-click menu and the first Peek after the app's start.
function MoreSetup($withFolders) {
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  $script:MDir = Join-Path $PfRun 'L4文件夹'; $script:MSmart = Join-Path $PfRun 'L4智能'
  if ($withFolders) {
    New-Item -ItemType Directory -Force $script:MDir, $script:MSmart | Out-Null
    foreach ($k in 1..10) { Set-Content -LiteralPath (Join-Path $script:MDir ("a{0:d2} 报告.txt" -f $k)) -Value 'x' -Encoding ASCII }
    foreach ($k in 1..4) { Set-Content -LiteralPath (Join-Path $script:MSmart ("b{0:d2} 纪要.txt" -f $k)) -Value 'x' -Encoding ASCII }
  }
  PfStart -Script @('sleep 3600000')
  Start-Sleep -Seconds 2
  ClearFences
  $links = L4Links 12
  Pf fence create --title 'L4 图标' --rect '300,160,640,330' | Out-Null
  WaitL4 12
  Pf item move @links --to 'L4 图标' | Out-Null
  if ($withFolders) {
    Pf fence create --portal $script:MDir --title 'L4 文件夹' --rect '980,340,420,500' | Out-Null
    Pf fence create --title 'L4 智能' --smart-folder $script:MSmart --rect '980,40,420,280' | Out-Null
  }
  Start-Sleep -Seconds 3
  ParkMouse 800 860
}
function MoreEnd() {
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  if ($script:MDir) { Remove-Item -LiteralPath $script:MDir, $script:MSmart -Recurse -Force -ErrorAction SilentlyContinue }
  LtEnd
}

function PartMORE {
  trap { PfTrap $_; continue }
  Case 'MORE setup'
  . C:\pf\kit\smart.ps1
  MoreSetup $true
  $TP = 'L4 文件夹'; $TS = 'L4 智能'
  $wp = (FenceGet $TP).windowRect; $ws = (FenceGet $TS).windowRect
  Check 'MORE fences are there' ($wp.w -eq 420 -and $ws.w -eq 420) "folder $($wp.x),$($wp.y) smart $($ws.x),$($ws.y): $(SmartLine $TS)"

  Case 'MORE two file changes within 0.4 s'
  $note = ''
  foreach ($k in 1..3) {
    $n = 10 + 3 * ($k - 1)
    $fa = Join-Path $script:MDir "z${k}a 甲.txt"; $fb = Join-Path $script:MDir "z${k}b 乙.txt"; $fc = Join-Path $script:MDir "z${k}c 丙.txt"
    # The second file's own place: its first change after the file was made (200 ms after the first).
    $p = CellPt $TP ($n + 1); CellBox $p
    $pos = LogPos
    $r = LatWatch 1600 $null { Set-Content -LiteralPath $fa -Value 'x' -Encoding ASCII } { Set-Content -LiteralPath $fb -Value 'x' -Encoding ASCII } 200
    Add1 'two2' (After $r 200)
    if ($k -eq 1) { $note = LogLines $pos $r.utc 'fs change batch' }
    Start-Sleep -Milliseconds 1500
    # A file alone, long after the last change.
    $p = CellPt $TP ($n + 2); CellBox $p
    $r = LatWatch 1200 $null { Set-Content -LiteralPath $fc -Value 'x' -Encoding ASCII }
    Add1 'two1' (After $r 0)
    Start-Sleep -Milliseconds 1500
  }
  Row 'a file created in a watched folder (icon there)' 'alone, in its own cell' 'two1'
  Row 'a second file created 0.2 s after another (icon there)' 'from the second file' 'two2' $note
  $f2 = @($script:Ms['two2'] | ForEach-Object { $_.first })
  Check 'MORE a second file created 0.2 s after another shows its icon within 200 ms too' ((Worst $f2) -le 200 -and (Med $f2) -gt 0) "first: $($f2 -join ' ') ms"

  Case 'MORE a new match in a smart fence'
  [Lat]::Region($ws.x + 12, $ws.y + 50, $ws.w - 44, $ws.h - 70)
  $ev = EvRunning
  foreach ($k in 1..3) {
    $f = Join-Path $script:MSmart "c${k} 新纪要.txt"
    $pos = LogPos
    $r = LatWatch 6000 $null { Set-Content -LiteralPath $f -Value 'x' -Encoding ASCII }
    Add1 'smart' (After $r 0)
    if ($k -eq 1) { $note = LogLines $pos $r.utc 'smart fence' }
    Start-Sleep -Milliseconds 1000
  }
  Row 'a file created in a smart fence''s folder (icon there)' "Everything running: $ev" 'smart' $note

  Case 'MORE a page change in the settings window'
  $tr = TrayIconRect
  if ($tr) {
    $tx = [int]($tr.X + $tr.Width / 2); $ty = [int]($tr.Y + $tr.Height / 2)
    [G]::Click($tx, $ty); Start-Sleep -Seconds 5
    $hs = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value)
    $rr = Rect $hs; Log "settings window $hs at $($rr.L),$($rr.T),$($rr.R),$($rr.B)"
    $a = OcrFind '^About$' $rr.L $rr.T 300 ($rr.B - $rr.T)
    $g = OcrFind '^General$' $rr.L $rr.T 300 ($rr.B - $rr.T)
    Log "nav: About at $($a -join ','), General at $($g -join ',')"
    if ($a -and $g) {
      [Lat]::Region($rr.L + 320, $rr.T + 80, [math]::Min(600, $rr.R - $rr.L - 360), 300)
      foreach ($k in 1..6) {
        $t = if ($k % 2) { $a } else { $g }
        [G]::MoveTo($t[0], $t[1]); Start-Sleep -Milliseconds 700
        $r = LatWatch 900 $null { MDown } { MUp } 40
        Add1 'page' (After $r 40 2000)
      }
    }
    $res = [IntPtr]::Zero
    [G]::SendMessageTimeoutW($hs, 0x10, [IntPtr]0, [IntPtr]0, 2, 5000, [ref]$res) | Out-Null
    Start-Sleep -Milliseconds 800
  }
  Row 'another page clicked in the settings window' 'General and About; from the release' 'page'

  Case 'MORE the whole-computer rows of the Peek search'
  if (EvStart) {
    Pf settings set tools.everything true | Out-Null
    Start-Sleep -Seconds 3
    $shown = @(); $asked = @(); $answered = @(); $note = ''
    foreach ($k in 1..4) {
      PeekKey; Start-Sleep -Milliseconds 900
      [G]::Type('notepa'); Start-Sleep -Milliseconds 900
      $sb = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value)
      $br = Rect $sb
      [Lat]::Region($br.L, $br.T, [math]::Min(700, $br.R - $br.L), 420)
      $pos = LogPos
      $r = LatWatch 1200 $null { [G]::Type('d') }
      $m = After $r 0
      Start-Sleep -Milliseconds 200
      $t = LogLines $pos $r.utc 'Everything|search list laid out'
      $asked += (LogAt $t 'Everything is asked|asks Everything|Everything asked'); $answered += (LogAt $t 'Everything answered')
      $shown += $m.done
      if ($k -eq 1) { $note = $t }
      Add1 'evrows' $m
      PeekOff
    }
    Row 'a letter typed in a Peek, the whole-computer rows (Everything)' 'the last letter of notepad' 'evrows' "log: Everything answered at +$($answered -join ' ') ms | $note"
    Pf settings set tools.everything false | Out-Null
  } else {
    LatRow 'a letter typed in a Peek, the whole-computer rows (Everything)' 'Everything is not on this box' 'not measured' '' '' ''
  }
  MoreEnd
}

# Starts the test instance again without the kit's waits; returns the ms until the fence window is there.
function QuickStart($title) {
  $env:PECOFENCE_INSTANCE = $PfInst; $env:RUST_LOG = 'pecofence=debug'
  [IO.File]::WriteAllLines("$PfRun\script.txt", @('sleep 3600000'), (New-Object Text.UTF8Encoding($false)))
  $sw = [Diagnostics.Stopwatch]::StartNew()
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable', '--test-script', "$PfRun\script.txt" -WorkingDirectory $PfApp
  while ($sw.ElapsedMilliseconds -lt 20000) {
    $h = [G]::FindWindowW('PecoFence.Fence', $title)
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { break }
    Start-Sleep -Milliseconds 20
  }
  return [int]$sw.ElapsedMilliseconds
}
function PartMORE2 {
  trap { PfTrap $_; continue }
  Case 'MORE2 setup'
  MoreSetup $false
  $TI = 'L4 图标'
  $p = CellPt $TI 2

  Case 'MORE2 the first right-click menu after the app''s start'
  $firsts = @(); $later = @(); $shownAt = @(); $note = ''
  foreach ($k in 1..3) {
    PfQuit
    Start-Sleep -Milliseconds 800
    $ms = QuickStart $TI
    $shownAt += $ms
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300
    [Lat]::Region($p[0] + 60, $p[1] + 10, 160, 240)
    $pos = LogPos
    $r = LatWatch 4000 $null { RDown } { RUp } 40
    $m = After $r 40 12000
    $firsts += $m.first
    if ($k -eq 1) { $note = LogLines $pos $r.utc 'slow command|menu|shell' ; [Lat]::Strip("$PfShots\act-firstmenu.png", 10) }
    [G]::Press(0x1B); Start-Sleep -Milliseconds 600
    $null = DeskClick 1300 780
    Start-Sleep -Seconds 4
    [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 400
    $r = LatWatch 1500 $null { RDown } { RUp } 40
    $later += (After $r 40 12000).first
    [G]::Press(0x1B); Start-Sleep -Milliseconds 600
    $null = DeskClick 1300 780
  }
  LatRow 'right-click on an icon right after the app''s start (menu)' "the fence was on screen $($shownAt -join ' ') ms after the start; the click 0.3 s later" "$(Med $firsts) (worst $(Worst $firsts))" '' 31 "first: $($firsts -join ' ') | the same icon 5 s later: $($later -join ' ') | $note"
  Check 'MORE2 the first menu after the start came' ((Med $firsts) -gt 0) "first: $($firsts -join ' ') later: $($later -join ' ')"

  Case 'MORE2 the first Peek after the app''s start'
  Start-Process notepad
  $np = WaitWin 'Notepad' 10
  [L]::MoveWindow($np, 250, 120, 800, 450, $true) | Out-Null
  Start-Sleep -Seconds 1
  $f1 = @(); $d1 = @(); $f2 = @(); $d2 = @(); $note = ''
  foreach ($k in 1..3) {
    PfQuit
    Start-Sleep -Milliseconds 800
    $null = QuickStart $TI
    Start-Sleep -Seconds 3
    [G]::Click(700, 400); Start-Sleep -Milliseconds 500
    ParkMouse 1300 860
    [Lat]::Region(340, 240, 300, 150)
    $pos = LogPos
    $r = LatWatch 1200 $null { PeekKey }
    $m = After $r 0; $f1 += $m.first; $d1 += $m.done
    if ($k -eq 1) { $note = LogLines $pos $r.utc 'peek|slow command' }
    Start-Sleep -Milliseconds 600
    PeekKey; Start-Sleep -Milliseconds 900
    $r = LatWatch 1200 $null { PeekKey }
    $m = After $r 0; $f2 += $m.first; $d2 += $m.done
    Start-Sleep -Milliseconds 600
    PeekKey; Start-Sleep -Milliseconds 900
  }
  LatRow 'the Peek hotkey, the first time after the app''s start' 'a Notepad window over the fence, 3 s after the start' "$(Med $f1) (worst $(Worst $f1))" "$(Med $d1) (worst $(Worst $d1))" 31 "first: $($f1 -join ' ') done: $($d1 -join ' ') | $note"
  LatRow 'the Peek hotkey, the second time' 'a Notepad window over the fence' "$(Med $f2) (worst $(Worst $f2))" "$(Med $d2) (worst $(Worst $d2))" 31 "first: $($f2 -join ' ') done: $($d2 -join ' ')"
  Check 'MORE2 the first Peek after the start came' ((Med $f1) -gt 0) "first: $($f1 -join ' ') second: $($f2 -join ' ')"
  MoreEnd
}

if ($Part -eq 'MORE') { PartMORE }
if ($Part -eq 'MORE2') { PartMORE2 }

# COL: rolled fences one below the other (the open one pushes the ones below down, so the next
# title is found from the fence's place at that moment).
function PartCOL {
  trap { PfTrap $_; continue }
  Case 'COL setup: four rolled fences in a column'
  [Lat]::Tol = 1
  CloseWindows
  $script:AnimWas = [Anim]::Get(); [Anim]::Set($true)
  LtClean
  PfStart -Script @('sleep 3600000')
  Start-Sleep -Seconds 2
  ClearFences
  Pf settings set rollUp.hoverSpeed normal | Out-Null
  $links = L4Links 12
  WaitL4 12
  foreach ($k in 0..3) {
    $t = "L4 竖排 $($k + 1)"
    Pf fence create --title $t --rect "500,$(120 + 48 * $k),300,240" | Out-Null
    $a = @($links[(3 * $k)..(3 * $k + 2)]); Pf item move @a --to $t | Out-Null
  }
  Start-Sleep -Seconds 2
  foreach ($k in 0..3) { $t = "L4 竖排 $($k + 1)"; Pf fence roll $t | Out-Null; Start-Sleep -Milliseconds 300 }
  ParkMouse 1300 860; Start-Sleep -Milliseconds 1500
  $r0 = @(0..3 | ForEach-Object { $w = (FenceGet "L4 竖排 $($_ + 1)").windowRect; "$($w.y)+$($w.h)" }) -join ' '
  Check 'COL four rolled fences one below the other' ($true) "top+height: $r0"

  Case 'COL the first rolled fence, then the one below it'
  $log = ''
  foreach ($k in 1..3) {
    ParkMouse 1300 860; Start-Sleep -Milliseconds 1800
    $w = (FenceGet 'L4 竖排 1').windowRect
    [Lat]::Region($w.x + 20, $w.y + 44, 200, 150)
    $r = LatWatch 1000 @(0, ($w.x + 120), ($w.y + 18))
    Add1 'col1' (After $r 0 1000)
    # Where the second fence's title is now (pushed down by the open one).
    $w2 = (FenceGet 'L4 竖排 2').windowRect
    [Lat]::Region($w2.x + 20, $w2.y + 44, 200, 150)
    $pos = LogPos
    $r = LatWatch 1000 @(0, ($w2.x + 120), ($w2.y + 18))
    $m = After $r 0 1000
    Add1 'col2' $m
    Start-Sleep -Milliseconds 150
    if ($k -eq 1) { $log = "second title at y=$($w2.y) | " + (LogLines $pos $r.utc 'hover expand') }
  }
  Row 'a rolled fence under the pointer (expands)' 'the first of a column, 展开快慢 默认' 'col1'
  Row 'the next rolled fence after one expanded under the pointer' 'the one below it in a column' 'col2' $log
  LtEnd
}

if ($Part -eq 'COL') { PartCOL }
