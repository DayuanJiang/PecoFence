# Owner round 2026-11-12, third batch, package ux3-folder: resizing a fence, scrolling in it,
# its views, the tab strip in use. Parts (run.sh --var Part=...):
#   P  Peek search: a tab that gets its match badge; the tabs after it stand at their new
#      places at once (they slid across the wider tab for 167 ms)
#   C  a fence in 详细信息 made wider gets 大小, then 修改日期, then 类型 (类型 stood in for
#      修改日期 between 328 and 355 px)
#   Z  Ctrl+wheel larger and back: the fence is scrolled where it was
#   H  recordings and pictures for a look (no checks): resize, scroll, views, tab strip
#   G  more recordings (no checks): the thumb dragged, a sort change while scrolled, files
#      removed at the end of the content, a rolled fence made wider
# Default: the parts with checks. Runs on a 1600x900 box at 100 %, Fluent, light mode, grey
# wallpaper. The test files live in folders under the job's run folder (folder fences), not
# on the desktop.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'PCZ' }

function Sx($d, $x) { return [int]($d.l + $d.ox + $x) }
function TitleY($d) { return [int]($d.t + $d.oy + 18) }
function TabX($d, $i) { return (Sx $d ($d.rects[$i][0] + $d.rects[$i][1] / 2)) }
function FWin($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function LastLog($pattern) { return ((PfLogAll | Select-String -Pattern $pattern | Select-Object -Last 1) -replace '^\S+\s+', '') }
function SmallInbox() {
  Pf fence set inbox desktopLayout false | Out-Null
  Pf fence move inbox --x 1300 --y 560 | Out-Null
  Pf fence resize inbox --w 280 --h 240 | Out-Null
}
function SaveCrop($name, $x, $y, $w, $h) { $b = Crop $name $x $y $w $h; $b.Dispose() }
# A folder with mixed-length Chinese names; returns its path.
function TestFolder($name, $count) {
  $root = "$PfRun\$name"
  New-Item -ItemType Directory -Force $root | Out-Null
  $names = @('报告', '第三季度销售数据汇总表（最终版）', '照片', 'readme', '会议纪要二零二六年十一月', '预算', 'a', '设计稿第二版修改意见',
    '合同', '说明书', '年度总结与来年计划草稿', '图标', 'notes', '发票扫描件', '备忘', '很长很长很长很长很长很长很长的文件名称用来看省略号', '清单', '数据', '日志', '脚本')
  $ext = @('.txt', '.docx', '.png', '.md', '.xlsx', '.pdf')
  for ($i = 0; $i -lt $count; $i++) {
    $n = $names[$i % $names.Count]; if ($i -ge $names.Count) { $n = "$n$([int]($i / $names.Count) + 1)" }
    Set-Content -LiteralPath "$root\$n$($ext[$i % $ext.Count])" ('x' * (($i + 1) * 137))
  }
  return $root
}
function WheelRaw($delta) {
  $d = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int]$delta), 0)
  [G]::mouse_event(0x800, 0, 0, $d, [UIntPtr]::Zero)
}

function PartP() {
  trap { PfTrap $_; continue }
  Case 'P Peek search: a tab gets its match badge'
  ClearFences | Out-Null; SmallInbox
  $a = TestFolder '搜索甲' 6; $b = TestFolder '搜索乙' 6
  $fa = (PfJson fence create --portal $a --rect '80,80,520,300').fence.id
  $fb = (PfJson fence create --portal $b --rect '700,80,300,220').fence.id
  Pf fence create --title 第三个标签 --rect '700,400,300,200' | Out-Null
  Pf fence create --title 第四个 --rect '1040,400,240,200' | Out-Null
  Pf fence merge $fb --into $fa | Out-Null
  Pf fence merge 第三个标签 --into $fa | Out-Null
  Pf fence merge 第四个 --into $fa | Out-Null
  Start-Sleep -Seconds 2
  DeskClick 1100 800; Start-Sleep -Milliseconds 400
  $d = DumpOf '第四个'
  $before = @($d.rects | ForEach-Object { [int]$_[0] }) -join ' '
  RecStart 'P-badge' -X 70 -Y 70 -W 540 -H 60 -Fps 30
  PeekKey; Start-Sleep -Milliseconds 900
  RecMark '输入要找的字'
  [G]::Type('报'); Start-Sleep -Milliseconds 1200
  RecStop
  Shot 'P-search'
  $d2 = DumpOf '第四个'
  $after = @($d2.rects | ForEach-Object { [int]$_[0] }) -join ' '
  Check 'P the tabs with matches got wider (a badge)' ($before -ne $after) "tab x before: $before; with badges: $after"
  $line = LastLog 'a tab got its match badge'
  Check 'P the tabs after a badge stand at their new places at once' ([bool]$line) "$line"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Milliseconds 600
}

