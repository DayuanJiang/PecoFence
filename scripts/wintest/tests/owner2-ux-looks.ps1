# Owner round 2026-11-12, package ux-looks: how a fence looks at rest.
# Parts (run.sh --var Part=...; in a Hyper-V guest set $Part in a copy of this file):
#   T  a fully transparent fence (不透明度 全透明) takes its text colour from the wallpaper it
#      sits on, and from its plate only while that shows under the pointer. It took the
#      plate's colour at rest too: a note, a clock and item names were white on a white
#      wallpaper with a dark theme file (1.1 : 1), and dark on a dark wallpaper in light mode.
#   R  tabs at the left (标签栏位置 左侧): a column too short for its tabs keeps rows as tall
#      as a caption and scrolls, the shown tab in view. The rows were squeezed until the
#      captions ran into each other (seven tabs in a fence 150 px tall: rows of 10 px).
#   H  on a fully transparent fence the note text, the clock and the script rows have the halo
#      that its title and item names have (they had none: on a wallpaper with light and dark
#      parts a line of a note could not be read).
#   G  Liquid Glass, dark mode, 不透明度 更厚实, a light wallpaper: the shown tab's caption reads
#      on its pill. The pill's lens shows the light wallpaper through the dark veil, and the
#      caption picked for the veil was white on it (2.0 : 1).
# Default: all.
if (-not $Part) { $Part = 'TRHG' }
trap { PfTrap $_; continue }

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
// A crop that holds one kind of text on its background: the background is the median pixel by
// luminance, the ink the pixel farthest from it; their WCAG contrast and the pixel farthest on
// the other side (a halo). "contrast|background|ink|dark or light|halo contrast".
public static class UlInk {
  static double Lin(int v) { double c = v / 255.0; return c <= 0.04045 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4); }
  static double Lum(int c) { return 0.2126 * Lin((c >> 16) & 255) + 0.7152 * Lin((c >> 8) & 255) + 0.0722 * Lin(c & 255); }
  static string Rgb(int c) { return ((c >> 16) & 255) + "," + ((c >> 8) & 255) + "," + (c & 255); }
  public static string Read(Bitmap b) {
    var d = b.LockBits(new Rectangle(0, 0, b.Width, b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
    var px = new int[b.Width * b.Height]; Marshal.Copy(d.Scan0, px, 0, px.Length); b.UnlockBits(d);
    var lum = new double[px.Length];
    for (int i = 0; i < px.Length; i++) lum[i] = Lum(px[i]);
    var sorted = (double[])lum.Clone(); Array.Sort(sorted);
    double med = sorted[sorted.Length / 2];
    int plate = 0, ink = 0; double near = 9, far = -1;
    for (int i = 0; i < px.Length; i++) {
      double diff = Math.Abs(lum[i] - med);
      if (diff < near) { near = diff; plate = i; }
      if (diff > far) { far = diff; ink = i; }
    }
    bool dark = lum[ink] < lum[plate];
    double rim = lum[plate];
    for (int i = 0; i < px.Length; i++) { if (dark ? lum[i] > rim : lum[i] < rim) rim = lum[i]; }
    double hi = Math.Max(lum[ink], lum[plate]), lo = Math.Min(lum[ink], lum[plate]);
    double rhi = Math.Max(rim, lum[plate]), rlo = Math.Min(rim, lum[plate]);
    return string.Format(System.Globalization.CultureInfo.InvariantCulture, "{0:F2}|{1}|{2}|{3}|{4:F2}",
      (hi + 0.05) / (lo + 0.05), Rgb(px[plate]), Rgb(px[ink]), dark ? "dark" : "light", (rhi + 0.05) / (rlo + 0.05));
  }
}
'@

function UlWall($r, $g, $b, $name) {
  $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($r, $g, $b)); $gr.Dispose()
  $p = "$PfRun\wall-$name.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
  # PecoFence looks at the wallpaper every 5 seconds.
  Start-Sleep -Seconds 8
}
# One reading of a crop: contrast, background, ink, whether the ink is the darker one.
function UlRead($tag, $x, $y, $w, $h) {
  $b = [Px]::Grab([int]$x, [int]$y, [int]$w, [int]$h)
  $r = [UlInk]::Read($b)
  $b.Save("$PfShots\crop-$($tag -replace '[^\w.-]', '_').png"); $b.Dispose()
  $p = $r -split '\|'
  $inv = [Globalization.CultureInfo]::InvariantCulture
  "$tag`t$($p -join "`t")" | Add-Content "$PfShots\ink.tsv" -Encoding UTF8
  return [pscustomobject]@{ Contrast = [double]::Parse($p[0], $inv); Dark = ($p[3] -eq 'dark'); Rim = [double]::Parse($p[4], $inv); Text = "$($p[0]) : 1, text $($p[2]) on $($p[1]), halo $($p[4]) : 1" }
}
function UlPark() { ParkMouse 1540 780; Start-Sleep -Milliseconds 600 }
$UlFiles = '季度报告 2026.docx', '笔记.txt', 'a1 新年活动安排与分工.txt'
function UlClean() { foreach ($n in $UlFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }

Case 'setup'
CloseWindows
UlClean
foreach ($n in $UlFiles) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value "ux looks $n" -Encoding UTF8 }
UlWall 255 255 255 white
PfStart -Script (DumpScript) -Lang 'zh-CN'
Start-Sleep -Seconds 2
ClearFences | Out-Null
Pf settings set featureTips false | Out-Null
Pf settings set themeStyle fluent | Out-Null
# A theme file with a dark plate in dark mode, as the example theme 暖色 has.
New-Item -ItemType Directory -Force "$PfApp\config\themes" | Out-Null
Set-Content -LiteralPath "$PfApp\config\themes\dark-plate.json" -Encoding ASCII -Value '{ "base": "fluent", "dark": { "tint": "#3A2414D9", "glassLayerFill": "#2A180C40", "textPrimary": "#FFF3E6", "textSecondary": "#F2DCC6" } }'
Pf fence create --title 文档 --rect '200,100,340,150' | Out-Null
Pf item move @($UlFiles | ForEach-Object { Join-Path $PfDesk $_ }) --to 文档 | Out-Null
Pf fence set 文档 sort name | Out-Null
Pf fence create --tool note --title 便签 --text "今天要做的事`n给安娜打电话" --rect '600,100,280,150' | Out-Null
Pf fence create --tool clock --title 时钟 --rect '600,300,260,130' | Out-Null
Start-Sleep -Seconds 3
UlPark
Check 'the fences are there' (@(PfJson fence list | Where-Object { $_ -and $_.id }).Count -ge 4) "fences=$(@(PfJson fence list).Count)"

if ($Part -match 'T') {
  # Where the texts are, from the fences' own rects (100 %).
  function UlTexts() {
    $n = (FenceGet '便签').windowRect; $c = (FenceGet '时钟').windowRect; $p = CellPt '文档' 1
    return @(
      @('note text', ($n.x + 14), ($n.y + 38), 100, 18),
      @('clock time', ($c.x + 14), ($c.y + 44), 130, 40),
      @('item name', ($p[0] - 36), ($p[1] + 28), 72, 16)
    )
  }
  # One look: every fence fully transparent, the pointer away, then over the note, then away.
  function UlClear($tag, $wantDark) {
    trap { PfTrap $_; continue }
    Pf fence set --all opacity transparent | Out-Null
    Start-Sleep -Seconds 3
    UlPark
    Shot "T-$tag-rest"
    foreach ($q in (UlTexts)) {
      $m = UlRead "$tag/rest/$($q[0])" $q[1] $q[2] $q[3] $q[4]
      # The item name has a halo: its colour counts. The note and the clock must read by themselves.
      if ($q[0] -eq 'item name') { Check "T $tag, pointer away: the $($q[0]) is $(if ($wantDark) { 'dark' } else { 'light' })" ($m.Dark -eq $wantDark -or $m.Contrast -ge 3.9) $m.Text }
      else { Check "T $tag, pointer away: the $($q[0]) reads on the wallpaper" ($m.Dark -eq $wantDark -and $m.Contrast -ge 3.9) "$($m.Text) (needs 3.9 : 1)" }
    }
    # Over the note the plate shows: its text reads on the plate.
    $n = (FenceGet '便签').windowRect
    [G]::MoveTo(($n.x + 200), ($n.y + 120)); Start-Sleep -Milliseconds 1200
    Shot "T-$tag-hover"
    $m = UlRead "$tag/hover/note text" ($n.x + 14) ($n.y + 38) 100 18
    Check "T $tag, pointer over the note: its text reads on the plate" ($m.Contrast -ge 3.9) "$($m.Text) (needs 3.9 : 1)"
    UlPark; Start-Sleep -Milliseconds 800
    $m = UlRead "$tag/rest again/note text" ($n.x + 14) ($n.y + 38) 100 18
    Check "T $tag, pointer away again: the note reads on the wallpaper" ($m.Dark -eq $wantDark -and $m.Contrast -ge 3.9) "$($m.Text) (needs 3.9 : 1)"
    Pf fence set --all opacity default | Out-Null
    Start-Sleep -Milliseconds 800
  }
  Case 'T: a dark theme file over a white wallpaper, dark mode'
  Pf settings set theme dark | Out-Null
  Pf settings set customTheme dark-plate --string | Out-Null
  Start-Sleep -Seconds 2
  UlClear 'dark theme, white wallpaper' $true
  Case 'T: Fluent, light mode over a dark wallpaper'
  Pf settings set customTheme null | Out-Null
  Pf settings set theme light | Out-Null
  UlWall 90 92 93 darkgrey
  UlClear 'light mode, dark wallpaper' $false
  Case 'T: Fluent, dark mode over a white wallpaper'
  Pf settings set theme dark | Out-Null
  UlWall 255 255 255 white
  UlClear 'dark mode, white wallpaper' $true
}

if ($Part -match 'H') {
  Case 'H: note, clock and script text on a fully transparent fence have the halo'
  Pf settings set customTheme null | Out-Null
  Pf settings set themeStyle fluent | Out-Null
  Pf settings set theme light | Out-Null
  Set-Content -LiteralPath "$PfRun\rows.ps1" -Encoding UTF8 -Value "'CPU 12 %'`r`n'内存 5.2 GB / 16 GB'"
  Pf fence create --script "$PfRun\rows.ps1" --title 脚本 --rect '920,100,260,150' | Out-Null
  UlWall 128 128 128 midgrey
  Pf fence set --all opacity transparent | Out-Null
  Start-Sleep -Seconds 3
  UlPark
  Shot 'H-transparent-midgrey'
  $n = (FenceGet '便签').windowRect; $c = (FenceGet '时钟').windowRect; $s = (FenceGet '脚本').windowRect; $p = CellPt '文档' 1
  $name = UlRead 'H/item name' ($p[0] - 36) ($p[1] + 28) 72 16
  Check 'H the item name has a halo (as before)' ($name.Rim -ge 1.25) $name.Text
  foreach ($q in @(@('note text', ($n.x + 14), ($n.y + 38), 100, 18), @('clock time', ($c.x + 14), ($c.y + 44), 130, 40), @('script rows', ($s.x + 14), ($s.y + 40), 150, 20))) {
    $m = UlRead "H/$($q[0])" $q[1] $q[2] $q[3] $q[4]
    Check "H the $($q[0]) has a halo" ($m.Rim -ge 1.25) "$($m.Text) (a halo shows from 1.25 : 1)"
  }
  Pf fence set --all opacity default | Out-Null
  Pf fence delete 脚本 | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'G') {
  Case 'G: Liquid Glass, dark mode, 更厚实: the shown tab on a light wallpaper'
  Pf settings set customTheme null | Out-Null
  Pf settings set theme dark | Out-Null
  Pf settings set themeStyle liquidGlass | Out-Null
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  # Light, but dark enough under the veil of 更厚实 for the plate to take light text.
  UlWall 156 186 206 lightblue
  foreach ($t in '工作', '学习', '娱乐与游戏') { Pf fence create --title $t --rect '920,300,340,150' | Out-Null }
  Pf fence merge 学习 --into 工作 | Out-Null
  Pf fence merge 娱乐与游戏 --into 工作 | Out-Null
  Pf fence set 工作 opacity solid | Out-Null
  Start-Sleep -Seconds 3
  UlPark
  FenceIds
  $d = DumpOf '工作'
  $i = [array]::IndexOf(@($d.tabs), $script:IdTitle["$($d.active)"])
  $tab = $d.rects[$i]
  Shot 'G-glass-solid-tabs'
  # The shown tab's caption inside its pill, and a resting tab's on the plate.
  $m = UlRead 'G/shown tab' ($d.l + $tab[0] + 10) ($d.t + 10) ($tab[1] - 20) 16
  Check 'G the shown tab reads on its pill' ($m.Contrast -ge 3.9) "$($m.Text) (needs 3.9 : 1); tab $i of $(@($d.tabs) -join ', ')"
  $j = if ($i -eq 0) { 1 } else { 0 }
  $rest = $d.rects[$j]
  $m2 = UlRead 'G/resting tab' ($d.l + $rest[0] + 10) ($d.t + 10) ($rest[1] - 20) 16
  # The plate around the pill keeps its light text (secondary: 2.9 : 1 here, as before).
  Check 'G a resting tab keeps its light caption on the dark plate' ((-not $m2.Dark) -and $m2.Contrast -ge 2.6) "$($m2.Text)"
  foreach ($t in '工作', '学习', '娱乐与游戏') { Pf fence delete $t | Out-Null }
  Pf settings set themeStyle fluent | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'R') {
  Case 'R: seven tabs at the left in a fence 150 px tall'
  $UlMonths = '一月', '二月', '三月', '四月', '五月', '六月', '七月'
  foreach ($t in $UlMonths) { Pf fence create --title $t --rect '200,420,340,150' | Out-Null }
  foreach ($t in $UlMonths[1..6]) { Pf fence merge $t --into 一月 | Out-Null }
  # A fence right under it: the tab fence cannot grow for its column.
  Pf fence create --title 下面 --rect '200,586,340,120' | Out-Null
  Pf settings set tabBar left | Out-Null; Start-Sleep -Seconds 2
  UlPark
  FenceIds
  # The column as dumped: (y, h) per row in DIPs; the shown tab's row.
  function UlRail($tag) {
    $d = DumpOf '一月'
    $hs = @($d.rects | ForEach-Object { $_[1] })
    $shown = $script:IdTitle["$($d.active)"]
    $i = [array]::IndexOf(@($d.tabs), $shown)
    $row = if ($i -ge 0) { $d.rects[$i] } else { @(-1, 0) }
    Crop "R-$tag" $d.l $d.t 120 ($d.b - $d.t) | Out-Null
    return [pscustomobject]@{ Least = ($hs | Measure-Object -Minimum).Minimum; Active = $i; Top = $row[0]; Bottom = $row[0] + $row[1]; H = ($d.b - $d.t)
      Text = "rows (y, h): $(($d.rects | ForEach-Object { '{0:F0},{1:F0}' -f $_[0], $_[1] }) -join ' '); shown tab $i '$shown'; window $($d.b - $d.t) px tall" }
  }
  $r = UlRail 'last-tab'
  Check 'R the rows are as tall as a caption (24 px)' ($r.Least -ge 23.5) $r.Text
  Check 'R the shown tab is whole inside the column' ($r.Active -ge 0 -and $r.Top -ge 7.5 -and $r.Bottom -le $r.H - 43.5) $r.Text
  # The wheel over the column goes to the tabs before: the column follows.
  $d = DumpOf '一月'
  Wheel ($d.l + 40) ($d.t + 60) 6; Start-Sleep -Milliseconds 1500
  UlPark
  $r2 = UlRail 'first-tab'
  Check 'R after the wheel the first tab is shown, whole, in rows as tall as a caption' ($r2.Active -eq 0 -and $r2.Top -ge 7.5 -and $r2.Bottom -le $r2.H - 43.5 -and $r2.Least -ge 23.5) $r2.Text
  # A click on a row in view shows that tab.
  $d = DumpOf '一月'
  [G]::Click(($d.l + 40), [int]($d.t + $d.rects[2][0] + $d.rects[2][1] / 2)); Start-Sleep -Milliseconds 900
  UlPark
  $r3 = UlRail 'third-tab'
  Check 'R a click on the third row shows the third tab' ($r3.Active -eq 2) $r3.Text
  # Three tabs in the same fence fit (98 px for rows of 32: they shrink to 30): nothing scrolls.
  foreach ($t in '四月', '五月', '六月', '七月') { Pf fence delete $t | Out-Null }
  Start-Sleep -Seconds 1
  $d = DumpOf '一月'
  $hs = @($d.rects | ForEach-Object { $_[1] })
  $last = $d.rects[$hs.Count - 1]
  Check 'R three tabs in the same fence fit without scrolling' ($hs.Count -eq 3 -and ($hs | Measure-Object -Minimum).Minimum -ge 29.5 -and [math]::Abs($d.rects[0][0] - 8) -le 0.5 -and ($last[0] + $last[1]) -le ($d.b - $d.t) - 43.5) "rows (y, h): $(($d.rects | ForEach-Object { '{0:F0},{1:F0}' -f $_[0], $_[1] }) -join ' ')"
  Pf settings set tabBar top | Out-Null
}

Case 'cleanup'
Pf settings set customTheme null | Out-Null
UlClean
