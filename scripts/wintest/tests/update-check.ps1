# 自动检查更新 against a local release feed that says the next patch version is out (one above the
# build's version, so the test keeps working after a version bump; PECOFENCE_UPDATE_URL, a named
# instance; the debug build's PECOFENCE_TEST_UPDATE_MS makes the first check come 8 s after start,
# the retry after a failure 6 s later and the check after a resume 3 s after it): the first check
# after start, the card once per version when someone was there (a restart shows none), a card
# nobody was there for once more a day later (notices.json's time moved back), the card waiting
# for a full-screen app and for the Peek (one log line for the wait and one for its end), the
# switch turned off while the card waits, the installer card's two buttons and the ZIP card's one,
# the tray item, the switch off (no request), offline (nothing shown, one retry), a resume (a
# check when none succeeded, none after a recent one), and the card's line from the Store notes
# of the UI language (en-US and zh-CN served; German falls back to the general line).
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
  # First check, between checks, retry after a failure, check after a resume.
  $env:PECOFENCE_TEST_UPDATE_MS = '8000,600000,6000,3000'
  $script:Since = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ss.ffffff') + 'Z'
  PfStart -Lang $Lang
  Pf settings set featureTips false | Out-Null
  [G]::MoveTo(700, 450)
}
function Since($re) { return @(Lines $re | Where-Object { $_.Length -gt 27 -and $_.Substring(0, 27) -ge $script:Since }) }
# How many times it was logged since the restart (the share log has each line too, a few us apart).
function Times($re) { return @(Since $re | ForEach-Object { $_.Substring(0, 23) } | Sort-Object -Unique).Count }
function WaitLine($re, $sec = 20) { for ($i = 0; $i -lt $sec * 4; $i++) { $l = Since $re; if ($l.Count) { return $l[-1] }; Start-Sleep -Milliseconds 250 }; return $null }
function CardText($name) { $n = NoticeWin; if ($n -eq [IntPtr]::Zero) { return '' }; $r = Rect $n; return (OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) $name) }
# What Windows sends after a sleep (WM_POWERBROADCAST, PBT_APMRESUMEAUTOMATIC).
function Resume() { $h = [G]::FindWindowW('PecoFence.Control', [NullString]::Value); $r = [IntPtr]::Zero; [G]::SendMessageTimeoutW($h, 0x218, [IntPtr]0x12, [IntPtr]0, 2, 5000, [ref]$r) | Out-Null }
# The card's record from notices.json (updateCard, updateCardUnseen, updateCardUnseenAt).
function Record() { $j = Get-Content $script:Notices -Raw -ErrorAction SilentlyContinue; if ($j) { return ($j | ConvertFrom-Json) }; return $null }

Case 'setup'
CloseWindows
# The feed offers the build's version plus one patch.
# The CLI says its version without the app (kit builds have no version resource).
$v = [version]((& "$PfBin\pecofence-cli.exe" describe 2>$null | Out-String) | ConvertFrom-Json).cli
$script:Next = "$($v.Major).$($v.Minor).$($v.Build + 1)"
$script:NextRe = [regex]::Escape($script:Next)
Log "build $v, the feed offers $script:Next"
$feed = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\kit\assets\serve-update.ps1', '-Port', '8721', '-Tag', "v$script:Next" -WindowStyle Hidden -PassThru
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
Check 'the ZIP card says the new version is out with one button, Download' ($card -and $t -match $script:NextRe -and $t -match 'Download' -and $t -notmatch "What.s new") "ocr: $t"
Check 'its line is the first point of the en-US Store notes' ($t -match 'Typing Chinese in the Peek search' -and (Since 'update card: its line line_from="Store notes"').Count -ge 1 -and @(Get-Content "$PfRun\feed.log" | Where-Object { $_ -match 'v2-i18n/en-US\.json HTTP/1\.1 -> 200' }).Count -ge 1) "ocr: $t"
$m = TrayMenu; Start-Sleep -Milliseconds 500
$menu = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
[G]::Press(0x1B); Start-Sleep -Milliseconds 400; [G]::Press(0x1B); [G]::MoveTo(700, 450)
Check 'the tray menu offers Update to the new version at the top' ($menu -match "^Update to $script:NextRe") $menu
# The card goes by itself after 30 s, as later; the menu above was the user being there.
for ($i = 0; $i -lt 40 -and (NoticeWin) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Seconds 1 }
$rec = WaitLine 'update card: recorded' 5
Check 'the card leaves after its 30 s and counts as seen (input while it showed)' ((NoticeWin) -eq [IntPtr]::Zero -and (Since 'update card: later').Count -ge 1 -and $rec -match 'there=true' -and (Record).updateCard -eq $script:Next) "$rec"

