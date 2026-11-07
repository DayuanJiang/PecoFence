# Fence fix round 2026-11-07, area ux-snap-fit, part C: the 「桌面」 fence arranged like the
# desktop. A press beside an icon must not drag the whole fence (S6); a Ctrl rectangle on the
# desktop keeps the icons selected when it started while a desktop file goes away (C16).

function ClearFences() {
  foreach ($f in @(PfJson fence list)) { if ($f -and $f.title -and $f.kind -ne 'inbox') { Pf fence delete "$($f.id)" | Out-Null } }
  Start-Sleep -Seconds 1
}
function WinRect($h) { $r = Rect $h; return "$($r.L),$($r.T),$($r.R),$($r.B)" }
# Centre of the desktop cell of display index $i: columns of 96 px rows from the top-left of the
# work area (4 px in), as the Windows desktop arranges them; no other fence covers a cell here.
function DeskCell($i) {
  $rows = [math]::Floor(([Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Height - 4) / 96)
  return @([int](4 + [math]::Floor($i / $rows) * 80 + 40), [int](4 + ($i % $rows) * 96 + 40))
}
# Where icon $name (display index $i) stands: its saved cell when the build places the icons
# freely (`fence get` desktopCells, 自由摆放), else the cell of its place in the order.
function IconCell($name, $i) {
  $g = FenceGet inbox
  if ($g.desktopCells) {
    $id = "$((@(PfJson item list --fence inbox) | Where-Object { $_.name -eq $name } | Select-Object -First 1).id)"
    $c = $g.desktopCells.PSObject.Properties[$id]
    if ($c) { return @([int](4 + $c.Value[0] * 80 + 40), [int](4 + $c.Value[1] * 96 + 40)) }
  }
  return DeskCell $i
}
$names = @('fxc-a.txt', 'fxc-b.txt', 'fxc-c.txt', 'fxc-d.txt', 'fxc-e.txt')

Case 'setup'
CloseWindows
foreach ($n in $names) { Set-Content -LiteralPath (Join-Path $PfDesk $n) 'fx' }
PfStart -Script (DumpScript 250 12)
ClearFences
Pf item move @($names | ForEach-Object { Join-Path $PfDesk $_ }) --to inbox | Out-Null
$ib = FenceGet 'inbox'
if (-not $ib.desktopLayout) { Pf fence set inbox desktopLayout true | Out-Null; Start-Sleep -Seconds 1; $ib = FenceGet 'inbox' }
Start-Sleep -Seconds 2
[G]::MoveTo(1100, 830)
$d = DumpOf $ib.title
Log "inbox '$($ib.title)' desktopLayout=$($ib.desktopLayout) window $(WinRect $d.hwnd); order [$((@(PfJson item list --fence inbox) | ForEach-Object { $_.name }) -join ', ')]"
Shot 'c-start'

Case 'S6 a press beside the top icon of the last column of the Desktop fence'
# Where a fence with a title row has its roll-up arrow: the top right of the window (72,8 when
# the Desktop fence is one column wide, as in the review).
$rows = @(); $ok = 0
$right = (Rect (DumpOf $ib.title).hwnd).R
$order0 = (@(PfJson item list --fence inbox) | ForEach-Object { $_.name }) -join ', '
# Beside the top icon of the last column, off the icon and its name (those take an icon drag).
foreach ($pt in @(@(($right - 16), 8), @(($right - 8), 20), @(($right - 10), 60))) {
  $h = (DumpOf $ib.title).hwnd
  $before = WinRect $h
  $hit = NcHit $h $pt[0] $pt[1]
  DragTo $pt[0] $pt[1] ($pt[0] + 300) ($pt[1] + 300) 15
  Start-Sleep -Seconds 1
  $after = WinRect $h
  CloseMenus
  if ($after -eq $before) { $ok++ } else {
    Shot "s6-moved-from-$($pt[0])-$($pt[1])"
    Pf fence set inbox desktopLayout false | Out-Null; Start-Sleep -Milliseconds 800
    Pf fence set inbox desktopLayout true | Out-Null; Start-Sleep -Seconds 1
  }
  $rows += "$($pt -join ','): hit $hit, window $before -> $after"
}
Shot 's6-after-drags'
Check 'a drag from beside the first icon leaves the Desktop fence in place (3 points)' ($ok -eq 3) "$($rows -join ' | ')"
$order1 = (@(PfJson item list --fence inbox) | ForEach-Object { $_.name }) -join ', '
Check 'and it drags no icon either (a rectangle; the order stays)' ($order1 -eq $order0) "before [$order0] after [$order1]"

Case 'C16 a Ctrl rectangle on the desktop while a file goes away'
$ok = 0; $rows = @()
foreach ($run in 1, 2) {
  foreach ($n in $names) { Set-Content -LiteralPath (Join-Path $PfDesk $n) 'fx' }
  Start-Sleep -Seconds 2
  Pf item move @($names | ForEach-Object { Join-Path $PfDesk $_ }) --to inbox | Out-Null; Start-Sleep -Seconds 1
  $order = @(@(PfJson item list --fence inbox) | ForEach-Object { $_.name })
  $ic = [array]::IndexOf($order, 'fxc-c')
  # The file shown just before fxc-c goes away during the rectangle: fxc-c moves up one cell.
  $gone = if ($ic -gt 0 -and $order[$ic - 1] -like 'fxc-*') { "$($order[$ic - 1]).txt" } else { 'fxc-a.txt' }
  $p = IconCell 'fxc-c' $ic
  # A click on the empty desktop first: the first desktop press after a start can go unseen.
  [G]::Click(1000, 700); Start-Sleep -Milliseconds 500
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  $sel0 = DumpVal (DumpOf $ib.title) 'selected'
  # Ctrl held: a rectangle on the empty desktop that touches no icon; meanwhile the file
  # before fxc-c goes away.
  [G]::KeyDown(0x11); Start-Sleep -Milliseconds 100
  [G]::MoveTo(900, 500); Start-Sleep -Milliseconds 150; MDown; Start-Sleep -Milliseconds 150
  Glide 900 500 860 520 6 30
  Remove-Item -LiteralPath (Join-Path $PfDesk $gone)
  Start-Sleep -Milliseconds 2500
  Glide 860 520 840 530 4 30; Start-Sleep -Milliseconds 400
  MUp; Start-Sleep -Milliseconds 200; [G]::KeyUp(0x11); Start-Sleep -Milliseconds 800
  CloseMenus
  Chord 0x11 0x43; Start-Sleep -Milliseconds 700
  $clip = @(ClipFiles | ForEach-Object { Split-Path -Leaf $_ }) -join ', '
  if ($run -eq 1) { Shot 'c16-after-rectangle' }
  if ($clip -eq 'fxc-c.txt') { $ok++ }
  $rows += "run ${run}: fxc-c at $ic of [$($order -join ', ')], $gone deleted, selected before $sel0, copied after the rectangle [$clip]"
  [G]::Click(1000, 700); Start-Sleep -Milliseconds 500
}
Check 'the icon selected before the rectangle is still the one selected (2 runs)' ($ok -eq 2) "$($rows -join ' | ')"

Case 'end'
PfQuit
foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -ErrorAction SilentlyContinue }
