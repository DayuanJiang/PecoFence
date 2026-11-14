# Owner round 2026-11-12, third batch, package ux3-looks: this evening's visible changes in the
# looks their authors did not see (they looked in Fluent, light mode, 100 %, a grey wallpaper).
# One scene holds the changes; every look in $Look gets the same pictures of it:
#   rest       the whole desktop: secondary text (tabs not shown, the clock's date, ticked to-dos,
#              a footer, column headers), a smart fence's count capsule, a link in a note, a
#              fence with a tint and 更透明, a rolled fence's count
#   dock       a fence at the top edge of the screen (square top corners), closed and opened
#   tip        an icon's tip, and the tip on a smart fence's count with a plus
#   name-*     the name box of a centred and a right-aligned title, of a tab with a glyph, of a
#              row in the left tab column
#   guide      the alignment guide while a fence is held level with another, its fade at the
#              release (a recording), the landing outline over another fence
#   note-*     a link under the pointer, a check box under the pointer while the note is edited
#   peek       the Peek search box with a list
#   settings   the settings page (follows the colour mode)
# run.sh --extra ui/themes --var Look=<look>,<look>... [--var Scale=150]
#   a look is <style>-<mode>-<wallpaper>: style fluent | glass | warm | frosted | rounded (the
#   three example theme files), mode light | dark, wallpaper bright | dark | grey | photo (the
#   Windows 11 picture). In a Hyper-V guest set $Look in a copy of this file.
# The checks are few and plain (the thing is on screen); the pictures are what is judged.
if (-not (Get-Variable Look -ErrorAction SilentlyContinue)) { $Look = 'fluent-dark-bright' }
if (-not (Get-Variable Scale -ErrorAction SilentlyContinue)) { $Scale = 100 }
$S = [double]$Scale / 100
trap { PfTrap $_; continue }
. C:\pf\kit\rec.ps1

