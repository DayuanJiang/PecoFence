# Owner round 2026-11-12, second batch: sorting rules, snapshots and backups, export / import.
# One check per fix; runs on a 1600x900 box at 100 %, release build.
#   run.sh --var Part=S ...   a snapshot restore files the desktop items that came after it
#   run.sh --var Part=I ...   undoing an import with its snapshot brings the icons back
#   run.sh --var Part=N ...   the 「删除栅栏前」 snapshot leaves the other notes as they are
#   run.sh --var Part=L ...   CLI: a full list gives an automatic snapshot's place to the safety one
#   run.sh --var Part=X ...   a configuration without a 桌面 fence gets one
#   run.sh --var Part=R ...   a rule cannot point to a smart fence
#   run.sh --var Part=K ...   config locked at start: import and export say they cannot run
#   run.sh --var Part=U ...   locked at start and unreadable once the lock goes: the notice says so
#   run.sh --var Part=D ...   the notice of an unreadable configuration names the backup in use
#   run.sh --var Part=B ...   no "cannot save" when only the day's backup copy fails
#   run.sh --var Part=G ...   a snapshot restore glides the fences that moved (recorded)
#   run.sh --var Part=F ...   icons taken out of a fence fade out, however many (recorded)
#   run.sh --var Part=Z ...   立即应用整理规则 with nothing to sort says so
#   run.sh --bin target/wintest-bin-debug --var Part=P ...   the settings page (CDP): the notice
#                             after a restore has 撤销; no smart fence among a rule's targets
# Without --var every part but P runs (about 4 minutes).
if (-not $Part) { $Part = 'SINLXRKUDBGFZ' }
$HrCfg = "$PfApp\config\config.json"
function HrClean() { Get-ChildItem -LiteralPath $PfDesk -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'hr2-*' } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue }
function HrUtf8($path, $text) { [IO.File]::WriteAllText($path, $text, (New-Object Text.UTF8Encoding($false))) }
function HrItem($name) { return (@(PfJson item list) | Where-Object { $_ -and $_.fileName -eq $name } | Select-Object -First 1) }
function HrFresh() { HrClean; PfFresh; Pf settings set language zh-CN | Out-Null; Start-Sleep -Seconds 1 }
function HrSnap($name) { return (Snaps | Where-Object { $_.name -eq $name } | Select-Object -Last 1) }

Case 'setup'
CloseWindows
HrClean

if ($Part -match 'S') {
  Case 'S a snapshot restore files what came after it'
  HrFresh
  Pf snapshot save '整理之前' | Out-Null
  DeskFile 'hr2-快照之后来的 季度报告（终稿）.docx' | Out-Null
  Check 'the new file is filed' ((WaitItemFence 'hr2-快照之后来的 季度报告（终稿）.docx' '文件与文档') -eq '文件与文档')
  Pf snapshot restore (HrSnap '整理之前').id | Out-Null
  Start-Sleep -Milliseconds 1200
  $it = HrItem 'hr2-快照之后来的 季度报告（终稿）.docx'
  Check 'right after the restore the file is in a fence' ($it -and $it.fenceTitle -eq '文件与文档') "fence='$($it.fenceTitle)' listed=$([bool]$it)"
  PfQuit; HrClean
}

if ($Part -match 'I') {
  Case 'I undoing an import brings the icons back'
  HrFresh
  Pf config export "$PfRun\hr2-old.json" | Out-Null
  DeskFile 'hr2-导出之后的 照片.jpg' | Out-Null
  Pf fence create --title '手动整理' --rect '420,120,360,220' | Out-Null
  Start-Sleep -Seconds 2
  Pf item move 'hr2-导出之后的 照片.jpg' --to '手动整理' | Out-Null
  Check 'the file is in the fence it was put in' ((HrItem 'hr2-导出之后的 照片.jpg').fenceTitle -eq '手动整理')
  $imp = PfJson config import "$PfRun\hr2-old.json"
  Start-Sleep -Seconds 2
  Check 'the import took a snapshot first' ([bool]$imp.snapshotId) "$($imp | ConvertTo-Json -Compress)"
  Pf snapshot restore $imp.snapshotId | Out-Null
  Start-Sleep -Seconds 2
  $it = HrItem 'hr2-导出之后的 照片.jpg'
  Check 'the snapshot from before the import puts the icon back in its fence' ($it -and $it.fenceTitle -eq '手动整理') "fence='$($it.fenceTitle)' by=$($it.assignedBy)"
  PfQuit; HrClean
}

