# #186 对话框跳转: an Open dialog keeps its own folder, jumps once to Explorer's folder when the
# user comes back from Explorer, Ctrl+G jumps any time, Save As keeps the typed name, the folder
# picker jumps and stays open, pressing a folder portal fence jumps at once, switched off nothing
# jumps. Each check reads the dialog's address bar.
Case 'setup'
CloseWindows
Get-CimInstance Win32_Process | Where-Object { $_.Name -eq 'powershell.exe' -and $_.CommandLine -like '*picker.ps1*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
$a = 'C:\pf\run\demo\项目资料'; $b = 'C:\pf\run\demo\其他'
New-Item -ItemType Directory -Force $a, $b | Out-Null
foreach ($f in '测试报告.docx', '预算表.xlsx', 'readme.txt') { Set-Content -Path (Join-Path $a $f) -Value 'x' }
Set-Culture en-US
PfStart
Pf settings set tools.dialogJump true | Out-Null
function AltD($path) { [G]::KeyDown(0x12); [G]::Press(0x44); [G]::KeyUp(0x12); Start-Sleep -Milliseconds 500; [G]::Type("$path`n"); Start-Sleep -Seconds 2 }
function OpenExplorer($path) {
  Start-Process explorer.exe "`"$path`""; Start-Sleep -Seconds 4
  $h = [G]::FindWindowW('CabinetWClass', [NullString]::Value)
  [L]::MoveWindow($h, 820, 60, 760, 640, $true) | Out-Null; Start-Sleep -Milliseconds 500
  return $h
}
function OpenNotepad() {
  Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
  $h = [G]::FindWindowW('Notepad', [NullString]::Value)
  [L]::MoveWindow($h, 10, 40, 800, 760, $true) | Out-Null; Start-Sleep -Milliseconds 500
  return $h
}

Case 'Open dialog'
$ex = OpenExplorer $a
TitleClick $ex
$np = OpenNotepad
TitleClick $np
Chord 0x11 0x4F; Start-Sleep -Seconds 3
$dlg = [G]::FindWindowW('#32770', 'Open')
$dr = Rect $dlg
$addr = [QS]::Address($dlg)
Check 'the dialog opens in its own folder, not Explorer''s' ($dlg -ne [IntPtr]::Zero -and $addr -notlike '*项目资料') (DlgState 'open' $dlg)

Case 'back from Explorer'
TitleClick $ex
AltD $b
[G]::Click($dr.L + 120, $dr.T + 14)
$t0 = Get-Date
$got = WaitAddr $dlg '其他'
Check 'jumps to Explorer''s folder' ($got -like '*其他') "address=$got after $([int]((Get-Date) - $t0).TotalMilliseconds) ms"
Start-Sleep -Milliseconds 500
[G]::Type('abc'); Start-Sleep -Milliseconds 300
Check 'typing after the jump goes to the file name box' ([QS]::FileName($dlg) -eq 'abc') "filename='$([QS]::FileName($dlg))'"
[QS]::SetFileName($dlg, '')
Shot 'd02-back-jumped'

Case 'Ctrl+G'
[QS]::SetFileName($dlg, 'C:\Windows'); [L]::PostMessageW($dlg, 0x111, [IntPtr]1, [IntPtr]0) | Out-Null
$away = WaitAddr $dlg 'Windows'
[QS]::SetFileName($dlg, '')
Chord 0x11 0x47
$got = WaitAddr $dlg '其他'
Check 'Ctrl+G jumps back to Explorer''s folder' ($away -like '*Windows' -and $got -like '*其他') "away=$away after Ctrl+G=$got"

Case 'only once per visit'
[G]::Click($dr.L + 120, $dr.T + 14); Start-Sleep -Milliseconds 300
[QS]::SetFileName($dlg, 'C:\Windows'); [L]::PostMessageW($dlg, 0x111, [IntPtr]1, [IntPtr]0) | Out-Null
$away = WaitAddr $dlg 'Windows'; [QS]::SetFileName($dlg, '')
[G]::Click($dr.L + 200, $dr.T + 14); Start-Sleep -Seconds 2
$still = [QS]::Address($dlg)
Check 'clicking the dialog again without Explorer in between does not jump' ($still -like '*Windows') "address=$still"
TitleClick $ex; AltD $a
[G]::Click($dr.L + 120, $dr.T + 14)
$got = WaitAddr $dlg '项目资料'
Check 'after Explorer again it jumps again' ($got -like '*项目资料') "address=$got"
[L]::PostMessageW($dlg, 0x111, [IntPtr]2, [IntPtr]0) | Out-Null; Start-Sleep -Seconds 1

Case 'Save As'
TitleClick $np
Chord 0x11 0x53; Start-Sleep -Seconds 3
$sv = [G]::FindWindowW('#32770', 'Save As')
[QS]::SetFileName($sv, '会议记录.txt')
$sr = Rect $sv
TitleClick $ex; AltD $b
[G]::Click($sr.L + 120, $sr.T + 14)
$got = WaitAddr $sv '其他'
Check 'Save As jumps' ($got -like '*其他') "address=$got"
Check 'Save As keeps the typed name' ([QS]::FileName($sv) -eq '会议记录.txt') "filename='$([QS]::FileName($sv))'"
Shot 'd04-save-jumped'
[G]::Press(0x1B); Start-Sleep -Seconds 1
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force

Case 'folder picker'
Remove-Item C:\pf\run\picker.out -ErrorAction SilentlyContinue
Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-WindowStyle', 'Hidden', '-File', 'C:\pf\kit\assets\picker.ps1'
Start-Sleep -Seconds 5
$pk = [G]::FindWindowW('#32770', 'Select Folder')
$pr = Rect $pk
Check 'the folder picker is open' ($pk -ne [IntPtr]::Zero) (DlgState 'picker' $pk)
[G]::Click($pr.L + 120, $pr.T + 14); Start-Sleep -Seconds 1
TitleClick $ex; AltD $a
[G]::Click($pr.L + 120, $pr.T + 14)
$got = WaitAddr $pk '项目资料'
Check 'the picker jumps' ($got -like '*项目资料') "address=$got"
Check 'the picker stays open' ([QS]::IsWindowVisible($pk) -and -not (Test-Path C:\pf\run\picker.out)) "result=$(Get-Content C:\pf\run\picker.out -ErrorAction SilentlyContinue)"

Case 'folder portal fence'
Pf fence create --portal $b --rect '900,720,300,120' | Out-Null
Start-Sleep -Seconds 2
CloseWindows
[G]::Click($pr.L + 120, $pr.T + 14); Start-Sleep -Seconds 1
[G]::Click(1000, 735)
$got = WaitAddr $pk '其他'
Check 'pressing a folder portal fence jumps the dialog at once' ($got -like '*其他') "address=$got fg=$(Fg)"
Shot 'd06-fence-jumped'
[G]::Click($pr.L + 120, $pr.T + 14); Start-Sleep -Milliseconds 500
[G]::Press(0x1B); Start-Sleep -Milliseconds 500; [G]::Press(0x1B); Start-Sleep -Seconds 1
Get-CimInstance Win32_Process | Where-Object { $_.Name -eq 'powershell.exe' -and $_.CommandLine -like '*picker.ps1*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }

Case 'switched off'
Pf settings set tools.dialogJump false | Out-Null
$ex = OpenExplorer $a
$np = OpenNotepad
TitleClick $np; Chord 0x11 0x4F; Start-Sleep -Seconds 3
$dlg = [G]::FindWindowW('#32770', 'Open'); $dr = Rect $dlg
[QS]::SetFileName($dlg, 'C:\Windows'); [L]::PostMessageW($dlg, 0x111, [IntPtr]1, [IntPtr]0) | Out-Null
Start-Sleep -Seconds 2; [QS]::SetFileName($dlg, '')
TitleClick $ex; Start-Sleep -Seconds 1
[G]::Click($dr.L + 120, $dr.T + 14); Start-Sleep -Seconds 3
Chord 0x11 0x47; Start-Sleep -Seconds 2
$still = [QS]::Address($dlg)
Check 'switched off: neither coming back nor Ctrl+G jumps' ($still -like '*Windows') "address=$still"
[G]::Press(0x1B); Start-Sleep -Seconds 1

Case 'cleanup'
PfQuit
CloseWindows
