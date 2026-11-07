# 设置 › 栅栏 › 搜索条件 on the real page (a debug build: build.sh --debug, run.sh --bin
# target/wintest-bin-debug): every box of the group changes the smart fence's search and the
# page shows what the host answers; 选择文件夹… opens the folder picker; without Everything the
# card offers to start it and the fence fills afterwards. Needs Everything (see smart.ps1).
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
function GroupState() { return Js "(() => ({ shown: document.getElementById('fenceSmartGroup').style.display !== 'none', where: document.getElementById('smartWhereText').textContent, scope: document.getElementById('smartScope').value, days: document.getElementById('smartDays').value, sub: document.getElementById('smartSub').classList.contains('on'), limit: document.getElementById('smartLimit').value, name: document.getElementById('smartName').value, conds: [...document.querySelectorAll('#smartConds .cond .c-kind')].map(k => k.value), status: document.getElementById('smartStatus').textContent, help: document.getElementById('smartStatusHelp').textContent, evBtn: document.getElementById('smartEverythingBtn').hidden ? '' : document.getElementById('smartEverythingBtn').textContent }))()" }

Case 'the group shows the search'
Check 'settings open over CDP' (PfSettings)
PageClick 'nav a[data-page=fences]'; Start-Sleep -Milliseconds 600
PageSet 'fenceSel' $id
$g = GroupState
Check 'shown for the smart fence, with its folder, time, conditions and limit' ($g.shown -and $g.where -match 'Downloads, with subfolders' -and $g.scope -eq 'folder' -and $g.days -eq '7' -and $g.sub -and ($g.conds -join ',') -eq 'files' -and $g.limit -eq '200') ($g | ConvertTo-Json -Compress)
Check 'the status says what the last run found' ($g.status -eq 'Showing 7 files now' -and $g.help -match 'Searched by Everything' -and -not $g.evBtn) ($g | ConvertTo-Json -Compress)
Shot 'p01-group'
$plain = (@(PfJson fence list) | Where-Object { $_ -and -not $_.smart -and $_.kind -eq 'virtual' } | Select-Object -First 1).id
PageSet 'fenceSel' $plain
Check 'hidden for a fence of icons' (-not (GroupState).shown) ''
PageSet 'fenceSel' $id

Case 'every box changes the search'
PageSet 'smartDays' '30'
Check 'time: last 30 days brings the older files of 12 and 25 days' (WaitSmart $D { $s.spec.withinDays -eq 30 -and $n -match 'old contract' -and $n -match 'wallpaper' } 15) (SmartLine $D)
PageSet 'smartDays' ''
Check 'time: any time' (WaitSmart $D { $null -eq $s.spec.withinDays -and $n -match 'drivers' } 15) (SmartLine $D)
PageClick '#smartSub'; Start-Sleep -Milliseconds 800
Check 'include subfolders off: the file in sub leaves' (WaitSmart $D { -not $s.spec.subfolders -and -not ($n -match 'deep') } 15) (SmartLine $D)
PageClick '#smartSub'; Start-Sleep -Milliseconds 800
Check 'and on again' (WaitSmart $D { $s.spec.subfolders -and $n -match 'deep' } 15) (SmartLine $D)
PageType '#smartName' 'quart'; [G]::Press(0x0D); Start-Sleep -Milliseconds 800
Check 'name: quart leaves one file' (WaitSmart $D { $s.spec.name -eq 'quart' -and $s.shown -eq 1 } 15) (SmartLine $D)
PageType '#smartName' ''; [G]::Press(0x0D); Start-Sleep -Milliseconds 800
Check 'name emptied: no name filter' (WaitSmart $D { $null -eq $s.spec.name -and $s.shown -gt 1 } 15) (SmartLine $D)
PageClick '#smartAddCond'; Start-Sleep -Milliseconds 400
Js "(() => { const k = document.querySelectorAll('#smartConds .c-kind')[1]; k.value = 'ext'; k.dispatchEvent(new Event('change', { bubbles: true })); return true; })()" | Out-Null
PageType '#smartConds .c-text' 'pdf' 1
PageClick '#smartSaveConds'; Start-Sleep -Milliseconds 800
Check 'a condition added and saved: only PDFs' (WaitSmart $D { @($s.spec.allOf).Count -eq 2 -and $n.Count -ge 2 -and -not ($n | Where-Object { $_ -notmatch 'quarterly|contract|deep' }) } 15) (SmartLine $D)
Check 'the page toast says saved' ((PageToast).text -match 'Conditions saved') ((PageToast) | ConvertTo-Json -Compress)
PageClick '#smartConds .c-del' 1; PageClick '#smartSaveConds'; Start-Sleep -Milliseconds 800
Check 'the condition removed again' (WaitSmart $D { @($s.spec.allOf).Count -eq 1 } 15) (SmartLine $D)
PageSet 'smartLimit' '50'
Check 'show at most 50' (WaitSmart $D { $s.spec.limit -eq 50 } 10) (SmartLine $D)
Js "(() => { document.getElementById('smartAdvanced').open = true; return true; })()" | Out-Null
PageType '#smartEverything' 'meeting'; [G]::Press(0x0D); Start-Sleep -Milliseconds 800
Check 'Everything syntax narrows the search with the conditions' (WaitSmart $D { $s.spec.everything -eq 'meeting' -and $s.shown -eq 1 } 15) (SmartLine $D)
PageType '#smartEverything' ''; [G]::Press(0x0D); Start-Sleep -Milliseconds 800
Check 'and gone again' (WaitSmart $D { $null -eq $s.spec.everything -and $s.shown -gt 1 } 15) (SmartLine $D)
$t = Js "document.querySelector('#fenceSmartGroup [data-fence=portalTitleIcon]').classList.contains('on')"
PageClick '#fenceSmartGroup [data-fence=portalTitleIcon]'; Start-Sleep -Milliseconds 800
$after = Js "document.querySelector('#fenceSmartGroup [data-fence=portalTitleIcon]').classList.contains('on')"
Check 'the magnifier switch goes off and stays off after the host answers' ($t -and -not $after) "before=$t after=$after"
PageClick '#fenceSmartGroup [data-fence=portalTitleIcon]'; Start-Sleep -Milliseconds 500