if ($Part -match 'N') {
  Case 'N the delete snapshot leaves the notes alone'
  HrFresh
  Pf fence create --tool note --title '待办' --text '第一版' --rect '420,120,260,160' | Out-Null
  Pf fence create --title '要删的栅栏' --rect '720,120,300,200' | Out-Null
  Start-Sleep -Seconds 2
  $f = PfFence '要删的栅栏'
  [G]::RClick($f.windowRect.x + 150, $f.windowRect.y + 14); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^(删除栅栏|Delete fence)'
  Start-Sleep -Seconds 1
  Check 'the fence is deleted from its menu' ($picked -and -not (PfFence '要删的栅栏')) "picked='$picked'"
  Pf fence set '待办' noteText '第二版，删除之后写的' | Out-Null
  $snap = HrSnap '删除栅栏前'
  Check 'a 「删除栅栏前」 snapshot was saved' ([bool]$snap)
  Pf snapshot restore $snap.id | Out-Null
  Start-Sleep -Seconds 1
  $text = (FenceGet '待办').noteText
  Check 'the fence is back and the note keeps its newer text' ((PfFence '要删的栅栏') -and $text -eq '第二版，删除之后写的') "note='$text'"
  # A note deleted from its menu comes back with its own text.
  Pf fence create --tool note --title '要删的便签' --text '删掉又回来的字' --rect '720,360,260,160' | Out-Null
  Start-Sleep -Seconds 2
  $n = PfFence '要删的便签'
  [G]::RClick($n.windowRect.x + 150, $n.windowRect.y + 14); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^(删除栅栏|Delete fence)'
  # A note with text asks first: Y is 「是」.
  Start-Sleep -Milliseconds 900
  [G]::Press(0x59)
  Start-Sleep -Seconds 1
  $gone = -not (PfFence '要删的便签')
  $snap = HrSnap '删除栅栏前'
  Pf snapshot restore $snap.id | Out-Null
  Start-Sleep -Seconds 1
  $back = (FenceGet '要删的便签').noteText
  Check 'a deleted note comes back with its text' ($picked -and $gone -and $back -eq '删掉又回来的字') "picked='$picked' gone=$gone note='$back'"
  PfQuit
}

if ($Part -match 'L') {
  Case 'L a full list still gives the CLI its safety snapshot'
  HrFresh
  foreach ($i in 1..19) { Pf snapshot save "我的快照 $i" | Out-Null }
  # As the settings page names the one it saves before a restore.
  Pf snapshot save '恢复快照前' | Out-Null
  Check '20 snapshots' (@(Snaps).Count -eq 20) "$(@(Snaps).Count)"
  $r = PfJson snapshot restore (HrSnap '我的快照 3').id
  Check 'the restore took its safety snapshot (the automatic one gave its place)' ($r.snapshotId -and -not $r.warning) "snapshotId=$($r.snapshotId) warning='$($r.warning)'"
  Check 'the snapshots I saved are all there' (@(Snaps | Where-Object { $_.name -like '我的快照*' }).Count -eq 19) "$(@(Snaps | Where-Object { $_.name -like '我的快照*' }).Count)"
  PfQuit
}

if ($Part -match 'X') {
  Case 'X a configuration without a 桌面 fence'
  HrFresh
  Pf config export "$PfRun\hr2-noinbox.json" | Out-Null
  $t = [IO.File]::ReadAllText("$PfRun\hr2-noinbox.json")
  HrUtf8 "$PfRun\hr2-noinbox.json" ($t -replace '"kind":\s*"inbox"', '"kind": "virtual"' -replace '"desktopLayout":\s*true', '"desktopLayout": false')
  $chk = Pf config check "$PfRun\hr2-noinbox.json"
  Check 'config check warns that the file has no 桌面 fence' ($chk -match 'no inbox fence') ($chk -replace '\s+', ' ')
  Pf config import "$PfRun\hr2-noinbox.json" | Out-Null
  Start-Sleep -Seconds 2
  $inbox = @(@(PfJson fence list) | Where-Object { $_ -and $_.kind -eq 'inbox' })
  Check 'after the import there is a 桌面 fence' ($inbox.Count -eq 1) "inbox fences: $($inbox.Count)"
  DeskFile 'hr2-没有规则管的文件.hr2x' | Out-Null
  Start-Sleep -Seconds 3
  $it = HrItem 'hr2-没有规则管的文件.hr2x'
  Check 'a file no rule matches shows in it' ($it -and $inbox.Count -eq 1 -and $it.fence -eq $inbox[0].id) "fence='$($it.fenceTitle)'"
  PfQuit; HrClean
}

