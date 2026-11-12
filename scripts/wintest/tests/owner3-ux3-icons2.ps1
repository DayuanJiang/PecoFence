# Owner round 2026-11-12, third batch, second pass on icons: what a user sees when icons arrive,
# are pasted, cut and pointed at.
#   run.sh --var Part=A tests/owner3-ux3-icons2.ps1    a new file: name and picture together; a pasted copy: its selection box
#   run.sh --var Part=B ...                            Ctrl+X, then Esc (as Explorer's desktop does)
#   run.sh --var Part=C ...                            the tip when the pointer comes back to the very point
#   run.sh --var Part=D ...                            the name box: F2, and a new name confirmed by a click in the fence
# Parts A and D record small regions at 30 frames a second and read the frames (rec-*).
# Real mouse and keys, zh-CN, a new configuration (「桌面」 arranged like the desktop, 自由摆放).
# Every part ends with the desktop cleaned (U2End).
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
. C:\pf\kit\rec.ps1
function U2Items() { return @(@(PfJson item list --fence inbox) | Where-Object { $_ -and $_.id }) }
function U2Cells() {
  $h = @{}; $g = FenceGet inbox
  if ($g.desktopCells) { foreach ($p in $g.desktopCells.PSObject.Properties) { $h[$p.Name] = "$($p.Value[0]),$($p.Value[1])" } }
  return $h
}
# The cell of a loose icon by file name, its top-left corner on screen and the point to press
# it at (100 %: cells of 80 x 96 px, 4 px in from the work area's corner).
function U2Cell($name) { $c = U2Cells; $it = U2Items | Where-Object { $_.fileName -eq $name -or $_.name -eq $name } | Select-Object -First 1; if ($it) { return $c["$($it.id)"] }; return $null }
function U2Corner($cell) { $p = $cell -split ','; return @([int](4 + [int]$p[0] * 80), [int](4 + [int]$p[1] * 96)) }
function U2Pt($name) { $v = U2Cell $name; if (-not $v) { return $null }; $c = U2Corner $v; return @(($c[0] + 40), ($c[1] + 30)) }
function U2Wait([scriptblock]$ok, $sec = 8) { for ($k = 0; $k -lt $sec * 5; $k++) { if (& $ok) { return $true }; Start-Sleep -Milliseconds 200 }; return $false }
$U2Names = @('季度预算.fpx', '图.fpx', '一个名字特别长的客户往来文件最终版.fpx', 'b.fpx')
function U2Clean() {
  foreach ($n in $U2Names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  Get-ChildItem -LiteralPath $PfDesk -Filter '*.fpx' -ErrorAction SilentlyContinue | Where-Object { $_.Name -match 'Copy|副本|新来的|这个' } | Remove-Item -Force -ErrorAction SilentlyContinue
}
function U2Start() {
  CloseWindows
  Get-Process OpenWith -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  U2Clean
  foreach ($n in $U2Names) { DeskFile $n | Out-Null }
  [G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
  $script:U2AnimWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
}
function U2End() { [Anim]::Set($script:U2AnimWas); U2Clean }
# The frames of a recording as bitmaps, in order.
function U2Frames($name) { return @(Get-ChildItem (Join-Path $PfShots "rec-$name") -Filter '*.jpg' | Sort-Object Name | ForEach-Object { New-Object Drawing.Bitmap $_.FullName }) }
function U2Fmt($vals) { return (($vals | ForEach-Object { '{0:N1}' -f $_ }) -join ' ') }

if ($Part -eq 'A') {
  # A new file: its name faded in at once and its picture two to three frames later (the
  # picture comes from the worker). The fade-in now waits for the picture (167 ms at most).
  Case 'A new file shows its name and its picture together'
  U2Start
  ParkMouse 1100 700
  # Where a new file lands (the first free place): made once, looked up, and removed again.
  $new = Join-Path $PfDesk '新来的文件.fpx'
  DeskFile '新来的文件.fpx' | Out-Null
  $came = U2Wait { U2Cell '新来的文件.fpx' } 8
  $c = U2Corner (U2Cell '新来的文件.fpx')
  Remove-Item -LiteralPath $new -Force
  $left = U2Wait { -not (U2Cell '新来的文件.fpx') } 8
  Start-Sleep -Milliseconds 1200
  Check 'setup: the place of a new file is known and free again' ($came -and $left) "corner $($c -join ',')"
  # The cell, recorded at 30 frames a second while the file is made again: the picture is the
  # top 56 px of the cell, the name the rest.
  RecStart 'a-new' -X $c[0] -Y $c[1] -W 80 -H 96 -Fps 30
  RecMark '一个新文件出现在桌面上'
  Start-Sleep -Milliseconds 200
  DeskFile '新来的文件.fpx' | Out-Null
  Start-Sleep -Milliseconds 1800
  RecStop
  $f = U2Frames 'a-new'
  $pic = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 14, 65, 4, 55) })
  $nam = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 2, 77, 60, 93) })
  $picEnd = $pic[-1]; $namEnd = $nam[-1]
  $alone = @(0..($f.Count - 1) | Where-Object { $nam[$_] -gt 0.2 * $namEnd -and $pic[$_] -lt 0.06 * $picEnd })
  $first = @(0..($f.Count - 1) | Where-Object { $nam[$_] -gt 0.1 * $namEnd }) | Select-Object -First 1
  $around = @($first..([Math]::Min($f.Count - 1, $first + 7)))
  $line = @(PfLogAll | Where-Object { $_ -match 'new icons fade in with their pictures' }) | Select-Object -Last 1
  Check 'setup: the new file shows in the cell' ($picEnd -gt 5 -and $namEnd -gt 2 -and (U2Cell '新来的文件.fpx')) "change of the picture's place $('{0:N1}' -f $picEnd), of the name's place $('{0:N1}' -f $namEnd)"
  Check 'no frame shows the name without the picture' ($picEnd -gt 5 -and $alone.Count -eq 0) "$($alone.Count) frames with the name alone; from the first frame with the name, name: $(U2Fmt ($around | ForEach-Object { $nam[$_] })) ; picture: $(U2Fmt ($around | ForEach-Object { $pic[$_] })) ; log: $(if ($line -match 'new icons.*') { $Matches[0] })"
  foreach ($b in $f) { $b.Dispose() }
  Remove-Item -LiteralPath $new -Force -ErrorAction SilentlyContinue
  U2Wait { -not (U2Cell '新来的文件.fpx') } 8 | Out-Null
  Start-Sleep -Milliseconds 1200

  # The same on the freely placed desktop, where the box of the copy came, went for a frame
  # and came again. The copy takes the first free place: the one the new file had.
  Case 'A copy pasted on the desktop keeps its selection box once it shows'
  $p = U2Pt '季度预算.fpx'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  Chord 0x11 0x43; Start-Sleep -Milliseconds 500
  RecStart 'a-paste-desk' -X $c[0] -Y $c[1] -W 80 -H 96 -Fps 30
  RecMark '选中桌面上的一个图标，按 Ctrl+C，再按 Ctrl+V'
  Start-Sleep -Milliseconds 200
  Chord 0x11 0x56
  Start-Sleep -Milliseconds 2000
  RecStop
  $f = U2Frames 'a-paste-desk'
  $box = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 4, 11, 8, 50) })
  $pic = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 16, 63, 6, 52) })
  $boxEnd = $box[-1]; $picEnd = $pic[-1]
  $first = @(0..($f.Count - 1) | Where-Object { $box[$_] -gt 0.3 * $boxEnd }) | Select-Object -First 1
  $gone = @($first..($f.Count - 1) | Where-Object { $box[$_] -lt 0.12 * $boxEnd })
  $around = @($first..([Math]::Min($f.Count - 1, $first + 9)))
  Check 'setup: the copy stands in the free place, selected' ($picEnd -gt 5 -and $boxEnd -gt 2) "change of the picture's place $('{0:N1}' -f $picEnd), of the box strip $('{0:N1}' -f $boxEnd)"
  Check 'the box does not go again once it shows' ($boxEnd -gt 2 -and $gone.Count -eq 0) "$($gone.Count) frames without the box after frame $first; box strip from there: $(U2Fmt ($around | ForEach-Object { $box[$_] })) ; picture: $(U2Fmt ($around | ForEach-Object { $pic[$_] }))"
  foreach ($b in $f) { $b.Dispose() }
  Get-ChildItem -LiteralPath $PfDesk -Filter '*.fpx' | Where-Object { $_.Name -match 'Copy|副本' } | Remove-Item -Force -ErrorAction SilentlyContinue
  Start-Sleep -Milliseconds 1500

  # Ctrl+C, Ctrl+V in a fence: the copy arrives selected. Its selection box was drawn only
  # when the copy's fade-in was over (167 ms) and then stood there in one frame; on the bare
  # desktop it came, went and came again. The box now fades in with the copy.
  Case 'A pasted copy shows its selection box while it fades in'
  Pf fence create --title '工作文件' --rect '560,200,420,300' | Out-Null
  Pf item move "$PfDesk\b.fpx" "$PfDesk\图.fpx" --to '工作文件' | Out-Null
  Start-Sleep -Seconds 2
  $p = ItemPt '工作文件' 'b.fpx'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  Chord 0x11 0x43; Start-Sleep -Milliseconds 500
  # The copy takes the place after the two icons. Its cell without the outer 4 px: the strip
  # left of the picture holds the box alone, the middle the picture.
  $q = CellPt '工作文件' 2
  RecStart 'a-paste' -X ($q[0] - 36) -Y ($q[1] - 28) -W 72 -H 84 -Fps 30
  RecMark '选中一个图标，按 Ctrl+C，再按 Ctrl+V'
  Start-Sleep -Milliseconds 200
  Chord 0x11 0x56
  Start-Sleep -Milliseconds 2000
  RecStop
  $f = U2Frames 'a-paste'
  $box = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 1, 8, 6, 46) })
  $pic = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 16, 55, 6, 46) })
  $boxEnd = $box[-1]; $picEnd = $pic[-1]
  $bare = @(0..($f.Count - 1) | Where-Object { $pic[$_] -gt 0.2 * $picEnd -and $box[$_] -lt 0.05 * $boxEnd })
  $partBox = @(0..($f.Count - 1) | Where-Object { $box[$_] -gt 0.15 * $boxEnd -and $box[$_] -lt 0.6 * $boxEnd })
  $first = @(0..($f.Count - 1) | Where-Object { $pic[$_] -gt 0.1 * $picEnd }) | Select-Object -First 1
  $around = @($first..([Math]::Min($f.Count - 1, $first + 9)))
  $copies = @(@(PfJson item list --fence '工作文件') | Where-Object { $_ -and $_.id })
  Check 'setup: the copy is in the fence, selected' ($copies.Count -eq 3 -and $picEnd -gt 5 -and $boxEnd -gt 3) "icons in the fence: $($copies.Count); change of the picture's place $('{0:N1}' -f $picEnd), of the box strip $('{0:N1}' -f $boxEnd)"
  Check 'no frame shows the copy without its box, and the box comes at part strength first' ($bare.Count -eq 0 -and $partBox.Count -ge 1) "$($bare.Count) frames with the picture and no box, $($partBox.Count) frames with the box at part strength; from the first frame with the picture, picture: $(U2Fmt ($around | ForEach-Object { $pic[$_] })) ; box: $(U2Fmt ($around | ForEach-Object { $box[$_] }))"
  # Once there, the box does not go again.
  $full = @(0..($f.Count - 1) | Where-Object { $box[$_] -gt 0.6 * $boxEnd }) | Select-Object -First 1
  $gone = @($full..($f.Count - 1) | Where-Object { $box[$_] -lt 0.3 * $boxEnd })
  Check 'the box does not go again once it is there' ($boxEnd -gt 3 -and $gone.Count -eq 0) "$($gone.Count) frames without the box after frame $full"
  foreach ($b in $f) { $b.Dispose() }
  Pf fence delete '工作文件' | Out-Null
  U2End
}

