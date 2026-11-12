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
