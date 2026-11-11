# Fence polish round 2026-11-12, package metrics (visual review V2, V4, V5, code review S1).
# Parts (run.sh --var Part=...), default all:
#   N  an icon's name takes a wider box when that shows more of it (README.md and six Chinese
#      characters on one line, twelve characters whole), long names keep their room, every name
#      stays inside its plate and apart from its neighbours; a fence that scrolls keeps its
#      names and plates out of the scrollbar's strip
#   L  one left edge: title, note, script and clock text, row names, the 名称 caption and the
#      date group caption start where the title does
#   S  list and details rows of a fence that scrolls end before the scrollbar's strip
#   T  a tab dragged past its neighbour leaves no hover fade on that neighbour
# Runs at 1600x900, 100 %, dark mode over a plain grey wallpaper (white text, pixel reads).
if (-not $Part) { $Part = 'NLST' }
trap { PfTrap $_; continue }

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
public static class Mx {
  static int[] Pixels(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var a = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, a, 0, a.Length); b.UnlockBits(d); return a;
  }
  static int Lum(int c) { return (((c >> 16) & 255) * 299 + ((c >> 8) & 255) * 587 + (c & 255) * 114) / 1000; }
  static int Sat(int c) { int r = (c >> 16) & 255, g = (c >> 8) & 255, b = c & 255; return Math.Max(r, Math.Max(g, b)) - Math.Min(r, Math.Min(g, b)); }
  // Box [left, top, right, bottom] of the colourless pixels at least thr brighter than the
  // crop's median (white text on a darker plate); -1s when there are none.
  public static int[] InkBox(Bitmap b, int thr) {
    var px = Pixels(b); int w = b.Width, h = b.Height;
    var s = new int[px.Length]; for (int i = 0; i < px.Length; i++) s[i] = Lum(px[i]);
    var sorted = (int[])s.Clone(); Array.Sort(sorted); int med = sorted[sorted.Length / 2];
    int l = -1, t = -1, r = -1, bt = -1;
    for (int y = 0; y < h; y++) for (int x = 0; x < w; x++) {
      if (Sat(px[y * w + x]) > 40 || s[y * w + x] - med < thr) continue;
      if (l < 0 || x < l) l = x; if (x > r) r = x; if (t < 0) t = y; bt = y;
    }
    return new[] { l, t, r, bt };
  }
  // Box of the pixels whose brightness differs by at least thr between two same-size crops.
  public static int[] DiffBox(Bitmap a, Bitmap b, int thr) {
    var pa = Pixels(a); var pb = Pixels(b); int w = a.Width, h = a.Height;
    int l = -1, t = -1, r = -1, bt = -1;
    for (int y = 0; y < h; y++) for (int x = 0; x < w; x++) {
      if (Math.Abs(Lum(pa[y * w + x]) - Lum(pb[y * w + x])) < thr) continue;
      if (l < 0 || x < l) l = x; if (x > r) r = x; if (t < 0) t = y; bt = y;
    }
    return new[] { l, t, r, bt };
  }
  // Mean brightness of a crop.
  public static double Mean(Bitmap b) { var px = Pixels(b); double s = 0; foreach (var c in px) s += Lum(c); return s / px.Length; }
}
'@