if ($Part -eq 'B') {
  # Ctrl+X on an icon, then Esc: the selection went and the icon stayed faded, with the file
  # still on the clipboard. Explorer's desktop on this box (measured 2026-11-12): Esc takes the
  # cut back (icon whole, clipboard empty) and the icon stays selected; a click elsewhere
  # leaves the icon faded and the file on the clipboard.
  Case 'Esc after Ctrl+X takes the cut back'
  U2Start
  Pf fence create --title '工作文件' --rect '560,200,420,300' | Out-Null
  Pf item move "$PfDesk\b.fpx" "$PfDesk\图.fpx" --to '工作文件' | Out-Null
  Start-Sleep -Seconds 2
  $p = ItemPt '工作文件' 'b.fpx'
  $r = @(($p[0] - 24), ($p[1] - 24), 48, 48)
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800
  # The pointer off the icon's picture, still in the fence.
  [G]::MoveTo(900, 440); Start-Sleep -Milliseconds 600
  $sel = [Px]::Grab($r[0], $r[1], $r[2], $r[3])
  Chord 0x11 0x58; Start-Sleep -Milliseconds 900
  $cut = [Px]::Grab($r[0], $r[1], $r[2], $r[3])
  $faded = [Px]::Diff($sel, $cut, 0, 47, 0, 47)
  $clipCut = "$((ClipFiles) -join ',') effect=$(ClipEffect)"
  Check 'setup: Ctrl+X fades the icon and puts the file on the clipboard' ($faded -gt 3 -and $clipCut -match 'b\.fpx') "the picture against the selected icon: $('{0:N1}' -f $faded); clipboard: $clipCut"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  $esc = [Px]::Grab($r[0], $r[1], $r[2], $r[3])
  $back = [Px]::Diff($sel, $esc, 0, 47, 0, 47)
  $clipEsc = @(ClipFiles)
  Shot 'B-after-esc'
  Check 'after Esc the icon is whole again and still selected' ($back -lt 1.0) "the picture against the selected icon before the cut: $('{0:N1}' -f $back) (faded: $('{0:N1}' -f $faded))"
  Check 'after Esc the clipboard holds no file' ($clipEsc.Count -eq 0) "clipboard: $($clipEsc -join ',')"
  Check 'the file is still there' (Test-Path -LiteralPath (Join-Path $PfDesk 'b.fpx')) ''
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700
  $esc2 = [Px]::Grab($r[0], $r[1], $r[2], $r[3])
  Check 'a second Esc ends the selection' ([Px]::Diff($sel, $esc2, 0, 47, 0, 47) -gt 1.0) "the picture against the selected icon: $('{0:N1}' -f ([Px]::Diff($sel, $esc2, 0, 47, 0, 47)))"

  Case 'A click elsewhere leaves the cut as it is'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800
  Chord 0x11 0x58; Start-Sleep -Milliseconds 900
  [G]::Click(900, 440); Start-Sleep -Milliseconds 900
  $plain = [Px]::Grab($r[0], $r[1], $r[2], $r[3])
  $still = [Px]::Diff($esc2, $plain, 0, 47, 0, 47)
  Check 'the icon stays faded and the file stays on the clipboard' ($still -gt 3 -and ((ClipFiles) -join ',') -match 'b\.fpx') "the picture against the whole icon: $('{0:N1}' -f $still); clipboard: $((ClipFiles) -join ',') effect=$(ClipEffect)"
  [Windows.Forms.Clipboard]::Clear()
  Pf fence delete '工作文件' | Out-Null
  U2End
}

