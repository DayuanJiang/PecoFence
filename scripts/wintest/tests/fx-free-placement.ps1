# 自由摆放 (free placement) of the 「桌面」 fence arranged like the desktop, with the real mouse.
#   run.sh tests/fx-free-placement.ps1                        drags, menus, keys, restart, fences, files
#   run.sh --var Build=old tests/fx-free-placement.ps1        the same drags on a build without it
#   tests/fx-free-placement.sh [--part C]                      part B: the RDP size changes mid-run
#                                                              part C: the scale goes to 150 %
# Parts B and C need the dev box to change the RDP session while the job waits:
# fx-free-placement.sh runs run.sh --var Part=B (or C) and answers C:\pf\resize-want.txt.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
if (-not (Get-Variable Build -ErrorAction SilentlyContinue)) { $Build = 'new' }
$New = $Build -ne 'old'
$FpNames = @('Alpha.fpx', 'Bravo.fpx', 'Charlie.fpx', 'Delta.fpx', 'Echo.fpx', 'Foxtrot.fpx')
$FpExtra = @('Golf.fpx', 'Hotel.fpx')

function FpItems() { return @(@(PfJson item list --fence inbox) | Where-Object { $_ -and $_.id }) }
function FpId($name) { return "$((FpItems | Where-Object { $_.fileName -eq $name -or $_.name -eq $name } | Select-Object -First 1).id)" }
function FpMem() { $p = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1; if ($p) { return "private $([int]($p.PrivateMemorySize64 / 1MB)) MB, working set $([int]($p.WorkingSet64 / 1MB)) MB" }; return '' }
function FpCells() {
  $h = @{}; $g = FenceGet inbox
  if ($g.desktopCells) { foreach ($p in $g.desktopCells.PSObject.Properties) { $h[$p.Name] = "$($p.Value[0]),$($p.Value[1])" } }
  return $h
}
function FpCell($name) { $c = FpCells; $id = FpId $name; if ($id -and $c.ContainsKey($id)) { return $c[$id] }; return $null }
# The cell grid at 100 %: 80 x 96 px cells, 4 px in from the work area's top-left corner.
function FpPt($col, $row) { return @([int](4 + $col * 80 + 40), [int](4 + $row * 96 + 30)) }
function FpAt($cell) { $c = $cell -split ','; return FpPt ([int]$c[0]) ([int]$c[1]) }
function FpCellOf($x, $y) { return "$([math]::Floor(($x - 4) / 80)),$([math]::Floor(($y - 4) / 96))" }
function FpAll() { $c = FpCells; return ((FpItems | Sort-Object fileName | ForEach-Object { "$($_.fileName)=$($c["$($_.id)"])" }) -join ' ') }
# Icons of 「桌面」 that share a cell, and icons whose cell lies under another fence's window.
function FpDupes() { return ((FpCells).Values | Group-Object | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name }) -join ' ' }
function FpUnder() {
  $c = FpCells; $out = @()
  $others = @(@(PfJson fence list) | Where-Object { $_ -and $_.kind -ne 'inbox' -and $_.windowRect })
  foreach ($it in FpItems) {
    $v = $c["$($it.id)"]; if (-not $v) { continue }
    $p = $v -split ','; $l = 4 + [int]$p[0] * 80; $t = 4 + [int]$p[1] * 96
    foreach ($f in $others) {
      $r = $f.windowRect
      if ($l -lt $r.x + $r.w -and $r.x -lt $l + 80 -and $t -lt $r.y + $r.h -and $r.y -lt $t + 96) { $out += "$($it.fileName)@$v under $($f.title)" }
    }
  }
  return ($out -join '; ')
}
# Whether an icon of 「桌面」 is at the screen point (the window there is a fence cell).
function FpIconAt($x, $y) { return ([W]::RootClassAt($x, $y) -eq 'PecoFence.Fence') }
function FpLog($what) { Log "$what :: $(FpAll)" }
function Park() { [G]::MoveTo(1560, 820); Start-Sleep -Milliseconds 300 }

Case 'seed'
foreach ($n in $FpNames + $FpExtra) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
foreach ($n in $FpNames) { DeskFile $n | Out-Null }
PfStart -Script (DumpScript 300 14)
Start-Sleep -Seconds 3
Park
$g = FenceGet inbox
Log "fences: $((@(PfJson fence list) | Where-Object { $_ -and $_.title } | ForEach-Object { "$($_.title)[$($_.windowRect.x),$($_.windowRect.y),$($_.windowRect.w)x$($_.windowRect.h)]" }) -join ' ; ')"
Check 'the inbox is arranged like the desktop' ($g.desktopLayout -eq $true) "rect $($g.windowRect | ConvertTo-Json -Compress)"
Log "memory arranged: $(FpMem)"
Shot "$Build-01-arranged"

