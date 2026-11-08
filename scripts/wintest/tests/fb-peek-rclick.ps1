# Feedback #124: a right-click on a row of the Peek search list opens that row's actions at the
# pointer (打开 / 打开所在文件夹 / 复制路径, then the Windows menu), the menu that → (caret at
# the end) and Ctrl+O show under the row; the menu key and Shift+F10 show it too.
#   --var Part=h  the search opened by Ctrl+Alt+Space   } every row source (a desktop file in a
#   --var Part=d  the search opened by double Ctrl      } fence, a file in a folder fence, an
#   --var Part=t  the Peek opened from the tray menu    } Everything result) x every action
#   --var Part=k  the keys, a folder row, two rows, the spots without a row menu (run it on the
#                 old build too: the cases that changed fail there)
# Needs Everything running (box ec2-7). Chinese UI, 1600x900 at 100 %. Every check searches a
# file of its own, so ranking and history cannot hide a result; the names carry the job's time
# ($Tag), so the Recent shortcuts an earlier run left (found by Everything) never match.
if (-not $Part) { $Part = 'h' }
Case "setup ($Part)"
CloseWindows
Set-Culture zh-CN
# Everything answers through its window in pf's session (the service alone does not). Another
# test may have closed it: started here when missing, and closed again at the end.
function EvWin() {
  foreach ($c in 'EVERYTHING_TASKBAR_NOTIFICATION', 'EVERYTHING_TASKBAR_NOTIFICATION_(1.5a)') {
    $h = [G]::FindWindowW($c, [NullString]::Value); if ($h -ne [IntPtr]::Zero) { return $h } }
  return [IntPtr]::Zero
}
$evStarted = $null
if ((EvWin) -eq [IntPtr]::Zero) {
  $exe = @('C:\Program Files\Everything\Everything.exe', 'C:\Program Files\Everything 1.5a\Everything.exe') | Where-Object { Test-Path $_ } | Select-Object -First 1
  if ($exe) {
    $evStarted = Start-Process $exe -ArgumentList '-startup' -WorkingDirectory (Split-Path $exe) -PassThru
    for ($i = 0; $i -lt 60 -and (EvWin) -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 250 }
  }
}
Check 'Everything answers' ((EvWin) -ne [IntPtr]::Zero) "started here=$([bool]$evStarted)"
$ff = 'C:\pf\rkff'; $evd = 'C:\pf\rkev'
New-Item -ItemType Directory -Force $ff, $evd | Out-Null
$srcDir = @{ n = $PfDesk; f = $ff; e = $evd }
$srcName = @{ n = 'desktop fence row'; f = 'folder fence row'; e = 'Everything row' }
$made = @()
$Tag = 'x' + (Get-Date -Format 'HHmmss')
if ($Part -ne 'k') {
  foreach ($s in 'n', 'f', 'e') { for ($a = 1; $a -le 8; $a++) {
    $p = Join-Path $srcDir[$s] "rk$Part$s$a$Tag doc.txt"; Set-Content -LiteralPath $p $s; $made += $p } }
} else {
  foreach ($n in 'rkkr', 'rkko', 'rkkg', 'rkkm', 'rkks', 'rkke', 'rkkw', 'rkkx', 'rkky', 'rkkb') {
    $p = Join-Path $PfDesk "$n$Tag doc.txt"; Set-Content -LiteralPath $p 'k'; $made += $p }
  foreach ($n in "rkkpair$Tag ab", "rkkpair$Tag cd") { $p = Join-Path $PfDesk "$n.txt"; Set-Content -LiteralPath $p 'k'; $made += $p }
  $kdir = Join-Path $evd "rkkfolder$Tag"; New-Item -ItemType Directory -Force $kdir | Out-Null
}
PfStart -Lang zh-CN
Pf settings set theme light | Out-Null
Pf settings set tools.everything true | Out-Null
Pf settings set peek.doubleCtrl ($(if ($Part -eq 'd') { 'true' } else { 'false' })) | Out-Null
Pf fence create --portal $ff --title 'RK folder' --rect '300,520,300,200' | Out-Null
# Everything sees new files through the NTFS journal within a second or two.
Start-Sleep -Seconds 4
Shot "k00-desktop-$Part"

