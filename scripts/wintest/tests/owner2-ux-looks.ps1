# Owner round 2026-11-12, package ux-looks: how a fence looks at rest.
# Parts (run.sh --var Part=...; in a Hyper-V guest set $Part in a copy of this file):
#   T  a fully transparent fence (不透明度 全透明) takes its text colour from the wallpaper it
#      sits on, and from its plate only while that shows under the pointer. It took the
#      plate's colour at rest too: a note, a clock and item names were white on a white
#      wallpaper with a dark theme file (1.1 : 1), and dark on a dark wallpaper in light mode.
#   R  tabs at the left (标签栏位置 左侧): a column too short for its tabs keeps rows as tall
#      as a caption and scrolls, the shown tab in view. The rows were squeezed until the
#      captions ran into each other (seven tabs in a fence 150 px tall: rows of 10 px).
# Default: all.
if (-not $Part) { $Part = 'TR' }
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
  return [pscustomobject]@{ Contrast = [double]::Parse($p[0], $inv); Dark = ($p[3] -eq 'dark'); Text = "$($p[0]) : 1, text $($p[2]) on $($p[1])" }
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
  # Three tabs in the same fence fit: nothing scrolls, the rows keep their 32 px.
  foreach ($t in '四月', '五月', '六月', '七月') { Pf fence delete $t | Out-Null }
  Start-Sleep -Seconds 1
  $d = DumpOf '一月'
  $hs = @($d.rects | ForEach-Object { $_[1] })
  Check 'R three tabs in the same fence keep rows of 32 px' ($hs.Count -eq 3 -and ($hs | Measure-Object -Minimum).Minimum -ge 31.5 -and $d.rects[0][0] -ge 7.5) "rows (y, h): $(($d.rects | ForEach-Object { '{0:F0},{1:F0}' -f $_[0], $_[1] }) -join ' ')"
  Pf settings set tabBar top | Out-Null
}

Case 'cleanup'
Pf settings set customTheme null | Out-Null
UlClean
