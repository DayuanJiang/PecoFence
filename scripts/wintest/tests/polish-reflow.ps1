# Fence polish 2026-11-12, package reflow (motion review MC3).
#   Part A  an edge dragged across a column boundary: the icons glide to their new cells (right
#           edge, left edge, two boundaries in a row), and snap with Windows' animations off.
#   Part P  300 files: what one resize step and the glide frames cost (numbers, one loose bound).
# run.sh --var Part=A | P; default A. Runs on a 1600x900 box.
# The marker is a folder among text files: its yellow icon is found in every recorded frame.
if (-not $Part) { $Part = 'A' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System; using System.Collections.Generic; using System.Drawing; using System.Drawing.Imaging;
using System.IO; using System.Runtime.InteropServices;
public static class Flow {
  static List<double[]> Index(string dir) {
    var rows = new List<double[]>();
    foreach (var line in File.ReadAllLines(Path.Combine(dir, "frames.tsv"))) {
      var p = line.Split('\t'); if (p.Length < 2) continue;
      rows.Add(new double[] { double.Parse(p[0]), double.Parse(p[1]) });
    }
    return rows;
  }
  static int[] Load(string dir, double index, out int w, out int h) {
    using (var b = new Bitmap(Path.Combine(dir, ((int)index).ToString("D5") + ".jpg"))) {
      w = b.Width; h = b.Height;
      var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
      var px = new int[w * h]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d);
      return px;
    }
  }
  // The yellow folder icon in every frame of a rec.ps1 recording: { ms, x, y, pixels }.
  public static double[][] Track(string dir) {
    var rows = new List<double[]>();
    foreach (var f in Index(dir)) {
      int w, h; var px = Load(dir, f[0], out w, out h);
      long sx = 0, sy = 0, n = 0;
      for (int y = 0; y < h; y++) for (int x = 0; x < w; x++) {
        int c = px[y * w + x], r = (c >> 16) & 255, g = (c >> 8) & 255, bl = c & 255;
        if (r > 190 && g > 140 && bl < 120 && g - bl > 50) { sx += x; sy += y; n++; }
      }
      rows.Add(new double[] { f[1], n == 0 ? -1 : (double)sx / n, n == 0 ? -1 : (double)sy / n, n });
    }
    return rows.ToArray();
  }
  static double Diff(int[] a, int[] b, int w, int y0, int y1) {
    double sum = 0; long n = 0;
    for (int y = y0; y < y1; y++) for (int x = 0; x < w; x++) {
      int p = a[y * w + x], q = b[y * w + x];
      int lp = (((p >> 16) & 255) * 299 + ((p >> 8) & 255) * 587 + (p & 255) * 114) / 1000;
      int lq = (((q >> 16) & 255) * 299 + ((q >> 8) & 255) * 587 + (q & 255) * 114) / 1000;
      sum += Math.Abs(lp - lq); n++;
    }
    return n == 0 ? 0 : sum / n;
  }
  // Frames between two marks that show a blend: the rows y0..y1 differ from the picture before
  // the first mark and from the last picture before the second one by more than a fifth of what
  // those two differ. { how much old and new differ, blended frames, frames } and, per blended
  // frame, "ms after the mark: share of the way" in `way`.
  public static double[] Blend(string dir, double t0, double t1, int y0, int y1, out string way) {
    way = "";
    var idx = Index(dir);
    double[] before = null; var inside = new List<double[]>();
    // A frame's time is when its grab began; the grab takes about 30 ms, so a frame stamped just
    // before a mark may already show what came after it.
    foreach (var f in idx) { if (f[1] < t0 - 40) before = f; else if (f[1] < t1 - 60) inside.Add(f); }
    if (before == null || inside.Count == 0) return new double[] { 0, 0, 0 };
    int w, h; var old = Load(dir, before[0], out w, out h);
    var last = Load(dir, inside[inside.Count - 1][0], out w, out h);
    y1 = Math.Min(y1, h);
    double D = Diff(old, last, w, y0, y1); int blended = 0;
    foreach (var f in inside) {
      var px = Load(dir, f[0], out w, out h);
      double a = Diff(px, old, w, y0, y1), b = Diff(px, last, w, y0, y1);
      if (D > 0.5 && Math.Min(a, b) > 0.2 * D) {
        blended++;
        way += string.Format("{0:F0}ms:{1:F0}% ", f[1] - t0, 100 * a / (a + b));
      }
    }
    return new double[] { D, blended, inside.Count };
  }
}
'@