Case 'once per version: a restart shows no card'
Restart -KeepNotices
$skip = WaitLine 'update card: already shown for this version' 15
Start-Sleep -Seconds 2
Check 'no second card for the same version' ($skip -and (NoticeWin) -eq [IntPtr]::Zero) "$skip"

Case 'a card nobody was there for comes once more a day later'
Restart
$card = WaitLine 'update card shown' 15
# No key, click or mouse move while it shows.
for ($i = 0; $i -lt 40 -and (NoticeWin) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Seconds 1 }
$rec = WaitLine 'update card: recorded' 5
$r1 = Record
Check 'without input during its 30 s the card is not counted as seen' ($card -and $rec -match 'there=false' -and $r1.updateCard -ne $script:Next -and $r1.updateCardUnseen -eq $script:Next -and $r1.updateCardUnseenAt -gt 0) "$rec | $(Get-Content $script:Notices -Raw)"
Restart -KeepNotices
$skip = WaitLine 'update card: shown while nobody was there' 15
Start-Sleep -Seconds 2
Check 'a restart the same day shows no card' ($skip -and (NoticeWin) -eq [IntPtr]::Zero) "$skip"
# A day later: the time in notices.json moved back 21 hours.
PfQuit
$j = Record
$j.updateCardUnseenAt = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds() - 21 * 3600
[IO.File]::WriteAllText($script:Notices, ($j | ConvertTo-Json -Depth 10), (New-Object Text.UTF8Encoding($false)))
Restart -KeepNotices
$card = WaitLine 'update card shown' 15
Check 'a day later the card comes once more' ($card -match 'again=true' -and (NoticeWin) -ne [IntPtr]::Zero) "$card"
for ($i = 0; $i -lt 40 -and (NoticeWin) -ne [IntPtr]::Zero; $i++) { Start-Sleep -Seconds 1 }
$rec = WaitLine 'update card: recorded' 5
Restart -KeepNotices
$skip = WaitLine 'update card: already shown for this version' 15
Start-Sleep -Seconds 2
Check 'after its second offer the version counts as seen, with or without input' ($rec -match 'last_offer=true' -and (Record).updateCard -eq $script:Next -and $skip -and (NoticeWin) -eq [IntPtr]::Zero) "$rec | $skip"

Case 'the card waits for a full-screen app'
Restart
$fs = Start-Process powershell.exe -ArgumentList '-NoProfile', '-Command', "Add-Type -AssemblyName System.Windows.Forms; `$f = New-Object Windows.Forms.Form; `$f.FormBorderStyle = 'None'; `$f.WindowState = 'Maximized'; `$f.TopMost = `$true; `$f.BackColor = 'Black'; [Windows.Forms.Application]::Run(`$f)" -PassThru
Start-Sleep -Seconds 2
$q = 0; [Qn]::SHQueryUserNotificationState([ref]$q) | Out-Null
$wait = WaitLine 'update card waits why="full screen or presentation"' 15
Start-Sleep -Seconds 1
Shot 'u02-full-screen'
Check 'with a full-screen app the card waits' ($wait -and (NoticeWin) -eq [IntPtr]::Zero) "notification state=$q; $wait"
Stop-Process -Id $fs.Id -Force -ErrorAction SilentlyContinue
$card = WaitLine 'update card shown' 20
Check 'once the full-screen app is gone the card shows' ($card -and (NoticeWin) -ne [IntPtr]::Zero) "$card"
CloseNotice | Out-Null

Case 'the card waits for the Peek (installer edition: What''s new)'
Restart -Installer
Pf peek start | Out-Null
$wait = WaitLine 'update card waits why="Peek"' 15
Check 'during the Peek the card waits' ($wait -and (NoticeWin) -eq [IntPtr]::Zero) "$wait"
# Two more tries every 10 s: no more lines.
Start-Sleep -Seconds 22
Pf peek end | Out-Null
$card = WaitLine 'update card shown' 20
$over = "$(Since 'update card: the wait is over')"
Check 'after the Peek the card shows' ($card -and (NoticeWin) -ne [IntPtr]::Zero) "$card"
Check 'the wait is logged once, and its end with how long' ((Times 'update card waits') -eq 1 -and (Times 'update card: the wait is over') -eq 1 -and $over -match 'waited_s=(2\d|3\d)') ((Since 'update card waits|wait is over') -join ' | ')
# 「What's new」 opens the release page.
Start-Sleep -Milliseconds 800
$n = NoticeWin; $r = Rect $n
$t = CardText 'card-buttons'
Shot 'u03-card-installer'
Check 'the installer edition''s card has Update and What''s new' ($card -match 'installs=true' -and $t -match 'Update' -and $t -match "What.s new" -and $t -notmatch 'Download') "ocr: $t | $card"
[G]::Click($r.R - 200, $r.B - 34); Start-Sleep -Seconds 2
Check 'What''s new opens the release page' ((Since 'update card: what.s new').Count -ge 1 -and (Since 'update: release page opened').Count -ge 1) "ocr: $t"
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Case 'the switch turned off while the card waits'
Restart
Pf peek start | Out-Null
$wait = WaitLine 'update card waits why="Peek"' 15
Pf settings set autoCheckUpdates false | Out-Null
Pf peek end | Out-Null
$off = WaitLine 'update card: the automatic check was switched off' 15
Start-Sleep -Seconds 2
Check 'turned off before it shows, the card does not show' ($wait -and $off -and (NoticeWin) -eq [IntPtr]::Zero -and (Since 'update card shown').Count -eq 0) "$wait | $off"
Pf settings set autoCheckUpdates true | Out-Null