if ($New) {
  Case 'switch on (CLI)'
  Pf fence set inbox freePlacement true | Out-Null
  Start-Sleep -Seconds 2
  FpLog 'on'
  $cells = FpCells
  Check 'every inbox icon gets a cell' ($cells.Count -eq (FpItems).Count) (FpAll)
  Check 'switching on keeps the columns (cells 0,0 down)' ((FpCell 'Alpha.fpx') -eq '0,0' -and (FpCell 'Recycle Bin') -eq '0,6') (FpAll)
  Check 'fence get reports freePlacement' ((FenceGet inbox).freePlacement -eq $true) ''
}

Case 'drag one icon to an empty spot'
$from = FpPt 0 0
DragTo $from[0] $from[1] 804 450 20
Start-Sleep -Seconds 1
Park
Check 'an icon is where Alpha was dropped' (FpIconAt 804 450) "root at 804,450: $([W]::RootClassAt(804, 450))"
if ($New) { Check 'Alpha keeps the cell it was dropped in' ((FpCell 'Alpha.fpx') -eq '10,4') "$(FpAll)" }
Shot "$Build-02-alpha-dropped"

if ($Part -eq 'A' -and $New) {
  Case 'more drags'
  $from = FpAt (FpCell 'Bravo.fpx')
  DragTo $from[0] $from[1] 300 700 20
  Start-Sleep -Milliseconds 800
  Check 'Bravo keeps its drop cell' ((FpCell 'Bravo.fpx') -eq '3,7') (FpAll)
  # Onto Alpha: the nearest free cell (Explorer does the same).
  $from = FpAt (FpCell 'Charlie.fpx'); $to = FpAt '10,4'
  DragTo $from[0] $from[1] $to[0] $to[1] 20
  Start-Sleep -Milliseconds 800
  $ch = FpCell 'Charlie.fpx'
  Check 'Charlie dropped on Alpha goes to a free cell next to it' ($ch -ne '10,4' -and (FpCell 'Alpha.fpx') -eq '10,4' -and @('9,4', '11,4', '10,3', '10,5') -contains $ch) "Charlie=$ch; $(FpAll)"
  Check 'no two icons share a cell' (-not (FpDupes)) "dupes: $(FpDupes)"
  # Into another fence and back to the desktop.
  FenceIds
  $prog = FenceGet 'Programs'
  $from = FpAt (FpCell 'Delta.fpx')
  DragTo $from[0] $from[1] ($prog.windowRect.x + 120) ($prog.windowRect.y + 100) 20
  Start-Sleep -Milliseconds 800
  Check 'Delta dropped on Programs moves into it' ((ItemFence 'Delta.fpx') -eq 'Programs') "fence $(ItemFence 'Delta.fpx')"
  $p = ItemPt 'Programs' 'Delta.fpx'
  if ($p) { DragTo $p[0] $p[1] 604 220 20 }
  Start-Sleep -Milliseconds 800
  Check 'Delta dragged back lands in its drop cell' ((ItemFence 'Delta.fpx') -eq 'Desktop' -and (FpCell 'Delta.fpx') -eq '7,2') "fence $(ItemFence 'Delta.fpx'), cell $(FpCell 'Delta.fpx')"
  # From Programs onto Charlie (9,4), an icon of Desktop that is no folder: the free cell
  # nearest to it, (8,4), as Explorer does.
  $p = ItemPt 'Programs' 'Microsoft Edge'
  $to = FpAt '9,4'
  if ($p) { DragTo $p[0] $p[1] $to[0] $to[1] 20 }
  Start-Sleep -Milliseconds 800
  Check 'an icon from a fence dropped on an icon of Desktop goes next to it' ((ItemFence 'Microsoft Edge') -eq 'Desktop' -and (FpCell 'Microsoft Edge') -eq '8,4' -and (FpCell 'Charlie.fpx') -eq '9,4') "fence $(ItemFence 'Microsoft Edge'), cell $(FpCell 'Microsoft Edge'); $(FpAll)"
  Log "memory with icons spread: $(FpMem)"
  Park
  Shot 'new-03-four-dropped'

  Case 'arrow keys go by place'
  # Alpha 10,4, Charlie 9,4, Delta 7,2, Bravo 3,7, Echo 0,4, Foxtrot 0,5, Recycle Bin 0,6.
  $p = FpAt '10,4'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  $f = @((DumpVal (DumpOf 'Desktop') 'focused'))
  foreach ($k in 0x25, 0x27, 0x26, 0x25) { [G]::Press($k); Start-Sleep -Milliseconds 500; $f += DumpVal (DumpOf 'Desktop') 'focused' }
  Log "click, Left, Right, Up, Left: $($f -join ' ')"
  Check 'Left from Alpha goes to Charlie beside it' ($f[0] -match 'Alpha' -and $f[1] -match 'Charlie') "$($f -join ' ')"
  Check 'Right goes back to Alpha' ($f[2] -match 'Alpha') "$($f -join ' ')"
  Check 'Up goes to Delta, the nearest icon above' ($f[3] -match 'Delta') "$($f -join ' ')"
  Check 'Left from Delta goes to Echo, the nearest icon that way' ($f[4] -match 'Echo') "$($f -join ' ')"
  # A click on the empty desktop after one on an icon also brings the desktop to the front for
  # the rectangle below (one with the desktop already in front is not seen: finding S2).
  [G]::Click(700, 120); Start-Sleep -Milliseconds 800
  Park

  Case 'two icons at once'
  # A rectangle on the empty desktop selects Echo (0,4) and Foxtrot (0,5).
  DragTo 130 400 30 560 12
  Start-Sleep -Milliseconds 500
  $d = DumpOf 'Desktop'
  Log "after the rectangle: selected=$(DumpVal $d 'selected') focused=$(DumpVal $d 'focused')"
  Check 'the rectangle selects the two icons' ((DumpVal $d 'selected') -eq '2') "selected=$(DumpVal $d 'selected')"
  $from = FpAt (FpCell 'Echo.fpx')
  DragTo $from[0] $from[1] 1004 520 20
  Start-Sleep -Milliseconds 800
  Check 'both keep their places to each other (Foxtrot one row below Echo)' ((FpCell 'Echo.fpx') -eq '12,5' -and (FpCell 'Foxtrot.fpx') -eq '12,6') (FpAll)
  Park
  Shot 'new-04-two-dropped'

  Case 'restart'
  $before = FpAll
  PfStart -Script (DumpScript 300 10)
  Start-Sleep -Seconds 3
  Park
  Check 'every cell is kept after a restart' ((FpAll) -eq $before) "before $before | after $(FpAll)"
  Check 'the icon is still at the drop point' (FpIconAt 804 450) ''
  Shot 'new-05-after-restart'

  Case 'a fence dragged over icons'
  $prog = FenceGet 'Programs'
  $r = $prog.windowRect
  # Title row of Programs, dragged so the fence covers Alpha (10,4) and Charlie.
  DragTo ($r.x + 150) ($r.y + 18) (150 + 744) (18 + 380) 25
  Start-Sleep -Seconds 1
  Park
  $prog = FenceGet 'Programs'
  FpLog "Programs now at $($prog.windowRect | ConvertTo-Json -Compress)"
  Check 'the fence moved over the icons' ($prog.windowRect.x -lt 884 -and $prog.windowRect.x + $prog.windowRect.w -gt 804 -and $prog.windowRect.y -lt 484) "Programs $($prog.windowRect | ConvertTo-Json -Compress)"
  Check 'no icon of Desktop is under a fence' (-not (FpUnder)) "$(FpUnder)"
  Check 'no two icons share a cell after the move' (-not (FpDupes)) "dupes: $(FpDupes)"
  Shot 'new-06-fence-over-icons'

  Case 'new desktop files'
  DeskFile 'Golf.fpx' | Out-Null
  Start-Sleep -Seconds 2
  DeskFile 'Hotel.fpx' | Out-Null
  Start-Sleep -Seconds 2
  FpLog 'new files'
  # Column 0 from the top: Alpha, Bravo, Charlie, Delta, Echo and Foxtrot have left it.
  Check 'a new file takes the first free cell down the columns' ((FpCell 'Golf.fpx') -eq '0,0' -and (FpCell 'Hotel.fpx') -eq '0,1') "Golf $(FpCell 'Golf.fpx'), Hotel $(FpCell 'Hotel.fpx')"
  Check 'no icon of Desktop is under a fence with new files' (-not (FpUnder)) "$(FpUnder)"
  Shot 'new-07-new-files'

  Case 'off and on from the item menu'
  $saved = FpAll
  $p = FpAt (FpCell 'Golf.fpx')
  [G]::RClick($p[0], $p[1])
  $m = MenuWin 8
  $texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
  $row = @(MenuRows $m) | Where-Object { $_.text -eq 'Place freely' } | Select-Object -First 1
  Check 'the item menu has Place freely, ticked' ($row -and ($row.state -band 8)) "state=$($row.state) menu: $texts"
  Shot 'new-08a-item-menu'
  $t = MenuPick '^Place freely$' $m
  Start-Sleep -Seconds 1
  Park
  $bp = FpAt (FpCell 'Bravo.fpx')
  Check 'off: the icons stand in columns again' ((FenceGet inbox).freePlacement -eq $false -and -not (FpIconAt $bp[0] $bp[1])) "free=$((FenceGet inbox).freePlacement) icon at Bravo's cell $($bp -join ','): $(FpIconAt $bp[0] $bp[1])"
  Shot 'new-08-off'
  Check 'off keeps the saved cells' ((FpAll) -eq $saved) "now $(FpAll)"
  Pf fence set inbox freePlacement true | Out-Null
  Start-Sleep -Seconds 1
  Check 'on again: the icons are back where they were' ((FpAll) -eq $saved -and (FpIconAt $bp[0] $bp[1])) "now $(FpAll)"

  Case 'the fence menu and settings'
  # The Desktop fence menu has the switch below Arrange like the desktop (CLI: fence menu of the inbox
  # is out of reach while arranged, so it is checked through the CLI's property list instead).
  $desc = (Pf describe) -join ' '
  Check 'the CLI describes freePlacement' ($desc -match 'freePlacement') ''
  $r = Pf fence set inbox freePlacement maybe
  Check 'a bad value is refused and changes nothing' ((FenceGet inbox).freePlacement -eq $true -and $r -notmatch '"changed":\s*true') "$r"

  Case 'a new sort lines the icons up again'
  Pf fence set inbox sort name | Out-Null
  Start-Sleep -Seconds 2
  Park
  FpLog 'sorted by name'
  # Cells in column order: col * 100 + row.
  $at = { param($n) $c = (FpCell $n) -split ','; [int]$c[0] * 100 + [int]$c[1] }
  $order = @('Alpha.fpx', 'Bravo.fpx', 'Charlie.fpx', 'Delta.fpx', 'Echo.fpx', 'Foxtrot.fpx', 'Golf.fpx', 'Hotel.fpx') | ForEach-Object { & $at $_ }
  $inOrder = $true; for ($k = 1; $k -lt $order.Count; $k++) { if ($order[$k] -le $order[$k - 1]) { $inOrder = $false } }
  Check 'sorted by name: the icons stand down the columns in name order' ($inOrder -and $order[0] -le 1) "cells $($order -join ' '); $(FpAll)"
  Shot 'new-09-sorted'

  Case 'the fence menu while not arranged'
  $sorted = FpAll
  Pf fence set inbox desktopLayout false | Out-Null
  Start-Sleep -Seconds 2
  $r = (FenceGet inbox).windowRect
  [G]::RClick($r.x + 60, $r.y + 14)
  $m = MenuWin 8
  $row = @(MenuRows $m) | Where-Object { $_.text -eq 'Place freely' } | Select-Object -First 1
  Check 'the Desktop fence menu has Place freely, ticked and greyed while not arranged' ($row -and ($row.state -band 3) -and ($row.state -band 8)) "state=$($row.state) menu: $(if ($m -ne [IntPtr]::Zero) { MenuTexts $m })"
  Shot 'new-10-fence-menu'
  CloseMenus
  Pf fence set inbox desktopLayout true | Out-Null
  Start-Sleep -Seconds 2
  Park
  Check 'arranged again: the icons are back in their cells' ((FpAll) -eq $sorted) "before $sorted | now $(FpAll)"
}

