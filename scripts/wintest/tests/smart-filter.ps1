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
$w = DumpOf $P
Check 'while the box is open the title is the plain name' ($w.title -eq $P) $w.title
[G]::Press(0x0D); Start-Sleep -Milliseconds 500
Check 'Enter closes the box and keeps the filter' (-not (FilterBoxOpen) -and (FilterOf $P).text -eq 'pdf') ((FilterOf $P) | ConvertTo-Json -Compress)
$w = DumpOf $P
Check 'the title says what it is filtered by' ($w.title -eq 'Downloads · Filter: pdf' -and $w.raw -match ' filtered=true') $w.title
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
Check 'a smart fence of the folder and the filter' ($spec -and $spec.folder -eq $PfDl -and $spec.name -eq 'pdf' -and -not $spec.subfolders) ($spec | ConvertTo-Json -Compress)
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
Check 'the count after the name is what the filter shows' ($w.raw -match 'title_count=Some\(" · 4"\)') ([regex]::Match($w.raw, 'title_count=\S+ \S*').Value)
Shot 'r05-smart-filtered'
$log = (PfLogAll) -join "`n"
Check 'log: filter opened, changed (with the count shown), box closed, cleared, saved' ($log -match 'filter opened' -and $log -match 'filter changed.*shown=' -and $log -match 'filter box closed' -and $log -match 'filter saved as a smart fence') ''
PfQuit
ClearDownloads
