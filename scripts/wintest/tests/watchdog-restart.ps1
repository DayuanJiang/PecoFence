# Fence fix round 2026-11-07, area config-undo, C14: the watchdog starts PecoFence again after a
# crash, after Windows ended it as not responding (0xCFFFFFFF) and once after an early crash, but
# not again when the restarted run crashes early too. The watchdog starts PecoFence first, so
# Windows' own icons do not show in between; they come back when it starts nothing. It also
# runs when PecoFence leaves the Windows icons alone (--no-hide-icons). About 5 minutes.
$PfAllowLogErrors = $true
$stamp = "$env:LOCALAPPDATA\PecoFence\icons-hidden.$PfInst.restarted"
Remove-Item $stamp -ErrorAction SilentlyContinue
# -Running: crash the PecoFence the watchdog started again instead of a new one.
function WdRun([uint32]$code, [int]$wait, [string]$label, [bool]$want = $true, [switch]$Running, [switch]$NoHide) {
  trap { PfTrap $_; continue }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  $al = @('--portable'); if ($NoHide) { $al += '--no-hide-icons' }
  if (-not $Running) { Start-Process "$PfApp\pecofence.exe" -ArgumentList $al -WorkingDirectory $PfApp }
  $null = WaitWin 'PecoFence.Control' 20 -Hidden
  Start-Sleep -Seconds $wait
  $p1 = Get-Process pecofence | Select-Object -First 1
  $wd = @(Get-Process pecofence-watchdog -ErrorAction SilentlyContinue | ForEach-Object Id) -join ','
  $hidden = -not [W]::IconsVisible()
  $t0 = Get-Date
  [Th]::Terminate($p1.Id, $code) | Out-Null
  $p2 = $null; $iconsBack = $null
  for ($i = 0; $i -lt 100; $i++) {
    Start-Sleep -Milliseconds 250
    if ($null -eq $iconsBack -and [W]::IconsVisible()) { $iconsBack = [int]((Get-Date) - $t0).TotalMilliseconds }
    $p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $p1.Id } | Select-Object -First 1
    if ($p2) { break }
  }
  $after = [int]((Get-Date) - $t0).TotalMilliseconds
  $hex = '{0:X8}' -f $code
  Log "C14 $label code $hex ran ${wait}s: watchdogs=[$wd] icons hidden before=$hidden; icons back after $iconsBack ms; restarted=$([bool]$p2) after $after ms"
  $what = if ($want) { 'started again' } else { 'not started again' }
  Check "C14 $label (exit $hex after ${wait} s): $what" ([bool]$p2 -eq $want) "restarted=$([bool]$p2) watchdogs=[$wd] icons hidden before kill=$hidden icons back after ${iconsBack} ms; waited $after ms; stamp: $(Get-Content $stamp -ErrorAction SilentlyContinue)"
  if ($want -and $p2 -and $hidden) {
    # The new run takes over the icons the crashed one hid: Windows' own never show.
    Start-Sleep -Milliseconds 2500
    Check "C14 ${label}: Windows' own icons did not show between the crash and the restart" ($null -eq $iconsBack -and -not [W]::IconsVisible()) "seen at $iconsBack ms after the crash (empty = never); hidden 2.5 s after the restart=$(-not [W]::IconsVisible())"
  }
  if (-not $want -and $hidden) {
    Check "C14 ${label}: nothing was started, so Windows' own icons are shown again" ($null -ne $iconsBack) "icons back after $iconsBack ms"
  }
  if ($p2) {
    Start-Sleep -Seconds 4
    $line = Get-Content (PfLogPath) -Encoding UTF8 -TotalCount 5 | Where-Object { $_ -match 'process started' } | Select-Object -First 1
    Log "C14 restarted run: $line"
  }
}
function WdEnd() {
  if (Get-Process pecofence -ErrorAction SilentlyContinue) { PfQuit }
  Get-Process pecofence -ErrorAction SilentlyContinue | ForEach-Object { [Th]::Terminate($_.Id, 1) | Out-Null }
  Start-Sleep -Seconds 12
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons() }
  Start-Sleep -Seconds 1
}

Case 'C14 watchdog restarts'
# A first run makes the config (and its first-run work) so the timed runs are plain restarts.
PfStart
PfQuit
WdRun 3221225477 15 'early crash'
# The run started again crashes early too: a start that keeps crashing must not loop.
WdRun 3221225477 10 'early crash of the restarted run' $false -Running
WdEnd
WdRun 3221225477 62 'crash'
WdEnd
WdRun 3489660927 62 'hang kill'
WdEnd
# With the Windows icons left alone no watchdog ran, and nothing started PecoFence again.
Remove-Item $stamp -ErrorAction SilentlyContinue
WdRun 3221225477 15 'crash with the Windows icons not hidden' $true -NoHide
WdEnd
