# 设置 › 栅栏 › 搜索条件 on the real page (a debug build: build.sh --debug, run.sh --bin
# target/wintest-bin-debug): a smart fence's search in one line, what its last run found,
# 编辑条件… (the smart fence editor's window), 立即刷新 and, without Everything, the button that
# starts it; the line follows a change made elsewhere (the CLI, the editor). Needs Everything
# (see smart.ps1).
Case 'setup'
CloseWindows
DownloadsScene
AgedFile "$PfDl\sub" 'deep.pdf' 1 | Out-Null
Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
Check 'Everything runs' (EvStart)
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang en
Start-Sleep -Seconds 2
$D = 'Downloads of the last 7 days'
Pf fence create --smart downloads --rect '420,140,560,330' | Out-Null
Check 'the smart fence is filled' (WaitSmart $D { $s.source -eq 'everything' -and $s.shown -eq 7 } 30) (SmartLine $D)
$id = (FenceGet $D).id
# Sets a box of the page as a user's change would (the page's own change handler runs).
function PageSet($elId, $value) {
  $v = $value | ConvertTo-Json
  Js "(() => { const e = document.getElementById('$elId'); e.value = $v; e.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 600
}
function GroupState() { return Js "(() => ({ shown: document.getElementById('fenceSmartGroup').style.display !== 'none', summary: document.getElementById('smartSummary').textContent, status: document.getElementById('smartStatus').textContent, evBtn: document.getElementById('smartEverythingBtn').hidden ? '' : document.getElementById('smartEverythingBtn').textContent, old: !!document.getElementById('smartScope') || !!document.getElementById('smartConds') }))()" }

Case 'the group shows the search in one line'
Check 'settings open over CDP' (PfSettings)
PageClick 'nav a[data-page=fences]'; Start-Sleep -Milliseconds 600
PageSet 'fenceSel' $id
$g = GroupState
Check 'shown for the smart fence: folder, subfolders, time and files only in one line' ($g.shown -and $g.summary -eq 'Downloads folder (with subfolders) · Last 7 days · Files only' -and -not $g.old) ($g | ConvertTo-Json -Compress)
Check 'the status says what the last run found' ($g.status -match '^Showing 7 files now · Searched by Everything' -and -not $g.evBtn) ($g | ConvertTo-Json -Compress)
Shot 'p01-group'
$plain = (@(PfJson fence list) | Where-Object { $_ -and -not $_.smart -and $_.kind -eq 'virtual' } | Select-Object -First 1).id
PageSet 'fenceSel' $plain
Check 'hidden for a fence of icons' (-not (GroupState).shown) ''
PageSet 'fenceSel' $id

Case 'the line follows a change made elsewhere'
Pf fence set $D smartWithinDays 30 | Out-Null
$s = JsWait "(() => { const t = document.getElementById('smartSummary').textContent; return t.includes('30') ? t : null; })()" 10
Check 'fence set smartWithinDays 30: the line says 30 days' ($s -eq 'Downloads folder (with subfolders) · Last 30 days · Files only') "$s"
Pf fence set $D smartMatch any | Out-Null
Pf fence set $D smartConditions '[{\"cond\":\"ext\",\"value\":[\"pdf\"]},{\"cond\":\"name\",\"value\":{\"op\":\"contains\",\"value\":\"photo\"}}]' | Out-Null
$s = JsWait "(() => { const t = document.getElementById('smartSummary').textContent; return t.includes('Any one of') ? t : null; })()" 10
Check 'smartMatch any and two conditions: Any one of: …' ($s -eq 'Downloads folder (with subfolders) · Any one of: Last 30 days · Extensions: pdf · Name contains “photo”') "$s"
Check 'the fence follows: the PDFs or photo, of the last 30 days or not' (WaitSmart $D { $n -match 'old contract' -and $n -match 'photos' -and $n -match 'deep' } 15) (SmartLine $D)
Pf fence set $D smartMatch all | Out-Null
Pf fence set $D smartConditions '[{\"cond\":\"filesOnly\",\"value\":null}]' | Out-Null
Pf fence set $D smartWithinDays 7 | Out-Null
Start-Sleep -Seconds 1

Case 'Edit conditions… opens the editor on this fence'
PageClick '#smartEdit'
$h = EditorWin
Check 'the editor window opens with the fence' ($h -ne [IntPtr]::Zero -and (Js "document.querySelector('#seWindow .se-name').value") -eq $D) ''
Js "(() => { const s = document.querySelector('#seWindow .se-row[data-kind=sub] .se-sub'); s.value = 'off'; s.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
Start-Sleep -Milliseconds 300
PageClick '#seWindow .se-ok'
Check 'OK closes it' (WaitGoneWin 'PecoFence.SmartEditor' 10) ''
Check 'without subfolders: the file in sub leaves' (WaitSmart $D { -not $s.spec.subfolders -and -not ($n -match 'deep') } 15) (SmartLine $D)
CdpOpen | Out-Null
$s = JsWait "(() => { const t = document.getElementById('smartSummary').textContent; return t.includes('(with subfolders)') ? null : t; })()" 10
Check 'the line on Settings follows the save in the editor' ($s -eq 'Downloads folder · Last 7 days · Files only') "$s"
$t = Js "document.querySelector('#fenceSmartGroup [data-fence=portalTitleIcon]').classList.contains('on')"
PageClick '#fenceSmartGroup [data-fence=portalTitleIcon]'; Start-Sleep -Milliseconds 800
$after = Js "document.querySelector('#fenceSmartGroup [data-fence=portalTitleIcon]').classList.contains('on')"
Check 'the magnifier switch goes off and stays off after the host answers' ($t -and -not $after) "before=$t after=$after"
PageClick '#fenceSmartGroup [data-fence=portalTitleIcon]'; Start-Sleep -Milliseconds 500
Shot 'p02-after-editor'

Case 'without Everything the card offers to start it'
Pf fence set $D smartFolder null | Out-Null
Check 'Everything stops' (EvStop)
PageClick '#smartRefresh'; Start-Sleep -Seconds 1
Check 'Refresh now: the fence waits for Everything' (WaitSmart $D { $s.source -eq 'needsEverything' } 10) (SmartLine $D)
$g = JsWait "(() => { const b = document.getElementById('smartEverythingBtn'); return b.hidden ? null : b.textContent; })()" 10
Check 'the card has Start Everything and says the whole computer' ($g -eq 'Start Everything' -and (GroupState).summary -match '^Whole computer') "$g $((GroupState).summary)"
Shot 'p03-start-everything'
PageClick '#smartEverythingBtn'
for ($i = 0; $i -lt 30 -and -not (EvRunning); $i++) { Start-Sleep -Milliseconds 500 }
Check 'the button starts Everything and the fence fills' ((EvRunning) -and (WaitSmart $D { $s.source -eq 'everything' -and $s.shown -ge 1 } 40)) (SmartLine $D)
$log = (PfLogAll) -join "`n"
Check 'log: the changes, the editor and the refresh' ($log -match 'smart fence search changed.*what="within days"' -and $log -match 'what="match"' -and $log -match 'what="the whole search"' -and $log -match 'smart fence editor opened by="settings page"' -and $log -match 'smart fence: needs Everything, which is not running.*why="settings page"') ''
PfQuit
ClearDownloads
