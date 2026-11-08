# #186 search box: every way of opening it x every key (1600x900, 100 %). Runs first in a
# round: a key that only fails after one way of opening is the kind of bug found last before.
# Each check reads the outcome (the box's text, which file opened), never only a screenshot.
Case 'setup'
$paths = @('w', 'x', 'y', 'z')
$how = @{ w = 'Ctrl+Alt+Space, wait'; x = 'Ctrl+Alt+Space, at once'; y = 'double Ctrl, wait'; z = 'double Ctrl, at once' }
$made = @()
foreach ($p in $paths) { foreach ($c in 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'k') { foreach ($n in 'one', 'two') {
  $f = "$PfDesk\q$p$c $n.txt"; Set-Content $f $n; $made += $f } } }
Set-Culture en-US
PfStart
Pf settings set tools.dialogJump true | Out-Null
Start-Process notepad.exe -WorkingDirectory C:\pf
Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 10, 40, 700, 600, $true) | Out-Null; Start-Sleep -Milliseconds 500
Check 'Notepad is there' ($np -ne [IntPtr]::Zero)

# Opens the box the given way, with Notepad in front first.
function OpenBox($p) {
  Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowTitle -like 'q*' } | Stop-Process -Force
  (New-Object -ComObject Shell.Application).Windows() | ForEach-Object { $_.Quit() }
  Start-Sleep -Milliseconds 300
  $script:np = [G]::FindWindowW('Notepad', [NullString]::Value)
  if ($script:np -eq [IntPtr]::Zero) {
    Start-Process notepad.exe -WorkingDirectory C:\pf; Start-Sleep -Seconds 2
    $script:np = [G]::FindWindowW('Notepad', [NullString]::Value)
    [L]::MoveWindow($script:np, 10, 40, 700, 600, $true) | Out-Null; Start-Sleep -Milliseconds 500
  }
  TitleClick $script:np
  OpenKeys $p
}
function OpenKeys($p) {
  switch ($p) {
    'w' { PeekKey; Start-Sleep -Milliseconds 1200 }
    'x' { PeekKey; Start-Sleep -Milliseconds 150 }
    'y' { DoubleCtrl; Start-Sleep -Milliseconds 1200 }
    'z' { DoubleCtrl; Start-Sleep -Milliseconds 150 }
  }
}
function Opened($want) { Start-Sleep -Seconds 2; $t = NotepadTitles; return @(($t -like "*$want*"), $t) }
function CloseBox() { for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 } }