# The columns a details fence shows at a width, read from its rows by Windows' OCR: the dates
# (11/12/2026) and the type names (PNG File, DOCX File).
function ColsAt($fid, $w, $crop) {
  Pf fence resize $fid --w $w --h 330 | Out-Null; Start-Sleep -Milliseconds 700
  SaveCrop $crop 190 110 ($w + 20) 200
  $t = (OcrText 200 156 $w 280) -replace '\s+', ' '
  return [pscustomobject]@{ date = ($t -match '\d+/\d+/20\d\d'); type = ($t -match '(?i)\bfile\b|document'); text = $t }
}
function PartC() {
  trap { PfTrap $_; continue }
  Case 'C details: the columns of a fence made wider'
  ClearFences | Out-Null; SmallInbox
  $root = TestFolder '列夹' 12
  $fid = (PfJson fence create --portal $root --rect '200,120,300,330').fence.id
  Pf fence rename $fid 资料 | Out-Null
  Pf fence set $fid layout details | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1200 820
  $a = ColsAt $fid 300 'C-300'
  $b = ColsAt $fid 344 'C-344'
  $c = ColsAt $fid 400 'C-400'
  $d = ColsAt $fid 500 'C-500'
  Check 'C 300 px: neither 修改日期 nor 类型' (-not $a.date -and -not $a.type) "date=$($a.date) type=$($a.type)"
  Check 'C 344 px: 类型 does not stand in for 修改日期' (-not $b.date -and -not $b.type) "date=$($b.date) type=$($b.type) | $($b.text)"
  Check 'C 400 px: 修改日期, no 类型 yet' ($c.date -and -not $c.type) "date=$($c.date) type=$($c.type)"
  Check 'C 500 px: both' ($d.date -and $d.type) "date=$($d.date) type=$($d.type)"
  # For the report: the right edge dragged from 300 to 520 px.
  Pf fence resize $fid --w 300 --h 330 | Out-Null; Start-Sleep -Milliseconds 700
  RecStart 'C-wider' -X 190 -Y 110 -W 560 -H 200 -Fps 30
  [G]::MoveTo(499, 300); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '把详细信息视图的栅栏拖宽'
  Glide 499 300 720 300 44 40; Start-Sleep -Milliseconds 400
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 600
  RecStop
}

# How many pixels of a screen rectangle have the selection fill's pale blue (the plate is grey,
# the picture icons' blue is far stronger).
function SelPixels($x, $y, $w, $h) {
  $b = [Px]::Grab($x, $y, $w, $h); $px = [Px]::Pixels($b); $b.Dispose()
  $n = 0
  foreach ($c in $px) { $d = ($c -band 255) - (($c -shr 16) -band 255); if ($d -gt 22 -and $d -lt 90) { $n++ } }
  return $n
}
function PartZ() {
  trap { PfTrap $_; continue }
  Case 'Z Ctrl+wheel larger and back keeps the place'
  ClearFences | Out-Null; SmallInbox
  $root = TestFolder '缩放夹' 34
  $fid = (PfJson fence create --portal $root --rect '200,120,420,330').fence.id
  Pf fence rename $fid 资料 | Out-Null
  Start-Sleep -Seconds 3
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 300
  WheelRaw -120; Start-Sleep -Milliseconds 400; WheelRaw -120; Start-Sleep -Milliseconds 900
  # An icon of the lowest row in view is selected.
  [G]::Click(410, 388); Start-Sleep -Milliseconds 500
  $sel0 = SelPixels 200 160 400 286
  RecStart 'Z-zoom' -X 190 -Y 110 -W 440 -H 350 -Fps 30
  [G]::KeyDown(0x11); Start-Sleep -Milliseconds 100
  RecMark 'Ctrl+滚轮放大两级'; WheelRaw 120; Start-Sleep -Milliseconds 900; WheelRaw 120; Start-Sleep -Milliseconds 900
  RecMark 'Ctrl+滚轮缩回原来的大小'; WheelRaw -120; Start-Sleep -Milliseconds 900; WheelRaw -120; Start-Sleep -Milliseconds 1200
  [G]::KeyUp(0x11)
  RecStop
  $sel1 = SelPixels 200 160 400 286
  SaveCrop 'Z-after' 190 110 440 350
  $f = FenceGet $fid
  Check 'Z the fence has its height back' ($f.windowRect.h -eq 330) "h=$($f.windowRect.h)"
  Check 'Z the selected icon is in view again after larger and back' ($sel0 -gt 2000 -and $sel1 -gt 2000) "pixels of the selection fill in the content: $sel0 before, $sel1 after"
}