$MxMade = New-Object Collections.ArrayList
function MxFiles([string[]]$names) {
  foreach ($n in $names) {
    $p = Join-Path $PfDesk $n
    if ($n -notmatch '\.') { New-Item -ItemType Directory -Force $p | Out-Null } else { Set-Content -LiteralPath $p -Value "polish $n" -Encoding UTF8 }
    [void]$MxMade.Add($p)
  }
}
function MxPaths([string[]]$names) { return @($names | ForEach-Object { Join-Path $PfDesk $_ }) }
# The kit keeps pf's desktop between jobs: every polish file goes at the start and at the end.
function MxClean() {
  foreach ($p in @($MxMade)) { if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue } }
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'mx *' -or $_.Name -like 'mx-*' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
function MxStart([string[]]$Script = @('sleep 3600000')) {
  CloseWindows
  GreyWallpaper
  [Anim]::Set($true)
  PfStart -Script $Script -Lang 'zh-CN'
  Pf settings set featureTips false | Out-Null
  Pf settings set theme dark | Out-Null
  Start-Sleep -Seconds 2
  ClearFences | Out-Null
}
function MxPark() { ParkMouse 1450 840; Start-Sleep -Milliseconds 500 }
# Where the cell of an item is on screen in an icons fence that is not scrolled: cells of
# (icon + 32) x (icon + 48) under the 36 px title row and 8 px of padding, two label lines.
function MxCell($fence, $name, $icon = 48) {
  $f = FenceGet $fence; $wr = $f.windowRect
  $order = @(@(PfJson item list --fence $fence) | ForEach-Object { $_.name })
  $i = [array]::IndexOf($order, $name)
  if ($i -lt 0) { Log "MxCell: '$name' not in '$fence' ($($order -join ', '))"; return $null }
  $cw = $icon + 32; $ch = $icon + 48
  $cols = [math]::Max(1, [math]::Floor($wr.w / $cw)); $pitch = $wr.w / $cols
  return @{
    i = $i; col = $i % $cols; cols = $cols; row = [math]::Floor($i / $cols)
    x = [int][math]::Round($wr.x + ($i % $cols) * $pitch + ($pitch - $cw) / 2)
    y = [int]($wr.y + 44 + [math]::Floor($i / $cols) * $ch)
    w = $cw; h = $ch; labelTop = [int]($wr.y + 44 + [math]::Floor($i / $cols) * $ch + 8 + $icon + 4)
    left = $wr.x; right = $wr.x + $wr.w
    shown = ($wr.y + 44 + [math]::Floor($i / $cols) * $ch + $ch) -le ($wr.y + $wr.h)
  }
}
# The ink of an item's name: box in cell coordinates (left, right), its height and line count.
function MxLabel($fence, $name, $icon = 48, $lines = 2) {
  $c = MxCell $fence $name $icon
  if (-not $c) { return $null }
  $b = [Px]::Grab($c.x, $c.labelTop - 1, $c.w, 16 * $lines + 2)
  $k = [Mx]::InkBox($b, 60); $b.Dispose()
  $h = if ($k[0] -lt 0) { 0 } else { $k[3] - $k[1] + 1 }
  return @{ l = $k[0]; r = $k[2]; h = $h; lines = [int][math]::Ceiling($h / 16.0); cell = $c }
}
function MxLabelText($l) { if (-not $l) { return 'no label' }; return "ink x $($l.l)..$($l.r) of the $($l.cell.w) px cell, $($l.h) px high, $($l.lines) line(s)" }
# First bright ink column of a screen region, as px from $x0; -1 when none.
function MxInkLeft($x0, $y, $w, $h, $thr = 50) {
  $b = [Px]::Grab($x0, $y, $w, $h); $k = [Mx]::InkBox($b, $thr); $b.Dispose(); return $k[0]
}
# Last bright ink column of a screen region, as px from $x0; -1 when none.
function MxInkRight($x0, $y, $w, $h, $thr = 50) {
  $b = [Px]::Grab($x0, $y, $w, $h); $k = [Mx]::InkBox($b, $thr); $b.Dispose(); return $k[2]
}

MxClean

