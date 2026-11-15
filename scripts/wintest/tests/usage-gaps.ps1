# Usage statistics, what the first week's data could not tell (2026-11-15): which corner card
# a `notice` was and how it ended; 对话框跳转's jumps that are not Ctrl+G; a run that was cut
# off apart from one that crashed; and, with --var Midnight=1, a start in the last minute of a
# day (run.sh's caller sets the box's clock to 23:58:50 while the job waits; the steps only
# read it). Every check reads usage.json after PecoFence quit. About 4 minutes, 6 with Midnight.
$PfAllowLogErrors = $true
function UsageFile() { return "$PfApp\config\usage.json" }
function Reports() {
  $t = Get-Content (UsageFile) -Raw -ErrorAction SilentlyContinue
  if (-not $t) { return @() }
  return @(($t | ConvertFrom-Json) | ForEach-Object { $_ })
}
# Today's events as "name key".
function Events() { $r = @(Reports); if (-not $r.Count) { return @() }; return @($r[-1].events | ForEach-Object { "$($_[1]) $($_[2])".Trim() }) }
function Cards() { return @(@([W]::Tops('PecoFence.Notice')) | Where-Object { [G]::IsWindowVisible($_) }) }
# Nothing listens there: a test run records, and an upload fails without harm.
$env:PECOFENCE_USAGE_URL = 'http://127.0.0.1:9'

function UgNotice() {
  trap { PfTrap $_; continue }
  Case 'N the undo card after a fence was deleted from its menu, clicked'
  CloseWindows
  PfStart
  $fr = FreeRect 300 160 600 300
  Pf fence create --title 'Spare' --rect "$($fr[0]),$($fr[1]),300,160" | Out-Null
  Start-Sleep -Seconds 2
  $sp = @(PfJson fence list) | Where-Object { $_ -and $_.title -eq 'Spare' } | Select-Object -First 1
  $before = @(Cards)
  [G]::RClick($sp.windowRect.x + 150, $sp.windowRect.y + 14); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^Delete fence'
  $card = [IntPtr]::Zero
  for ($i = 0; $i -lt 30 -and $card -eq [IntPtr]::Zero; $i++) {
    $new = @(Cards | Where-Object { $before -notcontains $_ })
    if ($new.Count) { $card = $new[0] } else { Start-Sleep -Milliseconds 200 }
  }
  Start-Sleep -Milliseconds 800
  Shot 'n1-undo-card'
  if ($card -ne [IntPtr]::Zero) { $r = Rect $card; [G]::Click([int](($r.L + $r.R) / 2), [int](($r.T + $r.B) / 2)); Start-Sleep -Seconds 2 }
  $back = @(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq 'Spare' }).Count
  Check 'N setup: the menu deleted the fence, the undo card came, the click brought the fence back' ($picked -and $card -ne [IntPtr]::Zero -and $back -eq 1) "picked=$picked card=$card back=$back"
  PfQuit
  $ev = @(Events)
  Check 'N notice says the card and the answer: undo.action' ($ev -contains 'notice undo.action') ($ev -join ',')
  Check 'N no notice without a card' (-not ($ev -contains 'notice')) ($ev -join ',')
}

