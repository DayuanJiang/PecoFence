# Owner round 2026-11-12, second batch: the 「桌面」 fence, hiding, drag and drop.
#   run.sh --var Part=A tests/owner2-hunt-desktop.ps1     a loose icon keeps its place
# Real mouse and keys, zh-CN, a new configuration (「桌面」 arranged like the desktop, 自由摆放).
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
function HdItems() { return @(@(PfJson item list --fence inbox) | Where-Object { $_ -and $_.id }) }
function HdCells() {
  $h = @{}; $g = FenceGet inbox
  if ($g.desktopCells) { foreach ($p in $g.desktopCells.PSObject.Properties) { $h[$p.Name] = "$($p.Value[0]),$($p.Value[1])" } }
  return $h
}
# The cell of a loose icon by file name or shown name, and the point to press it at (100 %:
# cells of 80 x 96 px, 4 px in from the work area's top-left corner).
function HdCell($name) { $c = HdCells; $it = HdItems | Where-Object { $_.fileName -eq $name -or $_.name -eq $name } | Select-Object -First 1; if ($it) { return $c["$($it.id)"] }; return $null }
function HdPt($name) { $v = HdCell $name; if (-not $v) { return $null }; $p = $v -split ','; return @([int](4 + [int]$p[0] * 80 + 40), [int](4 + [int]$p[1] * 96 + 30)) }
function HdAll() { $c = HdCells; return ((HdItems | Sort-Object name | ForEach-Object { $n = if ($_.fileName) { $_.fileName } else { $_.name }; "$n=$($c["$($_.id)"])" }) -join ' ; ') }
function HdWait([scriptblock]$ok, $sec = 8) { for ($k = 0; $k -lt $sec * 5; $k++) { if (& $ok) { return $true }; Start-Sleep -Milliseconds 200 }; return $false }
$HdNames = @('季度预算.fpx', '图.fpx', '一个名字特别长的客户往来文件最终版.fpx', 'b.fpx')
function HdClean() { foreach ($n in $HdNames) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }

if ($Part -eq 'A') {
  Case 'A loose icon keeps its place when its file goes and comes back'
  HdClean
  foreach ($n in $HdNames) { DeskFile $n | Out-Null }
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  $g = FenceGet inbox
  Check 'setup: 「桌面」 places its icons freely' ($g.desktopLayout -eq $true -and $g.freePlacement -eq $true) (HdAll)
  # Dragged by hand to places away from the columns.
  $to = FreeSpot 500 300; $p = HdPt '图.fpx'
  DragTo $p[0] $p[1] $to[0] $to[1] 20; Start-Sleep -Seconds 1
  $bin = (HdItems | Where-Object { $_.kind -eq 'namespace' } | Select-Object -First 1).name
  $to = FreeSpot 700 500; $p = HdPt $bin
  DragTo $p[0] $p[1] $to[0] $to[1] 20; Start-Sleep -Seconds 1
  ParkMouse
  $want = HdCell '图.fpx'; $wantBin = HdCell $bin
  Check 'setup: the two icons stand where they were dropped' ($want -and $wantBin -and $want -notmatch '^0,' -and $wantBin -notmatch '^0,') "图=$want $bin=$wantBin ; $(HdAll)"

  # A program that saves by deleting the file and writing it again; a download saved again.
  $f = Join-Path $PfDesk '图.fpx'
  Remove-Item -LiteralPath $f -Force
  $gone = HdWait { -not (HdCell '图.fpx') -or -not (HdItems | Where-Object { $_.fileName -eq '图.fpx' }) }
  Start-Sleep -Milliseconds 1500
  Set-Content -LiteralPath $f -Value 'saved again' -Encoding ASCII
  $back = HdWait { HdItems | Where-Object { $_.fileName -eq '图.fpx' } }
  Start-Sleep -Seconds 2
  Check 'a file deleted and written again stands in the cell it was dropped in' ($back -and (HdCell '图.fpx') -eq $want) "icon gone meanwhile=$gone back=$back cell=$(HdCell '图.fpx') want=$want ; $(HdAll)"

  # Delete, then Ctrl+Z, with the keyboard in 「桌面」 the whole time.
  $p = HdPt '图.fpx'
  if ($p) { [G]::Click($p[0], $p[1]) }; Start-Sleep -Milliseconds 600
  [G]::Press(0x2E)
  $deleted = HdWait { -not (Test-Path -LiteralPath $f) } 10
  Start-Sleep -Milliseconds 1500
  $fg = Fg
  Chord 0x11 0x5A
  $restored = HdWait { Test-Path -LiteralPath $f } 10
  HdWait { HdItems | Where-Object { $_.fileName -eq '图.fpx' } } | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse
  Shot 'A-after-delete-and-undo'
  Check 'Delete then Ctrl+Z puts the icon back in its cell' ($deleted -and $restored -and (HdCell '图.fpx') -eq $want) "deleted=$deleted restored=$restored (foreground before Ctrl+Z: $fg) cell=$(HdCell '图.fpx') want=$want ; $(HdAll)"

  # Windows' own icons shown for a while (the Recycle Bin then leaves 「桌面」) and hidden again.
  Pf settings set hideRealIcons false | Out-Null
  $left = HdWait { -not (HdItems | Where-Object { $_.kind -eq 'namespace' }) }
  Start-Sleep -Seconds 2
  Pf settings set hideRealIcons true | Out-Null
  $came = HdWait { HdItems | Where-Object { $_.kind -eq 'namespace' } } 12
  Start-Sleep -Seconds 2
  Check 'the Recycle Bin is in its cell after Windows'' icons were shown and hidden again' ($came -and (HdCell $bin) -eq $wantBin) "left meanwhile=$left came back=$came cell=$(HdCell $bin) want=$wantBin ; $(HdAll)"
  HdClean
}

