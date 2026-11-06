# scripts/wintest/pixels.ps1: pixel reads of screen regions, loaded by lib.ps1.

# Pixel reads of screen regions: crops, an ink profile (how far each column departs from its
# row's median brightness, colourless pixels only), the accent marker under the active tab, a
# frame burst around a click, and a filmstrip PNG of the burst.
Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Text; using System.Drawing; using System.Drawing.Imaging;
using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices;
public static class Px {
  [DllImport("user32.dll")] static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  public static Bitmap Grab(int x, int y, int w, int h) {
    var b = new Bitmap(w, h, PixelFormat.Format32bppArgb);
    using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, new Size(w, h));
    return b;
  }
  public static int[] Pixels(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var a = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  static int Sat(int c) { int r = (c >> 16) & 255, g = (c >> 8) & 255, b = c & 255; return Math.Max(r, Math.Max(g, b)) - Math.Min(r, Math.Min(g, b)); }
  public static int[] Ink(Bitmap b, int y0, int y1) { return Ink(Pixels(b), b.Width, b.Height, y0, y1); }
  public static int[] Ink(int[] px, int w, int h, int y0, int y1) {
    var prof = new int[w];
    for (int y = Math.Max(0, y0); y <= Math.Min(h - 1, y1); y++) {
      var row = new int[w]; for (int x = 0; x < w; x++) row[x] = Lum(px[y * w + x]);
      var s = (int[])row.Clone(); Array.Sort(s); int med = s[w / 2];
      for (int x = 0; x < w; x++) { if (Sat(px[y * w + x]) > 40) continue; int d = Math.Abs(row[x] - med); if (d > prof[x]) prof[x] = d; }
    }
    return prof;
  }
  // First column in [x0, x1] whose ink in rows y0..y1 reaches thr; -1 when none.
  public static int FirstInk(Bitmap b, int x0, int x1, int y0, int y1, int thr) {
    var p = Ink(b, y0, y1);
    for (int x = Math.Max(0, x0); x <= Math.Min(b.Width - 1, x1); x++) if (p[x] >= thr) return x;
    return -1;
  }
  public static int MaxInk(Bitmap b, int x0, int x1, int y0, int y1) {
    var p = Ink(b, y0, y1); int m = 0;
    for (int x = Math.Max(0, x0); x <= Math.Min(b.Width - 1, x1); x++) m = Math.Max(m, p[x]);
    return m;
  }
  // Centre x of the accent marker (a saturated run 12 to 30 px long) in rows y0..y1, the lowest
  // such row winning; -1 when none.
  public static double Marker(Bitmap b, int y0, int y1) { return Marker(Pixels(b), b.Width, b.Height, y0, y1); }
  public static double Marker(int[] px, int w, int h, int y0, int y1) {
    for (int y = Math.Min(h - 1, y1); y >= Math.Max(0, y0); y--) {
      int run = 0;
      for (int x = 0; x <= w; x++) {
        bool s = x < w && Sat(px[y * w + x]) >= 70;
        if (s) { run++; continue; }
        if (run >= 12 && run <= 30) return x - run / 2.0 - 0.5;
        run = 0;
      }
    }
    return -1;
  }
  public static string Runs(Bitmap b, int y0, int y1) {
    var px = Pixels(b); int w = b.Width; var sb = new StringBuilder();
    for (int y = Math.Max(0, y0); y <= Math.Min(b.Height - 1, y1); y++) {
      int run = 0;
      for (int x = 0; x <= w; x++) {
        bool s = x < w && Sat(px[y * w + x]) >= 70;
        if (s) { run++; continue; }
        if (run >= 3) sb.Append("y" + y + ":" + (x - run) + "-" + (x - 1) + " ");
        run = 0;
      }
    }
    return sb.ToString();
  }
  // Mean brightness difference of two same-size crops in a sub-rectangle.
  public static double Diff(Bitmap a, Bitmap b, int x0, int x1, int y0, int y1) {
    var pa = Pixels(a); var pb = Pixels(b); int w = a.Width; double sum = 0; int n = 0;
    for (int y = Math.Max(0, y0); y <= Math.Min(a.Height - 1, y1); y++)
      for (int x = Math.Max(0, x0); x <= Math.Min(w - 1, x1); x++) { sum += Math.Abs(Lum(pa[y * w + x]) - Lum(pb[y * w + x])); n++; }
    return n == 0 ? 0 : sum / n;
  }
  public static List<Bitmap> Frames = new List<Bitmap>();
  public static List<long> Times = new List<long>();
  // Frames of a region for `ms` ms; action 1 clicks where the pointer is after the first frame,
  // 2 presses the left button only.
  public static void Burst(int x, int y, int w, int h, int ms, int action) {
    foreach (var f in Frames) f.Dispose();
    Frames.Clear(); Times.Clear();
    var sw = Stopwatch.StartNew();
    Frames.Add(Grab(x, y, w, h)); Times.Add(0);
    if (action >= 1) mouse_event(2, 0, 0, 0, UIntPtr.Zero);
    if (action == 1) { System.Threading.Thread.Sleep(30); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
    while (sw.ElapsedMilliseconds < ms) { Frames.Add(Grab(x, y, w, h)); Times.Add(sw.ElapsedMilliseconds); }
  }
  static int BestShift(int[] a, int[] b, int max) {
    int w = a.Length, best = 0; double bc = double.MaxValue;
    for (int s = -max; s <= max; s++) {
      double c = 0; int n = 0;
      for (int x = 0; x < w; x++) { int xb = x + s; if (xb < 0 || xb >= w) continue; c += Math.Abs(a[x] - b[xb]); n++; }
      if (n < w / 2) continue; c /= n;
      if (c < bc - 0.01 || (Math.Abs(c - bc) <= 0.01 && Math.Abs(s) < Math.Abs(best))) { bc = c; best = s; }
    }
    return best;
  }
  // "ms:marker:shift" per frame: the accent marker's centre and how far the ink profile of rows
  // y0..y1 moved since the frame before (negative = to the left).
  public static string Track(int y0, int y1, int my0, int my1) {
    var sb = new StringBuilder(); int[] prev = null;
    for (int f = 0; f < Frames.Count; f++) {
      var b = Frames[f]; var px = Pixels(b);
      var m = Marker(px, b.Width, b.Height, my0, my1);
      var p = Ink(px, b.Width, b.Height, y0, y1);
      int s = prev == null ? 0 : BestShift(prev, p, 140);
      sb.Append(Times[f] + ":" + m.ToString("0.#") + ":" + s + " ");
      prev = p;
    }
    return sb.ToString().Trim();
  }
  public static void Strip(string path, int every) {
    if (Frames.Count == 0) return;
    int n = (Frames.Count + every - 1) / every, w = Frames[0].Width, h = Frames[0].Height;
    using (var o = new Bitmap(w, n * (h + 2))) using (var g = Graphics.FromImage(o)) {
      g.Clear(Color.Magenta);
      for (int i = 0, k = 0; i < Frames.Count; i += every, k++) g.DrawImage(Frames[i], 0, k * (h + 2));
      o.Save(path, ImageFormat.Png);
    }
  }
}
'@
function Crop($name, $x, $y, $w, $h) { $b = [Px]::Grab($x, $y, $w, $h); $b.Save("$PfShots\$name.png"); Log "crop $name"; return $b }

# A plain grey wallpaper, so pixel reads see only the fences.
function GreyWallpaper() {
  $b = New-Object Drawing.Bitmap 32, 32; $g = [Drawing.Graphics]::FromImage($b); $g.Clear([Drawing.Color]::FromArgb(128, 128, 128)); $g.Dispose()
  $b.Save("$PfRun\grey.bmp", [Drawing.Imaging.ImageFormat]::Bmp); $b.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, "$PfRun\grey.bmp", 3) | Out-Null
}

