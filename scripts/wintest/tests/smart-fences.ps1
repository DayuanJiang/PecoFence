# 智能栅栏 with Everything 1.4 (installed on the box, see smart.ps1): every way to make one (CLI
# template, CLI conditions, tray menu, fence menu, the Peek search's last row) and what a smart
# fence does: Everything's results, the count after the name, live updates (new, subfolder,
# deleted, written again, a download in flight), F5 and 立即刷新, fence set on each search
# property, no drops or paste into it, files left where they are, a restart, deleting it.
Case 'setup'
CloseWindows
DownloadsScene
AgedFile $PfDocs 'poster design.psd' 30 | Out-Null
AgedFile $PfDocs '封面.psd' 3 | Out-Null
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class Rd { [DllImport("shell32.dll", CharSet = CharSet.Unicode)] public static extern void SHAddToRecentDocs(uint flags, string path); }
"@
$recent = [Environment]::GetFolderPath('Recent')
foreach ($n in @('weekly.docx', 'budget.xlsx', 'slides.pptx')) { [Rd]::SHAddToRecentDocs(3, (AgedFile $PfDocs $n 1)) }
Check 'Everything 1.4 runs in pf session' (EvStart) "installed=$(EvInstalled)"
PfStart -Lang en -Script (DumpScript 500 20)
Start-Sleep -Seconds 2
$Dl = 'Downloads of the last 7 days'

Case 'CLI: fence create --smart downloads'
$r = Pf fence create --smart downloads --rect '420,140,560,330'
$f = FenceGet $Dl
Check 'a virtual fence with the downloads search' ($f -and $f.kind -eq 'virtual' -and $f.smart.spec.template -eq 'downloads' -and $f.smart.spec.withinDays -eq 7) $r
Check 'Everything answers with the six files of the last week' (WaitSmart $Dl { $s.source -eq 'everything' -and $s.shown -eq 6 } 40) (SmartLine $Dl)
$n = SmartNames $Dl
Check 'none of the older files, newest first' (-not ($n -match 'old contract|annual|drivers|readme|wallpaper|简历') -and $n[0] -match 'quarterly' -and $n[-1] -match 'photos') ($n -join ', ')
$q = (SmartOf $Dl).query
Check 'the search names the folder, files and both dates' ($q -match 'Downloads\\' -and $q -match 'file:' -and $q -match '<dm:>=\d{4}-\d\d-\d\d\|dc:>=') $q
$w = DumpOf $Dl
Check 'the count is drawn after the name, the name stays the name' ($w -and $w.title -eq $Dl -and $w.raw -match 'title_count=Some\(" · 6"\)') ($w.raw.Substring(0, [math]::Min(220, $w.raw.Length)))
Check 'a smart fence holds no items of its own (item list says smart)' (@(@(PfJson item list --fence $Dl) | Where-Object { $_ -and $_.assignedBy -ne 'smart' }).Count -eq 0) ''
Shot 's01-downloads'

Case 'live updates'
AgedFile $PfDl 'new invoice.pdf' 0 | Out-Null
$t0 = Get-Date
Check 'a new download shows (within 15 s)' (WaitSmart $Dl { $n -match 'new invoice' } 15) "$([int]((Get-Date) - $t0).TotalSeconds) s; $(SmartLine $Dl)"
$w = DumpOf $Dl
Check 'the count follows: · 7' ($w.raw -match 'title_count=Some\(" · 7"\)') ([regex]::Match($w.raw, 'title_count=\S+ \S*').Value)
AgedFile "$PfDl\project" 'requirements.docx' 0.2 | Out-Null
Check 'a new file in a subfolder shows' (WaitSmart $Dl { $n -match 'requirements' } 15) (SmartLine $Dl)
Remove-Item -LiteralPath (Join-Path $PfDl '数据表.xlsx')
Check 'a deleted file leaves' (WaitSmart $Dl { -not ($n -match '数据表') } 15) (SmartLine $Dl)
(Get-Item -LiteralPath (Join-Path $PfDl 'readme.txt')).LastWriteTime = Get-Date
Check 'an old file written again comes in' (WaitSmart $Dl { $n -match 'readme' } 15) (SmartLine $Dl)
AgedFile $PfDl 'big.iso.crdownload' 0 | Out-Null
Start-Sleep -Seconds 7
Check 'a download in flight stays out' (-not ((SmartNames $Dl) -match 'crdownload')) (SmartLine $Dl)
Remove-Item -LiteralPath (Join-Path $PfDl 'big.iso.crdownload')

