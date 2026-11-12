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
