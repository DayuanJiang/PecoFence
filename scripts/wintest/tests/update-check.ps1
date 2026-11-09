# 自动检查更新 against a local release feed that says v0.3.3 is out (PECOFENCE_UPDATE_URL, a named
# instance; the debug build's PECOFENCE_TEST_UPDATE_MS makes the first check come 8 s after start):
# the first check after start, the card once per version (a restart shows none), the card waiting
# for a full-screen app and for the Peek, its buttons in the installer and ZIP editions, the tray
# item, the switch off (no request), offline (nothing shown), and the card's line from the Store
# notes of the UI language (en-US and zh-CN served; German falls back to the general line).
# English UI (the card is read by OCR). Debug build.
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class Qn { [DllImport("shell32.dll")] public static extern int SHQueryUserNotificationState(out int state); }
"@
function NoticeWin() { $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value); if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }; return [IntPtr]::Zero }
function Lines($re) { return @(PfLogAll | Where-Object { $_ -match $re -and $_ -notmatch 'omitted' } | Sort-Object -Unique) }
function FeedHits() { return @(Get-Content "$PfRun\feed.log" -ErrorAction SilentlyContinue | Where-Object { $_ -match 'releases/latest' }).Count }
function NoticesFile() { $st = PfJson status; return (Join-Path (Split-Path $st.configPath) 'notices.json') }
# PecoFence again with a clean notice record (or not) and the given feed; waits for the first check.
function Restart([string]$url = '', [switch]$Installer, [switch]$KeepNotices, [string]$Lang = 'en') {
  # The kit runs the lines before the first case apart: the feed's address is spelled out here.
  if (-not $url) { $url = 'http://127.0.0.1:8721/repos/DayuanJiang/PecoFence/releases/latest' }
  PfQuit
  Remove-Item "$PfApp\unins000.exe" -ErrorAction SilentlyContinue
  if ($Installer) { Set-Content "$PfApp\unins000.exe" 'test' }
  if (-not $KeepNotices -and $script:Notices) { Remove-Item $script:Notices -ErrorAction SilentlyContinue }
  $env:PECOFENCE_UPDATE_URL = $url
  $env:PECOFENCE_TEST_UPDATE_MS = '8000,600000'
  $script:Since = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ss.ffffff') + 'Z'
  PfStart -Lang $Lang
  Pf settings set featureTips false | Out-Null
  [G]::MoveTo(700, 450)
}
function Since($re) { return @(Lines $re | Where-Object { $_.Length -gt 27 -and $_.Substring(0, 27) -ge $script:Since }) }
function WaitLine($re, $sec = 20) { for ($i = 0; $i -lt $sec * 4; $i++) { $l = Since $re; if ($l.Count) { return $l[-1] }; Start-Sleep -Milliseconds 250 }; return $null }
function CardText($name) { $n = NoticeWin; if ($n -eq [IntPtr]::Zero) { return '' }; $r = Rect $n; return (OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) $name) }

Case 'setup'
CloseWindows
$feed = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\kit\assets\serve-update.ps1', '-Port', '8721' -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 1
Restart
$script:Notices = NoticesFile
Log "notices: $script:Notices"
Check 'the feed runs and PecoFence starts' ($feed -and -not $feed.HasExited -and $script:Notices) ''

Case 'the first check comes after start and shows the card (ZIP edition)'
$check = WaitLine 'update: automatic check$'
$card = WaitLine 'update card shown' 15
Start-Sleep -Milliseconds 800
$t = CardText 'card-zip'
Shot 'u01-card-zip'
Check 'the first automatic check runs a moment after start (test delay) and asks the feed' ($check -and (FeedHits) -ge 1) "$check feed=$(FeedHits)"
Check 'the card says 0.3.3 is out with Download and What''s new' ($card -and $t -match '0\.3\.3' -and $t -match 'Download' -and $t -match "What.s new") "ocr: $t"
Check 'its line is the first point of the en-US Store notes' ($t -match 'Typing Chinese in the Peek search' -and (Since 'update card: its line line_from="Store notes"').Count -ge 1 -and @(Get-Content "$PfRun\feed.log" | Where-Object { $_ -match 'v2-i18n/en-US\.json HTTP/1\.1 -> 200' }).Count -ge 1) "ocr: $t"
$m = TrayMenu; Start-Sleep -Milliseconds 500
$menu = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
[G]::Press(0x1B); Start-Sleep -Milliseconds 400; [G]::Press(0x1B); [G]::MoveTo(700, 450)
Check 'the tray menu offers Update to 0.3.3 at the top' ($menu -match '^Update to 0\.3\.3') $menu
# The card goes by itself after 30 s, as later.
for ($i = 0; $i -lt 40 -and (NoticeWin) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Seconds 1 }
Check 'the card leaves after its 30 s' ((NoticeWin) -eq [IntPtr]::Zero -and (Since 'update card: later').Count -ge 1) ''

Case 'once per version: a restart shows no card'
Restart -KeepNotices
$skip = WaitLine 'update card: already shown for this version' 15
Start-Sleep -Seconds 2
Check 'no second card for 0.3.3' ($skip -and (NoticeWin) -eq [IntPtr]::Zero) "$skip"

