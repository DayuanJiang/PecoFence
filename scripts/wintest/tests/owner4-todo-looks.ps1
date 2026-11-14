# Owner round 2026-11-12, the to-do drag, second round: the drag looked at once in dark mode
# with Liquid Glass, in a rolled fence that the pointer expanded (the drag must not roll it
# up), and at 150 % scale. Pictures in shots\ (l1-*, l2-*, l3-*); the checks are the outcomes.
# Run through tests/free-placement.sh --box <box> --steps <this file> (it answers
# C:\pf\resize-want.txt by reconnecting the RDP session and ends at 1600x900, 100 %).
trap { PfTrap $_; continue }
Case 'setup'
CloseWindows
Set-Culture zh-CN
$Anim0 = [Anim]::Get()
[Anim]::Set($true)
GreyWallpaper
PfStart -Lang 'zh-CN' -Script (DumpScript 500 25 3000)
Start-Sleep -Seconds 3
ClearPfNotices
ClearFences
Pf settings set featureTips false | Out-Null
function DisplayChanges() { return @(PfLogAll | Select-String -Pattern 'display changed monitors=').Count }
# Asks for the scale and waits until PecoFence has placed its fences for it.
function WantScale($want) {
  $n = DisplayChanges
  Set-Content C:\pf\resize-want.txt $want -Encoding ASCII
  for ($k = 0; $k -lt 150; $k++) {
    Start-Sleep -Seconds 1
    if (-not (Test-Path C:\pf\resize-want.txt) -and (DisplayChanges) -gt $n) { break }
  }
  Start-Sleep -Seconds 6
  Log "$want asked; display changes seen: $(DisplayChanges)"
}
function Rects($w, $name) {
  $o = @{}; $v = DumpVal $w $name
  if ($v -and $v -ne '-') { foreach ($p in ($v -split ';')) { $kv = $p -split ':'; if (-not $o.ContainsKey([int]$kv[0])) { $o[[int]$kv[0]] = @($kv[1] -split ',' | ForEach-Object { [double]$_ }) } } }
  return $o
}
function FCrop($name, $w, $m = 16) { Crop $name ([math]::Max(0, $w.l - $m)) ([math]::Max(0, $w.t - $m)) ($w.r - $w.l + 2 * $m) ($w.b - $w.t + 2 * $m) | Out-Null }
function NoteText($id) { return "$((FenceGet $id).noteText)" }
function OneLine($t) { return ("$t" -replace "`n", ' / ') }
function Lifts() { return @(PfLogAll | Where-Object { $_ -match 'to-do drag starts' }).Count }
# A point on the text of the to-do on line $line of dumped note $w. Its boxes are in DIPs;
# the title row is 36 DIPs high, so the content's top in px gives the display scale.
function TodoPt($w, $line) {
  $b = (Rects $w 'note_boxes')[$line]
  $cx = [int](DumpVal $w 'content_x'); $cy = [int](DumpVal $w 'content_y')
  $k = $cy / 36.0
  return @(($w.l + $cx + [int](($b[2] + 40) * $k)), ($w.t + $cy + [int]((($b[1] + $b[3]) / 2) * $k)))
}
# Drags the to-do on line $fromLine above the one on line $toLine and holds it there; returns
# the point it is held at.
function HoldAbove($w, $fromLine, $toLine) {
  $from = TodoPt $w $fromLine; $to = TodoPt $w $toLine
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  Glide $from[0] $from[1] ($from[0] + 6) ($to[1] - 5) 14 30
  Start-Sleep -Milliseconds 500
}
$Long = '把上周的周报和三张报销单一起发给财务，抄送组长'
$T0 = "周会后要做`n[ ] 买菜`n[x] 回邮件`n[ ] $Long`n备注：周五前"
$Moved = "周会后要做`n[ ] $Long`n[ ] 买菜`n[x] 回邮件`n备注：周五前"

Case 'L1 dark mode with Liquid Glass'
Pf settings set theme dark | Out-Null
Pf settings set customTheme null | Out-Null
Pf settings set themeStyle liquidGlass | Out-Null
$nid = "$((PfJson fence create --tool note --title 便签 --text $T0 --rect '100,100,300,240').fence.id)"
ParkMouse 900 700
Start-Sleep -Seconds 2
$w = DumpOf '便签'
HoldAbove $w 3 1
FCrop 'l1-dark-glass-dragging' $w
MUp; Start-Sleep -Milliseconds 900
FCrop 'l1-dark-glass-dropped' $w
$after = NoteText $nid
Check 'dark mode, Liquid Glass: the third to-do lands above the first' ($after -eq $Moved) "note: $(OneLine $after)"

Case 'L2 a rolled fence that the pointer expanded is not rolled up by the drag'
Pf fence set $nid noteText $T0 --string | Out-Null
Pf fence roll $nid | Out-Null
ParkMouse 900 700
Start-Sleep -Seconds 2
$rolled0 = "$((FenceGet $nid).rolledUp)"
$w = DumpOf '便签'
[G]::MoveTo($w.l + 120, $w.t + 14); Start-Sleep -Milliseconds 1500
$w = DumpOf '便签'
$peek0 = DumpVal $w 'peeking'
$lift0 = Lifts
HoldAbove $w 3 1
Start-Sleep -Milliseconds 1200
$wHeld = DumpOf '便签'
$peekHeld = DumpVal $wHeld 'peeking'; $hHeld = $wHeld.b - $wHeld.t
FCrop 'l2-expanded-dragging' $w
# Held under the fence's bottom edge for longer than a rolled fence waits to roll up again.
$p = TodoPt $w 1
Glide ($p[0] + 6) ($p[1] - 5) ($p[0] + 6) ($w.b + 14) 10 30
Start-Sleep -Milliseconds 1200
$wOut = DumpOf '便签'
$hOut = $wOut.b - $wOut.t
Glide ($p[0] + 6) ($w.b + 14) ($p[0] + 6) ($p[1] - 5) 10 30
Start-Sleep -Milliseconds 400
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
$lift1 = Lifts
FCrop 'l2-expanded-dropped' $w
ParkMouse 900 700
Start-Sleep -Seconds 2
$hAway = (DumpOf '便签'); $hAway = $hAway.b - $hAway.t
Check 'the rolled note expands under the pointer and a to-do in it can be lifted' ($rolled0 -eq 'True' -and $peek0 -eq 'true' -and $lift1 -eq $lift0 + 1) "rolled: $rolled0; expanded by the pointer: $peek0; lifted (log): $lift0 -> $lift1"
Check 'it stays expanded while the to-do is held, also with the pointer just under the fence' ($peekHeld -eq 'true' -and $hHeld -gt 150 -and $hOut -gt 150) "expanded while held: $peekHeld, height $hHeld; height with the pointer 14 px under the fence for 1.2 s: $hOut"
Check 'the drop lands, and the fence rolls up again once the pointer has left' ($after -eq $Moved -and $hAway -lt 80) "note: $(OneLine $after); height after the pointer left: $hAway"
Pf fence unroll $nid | Out-Null
Start-Sleep -Seconds 1

Case 'L3 at 150 % scale, light and dark'
Pf fence set $nid noteText $T0 --string | Out-Null
Pf settings set theme light | Out-Null
Pf settings set themeStyle fluent | Out-Null
WantScale '2400x1350@150'
ParkMouse 1500 1000
Start-Sleep -Seconds 2
$w = DumpOf '便签'
$scale = [int](DumpVal $w 'content_y') / 36.0
HoldAbove $w 3 1
FCrop 'l3-150-light-dragging' $w
MUp; Start-Sleep -Milliseconds 900
FCrop 'l3-150-light-dropped' $w
$after = NoteText $nid
Check 'at 150 % the third to-do lands above the first' ($scale -eq 1.5 -and $after -eq $Moved) "scale: $scale; note: $(OneLine $after)"
Pf fence set $nid noteText $T0 --string | Out-Null
Pf settings set theme dark | Out-Null
Pf settings set themeStyle liquidGlass | Out-Null
ParkMouse 1500 1000
Start-Sleep -Seconds 2
$w = DumpOf '便签'
HoldAbove $w 3 1
FCrop 'l3-150-dark-glass-dragging' $w
MUp; Start-Sleep -Milliseconds 900
$after = NoteText $nid
Check 'at 150 % in dark mode with Liquid Glass too' ($after -eq $Moved) "note: $(OneLine $after)"

Case 'cleanup'
Pf fence delete $nid | Out-Null
Pf settings set theme light | Out-Null
Pf settings set themeStyle fluent | Out-Null
WantScale '1600x900'
[Anim]::Set($Anim0)
