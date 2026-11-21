# PR #223's script fence features, with a test script (no built-in example uses them now): the
# title from <pecofence.title>, PECOFENCE_LANGUAGE, a row of cells (an icon button, a check box,
# a filling cell, a cell under another), a picture cell and a picture row, 120 rows that scroll
# under a pane= row (more than the 40 lines of a script that is not streaming), and clicks that
# reach the script's standard input. Then the fence is deleted: its script ends, and its data
# folder goes at the next start.
#   run.sh scripts/wintest/tests/pr223-cells.ps1

function CellRects($w, $field = 'script_cells') {
  $v = DumpVal $w $field
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function Cell($w, $i) { return @(CellRects $w | Where-Object { $_.i -eq $i }) | Select-Object -First 1 }
# Screen point of content DIPs (x, y) of a dumped window.
function ContentPt($w, $x, $y) {
  $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + $x * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + $y * $sc))
}
function ClickCell($title, $i) {
  $w = DumpOf $title 5; $r = Cell $w $i
  if (-not $r) { Log "no cell $i in $(DumpVal $w 'script_cells')"; return $false }
  $p = ContentPt $w (($r.l + $r.r) / 2) (($r.t + $r.b) / 2)
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 900; ParkMouse
  return $true
}
function WaitOut($pattern, $sec = 10) {
  for ($k = 0; $k -lt $sec * 4; $k++) { if ((FenceOut $script:Id) -match $pattern) { return $true }; Start-Sleep -Milliseconds 250 }
  return $false
}
function Red($w, $x, $y) {
  $p = ContentPt $w $x $y; $b = [Px]::Grab($p[0] - 3, $p[1] - 3, 6, 6); $c = MeanRgb $b; $b.Dispose()
  return @{ ok = ($c[0] -gt 180 -and $c[1] -lt 90 -and $c[2] -lt 90); rgb = ($c -join ' ') }
}
function ScriptProc() { return @(Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" | Where-Object { $_.CommandLine -match 'cellstest' }).Count }

$path = "$PfRun\cellstest.ps1"
Set-Content $path -Encoding UTF8 -Value @'
# <pecofence.type>streamable</pecofence.type>
# <pecofence.title>CellsTest</pecofence.title>
Add-Type -AssemblyName System.Drawing
$png = Join-Path $env:PECOFENCE_DATA 'red.png'
$b = New-Object Drawing.Bitmap 200, 80; $g = [Drawing.Graphics]::FromImage($b); $g.Clear([Drawing.Color]::Red); $g.Dispose(); $b.Save($png); $b.Dispose()
$in = New-Object IO.StreamReader([Console]::OpenStandardInput())
$read = $in.ReadLineAsync(); $got = '-'; $on = $false
function Show {
  $rows = @("lang=$env:PECOFENCE_LANGUAGE got=$got data=$env:PECOFENCE_DATA",
    ":: $([char]0xE768) | action=play :: Shuffle | checkbox=$("$on".ToLower()) action=shuffle :: Wide | fill=true action=wide :: Top | action=top :: Low | under=true action=low",
    ":: | image=""$png"" height=40 width=120 action=pic :: Beside | action=beside",
    " | image=""$png"" height=60",
    ' | pane=list hidden=true')
  1..120 | ForEach-Object { $rows += "Row $_ | action=row:$_" }
  Write-Output ("~~~`n" + ($rows -join "`n"))
}
Show
while ($true) {
  if ($read.IsCompleted) {
    $line = $read.Result; if ($null -eq $line) { break }
    $read = $in.ReadLineAsync(); $got = $line; if ($line -eq 'shuffle') { $on = -not $on }; Show
  }
  Start-Sleep -Milliseconds 60
}
'@

PfStart -Script (DumpScript 400 12 3000)
Start-Sleep -Seconds 2
ClearFences
Pf fence create --script $path --rect '200,110,420,560' | Out-Null
FenceIds; $script:Id = $script:Id['CellsTest']
Check 'the fence takes the title of <pecofence.title>' ([bool]$script:Id) "fences: $($script:IdTitle.Values -join ', ')"
$started = WaitOut 'Row 120' 20
$lines = @((FenceOut $script:Id) -split "`n")
Check 'all 125 lines of the streaming screen are kept' ($started -and $lines.Count -ge 125) "lines=$($lines.Count)"
Check 'PECOFENCE_LANGUAGE is the app language' ($lines[0] -match '^lang=en ') $lines[0]
$data = if ($lines[0] -match 'data=(.+)$') { $Matches[1].Trim() } else { '' }
Start-Sleep -Seconds 2
$w = DumpOf 'CellsTest' 5
Shot 'cells'

Case 'cells'
$c = @(100..104 | ForEach-Object { Cell $w $_ })
$fill = $c[2].r - $c[2].l; $top = $c[3]; $low = $c[4]
Check 'the row has its five cells' (@($c | Where-Object { $_ }).Count -eq 5) (DumpVal $w 'script_cells')
Check 'the filling cell is the widest' ($fill -gt 2 * ($c[1].r - $c[1].l)) "fill=$fill shuffle=$($c[1].r - $c[1].l)"
Check 'under=true puts Low under Top, in its column' ($low.t -ge $top.b - 1 -and [math]::Abs($low.l - $top.l) -lt 2) "top=$($top.l),$($top.t),$($top.b) low=$($low.l),$($low.t)"
$clicked = ClickCell 'CellsTest' 101
Check 'a click on the check box reaches the script' ($clicked -and (WaitOut 'got=shuffle' 5) -and (FenceOut $script:Id) -match 'checkbox=true') ((FenceOut $script:Id) -split "`n" | Select-Object -First 2)
$clicked = ClickCell 'CellsTest' 100
Check 'a click on the icon button reaches the script' ($clicked -and (WaitOut 'got=play' 5)) ((FenceOut $script:Id) -split "`n" | Select-Object -First 1)

Case 'pictures'
$w = DumpOf 'CellsTest' 5
$pic = Cell $w 200
$inCell = if ($pic) { Red $w (($pic.l + $pic.r) / 2) (($pic.t + $pic.b) / 2) } else { @{ ok = $false } }
Check 'the picture cell draws the picture' ($pic -and $inCell.ok -and ($pic.b - $pic.t) -ge 38) "cell=$($pic.l),$($pic.t),$($pic.r),$($pic.b) rgb=$($inCell.rgb)"
$rows = @(CellRects $w 'script_rows'); $first = $rows | Where-Object { $_.i -eq 5 } | Select-Object -First 1
$mid = if ($pic -and $first) { Red $w (($first.l + $first.r) / 2) (($pic.b + $first.t) / 2) } else { @{ ok = $false } }
Check 'the picture row draws the picture between the cells and the list' ($mid.ok) "rgb=$($mid.rgb) first row=$($first.t)"

Case 'scrolling'
$before = [double](DumpVal $w 'scroll')
$p = ContentPt $w (($first.l + $first.r) / 2) ($first.t + 60)
Wheel $p[0] $p[1] -3; Start-Sleep -Seconds 1; ParkMouse
$w2 = DumpOf 'CellsTest' 5
$after = [double](DumpVal $w2 'scroll')
Check 'the wheel scrolls the list' ($after -gt $before) "scroll $before -> $after"
Check 'the rows above pane= stay in place' ((Cell $w2 100).t -eq $c[0].t -and (Cell $w2 200).t -eq $pic.t) "cell 100 top $($c[0].t) -> $((Cell $w2 100).t)"
Shot 'scrolled'

Case 'the fence is deleted'
Check 'the data folder holds the picture' ($data -and (Test-Path (Join-Path $data 'red.png'))) $data
Pf fence delete $script:Id | Out-Null
Start-Sleep -Seconds 3
Check 'deleting the fence ends its script' ((ScriptProc) -eq 0) "processes=$(ScriptProc)"
PfQuit
Start-Sleep -Seconds 2
$kept = Test-Path $data
PfStart -Script (DumpScript 400 12 3000)
Start-Sleep -Seconds 4
Check 'the next start removes the deleted fence''s data folder' ($kept -and -not (Test-Path $data)) "before start=$kept after=$(Test-Path $data)"
PfQuit
