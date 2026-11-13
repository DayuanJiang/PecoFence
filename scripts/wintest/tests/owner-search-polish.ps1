# Owner round 2026-11-12, package search-polish: how the Peek search box looks and answers.
# PK-2 the row and the type tab under the pointer get a hover fill (83 ms fade), a row the hand,
#      a click opens the row as Enter does and the hover does not move the highlighted row;
# PK-5 a file that two fences show is listed once (from the leftmost fence);
# PK-6 the empty box has a grey hint; PK-7 the box follows the Windows accent colour, also while
#      it is open; PK-8 the bottom bar writes "Enter"; PK-9 a long folder of a whole-computer
#      result is cut in the middle and the row's tip has the full path; PK-11 the typed text
#      starts where the names start; PK-12 Chinese in the box is in the list's font; PK-13 the
#      box has an outline; PK-14 an action row's bar offers only Enter.
# Parts (run.sh --var Part=...): A without Everything (PK-2, 5, 6, 7, 8, 11, 12, 13), B with
# Everything (PK-6 whole computer, 9, 14), C the hint while Microsoft Pinyin composes (pf's
# language list gets zh-Hans-CN for the run). Default: A and B. --var Record=1 also records PK-2.
# Part C is for a box that never had the Chinese input language: on ec2-33 Pinyin showed its
# letters and candidates in the first run only; in every later run (main's build too) the
# keys started a composition that stayed invisible, so the two checks about it fail there.
# The geometry is the box's at 100 %.
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1

Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System; using System.Text; using System.Drawing; using System.Drawing.Imaging;
using System.Diagnostics; using System.Runtime.InteropServices;
public static class SpPx {
  [StructLayout(LayoutKind.Sequential)] struct CI { public int cb; public int flags; public IntPtr h; public int x; public int y; }
  [DllImport("user32.dll")] static extern bool GetCursorInfo(ref CI ci);
  [DllImport("user32.dll")] static extern IntPtr LoadCursorW(IntPtr inst, IntPtr id);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, string l, uint f, uint t, out IntPtr r);
  // The pointer's shape: hand, arrow, ibeam or other.
  public static string Cursor() {
    var ci = new CI(); ci.cb = Marshal.SizeOf(typeof(CI)); GetCursorInfo(ref ci);
    if (ci.h == LoadCursorW(IntPtr.Zero, (IntPtr)32649)) return "hand";
    if (ci.h == LoadCursorW(IntPtr.Zero, (IntPtr)32512)) return "arrow";
    if (ci.h == LoadCursorW(IntPtr.Zero, (IntPtr)32513)) return "ibeam";
    return "other";
  }
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, IntPtr pid);
  [DllImport("user32.dll")] static extern IntPtr GetKeyboardLayout(uint thread);
  // The keyboard layout of the window in front (ends in 804 for Chinese, PRC).
  public static string Layout() { return GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero)).ToInt64().ToString("X"); }
  [DllImport("imm32.dll")] static extern IntPtr ImmGetDefaultIMEWnd(IntPtr h);
  [DllImport("user32.dll")] static extern IntPtr SendMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  // The input method's mode in the window in front (WM_IME_CONTROL, IMC_GETCONVERSIONMODE): bit 1 set = Chinese, clear = its English mode.
  public static int ImeMode() { return (int)SendMessageW(ImmGetDefaultIMEWnd(GetForegroundWindow()), 0x283, (IntPtr)1, IntPtr.Zero); }
  // What Windows sends every window when the accent colour changes.
  public static void ColorsChanged() { IntPtr r; SendMessageTimeoutW((IntPtr)0xFFFF, 0x001A, IntPtr.Zero, "ImmersiveColorSet", 2, 2000, out r); }
  static int[] Grab(int x, int y, int w, int h) {
    using (var b = new Bitmap(w, h, PixelFormat.Format32bppArgb)) {
      using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, new Size(w, h));
      var d = b.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
      var a = new int[w * h]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
    }
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  // One screen pixel as RRGGBB.
  public static string Rgb(int x, int y) { return (Grab(x, y, 1, 1)[0] & 0xFFFFFF).ToString("X6"); }
  // The pixels of a screen rectangle that differ in brightness from its top-left pixel by more
  // than thr: left, right, top, bottom (screen pixels) and how many; -1 and 0 when none.
  public static int[] Ink(int x, int y, int w, int h, int thr) {
    var a = Grab(x, y, w, h); int bg = Lum(a[0]);
    int x0 = int.MaxValue, x1 = -1, y0 = int.MaxValue, y1 = -1, n = 0;
    for (int yy = 0; yy < h; yy++) for (int xx = 0; xx < w; xx++) {
      if (Math.Abs(Lum(a[yy * w + xx]) - bg) <= thr) continue;
      n++; x0 = Math.Min(x0, xx); x1 = Math.Max(x1, xx); y0 = Math.Min(y0, yy); y1 = Math.Max(y1, yy);
    }
    if (n == 0) return new int[] { -1, -1, -1, -1, 0 };
    return new int[] { x + x0, x + x1, y + y0, y + y1, n };
  }
  // The colour furthest in brightness from the rectangle's top-left pixel (the text's own colour).
  public static string InkColor(int x, int y, int w, int h) {
    var a = Grab(x, y, w, h); int bg = Lum(a[0]), best = 0, c = a[0];
    foreach (var p in a) { int d = Math.Abs(Lum(p) - bg); if (d > best) { best = d; c = p; } }
    return (c & 0xFFFFFF).ToString("X6");
  }
  // How many pixels of two same-size screen rectangles differ in brightness by more than thr.
  public static int Differ(int ax, int ay, int bx, int by, int w, int h, int thr) {
    var a = Grab(ax, ay, w, h); var b = Grab(bx, by, w, h); int n = 0;
    for (int i = 0; i < a.Length; i++) if (Math.Abs(Lum(a[i]) - Lum(b[i])) > thr) n++;
    return n;
  }
  // One pixel read as fast as the screen allows for ms: "ms:RRGGBB ms:RRGGBB ...".
  public static string Series(int x, int y, int ms) {
    var sb = new StringBuilder(); var sw = Stopwatch.StartNew();
    while (sw.ElapsedMilliseconds < ms) { var c = Grab(x, y, 1, 1)[0] & 0xFFFFFF; sb.Append(sw.ElapsedMilliseconds + ":" + c.ToString("X6") + " "); }
    return sb.ToString().Trim();
  }
}
'@