$RfWords = '季度报告', '年度预算', '工作计划', '会议记录', '新年活动安排与分工', '文件整理', '图片素材', '报销单', '发票', '合同', '项目进度', '说明'
$RfPortal = "$env:USERPROFILE\polish-reflow-portal"
$RfBig = "$env:USERPROFILE\polish-reflow-300"
function RfClean() {
  Get-ChildItem $PfDesk -Filter 'rf *' -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -Recurse -Force $RfPortal, $RfBig -ErrorAction SilentlyContinue
}
function RfRect($t) { return (FenceGet $t).windowRect }
function RfMarks($name) { return @(Get-Content "$PfShots\rec-$name\marks.tsv" | ForEach-Object { [double]($_.Split("`t")[0]) }) }
# The marker's way between mark $k and the next one: how far it went, and in how many frames
# it was seen on the way (more than 10 px from where it started and from where it rests).
function MarkerWay($track, $marks, $k) {
  $t0 = $marks[$k]; $t1 = 1e12
  if ($k + 1 -lt $marks.Count) { $t1 = $marks[$k + 1] }
  $seen = @($track | Where-Object { $_[3] -gt 80 })
  # A frame's time is when its grab began and the grab takes about 30 ms: a frame stamped just
  # before a mark may already show what came after it.
  $before = @($seen | Where-Object { $_[0] -lt $t0 - 40 })
  $in = @($seen | Where-Object { $_[0] -ge $t0 - 40 -and $_[0] -lt $t1 - 60 })
  if ($before.Count -eq 0 -or $in.Count -eq 0) { return @{ travel = 0; between = 0; way = 'marker not seen' } }
  $p0 = $before[$before.Count - 1]; $p1 = $in[$in.Count - 1]
  $travel = [math]::Sqrt([math]::Pow($p1[1] - $p0[1], 2) + [math]::Pow($p1[2] - $p0[2], 2))
  $between = 0; $way = @()
  foreach ($f in $in) {
    $d0 = [math]::Sqrt([math]::Pow($f[1] - $p0[1], 2) + [math]::Pow($f[2] - $p0[2], 2))
    $d1 = [math]::Sqrt([math]::Pow($f[1] - $p1[1], 2) + [math]::Pow($f[2] - $p1[2], 2))
    if ($d0 -gt 10 -and $d1 -gt 10) { $between++; $way += ('{0:F0}ms:{1:F0},{2:F0}' -f ($f[0] - $t0), $f[1], $f[2]) }
  }
  return @{ travel = $travel; between = $between; way = ('from {0:F0},{1:F0} to {2:F0},{3:F0} ({4:F0} px), on the way in {5} frames [{6}]' -f $p0[1], $p0[2], $p1[1], $p1[2], $travel, $between, ($way -join ' ')) }
}
# Drags one edge of 资料 (420 wide, five columns of 80): 12 px in (still five), in one move on
# to 380 (four), back to 408 (five), then in two moves 20 ms apart to 290 (three), and lets go.
# One move per boundary: with the left edge the whole content moves with every mouse step.
function EdgeRun($name, $edge) {
  Pf fence move 资料 --rect '300,140,420,560' | Out-Null; Start-Sleep -Milliseconds 900
  $r = RfRect '资料'
  $ex = $r.x + $r.w - 2; $dir = -1
  if ($edge -eq 'left') { $ex = $r.x + 1; $dir = 1 }
  $ey = $r.y + 300
  ParkMouse 1300 760
  RecStart $name -X ($r.x - 20) -Y ($r.y - 10) -W ($r.w + 40) -H ($r.h + 20) -Fps 60
  Start-Sleep -Milliseconds 300
  [G]::MoveTo($ex, $ey); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  RecMark 'same columns'
  Glide $ex $ey ($ex + $dir * 12) $ey 6 30
  Start-Sleep -Milliseconds 400
  RecMark 'one column fewer'
  Glide ($ex + $dir * 12) $ey ($ex + $dir * 40) $ey 1 15
  Start-Sleep -Milliseconds 800
  RecMark 'one column more'
  Glide ($ex + $dir * 40) $ey ($ex + $dir * 12) $ey 1 15
  Start-Sleep -Milliseconds 800
  RecMark 'two columns fewer'
  Glide ($ex + $dir * 12) $ey ($ex + $dir * 130) $ey 2 20
  Start-Sleep -Milliseconds 800
  RecMark 'release'
  MUp; Start-Sleep -Milliseconds 600
  RecStop
  $track = [Flow]::Track("$PfShots\rec-$name")
  $marks = RfMarks $name
  Log "${name}: $(Get-Content "$PfShots\rec-$name\stats.txt")"
  return @{ track = $track; marks = $marks }
}