function PeekOn() { return (@([W]::Tops('PecoFence.PeekDimmer')).Count -gt 0) }
function BoxRect() { $b = [G]::FindWindowW('PecoFence.PeekSearch', [NullString]::Value); if ($b -eq [IntPtr]::Zero) { return $null }; return (Rect $b) }
function ClipText() { return [Windows.Forms.Clipboard]::GetText() }
function EndSearch() { CloseMenus; for ($i = 0; $i -lt 3 -and ([M]::BoxText() -ne $null -or (PeekOn)); $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 } }
# A Properties window of $name: found, visible, and on top at its middle (not under the Peek).
function PropsWin($name) {
  $h = @([W]::Tops('#32770') | Where-Object { [W]::Title($_) -like "$name*Properties" }) | Select-Object -First 1
  if (-not $h) { return @($false, 'none') }
  $r = Rect $h; $cx = [int](($r.L + $r.R) / 2); $cy = [int](($r.T + $r.B) / 2)
  $top = [W]::RootClassAt($cx, $cy); $title = [W]::Title($h)
  [L]::PostMessageW($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 600
  return @(($top -eq '#32770'), "title='$title' class at its middle=$top")
}
function MenuAt($m) { if ($m -eq [IntPtr]::Zero) { return 'none' }; $r = Rect $m; return "$($r.L),$($r.T)" }
# Opens the search the way this part tests and types $q; $script:box is its rectangle.
function OpenSearch($q) {
  CloseMenus
  [G]::Click(900, 800); Start-Sleep -Milliseconds 400
  switch ($Part) {
    'd' { DoubleCtrl; Start-Sleep -Milliseconds 1000 }
    't' { TrayPick '^速览所有栅栏' | Out-Null; Start-Sleep -Milliseconds 900 }
    default { PeekKey; Start-Sleep -Milliseconds 800 }
  }
  [G]::Type($q); Start-Sleep -Milliseconds 1600
  $script:box = BoxRect
  return ([M]::BoxText() -eq $q)
}
# Row i's middle (one result row and the show-all row: no group headings).
function RowPt($i) { return @(($script:box.L + 200), ($script:box.T + 82 + 44 * $i)) }
function RightClickRow($i) {
  $p = RowPt $i
  [G]::RClick($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $m = MenuWin 3
  $script:near = ($m -ne [IntPtr]::Zero) -and ([Math]::Abs((Rect $m).L - $p[0]) -le 12) -and ([Math]::Abs((Rect $m).T - $p[1]) -le 12)
  return $m
}

# One check of the matrix: the file of source $s for action $a, its row right-clicked.
function MatrixCase($s, $a) {
  trap { PfTrap $_; continue }
  $name = "rk$Part$s$a$Tag"; $path = Join-Path $srcDir[$s] "$name doc.txt"; $label = $srcName[$s]
  [Windows.Forms.Clipboard]::Clear()
  $opened = OpenSearch $name
  if ($a -eq 8) {
    # The keys still work in this way of opening: Ctrl+O, 复制路径.
    Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
    $m = MenuWin 3; $items = MenuTexts $m
    $t = MenuPick '^复制路径' $m; Start-Sleep -Milliseconds 600
    $c = ClipText
    Check "$label | Ctrl+O, 复制路径 (keys as before)" ($c -eq $path -and [M]::BoxText() -eq $name) "typed=$opened clip='$c' box='$([M]::BoxText())' items=$items"
    EndSearch; return
  }
  $m = RightClickRow 0
  $items = MenuTexts $m
  $menuOk = ($m -ne [IntPtr]::Zero) -and ($items -match '复制路径') -and ($items -match 'Properties') -and $script:near
  $why = "typed=$opened menu at=$(MenuAt $m) near pointer=$script:near items=$items"
  switch ($a) {
    1 {
      MenuPick '^打开$' $m | Out-Null; Start-Sleep -Seconds 2
      $np = NotepadTitles
      Check "$label | right-click, 打开" ($menuOk -and $np -like "*$name*" -and [M]::BoxText() -eq $null -and -not (PeekOn)) "notepad='$np' box='$([M]::BoxText())' peek=$(PeekOn) $why"
    }
    2 {
      MenuPick '^打开所在文件夹' $m | Out-Null; Start-Sleep -Seconds 3
      $ex = Explorers
      Check "$label | right-click, 打开所在文件夹" ($menuOk -and $ex -like "*path=$($srcDir[$s])*" -and [M]::BoxText() -eq $null) "explorer=$($ex -replace "`n", ' / ') box='$([M]::BoxText())' $why"
    }
    3 {
      MenuPick '^复制路径' $m | Out-Null; Start-Sleep -Milliseconds 700
      $c = ClipText
      Check "$label | right-click, 复制路径 (search stays)" ($menuOk -and $c -eq $path -and [M]::BoxText() -eq $name -and (PeekOn)) "clip='$c' box='$([M]::BoxText())' peek=$(PeekOn) $why"
    }
    4 {
      MenuPick '^Copy$' $m | Out-Null; Start-Sleep -Milliseconds 900
      $files = @(ClipFiles)
      Check "$label | right-click, Windows Copy (file on the clipboard, Peek ends)" ($menuOk -and $files -contains $path -and [M]::BoxText() -eq $null -and -not (PeekOn)) "clipboard files=$($files -join ';') box='$([M]::BoxText())' peek=$(PeekOn) $why"
    }
    5 {
      [G]::Press(0x1B); Start-Sleep -Milliseconds 500
      $gone = (MenuWin 1) -eq [IntPtr]::Zero
      [G]::Type('q'); Start-Sleep -Milliseconds 500
      $t = [M]::BoxText()
      Check "$label | right-click, Esc (menu closes, typing goes on)" ($menuOk -and $gone -and $t -eq "${name}q" -and (PeekOn)) "menu gone=$gone box='$t' peek=$(PeekOn) $why"
    }
    6 {
      [G]::Click(150, 780); Start-Sleep -Milliseconds 1500
      $gone = (MenuWin 1) -eq [IntPtr]::Zero
      Check "$label | right-click, click elsewhere (menu closes, Peek ends as after Ctrl+O)" ($menuOk -and $gone -and [M]::BoxText() -eq $null -and -not (PeekOn)) "menu gone=$gone box='$([M]::BoxText())' peek=$(PeekOn) $why"
    }
    7 {
      MenuPick '^Properties$' $m | Out-Null; Start-Sleep -Seconds 3
      $peek = PeekOn; $bt = [M]::BoxText()
      Shot "k-$Part-$s-properties"
      $pw = PropsWin $name
      Check "$label | right-click, Windows Properties (in front, Peek ends)" ($menuOk -and $pw[0] -and $bt -eq $null -and -not $peek) "$($pw[1]) box='$bt' peek=$peek $why"
    }
  }
  EndSearch
  CloseWindows
}

if ($Part -ne 'k') {
  foreach ($s in 'n', 'f', 'e') {
    Case "$($srcName[$s])s, opened by $Part"
    for ($a = 1; $a -le 8; $a++) { MatrixCase $s $a }
    Shot "k-$Part-$s-end"
  }
  Case 'log lines'
  $shown = @(PfLogAll | Where-Object { $_ -match 'search actions menu shown' -and $_ -match 'by="?right-click' })
  $kinds = (@($shown | ForEach-Object { if ($_ -match 'what="?([^"=]+?)"? windows_menu') { $Matches[1] } }) | Sort-Object -Unique) -join '; '
  Check 'the log names the row kind of every right-click' ($kinds -match '(^|; )fence, file' -and $kinds -match 'folder fence, file' -and $kinds -match 'Everything, file') "kinds=$kinds lines=$($shown.Count)"
  $closed = @(PfLogAll | Where-Object { $_ -match 'search actions menu closed' -and $_ -match 'by="?right-click' })
  $outs = (@($closed | ForEach-Object { if ($_ -match 'outcome="?([^"=]+?)"? peek') { $Matches[1] } }) | Sort-Object -Unique) -join '; '
  Check 'the log says what each menu did' ($outs -match 'open;' -and $outs -match 'copy path' -and $outs -match 'closed without a choice' -and $outs -match 'open containing folder') "outcomes=$outs"
}

if ($Part -eq 'k') {
  Case 'keys as before'
  $p = Join-Path $PfDesk "rkkr$Tag doc.txt"; [Windows.Forms.Clipboard]::Clear()
  OpenSearch "rkkr$Tag" | Out-Null
  [G]::Press(0x27); Start-Sleep -Milliseconds 900
  $m = MenuWin 3; $at = MenuAt $m
  $under = ($m -ne [IntPtr]::Zero) -and ([Math]::Abs((Rect $m).L - ($box.L + 16)) -le 12) -and ([Math]::Abs((Rect $m).T - ($box.T + 104)) -le 12)
  MenuPick '^复制路径' $m | Out-Null; Start-Sleep -Milliseconds 600
  Check '→ at the end: the menu under the row, 复制路径' ($under -and (ClipText) -eq $p) "menu at=$at box=$($box.L),$($box.T) clip='$(ClipText)'"
  EndSearch

  $p = Join-Path $PfDesk "rkko$Tag doc.txt"; [Windows.Forms.Clipboard]::Clear()
  OpenSearch "rkko$Tag" | Out-Null
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
  MenuPick '^Copy$' | Out-Null; Start-Sleep -Milliseconds 900
  $files = @(ClipFiles)
  Check 'Ctrl+O, Windows Copy' ($files -contains $p) "clipboard files=$($files -join ';')"
  EndSearch

  OpenSearch "rkkg$Tag" | Out-Null
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
  MenuPick '^Properties$' | Out-Null; Start-Sleep -Seconds 3
  $peek = PeekOn; $bt = [M]::BoxText()
  Shot 'k00-ctrlo-properties'
  $pw = PropsWin "rkkg$Tag"
  Check 'Ctrl+O, Windows Properties: its window is in front, the Peek ends' ($pw[0] -and $bt -eq $null -and -not $peek) "$($pw[1]) box='$bt' peek=$peek"
  EndSearch

  Case 'menu key and Shift+F10 on a row'
  $p = Join-Path $PfDesk "rkkm$Tag doc.txt"; [Windows.Forms.Clipboard]::Clear()
  OpenSearch "rkkm$Tag" | Out-Null
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  $m = MenuWin 3; $items = MenuTexts $m; $at = MenuAt $m
  $under = ($m -ne [IntPtr]::Zero) -and ([Math]::Abs((Rect $m).L - ($box.L + 16)) -le 12)
  Shot 'k01-menu-key'
  MenuPick '^复制路径' $m | Out-Null; Start-Sleep -Milliseconds 600
  Check 'the menu key shows the row menu under the row' ($items -match '复制路径' -and $under -and (ClipText) -eq $p) "menu at=$at items=$items clip='$(ClipText)'"
  EndSearch

  $p = Join-Path $PfDesk "rkks$Tag doc.txt"; [Windows.Forms.Clipboard]::Clear()
  OpenSearch "rkks$Tag" | Out-Null
  Chord 0x10 0x79; Start-Sleep -Milliseconds 900
  $m = MenuWin 3; $items = MenuTexts $m
  MenuPick '^Copy$' $m | Out-Null; Start-Sleep -Milliseconds 900
  $files = @(ClipFiles)
  Check 'Shift+F10 shows the row menu (Windows Copy)' ($items -match '复制路径' -and $files -contains $p) "items=$items clipboard files=$($files -join ';')"
  EndSearch

  Case 'text box menu'
  OpenSearch 'rkkzzq9' | Out-Null
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  $m = MenuWin 3; $items = MenuTexts $m
  Shot 'k02-menu-key-no-row'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  Check 'the menu key with no row: the text box menu' ($items -match 'Select All' -and $items -notmatch '复制路径' -and [M]::BoxText() -eq 'rkkzzq9') "items=$items box='$([M]::BoxText())'"
  EndSearch

  OpenSearch "rkke$Tag" | Out-Null
  [G]::RClick($box.L + 90, $box.T + 28); Start-Sleep -Milliseconds 900
  $m = MenuWin 3; $items = MenuTexts $m
  Shot 'k03-text-rclick'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  Check 'a right-click on the typed text: the text box menu' ($items -match 'Select All' -and $items -notmatch '复制路径' -and [M]::BoxText() -eq "rkke$Tag") "items=$items box='$([M]::BoxText())'"

  Case 'spots without a row menu'
  [G]::RClick($box.R - 60, $box.T + 28); Start-Sleep -Milliseconds 900
  $m = MenuWin 1
  Check 'a right-click on the box around the text: no menu, the search stays' ($m -eq [IntPtr]::Zero -and [M]::BoxText() -eq "rkke$Tag") "menu=$(MenuTexts $m) box='$([M]::BoxText())'"
  CloseMenus
  # The show-all row is the last one (no group headings: 44 px rows under the 56 px box).
  $rows = [int](($box.B - $box.T - 68) / 44)
  $q = RowPt ($rows - 1)
  [G]::RClick($q[0], $q[1]); Start-Sleep -Milliseconds 900
  $m = MenuWin 1
  $line = @(PfLogAll | Where-Object { $_ -match 'the row has no file path' -and $_ -match 'show all in Everything' }).Count
  Check 'a right-click on 在 Everything 里查看全部结果: no menu, the search stays' ($m -eq [IntPtr]::Zero -and [M]::BoxText() -eq "rkke$Tag" -and $line -gt 0) "menu=$(MenuTexts $m) box='$([M]::BoxText())' log lines=$line rows=$rows"
  CloseMenus
  EndSearch

  Case 'a folder row'
  OpenSearch "rkkfolder$Tag" | Out-Null
  $m = RightClickRow 0; $items = MenuTexts $m
  Shot 'k04-folder-row'
  MenuPick '^打开$' $m | Out-Null; Start-Sleep -Seconds 3
  $ex = Explorers
  Check 'right-click on an Everything folder, 打开: Explorer shows it' ($items -match '复制路径' -and $ex -like "*path=$kdir*") "items=$items explorer=$($ex -replace "`n", ' / ')"
  CloseWindows
  EndSearch

  Case 'two rows'
  [Windows.Forms.Clipboard]::Clear()
  OpenSearch "rkkpair$Tag" | Out-Null
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
  MenuPick '^复制路径' | Out-Null; Start-Sleep -Milliseconds 600
  $first = ClipText
  $m = RightClickRow 1
  Shot 'k05-second-row'
  MenuPick '^复制路径' $m | Out-Null; Start-Sleep -Milliseconds 600
  $second = ClipText
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
  MenuPick '^复制路径' | Out-Null; Start-Sleep -Milliseconds 600
  $again = ClipText
  $both = ($first -like "*rkkpair$Tag ab.txt" -or $first -like "*rkkpair$Tag cd.txt") -and ($second -like "*rkkpair$Tag ab.txt" -or $second -like "*rkkpair$Tag cd.txt")
  Check 'a right-click on the second row acts on that row' ($both -and $first -ne $second) "first row='$first' right-clicked second='$second'"
  Check 'the right-clicked row stays highlighted (Ctrl+O acts on it next)' ($again -eq $second -and $first -ne $second) "Ctrl+O after it='$again'"
  EndSearch

  Case 'click elsewhere'
  OpenSearch "rkkw$Tag" | Out-Null
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
  $m = MenuWin 3
  [G]::Click(150, 780); Start-Sleep -Milliseconds 1500
  Check 'Ctrl+O, click on the empty desktop: the menu closes and the Peek ends' ($m -ne [IntPtr]::Zero -and (MenuWin 1) -eq [IntPtr]::Zero -and [M]::BoxText() -eq $null -and -not (PeekOn)) "box='$([M]::BoxText())' peek=$(PeekOn)"
  EndSearch
  OpenSearch "rkkx$Tag" | Out-Null
  Chord 0x11 0x4F; Start-Sleep -Milliseconds 900
  $m = MenuWin 3
  [G]::Click($box.L + 300, $box.T + 28); Start-Sleep -Milliseconds 900
  [G]::Type('q'); Start-Sleep -Milliseconds 500
  $t = [M]::BoxText()
  Check 'Ctrl+O, click on the typed text: the menu closes, the search stays' ($m -ne [IntPtr]::Zero -and (MenuWin 1) -eq [IntPtr]::Zero -and $t -like 'rkkx*q' -and (PeekOn)) "box='$t' peek=$(PeekOn)"
  EndSearch
  OpenSearch "rkky$Tag" | Out-Null
  $m = RightClickRow 0
  [G]::Click($box.L + 300, $box.T + 28); Start-Sleep -Milliseconds 900
  [G]::Type('q'); Start-Sleep -Milliseconds 500
  $t = [M]::BoxText()
  Check 'right-click, click on the typed text: the menu closes, the search stays' ($m -ne [IntPtr]::Zero -and (MenuWin 1) -eq [IntPtr]::Zero -and $t -like 'rkky*q' -and (PeekOn)) "box='$t' peek=$(PeekOn)"
  EndSearch

  Case 'right-click on a row'
  OpenSearch "rkkb$Tag" | Out-Null
  $m = RightClickRow 0
  Shot 'k06-rclick-row'
  Check 'a right-click on a row opens its menu at the pointer' ($m -ne [IntPtr]::Zero -and $script:near -and (MenuTexts $m) -match '复制路径') "menu at=$(MenuAt $m) items=$(MenuTexts $m)"
  EndSearch
}

Case 'cleanup'
PfQuit
CloseWindows
foreach ($f in $made) { Remove-Item -LiteralPath $f -ErrorAction SilentlyContinue }
Remove-Item -LiteralPath $ff -Recurse -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $evd -Recurse -ErrorAction SilentlyContinue
if ($evStarted) { Stop-Process -Id $evStarted.Id -ErrorAction SilentlyContinue }
