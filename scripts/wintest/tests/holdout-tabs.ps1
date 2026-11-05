# Hold-out test of the kit: the tab strip of merged fences (scrolling strip, wheel, Ctrl+Tab,
# Peek, tab drags to the ends, title-drag merges in Fluent and Liquid Glass, rolled count, folder
# tabs, hidden count and chevron, the left tab column, three suspected bugs).
# State comes from `dump` lines of the test script (tab_rects, active, rolled) and the CLI; the
# visual parts (accent marker under the active tab, fades, chevron) from pixel reads of crops.

# ---- lib candidates

# DumpScript: test-script lines that log a `dump dN` every $ms ms for $minutes, so a step can
# read the fence windows' state at any moment with PfDump.
function DumpScript($ms = 250, $minutes = 25, $first = 4000) {
  $n = [int]($minutes * 60000 / $ms)
  return @("sleep $first") + (1..$n | ForEach-Object { "dump d$_"; "sleep $ms" }) + @('sleep 3600000')
}

# ParseDumpLine: one `[dN] live ...` dump line as an object: hwnd, l t r b (window rect, px),
# ox oy (client offset), cw ch (chrome size px), title, ntabs, tabs (strip order of tab
# titles), rects ((x, w) DIPs per tab, window x; (y, h) in a left column), active (fence id),
# rolled, tabDrag, winDrag, contentX.
function ParseDumpLine($s) {
  $o = [ordered]@{ raw = $s }
  if ($s -match 'hwnd=0x([0-9a-f]+)') { $o.hwnd = [IntPtr][Convert]::ToInt64($Matches[1], 16) }
  if ($s -match ' rect=\((-?\d+),(-?\d+),(-?\d+),(-?\d+)\)') { $o.l = [int]$Matches[1]; $o.t = [int]$Matches[2]; $o.r = [int]$Matches[3]; $o.b = [int]$Matches[4] }
  if ($s -match ' title="((?:[^"\\]|\\.)*)"') { $o.title = $Matches[1] }
  if ($s -match ' rolled=(\w+)') { $o.rolled = $Matches[1] -eq 'true' }
  if ($s -match ' tabs=(\d+)') { $o.ntabs = [int]$Matches[1] }
  if ($s -match ' active=([0-9a-f-]+)') { $o.active = $Matches[1] }
  if ($s -match ' chrome=\((\d+), (\d+)\)') { $o.cw = [int]$Matches[1]; $o.ch = [int]$Matches[2] }
  if ($s -match ' client_offset=\((-?\d+),(-?\d+)\)') { $o.ox = [int]$Matches[1]; $o.oy = [int]$Matches[2] }
  if ($s -match ' content_x=(-?\d+)') { $o.contentX = [int]$Matches[1] }
  if ($s -match ' tab_fonts=\[(.*?)\] tab_rects') { $o.tabs = @([regex]::Matches($Matches[1], '\("((?:[^"\\]|\\.)*)", \d+\)') | ForEach-Object { $_.Groups[1].Value }) }
  if ($s -match ' tab_rects=\[(.*?)\]') { $o.rects = @([regex]::Matches($Matches[1], '\((-?[\d.]+), (-?[\d.]+)\)') | ForEach-Object { , @([double]$_.Groups[1].Value, [double]$_.Groups[2].Value) }) }
  if ($s -match ' tab_drag=(\w+)') { $o.tabDrag = $Matches[1] -eq 'true' }
  if ($s -match ' window_drag=(\w+)') { $o.winDrag = $Matches[1] -eq 'true' }
  return [pscustomobject]$o
}

# PfDump: the first complete dump logged after this call (one object per live fence window),
# read from the end of the log, so it stays fast however big the log grows.
function PfDump($sec = 5) {
  $p = PfLogPath
  if (-not (Test-Path $p)) { return @() }
  $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $start = $fs.Length; $fs.Close()
  $deadline = (Get-Date).AddSeconds($sec)
  while ((Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 100
    $fs = [IO.File]::Open($p, 'Open', 'Read', 'ReadWrite'); $fs.Position = $start
    $sr = New-Object IO.StreamReader($fs, [Text.Encoding]::UTF8); $buf = $sr.ReadToEnd(); $sr.Close()
    foreach ($m in [regex]::Matches($buf, '\[(d\d+)\] fences=')) {
      $tag = $m.Groups[1].Value
      if ($buf -notmatch ">> dump $tag\b") { continue }
      return @([regex]::Matches($buf, "\[$tag\] live ([^\r\n]*)") | ForEach-Object { ParseDumpLine $_.Groups[1].Value })
    }
  }
  return @()
}
# DumpOf: the window of that fence (its title or one of its tabs) in the next dump.
function DumpOf($name, $sec = 5) {
  foreach ($w in @(PfDump $sec)) { if ($w.title -eq $name -or (@($w.tabs) -contains $name)) { return $w } }
  return $null
}

# Real mouse: button down / up where the pointer is, a glide in steps (raw input, so drags and
# the Windows move loop see it), the wheel (notches < 0 = towards the user).
function MDown() { [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero) }
function MUp() { [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero) }
function Glide($x0, $y0, $x1, $y1, $steps = 12, $ms = 25) {
  for ($i = 1; $i -le $steps; $i++) { [G]::MoveTo([int]($x0 + ($x1 - $x0) * $i / $steps), [int]($y0 + ($y1 - $y0) * $i / $steps)); Start-Sleep -Milliseconds $ms }
}
function Wheel($x, $y, $notches) {
  [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 80
  $d = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int](120 * $notches)), 0)
  [G]::mouse_event(0x800, 0, 0, $d, [UIntPtr]::Zero)
}
function DoubleClick($x, $y) { [G]::Click($x, $y); Start-Sleep -Milliseconds 90; MDown; Start-Sleep -Milliseconds 30; MUp }

# NcHit: WM_NCHITTEST of a window at a screen point (2 caption, 1 client, 11 right edge ...).
function NcHit($h, $x, $y) {
  $res = [IntPtr]::Zero
  [G]::SendMessageTimeoutW($h, 0x84, [IntPtr]::Zero, [IntPtr](($y -shl 16) -bor ($x -band 0xFFFF)), 2, 1000, [ref]$res) | Out-Null
  return [int64]$res
}