function UgDialog() {
  trap { PfTrap $_; continue }
  Case 'D 对话框跳转: back from Explorer, then Ctrl+G'
  CloseWindows
  $a = 'C:\pf\run\demo\one'; $b = 'C:\pf\run\demo\two'
  New-Item -ItemType Directory -Force $a, $b | Out-Null
  Set-Content -Path (Join-Path $a 'readme.txt') -Value 'x'
  Set-Culture en-US
  PfStart
  Pf settings set tools.dialogJump true | Out-Null
  Start-Process explorer.exe "`"$a`""; Start-Sleep -Seconds 4
  $ex = [G]::FindWindowW('CabinetWClass', [NullString]::Value)
  [L]::MoveWindow($ex, 820, 60, 760, 640, $true) | Out-Null; Start-Sleep -Milliseconds 500
  TitleClick $ex
  Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
  $np = [G]::FindWindowW('Notepad', [NullString]::Value)
  [L]::MoveWindow($np, 10, 40, 800, 760, $true) | Out-Null; Start-Sleep -Milliseconds 500
  TitleClick $np
  Chord 0x11 0x4F; Start-Sleep -Seconds 3
  $dlg = [G]::FindWindowW('#32770', 'Open')
  $dr = Rect $dlg
  # Back from Explorer, which now shows the other folder: the dialog jumps there.
  TitleClick $ex
  [G]::KeyDown(0x12); [G]::Press(0x44); [G]::KeyUp(0x12); Start-Sleep -Milliseconds 500
  [G]::Type("$b`n"); Start-Sleep -Seconds 2
  [G]::Click($dr.L + 120, $dr.T + 14)
  $got = WaitAddr $dlg 'two'
  Check 'D setup: coming back from Explorer made the dialog jump' ($dlg -ne [IntPtr]::Zero -and $got -like '*two') (DlgState 'open' $dlg)
  # Away to C:\Windows, then Ctrl+G jumps back.
  [QS]::SetFileName($dlg, 'C:\Windows'); [L]::PostMessageW($dlg, 0x111, [IntPtr]1, [IntPtr]0) | Out-Null
  $away = WaitAddr $dlg 'Windows'
  [QS]::SetFileName($dlg, '')
  Chord 0x11 0x47
  $got2 = WaitAddr $dlg 'two'
  Check 'D setup: Ctrl+G made it jump again' ($away -like '*Windows' -and $got2 -like '*two') "away=$away after Ctrl+G=$got2"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  CloseWindows
  PfQuit
  $ev = @(Events)
  Check 'D the jump on coming back is recorded: dialog_jump on_return' ($ev -contains 'dialog_jump on_return') ($ev -join ',')
  Check 'D Ctrl+G is recorded as the key and as the jump' ($ev -contains 'hotkey dialog_jump' -and $ev -contains 'dialog_jump ctrl_g') ($ev -join ',')
}

