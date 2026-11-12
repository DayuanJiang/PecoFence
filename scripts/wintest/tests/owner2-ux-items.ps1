# Owner round 2026-11-12, package "ux-items": the icons and rows inside a fence.
#   T  the tip of an icon in the bottom row of a fence at the bottom of the screen stands just
#      above the pointer (it stood above the whole fence)
#   V  after a view switch the selected file is still on screen (the fence went back to the top)
#   D  a column divider dragged in the details view: no column vanishes, the divider stops
#   G  「按时间分组」 on and off: the icons glide to their places (they jumped in one frame)
#   S  Space selects the icon that only has the keyboard focus, and keeps the selection
#   L  the tip of a shortcut shows its comment or says where it leads, as File Explorer
#   K  Left and Right leave the selection alone in the list and details views
# Parts: run.sh --var Part=TV ... (default: all). --var Rec=1 also records the parts that move
# (for before / after animations). Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'TVDGSLK' }
trap { PfTrap $_; continue }
if ($Rec) { . C:\pf\kit\rec.ps1 }
function UiRec($name, $x, $y, $w, $h) { if ($Rec) { RecStart $name -X $x -Y $y -W $w -H $h -Fps 30 } }
function UiMark($t) { if ($Rec) { RecMark $t } }
function UiStop() { if ($Rec) { RecStop } }

