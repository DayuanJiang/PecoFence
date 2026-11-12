# Owner round 2026-11-12, fifth batch, package ux5-fixes-a: what a reader of the morning's and
# the afternoon's merged fence code found. Parts (run.sh --var Part=...):
#   N  notes: a rolled note fence stays open while its note is edited (hover expansion, and
#      「编辑便签」 from the rolled title's menu opens it first); a press on the scroll bar of a
#      long note leaves the edit going; selected right-to-left text has its fill; the editor's
#      control follows the caret after a wheel scroll; Shift+Right across the start of a
#      to-do; what a draw of a 300-line note costs
#   C  chrome: the chevron's tip during a hover expansion says what the click does; the name
#      box of a rolled note fence with a right-aligned title stands on the name
# Runs on a 1600x900 box at 100 %, English interface.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'NC' }
trap { PfTrap $_; continue }

function AfNoteText($title) { return "$((FenceGet $title).noteText)" }
function AfWin($title) { return [G]::FindWindowW('PecoFence.Fence', $title) }
function AfHeight($title) { $r = Rect (AfWin $title); return $r.B - $r.T }
function AfNow() { return (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ss') }
function AfLogSince($stamp) { return @(PfLogAll | Where-Object { $_.Length -gt 19 -and [string]::CompareOrdinal($_.Substring(0, 19), $stamp) -ge 0 }) }
# How many pixels of two grabs of the same place differ clearly.
function AfChanged($a, $b) {
  $n = 0
  foreach ($y in 0..($a.Height - 1)) { foreach ($x in 0..($a.Width - 1)) { $p = $a.GetPixel($x, $y); $q = $b.GetPixel($x, $y); if ([math]::Abs($p.R - $q.R) + [math]::Abs($p.G - $q.G) + [math]::Abs($p.B - $q.B) -gt 30) { $n++ } } }
  return $n
}

Case 'setup'
CloseWindows

function AfNotes() {
  trap { PfTrap $_; continue }
  Case 'N setup'
  $animWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang en
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf settings set tools.enabled true | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --tool note --title 'Shopping' --text "milk`neggs" --rect '200,120,320,220' | Out-Null
  $long = (1..60 | ForEach-Object { "line $_ of the long note" }) -join "`n"
  Pf fence create --tool note --title 'Long' --text $long --rect '600,120,320,220' | Out-Null
  Pf fence create --tool note --title 'Hebrew' --text 'x' --rect '1000,120,320,160' | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 800 700

  # 1a: the note fence rolled up, opened by the pointer resting on it, a click into its text,
  # typing, the pointer away: the fence stays open while the note is edited.
  Case 'N1 a hover-expanded note fence stays open while its note is edited'
  Pf fence roll 'Shopping' | Out-Null
  Start-Sleep -Milliseconds 1200
  ParkMouse 800 700; Start-Sleep -Milliseconds 800
  $rolledH = AfHeight 'Shopping'
  [G]::MoveTo(330, 134); Start-Sleep -Milliseconds 1500
  $openH = AfHeight 'Shopping'
  [G]::Click(300, 215); Start-Sleep -Milliseconds 900
  [G]::Type(' bread'); Start-Sleep -Milliseconds 400
  ParkMouse 800 700; Start-Sleep -Milliseconds 1800
  $editH = AfHeight 'Shopping'
  Shot 'n1-editing-pointer-away'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 2000
  $afterH = AfHeight 'Shopping'
  $text = AfNoteText 'Shopping'
  Check 'N1 setup: the rolled fence opened under the pointer and took the typing' ($rolledH -lt 60 -and $openH -gt 150 -and $text -match 'bread') "heights: rolled $rolledH, under the pointer $openH; text [$($text -replace "`n", ' / ')]"
  Check 'N1 the fence stays open while the note is edited, the pointer away' ($editH -gt 150) "height while editing with the pointer away: $editH"
  Check 'N1 it rolls up again once the edit has ended' ($afterH -lt 60) "height after Esc: $afterH"

  # 1b: 「编辑便签」 from the rolled title's menu opens the fence first.
  Case 'N1 Edit note from the menu of a rolled note fence opens the fence'
  ParkMouse 800 700; Start-Sleep -Milliseconds 600
  $h0 = AfHeight 'Shopping'
  [G]::RClick(330, 134); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^Edit note'
  Start-Sleep -Milliseconds 1500
  [G]::Type('tea '); Start-Sleep -Milliseconds 500
  ParkMouse 800 700; Start-Sleep -Milliseconds 1500
  $h1 = AfHeight 'Shopping'
  Shot 'n1-menu-edit'
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1200
  $text = AfNoteText 'Shopping'
  Check 'N1 Edit note on a rolled fence: the fence is open while the note is edited' ($h0 -lt 60 -and $h1 -gt 150 -and $text -match 'tea') "menu=[$picked]; height before $h0, while editing $h1; text [$($text -replace "`n", ' / ')]"

  # 2: a press on the scroll bar of a long note leaves the edit going.
  Case 'N2 a press on the scroll bar of a note that is being edited'
  [G]::Click(700, 200); Start-Sleep -Milliseconds 900
  $r = Rect (AfWin 'Long')
  # The track below the thumb: one page down.
  [G]::Click($r.R - 5, $r.B - 40); Start-Sleep -Milliseconds 700
  [G]::Type('Q'); Start-Sleep -Milliseconds 400
  [G]::Press(0x1B); Start-Sleep -Milliseconds 1000
  $text = AfNoteText 'Long'
  Check 'N2 after the press on the scroll bar the typing still goes into the note' ($text -match 'Q') "Q in the note: $($text -match 'Q'); first line [$(($text -split "`n")[0])]"

  # 11: the editor's control follows the caret after a wheel scroll.
  Case 'N11 the editor follows the caret after a wheel scroll'
  [G]::Click(700, 200); Start-Sleep -Milliseconds 900
  # The editor's text box inside its (unseen) window: the input method's window opens at it.
  $ed = WaitWin 'PecoFence.NoteEdit' 5 -Hidden
  $box = [M]::FindWindowExW($ed, [IntPtr]::Zero, 'Edit', [NullString]::Value)
  $e0 = Rect $box
  Wheel 700 250 -3; Start-Sleep -Milliseconds 1500
  $e1 = Rect $box
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  Check 'N11 the control has moved with the scrolled text within 1.5 s' ($box -ne [IntPtr]::Zero -and $e1.T -ne $e0.T) "the control's top before the wheel $($e0.T), after $($e1.T)"

  # 9: selected right-to-left text has its fill.
  Case 'N9 selected right-to-left text in a note'
  [G]::Click(1100, 200); Start-Sleep -Milliseconds 900
  Chord 0x11 0x41; Start-Sleep -Milliseconds 200
  [G]::Type(([string][char]0x05E9) + [char]0x05DC + [char]0x05D5 + [char]0x05DD + ' ' + [char]0x05E2 + [char]0x05D5 + [char]0x05DC + [char]0x05DD)
  Start-Sleep -Milliseconds 600
  ParkMouse 800 700; Start-Sleep -Milliseconds 300
  $g0 = [Px]::Grab(1010, 160, 300, 40)
  Chord 0x11 0x41; Start-Sleep -Milliseconds 700
  $g1 = [Px]::Grab(1010, 160, 300, 40)
  $n = AfChanged $g0 $g1
  Crop 'n9-hebrew-selected' 1000 120 320 100 | Out-Null
  $g0.Dispose(); $g1.Dispose()
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  Check 'N9 Ctrl+A in a line of Hebrew shows the selection fill' ($n -gt 400) "pixels that changed with the selection: $n"

  # 8: Shift+Right from the end of a line across the start of a to-do: the selection's end
  # does not stop inside the unseen marker, so the letter typed next takes the whole marker.
  Case 'N8 Shift+Right across the start of a to-do line'
  Pf fence create --tool note --title 'Todo' --text "abc`n[ ] milk" --rect '1000,320,320,160' | Out-Null
  Start-Sleep -Seconds 2
  [G]::Click(1250, 372); Start-Sleep -Milliseconds 900
  Chord 0x11 0x24; Start-Sleep -Milliseconds 200
  [G]::Press(0x23); Start-Sleep -Milliseconds 200
  [G]::KeyDown(0x10); [G]::Press(0x27); Start-Sleep -Milliseconds 150; [G]::Press(0x27); [G]::KeyUp(0x10)
  Start-Sleep -Milliseconds 400
  Crop 'n8-two-shift-right' 1000 320 320 110 | Out-Null
  [G]::Type('x'); Start-Sleep -Milliseconds 400
  [G]::Press(0x1B); Start-Sleep -Milliseconds 900
  $text = AfNoteText 'Todo'
  Check 'N8 the letter typed after two Shift+Right takes the line break and the whole marker' ($text -eq 'abcxmilk') "note text: [$($text -replace "`n", ' / ')]"

  # 13: what one draw of a 300-line note costs while its caret blinks (the note is laid out
  # once and its rows are kept: every draw laid it out twice, 18 ms each on this box).
  Case 'N13 the draws of a 300-line note that is being edited'
  $big = (1..300 | ForEach-Object { "row $_ of three hundred, with some words to wrap at the edge" }) -join "`n"
  Pf fence create --tool note --title 'Big' --text $big --rect '200,420,320,300' | Out-Null
  Start-Sleep -Seconds 2
  [G]::Click(300, 520); Start-Sleep -Milliseconds 900
  ParkMouse 800 800
  $stamp = AfNow; Start-Sleep -Milliseconds 4500
  $id = (FenceGet 'Big').id
  $ms = @(AfLogSince $stamp | Where-Object { $_ -match 'note draw' -and $_ -match $id -and $_ -match ' ms=([0-9.]+)' } | ForEach-Object { [double]([regex]::Match($_, ' ms=([0-9.]+)').Groups[1].Value) } | Sort-Object)
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  $med = if ($ms.Count) { $ms[[int]($ms.Count / 2)] } else { -1 }
  $max = if ($ms.Count) { $ms[-1] } else { -1 }
  Check 'N13 a draw of the 300-line note while its caret blinks takes under 2 ms' ($ms.Count -ge 4 -and $med -lt 2) "draws in 4.5 s: $($ms.Count); median $med ms, slowest $max ms"
  [Anim]::Set($animWas)
  PfQuit
}
if ($Part -match 'N') { AfNotes }

function AfChrome() {
  trap { PfTrap $_; continue }
  Case 'C setup'
  DeskFile 'report.pdf' | Out-Null
  PfStart -Lang en
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
  ClearFences | Out-Null
  Pf settings set tools.enabled true | Out-Null
  Pf settings set featureTips false | Out-Null
  Pf fence create --title 'Papers' --rect '200,120,340,240' | Out-Null
  Pf item move --glob 'report.pdf' --to 'Papers' | Out-Null
  Pf fence create --tool note --title 'Memo' --text 'a note' --rect '700,120,320,200' | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 800 700

  # 5: during a hover expansion a click on the chevron keeps the fence open: the tip says so.
  Case 'C5 the tip of the chevron during a hover expansion'
  Pf fence roll 'Papers' | Out-Null
  Start-Sleep -Milliseconds 1200
  ParkMouse 800 700; Start-Sleep -Milliseconds 800
  [G]::MoveTo(320, 134); Start-Sleep -Milliseconds 1500
  $openH = AfHeight 'Papers'
  $r = Rect (AfWin 'Papers')
  [G]::MoveTo($r.R - 18, $r.T + 18); Start-Sleep -Milliseconds 300
  [G]::MoveTo($r.R - 17, $r.T + 18); Start-Sleep -Milliseconds 1600
  $tip = (OcrText ($r.R - 170) ($r.T - 60) 330 150) -replace '\s+', ' '
  Crop 'c5-chevron-tip' ($r.R - 170) ($r.T - 60) 330 150 | Out-Null
  [G]::Click($r.R - 17, $r.T + 18); Start-Sleep -Milliseconds 800
  ParkMouse 800 700; Start-Sleep -Milliseconds 1500
  $stayH = AfHeight 'Papers'
  Check 'C5 setup: the fence is open by the pointer, and the click on the chevron keeps it open' ($openH -gt 150 -and $stayH -gt 150) "height under the pointer $openH, after the click and the pointer away $stayH"
  Check 'C5 the tip says Expand (what the click does), not Roll up' ($tip -match 'Expand' -and $tip -notmatch 'Roll up') "the tip reads: [$tip]"

  # 7: a rolled note fence shows no count: the name box of its right-aligned title stands on
  # the name (it stood 64 px and more to the left, where a count would have its room).
  Case 'C7 the name box of a rolled note fence with a right-aligned title'
  Pf settings set titleAlign right | Out-Null
  Pf fence roll 'Memo' | Out-Null
  Start-Sleep -Milliseconds 1500
  ParkMouse 800 700; Start-Sleep -Milliseconds 800
  $r = Rect (AfWin 'Memo')
  Crop 'c7-rolled-title' ($r.L) ($r.T - 4) ($r.R - $r.L) 48 | Out-Null
  [G]::RClick($r.L + 60, $r.T + 18); Start-Sleep -Milliseconds 900
  $picked = MenuPick '^Rename'
  Start-Sleep -Milliseconds 1200
  $box = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value)
  $b = Rect $box
  Crop 'c7-name-box' ($r.L) ($r.T - 4) ($r.R - $r.L) 48 | Out-Null
  [G]::Press(0x1B); Start-Sleep -Milliseconds 800
  $gap = $r.R - $b.R
  Check 'C7 the name box ends where the name does, not a count''s room further left' ($box -ne [IntPtr]::Zero -and $gap -ge 0 -and $gap -lt 64) "menu=[$picked]; the fence ends at $($r.R), the name box at $($b.R): $gap px between"
  Pf settings set titleAlign left | Out-Null
  PfQuit
}
if ($Part -match 'C') { AfChrome }

Case 'cleanup'
if (PfAlive) { PfQuit }
[G]::KeyUp(0x11); [G]::KeyUp(0x12)
Remove-Item -LiteralPath (Join-Path $PfDesk 'report.pdf') -Force -ErrorAction SilentlyContinue
Check 'test files removed' (-not (Test-Path (Join-Path $PfDesk 'report.pdf'))) ''