if ($Part -match 'R') {
  Case 'R a rule cannot point to a smart fence'
  HrFresh
  Pf fence create --title '最近的文档' --smart-folder "$env:USERPROFILE\Documents" --ext 'txt' --rect '420,120,300,200' | Out-Null
  $r = Pf rule add --name '到智能栅栏' --ext 'hr2s' --to '最近的文档'
  Check 'rule add --to a smart fence is refused' ($r -match '"code":"unsupported"' -and $r -match 'smart fence') ($r -replace '\s+', ' ')
  Check 'no such rule was added' (-not (Rules | Where-Object { $_.name -eq '到智能栅栏' }))
  # A rule list whose rules point to a fence that is not there: stored, with a warning.
  $rl = Pf rule list
  HrUtf8 "$PfRun\hr2-rules-astray.json" ($rl -replace '"fence":\s*"[0-9a-f-]+"', '"fence":"00000000-0000-0000-0000-000000000001"')
  $imp = Pf rule import "$PfRun\hr2-rules-astray.json"
  Check 'rule import warns about rules whose fence does not exist' ($imp -match '"warning"' -and $imp -match 'target a fence that does not exist') ($imp -replace '\s+', ' ').Substring(0, [Math]::Min(160, ($imp -replace '\s+', ' ').Length))
  PfQuit
}

if ($Part -match 'K') {
  Case 'K import and export while the configuration is locked'
  HrFresh
  Pf config export "$PfRun\hr2-mine.json" | Out-Null
  PfQuit
  $lock = [IO.File]::Open($HrCfg, 'Open', 'ReadWrite', 'None')
  PfStart -Lang ''
  Check 'the app holds its saves' ([bool](PfWaitLog 'saves held until it can be read' 20))
  $e = Pf config export "$PfRun\hr2-while-locked.json"
  Check 'export says it cannot run now' ($e -match 'held open by another program' -and -not (Test-Path "$PfRun\hr2-while-locked.json")) ($e -replace '\s+', ' ')
  $i = Pf config import "$PfRun\hr2-mine.json"
  Check 'import says it cannot run now' ($i -match 'held open by another program') ($i -replace '\s+', ' ')
  $lock.Close()
  Check 'the configuration is read once the lock goes' ([bool](PfWaitLog 'config files readable again' 15))
  $e2 = Pf config export "$PfRun\hr2-after.json"
  Check 'export works again' (Test-Path "$PfRun\hr2-after.json") ($e2 -replace '\s+', ' ')
  PfQuit
}

if ($Part -match 'U') {
  Case 'U locked at start, and damaged when the lock goes'
  HrFresh
  Pf fence create --title '我的栅栏' --rect '420,120,300,200' | Out-Null
  Start-Sleep -Seconds 3
  Pf fence create --title '后来的栅栏' --rect '760,120,300,200' | Out-Null
  Start-Sleep -Seconds 3
  PfQuit
  $PfAllowLogErrors = $true
  $lock = [IO.File]::Open($HrCfg, 'Open', 'ReadWrite', 'None')
  PfStart -Lang ''
  Check 'the app holds its saves' ([bool](PfWaitLog 'saves held until it can be read' 20))
  # The program that held the file leaves it cut in half.
  $lock.SetLength([long]($lock.Length / 2))
  $lock.Close()
  $line = PfWaitLog 'config files readable again' 15
  Start-Sleep -Milliseconds 1500
  Crop 'U-notice' 1200 640 400 215 | Out-Null
  Check 'the notice says the file could not be used and which backup shows (not: your fences are back)' ($line -match 'told_unreadable=true') "$line"
  PfQuit
}

if ($Part -match 'D') {
  Case 'D the notice of an unreadable configuration'
  HrFresh
  Pf fence create --title '我的栅栏' --rect '420,120,300,200' | Out-Null
  Start-Sleep -Seconds 3
  # A second save: config.bak then holds the fence too.
  Pf fence create --title '后来的栅栏' --rect '760,120,300,200' | Out-Null
  Start-Sleep -Seconds 3
  PfQuit
  $good = [IO.File]::ReadAllText($HrCfg)
  HrUtf8 $HrCfg $good.Substring(0, [int]($good.Length / 2))
  $PfAllowLogErrors = $true
  PfStart -Lang ''
  Start-Sleep -Milliseconds 1500
  Crop 'D-notice' 1200 640 400 215 | Out-Null
  $line = PfWaitLog 'notice: config.json unreadable' 5
  Check 'the notice names the backup in use (no guess about a newer version)' ($line -match 'newer=false' -and $line -match 'from_backup=true') "$line"
  Check 'the fences of the backup show' ([bool](PfFence '我的栅栏'))
  PfQuit
}