if ($Part -eq 'D') {
  # F2 on an icon: the fence drew the icon without its name at once and the name box was
  # painted a frame later, so the name went blank for a frame. Seen best on a selected icon
  # whose long name shows in full (several lines).
  Case 'F2: the name does not go blank before the name box shows'
  U2Start
  Pf fence create --title '工作文件' --rect '560,200,420,300' | Out-Null
  Pf item move "$PfDesk\b.fpx" "$PfDesk\一个名字特别长的客户往来文件最终版.fpx" --to '工作文件' | Out-Null
  Start-Sleep -Seconds 2
  $q = CellPt '工作文件' 1
  [G]::Click($q[0], $q[1]); Start-Sleep -Milliseconds 900
  [G]::MoveTo(900, 440); Start-Sleep -Milliseconds 500
  # The place of the name under the icon, recorded at 30 frames a second across F2.
  RecStart 'd-f2' -X ($q[0] - 38) -Y ($q[1] + 28) -W 76 -H 44 -Fps 30
  RecMark '选中一个名字很长的图标，按 F2'
  Start-Sleep -Milliseconds 300
  [G]::Press(0x71)
  $box = WaitWin 'PecoFence.Rename' 5
  Start-Sleep -Milliseconds 900
  RecStop
  $f = U2Frames 'd-f2'
  $d0 = @($f | ForEach-Object { [Px]::Diff($f[0], $_, 0, 75, 0, 43) })
  $d1 = @($f | ForEach-Object { [Px]::Diff($f[-1], $_, 0, 75, 0, 43) })
  $blank = @(0..($f.Count - 1) | Where-Object { $d0[$_] -gt 6 -and $d1[$_] -gt 6 })
  $turn = @(0..($f.Count - 1) | Where-Object { $d0[$_] -gt 6 }) | Select-Object -First 1
  $around = @(([Math]::Max(0, $turn - 2))..([Math]::Min($f.Count - 1, $turn + 3)))
  Check 'setup: the name box opened over the name' ($box -ne [IntPtr]::Zero -and $d0[-1] -gt 6) "box=$($box -ne [IntPtr]::Zero); the name's place at the end against the start: $('{0:N1}' -f $d0[-1])"
  Check 'no frame shows the icon with neither its name nor the box' ($blank.Count -eq 0) "$($blank.Count) frames unlike the name and unlike the box; around the change, against the name: $(U2Fmt ($around | ForEach-Object { $d0[$_] })) ; against the box: $(U2Fmt ($around | ForEach-Object { $d1[$_] }))"
  foreach ($b in $f) { $b.Dispose() }
  [G]::Press(0x1B); Start-Sleep -Milliseconds 700

  # A new name confirmed by a click on an empty part of the fence: the icon stood at its new
  # place in the sort order in one frame (the press had begun a rubber band, and that switched
  # the motion off). After Enter it glides (250 ms).
  Case 'A new name confirmed by a click in the fence glides the icon to its new place'
  $p = CellPt '工作文件' 0
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800
  [G]::Press(0x71)
  $box = WaitWin 'PecoFence.Rename' 5
  Start-Sleep -Milliseconds 500
  [G]::Type('这个'); Start-Sleep -Milliseconds 500
  RecStart 'd-click' -X 560 -Y 236 -W 260 -H 150 -Fps 30
  RecMark '输入新名字后，点一下栅栏里的空白处'
  Start-Sleep -Milliseconds 300
  [G]::Click(900, 440); Start-Sleep -Milliseconds 1500
  RecStop
  $log = @(PfLogAll)
  $at = -1; for ($i = 0; $i -lt $log.Count; $i++) { if ($log[$i] -match 'item renamed from=.*b\.fpx') { $at = $i } }
  $next = if ($at -ge 0) { @($log[$at..($log.Count - 1)] | Where-Object { $_ -match 'replace_items animate=' }) | Select-Object -First 1 } else { '' }
  Check 'the file has its new name' (Test-Path -LiteralPath (Join-Path $PfDesk '这个.fpx')) "renamed line found=$($at -ge 0)"
  Check 'the new order is taken with motion' ($next -match 'animate=true') "$(if ($next -match 'replace_items.*') { $Matches[0] })"
  Remove-Item -LiteralPath (Join-Path $PfDesk '这个.fpx') -Force -ErrorAction SilentlyContinue
  Pf fence delete '工作文件' | Out-Null
  U2End
}

