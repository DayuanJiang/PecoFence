# Fence polish round 2026-11-12, package "rename" (findings MC2 / V3, C2, S3, UX5, MF-6 b).
# Parts (run.sh --var Part=...):
#   I  icons while renaming: a renamed folder and shortcut keep their picture (C2), a new folder
#      shows its icon while its name box is open (UX5)
#   B  the item rename box (MC2 / V3): inside the cell, as tall as the wrapped name, the name
#      under it hidden, grows and shrinks while typing, Enter / Ctrl+Enter / Esc, list rows
#   F  the name box of a fence just made waits for the fence's fade-in (MF-6 b)
#   S  the box never shows more lines than the name under the icon (also at 125 / 150 %: run
#      it in a scaled RDP session with --var Part=S --var Zoom=150; its points are scaled)
#   W  the box is as wide as the name is drawn: a name fitted to the wider box (one line
#      in place of two) gets a box that wide, others and a fence that scrolls the usual one
# Default: all parts.
if (-not $Part) { $Part = 'IBFWS' }
trap { PfTrap $_; continue }
$PrNames = @('a1 报告.txt', 'b2 预算表.txt', '04 新年活动安排与分工.txt', 'd4 设计稿 v3 final.txt', 'e5 合同扫描件.txt')
function PrClean() {
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -notmatch '^EC2 ' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
}
function PrKey($vk, $ms = 300) { [G]::Press($vk); Start-Sleep -Milliseconds $ms }
function PrRenameWin() {
  $h = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
  if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { return $h }
  return [IntPtr]::Zero
}
function PrWaitRename($sec = 3) {
  for ($i = 0; $i -lt $sec * 20; $i++) { $h = PrRenameWin; if ($h -ne [IntPtr]::Zero) { return $h }; Start-Sleep -Milliseconds 50 }
  return [IntPtr]::Zero
}
# A number an EDIT control answers (EM_GETLINECOUNT 0xBA, EM_GETFIRSTVISIBLELINE 0xCE) for the
# rename box's edit.
function PrEdit($box, $msg) {
  $e = [M]::FindWindowExW($box, [IntPtr]::Zero, 'EDIT', [NullString]::Value)
  $r = [IntPtr]::Zero
  [G]::SendMessageTimeoutW($e, $msg, [IntPtr]0, [IntPtr]0, 2, 2000, [ref]$r) | Out-Null
  return [int]$r
}
# Largest mean brightness difference between a reference crop and the frames of a burst.
function PrMaxDiff($ref) {
  $m = 0.0
  foreach ($f in [Px]::Frames) { $d = [Px]::Diff($ref, $f, 0, $ref.Width - 1, 0, $ref.Height - 1); if ($d -gt $m) { $m = $d } }
  return [math]::Round($m, 1)
}

Case 'setup'
CloseWindows
PrClean
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
foreach ($n in $PrNames) { DeskFile $n | Out-Null }
$PrDir = Join-Path $PfDesk '项目资料'
New-Item -ItemType Directory -Force $PrDir | Out-Null
Set-Content -LiteralPath (Join-Path $PrDir '说明.txt') -Value 'x' -Encoding ASCII
$PrLnk = DeskLink 'Notepad link.lnk'
PfStart -Script @('sleep 3600000') -Lang 'zh-CN'
Start-Sleep -Seconds 4
ClearFences | Out-Null
ClearPfNotices