Case 'F5 and fence refresh run the search again'
FocusFence $Dl
[G]::Press(0x74); Start-Sleep -Milliseconds 800
Pf fence refresh $Dl | Out-Null; Start-Sleep -Milliseconds 800
$log = (PfLogAll) -join "`n"
Check 'F5 asks Everything again' ($log -match 'smart fence: Everything asked.*why="f5"') ''
Check 'fence refresh asks Everything again' ($log -match 'smart fence: Everything asked.*why="cli"') ''

Case 'nothing is dropped or pasted into it, nothing is moved'
$desk = DeskFile 'smart-drop-test.txt'
$from = $null
for ($i = 0; $i -lt 10 -and -not $from; $i++) { Start-Sleep -Seconds 1; $from = ItemFence 'smart-drop-test.txt' }
$src = ItemPt $from 'smart-drop-test'
$wr = (FenceGet $Dl).windowRect
if ($src) { DragTo $src[0] $src[1] ([int]($wr.x + $wr.w / 2)) ([int]($wr.y + $wr.h - 40)); Start-Sleep -Seconds 1 }
Check 'a desktop item dragged onto it stays where it was' ($from -and (ItemFence 'smart-drop-test.txt') -eq $from -and (Test-Path -LiteralPath $desk)) "from=$from now=$(ItemFence 'smart-drop-test.txt') src=$src"
Set-Clipboard -LiteralPath $desk
FocusFence $Dl
Chord 0x11 0x56; Start-Sleep -Seconds 1
Check 'Ctrl+V into it pastes nothing' (-not (Test-Path -LiteralPath (Join-Path $PfDl 'smart-drop-test.txt')) -and ((PfLogAll) -join "`n") -match 'paste into a tool or smart fence ignored') ''
Remove-Item -LiteralPath $desk
Check 'the files are still in Downloads, none on the desktop' ((Test-Path -LiteralPath (Join-Path $PfDl 'quarterly report.pdf')) -and -not (Get-ChildItem -LiteralPath $PfDesk | Where-Object { $_.Name -match 'quarterly' })) ''

Case 'its menu: refresh, folder, no New or Paste'
$m = FenceMenuOf $Dl
$t = MenuTexts $m
Check 'Refresh now F5 and Open folder in File Explorer' ($t -match 'Refresh now ~ F5' -and $t -match 'Open folder in File Explorer') $t
Check 'no New and no Paste' (-not ($t -match '(^| )New \|') -and -not ($t -match 'Paste')) $t
Check 'the New smart fence submenu' ($t -match 'New smart fence') $t
Shot 's02-fence-menu'
CloseMenus

Case 'tray menu: New smart fence › Recently opened files'
$m = TrayMenu
$t = MenuTexts $m
Check 'New smart fence follows Show a folder on the desktop' ($t -match 'Show a folder on the desktop \| New smart fence') $t
$sub = OpenSubmenu $m '^New smart fence' 'Downloads of the last 7 days'
$st = MenuTexts $sub
Check 'five templates' ($st -eq 'Downloads of the last 7 days | Screenshots of the last week | Files over 1 GB | All .psd files | Recently opened files') $st
Shot 's03-tray-submenu'
MenuPick '^Recently opened files' $sub | Out-Null
Check 'recent files: the three documents opened' (WaitSmart 'Recently opened files' { $s.source -eq 'everything' -and (($n -join ',') -match 'weekly') -and (($n -join ',') -match 'slides') } 30) (SmartLine 'Recently opened files')
CloseMenus

