# #186 edge cases: Ctrl+G when the dialog is already there, a folder whose name has %OS% and #,
# double Ctrl keys over Notepad, a file deleted while its row shows.
Case 'setup'
CloseWindows
$base = 'C:\pf\run\edge'
$plain = "$base\plain"
$pct = "$base\pct %OS% and #hash"
New-Item -ItemType Directory -Force $plain | Out-Null
[IO.Directory]::CreateDirectory($pct) | Out-Null
$ga = "$PfDesk\gone-test a.txt"; $gb = "$PfDesk\gone-test b.txt"
Set-Content $ga 'a'; Set-Content $gb 'b'
Set-Culture en-US
PfStart
Pf settings set tools.dialogJump true | Out-Null
Pf settings set peek.doubleCtrl true | Out-Null
Start-Process explorer.exe $plain
Start-Sleep -Seconds 4
$ex = [G]::FindWindowW('CabinetWClass', [NullString]::Value)
[L]::MoveWindow($ex, 820, 60, 760, 640, $true) | Out-Null; Start-Sleep -Milliseconds 500
TitleClick $ex
Start-Process notepad.exe -WorkingDirectory C:\pf
Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 10, 40, 800, 760, $true) | Out-Null; Start-Sleep -Milliseconds 500
TitleClick $np
Chord 0x11 0x4F; Start-Sleep -Seconds 3
$dlg = [G]::FindWindowW('#32770', 'Open')
$dr = Rect $dlg

Case 'jump, then Ctrl+G at the same folder'
TitleClick $ex
Start-Sleep -Seconds 1
[G]::Click($dr.L + 120, $dr.T + 14)
$got = WaitAddr $dlg 'plain'
Check 'back from Explorer jumps to plain' ($got -like '*\plain') "address=$got"
Start-Sleep -Milliseconds 500
$t0 = Get-Date
Chord 0x11 0x47
Start-Sleep -Milliseconds 400
$addr = [QS]::Address($dlg)
Check 'Ctrl+G at the folder it already shows changes nothing' ($addr -like '*\plain') "address=$addr after $([int]((Get-Date) - $t0).TotalMilliseconds) ms"

Case 'folder name with %OS% and #'
Start-Process explorer.exe "`"$pct`""
Start-Sleep -Seconds 4
$ex2 = $null
foreach ($w in @((New-Object -ComObject Shell.Application).Windows())) { $p = try { $w.Document.Folder.Self.Path } catch { '' }; if ($p -like '*pct*') { $ex2 = [IntPtr]$w.HWND } }
Check 'Explorer shows the %OS% folder' ($ex2 -ne $null) (Explorers)
if ($ex2) { [L]::MoveWindow($ex2, 840, 120, 740, 600, $true) | Out-Null; Start-Sleep -Milliseconds 400; TitleClick $ex2 }
Start-Sleep -Seconds 1
[G]::Click($dr.L + 120, $dr.T + 14)
Start-Sleep -Seconds 3
$tops = [L]::Tops('notepad')
$boxes = ([regex]::Matches($tops, '#32770')).Count
Check 'no error box from Notepad after the %OS% folder' ($boxes -eq 1) "dialogs=$boxes fg=$(Fg)"
Shot 'e03-percent-folder'
[G]::Press(0x1B); Start-Sleep -Milliseconds 600
(New-Object -ComObject Shell.Application).Windows() | ForEach-Object { $_.Quit() }
Start-Sleep -Seconds 1

Case 'double Ctrl over Notepad'
TitleClick $np
Start-Sleep -Milliseconds 500
DoubleCtrl
Start-Sleep -Milliseconds 1200
[G]::Type('gone-testx'); Start-Sleep -Milliseconds 600
[G]::Press(0x08); Start-Sleep -Milliseconds 800
Check 'Backspace reaches the box' ([M]::BoxText() -eq 'gone-test') "box='$([M]::BoxText())'"
[G]::Press(0x28); Start-Sleep -Milliseconds 400
[G]::Press(0x0D); Start-Sleep -Seconds 3
$titles = NotepadTitles
Check 'Down and Enter open the second row (gone-test b)' ($titles -like '*gone-test b*') "notepad=$titles"
Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like '*gone-test*' } | Stop-Process -Force
Start-Sleep -Seconds 1

Case 'file deleted while its row shows'
TitleClick $np
DoubleCtrl
Start-Sleep -Milliseconds 1200
[G]::Type('gone-test a'); Start-Sleep -Milliseconds 1000
Remove-Item $ga
Start-Sleep -Milliseconds 300
$t0 = Get-Date
[G]::Press(0x0D); Start-Sleep -Seconds 2
$tops = [L]::Tops('pecofence')
$boxes = ([regex]::Matches($tops, '#32770')).Count
Check 'Enter on the deleted file shows no Windows error box' ($boxes -eq 0) "boxes=$boxes ms=$([int]((Get-Date) - $t0).TotalMilliseconds)"
Check 'PecoFence still runs' (PfAlive)
[G]::Press(0x1B); Start-Sleep -Milliseconds 400; [G]::Press(0x1B); Start-Sleep -Milliseconds 600

Case 'cleanup'
PfQuit
Remove-Item $ga, $gb -ErrorAction SilentlyContinue
CloseWindows