# ---- helpers
function SpBox() { $h = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { return $h }; return [IntPtr]::Zero }
function SpEnd() { for ($i = 0; $i -lt 4 -and (SpBox) -ne [IntPtr]::Zero; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 } }
# A Peek from the desktop with $text typed; returns the box's rect.
function SpSearch($text, $ms = 1000) {
  SpEnd
  DeskClick 1500 820 | Out-Null
  PeekKey; Start-Sleep -Milliseconds 800
  [G]::Type($text); Start-Sleep -Milliseconds $ms
  return (Rect (SpBox))
}
# Middle of row $i (0 = first) when $headings group headings are above it.
function SpRowY($r, $i, $headings = 1) { return $r.T + 56 + 38 + 4 + 24 * $headings + 44 * $i + 22 }
# The type tabs' left and right edges in the box, from the app's log.
function SpTabs() {
  $line = @(PfLogAll | Where-Object { $_ -match 'search tabs drawn' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
  return @([regex]::Matches("$line", '\((\d+), (\d+)\)') | ForEach-Object { , @([int]$_.Groups[1].Value, [int]$_.Groups[2].Value) })
}
function SpLaid() { return "$(@(PfLogAll | Where-Object { $_ -match 'search list laid out' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1)" -replace '^.*search list laid out', '' }
# The distinct colours of a pixel series, in order.
function SpSteps($series) { $out = @(); foreach ($s in ("$series" -split ' ')) { $c = ($s -split ':')[1]; if ($c -and ($out.Count -eq 0 -or $out[-1] -ne $c)) { $out += $c } }; return $out }
function SpRB($hex) { return [Convert]::ToInt32($hex.Substring(0, 2), 16) - [Convert]::ToInt32($hex.Substring(4, 2), 16) }
function SpG($hex) { return [Convert]::ToInt32($hex.Substring(2, 2), 16) }
function SpR($hex) { return [Convert]::ToInt32($hex.Substring(0, 2), 16) }
function SpB($hex) { return [Convert]::ToInt32($hex.Substring(4, 2), 16) }

$SpFiles = 'spq.txt', 'spq 第二个文件名比较长一些.txt', '报告 spq 第三.txt', '季度报告xspq第四（终稿）.txt', 'sp报告归档.txt'
$SpDup = Join-Path $PfDocs 'sp 资料'
$SpDeep = Join-Path $PfDocs 'sp项目资料归档'
$SpDeepA = Join-Path $SpDeep '2026 年度\第三季度\客户往来文件和合同扫描件\最终确认版本（含全部批注）甲'
$SpDeepB = Join-Path $SpDeep '2026 年度\第三季度\客户往来文件和合同扫描件\最终确认版本（含全部批注）乙'
function SpClean() {
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  foreach ($n in $SpFiles + 'spzdeep 桌面上的.txt') { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  Remove-Item -LiteralPath $SpDup -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $SpDeep -Recurse -Force -ErrorAction SilentlyContinue
}
$SpAccentKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent'
# Light3, Light2, Light1, the accent, Dark1, Dark2, Dark3, the complement (R, G, B, 0 each).
[byte[]]$SpOrange = 0xFF,0xE8,0xCC,0, 0xFF,0xC0,0x78,0, 0xFF,0xA9,0x4D,0, 0xF7,0x67,0x07,0, 0xD9,0x48,0x0F,0, 0xA6,0x3A,0x0C,0, 0x7A,0x2A,0x08,0, 0x00,0x63,0xB1,0
[byte[]]$SpGreen = 0xCC,0xF5,0xCC,0, 0x9A,0xE6,0x9A,0, 0x5C,0xD0,0x5C,0, 0x10,0xA0,0x10,0, 0x0F,0x7B,0x0F,0, 0x0B,0x5A,0x0B,0, 0x07,0x3D,0x07,0, 0x00,0x63,0xB1,0
function SpAccent($bytes) {
  if ($bytes) { Set-ItemProperty $SpAccentKey -Name AccentPalette -Value ([byte[]]$bytes) -Type Binary } else { Remove-ItemProperty $SpAccentKey -Name AccentPalette -ErrorAction SilentlyContinue }
  [SpPx]::ColorsChanged()
}

Case 'setup'
CloseWindows
Set-Culture zh-CN
$SpAnim = [Anim]::Get()
[Anim]::Set($true)
$SpAccent0 = (Get-ItemProperty $SpAccentKey -ErrorAction SilentlyContinue).AccentPalette
SpClean

function PartA() {
  trap { PfTrap $_; continue }
  Case 'A setup'
  foreach ($n in $SpFiles) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value 'sample' -Encoding UTF8 }
  New-Item -ItemType Directory -Force $SpDup | Out-Null
  foreach ($n in 'spdup 合同（终稿）.txt', 'spdup 报价单.txt') { Set-Content -LiteralPath (Join-Path $SpDup $n) -Value 'sample' -Encoding UTF8 }
  PfStart -Lang zh-CN
  Pf settings set theme light | Out-Null
  # The same folder in two fences: a folder fence at the left, a smart fence right of it.
  Pf fence create --portal $SpDup --title 'Left docs' --rect '40,60,200,200' | Out-Null
  Pf fence create --title 'Right docs' --smart-folder $SpDup --name-matches spdup --rect '250,60,200,200' | Out-Null
  Start-Sleep -Seconds 3

  Case 'PK-2 hover'
  $r = SpSearch 'spq'
  Check 'the search list shows the four files' ((SpLaid) -match 'results=4') (SpLaid)
  $px = $r.R - 40; $y2 = SpRowY $r 2
  $bg = [SpPx]::Rgb($px, $y2)
  $sel = [SpPx]::Rgb($px - 60, (SpRowY $r 0))
  if ($Record) { RecStart 'pk2' -X ($r.L - 20) -Y ($r.T - 20) -W 680 -H 400 -Fps 30; Start-Sleep -Milliseconds 300; RecMark 'pointer to the third row' }
  [G]::MoveTo($r.L + 300, $y2)
  $ser = [SpPx]::Series($px, $y2, 450)
  $steps = @(SpSteps $ser)
  $hov = [SpPx]::Rgb($px, $y2)
  Shot 'pk2-hover-row'
  Crop 'pk2-row' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-2 the row under the pointer gets a hover fill' ($hov -ne $bg) "row 3 at rest $bg, under the pointer $hov"
  # The screen recorder slows the pixel reads to one in 60 ms, too few to see an 83 ms fade.
  Check 'PK-2 the fill fades in: a frame between none and full' ($hov -ne $bg -and ($Record -or $steps.Count -ge 3) -and $steps[-1] -eq $hov) "colours seen: $($steps -join ' > ') ($ser)"
  Check 'PK-2 the hand over a row' ([SpPx]::Cursor() -eq 'hand') "cursor=$([SpPx]::Cursor())"
  Check 'PK-2 the highlighted row stays the first one' ([SpPx]::Rgb($px - 60, (SpRowY $r 0)) -eq $sel -and $sel -ne $bg -and $sel -ne $hov -and (SpLaid) -match 'selected=Some\(0\)') "first row $sel, hovered $hov, plain $bg; $(SpLaid)"
  $tabs = @(SpTabs)
  $tx = $r.L + $tabs[2][0] + 7; $ty = $r.T + 56 + 19
  $tab0 = [SpPx]::Rgb($tx, $ty)
  if ($Record) { RecMark 'pointer to a type tab' }
  [G]::MoveTo($r.L + [int](($tabs[2][0] + $tabs[2][1]) / 2), $ty); Start-Sleep -Milliseconds 400
  $tab1 = [SpPx]::Rgb($tx, $ty)
  Crop 'pk2-tab' $r.L $r.T ($r.R - $r.L) 100 | Out-Null
  Check 'PK-2 the type tab under the pointer gets a hover fill' ($tab1 -ne $tab0 -and $tab0 -eq $bg) "tab 3 at rest $tab0, under the pointer $tab1"
  Check 'PK-2 the arrow over a tab, the row left by the pointer is plain again' ([SpPx]::Cursor() -eq 'arrow' -and [SpPx]::Rgb($px, $y2) -eq $bg) "cursor=$([SpPx]::Cursor()) row 3 $([SpPx]::Rgb($px, $y2))"
  if ($Record) { RecMark 'pointer away' }
  ParkMouse 1500 820; Start-Sleep -Milliseconds 400
  if ($Record) { Start-Sleep -Milliseconds 600; RecStop }
  Check 'PK-2 the fills go when the pointer leaves the box' ([SpPx]::Rgb($tx, $ty) -eq $bg) "tab 3 $([SpPx]::Rgb($tx, $ty))"
  # Windows' animation effects off: the fill is there at once.
  [Anim]::Set($false); Start-Sleep -Milliseconds 300
  $y3 = SpRowY $r 3
  [G]::MoveTo($r.L + 300, $y3); Start-Sleep -Milliseconds 300
  $over = "$(@(PfLogAll | Where-Object { $_ -match 'pointer is over another row or tab' }) | Select-Object -Last 1)" -replace '^.*search: ', ''
  Check 'PK-2 animations off: the fill appears without a fade' ([SpPx]::Rgb($px, $y3) -eq $hov -and $over -match 'animate=false') "row 4 $([SpPx]::Rgb($px, $y3)); $over"
  [Anim]::Set($true); Start-Sleep -Milliseconds 300
  # The pointer rests on the fourth row; Enter still opens the highlighted first one.
  [G]::Press(0x0D); Start-Sleep -Seconds 3
  $t = NotepadTitles
  Check 'PK-2 Enter opens the highlighted row, not the one under the pointer' ($t -match '^spq(\.txt)? - ') "notepad=$t"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500
  $r = SpSearch 'spq'
  [G]::Click($r.L + 300, (SpRowY $r 2)); Start-Sleep -Seconds 3
  $t = NotepadTitles
  Check 'PK-2 a click on the third row opens it, as Enter on that row' ($t -like '*第三*' -and (SpBox) -eq [IntPtr]::Zero) "notepad=$t box open=$((SpBox) -ne [IntPtr]::Zero)"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500

  Case 'PK-5 one row per file'
  $r = SpSearch 'spdup'
  $laid = SpLaid
  Crop 'pk5-list' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-5 two files that two fences show: two rows, counted as two' ($laid -match 'rows=2 ' -and $laid -match 'total=2 ') $laid
  $d = OcrText ($r.L + 44) ((SpRowY $r 0) - 2) 400 22 'pk5-detail'
  Check 'PK-5 the row names the leftmost fence' ($d -match 'Left' -and $d -notmatch 'Right') "line under the first name reads '$d'"

  Case 'PK-6 hint in the empty box'
  foreach ($i in 1..5) { [G]::Press(0x08); Start-Sleep -Milliseconds 80 }
  Start-Sleep -Milliseconds 700
  $ink = [SpPx]::Ink($r.L + 44, $r.T + 14, 400, 28, 60)
  $cue = "$(@(PfLogAll | Where-Object { $_ -match 'the empty box shows its hint' }) | Select-Object -Last 1)" -replace '^.*search: ', ''
  Crop 'pk6-empty' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-6 the empty box shows a hint' ((SpBox) -ne [IntPtr]::Zero -and ($ink[1] - $ink[0]) -gt 150 -and $cue -match 'whole_pc=false') "box='$([M]::BoxText())' hint from x=$($ink[0] - $r.L) to $($ink[1] - $r.L), colour $([SpPx]::InkColor($r.L + 70, $r.T + 14, 300, 28)); $cue"
  SpEnd
  Pf settings set theme dark | Out-Null; Start-Sleep -Seconds 2
  $r = SpSearch 'a'
  [G]::Press(0x08); Start-Sleep -Milliseconds 800
  $ink = [SpPx]::Ink($r.L + 44, $r.T + 14, 400, 28, 25)
  Crop 'pk6-empty-dark' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-6 the hint shows in dark mode too' (($ink[1] - $ink[0]) -gt 150) "hint from x=$($ink[0] - $r.L) to $($ink[1] - $r.L), colour $([SpPx]::InkColor($r.L + 70, $r.T + 14, 300, 28)) on $([SpPx]::Rgb($r.L + 44, $r.T + 14))"
  SpEnd
  # The filter box of a folder fence uses the same hint: its look in dark mode, for the record.
  [G]::Click(100, 150); Start-Sleep -Milliseconds 500
  Chord 0x11 0x46; Start-Sleep -Milliseconds 900
  Crop 'pk6-filter-box-dark' 30 50 230 60 | Out-Null
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  Pf settings set theme light | Out-Null; Start-Sleep -Seconds 2

  Case 'PK-7 accent colour'
  SpAccent $SpOrange; Start-Sleep -Milliseconds 800
  $r = SpSearch 'spq'
  $tabs = @(SpTabs)
  $rowc = [SpPx]::Rgb($r.R - 100, (SpRowY $r 0)); $tabc = [SpPx]::Rgb($r.L + $tabs[0][0] + 7, $r.T + 56 + 19)
  $name = [SpPx]::InkColor($r.L + 48, (SpRowY $r 0) - 16, 24, 14)
  Crop 'pk7-orange' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-7 an orange accent: the highlighted row, the chosen tab and the matched letters are orange' ((SpRB $rowc) -gt 8 -and (SpRB $tabc) -gt 8 -and (SpRB $name) -gt 60) "row $rowc tab $tabc matched letters $name"
  SpAccent $SpGreen; Start-Sleep -Milliseconds 900
  $rowg = [SpPx]::Rgb($r.R - 100, (SpRowY $r 0)); $tabg = [SpPx]::Rgb($r.L + $tabs[0][0] + 7, $r.T + 56 + 19)
  Crop 'pk7-green-while-open' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-7 the accent changes to green while the box is open: the box follows' ((SpG $rowg) -gt (SpR $rowg) -and (SpG $rowg) -gt (SpB $rowg) -and (SpG $tabg) -gt (SpR $tabg)) "row $rowg tab $tabg"
  SpEnd
  SpAccent $SpAccent0; Start-Sleep -Milliseconds 500

  Case 'PK-8 PK-11 PK-12 the bar and the text'
  $r = SpSearch 'spq'
  $e = [SpPx]::Ink($r.L + 14, $r.B - 32, 10, 28, 50)
  $bar = OcrText ($r.L + 8) ($r.B - 34) 330 32 'pk8-bar'
  Crop 'pk8-bar' $r.L ($r.B - 40) 360 40 | Out-Null
  Check 'PK-8 the Enter hint is as tall as a letter' (($e[3] - $e[2] + 1) -ge 8) "first sign of the bar is $($e[3] - $e[2] + 1) px high; the bar reads '$bar'"
  $typed = [SpPx]::Ink($r.L + 40, $r.T + 14, 120, 28, 60)
  $named = [SpPx]::Ink($r.L + 40, (SpRowY $r 0) - 17, 120, 14, 60)
  $glyph = [SpPx]::Ink($r.L + 8, $r.T + 14, 30, 28, 40)
  $icon = [SpPx]::Ink($r.L + 8, (SpRowY $r 0) - 14, 30, 28, 25)
  Crop 'pk11-top' $r.L $r.T 300 180 | Out-Null
  Check 'PK-11 the typed text starts where the names start' ([math]::Abs($typed[0] - $named[0]) -le 2) "typed text from x=$($typed[0] - $r.L), first name from x=$($named[0] - $r.L); magnifier from x=$($glyph[0] - $r.L), row icon from x=$($icon[0] - $r.L)"
  $r = SpSearch '报告归档'
  $n = 99999
  foreach ($i in 1..6) { $k = ([SpPx]::Ink($r.L + 44, $r.T + 12, 110, 32, 110))[4]; if ($k -lt $n) { $n = $k }; Start-Sleep -Milliseconds 170 }
  $nrow = ([SpPx]::Ink($r.L + 48, (SpRowY $r 0) - 17, 80, 14, 60))[4]
  Crop 'pk12-han' $r.L $r.T 300 180 | Out-Null
  # The fallback face is a light weight: 347 dark pixels on main's build, 455 in the list's font.
  Check 'PK-12 Chinese in the box has the stroke weight of the list font' ($n -ge 400) "dark pixels of the four typed characters: $n (the same four in the first row, 14 px: $nrow)"
  SpEnd

  Case 'PK-13 outline'
  Pf settings set peek.dim false | Out-Null; Start-Sleep -Milliseconds 800
  Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
  $np = [G]::FindWindowW('Notepad', [NullString]::Value)
  [L]::MoveWindow($np, 300, 100, 1000, 700, $true) | Out-Null
  TitleClick $np
  PeekKey; Start-Sleep -Milliseconds 800
  [G]::Type('spq'); Start-Sleep -Milliseconds 1000
  $r = Rect (SpBox)
  $left = [SpPx]::Rgb($r.L, $r.T + 28); $right = [SpPx]::Rgb($r.R - 1, $r.T + 28); $top = [SpPx]::Rgb($r.L + 320, $r.T); $in = [SpPx]::Rgb($r.L + 4, $r.T + 28)
  Crop 'pk13-edge' ($r.R - 200) ($r.T - 20) 240 160 | Out-Null
  Check 'PK-13 dimming off, over a white window: the box has an outline' ($left -eq $right -and $left -eq $top -and $left -ne $in -and (SpR $left) -lt 215) "edge left $left right $right top $top, inside $in"
  SpEnd
  Pf settings set peek.dim true | Out-Null
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force

  # The box and its hint are destroyed while the program's thread ends (as the fence name box
  # in polish-quit-namebox.ps1).
  Case 'quit with the search box open'
  $r = SpSearch 'a'
  [G]::Press(0x08); Start-Sleep -Milliseconds 700
  $open = (SpBox) -ne [IntPtr]::Zero
  PfQuit
  Start-Sleep -Seconds 2
  $p = @(PfLogAll | Where-Object { $_ -match 'PANIC' })
  Check 'no panic when PecoFence quits with the empty search box open' ($open -and $p.Count -eq 0) "box open=$open; panic lines: $($p.Count); $(if ($p.Count) { $p[0].Substring(0, [math]::Min(300, $p[0].Length)) })"
}

function PartB() {
  trap { PfTrap $_; continue }
  Case 'B setup'
  New-Item -ItemType Directory -Force $SpDeepA, $SpDeepB | Out-Null
  Set-Content -LiteralPath (Join-Path $SpDeepA 'spzdeep 季度总结.txt') -Value 'x' -Encoding UTF8
  Set-Content -LiteralPath (Join-Path $SpDeepB 'spzdeep 客户名单.txt') -Value 'x' -Encoding UTF8
  Set-Content -LiteralPath (Join-Path $PfDesk 'spzdeep 桌面上的.txt') -Value 'x' -Encoding UTF8
  $ev = EvStart
  PfStart -Lang zh-CN
  Pf settings set theme light | Out-Null
  Pf settings set tools.everything true | Out-Null
  Start-Sleep -Seconds 4
  $r = SpSearch 'spzdeep' 2500
  $laid = SpLaid
  Check 'Everything answers: one fence row, two whole-computer rows, two action rows' ($ev -and $laid -match 'rows=5 ' -and $laid -match 'headings=3 ' -and $laid -match 'action_glyphs=2 ') "Everything running=$ev; $laid"

  Case 'PK-9 folder paths'
  # The line under the name of the two whole-computer rows (rows 2 and 3, two headings above).
  $ya = (SpRowY $r 1 2); $yb = (SpRowY $r 2 2)
  $diff = [SpPx]::Differ($r.L + 48, $ya, $r.L + 48, $yb, $r.R - $r.L - 64, 18, 60)
  $end = ([SpPx]::Ink($r.L + 48, $ya, $r.R - $r.L - 64, 18, 60))[1]
  Crop 'pk9-paths' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-9 two results from neighbouring folders read differently' ($diff -ge 8) "pixels that differ between the two folder lines: $diff; the first line ends $($r.R - $end) px before the right edge"
  [G]::MoveTo($r.L + 300, $ya - 8); Start-Sleep -Milliseconds 1500
  $tip = @([W]::Tops('tooltips_class32')) | Where-Object { [G]::IsWindowVisible($_) } | Select-Object -First 1
  $tipAt = 'none'; $tipOk = $false
  if ($tip) {
    $tr = Rect $tip
    $tipAt = "$($tr.L - $r.L),$($tr.T - $r.T) $($tr.R - $tr.L)x$($tr.B - $tr.T)"
    # Below the pointer, and as wide as a long path makes it (the tip wraps at 360 px).
    $tipOk = (($tr.R - $tr.L) -ge 250 -and [math]::Abs($tr.L - ($r.L + 300)) -le 8 -and ($tr.T - ($ya - 8)) -ge 10 -and ($tr.T - ($ya - 8)) -le 40)
    Crop 'pk9-tip-shot' ($r.L) ($r.T + 100) ($r.R - $r.L + 120) 300 | Out-Null
  }
  $shown = @(PfLogAll | Where-Object { $_ -match 'full path is shown in a tip' -and $_ -notmatch 'omitted' }).Count
  Check 'PK-9 the pointer resting on the row shows the full path in a tip' ($tipOk -and $shown -ge 1) "tip window (x,y in the box, size): $tipAt; the pointer is at 300,$($ya - 8 - $r.T); log lines=$shown"
  ParkMouse 1500 820; Start-Sleep -Milliseconds 500
  Check 'PK-9 the tip goes when the pointer leaves' (-not (@([W]::Tops('tooltips_class32')) | Where-Object { [G]::IsWindowVisible($_) })) ''

  Case 'PK-14 keys on an action row'
  [G]::Press(0x28); Start-Sleep -Milliseconds 300
  $file = [SpPx]::Ink($r.L + 14, $r.B - 32, 420, 28, 50)
  foreach ($i in 1..2) { [G]::Press(0x28); Start-Sleep -Milliseconds 200 }
  Start-Sleep -Milliseconds 300
  $act = [SpPx]::Ink($r.L + 14, $r.B - 32, 420, 28, 50)
  Crop 'pk14-action-row' $r.L ($r.B - 150) ($r.R - $r.L) 150 | Out-Null
  Check 'PK-14 a file row offers the three keys, an action row only Enter' (($file[1] - $r.L) -gt 250 -and ($act[1] - $r.L) -lt 150 -and (SpLaid) -match 'selected=Some\(3\)') "key hints end at x=$($file[1] - $r.L) on a file row and at x=$($act[1] - $r.L) on an action row; $(SpLaid)"

  Case 'PK-6 hint with Everything'
  foreach ($i in 1..8) { [G]::Press(0x08); Start-Sleep -Milliseconds 80 }
  Start-Sleep -Milliseconds 700
  $ink = [SpPx]::Ink($r.L + 44, $r.T + 14, 400, 28, 60)
  $cue = "$(@(PfLogAll | Where-Object { $_ -match 'the empty box shows its hint' }) | Select-Object -Last 1)" -replace '^.*search: ', ''
  Crop 'pk6-empty-everything' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) | Out-Null
  Check 'PK-6 with Everything running the hint names the whole computer' (($ink[1] - $ink[0]) -gt 150 -and $cue -match 'whole_pc=true') "hint from x=$($ink[0] - $r.L) to $($ink[1] - $r.L); $cue"
  SpEnd
  PfQuit
  EvStop | Out-Null
}

function PartC() {
  trap { PfTrap $_; continue }
  Case 'C setup'
  $langs = Get-WinUserLanguageList
  $added = -not ($langs | Where-Object { $_.LanguageTag -like 'zh-Hans*' })
  # Microsoft Pinyin starts in Chinese mode (it remembers an English mode from an earlier run).
  New-Item -Path 'HKCU:\Software\Microsoft\InputMethod\Settings\CHS' -Force | Out-Null
  Set-ItemProperty 'HKCU:\Software\Microsoft\InputMethod\Settings\CHS' -Name 'Default Mode' -Value 0 -Type DWord
  if ($added) { $langs.Add('zh-Hans-CN'); Set-WinUserLanguageList $langs -Force }
  foreach ($n in $SpFiles) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value 'sample' -Encoding UTF8 }
  PfStart -Lang zh-CN
  Pf settings set theme light | Out-Null
  Start-Sleep -Seconds 2

  Case 'PK-6 hint and the input method'
  $r = SpSearch 'a'
  [G]::Press(0x08); Start-Sleep -Milliseconds 700
  # Win+Space: Windows' key for the next input language, until it is Microsoft Pinyin (Alt+Shift
  # leaves the box in menu mode, and its Shift can put Pinyin in English mode).
  $first = [SpPx]::Layout()
  for ($i = 0; $i -lt 3 -and [SpPx]::Layout() -notmatch '804$'; $i++) { [G]::KeyDown(0x5B); [G]::Press(0x20); [G]::KeyUp(0x5B); Start-Sleep -Milliseconds 1200 }
  $layout = "$first > $([SpPx]::Layout()) mode $([SpPx]::ImeMode())"
  $empty = [SpPx]::Ink($r.L + 44, $r.T + 14, 400, 28, 60)
  # Pinyin remembers its English mode (the tray's 英): when the letters start no composition, a
  # Shift tap switches it to Chinese and the letters are typed again.
  $tries = ''
  foreach ($try in 1..3) {
    foreach ($c in 'BAO'.ToCharArray()) { [G]::Press([uint16][int]$c); Start-Sleep -Milliseconds 150 }
    Start-Sleep -Milliseconds 900
    $comp = [SpPx]::Ink($r.L + 44, $r.T + 14, 400, 28, 60)
    $cand = @([W]::Tops('Microsoft.IME.UIManager.CandidateWindow.Host')).Count
    $tries += "try $try text='$([M]::BoxText())' ink $($comp[0] - $r.L)..$($comp[1] - $r.L) candidates=$cand; "
    if ([M]::BoxText() -eq '' -and ($comp[0] -ne $empty[0] -or $comp[1] -ne $empty[1])) { break }
    foreach ($i in 1..4) { [G]::Press(0x08); Start-Sleep -Milliseconds 80 }
    [G]::Press(0x10); Start-Sleep -Milliseconds 700
  }
  Crop 'pk6-composing' $r.L $r.T ($r.R - $r.L) 140 | Out-Null
  Check 'PK-6 the hint gives way to the letters being composed' ($layout -match '804 mode' -and ($empty[1] - $empty[0]) -gt 150 -and $comp[4] -gt 0 -and ($comp[1] - $r.L) -lt 200) "layout $layout; $tries hint from x=$($empty[0] - $r.L) to $($empty[1] - $r.L); while composing ink from x=$($comp[0] - $r.L) to $($comp[1] - $r.L)"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  $back = [SpPx]::Ink($r.L + 44, $r.T + 14, 400, 28, 60)
  Crop 'pk6-after-composing' $r.L $r.T ($r.R - $r.L) 140 | Out-Null
  Check 'PK-6 Esc ends the composition: the box stays and the hint is back' ((SpBox) -ne [IntPtr]::Zero -and ($back[1] - $back[0]) -gt 150) "box open=$((SpBox) -ne [IntPtr]::Zero); hint from x=$($back[0] - $r.L) to $($back[1] - $r.L)"
  for ($i = 0; $i -lt 3 -and [SpPx]::Layout() -match '804$'; $i++) { [G]::KeyDown(0x5B); [G]::Press(0x20); [G]::KeyUp(0x5B); Start-Sleep -Milliseconds 1200 }
  SpEnd
  PfQuit
  if ($added) {
    $l = Get-WinUserLanguageList
    $keep = New-WinUserLanguageList ($l[0].LanguageTag)
    foreach ($x in $l) { if ($x.LanguageTag -notlike 'zh-Hans*' -and $x.LanguageTag -ne $l[0].LanguageTag) { $keep.Add($x.LanguageTag) } }
    Set-WinUserLanguageList $keep -Force
  }
  Check 'the language list is as before' (-not ($added -and (Get-WinUserLanguageList | Where-Object { $_.LanguageTag -like 'zh-Hans*' }))) "languages: $((Get-WinUserLanguageList | ForEach-Object { $_.LanguageTag }) -join ', ')"
}

if ($Part -match 'A') { PartA }
if ($Part -match 'B') { PartB }
if ($Part -match 'C') { PartC }

Case 'cleanup'
SpEnd
if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
SpAccent $SpAccent0
[Anim]::Set($SpAnim)
SpClean
