# 撤销提示 (feedback #185, #188): the card 「已删除栅栏」 with 撤销 after a fence is deleted from
# its menu leaves by itself after 10 s (it stayed 30 s); Settings › 常规 › 撤销提示 off
# (`undoNotices`) shows no card, and Ctrl+Z on a fence still brings the fence back.
#   run.sh --bin target/wintest-bin-debug --var Part=P tests/undo-notices.ps1   the switch on the
#                                       settings page in Chinese (CDP: a debug build), with a shot
if (-not $Part) { $Part = 'A' }
if ($Part -eq 'P') {
Case 'settings page switch'
CloseWindows
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang zh-CN
$ok = PfSettings
Check 'settings open over CDP' $ok "debug=$PfDebugBuild"
PageRect '[data-bind=featureTips]' | Out-Null
Start-Sleep -Milliseconds 500
$r = PageRect '[data-bind=featureTips]'
$w = Rect ([G]::FindWindowW('PecoFence.Settings', [NullString]::Value))
if ($r) { Crop 'p-general-tips' ($w.L + 8) ($r.y - 150) ($w.R - $w.L - 16) 330 | Out-Null }
$card = Js "(() => { const t = document.querySelector('[data-bind=undoNotices]'); if (!t) return null; const c = t.closest('.card'); return { title: c.querySelector('.t').textContent, on: t.getAttribute('aria-checked'), page: t.closest('section').id }; })()"
Check '常规 has 撤销提示, on' ($card -and $card.title -eq '撤销提示' -and $card.on -eq 'true' -and $card.page -eq 'page-general') ($card | ConvertTo-Json -Compress)
if ($card) {
  PageClick '[data-bind=undoNotices]'
  Start-Sleep -Seconds 1
  Check 'a click on the switch turns undoNotices off' ((Pf settings get undoNotices) -match 'false') ((Pf settings get undoNotices) -join ' ')
  PageClick '[data-bind=undoNotices]'
  Start-Sleep -Seconds 1
}
Check 'a second click turns it back on' ((Pf settings get undoNotices) -match 'true') ((Pf settings get undoNotices) -join ' ')
PageType '#settingsSearch' '删除提示'
Start-Sleep -Milliseconds 800
$hits = Js "document.getElementById('settingsSearchList').textContent"
Check "查找设置: '删除提示' finds 撤销提示" ($hits -match '撤销提示') $hits
PageType '#settingsSearch' ''
PfQuit
}

if ($Part -eq 'A') {
Case 'setup'
PfStart
ClearFences | Out-Null
Pf fence create --title 'Keep' --rect '120,120,320,220' | Out-Null
function NoticeWin() {
  foreach ($n in @([W]::Tops('PecoFence.Notice'))) { if ([G]::IsWindowVisible($n)) { return $n } }
  return [IntPtr]::Zero
}
function WaitNotice($sec = 6) {
  for ($i = 0; $i -lt $sec * 5; $i++) { $n = NoticeWin; if ($n -ne [IntPtr]::Zero) { return $n }; Start-Sleep -Milliseconds 200 }
  return [IntPtr]::Zero
}
function HasFence($title) { return [bool](@(PfJson fence list) | Where-Object { $_ -and $_.title -eq $title }) }
# An empty fence deleted from its title menu (no question asked); returns what was picked.
function MenuDelete($title) {
  $f = FenceGet $title
  $r = $f.windowRect
  [G]::RClick([int]($r.x + 50), [int]($r.y + 14))
  return (MenuPick 'Delete fence')
}

Case 'the card leaves after 10 seconds'
Pf fence create --title 'Gone A' --rect '520,140,320,220' | Out-Null
Start-Sleep -Seconds 1
$picked = MenuDelete 'Gone A'
$n = WaitNotice 6
$t0 = Get-Date
ParkMouse
Check 'deleting a fence from its menu shows the undo card' ($n -ne [IntPtr]::Zero) "picked '$picked'; card '$([L]::Text($n))'"
$r = Rect $n
if ($r.R -gt $r.L) { Crop 'card-deleted' ($r.L - 12) ($r.T - 12) ($r.R - $r.L + 24) ($r.B - $r.T + 24) | Out-Null }
Start-Sleep -Milliseconds ([Math]::Max(0, 8000 - ((Get-Date) - $t0).TotalMilliseconds))
$at8 = (NoticeWin) -ne [IntPtr]::Zero
Start-Sleep -Milliseconds ([Math]::Max(0, 12500 - ((Get-Date) - $t0).TotalMilliseconds))
$at12 = (NoticeWin) -ne [IntPtr]::Zero
Check 'the card is still there after 8 s' $at8 "visible at 8 s: $at8"
Check 'the card is gone after 12 s (it stayed 30 s)' (-not $at12) "visible at 12.5 s: $at12"
Shot 'after-12s'

Case 'Ctrl+Z after the card went'
$k = FenceGet 'Keep'
[G]::Click([int]($k.windowRect.x + 160), [int]($k.windowRect.y + 120)); Start-Sleep -Milliseconds 500
Chord 0x11 0x5A; Start-Sleep -Seconds 2
$back = HasFence 'Gone A'
Check 'Ctrl+Z on a fence brings the deleted fence back' $back "Gone A listed: $back"
ParkMouse

Case 'switch off'
Pf settings set undoNotices false | Out-Null
Start-Sleep -Milliseconds 500
$val = (Pf settings get undoNotices) -join ' '
Check 'settings set undoNotices false is taken' ($val -match 'false') "get: '$val'"
Pf fence create --title 'Gone B' --rect '520,420,320,220' | Out-Null
Start-Sleep -Seconds 1
$picked = MenuDelete 'Gone B'
$n = WaitNotice 4
ParkMouse
$gone = -not (HasFence 'Gone B')
Check 'with 撤销提示 off the fence is deleted and no card shows' ($gone -and $n -eq [IntPtr]::Zero) "picked '$picked'; deleted=$gone; card '$([L]::Text($n))'"
$logged = @(PfLogAll | Where-Object { $_ -match 'undo notice not shown: 撤销提示 is off' }).Count
Check 'the log says why no card showed' ($logged -ge 1) "lines=$logged"
$k = FenceGet 'Keep'
[G]::Click([int]($k.windowRect.x + 160), [int]($k.windowRect.y + 120)); Start-Sleep -Milliseconds 500
Chord 0x11 0x5A; Start-Sleep -Seconds 2
$back = HasFence 'Gone B'
Check 'Ctrl+Z still brings it back with the switch off' $back "Gone B listed: $back"
Pf settings set undoNotices true | Out-Null
ParkMouse
PfQuit
}
