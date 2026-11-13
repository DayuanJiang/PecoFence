# Owner round 2026-11-13, package ux6-align: two per-fence options asked for in GitHub #222.
#   A  图标对齐 右对齐 (fence set <fence> iconAlign right): every row fills from the right edge,
#      the first icon at the right and the next to its left. Clicks, the arrow keys, the
#      selection rectangle, the insertion mark and the drop, 按时间分组, a fence that scrolls
#      and a tab group all go by the mirrored cells. In a fence a whole number of cells wide
#      the mark at the right edge is whole (2 px), left-aligned too.
#   T  鼠标移入时保持透明 (fence set <fence> transparentOnHover true): a fully transparent
#      fence looks under the pointer as it does at rest, in 浅色 and 深色, Fluent and Liquid
#      Glass: no plate, no folded title row; the icons keep their own hover fill; the unseen
#      title row still moves the fence (move cursor), the edges still resize it, the roll-up
#      chevron takes no click; a rolled-up fence shows its plate and title as before. In a
#      Peek, where its glass shows, the pointer brings the title row.
#   P  the two options in 栅栏选项 (a debug build: run.sh --bin target/wintest-bin-debug --var
#      Part=P): 右对齐 is in the 图标对齐 list; 鼠标移入时保持透明 shows only while 不透明度
#      is 全透明, and both reach the fence.
#   S  右对齐 at 150 %: run through tests/free-placement.sh --box <box> --part S --steps <this
#      file> (it answers C:\pf\resize-want.txt by reconnecting the RDP session and ends at
#      1600x900, 100 %).
# Parts: run.sh --var Part=A ... (default: A and T). Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'AT' }
# An error inside a part's block becomes a failed check and the next line runs.
trap { PfTrap $_; continue }

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class Oa {
  [StructLayout(LayoutKind.Sequential)] public struct CI { public int cbSize, flags; public IntPtr hCursor; public int x, y; }
  [DllImport("user32.dll")] static extern bool GetCursorInfo(ref CI ci);
  [DllImport("user32.dll")] static extern IntPtr LoadCursorW(IntPtr inst, IntPtr name);
  // The standard cursor shown now: arrow, sizeall, sizewe ... ("other" for any other).
  public static string Cursor() {
    var ci = new CI(); ci.cbSize = Marshal.SizeOf(typeof(CI)); GetCursorInfo(ref ci);
    int[] ids = { 32512, 32646, 32644, 32645, 32642, 32643, 32649, 32513 };
    string[] names = { "arrow", "sizeall", "sizewe", "sizens", "sizenwse", "sizenesw", "hand", "ibeam" };
    for (int i = 0; i < ids.Length; i++) if (LoadCursorW(IntPtr.Zero, (IntPtr)ids[i]) == ci.hCursor) return names[i];
    return "other";
  }
  static int[] Pixels(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var a = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
  }
  // How many pixels of two grabs of one region differ by more than 24 (R + G + B), or thr.
  public static int Diff(Bitmap a, Bitmap b) { return Diff(a, b, 24); }
  public static int Diff(Bitmap a, Bitmap b, int thr) {
    var pa = Pixels(a); var pb = Pixels(b); int n = 0;
    for (int i = 0; i < pa.Length && i < pb.Length; i++) {
      int x = pa[i], y = pb[i];
      int d = Math.Abs(((x >> 16) & 255) - ((y >> 16) & 255)) + Math.Abs(((x >> 8) & 255) - ((y >> 8) & 255)) + Math.Abs((x & 255) - (y & 255));
      if (d > thr) n++;
    }
    return n;
  }
}
'@

