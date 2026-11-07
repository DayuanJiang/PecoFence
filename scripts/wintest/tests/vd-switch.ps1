# 在哪个虚拟桌面显示, switching: the fence menu, the CLI and the fade on Win+Ctrl+Left/Right and
# Task View clicks; rolled up fences, tab groups, quick hide with a fence kept on the desktop,
# Peek and the Peek search across desktops. Fences are found by title whether shown or hidden
# (vdesk.ps1: FVis); outcomes come from `fence get` and the log, screenshots are for reports.

Case 'setup'
CloseWindows
GreyWallpaper
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
VdTwo 'Work' 'Games'
$ids = @(VdIds)
Check 'two desktops named Work and Games' ($ids.Count -eq 2 -and (VdName $ids[0]) -eq 'Work' -and (VdName $ids[1]) -eq 'Games' -and (VdCurrent) -eq $ids[0]) (VdState)
$made = 'work-plan.txt', 'work-notes.txt', 'game-save.txt', 'game-mods.txt'
foreach ($n in $made) { Set-Content "$PfDesk\$n" 'x' }
PfStart
Pf settings set theme light | Out-Null
Pf fence create --title Work --rect '80,80,420,240' | Out-Null
Pf fence create --title Games --rect '600,80,420,240' | Out-Null
Pf item move "$PfDesk\work-plan.txt" --to Work | Out-Null
Pf item move "$PfDesk\work-notes.txt" --to Work | Out-Null
Pf item move "$PfDesk\game-save.txt" --to Games | Out-Null
Pf item move "$PfDesk\game-mods.txt" --to Games | Out-Null
[G]::MoveTo(1200, 700)
Start-Sleep -Seconds 2
$all = 'Work', 'Games'

Case 'CLI lists the desktops'
$l = @(PfJson virtual-desktop list)
Check 'virtual-desktop list: Task View order, names, current' ($l.Count -eq 2 -and $l[0].name -eq 'Work' -and $l[1].name -eq 'Games' -and $l[0].current -and -not $l[1].current -and $l[1].id -eq "{$($ids[1])}" -and $l[1].number -eq 2) (($l | ConvertTo-Json -Compress))
$g = FenceGet Games
Check 'a new fence shows on every desktop' (@($g.virtualDesktops).Count -eq 0 -and $g.onCurrentDesktop) "vd=$($g.virtualDesktops) on=$($g.onCurrentDesktop)"
Check 'no poll while no fence is bound' (-not (PfLogAll | Where-Object { $_ -match 'virtual desktop poll' })) ''

Case 'bind by CLI on desktop 1'
Pf fence set Games virtualDesktops Games | Out-Null
Start-Sleep -Milliseconds 900
$g = FenceGet Games
Check 'fence set Games virtualDesktops Games binds it to desktop 2' (@($g.virtualDesktops).Count -eq 1 -and $g.virtualDesktops[0] -eq "{$($ids[1])}" -and -not $g.onCurrentDesktop) "vd=$($g.virtualDesktops) on=$($g.onCurrentDesktop)"
Check 'Games fades out here, Work stays' ((FVis Games) -eq $false -and (FVis Work) -eq $true) (VisText $all)
Check 'the poll started' ([bool](PfLogAll | Where-Object { $_ -match 'virtual desktop poll' -and $_ -match 'on=true' })) ''
Shot 'after-d1-games-bound'
$bad = Pf fence set Games virtualDesktops 7
Check 'an unknown desktop number is refused' ($bad -match 'no virtual desktop' -and $bad -match 'virtual-desktop list') ($bad -replace '\s+', ' ')
$bad = Pf fence set inbox virtualDesktops 2
Check 'the inbox is refused' ($bad -match 'every virtual desktop') ($bad -replace '\s+', ' ')

