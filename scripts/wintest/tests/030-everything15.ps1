# 改用 1.5: Everything 1.4 runs although 1.5 is installed (only 1.4 starts for all users). The card
# says so; 改用 1.5 closes 1.4, takes it out of startup and makes 1.5 start at sign-in.
# Needs both installed on the box (1.4 x86 in Program Files (x86)\Everything, 1.5a x64) and the
# machine-wide Run value "Everything" of 1.4. --var Expect=deny: pf cannot elevate (the
# machine entry stays); Expect=ok: pf elevates without a prompt (the entry goes).
Case 'setup'
CloseWindows
Set-Culture en-US
if (-not $Expect) { $Expect = 'deny' }
$e14 = 'C:\Program Files (x86)\Everything\Everything.exe'
$e15 = 'C:\Program Files\Everything 1.5a\Everything.exe'
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class EvW { [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindowW(string c, string t); }
"@
function Has14() { return [EvW]::FindWindowW('EVERYTHING_TASKBAR_NOTIFICATION', $null) -ne [IntPtr]::Zero }
function Has15() { return [EvW]::FindWindowW('EVERYTHING_TASKBAR_NOTIFICATION_(1.5a)', $null) -ne [IntPtr]::Zero }
function Machine14() { return [bool](Get-ItemProperty 'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run' -Name Everything -ErrorAction SilentlyContinue) }
function User15() { return (Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue).'Everything 1.5a' }
# 1.4 running in pf's session, 1.5 not, no startup entry of 1.5 for pf.
if (Has15) { Start-Process $e15 -ArgumentList '-exit' -Wait; Start-Sleep -Seconds 2 }
Remove-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -Name 'Everything 1.5a' -ErrorAction SilentlyContinue
if (-not (Has14)) { Start-Process $e14 -ArgumentList '-startup' -WorkingDirectory (Split-Path $e14); Start-Sleep -Seconds 4 }
Check 'start: 1.4 runs, 1.5 does not, 1.4 starts for all users' ((Has14) -and -not (Has15) -and (Machine14)) "1.4=$(Has14) 1.5=$(Has15) machine=$(Machine14) expect=$Expect"
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en -Script @('sleep 3600000')
Start-Sleep -Seconds 3
Pf settings set tools.everything true | Out-Null

Case 'the card says 1.5 is installed'
$ok = PfSettings
Check 'settings open over CDP' $ok
PageClick 'nav a[data-page=peek]'; Start-Sleep -Seconds 1
$card = JsWait "(() => { const s = document.getElementById('everythingStatus').textContent; return s ? { status: s, help: document.getElementById('everythingHelp').textContent, button: document.getElementById('everythingAction').textContent, hidden: document.getElementById('everythingAction').hidden } : null; })()" 10
Check 'status: 1.5 installed, 1.4 running' ($card.status -eq 'Everything 1.5 is installed, but 1.4 is running') ($card | ConvertTo-Json -Compress)
Check 'button: Switch to 1.5' (-not $card.hidden -and $card.button -eq 'Switch to 1.5') ($card | ConvertTo-Json -Compress)
Shot 'e01-card-before'

Case 'Switch to 1.5'
PageClick '#everythingAction'
for ($i = 0; $i -lt 40 -and -not ((Has15) -and -not (Has14) -and (User15)); $i++) { Start-Sleep -Milliseconds 500 }
Start-Sleep -Seconds 3
Check '1.4 is closed' (-not (Has14))
Check '1.5 runs' (Has15)
Check '1.5 starts at sign-in for this user' ([bool](User15)) "HKCU Run: $(User15)"
if ($Expect -eq 'ok') {
  Check "1.4's all-users startup entry is gone" (-not (Machine14))
} else {
  Check "without administrator rights the all-users entry stays" (Machine14)
}
$log = (PfLogAll) -join "`n"
Check 'log: 1.4 closed and taken out of startup' ($log -match 'Everything 1\.4 closed and taken out of startup.*exited=true') ([regex]::Match($log, '.*Everything 1\.4 closed.*').Value)
Check 'log: 1.5 started and set to start at sign-in' ($log -match 'Everything 1\.5 started and set to start at sign-in.*started=true.*startup=true') ([regex]::Match($log, '.*Everything 1\.5 started.*').Value)
$toast = PageToast
Check 'the notice says 1.5 is used now' ($toast.text -match 'Switched to Everything 1\.5') ($toast | ConvertTo-Json -Compress)
$card = JsWait "(() => { const s = document.getElementById('everythingStatus').textContent; return /1\.5 is running|1\.5\b.*running/.test(s) ? { status: s, hidden: document.getElementById('everythingAction').hidden } : null; })()" 10
Check 'the card now: 1.5 running, no button' ($card -and $card.hidden) ($card | ConvertTo-Json -Compress)
Shot 'e02-card-after'
PfQuit