# The open popup menu (#32768): its item texts, and a click on the item with that text.
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class Menu {
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr FindWindowW(string c, string t);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern int GetMenuItemCount(IntPtr m);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetMenuStringW(IntPtr m, uint id, StringBuilder s, int n, uint f);
  [DllImport("user32.dll")] static extern bool GetMenuItemRect(IntPtr w, IntPtr m, uint i, out RC r);
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  public static IntPtr Handle() { var w = FindWindowW("#32768", null); return w == IntPtr.Zero ? IntPtr.Zero : SendMessageW(w, 0x01E1, IntPtr.Zero, IntPtr.Zero); }
  public static string Items() {
    var m = Handle(); if (m == IntPtr.Zero) return null;
    var sb = new StringBuilder();
    for (int i = 0; i < GetMenuItemCount(m); i++) { var s = new StringBuilder(256); GetMenuStringW(m, (uint)i, s, 256, 0x400); sb.Append(s.ToString().Replace("&", "") + "|"); }
    return sb.ToString();
  }
  // Centre (x, y) of the first item whose text starts with `text` (screen px), or null.
  public static int[] ItemPoint(string text) {
    var m = Handle(); if (m == IntPtr.Zero) return null;
    for (int i = 0; i < GetMenuItemCount(m); i++) {
      var s = new StringBuilder(256); GetMenuStringW(m, (uint)i, s, 256, 0x400);
      if (s.ToString().Replace("&", "").StartsWith(text)) { RC r; GetMenuItemRect(IntPtr.Zero, m, (uint)i, out r); return new int[] { (r.L + r.R) / 2, (r.T + r.B) / 2 }; }
    }
    return null;
  }
}
'@
function MenuClick($text) {
  $p = [Menu]::ItemPoint($text)
  if ($null -eq $p) { return $false }
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 600; return $true
}
# Esc until no popup menu is open (an open menu blocks the CLI and the test script).
function CloseMenus() { for ($i = 0; $i -lt 4 -and [Menu]::Handle() -ne [IntPtr]::Zero; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 300 } }

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

# Windows' client-area animations (SPI_GETCLIENTAREAANIMATION): an RDP session starts with them
# off, which makes PecoFence snap where a normal desktop animates (tab slides, fades).
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class Anim {
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, ref int v, uint f);
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, IntPtr v, uint f);
  public static bool Get() { int v = 0; SystemParametersInfoW(0x1042, 0, ref v, 0); return v != 0; }
  public static void Set(bool on) { SystemParametersInfoW(0x1043, 0, (IntPtr)(on ? 1 : 0), 3); }
}
'@

# A plain grey wallpaper, so pixel reads see only the fences.
function GreyWallpaper() {
  $b = New-Object Drawing.Bitmap 32, 32; $g = [Drawing.Graphics]::FromImage($b); $g.Clear([Drawing.Color]::FromArgb(128, 128, 128)); $g.Dispose()
  $b.Save("$PfRun\grey.bmp", [Drawing.Imaging.ImageFormat]::Bmp); $b.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, "$PfRun\grey.bmp", 3) | Out-Null
}

# fence list as title -> id and id -> title.
function FenceIds() {
  $j = PfJson fence list
  $script:Id = @{}; $script:IdTitle = @{}
  foreach ($f in @($j.fences) + @($j)) { if ($f -and $f.title) { $script:Id[$f.title] = "$($f.id)"; $script:IdTitle["$($f.id)"] = $f.title } }
}
function FenceGet($t) { $j = PfJson fence get $t; if ($j.fence) { return $j.fence }; return $j }

# ---- tab strip helpers of this test

$Park = @(1150, 720)
function ParkPointer() { [G]::MoveTo($Park[0], $Park[1]); Start-Sleep -Milliseconds 300 }
function ActiveIdx($d) { return [array]::IndexOf(@($d.tabs), $IdTitle[$d.active]) }
function ActiveName($d) { return $IdTitle[$d.active] }
function Sx($d, $x) { return [int]($d.l + $d.ox + $x) }
function TitleY($d) { return [int]($d.t + $d.oy + 18) }
function TabX($d, $i) { return (Sx $d ($d.rects[$i][0] + $d.rects[$i][1] / 2)) }
function StripEnd($d) { if ($HideChevron) { return $d.cw - 8 } elseif ($d.rolled) { return $d.cw - 44 - $CountW } else { return $d.cw - 44 } }
$HideChevron = $false; $CountW = 64
# The active tab shows whole with 28 DIP (16 of the next tab + the 12 DIP fade) beside it where
# more tabs lie behind an end; at the strip's own ends it sits on the end.
function InView($d) {
  $i = ActiveIdx $d; $n = @($d.rects).Count; $end = StripEnd $d
  if ($i -lt 0) { return @($false, "active $($d.active) not in $($d.tabs -join ',')") }
  $x = $d.rects[$i][0]; $w = $d.rects[$i][1]
  $okR = if ($i -eq $n - 1) { [math]::Abs($x + $w - $end) -lt 1.5 } else { ($x + $w -le $end - 28 + 0.6) -and ($d.rects[$i + 1][0] -lt $end - 12) }
  $okL = if ($i -eq 0) { [math]::Abs($x - 8) -lt 1.5 } else { $x -ge 8 + 28 - 0.6 }
  $next = if ($i -lt $n - 1) { [math]::Round($d.rects[$i + 1][0], 1) } else { '-' }
  return @(($okR -and $okL), "active=$i $($d.tabs[$i]) x=$([math]::Round($x, 1)) w=$([math]::Round($w, 1)) end=$end next.x=$next")
}
function RectsText($d) { return (@($d.rects) | ForEach-Object { "$([math]::Round($_[0]))+$([math]::Round($_[1]))" }) -join ' ' }
# Accent marker centre in the fence's title row (screen x) vs the active tab's centre.
function MarkerOn($d, $name) {
  $b = Crop $name ($d.l + $d.ox) ($d.t + $d.oy) $d.cw 36
  $m = [Px]::Marker($b, 20, 33)
  $i = ActiveIdx $d
  $x = $d.rects[$i][0]; $w = $d.rects[$i][1]
  return @(($m -ge $x + 8 -and $m -le $x + $w - 8), "marker x=$m active tab $([math]::Round($x, 1))..$([math]::Round($x + $w, 1)) runs=$([Px]::Runs($b, 20, 33))")
}
# Wheel the strip of fence $name until its first tab is active (hover switching off meanwhile).
function ToFirstTab($name) {
  $d = DumpOf $name
  Wheel (Sx $d 20) (TitleY $d) 15; Start-Sleep -Milliseconds 700; ParkPointer
}
function ToLastTab($name) {
  $d = DumpOf $name
  Wheel (Sx $d 20) (TitleY $d) -15; Start-Sleep -Milliseconds 700; ParkPointer
}

# ---- steps