if ($Part -eq 'B') {
  # 「临时显示 Windows 桌面」: every fence is away and Windows' own icons show. The double-click
  # that hides and shows the fences showed them over Windows' icons (each loose icon twice),
  # left a fence marked 「双击桌面隐藏时保留此栅栏」 hidden and the mode on.
  Case 'A double-click during the temporary Windows desktop ends it'
  HdClean
  foreach ($n in $HdNames) { DeskFile $n | Out-Null }
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  $all = @(@(PfJson fence list) | Where-Object { $_ -and $_.id })
  $kept = $all | Where-Object { $_.kind -ne 'inbox' } | Select-Object -First 1
  Pf fence set $kept.id excludeFromQuickHide true | Out-Null
  $spot = FreeSpot 900 600
  Check 'setup: every fence shows and Windows'' icons are hidden' ((VisFences) -eq $all.Count -and -not [W]::IconsVisible()) "fences $(VisFences) of $($all.Count), Windows icons $([W]::IconsVisible())"
  Pf windows-desktop start | Out-Null
  $in = HdWait { (VisFences) -eq 0 -and [W]::IconsVisible() } 6
  Check 'setup: in the mode no fence shows and Windows'' icons show' $in "fences $(VisFences), Windows icons $([W]::IconsVisible())"
  DblClick $spot[0] $spot[1]
  $out = HdWait { (VisFences) -eq $all.Count -and -not [W]::IconsVisible() } 4
  Start-Sleep -Milliseconds 600
  ParkMouse
  Shot 'B-after-double-click'
  $vis = @([W]::Tops('PecoFence.Fence') | ForEach-Object { [W]::Title($_) }) -join ', '
  Check 'after the double-click every fence is back and Windows'' icons are hidden' ($out -and (VisFences) -eq $all.Count -and -not [W]::IconsVisible()) "double-click at $($spot -join ','): fences $(VisFences) of $($all.Count) [$vis] (kept on quick hide: $($kept.title)), Windows icons $([W]::IconsVisible())"
  $m = TrayMenu
  $texts = if ($m -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 300; MenuTexts $m } else { 'the tray menu did not open' }
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  Check 'the tray menu offers to hide the fences and to show the Windows desktop again' ($texts -match '^隐藏所有栅栏 \| 临时显示 Windows 桌面 ') "$texts"
  Pf windows-desktop end | Out-Null
  Pf fence show-all | Out-Null
  HdClean
}

