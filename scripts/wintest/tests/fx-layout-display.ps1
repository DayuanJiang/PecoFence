# Fence fix round 2026-11-07, area layout-display (fence review U15): the first run lays the
# automatic fences out so every one shows all its rows (another column to the left where the
# column has no room), and the first tip card covers no fence. Runs on a 1600x900 box.
# S7 (a new monitor set takes the spots of the same monitor at the same scale) and S8 (no fence
# left on top of another after a scale change) need scale changes and a second monitor: their
# Hyper-V scripts are in the review folder, notes/fix/layout-display/scripts.

# A normal desktop: shortcuts, two folders, fourteen files (the first-run fences get 2, 1 and 5
# rows, more than one 1600x900 column holds).
$FxFiles = @('Fx report.docx', 'Fx budget.xlsx', 'Fx slides.pptx', 'Fx archive.zip', 'Fx music.mp3', 'Fx clip.mp4',
  'Fx notes.md', 'Fx data.csv', 'Fx doc 1.txt', 'Fx doc 2.txt', 'Fx doc 3.txt', 'Fx doc 4.txt', 'Fx doc 5.txt', 'Fx doc 6.txt')
$FxLinks = @('Fx Notepad.lnk', 'Fx Calculator.lnk', 'Fx Paint.lnk', 'Fx Command Prompt.lnk', 'Fx Task Manager.lnk')
$FxDirs = @('FxPhotos', 'FxProjects')
function FxClean() {
  foreach ($n in @($FxFiles) + @($FxLinks) + @($FxDirs)) {
    $p = Join-Path $PfDesk $n
    if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue }
  }
  # Explorer may still hold a shortcut it just showed: try again for a few seconds.
  foreach ($k in 1..10) {
    $left = @(Get-ChildItem $PfDesk -Filter 'FxTool *.lnk' -ErrorAction SilentlyContinue)
    if ($left.Count -eq 0) { break }
    $left | Remove-Item -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
  }
}
function FxSeed() {
  FxClean
  foreach ($n in $FxFiles) { DeskFile $n | Out-Null }
  foreach ($n in $FxLinks) { DeskLink $n | Out-Null }
  foreach ($n in $FxDirs) { New-Item -ItemType Directory -Force (Join-Path $PfDesk $n) | Out-Null }
}

# A first run as a user starts it: fresh data, no test script (the tips show).
function FxFirstRun() {
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  WaitWin 'PecoFence.Control' 20 -Hidden | Out-Null
  $n = WaitWin 'PecoFence.Notice' 30
  Start-Sleep -Seconds 4
  return $n
}

# The fences the tip card covers, and the fences that scroll.
function FxCheck($tag, $n) {
  $fences = @(@(PfJson fence list) | Where-Object { $_ -and $_.title -and $_.windowRect -and $_.kind -ne 'inbox' })
  foreach ($f in $fences) { Log "$tag fence '$($f.title)' $($f.windowRect.x),$($f.windowRect.y),$($f.windowRect.w)x$($f.windowRect.h) rows=$($f.fit.rows) overflow=$($f.fit.overflow)" }
  Check "$tag the three automatic fences show" ($fences.Count -eq 3) "fences=$($fences.Count)"
  $cover = @()
  if ($n -ne [IntPtr]::Zero) {
    $r = Rect $n
    Log "$tag tip card $($r.L),$($r.T),$($r.R),$($r.B)"
    foreach ($f in $fences) {
      $w = $f.windowRect
      $ow = [Math]::Min($r.R, $w.x + $w.w) - [Math]::Max($r.L, $w.x)
      $oh = [Math]::Min($r.B, $w.y + $w.h) - [Math]::Max($r.T, $w.y)
      if ($ow -gt 0 -and $oh -gt 0) { $cover += "'$($f.title)' ${ow}x${oh}" }
    }
  }
  Check "$tag the first tip card shows" ($n -ne [IntPtr]::Zero) "notice=$n"
  Check "$tag the tip card covers no fence" ($cover.Count -eq 0) ($cover -join '; ')
  return $fences
}

Case 'first run: every automatic fence shows all its rows'
FxSeed
$n = FxFirstRun
Shot 'fx-u15-first-run'
$fences = FxCheck 'first run' $n
$scroll = @($fences | Where-Object { $_.fit.overflow } | ForEach-Object { $_.title })
Check 'no automatic fence scrolls' ($scroll.Count -eq 0) ($scroll -join ', ')
$files = $fences | Where-Object { $_.title -eq 'Files and documents' } | Select-Object -First 1
$prog = $fences | Where-Object { $_.title -eq 'Programs' } | Select-Object -First 1
Check 'the fence that does not fit below the others starts a column to the left' ($files -and $prog -and $files.windowRect.x + $files.windowRect.w -le $prog.windowRect.x -and $files.windowRect.y -lt 100) "files=$($files.windowRect.x),$($files.windowRect.y) programs=$($prog.windowRect.x)"
Check 'the log names the columns' ([bool](PfLogAll | Select-String 'first run: fences laid out in columns' -SimpleMatch)) ''

Case 'first run with many shortcuts: the tip card moves off the fences'
1..30 | ForEach-Object { DeskLink ('FxTool {0:D2}.lnk' -f $_) | Out-Null }
$n = FxFirstRun
Shot 'fx-u15-many-shortcuts'
$fences = FxCheck 'many shortcuts' $n
$tall = @($fences | Where-Object { $_.fit.overflow } | ForEach-Object { $_.title })
Check 'only the fence taller than the screen scrolls' ($tall.Count -le 1) ($tall -join ', ')
Check 'the log says the card moved' ([bool](PfLogAll | Select-String 'notice moved off the fences' -SimpleMatch)) ''

Case 'cleanup'
if (PfAlive) { PfQuit }
FxClean
Check 'test files removed' (-not (Test-Path (Join-Path $PfDesk 'FxPhotos')) -and @(Get-ChildItem $PfDesk -Filter 'FxTool *.lnk').Count -eq 0) ''
