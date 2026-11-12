# Owner round 2026-11-12, second batch: small things users asked for. One part per call:
#   --var Part=HS   Explorer's 「单击打开项目」: resting on an icon selects it in whichever fence
#                   is under the pointer, F2 renames it, F2 on a title renames the fence (#184)
#   --var Part=TD   a to-do in a note without typing the box (feedback #141)
#   --var Part=FV   a folder fence remembers view and sort order per folder (feedback #131)
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'HS' }

# How many icons of fence $t are selected, from the next dump line.
function RqSel($t) { return (DumpVal (DumpOf $t 3) 'selected') }
# The pointer glides to a point and rests there longer than Windows' hover time (400 ms).
function RqRest($x, $y) {
  $c = [Windows.Forms.Cursor]::Position
  Glide $c.X $c.Y $x $y 10 20
  Start-Sleep -Milliseconds 1500
}

if ($Part -eq 'HS') {
Case 'HS setup'
$made = @()
foreach ($n in '悬停甲.txt', '悬停乙的长名字.txt', '悬停丙.txt') { $made += DeskFile $n }
$made += "$PfDesk\hover one.txt"
[Th]::SetSingleClick($true)
PfFresh (DumpScript)
ClearFences
Pf fence create --title Alpha --rect '300,120,320,220' | Out-Null
Pf fence create --title Beta --rect '680,120,320,220' | Out-Null
Start-Sleep -Seconds 2
Pf item move "$PfDesk\悬停甲.txt" --to Alpha | Out-Null
Pf item move "$PfDesk\悬停乙的长名字.txt" "$PfDesk\悬停丙.txt" --to Beta | Out-Null
Start-Sleep -Seconds 2
Check 'the test icons are in their fences' ((ItemFence '悬停甲.txt') -eq 'Alpha' -and (ItemFence '悬停丙.txt') -eq 'Beta') "甲=$(ItemFence '悬停甲.txt') 丙=$(ItemFence '悬停丙.txt')"

Case 'HS the desktop has the keyboard'
DeskClick | Out-Null
$p = ItemPt 'Alpha' '悬停甲'
RqRest $p[0] $p[1]
$sel = RqSel 'Alpha'; $fg = Fg
Shot 'hs-1-rest-on-icon'
Check 'resting on an icon selects it while the desktop has the keyboard' ($sel -eq '1') "Alpha selected=$sel foreground=$fg"
Check 'the fence under the pointer has the keyboard then' ($fg -match 'PecoFence\.Fence') "foreground=$fg"
[G]::Press(0x71); $ren = WaitWin 'PecoFence.Rename' 3
if ($ren -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 300; [G]::Type('hover one'); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 2 }
Check 'F2 renames the icon the pointer selected' (Test-Path "$PfDesk\hover one.txt") "name box=$ren"

Case 'HS another fence has the keyboard'
# Alpha has it now (a press on its icon, in case the rest above did not give it).
$p = ItemPt 'Alpha' 'hover one'
if (-not $p) { $p = ItemPt 'Alpha' '悬停甲' }
if ((RqSel 'Alpha') -ne '1') { [G]::KeyDown(0x11); [G]::Click($p[0], $p[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 600 }
$a0 = RqSel 'Alpha'
$q = ItemPt 'Beta' '悬停丙'
RqRest $q[0] $q[1]
$sa = RqSel 'Alpha'; $sb = RqSel 'Beta'
Check 'resting on an icon of another fence selects it and ends the selection in the first' ($a0 -eq '1' -and $sb -eq '1' -and $sa -eq '0') "Alpha selected before=$a0 after=$sa, Beta selected=$sb, foreground=$(Fg)"

Case 'HS F2 on a title'
DeskClick | Out-Null
$b = (PfFence 'Beta').windowRect
RqRest ($b.x + 70) ($b.y + 14)
$fg = Fg
[G]::Press(0x71); $ren = WaitWin 'PecoFence.Rename' 3
if ($ren -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 300; [G]::Type('Gamma'); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 1 }
Shot 'hs-3-title-f2'
Check 'F2 while the pointer rests on a title renames the fence' ([bool](PfFence 'Gamma')) "foreground before F2=$fg name box=$ren titles=$((@(PfJson fence list) | ForEach-Object { $_.title }) -join ',')"

Case 'HS another program in front keeps the keyboard'
DeskClick | Out-Null
Start-Process notepad.exe; Start-Sleep -Seconds 2
$np = Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } | Select-Object -First 1
if ($np) { [L]::MoveWindow($np.MainWindowHandle, 1050, 420, 480, 380, $true) | Out-Null; Start-Sleep -Milliseconds 500 }
$a0 = RqSel 'Alpha'
$p = ItemPt 'Alpha' 'hover one'
if (-not $p) { $p = ItemPt 'Alpha' '悬停甲' }
RqRest $p[0] $p[1]
$sa = RqSel 'Alpha'; $fg = Fg
Check 'resting on an icon selects nothing while another program is in front' ($a0 -eq '0' -and $sa -eq '0' -and $fg -match 'Notepad') "Alpha selected before=$a0 after=$sa foreground=$fg"
CloseWindows

Case 'HS double-click mode: F2 on the title of the fence that has the keyboard'
[Th]::SetSingleClick($false)
Start-Sleep -Seconds 2
DeskClick | Out-Null
$t = if (PfFence 'Gamma') { 'Gamma' } else { 'Beta' }
$b = (PfFence $t).windowRect
[G]::Click($b.x + 70, $b.y + 14); Start-Sleep -Milliseconds 800
[G]::Press(0x71); $ren = WaitWin 'PecoFence.Rename' 3
if ($ren -ne [IntPtr]::Zero) { Start-Sleep -Milliseconds 300; [G]::Type('Delta'); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 1 }
Check 'F2 after a click on the title renames the fence' ([bool](PfFence 'Delta')) "fence=$t name box=$ren titles=$((@(PfJson fence list) | ForEach-Object { $_.title }) -join ',')"

Case 'HS cleanup'
[Th]::SetSingleClick($false)
PfQuit
foreach ($f in $made) { Remove-Item -LiteralPath $f -Force -ErrorAction SilentlyContinue }
}