if ($Part -eq 'C') {
  # 自由摆放: one loose icon leaves (here: dropped on a folder icon in a fence). The others keep
  # their cells, so nothing in their cells may change. Before, every icon after the one that
  # left glided into the cell of the icon before it in the list, another file's icon stood in
  # its place for a moment, and then all were back (about 250 ms).
  Case 'When a loose icon leaves, the other loose icons stay still'
  CloseWindows
  Get-Process OpenWith -ErrorAction SilentlyContinue | Stop-Process -Force
  HdClean
  $HdFolder = Join-Path $PfDesk '项目资料'
  Remove-Item -LiteralPath $HdFolder -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force $HdFolder | Out-Null
  foreach ($n in $HdNames) { DeskFile $n | Out-Null }
  [G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  $fold = @(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq '文件夹' }) | Select-Object -First 1
  $inFold = @(@(PfJson item list --fence $fold.id) | Where-Object { $_ -and $_.id })
  # Two icons away from the column, so an icon that takes another one's cell travels far.
  $to = FreeSpot 500 300; $p = HdPt '图.fpx'; DragTo $p[0] $p[1] $to[0] $to[1] 20; Start-Sleep -Seconds 1
  $to = FreeSpot 760 520; $p = HdPt '季度预算.fpx'; DragTo $p[0] $p[1] $to[0] $to[1] 20; Start-Sleep -Seconds 1
  ParkMouse
  $leaves = 'b.fpx'
  $cells = HdCells
  # Every other loose icon that has a cell (never the cell of the icon that leaves).
  $watch = @(HdItems | Where-Object { $_.fileName -ne $leaves -and $cells["$($_.id)"] -and $cells["$($_.id)"] -ne (HdCell $leaves) })
  $rects = @($watch | ForEach-Object { $c = $cells["$($_.id)"] -split ','; , @((4 + [int]$c[0] * 80), (4 + [int]$c[1] * 96)) })
  Check 'setup: the folder icon is alone in its fence and four loose icons are watched' ($inFold.Count -eq 1 -and $inFold[0].name -eq '项目资料' -and $watch.Count -ge 4) "文件夹: $(($inFold | ForEach-Object { $_.name }) -join ', '); order in 「桌面」: $((HdItems | ForEach-Object { if ($_.fileName) { $_.fileName } else { $_.name } }) -join ' | '); $(HdAll)"
  $before = HdAll
  $fp = CellPt $fold.id 0; $p = HdPt $leaves
  # Press and drag in one go (a click and a press within the double-click time open the file).
  [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  Glide $p[0] $p[1] ($p[0] + 14) ($p[1] + 14) 3 30
  Glide ($p[0] + 14) ($p[1] + 14) $fp[0] $fp[1] 20 30
  Start-Sleep -Milliseconds 700
  # The watched cells as they are while the drag rests on the folder, then as often as the
  # screen can be read for a second after the release: the largest mean brightness change of
  # any of them. One grab of the area around all of them per pass (a grab costs about 30 ms
  # here whatever its size; a grab per cell gave 8 passes in the second).
  $bx = [int]($rects | ForEach-Object { $_[0] } | Measure-Object -Minimum).Minimum; $by = [int]($rects | ForEach-Object { $_[1] } | Measure-Object -Minimum).Minimum
  $bw = [int]($rects | ForEach-Object { $_[0] } | Measure-Object -Maximum).Maximum + 80 - $bx; $bh = [int]($rects | ForEach-Object { $_[1] } | Measure-Object -Maximum).Maximum + 96 - $by
  $ref = [Px]::Grab($bx, $by, $bw, $bh)
  MUp
  $sw = [Diagnostics.Stopwatch]::StartNew(); $max = 0.0; $at = 0; $who = ''; $n = 0
  while ($sw.ElapsedMilliseconds -lt 1000) {
    $t = $sw.ElapsedMilliseconds
    $g = [Px]::Grab($bx, $by, $bw, $bh)
    for ($i = 0; $i -lt $rects.Count; $i++) {
      $x = $rects[$i][0] - $bx; $y = $rects[$i][1] - $by
      $d = [Px]::Diff($ref, $g, $x, $x + 79, $y, $y + 95)
      if ($d -gt $max) { $max = $d; $at = $t; $who = if ($watch[$i].fileName) { $watch[$i].fileName } else { $watch[$i].name } }
    }
    $g.Dispose(); $n++
  }
  $moved = HdWait { Test-Path -LiteralPath (Join-Path $HdFolder $leaves) } 6
  Start-Sleep -Seconds 1
  ParkMouse
  Check 'setup: the dropped file is in the folder and the other icons kept their cells' ($moved -and -not (HdCell $leaves) -and ((HdAll) -eq (($before -split ' ; ' | Where-Object { $_ -notlike "$leaves=*" }) -join ' ; '))) "in the folder=$moved ; before: $before ; after: $(HdAll)"
  $where = if ($max -gt 0) { "largest change {0:N1} (mean brightness, 0 to 255) in the cell of {1}, {2} ms after the release" -f $max, $who, $at } else { 'no pixel changed' }
  Check 'no other loose icon''s cell changes while the icon leaves' ($max -lt 3.0) "$where; $n samples of $($rects.Count) cells in 1 s"
  # The window is cut to the icons' cells: each icon that stays still takes a click in its
  # cell, and the cell of the icon that left is bare desktop again.
  $own = @($rects | ForEach-Object { [W]::RootClassAt($_[0] + 40, $_[1] + 30) })
  $left = [W]::RootClassAt($p[0], $p[1])
  Check 'the window still covers the cells of the icons that stay, and no longer the cell of the one that left' (@($own | Where-Object { $_ -ne 'PecoFence.Fence' }).Count -eq 0 -and $left -ne 'PecoFence.Fence') "window at the icons that stay: $($own -join ', '); at the cell that was left ($($p -join ',')): $left"
  [Anim]::Set($animWas)
  HdClean
  Remove-Item -LiteralPath $HdFolder -Recurse -Force -ErrorAction SilentlyContinue
}

if ($Part -eq 'D') {
  # 自由摆放: an icon of another fence let go on 「桌面」 (on a loose icon, or on the bare
  # desktop). Its landing glide went to the spare cell at the far side of the screen, across
  # bare desktop where the window is cut away: the icon was nowhere for about 170 ms and then
  # stood in its cell. It now starts towards the cell it was dropped into.
  Case 'An icon of a fence let go on the desktop is seen at once'
  CloseWindows
  Get-Process OpenWith -ErrorAction SilentlyContinue | Stop-Process -Force
  HdClean
  $HdTxt = @('读我.txt', '草稿.txt')
  foreach ($n in $HdTxt) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  foreach ($n in $HdNames + $HdTxt) { DeskFile $n | Out-Null }
  [G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  $docs = (@(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq '文件与文档' }) | Select-Object -First 1).id
  function HdPtIn($file) { $o = @(@(PfJson item list --fence $docs) | Where-Object { $_ -and $_.id } | ForEach-Object { $_.fileName }); $i = [array]::IndexOf($o, $file); if ($i -lt 0) { return $null }; return (CellPt $docs $i) }
  # A loose icon at cell 14,3; the cell left of it (13,3) and the one above it (14,2) are empty.
  $p = HdPt '图.fpx'; DragTo $p[0] $p[1] 1164 340 20; Start-Sleep -Seconds 1
  ParkMouse 1100 700
  # A first start takes the places Windows has the icons at, so the other icons' cells differ
  # from run to run: the two cells the drops go to must be empty.
  $taken = @((HdCells).Values | Where-Object { $_ -eq '13,3' -or $_ -eq '14,2' })
  Check 'setup: the loose icon stands at 14,3, cells 13,3 and 14,2 are empty, both files are in the fence' ((HdCell '图.fpx') -eq '14,3' -and $taken.Count -eq 0 -and (HdPtIn '读我.txt') -and (HdPtIn '草稿.txt')) "图=$(HdCell '图.fpx'); $(HdAll)"
  # Drags $file from the fence to ($x, $y), rests, lets go. Reads the cell $cell (empty before
  # the drag) as often as the screen can be read for 400 ms after the release: how many of
  # the reads in the first 300 ms show it empty, and where the app's log says the landing
  # glide went.
  function HdDrop($file, $x, $y, $cell) {
    trap { PfTrap $_; continue }
    $c = $cell -split ','; $rx = 4 + [int]$c[0] * 80; $ry = 4 + [int]$c[1] * 96
    $empty = [Px]::Grab($rx, $ry, 80, 96)
    $from = HdPtIn $file
    [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
    MDown; Start-Sleep -Milliseconds 250
    Glide $from[0] $from[1] ($from[0] - 14) ($from[1] + 6) 3 30
    Glide ($from[0] - 14) ($from[1] + 6) $x $y 20 30
    Start-Sleep -Milliseconds 600
    MUp
    # $strength: how much of the icon shows in the first read that shows it, against the icon
    # at rest (mean brightness change of the cell): about 1 for an icon that arrives at full
    # strength, well under 1 for one that fades in.
    $sw = [Diagnostics.Stopwatch]::StartNew(); $reads = 0; $blank = 0; $first = -1; $firstD = 0.0; $d = 0.0
    while ($sw.ElapsedMilliseconds -lt 400) {
      $t = $sw.ElapsedMilliseconds
      $g = [Px]::Grab($rx, $ry, 80, 96); $d = [Px]::Diff($empty, $g, 0, 79, 0, 95); $g.Dispose()
      if ($t -le 300) { $reads++; if ($d -lt 3.0) { $blank++ } }
      if ($d -ge 3.0 -and $first -lt 0 -and $t -gt 15) { $first = $t; $firstD = $d }
    }
    Start-Sleep -Milliseconds 900
    # A click on the bare desktop takes the selection box off the dropped icon: the icon alone.
    DeskClick 1100 700 | Out-Null; Start-Sleep -Milliseconds 400
    $g = [Px]::Grab($rx, $ry, 80, 96); $rest = [Px]::Diff($empty, $g, 0, 79, 0, 95); $g.Dispose()
    $shown = [IO.Path]::GetFileNameWithoutExtension($file)
    $glide = @(PfLogAll | Where-Object { $_ -match "dropped icon glides from where it was let go.* name=$([regex]::Escape($shown)) " }) | Select-Object -Last 1
    $to = if ($glide -match 'to_x=([\d.]+) to_y=([\d.]+)') { @([double]$Matches[1], [double]$Matches[2]) } else { $null }
    ParkMouse 1100 700
    $strength = if ($rest -gt 0) { [math]::Round($firstD / $rest, 2) } else { 0 }
    return [pscustomobject]@{ cell = (HdCell $file); reads = $reads; blank = $blank; first = $first; strength = $strength; firstD = [math]::Round($firstD, 1); rest = [math]::Round($rest, 1); to = $to; inCell = ($to -and $to[0] -ge $rx -and $to[0] -lt $rx + 80 -and $to[1] -ge $ry -and $to[1] -lt $ry + 96) }
  }
  # On the loose icon: the nearest free cell is the one left of it.
  $t = HdPt '图.fpx'
  $a = HdDrop '读我.txt' $t[0] $t[1] '13,3'
  Shot 'D-on-a-loose-icon'
  Check 'let go on a loose icon: it lands in the free cell beside it' ($a.cell -eq '13,3') "cell $($a.cell)"
  Check 'let go on a loose icon: the landing glide goes to that cell' ($a.inCell) "the log's glide target $($a.to -join ','); cell 13,3 is x 1044 to 1124, y 292 to 388"
  # Measured 2026-11-12 on ec2-7: 5 of 10 reads empty and first seen after 164 ms before, 1 of
  # 11 and 21 ms after. Two empty reads are allowed for the moment between the drag picture
  # going and the first frame with the icon.
  Check 'let go on a loose icon: its cell is not empty after the release' ($a.reads -ge 5 -and $a.blank -le 2 -and $a.first -ge 0 -and $a.first -le 100) "$($a.blank) of $($a.reads) reads in the first 300 ms show the cell empty; first seen $($a.first) ms after the release"
  # On the bare desktop, in the cell above the loose icon.
  $b = HdDrop '草稿.txt' 1164 232 '14,2'
  Shot 'D-on-the-bare-desktop'
  Check 'let go on the bare desktop: it lands in the cell under it' ($b.cell -eq '14,2') "cell $($b.cell)"
  # The drag picture rests in this very cell until the release, so a first read can still show
  # it: only the count of empty reads says something here.
  # Measured 2026-11-12 on ec2-7: 5 of 11 reads empty before, 1 of 11 after; two are allowed.
  Check 'let go on the bare desktop: its cell is not empty after the release' ($b.reads -ge 5 -and $b.blank -le 2) "$($b.blank) of $($b.reads) reads in the first 300 ms show the cell empty"
  Log "strength of the first read that shows the icon: on a loose icon $($a.strength) ($($a.firstD) of $($a.rest), $($a.first) ms); on the bare desktop $($b.strength) ($($b.firstD) of $($b.rest), $($b.first) ms); bare desktop glide target $($b.to -join ',')"
  # Let go on the bare desktop, the drag source ends the drop and the window of 「桌面」 did not
  # learn where the icon was let go: it faded in at its cell. It now arrives from there at full
  # strength, as one let go on a loose icon does. How much of the icon the first read shows,
  # against the icon at rest: 0.32 and 0.30 while it faded in (reads 47 and 43 ms after the
  # release), 1.0 now (42 ms); 2026-11-12, ec2-7. A first read later than 60 ms cannot tell a
  # fade from an arrival and fails.
  Check 'let go on the bare desktop: the landing glide goes to its cell' ($b.inCell) "the log's glide target $($b.to -join ','); cell 14,2 is x 1124 to 1204, y 196 to 292"
  Check 'let go on the bare desktop: the icon arrives at full strength' ($b.first -ge 0 -and $b.first -le 60 -and $b.strength -ge 0.6) "first read that shows it: $($b.first) ms after the release, $($b.firstD) of $($b.rest) at rest ($($b.strength))"
  [Anim]::Set($animWas)
  HdClean
  foreach ($n in $HdTxt) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
}