# Mean colours of the icons Explorer shows for some files in a Large icons view of their folder
# (one window; name -> @(r, g, b)).
function ExplorerIconRgb($folder, [string[]]$names) {
  Start-Process explorer.exe $folder
  $w = $null
  for ($k = 0; $k -lt 30 -and -not $w; $k++) { Start-Sleep -Milliseconds 300; $w = @((New-Object -ComObject Shell.Application).Windows()) | Where-Object { try { $_.Document.Folder.Self.Path -eq $folder } catch { $false } } | Select-Object -First 1 }
  $out = @{}
  if (-not $w) { return $out }
  [L]::MoveWindow([IntPtr]$w.HWND, 40, 100, 700, 500, $true) | Out-Null
  $w.Document.CurrentViewMode = 1; $w.Document.IconSize = 96
  Start-Sleep -Seconds 3
  $li = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
  $rows = @([Windows.Automation.AutomationElement]::FromHandle([IntPtr]$w.HWND).FindAll([Windows.Automation.TreeScope]::Descendants, $li))
  foreach ($name in $names) {
    $row = $rows | Where-Object { $_.Current.Name -like "$name*" } | Select-Object -First 1
    if (-not $row) { continue }
    $r = $row.Current.BoundingRectangle
    $out[$name] = MeanRgb (Crop "explorer-$name" ([int]($r.X + $r.Width / 2 - 20)) ([int]($r.Y + 30)) 40 40)
  }
  return $out
}
# Mean colour of a crop: "r g b".
function MeanRgb($b) {
  $px = [Px]::Pixels($b); $r = 0; $g = 0; $bl = 0
  foreach ($c in $px) { $r += ($c -shr 16) -band 255; $g += ($c -shr 8) -band 255; $bl += $c -band 255 }
  $n = [math]::Max(1, $px.Count)
  return @([int]($r / $n), [int]($g / $n), [int]($bl / $n))
}