Case 'setup'
CloseWindows
RfClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
# An RDP session shows only an outline while a window is sized: show the contents.
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
$logLevel = 'pecofence=debug'
if ($Part -match 'P') { $logLevel = 'pecofence=debug,pecofence::render_timing=trace' }
PfStart -Script @('sleep 3600000') -Lang 'zh-CN' -LogLevel $logLevel
ClearFences | Out-Null

if ($Part -match 'A') {
  # Fourteen text files and one folder, by name: the folder is the fifth icon, the last of the
  # first row with five columns and the first of the second row with four.
  foreach ($i in 1..15) {
    if ($i -eq 5) { New-Item -ItemType Directory -Force (Join-Path $PfDesk 'rf 05 标记文件夹') | Out-Null; continue }
    Set-Content -LiteralPath (Join-Path $PfDesk ('rf {0:d2} {1}.txt' -f $i, $RfWords[$i % 12])) 'x'
  }
  Pf fence create --title 资料 --rect '300,140,420,560' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move --glob 'rf *' --to 资料 | Out-Null
  Pf fence set 资料 sort name | Out-Null
  Pf fence set 资料 foldersFirst false | Out-Null
  Start-Sleep -Seconds 2
  $order = @(@(PfJson item list --fence 资料) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name })
  Check 'setup: fifteen icons, the folder fifth' ($order.Count -eq 15 -and $order[4] -match '标记') "order: $($order -join ', ')"
  Shot 'rf-setup'
}

if ($Part -match 'A') {
  foreach ($edge in 'right', 'left') {
    Case "A $edge edge dragged across column boundaries"
    $run = EdgeRun "edge-$edge" $edge
    $same = MarkerWay $run.track $run.marks 0
    Check "A $edge edge, columns kept: the folder icon stays in its cell" ($same.travel -lt 30 -and $same.between -eq 0) $same.way
    $fewer = MarkerWay $run.track $run.marks 1
    Check "A $edge edge, five to four columns: the folder icon is seen on its way to the next row" ($fewer.travel -gt 150 -and $fewer.between -ge 2) $fewer.way
    $more = MarkerWay $run.track $run.marks 2
    Check "A $edge edge, four to five columns: the folder icon is seen on its way back" ($more.travel -gt 150 -and $more.between -ge 2) $more.way
    $two = MarkerWay $run.track $run.marks 3
    Check "A $edge edge, five to three columns in one move: the folder icon is seen on its way" ($two.travel -gt 100 -and $two.between -ge 2) $two.way
    $f = FenceGet '资料'
    Check "A $edge edge: three columns after the drag" ($f.fit.columns -eq 3) "columns=$($f.fit.columns) window=$($f.windowRect.x),$($f.windowRect.y),$($f.windowRect.w),$($f.windowRect.h)"
  }
  Case 'A a slow drag, for the eye'
  # No checks: the recording shows the icons flowing while the edge moves at hand speed.
  Pf fence move 资料 --rect '300,140,420,560' | Out-Null; Start-Sleep -Milliseconds 900
  $r = RfRect '资料'
  $ex = $r.x + $r.w - 2; $ey = $r.y + 300
  ParkMouse 1300 760
  RecStart 'edge-demo' -X ($r.x - 20) -Y ($r.y - 10) -W ($r.w + 40) -H ($r.h + 20) -Fps 60
  Start-Sleep -Milliseconds 400
  [G]::MoveTo($ex, $ey); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  RecMark 'narrower'
  Glide $ex $ey ($ex - 175) $ey 50 28
  Start-Sleep -Milliseconds 500
  RecMark 'wider'
  Glide ($ex - 175) $ey $ex $ey 50 28
  Start-Sleep -Milliseconds 500
  RecMark 'release'
  MUp; Start-Sleep -Milliseconds 600
  RecStop
  Check 'A slow drag recorded' $true (Get-Content "$PfShots\rec-edge-demo\stats.txt")

  Case 'A animations off'
  $glides = @(PfLogAll | Where-Object { $_ -match 'resize changed the columns' }).Count
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  $run = EdgeRun 'edge-noanim' 'right'
  $fewer = MarkerWay $run.track $run.marks 1
  Check "A Windows' animations off: the folder icon is in its new cell at once" ($fewer.travel -gt 150 -and $fewer.between -eq 0) $fewer.way
  [Anim]::Set($true); Start-Sleep -Milliseconds 800
  $logged = @(PfLogAll | Where-Object { $_ -match 'resize changed the columns' })
  Check 'A the app logs each glide, and none with animations off' ($glides -ge 10 -and $logged.Count -eq $glides) ("$glides lines before the run with animations off, $($logged.Count) after; last: " + ($logged | Select-Object -Last 1))
}

