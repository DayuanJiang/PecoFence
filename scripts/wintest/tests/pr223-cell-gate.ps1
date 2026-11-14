# PR #223: a cell of a row of cells (a `::` line) follows main's rule for rows that start
# programs: run= in a cell starts nothing unless the script's header allows it, and the click is
# refused with the app's log line; with the header line it starts the program. Notepad is the
# harmless payload. A plain row and a button with action= keep working.
#   run.sh scripts/wintest/tests/pr223-cell-gate.ps1

$Allow = '<pecofence.allow>run</pecofence.allow>'
function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function WaitCells($title, $n, $sec = 30) {
  for ($k = 0; $k -lt $sec * 2; $k++) { $w = DumpOf $title 3; if (@(CellRects $w).Count -ge $n) { return $w }; Start-Sleep -Milliseconds 300 }
  return $w
}
# Clicks cell `$cell` of output line `$line` of fence `$title`.
function ClickCell($title, $line, $cell) {
  $w = DumpOf $title 5
  $r = @(CellRects $w | Where-Object { $_.i -eq $line * 100 + $cell }) | Select-Object -First 1
  if (-not $r) { Log "no cell $line/$cell in $(DumpVal $w 'script_cells')"; return $false }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  $x = [int]($w.l + [double](DumpVal $w 'content_x') + ($r.l + $r.r) / 2 * $sc)
  $y = [int]($w.t + [double](DumpVal $w 'content_y') + ($r.t + $r.b) / 2 * $sc)
  [G]::Click($x, $y); Start-Sleep -Milliseconds 900
  ParkMouse
  return $true
}
function Notepads() { return @(Get-Process notepad -ErrorAction SilentlyContinue) }
function EndNotepads() { Notepads | Stop-Process -Force -ErrorAction SilentlyContinue; Start-Sleep -Milliseconds 400 }
function WaitNotepad($sec = 4) {
  for ($k = 0; $k -lt $sec * 5; $k++) { if ((Notepads).Count) { return $true }; Start-Sleep -Milliseconds 200 }
  return $false
}
function Refusals() { return @(PfLogAll | Where-Object { $_ -match 'script row refused.* key="run"' }).Count }
function ActionLines($file) { if (Test-Path $file) { return @(Get-Content $file) } else { return @() } }

# A script that prints a row of cells and writes each action it gets into a file (it runs again
# for each click, with the action in PECOFENCE_ACTION).
function GateScript($path, $header, $log) {
  $text = @"
# $header
if (`$env:PECOFENCE_ACTION) { Add-Content '$log' `$env:PECOFENCE_ACTION }
'Cells below'
':: Notepad | run=notepad.exe :: Hello | action=hello'
"@
  Set-Content $path -Value $text -Encoding UTF8
}

EndNotepads
PfStart -Script (DumpScript 400 12 3000)
Start-Sleep -Seconds 2
ClearFences
$plain = "$PfRun\gate-plain.ps1"; $plainLog = "$PfRun\gate-plain.txt"
$allowed = "$PfRun\gate-allowed.ps1"; $allowedLog = "$PfRun\gate-allowed.txt"
GateScript $plain 'cells, no header line' $plainLog
GateScript $allowed $Allow $allowedLog
Pf fence create --script $plain --title GatePlain --rect '120,110,520,300' | Out-Null
Pf fence create --script $allowed --title GateAllowed --rect '620,110,1020,300' | Out-Null
$w1 = WaitCells 'GatePlain' 2
$w2 = WaitCells 'GateAllowed' 2
Check 'both fences draw their two cells' (@(CellRects $w1).Count -ge 2 -and @(CellRects $w2).Count -ge 2) "plain=$(DumpVal $w1 'script_cells') allowed=$(DumpVal $w2 'script_cells')"

Case 'without the header line'
$before = Refusals
$clicked = ClickCell 'GatePlain' 1 0
$started = WaitNotepad 3
Shot 'plain-refused'
Check 'a cell with run= starts no program' ($clicked -and -not $started) "clicked=$clicked notepads=$((Notepads).Count)"
Check 'and the app logs the refusal' ((Refusals) -gt $before) "refusals $before -> $(Refusals)"
EndNotepads
$clicked = ClickCell 'GatePlain' 1 1
Start-Sleep -Seconds 3
Check 'a cell with action= still reaches the script' ($clicked -and ((ActionLines $plainLog) -contains 'hello')) "clicked=$clicked actions=$((ActionLines $plainLog) -join ',')"

Case 'with the header line'
$clicked = ClickCell 'GateAllowed' 1 0
$started = WaitNotepad 5
Check 'a cell with run= starts the program' ($clicked -and $started) "clicked=$clicked notepads=$((Notepads).Count)"
EndNotepads
PfQuit