if ($Part -match 'N') {
  Case 'names: setup'
  # Thirteen characters: cut in either box, so they keep the usual one.
  $long = '很长很长的文件名甲乙丙丁戊.txt', '很长很长的文件名子丑寅卯辰.txt', '很长很长的文件名金木水火土.txt', '很长很长的文件名东南西北中.txt', '很长很长的文件名春夏秋冬年.txt'
  # Two lines in the usual box and one in the wider one; twelve characters, whole only in the
  # wider one; eight characters, two whole lines either way.
  $short = 'README.md', '项目计划书稿.txt', '十二个字的名称正好两行整.txt', '八个字的名称两行.txt'
  # 「i 滚动五字名」 would fit the wider box; in a fence that scrolls it keeps the usual one.
  $scroll = 'i 滚动五字名.txt', 'mx 滚动二号文件名称长.txt', 'mx 滚动三号文件名称长.txt', 'mx 滚动四号文件名称长.txt', 'mx 滚动五号文件名称长.txt', 'mx 滚动六号文件名称长.txt', 'mx 滚动七号文件名称长.txt'
  $loose = '桌面上的计划.txt'
  MxFiles ($long + $short + $scroll + $loose)
  MxStart
  Pf fence set inbox desktopLayout true | Out-Null
  # Three columns of 48 px icons, three whole rows: nothing scrolls.
  Pf fence create --title 名称 --rect '200,120,240,340' | Out-Null
  Pf item move (MxPaths ($long + $short)) --to 名称 | Out-Null
  Pf fence set 名称 sort name | Out-Null
  # Three columns, two rows shown of three: this one scrolls.
  Pf fence create --title 滚动 --rect '520,120,240,240' | Out-Null
  Pf item move (MxPaths $scroll) --to 滚动 | Out-Null
  Pf fence set 滚动 sort name | Out-Null
  Start-Sleep -Seconds 2
  MxPark
  Shot 'N-names'

  Case 'names: a wider box when it shows more'
  foreach ($n in 'README.md', '项目计划书稿') {
    $l = MxLabel 名称 $n
    Check "「$n」 stays on one line" ($l -and $l.lines -eq 1) (MxLabelText $l)
  }
  $l = MxLabel 名称 '十二个字的名称正好两行整'
  Check 'a twelve-character name shows whole on its two lines' ($l -and $l.lines -eq 2 -and ($l.r - $l.l + 1) -ge 68) (MxLabelText $l)
  $l = MxLabel 名称 '八个字的名称两行'
  Check 'a name with two whole lines either way keeps the usual box' ($l -and $l.lines -eq 2 -and $l.l -ge 8 -and $l.r -le 71) (MxLabelText $l)
  $cut = @(@(PfJson item list --fence 名称) | ForEach-Object { $_.name } | Where-Object { $_ -like '很长很长*' } | ForEach-Object { MxLabel 名称 $_ })
  $wide = @($cut | Where-Object { $_.l -lt 8 -or $_.r -gt 71 })
  Check 'names that are cut either way keep 8 px beside them' ($cut.Count -eq 5 -and $wide.Count -eq 0) (($cut | ForEach-Object { "$($_.l)..$($_.r)" }) -join ' ')
  $f = FenceGet 名称
  Check 'the fence keeps its size and its three columns' ($f.windowRect.w -eq 240 -and $f.windowRect.h -eq 340 -and $f.fit.columns -eq 3) "window $($f.windowRect.w)x$($f.windowRect.h) columns=$($f.fit.columns) rows=$($f.fit.rows) overflow=$($f.fit.overflow)"

  Case 'names: inside the plate, apart from the neighbours'
  $order = @(@(PfJson item list --fence 名称) | ForEach-Object { $_.name })
  $labels = @($order | ForEach-Object { MxLabel 名称 $_ })
  $out = @($labels | Where-Object { $_.l -lt 2 -or $_.r -gt 77 })
  Check 'every name stays inside the 76 px plate of its cell' ($labels.Count -eq 9 -and $out.Count -eq 0) (($labels | ForEach-Object { "$($_.l)..$($_.r)" }) -join ' ')
  $gaps = @()
  for ($i = 0; $i + 1 -lt $labels.Count; $i++) {
    $a = $labels[$i]; $b = $labels[$i + 1]
    if ($a.cell.row -ne $b.cell.row) { continue }
    $gaps += ($b.cell.x + $b.l) - ($a.cell.x + $a.r) - 1
  }
  $min = ($gaps | Measure-Object -Minimum).Minimum
  Check 'neighbouring names keep at least 4 px between them' ($gaps.Count -ge 4 -and $min -ge 4) "gaps between names in a row: $($gaps -join ', ') px"
  # The selected icon shows its whole name (unfolded): it stays inside the plate too.
  $c = MxCell 名称 $order[0]
  [G]::Click($c.x + 40, $c.y + 30); Start-Sleep -Milliseconds 700; MxPark
  $b = [Px]::Grab($c.x, $c.labelTop - 1, 80, 16 * 6 + 2); $k = [Mx]::InkBox($b, 60); $b.Dispose()
  Shot 'N-unfolded'
  Check 'the selected icon unfolds its whole name inside the plate' ($k[0] -ge 2 -and $k[2] -le 77 -and ($k[3] - $k[1]) -gt 34) "unfolded ink x $($k[0])..$($k[2]), $($k[3] - $k[1] + 1) px high"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300

  Case 'names: loose icons on the desktop'
  $p = ItemPt inbox '桌面上的计划'
  $lb = if ($p) { $b = [Px]::Grab($p[0] - 40, $p[1] + 27, 80, 34); $k = [Mx]::InkBox($b, 60); $b.Dispose(); $k } else { @(-1, -1, -1, -1) }
  $lh = if ($lb[0] -lt 0) { 0 } else { $lb[3] - $lb[1] + 1 }
  Check 'a six-character name of a loose icon stays on one line' ($lh -gt 0 -and $lh -le 16) "ink x $($lb[0])..$($lb[2]), $lh px high, icon at $($p -join ',')"

  Case 'names: every icon size'
  foreach ($s in 24, 32, 64, 96) {
    Pf fence set 名称 iconSize $s | Out-Null; Start-Sleep -Milliseconds 1200; MxPark
    Shot "N-size-$s"
    $names = @(@(PfJson item list --fence 名称) | ForEach-Object { $_.name })
    $ls = @($names | ForEach-Object { MxLabel 名称 $_ $s } | Where-Object { $_.cell.shown })
    $bad = @($ls | Where-Object { $_.l -lt 2 -or $_.r -gt ($s + 32 - 3) })
    Check "icon size ${s}: every name shown stays inside its plate" ($ls.Count -ge 2 -and $bad.Count -eq 0) (($ls | ForEach-Object { "$($_.l)..$($_.r)" }) -join ' ')
    Log "icon size ${s}: README.md $(MxLabelText (MxLabel 名称 'README.md' $s)); 项目计划书稿 $(MxLabelText (MxLabel 名称 '项目计划书稿' $s))"
  }
  Pf fence set 名称 iconSize 48 | Out-Null; Start-Sleep -Milliseconds 1000
  foreach ($n in 1, 3) {
    Pf fence set 名称 labelLines $n | Out-Null; Start-Sleep -Milliseconds 1000; MxPark
    Shot "N-lines-$n"
  }
  Pf fence set 名称 labelLines 2 | Out-Null; Start-Sleep -Milliseconds 800

  Case 'names: a fence that scrolls keeps the scrollbar strip clear'
  $f = FenceGet 滚动
  Check 'the fence scrolls' ($f.fit.overflow -eq $true) "rows=$($f.fit.rows) overflow=$($f.fit.overflow) window $($f.windowRect.w)x$($f.windowRect.h)"
  $l = MxLabel 滚动 'i 滚动五字名'
  Check 'a name that would fit the wider box keeps the usual one where the fence scrolls' ($l -and $l.lines -eq 2 -and $l.l -ge 8 -and $l.r -le 71) (MxLabelText $l)
  $order = @(@(PfJson item list --fence 滚动) | ForEach-Object { $_.name })
  $last = $order[5]
  $c = MxCell 滚动 $last
  $l = MxLabel 滚动 $last
  Check 'the last column''s name ends before the thumb (9 px from the edge)' ($l -and ($c.x + $l.r) -le ($c.right - 9)) "name ink ends $($c.right - ($c.x + $l.r)) px before the fence edge; $(MxLabelText $l)"
  # The plate of the selected icon in the last column: what changes on screen when it is selected.
  $r0 = [Px]::Grab($c.x - 4, $c.y, $c.right - $c.x + 4, 96)
  [G]::Click($c.x + 40, $c.y + 30); Start-Sleep -Milliseconds 700; MxPark
  $r1 = [Px]::Grab($c.x - 4, $c.y, $c.right - $c.x + 4, 96)
  $d = [Mx]::DiffBox($r0, $r1, 6); $r0.Dispose(); $r1.Dispose()
  $plateRight = $c.x - 4 + $d[2]
  Shot 'N-scroll-selected'
  Check 'the selected icon''s plate ends before the strip (8 px from the edge)' ($d[2] -ge 0 -and ($c.right - 1 - $plateRight) -ge 8) "plate's last px is $($c.right - 1 - $plateRight) px before the fence's last px (diff box $($d -join ','))"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  PfQuit
}

