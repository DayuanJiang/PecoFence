# Owner round 2026-11-12, third batch, package ux3-chrome: the display scale changes while
# PecoFence runs (2560x1440 at 200 % and back to 1600x900 at 100 %).
#   - one scale change sets each fence's scale once (it was set three times: new, old, new)
#   - back at 100 % the icons are as sharp as before the change (a finding of the second
#     batch said they stay the 200 % pictures made smaller until a restart: on ec2-1 they
#     were pixel-equal on the reference build, twice)
# Run through tests/free-placement.sh --box <box> --steps <this file> (it answers
# C:\pf\resize-want.txt by reconnecting the RDP session and ends at 1600x900, 100 %).
function FWin($t) { return [G]::FindWindowW('PecoFence.Fence', $t) }
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
# The icons of 文件 as a bitmap (the content area under the title row).
function IconsOf($t) { $r = Rect (FWin $t); return [Px]::Grab($r.L + 6, $r.T + 40, 300, 100) }
function Differing($a, $b) {
  $pa = [Px]::Pixels($a); $pb = [Px]::Pixels($b); $n = 0
  for ($i = 0; $i -lt $pa.Count; $i++) {
    $x = $pa[$i]; $y = $pb[$i]
    $d = [Math]::Abs((($x -shr 16) -band 255) - (($y -shr 16) -band 255)) + [Math]::Abs((($x -shr 8) -band 255) - (($y -shr 8) -band 255)) + [Math]::Abs(($x -band 255) - ($y -band 255))
    if ($d -gt 24) { $n++ }
  }
  return $n
}

trap { PfTrap $_; continue }
Case 'setup'
CloseWindows
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue
GreyWallpaper
$made = @()
foreach ($n in 'ux3 清单.txt', 'ux3 预算表 2026.txt', 'ux3 notes.txt') { $made += (DeskFile $n) }
PfStart -Lang 'zh-CN'
Start-Sleep -Seconds 5
ClearFences | Out-Null
Pf fence set inbox desktopLayout false | Out-Null
Pf fence move inbox --rect '8,560,300,280' | Out-Null
Pf fence create --title 文件 --rect '300,120,340,200' | Out-Null
Pf fence create --title 空的 --rect '700,120,300,200' | Out-Null
Pf item move $made[0] $made[1] $made[2] --to 文件 | Out-Null
Start-Sleep -Seconds 3
ParkMouse 1200 780
$before = IconsOf '文件'
$before.Save("$PfShots\S-icons-0-before.png")
$rectBefore = Rect (FWin '文件')

Case 'S the scale goes to 200 %'
WantScale '2560x1440@200'
# "fence DPI changed fence=<id> from=96 to=192": one line per fence for one scale change.
$lines = @(Get-Content (PfLogPath) -Encoding UTF8 | Select-String -Pattern 'fence DPI changed fence=(\S+) from=(\d+) to=(\d+)')
$per = @{}
foreach ($l in $lines) { $id = $l.Matches[0].Groups[1].Value; $per[$id] = 1 + [int]$per[$id] }
$most = ($per.Values | Measure-Object -Maximum).Maximum
Check 'S each fence took the new scale once' ($per.Count -ge 2 -and $most -eq 1) "$($per.Count) fences, most scale changes for one fence: $most; $(($lines | Select-Object -First 4 | ForEach-Object { $_.Matches[0].Value -replace 'fence=\S+ ', '' }) -join ' | ')"
Shot 'S-at-200'

Case 'S back at 100 %'
WantScale '1600x900'
Start-Sleep -Seconds 4
ParkMouse 1200 780
$r = Rect (FWin '文件')
Check 'S the fence is back where it was' ($r.L -eq $rectBefore.L -and $r.T -eq $rectBefore.T -and $r.R -eq $rectBefore.R) "now $($r.L),$($r.T),$($r.R),$($r.B); before $($rectBefore.L),$($rectBefore.T),$($rectBefore.R),$($rectBefore.B)"
$after = IconsOf '文件'
$after.Save("$PfShots\S-icons-1-back.png")
$n = Differing $before $after
Check 'S the icons are as sharp as before the scale changed' ($n -le 20) "$n of 30000 pixels differ from the picture taken before the change; $((PfLogAll | Select-String -Pattern 'icons extracted' | Select-Object -Last 2 | ForEach-Object { ($_ -replace '^\S+\s+', '') }) -join ' | ')"
Shot 'S-back-100'

Case 'end'
if (PfAlive) { PfQuit }
foreach ($f in $made) { Remove-Item -LiteralPath $f -Force -ErrorAction SilentlyContinue }
Remove-Item C:\pf\resize-want.txt -Force -ErrorAction SilentlyContinue
