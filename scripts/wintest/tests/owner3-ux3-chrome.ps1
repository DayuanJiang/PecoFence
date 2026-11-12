# Owner round 2026-11-12, third batch, package ux3-chrome: the fence's chrome (title strip, tab
# strip, edges, shadow, drag feedback). Parts (run.sh --var Part=...):
#   G  the alignment guides fade out at the release (they vanished in one frame); looks in
#      light and dark mode, the landing outline
#   K  a fence at the top edge of the screen has square top corners, round again when it leaves
#   T  a tab pulled out of its fence is drawn from its first frame
#   F  a folder fence that is a tab: its name fades in with the icons of the subfolder
#   N  the name box of a centred and a right-aligned title and of a tab with a glyph opens on
#      the name
#   B  measures: the bar under the shown tab against a Chinese caption, where a tab's caption
#      and a single title start
# Default: all parts. Runs on a 1600x900 box at 100 %, Fluent, light mode, grey wallpaper.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'GKTFNB' }

function Sx($d, $x) { return [int]($d.l + $d.ox + $x) }
function TitleY($d) { return [int]($d.t + $d.oy + 18) }
function TabX($d, $i) { return (Sx $d ($d.rects[$i][0] + $d.rects[$i][1] / 2)) }
function FWin($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function LastLog($pattern) { return ((PfLogAll | Select-String -Pattern $pattern | Select-Object -Last 1) -replace '^\S+\s+', '') }
function SmallInbox() {
  Pf fence set inbox desktopLayout false | Out-Null
  Pf fence move inbox --x 1300 --y 500 | Out-Null
  Pf fence resize inbox --w 280 --h 300 | Out-Null
}
# Brightness (0..255) of one screen pixel.
function Lum($x, $y) { $b = [Px]::Grab($x, $y, 1, 1); $c = MeanRgb $b; $b.Dispose(); return [int](($c[0] + $c[1] + $c[2]) / 3) }
function SaveCrop($name, $x, $y, $w, $h) { $b = Crop $name $x $y $w $h; $b.Dispose() }
# The box of the dark pixels (text) and the rows of the saturated pixels (the accent bar) in a
# screen rectangle: @(left, top, right, bottom, barTop, barBottom), -1 where there is none.
function InkBox($x, $y, $w, $h) {
  $b = [Px]::Grab($x, $y, $w, $h); $px = [Px]::Pixels($b); $b.Dispose()
  $l = -1; $t = -1; $r = -1; $bt = -1; $at = -1; $ab = -1
  for ($j = 0; $j -lt $h; $j++) {
    for ($i = 0; $i -lt $w; $i++) {
      $c = $px[$j * $w + $i]; $cr = ($c -shr 16) -band 255; $cg = ($c -shr 8) -band 255; $cb = $c -band 255
      $mx = [Math]::Max($cr, [Math]::Max($cg, $cb)); $mn = [Math]::Min($cr, [Math]::Min($cg, $cb))
      if ($mx - $mn -gt 90) { if ($at -lt 0) { $at = $j }; $ab = $j }
      elseif ($mx -lt 100) { if ($l -lt 0 -or $i -lt $l) { $l = $i }; if ($i -gt $r) { $r = $i }; if ($t -lt 0) { $t = $j }; $bt = $j }
    }
  }
  return @($l, $t, $r, $bt, $at, $ab)
}

function PartG() {
  trap { PfTrap $_; continue }
  Case 'G guides and the landing outline fade'
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 左边 --rect '300,200,300,220' | Out-Null
  Pf fence create --title 右边 --rect '700,460,300,200' | Out-Null
  Start-Sleep -Seconds 1
  ParkMouse 1200 820
  # 右边 held level with 左边's top, 40 px to its right: a guide runs along the two top edges.
  [G]::MoveTo(850, 478); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  Glide 850 478 790 220 20 25; Start-Sleep -Milliseconds 700
  $g = @([W]::Tops('PecoFence.DragGuide') | ForEach-Object { Rect $_ } | Where-Object { ($_.R - $_.L) -gt ($_.B - $_.T) }) | Select-Object -First 1
  $held = Rect (FWin '右边')
  Check 'G a guide runs along the two top edges while the fence is held level' ($g -and $held.T -eq 200) "guide $(if ($g) { "$($g.L),$($g.T),$($g.R),$($g.B)" }); 右边 at $($held.L),$($held.T)"
  SaveCrop 'G-guide-light' 560 170 130 70
  # The white line between the two fences (bare wallpaper), frame by frame; the button is let
  # go after the first frame.
  $gy = if ($g) { $g.T + 1 } else { 200 }
  [Px]::Burst(606, $gy, 28, 1, 500, 3)
  $vals = @([Px]::Frames | ForEach-Object { (MeanRgb $_)[0] })
  $first = $vals[0]; $last = $vals[-1]
  $part = @($vals | Where-Object { $_ -lt $first - 10 -and $_ -gt $last + 10 }).Count
  Check 'G the line is on screen while held and gone after the release' ($first - $last -ge 60) "brightness $first held, $last after"
  Check 'G the guide fades out (frames that show it part-way)' ($part -ge 1) "$part of $($vals.Count) frames part-way; brightness per frame: $(($vals | Select-Object -First 40) -join ' ')"
  Start-Sleep -Milliseconds 600

  # The same drag for the report: in, held, let go.
  Pf fence move 右边 --rect '700,460,300,200' | Out-Null; Start-Sleep -Milliseconds 700
  RecStart 'G-guides' -X 280 -Y 150 -W 760 -H 380 -Fps 30
  [G]::MoveTo(850, 478); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  RecMark '拖动'; Glide 850 478 790 221 24 25; Start-Sleep -Milliseconds 500
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 600
  RecStop

  # The landing outline: 右边 held over 左边.
  Pf fence move 右边 --rect '700,460,300,200' | Out-Null; Start-Sleep -Milliseconds 700
  RecStart 'G-outline' -X 180 -Y 150 -W 900 -H 560 -Fps 30
  [G]::MoveTo(850, 478); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  RecMark '拖到另一个栅栏上'; Glide 850 478 520 330 20 25; Start-Sleep -Milliseconds 600
  $o = @([W]::Tops('PecoFence.DropPreview') | ForEach-Object { Rect $_ }) | Select-Object -First 1
  Check 'G the landing outline shows while the fence is held over another' ([bool]$o) "outline $(if ($o) { "$($o.L),$($o.T),$($o.R),$($o.B)" })"
  if ($o) { SaveCrop 'G-outline-light' ($o.L - 20) ($o.T - 20) 200 120 }
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 700
  RecStop

  # Dark mode: the same two looks.
  Pf settings set theme dark | Out-Null; Start-Sleep -Seconds 2
  Pf fence move 右边 --rect '700,460,300,200' | Out-Null; Start-Sleep -Milliseconds 700
  [G]::MoveTo(850, 478); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  Glide 850 478 790 220 20 25; Start-Sleep -Milliseconds 700
  SaveCrop 'G-guide-dark' 560 170 130 70
  Glide 790 220 520 330 12 25; Start-Sleep -Milliseconds 600
  $o = @([W]::Tops('PecoFence.DropPreview') | ForEach-Object { Rect $_ }) | Select-Object -First 1
  if ($o) { SaveCrop 'G-outline-dark' ($o.L - 20) ($o.T - 20) 200 120 }
  MUp; Start-Sleep -Milliseconds 700
  Pf settings set theme light | Out-Null; Start-Sleep -Seconds 2

  # With Windows' animations off the guide still goes at once.
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  Pf fence move 右边 --rect '700,460,300,200' | Out-Null; Start-Sleep -Milliseconds 700
  [G]::MoveTo(850, 478); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  Glide 850 478 790 220 20 25; Start-Sleep -Milliseconds 700
  $g = @([W]::Tops('PecoFence.DragGuide') | ForEach-Object { Rect $_ } | Where-Object { ($_.R - $_.L) -gt ($_.B - $_.T) }) | Select-Object -First 1
  $gy = if ($g) { $g.T + 1 } else { 200 }
  [Px]::Burst(606, $gy, 28, 1, 400, 3)
  $vals = @([Px]::Frames | ForEach-Object { (MeanRgb $_)[0] })
  $first = $vals[0]; $last = $vals[-1]
  $part = @($vals | Where-Object { $_ -lt $first - 10 -and $_ -gt $last + 10 }).Count
  Check 'G with animations off the guide shows and goes at once' ($first - $last -ge 60 -and $part -eq 0) "$part of $($vals.Count) frames part-way; brightness per frame: $(($vals | Select-Object -First 20) -join ' ')"
  [Anim]::Set($true); Start-Sleep -Milliseconds 600
}

function PartK() {
  trap { PfTrap $_; continue }
  Case 'K a fence at the top edge has square top corners'
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 抽屉 --rect '700,300,320,240' | Out-Null
  Start-Sleep -Seconds 1
  ParkMouse 1200 820
  Pf fence dock-top 抽屉 | Out-Null; Start-Sleep -Milliseconds 1800
  $r = Rect (FWin '抽屉')
  # One pixel inside each top corner against the plate further along the top edge.
  $cl = Lum ($r.L + 1) ($r.T + 1); $cr = Lum ($r.R - 2) ($r.T + 1); $mid = Lum ($r.L + 40) ($r.T + 1)
  Check 'K docked: the fence stands rolled up at the top edge' ($r.T -eq 0 -and ($r.B - $r.T) -lt 60) "top $($r.T), height $($r.B - $r.T)"
  Check 'K docked: both top corners are filled by the plate (square)' ([Math]::Abs($cl - $mid) -le 25 -and [Math]::Abs($cr - $mid) -le 25) "brightness: left corner $cl, right corner $cr, plate $mid"
  SaveCrop 'K-docked-left' ($r.L - 14) 0 60 50
  SaveCrop 'K-docked' ($r.L - 20) 0 ($r.R - $r.L + 40) 60
  Pf settings set themeStyle liquidGlass | Out-Null; Start-Sleep -Seconds 2
  SaveCrop 'K-docked-glass' ($r.L - 20) 0 ($r.R - $r.L + 40) 60
  Pf settings set themeStyle fluent | Out-Null; Start-Sleep -Seconds 2
  # Away from the edge the corners are round again: the wallpaper shows in them.
  Pf fence move 抽屉 --x 700 --y 140 | Out-Null; Start-Sleep -Milliseconds 1200
  $r = Rect (FWin '抽屉')
  $cl = Lum ($r.L + 1) ($r.T + 1); $cr = Lum ($r.R - 2) ($r.T + 1); $mid = Lum ($r.L + 40) ($r.T + 2)
  Check 'K moved away: the top corners are round again' ($r.T -eq 140 -and [Math]::Abs($cl - $mid) -ge 40 -and [Math]::Abs($cr - $mid) -ge 40) "top $($r.T); brightness: left corner $cl, right corner $cr, plate $mid"
  SaveCrop 'K-away' ($r.L - 20) ($r.T - 14) ($r.R - $r.L + 40) 60
  # Dragged back up against the edge by its title: square again.
  DragTo ($r.L + 100) ($r.T + 18) ($r.L + 100) 14 12
  Start-Sleep -Milliseconds 600; ParkMouse 1200 820
  $r = Rect (FWin '抽屉')
  $cl = Lum ($r.L + 1) ($r.T + 1); $mid = Lum ($r.L + 40) ($r.T + 1)
  Check 'K dragged up against the edge: square again' ($r.T -eq 0 -and [Math]::Abs($cl - $mid) -le 25) "top $($r.T); brightness: left corner $cl, plate $mid"
}

function PartT() {
  trap { PfTrap $_; continue }
  Case 'T a tab pulled out is drawn from its first frame'
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 工作 --rect '60,60,420,260' | Out-Null
  Pf fence create --title 图片素材 --rect '60,400,300,200' | Out-Null
  Pf fence merge 图片素材 --into 工作 | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1100 760; Start-Sleep -Milliseconds 400
  $d = DumpOf '工作'
  $i = [array]::IndexOf(@($d.tabs), '图片素材')
  $x0 = TabX $d $i; $y0 = TitleY $d
  RecStart 'T-tear' -X 40 -Y 40 -W 760 -H 560 -Fps 30
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 250; MDown; Start-Sleep -Milliseconds 150
  RecMark '把标签拖出来'
  Glide $x0 $y0 ($x0 + 80) ($y0 + 140) 12 30; Start-Sleep -Milliseconds 500
  Glide ($x0 + 80) ($y0 + 140) ($x0 + 300) ($y0 + 300) 10 30; Start-Sleep -Milliseconds 300
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 800
  RecStop
  $out = FenceGet '图片素材'
  Check 'T the tab is a fence of its own' (-not $out.tabHost) "host=$($out.tabHost)"
  $line = LastLog 'fade in ended at once'
  Check 'T the fence in the hand stands at full strength at once (no fade-in)' ([bool]$line) "$line"
}

function PartF() {
  trap { PfTrap $_; continue }
  Case 'F a folder tab: its name fades in with the icons'
  $root = "$PfRun\标题文件夹"
  New-Item -ItemType Directory -Force "$root\子文件夹甲乙丙丁" | Out-Null
  Set-Content -LiteralPath "$root\计划书.txt" 'x'; Set-Content -LiteralPath "$root\子文件夹甲乙丙丁\里面的文件.txt" 'x'
  ClearFences | Out-Null; SmallInbox
  $fid = (PfJson fence create --portal $root --rect '60,60,480,330').fence.id
  Pf fence rename $fid 资料 | Out-Null
  Pf fence create --title 普通 --rect '620,60,300,220' | Out-Null
  Pf fence merge 普通 --into $fid | Out-Null
  Start-Sleep -Seconds 2
  $d = DumpOf '资料'
  $i = [array]::IndexOf(@($d.tabs), '资料')
  [G]::Click((TabX $d $i), (TitleY $d)); Start-Sleep -Milliseconds 700
  DeskClick 1100 760; Start-Sleep -Milliseconds 300
  $p = ItemAt $fid '子文件夹甲乙丙丁'
  RecStart 'F-tab-title' -X 60 -Y 60 -W 480 -H 140 -Fps 30
  Start-Sleep -Milliseconds 300
  RecMark '进入子文件夹'
  DblClick $p[0] $p[1]; Start-Sleep -Milliseconds 900
  RecStop
  $d2 = DumpOf '普通'
  Check 'F the folder tab went into the subfolder' (@($d2.tabs) -contains '子文件夹甲乙丙丁') "tabs: $($d2.tabs -join ', ')"
  $line = LastLog 'folder navigation: the title fades in'
  Check 'F the tab''s name fades in with the icons' ($line -match 'tabs=2') "$line"
  # The longer name makes the pill wider: the next tab stands at its new place at once (it
  # slid there over 167 ms, across the end of the new name).
  $line = LastLog 'a tab got wider'
  Check 'F the next tab does not slide across the longer name' ([bool]$line) "$line"
  Shot 'F-after'
}

function NameBoxAt($what) {
  $h = WaitWin 'PecoFence.Rename' 5
  if ($h -eq [IntPtr]::Zero) { return $null }
  Start-Sleep -Milliseconds 300
  return (Rect $h)
}

function PartN() {
  trap { PfTrap $_; continue }
  Case 'N the name box opens on the name'
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 季度报告 --rect '200,200,420,220' | Out-Null
  Start-Sleep -Seconds 1
  foreach ($align in 'center', 'right') {
    Pf settings set titleAlign $align | Out-Null; Start-Sleep -Milliseconds 900
    ParkMouse 1200 820
    $r = Rect (FWin '季度报告')
    # The title's text before: its dark pixels in the title row.
    $t = InkBox ($r.L + 8) ($r.T + 4) ($r.R - $r.L - 48) 28
    SaveCrop "N-$align-title" $r.L $r.T ($r.R - $r.L) 40
    [G]::RClick($r.L + 30, $r.T + 18); Start-Sleep -Milliseconds 900
    MenuPick '^重命名' (MenuWin 5) | Out-Null
    $box = NameBoxAt $align
    Check "N ${align}: the name box opens" ($box -ne $null) ''
    if ($box) {
      # The same text in the box (selected: light on the accent colour, so take the selection's
      # run of saturated pixels and the box's frame).
      SaveCrop "N-$align-box" $r.L $r.T ($r.R - $r.L) 40
      $sel = InkBox $box.L $box.T ($box.R - $box.L) ($box.B - $box.T)
      $titleL = $r.L + 8 + $t[0]; $titleR = $r.L + 8 + $t[2]
      $inside = ($box.L -le $titleL -and $box.R -ge $titleR -and $box.L -ge $r.L -and $box.R -le $r.R)
      Check "N ${align}: the box lies on the title's text, inside the fence" $inside "title text $titleL to $titleR; box $($box.L) to $($box.R); fence $($r.L) to $($r.R)"
      $mid = ($titleL + $titleR) / 2
      $off = if ($align -eq 'center') { [Math]::Abs((($box.L + $box.R) / 2) - $mid) } else { [Math]::Abs(($box.R - 4) - $titleR) }
      Check "N ${align}: the text in the box stands where the title stood" ($off -le 3) "offset $off px (title text $titleL to $titleR, box $($box.L) to $($box.R))"
    }
    [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  }
  Pf settings set titleAlign left | Out-Null; Start-Sleep -Milliseconds 600

  # A folder tab (a glyph before its caption): the box opens on the caption, not on the first tab.
  $root = "$PfRun\名称文件夹"
  New-Item -ItemType Directory -Force $root | Out-Null
  Set-Content -LiteralPath "$root\计划书.txt" 'x'
  $fid = (PfJson fence create --portal $root --rect '200,480,420,220').fence.id
  Pf fence rename $fid 资料夹 | Out-Null
  Pf fence merge $fid --into 季度报告 | Out-Null
  Start-Sleep -Seconds 2
  $d = DumpOf '季度报告'
  $i = [array]::IndexOf(@($d.tabs), '资料夹')
  $tx = Sx $d $d.rects[$i][0]; $tw = [int]$d.rects[$i][1]
  [G]::Click((TabX $d $i), (TitleY $d)); Start-Sleep -Milliseconds 700
  ParkMouse 1200 820
  $r = Rect (FWin '季度报告')
  SaveCrop 'N-tab-title' $r.L $r.T ($r.R - $r.L) 40
  # The caption's text: the dark pixels right of the glyph (the glyph is 16 px wide).
  $t = InkBox ($tx + 26) ($r.T + 4) ($tw - 30) 28
  [G]::RClick((TabX $d $i), (TitleY $d)); Start-Sleep -Milliseconds 900
  MenuPick '^重命名' (MenuWin 5) | Out-Null
  $box = NameBoxAt 'tab'
  Check 'N folder tab: the name box opens' ($box -ne $null) ''
  if ($box) {
    SaveCrop 'N-tab-box' $r.L $r.T ($r.R - $r.L) 40
    $capL = $tx + 26 + $t[0]
    Check 'N folder tab: the box starts at the caption, after the glyph' ([Math]::Abs(($box.L + 4) - $capL) -le 3 -and $box.L -gt $tx) "caption text from $capL; box from $($box.L); tab pill from $tx"
  }
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
}

function PartB() {
  trap { PfTrap $_; continue }
  Case 'B measures of the tab strip'
  ClearFences | Out-Null; SmallInbox
  Pf fence create --title 页签甲乙丙丁 --rect '60,60,420,220' | Out-Null
  Pf fence create --title 页签乙 --rect '60,400,300,200' | Out-Null
  Pf fence create --title 单独的标题 --rect '560,60,300,200' | Out-Null
  Pf fence merge 页签乙 --into 页签甲乙丙丁 | Out-Null
  Start-Sleep -Seconds 2
  $d = DumpOf '页签甲乙丙丁'
  $i = [array]::IndexOf(@($d.tabs), '页签甲乙丙丁')
  [G]::Click((TabX $d $i), (TitleY $d)); Start-Sleep -Milliseconds 700
  ParkMouse 1200 820; Start-Sleep -Milliseconds 500
  $d = DumpOf '页签甲乙丙丁'
  $tx = Sx $d $d.rects[$i][0]; $tw = [int]$d.rects[$i][1]
  $top = [int]($d.t + $d.oy)
  $b = InkBox $tx $top $tw 36
  $gap = $b[4] - $b[3] - 1
  SaveCrop 'B-tab' ([int]$d.l) $top 300 40
  Check 'B the bar under the shown tab keeps 2 px or more under a Chinese caption' ($b[4] -gt 0 -and $gap -ge 2) "caption rows $($b[1]) to $($b[3]), bar rows $($b[4]) to $($b[5]) of the 36 px title row: $gap px between"
  $capX = $tx + $b[0] - [int]$d.l
  $r = Rect (FWin '单独的标题')
  $t = InkBox $r.L $r.T 200 36
  Check 'B a tab''s caption starts where a single title starts' ([Math]::Abs($capX - $t[0]) -le 1) "tab caption $capX px from the fence's edge (pill from $($tx - [int]$d.l), $tw wide), single title $($t[0]) px"
}

# Pictures of the chrome for a look (no checks): the scrollbar at rest, under the pointer and at
# the end, a locked fence, a narrow fence, a rolled fence with a long title, in light and dark.
function PartH() {
  trap { PfTrap $_; continue }
  Case 'H pictures of the chrome'
  ClearFences | Out-Null; SmallInbox
  $files = @(1..18 | ForEach-Object { DeskFile "ux3 文件 $_ 的名字.txt" })
  Pf fence create --title 很多文件的栅栏 --rect '80,80,340,260' | Out-Null
  Pf item move @($files) --to 很多文件的栅栏 | Out-Null
  Pf fence create --title 锁住的栅栏 --rect '480,80,300,200' | Out-Null
  Pf fence set 锁住的栅栏 locked true | Out-Null
  Pf fence create --title 一个名字很长很长很长的栅栏标题放不下 --rect '840,80,200,160' | Out-Null
  Pf fence create --title 收起来的栅栏名字也很长很长很长 --rect '840,300,260,200' | Out-Null
  Pf fence roll 收起来的栅栏名字也很长很长很长 | Out-Null
  Start-Sleep -Seconds 2
  foreach ($mode in 'light', 'dark') {
    Pf settings set theme $mode | Out-Null; Start-Sleep -Seconds 2
    ParkMouse 1200 820
    SaveCrop "H-$mode-rest" 60 60 1060 460
    $r = Rect (FWin '很多文件的栅栏')
    [G]::MoveTo($r.L + 150, $r.T + 150); Start-Sleep -Milliseconds 700
    SaveCrop "H-$mode-scroll-hover" ($r.R - 60) ($r.T) 80 ($r.B - $r.T + 10)
    [G]::MoveTo($r.R - 6, $r.T + 100); Start-Sleep -Milliseconds 700
    SaveCrop "H-$mode-scroll-on-bar" ($r.R - 60) ($r.T) 80 ($r.B - $r.T + 10)
    Wheel ($r.L + 150) ($r.T + 150) -30; Start-Sleep -Milliseconds 1200
    SaveCrop "H-$mode-scroll-end" ($r.R - 60) ($r.T) 80 ($r.B - $r.T + 10)
    Wheel ($r.L + 150) ($r.T + 150) 30; Start-Sleep -Milliseconds 900
    $l = Rect (FWin '锁住的栅栏')
    [G]::MoveTo($l.L + 100, $l.T + 120); Start-Sleep -Milliseconds 700
    SaveCrop "H-$mode-lock-body" $l.L $l.T ($l.R - $l.L) 44
    [G]::MoveTo($l.L + 100, $l.T + 18); Start-Sleep -Milliseconds 700
    SaveCrop "H-$mode-lock-title" $l.L $l.T ($l.R - $l.L) 44
    [G]::MoveTo($l.R - 2, $l.T + 100); Start-Sleep -Milliseconds 500
    SaveCrop "H-$mode-lock-edge" ($l.R - 60) ($l.T + 60) 80 80
  }
  Pf settings set theme light | Out-Null
  foreach ($f in $files) { Remove-Item -LiteralPath $f -Force -ErrorAction SilentlyContinue }
}

Case 'setup'
CloseWindows
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
# An RDP session shows only an outline while a window is dragged: show the window itself.
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
GreyWallpaper
. C:\pf\kit\rec.ps1
PfStart -Script (DumpScript) -Lang 'zh-CN'
Start-Sleep -Seconds 3
Pf settings set theme light | Out-Null
Pf settings set rollUp.switchTabOnHover false | Out-Null
if ($Part -match 'G') { PartG }
if ($Part -match 'K') { PartK }
if ($Part -match 'T') { PartT }
if ($Part -match 'F') { PartF }
if ($Part -match 'N') { PartN }
if ($Part -match 'B') { PartB }
if ($Part -match 'H') { PartH }

Case 'end'
[Anim]::Set($AnimWas)
if (PfAlive) { PfQuit }