if ($Part -match 'L') {
  Case 'left edges: setup'
  MxClean
  $rows = 'mx 资料夹甲', 'mx 资料夹乙', 'mx 资料夹丙'
  $grp = 'mx 今天的文件一.txt', 'mx 今天的文件二.txt'
  $tabs = 'mx 标签文件.txt'
  MxFiles ($rows + $grp + $tabs)
  Set-Content C:\pf\run\mx-lines.ps1 -Value 'Write-Output "第一行输出"; Write-Output "second line"' -Encoding UTF8
  MxStart
  Pf fence create --title 列表 --rect '60,100,260,160' | Out-Null
  Pf item move (MxPaths $rows[0..1]) --to 列表 | Out-Null
  Pf fence set 列表 layout list | Out-Null
  Pf fence create --title 明细 --rect '60,300,520,150' | Out-Null
  Pf item move (MxPaths $rows[2]) --to 明细 | Out-Null
  Pf fence set 明细 layout details | Out-Null
  Pf fence create --title 分组 --rect '620,100,260,300' | Out-Null
  Pf item move (MxPaths $grp) --to 分组 | Out-Null
  Pf fence set 分组 sort date | Out-Null
  Pf fence set 分组 groupByDate true | Out-Null
  Pf fence create --tool note --title 便签 --text "便签第一行文字`n第二行" --rect '920,100,300,180' | Out-Null
  Pf fence create --tool clock --title 时钟 --rect '920,320,240,140' | Out-Null
  Pf fence create --script C:\pf\run\mx-lines.ps1 --title 脚本 --rect '1240,100,300,160' | Out-Null
  # A long first tab caption, and a two-character one (centred in the 48 px smallest pill).
  Pf fence create --title 工作资料夹 --rect '60,500,300,160' | Out-Null
  Pf fence create --title 学习 --rect '400,700,200,120' | Out-Null
  Pf item move (MxPaths $tabs) --to 学习 | Out-Null
  Pf fence merge 学习 --into 工作资料夹 | Out-Null
  Pf fence create --title 短 --rect '400,500,300,160' | Out-Null
  Pf fence create --title 第二个标签 --rect '760,700,200,120' | Out-Null
  Pf fence merge 第二个标签 --into 短 | Out-Null
  Start-Sleep -Seconds 4
  MxPark
  Shot 'L-edges'

  # Each fence's title ink is the reference (16 px from the plate's edge, less the glyph's own
  # side bearing); a text "lines up" when its ink starts within 1.5 px of it.
  function MxTitle($t) { $w = (FenceGet $t).windowRect; return @{ x = $w.x; y = $w.y; ink = (MxInkLeft $w.x ($w.y + 8) 140 20) } }
  function MxEdge($what, $t, $dy, $h, $want = 0, $x0 = 0) {
    $ti = MxTitle $t
    $ink = MxInkLeft ($ti.x + $x0) ($ti.y + $dy) 200 $h
    $at = if ($ink -lt 0) { -99 } else { $ink + $x0 }
    Check "$what starts at the title's left edge$(if ($want) { " plus $want px" })" ([math]::Abs($at - $ti.ink - $want) -le 1.5) "ink at $at px from the plate's edge, title ink at $($ti.ink) px"
  }
  Case 'left edges: one line down the fence'
  $ti = MxTitle 列表
  Check 'the title ink starts 16 px from the plate''s edge' ($ti.ink -ge 15 -and $ti.ink -le 17.5) "title ink at $($ti.ink) px"
  MxEdge 'a note''s text' 便签 38 22
  MxEdge 'a script''s first row' 脚本 38 26
  MxEdge 'the clock''s text' 时钟 38 100
  MxEdge 'the 名称 caption of the details view' 明细 40 22
  MxEdge 'the date group caption (今天)' 分组 38 22
  # Row names follow the 20 px icon and its 8 px gap: 28 px after the title's edge. Folders
  # (yellow icons) keep the icon out of the white-ink read.
  MxEdge 'a list row''s name' 列表 40 24 28
  MxEdge 'a details row''s name' 明细 68 24 28
  $long = MxTitle 工作资料夹
  Check 'a first tab caption that fills its pill starts at the title''s left edge' ($long.ink -ge 15 -and $long.ink -le 17.5) "first caption ink at $($long.ink) px"
  $shortTab = MxTitle 短
  Log "a two-character first tab caption (centred in the 48 px smallest pill): ink at $($shortTab.ink) px"
  PfQuit
}