Case 'setup'
CloseWindows
Set-Culture en-US
GreyWallpaper
$made = @("$PfDesk\zulu-report.txt", "$PfDesk\yak-list.txt")
foreach ($f in $made) { Set-Content $f 'sample' }
New-Item -ItemType Directory -Force "$PfRun\Project files" | Out-Null
Set-Content "$PfRun\Project files\plan.txt" 'plan'
$AnimWas = [Anim]::Get()
[Anim]::Set($true)
Log "client-area animations were $AnimWas, now $([Anim]::Get())"
PfStart -Script (DumpScript)
Pf settings set theme light | Out-Null
Pf settings set rollUp.switchTabOnHover false | Out-Null
Log ("settings before: " + (Pf settings get rollUp) + (Pf settings get themeStyle) + (Pf settings get tabBar))
$names = 'Bravo', 'Charlie', 'Delta', 'Echo', 'Foxtrot', 'Golf', 'Hotel', 'India'
Pf fence create --title Alpha --rect '60,60,900,240' | Out-Null
foreach ($t in $names) {
  Pf fence create --title $t --rect '60,420,300,200' | Out-Null
  if ($t -eq 'Bravo') { Pf item move "$PfDesk\zulu-report.txt" --to Bravo | Out-Null }
  Pf fence merge $t --into Alpha | Out-Null
}
Pf fence create --title Kilo --rect '560,640,260,150' | Out-Null
ParkPointer
Start-Sleep -Seconds 2
FenceIds
$wide = DumpOf 'Alpha'
Log "wide Alpha: $($wide.raw)"
Check 'nine fences merged into one strip' ($wide.ntabs -eq 9 -and @($wide.tabs).Count -eq 9) "tabs=$($wide.tabs -join ',')"
Check 'Bravo holds zulu-report.txt' ((FenceGet Bravo).itemCount -ge 1) "itemCount=$((FenceGet Bravo).itemCount)"
Shot 'a00-setup'

Case '1 narrow strip keeps whole names and scrolls'
$lastW = $wide.rects[8][0] + $wide.rects[8][1]
Check 'at 900 px the strip fits' ($lastW -le $wide.cw - 44) "last ends at $lastW, cw=$($wide.cw)"
Pf fence resize Alpha --w 300 | Out-Null; Start-Sleep -Seconds 1
# The last merged tab is the active one; the left end fades then.
$n = DumpOf 'Alpha'
Check 'the last merged tab is active' ((ActiveIdx $n) -eq 8) "active=$(ActiveName $n)"
$b = Crop 'c01-rest-last' ($n.l + $n.ox) ($n.t + $n.oy) $n.cw 36
$ink = [Px]::Ink($b, 8, 28)
$fadeL = ($ink[8..11] | Measure-Object -Maximum).Maximum; $bodyL = ($ink[40..250] | Measure-Object -Maximum).Maximum
Check 'the left end (more tabs before it) fades' ($fadeL -lt $bodyL * 0.5) "ink first 4 px=$fadeL, strip body max=$bodyL"
ToFirstTab 'Alpha'
$n = DumpOf 'Alpha'
Log "narrow Alpha: $($n.raw)"
$same = $true; for ($i = 0; $i -lt 9; $i++) { if ([math]::Abs($n.rects[$i][1] - $wide.rects[$i][1]) -gt 0.6) { $same = $false } }
Check 'at 300 px every tab keeps its width (whole name)' ($same -and $n.cw -eq 300) "wide: $(RectsText $wide) | narrow: $(RectsText $n)"
Check 'the strip is longer than its room (it scrolls)' (($n.rects[8][0] + $n.rects[8][1]) -gt (StripEnd $n)) "last ends at $($n.rects[8][0] + $n.rects[8][1]), end=$(StripEnd $n)"
$r = InView $n; Check 'wheeled to the first tab, the strip starts at it' $r[0] $r[1]
$b = Crop 'c01-rest' ($n.l + $n.ox) ($n.t + $n.oy) $n.cw 36
$ink = [Px]::Ink($b, 8, 28)
$endX = $n.cw - 8
$fadeInk = ($ink[($endX - 4)..($endX - 1)] | Measure-Object -Maximum).Maximum
$bodyInk = ($ink[40..($endX - 30)] | Measure-Object -Maximum).Maximum
Log "rest ink profile: $($ink -join ',')"
Check 'the right end (more tabs behind it) fades' ($fadeInk -lt $bodyInk * 0.5) "ink last 4 px=$fadeInk, strip body max=$bodyInk"

Case '1 click on a cut-off tab'
$d = DumpOf 'Alpha'; $end = StripEnd $d
$cut = -1; for ($i = 0; $i -lt 9; $i++) { if ($d.rects[$i][0] -lt $end - 16 -and $d.rects[$i][0] + $d.rects[$i][1] -gt $end) { $cut = $i; break } }
Check 'a tab is cut off at the right end' ($cut -ge 0) (RectsText $d)
$cx = Sx $d (($d.rects[$cut][0] + $end - 14) / 2)
[G]::Click($cx, (TitleY $d)); Start-Sleep -Milliseconds 800
$d2 = DumpOf 'Alpha'
Check 'the click makes it active' ((ActiveIdx $d2) -eq $cut) "active=$(ActiveName $d2) want=$($d.tabs[$cut])"
$r = InView $d2; Check 'it slides into view with a bit of the next tab' $r[0] $r[1]
$r = MarkerOn $d2 'c02-click'; Check 'the accent marker sits under it' $r[0] $r[1]
ParkPointer

Case '1 wheel on the title row'
for ($k = 1; $k -le 3; $k++) {
  $d = DumpOf 'Alpha'; $i0 = ActiveIdx $d
  Wheel (TabX $d $i0) (TitleY $d) -1; Start-Sleep -Milliseconds 700
  $d2 = DumpOf 'Alpha'
  Check "wheel notch $k moves to the next tab" ((ActiveIdx $d2) -eq $i0 + 1) "from $i0 to $(ActiveIdx $d2)"
  $r = InView $d2; Check "notch ${k}: the tab shows with a bit of the next" $r[0] $r[1]
}
Wheel (TabX $d2 (ActiveIdx $d2)) (TitleY $d2) 1; Start-Sleep -Milliseconds 700
$d3 = DumpOf 'Alpha'
Check 'a notch away goes back one tab' ((ActiveIdx $d3) -eq (ActiveIdx $d2) - 1) "from $(ActiveIdx $d2) to $(ActiveIdx $d3)"
$r = InView $d3; Check 'going back, it shows with a bit of the previous' $r[0] $r[1]
ParkPointer

