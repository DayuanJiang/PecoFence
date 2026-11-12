# The update check's second source, the website's update.json, against local stand-ins for both
# sources (PECOFENCE_UPDATE_URL for GitHub's API, PECOFENCE_UPDATE_SITE_URL for the site's file;
# assets/serve-update.ps1 answers both on port 8721 and, with -Hang, takes connections on 8722
# without ever answering; nothing listens on 8798 and 8799):
#   A  GitHub refuses, the site answers: the automatic check shows the card with the site's line,
#      and its button opens the site's download page
#   B  GitHub never answers: the button in Settings has its answer from the site after 8 s
#   C  both fail: the automatic check stays quiet; the button's check says so, names the site's
#      address and offers the download page
#   D  GitHub answers: the site is not asked
#   E  GitHub answers but the Store notes file of the UI language cannot be read there (French:
#      the feed has none): the card's line comes from the site's copy
#   F  installer edition, Update on the card, the download fails (the feed serves no setup): the
#      About page opens, says so and offers the download page
#   G  installer edition, Update on the card, a slow download (port 8723 answers the setup 25 s
#      late): a notification says that it downloads, the tray item says so and takes no click
# English UI (the card is read by OCR). Debug build (test delays, the settings page over CDP).
function NoticeWin() { $n = [G]::FindWindowW('PecoFence.Notice', [NullString]::Value); if ($n -ne [IntPtr]::Zero -and [G]::IsWindowVisible($n)) { return $n }; return [IntPtr]::Zero }
function Lines($re) { return @(PfLogAll | Where-Object { $_ -match $re -and $_ -notmatch 'omitted' } | Sort-Object -Unique) }
function Since($re) { return @(Lines $re | Where-Object { $_.Length -gt 27 -and $_.Substring(0, 27) -ge $script:Since }) }
function WaitLine($re, $sec = 20) { for ($i = 0; $i -lt $sec * 4; $i++) { $l = Since $re; if ($l.Count) { return $l[-1] }; Start-Sleep -Milliseconds 250 }; return $null }
function CardText($name) { $n = NoticeWin; if ($n -eq [IntPtr]::Zero) { return '' }; $r = Rect $n; return (OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) $name) }
function SiteHits() { return @(Get-Content "$PfRun\feed.log" -ErrorAction SilentlyContinue | Where-Object { $_ -match 'GET /update\.json' }).Count }
# PecoFence again with a clean notice record: GitHub's stand-in on port $Api, the site's on $Site;
# $Delays = first check, between checks, retry after a failure, check after a resume (ms).
function Again([int]$Api, [int]$Site, [string]$Delays = '4000,600000,600000,600000', [string]$Lang = 'en') {
  PfQuit
  if ($script:Notices) { Remove-Item $script:Notices -ErrorAction SilentlyContinue }
  $env:PECOFENCE_UPDATE_URL = "http://127.0.0.1:$Api/repos/DayuanJiang/PecoFence/releases/latest"
  $env:PECOFENCE_UPDATE_SITE_URL = "http://127.0.0.1:$Site/update.json"
  $env:PECOFENCE_TEST_UPDATE_MS = $Delays
  $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  $script:Since = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ss.ffffff') + 'Z'
  PfStart -Lang $Lang
  Pf settings set featureTips false | Out-Null
  [G]::MoveTo(700, 450)
}
# Settings on its About page; the update card's status line and whether the extra button shows.
function OpenAbout() { $ok = PfSettings; if ($ok) { PageClick 'nav a[data-page=about]'; Start-Sleep -Milliseconds 500 }; return $ok }
function Status() { return "$(Js "document.getElementById('updateStatus').textContent")" }
function SiteButton() { return [bool](Js "(() => { const b = document.getElementById('updateSite'); return !!b && !b.hidden && b.getBoundingClientRect().width > 0; })()") }

Case 'setup'
CloseWindows
# The feed offers the build's version plus one patch (the CLI says the version without the app).
$v = [version]((& "$PfBin\pecofence-cli.exe" describe 2>$null | Out-String) | ConvertFrom-Json).cli
$script:Next = "$($v.Major).$($v.Minor).$($v.Build + 1)"
$script:NextRe = [regex]::Escape($script:Next)
Log "build $v, the feed offers $script:Next"
$feed = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\kit\assets\serve-update.ps1', '-Port', '8721', '-Tag', "v$script:Next" -WindowStyle Hidden -PassThru
$hang = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\kit\assets\serve-update.ps1', '-Port', '8722', '-Tag', "v$script:Next", '-Hang' -WindowStyle Hidden -PassThru
$slow = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\kit\assets\serve-update.ps1', '-Port', '8723', '-Tag', "v$script:Next", '-SetupDelay', '25' -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 2
Check 'the stand-ins run' ($feed -and -not $feed.HasExited -and $hang -and -not $hang.HasExited -and $slow -and -not $slow.HasExited) ''