if ($Part -match 'P') {
  Case 'P 300 files'
  New-Item -ItemType Directory -Force $RfBig | Out-Null
  for ($i = 1; $i -le 300; $i++) { Set-Content -LiteralPath ("$RfBig\{0:d3} {1} 第{2}份.txt" -f $i, $RfWords[$i % 12], $i) 'x' }
  Pf fence create --portal $RfBig --title 三百个文件 --rect '300,140,660,600' | Out-Null
  Pf fence set 三百个文件 sort name | Out-Null
  Start-Sleep -Seconds 5
  $n = @(@(PfJson item list --fence 三百个文件) | Where-Object { $_ -and $_.name }).Count
  $r = RfRect '三百个文件'
  $ex = $r.x + $r.w - 2; $ey = $r.y + 300
  $from = @(PfLogAll).Count
  [G]::MoveTo($ex, $ey); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  foreach ($round in 1..3) {
    Glide $ex $ey ($ex - 300) $ey 60 16
    Glide ($ex - 300) $ey $ex $ey 60 16
  }
  Start-Sleep -Milliseconds 400
  MUp; Start-Sleep -Milliseconds 800
  $lines = @(PfLogAll | Select-Object -Skip $from)
  $draw = @($lines | Where-Object { $_ -match 'resize stages' -and $_ -match 'draw_us=(\d+)' } | ForEach-Object { [void]($_ -match 'draw_us=(\d+)'); [double]$Matches[1] })
  $runs = @($lines | Where-Object { $_ -match 'animation run' -and $_ -match 'max_frame_ms=([\d.]+)' } | ForEach-Object { [void]($_ -match 'max_frame_ms=([\d.]+)'); [double]$Matches[1] })
  $frames = @($lines | Where-Object { $_ -match 'animation run' -and $_ -match 'frames=(\d+)' } | ForEach-Object { [void]($_ -match 'frames=(\d+)'); [double]$Matches[1] })
  $glides = @($lines | Where-Object { $_ -match 'resize changed the columns' }).Count
  $mean = 0; $max = 0
  if ($draw.Count -gt 0) { $mean = ($draw | Measure-Object -Average).Average / 1000; $max = ($draw | Measure-Object -Maximum).Maximum / 1000 }
  $slowest = 0; $fsum = 0
  if ($runs.Count -gt 0) { $slowest = ($runs | Measure-Object -Maximum).Maximum }
  if ($frames.Count -gt 0) { $fsum = ($frames | Measure-Object -Sum).Sum }
  $sorted = @($draw | Sort-Object)
  $p95 = 0
  if ($sorted.Count -gt 0) { $p95 = $sorted[[int][math]::Floor(($sorted.Count - 1) * 0.95)] / 1000 }
  $numbers = ("{0} files; {1} resize steps: redraw mean {2:F2} ms, 95th {3:F2} ms, slowest {4:F2} ms; column changes that glided {5}; animation runs {6} with {7} frames, slowest frame {8:F1} ms" -f $n, $draw.Count, $mean, $p95, $max, $glides, $runs.Count, $fsum, $slowest)
  Log "P numbers: $numbers"
  Check 'P 300 files: a resize step redraws in under 6.9 ms on average (a 144 Hz frame)' ($draw.Count -gt 50 -and $mean -lt 6.9) $numbers
  Check 'P 300 files: no glide frame took longer than a 60 Hz frame' ($slowest -lt 16.7) $numbers
}

Case 'cleanup'
PfQuit
RfClean
[G]::SystemParametersInfoW(0x25, 0, [NullString]::Value, 3) | Out-Null
[Anim]::Set($AnimWas)
