# Screen recorder for before/after animations (dot-source it in a steps file).
#   RecStart 'S1-before' [-X 0 -Y 0 -W 1600 -H 900] [-Fps 12]
#   RecMark 'Drag the fence title'        # one mark per user step, same steps in before and after
#   RecMetric 3.4                         # optional number to chart under the frames (e.g. MB, ms)
#   RecStop                               # writes frames.tsv, marks.tsv, metric.tsv, stats.txt
# Output: $PfShots\rec-<name>\NNNNN.jpg plus the tsv files; copy the folder back with the job.
# frames.tsv: index, ms since start, cursor x, cursor y (relative to the region), buttons (1 left, 2 right).
# stats.txt: frames per second reached and the time per grab (EC2 boxes: 960x540 about 25 ms, so up
# to about 30 frames per second; JPEG encoding about 2 ms).
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
public static class Rec {
  [StructLayout(LayoutKind.Sequential)] struct PT { public int X; public int Y; }
  [DllImport("user32.dll")] static extern bool GetCursorPos(out PT p);
  [DllImport("user32.dll")] static extern short GetAsyncKeyState(int vk);
  static Thread worker; static volatile bool running; static Stopwatch clock;
  static string dir; static int rx, ry, rw, rh, interval;
  static List<string> frames, marks, metric;
  static long grabMs, stopped;
  public static void Start(string folder, int x, int y, int w, int h, int fps) {
    if (running) Stop();
    dir = folder; Directory.CreateDirectory(dir);
    rx = x; ry = y; rw = w; rh = h; interval = Math.Max(15, 1000 / Math.Max(1, fps));
    frames = new List<string>(); marks = new List<string>(); metric = new List<string>();
    grabMs = 0; clock = Stopwatch.StartNew(); running = true;
    worker = new Thread(Loop); worker.IsBackground = true; worker.Start();
  }
  static void Loop() {
    var codec = Array.Find(ImageCodecInfo.GetImageEncoders(), c => c.MimeType == "image/jpeg");
    var ep = new EncoderParameters(1); ep.Param[0] = new EncoderParameter(System.Drawing.Imaging.Encoder.Quality, 82L);
    int i = 0; long next = 0;
    while (running) {
      long now = clock.ElapsedMilliseconds;
      if (now < next) { Thread.Sleep((int)Math.Min(5, next - now)); continue; }
      // A fixed cadence: the sleep's 15 ms granularity no longer adds up (12 asked gave 11).
      next += interval;
      if (next <= now) next = now + interval;
      try {
        using (var bmp = new Bitmap(rw, rh, PixelFormat.Format24bppRgb)) {
          using (var g = Graphics.FromImage(bmp)) g.CopyFromScreen(rx, ry, 0, 0, new Size(rw, rh));
          PT p; GetCursorPos(out p);
          int b = ((GetAsyncKeyState(0x01) & 0x8000) != 0 ? 1 : 0) | ((GetAsyncKeyState(0x02) & 0x8000) != 0 ? 2 : 0);
          bmp.Save(Path.Combine(dir, i.ToString("D5") + ".jpg"), codec, ep);
          lock (frames) frames.Add(i + "\t" + now + "\t" + (p.X - rx) + "\t" + (p.Y - ry) + "\t" + b);
          grabMs += clock.ElapsedMilliseconds - now;
          i++;
        }
      } catch { }
    }
  }
  public static void Mark(string text) { lock (marks) marks.Add(clock.ElapsedMilliseconds + "\t" + text.Replace("\t", " ")); }
  public static void Metric(double v) { lock (metric) metric.Add(clock.ElapsedMilliseconds + "\t" + v.ToString(System.Globalization.CultureInfo.InvariantCulture)); }
  public static void Stop() {
    if (!running) return;
    stopped = clock.ElapsedMilliseconds;
    running = false; worker.Join(3000);
    var utf8 = new UTF8Encoding(false);
    File.WriteAllLines(Path.Combine(dir, "frames.tsv"), frames, utf8);
    File.WriteAllLines(Path.Combine(dir, "marks.tsv"), marks, utf8);
    File.WriteAllLines(Path.Combine(dir, "metric.tsv"), metric, utf8);
    File.WriteAllText(Path.Combine(dir, "stats.txt"), string.Format(System.Globalization.CultureInfo.InvariantCulture,
      "region {0}x{1}, asked {2} ms per frame: {3} frames in {4} ms ({5:F1} per second), {6:F1} ms per grab and save\n",
      rw, rh, interval, frames.Count, stopped, frames.Count * 1000.0 / Math.Max(1, stopped), grabMs / (double)Math.Max(1, frames.Count)), utf8);
  }
}
'@
function RecStart([string]$Name, [int]$X = 0, [int]$Y = 0, [int]$W = 0, [int]$H = 0, [int]$Fps = 12) {
  Add-Type -AssemblyName System.Windows.Forms
  if ($W -le 0 -or $H -le 0) {
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds; $X = $b.X; $Y = $b.Y; $W = $b.Width; $H = $b.Height
  }
  $root = if ($PfShots) { $PfShots } else { $env:TEMP }
  [Rec]::Start((Join-Path $root "rec-$Name"), $X, $Y, $W, $H, $Fps)
}
function RecMark([string]$Text) { [Rec]::Mark($Text) }
function RecMetric([double]$Value) { [Rec]::Metric($Value) }
function RecStop { [Rec]::Stop() }