if ($Part -match 'S') {
  Case 'scroll strip: setup'
  MxClean
  $many = 1..8 | ForEach-Object { "mx 报告第 $_ 份.txt" }
  $few = 'mx 少量甲.txt', 'mx 少量乙.txt'
  $lst = 1..8 | ForEach-Object { "mx 列表第 $_ 项.txt" }
  MxFiles ($many + $few + $lst)
  MxStart
  Pf fence create --title 明细滚动 --rect '100,120,520,150' | Out-Null
  Pf item move (MxPaths $many) --to 明细滚动 | Out-Null
  Pf fence set 明细滚动 layout details | Out-Null
  Pf fence create --title 明细不滚 --rect '100,320,520,150' | Out-Null
  Pf item move (MxPaths $few) --to 明细不滚 | Out-Null
  Pf fence set 明细不滚 layout details | Out-Null
  Pf fence create --title 列表滚动 --rect '700,120,300,150' | Out-Null
  Pf item move (MxPaths $lst) --to 列表滚动 | Out-Null
  Pf fence set 列表滚动 layout list | Out-Null
  Start-Sleep -Seconds 3
  MxPark
  Shot 'S-rows'

  # The size column's values (right-aligned) against the resting thumb, 7 px from the edge.
  function MxSizeGap($t) {
    $w = (FenceGet $t).windowRect
    $r = MxInkRight ($w.x + $w.w - 80) ($w.y + 36 + 30) 72 80
    if ($r -lt 0) { return -99 }
    return ($w.w - 7) - ($w.w - 80 + $r) - 1
  }
  # The plate of a row: what changes on screen when the first row is selected.
  function MxRowPlate($t, $dy) {
    $w = (FenceGet $t).windowRect
    $b0 = [Px]::Grab($w.x, $w.y + $dy, $w.w, 26)
    [G]::Click($w.x + 120, $w.y + $dy + 13); Start-Sleep -Milliseconds 700; MxPark
    $b1 = [Px]::Grab($w.x, $w.y + $dy, $w.w, 26)
    $d = [Mx]::DiffBox($b0, $b1, 6); $b0.Dispose(); $b1.Dispose()
    [G]::Press(0x1B); Start-Sleep -Milliseconds 300
    return @{ left = $d[0]; fromRight = $w.w - 1 - $d[2] }
  }
  Case 'scroll strip: details view'
  $f = FenceGet 明细滚动
  Check 'the details fence scrolls' ($f.fit.overflow -eq $true) "overflow=$($f.fit.overflow)"
  $g = MxSizeGap 明细滚动
  Check 'the size column ends at least 5 px before the thumb' ($g -ge 5) "gap between the values and the thumb: $g px"
  $p = MxRowPlate 明细滚动 65
  Shot 'S-details-selected'
  Check 'a selected row''s plate ends before the strip (8 px from the edge)' ($p.fromRight -ge 8) "plate's last px is $($p.fromRight) px before the fence's last px, first px at $($p.left)"
  $g2 = MxSizeGap 明细不滚
  $p2 = MxRowPlate 明细不滚 65
  Check 'a details fence that does not scroll keeps its columns and plates to the edge' ($g2 -le 3 -and $p2.fromRight -le 5) "values end $g2 px before where a thumb would rest; plate's last px $($p2.fromRight) px before the fence's last px"

  Case 'scroll strip: list view'
  $p3 = MxRowPlate 列表滚动 37
  Shot 'S-list-selected'
  Check 'a selected list row''s plate ends before the strip' ($p3.fromRight -ge 8) "plate's last px is $($p3.fromRight) px before the fence's last px"
  PfQuit
}

