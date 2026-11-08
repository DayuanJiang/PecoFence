# Fence fix round 2026-11-07, area ux-snap-fit, part A: lining fences up by dragging (U3, fb#100),
# Ctrl+wheel with a fence right below (U16), fitting the height to the icons (U6).
# Real mouse throughout; outcomes read from `fence get` and the guide windows.

function FR($t) { return (FenceGet $t).windowRect }
function GuideOn() { return @([W]::Tops('PecoFence.DragGuide')).Count }
function ClearFences() {
  foreach ($f in @(PfJson fence list)) { if ($f -and $f.title -and $f.kind -ne 'inbox') { Pf fence delete "$($f.id)" | Out-Null } }
  Start-Sleep -Seconds 1
}
$names = @('fxa Budget.xlsx', 'fxa Notes.docx', 'fxa Report.docx', 'fxa Slides.pptx', 'fxa Todo.txt', 'fxa Readme.txt')

Case 'setup'
CloseWindows
[G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
foreach ($n in $names) { Set-Content -LiteralPath (Join-Path $PfDesk $n) 'fx' }
PfStart -Script (DumpScript 250 12)
ClearFences
Pf fence create --title Work --rect '420,120,300,250' | Out-Null
Pf fence create --title Photos --rect '900,500,240,200' | Out-Null
Pf fence create --title Games --rect '200,560,240,200' | Out-Null
Pf item move @($names | ForEach-Object { Join-Path $PfDesk $_ }) --to Work | Out-Null
Start-Sleep -Seconds 2
ParkMouse 1100 830
Log "fences: $((@(PfJson fence list) | Where-Object { $_.title } | ForEach-Object { "$($_.title) $($_.windowRect.x),$($_.windowRect.y) $($_.windowRect.w)x$($_.windowRect.h)" }) -join ' ; ')"

Case 'U3 dropped beside another fence'
$hits = 0; $rows = @()
foreach ($g in 3, 6, 9, 12, 15) {
  Pf fence move Photos --rect '900,500,240,200' | Out-Null; Start-Sleep -Milliseconds 600
  $w = FR 'Work'; $ph = FR 'Photos'
  # The left edge $g px right of Work, the top 5 px below Work's top.
  DragTo ($ph.x + 100) ($ph.y + 16) ($w.x + $w.w + $g + 100) ($w.y + 5 + 16) 20
  $p2 = FR 'Photos'
  if ($p2.y -eq $w.y -and $p2.x - $w.x - $w.w -eq 8) { $hits++ }
  $rows += "pre-gap ${g}: gap $($p2.x - $w.x - $w.w), top offset $($p2.y - $w.y)"
}
Shot 'u3-beside-last'
Check 'beside: 8 px gap and the same top for every drop (5 drops, 5 px off)' ($hits -eq 5) "$hits of 5: $($rows -join ' | ')"

Case 'U3 dropped under another fence'
Pf fence move Photos --rect '900,500,240,200' | Out-Null
$hits = 0; $rows = @()
foreach ($g in 3, 6, 9, 12, 15) {
  Pf fence move Games --rect '200,560,240,200' | Out-Null; Start-Sleep -Milliseconds 600
  $w = FR 'Work'; $gm = FR 'Games'
  DragTo ($gm.x + 100) ($gm.y + 16) ($w.x + 5 + 100) ($w.y + $w.h + $g + 16) 20
  $g2 = FR 'Games'
  if ($g2.x -eq $w.x -and $g2.y - $w.y - $w.h -eq 8) { $hits++ }
  $rows += "pre-gap ${g}: gap $($g2.y - $w.y - $w.h), left offset $($g2.x - $w.x)"
}
Shot 'u3-under-last'
Check 'under: 8 px gap and the same left edge for every drop (5 drops, 5 px off)' ($hits -eq 5) "$hits of 5: $($rows -join ' | ')"

Case 'U3 a fence at the far side of the screen (fb#100)'
Pf fence move Games --rect '200,560,240,200' | Out-Null
Pf fence move Work --rect '100,120,300,250' | Out-Null; Start-Sleep -Milliseconds 700
$w = FR 'Work'
$res = @(); $ok = 0
foreach ($off in 0, 2, 4, 6) {
  Pf fence move Photos --rect '1000,520,240,200' | Out-Null; Start-Sleep -Milliseconds 700
  $ph = FR 'Photos'
  $sx = $ph.x + 100; $sy = $ph.y + 16; $tx = 1100; $ty = $w.y + $off + 16
  [G]::MoveTo($sx, $sy); Start-Sleep -Milliseconds 150; MDown; Start-Sleep -Milliseconds 150
  Glide $sx $sy $tx $ty 24 25; Start-Sleep -Milliseconds 600
  $g = GuideOn
  if ($off -eq 4) { Shot 'u3-far-4px-mid-drag' }
  MUp; Start-Sleep -Milliseconds 900
  $p2 = FR 'Photos'
  if ($g -ge 1 -and $p2.y -eq $w.y) { $ok++ }
  $res += "aim ${off} px off: guide windows $g, dropped top offset $($p2.y - $w.y), gap $($p2.x - $w.x - $w.w)"
}
Check 'far (600 px apart): a guide shows and the tops end level (aim 0, 2, 4, 6 px off)' ($ok -eq 4) "$ok of 4: $($res -join ' | ')"
# 30 px off stays where it is dropped: far fences do not pull from farther than the snap distance.
Pf fence move Photos --rect '1000,520,240,200' | Out-Null; Start-Sleep -Milliseconds 700
$ph = FR 'Photos'
DragTo ($ph.x + 100) ($ph.y + 16) 1100 ($w.y + 30 + 16) 20
$p2 = FR 'Photos'
Check 'far, 30 px off: no pull, it stays where it was dropped' ($p2.y - $w.y -eq 30) "dropped top offset $($p2.y - $w.y)"
Pf fence move Photos --rect '900,500,240,200' | Out-Null
Pf fence move Work --rect '420,120,300,250' | Out-Null; Start-Sleep -Milliseconds 700

Case 'U16 Ctrl+wheel with a fence right below'
Pf fence move Photos --rect '1100,500,240,200' | Out-Null
Pf fence create --title Below --rect '420,378,300,200' | Out-Null; Start-Sleep -Seconds 1
$w0 = FenceGet 'Work'; $b0 = FR 'Below'
Log "start: Work $($w0.windowRect | ConvertTo-Json -Compress) icon $($w0.iconSize) fit $($w0.fit | ConvertTo-Json -Compress); Below y $($b0.y)"
[G]::KeyDown(0x11); Wheel ($w0.windowRect.x + 150) ($w0.windowRect.y + 150) 1; Start-Sleep -Milliseconds 100; [G]::KeyUp(0x11)
Start-Sleep -Milliseconds 1800
$w1 = FenceGet 'Work'; $b1 = FR 'Below'
$r1 = $w1.windowRect
Shot 'u16-ctrl-wheel-larger'
Check 'Ctrl+wheel larger: Work grows to show every row' ($w1.iconSize -gt $w0.iconSize -and -not $w1.fit.overflow -and $r1.h -eq $w1.fit.fittingHeight) "icon $($w1.iconSize), h $($r1.h), fitting $($w1.fit.fittingHeight), overflow $($w1.fit.overflow)"
Check 'the fence below makes room (pushed under it, a gap apart)' ($b1.y -ge $r1.y + $r1.h) "Work bottom $($r1.y + $r1.h), Below top $($b1.y) (was $($b0.y))"
[G]::KeyDown(0x11); Wheel ($r1.x + 150) ($r1.y + 150) -1; Start-Sleep -Milliseconds 100; [G]::KeyUp(0x11)
Start-Sleep -Milliseconds 1800
$w2 = FenceGet 'Work'; $b2 = FR 'Below'
Shot 'u16-ctrl-wheel-back'
Check 'Ctrl+wheel back: Work has its old height again' ($w2.iconSize -eq $w0.iconSize -and $w2.windowRect.h -eq $w0.windowRect.h) "icon $($w2.iconSize), h $($w2.windowRect.h) (was $($w0.windowRect.h))"
Check 'the fence below is back where it was' ($b2.y -eq $b0.y) "Below top $($b2.y) (was $($b0.y))"
Pf fence delete Below | Out-Null
Pf fence move Work --rect '420,120,300,250' | Out-Null; Start-Sleep -Milliseconds 700

Case 'U6 double-click on the bottom edge'
$ok = 0; $rows = @()
foreach ($h in 420, 160) {
  Pf fence move Work --rect "420,120,300,$h" | Out-Null; Start-Sleep -Milliseconds 900
  $f = FenceGet 'Work'; $r = $f.windowRect; $fit = $f.fit.fittingHeight
  $hit = NcHit (DumpOf 'Work').hwnd ($r.x + 150) ($r.y + $r.h - 2)
  DblClick ($r.x + 150) ($r.y + $r.h - 2); Start-Sleep -Milliseconds 1500
  $r2 = FR 'Work'
  if ($r2.h -eq $fit -and $r2.y -eq $r.y) { $ok++ }
  $rows += "h $h (hit $hit) -> $($r2.h) at y $($r2.y), fitting $fit"
  if ($h -eq 420) { Shot 'u6-dblclick-bottom-edge' }
}
Check 'a double-click on the bottom edge fits the height to the icons (taller and shorter)' ($ok -eq 2) "$($rows -join ' | ')"

Case 'U6 the fence menu: Fit to content'
Pf fence move Work --rect '420,120,300,420' | Out-Null; Start-Sleep -Milliseconds 900
$f = FenceGet 'Work'; $r = $f.windowRect; $fit = $f.fit.fittingHeight
[G]::RClick($r.x + 60, $r.y + 16)
$m = MenuWin 5
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { 'no menu' }
Shot 'u6-fence-menu'
Check 'the fence menu has Fit to content, next to Lock position and size' ($texts -match 'Fit to content \| Lock position and size') $texts
$picked = if ($texts -match 'Fit to content') { MenuPick 'Fit to content' $m } else { CloseMenus; $null }
Start-Sleep -Milliseconds 1500
$r2 = FR 'Work'
Check 'Fit to content fits the height' ($r2.h -eq $fit) "picked '$picked'; h $($r.h) -> $($r2.h), fitting $fit"

Case 'U6 dragging the bottom edge stops at the content height'
$rows = @(); $ok = 0
foreach ($d in 6, -6, 9, 30) {
  Pf fence move Work --rect '420,120,300,420' | Out-Null; Start-Sleep -Milliseconds 900
  $f = FenceGet 'Work'; $r = $f.windowRect; $fit = $f.fit.fittingHeight
  DragTo ($r.x + 150) ($r.y + $r.h - 2) ($r.x + 150) ($r.y + $fit + $d - 2) 20
  $r2 = FR 'Work'
  $want = if ([math]::Abs($d) -le 10) { $fit } else { $fit + $d }
  if ([math]::Abs($r2.h - $want) -le 1) { $ok++ }
  $rows += "released at fitting $(if ($d -ge 0) { '+' })${d}: h $($r2.h) (fitting $fit)"
}
Check 'within 10 px of the content height the edge stops there; 30 px away it does not' ($ok -eq 4) "$ok of 4: $($rows -join ' | ')"

Case 'end'
PfQuit
foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -ErrorAction SilentlyContinue }