# Text on screen by Windows' OCR (Windows.Media.Ocr), for what UI Automation cannot see: a toast
# sits in a window band a normal program cannot read, the settings page of a release build.
# The crop is enlarged twice first; '' when no OCR language is installed.
function OcrText($x, $y, $w, $h, $name = 'ocr') {
  Add-Type -AssemblyName System.Runtime.WindowsRuntime
  $null = [Windows.Media.Ocr.OcrEngine, Windows.Foundation, ContentType = WindowsRuntime]
  $null = [Windows.Graphics.Imaging.BitmapDecoder, Windows.Graphics, ContentType = WindowsRuntime]
  $null = [Windows.Storage.StorageFile, Windows.Storage, ContentType = WindowsRuntime]
  $asTask = @([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' })[0]
  $await = { param($op, $type) $t = $asTask.MakeGenericMethod($type).Invoke($null, @($op)); $t.Wait(-1) | Out-Null; $t.Result }
  $src = [Px]::Grab($x, $y, $w, $h)
  $big = New-Object Drawing.Bitmap ($w * 2), ($h * 2)
  $g = [Drawing.Graphics]::FromImage($big); $g.InterpolationMode = 'HighQualityBicubic'; $g.DrawImage($src, 0, 0, $w * 2, $h * 2); $g.Dispose()
  $path = "$PfShots\$name.png"; $big.Save($path); $big.Dispose(); $src.Dispose()
  $eng = [Windows.Media.Ocr.OcrEngine]::TryCreateFromUserProfileLanguages()
  if (-not $eng) { Log 'ocr: no OCR language'; return '' }
  $file = & $await ([Windows.Storage.StorageFile]::GetFileFromPathAsync($path)) ([Windows.Storage.StorageFile])
  $stream = & $await ($file.OpenAsync([Windows.Storage.FileAccessMode]::Read)) ([Windows.Storage.Streams.IRandomAccessStream])
  $dec = & $await ([Windows.Graphics.Imaging.BitmapDecoder]::CreateAsync($stream)) ([Windows.Graphics.Imaging.BitmapDecoder])
  $bmp = & $await ($dec.GetSoftwareBitmapAsync()) ([Windows.Graphics.Imaging.SoftwareBitmap])
  $res = & $await ($eng.RecognizeAsync($bmp)) ([Windows.Media.Ocr.OcrResult])
  $stream.Dispose()
  return "$($res.Text)"
}
# The toast at the bottom right, read by OCR until it says `$want` (or $sec run out).
function ToastOcr($want = 'PecoFence', $sec = 8) {
  $t = ''
  for ($i = 0; $i -lt $sec; $i++) {
    $t = OcrText 1200 640 400 210 "toast-$i"
    if ($t -match $want) { return $t }
    Start-Sleep -Milliseconds 700
  }
  return $t
}