Case '1 Ctrl+Tab'
ToFirstTab 'Alpha'
$d = DumpOf 'Alpha'
[G]::Click((Sx $d 250), ($d.t + $d.oy + 200)); Start-Sleep -Milliseconds 500
Log ("fg after content click: " + (Fg))
for ($k = 1; $k -le 3; $k++) {
  $d = DumpOf 'Alpha'; $i0 = ActiveIdx $d
  Chord 0x11 0x09; Start-Sleep -Milliseconds 700
  $d2 = DumpOf 'Alpha'
  Check "Ctrl+Tab $k goes to the next tab" ((ActiveIdx $d2) -eq $i0 + 1) "from $i0 to $(ActiveIdx $d2), fg=$(Fg)"
  $r = InView $d2; Check "Ctrl+Tab ${k}: it shows with a bit of the next" $r[0] $r[1]
}
ParkPointer

Case '1+4 Peek search switches the tab'
# The last tab first, so Bravo (the second) lies off the strip's left end.
ToLastTab 'Alpha'
$d = DumpOf 'Alpha'; $before = ActiveName $d
Log "before peek: active=$before $(RectsText $d)"
[G]::Click($Park[0], $Park[1]); Start-Sleep -Milliseconds 400
PeekKey; Start-Sleep -Milliseconds 900
[G]::Type('zulu'); Start-Sleep -Milliseconds 1500
$d2 = DumpOf 'Alpha'
Check 'the search box has the text' ([M]::BoxText() -eq 'zulu') "box='$([M]::BoxText())'"
Check 'Peek switches Alpha to Bravo (the tab with the match)' ((ActiveName $d2) -eq 'Bravo') "active=$(ActiveName $d2) $(RectsText $d2)"
$r = InView $d2; Check 'Bravo slides into view with a bit of its neighbours' $r[0] $r[1]
$r = MarkerOn $d2 'c05-peek-fluent'; Check 'the selected background (accent marker) is on Bravo' $r[0] $r[1]
for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
Start-Sleep -Milliseconds 600
$d3 = DumpOf 'Alpha'
Check 'ending the search puts the tab back' ((ActiveName $d3) -eq $before) "active=$(ActiveName $d3) want=$before"
ParkPointer

Case '1 tab dragged to the strip ends scrolls it'
# The third tab at an unscrolled strip, dragged to the right end, then to the left end.
ToFirstTab 'Alpha'
$d = DumpOf 'Alpha'; [G]::Click((TabX $d 2), (TitleY $d)); Start-Sleep -Milliseconds 700
$d = DumpOf 'Alpha'; $i0 = ActiveIdx $d; $name0 = $d.tabs[$i0]; $end = StripEnd $d; $y = TitleY $d
$x0 = TabX $d $i0
[G]::MoveTo($x0, $y); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 100
Glide $x0 $y (Sx $d ($end - 10)) $y 10 30
Start-Sleep -Milliseconds 700
$mid = DumpOf 'Alpha'
Check 'held at the right end, the strip scrolls on' ($mid.tabDrag -and $mid.rects[0][0] -lt $d.rects[0][0] - 20) "first tab x before=$($d.rects[0][0]) held=$($mid.rects[0][0]) tab_drag=$($mid.tabDrag)"
Glide (Sx $d ($end - 10)) $y (Sx $d 14) $y 14 30
Start-Sleep -Milliseconds 1200
$mid2 = DumpOf 'Alpha'
Check 'held at the left end, it scrolls back' ($mid2.rects[0][0] -gt $mid.rects[0][0] + 20) "first tab x at the right end=$($mid.rects[0][0]) at the left end=$($mid2.rects[0][0])"
Shot 'c06-tab-drag-left-end'
MUp; Start-Sleep -Milliseconds 800
$d2 = DumpOf 'Alpha'
$i2 = [array]::IndexOf(@($d2.tabs), $name0)
Check 'the dropped tab moved towards the start' ($i2 -ge 0 -and $i2 -lt $i0) "$name0 from $i0 to $i2; order $($d2.tabs -join ',')"
$order = (FenceGet Alpha).tabs | ForEach-Object { $IdTitle["$_"] }
Log "fence get Alpha tabs: $($order -join ',')"
ParkPointer

# Title drag of Kilo onto Alpha's scrolled strip; the gap read mid-drag, the slot after the drop.
function MergeDrag($style) {
  $d = DumpOf 'Alpha'
  $k = DumpOf 'Kilo'
  $y = TitleY $d; $tx = Sx $d 130
  $kx = Sx $k 70; $ky = TitleY $k
  Log "merge ${style}: Alpha $(RectsText $d) active=$(ActiveName $d); Kilo at $($k.l),$($k.t)"
  [G]::MoveTo($kx, $ky); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 120
  Glide $kx $ky $tx $y 24 30
  Start-Sleep -Milliseconds 900
  $mid = DumpOf 'Alpha'
  Shot "c07-merge-$style-mid"
  $g = -1; $gx = $null
  $rs = @($mid.rects)
  if ($rs.Count -gt 0 -and $rs[0][0] -gt 8 + 60) { $g = 0; $gx = $rs[0][0] - 100 }
  for ($i = 1; $i -lt $rs.Count -and $g -lt 0; $i++) { if ($rs[$i][0] - ($rs[$i - 1][0] + $rs[$i - 1][1]) -gt 60) { $g = $i; $gx = $rs[$i - 1][0] + $rs[$i - 1][1] + 4 } }
  $px = $tx - $mid.l - $mid.ox
  Check "$style mid-drag: a gap opens in Alpha's strip" ($g -ge 0) "rects $(RectsText $mid)"
  # TabView's rule: the gap sits where the pointer is, between the centres of the pills around it.
  $lo = if ($g -gt 0) { $rs[$g - 1][0] + $rs[$g - 1][1] / 2 } else { -9999 }
  $hi = if ($g -ge 0 -and $g -lt $rs.Count) { $rs[$g][0] + $rs[$g][1] / 2 } else { 9999 }
  Check "${style}: the gap is at the pointer" ($g -ge 0 -and $px -ge $lo -and $px -le $hi) "pointer x=$px gap x=$gx slot=$g, neighbour centres $lo..$hi"
  MUp; Start-Sleep -Milliseconds 1500
  $after = DumpOf 'Alpha'
  $ki = [array]::IndexOf(@($after.tabs), 'Kilo')
  Check "${style}: the drop merges Kilo into that slot" ($ki -ge 0 -and $ki -eq $g) "Kilo at $ki, gap slot $g; order $($after.tabs -join ',')"
  Check "${style}: Kilo is hosted by Alpha" ("$((FenceGet Kilo).tabHost)" -eq $Id['Alpha']) "tabHost=$((FenceGet Kilo).tabHost)"
  ParkPointer
}