Case 'fence menu: New smart fence › All .psd files'
$m = FenceMenuOf $Dl
$sub = OpenSubmenu $m '^New smart fence' 'All \.psd files'
MenuPick '^All \.psd files' $sub | Out-Null
Check 'both .psd files of Documents, from the whole computer' (WaitSmart 'All .psd files' { $s.source -eq 'everything' -and ($n -match 'poster') -and ($n -match '封面') } 30) (SmartLine 'All .psd files')
Check 'Windows, AppData and the Recycle Bin are left out' ((SmartOf 'All .psd files').query -match '^!"C:\\Windows\\" !\\AppData\\ ') (SmartOf 'All .psd files').query
CloseMenus

Case 'fence set: each search property'
$Seven = 'Downloads of the last 7 days'
Pf fence set $Seven smartWithinDays 30 | Out-Null
Check 'smartWithinDays 30: the files of 12 and 25 days come in' (WaitSmart $Seven { $n -match 'old contract' -and $n -match 'wallpaper' -and -not ($n -match 'drivers') } 15) (SmartLine $Seven)
Pf fence set $Seven smartWithinDays null | Out-Null
Check 'smartWithinDays null: every file' (WaitSmart $Seven { $n -match 'drivers' -and $n -match '简历' } 15) (SmartLine $Seven)
Pf fence set $Seven smartConditions '[{\"cond\":\"ext\",\"value\":[\"pdf\"]}]' | Out-Null
Check 'smartConditions ext pdf: only PDFs' (WaitSmart $Seven { $n.Count -ge 2 -and -not ($n | Where-Object { $_ -notmatch 'pdf|quarterly|invoice|contract' }) } 15) (SmartLine $Seven)
Pf fence set $Seven smartName 'quart' | Out-Null
Check 'smartName quart: one file' (WaitSmart $Seven { $n.Count -eq 1 -and $n[0] -match 'quarterly' } 15) (SmartLine $Seven)
Pf fence set $Seven smartName null | Out-Null
Pf fence set $Seven smartLimit 2 | Out-Null
Check 'smartLimit 2: two shown, the count says more' (WaitSmart $Seven { $s.shown -eq 2 } 15) (SmartLine $Seven)
$w = DumpOf $Seven
Check 'the count after the name reads · 2+' ($w.raw -match 'title_count=Some\(" · 2\+"\)') ([regex]::Match($w.raw, 'title_count=\S+ \S*').Value)
Pf fence set $Seven smartLimit 200 | Out-Null
Pf fence set $Seven smartConditions '[]' | Out-Null
Pf fence set $Seven smartSubfolders false | Out-Null
Check 'smartSubfolders false: the subfolder file leaves' (WaitSmart $Seven { -not ($n -match 'requirements') -and ($n -match 'quarterly') } 15) (SmartLine $Seven)
Pf fence set $Seven smartFolder $PfDocs | Out-Null
Check 'smartFolder Documents: the documents, no downloads' (WaitSmart $Seven { $n -match 'weekly' -and -not ($n -match 'quarterly') } 15) (SmartLine $Seven)
Check 'a folder picked by hand drops the template' ($null -eq (SmartOf $Seven).spec.template) ((SmartOf $Seven).spec | ConvertTo-Json -Compress)
Pf fence set $Seven smartEverything 'quarterly' | Out-Null
Pf fence set $Seven smartFolder $PfDl | Out-Null
Check 'smartEverything: the typed search in the folder' (WaitSmart $Seven { $n.Count -eq 1 -and $n[0] -match 'quarterly' } 15) (SmartLine $Seven)
# The typed search narrows the other conditions, it does not replace them.
Pf fence set $Seven smartEverything 'pdf' | Out-Null
Check 'smartEverything pdf: the three PDFs' (WaitSmart $Seven { $n.Count -eq 3 -and $n -match 'old contract' } 15) (SmartLine $Seven)
Pf fence set $Seven smartWithinDays 7 | Out-Null
Check 'with smartWithinDays 7 the time window still applies' (WaitSmart $Seven { $n.Count -eq 2 -and -not ($n -match 'old contract') } 15) (SmartLine $Seven)
Pf fence set $Seven smartWithinDays null | Out-Null
Pf fence set $Seven smartName 'contract' | Out-Null
Check 'and the name filter too' (WaitSmart $Seven { $n.Count -eq 1 -and $n[0] -match 'old contract' } 15) (SmartLine $Seven)
Pf fence set $Seven smartName null | Out-Null
Pf fence set $Seven smartEverything null | Out-Null
$bad = & "$PfApp\pecofence-cli.exe" fence set Programs smartWithinDays 7 2>&1 | Out-String
Check 'a search property on a plain fence is refused' ($LASTEXITCODE -ne 0 -and $bad -match 'smart fences') $bad
$bad = & "$PfApp\pecofence-cli.exe" fence set $Seven smartFolder 'C:\no\such\folder' 2>&1 | Out-String
Check 'a folder that is not there is refused' ($LASTEXITCODE -ne 0 -and $bad -match 'no folder') $bad
$log = (PfLogAll) -join "`n"
Check 'log: each change with what changed' ($log -match 'smart fence search changed.*what="within days"' -and $log -match 'what="conditions"' -and $log -match 'what="folder"') ''

