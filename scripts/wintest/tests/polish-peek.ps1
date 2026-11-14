# Fence polish 2026-11-12, package peek.
# MF-5: when a Peek opens, a fence that another program's window hides completely fades in with
# the dimmer (83 ms) instead of popping up in one frame, and fades out with it when the Peek
# ends; a fence on visible desktop, or one only partly hidden, keeps its look the whole time.
# However the Peek ends, every fence is opaque, shown and at rest afterwards.
# MF-12: `pecofence-cli fence fit` glides to the new height like 适应内容大小 in the fence menu.
# Parts (run.sh --var Part=...): A the fade (dimmer on and off, animations off), B every way a
# Peek ends, C the CLI fit. Default: all. Record=1 also records the clips for the report.
# T (not in the default) times ten Peeks: when the covered fence is first seen and fully there
# after the key, and from the app's log when it was made transparent and when it began to fade.
# PkBuild=<folder> runs the build in that folder of --extra instead of the job's (several builds
# in one boot of a guest).
if (-not $Part) { $Part = 'ABC' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1
. C:\pf\kit\vdesk.ps1

Add-Type -ReferencedAssemblies System.Windows.Forms, System.Drawing -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Windows.Forms;
// A plain white window of the job's own process: "another program's window" over the fences.
public static class PkCover {
  static Form f; static Thread t;
  public static IntPtr Handle = IntPtr.Zero;
  public static void Open(int x, int y, int w, int h) {
    var ready = new ManualResetEvent(false);
    t = new Thread(() => {
      f = new Form();
      f.FormBorderStyle = FormBorderStyle.None; f.StartPosition = FormStartPosition.Manual;
      f.Bounds = new Rectangle(x, y, w, h); f.BackColor = Color.White; f.Text = "pk cover";
      f.Shown += (s, e) => { Handle = f.Handle; ready.Set(); };
      Application.Run(f);
    });
    t.SetApartmentState(ApartmentState.STA); t.IsBackground = true; t.Start();
    ready.WaitOne(8000);
  }
  public static void Show(bool on) { if (f != null) f.Invoke((Action)(() => { if (on) f.Show(); else f.Hide(); })); }
  public static void Move(int x, int y, int w, int h) { if (f != null) f.Invoke((Action)(() => { f.Bounds = new Rectangle(x, y, w, h); })); }
  public static void Close() { try { if (f != null) f.Invoke((Action)(() => f.Close())); } catch { } f = null; Handle = IntPtr.Zero; }
}
// Samples small screen patches as fast as the screen can be read (about 30 times a second on
// the boxes). Each value is the patch's mean chroma (max - min of R, G, B per pixel): 0 on white,
// grey and the dimmer over them, high on a yellow folder icon, so it follows how opaque the
// icon is and nothing else.
public static class PkScan {
  static Thread worker; static volatile bool running; static Stopwatch clock = Stopwatch.StartNew();
  static int[] px; static int bx, by, bw, bh;
  public static List<long> T = new List<long>(); public static List<double[]> V = new List<double[]>();
  public static long Now() { return clock.ElapsedMilliseconds; }
  public static void Start(int[] patches) {
    Stop(); px = patches; T = new List<long>(); V = new List<double[]>();
    int x0 = int.MaxValue, y0 = int.MaxValue, x1 = 0, y1 = 0;
    for (int i = 0; i < px.Length; i += 4) { x0 = Math.Min(x0, px[i]); y0 = Math.Min(y0, px[i + 1]); x1 = Math.Max(x1, px[i] + px[i + 2]); y1 = Math.Max(y1, px[i + 1] + px[i + 3]); }
    bx = x0; by = y0; bw = x1 - x0; bh = y1 - y0; running = true;
    worker = new Thread(Loop); worker.IsBackground = true; worker.Start();
  }
  public static void Stop() { if (!running) return; running = false; worker.Join(3000); }
  static void Loop() {
    using (var bmp = new Bitmap(bw, bh, PixelFormat.Format32bppRgb)) using (var g = Graphics.FromImage(bmp)) {
      while (running) {
        try { g.CopyFromScreen(bx, by, 0, 0, new Size(bw, bh)); } catch { Thread.Sleep(5); continue; }
        long t = clock.ElapsedMilliseconds;
        var d = bmp.LockBits(new Rectangle(0, 0, bw, bh), ImageLockMode.ReadOnly, PixelFormat.Format32bppRgb);
        var v = new double[px.Length / 4];
        for (int k = 0; k < v.Length; k++) {
          int x = px[k * 4] - bx, y = px[k * 4 + 1] - by, w = px[k * 4 + 2], h = px[k * 4 + 3];
          var row = new byte[w * 4]; double sum = 0;
          for (int r = 0; r < h; r++) {
            Marshal.Copy(IntPtr.Add(d.Scan0, (y + r) * d.Stride + x * 4), row, 0, row.Length);
            for (int c = 0; c < w; c++) { int b = row[c * 4], gg = row[c * 4 + 1], rr = row[c * 4 + 2]; sum += Math.Max(rr, Math.Max(gg, b)) - Math.Min(rr, Math.Min(gg, b)); }
          }
          v[k] = sum / (w * h);
        }
        bmp.UnlockBits(d);
        lock (T) { T.Add(t); V.Add(v); }
      }
    }
  }
  static List<double> Vals(int k, long from, long to) { var o = new List<double>(); lock (T) { for (int i = 0; i < T.Count; i++) if (T[i] >= from && T[i] <= to) o.Add(V[i][k]); } return o; }
  public static string Series(int k, long from, long to) { var sb = new StringBuilder(); lock (T) { for (int i = 0; i < T.Count; i++) if (T[i] >= from && T[i] <= to) sb.Append((T[i] - from) + ":" + V[i][k].ToString("0") + " "); } return sb.ToString().Trim(); }
  public static double Min(int k, long from, long to) { double m = double.MaxValue; foreach (var v in Vals(k, from, to)) m = Math.Min(m, v); return m == double.MaxValue ? -1 : m; }
  public static double Max(int k, long from, long to) { double m = -1; foreach (var v in Vals(k, from, to)) m = Math.Max(m, v); return m; }
  public static double Last(int k, long from, long to) { var v = Vals(k, from, to); return v.Count == 0 ? -1 : v[v.Count - 1]; }
  // Samples strictly between lo and hi: frames in which the icon was partly opaque.
  public static int Between(int k, long from, long to, double lo, double hi) { int n = 0; foreach (var v in Vals(k, from, to)) if (v > lo && v < hi) n++; return n; }
  // The biggest fall after a rise (a flash while fading in), or with up = false the biggest rise after a fall.
  public static double Reversal(int k, long from, long to, bool up) {
    double worst = 0, peak = up ? double.MinValue : double.MaxValue;
    foreach (var v in Vals(k, from, to)) { if (up) { peak = Math.Max(peak, v); worst = Math.Max(worst, peak - v); } else { peak = Math.Min(peak, v); worst = Math.Max(worst, v - peak); } }
    return worst;
  }
  // First sample time (ms after `from`) at which the value passed `level` going up (or down).
  public static long Reached(int k, long from, long to, double level, bool up) { lock (T) { for (int i = 0; i < T.Count; i++) if (T[i] >= from && T[i] <= to && (up ? V[i][k] >= level : V[i][k] <= level)) return T[i] - from; } return -1; }
}
'@

# ---- setup: three fences with a folder each, a white window over the first, half over the second
$PkNames = @('pk 资料夹甲', 'pk 资料夹乙', 'pk 部分一', 'pk 部分二', 'pk 部分三', 'pk 桌面可见')
function PkClean() {
  PkCover-Close
  Get-ChildItem -LiteralPath $PfDesk -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'pk *' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}
function PkCover-Close() { [PkCover]::Close() }
# The cover window in front (a real click on it), the pointer parked on it away from the fences.
function PkCoverFront() {
  [PkCover]::Show($true); Start-Sleep -Milliseconds 250
  [G]::Click($CoverX + 40, $CoverY + $CoverH - 40); Start-Sleep -Milliseconds 350
}
function PkPeekOn() { return ([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0 }
function PkWaitPeek($on, $sec = 4) { for ($i = 0; $i -lt $sec * 20 -and (PkPeekOn) -ne $on; $i++) { Start-Sleep -Milliseconds 50 }; return ((PkPeekOn) -eq $on) }
# Every fence at rest: shown, no fade, not taken by the Peek, shadow at full alpha.
function PkRest($tag) {
  $bad = @()
  $d = @(PfDump)
  foreach ($w in $d) {
    $s = $w.raw
    # (a build before this change has no peek_faded field)
    # The shadow's alpha is the plate's (0 for the fence arranged like the desktop, which has none).
    $plate = if ($s -match ' plate_alpha=([\d.]+)') { [double]$Matches[1] } else { 1.0 }
    $shadow = if ($s -match ' shadow_alpha=(\d+)') { [int]$Matches[1] } else { -1 }
    $ok = $s -match 'visible=true ' -and $s -match ' fade=none ' -and $s -notmatch ' peek_faded=true' -and $s -match ' opacity=1\.00 ' -and [math]::Abs($shadow - 255 * $plate) -le 1
    if (-not $ok) { $m = [regex]::Match($s, 'visible=\w+'); $bad += "$($w.title): $($m.Value) $([regex]::Match($s, 'opacity=\S+').Value) $([regex]::Match($s, 'fade=\S+').Value) $([regex]::Match($s, 'shadow_alpha=\S+').Value) $([regex]::Match($s, 'peek_faded=\S+').Value)" }
  }
  Log "$tag rest: $($d.Count) fences, bad: $($bad -join ' | ')"
  return @($d.Count, ($bad -join ' | '))
}
# Chroma of the three icons right now (one fresh sample each).
function PkNow() {
  $t = [PkScan]::Now(); Start-Sleep -Milliseconds 220; $e = [PkScan]::Now()
  return @([PkScan]::Last(0, $t, $e), [PkScan]::Last(1, $t, $e), [PkScan]::Last(2, $t, $e))
}
# After a Peek ended some way: nothing floats, every fence is at rest, the covered fence is
# hidden by the window again, and with the window away it shows fully.
function PkAfter($how) {
  Start-Sleep -Milliseconds 700
  Check "${how}: the Peek is over" (-not (PkPeekOn)) "dimmers=$(([W]::Tops('PecoFence.PeekDimmer')).Count)"
  $r = PkRest $how
  Check "${how}: every fence is shown, opaque and at rest" ($r[0] -ge 3 -and $r[1] -eq '') "fences=$($r[0]) $($r[1])"
  $v = PkNow
  Check "${how}: the covered fence is under the window again, the others show" ($v[0] -lt 0.15 * $Ref[0] -and $v[1] -gt 0.85 * $Ref[1] -and $v[2] -gt 0.85 * $Ref[2]) "chroma covered=$('{0:0}' -f $v[0]) partly=$('{0:0}' -f $v[1]) visible=$('{0:0}' -f $v[2]) (shown: $('{0:0}' -f $Ref[0]) $('{0:0}' -f $Ref[1]) $('{0:0}' -f $Ref[2]))"
  [PkCover]::Show($false); Start-Sleep -Milliseconds 400
  $v = PkNow
  Check "${how}: with the window away the covered fence shows fully" ($v[0] -gt 0.9 * $Ref[0]) "chroma=$('{0:0}' -f $v[0]) of $('{0:0}' -f $Ref[0])"
  PkCoverFront
}

Case 'setup'
CloseWindows
PkClean
VdOnlyOne
GreyWallpaper
foreach ($n in $PkNames) { New-Item -ItemType Directory -Force (Join-Path $PfDesk $n) | Out-Null }
Set-Content -LiteralPath (Join-Path $PfDesk 'pk 说明文件.txt') -Value 'peek fade' -Encoding ASCII
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
if ($PkBuild) {
  New-Item -ItemType Directory -Force $PfApp | Out-Null
  Copy-Item "$PfExtra\$PkBuild\*" $PfApp -Force
  Log "build ${PkBuild}: $((Get-FileHash "$PfApp\pecofence.exe").Hash.Substring(0, 12))"
}
PfStart -Script (DumpScript 250 20)
ClearFences | Out-Null
$CoverX = 120; $CoverY = 80; $CoverW = 620; $CoverH = 560
Pf fence create --title Covered --rect '200,160,300,220' | Out-Null
Pf fence create --title Partly --rect '640,420,300,220' | Out-Null
Pf fence create --title Visible --rect '1000,160,300,220' | Out-Null
Pf item move "$PfDesk\pk 资料夹甲" "$PfDesk\pk 资料夹乙" "$PfDesk\pk 说明文件.txt" --to Covered | Out-Null
Pf item move "$PfDesk\pk 部分一" "$PfDesk\pk 部分二" "$PfDesk\pk 部分三" --to Partly | Out-Null
Pf item move "$PfDesk\pk 桌面可见" --to Visible | Out-Null
Pf settings set peek.dim true | Out-Null
Start-Sleep -Seconds 2
ParkMouse 1400 700
$pc = ItemPt 'Covered' 'pk 资料夹甲'
$pp = ItemPt 'Partly' 'pk 部分三'
$pv = ItemPt 'Visible' 'pk 桌面可见'
Check 'setup: the three icons are found' ($pc -and $pp -and $pv) "covered=$($pc -join ',') partly=$($pp -join ',') visible=$($pv -join ',')"
Check 'setup: the partly hidden fence has its third icon beside the window' ($pp[0] -gt $CoverX + $CoverW + 20) "icon x=$($pp[0]) window right=$($CoverX + $CoverW)"
$PkPatches = [int[]]@(($pc[0] - 8), ($pc[1] - 6), 16, 16, ($pp[0] - 8), ($pp[1] - 6), 16, 16, ($pv[0] - 8), ($pv[1] - 6), 16, 16)
[PkScan]::Start($PkPatches)
Start-Sleep -Milliseconds 600
$Ref = PkNow
Shot 'pk-setup'
Check 'setup: the folder icons read as coloured (all three on the desktop)' ($Ref[0] -gt 60 -and $Ref[1] -gt 60 -and $Ref[2] -gt 60) "chroma $('{0:0}' -f $Ref[0]) $('{0:0}' -f $Ref[1]) $('{0:0}' -f $Ref[2])"
[PkCover]::Open($CoverX, $CoverY, $CoverW, $CoverH)
PkCoverFront
$v = PkNow
Check 'setup: the window hides the first fence only' ($v[0] -lt 5 -and $v[1] -gt 0.85 * $Ref[1] -and $v[2] -gt 0.85 * $Ref[2]) "chroma covered=$('{0:0}' -f $v[0]) partly=$('{0:0}' -f $v[1]) visible=$('{0:0}' -f $v[2])"
Shot 'pk-covered'

# One Peek open and close with the shortcut; returns what the samples say.
function PkCycle($tag, $hold = 700) {
  $o = [ordered]@{}
  $t0 = [PkScan]::Now(); PeekKey; Start-Sleep -Milliseconds $hold
  $t1 = [PkScan]::Now(); PeekKey; Start-Sleep -Milliseconds 700
  $t2 = [PkScan]::Now()
  $S = $Ref[0]
  $o.openMid = [PkScan]::Between(0, $t0, $t0 + 500, 0.15 * $S, 0.85 * $S)
  $o.openFlash = [PkScan]::Reversal(0, $t0, $t0 + 500, $true)
  $o.openFull = [PkScan]::Max(0, $t0, $t1)
  $o.openAt = [PkScan]::Reached(0, $t0, $t1, 0.15 * $S, $true)
  $o.fullAt = [PkScan]::Reached(0, $t0, $t1, 0.85 * $S, $true)
  $o.closeMid = [PkScan]::Between(0, $t1, $t1 + 500, 0.15 * $S, 0.85 * $S)
  $o.closeFlash = [PkScan]::Reversal(0, $t1, $t2, $false)
  $o.closeEnd = [PkScan]::Last(0, $t1, $t2)
  $o.goneAt = [PkScan]::Reached(0, $t1, $t2, 0.15 * $S, $false)
  $o.partMin = [PkScan]::Min(1, $t0, $t2)
  $o.visMin = [PkScan]::Min(2, $t0, $t2)
  Log "$tag open:  $([PkScan]::Series(0, $t0, $t0 + 500))"
  Log "$tag close: $([PkScan]::Series(0, $t1, $t1 + 500))"
  Log "$tag partly: $([PkScan]::Series(1, $t0, $t2))"
  Log "$tag visible: $([PkScan]::Series(2, $t0, $t2))"
  return [pscustomobject]$o
}
function PkSummary($runs) {
  return ("opens with a part-way frame {0}/{1}, closes {2}/{1}; first seen {3} ms, full {4} ms after the key; largest step back while opening {5:0}, while closing {6:0}; lowest chroma partly {7:0} visible {8:0}" -f
    @($runs | Where-Object { $_.openMid -ge 1 }).Count, $runs.Count, @($runs | Where-Object { $_.closeMid -ge 1 }).Count,
    (($runs | ForEach-Object { $_.openAt }) -join '/'), (($runs | ForEach-Object { $_.fullAt }) -join '/'),
    ($runs | Measure-Object openFlash -Maximum).Maximum, ($runs | Measure-Object closeFlash -Maximum).Maximum,
    ($runs | Measure-Object partMin -Minimum).Minimum, ($runs | Measure-Object visMin -Minimum).Minimum)
}

if ($Part -match 'T') {
  Case 'T ten Peeks over a covered fence, timed'
  $runs = @(1..10 | ForEach-Object { PkCycle "T$_" })
  $sum = PkSummary $runs
  Start-Sleep -Milliseconds 1500
  # The app's own times: the Peek opens, the covered fence is made transparent, it is raised and
  # begins to fade in (builds before the fade log none of the last two).
  $open = @(); $clear = @(); $fade = @(); $late = 0
  foreach ($line in @(PfLogAll)) {
    if ($line -match 'fences redrawn for the Peek floating=true') { $open += LogTime $line }
    elseif ($line -match 'peek fade: fence made transparent before its raise title=Covered') { $clear += LogTime $line }
    elseif ($line -match 'peek fade started title=Covered to=1') { $fade += LogTime $line }
    elseif ($line -match 'covered fences raised at the limit') { $late++ }
  }
  $toClear = @(); $toFade = @()
  foreach ($c in $clear) {
    $o = @($open | Where-Object { $_ -le $c }) | Select-Object -Last 1
    $f = @($fade | Where-Object { $_ -ge $c }) | Select-Object -First 1
    if ($o) { $toClear += [int]($c - $o).TotalMilliseconds }
    if ($f) { $toFade += [int]($f - $c).TotalMilliseconds }
  }
  $d = "$sum; app log: Peeks opened $($open.Count); transparent $($toClear -join '/') ms after the Peek opened; raised and fading $($toFade -join '/') ms after that; raised by the limit instead of the frame clock: $late"
  Log "T $d"
  Check 'T: ten Peeks were timed and the covered fence was fully there in each' ($runs.Count -eq 10 -and (($runs | Measure-Object openFull -Minimum).Minimum -gt 0.9 * $Ref[0])) $d
  Check 'T: the covered fence never shows opaque before it fades in' (($runs | Measure-Object openFlash -Maximum).Maximum -lt 0.2 * $Ref[0]) $d
  PkAfter 'after ten timed Peeks'
}

if ($Part -match 'A') {
  Case 'MF-5 fade with the dimmer'
  if ($Record) {
    # The covered fence on the white window. The sampler rests meanwhile: two readers of the
    # screen halve each other's rate.
    [PkScan]::Stop()
    [G]::MoveTo($CoverX + 40, $CoverY + $CoverH - 40)
    RecStart 'MF-5' -X 150 -Y 110 -W 420 -H 310 -Fps 30
    Start-Sleep -Milliseconds 500
    RecMark '一个窗口盖住了栅栏'
    Start-Sleep -Milliseconds 700
    RecMark '按下速览快捷键'
    PeekKey; Start-Sleep -Milliseconds 800
    RecMark '再按一次，结束速览'
    PeekKey; Start-Sleep -Milliseconds 800
    RecStop
    [PkScan]::Start($PkPatches)
    Start-Sleep -Milliseconds 500
  }
  $runs = @(1..6 | ForEach-Object { PkCycle "A$_" })
  $sum = PkSummary $runs
  Check 'MF-5 the covered fence fades in with the dimmer (a part-way frame in at least 4 of 6 opens)' (@($runs | Where-Object { $_.openMid -ge 1 }).Count -ge 4) $sum
  Check 'MF-5 the covered fence fades out with the dimmer (a part-way frame in at least 4 of 6 closes)' (@($runs | Where-Object { $_.closeMid -ge 1 }).Count -ge 4) $sum
  Check 'MF-5 it never shows opaque before it fades in, nor again after it faded out' ((($runs | Measure-Object openFlash -Maximum).Maximum -lt 0.2 * $Ref[0]) -and (($runs | Measure-Object closeFlash -Maximum).Maximum -lt 0.2 * $Ref[0])) $sum
  Check 'MF-5 during the Peek it is fully opaque, afterwards hidden by the window again' ((($runs | Measure-Object openFull -Minimum).Minimum -gt 0.9 * $Ref[0]) -and (($runs | Measure-Object closeEnd -Maximum).Maximum -lt 0.15 * $Ref[0])) "full $((($runs | ForEach-Object { '{0:0}' -f $_.openFull }) -join '/')) end $((($runs | ForEach-Object { '{0:0}' -f $_.closeEnd }) -join '/')) of $('{0:0}' -f $Ref[0])"
  Check 'MF-5 the fence on visible desktop keeps its look through open and close' (($runs | Measure-Object visMin -Minimum).Minimum -gt 0.85 * $Ref[2]) $sum
  Check 'MF-5 the partly hidden fence does not blink' (($runs | Measure-Object partMin -Minimum).Minimum -gt 0.85 * $Ref[1]) $sum
  PkAfter 'after six shortcut cycles'

  Case 'MF-5 a window as large as the work area'
  # A maximized program: every fence is under it, also one snapped into the screen's bottom
  # corner, whose shadow reaches under the taskbar. All of them fade, none pops.
  $wa = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  Pf fence create --title Corner --rect "$($wa.Right - 308),$($wa.Bottom - 228),300,220" | Out-Null
  Start-Sleep -Milliseconds 800
  [PkCover]::Move($wa.Left, $wa.Top, $wa.Width, $wa.Height); Start-Sleep -Milliseconds 300
  [G]::Click($wa.Left + 60, $wa.Bottom - 60); Start-Sleep -Milliseconds 400
  $n0 = @(PfLogAll | Where-Object { $_ -match 'peek: fences hidden by a window fade in' }).Count
  $run = PkCycle 'Amax'
  $line = @(PfLogAll | Where-Object { $_ -match 'peek: fences hidden by a window fade in' }) | Select-Object -Last 1
  $all = @(@(PfJson fence list) | Where-Object { $_ -and $_.id }).Count
  Check 'MF-5 under a window as large as the work area every fence fades, the one in the bottom corner too' ("$line" -match "covered=$all held=$all " -and $run.openMid -ge 1) "fences=$all; log: $("$line" -replace '^.*peek: ', ''); part-way frames of the covered fence: open $($run.openMid) close $($run.closeMid)"
  [PkCover]::Move($CoverX, $CoverY, $CoverW, $CoverH); Start-Sleep -Milliseconds 300
  Pf fence delete Corner | Out-Null; Start-Sleep -Milliseconds 600
  PkCoverFront
  PkAfter 'after the large window'

  Case 'MF-5 dimming off'
  Pf settings set peek.dim false | Out-Null; Start-Sleep -Milliseconds 800
  PkCoverFront
  $runs = @(1..4 | ForEach-Object { PkCycle "Adim$_" })
  $sum = PkSummary $runs
  Check 'MF-5 dimming off: the covered fence still fades in and out (a part-way frame in at least 3 of 4)' (@($runs | Where-Object { $_.openMid -ge 1 }).Count -ge 3 -and @($runs | Where-Object { $_.closeMid -ge 1 }).Count -ge 3) $sum
  Check 'MF-5 dimming off: the visible and the partly hidden fence keep their look' ((($runs | Measure-Object visMin -Minimum).Minimum -gt 0.85 * $Ref[2]) -and (($runs | Measure-Object partMin -Minimum).Minimum -gt 0.85 * $Ref[1])) $sum
  PkAfter 'dimming off'
  Pf settings set peek.dim true | Out-Null; Start-Sleep -Milliseconds 800
  PkCoverFront

  Case 'MF-5 Liquid Glass'
  Pf settings set themeStyle liquidGlass | Out-Null; Start-Sleep -Milliseconds 1200
  PkCoverFront
  $runs = @(1..3 | ForEach-Object { PkCycle "Aglass$_" })
  $sum = PkSummary $runs
  Check 'MF-5 Liquid Glass: the covered fence fades in and out too (a part-way frame in at least 2 of 3)' (@($runs | Where-Object { $_.openMid -ge 1 }).Count -ge 2 -and @($runs | Where-Object { $_.closeMid -ge 1 }).Count -ge 2) $sum
  PkAfter 'Liquid Glass'
  Pf settings set themeStyle fluent | Out-Null; Start-Sleep -Milliseconds 1200
  PkCoverFront

  Case 'MF-5 animations off'
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  $runs = @(1..3 | ForEach-Object { PkCycle "Aoff$_" })
  $sum = PkSummary $runs
  # (At the close a fence can be under the dimmer for one frame, which reads as 70 %: that is
  # how it was before, so only the time until it is gone is checked.)
  Check 'MF-5 animations off: the fences show and go at once, as before' (@($runs | Where-Object { $_.openMid -ge 1 }).Count -eq 0 -and (($runs | Measure-Object openFull -Minimum).Minimum -gt 0.9 * $Ref[0]) -and (($runs | Measure-Object goneAt -Maximum).Maximum -lt 150)) "$sum; gone $((($runs | ForEach-Object { $_.goneAt }) -join '/')) ms after the key"
  PkAfter 'animations off'
  [Anim]::Set($true); Start-Sleep -Milliseconds 800
}

if ($Part -match 'B') {
  Case 'MF-5 every way a Peek ends'
  # 1 the shortcut again (also the quick double and triple presses)
  PeekKey; Check 'B the shortcut opens the Peek' (PkWaitPeek $true) ''; Start-Sleep -Milliseconds 500
  PeekKey; PkAfter 'ended by the shortcut'
  # 2 Esc
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  [G]::Press(0x1B); PkAfter 'ended by Esc'
  # 3 a click beside the fences
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  [G]::Click($CoverX + 40, $CoverY + $CoverH - 40); PkAfter 'ended by a click on the dimmer'
  # 4 opening a file of the covered fence
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  $pt = ItemPt 'Covered' 'pk 说明文件.txt'
  if (-not $pt) { $pt = ItemPt 'Covered' 'pk 说明文件' }
  DoubleClick $pt[0] $pt[1]; Start-Sleep -Milliseconds 1500
  Check 'B opening a file from the Peek opens it' ([bool](Get-Process notepad -ErrorAction SilentlyContinue)) (NotepadTitles)
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 500; PkCoverFront; PkAfter 'ended by opening a file'
  # 5 the search box, closed with Esc
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  [G]::Type('pk'); Start-Sleep -Milliseconds 900
  Check 'B typing opens the search box' ((WaitWin 'PecoFence.PeekSearch' 4) -ne [IntPtr]::Zero) "box='$([M]::BoxText())'"
  [G]::Press(0x1B); PkAfter 'ended by Esc in the search box'
  # 6 another program coming to the front
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  Start-Process notepad; Start-Sleep -Milliseconds 1800
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 500; PkCoverFront; PkAfter 'ended by another window in front'
  # 7 the command line (start and end)
  Pf peek start | Out-Null; Check 'B the command line opens the Peek' (PkWaitPeek $true) ''; Start-Sleep -Milliseconds 600
  $v = PkNow
  Check 'B opened from the command line: the covered fence shows fully' ($v[0] -gt 0.9 * $Ref[0]) "chroma=$('{0:0}' -f $v[0]) of $('{0:0}' -f $Ref[0])"
  Pf peek end | Out-Null; PkCoverFront; PkAfter 'ended by the command line'
  # 8 quick hide during the Peek, then the Peek ends, then show all
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  Pf fence hide-all | Out-Null; Start-Sleep -Milliseconds 700
  if (PkPeekOn) { PeekKey }
  Start-Sleep -Milliseconds 700
  Pf fence show-all | Out-Null; Start-Sleep -Milliseconds 300; PkCoverFront; PkAfter 'quick hide during the Peek, shown again'
  # 9 a settings change that ends the Peek at once
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  Pf settings set theme dark | Out-Null; Start-Sleep -Milliseconds 900
  if (PkPeekOn) { PeekKey; Start-Sleep -Milliseconds 400 }
  Pf settings set theme light | Out-Null; Start-Sleep -Milliseconds 900; PkCoverFront; PkAfter 'a settings change during the Peek'
  # 10 pressed twice quickly (opened and closed before the covered fence came up)
  foreach ($gap in 15, 45, 90) {
    PeekKey; Start-Sleep -Milliseconds $gap; PeekKey
    PkAfter "opened and closed $gap ms apart"
  }
  # 11 pressed three times quickly: it ends open, with the covered fence fully there. (A third
  # press that lands just after the second one's close has finished starts a new Peek, which
  # the window coming back to the front can end at once, in builds before this change too: then
  # the fences must simply be at rest, which PkAfter checks.)
  foreach ($gap in 30, 70) {
    PeekKey; Start-Sleep -Milliseconds $gap; PeekKey; Start-Sleep -Milliseconds $gap; PeekKey
    Start-Sleep -Milliseconds 900
    $v = PkNow; $on = PkPeekOn
    Check "B three presses $gap ms apart: while the Peek is on the covered fence shows fully" (-not $on -or $v[0] -gt 0.9 * $Ref[0]) "peek=$on chroma=$('{0:0}' -f $v[0]) of $('{0:0}' -f $Ref[0])"
    if (PkPeekOn) { PeekKey } else { PkCoverFront }
    PkAfter "three presses $gap ms apart, then ended"
  }
  # 12 a switch to another virtual desktop during the Peek
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  VdNew; Start-Sleep -Milliseconds 1500
  if (PkPeekOn) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
  $r = PkRest 'on the new desktop'
  $v = PkNow
  Check 'B a virtual desktop switch during the Peek: on the new desktop every fence shows, at rest' (-not (PkPeekOn) -and $r[1] -eq '' -and $v[0] -gt 0.9 * $Ref[0] -and $v[2] -gt 0.85 * $Ref[2]) "peek=$(PkPeekOn) $($r[1]) chroma covered=$('{0:0}' -f $v[0]) visible=$('{0:0}' -f $v[2])"
  VdClose; Start-Sleep -Milliseconds 1500; VdOnlyOne
  PkCoverFront; PkAfter 'back from the other virtual desktop'
  # 13 Alt+Tab to another program
  Start-Process notepad; Start-Sleep -Milliseconds 1800; PkCoverFront
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  [G]::KeyDown(0x12); Start-Sleep -Milliseconds 60; [G]::Press(0x09); Start-Sleep -Milliseconds 60; [G]::KeyUp(0x12); Start-Sleep -Milliseconds 1200
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 500; PkCoverFront; PkAfter 'ended by Alt+Tab'
  # 14 the Start menu
  PeekKey; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 500
  [G]::Press(0x5B); Start-Sleep -Milliseconds 1500; [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  if (PkPeekOn) { PeekKey; Start-Sleep -Milliseconds 400 }
  PkCoverFront; PkAfter 'ended by the Start menu'
  # 15 opened together with the search box: the box is over the fences, the fences over the dimmer
  Pf peek start --search pk | Out-Null; PkWaitPeek $true | Out-Null; Start-Sleep -Milliseconds 900
  $z = @(([L]::Tops('pecofence') -split "`n") | Where-Object { $_ } | ForEach-Object { ($_ -split ' \| ')[1] })
  $box = [array]::IndexOf($z, 'PecoFence.PeekSearch'); $dim = [array]::IndexOf($z, 'PecoFence.PeekDimmer')
  $fz = @(0..($z.Count - 1) | Where-Object { $z[$_] -eq 'PecoFence.Fence' })
  Check 'B opened with the search box: the box is over every fence and every fence over the dimmer' ($box -ge 0 -and $dim -ge 0 -and $fz.Count -ge 4 -and ($fz | Measure-Object -Minimum).Minimum -gt $box -and ($fz | Measure-Object -Maximum).Maximum -lt $dim) "top first: $($z -join ' > ')"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; PkCoverFront; PkAfter 'opened with the search box, ended by Esc'
  # Windows' own surfaces (the desktop switcher, the task switcher, Start) span the screen but
  # hide nothing: only a program's window (the white one, Notepad) ever counted as hiding a fence.
  $by = @(PfLogAll | Where-Object { $_ -match 'fence hidden by a window' } | ForEach-Object { if ($_ -match 'class=(\S+)') { $Matches[1] } } | Sort-Object -Unique)
  $wrong = @($by | Where-Object { $_ -notmatch '^(Notepad|WindowsForms10\.)' })
  Check 'B only a program window counts as hiding a fence (never the desktop switcher, the task switcher or Start)' ($wrong.Count -eq 0) "hidden by: $($by -join ', ')"
  Check 'B the app log has no error' (@(PfLogErrors).Count -eq 0) "$(@(PfLogErrors) | Select-Object -First 2)"
}

[PkScan]::Stop()
PkCover-Close

if ($Part -match 'C') {
  Case 'MF-12 the command line fit glides'
  ClearFences | Out-Null
  foreach ($i in 1..7) { Set-Content -LiteralPath (Join-Path $PfDesk "pk 适应文件 $i.txt") -Value "fit $i" -Encoding ASCII }
  Pf fence create --title Fit --rect '300,150,340,470' | Out-Null
  $files = @(1..7 | ForEach-Object { "$PfDesk\pk 适应文件 $_.txt" })
  # The files are new on the desktop: PecoFence may need a moment to know them.
  foreach ($try in 1..20) { $r = Pf item move @files --to Fit; if ($r -notmatch 'item_not_found') { break }; Start-Sleep -Milliseconds 500 }
  Start-Sleep -Seconds 1
  $h = [G]::FindWindowW('PecoFence.Fence', 'Fit')
  function PkHeights($ms) {
    $seen = New-Object System.Collections.Generic.List[int]
    $sw = [Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $ms) { $r = Rect $h; $v = $r.B - $r.T; if ($seen.Count -eq 0 -or $seen[$seen.Count - 1] -ne $v) { $seen.Add($v) }; Start-Sleep -Milliseconds 4 }
    return @($seen)
  }
  $from = (Rect $h).B - (Rect $h).T
  ParkMouse 1400 700
  if ($Record) { RecStart 'MF-12' -X 270 -Y 120 -W 400 -H 530 -Fps 30; Start-Sleep -Milliseconds 400; RecMark '栅栏比图标需要的高'; Start-Sleep -Milliseconds 700; RecMark '运行 pecofence-cli fence fit' }
  # Not through Pf: the heights are read while the command runs.
  $out = "$PfRun\pk-fit.json"
  $proc = Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'fence', 'fit', 'Fit' -NoNewWindow -PassThru -RedirectStandardOutput $out
  $seen = PkHeights 1200
  $proc.WaitForExit(5000) | Out-Null
  $reply = Get-Content -LiteralPath $out -Raw -ErrorAction SilentlyContinue
  if ($Record) { Start-Sleep -Milliseconds 300; RecStop }
  $to = (Rect $h).B - (Rect $h).T
  $fit = (FenceGet 'Fit')
  Log "fit: from $from to $to, heights seen: $($seen -join ' '); reply: $($reply -replace '\s+', ' ')"
  Check 'MF-12 fit makes the fence as tall as its two rows of icons need' ($to -lt $from -and [int]$fit.fit.rows -eq 2 -and $to -eq [int]$fit.fit.fittingHeight -and [int]$fit.rect.h -eq $to) "from $from to $to, rows=$($fit.fit.rows) fittingHeight=$($fit.fit.fittingHeight) saved h=$($fit.rect.h)"
  Check 'MF-12 the bottom edge glides there (at least 3 heights in between)' (@($seen | Where-Object { $_ -gt $to -and $_ -lt $from }).Count -ge 3) "heights seen: $($seen -join ' ')"
  Check 'MF-12 the reply says it changed' ($reply -match '"changed":\s*true') ($reply -replace '\s+', ' ')
  # A rolled fence: fit still sets the height it opens to, without opening it.
  Pf fence move Fit --rect '300,150,340,470' | Out-Null; Start-Sleep -Milliseconds 600
  Pf fence roll Fit | Out-Null; Start-Sleep -Milliseconds 800
  Pf fence fit Fit | Out-Null; Start-Sleep -Milliseconds 600
  $f = FenceGet 'Fit'
  Check 'MF-12 fit on a rolled fence keeps it rolled and sets the height it opens to' ($f.rolledUp -and [int]$f.rect.h -eq $to) "rolled=$($f.rolledUp) saved h=$($f.rect.h) window h=$($f.windowRect.h)"
  Pf fence unroll Fit | Out-Null; Start-Sleep -Milliseconds 900
  $r = Rect $h
  Check 'MF-12 it then opens to that height' (($r.B - $r.T) -eq $to) "window h=$($r.B - $r.T)"
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
PkClean
VdOnlyOne