Case '1 title drag merges into the gap, Fluent'
# A tab in the middle first, so the strip is scrolled.
$d = DumpOf 'Alpha'; [G]::Click((TabX $d 1), (TitleY $d)); Start-Sleep -Milliseconds 400
Wheel (TabX $d 1) (TitleY $d) -3; Start-Sleep -Milliseconds 800; ParkPointer
MergeDrag 'Fluent'

Case '1 title drag merges into the gap, Liquid Glass'
Pf fence detach Kilo | Out-Null; Start-Sleep -Milliseconds 800
Pf fence move Kilo --rect '560,640,260,150' | Out-Null
Pf settings set themeStyle liquidGlass | Out-Null; Start-Sleep -Seconds 2
Check 'Liquid Glass is on' ((Pf settings get themeStyle) -match 'liquidGlass') (Pf settings get themeStyle)
MergeDrag 'Glass'
$d = DumpOf 'Alpha'
[G]::Click($Park[0], $Park[1]); Start-Sleep -Milliseconds 300
PeekKey; Start-Sleep -Milliseconds 900; [G]::Type('zulu'); Start-Sleep -Milliseconds 1500
$d2 = DumpOf 'Alpha'
Check 'Glass: Peek switches to Bravo' ((ActiveName $d2) -eq 'Bravo') "active=$(ActiveName $d2)"
$r = MarkerOn $d2 'c08-peek-glass'; Check 'Glass: the selected background is on Bravo' $r[0] $r[1]
for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }
Pf fence detach Kilo | Out-Null; Start-Sleep -Milliseconds 800
Pf fence move Kilo --rect '560,640,260,150' | Out-Null
Pf settings set themeStyle fluent | Out-Null; Start-Sleep -Seconds 2
ParkPointer

Case '1 rolled fence with many tabs shows its count'
ToLastTab 'Alpha'
Pf fence roll Alpha | Out-Null; Start-Sleep -Seconds 1; ParkPointer
foreach ($wpx in 300, 216, 215) {
  Pf fence resize Alpha --w $wpx | Out-Null; Start-Sleep -Milliseconds 900
  $d = DumpOf 'Alpha'
  $last = $d.rects[$d.rects.Count - 1]; $lastEnd = $last[0] + $last[1]
  $shown = [math]::Abs($lastEnd - ($d.cw - 44 - 64)) -lt 1.5
  $want = $wpx -ge 216
  Check "rolled at $wpx px: count $(if ($want) { 'shown' } else { 'hidden' })" ($d.rolled -and $shown -eq $want) "rolled=$($d.rolled) cw=$($d.cw) last tab ends at $lastEnd (count room ends at $($d.cw - 108))"
  Crop "c09-rolled-$wpx" ($d.l + $d.ox) ($d.t + $d.oy) $d.cw 36 | Out-Null
}
Pf fence resize Alpha --w 300 | Out-Null
Pf fence unroll Alpha | Out-Null; Start-Sleep -Seconds 1

Case '2 folder portal as a tab'
Pf fence create --title Lima --rect '60,420,420,200' | Out-Null
Pf fence create --portal "$PfRun\Project files" --title 'Project files' --rect '560,60,300,200' | Out-Null
Pf fence create --title Oscar --rect '560,300,300,200' | Out-Null
Pf fence merge 'Project files' --into Lima | Out-Null
Pf fence merge Oscar --into Lima | Out-Null
Start-Sleep -Seconds 1; FenceIds; ParkPointer
$d = DumpOf 'Lima'
$pi = [array]::IndexOf(@($d.tabs), 'Project files')
$wIcon = $d.rects[$pi][1]
Crop 'c10-portal-tab' ($d.l + $d.ox) ($d.t + $d.oy) $d.cw 36 | Out-Null
$set = Pf fence set 'Project files' portalTitleIcon false
Start-Sleep -Milliseconds 900
$d2 = DumpOf 'Lima'
$wNo = $d2.rects[$pi][1]
Check 'the portal tab has room for the folder icon before its name' ([math]::Abs($wIcon - $wNo - 20) -lt 1) "width with icon=$wIcon without=$wNo; set: $($set.Trim())"
Crop 'c11-portal-tab-no-icon' ($d2.l + $d2.ox) ($d2.t + $d2.oy) $d2.cw 36 | Out-Null
$b1 = [Drawing.Bitmap]::FromFile("$PfShots\c10-portal-tab.png"); $b2 = [Drawing.Bitmap]::FromFile("$PfShots\c11-portal-tab-no-icon.png")
$p0 = [int]$d.rects[$pi][0]
$i1 = [Px]::FirstInk($b1, $p0 + 2, $p0 + 40, 8, 28, 50); $i2 = [Px]::FirstInk($b2, $p0 + 2, $p0 + 40, 8, 28, 50)
Log "portal pill first ink: with icon $i1, without $i2 (pill x $p0)"
Pf fence set 'Project files' portalTitleIcon true | Out-Null; Start-Sleep -Milliseconds 600

# Left column: rows (y, h); the first ink of each row's caption (glyph or text).
function RailInk($d, $name) {
  $b = Crop $name ($d.l + $d.ox) ($d.t + $d.oy) ([math]::Max($d.contentX, 40)) ($d.b - $d.t - $d.oy)
  $xs = @()
  foreach ($row in @($d.rects)) { $xs += [Px]::FirstInk($b, 2, $d.contentX - 30, [int]$row[0] + 9, [int]($row[0] + $row[1]) - 9, 60) }
  return $xs
}