function FWin($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function Dp($v) { return [int]($v * $S) }
function LkCrop($name, $x, $y, $w, $h) {
  $x = [Math]::Max(0, [int]$x); $y = [Math]::Max(0, [int]$y)
  $w = [Math]::Min([int]$w, 1600 - $x); $h = [Math]::Min([int]$h, 900 - $y)
  $b = Crop $name $x $y $w $h; $b.Dispose()
}
# The most saturated pixel of a part of the screen (an accent mark) against the part's median
# pixel (the plate): their WCAG contrast.
function LkLin($v) { $c = $v / 255.0; if ($c -le 0.04045) { return $c / 12.92 }; return [Math]::Pow(($c + 0.055) / 1.055, 2.4) }
function LkLum($c) { return 0.2126 * (LkLin (($c -shr 16) -band 255)) + 0.7152 * (LkLin (($c -shr 8) -band 255)) + 0.0722 * (LkLin ($c -band 255)) }
function LkRgb($c) { return "$(($c -shr 16) -band 255),$(($c -shr 8) -band 255),$($c -band 255)" }
function LkAccent($x, $y, $w, $h) {
  $b = [Px]::Grab([int]$x, [int]$y, [int]$w, [int]$h); $px = [Px]::Pixels($b); $b.Dispose()
  $best = 0; $sat = -1; $grey = New-Object 'int[]' $px.Count
  for ($i = 0; $i -lt $px.Count; $i++) {
    $c = $px[$i]; $cr = ($c -shr 16) -band 255; $cg = ($c -shr 8) -band 255; $cb = $c -band 255
    $s = [Math]::Max($cr, [Math]::Max($cg, $cb)) - [Math]::Min($cr, [Math]::Min($cg, $cb))
    if ($s -gt $sat) { $sat = $s; $best = $c }
    $grey[$i] = $cr * 299 + $cg * 587 + $cb * 114
  }
  $sorted = [int[]]$grey.Clone(); [Array]::Sort($sorted)
  $plate = $px[[Array]::IndexOf($grey, $sorted[[int]($sorted.Count / 2)])]
  $la = LkLum $best; $lp = LkLum $plate
  $ratio = ([Math]::Max($la, $lp) + 0.05) / ([Math]::Min($la, $lp) + 0.05)
  return [pscustomobject]@{ Ratio = $ratio; Text = ('{0:F2} : 1, mark {1} on {2}' -f $ratio, (LkRgb $best), (LkRgb $plate)) }
}
function LkPark() { ParkMouse 1500 800; Start-Sleep -Milliseconds 500 }
function LkWall($wall) {
  $rgb = switch ($wall) { 'bright' { @(236, 239, 243) } 'dark' { @(28, 32, 44) } default { @(128, 128, 128) } }
  $img = 'C:\Windows\Web\Wallpaper\Windows\img0.jpg'
  if ($wall -eq 'photo' -and (Test-Path $img)) { [G]::SystemParametersInfoW(0x14, 0, $img, 3) | Out-Null; return }
  $bm = New-Object Drawing.Bitmap 32, 32; $gr = [Drawing.Graphics]::FromImage($bm); $gr.Clear([Drawing.Color]::FromArgb($rgb[0], $rgb[1], $rgb[2])); $gr.Dispose()
  $p = "$PfRun\wall-$wall.bmp"; $bm.Save($p, [Drawing.Imaging.ImageFormat]::Bmp); $bm.Dispose()
  [G]::SystemParametersInfoW(0x14, 0, $p, 3) | Out-Null
}
function LkRename($x, $y) {
  [G]::RClick($x, $y); Start-Sleep -Milliseconds 900
  MenuPick '^重命名' (MenuWin 5) | Out-Null
  $h = WaitWin 'PecoFence.Rename' 5
  Start-Sleep -Milliseconds 400
  return ($h -ne [IntPtr]::Zero)
}
$LkFiles = '报告 第三季度.docx', '报价单 客户甲.txt', '预算表 2026.xlsx', '备忘 周会.md', '笔记 读书.txt', '宝贝计划.txt', '清单 甲.txt', '清单 乙 名字长一点的文件.txt'
function LkClean() { foreach ($n in $LkFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
function LkPath($i) { return (Join-Path $PfDesk $LkFiles[$i]) }

# The pictures of one look.
function LkShots($tag, $withSettings) {
  trap { PfTrap $_; continue }
  LkPark
  Shot "$tag-00-rest"
  # The accent marks against their plate (a plain wallpaper only: a photo has colours of its
  # own): the bar under the shown tab, the magnifier of the smart fence's title.
  if ($tag -notmatch 'photo') {
    $r = Rect (FWin '季度报告')
    $m = LkAccent ($r.L + 4) ($r.T + 2) (Dp 180) (Dp 34)
    Check "$tag the bar under the shown tab reads on its plate (3 : 1)" ($m.Ratio -ge 3.0) $m.Text
    $r = Rect (FWin '最近报告')
    $m = LkAccent ($r.L + 4) ($r.T + 2) (Dp 36) (Dp 34)
    Check "$tag the magnifier of a smart fence's title reads on its plate (2.8 : 1, a thin glyph)" ($m.Ratio -ge 2.8) $m.Text
  }

  # A fence at the top edge: closed, then opened by the pointer.
  $r = Rect (FWin '抽屉')
  LkCrop "$tag-10-dock" ($r.L - 20) 0 ($r.R - $r.L + 40) (Dp 70)
  [G]::MoveTo($r.L + (Dp 100), (Dp 18)); Start-Sleep -Milliseconds 300; [G]::MoveTo($r.L + (Dp 102), (Dp 18)); Start-Sleep -Milliseconds 1300
  $o = Rect (FWin '抽屉')
  LkCrop "$tag-11-dock-open" ($r.L - 20) 0 ($r.R - $r.L + 40) ($o.B + 20)
  Check "$tag the docked fence opens under the pointer" (($o.B - $o.T) -gt (Dp 100)) "height $($o.B - $o.T)"
  LkPark; Start-Sleep -Milliseconds 900

  # An icon's tip, and the tip on the smart fence's count.
  $p = CellPt '色调' 0
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300; [G]::MoveTo($p[0] + 2, $p[1]); Start-Sleep -Milliseconds 1700
  LkCrop "$tag-20-tip-icon" ($p[0] - (Dp 60)) ($p[1] - (Dp 40)) (Dp 420) (Dp 150)
  $tips = @([W]::Tops('PecoFence.Tooltip')).Count
  LkPark
  $r = Rect (FWin '最近报告')
  [G]::MoveTo($r.R - (Dp 50), $r.T + (Dp 18)); Start-Sleep -Milliseconds 300; [G]::MoveTo($r.R - (Dp 48), $r.T + (Dp 18)); Start-Sleep -Milliseconds 1700
  LkCrop "$tag-21-tip-count" ($r.L - 10) ($r.T - 10) ($r.R - $r.L + (Dp 160)) (Dp 110)
  LkPark

  # The name box: a centred title, a right-aligned title, a tab with a glyph, a row at the left.
  foreach ($align in 'center', 'right') {
    Pf settings set titleAlign $align | Out-Null; Start-Sleep -Milliseconds 900
    $r = Rect (FWin '清单')
    $ok = LkRename ($r.L + (Dp 30)) ($r.T + (Dp 18))
    LkPark
    LkCrop "$tag-30-name-$align" ($r.L - 10) ($r.T - 10) ($r.R - $r.L + 20) (Dp 64)
    Check "$tag the name box of a $align title opens" $ok ''
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Pf settings set titleAlign left | Out-Null; Start-Sleep -Milliseconds 700
  $d = DumpOf '季度报告'
  $i = [array]::IndexOf(@($d.tabs), '资料夹')
  $r = Rect (FWin '季度报告')
  if ($i -ge 0) {
    $tx = [int]($d.l + $d.ox + ($d.rects[$i][0] + $d.rects[$i][1] / 2))
    $ok = LkRename $tx ($r.T + (Dp 18))
    LkPark
    LkCrop "$tag-32-name-tab" ($r.L - 10) ($r.T - 10) ($r.R - $r.L + 20) (Dp 64)
    Check "$tag the name box of a tab with a glyph opens" $ok "tab $i at x=$tx"
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Pf settings set tabBar left | Out-Null; Start-Sleep -Seconds 2
  $d = DumpOf '季度报告'
  $i = [array]::IndexOf(@($d.tabs), '资料夹')
  $r = Rect (FWin '季度报告')
  if ($i -ge 0) {
    $rowY = [int]($d.t + $d.oy + $d.rects[$i][0] + $d.rects[$i][1] / 2)
    LkPark
    LkCrop "$tag-33-rail" ($r.L - 10) ($r.T - 10) (Dp 240) (Dp 170)
    $ok = LkRename ($r.L + (Dp 60)) $rowY
    LkPark
    LkCrop "$tag-34-name-rail" ($r.L - 10) ($r.T - 10) (Dp 240) (Dp 170)
    Check "$tag the name box of a row in the left tab column opens" $ok "row at y=$rowY"
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Pf settings set tabBar top | Out-Null; Start-Sleep -Seconds 2

  # The guide while 右边 is held level with 左边, its fade at the release, the landing outline.
  $l = Rect (FWin '左边'); $r = Rect (FWin '右边')
  $gx = $r.L + (Dp 120); $gy = $r.T + (Dp 18)
  $tx = $l.R + 40 + (Dp 120); $ty = $l.T + (Dp 18) + 2
  [G]::MoveTo($gx, $gy); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  Glide $gx $gy $tx $ty 20 25; Start-Sleep -Milliseconds 700
  $g = @([W]::Tops('PecoFence.DragGuide')).Count
  LkCrop "$tag-40-guide" ($l.R - 90) ($l.T - 40) 220 100
  Check "$tag a guide shows while the fence is held level" ($g -ge 1) "guides=$g"
  RecStart "$tag-guide-release" -X ($l.R - 90) -Y ($l.T - 40) -W 220 -H 100 -Fps 30
  Start-Sleep -Milliseconds 200; RecMark '松开'; MUp; Start-Sleep -Milliseconds 500
  RecStop
  Pf fence move 右边 --x ([int]($r.L / $U)) --y ([int]($r.T / $U)) | Out-Null; Start-Sleep -Milliseconds 800
  [G]::MoveTo($gx, $gy); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  Glide $gx $gy ($l.L + (Dp 120)) ($l.T + (Dp 90)) 20 25; Start-Sleep -Milliseconds 700
  $o = @([W]::Tops('PecoFence.DropPreview') | ForEach-Object { Rect $_ }) | Select-Object -First 1
  LkCrop "$tag-41-outline" ($l.L - 30) ($l.T - 30) ($l.R - $l.L + 200) ($l.B - $l.T + 60)
  Check "$tag the landing outline shows over another fence" ([bool]$o) ''
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 700
  Pf fence move 右边 --x ([int]($r.L / $U)) --y ([int]($r.T / $U)) | Out-Null; Start-Sleep -Milliseconds 600
  LkPark

  # The note: its link under the pointer; a check box under the pointer while it is edited.
  $n = Rect (FWin '便签')
  [G]::MoveTo($n.L + (Dp 60), $n.T + (Dp 47)); Start-Sleep -Milliseconds 800
  LkCrop "$tag-50-note-link-hover" ($n.L - 10) ($n.T - 10) ($n.R - $n.L + 20) ($n.B - $n.T + 20)
  [G]::Click($n.L + (Dp 200), $n.T + (Dp 130)); Start-Sleep -Milliseconds 900
  [G]::MoveTo($n.L + (Dp 21), $n.T + (Dp 69)); Start-Sleep -Milliseconds 800
  LkCrop "$tag-51-note-edit-box-hover" ($n.L - 10) ($n.T - 10) ($n.R - $n.L + 20) ($n.B - $n.T + 20)
  [G]::Press(0x1B); Start-Sleep -Milliseconds 600
  LkPark

  # The Peek search box with a list.
  PeekKey; Start-Sleep -Milliseconds 1200
  [G]::Type('b'); Start-Sleep -Milliseconds 1500
  Shot "$tag-60-peek-search"
  $box = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value)
  Check "$tag the Peek search box is open" ($box -ne [IntPtr]::Zero -and [G]::IsWindowVisible($box)) ''
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500; [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  if (([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0) { [G]::Press(0x1B); Start-Sleep -Milliseconds 900 }

  if ($withSettings) {
    Pf settings open-ui | Out-Null
    $h = WaitWin 'PecoFence.Settings' 30
    Start-Sleep -Seconds 6
    if ($h -ne [IntPtr]::Zero) {
      $w = Rect $h
      LkCrop "$tag-70-settings" $w.L $w.T ($w.R - $w.L) ($w.B - $w.T)
      # The pointer on a row of the navigation.
      [G]::MoveTo($w.L + (Dp 90), $w.T + (Dp 200)); Start-Sleep -Milliseconds 700
      LkCrop "$tag-71-settings-nav-hover" $w.L $w.T (Dp 320) (Dp 420)
      [G]::Click($w.L + (Dp 400), $w.T + (Dp 20)); Start-Sleep -Milliseconds 300
      [G]::KeyDown(0x12); [G]::Press(0x73); [G]::KeyUp(0x12); Start-Sleep -Milliseconds 1200
    }
    Check "$tag the settings window opened" ($h -ne [IntPtr]::Zero) ''
    LkPark
  }
}

Case 'setup'
CloseWindows
LkClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
# An RDP session shows only an outline while a window is dragged: show the window itself.
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
GreyWallpaper
foreach ($n in $LkFiles) { DeskFile $n | Out-Null }
$sd = "$PfRun\scripts"; $folder = "$PfRun\资料夹"; $recent = "$PfRun\最近的报告"
New-Item -ItemType Directory -Force $sd, $folder, $recent | Out-Null
Set-Content -LiteralPath "$folder\计划书.txt" 'x'
foreach ($k in 1..8) { $p = "$recent\报告 第 $k 期.txt"; Set-Content -LiteralPath $p -Value 'x'; (Get-Item -LiteralPath $p).LastWriteTime = (Get-Date).AddDays(-$k) }
$utf8 = New-Object Text.UTF8Encoding($true)
[IO.File]::WriteAllText("$sd\状态.ps1", "Write-Output `"处理器 12 %`"`r`nWrite-Output `"打开项目主页 | href=https://example.com`"`r`nWrite-Output `"---`"`r`nWrite-Output `"[x] 续费域名`"`r`n", $utf8)
PfStart -Lang 'zh-CN' -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 3
ClearPfNotices
ClearFences | Out-Null
Pf settings set featureTips false | Out-Null
Pf settings set rollUp.switchTabOnHover false | Out-Null
Pf settings set theme light | Out-Null
Pf settings set themeStyle fluent | Out-Null
Pf fence set inbox desktopLayout false | Out-Null
Pf fence move inbox --rect '1340,330,240,200' | Out-Null
New-Item -ItemType Directory -Force "$PfApp\config\themes" | Out-Null
foreach ($t in 'warm', 'frosted-glass', 'rounded') { if ($PfExtra -and (Test-Path "$PfExtra\$t.json")) { Copy-Item "$PfExtra\$t.json" "$PfApp\config\themes\$t.json" -Force } }
# Fence rects are given in screen pixels here; if the CLI takes them in DIPs, divide.
Pf fence create --title 季度报告 --rect '40,70,440,230' | Out-Null
Start-Sleep -Milliseconds 800
$probe = Rect (FWin '季度报告')
$U = if ([Math]::Abs(($probe.R - $probe.L) - 440) -le 2) { 1.0 } else { $S }
function LkRect($x, $y, $w, $h) { return "$([int]($x / $U)),$([int]($y / $U)),$([int]($w / $U)),$([int]($h / $U))" }
Log "scale $Scale, rect unit divisor $U, probe width $($probe.R - $probe.L)"
if ($U -ne 1.0) { Pf fence move 季度报告 --rect (LkRect 40 70 440 230) | Out-Null }
Pf fence create --title 学习 --rect (LkRect 40 620 300 200) | Out-Null
$fid = (PfJson fence create --portal $folder --rect (LkRect 400 620 300 200)).fence.id
Pf fence rename $fid 资料夹 | Out-Null
Pf fence merge $fid --into 季度报告 | Out-Null
Pf fence merge 学习 --into 季度报告 | Out-Null
Pf item move (LkPath 0) (LkPath 1) (LkPath 2) --to 季度报告 | Out-Null
Pf fence create --tool note --title 便签 --text "https://example.com/docs 文档`n[x] 续费域名`n[ ] 给安娜打电话`n今天要做的事" --rect (LkRect 520 70 320 230) | Out-Null
Pf fence create --tool clock --title 时钟 --rect (LkRect 880 70 240 150) | Out-Null
Pf fence create --title 抽屉 --rect (LkRect 1180 300 340 220) | Out-Null
Pf fence create --title 清单 --rect (LkRect 40 340 440 230) | Out-Null
Pf item move (LkPath 6) (LkPath 7) --to 清单 | Out-Null
Pf fence set 清单 layout details | Out-Null
Pf fence create --script "$sd\状态.ps1" --title 脚本 --rect (LkRect 520 340 320 200) | Out-Null
Pf fence create --title 最近报告 --smart-folder $recent --rect (LkRect 880 260 260 230) | Out-Null
Pf fence set 最近报告 smartLimit 3 | Out-Null
Pf fence create --title 色调 --rect (LkRect 40 610 300 200) | Out-Null
Pf item move (LkPath 3) (LkPath 4) --to 色调 | Out-Null
Pf fence set 色调 tint '"#ff8800"' | Out-Null
Pf fence set 色调 opacity clear | Out-Null
Pf fence set 色调 sort name | Out-Null
Pf fence create --title 收起 --rect (LkRect 380 610 260 160) | Out-Null
Pf item move (LkPath 5) --to 收起 | Out-Null
Pf fence roll 收起 | Out-Null
Pf fence create --title 左边 --rect (LkRect 700 600 280 200) | Out-Null
Pf fence create --title 右边 --rect (LkRect 1060 690 260 150) | Out-Null
Start-Sleep -Seconds 2
Pf fence dock-top 抽屉 | Out-Null
# The first tab shown.
$d = DumpOf '季度报告'
$i = [array]::IndexOf(@($d.tabs), '季度报告')
if ($i -ge 0) { [G]::Click([int]($d.l + $d.ox + ($d.rects[$i][0] + $d.rects[$i][1] / 2)), [int]($d.t + $d.oy + (Dp 18))); Start-Sleep -Milliseconds 700 }
Start-Sleep -Seconds 3
LkPark
Check 'the scene is there' (@(PfJson fence list | Where-Object { $_ -and $_.id }).Count -ge 11) "fences=$(@(PfJson fence list | Where-Object { $_ -and $_.id }).Count)"

$seenModes = @{}
foreach ($look in ($Look -split ',')) {
  $p = $look -split '-'
  $style = $p[0]; $mode = $p[1]; $wall = $p[2]
  Case "look $look"
  LkWall $wall
  Pf settings set theme $mode | Out-Null
  switch ($style) {
    'fluent' { Pf settings set customTheme null | Out-Null; Pf settings set themeStyle fluent | Out-Null }
    'glass' { Pf settings set customTheme null | Out-Null; Pf settings set themeStyle liquidGlass | Out-Null }
    'warm' { Pf settings set customTheme warm --string | Out-Null }
    'frosted' { Pf settings set customTheme frosted-glass --string | Out-Null }
    'rounded' { Pf settings set customTheme rounded --string | Out-Null }
  }
  # PecoFence looks at the wallpaper every 5 seconds.
  Start-Sleep -Seconds 8
  $tag = if ($Scale -ne 100) { "$look-$Scale" } else { $look }
  $withSettings = -not $seenModes.ContainsKey($mode)
  $seenModes[$mode] = $true
  LkShots $tag $withSettings
}

Case 'cleanup'
[G]::Press(0x1B)
Pf settings set customTheme null | Out-Null
[Anim]::Set($AnimWas)
if (PfAlive) { PfQuit }
LkClean
GreyWallpaper
