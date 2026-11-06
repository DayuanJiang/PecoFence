# #186 setup: install the released 0.2.5 the way a user does, change a fence, upgrade with the
# new setup, check that the fences and their titles stay and the new 小工具 switches are off,
# then uninstall. Needs --extra with old-setup.exe (released) and new-setup.exe (under test).
Case 'setup'
CloseWindows
$prog = "$env:LOCALAPPDATA\Programs\PecoFence"
$cli = "$prog\pecofence-cli.exe"
Remove-Item Env:\PECOFENCE_INSTANCE -ErrorAction SilentlyContinue
function Ipf { $a = @($args | ForEach-Object { $_ }); $r = & $cli @a 2>&1 | Out-String; Log ("installed cli " + ($a -join ' ') + " -> " + $r.Trim()); return $r }
function Setup($exe, $log) {
  $p = Start-Process $exe -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', "/LOG=$PfRun\$log" -PassThru
  $p.WaitForExit(180000) | Out-Null
  return $p.ExitCode
}
function StartInstalled() {
  Start-Process explorer.exe "`"$prog\pecofence.exe`""
  $h = WaitWin 'PecoFence.Control' 30 -Hidden
  Start-Sleep -Seconds 5
  return $h
}
if (Test-Path "$prog\unins000.exe") { Start-Process "$prog\unins000.exe" -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait }
Check 'the installers are there' ((Test-Path "$PfExtra\old-setup.exe") -and (Test-Path "$PfExtra\new-setup.exe"))

Case 'install 0.2.5'
$code = Setup "$PfExtra\old-setup.exe" 'setup-old.log'
Check 'the 0.2.5 setup succeeds' ($code -eq 0 -and (Test-Path "$prog\pecofence.exe")) "exit=$code"
$lnk = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PecoFence\PecoFence.lnk"
Check 'Start menu shortcut' (Test-Path $lnk) $lnk
$h = StartInstalled
Check '0.2.5 starts' ($h -ne [IntPtr]::Zero)
$j = (Ipf fence list) | ConvertFrom-Json
$titles = @(@($j.fences) + @($j) | Where-Object { $_.title } | ForEach-Object { $_.title })
Check 'first run makes fences' ($titles.Count -ge 3) ($titles -join ', ')
$first = $titles[0]
Ipf fence rename $first 'Renamed by test' | Out-Null
Ipf fence move 'Renamed by test' --rect '400,300,420,260' | Out-Null
Start-Sleep -Seconds 2
$moved = ((Ipf fence get 'Renamed by test') | ConvertFrom-Json)
$mr = if ($moved.fence) { $moved.fence.windowRect } else { $moved.windowRect }
Shot 'i01-old-version'

Case 'upgrade to the new setup'
# The setup refuses to replace a running PecoFence (AppMutex): quit it the way the user would.
PfQuit
$code = Setup "$PfExtra\new-setup.exe" 'setup-new.log'
Check 'the new setup succeeds over 0.2.5' ($code -eq 0) "exit=$code"
$h = StartInstalled
Check 'the new version starts' ($h -ne [IntPtr]::Zero)
$j = (Ipf fence list) | ConvertFrom-Json
$after = @(@($j.fences) + @($j) | Where-Object { $_.title } | ForEach-Object { $_.title })
Check 'the fences and the renamed title stay' (($after -contains 'Renamed by test') -and $after.Count -eq $titles.Count) ($after -join ', ')
$now = ((Ipf fence get 'Renamed by test') | ConvertFrom-Json)
$nr = if ($now.fence) { $now.fence.windowRect } else { $now.windowRect }
Check 'the moved fence keeps its place' ($nr.x -eq $mr.x -and $nr.y -eq $mr.y) "before=$($mr.x),$($mr.y) after=$($nr.x),$($nr.y)"
$tools = (Ipf settings get tools) | ConvertFrom-Json
Check 'after the upgrade 小工具 is on and the new opt-in switches are off' ($tools.enabled -and -not ($tools.dialogJump -or $tools.everything)) ((Ipf settings get tools).Trim())
$log = "$env:LOCALAPPDATA\PecoFence\pecofence.log"
$warn = @(Get-Content $log -Encoding UTF8 -ErrorAction SilentlyContinue | Where-Object { $_ -match ' (ERROR|WARN) .*config' })
Check 'no config warnings in the log' ($warn.Count -eq 0) (($warn | Select-Object -First 2) -join ' | ')
Shot 'i02-upgraded'

Case 'uninstall'
# Like setup, the uninstaller will not remove a running PecoFence (AppMutex: interactively it
# asks the user to close it, silently it gives up): quit it first, as the user would.
PfQuit
$p = Start-Process "$prog\unins000.exe" -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -PassThru
$p.WaitForExit(120000) | Out-Null
Start-Sleep -Seconds 3
Check 'the uninstaller succeeds' ($p.ExitCode -eq 0) "exit=$($p.ExitCode)"
Check 'the program folder is gone' (-not (Test-Path "$prog\pecofence.exe"))
Check 'the Start menu shortcut is gone' (-not (Test-Path $lnk))
$shown = (Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced').HideIcons
Check 'Windows desktop icons are shown again' ($shown -ne 1) "HideIcons=$shown"
Shot 'i03-uninstalled'

Case 'cleanup'
$PfAllowLogErrors = $true