Case '2+6 left tab column: names aligned, switch, reorder, roll up'
Pf settings set tabBar left | Out-Null; Start-Sleep -Seconds 1; ParkPointer
$d = DumpOf 'Lima'
Log "rail Lima: $($d.raw)"
$xs = RailInk $d 'c12-rail-icon'
$pi = [array]::IndexOf(@($d.tabs), 'Project files')
$others = @(); for ($i = 0; $i -lt $xs.Count; $i++) { if ($i -ne $pi) { $others += $xs[$i] } }
$aligned = ($others | Measure-Object -Maximum -Minimum)
Check 'rail: the names start at the same x' (($aligned.Maximum - $aligned.Minimum) -le 2 -and $aligned.Minimum -ge 24) "first ink per row: $($xs -join ',') (portal row $pi); content_x=$($d.contentX)"
Check 'rail: the folder glyph sits before the portal name' ($xs[$pi] -ge 0 -and $xs[$pi] -le $aligned.Minimum - 10) "portal row first ink=$($xs[$pi]), others from $($aligned.Minimum)"
Pf fence set 'Project files' portalTitleIcon false | Out-Null; Start-Sleep -Milliseconds 900
$d2 = DumpOf 'Lima'
$xs2 = RailInk $d2 'c13-rail-no-icon'
$m2 = ($xs2 | Measure-Object -Maximum -Minimum)
Check 'rail without the glyph: names aligned further left' (($m2.Maximum - $m2.Minimum) -le 2 -and $m2.Maximum -lt $aligned.Minimum - 10) "first ink per row: $($xs2 -join ',')"
Pf fence set 'Project files' portalTitleIcon true | Out-Null; Start-Sleep -Milliseconds 600
# Switch with a click on the last row.
$d = DumpOf 'Lima'; $n = @($d.rects).Count
$rx = Sx $d ($d.contentX / 2); $ry = { param($i) [int]($d.t + $d.oy + $d.rects[$i][0] + $d.rects[$i][1] / 2) }
[G]::Click($rx, (& $ry ($n - 1))); Start-Sleep -Milliseconds 700
$d2 = DumpOf 'Lima'
Check 'rail: a click on the last row switches to it' ((ActiveIdx $d2) -eq $n - 1) "active=$(ActiveName $d2)"
# Reorder: the first row dragged below the last.
$first = $d.tabs[0]
[G]::MoveTo($rx, (& $ry 0)); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 100
Glide $rx (& $ry 0) $rx ((& $ry ($n - 1)) + 12) 12 30; Start-Sleep -Milliseconds 500; MUp; Start-Sleep -Milliseconds 800
$d3 = DumpOf 'Lima'
Check 'rail: the first row dragged down lands last' ($d3.tabs[$n - 1] -eq $first) "order $($d3.tabs -join ',')"
ParkPointer
# Roll up to the column.
Pf fence roll Lima | Out-Null; Start-Sleep -Seconds 1
$g = FenceGet Lima; $d4 = DumpOf 'Lima'
Check 'rail: rolled up, the fence shrinks to the column' ($d4.rolled -and ($d4.r - $d4.l) -le $d4.contentX + 2) "window $($d4.l),$($d4.t),$($d4.r),$($d4.b) column=$($d4.contentX) windowRect=$($g.windowRect | ConvertTo-Json -Compress)"
Shot 'c15-rail-rolled'
Pf fence unroll Lima | Out-Null; Start-Sleep -Seconds 1
# The column's chevron (hover, count and chevron shown) as the baseline for case 3.
$d = DumpOf 'Lima'
[G]::MoveTo((Sx $d ($d.contentX / 2)), (& $ry 0)); Start-Sleep -Milliseconds 700
$H = $d.b - $d.t - $d.oy
$rb = Crop 'c16-rail-chevron-shown' ($d.l + $d.ox) ($d.t + $d.oy) ([math]::Max($d.contentX, 40)) $H
$cvx = [int]($d.contentX / 2); $railChevInk = [Px]::MaxInk($rb, $cvx - 12, $cvx + 12, $H - 36, $H - 10)
Log "rail chevron ink with the chevron shown: $railChevInk"
ParkPointer
Pf settings set tabBar top | Out-Null; Start-Sleep -Seconds 1

Case '3 hidden count and chevron'
# Baseline: Kilo's chevron while its title is hovered.
$k = DumpOf 'Kilo'
[G]::MoveTo((Sx $k 60), (TitleY $k)); Start-Sleep -Milliseconds 700
$kb = Crop 'c17-chevron-shown' ($k.l + $k.ox) ($k.t + $k.oy) $k.cw 36
$chevShown = [Px]::MaxInk($kb, $k.cw - 34, $k.cw - 6, 6, 30)
$hitShown = NcHit $k.hwnd (Sx $k ($k.cw - 20)) (TitleY $k)
ParkPointer
Pf settings set rollUp.hideCountAndChevron true | Out-Null; Start-Sleep -Seconds 1
$HideChevron = $true
$k = DumpOf 'Kilo'
[G]::MoveTo((Sx $k 60), (TitleY $k)); Start-Sleep -Milliseconds 700
$kb = Crop 'c18-chevron-hidden' ($k.l + $k.ox) ($k.t + $k.oy) $k.cw 36
$chevHidden = [Px]::MaxInk($kb, $k.cw - 34, $k.cw - 6, 6, 30)
$hitHidden = NcHit $k.hwnd (Sx $k ($k.cw - 20)) (TitleY $k)
Check 'no chevron on a hovered title' ($chevShown -ge 60 -and $chevHidden -lt 25) "ink in the chevron box: shown=$chevShown hidden=$chevHidden; NCHITTEST there: shown=$hitShown hidden=$hitHidden"
# Rolled tabbed fence: the strip runs to the right padding (no count room).
ToLastTab 'Alpha'
Pf fence roll Alpha | Out-Null; Start-Sleep -Seconds 1; ParkPointer
$d = DumpOf 'Alpha'
$last = $d.rects[$d.rects.Count - 1]
Check 'rolled with many tabs: no count, the strip ends at the padding' ($d.rolled -and [math]::Abs($last[0] + $last[1] - ($d.cw - 8)) -lt 1.5) "last tab ends at $($last[0] + $last[1]), cw=$($d.cw)"
Crop 'c19-rolled-hidden-count' ($d.l + $d.ox) ($d.t + $d.oy) $d.cw 36 | Out-Null
Pf fence unroll Alpha | Out-Null; Start-Sleep -Seconds 1
# The chevron's place drags the fence and opens the menu.
$k = DumpOf 'Kilo'; $g0 = FenceGet Kilo
$cx = Sx $k ($k.cw - 20); $cy = TitleY $k
[G]::MoveTo($cx, $cy); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 100
Glide $cx $cy ($cx - 150) ($cy + 40) 15 30; Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 900
$g1 = FenceGet Kilo
Check 'a drag from the chevron place moves the fence' ([math]::Abs($g1.rect.x - $g0.rect.x + 150) -le 3 -and [math]::Abs($g1.rect.y - $g0.rect.y - 40) -le 3 -and -not $g1.rolledUp) "rect before $($g0.rect | ConvertTo-Json -Compress) after $($g1.rect | ConvertTo-Json -Compress) rolled=$($g1.rolledUp)"
$k = DumpOf 'Kilo'
[G]::RClick((Sx $k ($k.cw - 20)), (TitleY $k)); Start-Sleep -Milliseconds 800
$items = [Menu]::Items()
Check 'a right-click at the chevron place opens the fence menu' ($items -match 'Roll up') "menu: $items"
CloseMenus
# Double-click on the title rolls up and expands.
$k = DumpOf 'Kilo'
DoubleClick (Sx $k 60) (TitleY $k); Start-Sleep -Milliseconds 900
$r1 = (FenceGet Kilo).rolledUp
DoubleClick (Sx $k 60) (TitleY $k); Start-Sleep -Milliseconds 900
$r2 = (FenceGet Kilo).rolledUp
Check 'title double-click rolls up, again expands' ($r1 -eq $true -and $r2 -eq $false) "after 1st=$r1 2nd=$r2"
ParkPointer
# The fence menu rolls up and expands.
[G]::RClick((Sx $k 60), (TitleY $k)); Start-Sleep -Milliseconds 800
$ok1 = MenuClick 'Roll up'; CloseMenus; ParkPointer; Start-Sleep -Milliseconds 600
$r1 = (FenceGet Kilo).rolledUp
[G]::RClick((Sx $k 60), (TitleY $k)); Start-Sleep -Milliseconds 800
$items = [Menu]::Items()
$ok2 = MenuClick 'Expand'; CloseMenus; ParkPointer; Start-Sleep -Milliseconds 600
$r2 = (FenceGet Kilo).rolledUp
Check 'the menu rolls up and expands' ($ok1 -and $ok2 -and $r1 -eq $true -and $r2 -eq $false) "clicked=$ok1/$ok2 rolled after=$r1/$r2 menu 2: $items"
# The left column has no chevron either.
Pf settings set tabBar left | Out-Null; Start-Sleep -Seconds 1
$d = DumpOf 'Lima'
[G]::MoveTo((Sx $d ($d.contentX / 2)), [int]($d.t + $d.oy + $d.rects[0][0] + 16)); Start-Sleep -Milliseconds 700
$H = $d.b - $d.t - $d.oy
$rb = Crop 'c20-rail-chevron-hidden' ($d.l + $d.ox) ($d.t + $d.oy) ([math]::Max($d.contentX, 40)) $H
$cvx = [int]($d.contentX / 2); $railHidden = [Px]::MaxInk($rb, $cvx - 12, $cvx + 12, $H - 36, $H - 10)
Check 'left column: no chevron while hovered' ($railChevInk -ge 60 -and $railHidden -lt 25) "ink at the column chevron: shown=$railChevInk hidden=$railHidden"
ParkPointer