Case 'installer edition: Update downloads the setup'
Restart -Installer
$card = WaitLine 'update card shown' 15
Start-Sleep -Milliseconds 800
$n = NoticeWin; $r = Rect $n
[G]::Click($r.R - 70, $r.B - 34); Start-Sleep -Seconds 3
Check 'Update starts the download from the feed (a test release without a setup file: it fails quietly)' ((Since 'update: update now').Count -ge 1 -and (Since 'downloading update').Count -ge 1) ((Since 'update: update now|downloading update|update failed') -join ' | ')

Case 'ZIP edition: the tray item opens the download page'
Restart -KeepNotices
WaitLine 'update available' 15 | Out-Null
TrayPick "^Update to $script:NextRe" | Out-Null
Start-Sleep -Seconds 2
Check 'the tray item opens the download page in the ZIP edition' ((Since 'update: update now').Count -ge 1 -and (Since 'update: release page opened').Count -ge 1) ((Since 'update: update now|release page') -join ' | ')
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Case 'the card''s line in Chinese and in German'
Restart -Lang 'zh-CN'
$card = WaitLine 'update card shown' 15
Start-Sleep -Milliseconds 800
Shot 'u04-card-zh'
Check 'Chinese: the first point of the zh-CN Store notes' ($card -and (Since 'update card: its line line_from="Store notes"').Count -ge 1 -and @(Get-Content "$PfRun\feed.log" | Where-Object { $_ -match 'v2-i18n/zh-CN\.json HTTP/1\.1 -> 200' }).Count -ge 1) ((Since 'Store notes|its line') -join ' | ')
# The ZIP card's one button opens the download page.
$n = NoticeWin; $r = Rect $n
[G]::Click($r.R - 70, $r.B - 34); Start-Sleep -Seconds 2
Check 'the ZIP card''s button opens the download page' ((Since 'update: update now').Count -ge 1 -and (Since 'update: release page opened').Count -ge 1) ((Since 'update: update now|release page') -join ' | ')
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
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

Case 'offline: nothing shown, one retry'
Restart 'http://127.0.0.1:8799/repos/DayuanJiang/PecoFence/releases/latest'
$fail = WaitLine 'update: the automatic check failed; one more try' 40
$again = WaitLine 'update: the automatic check failed again; next try tomorrow' 40
Start-Sleep -Seconds 2
$m = TrayMenu; Start-Sleep -Milliseconds 500
$menu = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
[G]::Press(0x1B); Start-Sleep -Milliseconds 400; [G]::Press(0x1B)
Check 'offline: a quiet log line, one retry (6 s in the test, an hour normally), then tomorrow' ($fail -and $again -and (Times 'update: automatic check$') -eq 2) "$fail | $again"
Check 'offline: no card, no tray item' ((NoticeWin) -eq [IntPtr]::Zero -and $menu -notmatch 'Update to') "menu: $menu"
Resume
$after = WaitLine 'update: check after the resume' 15
Start-Sleep -Seconds 3
Check 'a resume checks when no check has succeeded yet' ($after -match 'age_s=-1' -and (Times 'update: automatic check$') -ge 3) "$after"

Case 'a resume right after a good check asks nothing'
Restart
WaitLine 'update available' 15 | Out-Null
CloseNotice | Out-Null
$hits = FeedHits
Resume
$skip = WaitLine 'update: no check after the resume' 15
Start-Sleep -Seconds 2
Check 'the last check is recent: no request after the resume' ($skip -and (FeedHits) -eq $hits) "$skip feed $hits -> $(FeedHits)"

Case 'cleanup'
PfQuit
Remove-Item "$PfApp\unins000.exe" -ErrorAction SilentlyContinue
Stop-Process -Id $feed.Id -Force -ErrorAction SilentlyContinue
Remove-Item Env:\PECOFENCE_UPDATE_URL, Env:\PECOFENCE_TEST_UPDATE_MS -ErrorAction SilentlyContinue
Log ("feed: " + ((Get-Content "$PfRun\feed.log" -ErrorAction SilentlyContinue) -join ' | '))