if ($Part -match 'B') {
  Case 'B only the daily backup copy fails'
  HrFresh
  PfQuit
  Remove-Item -Recurse -Force "$PfApp\config\backups" -ErrorAction SilentlyContinue
  HrUtf8 "$PfApp\config\backups" 'a file where the backups folder should be'
  PfStart -Lang ''
  Pf fence create --title '备份写不进去时新建' --rect '420,120,300,200' | Out-Null
  Start-Sleep -Seconds 4
  $warn = PfWaitLog 'daily backup copy could not be written' 6
  $failed = @(PfLogAll | Where-Object { $_ -match 'config save failed' })
  Check 'the save counts as done: a log line, no failed save' ($warn -and $failed.Count -eq 0) "warn=$([bool]$warn) failed=$($failed.Count)"
  PfQuit
  $saved = [IO.File]::ReadAllText($HrCfg)
  Check 'config.json holds the change' ($saved -match '备份写不进去时新建')
  Remove-Item -Force "$PfApp\config\backups" -ErrorAction SilentlyContinue
}

if ($Part -match 'G') {
  Case 'G a snapshot restore glides the fences that moved'
  . C:\pf\kit\rec.ps1
  $anim0 = [Anim]::Get(); [Anim]::Set($true)
  HrFresh
  Pf fence create --title '工作资料' --rect '420,120,360,220' | Out-Null
  Start-Sleep -Seconds 1
  Pf snapshot save '整理之前' | Out-Null
  Pf fence move '工作资料' --x 700 --y 520 | Out-Null
  Start-Sleep -Seconds 2
  RecStart "G-restore" -X 380 -Y 80 -W 760 -H 700 -Fps 30
  Start-Sleep -Milliseconds 400
  RecMark '恢复快照'
  Pf snapshot restore (HrSnap '整理之前').id | Out-Null
  Start-Sleep -Milliseconds 1200
  RecStop
  $glide = @(PfLogAll | Where-Object { $_ -match 'restored fence glides to its saved place' })
  Check 'the fence that moved glides to its saved place' ($glide.Count -ge 1) "log lines: $($glide.Count)"
  $f = PfFence '工作资料'
  Check 'it ends at the saved place' ($f.windowRect.x -eq 420 -and $f.windowRect.y -eq 120) "$($f.windowRect.x),$($f.windowRect.y)"
  # With Windows' animations off the fence is at its place at once.
  [Anim]::Set($false)
  Start-Sleep -Seconds 1
  Pf fence move '工作资料' --x 700 --y 520 | Out-Null
  Start-Sleep -Seconds 1
  Pf snapshot restore (HrSnap '整理之前').id | Out-Null
  Start-Sleep -Milliseconds 150
  $f = PfFence '工作资料'
  $glide2 = @(PfLogAll | Where-Object { $_ -match 'restored fence glides to its saved place' })
  Check 'animations off: no glide, the fence is at its saved place at once' ($glide2.Count -eq $glide.Count -and $f.windowRect.x -eq 420 -and $f.windowRect.y -eq 120) "glides $($glide.Count) -> $($glide2.Count); at $($f.windowRect.x),$($f.windowRect.y)"
  [Anim]::Set($anim0)
  PfQuit
}