Case '6 left column with the count and chevron hidden'
$d = DumpOf 'Lima'; $n = @($d.rects).Count
$rx = Sx $d ($d.contentX / 2)
[G]::Click($rx, [int]($d.t + $d.oy + $d.rects[0][0] + $d.rects[0][1] / 2)); Start-Sleep -Milliseconds 700
$d2 = DumpOf 'Lima'
Check 'rail (hidden chevron): a click switches to the first row' ((ActiveIdx $d2) -eq 0) "active=$(ActiveName $d2) order $($d2.tabs -join ',')"
$xs = RailInk $d2 'c21-rail-hidden'
$pi = [array]::IndexOf(@($d2.tabs), 'Project files')
$others = @(); for ($i = 0; $i -lt $xs.Count; $i++) { if ($i -ne $pi) { $others += $xs[$i] } }
$m = $others | Measure-Object -Maximum -Minimum
Check 'rail (hidden chevron): names still aligned' (($m.Maximum - $m.Minimum) -le 2) "first ink per row: $($xs -join ',')"
ParkPointer
Pf fence roll Lima | Out-Null; Start-Sleep -Seconds 1
$d4 = DumpOf 'Lima'
Check 'rail (hidden chevron): rolls up to the column' ($d4.rolled -and ($d4.r - $d4.l) -le $d4.contentX + 2) "window width $($d4.r - $d4.l) column $($d4.contentX)"
Pf fence unroll Lima | Out-Null; Start-Sleep -Seconds 1
Pf settings set tabBar top | Out-Null
Pf settings set rollUp.hideCountAndChevron false | Out-Null; Start-Sleep -Seconds 1
$HideChevron = $false

Case '5a hover switching with the chevron hidden'
$two = '文档', '图片', '音乐', '视频', '下载', '工具', '游戏', '项目'
Pf fence create --title $two[0] --rect '560,360,300,200' | Out-Null
foreach ($t in $two[1..7]) { Pf fence create --title $t --rect '900,360,300,200' | Out-Null; Pf fence merge $t --into $two[0] | Out-Null }
Start-Sleep -Seconds 1; FenceIds; ParkPointer
# Rests the pointer at strip x $at for 6 s; returns the active tabs seen, one per dump.
function RestAt($at) {
  ToFirstTab $two[0]
  Pf settings set rollUp.switchTabOnHover true | Out-Null; Start-Sleep -Milliseconds 500
  $d = DumpOf $two[0]
  Log "hover host: $(RectsText $d) cw=$($d.cw)"
  [G]::MoveTo((Sx $d $at), (TitleY $d))
  $seen = @(); $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 6) { $x = DumpOf $two[0] 2; if ($x) { $seen += (ActiveIdx $x) } }
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  ParkPointer
  $runs = @(); foreach ($s in $seen) { if ($runs.Count -eq 0 -or $runs[-1] -ne $s) { $runs += $s } }
  return @($runs, $d)
}
foreach ($hide in $true, $false) {
  Pf settings set rollUp.hideCountAndChevron ("$hide".ToLower()) | Out-Null; Start-Sleep -Milliseconds 800
  foreach ($at in 250, 280) {
    $res = RestAt $at; $runs = $res[0]
    $label = if ($hide) { 'chevron hidden' } else { 'chevron shown' }
    Log "5a $label at ${at}: active sequence $($runs -join ' > ')"
    Check "5a $label, pointer resting at x=${at}: at most one switch" ($runs.Count -le 2) "active sequence $($runs -join ' > ') (tabs $(RectsText $res[1]))"
  }
}
Pf settings set rollUp.hideCountAndChevron false | Out-Null