# The folders of this test (C:\pf\ui-<name>, one per fence: a folder is shown by one fence only).
function UiClean() {
  Get-ChildItem -LiteralPath 'C:\pf' -Directory -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'ui-*' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
function UiFile($dir, $name) { Set-Content -LiteralPath (Join-Path $dir $name) -Value "ui $name" -Encoding ASCII }
$UiNames = @('季度报告.txt', '预算表.txt', '会议纪要与分工.txt', '照片合集.txt', 'Zoom 会议.txt', '报销单.txt', '合同 最终版.txt')
# A folder with the seven files, and a fence sorted by name that shows it.
function UiFence($title, $rect) {
  $dir = "C:\pf\ui-$title"
  New-Item -ItemType Directory -Force $dir | Out-Null
  foreach ($n in $UiNames) { UiFile $dir $n }
  Pf fence create --portal $dir --title $title --rect $rect | Out-Null
  Pf fence set $title sort name | Out-Null
  Start-Sleep -Seconds 3
  return $dir
}
# From a dump line of a fence: how many icons are selected, the focused one, the scroll offset.
function UiState($t) {
  $w = DumpOf $t; $f = ''
  if ($w -and $w.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { $f = $Matches[1] }
  $sc = -1.0; if ($w -and $w.raw -match ' scroll=([\d.-]+)') { $sc = [double]$Matches[1] }
  return [pscustomobject]@{ selected = "$(DumpVal $w 'selected')"; focused = $f; scroll = $sc }
}
function UiCopy() {
  [Windows.Forms.Clipboard]::Clear()
  Chord 0x11 0x43; Start-Sleep -Milliseconds 500
  return ((ClipFiles | ForEach-Object { [IO.Path]::GetFileNameWithoutExtension($_) } | Sort-Object) -join ' | ')
}
# The tooltip that is on screen: its rectangle, or $null.
function UiTip() {
  foreach ($h in @([W]::Tops('tooltips_class32'))) { $r = Rect $h; if ($r.R - $r.L -gt 20) { return $r } }
  return $null
}
# The pointer comes to rest on a point and waits for the tip.
function UiRest($x, $y, $ms = 1800) { [G]::MoveTo($x - 4, $y); Start-Sleep -Milliseconds 60; [G]::MoveTo($x, $y); Start-Sleep -Milliseconds $ms }
# How many of the frames [Px]::Burst took differ from the frame before.
function UiMoving($w, $h) {
  $n = 0
  for ($i = 1; $i -lt [Px]::Frames.Count; $i++) { if ([Px]::Diff([Px]::Frames[$i - 1], [Px]::Frames[$i], 0, $w, 0, $h) -gt 0.05) { $n++ } }
  return $n
}

Case 'setup'
CloseWindows
UiClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
GreyWallpaper
PfStart -Script (DumpScript 250 20)
Pf settings set featureTips false | Out-Null
Start-Sleep -Seconds 2
ClearFences | Out-Null
$sw = [G]::GetSystemMetrics(0); $sh = [G]::GetSystemMetrics(1)

if ($Part -match 'T') {
  Case 'T the tip of an icon in the bottom row'
  # A fence that ends 10 px above the taskbar: two rows of icons, the second at the bottom.
  $work = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  UiFence 'UiT' "700,$($work.Bottom - 250),340,240" | Out-Null
  $p = CellPt 'UiT' 5
  $py = $p[1] + 40
  UiRest $p[0] $py
  $tip = UiTip
  $top = (FenceGet 'UiT').windowRect.y
  $gap = if ($tip) { $py - $tip.B } else { 9999 }
  Crop 'T-tip-bottom-row' 640 ($work.Bottom - 420) 520 ($sh - $work.Bottom + 420) | Out-Null
  Check 'T the tip of a bottom-row icon is on screen, above the taskbar' ($tip -and $tip.B -le $work.Bottom -and $tip.T -ge 0) "tip=$(if ($tip) { "$($tip.L),$($tip.T),$($tip.R),$($tip.B)" } else { 'none' }) work bottom=$($work.Bottom)"
  Check 'T it stands just above the pointer (within 12 px), not above the fence' ($tip -and $gap -ge 0 -and $gap -le 12) "pointer y=$py, tip bottom=$(if ($tip) { $tip.B } else { 'none' }) (gap $gap px), fence top=$top"
  # An icon of the first row has room below: the tip is under the pointer as before.
  $p0 = CellPt 'UiT' 1
  UiRest $p0[0] $p0[1]
  $tip0 = UiTip
  Check 'T with room below, the tip is 20 px under the pointer' ($tip0 -and $tip0.T -eq $p0[1] + 20 -and $tip0.L -eq $p0[0] + 12) "pointer=$($p0 -join ','), tip=$(if ($tip0) { "$($tip0.L),$($tip0.T)" } else { 'none' })"
  # At the right edge of the screen the tip moves left and stays whole.
  UiFence 'UiR' "$($sw - 180),200,170,330" | Out-Null
  $pr = CellPt 'UiR' 1
  UiRest $pr[0] $pr[1]
  $tipr = UiTip
  Check 'T at the right edge the whole tip is on screen' ($tipr -and $tipr.R -le $sw -and $tipr.T -eq $pr[1] + 20) "tip=$(if ($tipr) { "$($tipr.L),$($tipr.T),$($tipr.R),$($tipr.B)" } else { 'none' }) screen width=$sw"
  ParkMouse
  Pf fence delete UiT | Out-Null; Pf fence delete UiR | Out-Null; Start-Sleep -Milliseconds 800
}

if ($Part -match 'V') {
  Case 'V the selected file after a view switch'
  $big = 'C:\pf\ui-big'
  New-Item -ItemType Directory -Force $big | Out-Null
  foreach ($i in 1..60) { UiFile $big ('b{0:D2} 文件 {0}.txt' -f $i) }
  Pf fence create --portal $big --title 'UiV' --rect '860,150,340,260' | Out-Null
  Pf fence set UiV sort name | Out-Null; Start-Sleep -Seconds 3
  $wv = (FenceGet 'UiV').windowRect
  $c0 = CellPt 'UiV' 0
  [G]::Click($c0[0], $c0[1]); Start-Sleep -Milliseconds 300
  [G]::Type('b47'); Start-Sleep -Milliseconds 900
  ParkMouse; Start-Sleep -Milliseconds 300
  $s0 = UiState 'UiV'
  Check 'V setup: file 47 of 60 is selected and the fence is scrolled to it' ($s0.selected -eq '1' -and $s0.focused -match 'b47' -and $s0.scroll -gt 800) "selected=$($s0.selected) focused=$($s0.focused) scroll=$($s0.scroll)"
  Crop 'V-1-icons' ($wv.x - 10) ($wv.y - 10) ($wv.w + 20) ($wv.h + 20) | Out-Null
  Pf fence set UiV layout list | Out-Null; Start-Sleep -Milliseconds 1200
  $s1 = UiState 'UiV'
  Crop 'V-2-list' ($wv.x - 10) ($wv.y - 10) ($wv.w + 20) ($wv.h + 20) | Out-Null
  # Row 47 of 28 px rows: on screen with the offset between about 1090 and 1300.
  Check 'V list: the selected file is on screen' ($s1.selected -eq '1' -and $s1.focused -match 'b47' -and $s1.scroll -ge 1080 -and $s1.scroll -le 1300) "selected=$($s1.selected) focused=$($s1.focused) scroll=$($s1.scroll) (0 = back at the top)"
  Pf fence set UiV layout details | Out-Null; Start-Sleep -Milliseconds 1200
  $s2 = UiState 'UiV'
  Check 'V details: the selected file is on screen' ($s2.focused -match 'b47' -and $s2.scroll -ge 1080 -and $s2.scroll -le 1330) "focused=$($s2.focused) scroll=$($s2.scroll)"
  Pf fence set UiV layout icons | Out-Null; Start-Sleep -Milliseconds 1200
  $s3 = UiState 'UiV'
  Check 'V icons again: the selected file is on screen' ($s3.focused -match 'b47' -and $s3.scroll -ge 930 -and $s3.scroll -le 1070) "focused=$($s3.focused) scroll=$($s3.scroll)"
  # Nothing selected: a view switch starts at the top, as before.
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300
  Pf fence set UiV layout list | Out-Null; Start-Sleep -Milliseconds 1200
  $s4 = UiState 'UiV'
  Check 'V nothing selected: the new view starts at the top' ($s4.selected -eq '0' -and $s4.scroll -eq 0) "selected=$($s4.selected) scroll=$($s4.scroll)"
  Pf fence delete UiV | Out-Null; Start-Sleep -Milliseconds 800
}

if ($Part -match 'D') {
  Case 'D a column divider dragged to the left'
  UiFence 'UiD' '300,150,500,330' | Out-Null
  Pf fence set UiD layout details | Out-Null; Start-Sleep -Seconds 2
  $wd = (FenceGet 'UiD').windowRect
  $hy = $wd.y + 36 + 12
  $hx = $wd.x; $hw = $wd.w
  $type0 = OcrFind '\bType\b' $hx ($wd.y + 30) $hw 40
  $date0 = OcrFind '\bDate\b' $hx ($wd.y + 30) $hw 40
  Check 'D setup: the header shows Date modified and Type' ($type0 -and $date0) "Date at $($date0 -join ','), Type at $($type0 -join ',')"
  # The divider stands 4 px left of the Date modified caption's column.
  $dx = $wd.x + 148
  UiRec 'D' ($wd.x - 20) ($wd.y - 20) ($wd.w + 40) 150
  UiMark '按住「修改日期」左边的分隔线'
  [G]::MoveTo($dx, $hy); Start-Sleep -Milliseconds 400
  MDown; Start-Sleep -Milliseconds 200
  UiMark '向左拖 60 像素'
  Glide $dx $hy ($dx - 60) $hy 20 40; Start-Sleep -Milliseconds 600
  $type1 = OcrFind '\bType\b' $hx ($wd.y + 30) $hw 40
  $date1 = OcrFind '\bDate\b' $hx ($wd.y + 30) $hw 40
  Crop 'D-dragging' ($wd.x - 20) ($wd.y - 20) ($wd.w + 40) 150 | Out-Null
  MUp; Start-Sleep -Milliseconds 800
  UiStop
  $moved = if ($date0 -and $date1) { $date0[0] - $date1[0] } else { -999 }
  Check 'D the Type column is still there while the divider is held' ([bool]$type1) "Type at $($type1 -join ',') (none = the column vanished)"
  Check 'D the divider went left with the pointer, at most 60 px, and never to the right' ($moved -ge 5 -and $moved -le 62) "Date modified moved $moved px to the left (pointer 60 px)"
  # Esc during a drag still gives the width back.
  $dx2 = $dx - $moved
  [G]::MoveTo($dx2, $hy); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 150
  Glide $dx2 $hy ($dx2 + 30) $hy 8 30; Start-Sleep -Milliseconds 300
  [G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 500
  $date2 = OcrFind '\bDate\b' $hx ($wd.y + 30) $hw 40
  Check 'D Esc gives the column its width back' ($date2 -and [math]::Abs($date2[0] - $date1[0]) -le 2) "Date modified at $($date2 -join ',') after Esc, $($date1 -join ',') before the drag"
  ParkMouse
  Pf fence delete UiD | Out-Null; Start-Sleep -Milliseconds 800
}

if ($Part -match 'G') {
  Case 'G group by date on and off'
  UiFence 'UiG' '300,150,500,330' | Out-Null
  $wg = (FenceGet 'UiG').windowRect
  $gx = $wg.x; $gy = $wg.y + 36; $gw = $wg.w; $gh = $wg.h - 36
  ParkMouse; Start-Sleep -Milliseconds 500
  UiRec 'G' ($wg.x - 20) ($wg.y - 20) ($wg.w + 40) ($wg.h + 40)
  Start-Sleep -Milliseconds 300
  UiMark '打开「按时间分组」'
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'fence', 'set', 'UiG', 'groupByDate', 'true' -WindowStyle Hidden
  [Px]::Burst($gx, $gy, $gw, $gh, 1500, 0)
  $on = UiMoving $gw $gh
  UiMark '关闭「按时间分组」'
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'fence', 'set', 'UiG', 'groupByDate', 'false' -WindowStyle Hidden
  [Px]::Burst($gx, $gy, $gw, $gh, 1500, 0)
  $off = UiMoving $gw $gh
  Start-Sleep -Milliseconds 300
  UiStop
  # Each line once: the log to share repeats the lines of the app's log.
  $lines = @(PfLogAll | Where-Object { $_ -match 'group by date changed: icons glide' } | Sort-Object -Unique)
  # A 250 ms glide is 4 to 8 of the frames a burst takes (fewer while a recording runs too).
  Check 'G on: the icons move over several frames (one frame = a jump)' ($on -ge 3) "frames that changed: $on of $([Px]::Frames.Count) taken while it went off: $off"
  Check 'G off: the icons move over several frames' ($off -ge 3) "frames that changed: $off"
  Check 'G the app logs both glides' ($lines.Count -eq 2 -and $lines[0] -match 'gliding=7') "$($lines.Count) lines; first: $(if ($lines.Count) { ($lines[0] -replace '^.*group by date', 'group by date') })"
  # With Windows' animations off it snaps.
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  Start-Process "$PfApp\pecofence-cli.exe" -ArgumentList 'fence', 'set', 'UiG', 'groupByDate', 'true' -WindowStyle Hidden
  [Px]::Burst($gx, $gy, $gw, $gh, 1200, 0)
  $snap = UiMoving $gw $gh
  Check 'G with animations off the icons jump (one frame)' ($snap -ge 1 -and $snap -le 2) "frames that changed: $snap"
  [Anim]::Set($true); Start-Sleep -Milliseconds 500
  Pf fence delete UiG | Out-Null; Start-Sleep -Milliseconds 800
}

if ($Part -match 'S') {
  Case 'S Space on the focused icon'
  UiFence 'UiS' '300,150,500,330' | Out-Null
  $ws = (FenceGet 'UiS').windowRect
  $names = @(@(PfJson item list --fence UiS) | Where-Object { $_ -and $_.name } | ForEach-Object { $_.name })
  $q0 = CellPt 'UiS' 0
  [G]::Click($q0[0], $q0[1]); Start-Sleep -Milliseconds 300
  [G]::KeyDown(0x11); [G]::Press(0x27); Start-Sleep -Milliseconds 150; [G]::Press(0x27); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 400
  ParkMouse; Start-Sleep -Milliseconds 300
  $a = UiState 'UiS'
  Crop 'S-1-focus-only' ($ws.x - 10) ($ws.y - 10) ($ws.w + 20) 160 | Out-Null
  Check 'S setup: one icon selected, the focus two icons further' ($a.selected -eq '1' -and $a.focused -eq ([IO.Path]::GetFileNameWithoutExtension($names[2]))) "selected=$($a.selected) focused=$($a.focused) (third icon: $($names[2]))"
  [G]::Press(0x20); Start-Sleep -Milliseconds 500
  $b = UiState 'UiS'
  Crop 'S-2-after-space' ($ws.x - 10) ($ws.y - 10) ($ws.w + 20) 160 | Out-Null
  $clip = UiCopy
  $want = (@($names[0], $names[2]) | ForEach-Object { [IO.Path]::GetFileNameWithoutExtension($_) } | Sort-Object) -join ' | '
  Check 'S Space selects the focused icon too (File Explorer: 2 items selected)' ($b.selected -eq '2' -and $clip -eq $want) "selected=$($b.selected); Ctrl+C copied [$clip], expected [$want]"
  # Ctrl+Space still takes it out again, and Space on a selected icon selects nothing new.
  Chord 0x11 0x20; Start-Sleep -Milliseconds 400
  $c = UiState 'UiS'
  Check 'S Ctrl+Space takes the focused icon out again' ($c.selected -eq '1') "selected=$($c.selected)"
  [G]::Click($q0[0], $q0[1]); Start-Sleep -Milliseconds 300
  [G]::Press(0x20); Start-Sleep -Milliseconds 400
  $d = UiState 'UiS'
  Check 'S Space on the selected icon leaves the selection' ($d.selected -eq '1' -and $d.focused -eq ([IO.Path]::GetFileNameWithoutExtension($names[0]))) "selected=$($d.selected) focused=$($d.focused)"
  ParkMouse
  Pf fence delete UiS | Out-Null; Start-Sleep -Milliseconds 800
}

if ($Part -match 'L') {
  Case 'L the tip of a shortcut'
  $ld = 'C:\pf\ui-UiL'
  New-Item -ItemType Directory -Force $ld | Out-Null
  $wsh = New-Object -ComObject WScript.Shell
  $la = $wsh.CreateShortcut("$ld\ui a notepad.lnk"); $la.TargetPath = "$env:WINDIR\notepad.exe"; $la.Description = 'Write short notes'; $la.Save()
  $lb = $wsh.CreateShortcut("$ld\ui b calc.lnk"); $lb.TargetPath = "$env:WINDIR\System32\calc.exe"; $lb.Save()
  UiFile $ld 'ui c 文档.txt'
  Pf fence create --portal $ld --title 'UiL' --rect '300,150,500,330' | Out-Null
  Pf fence set UiL sort name | Out-Null; Start-Sleep -Seconds 3
  $wl = (FenceGet 'UiL').windowRect
  $tips = @()
  foreach ($i in 0..2) {
    $p = CellPt 'UiL' $i
    UiRest $p[0] $p[1]
    $t = UiTip
    $text = if ($t) { OcrText ($t.L - 2) ($t.T - 2) ($t.R - $t.L + 4) ($t.B - $t.T + 4) "L-tip-ocr-$i" } else { '' }
    Crop "L-tip-$i" ($wl.x - 10) ($wl.y - 10) ($wl.w + 20) 220 | Out-Null
    $tips += , "$text"
    ParkMouse; Start-Sleep -Milliseconds 700
  }
  Check 'L a shortcut with a comment: the tip shows the comment, not the link file''s type and size' ($tips[0] -match 'Write short notes' -and $tips[0] -notmatch 'Shortcut|Size') "tip: [$($tips[0])]"
  Check 'L a shortcut without a comment: the tip says where it leads' ($tips[1] -match 'Location' -and $tips[1] -match 'calc' -and $tips[1] -match '(?i)system32' -and $tips[1] -notmatch 'Size') "tip: [$($tips[1])]"
  Check 'L a text file keeps its type, size and date' ($tips[2] -match 'Type' -and $tips[2] -match 'Size' -and $tips[2] -match 'Date') "tip: [$($tips[2])]"
  Pf fence delete UiL | Out-Null; Start-Sleep -Milliseconds 800
}

if ($Part -match 'K') {
  Case 'K Left and Right in rows'
  UiFence 'UiK' '300,150,500,330' | Out-Null
  Pf fence set UiK layout details | Out-Null; Start-Sleep -Seconds 2
  $wk = (FenceGet 'UiK').windowRect
  # The third row: 36 px title, 28 px header, 28 px rows.
  [G]::Click($wk.x + 120, $wk.y + 36 + 28 + 28 * 2 + 14); Start-Sleep -Milliseconds 400
  ParkMouse
  $k0 = UiState 'UiK'
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  $k1 = UiState 'UiK'
  [G]::Press(0x25); Start-Sleep -Milliseconds 400; [G]::Press(0x25); Start-Sleep -Milliseconds 400
  $k2 = UiState 'UiK'
  Check 'K details: Right and Left leave the focused row where it is' ($k0.focused -ne '' -and $k1.focused -eq $k0.focused -and $k2.focused -eq $k0.focused -and $k2.selected -eq '1') "clicked [$($k0.focused)], after Right [$($k1.focused)], after Left twice [$($k2.focused)]"
  [G]::Press(0x28); Start-Sleep -Milliseconds 400
  $k3 = UiState 'UiK'
  Check 'K details: Down still goes to the next row' ($k3.focused -ne $k0.focused -and $k3.selected -eq '1') "after Down [$($k3.focused)]"
  # Icons view: Right goes to the next icon as before.
  Pf fence set UiK layout icons | Out-Null; Start-Sleep -Milliseconds 1200
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  $k4 = UiState 'UiK'
  Check 'K icons: Right goes to the next icon' ($k4.focused -ne $k3.focused) "before [$($k3.focused)], after Right [$($k4.focused)]"
  Pf fence delete UiK | Out-Null; Start-Sleep -Milliseconds 800
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
UiClean