if ($Part -eq 'C') {
  # The pointer rests on an icon (the tip shows), leaves, and comes back to the very same
  # point: no tip came, because a move to the point of the move before counted as no move.
  Case 'The tip comes again when the pointer returns to the same point'
  U2Start
  $p = U2Pt '季度预算.fpx'
  $region = @(86, ($p[1] + 22), 150, 56)
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 800
  $bare = [Px]::Grab($region[0], $region[1], $region[2], $region[3])
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 1600
  $g = [Px]::Grab($region[0], $region[1], $region[2], $region[3]); $d1 = [Px]::Diff($bare, $g, 0, 149, 0, 55); $g.Dispose()
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 1200
  $g = [Px]::Grab($region[0], $region[1], $region[2], $region[3]); $d0 = [Px]::Diff($bare, $g, 0, 149, 0, 55); $g.Dispose()
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 1600
  $g = [Px]::Grab($region[0], $region[1], $region[2], $region[3]); $d2 = [Px]::Diff($bare, $g, 0, 149, 0, 55); $g.Dispose()
  Shot 'C-second-rest'
  Check 'setup: the tip shows at the first rest and goes when the pointer leaves' ($d1 -gt 3 -and $d0 -lt 1) "change of the tip's place: $('{0:N1}' -f $d1) at rest, $('{0:N1}' -f $d0) after leaving"
  Check 'the tip shows again at the second rest on the same point' ($d2 -gt 3) "change of the tip's place: $('{0:N1}' -f $d2)"
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 600
  U2End
}
