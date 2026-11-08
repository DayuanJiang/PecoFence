# 在哪个虚拟桌面显示, the desktops themselves: a restart on desktop 2, two desktops' fences on one
# spot, the inbox, a desktop closed while PecoFence runs and while it does not, a whole new set
# of desktops (another PC: matched by name) after a restart and by `config import`, the poll
# alone (the Store edition gets no registry notifications), unbinding stopping the poll.

Case 'setup'
CloseWindows
GreyWallpaper
VdTwo 'Work' 'Games'
$ids = @(VdIds)
Check 'two desktops named Work and Games' ($ids.Count -eq 2 -and (VdName $ids[0]) -eq 'Work' -and (VdName $ids[1]) -eq 'Games') (VdState)
$made = 'work-plan.txt', 'game-save.txt'
foreach ($n in $made) { Set-Content "$PfDesk\$n" 'x' }
PfStart
Pf settings set theme light | Out-Null
Pf fence create --title Work --rect '80,80,420,240' | Out-Null
Pf fence create --title Games --rect '80,80,420,240' | Out-Null
Pf item move "$PfDesk\work-plan.txt" --to Work | Out-Null
Pf item move "$PfDesk\game-save.txt" --to Games | Out-Null
Pf fence set Work virtualDesktops 1 | Out-Null
Pf fence set Games virtualDesktops 2 | Out-Null
# Both on one spot: each desktop has its own fence there.
Pf fence move Games --x 80 --y 80 | Out-Null
[G]::MoveTo(1200, 700)
Start-Sleep -Seconds 1
$all = 'Work', 'Games'
$wr = Rect ([VD]::Fence('Work')); $gr = Rect ([VD]::Fence('Games'))
Check 'Work and Games may share a spot on different desktops' ($wr.L -eq $gr.L -and $wr.T -eq $gr.T) "work=$($wr.L),$($wr.T) games=$($gr.L),$($gr.T)"

Case 'restart while on desktop 2'
VdRight
PfQuit
PfStart
[G]::MoveTo(1200, 700)
Start-Sleep -Seconds 2
Check 'started on desktop 2: Games shows, Work stays hidden' ((FVis Games) -eq $true -and (FVis Work) -eq $false) (VisText $all)
$gr2 = Rect ([VD]::Fence('Games'))
Check 'Games kept its spot over the hidden Work' ($gr2.L -eq $gr.L -and $gr2.T -eq $gr.T) "games=$($gr2.L),$($gr2.T)"
Shot 'after-restart-d2'
VdLeft
Check 'then desktop 1: Work shows, Games hidden' ((FVis Work) -eq $true -and (FVis Games) -eq $false) (VisText $all)

Case 'the inbox stays on every desktop'
$inbox = PfFence 'Desktop'
$ih = [VD]::Fence('Desktop')
$r = Rect $ih
[G]::RClick($r.L + 120, $r.T + 14); Start-Sleep -Milliseconds 900
$m = MenuWin
$texts = MenuTexts $m
Check 'the inbox menu has no Show on virtual desktop' (-not ($texts -match 'virtual desktop')) $texts
CloseAllMenus
VdRight
Check 'the inbox shows on desktop 2' ([VD]::Visible($ih)) "inbox=$ih"
VdLeft

Case 'a desktop closed while PecoFence runs'
VdRight
VdNew
$ids3 = @(VdIds)
Pf fence create --title Temp --rect '900,420,300,160' | Out-Null
Pf fence set Temp virtualDesktops current | Out-Null
Start-Sleep -Milliseconds 600
$l = @(PfJson virtual-desktop list)
Check 'a desktop never renamed is listed as Desktop 3' ($l.Count -eq 3 -and $l[2].name -eq 'Desktop 3' -and $l[2].current) (($l | ForEach-Object { "$($_.number)=$($_.name)" }) -join ' ')
$t = FenceGet Temp
Check 'Temp bound to desktop 3' (@($t.virtualDesktops).Count -eq 1 -and $t.virtualDesktops[0] -eq "{$($ids3[2])}" -and (FVis Temp) -eq $true) "vd=$($t.virtualDesktops) ids=$($ids3 -join ',')"
VdClose
Start-Sleep -Milliseconds 600
$t = FenceGet Temp
Check 'closed desktop 3: Windows is on Games, Temp moved there with it' ((VdCurrent) -eq $ids[1] -and $t.virtualDesktops[0] -eq "{$($ids[1])}" -and (FVis Temp) -eq $true) ("vd=$($t.virtualDesktops) now=" + (VdState))
$moved = @(PfLogAll | Where-Object { $_ -match 'fence of a virtual desktop that is gone' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'the log says where it went and why' ($moved -match "$($ids[1])" -and $moved -match 'Neighbour') $moved

Case 'a desktop closed while PecoFence is not running'
VdNew
Pf fence set Temp virtualDesktops current | Out-Null
Start-Sleep -Milliseconds 600
PfQuit
VdClose
PfStart
[G]::MoveTo(1200, 700)
Start-Sleep -Seconds 2
$t = FenceGet Temp
Check 'its fence went to the left neighbour, as its windows did (the order was saved)' (@($t.virtualDesktops).Count -eq 1 -and $t.virtualDesktops[0] -eq "{$($ids[1])}" -and (VdCurrent) -eq $ids[1] -and (FVis Temp) -eq $true) ("vd=$($t.virtualDesktops) " + (VdState))
VdLeft
Check 'Temp is not on Work' ((FVis Temp) -eq $false) (VisText ($all + 'Temp'))

Case 'a new set of desktops after a restart: matched by name'
Pf config export "$PfRun\vd-export.json" | Out-Null
Pf fence set Temp virtualDesktops all | Out-Null
PfQuit
# [Work, Games] -> [Games, Work] with new ids: two new desktops named the other way round, the
# old two closed.
VdNew; VdNew
VdRename 3 'Games'; VdRename 4 'Work'
VdLeft; VdLeft; VdLeft
VdClose; VdClose
$new = @(VdIds)
Log ("new set: " + (VdState))
PfStart
[G]::MoveTo(1200, 700)
Start-Sleep -Seconds 2
$w = FenceGet Work; $g = FenceGet Games
Check 'Work and Games follow their names into the new set' ($new.Count -eq 2 -and $w.virtualDesktops[0] -eq "{$($new[1])}" -and $g.virtualDesktops[0] -eq "{$($new[0])}") ("work=$($w.virtualDesktops) games=$($g.virtualDesktops) " + (VdState))
$why = @(PfLogAll | Where-Object { $_ -match 'fence of a virtual desktop that is gone' -and $_ -match 'SameName' -and $_ -notmatch 'omitted' }).Count
Check 'the log says they were matched by name' ($why -ge 2) "lines=$why"
Check 'on the new Games desktop (first): Games shows, Work hidden' ((VdCurrent) -eq $new[0] -and (FVis Games) -eq $true -and (FVis Work) -eq $false) (VisText $all)

Case 'config import from another PC: matched by name'
Pf fence set Games virtualDesktops all | Out-Null
Pf fence set Work virtualDesktops all | Out-Null
$imp = Pf config import "$PfRun\vd-export.json"
Start-Sleep -Seconds 2
$w = FenceGet Work; $g = FenceGet Games
Check 'the imported fences of the old desktops land on the same names' ($w.virtualDesktops[0] -eq "{$($new[1])}" -and $g.virtualDesktops[0] -eq "{$($new[0])}") ("work=$($w.virtualDesktops) games=$($g.virtualDesktops) import=" + (($imp -replace '\s+', ' ') -replace '^(.{0,120}).*$', '$1'))
Check 'and show where they belong' ((FVis Games) -eq $true -and (FVis Work) -eq $false) (VisText $all)

Case 'the poll alone (as in the Store edition)'
PfQuit
PfStart -Script @('vd-poll-only', 'sleep 3600000')
[G]::MoveTo(1200, 700)
Start-Sleep -Seconds 2
Check 'started with the poll only' ([bool](PfWaitLog 'only the poll is followed' 5)) ''
$sw = [Diagnostics.Stopwatch]::StartNew()
[G]::KeyDown(0x5B); [G]::KeyDown(0x11); [G]::Press(0x27); [G]::KeyUp(0x11); [G]::KeyUp(0x5B)
while ((FVis Games) -ne $false -and $sw.ElapsedMilliseconds -lt 3000) { Start-Sleep -Milliseconds 10 }
$ms = $sw.ElapsedMilliseconds
Start-Sleep -Seconds 2
$line = @(PfLogAll | Where-Object { $_ -match 'virtual desktop switched from=' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'the poll alone follows the switch' ((FVis Games) -eq $false -and (FVis Work) -eq $true -and $line -match 'source="?poll') "after ${ms} ms; $line"
Check 'within a second' ($ms -lt 1000) "${ms} ms"

Case 'unbinding everything stops the poll'
Pf fence set Work virtualDesktops all | Out-Null
Pf fence set Games virtualDesktops all | Out-Null
Pf fence set Temp virtualDesktops all | Out-Null
Start-Sleep -Milliseconds 800
Check 'all on every desktop again: they show' ((FVis Work) -eq $true -and (FVis Games) -eq $true -and (FVis Temp) -eq $true) (VisText ($all + 'Temp'))
$poll = @(PfLogAll | Where-Object { $_ -match 'virtual desktop poll' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'the poll stopped' ($poll -match 'on=false') $poll

Case 'cleanup'
PfQuit
VdOnlyOne
ClearPfNotices
foreach ($n in $made) { Remove-Item "$PfDesk\$n" -ErrorAction SilentlyContinue }