if ($Part -match 'T') {
  # The hover fade of a tab pill belongs to the tab, not to its place in the strip: after a
  # reorder the fill shows on the pill under the pointer and not on the one that moved away.
  Case 'tab fades: setup'
  MxClean
  MxFiles @('mx 标签页文件.txt')
  # Dumps for 25 s, then the tab menu's reorder (first tab to the second place), then dumps again.
  $script = @('sleep 4000') + (1..100 | ForEach-Object { "dump d$_"; 'sleep 250' }) + @('reorder 甲标签页 1') + (101..400 | ForEach-Object { "dump d$_"; 'sleep 250' }) + @('sleep 3600000')
  MxStart $script
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  Pf fence create --title 甲标签页 --rect '300,200,460,200' | Out-Null
  Pf fence create --title 乙标签页 --rect '300,500,300,120' | Out-Null
  Pf fence create --title 丙标签页 --rect '700,500,300,120' | Out-Null
  Pf fence merge 乙标签页 --into 甲标签页 | Out-Null
  Pf fence merge 丙标签页 --into 甲标签页 | Out-Null
  Start-Sleep -Seconds 1
  $d = DumpOf 甲标签页
  Check 'three tabs, the last merged one active' ($d -and $d.ntabs -eq 3 -and $d.tabs[0] -eq '甲标签页') "tabs: $($d.tabs -join ', ') rects: $(($d.rects | ForEach-Object { $_ -join '+' }) -join ' ')"
  # Mean brightness of the inside of the pill at strip place $i (the caption is part of it).
  function MxPill($d, $i) {
    $x = [int]($d.l + $d.rects[$i][0] + 3); $w = [int]($d.rects[$i][1] - 6)
    $b = [Px]::Grab($x, $d.t + 6, $w, 24); $m = [Mx]::Mean($b); $b.Dispose(); return [math]::Round($m, 2)
  }
  MxPark
  $rest0 = MxPill $d 0
  [G]::MoveTo([int]($d.l + $d.rects[0][0] + $d.rects[0][1] / 2), $d.t + 18); Start-Sleep -Milliseconds 600
  $hov0 = MxPill $d 0
  Shot 'T-hover-first'
  Check 'a resting tab under the pointer shows the hover fill' (($hov0 - $rest0) -ge 1.0) "pill brightness $rest0 at rest, $hov0 under the pointer"

  Case 'tab fades: after a reorder'
  # The pointer stays where it is; the script moves the first tab to the second place.
  $moved = $null
  $until = (Get-Date).AddSeconds(60)
  while (-not $moved -and (Get-Date) -lt $until) { $n = DumpOf 甲标签页 2; if ($n -and $n.tabs[0] -ne '甲标签页') { $moved = $n } }
  Check 'the first tab moved to the second place' ($moved -and $moved.tabs[1] -eq '甲标签页') "tabs: $($moved.tabs -join ', ')"
  Start-Sleep -Milliseconds 500
  # Nudge the pointer by a pixel inside the same pill, as a hand does.
  [G]::MoveTo([int]($moved.l + $moved.rects[0][0] + $moved.rects[0][1] / 2) + 1, $moved.t + 18); Start-Sleep -Milliseconds 500
  $under = MxPill $moved 0; $away = MxPill $moved 1
  Shot 'T-after-reorder'
  MxPark
  $underRest = MxPill $moved 0; $awayRest = MxPill $moved 1
  Check 'the pill now under the pointer shows the hover fill' (($under - $underRest) -ge 1.0) "pill brightness $underRest at rest, $under under the pointer"
  Check 'the pill that moved away shows none' ([math]::Abs($away - $awayRest) -le 0.5) "pill brightness $awayRest at rest, $away after the reorder"
  PfQuit
}

Case 'cleanup'
MxClean
