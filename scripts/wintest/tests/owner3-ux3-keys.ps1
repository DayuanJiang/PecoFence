# Owner round 2026-11-12, third batch, package ux3-keys: the screen says where the keyboard is.
# Parts (run.sh --var Part=...):
#   K  the focus mark of a rolled-up fence, and after Ctrl+Tab (an icon of the new tab, or the
#      caption of a tab with nothing in it)
#   P  a folder fence: after Enter into a folder its first icon has the focus mark
#   C  the cards: the last tip's button says what it opens, F6 reaches a card's buttons, and a
#      rule offer takes the place of a first-start tip, which comes back afterwards
# Each check fails on the round's base and passes after. Runs on a 1600x900 box at 100 %.
if (-not $Part) { $Part = 'KCP' }
trap { PfTrap $_; continue }

$KkFiles = @('季度销售报告.pdf', '合同 2026.pdf', '发票.pdf', '说明.txt', '会议记录 11-12.txt', '全家福.png', '截图 2026-11-12.png')
function KkClean() {
  foreach ($n in $KkFiles) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
}
# The longest run of dark pixels in one of the screen rows $y0..$y1 between $x0 and $x1: the top
# edge of the focus mark (a dark 2 px outline in light mode; a title row has no dark row there).
function KkDarkRow($x0, $x1, $y0, $y1) {
  $b = [Px]::Grab($x0, $y0, $x1 - $x0, $y1 - $y0 + 1); $best = 0
  foreach ($r in 0..($y1 - $y0)) {
    $n = 0
    foreach ($c in 0..($x1 - $x0 - 1)) { $p = $b.GetPixel($c, $r); if ($p.R + $p.G + $p.B -lt 200) { $n++ } }
    if ($n -gt $best) { $best = $n }
  }
  $b.Dispose(); return $best
}
function KkFocus($w) { if ($w -and $w.raw -match ' focused=Some\("((?:[^"\\]|\\.)*)"\)') { return $Matches[1] }; return '-' }
# Picks a row of the open popup menu with the Down key and Enter.
function KkMenuKeys($regex) {
  $m = MenuWin 5
  if ($m -eq [IntPtr]::Zero) { Log "no menu for '$regex'"; return $false }
  $rows = @(MenuRows $m | Where-Object { $_.text })
  $i = 0; $at = -1
  foreach ($r in $rows) { if ($r.text -match $regex) { $at = $i; break }; $i++ }
  if ($at -lt 0) { Log "menu rows: $(MenuTexts $m); no '$regex'"; CloseMenus; return $false }
  foreach ($k in 0..$at) { [G]::Press(0x28); Start-Sleep -Milliseconds 120 }
  Start-Sleep -Milliseconds 300
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1200
  return $true
}

Case 'setup'
CloseWindows
KkClean
foreach ($n in $KkFiles) { DeskFile $n | Out-Null }

