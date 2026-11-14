# Owner round 2026-11-12, fourth batch, package ux4-fixes: what a reader of the evening's and
# the night's merged code found (it was never released). Parts (run.sh --var Part=...):
#   N  a note that is being edited: Ctrl+Alt+Z, which is what Windows reports for AltGr+Z,
#      takes nothing back
#   H  hover switching of tabs across a tab with nothing in it: the pages stand at once (the
#      icons faded out and in one by one)
#   P  during a Peek F6 gave the keyboard to a card: a click on a fence gives it back to the
#      Peek; a card answered by keys loses its focus ring at once
#   T  a first start: with the undo card up beside a tip, three files of one kind dragged into
#      a fence leave the tip where it is (it left for an offer that could not show)
#   C  a copy of our own that takes long, and the user goes to another program: the selection
#      takes the colour of a fence without the keyboard (about a minute: a 6 GB file)
# Runs on a 1600x900 box at 100 %.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'NHPT' }
trap { PfTrap $_; continue }

$OfDocs = @('季度销售报告.pdf', '合同 2026.pdf', '发票.pdf', '说明.txt', '会议记录 11-12.txt', '全家福.png', '截图 2026-11-12.png')
function OfClean() {
  foreach ($n in $OfDocs) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
  Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^of-big' } | Remove-Item -Force -ErrorAction SilentlyContinue
}
function OfCard() { foreach ($h in @([W]::Tops('PecoFence.Notice'))) { if ([G]::IsWindowVisible($h)) { return $h } }; return $null }
function OfCards() { return @(@([W]::Tops('PecoFence.Notice')) | Where-Object { [G]::IsWindowVisible($_) } | ForEach-Object { [L]::Text($_) }) }
# The lines the app has logged since $stamp (UTC, yyyy-MM-ddTHH:mm:ss).
function OfLogSince($stamp) { return @(PfLogAll | Where-Object { $_.Length -gt 19 -and [string]::CompareOrdinal($_.Substring(0, 19), $stamp) -ge 0 }) }
function OfNow() { return (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ss') }
# The middle of tab $i of the strip of dump $d.
function OfTabPt($d, $i) { return @([int]($d.l + $d.ox + $d.rects[$i][0] + $d.rects[$i][1] / 2), [int]($d.t + $d.oy + 18)) }

Case 'setup'
CloseWindows
OfClean

# ---- N: AltGr in a note
function OfNote() {
  trap { PfTrap $_; continue }
  Case 'N1 Ctrl+Alt+Z (AltGr+Z) in a note that is being edited'
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf settings set tools.enabled true | Out-Null
  Pf fence create --tool note --title '便签' --text 'milk' --rect '600,120,320,200' | Out-Null
  Start-Sleep -Seconds 2
  # A click behind the word opens the editor with the caret at the end of the line.
  [G]::Click(840, 172); Start-Sleep -Milliseconds 900
  [G]::Press(0x23); Start-Sleep -Milliseconds 200
  [G]::Type(' eggs'); Start-Sleep -Milliseconds 500
  # What Windows sends for AltGr+Z: Ctrl and Alt down, then Z.
  [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x5A); [G]::KeyUp(0x12); [G]::KeyUp(0x11)
  Start-Sleep -Milliseconds 500
  Shot 'n1-after-altgr-z'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  $text = "$((FenceGet '便签').noteText)"
  Check 'N1 Ctrl+Alt+Z leaves the word that was just typed' ($text -match 'eggs') "note text: [$text]"
  # Ctrl+Z alone still takes it back.
  [G]::Click(840, 172); Start-Sleep -Milliseconds 900
  [G]::Press(0x23); Start-Sleep -Milliseconds 200
  [G]::Type(' bread'); Start-Sleep -Milliseconds 500
  Chord 0x11 0x5A; Start-Sleep -Milliseconds 500
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  $text2 = "$((FenceGet '便签').noteText)"
  Check 'N1 Ctrl+Z still takes the last word back' ($text2 -match 'eggs' -and $text2 -notmatch 'bread') "note text: [$text2]"
  PfQuit
}
if ($Part -match 'N') { OfNote }

# ---- H: hover switching across a tab with nothing in it
function OfHover() {
  trap { PfTrap $_; continue }
  Case 'H4 hover switching across a tab with nothing in it'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  foreach ($n in $OfDocs) { DeskFile $n | Out-Null }
  PfStart -Lang zh-CN -Script (DumpScript 250 6)
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '文档' --rect '200,120,420,240' | Out-Null
  Pf item move --glob '*.pdf' --to '文档' | Out-Null
  Pf fence create --title '空的' --rect '700,120,340,200' | Out-Null
  Pf fence create --title '图片' --rect '1100,120,340,240' | Out-Null
  Pf item move --glob '*.png' --to '图片' | Out-Null
  Pf fence merge '空的' --into '文档' | Out-Null
  Pf fence merge '图片' --into '文档' | Out-Null
  Pf settings set rollUp.switchTabOnHover true | Out-Null
  Start-Sleep -Seconds 2
  $d = DumpOf '文档' 4
  $iDoc = [array]::IndexOf(@($d.tabs), '文档'); $iEmpty = [array]::IndexOf(@($d.tabs), '空的'); $iPic = [array]::IndexOf(@($d.tabs), '图片')
  # The first tab shown, the pointer off the strip.
  $p = OfTabPt $d $iDoc; [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  ParkMouse 1300 800; Start-Sleep -Seconds 2
  # A rest on the tab of the pictures (the first hover switch: with its swap), then on to
  # the tab with nothing in it and on to the documents, each shown at once.
  $p = OfTabPt $d $iPic; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 1200
  $stamp = OfNow; Start-Sleep -Milliseconds 1100
  $p = OfTabPt $d $iEmpty; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $onEmpty = (DumpOf '文档' 4).title
  $p = OfTabPt $d $iDoc; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 900
  $onDoc = (DumpOf '文档' 4).title
  ParkMouse 1300 800; Start-Sleep -Milliseconds 800
  $lines = OfLogSince $stamp
  $oneByOne = @($lines | Where-Object { $_ -match 'icons leaving fade out' -or ($_ -match 'replace_items animate=true' -and $_ -match 'old_len=0 new_len=[1-9]') } | ForEach-Object { $_ -replace '^\S+\s+\S+\s+', '' } | Select-Object -Unique)
  $atOnce = @($lines | Where-Object { $_ -match 'tab hover switch' -and $_ -match 'at_once=true' }).Count
  Check 'H4 setup: the two switches were shown at once' ($atOnce -ge 2) "hover switches shown at once: $atOnce; tabs $($d.tabs -join ','); shown $onEmpty then $onDoc"
  Check 'H4 no icon fades out or in by itself on the way across the tab with nothing in it' ($oneByOne.Count -eq 0) "$($oneByOne -join ' | ')"
  Pf settings set rollUp.switchTabOnHover false | Out-Null
  [Anim]::Set($animWas)
  PfQuit
}
if ($Part -match 'H') { OfHover }

# ---- P: a Peek, F6 to a card, then a click on a fence
function OfPeekCard() {
  trap { PfTrap $_; continue }
  Case 'P setup'
  foreach ($n in $OfDocs) { DeskFile $n | Out-Null }
  PfStart -Lang zh-CN -Script (DumpScript 250 6)
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '文档' --rect '200,120,420,240' | Out-Null
  Pf item move --glob '*.pdf' --to '文档' | Out-Null
  Pf fence create --title '多余的' --rect '200,500,300,200' | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1300 800
  $w = DumpOf '多余的' 4
  [G]::RClick($w.l + 150, $w.t + 14); Start-Sleep -Milliseconds 900
  MenuPick '^删除栅栏' | Out-Null
  Start-Sleep -Milliseconds 1500

  Case 'P5 during a Peek F6 went to a card, then a fence is clicked'
  DeskClick 1300 700 | Out-Null; Start-Sleep -Milliseconds 600
  PeekKey; Start-Sleep -Seconds 1
  [G]::Press(0x75); Start-Sleep -Milliseconds 800
  $fg1 = Fg
  $first = @(@(PfJson item list --fence '文档') | Where-Object { $_ -and $_.name }) | Select-Object -First 1
  $p = ItemPt '文档' $first.name
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 800
  $fg2 = Fg
  ParkMouse 1300 800
  [G]::Press(0x27); Start-Sleep -Milliseconds 600
  $d = DumpOf '文档' 4
  Shot 'p5-after-click'
  Check 'P5 setup: F6 gave the keyboard to the card' ($fg1 -match 'PecoFence.Notice') "after F6: $fg1"
  Check 'P5 after the click on a fence the Peek has the keyboard again' ($fg2 -match 'PeekDimmer') "after the click: $fg2; selected=$(DumpVal $d 'selected') focus_ring=$(DumpVal $d 'focus_ring')"

  # A card answered by keys (Esc on it): it hears that it lost the keyboard while it fades
  # out, so its focus ring goes at once.
  Case 'P11 a card answered by keys loses its focus ring at once'
  $stamp = OfNow; Start-Sleep -Milliseconds 1100
  [G]::Press(0x75); Start-Sleep -Milliseconds 800
  $fg3 = Fg
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1500
  $lost = @(OfLogSince $stamp | Where-Object { $_ -match 'card lost the keyboard: its focus ring goes' -and $_ -match 'leaving=true' }).Count
  Check 'P11 the card that Esc closed heard that it lost the keyboard' ($fg3 -match 'PecoFence.Notice' -and $lost -ge 1) "F6: $fg3; log lines: $lost; cards now: $((OfCards) -join ' | ')"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1200
  if (@([W]::Tops('PecoFence.PeekDimmer') | Where-Object { [G]::IsWindowVisible($_) }).Count -ge 1) { [G]::Press(0x1B); Start-Sleep -Milliseconds 1200 }
  PfQuit
}
if ($Part -match 'P') { OfPeekCard }

# ---- T: a first start as a user makes it (fresh data, no test script: the tips show)
function OfFirstStart() {
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  return (WaitWin 'PecoFence.Control' 20 -Hidden)
}
function OfTips() {
  trap { PfTrap $_; continue }
  Case 'T2 a tip, the undo card, then three files of one kind dragged into a fence'
  foreach ($n in $OfDocs) { DeskFile $n | Out-Null }
  OfFirstStart | Out-Null
  WaitWin 'PecoFence.Notice' 30 | Out-Null
  Start-Sleep -Seconds 3
  $src = $null
  foreach ($f in @(@(PfJson fence list) | Where-Object { $_ -and $_.title -and $_.windowRect })) {
    if (@(@(PfJson item list --fence $f.id --ext pdf) | Where-Object { $_ -and $_.name }).Count -ge 3) { $src = $f; break }
  }
  $fr = FreeRect 340 240 300 420
  Pf fence create --title 'Target' --rect "$($fr[0]),$($fr[1]),340,240" | Out-Null
  $fx = FreeRect 300 160 300 120
  Pf fence create --title 'Spare' --rect "$($fx[0]),$($fx[1]),300,160" | Out-Null
  Start-Sleep -Seconds 2
  $tg = @(PfJson fence list) | Where-Object { $_ -and $_.title -eq 'Target' } | Select-Object -First 1
  $sp = @(PfJson fence list) | Where-Object { $_ -and $_.title -eq 'Spare' } | Select-Object -First 1
  # The spare fence deleted from its menu: the undo card comes up beside the tip.
  [G]::RClick($sp.windowRect.x + 150, $sp.windowRect.y + 14); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^Delete fence'
  Start-Sleep -Milliseconds 1500
  $before = OfCards
  $stamp = OfNow; Start-Sleep -Milliseconds 1100
  foreach ($pdf in @(@(PfJson item list --fence $src.id --ext pdf) | Where-Object { $_ -and $_.name } | Select-Object -First 3)) {
    $pp = ItemPt $src.id $pdf.name
    if ($pp -and $tg) { DragTo $pp[0] $pp[1] ($tg.windowRect.x + 170) ($tg.windowRect.y + 150) 14; Start-Sleep -Milliseconds 600 }
  }
  Start-Sleep -Seconds 2
  ParkMouse 800 850
  $after = OfCards
  Shot 't2-after-drags'
  $gave = @(OfLogSince $stamp | Where-Object { $_ -match 'feature offer takes the place of a tip' }).Count
  $in = @(@(PfJson item list --fence $tg.id --ext pdf) | Where-Object { $_ -and $_.name }).Count
  Check 'T2 setup: a tip and the undo card are up, three PDF files went into the fence' (@($before | Where-Object { $_ -like 'Tip *' }).Count -eq 1 -and @($before | Where-Object { $_ -notlike 'Tip *' }).Count -ge 1 -and $in -eq 3) "menu=[$picked] cards before: $($before -join ' | '); files in the fence: $in"
  Check 'T2 the tip is still up (no offer can show beside the undo card)' (@($after | Where-Object { $_ -like 'Tip *' }).Count -eq 1 -and $gave -eq 0) "cards after: $($after -join ' | '); the tip gave way: $gave"
  PfQuit
}
if ($Part -match 'T') { OfTips }

# ---- C: a copy of our own that takes long, and the user goes to another program
function OfSelPx($x, $y) {
  # The selection box of the icon at cell centre $x, $y: a strip along its upper edge (the
  # shell's progress window covers the rest of the fence while the copy runs).
  $b = [Px]::Grab($x - 24, $y - 36, 48, 5); $r = 0; $g = 0; $bl = 0; $n = 0
  foreach ($yy in 0..4) { foreach ($xx in 0..47) { $c = $b.GetPixel($xx, $yy); $r += $c.R; $g += $c.G; $bl += $c.B; $n++ } }
  $b.Dispose(); return @([int]($r / $n), [int]($g / $n), [int]($bl / $n))
}
function OfLongCopy() {
  trap { PfTrap $_; continue }
  Case 'C8 a long copy of our own while the user goes to another program'
  $f = [IO.File]::Create("$PfDesk\of-big.bin"); $f.SetLength([int64]6 * 1GB); $f.Close()
  PfStart -Lang zh-CN
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf fence create --title '大文件' --rect '200,120,340,240' | Out-Null
  Pf item move --glob 'of-big.bin' --to '大文件' | Out-Null
  Start-Process notepad | Out-Null
  $n = WaitWin 'Notepad' 15
  [L]::MoveWindow($n, 800, 300, 600, 400, $true) | Out-Null
  Start-Sleep -Seconds 1
  $p = CellPt '大文件' 0; $q = CellPt '大文件' 1
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse 400 700
  $blue = OfSelPx $p[0] $p[1]
  Chord 0x11 0x43; Start-Sleep -Milliseconds 300
  Chord 0x11 0x56
  $sw = [Diagnostics.Stopwatch]::StartNew()
  Start-Sleep -Milliseconds 2500
  # The paste selects the copy, which stands in the second cell.
  $during = OfSelPx $q[0] $q[1]
  Crop 'c8-during' 200 120 340 240 | Out-Null
  # Another program: a click on Notepad's title bar.
  $stamp = OfNow
  $nr = Rect $n
  [G]::Click($nr.L + 200, $nr.T + 12); Start-Sleep -Milliseconds 2500
  $front = Fg
  $away = OfSelPx $q[0] $q[1]
  Crop 'c8-away' 200 120 340 240 | Out-Null
  $running = @(Get-ChildItem -LiteralPath $PfDesk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^of-big' }).Count
  $t = $sw.ElapsedMilliseconds
  # The copy is not waited for: PecoFence ends, and the copy with it.
  PfQuit
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Seconds 3
  OfClean
  $kept = @(PfLogAll | Where-Object { $_ -match 'activation lost while a file operation of our own runs' }).Count
  $off = @(OfLogSince $stamp | Where-Object { $_ -match 'file operation still running, another program in front' }).Count
  Check 'C8 setup: the copy ran and the selection kept its colour while the shell took the activation' ($kept -ge 1 -and $running -ge 2) "log lines: $kept; files of the copy on the desktop: $running; "
  Check 'C8 with Notepad in front the selection has the colour of a fence without the keyboard' ($front -match 'Notepad' -and $off -ge 1) "front: $front; the app's log says so: $off, $t ms into the copy (the pictures c8-during and c8-away show the box of the copy above the shell's progress window)"
}
if ($Part -match 'C') { OfLongCopy }

Case 'cleanup'
if (PfAlive) { PfQuit }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
[G]::KeyUp(0x11); [G]::KeyUp(0x12)
OfClean
Check 'test files removed' (@(Get-ChildItem $PfDesk | Where-Object { $OfDocs -contains $_.Name -or $_.Name -match '^of-big' }).Count -eq 0) ''