foreach ($p in $paths) {
  $h = $how[$p]
  Case "opened by $h"
  # 「双击 Ctrl」 replaces the key combination (since 0.3.0, FEATURES.md 双击 Ctrl): the
  # Ctrl+Alt+Space groups run with it off, the double Ctrl groups with it on.
  $dc = if ($p -in 'y', 'z') { 'true' } else { 'false' }
  Pf settings set peek.doubleCtrl $dc | Out-Null; Start-Sleep -Milliseconds 800
  OpenBox $p; [G]::Type("q${p}a oneq"); Start-Sleep -Milliseconds 400; [G]::Press(0x08); Start-Sleep -Milliseconds 400
  $t = [M]::BoxText(); Check 'Backspace' ($t -eq "q${p}a one") "box='$t'"
  [G]::Press(0x0D); $r = Opened "q${p}a one"; Check 'Enter after Backspace' $r[0] "notepad=$($r[1])"
  CloseBox

  OpenBox $p; [G]::Type("q${p}b"); Start-Sleep -Milliseconds 700; [G]::Press(0x28); Start-Sleep -Milliseconds 300; [G]::Press(0x0D)
  $r = Opened "q${p}b two"; Check 'Down, Enter' $r[0] "notepad=$($r[1])"; CloseBox

  OpenBox $p; [G]::Type("q${p}c"); Start-Sleep -Milliseconds 700; Chord 0x11 0x4E; Start-Sleep -Milliseconds 300; [G]::Press(0x0D)
  $r = Opened "q${p}c two"; Check 'Ctrl+N, Enter' $r[0] "notepad=$($r[1])"; CloseBox

  OpenBox $p; [G]::Type("q${p}d"); Start-Sleep -Milliseconds 700; [G]::Press(0x28); Start-Sleep -Milliseconds 200; [G]::Press(0x26); Start-Sleep -Milliseconds 300; [G]::Press(0x0D)
  $r = Opened "q${p}d one"; Check 'Down, Up, Enter' $r[0] "notepad=$($r[1])"; CloseBox

  OpenBox $p; [G]::Type("q${p}e"); Start-Sleep -Milliseconds 700; Chord 0x11 0x32
  $r = Opened "q${p}e two"; Check 'Ctrl+2' $r[0] "notepad=$($r[1])"; CloseBox

  OpenBox $p; [G]::Type("q${p}f one"); Start-Sleep -Milliseconds 700; Chord 0x11 0x0D; Start-Sleep -Seconds 3
  $ex = @((New-Object -ComObject Shell.Application).Windows() | ForEach-Object { try { $_.Document.Folder.Self.Path } catch { '?' } }) -join ' | '
  Check 'Ctrl+Enter shows the file in Explorer' ($ex -like '*Desktop*') "explorer=$ex"; CloseBox

  OpenBox $p; [G]::Type("q${p}g one"); Start-Sleep -Milliseconds 700; [G]::Press(0x25); Start-Sleep -Milliseconds 150; [G]::Press(0x27); Start-Sleep -Milliseconds 150
  $mid = [G]::FindWindowW('#32768', [NullString]::Value)
  [G]::Press(0x27); Start-Sleep -Milliseconds 900
  $menu = [G]::FindWindowW('#32768', [NullString]::Value)
  Check 'Right at the end opens the action menu (not before)' ($mid -eq [IntPtr]::Zero -and $menu -ne [IntPtr]::Zero) "menu before=$mid after=$menu"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400; CloseBox

  OpenBox $p; [G]::Type('zzz'); Start-Sleep -Milliseconds 400; Chord 0x11 0x41; Start-Sleep -Milliseconds 200; [G]::Type("q${p}h one"); Start-Sleep -Milliseconds 600
  $t = [M]::BoxText(); Check 'Ctrl+A, then typing replaces' ($t -eq "q${p}h one") "box='$t'"; CloseBox

  OpenBox $p; [G]::Type("q${p}i oneq"); Start-Sleep -Milliseconds 400; [G]::Press(0x25); Start-Sleep -Milliseconds 150; [G]::Press(0x2E); Start-Sleep -Milliseconds 400
  $t = [M]::BoxText(); Check 'Left, Delete' ($t -eq "q${p}i one") "box='$t'"
  [G]::Press(0x0D); $r = Opened "q${p}i one"; Check 'Enter after Delete' $r[0] "notepad=$($r[1])"; CloseBox

  TitleClick $np; Chord 0x11 0x4F; Start-Sleep -Seconds 3
  $dlg = [G]::FindWindowW('#32770', 'Open')
  OpenKeys $p
  [G]::Type("q${p}k two"); Start-Sleep -Milliseconds 800; [G]::Press(0x0D); Start-Sleep -Seconds 3
  $t = NotepadTitles
  Check 'Enter with an Open dialog in front hands the file to it' ($t -like "*q${p}k two*") "notepad=$t dialog open=$([G]::IsWindowVisible($dlg))"
  if ([G]::IsWindowVisible($dlg)) { [G]::Press(0x1B); Start-Sleep -Milliseconds 500 }

  OpenBox $p; [G]::Type("q${p}a"); Start-Sleep -Milliseconds 500; [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  Check 'Esc closes the box' ([M]::BoxText() -eq $null) "box='$([M]::BoxText())'"
  CloseBox
  Shot "m-$p-end"
}

Case 'note editor during the Peek'
Pf settings set peek.doubleCtrl false | Out-Null; Start-Sleep -Milliseconds 800
Pf settings set tools.enabled true | Out-Null
Pf fence create --tool note --title PeekNote --text first --rect '760,500,300,160' | Out-Null
Start-Sleep -Seconds 1
TitleClick $np
PeekKey; Start-Sleep -Milliseconds 300
[G]::Click(900, 600); Start-Sleep -Milliseconds 900
[G]::Type("`nsecond"); Start-Sleep -Milliseconds 300
[G]::Press(0x08); Start-Sleep -Milliseconds 300
Shot 'm-note-editor'
[G]::Press(0x1B); Start-Sleep -Milliseconds 800
$n = PfJson fence get PeekNote
$text = if ($n.fence) { $n.fence.noteText } else { $n.noteText }
Check 'typing and Backspace go to the note editor' ($text -eq "first`nsecon") ("noteText=" + ($text -replace "`n", ' / '))
[G]::Press(0x1B); Start-Sleep -Milliseconds 500

Case 'cleanup'
PfQuit
CloseWindows
foreach ($f in $made) { Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue }
