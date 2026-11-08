# #186 search list in the Peek (no Everything): pinyin, moving, opening, the action menu, the
# Open dialog hand-off, double Ctrl. Chinese UI, as the users who asked for pinyin see it.
Case 'setup'
CloseWindows
Set-Culture zh-CN
$files = '季度报告.docx', '预算.xlsx', '会议记录.txt', 'Launch checklist.md', '发票 10 月.pdf', '测试报告一.txt', '测试报告二.txt'
foreach ($n in $files) { Set-Content "$PfDesk\$n" 'sample' }
PfStart -Lang zh-CN
Pf settings set theme light | Out-Null
function SearchFor($text) {
  DeskClick 1500 820 | Out-Null
  PeekKey; Start-Sleep -Milliseconds 800
  [G]::Type($text); Start-Sleep -Milliseconds 900
}
function EndSearch() { for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 } }

Case 'pinyin initials'
SearchFor 'hyjl'
Check 'the box takes the query' ([M]::BoxText() -eq 'hyjl') "box='$([M]::BoxText())'"
Shot 's01-hyjl'
[G]::Press(0x0D); Start-Sleep -Seconds 3
$t = NotepadTitles
Check 'hyjl + Enter opens 会议记录.txt' ($t -like '*会议记录*') "notepad=$t"
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500
EndSearch
SearchFor 'jdbg'
Shot 's02-jdbg'
Chord 0x11 0x0D; Start-Sleep -Seconds 4
$ex = Explorers
Check 'jdbg + Ctrl+Enter shows the desktop folder' ($ex -like '*Desktop*') "explorer=$ex"
CloseWindows
EndSearch

Case 'several rows'
# Text files only, so every row opens in Notepad (the box has no Office).
SearchFor 'csbg'
Shot 's03-csbg-list'
[G]::Press(0x0D); Start-Sleep -Seconds 3
$first = NotepadTitles
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500
EndSearch
SearchFor 'csbg'
Chord 0x11 0x4E; Start-Sleep -Milliseconds 400; [G]::Press(0x0D); Start-Sleep -Seconds 3
$down = NotepadTitles
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500
EndSearch
SearchFor 'csbg'
Chord 0x11 0x32; Start-Sleep -Seconds 3
$second = NotepadTitles
Check 'Enter opens a matching row' ($first -like '*测试报告*') "Enter -> '$first'"
Check 'Ctrl+N, Enter opens the second row' ($down -like '*测试报告*' -and $down -ne $first) "Ctrl+N -> '$down'"
Check 'Ctrl+2 opens the second row' ($second -eq $down) "Ctrl+2 -> '$second'"
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
EndSearch

Case 'action menu'
SearchFor 'hyjl'
[G]::Press(0x27); Start-Sleep -Milliseconds 1200
$menu = [G]::FindWindowW('#32768', [NullString]::Value)
Check 'Right with the caret at the end opens the action menu' ($menu -ne [IntPtr]::Zero) "menu=$menu"
Shot 's05-actions'
[G]::Press(0x1B); Start-Sleep -Milliseconds 500
Check 'Esc closes the menu, the box stays' ([M]::BoxText() -eq 'hyjl') "box='$([M]::BoxText())'"
EndSearch

Case 'Open dialog in front'
Start-Process notepad.exe -WorkingDirectory C:\pf
Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 10, 40, 800, 760, $true) | Out-Null
TitleClick $np
Chord 0x11 0x4F; Start-Sleep -Seconds 3
$dlg = [G]::FindWindowW('#32770', '打开')
if ($dlg -eq [IntPtr]::Zero) { $dlg = [G]::FindWindowW('#32770', 'Open') }
Check 'Notepad Open dialog' ($dlg -ne [IntPtr]::Zero) (DlgState 'open' $dlg)
PeekKey; Start-Sleep -Milliseconds 800
[G]::Type('hyjl'); Start-Sleep -Milliseconds 900
Shot 's08-dialog-search'
[G]::Press(0x0D); Start-Sleep -Seconds 3
$t = NotepadTitles
Check 'Enter hands the file to the dialog (Notepad opens it)' ($t -like '*会议记录*' -and -not [QS]::IsWindow($dlg)) "notepad=$t dialog open=$([QS]::IsWindow($dlg))"
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force

Case 'double Ctrl'
Pf settings set peek.doubleCtrl true | Out-Null
Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
TitleClick $np
$before = Fg
DoubleCtrl; Start-Sleep -Seconds 1
Check 'two Ctrl taps open the box' ([M]::BoxText() -ne $null) "fg=$(Fg)"
[G]::Type('fp'); Start-Sleep -Milliseconds 900
Check 'typing goes into the box' ([M]::BoxText() -eq 'fp') "box='$([M]::BoxText())'"
Shot 's10-double-ctrl-fp'
DoubleCtrl; Start-Sleep -Seconds 1
Check 'two more taps close it' ([M]::BoxText() -eq $null) "box='$([M]::BoxText())'"
Check 'the window before gets the foreground back' ((Fg) -eq $before) "before=$before now=$(Fg)"
Chord 0x11 0x43; Start-Sleep -Milliseconds 150; Chord 0x11 0x43; Start-Sleep -Seconds 1
Check 'Ctrl+C twice does not open it' ([M]::BoxText() -eq $null) "fg=$(Fg)"

Case 'cleanup'
PfQuit
CloseWindows
foreach ($n in $files) { Remove-Item -LiteralPath "$PfDesk\$n" -ErrorAction SilentlyContinue }