function KkMarks() {
  trap { PfTrap $_; continue }
  Case 'K setup'
  PfStart -Lang zh-CN -Script (DumpScript 250 8)
  Start-Sleep -Seconds 4
  ClearFences
  Pf fence create --title '文档' --rect '200,120,340,240' | Out-Null
  Pf item move --glob '*.pdf' --to '文档' | Out-Null
  Pf fence create --title '文字' --rect '600,120,340,240' | Out-Null
  Pf item move --glob '*.txt' --to '文字' | Out-Null
  Pf fence create --title '图片' --rect '1000,120,340,240' | Out-Null
  Pf item move --glob '*.png' --to '图片' | Out-Null
  Pf fence create --title '空的' --rect '1000,500,340,200' | Out-Null
  Pf fence merge '图片' --into '文字' | Out-Null
  Pf fence merge '空的' --into '文字' | Out-Null
  Start-Sleep -Seconds 3
  ParkMouse 1300 800

  # K2: Ctrl+Tab three times goes through the three tabs. A tab with icons: one of them is
  # selected and has the focus mark. The tab with nothing in it: the mark is on its caption.
  Case 'K2 after Ctrl+Tab the new tab shows where the keyboard is'
  $w = DumpOf '文字' 4
  [G]::Click($w.r - 30, $w.b - 30); Start-Sleep -Milliseconds 500
  ParkMouse 1300 800
  $icons = 0; $marked = 0; $empty = 'not visited'; $seen = @()
  foreach ($i in 1..3) {
    Chord 0x11 0x09; Start-Sleep -Milliseconds 1200
    $w = DumpOf '文字' 4
    $row = KkDarkRow ($w.l + 4) ($w.r - 4) ($w.t + 3) ($w.t + 10)
    $seen += "$($w.title): items=$(DumpVal $w 'items') selected=$(DumpVal $w 'selected') focus_ring=$(DumpVal $w 'focus_ring') focused=$(KkFocus $w) caption mark=$row px"
    Shot "k2-ctrl-tab-$i"
    if ((DumpVal $w 'items') -eq '0') { $empty = if ((DumpVal $w 'focus_ring') -eq 'true' -and $row -ge 30) { 'marked' } else { 'no mark' } }
    else { $icons++; if ((DumpVal $w 'selected') -eq '1' -and (DumpVal $w 'focus_ring') -eq 'true' -and (KkFocus $w) -ne '-') { $marked++ } }
  }
  Check 'K2 after Ctrl+Tab an icon of the new tab is selected and has the focus mark' ($icons -eq 2 -and $marked -eq 2) ($seen -join ' / ')
  Check 'K2 a tab with nothing in it has the focus mark on its caption' ($empty -eq 'marked') "empty tab: $empty"
  # On to a tab with icons (the third Ctrl+Tab ended on the one with nothing in it).
  Chord 0x11 0x09; Start-Sleep -Milliseconds 1200
  [G]::Press(0x27); Start-Sleep -Milliseconds 500
  $w = DumpOf '文字' 4
  Check 'K2 the arrow keys go on from the marked icon' ((DumpVal $w 'selected') -eq '1' -and (DumpVal $w 'focus_ring') -eq 'true') "$($w.title): selected=$(DumpVal $w 'selected') focused=$(KkFocus $w)"

  # K1: a fence is rolled up from its menu by keys alone. The keyboard stays in it, and its
  # title row has the focus mark; Tab takes the mark along to the next fence.
  Case 'K1 a rolled-up fence that has the keyboard shows the focus mark'
  $first = @(PfJson item list --fence '文档') | Where-Object { $_ -and $_.name } | Select-Object -First 1
  $p = ItemPt '文档' $first.name
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse 1300 800
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  [G]::Press(0x5D); Start-Sleep -Milliseconds 900
  $ok = KkMenuKeys '^收起'
  Start-Sleep -Milliseconds 800
  $w = DumpOf '文档' 4
  $width = $w.r - $w.l
  $row = KkDarkRow ($w.l + 4) ($w.r - 4) ($w.t + 3) ($w.t + 8)
  Shot 'k1-rolled'
  Check 'K1 setup: the fence was rolled up by keys and has the keyboard' ($ok -and "$($w.rolled)" -match 'True' -and (Fg) -match "PecoFence.Fence '文档'") "rolled=$($w.rolled) fg=$(Fg)"
  Check 'K1 the title row of the rolled-up fence has the focus mark' ($row -ge $width * 0.6) "mark's top edge: $row px of $width px (focus_ring=$(DumpVal $w 'focus_ring'))"
  # Enter on the marked title row opens the fence again (the key did nothing there).
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1200
  $w = DumpOf '文档' 4
  Shot 'k1-enter-opened'
  Check 'K1 Enter on the marked title row opens the fence again' ("$($w.rolled)" -notmatch 'True' -and (Fg) -match "PecoFence.Fence '文档'") "rolled=$($w.rolled) fg=$(Fg) focus_ring=$(DumpVal $w 'focus_ring')"
  [G]::Press(0x09); Start-Sleep -Milliseconds 900
  $w = DumpOf '文档' 4
  $row2 = KkDarkRow ($w.l + 4) ($w.r - 4) ($w.t + 3) ($w.t + 8)
  Shot 'k1-tab-away'
  Check 'K1 after Tab the mark has left the fence' ($row2 -lt 10 -and (Fg) -notmatch "'文档'") "mark's top edge: $row2 px; fg=$(Fg)"
  # During a Peek the fences get no focus messages: Tab, Tab leaves the mark with one fence only
  # (the fence left behind would show it on its title row).
  DeskClick 1300 600 | Out-Null
  PeekKey; Start-Sleep -Seconds 1
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  $marked = @(@(PfDump 4) | Where-Object { (DumpVal $_ 'focus_ring') -eq 'true' })
  Shot 'k1-peek-tab-tab'
  Check 'K1 during a Peek only the fence that has the keys has a focus mark' ($marked.Count -eq 1) "fences with a mark: $(($marked | ForEach-Object { $_.title }) -join ', ')"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1200
  PfQuit
}
if ($Part -match 'K') { KkMarks }