function UgRunEnd() {
  trap { PfTrap $_; continue }
  Case 'C a run that was cut off, then one that crashed'
  if (PfAlive) { PfQuit }
  PfStart
  PfQuit
  Remove-Item (UsageFile) -ErrorAction SilentlyContinue
  # Ended from Task Manager (exit code 1): the watchdog leaves it alone.
  PfStart
  Get-Process pecofence -ErrorAction SilentlyContinue | ForEach-Object { [Th]::Terminate($_.Id, 1) | Out-Null }
  Start-Sleep -Seconds 4
  PfStart
  PfQuit
  $h = (@(Reports) | Select-Object -Last 1).health
  Check 'C cut off: cutOff is true, crashed is false' ($h.cutOff -eq $true -and $h.crashed -eq $false) "health=$($h | ConvertTo-Json -Compress)"
  # No Windows Error Reporting box over the desktop for the crashes (pf's own setting, put back).
  $wer = 'HKCU:\Software\Microsoft\Windows\Windows Error Reporting'
  if (-not (Test-Path $wer)) { New-Item -Path $wer | Out-Null }
  $werBefore = (Get-ItemProperty -Path $wer -Name DontShowUI -ErrorAction SilentlyContinue).DontShowUI
  Set-ItemProperty -Path $wer -Name DontShowUI -Value 1 -Type DWord
  # An access violation inside the program: its log has the CRASH line. The watchdog does not
  # start a test-script run again, so the steps start the next run.
  Remove-Item (UsageFile) -ErrorAction SilentlyContinue
  PfStart -Script @('sleep 4000', 'crash') -Lang ''
  for ($i = 0; $i -lt 60 -and (PfAlive); $i++) { Start-Sleep -Milliseconds 250 }
  $gone = -not (PfAlive)
  Start-Sleep -Seconds 2
  PfStart -Lang ''
  PfQuit
  $crashLine = @(PfLogAll | Where-Object { $_ -match 'CRASH \(unhandled exception\)' }).Count
  $h = (@(Reports) | Select-Object -Last 1).health
  Check 'C setup: the test-script crash ended the run and logged its CRASH line' ($gone -and $crashLine -ge 1) "gone=$gone crash lines=$crashLine"
  Check 'C crashed (log): crashed is true, cutOff is false' ($h.crashed -eq $true -and $h.cutOff -eq $false) "health=$($h | ConvertTo-Json -Compress)"
  # Ended with an access violation's exit code before it could log anything: the watchdog
  # starts it again (a plain start, as a user's; the variables reach the new run).
  # The watchdog noted the crash above as a restart (it starts nothing for a test script) and
  # would not start an early crash again so soon: forget that note.
  Remove-Item "$env:LOCALAPPDATA\PecoFence\icons-hidden.$PfInst.restarted" -ErrorAction SilentlyContinue
  Remove-Item (UsageFile) -ErrorAction SilentlyContinue
  $env:PECOFENCE_INSTANCE = $PfInst
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  $null = WaitWin 'PecoFence.Control' 20 -Hidden
  Start-Sleep -Seconds 4
  $p1 = Get-Process pecofence | Select-Object -First 1
  [Th]::Terminate($p1.Id, [uint32]3221225477) | Out-Null
  $p2 = $null
  for ($i = 0; $i -lt 80 -and -not $p2; $i++) {
    Start-Sleep -Milliseconds 250
    $p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $p1.Id } | Select-Object -First 1
  }
  $null = WaitWin 'PecoFence.Control' 20 -Hidden
  Start-Sleep -Seconds 4
  PfQuit
  if ($null -eq $werBefore) { Remove-ItemProperty -Path $wer -Name DontShowUI -ErrorAction SilentlyContinue } else { Set-ItemProperty -Path $wer -Name DontShowUI -Value $werBefore -Type DWord }
  $h = (@(Reports) | Select-Object -Last 1).health
  $ev = @(Events)
  Check 'C setup: the watchdog started it again' ([bool]$p2 -and ($ev -contains 'start after_crash')) "restarted=$([bool]$p2) events=$($ev -join ',')"
  Check 'C crashed (exit code): crashed is true, cutOff is false' ($h.crashed -eq $true -and $h.cutOff -eq $false) "health=$($h | ConvertTo-Json -Compress)"
}

function UgMidnight() {
  trap { PfTrap $_; continue }
  Case 'M a start in the last minute of the day'
  if (PfAlive) { PfQuit }
  Remove-Item (UsageFile) -ErrorAction SilentlyContinue
  # Asks the caller for the clock (want-midnight.txt), then waits for 23:59:05 to 23:59:35.
  Set-Content "$PfRun\want-midnight.txt" (Get-Date -Format o)
  $ready = $false
  for ($i = 0; $i -lt 600; $i++) {
    $t = (Get-Date).TimeOfDay.TotalSeconds
    if ($t -ge 86345 -and $t -le 86375) { $ready = $true; break }
    Start-Sleep -Milliseconds 500
  }
  $day = (Get-Date).ToString('yyyy-MM-dd')
  Log "M clock ready=$ready at $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
  PfStart
  Pf fence list | Out-Null
  # Past midnight and past the first once-a-minute look of the run.
  for ($i = 0; $i -lt 240 -and (Get-Date).ToString('yyyy-MM-dd') -eq $day; $i++) { Start-Sleep -Milliseconds 500 }
  Start-Sleep -Seconds 75
  PfQuit
  $days = @(Reports)
  $first = $days | Where-Object { $_.day -eq $day } | Select-Object -First 1
  Check 'M setup: the run started in the last minute of a day and went past midnight' ($ready -and $days.Count -ge 2 -and $first) "days=$(($days | ForEach-Object { $_.day }) -join ',')"
  Check 'M that day''s report has the version, the settings and the counts' ($first.app -match '^\d+\.\d+\.\d+' -and $first.settings.hideRealIcons -is [bool] -and $first.state.fences -ge 1) "app='$($first.app)' state=$($first.state | ConvertTo-Json -Compress) events=$(@($first.events).Count)"
}

UgNotice
UgDialog
UgRunEnd
if ($Midnight) { UgMidnight }