Case 'Peek search: put this search on the desktop'
Pf settings set tools.everything true | Out-Null
Start-Sleep -Milliseconds 500
PeekKey
$box = WaitWin 'PecoFence.PeekSearch' 5
Start-Sleep -Milliseconds 400
[G]::Type('quarterly'); Start-Sleep -Seconds 2
Shot 's04-peek-rows'
[G]::Press(0x26); Start-Sleep -Milliseconds 300
[G]::Press(0x0D); Start-Sleep -Seconds 2
$made = @(@(PfJson fence list) | Where-Object { $_ -and $_.smart -and $_.smart.spec.everything -eq 'quarterly' })
Check 'the last row makes a smart fence with that search' ($made.Count -eq 1 -and $made[0].title -match 'quarterly') (($made | ForEach-Object { $_.title }) -join ', ')
if ($made.Count -eq 1) { Check 'it finds the file' (WaitSmart $made[0].title { $n -match 'quarterly' } 20) (SmartLine $made[0].title) }

Case 'a restart keeps the smart fences'
PfQuit
PfStart -Lang '' -Script (DumpScript 500 10)
Check 'the downloads fence searches again after the restart' (WaitSmart $Dl { $s.source -eq 'everything' -and $s.shown -ge 6 } 30) (SmartLine $Dl)
Shot 's05-after-restart'

Case 'delete a smart fence'
Pf fence delete 'All .psd files' | Out-Null
Start-Sleep -Seconds 1
Check 'the .psd files stay in Documents' ((Test-Path -LiteralPath (Join-Path $PfDocs 'poster design.psd')) -and (Test-Path -LiteralPath (Join-Path $PfDocs '封面.psd'))) ''
Check 'no .psd went to a fence of icons' (@(@(PfJson item list) | Where-Object { $_ -and $_.name -match 'poster|封面' }).Count -eq 0) ''
$log = (PfLogAll) -join "`n"
Check 'log: created, asked, answered, changed, watcher' ($log -match 'smart fence created' -and $log -match 'smart fence: Everything answered' -and $log -match 'smart fence results changed' -and $log -match 'smart fence watcher started.*subfolders=true') ''
PfQuit
Remove-Item -LiteralPath (Join-Path $PfDocs 'poster design.psd'), (Join-Path $PfDocs '封面.psd') -ErrorAction SilentlyContinue
foreach ($n in @('weekly.docx', 'budget.xlsx', 'slides.pptx')) { Remove-Item -LiteralPath (Join-Path $PfDocs $n) -ErrorAction SilentlyContinue; Remove-Item -LiteralPath (Join-Path $recent "$n.lnk") -ErrorAction SilentlyContinue }
ClearDownloads