# ---- C: the cards. A first start as a user makes it: fresh data, no test script (the tips show).
function KcStart() {
  if (PfAlive) { PfQuit }
  Remove-Item -Recurse -Force "$PfApp\config", "$PfApp\notices.json" -ErrorAction SilentlyContinue
  if (-not (Test-Path "$PfApp\pecofence.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
  $env:PECOFENCE_INSTANCE = $PfInst
  $env:RUST_LOG = 'pecofence=debug'
  Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
  return (WaitWin 'PecoFence.Control' 20 -Hidden)
}
# The card on screen (a card that was answered fades out and stays as a hidden window).
function KcCard() { foreach ($h in @([W]::Tops('PecoFence.Notice'))) { if ([G]::IsWindowVisible($h)) { return $h } }; return $null }
function KcTitle() { $n = KcCard; if ($n) { return [L]::Text($n) }; return 'none' }
# Puts the keyboard into fence $f: a click on its first icon, then the Right key.
function KcKeysInto($f) {
  $first = @(@(PfJson item list --fence $f.id) | Where-Object { $_ -and $_.name }) | Select-Object -First 1
  $p = ItemPt $f.id $first.name
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse 800 850
  [G]::Press(0x27); Start-Sleep -Milliseconds 400
}
function KcCards() {
  trap { PfTrap $_; continue }
  Case 'C setup: a first start with its tips'
  KcStart | Out-Null
  WaitWin 'PecoFence.Notice' 30 | Out-Null
  Start-Sleep -Seconds 3
  $src = $null
  foreach ($f in @(@(PfJson fence list) | Where-Object { $_ -and $_.title -and $_.windowRect })) {
    if (@(@(PfJson item list --fence $f.id --ext pdf) | Where-Object { $_ -and $_.name }).Count -ge 3) { $src = $f; break }
  }
  $fr = FreeRect 340 240 300 420
  Pf fence create --title 'Target' --rect "$($fr[0]),$($fr[1]),340,240" | Out-Null
  Start-Sleep -Seconds 2
  $tg = @(PfJson fence list) | Where-Object { $_ -and $_.title -eq 'Target' } | Select-Object -First 1
  $t0 = KcTitle
  Check 'C setup: the first tip is up and a fence holds three PDF files' ($t0 -like 'Tip 1/*' -and $src -and $tg) "card=[$t0] source fence=[$($src.title)]"

  # C3: three PDF files dragged into one fence while a first-start tip is up: the rule offer
  # takes the tip's place (it was dropped), and the tip comes back once the offer is closed.
  Case 'C3 a rule offer takes the place of a first-start tip, and the tip comes back'
  foreach ($pdf in @(@(PfJson item list --fence $src.id --ext pdf) | Where-Object { $_ -and $_.name } | Select-Object -First 3)) {
    $pp = ItemPt $src.id $pdf.name
    if ($pp -and $tg) { DragTo $pp[0] $pp[1] ($tg.windowRect.x + 170) ($tg.windowRect.y + 150) 14; Start-Sleep -Milliseconds 600 }
  }
  Start-Sleep -Seconds 2
  ParkMouse 800 850
  Shot 'c3-offer'
  $rule = @(PfLogAll | Select-String 'welcome notice shown notice=RuleTip' -SimpleMatch).Count
  $t1 = KcTitle
  Check 'C3 setup: three PDF files are in the fence' (@(@(PfJson item list --fence $tg.id --ext pdf) | Where-Object { $_ -and $_.name }).Count -eq 3) ''
  Check 'C3 the rule offer shows although a first-start tip was up' ($rule -eq 1 -and $t1 -notlike 'Tip *' -and $t1 -ne 'none') "rule offer lines=$rule card=[$t1]"
  if ($rule -eq 1) { $r = Rect (KcCard); [G]::Click($r.R - 24, $r.T + 24); Start-Sleep -Seconds 2; ParkMouse 800 850 }
  $back = @(PfLogAll | Select-String 'the first-start tip comes back after the offer' -SimpleMatch).Count
  $t2 = KcTitle
  Shot 'c3-tip-back'
  Check 'C3 the tip comes back once the offer is closed' ($back -ge 1 -and $t2 -like 'Tip 1/*') "tip back lines=$back (the log and its shareable copy) card=[$t2]"

  # C2: F6 in a fence moves the keyboard to the card: the button has the focus mark, Enter
  # presses it, Tab goes to × and back, F6 gives the keyboard back to the fence.
  Case 'C2 F6 moves the keyboard to the card and back'
  KcKeysInto $tg
  $fg0 = Fg
  [G]::Press(0x75); Start-Sleep -Milliseconds 700
  $fg1 = Fg
  Shot 'c2-f6-on-card'
  Check 'C2 F6 in a fence moves the keyboard to the card' ($fg0 -match 'PecoFence.Fence' -and $fg1 -match 'PecoFence.Notice') "before F6: $fg0; after: $fg1"
  if ($fg1 -match 'PecoFence.Notice') {
    $ta = KcTitle
    [G]::Press(0x0D); Start-Sleep -Milliseconds 900
    $tb = KcTitle
    Check 'C2 Enter presses the focused button: the next tip' ($ta -like 'Tip 1/*' -and $tb -like 'Tip 2/*' -and (Fg) -match 'PecoFence.Notice') "[$ta] then [$tb]; fg=$(Fg)"
    [G]::Press(0x09); Start-Sleep -Milliseconds 400
    Shot 'c2-tab-close'
    [G]::Press(0x09); Start-Sleep -Milliseconds 400
    [G]::Press(0x75); Start-Sleep -Milliseconds 700
    Shot 'c2-f6-back'
    Check 'C2 F6 on the card gives the keyboard back to the fence' ((Fg) -match "PecoFence.Fence 'Target'") "fg=$(Fg)"
  }

  # C4: the same during a Peek: F6 moves the keyboard to the card and the Peek stays open,
  # F6 again gives it back to the Peek.
  Case 'C4 F6 during a Peek'
  DeskClick 700 780 | Out-Null
  PeekKey; Start-Sleep -Seconds 1
  [G]::Press(0x09); Start-Sleep -Milliseconds 700
  $fg0 = Fg
  [G]::Press(0x75); Start-Sleep -Milliseconds 700
  $fg1 = Fg
  $open1 = @([W]::Tops('PecoFence.PeekDimmer') | Where-Object { [G]::IsWindowVisible($_) }).Count
  Shot 'c4-peek-f6'
  [G]::Press(0x75); Start-Sleep -Milliseconds 700
  $fg2 = Fg
  $open2 = @([W]::Tops('PecoFence.PeekDimmer') | Where-Object { [G]::IsWindowVisible($_) }).Count
  Check 'C4 F6 during a Peek moves the keyboard to the card and the Peek stays open' ($fg0 -match 'PeekDimmer' -and $fg1 -match 'PecoFence.Notice' -and $open1 -ge 1) "before F6: $fg0; after: $fg1; Peek windows=$open1"
  Check 'C4 F6 on the card gives the keyboard back to the Peek' ($fg2 -match 'PeekDimmer' -and $open2 -ge 1) "fg=$fg2; Peek windows=$open2"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1200
  if (@([W]::Tops('PecoFence.PeekDimmer') | Where-Object { [G]::IsWindowVisible($_) }).Count -ge 1) { [G]::Press(0x1B); Start-Sleep -Milliseconds 1200 }

  # C1: the last tip's button says what it opens (it read "Manual"): its label is longer than
  # the button's smallest width, 120 px. The words are checked by a unit test of the catalogs.
  Case 'C1 the last tip''s button says that it opens the manual'
  foreach ($i in 1..5) {
    $t = KcTitle
    if ($t -match '^Tip (\d+)/(\d+)' -and $Matches[1] -eq $Matches[2]) { break }
    $r = Rect (KcCard); [G]::Click($r.R - 56, $r.B - 32); Start-Sleep -Milliseconds 800
  }
  ParkMouse 800 850
  Start-Sleep -Milliseconds 500
  $r = Rect (KcCard)
  $b = [Px]::Grab($r.L, $r.B - 44, $r.R - $r.L, 1); $blue = 0
  foreach ($c in 0..($r.R - $r.L - 1)) { $p = $b.GetPixel($c, 0); if ($p.B -gt $p.R + 80) { $blue++ } }
  $b.Dispose()
  Shot 'c1-last-tip'
  Check 'C1 the button of the last tip is wider than the smallest button' ((KcTitle) -match '^Tip (\d+)/\1' -and $blue -ge 126) "card=[$(KcTitle)] button width=$blue px"

  Case 'C2 Esc closes the card'
  KcKeysInto $tg
  [G]::Press(0x75); Start-Sleep -Milliseconds 700
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1500
  Shot 'c2-esc'
  Check 'C2 Esc on the card closes it and the keyboard is back in the fence' ((KcCard) -eq $null -and (Fg) -match "PecoFence.Fence 'Target'") "card=[$(KcTitle)] fg=$(Fg)"
  PfQuit
}
if ($Part -match 'C') { KcCards }

# ---- P: a folder fence. Enter on a folder goes into it: the first icon of the folder that
# arrives has the focus mark (nothing was marked until an arrow key was pressed).
function KpFolder() {
  trap { PfTrap $_; continue }
  Case 'P1 after Enter into a folder its first icon has the focus mark'
  $root = "$PfRun\kp-folder"
  New-Item -ItemType Directory -Force "$root\子文件夹" | Out-Null
  foreach ($n in @('子文件夹\里面的第一个文件.txt', '子文件夹\第二个.txt', '外面的文件.txt')) { Set-Content -LiteralPath "$root\$n" -Value 'x' -Encoding ASCII }
  PfStart -Lang zh-CN -Script (DumpScript 250 6)
  Start-Sleep -Seconds 4
  ClearFences
  Pf fence create --portal $root --title 'Kp' --rect '520,160,340,240' | Out-Null
  Start-Sleep -Seconds 3
  $file = @(@(PfJson item list --fence 'Kp') | Where-Object { $_ -and $_.name -like '外面*' }) | Select-Object -First 1
  $p = ItemPt 'Kp' $file.name
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
  ParkMouse 1300 800
  # Home or End puts the selection and the mark on the folder (it stands first or last).
  $on = '-'
  foreach ($vk in 0x24, 0x23) {
    [G]::Press($vk); Start-Sleep -Milliseconds 500
    $w = @(PfDump 4) | Where-Object { $_.l -eq 520 -and $_.t -eq 160 } | Select-Object -First 1
    $on = KkFocus $w
    if ($on -eq '子文件夹') { break }
  }
  if ($on -eq '子文件夹') { [G]::Press(0x0D); Start-Sleep -Milliseconds 1800 }
  $w = @(PfDump 4) | Where-Object { $_.l -eq 520 -and $_.t -eq 160 } | Select-Object -First 1
  Shot 'p1-entered'
  Check 'P1 setup: the keyboard is on the folder' ($on -eq '子文件夹') "focused=[$on]"
  Check 'P1 after Enter the first icon of the folder has the focus mark' ((DumpVal $w 'items') -eq '2' -and (DumpVal $w 'focus_ring') -eq 'true' -and (KkFocus $w) -ne '-' -and (DumpVal $w 'selected') -eq '0') "items=$(DumpVal $w 'items') focus_ring=$(DumpVal $w 'focus_ring') focused=$(KkFocus $w) selected=$(DumpVal $w 'selected')"
  [G]::Press(0x27); Start-Sleep -Milliseconds 500
  $w = @(PfDump 4) | Where-Object { $_.l -eq 520 -and $_.t -eq 160 } | Select-Object -First 1
  Check 'P1 an arrow key goes on from there' ((DumpVal $w 'selected') -eq '1' -and (DumpVal $w 'focus_ring') -eq 'true') "selected=$(DumpVal $w 'selected') focused=$(KkFocus $w)"
  PfQuit
}
if ($Part -match 'P') { KpFolder }

Case 'cleanup'
if (PfAlive) { PfQuit }
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
KkClean
Check 'test files removed' (@(Get-ChildItem $PfDesk | Where-Object { $KkFiles -contains $_.Name }).Count -eq 0) ''