# The files of this test (every name starts with "oa ").
function OaClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'oa *' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
function P($n) { return (Join-Path $PfDesk "$n.txt") }
# A fence sorted by name that holds new desktop files with these names.
function OaFence($title, $rect, $names) {
  foreach ($n in $names) { Set-Content -LiteralPath (P $n) "owner6-align $n" -Encoding UTF8 }
  Start-Sleep -Seconds 2
  Pf fence create --title $title --rect $rect | Out-Null
  Pf item move @($names | ForEach-Object { P $_ }) --to $title | Out-Null
  Pf fence set $title sort name | Out-Null
  Start-Sleep -Seconds 2
}
function Order($t) { return ((@(PfJson item list --fence $t) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name }) -join ' | ') }
function FR($t) { return (FenceGet $t).windowRect }
# Centre of the icon in row $row, $fromRight columns from the right edge, of the window of
# $fence ($k: the display scale; $dy: DIPs the rows start lower, a group header).
function OaCell($fence, $row, $fromRight, $dy = 0, $k = 1) {
  $wr = FR $fence
  $cols = [math]::Max(1, [math]::Floor($wr.w / (80 * $k)))
  $pitch = $wr.w / $cols
  return @([int]($wr.x + ($cols - 1 - $fromRight) * $pitch + $pitch / 2), [int]($wr.y + (36 + 8 + $row * 96 + 32 + $dy) * $k))
}
function OaSel($t) { return "$(DumpVal (DumpOf $t) 'selected')" }
function OaFocus($t) { $w = DumpOf $t; if ($w -and $w.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { return $Matches[1] }; return '' }
# A click on a cell, then the name of the icon that has the focus ('' when the click hit none).
function OaClickFocus($fence, $row, $fromRight, $dy = 0, $k = 1, $dump = '') {
  if (-not $dump) { $dump = $fence }
  $p = OaCell $fence $row $fromRight $dy $k
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  $w = DumpOf $dump
  $f = if ("$(DumpVal $w 'selected')" -eq '0') { '' } elseif ($w -and $w.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { $Matches[1] } else { '' }
  Log "click row $row, $fromRight from the right of '$fence' at $($p -join ','): focused '$f'"
  return $f
}
function Hold($x0, $y0, $x1, $y1, $ms = 700) {
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 200
  MDown; Start-Sleep -Milliseconds 250
  Glide $x0 $y0 $x1 $y1 16 30
  Start-Sleep -Milliseconds $ms
}
# How much the darkest-to-lightest column of a strip differs: an insertion line is a 2 px
# column of its own colour on the plain plate.
function ColumnContrast($x, $y, $w, $h, $name) {
  $b = [Px]::Grab($x, $y, $w, $h); $b.Save("$PfShots\$name.png")
  $cols = @()
  for ($i = 0; $i -lt $w; $i++) { $s = 0; for ($j = 0; $j -lt $h; $j += 3) { $c = $b.GetPixel($i, $j); $s += $c.R + $c.G + $c.B }; $cols += $s / [math]::Ceiling($h / 3) }
  $b.Dispose()
  $m = ($cols | Measure-Object -Minimum -Maximum)
  return [int]($m.Maximum - $m.Minimum)
}
function TabPt($d, $i) { $rc = $d.rects[$i]; return @([int]($d.l + $rc[0] + $rc[1] / 2), [int]($d.t + 18)) }
# The insertion mark at the right edge of $fence: the icon at $from (grabbed near its top, so
# its drag image hangs below the pointer) is held low in the right half of the cell at $to, and
# the last 6 pixel columns of the fence are read in the upper part of that row. Returns how
# many of them took the mark's colour (they differ from the fence at rest by more than the
# column 6 px in from the edge does), and saves the fence's right end as $name.png.
function OaEdgeMark($fence, $from, $to, $name) {
  $wr = FR $fence
  $sx = $wr.x + $wr.w - 6; $sy = $to[1] - 32 + 4
  ParkMouse 1300 700; Start-Sleep -Milliseconds 500
  $rest = [Px]::Grab($sx, $sy, 6, 32)
  Hold $from[0] ($from[1] - 16) ($to[0] + 30) ($to[1] + 56) 900
  $held = [Px]::Grab($sx, $sy, 6, 32)
  Crop $name ($wr.x + $wr.w - 110) ($to[1] - 40) 130 110 | Out-Null
  # Esc ends the drag where it began: nothing moves.
  [G]::Press(0x1B); Start-Sleep -Milliseconds 200; MUp; Start-Sleep -Milliseconds 900
  $d = @()
  for ($i = 0; $i -lt 6; $i++) {
    $s = 0
    for ($j = 0; $j -lt 32; $j++) { $a = $rest.GetPixel($i, $j); $b = $held.GetPixel($i, $j); $s += [math]::Abs($a.R - $b.R) + [math]::Abs($a.G - $b.G) + [math]::Abs($a.B - $b.B) }
    $d += [int]($s / 32)
  }
  $rest.Dispose(); $held.Dispose()
  $n = @($d | Where-Object { $_ - $d[0] -gt 40 }).Count
  Log "edge mark $name of '$fence': change per column, 6 px in to the edge: $($d -join ' ') -> $n columns"
  return @($n, ($d -join ' '))
}
$OaNames = @('oa 1 报告', 'oa 2 预算表', 'oa 3 会议纪要与分工', 'oa 4 图', 'oa 5 季度计划', 'oa 6 清单', 'oa 7 合同扫描件')

Case 'setup'
CloseWindows
OaClean
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
GreyWallpaper
if ($Part -match 'P' -and $PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Script (DumpScript 250 20) -Lang 'zh-CN'
Pf settings set featureTips false | Out-Null
Start-Sleep -Seconds 3

if ($Part -match 'A') {
  Case 'A right alignment: the option and the cells'
  ClearFences | Out-Null
  # 420 px: five columns, so seven icons make a full row and a row of two.
  OaFence 'OaR' '300,160,420,260' $OaNames
  ParkMouse 1300 700
  Crop 'A-1-left' 280 140 460 300 | Out-Null
  $r = Pf fence set OaR iconAlign right
  Start-Sleep -Milliseconds 1200
  $g = FenceGet 'OaR'
  Check 'A fence set iconAlign right is taken and reported by fence get' ($g.iconAlign -eq 'right') "iconAlign=$($g.iconAlign); cli: $("$r".Trim() -replace '\s+', ' ')"
  ParkMouse 1300 700
  Crop 'A-2-right' 280 140 460 300 | Out-Null
  $f00 = OaClickFocus 'OaR' 0 0; $f04 = OaClickFocus 'OaR' 0 4
  Check 'A the first icon stands at the right end of the first row, the fifth at its left end' ($f00 -like 'oa 1 *' -and $f04 -like 'oa 5 *') "right end '$f00', left end '$f04'"
  $f10 = OaClickFocus 'OaR' 1 0; $f11 = OaClickFocus 'OaR' 1 1
  Check 'A a last row of two stays at the right: the sixth at the right end, the seventh to its left' ($f10 -like 'oa 6 *' -and $f11 -like 'oa 7 *') "right end '$f10', next '$f11'"
  $f14 = OaClickFocus 'OaR' 1 4
  Check 'A the cells left of the short last row are free (a click there selects nothing)' ($f14 -eq '' -and (OaSel 'OaR') -eq '0') "focused '$f14', selected $(OaSel 'OaR')"

  Case 'A the configuration file and a restart'
  # 右对齐 is a key of its own beside an iconAlign that versions up to 0.3.3 read.
  PfQuit
  $cfg = Get-Content "$PfApp\config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json
  $v = (@($cfg.layouts | ForEach-Object { $_.fences }) | Where-Object { $_.title -eq 'OaR' } | Select-Object -First 1).view
  Check 'A the file holds iconAlign left and iconAlignRight true for the right-aligned fence' ($v.iconAlign -eq 'left' -and $v.iconAlignRight -eq $true) "iconAlign=$($v.iconAlign) iconAlignRight=$($v.iconAlignRight)"
  PfStart -Script (DumpScript 250 20) -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  $again = (FenceGet 'OaR').iconAlign; $r00 = OaClickFocus 'OaR' 0 0
  Check 'A after a restart the fence is still right-aligned' ($again -eq 'right' -and $r00 -like 'oa 1 *') "iconAlign=$again, right end '$r00'"

  Case 'A the arrow keys go where the eye goes'
  OaClickFocus 'OaR' 0 0 | Out-Null
  [G]::Press(0x25); Start-Sleep -Milliseconds 500; $kl = OaFocus 'OaR'
  [G]::Press(0x27); Start-Sleep -Milliseconds 500; $kr = OaFocus 'OaR'
  Check 'A Left from the first icon goes to the second (on its left), Right comes back' ($kl -like 'oa 2 *' -and $kr -like 'oa 1 *') "Left: '$kl', Right: '$kr'"
  [G]::Press(0x27); Start-Sleep -Milliseconds 500; $kr2 = OaFocus 'OaR'
  Check 'A Right on the first icon (nothing to its right) stays' ($kr2 -like 'oa 1 *') "'$kr2'"
  [G]::Press(0x28); Start-Sleep -Milliseconds 500; $kd = OaFocus 'OaR'
  [G]::Press(0x25); Start-Sleep -Milliseconds 500; $kdl = OaFocus 'OaR'
  [G]::Press(0x25); Start-Sleep -Milliseconds 500; $kdl2 = OaFocus 'OaR'
  [G]::Press(0x26); Start-Sleep -Milliseconds 500; $ku = OaFocus 'OaR'
  Check 'A Down the right column, Left to the last icon and no further, Up to the icon above it' ($kd -like 'oa 6 *' -and $kdl -like 'oa 7 *' -and $kdl2 -like 'oa 7 *' -and $ku -like 'oa 2 *') "Down '$kd', Left '$kdl', Left '$kdl2', Up '$ku'"
  OaClickFocus 'OaR' 0 4 | Out-Null
  [G]::Press(0x25); Start-Sleep -Milliseconds 500; $kw = OaFocus 'OaR'
  Check 'A Left at the left end of a row goes on to the next row (its right end)' ($kw -like 'oa 6 *') "'$kw'"

  Case 'A the selection rectangle'
  DeskClick 1300 700 | Out-Null
  $from = OaCell 'OaR' 1 3; $to = OaCell 'OaR' 1 0
  DragTo $from[0] ($from[1] - 20) ($to[0] + 20) ($to[1] + 30)
  Start-Sleep -Milliseconds 500
  $band = OaSel 'OaR'
  Check 'A a rectangle drawn from the free cells over the short row selects its two icons' ($band -eq '2') "selected=$band (from $($from -join ',') to $($to -join ','))"
  DeskClick 1300 700 | Out-Null

  Case 'A drag and drop: the mark and the place'
  Pf fence set OaR sort manual | Out-Null
  Start-Sleep -Milliseconds 800
  $o0 = Order 'OaR'
  $from = OaCell 'OaR' 0 0; $to = OaCell 'OaR' 1 4; $last = OaCell 'OaR' 1 1
  $wr = FR 'OaR'; $pitch = $wr.w / 5
  # After the last icon (the seventh): the mark stands at the left edge of its cell.
  $lineX = [int]($last[0] - $pitch / 2)
  ParkMouse 1300 700
  $flat = ColumnContrast ($lineX - 12) ($wr.y + 36 + 8 + 96 + 12) 24 40 'A-line-before'
  Hold $from[0] $from[1] $to[0] $to[1] 900
  $line = ColumnContrast ($lineX - 12) ($wr.y + 36 + 8 + 96 + 12) 24 40 'A-line-hold'
  Crop 'A-3-drag' 280 140 460 300 | Out-Null
  MUp; Start-Sleep -Milliseconds 1200
  $o1 = Order 'OaR'
  Check 'A while the first icon is held over the free cells, the mark stands left of the last icon' ($line -ge 40 -and $flat -lt 20) "column contrast before the drag $flat, while holding $line"
  Check 'A dropped there, the first icon is the last one: it stands left of the seventh' ($o1 -eq 'oa 2 预算表 | oa 3 会议纪要与分工 | oa 4 图 | oa 5 季度计划 | oa 6 清单 | oa 7 合同扫描件 | oa 1 报告') "before [$o0] after [$o1]"
  $f11 = OaClickFocus 'OaR' 1 1
  Check 'A the dropped icon is where the mark was' ($f11 -like 'oa 1 *') "the cell left of the seventh holds '$f11'"
  Pf fence set OaR sort name | Out-Null
  Start-Sleep -Milliseconds 1000

  Case 'A the mark at the right edge of a fence a whole number of cells wide'
  # 400 px are five cells of 80 with nothing left over: the first cell of a row ends at the
  # fence's right edge. The mark before the first icon of a row stands there, 2 px wide, and
  # both of its pixels are inside the fence (the right one was cut off).
  Pf fence set OaR sort manual | Out-Null
  Pf fence move OaR --rect '300,160,400,260' | Out-Null
  Start-Sleep -Milliseconds 1500
  $e1 = OaEdgeMark 'OaR' (OaCell 'OaR' 0 1) (OaCell 'OaR' 0 0) 'A-8-edge-row1'
  $e2 = OaEdgeMark 'OaR' (OaCell 'OaR' 0 1) (OaCell 'OaR' 1 0) 'A-8-edge-row2'
  Check 'A the mark before the first icon of a row is whole at the right edge: 2 px wide, in the first and in the second row' ($e1[0] -eq 2 -and $e2[0] -eq 2) "columns with the mark, of the last 6: first row $($e1[0]) ($($e1[1])), second row $($e2[0]) ($($e2[1]))"
  # Left-aligned, 560 px (seven cells, one full row): the mark after the last icon stands at
  # the right edge too.
  Pf fence set OaR iconAlign left | Out-Null
  Pf fence move OaR --rect '300,160,560,260' | Out-Null
  Start-Sleep -Milliseconds 1500
  $e3 = OaEdgeMark 'OaR' (OaCell 'OaR' 0 6) (OaCell 'OaR' 0 0) 'A-8-edge-left-aligned'
  Check 'A left-aligned, the mark after the last icon of a full row is whole at the right edge' ($e3[0] -eq 2) "columns with the mark, of the last 6: $($e3[0]) ($($e3[1]))"
  Pf fence set OaR iconAlign right | Out-Null
  Pf fence move OaR --rect '300,160,420,260' | Out-Null
  Pf fence set OaR sort name | Out-Null
  Start-Sleep -Milliseconds 1200

  Case 'A grouped by date'
  Pf fence set OaR groupByDate true | Out-Null
  Pf fence resize OaR --h 300 | Out-Null
  Start-Sleep -Milliseconds 1500
  ParkMouse 1300 700
  Crop 'A-4-groups' 280 140 460 340 | Out-Null
  # One group (今天): its header is 24 DIPs, the rows start under it.
  $g00 = OaClickFocus 'OaR' 0 0 24; $g11 = OaClickFocus 'OaR' 1 1 24; $g14 = OaClickFocus 'OaR' 1 4 24
  Check 'A under a date header the rows fill from the right too (the short row: icons at the right, free cells at the left)' ($g00 -like 'oa *' -and $g11 -like 'oa *' -and $g14 -eq '') "right end of row 1 '$g00', second from the right of row 2 '$g11', left end of row 2 '$g14'"
  Pf fence set OaR groupByDate false | Out-Null
  Pf fence set OaR sort name | Out-Null
  Start-Sleep -Milliseconds 1000

  Case 'A a fence that scrolls'
  Pf fence resize OaR --h 150 | Out-Null
  Start-Sleep -Milliseconds 1500
  $s00 = OaClickFocus 'OaR' 0 0
  [G]::Press(0x28); Start-Sleep -Milliseconds 900; $sd = OaFocus 'OaR'
  ParkMouse 1300 700
  Crop 'A-5-scroll' 280 140 460 200 | Out-Null
  Check 'A one row high with a scroll bar: the first icon is still at the right end, Down scrolls to the sixth under it' ($s00 -like 'oa 1 *' -and $sd -like 'oa 6 *') "right end '$s00', Down '$sd'"
  Pf fence resize OaR --h 260 | Out-Null
  Start-Sleep -Milliseconds 1000

  Case 'A a tab group'
  OaFence 'OaS' '800,160,300,200' @('oa 8 标签页里的文件')
  Pf fence set OaS iconAlign right | Out-Null
  Pf fence merge OaS --into OaR | Out-Null
  Start-Sleep -Milliseconds 1800
  ParkMouse 1300 700
  $d = DumpOf 'OaR' 4
  $t = TabPt $d ([array]::IndexOf(@($d.tabs), 'OaS'))
  [G]::Click($t[0], $t[1]); Start-Sleep -Milliseconds 1200
  $t8 = OaClickFocus 'OaR' 0 0 0 1 'OaS'
  ParkMouse 1300 700
  Crop 'A-6-tab' 280 140 460 300 | Out-Null
  $d = DumpOf 'OaR' 4
  $t = TabPt $d ([array]::IndexOf(@($d.tabs), 'OaR'))
  [G]::Click($t[0], $t[1]); Start-Sleep -Milliseconds 1200
  $t1 = OaClickFocus 'OaR' 0 0
  Check 'A in a tab group each tab keeps its alignment: one icon alone stands at the right end, the other tab starts at the right too' ($t8 -like 'oa 8 *' -and $t1 -like 'oa 1 *') "tab OaS right end '$t8', tab OaR right end '$t1' (tabs $($d.tabs -join ', '))"

  Case 'A list and details are not changed'
  Pf fence set OaR layout list | Out-Null
  Start-Sleep -Milliseconds 1200
  $wr = FR 'OaR'
  [G]::Click($wr.x + 60, $wr.y + 36 + 4 + 14); Start-Sleep -Milliseconds 700
  $l0 = OaFocus 'OaR'
  ParkMouse 1300 700
  Crop 'A-7-list' 280 140 460 300 | Out-Null
  Check 'A in the list view the rows start at the left as before (the first row is the first file)' ($l0 -like 'oa 1 *') "first row '$l0'"
  Pf fence set OaR layout icons | Out-Null
  $back = Pf fence set OaR iconAlign left
  Start-Sleep -Milliseconds 1200
  $b00 = OaClickFocus 'OaR' 0 4
  Check 'A back to 左对齐: the first icon is at the left end again' ($b00 -like 'oa 1 *' -and (FenceGet 'OaR').iconAlign -eq 'left') "left end '$b00'"
  $bad = Pf fence set OaR iconAlign bottom
  Check 'A an unknown alignment is refused and the message lists right' ($bad -match 'invalid_value' -and $bad -match 'right') ("$bad".Trim() -replace '\s+', ' ')
}

if ($Part -match 'T') {
  Case 'T stay transparent under the pointer: setup'
  ClearFences | Out-Null
  OaClean
  OaFence 'OaT' '300,160,420,260' @('oa 1 报告', 'oa 2 预算表', 'oa 3 会议纪要与分工')
  Pf fence set OaT opacity transparent | Out-Null
  Pf fence set OaT titleOnHover hover | Out-Null
  Start-Sleep -Milliseconds 1000
  $first = $true
  foreach ($look in @('light fluent', 'dark fluent', 'light liquidGlass', 'dark liquidGlass')) {
    $mode, $style = $look -split ' '
    Case "T $look"
    Pf settings set theme $mode | Out-Null
    Pf settings set themeStyle $style | Out-Null
    Pf fence set OaT transparentOnHover false | Out-Null
    Start-Sleep -Milliseconds 2000
    $wr = FR 'OaT'
    # The window and 16 px around it (the shadow and the ring are drawn outside).
    $gx = $wr.x - 16; $gy = $wr.y - 16; $gw = $wr.w + 32; $gh = $wr.h + 32
    # An empty spot of the content (under the one row of icons), and the first icon.
    $in = @([int]($wr.x + 210), [int]($wr.y + 36 + 8 + 96 + 50)); $icon = CellPt 'OaT' 0
    ParkMouse 1300 700; Start-Sleep -Milliseconds 900
    $rest = [Px]::Grab($gx, $gy, $gw, $gh); $rest.Save("$PfShots\T-$mode-$style-0-rest.png")
    [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 900
    $off = [Px]::Grab($gx, $gy, $gw, $gh); $off.Save("$PfShots\T-$mode-$style-1-pointer-off.png")
    $dOff = [Oa]::Diff($rest, $off)
    Check "T $look, switch off: the plate and the title row appear under the pointer, as before" ($dOff -gt 500) "$dOff of $($gw * $gh) px differ from the fence at rest"
    ParkMouse 1300 700
    $r = Pf fence set OaT transparentOnHover true
    Start-Sleep -Milliseconds 1200
    $on = (FenceGet 'OaT').transparentOnHover
    if ($first) { Check 'T fence set transparentOnHover true is taken and reported by fence get' ($on -eq $true) "transparentOnHover=$on; cli: $("$r".Trim() -replace '\s+', ' ')" }
    $rest2 = [Px]::Grab($gx, $gy, $gw, $gh)
    [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 900
    $hov = [Px]::Grab($gx, $gy, $gw, $gh); $hov.Save("$PfShots\T-$mode-$style-2-pointer-on.png")
    $dOn = [Oa]::Diff($rest2, $hov)
    Check "T $look, switch on: with the pointer in the fence no pixel changes (no plate, no title row)" ($dOn -eq 0 -and [Oa]::Diff($rest, $rest2) -eq 0) "$dOn px differ from the fence at rest (switch off: $dOff)"
    # The pointer on an icon: its hover fill shows, nothing else.
    [G]::MoveTo($icon[0] + 4, $icon[1]); Start-Sleep -Milliseconds 60; [G]::MoveTo($icon[0], $icon[1]); Start-Sleep -Milliseconds 250
    $hi = [Px]::Grab($gx, $gy, $gw, $gh); $hi.Save("$PfShots\T-$mode-$style-3-icon.png")
    $cell = New-Object Drawing.Rectangle ($icon[0] - 42 - $gx), ($icon[1] - 34 - $gy), 84, 100
    $top = New-Object Drawing.Rectangle 0, 0, $gw, (16 + 36)
    $far = New-Object Drawing.Rectangle ($gw - 140), ($gh - 90), 140, 90
    $dCell = [Oa]::Diff($rest2.Clone($cell, $rest2.PixelFormat), $hi.Clone($cell, $hi.PixelFormat), 9)
    $dTop = [Oa]::Diff($rest2.Clone($top, $rest2.PixelFormat), $hi.Clone($top, $hi.PixelFormat))
    $dFar = [Oa]::Diff($rest2.Clone($far, $rest2.PixelFormat), $hi.Clone($far, $hi.PixelFormat))
    Check "T $look, switch on: an icon under the pointer still gets its hover fill; the title row and the far corner do not change" ($dCell -gt 150 -and $dTop -eq 0 -and $dFar -eq 0) "$dCell px differ in the icon's cell (by more than 9), $dTop in the title row, $dFar in the far corner"
    if ($first) {
      $h = [G]::FindWindowW('PecoFence.Fence', 'OaT')
      # The title row nobody sees: it is the caption, the pointer shows the move cursor.
      $tp = @([int]($wr.x + 150), [int]($wr.y + 16))
      [G]::MoveTo($tp[0] - 6, $tp[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($tp[0], $tp[1]); Start-Sleep -Milliseconds 800
      $cur = [Oa]::Cursor(); $ht = NcHit $h $tp[0] $tp[1]
      $trow = [Px]::Grab($gx, $gy, $gw, $gh); $trow.Save("$PfShots\T-4-title-row.png")
      $dRow = [Oa]::Diff($rest2, $trow)
      Check 'T the unseen title row stays unseen under the pointer, is the caption (hit 2) and shows the move cursor' ($dRow -eq 0 -and $ht -eq 2 -and $cur -eq 'sizeall') "$dRow px differ, hit=$ht, cursor=$cur"
      # The chevron's spot: title row, a click does not roll the fence up.
      $cp = @([int]($wr.x + $wr.w - 20), [int]($wr.y + 18))
      $hc = NcHit $h $cp[0] $cp[1]
      [G]::Click($cp[0], $cp[1]); Start-Sleep -Milliseconds 1200
      $rolled = (FenceGet 'OaT').rolledUp
      Check 'T the roll-up chevron is not drawn and takes no click while the title row is unseen' ($hc -eq 2 -and -not $rolled) "hit at its spot=$hc (2 caption, 1 a button), rolled after a click=$rolled"
      Pf fence unroll OaT | Out-Null; Start-Sleep -Milliseconds 800
      # The edges resize: the right edge is HTRIGHT (11) with the left-right arrows.
      $ep = @([int]($wr.x + $wr.w - 2), [int]($wr.y + 150))
      [G]::MoveTo($ep[0] - 30, $ep[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($ep[0], $ep[1]); Start-Sleep -Milliseconds 700
      $ecur = [Oa]::Cursor(); $eh = NcHit $h $ep[0] $ep[1]
      Check 'T the right edge still resizes (hit 11, the left-right arrows)' ($eh -eq 11 -and $ecur -eq 'sizewe') "hit=$eh cursor=$ecur"
      DragTo $ep[0] $ep[1] ($ep[0] + 90) $ep[1]
      $w2 = FR 'OaT'
      Check 'T dragging the right edge makes the fence wider' ($w2.w -ge $wr.w + 40 -and $w2.x -eq $wr.x) "width $($wr.w) -> $($w2.w)"
      # A drag by the unseen title row moves the fence.
      DragTo $tp[0] $tp[1] ($tp[0] + 120) ($tp[1] + 70)
      $w3 = FR 'OaT'
      Check 'T dragging the unseen title row moves the fence' ([math]::Abs($w3.x - $w2.x - 120) -le 40 -and [math]::Abs($w3.y - $w2.y - 70) -le 40 -and $w3.w -eq $w2.w) "from $($w2.x),$($w2.y) to $($w3.x),$($w3.y)"
      Pf fence move OaT --rect '300,160,420,260' | Out-Null; Start-Sleep -Milliseconds 1000
      # Rolled up it shows its plate and title, so it can be found and opened.
      ParkMouse 1300 700; Start-Sleep -Milliseconds 600
      $stripRest = [Px]::Grab(300, 160, 420, 36)
      Pf fence roll OaT | Out-Null; Start-Sleep -Milliseconds 1500
      $stripRolled = [Px]::Grab(300, 160, 420, 36); $stripRolled.Save("$PfShots\T-5-rolled.png")
      $dRoll = [Oa]::Diff($stripRest, $stripRolled)
      Pf fence unroll OaT | Out-Null; Start-Sleep -Milliseconds 1500
      Check 'T rolled up, the fence shows its plate and title as before' ($dRoll -gt 2000) "$dRoll of $(420 * 36) px of the title row differ from the unseen row"
      # 显示标题栏 始终显示: the title is there at rest and under the pointer; still no plate.
      Pf fence set OaT titleOnHover always | Out-Null; Start-Sleep -Milliseconds 1200
      ParkMouse 1300 700; Start-Sleep -Milliseconds 700
      $ra = [Px]::Grab($gx, $gy, $gw, $gh); $ra.Save("$PfShots\T-6-title-always-rest.png")
      [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 900
      $ha = [Px]::Grab($gx, $gy, $gw, $gh)
      $dTitle = [Oa]::Diff($rest2, $ra); $dA = [Oa]::Diff($ra, $ha)
      Check 'T with the title row always shown, the title stays and the plate still does not come' ($dTitle -gt 30 -and $dA -eq 0) "title at rest: $dTitle px differ from the fence without it; pointer in: $dA px differ"
      Pf fence set OaT titleOnHover hover | Out-Null; Start-Sleep -Milliseconds 1000
      # Another opacity: the switch does nothing, the fence looks as it always did.
      Pf fence set OaT opacity default | Out-Null; Start-Sleep -Milliseconds 1200
      ParkMouse 1300 700; Start-Sleep -Milliseconds 700
      $rd = [Px]::Grab($gx, $gy, $gw, $gh)
      [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 900
      $hd = [Px]::Grab($gx, $gy, $gw, $gh)
      $dDef = [Oa]::Diff($rd, $hd)
      Check 'T at 默认 opacity the switch does nothing: the title row still comes under the pointer' ($dDef -gt 100) "$dDef px differ"
      Pf fence set OaT opacity transparent | Out-Null; Start-Sleep -Milliseconds 1000
      # A Peek shows the glass of every fully transparent fence, this one too: the pointer
      # over it then brings the title row and the roll-up chevron, as on any other fence.
      ParkMouse 1300 700
      Pf peek start | Out-Null
      for ($i = 0; $i -lt 25 -and @([W]::Tops('PecoFence.PeekDimmer')).Count -eq 0; $i++) { Start-Sleep -Milliseconds 200 }
      Start-Sleep -Milliseconds 1200
      $pk0 = [Px]::Grab($wr.x, $wr.y, $wr.w, 36)
      Crop 'T-7-peek-rest' $gx $gy $gw $gh | Out-Null
      [G]::MoveTo($in[0] + 6, $in[1]); Start-Sleep -Milliseconds 100; [G]::MoveTo($in[0], $in[1]); Start-Sleep -Milliseconds 1000
      $pk1 = [Px]::Grab($wr.x, $wr.y, $wr.w, 36)
      Crop 'T-7-peek-pointer' $gx $gy $gw $gh | Out-Null
      $dPk = [Oa]::Diff($pk0, $pk1); $hcPk = NcHit $h $cp[0] $cp[1]
      $peekOn = @([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0
      Pf peek end | Out-Null; Start-Sleep -Milliseconds 1500
      $after = [Px]::Grab($gx, $gy, $gw, $gh)
      $dAfter = [Oa]::Diff($rest2, $after)
      Check 'T in a Peek the pointer over the fence brings its title row, and the chevron takes clicks' ($peekOn -and $dPk -gt 300 -and $hcPk -eq 1) "Peek on=$peekOn; $dPk of $($wr.w * 36) px of the title row differ from the row without the pointer; hit at the chevron=$hcPk (1 a button, 2 caption)"
      Check 'T after the Peek the pointer in the fence shows nothing again' ($dAfter -eq 0) "$dAfter px differ from the fence at rest"
      $first = $false
    }
  }
  ParkMouse 1300 700
  # The config schema with both options, for site/schema/config.json.
  $schema = & "$PfApp\pecofence-cli.exe" describe --schema Config --compact | Out-String
  [IO.File]::WriteAllText("$PfShots\config-schema.json", $schema.Trim(), (New-Object Text.UTF8Encoding($false)))
  Check 'T the config schema has transparentOnHover and the right alignment' ($schema -match '"transparentOnHover"' -and $schema -match '"iconAlignRight"') "length=$($schema.Length)"
}

if ($Part -match 'P') {
  Case 'P the two options in the fence options'
  ClearFences | Out-Null
  OaClean
  OaFence 'OaP' '300,160,420,260' @('oa 1 报告', 'oa 2 预算表')
  Check 'P a debug build' $PfDebugBuild
  Check 'P the settings page is reached' (PfSettings)
  Js "document.querySelector('a[data-page=fences]').click()" | Out-Null
  Start-Sleep -Milliseconds 800
  $id = (FenceGet 'OaP').id | ConvertTo-Json
  $set = "(id, v) => { const e = document.getElementById(id); e.value = v; e.dispatchEvent(new Event('change', { bubbles: true })); return e.value; }"
  Js "($set)('fenceSel', $id)" | Out-Null
  Start-Sleep -Milliseconds 800
  $row = "document.getElementById('fenceTransparentOnHover')"
  $h0 = Js "$row.hidden"
  Js "($set)('fenceOpacity', 'transparent')" | Out-Null
  Start-Sleep -Milliseconds 1500
  $h1 = Js "$row.hidden"
  Check 'P 鼠标移入时保持透明 is hidden at 默认 opacity and shows once 不透明度 is 全透明' ($h0 -eq $true -and $h1 -eq $false -and (FenceGet 'OaP').opacity -eq 'transparent') "hidden before=$h0, after=$h1, opacity=$((FenceGet 'OaP').opacity)"
  $r = PageRect '#fenceTransparentOnHover'
  if ($r) { Crop 'P-1-row' ($r.x - 8) ($r.y - 76) ($r.w + 16) ($r.h + 150) | Out-Null }
  PageClick '[data-fence=transparentOnHover]'
  Start-Sleep -Milliseconds 1200
  $on = Js "document.querySelector('[data-fence=transparentOnHover]').classList.contains('on')"
  Check 'P its switch turns the option on for the fence' ($on -eq $true -and (FenceGet 'OaP').transparentOnHover -eq $true) "switch on=$on, fence get transparentOnHover=$((FenceGet 'OaP').transparentOnHover)"
  $opts = Js "[...document.getElementById('fenceIconAlign').options].map(o => o.value + ' ' + o.textContent).join(', ')"
  Js "($set)('fenceIconAlign', 'right')" | Out-Null
  Start-Sleep -Milliseconds 1200
  Check 'P 图标对齐 lists 右对齐 and it reaches the fence' ($opts -eq 'left 左对齐, center 居中, right 右对齐' -and (FenceGet 'OaP').iconAlign -eq 'right') "options: $opts; fence get iconAlign=$((FenceGet 'OaP').iconAlign)"
  $r = PageRect '#fenceIconAlign'
  if ($r) { Crop 'P-2-align' ($r.x - 560) ($r.y - 20) 760 ($r.h + 40) | Out-Null }
  Js "($set)('fenceOpacity', 'default')" | Out-Null
  Start-Sleep -Milliseconds 1500
  $h2 = Js "$row.hidden"
  Check 'P back at 默认 opacity the switch is hidden again and the choice is kept' ($h2 -eq $true -and (FenceGet 'OaP').transparentOnHover -eq $true) "hidden=$h2, transparentOnHover=$((FenceGet 'OaP').transparentOnHover)"
}

if ($Part -match 'S') {
  Case 'S right alignment at 150 %'
  ClearFences | Out-Null
  OaClean
  $n0 = @(PfLogAll | Select-String -Pattern 'display changed monitors=').Count
  Set-Content C:\pf\resize-want.txt '2400x1350@150' -Encoding ASCII
  for ($w = 0; $w -lt 150; $w++) {
    Start-Sleep -Seconds 1
    if (-not (Test-Path C:\pf\resize-want.txt) -and @(PfLogAll | Select-String -Pattern 'display changed monitors=').Count -gt $n0) { break }
  }
  Start-Sleep -Seconds 6
  # 630 px at 150 % are 420 DIPs: five columns of 126 px.
  OaFence 'OaR' '450,240,630,390' $OaNames
  Pf fence set OaR iconAlign right | Out-Null
  Start-Sleep -Milliseconds 1500
  ParkMouse 1900 1100
  $wr = FR 'OaR'
  Crop 'S-150' ($wr.x - 20) ($wr.y - 20) ($wr.w + 40) ($wr.h + 40) | Out-Null
  $f00 = OaClickFocus 'OaR' 0 0 0 1.5; $f04 = OaClickFocus 'OaR' 0 4 0 1.5; $f11 = OaClickFocus 'OaR' 1 1 0 1.5; $f14 = OaClickFocus 'OaR' 1 4 0 1.5
  Check 'S at 150 % the rows fill from the right: first icon at the right end, fifth at the left end, the seventh second from the right, free cells at the left' ($f00 -like 'oa 1 *' -and $f04 -like 'oa 5 *' -and $f11 -like 'oa 7 *' -and $f14 -eq '') "'$f00', '$f04', '$f11', '$f14' (window $($wr.x),$($wr.y) $($wr.w)x$($wr.h))"
  OaClickFocus 'OaR' 0 0 0 1.5 | Out-Null
  [G]::Press(0x25); Start-Sleep -Milliseconds 500; $kl = OaFocus 'OaR'
  Check 'S at 150 % Left goes to the icon on the left' ($kl -like 'oa 2 *') "'$kl'"
}

Case 'cleanup'
PfQuit
OaClean
[Anim]::Set($AnimWas)