Case 'where to look: the folder picker and the whole computer'
PageClick '#smartPick'; Start-Sleep -Seconds 2
$dlg = @([W]::Tops('#32770') | Where-Object { [L]::Text($_) -match 'smart fence searches' })
Check 'Choose folder… opens the folder picker' ($dlg.Count -ge 1) (([W]::Tops('#32770') | ForEach-Object { [L]::Text($_) }) -join ' | ')
if ($dlg.Count -ge 1) { [G]::SetForegroundWindow($dlg[0]) | Out-Null; Start-Sleep -Milliseconds 300; [G]::Press(0x1B); Start-Sleep -Seconds 1 }
Check 'cancelled: the folder stays' ((SmartOf $D).spec.folder -eq $PfDl -and ((PfLogAll) -join "`n") -match 'folder picker cancelled') ''
PageSet 'smartScope' 'all'
Check 'the whole computer' (WaitSmart $D { $null -eq $s.spec.folder -and $s.query -match '^!"C:\\Windows\\"' } 15) (SmartLine $D)
$g = GroupState
Check 'the page says the whole computer and hides include subfolders' ($g.scope -eq 'all' -and $g.where -match 'whole computer' -and (Js "document.getElementById('smartSubCard').style.display") -eq 'none') ($g | ConvertTo-Json -Compress)
Shot 'p02-whole-computer'

Case 'without Everything the card offers to start it'
Check 'Everything stops' (EvStop)
PageClick '#smartRefresh'; Start-Sleep -Seconds 1
Check 'Refresh now: the fence waits for Everything' (WaitSmart $D { $s.source -eq 'needsEverything' } 10) (SmartLine $D)
$g = JsWait "(() => { const b = document.getElementById('smartEverythingBtn'); return b.hidden ? null : b.textContent; })()" 10
Check 'the card has Start Everything' ($g -eq 'Start Everything') "$g"
Shot 'p03-start-everything'
PageClick '#smartEverythingBtn'
for ($i = 0; $i -lt 30 -and -not (EvRunning); $i++) { Start-Sleep -Milliseconds 500 }
Check 'the button starts Everything and the fence fills' ((EvRunning) -and (WaitSmart $D { $s.source -eq 'everything' -and $s.shown -ge 1 } 40)) (SmartLine $D)
$log = (PfLogAll) -join "`n"
Check 'log: the settings page changes and the refresh' ($log -match 'smart fence search changed.*what="within days"' -and $log -match 'what="name"' -and $log -match 'smart fence: needs Everything, which is not running.*why="settings page"') ''
PfQuit
ClearDownloads