Case 'switch with Win+Ctrl+Right'
VdRight
Check 'desktop 2: Games and Work show' ((FVis Games) -eq $true -and (FVis Work) -eq $true) (VisText $all)
$sw = @(PfLogAll | Where-Object { $_ -match 'virtual desktop switched from=' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'the switch is noticed' ($sw -match "$($ids[1])") $sw
$fade = @(PfLogAll | Where-Object { $_ -match 'fade finished' -and $_ -match 'title=Games' }) | Select-Object -Last 1
Check 'Games fades in with the quick hide fade' ($fade -match 'Showing') $fade
Shot 'after-d2-both'

Case 'bind by menu on desktop 2'
$r = Rect ([VD]::Fence('Work'))
[G]::RClick($r.L + 140, $r.T + 14); Start-Sleep -Milliseconds 900
$m = MenuWin
$texts = MenuTexts $m
Check 'the fence menu offers Show on virtual desktop' ($texts -match 'Show on virtual desktop') $texts
MenuPick 'Show on virtual desktop' $m | Out-Null; Start-Sleep -Milliseconds 700
$sub = MenuWin
$st = MenuTexts $sub
$rows = @(MenuRows $sub)
$allRow = $rows | Where-Object { $_.text -eq 'All virtual desktops' }
Check 'submenu: All virtual desktops (checked), Work, Games (current)' ($st -match 'Work' -and $st -match 'Games \(current\)' -and $allRow -and ($allRow.state -band 8)) "$st; all state=$($allRow.state)"
Shot 'after-menu-submenu'
MenuPick '^Work$' $sub | Out-Null
Start-Sleep -Seconds 1
$w = FenceGet Work
Check 'menu: Work now only on desktop Work, gone from desktop 2' (@($w.virtualDesktops).Count -eq 1 -and $w.virtualDesktops[0] -eq "{$($ids[0])}" -and (FVis Work) -eq $false -and (FVis Games) -eq $true) ("vd=$($w.virtualDesktops) " + (VisText $all))
Shot 'after-d2-work-gone'
VdLeft
Check 'desktop 1: Work shows, Games hidden' ((FVis Work) -eq $true -and (FVis Games) -eq $false) (VisText $all)
Shot 'after-d1-work-only'
# The menu again, on desktop 1, for the window bound to it: its own desktop checked.
$r = Rect ([VD]::Fence('Work'))
[G]::RClick($r.L + 140, $r.T + 14); Start-Sleep -Milliseconds 900
$m = MenuWin
MenuPick 'Show on virtual desktop' $m | Out-Null; Start-Sleep -Milliseconds 700
$sub = MenuWin
$rows = @(MenuRows $sub)
$d1 = $rows | Where-Object { $_.text -eq 'Work (current)' }
$allRow = $rows | Where-Object { $_.text -eq 'All virtual desktops' }
Check 'desktop 1 menu: Work (current) checked, All not' ($d1 -and ($d1.state -band 8) -and -not ($allRow.state -band 8)) (MenuTexts $sub)
CloseAllMenus

Case 'switch back and forth three times'
for ($k = 0; $k -lt 3; $k++) {
  VdRight
  $a = VisText $all
  VdLeft
  $b = VisText $all
  Check "round ${k}: d2 shows Games only, d1 Work only" ($a -eq 'Work=False Games=True' -and $b -eq 'Work=True Games=False') "d2: $a; d1: $b"
}

Case 'switch by a click in Task View'
TaskViewPick 2
$a = VisText $all
TaskViewPick 1
$b = VisText $all
Check 'Task View clicks: d2 shows Games only, d1 Work only' ($a -eq 'Work=False Games=True' -and $b -eq 'Work=True Games=False') "d2: $a; d1: $b; $(VdState)"

Case 'rolled up'
Pf fence roll Games | Out-Null
Start-Sleep -Milliseconds 600
VdRight
$g = FenceGet Games
$gr = Rect ([VD]::Fence('Games'))
Check 'a fence rolled up on desktop 1 shows rolled up on desktop 2' ((FVis Games) -eq $true -and $g.rolledUp -and ($gr.B - $gr.T) -lt 80) "rolled=$($g.rolledUp) h=$($gr.B - $gr.T)"
Pf fence unroll Games | Out-Null
Start-Sleep -Milliseconds 600

Case 'tab group'
$gh = [VD]::Fence('Games')
function GVis() { return [VD]::Visible($gh) }
Pf fence create --title Mods --rect '600,420,300,160' | Out-Null
Start-Sleep -Milliseconds 600
Pf fence merge Mods --into Games | Out-Null
Start-Sleep -Milliseconds 800
$md = FenceGet Mods
Check 'a fence merged into Games takes its desktops' (@($md.virtualDesktops).Count -eq 1 -and $md.virtualDesktops[0] -eq "{$($ids[1])}") "vd=$($md.virtualDesktops)"
VdLeft
Check 'desktop 1: the tab window stays hidden' ((GVis) -eq $false) "window $gh title '$([W]::Title($gh))' visible=$(GVis)"
Pf fence detach Mods | Out-Null
Start-Sleep -Seconds 1
Check 'detached on desktop 1, Mods keeps desktop 2 and stays hidden' ((FVis Mods) -eq $false -and (FenceGet Mods).virtualDesktops[0] -eq "{$($ids[1])}") (VisText ($all + 'Mods'))
VdRight
Check 'desktop 2: Mods shows on its own' ((FVis Mods) -eq $true -and (FVis Games) -eq $true) (VisText ($all + 'Mods'))

Case 'quick hide with an excluded fence'
Pf fence set Mods excludeFromQuickHide true | Out-Null
[G]::MoveTo(1100, 700); Start-Sleep -Milliseconds 300
DoubleClick 1100 700; Start-Sleep -Seconds 1
Check 'desktop 2 quick hide: Games hides, excluded Mods stays' ((FVis Games) -eq $false -and (FVis Mods) -eq $true) (VisText ($all + 'Mods'))
VdLeft
Check 'desktop 1 while quick hidden: Work stays hidden, Mods hidden' ((FVis Work) -eq $false -and (FVis Mods) -eq $false) (VisText ($all + 'Mods'))
# Right after a switch a double-click on the desktop does not reach PecoFence (main too, see
# probe3): try up to three times and say how many it took.
$tries = 0
for ($k = 0; $k -lt 3 -and (FVis Work) -ne $true; $k++) {
  Log "before double-click $($k + 1): fg=$(Fg) at=$([W]::RootClassAt(1100, 700))"
  DoubleClick 1100 700; Start-Sleep -Seconds 1; $tries++
}
Log "quick show took $tries double-clicks"
Check 'quick show on desktop 1: Work only' ((FVis Work) -eq $true -and (FVis Games) -eq $false -and (FVis Mods) -eq $false) (VisText ($all + 'Mods'))
VdRight
Check 'desktop 2 after the show: Games and Mods' ((FVis Games) -eq $true -and (FVis Mods) -eq $true -and (FVis Work) -eq $false) (VisText ($all + 'Mods'))
VdLeft

Case 'Peek on desktop 1'
Start-Process notepad; Start-Sleep -Seconds 2
PeekKey; Start-Sleep -Seconds 1
Check 'Peek floats Work, Games and Mods stay hidden' ((FVis Work) -eq $true -and (FVis Games) -eq $false -and (FVis Mods) -eq $false) (VisText ($all + 'Mods'))
Shot 'after-d1-peek'
[G]::Press(0x1B); Start-Sleep -Seconds 1
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force; Start-Sleep -Milliseconds 500

Case 'Peek search finds a file of a hidden fence'
[G]::Click(1500, 820); Start-Sleep -Milliseconds 400
PeekKey; Start-Sleep -Milliseconds 800
[G]::Type('game-save'); Start-Sleep -Milliseconds 900
Shot 'after-d1-search'
[G]::Press(0x0D); Start-Sleep -Seconds 3
$t = NotepadTitles
Check 'Enter opens game-save.txt from Games (on desktop 2)' ($t -like '*game-save*') "notepad=$t"
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }

Case 'cleanup'
PfQuit
VdOnlyOne
foreach ($n in $made) { Remove-Item "$PfDesk\$n" -ErrorAction SilentlyContinue }
[Anim]::Set($AnimWas)