Case 'A: GitHub refuses, the site answers (automatic check)'
Again 8799 8721
$st = PfJson status
$script:Notices = Join-Path (Split-Path $st.configPath) 'notices.json'
$ask = WaitLine 'update: no answer from GitHub; asking the site' 25
$ans = WaitLine 'update: answer from the site' 15
$card = WaitLine 'update card shown' 15
Start-Sleep -Milliseconds 800
$t = CardText 'card-site'
Shot 'a1-card-from-site'
Check 'GitHub''s address refuses: the check goes on to the site''s file and finds the new version there' ($ask -and $ans -match "version=$script:NextRe" -and (SiteHits) -ge 1) "$ask | $ans | site hits $(SiteHits)"
Check 'the card shows, with the line from the site''s file' ($card -and $t -match $script:NextRe -and $t -match 'From the site') "ocr: $t"
$n = NoticeWin
if ($n -ne [IntPtr]::Zero) { $r = Rect $n; [G]::Click($r.R - 70, $r.B - 34); Start-Sleep -Seconds 2 }
$page = "$(Since 'update: release page opened')"
Check 'its Download button opens the site''s download page' ($page -match 'pecofence\.jiang\.jp/#download') "$page"
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

Case 'B: GitHub never answers, the site does (the button in Settings)'
Again 8722 8721 '600000,600000,600000,600000'
$open = OpenAbout
$t0 = Get-Date
PageClick '#updateAction'
$found = ''
for ($i = 0; $i -lt 80; $i++) { $s = Status; if ($s -match $script:NextRe) { $found = $s; break }; Start-Sleep -Milliseconds 250 }
$sec = [Math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
Shot 'b1-found-through-the-site'
Check 'GitHub never answers: about 8 s later the answer comes from the site and the page offers the new version' ($open -and $found -and $sec -ge 7 -and $sec -le 13) "after $sec s the page says '$(Status)' | $(Since 'update: answer from the site')"

Case 'C: both fail'
Again 8799 8798
$open = OpenAbout
WaitLine 'update: the automatic check failed; one more try' 40 | Out-Null
Start-Sleep -Seconds 1
# The app's own log names the sources (the shareable copy of the line keeps only the codes).
$fail = "$(Since 'update: the automatic check failed; one more try' | Where-Object { $_ -match 'GitHub:' })"
Check 'the automatic check fails quietly: one log line that names both sources, no card, nothing on the page' ($open -and $fail -match 'GitHub:' -and $fail -match 'the site:' -and (NoticeWin) -eq [IntPtr]::Zero -and (Status) -eq '') "$fail | page: '$(Status)'"
PageClick '#updateAction'
$s = ''
for ($i = 0; $i -lt 120; $i++) { $s = Status; if ($s -and $s -notmatch 'Checking') { break }; Start-Sleep -Milliseconds 250 }
Check 'the button''s check says that GitHub and the site gave no answer, and names the site''s address' ($s -match 'GitHub' -and $s -match 'pecofence\.jiang\.jp') "page: '$s'"
$retry = "$(Js "document.getElementById('updateAction').textContent")"
Check 'Open download page shows beside Check for updates' ((SiteButton) -and $retry -eq 'Check for updates') "extra button=$(SiteButton) main button='$retry'"
# The same card in Chinese, for the picture; its button opens the Chinese download page.
Pf settings set language zh-CN | Out-Null
Start-Sleep -Seconds 2
$r = PageRect '#updateCard'
if ($r) { Crop 'c1-both-fail-zh' ($r.x - 10) ($r.y - 10) ($r.w + 20) ($r.h + 20) | Out-Null }
if (SiteButton) { PageClick '#updateSite'; Start-Sleep -Seconds 2 }
$page = "$(Since 'update: release page opened')"
Check 'the button opens the site''s download page in the UI language' ($page -match 'pecofence\.jiang\.jp/zh-CN/#download') "$page"
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Pf settings set language en | Out-Null

Case 'D: GitHub answers, the site is not asked'
$before = SiteHits
Again 8721 8721
$card = WaitLine 'update card shown' 25
Start-Sleep -Milliseconds 800
$t = CardText 'card-github'
Check 'GitHub answers: the card''s line is the Store notes'' and the site''s file is not asked' ($card -and $t -match 'Typing Chinese in the Peek search' -and $t -notmatch 'From the site' -and (SiteHits) -eq $before -and (Since 'asking the site').Count -eq 0) "ocr: $t | site hits $before -> $(SiteHits)"
CloseNotice | Out-Null

Case 'E: the Store notes cannot be read from GitHub, the site has the line'
Again 8721 8721 '4000,600000,600000,600000' 'fr'
$card = WaitLine 'update card shown' 25
Start-Sleep -Milliseconds 800
$t = CardText 'card-fr'
$from = "$(Since 'update: Store notes not read from GitHub; the line of the site.s copy')"
Check 'French (no Store notes file in the feed): the card''s line comes from the site''s copy' ($card -and $from -match 'found=true' -and $t -match 'Depuis le site') "ocr: $t | $from"
CloseNotice | Out-Null
Pf settings set language en | Out-Null

Case 'F: installer edition, the download fails'
PfQuit
Set-Content "$PfApp\unins000.exe" 'test'
Again 8721 8721
$card = WaitLine 'update card shown' 25
Start-Sleep -Milliseconds 800
$n = NoticeWin
if ($n -ne [IntPtr]::Zero) { $r = Rect $n; [G]::Click($r.R - 70, $r.B - 34) }
$failed = WaitLine 'update failed' 20
# Nobody opened Settings: the failed download does.
$win = WaitWin 'PecoFence.Settings' 20
$open = $win -ne [IntPtr]::Zero -and (PfSettings)
$s = if ($open) { Status } else { '' }
Start-Sleep -Milliseconds 500
Shot 'f1-download-failed'
Check 'Update on the card, the setup cannot be downloaded: the About page opens and says so, with the site''s address' ($card -match 'installs=true' -and $failed -and $open -and $s -match 'pecofence\.jiang\.jp' -and (Js "document.querySelector('nav a.sel').dataset.page") -eq 'about') "settings window=$win page: '$s'"
Check 'Open download page shows there too' ($open -and (SiteButton)) "extra button=$(if ($open) { SiteButton })"
Remove-Item "$PfApp\unins000.exe" -ErrorAction SilentlyContinue

Case 'G: installer edition, a slow download says that it runs'
PfQuit
Set-Content "$PfApp\unins000.exe" 'test'
Again 8723 8723
$card = WaitLine 'update card shown' 25
Start-Sleep -Milliseconds 800
$n = NoticeWin
if ($n -ne [IntPtr]::Zero) { $r = Rect $n; [G]::Click($r.R - 70, $r.B - 34) }
$toast = ToastOcr 'Downloading' 6
Shot 'g1-downloading'
CloseToast
$m = TrayMenu; Start-Sleep -Milliseconds 500
$menu = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
[G]::Press(0x1B); Start-Sleep -Milliseconds 400; [G]::Press(0x1B); [G]::MoveTo(700, 450)
Check 'Update on the card: a notification says that the new version downloads' ($card -match 'installs=true' -and $toast -match 'Downloading the new version') "toast: $toast"
Check 'while it downloads the tray item says so and takes no click' ($menu -match '^Downloading the new version.{1,3} \(off\)') "menu: $menu"
PfQuit
Remove-Item "$PfApp\unins000.exe" -ErrorAction SilentlyContinue
ClearPfNotices

Case 'cleanup'
PfQuit
Stop-Process -Id $slow.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $feed.Id -Force -ErrorAction SilentlyContinue
Stop-Process -Id $hang.Id -Force -ErrorAction SilentlyContinue
Remove-Item Env:\PECOFENCE_UPDATE_URL, Env:\PECOFENCE_UPDATE_SITE_URL, Env:\PECOFENCE_TEST_UPDATE_MS, Env:\PECOFENCE_WEBVIEW_ARGS -ErrorAction SilentlyContinue
Log ("feed: " + ((Get-Content "$PfRun\feed.log" -ErrorAction SilentlyContinue) -join ' | '))