Case 'the card waits for a full-screen app'
Restart
$fs = Start-Process powershell.exe -ArgumentList '-NoProfile', '-Command', "Add-Type -AssemblyName System.Windows.Forms; `$f = New-Object Windows.Forms.Form; `$f.FormBorderStyle = 'None'; `$f.WindowState = 'Maximized'; `$f.TopMost = `$true; `$f.BackColor = 'Black'; [Windows.Forms.Application]::Run(`$f)" -PassThru
Start-Sleep -Seconds 2
$q = 0; [Qn]::SHQueryUserNotificationState([ref]$q) | Out-Null
$wait = WaitLine 'update card waits why="full screen' 15
Start-Sleep -Seconds 1
Shot 'u02-full-screen'
Check 'with a full-screen app the card waits' ($wait -and (NoticeWin) -eq [IntPtr]::Zero) "notification state=$q; $wait"
Stop-Process -Id $fs.Id -Force -ErrorAction SilentlyContinue
$card = WaitLine 'update card shown' 20
Check 'once the full-screen app is gone the card shows' ($card -and (NoticeWin) -ne [IntPtr]::Zero) "$card"
CloseNotice | Out-Null

Case 'the card waits for the Peek'
Restart
Pf peek start | Out-Null
$wait = WaitLine 'update card waits why="Peek"' 15
Check 'during the Peek the card waits' ($wait -and (NoticeWin) -eq [IntPtr]::Zero) "$wait"
Pf peek end | Out-Null
$card = WaitLine 'update card shown' 20
Check 'after the Peek the card shows' ($card -and (NoticeWin) -ne [IntPtr]::Zero) "$card"
# 「What's new」 opens the release page.
$n = NoticeWin; $r = Rect $n
$t = CardText 'card-buttons'
[G]::Click($r.R - 200, $r.B - 34); Start-Sleep -Seconds 2
Check 'What''s new opens the release page' ((Since 'update card: what.s new').Count -ge 1 -and (Since 'update: release page opened').Count -ge 1) "ocr: $t"
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Case 'installer edition: Update downloads the setup'
Restart -Installer
$card = WaitLine 'update card shown' 15
Start-Sleep -Milliseconds 800
$t = CardText 'card-installer'
Shot 'u03-card-installer'
Check 'the installer edition''s card says Update' ($card -match 'installs=true' -and $t -match 'Update' -and $t -notmatch 'Download') "ocr: $t | $card"
$n = NoticeWin; $r = Rect $n
[G]::Click($r.R - 70, $r.B - 34); Start-Sleep -Seconds 3
Check 'Update starts the download from the feed (a test release without a setup file: it fails quietly)' ((Since 'update: update now').Count -ge 1 -and (Since 'downloading update').Count -ge 1) ((Since 'update: update now|downloading update|update failed') -join ' | ')

Case 'ZIP edition: the tray item opens the download page'
Restart -KeepNotices
WaitLine 'update available' 15 | Out-Null
TrayPick '^Update to 0\.3\.3' | Out-Null
Start-Sleep -Seconds 2
Check 'the tray item opens the download page in the ZIP edition' ((Since 'update: update now').Count -ge 1 -and (Since 'update: release page opened').Count -ge 1) ((Since 'update: update now|release page') -join ' | ')
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Case 'the card''s line in Chinese and in German'
Restart -Lang 'zh-CN'
$card = WaitLine 'update card shown' 15
Shot 'u04-card-zh'
Check 'Chinese: the first point of the zh-CN Store notes' ($card -and (Since 'update card: its line line_from="Store notes"').Count -ge 1 -and @(Get-Content "$PfRun\feed.log" | Where-Object { $_ -match 'v2-i18n/zh-CN\.json HTTP/1\.1 -> 200' }).Count -ge 1) ((Since 'Store notes|its line') -join ' | ')
CloseNotice | Out-Null
Restart -Lang 'de'
$card = WaitLine 'update card shown' 15
Check 'German (no file in the feed): the general line, after one quiet log line' ($card -and (Since 'Store notes not read').Count -ge 1 -and (Since 'update card: its line line_from="general"').Count -ge 1) ((Since 'Store notes|its line') -join ' | ')
CloseNotice | Out-Null
Pf settings set language en | Out-Null

Case 'the switch off: no check'
Pf settings set autoCheckUpdates false | Out-Null
$hits = FeedHits
Restart
$skip = WaitLine 'update: automatic check skipped why="switched off"' 15
Start-Sleep -Seconds 2
Check 'with 自动检查更新 off nothing is asked' ($skip -and (FeedHits) -eq $hits -and (NoticeWin) -eq [IntPtr]::Zero) "$skip feed $hits -> $(FeedHits)"
Pf settings set autoCheckUpdates true | Out-Null

Case 'offline: nothing shown'
Restart 'http://127.0.0.1:8799/repos/DayuanJiang/PecoFence/releases/latest'
$fail = WaitLine 'update: the automatic check failed' 40
Start-Sleep -Seconds 2
$m = TrayMenu; Start-Sleep -Milliseconds 500
$menu = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
[G]::Press(0x1B); Start-Sleep -Milliseconds 400; [G]::Press(0x1B)
Check 'offline: one quiet log line, no card, no tray item' ($fail -and (NoticeWin) -eq [IntPtr]::Zero -and $menu -notmatch 'Update to') "$fail | menu: $menu"

Case 'cleanup'
PfQuit
Remove-Item "$PfApp\unins000.exe" -ErrorAction SilentlyContinue
Stop-Process -Id $feed.Id -Force -ErrorAction SilentlyContinue
Remove-Item Env:\PECOFENCE_UPDATE_URL, Env:\PECOFENCE_TEST_UPDATE_MS -ErrorAction SilentlyContinue
Log ("feed: " + ((Get-Content "$PfRun\feed.log" -ErrorAction SilentlyContinue) -join ' | '))