# Recordings for a look (no checks).
function PartH() {
  trap { PfTrap $_; continue }
  Case 'H recordings: resize, scroll, views, tab strip'
  ClearFences | Out-Null; SmallInbox
  $root = TestFolder '资料夹' 34
  $fid = (PfJson fence create --portal $root --rect '200,120,420,330').fence.id
  Pf fence rename $fid 资料 | Out-Null
  Start-Sleep -Seconds 3
  ParkMouse 1200 820
  Shot 'H0-start'
  # Resize by the lower right corner: slowly smaller down to the least size, then larger.
  RecStart 'H-resize-corner' -X 180 -Y 100 -W 620 -H 520 -Fps 30
  [G]::MoveTo(618, 448); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '拖右下角缩小'
  Glide 618 448 260 170 40 30; Start-Sleep -Milliseconds 400
  RecMark '再拖大'
  Glide 260 170 760 560 40 30; Start-Sleep -Milliseconds 400
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  Shot 'H1-after-corner'
  Pf fence resize $fid --w 420 --h 330 | Out-Null; Pf fence move $fid --x 200 --y 120 | Out-Null
  Start-Sleep -Milliseconds 800
  # Resize by the right edge only (columns come and go), fast.
  RecStart 'H-resize-edge' -X 180 -Y 100 -W 720 -H 380 -Fps 30
  [G]::MoveTo(619, 300); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '拖右边缘变宽'
  Glide 619 300 860 300 16 30; Start-Sleep -Milliseconds 300
  RecMark '拖窄'
  Glide 860 300 330 300 20 30; Start-Sleep -Milliseconds 300
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  Pf fence resize $fid --w 420 --h 330 | Out-Null; Pf fence move $fid --x 200 --y 120 | Out-Null
  Start-Sleep -Milliseconds 800
  # Scroll: wheel notches, small deltas, End / Home, a track click, the thumb dragged.
  RecStart 'H-scroll' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 300
  RecMark '滚轮向下三格'
  WheelRaw -120; Start-Sleep -Milliseconds 500; WheelRaw -120; Start-Sleep -Milliseconds 120; WheelRaw -120; Start-Sleep -Milliseconds 700
  RecMark '触摸板小步向下'
  for ($i = 0; $i -lt 12; $i++) { WheelRaw -15; Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 700
  RecMark '触摸板小步向上'
  for ($i = 0; $i -lt 12; $i++) { WheelRaw 15; Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 700
  RecMark '滚到底再多滚'
  for ($i = 0; $i -lt 10; $i++) { WheelRaw -120; Start-Sleep -Milliseconds 60 }
  Start-Sleep -Milliseconds 900
  RecMark '滚回顶再多滚'
  for ($i = 0; $i -lt 12; $i++) { WheelRaw 120; Start-Sleep -Milliseconds 60 }
  Start-Sleep -Milliseconds 900
  RecStop
  RecStart 'H-scrollbar' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  RecMark '指针停在滚动条上'
  [G]::MoveTo(614, 300); Start-Sleep -Milliseconds 900
  RecMark '点轨道下方'
  [G]::Click(614, 420); Start-Sleep -Milliseconds 900
  RecMark '拖动滑块'
  $th = 0
  for ($y = 160; $y -lt 440; $y += 4) { $bm = [Px]::Grab(614, $y, 1, 1); $c = MeanRgb $bm; $bm.Dispose(); if (($c[0] + $c[1] + $c[2]) / 3 -lt 150) { $th = $y; break } }
  Log "thumb top about y=$th"
  if ($th) { [G]::MoveTo(614, $th + 10); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 150; Glide 614 ($th + 10) 614 180 15 30; Start-Sleep -Milliseconds 300; Glide 614 180 640 440 15 30; Start-Sleep -Milliseconds 300; MUp }
  Start-Sleep -Milliseconds 700
  RecMark '指针离开'
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 1200
  RecStop
  # Keyboard: click an icon, End, Home, Page Down, arrows.
  $p = CellPt $fid 0
  RecStart 'H-keys' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  RecMark 'End'; [G]::Press(0x23); Start-Sleep -Milliseconds 800
  RecMark 'Home'; [G]::Press(0x24); Start-Sleep -Milliseconds 800
  RecMark 'Page Down'; [G]::Press(0x22); Start-Sleep -Milliseconds 800
  RecMark '向下四次'; for ($i = 0; $i -lt 4; $i++) { [G]::Press(0x28); Start-Sleep -Milliseconds 220 }
  Start-Sleep -Milliseconds 600
  RecStop
  # At the end of the content the fence is made taller until no scrollbar is needed.
  [G]::Press(0x23); Start-Sleep -Milliseconds 700
  ParkMouse 1200 820
  RecStart 'H-taller' -X 180 -Y 100 -W 460 -H 780 -Fps 30
  $r = Rect (FWin '资料')
  [G]::MoveTo([int](($r.L + $r.R) / 2), $r.B - 2); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '在内容末尾把栅栏拖高'
  Glide ([int](($r.L + $r.R) / 2)) ($r.B - 2) ([int](($r.L + $r.R) / 2)) 860 30 30; Start-Sleep -Milliseconds 400
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  Pf fence resize $fid --w 420 --h 330 | Out-Null; Pf fence move $fid --x 200 --y 120 | Out-Null
  Start-Sleep -Milliseconds 800
  # Views: icons, list, details and back; pictures of each with the pointer on a row.
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 200; WheelRaw -120; Start-Sleep -Milliseconds 700
  ParkMouse 1200 820
  RecStart 'H-views' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  RecMark '切到列表'; Pf fence set $fid layout list | Out-Null; Start-Sleep -Milliseconds 1000
  RecMark '切到详细信息'; Pf fence set $fid layout details | Out-Null; Start-Sleep -Milliseconds 1000
  RecMark '切回图标'; Pf fence set $fid layout icons | Out-Null; Start-Sleep -Milliseconds 1000
  RecStop
  Pf fence set $fid layout list | Out-Null; Start-Sleep -Milliseconds 900
  [G]::MoveTo(330, 240); Start-Sleep -Milliseconds 500; SaveCrop 'H-list-hover' 190 110 440 350
  [G]::Click(330, 266); Start-Sleep -Milliseconds 400; [G]::MoveTo(330, 300); Start-Sleep -Milliseconds 400; SaveCrop 'H-list-selected' 190 110 440 350
  Pf fence set $fid layout details | Out-Null; Start-Sleep -Milliseconds 900
  [G]::MoveTo(330, 240); Start-Sleep -Milliseconds 500; SaveCrop 'H-details-hover' 190 110 440 350
  [G]::Click(330, 266); Start-Sleep -Milliseconds 400; [G]::MoveTo(330, 300); Start-Sleep -Milliseconds 400; SaveCrop 'H-details-selected' 190 110 440 350
  # Details made narrow and wide.
  ParkMouse 1200 820
  RecStart 'H-details-resize' -X 180 -Y 100 -W 720 -H 380 -Fps 30
  [G]::MoveTo(619, 300); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '详细信息：拖窄'
  Glide 619 300 330 300 20 30; Start-Sleep -Milliseconds 300
  RecMark '拖宽'
  Glide 330 300 860 300 20 30; Start-Sleep -Milliseconds 300
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  SaveCrop 'H-details-wide' 190 110 700 350
  Pf fence resize $fid --w 420 --h 330 | Out-Null; Pf fence move $fid --x 200 --y 120 | Out-Null
  # Details scrolled: the header row.
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 300
  RecStart 'H-details-scroll' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  RecMark '详细信息里滚动'
  WheelRaw -120; Start-Sleep -Milliseconds 600; WheelRaw -360; Start-Sleep -Milliseconds 800
  RecStop
  SaveCrop 'H-details-scrolled' 190 110 440 350
  Pf fence set $fid layout icons | Out-Null; Start-Sleep -Milliseconds 900
  # Ctrl+wheel: icon size steps.
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 300
  RecStart 'H-ctrl-wheel' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  [G]::KeyDown(0x11); Start-Sleep -Milliseconds 100
  RecMark 'Ctrl+滚轮放大'; WheelRaw 120; Start-Sleep -Milliseconds 800; WheelRaw 120; Start-Sleep -Milliseconds 800
  RecMark 'Ctrl+滚轮缩小'; WheelRaw -120; Start-Sleep -Milliseconds 800; WheelRaw -120; Start-Sleep -Milliseconds 800; WheelRaw -120; Start-Sleep -Milliseconds 800
  [G]::KeyUp(0x11)
  RecStop
  Pf fence set $fid iconSize 48 | Out-Null
  # The tab strip: six tabs in a 420 px fence.
  foreach ($t in '工作文件', '图片素材', '下载', '很长的标签名字在这里', '杂项') {
    Pf fence create --title $t --rect '900,120,300,200' | Out-Null
    Pf fence merge $t --into $fid | Out-Null
  }
  Start-Sleep -Seconds 2
  ParkMouse 1200 820
  Shot 'H2-tabs'
  $d = DumpOf '杂项'
  Log "tabs: $($d.tabs -join ', ') rects: $(@($d.rects | ForEach-Object { "$([int]$_[0])+$([int]$_[1])" }) -join ' ')"
  $y0 = TitleY $d
  RecStart 'H-tabs-wheel' -X 180 -Y 100 -W 460 -H 120 -Fps 30
  [G]::MoveTo(360, $y0); Start-Sleep -Milliseconds 300
  RecMark '在标签栏上滚轮向下'
  for ($i = 0; $i -lt 5; $i++) { WheelRaw -120; Start-Sleep -Milliseconds 450 }
  RecMark '向上'
  for ($i = 0; $i -lt 5; $i++) { WheelRaw 120; Start-Sleep -Milliseconds 450 }
  RecStop
  $d = DumpOf '杂项'
  $x0 = TabX $d 0
  RecStart 'H-tabs-drag' -X 180 -Y 100 -W 460 -H 120 -Fps 30
  [G]::MoveTo($x0, $y0); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '把第一个标签拖到右边'
  Glide $x0 $y0 ($x0 + 330) $y0 40 30; Start-Sleep -Milliseconds 400
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  # The tabbed fence made narrow and wide by its right edge.
  $r = Rect (FWin (DumpOf '杂项').title)
  RecStart 'H-tabs-resize' -X 180 -Y 100 -W 720 -H 380 -Fps 30
  [G]::MoveTo($r.R - 1, 300); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '带标签的栅栏拖窄'
  Glide ($r.R - 1) 300 330 300 20 30; Start-Sleep -Milliseconds 300
  RecMark '拖宽'
  Glide 330 300 860 300 20 30; Start-Sleep -Milliseconds 300
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  # A fence dragged onto another's title to become a tab.
  Pf fence create --title 新来的 --rect '900,480,300,200' | Out-Null
  Start-Sleep -Seconds 1
  $r = Rect (FWin (DumpOf '杂项').title)
  RecStart 'H-merge' -X 180 -Y 100 -W 1060 -H 600 -Fps 30
  [G]::MoveTo(1000, 498); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '把栅栏拖到另一个的标题上'
  Glide 1000 498 ($r.R - 60) ($r.T + 18) 30 30; Start-Sleep -Milliseconds 500
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 1000
  RecStop
  Shot 'H3-merged'
  # A rolled fence resized by its edge.
  Pf fence create --title 卷起的 --rect '900,120,300,200' | Out-Null
  Pf fence set 卷起的 rolledUp true | Out-Null
  Start-Sleep -Seconds 1
  Shot 'H4-rolled'
}

# More recordings for a look (no checks): the thumb dragged, a sort change while scrolled, files
# removed at the end of the content, a rolled fence made wider.
function PartG() {
  trap { PfTrap $_; continue }
  Case 'G recordings: thumb drag, sort change, files removed, a rolled fence'
  ClearFences | Out-Null; SmallInbox
  $root = TestFolder '滚动夹' 34
  $fid = (PfJson fence create --portal $root --rect '200,120,420,330').fence.id
  Pf fence rename $fid 资料 | Out-Null
  Start-Sleep -Seconds 3
  # The thumb: the pointer rests on the bar until it is wide, then the darkest run below the
  # arrow button is the thumb.
  [G]::MoveTo(614, 300); Start-Sleep -Milliseconds 1000
  $min = 255; $th = 0
  for ($y = 176; $y -lt 430; $y += 3) { $bm = [Px]::Grab(614, $y, 1, 1); $c = MeanRgb $bm; $bm.Dispose(); $l = ($c[0] + $c[1] + $c[2]) / 3; if ($l -lt $min - 4) { $min = $l; $th = $y } }
  Log "thumb about y=$th (brightness $min)"
  RecStart 'G-thumb' -X 180 -Y 100 -W 500 -H 370 -Fps 30
  [G]::MoveTo(614, $th + 8); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '按住滑块往下拖'
  Glide 614 ($th + 8) 614 ($th + 120) 30 30; Start-Sleep -Milliseconds 300
  RecMark '指针横着离开滚动条'
  Glide 614 ($th + 120) 660 ($th + 150) 10 30; Start-Sleep -Milliseconds 300
  RecMark '拖回顶端'
  Glide 660 ($th + 150) 614 150 20 30; Start-Sleep -Milliseconds 300
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 800
  RecStop
  # A sort change while the fence is scrolled to the middle.
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 300
  WheelRaw -240; Start-Sleep -Milliseconds 900
  ParkMouse 1200 820
  RecStart 'G-sort' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  RecMark '排序方式改成类型'; Pf fence set $fid sort type | Out-Null; Start-Sleep -Milliseconds 1200
  RecMark '改回名称'; Pf fence set $fid sort name | Out-Null; Start-Sleep -Milliseconds 1200
  RecStop
  # At the end of the content most files are removed: the fence no longer needs its bar.
  [G]::MoveTo(400, 300); Start-Sleep -Milliseconds 300
  WheelRaw -1200; Start-Sleep -Milliseconds 1200
  ParkMouse 1200 820
  RecStart 'G-removed' -X 180 -Y 100 -W 460 -H 370 -Fps 30
  RecMark '在内容末尾删掉二十多个文件'
  Get-ChildItem -LiteralPath $root | Select-Object -Skip 9 | Remove-Item -Force
  Start-Sleep -Milliseconds 2500
  RecStop
  # A rolled fence made wider and narrower by its right edge.
  Pf fence create --title 卷起来的栅栏名称比较长 --rect '760,120,300,200' | Out-Null
  Pf fence roll 卷起来的栅栏名称比较长 | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1200 820
  $r = Rect (FWin '卷起来的栅栏名称比较长')
  RecStart 'G-rolled' -X 740 -Y 100 -W 520 -H 120 -Fps 30
  [G]::MoveTo($r.R - 1, $r.T + 18); Start-Sleep -Milliseconds 300; MDown; Start-Sleep -Milliseconds 150
  RecMark '卷起的栅栏拖窄'
  Glide ($r.R - 1) ($r.T + 18) ($r.L + 60) ($r.T + 18) 25 30; Start-Sleep -Milliseconds 300
  RecMark '拖宽'
  Glide ($r.L + 60) ($r.T + 18) ($r.R + 150) ($r.T + 18) 25 30; Start-Sleep -Milliseconds 300
  RecMark '松开'; MUp; Start-Sleep -Milliseconds 900
  RecStop
  ParkMouse 1200 820
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
if ($Part -match 'P') { PartP }
if ($Part -match 'C') { PartC }
if ($Part -match 'Z') { PartZ }
if ($Part -match 'H') { PartH }
if ($Part -match 'G') { PartG }

Case 'end'
[Anim]::Set($AnimWas)
if (PfAlive) { PfQuit }
# The test folders are in the job's run folder; nothing of this file is on the desktop.
Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^(搜索甲|搜索乙|资料夹|列夹|缩放夹)' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