if ($Part -match 'F') {
  Case 'F icons taken out of a fence fade out'
  . C:\pf\kit\rec.ps1
  $anim0 = [Anim]::Get(); [Anim]::Set($true)
  HrFresh
  $rl = Pf rule list
  HrUtf8 "$PfRun\hr2-rules-off.json" ($rl -replace '"keepUpdated":\s*true', '"keepUpdated": false')
  Pf rule import "$PfRun\hr2-rules-off.json" | Out-Null
  foreach ($i in 1..10) { New-Item -ItemType Directory -Force (Join-Path $PfDesk ("hr2-文件夹 {0:D2}" -f $i)) | Out-Null }
  Start-Sleep -Seconds 3
  $folders = PfFence '文件夹'
  $r = $folders.windowRect
  Pf rule apply | Out-Null
  Start-Sleep -Seconds 2
  Check 'the rules put ten folders into 文件夹' ((PfFence '文件夹').itemCount -eq 10) "$((PfFence '文件夹').itemCount)"
  RecStart "F-leave" -X ($r.x - 10) -Y ($r.y - 10) -W ($r.w + 20) -H ($r.h + 20) -Fps 30
  Start-Sleep -Milliseconds 400
  RecMark '图标离开栅栏'
  Pf item move --glob 'hr2-文件夹*' --to inbox | Out-Null
  Start-Sleep -Milliseconds 900
  RecStop
  Check 'the folders left the fence' ((PfFence '文件夹').itemCount -eq 0) "$((PfFence '文件夹').itemCount)"
  $fade = @(PfLogAll | Where-Object { $_ -match 'icons leaving fade out' })
  Check 'they fade out (all ten at once is no refill)' ($fade.Count -ge 1) "log lines: $($fade.Count)"
  [Anim]::Set($anim0)
  PfQuit; HrClean
}

if ($Part -match 'Z') {
  Case 'Z 立即应用整理规则 with nothing to sort'
  . C:\pf\kit\user.ps1
  # In English: the box's OCR reads the notification.
  HrClean; PfFresh
  Start-Sleep -Seconds 2
  $picked = TrayPick '立即应用整理规则|Apply'
  $said = ToastOcr 'Nothing to sort|sorted' 6
  Crop 'Z-notice' 1200 640 400 215 | Out-Null
  Check 'the notification says that nothing needed sorting (not: 0 items sorted)' ($picked -and $said -match 'Nothing to sort') "picked='$picked' said='$said'"
  PfQuit
}

if ($Part -match 'P') {
  Case 'P the settings page: 撤销 after a restore, no smart fence among the rule targets'
  Check 'a debug build (the page is reached over CDP)' $PfDebugBuild
  HrClean
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config" -ErrorAction SilentlyContinue
  $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 2
  Pf fence create --title '工作资料' --rect '420,120,360,220' | Out-Null
  Pf fence create --title '最近的文档' --smart-folder "$env:USERPROFILE\Documents" --ext 'txt' --rect '420,380,300,200' | Out-Null
  Pf snapshot save '整理之前' | Out-Null
  Pf fence move '工作资料' --x 700 --y 520 | Out-Null
  Start-Sleep -Seconds 1
  Check 'settings open over CDP' (PfSettings)
  PageClick 'nav a[data-page=rules]'
  $targets = Js "(() => { document.getElementById('nrPanel').open = true; return [...document.querySelectorAll('#nrTarget option')].map(o => o.textContent).join('|'); })()"
  Check 'the target list of a new rule leaves the smart fence out' ($targets -match '工作资料' -and $targets -notmatch '最近的文档') "$targets"
  PageClick 'nav a[data-page=layout]'
  $id = (HrSnap '整理之前').id
  PageClick "#snapList [data-row-id=`"$id`"] [data-act=restore]"
  Start-Sleep -Milliseconds 900
  $t = PageToast
  $undo = Js "!document.getElementById('toastUndo').hidden"
  Check 'the notice 已恢复快照 has 撤销' ($t.text -eq '已恢复快照' -and $undo) "toast='$($t.text)' undo=$undo"
  $f = PfFence '工作资料'
  Check 'the fence is back at its saved place' ($f.rect.x -eq 420 -and $f.rect.y -eq 120) "$($f.rect.x),$($f.rect.y)"
  $rows = Js "[...document.querySelectorAll('#snapList .card')].map(c => c.querySelector('.t').textContent + ': ' + c.querySelector('.s').textContent).join(' | ')"
  Check 'the list marks the automatic snapshot, not the one I saved' ($rows -match '恢复快照前: [^|]*自动保存' -and $rows -notmatch '整理之前: [^|]*自动保存') "$rows"
  Shot 'P-restore-notice'
  if ($undo) { PageClick '#toastUndo' }
  Start-Sleep -Milliseconds 1200
  $f = PfFence '工作资料'
  Check '撤销 puts the layout back as it was before the restore' ($f.rect.x -eq 700 -and $f.rect.y -eq 520) "$($f.rect.x),$($f.rect.y)"
  Check 'the 「恢复快照前」 snapshot is gone again' (-not (HrSnap '恢复快照前')) "$((Snaps | ForEach-Object { $_.name }) -join '; ')"
  PfQuit
  $env:PECOFENCE_WEBVIEW_ARGS = $null
}

Case 'cleanup'
HrClean