if ($Part -eq 'A' -and -not $New) {
  Case 'restart (old build)'
  PfStart -Script (DumpScript 300 10)
  Start-Sleep -Seconds 3
  Park
  Check 'an icon is at the drop point after a restart' (FpIconAt 804 450) ''
  Shot 'old-03-after-restart'
}

if ($Part -eq 'B' -or $Part -eq 'C') {
  # The RDP size (B) or scale (C) changes while PecoFence runs, then goes back.
  Case "monitor change (part $Part)"
  # Bravo to the bottom right (17,7), Charlie to the top right (15,0), Delta to the bottom (7,7).
  foreach ($d in @(@('Bravo.fpx', 1404, 750), @('Charlie.fpx', 1244, 30), @('Delta.fpx', 604, 750))) {
    $from = FpAt (FpCell $d[0])
    DragTo $from[0] $from[1] $d[1] $d[2] 20
    Start-Sleep -Milliseconds 800
  }
  Park
  $big = FpAll
  FpLog 'before the size change'
  Shot 'resize-01-1600x900'
  Add-Type @'
using System; using System.Runtime.InteropServices;
public static class FpScr {
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, ref RC r, uint f);
  public static string Size() { return GetSystemMetrics(0) + "x" + GetSystemMetrics(1); }
  public static int[] Work() { var r = new RC(); SystemParametersInfoW(0x30, 0, ref r, 0); return new int[] { r.L, r.T, r.R, r.B }; }
}
'@
  function WantSize($w, $h, $scale = '') {
    $want = if ($scale) { "${w}x${h}@$scale" } else { "${w}x${h}" }
    Set-Content C:\pf\resize-want.txt $want -Encoding ASCII
    for ($k = 0; $k -lt 150; $k++) {
      Start-Sleep -Seconds 1
      if (-not (Test-Path C:\pf\resize-want.txt) -and ($scale -or [FpScr]::Size() -eq "${w}x${h}")) { break }
    }
    if ($scale) { $l = PfWaitLog "dpi=$([int]$scale * 96 / 100) origin" 60; Log "monitor line: $l" }
    Start-Sleep -Seconds 8
    Log "screen now $([FpScr]::Size()), work area $([FpScr]::Work() -join ',')"
  }
}
if ($Part -eq 'C') {
  # 150 %: cells of 120 x 144 px, 6 px in; the icons keep their columns and rows where they fit.
  WantSize 1600 900 150
  Park
  FpLog 'at 150 %'
  $g = FenceGet inbox
  $wa = [FpScr]::Work()
  $cols = [math]::Floor(($wa[2] - $wa[0] - 6) / 120); $rows = [math]::Floor(($wa[3] - $wa[1] - 6) / 144)
  $outside = @((FpCells).Values | Where-Object { $p = $_ -split ','; [int]$p[0] -ge $cols -or [int]$p[1] -ge $rows })
  Check 'at 150 % every icon is inside the work area' ($outside.Count -eq 0) "grid ${cols}x$rows; outside: $($outside -join ' '); $(FpAll)"
  Check 'no two icons share a cell at 150 %' (-not (FpDupes)) "dupes: $(FpDupes)"
  # Echo (0,4) still fits; Alpha (10,4) is under a fence at 150 %, Foxtrot and the Recycle Bin
  # (rows 5, 6) are past the last row: they take the free cells nearest to theirs.
  Check 'at 150 % an icon that fits keeps its column and row' ((FpCell 'Echo.fpx') -eq '0,4' -and (FpCell 'Alpha.fpx') -eq '9,4') "$(FpAll)"
  Shot 'scale-02-150'
  WantSize 1600 900
  Park
  FpLog 'back at 100 %'
  Check 'back at 100 % every icon is where it was' ((FpAll) -eq $big) "before $big | now $(FpAll)"
  Shot 'scale-03-back'
}
if ($Part -eq 'B') {
  WantSize 1280 720
  Park
  FpLog 'at 1280x720'
  $wa = [FpScr]::Work()
  $cols = [math]::Floor(($wa[2] - $wa[0] - 4) / 80); $rows = [math]::Floor(($wa[3] - $wa[1] - 4) / 96)
  $outside = @((FpCells).Values | Where-Object { $p = $_ -split ','; [int]$p[0] -ge $cols -or [int]$p[1] -ge $rows })
  Check 'a smaller screen moves every icon inside its work area' ($outside.Count -eq 0) "grid ${cols}x$rows; outside: $($outside -join ' '); $(FpAll)"
  Check 'no two icons share a cell on the smaller screen' (-not (FpDupes)) "dupes: $(FpDupes)"
  Check 'icons that fit keep their cells' ((FpCell 'Alpha.fpx') -eq '10,4') "$(FpAll)"
  Shot 'resize-02-1280x720'
  WantSize 1600 900
  Park
  FpLog 'back at 1600x900'
  Check 'back on the first size every icon is where it was' ((FpAll) -eq $big) "before $big | now $(FpAll)"
  Shot 'resize-03-back'
}

Case 'end'
PfQuit
foreach ($n in $FpNames + $FpExtra) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue
