# Fence fix round 2026-11-07, area render-perf, S15: on a PC without a GPU (the EC2 boxes and the
# Hyper-V guests), rolling a fence up and down kept 35 to 65 KB of memory each time, which only a
# rebuild of the render stack gave back. Now PecoFence watches its private memory there and
# rebuilds at an idle point once it grew by 8 MB. About 5 minutes.
$PfAllowLogErrors = $true
[Anim]::Set($true)

Case 'S15: 300 rolls and expands of a fence, then an idle minute, keep no memory (CLI, as in the review)'
PfStart
Pf fence create --title C --rect '800,200,220,150' | Out-Null
function Q15 { & "$PfApp\pecofence-cli.exe" @args 2>&1 | Out-Null }
function Priv15 { $p = Get-Process pecofence | Where-Object { $_.Path -like "$PfApp\*" } | Select-Object -First 1; $p.Refresh(); return [math]::Round($p.PrivateMemorySize64 / 1KB) }
for ($i = 0; $i -lt 5; $i++) { Q15 fence roll C; Start-Sleep -Milliseconds 300; Q15 fence unroll C; Start-Sleep -Milliseconds 300 }
Start-Sleep -Seconds 40
$m0 = Priv15
for ($i = 0; $i -lt 300; $i++) { Q15 fence roll C; Start-Sleep -Milliseconds 300; Q15 fence unroll C; Start-Sleep -Milliseconds 300 }
Start-Sleep -Seconds 8
$m1 = Priv15
Start-Sleep -Seconds 40
$m2 = Priv15
$reb = @(PfLogAll | Where-Object { $_ -match 'render stack rebuilt to hand back' } | ForEach-Object { $_ -replace '^.*pecofence', 'pecofence' })
Check 'S15: an idle minute after 300 rounds the private memory is less than 3 MB above where it started (10 to 19 MB before)' (($m2 - $m0) -lt 3072) "private $m0 -> $m1 (after the rounds) -> $m2 KB (after the idle point); $($reb -join ' | ')"
Check 'S15: the hand-back rebuilt the render stack and logged it' ($reb.Count -ge 1) "rebuild lines: $($reb.Count)"
PfQuit