if ($Part -match 'I') {
  Case 'UX5 a new folder shows its icon while its name box is open'
  # In a fence of text files (no folder picture is cached yet), with Windows' animations off
  # and on: with them off nothing redrew the fence after the icon request was dropped.
  Pf fence create --title Fresh --rect '300,120,420,300' | Out-Null
  Pf item move (Join-Path $PfDesk 'b2 预算表.txt') (Join-Path $PfDesk 'e5 合同扫描件.txt') --to Fresh | Out-Null
  ParkMouse; Start-Sleep -Seconds 3
  foreach ($anim in $false, $true) {
    [Anim]::Set($anim); Start-Sleep -Milliseconds 800
    $f = FenceGet 'Fresh'
    # An empty spot of the fence takes the keyboard for it.
    [G]::Click($f.windowRect.x + $f.windowRect.w - 30, $f.windowRect.y + $f.windowRect.h - 30); Start-Sleep -Milliseconds 400
    [G]::KeyDown(0x11); [G]::KeyDown(0x10); [G]::Press(0x4E); [G]::KeyUp(0x10); [G]::KeyUp(0x11)
    $box = PrWaitRename 5
    Start-Sleep -Milliseconds 1500
    $order = @(@(PfJson item list --fence Fresh) | ForEach-Object { $_.name })
    $ni = $order.Count - 1
    $np = CellPt 'Fresh' $ni
    $new = [Px]::Grab($np[0] - 20, $np[1] - 20, 40, 40); $new.Save("$PfShots\ux5-new-folder-anim-$anim.png")
    # The folder picture is yellow; the loading tile under the selection fill is grey-blue.
    $warm = 0; foreach ($c in [Px]::Pixels($new)) { if (((($c -shr 16) -band 255) - ($c -band 255)) -gt 60) { $warm++ } }
    Shot "ux5-name-box-open-anim-$anim"
    Check "UX5 the new folder has its icon while the name box is open (animations $(if ($anim) { 'on' } else { 'off' }))" ($box -ne [IntPtr]::Zero -and $order[$ni] -match '新建文件夹' -and $warm -gt 400) "yellow pixels in the 40x40 icon box 1.5 s after Ctrl+Shift+N: $warm of 1600; name box open: $($box -ne [IntPtr]::Zero); items: $($order -join ' | ')"
    PrKey 0x1B 600
  }
  Pf fence delete Fresh | Out-Null
  Start-Sleep -Milliseconds 800

  Case 'C2 a renamed folder and shortcut keep their picture'
  Pf fence create --title Icons --rect '300,120,420,300' | Out-Null
  Pf item move $PrDir $PrLnk (Join-Path $PfDesk 'a1 报告.txt') --to Icons | Out-Null
  ParkMouse; Start-Sleep -Seconds 4
  foreach ($t in @(@('项目资料', '项目资料 2026', 'folder'), @('Notepad link', 'Notepad renamed', 'shortcut'))) {
    $order = @(@(PfJson item list --fence Icons) | ForEach-Object { $_.name })
    $i = [array]::IndexOf($order, $t[0])
    $p = CellPt 'Icons' $i
    ParkMouse; Start-Sleep -Milliseconds 500
    $ref = [Px]::Grab($p[0] - 20, $p[1] - 20, 40, 40)
    $ref.Save("$PfShots\c2-$($t[2])-before.png")
    [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 400
    PrKey 0x71 600
    $box = PrWaitRename
    [G]::Type($t[1]); Start-Sleep -Milliseconds 300
    # The pointer stays on the icon: its hover fill is in the reference too.
    $ref = [Px]::Grab($p[0] - 20, $p[1] - 20, 40, 40)
    [G]::Press(0x0D)
    [Px]::Burst($p[0] - 20, $p[1] - 20, 40, 40, 1200, 0)
    [Px]::Strip("$PfShots\c2-$($t[2])-strip.png", 2)
    $max = PrMaxDiff $ref
    $now = @(@(PfJson item list --fence Icons) | ForEach-Object { $_.name })
    Check "C2 the $($t[2])'s icon stays while it is renamed" ($box -ne [IntPtr]::Zero -and $now -contains $t[1] -and $max -lt 12) "largest change of the icon box in 1.2 s after Enter: $max (frames $([Px]::Frames.Count)); items: $($now -join ' | ')"
  }

  Pf fence delete Icons | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'B') {
  Case 'MC2 the rename box of a wrapped name'
  Pf fence create --title Box --rect '300,120,420,300' | Out-Null
  Pf item move @($PrNames | ForEach-Object { Join-Path $PfDesk $_ }) --to Box | Out-Null
  ParkMouse; Start-Sleep -Seconds 3
  $order = @(@(PfJson item list --fence Box) | ForEach-Object { $_.name })
  $i = [array]::IndexOf($order, '04 新年活动安排与分工')
  $p = CellPt 'Box' $i
  $f = FenceGet 'Box'
  $pitch = $f.windowRect.w / [int]$f.fit.columns
  $cellL = [int]($p[0] - $pitch / 2); $cellR = [int]($p[0] + $pitch / 2)
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse; Start-Sleep -Milliseconds 300
  PrKey 0x71 700
  $box = PrWaitRename
  $r = Rect $box
  Shot 'mc2-box-open'
  $crop = [Px]::Grab($cellL - 30, $p[1] - 40, [int]$pitch + 60, 150); $crop.Save("$PfShots\mc2-box-crop.png")
  Log "MC2 box $($r.L),$($r.T),$($r.R),$($r.B) cell $cellL..$cellR icon centre $($p -join ',')"
  # The box is the label's text width plus its 4 px frame: with a narrower label inset the frame
  # may stand a few pixels over the cell's edge (the old box stood out 6 px at 100 %).
  Check 'MC2 the box stays inside its cell' ($box -ne [IntPtr]::Zero -and $r.L -ge $cellL - 4 -and $r.R -le $cellR + 4) "box x $($r.L)..$($r.R), cell x $cellL..$cellR"
  Check 'MC2 the box is as tall as the wrapped name (2 lines or more)' (($r.B - $r.T) -ge 34) "box height $($r.B - $r.T) px"
  Check 'MC2 the box is centred under the icon' ([math]::Abs((($r.L + $r.R) / 2) - $p[0]) -le 2) "box centre $(($r.L + $r.R) / 2), icon centre $($p[0])"
  # Nothing of the old name shows around the box: the label rows of the cell below the box.
  $below = [Px]::Grab($cellL + 4, $r.B + 1, ($cellR - $cellL) - 8, 30)
  $below.Save("$PfShots\mc2-below-box.png")
  $ink = [Px]::MaxInk($below, 0, $below.Width - 1, 0, $below.Height - 1)
  Check 'MC2 the old name does not show under the box' ($ink -lt 40) "ink below the box: $ink"
  # The start of the name shows: the edit's first visible line is its first line.
  $first = PrEdit $box 0xCE
  $lines = PrEdit $box 0xBA
  Check 'MC2 the name starts at the top of the box' ($first -eq 0 -and $lines -ge 2) "first visible line $first, lines $lines"
  $h0 = $r.B - $r.T
  [G]::Type('一个比原来长得多的新名字用来看输入框会不会跟着变高'); Start-Sleep -Milliseconds 500
  $r1 = Rect $box; $h1 = $r1.B - $r1.T
  Shot 'mc2-box-long'
  Check 'MC2 the box grows with a longer name' ($h1 -gt $h0 -and $r1.L -eq $r.L -and $r1.R -eq $r.R) "height $h0 -> $h1 px, x $($r1.L)..$($r1.R)"
  [G]::KeyDown(0x11); [G]::Press(0x41); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 150
  [G]::Type('短名'); Start-Sleep -Milliseconds 500
  $r2 = Rect $box; $h2 = $r2.B - $r2.T
  Check 'MC2 the box shrinks to one line for a short name' ($h2 -lt $h0 -and $h2 -ge 16) "height $h1 -> $h2 px"
  # Ctrl+Enter must not put a line break into a file name; Enter commits.
  [G]::KeyDown(0x11); [G]::Press(0x0D); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 300
  $still = PrRenameWin
  if ($still -ne [IntPtr]::Zero) { PrKey 0x0D 300 }
  Start-Sleep -Milliseconds 1200
  $now = @(@(PfJson item list --fence Box) | ForEach-Object { $_.name })
  Check 'MC2 Enter renames the file to the typed name' ((Test-Path -LiteralPath (Join-Path $PfDesk '短名.txt')) -and (PrRenameWin) -eq [IntPtr]::Zero) "items: $($now -join ' | ')"
  # Esc keeps the old name.
  $i = [array]::IndexOf($now, 'a1 报告'); $p = CellPt 'Box' $i
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  PrKey 0x71 600
  [G]::Type('不要这个名字'); Start-Sleep -Milliseconds 200
  PrKey 0x1B 700
  Check 'MC2 Esc keeps the old name' ((Test-Path -LiteralPath (Join-Path $PfDesk 'a1 报告.txt')) -and (PrRenameWin) -eq [IntPtr]::Zero) "a1 still there: $(Test-Path -LiteralPath (Join-Path $PfDesk 'a1 报告.txt'))"
  # A short name: one line, still inside the cell.
  PrKey 0x71 600
  $box = PrWaitRename; $r = Rect $box
  Shot 'mc2-box-short'
  Check 'MC2 a short name gets a one-line box inside the cell' ($box -ne [IntPtr]::Zero -and ($r.B - $r.T) -le 26 -and ($r.R - $r.L) -le ($cellR - $cellL)) "box $($r.R - $r.L) x $($r.B - $r.T) px"
  PrKey 0x1B 500

  Case 'MC2 list rows keep a one-line box'
  Pf fence set Box layout list | Out-Null
  Start-Sleep -Milliseconds 1200
  $f = FenceGet 'Box'
  [G]::Click($f.windowRect.x + 60, $f.windowRect.y + 36 + 14); Start-Sleep -Milliseconds 500
  PrKey 0x71 700
  $box = PrWaitRename; $r = Rect $box
  Shot 'mc2-row-box'
  $lines = PrEdit $box 0xBA
  Check 'MC2 a list row gets a one-line box' ($box -ne [IntPtr]::Zero -and $lines -eq 1 -and ($r.B - $r.T) -le 28) "box $($r.R - $r.L) x $($r.B - $r.T) px, lines $lines"
  PrKey 0x1B 500
  Pf fence delete Box | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'W') {
  Case 'follow-up: the box is as wide as the name is drawn'
  DeskFile 'README.md' | Out-Null
  foreach ($k in 1..9) { DeskFile "文件 $k 说明.txt" | Out-Null }
  Start-Sleep -Seconds 2
  # A fence that does not scroll: 「README.md」 is drawn on one line in the wider box.
  Pf fence create --title Wide --rect '300,120,420,300' | Out-Null
  Pf item move (Join-Path $PfDesk 'README.md') (Join-Path $PfDesk 'a1 报告.txt') (Join-Path $PfDesk 'b2 预算表.txt') --to Wide | Out-Null
  ParkMouse; Start-Sleep -Seconds 3
  $order = @(@(PfJson item list --fence Wide) | ForEach-Object { $_.name })
  $i = 0; foreach ($k in 0..($order.Count - 1)) { if ($order[$k] -match '^README') { $i = $k } }
  $p = CellPt 'Wide' $i
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse; Start-Sleep -Milliseconds 300
  # The name as drawn: one line of ink under the icon, none on the second label line.
  $second = [Px]::Grab($p[0] - 36, $p[1] + 46, 72, 12)
  $ink2 = [Px]::MaxInk($second, 0, 71, 0, 11)
  PrKey 0x71 700
  $box = PrWaitRename; $r = Rect $box
  $lines = PrEdit $box 0xBA
  Shot 'fu-wide-box'
  Check 'follow-up: a name drawn on one line in the wider box gets a one-line box that wide' ($box -ne [IntPtr]::Zero -and $ink2 -lt 40 -and $lines -eq 1 -and ($r.R - $r.L) -ge 80) "name '$($order[$i])': ink on the second label line $ink2, box $($r.R - $r.L) x $($r.B - $r.T) px, lines $lines"
  $w0 = $r.R - $r.L; $h0 = $r.B - $r.T
  [G]::Type('README 的中文说明和更新记录'); Start-Sleep -Milliseconds 500
  $r1 = Rect $box
  Check 'follow-up: the box keeps that width while a longer name is typed' (($r1.R - $r1.L) -eq $w0 -and ($r1.B - $r1.T) -gt $h0) "box $w0 x $h0 -> $($r1.R - $r1.L) x $($r1.B - $r1.T) px"
  PrKey 0x1B 600
  # A short name in the same fence is drawn in the usual box: so is its rename box.
  $i = [array]::IndexOf($order, 'a1 报告'); $p = CellPt 'Wide' $i
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  PrKey 0x71 700
  $box = PrWaitRename; $r = Rect $box
  Check 'follow-up: a name drawn in the usual box gets the usual box' ($box -ne [IntPtr]::Zero -and ($r.R - $r.L) -le 74) "box $($r.R - $r.L) x $($r.B - $r.T) px"
  PrKey 0x1B 600
  Pf fence delete Wide | Out-Null
  Start-Sleep -Milliseconds 800
  # A fence that scrolls draws no name in the wider box: 「README.md」 takes two lines there.
  Pf fence create --title Scrolls --rect '300,120,420,150' | Out-Null
  $move = @((Join-Path $PfDesk 'README.md'), (Join-Path $PfDesk 'a1 报告.txt'), (Join-Path $PfDesk 'b2 预算表.txt')) + @(1..9 | ForEach-Object { Join-Path $PfDesk "文件 $_ 说明.txt" })
  Pf item move $move --to Scrolls | Out-Null
  ParkMouse; Start-Sleep -Seconds 3
  $f = FenceGet 'Scrolls'
  $order = @(@(PfJson item list --fence Scrolls) | ForEach-Object { $_.name })
  $i = 0; foreach ($k in 0..($order.Count - 1)) { if ($order[$k] -match '^README') { $i = $k } }
  $p = CellPt 'Scrolls' $i
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  PrKey 0x71 700
  $box = PrWaitRename; $r = Rect $box
  $lines = PrEdit $box 0xBA
  Shot 'fu-scrolling-box'
  # (Two lines as a name there; the box may fit it on one: its edit has a little slack.)
  Check 'follow-up: in a fence that scrolls the same name gets the usual box, no wider one' ($box -ne [IntPtr]::Zero -and [bool]$f.fit.overflow -and $i -lt 5 -and ($r.R - $r.L) -le 74 -and $lines -ge 1 -and $lines -le 2) "fence scrolls: $($f.fit.overflow); name '$($order[$i])' at index $i; box $($r.R - $r.L) x $($r.B - $r.T) px, lines $lines"
  PrKey 0x1B 600
  Pf fence delete Scrolls | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'S') {
  Case 'follow-up: the box never shows more lines than the name'
  # Names whose lines are nearly full under the icon: the edit control (GDI) lays text out a
  # little wider than the label (DirectWrite), most at 150 %.
  $Zf = if ($Zoom) { ([double]$Zoom) / 100.0 } else { 1.0 }
  Log "LINES zoom var '$Zoom' -> factor $Zf"
  $sNames = @('a1 新年活动安排与分工.txt', '04 新年活动安排与分工.txt', 'd4 设计稿 v3 final.txt', 'README.md', '2026-11-11 会议记录 v2.txt',
    'Quarterly report 2026 final v3.txt', '项目计划书 第二版 修订.txt', 'IMG_20261111_083000.txt', 'budget_FY27_rev-B.txt', 'mmmm wwww MMMM.txt')
  foreach ($n in $sNames) { DeskFile $n | Out-Null }
  Start-Sleep -Seconds 2
  $fw = [int](800 * $Zf); $fh = [int](260 * $Zf); $fx = [int](300 * $Zf); $fy = [int](120 * $Zf)
  Pf fence create --title Lines --rect "$fx,$fy,$fw,$fh" | Out-Null
  Pf item move @($sNames | ForEach-Object { Join-Path $PfDesk $_ }) --to Lines | Out-Null
  ParkMouse; Start-Sleep -Seconds 3
  $f = FenceGet 'Lines'; $wr = $f.windowRect
  $order = @(@(PfJson item list --fence Lines) | ForEach-Object { $_.name })
  $cols = [int]$f.fit.columns; $pitch = $wr.w / $cols
  $lh = [int](16 * $Zf); $worse = @(); $seen = 0
  for ($i = 0; $i -lt [math]::Min($order.Count, $cols); $i++) {
    $cx = [int]($wr.x + $i * $pitch + $pitch / 2); $cy = [int]($wr.y + 76 * $Zf)
    [G]::Click($cx, $cy); Start-Sleep -Milliseconds 500
    ParkMouse; Start-Sleep -Milliseconds 400
    # Lines of the name as drawn (the selected icon shows its whole name): label line slots with ink.
    $top = [int]($wr.y + 104 * $Zf)
    $b = [Px]::Grab([int]($cx - $pitch / 2 + 2), $top, [int]$pitch - 4, 6 * $lh)
    $nameLines = 0
    # A line of text inks a third of its slot's rows or more; the plate's bottom edge only one or two.
    foreach ($k in 0..5) {
      $rows = 0
      foreach ($y in ($k * $lh + 1)..(($k + 1) * $lh - 2)) { if ([Px]::MaxInk($b, 0, $b.Width - 1, $y, $y) -ge 50) { $rows++ } }
      if ($rows * 3 -ge $lh) { $nameLines = $k + 1 }
    }
    if ($order[$i] -match '^a1 ') { Shot 'fu-lines-name-selected' }
    PrKey 0x71 700
    $box = PrWaitRename
    $boxLines = if ($box -ne [IntPtr]::Zero) { PrEdit $box 0xBA } else { -1 }
    if ($order[$i] -match '^a1 ') { Shot 'fu-lines-name-box' }
    $r = Rect $box
    Log "LINES '$($order[$i])': name $nameLines, box $boxLines (box $($r.R - $r.L) x $($r.B - $r.T) px)"
    if ($box -ne [IntPtr]::Zero) { $seen++ }
    if ($boxLines -gt $nameLines -or $boxLines -lt 1) { $worse += "'$($order[$i])' name $nameLines box $boxLines" }
    PrKey 0x1B 500
  }
  Shot 'fu-lines'
  Check "follow-up: no rename box has more lines than its name (scale $([int]($Zf * 100)) %)" ($seen -ge 8 -and $worse.Count -eq 0) "boxes opened: $seen of $($order.Count); more lines in the box: $(if ($worse.Count) { $worse -join '; ' } else { 'none' })"
  Pf fence delete Lines | Out-Null
  Start-Sleep -Milliseconds 800
}

if ($Part -match 'F') {
  Case 'MF-6 the name box of a new fence waits for the fade-in'
  ParkMouse
  $spot = FreeRect 320 220 700 300
  $x = $spot[0]; $y = $spot[1]
  # Draw the rectangle on the empty desktop, then the offer's right button (＋ 新建栅栏).
  DragTo $x $y ($x + 320) ($y + 220) 12
  $offer = [IntPtr]::Zero
  for ($i = 0; $i -lt 40 -and $offer -eq [IntPtr]::Zero; $i++) {
    $h = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value)
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $offer = $h } else { Start-Sleep -Milliseconds 100 }
  }
  $o = Rect $offer; $bx = $o.R - 30; $by = [int](($o.T + $o.B) / 2)
  Glide ($bx - 50) ($by + 30) $bx $by 8 25; Start-Sleep -Milliseconds 200
  [G]::Click($bx, $by)
  $seen = [IntPtr]::Zero
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 4000) {
    $h = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $seen = $h; break }
  }
  $ms = $sw.ElapsedMilliseconds
  # The plate right when the box shows (a strip of the content area, clear of the box and of the
  # pointer), and at rest.
  $first = [Px]::Grab($x + 20, $y + 100, 200, 60)
  $first.Save("$PfShots\mf6-plate-when-box-shows.png")
  Start-Sleep -Milliseconds 1500
  $rest = [Px]::Grab($x + 20, $y + 100, 200, 60)
  $rest.Save("$PfShots\mf6-plate-at-rest.png")
  $d = [math]::Round([Px]::Diff($first, $rest, 0, 199, 0, 59), 1)
  Shot 'mf6-new-fence'
  Check 'MF-6 the fence is fully shown when its name box appears' ($offer -ne [IntPtr]::Zero -and $seen -ne [IntPtr]::Zero -and $d -lt 2) "the plate differs from its resting look by $d when the box shows, $ms ms after the click (0 = fully faded in)"
  PrKey 0x1B 500

  Case 'follow-up: no name box over a window that covers the new fence'
  # A window comes over the new fence while it still fades in (as when a Peek ends, or the user
  # goes to another window): the topmost name box must not open on top of that window.
  $note = "$PfRun\cover.txt"; Set-Content $note 'covers the new fence' -Encoding ASCII
  $np = Start-Process notepad.exe -ArgumentList $note -PassThru
  for ($k = 0; $k -lt 50; $k++) { Start-Sleep -Milliseconds 200; $np.Refresh(); if ($np.MainWindowHandle -ne 0) { break } }
  $nh = [IntPtr]$np.MainWindowHandle
  [L]::MoveWindow($nh, 20, 560, 300, 200, $true) | Out-Null
  Start-Sleep -Milliseconds 800
  ParkMouse
  $spot = FreeRect 320 220 900 200
  $x = $spot[0]; $y = $spot[1]
  $n0 = @(PfJson fence list).Count
  DragTo $x $y ($x + 320) ($y + 220) 12
  $offer = [IntPtr]::Zero
  for ($i = 0; $i -lt 40 -and $offer -eq [IntPtr]::Zero; $i++) {
    $h = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value)
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $offer = $h } else { Start-Sleep -Milliseconds 100 }
  }
  $o = Rect $offer; $bx = $o.R - 30; $by = [int](($o.T + $o.B) / 2)
  Glide ($bx - 50) ($by + 30) $bx $by 8 25; Start-Sleep -Milliseconds 200
  [G]::Click($bx, $by)
  [L]::MoveWindow($nh, $x - 30, $y - 30, 400, 300, $true) | Out-Null
  $seen = [IntPtr]::Zero
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt 1500) {
    $h = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
    if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $seen = $h; break }
  }
  Shot 'fu-covered-new-fence'
  $n1 = @(PfJson fence list).Count
  Check 'follow-up: a new fence covered by a window while it fades in gets no name box' ($nh -ne [IntPtr]::Zero -and $offer -ne [IntPtr]::Zero -and $n1 -eq $n0 + 1 -and $seen -eq [IntPtr]::Zero) "fences $n0 -> $n1; name box over the window: $($seen -ne [IntPtr]::Zero) (after $($sw.ElapsedMilliseconds) ms)"
  if ($seen -ne [IntPtr]::Zero) { PrKey 0x1B 500 }
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 800
  Check 'follow-up: the fence stays, without a name box, when the window is gone' ((PrRenameWin) -eq [IntPtr]::Zero -and @(PfJson fence list).Count -eq $n1) "name box: $((PrRenameWin) -ne [IntPtr]::Zero)"
}

Case 'cleanup'
[Anim]::Set($AnimWas)
PfQuit
PrClean
