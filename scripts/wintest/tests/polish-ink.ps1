# Fence polish 2026-11-12, package ink (visual review V1 and V6).
# V1: in the Fluent style with 文字颜色 = 自动 the text of a fence takes its colour from the plate
#     it is drawn on (the wallpaper shows through it), so it reads in dark mode over a light
#     wallpaper and in light mode over a dark one; a fence over light and dark gets the halo.
# V6: a title in the fence's own tint reads on the band washed with that tint.
# Parts (run.sh --var Part=...): A contrast of every kind of text, both modes over five
# wallpapers; B hover and selection fills on the flipped text; C a fence moved from a light to
# a dark part, a wallpaper change, 白色 / 黑色, loose icons, the tinted title; D a Peek over a white window. Default: all.
if (-not $Part) { $Part = 'ABCD' }
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1

Add-Type -ReferencedAssemblies System.Drawing @'
using System; using System.Drawing; using System.Drawing.Imaging; using System.Runtime.InteropServices;
// The text in a crop that holds a plate and one kind of text: the plate is the median pixel by
// luminance, the ink the pixel farthest from it; their WCAG contrast. A halo shows as pixels on
// the other side of the plate from the ink (a dark rim around light text): its contrast too.
public static class PInk {
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

function SolidWall($r, $g, $b, $name) {
  $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($r, $g, $b)); $gr.Dispose()
  $p = "$PfRun\wall-$name.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
}
# Left half light, right half dark: a fence can be moved from one to the other.
function SplitWall() {
  $bm = New-Object Drawing.Bitmap 1600, 900; $gr = [Drawing.Graphics]::FromImage($bm)
  $gr.Clear([Drawing.Color]::FromArgb(18, 20, 26))
  $br = New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb(236, 239, 243)); $gr.FillRectangle($br, 0, 0, 800, 900); $br.Dispose(); $gr.Dispose()
  $p = "$PfRun\wall-split.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
}
# Large light and dark blocks: every fence lies over both.
function BusyWall() {
  $bm = New-Object Drawing.Bitmap 1600, 900; $gr = [Drawing.Graphics]::FromImage($bm)
  $gr.Clear([Drawing.Color]::FromArgb(24, 30, 52))
  $rng = New-Object Random 11
  for ($i = 0; $i -lt 90; $i++) {
    $v = 200 + $rng.Next(56)
    $br = New-Object Drawing.SolidBrush ([Drawing.Color]::FromArgb($v, $v, [math]::Min(255, $v + 6)))
    $gr.FillRectangle($br, $rng.Next(1600), $rng.Next(900), 60 + $rng.Next(160), 40 + $rng.Next(120)); $br.Dispose()
  }
  $gr.Dispose(); $p = "$PfRun\wall-busy.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
}
# The wallpaper Windows 11 ships with (light blue; it stays when Windows is switched to dark).
function PhotoWall() {
  $img = @('C:\Windows\Web\Wallpaper\Windows\img0.jpg', 'C:\Windows\Web\4K\Wallpaper\Windows\img0_1920x1200.jpg') | Where-Object { Test-Path $_ } | Select-Object -First 1
  if ($img) { [G]::SystemParametersInfoW(0x14, 0, $img, 3) | Out-Null } else { SolidWall 150 195 225 photo }
  Log "photo wallpaper: $img"
}
function Wall($name) {
  switch ($name) {
    'grey' { SolidWall 128 128 128 grey }
    'light' { SolidWall 236 239 243 light }
    'dark' { SolidWall 18 20 26 dark }
    'photo' { PhotoWall }
    'busy' { BusyWall }
    'split' { SplitWall }
  }
  # PecoFence looks at the wallpaper every 5 seconds.
  Start-Sleep -Seconds 8
}
function Paths([string[]]$names) { return @($names | ForEach-Object { Join-Path $PfDesk $_ }) }
$PiDocs = '季度报告 2026.docx', '笔记.txt', '项目计划书.xlsx', 'a.txt', 'README.md', '会议纪要 11月.txt'
$PiList = '发票扫描件.pdf', '设计稿 v3.txt', 'todo.md', '压缩包.zip'
$PiTab1 = '周报.docx', '排期.xlsx'
$PiTab2 = '课程.txt'
$PiRoll = 'r1 收起.txt', 'r2 收起.txt', 'r3 收起.txt'
$PiLoose = '散落 一.txt'
$PiAll = $PiDocs + $PiList + $PiTab1 + $PiTab2 + $PiRoll + $PiLoose
function PiClean() { foreach ($n in $PiAll) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
function Win($t) { return (FenceGet $t).windowRect }
# One reading: contrast, plate, ink, dark or light; a line in ink.tsv; the crop when asked.
function InkAt($tag, $x, $y, $w, $h, [switch]$Save) {
  $b = [Px]::Grab([int]$x, [int]$y, [int]$w, [int]$h)
  $r = [PInk]::Read($b)
  if ($Save) { $b.Save("$PfShots\crop-$($tag -replace '[^\w.-]', '_').png") }
  $b.Dispose()
  $p = $r -split '\|'
  "$tag`t$($p[0])`t$($p[1])`t$($p[2])`t$($p[3])`t$($p[4])" | Add-Content "$PfShots\ink.tsv" -Encoding UTF8
  $inv = [Globalization.CultureInfo]::InvariantCulture
  $rim = [double]::Parse($p[4], $inv)
  return [pscustomobject]@{ Contrast = [double]::Parse($p[0], $inv); Plate = $p[1]; Ink = $p[2]; Dark = ($p[3] -eq 'dark'); Halo = ($rim -ge 1.25); Rim = $rim
    Text = "$($p[0]) : 1, plate $($p[1]), text $($p[2]), halo $(if ($rim -ge 1.25) { "yes ($($p[4]) : 1)" } else { 'no' })" }
}
# Where each kind of text is, from the fences' own rects, and the contrast it needs: 3.9 : 1 for
# names and titles (what white text has on a mid-grey plate, the weakest pair that matched
# before), 2.2 : 1 for the fainter count and hint. Text with a halo reads from 2.6 : 1.
function Regions() {
  $d = Win '文档'; $l = Win '列表'; $t = Win '工作'; $r = Win '收起'; $e = Win '空栅栏'; $n = Win '便签'; $s = Win '脚本'
  $c = CellPt '文档' 0
  return @(
    @('title', ($d.x + 12), ($d.y + 8), 120, 22, 3.9),
    @('icon name', ($c[0] - 36), ($c[1] + 28), 72, 30, 3.9),
    @('list row', ($l.x + 36), ($l.y + 40), 150, 18, 3.9),
    @('tab captions', ($t.x + 8), ($t.y + 7), 170, 22, 3.9),
    @('rolled count', ($r.x + $r.w - 100), ($r.y + 8), 62, 22, 2.2),
    @('empty hint', ($e.x + 50), ($e.y + 44), ($e.w - 100), ($e.h - 54), 2.2),
    @('note text', ($n.x + 14), ($n.y + 42), 220, 60, 3.9),
    @('script rows', ($s.x + 14), ($s.y + 42), 220, 60, 3.9)
  )
}
# The farther of the two sides counts as the text (a light halo around dark text on a mid plate
# is farther from the plate than the text itself).
function Reads($m, $need) { return ([math]::Max($m.Contrast, $m.Rim) -ge $need -or ($m.Halo -and $m.Contrast -ge 2.6)) }
function Park() { ParkMouse 1540 780; Start-Sleep -Milliseconds 500 }

Case 'setup'
CloseWindows
PiClean
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
foreach ($n in $PiAll) { Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value "polish ink $n" -Encoding UTF8 }
SolidWall 128 128 128 grey
PfStart -Script @('sleep 3600000') -Lang 'zh-CN'
Start-Sleep -Seconds 2
ClearFences | Out-Null
Pf settings set featureTips false | Out-Null
Pf settings set themeStyle fluent | Out-Null
Pf fence create --title 文档 --rect '120,60,340,240' | Out-Null
Pf fence create --title 列表 --rect '480,60,300,200' | Out-Null
Pf fence create --title 工作 --rect '120,330,340,180' | Out-Null
Pf fence create --title 学习 --rect '900,640,200,120' | Out-Null
Pf fence create --title 收起 --rect '120,540,340,200' | Out-Null
Pf fence create --title 空栅栏 --rect '480,290,300,130' | Out-Null
Pf item move (Paths $PiDocs) --to 文档 | Out-Null
Pf item move (Paths $PiList) --to 列表 | Out-Null
Pf item move (Paths $PiTab1) --to 工作 | Out-Null
Pf item move (Paths $PiTab2) --to 学习 | Out-Null
Pf item move (Paths $PiRoll) --to 收起 | Out-Null
Pf fence set 文档 sort name | Out-Null
Pf fence set 列表 layout list | Out-Null
Pf fence merge 学习 --into 工作 | Out-Null
Pf fence roll 收起 | Out-Null
Pf fence create --tool note --title 便签 --text "今天要做的事`n给安娜打电话`n普通的一行文字" --rect '800,60,300,170' | Out-Null
Set-Content "$PfRun\hello.cmd" "@echo off`r`necho CPU 12%%`r`necho 内存 48%%`r`necho 磁盘 C: 剩余 21 GB" -Encoding OEM
Pf fence create --script "$PfRun\hello.cmd" --title 脚本 --rect '800,250,300,150' | Out-Null
Start-Sleep -Seconds 3
Park
Check 'the fences are there' (@(PfJson fence list | Where-Object { $_ -and $_.id }).Count -ge 8) "fences=$(@(PfJson fence list).Count)"

if ($Part -match 'A') {
  # ---- V1: every kind of text against its plate. Dark mode over a light or the Windows 11
  # wallpaper and light mode over a dark one were unreadable (1.3 to 2.8 : 1).
  function PartA() {
    trap { PfTrap $_; continue }
    foreach ($mode in 'dark', 'light') {
      Pf settings set theme $mode | Out-Null; Start-Sleep -Seconds 2
      foreach ($wall in 'grey', 'light', 'dark', 'photo') {
        Case "V1 $mode mode, $wall wallpaper"
        Wall $wall; Park
        Shot "A-$mode-$wall"
        foreach ($q in (Regions)) {
          $m = InkAt "$mode/$wall/$($q[0])" $q[1] $q[2] $q[3] $q[4] -Save:($mode -eq 'dark' -and $wall -eq 'light')
          Check "V1 $mode mode, $wall wallpaper: $($q[0]) reads" (Reads $m $q[5]) "$($m.Text) (needs $($q[5]) : 1, or 2.6 : 1 with a halo)"
        }
      }
      # A wallpaper of light and dark blocks: every fence lies over both and gets the halo.
      Case "V1 $mode mode, busy wallpaper"
      Wall 'busy'; Park
      Shot "A-$mode-busy"
      $halo = @(PfLogAll | Where-Object { $_ -match 'text colour picked from the plate' -and $_ -match 'halo=true' })
      Check "V1 $mode mode, busy wallpaper: fences over light and dark get the halo" ($halo.Count -ge 1) "log lines with halo=true: $($halo.Count)"
    }
  }
  PartA
}

if ($Part -match 'B') {
  # ---- V1: hover and selection fills still show where the text flipped.
  function Fill($tag, $p) {
    $b = [Px]::Grab(($p[0] - 33), ($p[1] - 27), 5, 5); $m = MeanRgb $b; $b.Dispose()
    Log "fill $tag = $($m -join ',')"
    return $m
  }
  function Delta($a, $b) { return [math]::Max([math]::Max([math]::Abs($a[0] - $b[0]), [math]::Abs($a[1] - $b[1])), [math]::Abs($a[2] - $b[2])) }
  function PartB() {
    trap { PfTrap $_; continue }
    foreach ($combo in @(@('dark', 'light'), @('light', 'dark'))) {
      $mode = $combo[0]; $wall = $combo[1]
      Case "V1 fills, $mode mode over a $wall wallpaper"
      Pf settings set theme $mode | Out-Null; Wall $wall
      $e = Win '空栅栏'
      [G]::Click(($e.x + 30), ($e.y + 100)); Park
      $p = CellPt '文档' 1
      $rest = Fill "$mode rest" $p
      [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 700
      $hover = Fill "$mode hover" $p
      Shot "B-$mode-hover"
      [G]::Click($p[0], $p[1]); Park
      $sel = Fill "$mode selected" $p
      Shot "B-$mode-selected"
      # Another program's window in front: the selection takes its neutral look.
      $had = @(Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
      Start-Process notepad; Start-Sleep -Seconds 2
      $np = @(Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $had -notcontains $_.Id })
      foreach ($q in $np) { if ($q.MainWindowHandle -ne [IntPtr]::Zero) { [L]::MoveWindow($q.MainWindowHandle, 900, 470, 520, 300, $true) | Out-Null } }
      Start-Sleep -Milliseconds 900
      $inactive = Fill "$mode selected, another program active" $p
      Shot "B-$mode-inactive"
      foreach ($q in $np) { Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue }
      Start-Sleep -Milliseconds 500
      Check "V1 $mode mode over a $wall wallpaper: the hover fill shows" ((Delta $rest $hover) -ge 5) "rest $($rest -join ','), hover $($hover -join ','), difference $(Delta $rest $hover)"
      Check "V1 $mode mode over a $wall wallpaper: the selection fill shows" ((Delta $rest $sel) -ge 12) "rest $($rest -join ','), selected $($sel -join ','), difference $(Delta $rest $sel)"
      Check "V1 $mode mode over a $wall wallpaper: the selection shows while another program is in front" ((Delta $rest $inactive) -ge 6) "rest $($rest -join ','), selected $($inactive -join ','), difference $(Delta $rest $inactive)"
      $d = Win '文档'
      [G]::Click(($d.x + 300), ($d.y + 220)); Park
    }
  }
  PartB
}

if ($Part -match 'C') {
  function TitleInk($tag, $title) { $w = Win $title; return (InkAt $tag ($w.x + 12) ($w.y + 8) 120 22) }
  function PartC() {
    trap { PfTrap $_; continue }
    # ---- V1: a fence moved from the light half of the wallpaper to the dark half and back.
    Case 'V1 a fence moved from a light part to a dark part'
    Pf settings set theme dark | Out-Null
    Wall 'split'
    foreach ($t in '列表', '工作', '收起', '空栅栏', '便签', '脚本') { Pf fence move $t --rect '1250,560,300,130' | Out-Null }
    Pf fence move 文档 --rect '200,60,340,240' | Out-Null; Start-Sleep -Milliseconds 900; Park
    $xs = @(200, 300, 400, 450, 500, 550, 600, 650, 700, 750, 800, 900, 1000)
    $right = @(); $left = @()
    foreach ($x in $xs) {
      Pf fence move 文档 --rect "$x,60,340,240" | Out-Null; Start-Sleep -Milliseconds 700
      $m = TitleInk "move right/$x" '文档'; $right += $m
      if ($x -eq 200 -or $x -eq 1000 -or $x -eq 600) { Shot "C-move-right-$x" }
    }
    [array]::Reverse($xs)
    foreach ($x in $xs) {
      Pf fence move 文档 --rect "$x,60,340,240" | Out-Null; Start-Sleep -Milliseconds 700
      $m = TitleInk "move left/$x" '文档'; $left += $m
    }
    $seqR = ($right | ForEach-Object { if ($_.Dark) { 'D' } else { 'L' } }) -join ''
    $seqL = ($left | ForEach-Object { if ($_.Dark) { 'D' } else { 'L' } }) -join ''
    $flips = { param($s) $n = 0; for ($i = 1; $i -lt $s.Length; $i++) { if ($s[$i] -ne $s[$i - 1]) { $n++ } }; $n }
    Check 'V1 move: dark text on the light half' ($right[0].Dark -and $right[0].Contrast -ge 3.9) $right[0].Text
    Check 'V1 move: light text on the dark half' ((-not $right[-1].Dark) -and $right[-1].Contrast -ge 3.9) $right[-1].Text
    Check 'V1 move: the text changes colour once on the way right' ((& $flips $seqR) -eq 1) "title text at x = 200 .. 1000: $seqR"
    Check 'V1 move: and once on the way back' ((& $flips $seqL) -eq 1) "title text at x = 1000 .. 200: $seqL"
    $switchR = $seqR.IndexOf('L'); $switchL = $seqL.IndexOf('D')
    Check 'V1 move: it keeps its colour near the edge (switches later in both directions)' ($switchR -ge 0 -and $switchL -ge 0 -and ($switchR + $switchL) -ge $xs.Count) "right: switch at step $switchR, back: at step $switchL of $($xs.Count)"
    $halo = @(PfLogAll | Where-Object { $_ -match 'text colour picked from the plate' -and $_ -match '文档' })
    Log "ink log lines for 文档: $($halo.Count); last: $($halo | Select-Object -Last 2)"

    # The same with the mouse, recorded for the report.
    Case 'V1 a fence dragged across with the mouse'
    Pf fence move 文档 --rect '330,60,340,240' | Out-Null; Start-Sleep -Milliseconds 900; Park
    RecStart 'drag' -X 240 -Y 40 -W 1000 -H 290 -Fps 20
    RecMark '栅栏在浅色壁纸上'
    Start-Sleep -Milliseconds 1200
    [G]::MoveTo(480, 78); Start-Sleep -Milliseconds 300
    RecMark '按住标题向右拖到深色壁纸上'
    MDown; Start-Sleep -Milliseconds 150
    Glide 480 78 1000 78 40 45
    Start-Sleep -Milliseconds 400
    MUp
    RecMark '松开'
    Start-Sleep -Milliseconds 1500
    RecStop
    $m = TitleInk 'after the drag' '文档'
    Check 'V1 drag: the fence ends on the dark half with light text' ((-not $m.Dark) -and $m.Contrast -ge 3.9) "$($m.Text); window $((Win '文档').x)"

    # ---- V1: a wallpaper change switches the text once, with no flicker on the way.
    Case 'V1 wallpaper change'
    Pf fence move 文档 --rect '200,60,340,240' | Out-Null
    SolidWall 18 20 26 dark; Start-Sleep -Seconds 8; Park
    $before = TitleInk 'wallpaper dark' '文档'
    SolidWall 236 239 243 light
    $seq = ''
    $until = (Get-Date).AddSeconds(10)
    while ((Get-Date) -lt $until) { $m = TitleInk 'wallpaper change' '文档'; $seq += $(if ($m.Dark) { 'D' } else { 'L' }); Start-Sleep -Milliseconds 120 }
    $after = TitleInk 'wallpaper light' '文档'
    Check 'V1 wallpaper change: light text before, dark text after' ((-not $before.Dark) -and $after.Dark -and $after.Contrast -ge 3.9) "before: $($before.Text); after: $($after.Text)"
    Check 'V1 wallpaper change: the text changes colour once' ((& $flips $seq) -le 1) "title text every 120 ms: $seq"

    # ---- 白色 / 黑色 stay as chosen, whatever the wallpaper.
    Case 'V1 fixed text colours'
    Pf settings set labelColor white | Out-Null; Start-Sleep -Seconds 2; Park
    $c = CellPt '文档' 0
    $m = InkAt 'forced white/light wallpaper/icon name' ($c[0] - 36) ($c[1] + 28) 72 30
    Check 'V1 白色 stays white on a light wallpaper' (-not $m.Dark) $m.Text
    Pf settings set labelColor black | Out-Null
    SolidWall 18 20 26 dark; Start-Sleep -Seconds 8; Park
    $m = InkAt 'forced black/dark wallpaper/icon name' ($c[0] - 36) ($c[1] + 28) 72 30
    Check 'V1 黑色 stays black on a dark wallpaper' $m.Dark $m.Text
    Pf settings set labelColor auto | Out-Null

    # ---- Loose icons (像桌面一样排列) keep the white names of the Windows desktop.
    Case 'V1 loose icons'
    Pf fence set inbox desktopLayout true | Out-Null
    SolidWall 236 239 243 light; Start-Sleep -Seconds 8; Park
    $p = ItemPt '桌面' '散落 一'
    if (-not $p) { $p = ItemPt 'Desktop' '散落 一' }
    $white = $false
    if ($p) {
      $b = [Px]::Grab(($p[0] - 36), ($p[1] + 26), 72, 32); $b.Save("$PfShots\crop-loose.png")
      foreach ($px in [Px]::Pixels($b)) { if ((($px -shr 16) -band 255) -ge 250 -and (($px -shr 8) -band 255) -ge 250 -and ($px -band 255) -ge 250) { $white = $true; break } }
      $b.Dispose()
    }
    Shot 'C-loose'
    Check 'V1 loose icons keep white names on a light wallpaper' $white "icon at $($p -join ',')"
    Pf fence set inbox desktopLayout false | Out-Null

    # ---- V6: a title in the fence's own tint on the band washed with that tint.
    Case 'V6 tinted title'
    SolidWall 128 128 128 grey; Start-Sleep -Seconds 8
    Pf fence set 文档 tint '"#3b82f6"' | Out-Null
    Pf fence set 文档 titleColor tint | Out-Null
    foreach ($mode in 'dark', 'light') {
      Pf settings set theme $mode | Out-Null
      foreach ($only in 'true', 'false') {
        Pf fence set 文档 tintTitleOnly $only | Out-Null; Start-Sleep -Milliseconds 1500; Park
        $m = TitleInk "tinted title/$mode/title only $only" '文档'
        $w = Win '文档'
        Crop "V6-$mode-titleonly-$only" $w.x $w.y 340 60 | Out-Null
        $rgb = @($m.Ink -split ',' | ForEach-Object { [int]$_ })
        Check "V6 $mode mode, tint on the title row only = ${only}: the title reads on its band" ($m.Contrast -ge 2.9) $m.Text
        Check "V6 $mode mode, tint on the title row only = ${only}: the title is still blue" ($rgb[2] -gt $rgb[0]) "title colour $($m.Ink)"
      }
    }
    Pf fence set 文档 titleColor theme | Out-Null
    Pf fence set 文档 tint null | Out-Null
  }
  PartC
}

if ($Part -match 'D') {
  # ---- V1 in a Peek: the fence floats over other windows, not over the wallpaper its text was
  # picked for. Light text in light mode goes back to dark for the Peek (the windows behind may
  # be light); dark text reads over either.
  function PartD() {
    trap { PfTrap $_; continue }
    Pf fence set 文档 titleColor theme | Out-Null
    Pf fence set 文档 tint null | Out-Null
    Pf settings set labelColor auto | Out-Null
    foreach ($combo in @(@('light', 'dark'), @('dark', 'light'))) {
      $mode = $combo[0]; $wall = $combo[1]
      Case "V1 Peek over a white window, $mode mode over a $wall wallpaper"
      Pf settings set theme $mode | Out-Null; Wall $wall; Park
      $had = @(Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
      Start-Process notepad; Start-Sleep -Seconds 2
      $np = @(Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $had -notcontains $_.Id })
      foreach ($q in $np) { if ($q.MainWindowHandle -ne [IntPtr]::Zero) { [L]::MoveWindow($q.MainWindowHandle, 0, 0, 1600, 852, $true) | Out-Null } }
      Start-Sleep -Milliseconds 900
      PeekKey; Start-Sleep -Milliseconds 1500
      $w = Win '文档'; $c = CellPt '文档' 0
      $t = InkAt "peek/$mode/$wall/title" ($w.x + 12) ($w.y + 8) 120 22
      $n = InkAt "peek/$mode/$wall/icon name" ($c[0] - 36) ($c[1] + 28) 72 30
      Shot "D-$mode-peek"
      Check "V1 Peek, $mode mode over a $wall wallpaper: the title reads over a white window" (Reads $t 3.9) $t.Text
      Check "V1 Peek, $mode mode over a $wall wallpaper: the names read over a white window" (Reads $n 3.9) $n.Text
      PeekKey; Start-Sleep -Milliseconds 800
      foreach ($q in $np) { Stop-Process -Id $q.Id -Force -ErrorAction SilentlyContinue }
      Start-Sleep -Milliseconds 900; Park
      $t = InkAt "after the peek/$mode/$wall/title" ($w.x + 12) ($w.y + 8) 120 22
      Check "V1 Peek, $mode mode over a $wall wallpaper: after the Peek the title reads on the wallpaper again" (Reads $t 3.9) $t.Text
    }
  }
  PartD
}

Case 'cleanup'
PfQuit
PiClean
SolidWall 128 128 128 grey
[Anim]::Set($AnimWas)