# A click on the tab cut off at the right end, filmed: how far the strip moves per frame.
function SlideOrJump($label) {
  ToFirstTab 'Alpha'
  $d = DumpOf 'Alpha'; $end = StripEnd $d
  $cut = -1; for ($i = 0; $i -lt @($d.rects).Count; $i++) { if ($d.rects[$i][0] -lt $end - 16 -and $d.rects[$i][0] + $d.rects[$i][1] -gt $end) { $cut = $i; break } }
  $cx = Sx $d (($d.rects[$cut][0] + $end - 14) / 2)
  [G]::MoveTo($cx, (TitleY $d)); Start-Sleep -Milliseconds 600
  [Px]::Burst(($d.l + $d.ox), ($d.t + $d.oy), $d.cw, 36, 600, 1)
  $track = [Px]::Track(8, 22, 20, 33)
  [Px]::Strip("$PfShots\c22-click-cutoff-$label-film.png", 1)
  Log "5b $label track (ms:marker:shift): $track"
  $shifts = @($track -split ' ' | ForEach-Object { [int]($_ -split ':')[2] })
  $moving = @($shifts | Where-Object { $_ -ne 0 }).Count
  $total = ($shifts | Measure-Object -Sum).Sum
  $biggest = ($shifts | ForEach-Object { [math]::Abs($_) } | Measure-Object -Maximum).Maximum
  # The accent marker under the active tab: positions between the first and the last frame's
  # are frames of a slide (the profile shift alone misreads a frame now and then).
  $marks = @($track -split ' ' | ForEach-Object { [double]($_ -split ':')[1] })
  $m0 = $marks[0]; $m1 = $marks[-1]
  $between = @($marks | Where-Object { [math]::Abs($_ - $m0) -gt 3 -and [math]::Abs($_ - $m1) -gt 3 }).Count
  # With Windows' animations off PecoFence snaps on purpose (its motion follows that setting).
  $slides = $moving -ge 2 -and $between -ge 2
  $what = if ([Anim]::Get()) { 'the strip slides (several frames move) rather than jumps' } else { 'the strip moves in one step (animations off)' }
  $ok = if ([Anim]::Get()) { $slides } else { $between -eq 0 -and [math]::Abs($m1 - $m0) -gt 20 }
  Check "5b ${label}: $what" $ok "frames=$($shifts.Count) strip moving in $moving, marker between start and end in $between; tab $cut; track (ms:marker:shift) $track"
  ParkPointer
}
# Press on the tab that barely shows at the right end and drag it 40 px left at once (out of
# the 32 DIP end zone, where holding scrolls the strip): the tab must stay in its slot.
function PressDrag($label) {
  ToFirstTab 'Alpha'
  $d = DumpOf 'Alpha'; $end = StripEnd $d
  $cut = -1; for ($i = 0; $i -lt @($d.rects).Count; $i++) { if ($d.rects[$i][0] -lt $end - 4 -and $d.rects[$i][0] + $d.rects[$i][1] -gt $end) { $cut = $i; break } }
  $name = $d.tabs[$cut]
  $px = Sx $d ($d.rects[$cut][0] + 4); $y = TitleY $d
  Log "5b2 $label press $name (index $cut) at x=$px, visible $([math]::Round($end - $d.rects[$cut][0], 1)) DIP; $(RectsText $d)"
  [G]::MoveTo($px, $y); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 40
  Glide $px $y ($px - 40) $y 2 10
  [Px]::Burst(($d.l + $d.ox), ($d.t + $d.oy), $d.cw, 36, 300, 0)
  $mid = DumpOf 'Alpha'
  [Px]::Strip("$PfShots\c23-press-drag-$label-film.png", 2)
  $track = [Px]::Track(8, 22, 20, 33)
  $mi = [array]::IndexOf(@($mid.tabs), $name)
  $slot = if ($mi -ge 0) { "$([math]::Round($mid.rects[$mi][0]))..$([math]::Round($mid.rects[$mi][0] + $mid.rects[$mi][1]))" } else { '-' }
  Check "5b2 ${label}: dragged 40 px right after the press, the tab keeps its slot" ($mi -eq $cut) "$name pressed at index $cut, now at $mi (slot $slot); pointer at strip x $($px - 40 - $d.l - $d.ox); marker track $track; order $($mid.tabs -join ','); tab_drag=$($mid.tabDrag)"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 600
  ParkPointer
}

Case '5b click on a cut-off tab: slide or jump'
SlideOrJump 'anim-on'
PressDrag 'anim-on'
[Anim]::Set($false); Start-Sleep -Milliseconds 800
SlideOrJump 'anim-off'
PressDrag 'anim-off'
[Anim]::Set($true); Start-Sleep -Milliseconds 800

# Narrows Alpha (its last tab active) by a real drag of its right edge; compares the strip's
# right end right after the release with the same end after a hover redraw.
function EdgeNarrow($label) {
  Pf fence resize Alpha --w 300 | Out-Null; Start-Sleep -Milliseconds 800
  ToLastTab 'Alpha'
  $d = DumpOf 'Alpha'
  $my = [int](($d.t + $d.b) / 2)
  $ex = -1; for ($x = $d.r - 1; $x -ge $d.r - 12; $x--) { if ((NcHit $d.hwnd $x $my) -eq 11) { $ex = $x; break } }
  Check "5c ${label}: the right edge is a sizing border" ($ex -gt 0) "x=$ex window right $($d.r)"
  [G]::MoveTo($ex, $my); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 100
  Glide $ex $my ($ex - 40) $my 10 30; Start-Sleep -Milliseconds 200; MUp
  Start-Sleep -Milliseconds 600
  $d2 = DumpOf 'Alpha'
  $a = Crop "c24-$label-after-release" ($d2.l + $d2.ox) ($d2.t + $d2.oy) $d2.cw 36
  [G]::MoveTo((Sx $d2 60), (TitleY $d2)); Start-Sleep -Milliseconds 700; ParkPointer; Start-Sleep -Milliseconds 700
  $bb = Crop "c25-$label-after-redraw" ($d2.l + $d2.ox) ($d2.t + $d2.oy) $d2.cw 36
  $last = $d2.rects[$d2.rects.Count - 1]
  $x1 = [int]($last[0] + $last[1])
  $diff = [Px]::Diff($a, $bb, $x1 - 12, $x1 - 1, 6, 30)
  Check "5c ${label}: after the release the right end has no stale fade" ($diff -lt 4) "cw $($d.cw) -> $($d2.cw); active $(ActiveName $d2), last tab ends at $x1 (strip end $(StripEnd $d2)); mean brightness difference of its last 12 px, after release vs after a redraw = $([math]::Round($diff, 2))"
}

Case '5c narrowing by the right edge: stale fade at the right end'
foreach ($anim in $true, $false) {
  [Anim]::Set($anim); Start-Sleep -Milliseconds 600
  $al = if ($anim) { 'anim-on' } else { 'anim-off' }
  Pf settings set rollUp.hideCountAndChevron false | Out-Null; $HideChevron = $false; Start-Sleep -Milliseconds 600
  EdgeNarrow "chevron-shown-$al"
  Pf settings set rollUp.hideCountAndChevron true | Out-Null; $HideChevron = $true; Start-Sleep -Milliseconds 600
  EdgeNarrow "chevron-hidden-$al"
}
Pf settings set rollUp.hideCountAndChevron false | Out-Null; $HideChevron = $false

Case 'cleanup'
PfQuit
[Anim]::Set($AnimWas)
foreach ($f in $made) { Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue }
