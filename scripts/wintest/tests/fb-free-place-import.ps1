# 自由摆放 with no saved cells takes the places Windows' own desktop has the icons at (#204).
#   run.sh tests/fb-free-place-import.ps1                     part A at 100 %: first start, restart,
#                                                              sort, Windows auto-arrange, switching on
#   run.sh --var Build=old tests/fb-free-place-import.ps1     the first start on a build without it
#   tests/fx-free-placement.sh --part C --steps tests/fb-free-place-import.ps1
#                                                              part C: the first start at 150 %
#   run.sh --var Part=R tests/fb-free-place-import.ps1        the before/after recording (with Build)
# The user's own layout is made with real mouse drags of Windows' icons (a place set only with
# IFolderView::SelectAndPositionItems does not last). Expected cells follow the rule written out
# again here: the column and row of Windows' grid; one past PecoFence's grid counts from its far
# edge.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
if (-not (Get-Variable Build -ErrorAction SilentlyContinue)) { $Build = 'new' }
$New = $Build -ne 'old'
. C:\pf\kit\deskicons.ps1
$FiNames = @('fi-a.fpi', 'fi-b.fpi', 'fi-c.fpi', 'fi-d.fpi', 'fi-e.fpi', 'fi-f.fpi', 'fi-g.fpi')
$Bin = '::{645FF040-5081-101B-9F08-00AA002F954E}'
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class FiScr {
  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, ref RC r, uint f);
  [StructLayout(LayoutKind.Sequential)] public struct PT { public int X, Y; }
  [DllImport("user32.dll")] static extern IntPtr MonitorFromPoint(PT p, uint f);
  [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr m, int t, out uint x, out uint y);
  // The primary monitor's scale now (a process's system DPI stays what it was at its start).
  public static uint Dpi() { uint x, y; GetDpiForMonitor(MonitorFromPoint(new PT(), 1), 0, out x, out y); return x; }
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string c, string t);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static int[] Work() { var r = new RC(); SystemParametersInfoW(0x30, 0, ref r, 0); return new int[] { r.L, r.T, r.R, r.B }; }
}
'@

# ---- Windows' side
# Each Windows icon by the name PecoFence lists it under (file name, or the Recycle Bin's name).
function FiWin() {
  $h = @{}
  foreach ($l in @([DeskIcons]::List())) {
    $p = $l -split '\|'
    $key = if ($p[0] -like '::*') { $p[3] } else { Split-Path -Leaf $p[0] }
    $h[$key] = @([int]$p[1], [int]$p[2])
  }
  return $h
}
function FiWinText() { $w = FiWin; return (($w.Keys | Sort-Object | ForEach-Object { "$_@$($w[$_][0]),$($w[$_][1])" }) -join ' ') }
# Windows' grid spacing (px) on the primary monitor: Explorer reports it in the scale it started
# at, the icons stand at the monitor's scale.
function FiSpacing() {
  $s = ([DeskIcons]::Spacing() -split ' ')[0] -split ','; $e = [DeskIcons]::ExplorerDpi(); $k = if ($e) { [FiScr]::Dpi() / $e } else { 1 }
  return @([int][math]::Round([int]$s[0] * $k), [int][math]::Round([int]$s[1] * $k))
}
# Windows' grid: column c, row r -> the icon's top-left (its first icon is 14, 2 in at 100 %).
function FiWinPt($c, $r) { $s = FiSpacing; $k = [FiScr]::Dpi() / 96; return @([int]([math]::Round(14 * $k) + $c * $s[0]), [int]([math]::Round(2 * $k) + $r * $s[1])) }
function FiPlace($name, $c, $r) { $p = FiWinPt $c $r; $o = [DeskIcons]::Place($name, $p[0], $p[1]); Log "place $name at Windows cell $c,$r ($($p -join ',')): $o" }
# A real mouse drag of a Windows icon (from the centre of its icon) to Windows' cell c, r.
function FiAt($name) {
  foreach ($l in @([DeskIcons]::List())) { $p = $l -split '\|'; if ($p[0] -eq $name -or $p[0] -like "*\$name") { return @([int]$p[1], [int]$p[2]) } }
  return $null
}
function FiDrag($name, $c, $r) {
  $o = [int](24 * [FiScr]::Dpi() / 96); $p = FiAt $name; $t = FiWinPt $c $r
  if (-not $p) { Log "drag ${name}: not on the desktop"; return }
  $wa = [FiScr]::Work()
  [G]::Click(800, $wa[3] - 20); Start-Sleep -Milliseconds 200
  DragTo ($p[0] + $o) ($p[1] + $o) ($t[0] + $o) ($t[1] + $o) 20
  Start-Sleep -Milliseconds 800
  Log "drag $name to Windows cell ${c},${r}: now at $((FiAt $name) -join ',')"
}
# A graceful Explorer exit and a new start (also how Explorer takes a new scale's grid).
function FiRestartExplorer() {
  $t = [FiScr]::FindWindowW('Shell_TrayWnd', [NullString]::Value)
  [FiScr]::PostMessageW($t, 0x5B4, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
  for ($i = 0; $i -lt 50 -and (Get-Process explorer -ErrorAction SilentlyContinue); $i++) { Start-Sleep -Milliseconds 200 }
  Start-Process explorer.exe
  for ($i = 0; $i -lt 60; $i++) { Start-Sleep -Milliseconds 500; try { if (@([DeskIcons]::List()).Count -gt 0) { break } } catch {} }
  Start-Sleep -Seconds 4
  Log "explorer restarted; Windows icons: $(FiWinText)"
}

# ---- PecoFence's side
function FiItems() { return @(@(PfJson item list --fence inbox) | Where-Object { $_ -and $_.id }) }
function FiCells() {
  $h = @{}; $g = FenceGet inbox
  if ($g.desktopCells) { foreach ($p in $g.desktopCells.PSObject.Properties) { $h[$p.Name] = "$($p.Value[0]),$($p.Value[1])" } }
  return $h
}
function FiAll() { $c = FiCells; return ((FiItems | Sort-Object fileName | ForEach-Object { "$($_.fileName)=$($c["$($_.id)"])" }) -join ' ') }
function FiCell($name) { $c = FiCells; $it = FiItems | Where-Object { $_.fileName -eq $name } | Select-Object -First 1; if ($it) { return $c["$($it.id)"] }; return $null }
function FiDupes() { return ((FiCells).Values | Group-Object | Where-Object { $_.Count -gt 1 } | ForEach-Object { $_.Name }) -join ' ' }
# PecoFence's grid now: cells of 80 x 96 DIPs, 4 DIPs in from the work area's corner.
function FiGrid() {
  $k = [FiScr]::Dpi() / 96; $wa = [FiScr]::Work()
  $cw = [int][math]::Round(80 * $k); $ch = [int][math]::Round(96 * $k); $e = [int][math]::Round(4 * $k)
  return @{ k = $k; wa = $wa; cw = $cw; ch = $ch; e = $e; cols = [math]::Max(1, [math]::Floor(($wa[2] - $wa[0] - $e) / $cw)); rows = [math]::Max(1, [math]::Floor(($wa[3] - $wa[1] - $e) / $ch)) }
}
# The cell rule: Windows' column and row; one past this grid's edge counts from the far edge.
function FiExpect($x, $y, $grid, $sp) {
  $wa = $grid.wa
  $c = [math]::Floor(($x - $wa[0] + [math]::Floor($sp[0] / 2)) / $sp[0]); $r = [math]::Floor(($y - $wa[1] + [math]::Floor($sp[1] / 2)) / $sp[1])
  $tc = [math]::Floor(($wa[2] - $wa[0]) / $sp[0]); $tr = [math]::Floor(($wa[3] - $wa[1]) / $sp[1])
  if ($c -ge $grid.cols) { $c = [math]::Max($grid.cols - [math]::Max($tc - $c, 1), [math]::Floor($grid.cols / 2)) }
  if ($r -ge $grid.rows) { $r = [math]::Max($grid.rows - [math]::Max($tr - $r, 1), [math]::Floor($grid.rows / 2)) }
  return "$c,$r"
}
# Whether cell "c,r" lies under another fence's window.
function FiUnder($cell, $grid) {
  $p = $cell -split ','; $l = $grid.wa[0] + $grid.e + [int]$p[0] * $grid.cw; $t = $grid.wa[1] + $grid.e + [int]$p[1] * $grid.ch
  foreach ($f in @(@(PfJson fence list) | Where-Object { $_ -and $_.kind -ne 'inbox' -and $_.windowRect })) {
    $r = $f.windowRect
    if ($l -lt $r.x + $r.w -and $r.x -lt $l + $grid.cw -and $t -lt $r.y + $r.h -and $r.y -lt $t + $grid.ch) { return $f.title }
  }
  return ''
}
# Every inbox icon against the rule, from the Windows places $win read before PecoFence started:
# "ok" icons have their expected cell; an expected cell under another fence must give a free
# cell that is not under one. Returns @(ok count, problems).
function FiCompare($win) {
  $grid = FiGrid; $sp = FiSpacing; $cells = FiCells; $ok = 0; $bad = @()
  foreach ($it in FiItems) {
    $name = $it.fileName; $got = $cells["$($it.id)"]
    if (-not $win.ContainsKey($name)) { $bad += "$name not on Windows' desktop (cell $got)"; continue }
    $want = FiExpect $win[$name][0] $win[$name][1] $grid $sp
    $under = FiUnder $want $grid
    if ($under) {
      if ($got -and $got -ne $want -and -not (FiUnder $got $grid)) { $ok++ } else { $bad += "$name want $want (under $under) got $got" }
    } elseif ($got -eq $want) { $ok++ } else { $bad += "$name want $want got $got" }
  }
  return @($ok, ($bad -join '; '))
}
# App log lines of this job, oldest first (the shareable logs repeat them, so they are left out).
function FiLogLines($regex) {
  return @(PfLogFiles | Where-Object { $_.Name -notmatch '\.share\.' } | ForEach-Object { Get-Content $_.FullName -Encoding UTF8 } | Where-Object { $_ -match $regex })
}
function Park() { [G]::MoveTo(1560, 820); Start-Sleep -Milliseconds 300 }

# A user's own layout, dragged with the mouse: $targets is @(name, column, row) on Windows' grid.
# Every icon goes out of the way first (to the columns from $stage on, $rows to a column:
# Windows moves an icon dropped on a taken cell elsewhere), the other icons (Edge, EC2 links:
# they go into Programs) to the top of the first column.
function FiLayout($targets, $stage, $rows) {
  foreach ($n in $FiNames) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  foreach ($n in $FiNames) { DeskFile $n | Out-Null }
  Start-Sleep -Seconds 3
  $all = @(@([DeskIcons]::List()) | ForEach-Object { ($_ -split '\|')[0] })
  for ($i = 0; $i -lt $all.Count; $i++) { FiPlace $all[$i] ($stage + [math]::Floor($i / $rows)) ($i % $rows) }
  Start-Sleep -Seconds 2
  $r = 1
  foreach ($n in @($all | Where-Object { $_ -notlike '::*' -and $FiNames -notcontains (Split-Path -Leaf $_) } | Sort-Object)) { FiDrag $n 0 $r; $r++ }
  foreach ($t in $targets) { FiDrag $t[0] $t[1] $t[2] }
  Park
  Start-Sleep -Seconds 2
  $sp = FiSpacing
  Log "Windows layout: $(FiWinText) (spacing $($sp -join 'x'), Explorer started at $([DeskIcons]::ExplorerDpi()) dpi)"
  # Within half a cell of the place asked for (Windows' first row is a px or two off at 150 %).
  $off = @($targets | Where-Object { $p = FiAt $_[0]; $w = FiWinPt $_[1] $_[2]; -not $p -or [math]::Abs($p[0] - $w[0]) -ge $sp[0] / 2 -or [math]::Abs($p[1] - $w[1]) -ge $sp[1] / 2 } | ForEach-Object { $_[0] })
  Check 'the Windows layout stands in the cells it was dragged to' ($off.Count -eq 0) "off: $($off -join ', '); $(FiWinText)"
}
# Part A and the recording at 100 % (Windows: 21 columns, 8 rows; PecoFence 19 and 8): gaps in
# a column, the bottom row, the far right past PecoFence's columns, the middle, one where the
# first start puts the Programs fence, the Recycle Bin moved.
$Layout100 = @(@('fi-a.fpi', 3, 0), @('fi-b.fpi', 3, 1), @('fi-c.fpi', 3, 3), @('fi-d.fpi', 0, 7), @('fi-e.fpi', 20, 7),
  @('fi-f.fpi', 10, 4), @('fi-g.fpi', 19, 0), @($Bin, 6, 2))

if ($Part -eq 'A') {
  Case 'first start takes the Windows places'
  [DeskIcons]::SetFlags(1, 0) | Out-Null
  FiLayout $Layout100 12 7
  $win = FiWin
  Shot "$Build-a1-windows-layout"
  PfStart -Script (DumpScript 300 14)
  Start-Sleep -Seconds 3
  Park
  Log "inbox after the first start: $(FiAll)"
  $cmp = FiCompare $win
  Shot "$Build-a2-first-start"
  if ($New) {
    Check 'every inbox icon has a cell' ((FiCells).Count -eq (FiItems).Count -and (FiItems).Count -eq 8) (FiAll)
    Check 'icons keep their Windows column and row (gaps, bottom, far right, middle)' ($cmp[0] -eq 8) "ok $($cmp[0]) of 8; $($cmp[1]); $(FiAll)"
    Check 'the far-right icon is in the last column' ((FiCell 'fi-e.fpi') -eq "$((FiGrid).cols - 1),7") (FiAll)
    Check 'no two icons share a cell' (-not (FiDupes)) "dupes: $(FiDupes)"
    $read = @(FiLogLines 'Windows desktop icon places read')
    $shown = @(FiLogLines 'reanchor reason="after show"')
    Check 'Windows places were taken before the fences showed' ($read.Count -eq 1 -and $shown.Count -ge 1 -and (LogTime $read[0]) -and (LogTime $read[0]) -le (LogTime $shown[0])) "read: $($read -join ' | ') shown: $($shown -join ' | ')"
    $taken = @(FiLogLines 'cells taken from the Windows desktop')
    Check 'the log says how many took the Windows place' ($taken.Count -eq 1 -and $taken[0] -match 'windows_place=7 nearest_free=1' -and $taken[0] -match 'not_on_windows=0') ($taken -join ' | ')
  } else {
    Check 'old build: icons fill the columns from the top-left' ((FiCell 'fi-a.fpi') -match '^0,') "ok $($cmp[0]) of 8; $(FiAll)"
  }

  if ($New) {
    Case 'a restart keeps the cells'
    $before = FiAll
    PfQuit
    Log "Windows icons after the quit: $(FiWinText)"
    PfStart -Script (DumpScript 300 14)
    Start-Sleep -Seconds 3
    Check 'every icon is where it was before the restart' ((FiAll) -eq $before) "before $before | now $(FiAll)"
    Check 'the Windows desktop is not read again' ((FiLogLines 'reading the Windows desktop').Count -eq 1) ((FiLogLines 'reading the Windows desktop') -join ' | ')

    Case 'a sort lines the icons up and is not undone'
    Pf fence set inbox sort name | Out-Null
    Start-Sleep -Seconds 2
    $names = @(FiItems | ForEach-Object { $_.fileName })
    Log "after sort: $(FiAll) (order $($names -join ', '))"
    $col = @(FiItems | ForEach-Object { FiCell $_.fileName })
    Check 'sorting puts the icons down the first column' (($col | Where-Object { $_ -match '^0,' }).Count -eq 8) (FiAll)
    Check 'the Windows desktop is not read after the sort' ((FiLogLines 'reading the Windows desktop').Count -eq 1) ''
    $sorted = FiAll
    PfQuit; PfStart -Script (DumpScript 300 14); Start-Sleep -Seconds 3
    Check 'the sorted cells stay after a restart' ((FiAll) -eq $sorted) "before $sorted | now $(FiAll)"

    Case 'switching it on in an existing configuration'
    # A configuration from before: free placement off and no cells saved.
    PfQuit
    $cfg = "$PfApp\config\config.json"
    # Edited as text: ConvertTo-Json would round the 128-bit file ids.
    $t = [IO.File]::ReadAllText($cfg)
    $t = [regex]::Replace($t, '"desktopCells"\s*:\s*\{[^}]*\}\s*,', '')
    $t = [regex]::Replace($t, '"freePlacement"\s*:\s*true', '"freePlacement": false')
    [IO.File]::WriteAllText($cfg, $t, (New-Object Text.UTF8Encoding($false)))
    Log "config edited: cells $(([regex]::Matches($t, 'desktopCells')).Count), free placement on $(([regex]::Matches($t, '"freePlacement"\s*:\s*true')).Count)"
    PfStart -Script (DumpScript 300 14)
    Start-Sleep -Seconds 3
    Check 'the old configuration has it off and no cells' ((FenceGet inbox).freePlacement -eq $false -and (FiCells).Count -eq 0) (FiAll)
    # Windows' icons are hidden now: the view reports them down its columns.
    $hiddenWin = FiWin
    Log "Windows places while hidden: $(FiWinText)"
    $reads = (FiLogLines 'reading the Windows desktop').Count
    Pf fence set inbox freePlacement true | Out-Null
    Start-Sleep -Seconds 2
    Park
    Log "after switching on: $(FiAll)"
    $cmp = FiCompare $hiddenWin
    Check 'switching on takes the places Windows has now' ($cmp[0] -eq 8 -and (FiCells).Count -eq 8) "ok $($cmp[0]) of 8; $($cmp[1]); $(FiAll)"
    Check 'switching on read the Windows desktop once' ((FiLogLines 'reading the Windows desktop').Count -eq $reads + 1) ''
    Shot 'a3-switched-on'
    $on = FiAll
    Pf fence set inbox freePlacement false | Out-Null; Start-Sleep -Seconds 1
    Pf fence set inbox freePlacement true | Out-Null; Start-Sleep -Seconds 2
    Check 'off and on again keeps the cells, no new read' ((FiAll) -eq $on -and (FiLogLines 'reading the Windows desktop').Count -eq $reads + 1) "before $on | now $(FiAll)"

    Case 'Windows auto-arrange on'
    PfQuit
    Log "auto-arrange on: $([DeskIcons]::SetFlags(1, 1))"
    Start-Sleep -Seconds 2
    $win = FiWin
    Log "Windows icons auto-arranged: $(FiWinText)"
    PfFresh -Script (DumpScript 300 14)
    Start-Sleep -Seconds 3
    Park
    Log "inbox: $(FiAll)"
    $cmp = FiCompare $win
    Check 'with auto-arrange the icons keep Windows'' column order' ($cmp[0] -eq 8) "ok $($cmp[0]) of 8; $($cmp[1]); $(FiAll)"
    Check 'no two icons share a cell (auto-arrange)' (-not (FiDupes)) "dupes: $(FiDupes)"
    $readLine = @(FiLogLines 'Windows desktop icon places read') | Select-Object -Last 1
    Check 'the log says Windows arranges its icons itself' ($readLine -match 'auto_arrange=true') $readLine
    Shot 'a4-auto-arrange'
    PfQuit
    [DeskIcons]::SetFlags(1, 0) | Out-Null
  }
}

if ($Part -eq 'C') {
  # The first start at 150 % (fx-free-placement.sh answers the scale request).
  Case 'first start at 150 %'
  Set-Content C:\pf\resize-want.txt '1600x900@150' -Encoding ASCII
  for ($k = 0; $k -lt 150 -and (Test-Path C:\pf\resize-want.txt); $k++) { Start-Sleep -Seconds 1 }
  # Explorer lays its icons out again for the new scale a little later.
  Start-Sleep -Seconds 10
  Log "dpi $([FiScr]::Dpi()), work area $([FiScr]::Work() -join ','), Windows spacing $([DeskIcons]::Spacing())"
  [DeskIcons]::SetFlags(1, 0) | Out-Null
  # Windows has 14 columns and 5 rows at 150 % (114 x 156 px), PecoFence 13 and 5.
  FiLayout @(@('fi-a.fpi', 2, 0), @('fi-b.fpi', 2, 1), @('fi-c.fpi', 2, 3), @('fi-d.fpi', 0, 4), @('fi-e.fpi', 13, 4),
    @('fi-f.fpi', 6, 2), @('fi-g.fpi', 5, 0), @($Bin, 4, 1)) 9 5
  $win = FiWin
  Shot 'c1-windows-150'
  PfStart -Script (DumpScript 300 14)
  Start-Sleep -Seconds 4
  Park
  $g = FiGrid
  Log "inbox at 150 %: $(FiAll); grid $($g.cols)x$($g.rows) of $($g.cw)x$($g.ch)"
  $cmp = FiCompare $win
  Check 'at 150 % icons keep their Windows column and row' ($cmp[0] -eq 8) "ok $($cmp[0]) of 8; $($cmp[1]); $(FiAll)"
  Check 'no two icons share a cell at 150 %' (-not (FiDupes)) "dupes: $(FiDupes)"
  Shot 'c2-first-start-150'
  PfQuit
  Set-Content C:\pf\resize-want.txt '1600x900' -Encoding ASCII
  for ($k = 0; $k -lt 150 -and (Test-Path C:\pf\resize-want.txt); $k++) { Start-Sleep -Seconds 1 }
  Start-Sleep -Seconds 8
  # Explorer keeps the icon spacing of 150 % until it restarts (as until a sign-out): restart it,
  # so the next jobs find the 100 % grid.
  FiRestartExplorer
  Log "Windows spacing back at 100 %: $([DeskIcons]::Spacing())"
}

if ($Part -eq 'R') {
  # Before/after: a user's own layout, then PecoFence's first start.
  . C:\pf\kit\rec.ps1
  Case "recording ($Build)"
  [DeskIcons]::SetFlags(1, 0) | Out-Null
  FiLayout $Layout100 12 7
  Park
  RecStart "fi-$Build" -X 0 -Y 0 -W 1600 -H 900 -Fps 10
  RecMark '这是 Windows 桌面上的图标，位置是自己摆的'
  Start-Sleep -Seconds 3
  RecMark '第一次打开 PecoFence'
  PfStart -Script (DumpScript 300 14) -Lang 'zh-CN'
  Start-Sleep -Seconds 2
  Park
  RecMark '没进栅栏的图标在哪里'
  Start-Sleep -Seconds 4
  RecStop
  Log "inbox: $(FiAll)"
  Check 'recorded' $true ''
}

Case 'end'
PfQuit
[DeskIcons]::SetFlags(1, 0) | Out-Null
foreach ($n in $FiNames) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue
