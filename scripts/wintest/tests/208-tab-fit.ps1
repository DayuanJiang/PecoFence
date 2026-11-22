# Feedback #208 item 2: a fence switched to a tab with another number of icons takes that tab's
# size and pushes the fences below, as expanding does. Host (tabs Few: 3 items, Many: 12 items)
# with Below 8 px under it; 「展开时把下面的栅栏推开」 on (default). With 「自动高度」 off (the
# window's height belongs to all tabs) and on. Runs on a 1600x900 box at 100 %.
trap { PfTrap $_; continue }

function Park() { ParkMouse 1500 120; Start-Sleep -Milliseconds 400 }
function Hw($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
function Win($t) { return (Rect (Hw $t)) }
function WinH($t) { $r = Win $t; return $r.B - $r.T }
# Clicks tab $i of the window that shows tab $name (strip positions from the test-script dump).
function TabClick($name, $i) {
  $d = DumpOf $name
  if (-not $d) { Log "no dump of $name"; return }
  $x = [int]($d.l + $d.ox + $d.rects[$i][0] + $d.rects[$i][1] / 2); $y = [int]($d.t + $d.oy + 18)
  [G]::Click($x, $y); Start-Sleep -Milliseconds 1500; Park
}
function HostWin() { $d = DumpOf 'Few'; return $d }
function State() { $h = HostWin; $b = Win 'Below'; return "host $($h.t)..$($h.b) ($($h.b - $h.t) px), Below $($b.T)..$($b.B)" }

$names = @()
for ($i = 1; $i -le 15; $i++) { $n = "tf$i.txt"; Set-Content -LiteralPath (Join-Path $PfDesk $n) -Value $n; $names += $n }

Case 'setup'
CloseWindows
PfStart -Script (DumpScript 400 20)
ClearFences | Out-Null
Pf fence create --title Few --rect '400,60,300,200' | Out-Null
Pf item move ($names[0..2] | ForEach-Object { Join-Path $PfDesk $_ }) --to Few | Out-Null
Pf fence create --title Many --rect '800,60,300,200' | Out-Null
Pf item move ($names[3..14] | ForEach-Object { Join-Path $PfDesk $_ }) --to Many | Out-Null
Pf fence merge Many --into Few | Out-Null; Start-Sleep -Milliseconds 800
Pf fence create --title Below --rect '400,268,300,200' | Out-Null
Start-Sleep -Seconds 2; Park
Log "start: $(State); tabs $((DumpOf 'Few').tabs -join ',')"
Shot 'start'

foreach ($auto in 'false', 'true') {
  Case "autoHeight $auto"
  Pf fence set Few autoHeight $auto | Out-Null; Start-Sleep -Milliseconds 1500; Park
  TabClick 'Few' 0
  $few = State; $fewH = (HostWin).b - (HostWin).t
  Shot "auto-$auto-few"
  TabClick 'Few' 1
  $many = State; $d = HostWin; $manyH = $d.b - $d.t; $belowT = (Win 'Below').T
  Shot "auto-$auto-many"
  if ($auto -eq 'false') {
    # The window's height belongs to all tabs.
    Check 'autoHeight false: the height stays with a tab switch' ($manyH -eq $fewH -and $belowT -eq 268) "Few: $few; Many: $many"
  } else {
    # 12 icons in 3 columns: 4 rows of 96 px under the 36 px title row and the 8 px padding.
    Check 'autoHeight true: the tab with more icons shows all of them' ($manyH -ge 36 + 4 * 96) "Few: $few; Many: $many"
    Check 'autoHeight true: the fence below slides out of its way' ($belowT -ge $d.b + 8) "Few: $few; Many: $many"
  }
  TabClick 'Few' 0
  Check "autoHeight $auto`: back on the tab with few icons, the fence below is back at its place" ((Win 'Below').T -eq 268) "after: $(State)"
}
Case 'autoHeight true at the bottom of the screen, nothing to push'
$WorkB = [Windows.Forms.Screen]::PrimaryScreen.WorkingArea.Bottom
Pf fence move Below --rect '1200,60,300,200' | Out-Null
Pf fence move Few --rect "400,$($WorkB - 260),300,200" | Out-Null; Start-Sleep -Milliseconds 1200
TabClick 'Few' 1
$d = HostWin
Shot 'bottom-many'
Check 'the tab with more icons stays inside the work area' ($d.b -le $WorkB -and $d.b - $d.t -ge 200) "host $($d.t)..$($d.b), work area bottom $WorkB"
TabClick 'Few' 0
Pf fence move Few --rect '400,60,300,200' | Out-Null
Pf fence move Below --rect '400,268,300,200' | Out-Null; Start-Sleep -Milliseconds 1200
Pf settings set rollUp.pushNeighbors false | Out-Null; Start-Sleep -Milliseconds 800
Case 'autoHeight true, pushing off'
TabClick 'Few' 1
$d = HostWin
Shot 'push-off-many'
Check 'pushing off: the tab with more icons stops 8 px short of the fence below' ($d.b -eq 260 -and (Win 'Below').T -eq 268) "$(State)"
TabClick 'Few' 0
Pf settings set rollUp.pushNeighbors true | Out-Null
PfQuit
foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue }
