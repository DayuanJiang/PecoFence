# 筛选 in a folder fence and a smart fence: Ctrl+F and the fence menu open the box over the
# title row, every key filters at once (Peek search matching: pinyin initials, Chinese, no
# loose letters-in-order match), Enter keeps the filter and the title says so, Esc in the box or
# the fence and 清除筛选 clear it, another folder shown clears it, a smart fence filters too
# and a new matching file joins it, 把筛选存为智能栅栏 makes a smart fence of the filter. Runs
# without Everything (the folder fence needs none; the smart fence reads its folder).
Case 'setup'
CloseWindows
DownloadsScene
AgedFile $PfDl '季度报告.pdf' 1 | Out-Null
AgedFile "$PfDl\project files" 'spec.docx' 30 | Out-Null
EvStop | Out-Null
PfStart -Lang en -Script (DumpScript 500 15)
Start-Sleep -Seconds 2
Pf fence create --portal $PfDl --title 'Downloads' --rect '420,120,560,360' | Out-Null
Start-Sleep -Seconds 2
$P = 'Downloads'
$all = @(PfJson item list --fence $P).Count

Case 'Ctrl+F opens the box; typing filters at once'
FocusFence $P
Chord 0x11 0x46; Start-Sleep -Milliseconds 600
Check 'Ctrl+F opens the filter box' (FilterBoxOpen) (Fg)
Shot 'r01-box-empty'
[G]::Type('pdf'); Start-Sleep -Milliseconds 600
Check 'pdf: the three PDFs' (WaitFilterShown $P 3) ((FilterOf $P) | ConvertTo-Json -Compress)
$cnt = @(PfLogAll | Where-Object { $_ -match 'filter box count' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'the box says how many show: 3 / all' ($cnt -match 'count="?3 / \d+') "$cnt"
$w = DumpOf $P
Check 'while the box is open the title is the plain name' ($w.title -eq $P) $w.title
[G]::Press(0x0D); Start-Sleep -Milliseconds 500
Check 'Enter closes the box and keeps the filter' (-not (FilterBoxOpen) -and (FilterOf $P).text -eq 'pdf') ((FilterOf $P) | ConvertTo-Json -Compress)
$w = DumpOf $P
Check 'the title stays, a tag says what it is filtered by and how many show' ($w.title -eq $P -and $w.raw -match ' filtered=true' -and $w.raw -match 'filter_tag=Some\(\("pdf", "3 / \d+"\)\)') ([regex]::Match($w.raw, 'filter_tag=\S+ \S+').Value)
Shot 'r02-filtered'
Check 'the folder itself is untouched' (@(Get-ChildItem -LiteralPath $PfDl).Count -eq $all) "items=$all files=$(@(Get-ChildItem -LiteralPath $PfDl).Count)"
FocusFence $P
[G]::Press(0x1B); Start-Sleep -Milliseconds 600
Check 'Esc in the fence clears the filter' (WaitFilterGone $P) ((FilterOf $P) | ConvertTo-Json -Compress)

Case 'matching: pinyin initials, Chinese, nothing loose'
FocusFence $P
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('jdbg'); Start-Sleep -Milliseconds 600
Check 'jdbg (pinyin initials) finds 季度报告' (WaitFilterShown $P 1) ((FilterOf $P) | ConvertTo-Json -Compress)
[G]::Press(0x1B); Start-Sleep -Milliseconds 500
Check 'Esc in the box clears and closes' (-not (FilterBoxOpen) -and (WaitFilterGone $P)) ((FilterOf $P) | ConvertTo-Json -Compress)
FocusFence $P
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('数据'); Start-Sleep -Milliseconds 600
Check '数据 finds one file' (WaitFilterShown $P 1) ((FilterOf $P) | ConvertTo-Json -Compress)
[G]::Type('xyz'); Start-Sleep -Milliseconds 600
Check 'no match: an empty fence' (WaitFilterShown $P 0) ((FilterOf $P) | ConvertTo-Json -Compress)
Shot 'r03-no-match'
[G]::Press(0x1B); Start-Sleep -Milliseconds 400

Case 'the fence menu: Filter…, Clear filter'
$m = FenceMenuOf $P
Check 'the folder fence menu has Filter… Ctrl+F' ((MenuTexts $m) -match 'Filter… ~ Ctrl\+F') (MenuTexts $m)
MenuPick '^Filter' $m | Out-Null; Start-Sleep -Milliseconds 600
Check 'Filter… opens the box' (FilterBoxOpen)
[G]::Type('zip'); Start-Sleep -Milliseconds 600
Check 'zip: the two archives' (WaitFilterShown $P 2) ((FilterOf $P) | ConvertTo-Json -Compress)
[G]::Press(0x0D); Start-Sleep -Milliseconds 400
$m = FenceMenuOf $P
$t = MenuTexts $m
Check 'filtered: Clear filter and Save filter as a smart fence' ($t -match 'Clear filter ~ Esc' -and $t -match 'Save filter as a smart fence') $t
MenuPick '^Clear filter' $m | Out-Null; Start-Sleep -Milliseconds 600
Check 'Clear filter shows everything again' (WaitFilterGone $P) ((FilterOf $P) | ConvertTo-Json -Compress)

Case 'another folder shown clears the filter'
FocusFence $P
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('project'); Start-Sleep -Milliseconds 500
[G]::Press(0x0D); Start-Sleep -Milliseconds 400
Check 'project: only the subfolder' (WaitFilterShown $P 1) ((FilterOf $P) | ConvertTo-Json -Compress)
Pf fence open-folder $P 'project files' | Out-Null; Start-Sleep -Milliseconds 800
Check 'inside the subfolder the filter is gone' ($null -eq (FilterOf $P) -and (FenceGet $P).portal.current -match 'project files') ((FenceGet $P).portal | ConvertTo-Json -Compress)
Pf fence up $P | Out-Null; Start-Sleep -Milliseconds 500

Case 'Save filter as a smart fence'
FocusFence $P
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('pdf'); Start-Sleep -Milliseconds 500
[G]::Press(0x0D); Start-Sleep -Milliseconds 400
$m = FenceMenuOf $P
MenuPick '^Save filter as a smart fence' $m | Out-Null
Start-Sleep -Seconds 2
$Saved = 'Downloads: pdf'
$spec = (SmartOf $Saved).spec
Check 'a smart fence of the folder and the filter' ($spec -and @($spec.folders) -join '|' -eq $PfDl -and $spec.name -eq 'pdf' -and -not $spec.subfolders) ($spec | ConvertTo-Json -Compress)
Check 'it shows the same three PDFs' (WaitSmart $Saved { $s.shown -eq 3 } 20) (SmartLine $Saved)
AgedFile $PfDl 'fresh.pdf' 0 | Out-Null
Check 'a new PDF joins it' (WaitSmart $Saved { $n -match 'fresh' } 15) (SmartLine $Saved)
Shot 'r04-saved-as-smart'

Case 'a smart fence filters too'
$D = 'Downloads of the last 7 days'
Pf fence create --smart downloads --rect '1000,120,420,360' | Out-Null
Check 'the downloads fence is filled' (WaitSmart $D { $s.shown -ge 6 } 20) (SmartLine $D)
FocusFence $D
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('pdf'); Start-Sleep -Milliseconds 600
Check 'the smart fence: pdf leaves the recent PDFs' (WaitFilterShown $D 3) ((FilterOf $D) | ConvertTo-Json -Compress)
[G]::Press(0x0D); Start-Sleep -Milliseconds 300
AgedFile $PfDl 'newer.pdf' 0 | Out-Null
Check 'a new matching download joins the filtered fence' (WaitFilterShown $D 4 15) ((FilterOf $D) | ConvertTo-Json -Compress)
$w = DumpOf $D
Check 'the count capsule says what the filter shows' ($w.raw -match 'title_count=Some\("4"\)') ([regex]::Match($w.raw, 'title_count=\S+ \S*').Value)
Shot 'r05-smart-filtered'
Case 'save the filter of a smart fence: all, any of one row, any of two rows'
# 全部: the filter is one more condition.
$m = FenceMenuOf $D
MenuPick '^Save filter as a smart fence' $m | Out-Null
Start-Sleep -Seconds 2
$s = SmartOf "${D}: pdf"
Check 'an all fence: saved, the filter as its name, all' ($s -and $s.match -eq 'all' -and $s.spec.name -eq 'pdf' -and $s.spec.withinDays -eq 7) ($s.spec | ConvertTo-Json -Compress -Depth 4)
Check 'it shows what the filtered fence showed (the recent PDFs)' (WaitSmart "${D}: pdf" { $s.shown -eq 4 } 20) (SmartLine "${D}: pdf")
# 任一 of one row is 全部: saved as all.
Pf fence create --title 'Any one' --smart-folder $PfDl --ext pdf --match any --rect '40,500,420,300' | Out-Null
Check 'any of one row is filled' (WaitSmart 'Any one' { $s.shown -ge 3 } 20) (SmartLine 'Any one')
FocusFence 'Any one'
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('contract'); Start-Sleep -Milliseconds 500
[G]::Press(0x0D); Start-Sleep -Milliseconds 400
$m = FenceMenuOf 'Any one'
MenuPick '^Save filter as a smart fence' $m | Out-Null
Start-Sleep -Seconds 2
$s = SmartOf 'Any one: contract'
Check 'any of one row: saved as all with the filter' ($s -and $s.match -eq 'all' -and $s.spec.name -eq 'contract' -and (WaitSmart 'Any one: contract' { $s.shown -eq 1 -and $n -match 'old contract' } 20)) "$($s.spec | ConvertTo-Json -Compress -Depth 4) $(SmartLine 'Any one: contract')"
# 任一 of two rows: the item is off and its tip says why.
Pf fence create --title 'Any two' --smart-folder $PfDl --ext pdf --within-days 3 --match any --rect '480,500,420,300' | Out-Null
Check 'any of two rows is filled' (WaitSmart 'Any two' { $s.shown -ge 4 } 20) (SmartLine 'Any two')
FocusFence 'Any two'
Chord 0x11 0x46; Start-Sleep -Milliseconds 500
[G]::Type('pdf'); Start-Sleep -Milliseconds 500
[G]::Press(0x0D); Start-Sleep -Milliseconds 400
$before = @(@(PfJson fence list) | Where-Object { $_ -and $_.smart }).Count
$m = FenceMenuOf 'Any two'
$t = MenuTexts $m
Check 'any of two rows: Save filter as a smart fence is off' ($t -match 'Save filter as a smart fence \(off\)') $t
$p = MenuPoint $m '^Save filter as a smart fence'
if ($p) { [G]::MoveTo($p[0] - 30, $p[1]); Start-Sleep -Milliseconds 200; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 900 }
# The tip is a tooltip control beside the item: its text read by OCR.
$tips = @([W]::Tops('tooltips_class32') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' })
$tip = ''
foreach ($h in $tips) { $r = Rect $h; if ($r.R - $r.L -gt 40) { $tip += (OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) 'menu-tip') } }
Check 'its tip says why in one sentence' ($tip -match 'any one' -and $tip -match 'cannot be saved') "tips=$($tips.Count) $tip"
Shot 'r06-any-save-off'
if ($p) { [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800 }
CloseMenus
Check 'a click on it makes nothing' (@(@(PfJson fence list) | Where-Object { $_ -and $_.smart }).Count -eq $before) ''
$log = (PfLogAll) -join "`n"
Check 'log: off for any of several rows, any of one row saved as all' ($log -match 'save filter as smart fence: off for an any search of several rows.*rows=2' -and $log -match 'any of one row or none saved as all') ''

$log = (PfLogAll) -join "`n"
Check 'log: filter opened, changed (with the count shown), box closed, cleared, saved' ($log -match 'filter opened' -and $log -match 'filter changed.*shown=' -and $log -match 'filter box closed' -and $log -match 'filter saved as a smart fence') ''
PfQuit
ClearDownloads
