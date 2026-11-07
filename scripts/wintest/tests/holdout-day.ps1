# Hold-out test: installing the build under test and a normal user's day with it, with the real
# mouse and keyboard. Needs --extra with old-setup.exe (released 0.2.5), new-setup.exe and
# portable.zip (the build under test). Runs the installed edition (no test instance), so the
# kit's Pf/PfStart/PfLogErrors are not used: the helpers below talk to the installed copy.

# (The helpers this test needed are in the kit since 2026-11-06.)

# ---- steps
PfUseUser

Case 'setup'
CloseWindows
# Installers or uninstallers an aborted earlier job left open (ctl reset does not end them).
Get-Process | Where-Object { $_.ProcessName -match 'setup|^_iu|^unins' } | Stop-Process -Force -ErrorAction SilentlyContinue
# A browser an earlier job opened (the notice's Manual, an EC2 .url) covers the desktop.
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
$prog = "$env:LOCALAPPDATA\Programs\PecoFence"
$exe = "$prog\pecofence.exe"
$lnk = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PecoFence\PecoFence.lnk"
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$script:CliExe = "$prog\pecofence-cli.exe"
if (Test-Path "$prog\unins000.exe") { SilentUninstall "$prog\unins000.exe" }
Remove-Item -Recurse -Force "$env:APPDATA\PecoFence", "$env:APPDATA\OpenFence", "$env:LOCALAPPDATA\PecoFence" -ErrorAction SilentlyContinue
Remove-ItemProperty $runKey -Name PecoFence -ErrorAction SilentlyContinue
$testFiles = @('Alpha Folder', 'Beta notes.txt', 'Beta notes - Copy.txt', 'Gamma link.lnk', 'Gamma renamed.lnk', 'Report 2026.pdf', 'Delta temp.txt', 'Epsilon temp.txt', 'Zeta drag.txt', 'Eta drag.txt')
foreach ($n in $testFiles) { Remove-Item -LiteralPath "$PfDesk\$n" -Recurse -Force -ErrorAction SilentlyContinue }
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
# This PC on the desktop (Desktop icon settings), so it shows in the Desktop fence.
$hdi = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\HideDesktopIcons\NewStartPanel'
if (-not (Test-Path $hdi)) { New-Item $hdi -Force | Out-Null }
Set-ItemProperty $hdi -Name '{20D04FE0-3AEA-1069-A2D8-08002B30309D}' -Value 0 -Type DWord
New-Item -ItemType Directory -Force "$PfDesk\Alpha Folder" | Out-Null
Set-Content "$PfDesk\Alpha Folder\inside.txt" 'inside'
foreach ($n in 'Beta notes.txt', 'Delta temp.txt', 'Epsilon temp.txt', 'Zeta drag.txt', 'Eta drag.txt') { Set-Content "$PfDesk\$n" $n }
Set-Content "$PfDesk\Report 2026.pdf" '%PDF-1.4 test'
$sc = (New-Object -ComObject WScript.Shell).CreateShortcut("$PfDesk\Gamma link.lnk"); $sc.TargetPath = "$env:WINDIR\notepad.exe"; $sc.Save()
$bare = [W]::RootClassAt(1300, 650)
Check 'the setups, the zip and the desktop items are there, the desktop is bare' ((Test-Path "$PfExtra\old-setup.exe") -and (Test-Path "$PfExtra\new-setup.exe") -and (Test-Path "$PfExtra\portable.zip") -and (Test-Path "$PfDesk\Gamma link.lnk") -and -not (Test-Path $exe) -and $bare -match '^(Progman|WorkerW)$') "icons shown=$([W]::IconsVisible()) at 1300,650: $bare"

Case 'D1 install 0.2.5 and use it'
$code = SilentSetup "$PfExtra\old-setup.exe" 'setup-old.log'
Check 'the 0.2.5 setup succeeds' ($code -eq 0 -and (Test-Path $exe)) "exit=$code"
$oldHash = (Get-FileHash $exe).Hash
$h = StartUser $exe
$st = IpfJson status
Check '0.2.5 runs' ($h -ne [IntPtr]::Zero -and $st.appVersion) "version=$($st.appVersion) config=$($st.configPath)"
$fs = @(IpfJson fence list)
Check 'its first run makes fences' ($fs.Count -ge 3) (($fs | ForEach-Object { $_.title }) -join ', ')
$other = $fs | Where-Object { $_.kind -ne 'inbox' } | Select-Object -First 1
Ipf fence rename $other.id 'Renamed in 0.2.5' | Out-Null
Ipf fence create --title Work --rect '700,300,340,250' | Out-Null
Ipf fence set Work tint '#3366ff' | Out-Null
Ipf rule add --name 'PDF to Work' --ext pdf --to Work --index 0 | Out-Null
Ipf rule apply | Out-Null
Ipf item move "$PfDesk\Beta notes.txt" "$PfDesk\Alpha Folder" --to Work | Out-Null
Ipf settings set theme dark | Out-Null
Start-Sleep -Seconds 2
$before = StateText
Log ("state before the upgrade:`n" + ($before -join "`n"))
Check 'the fence, the rule and the moved icons took' ((@($before -match '^fence Work ').Count -eq 1) -and (@($before -match '^item Report 2026 in Work$').Count -eq 1) -and (@($before -match '^item Alpha Folder in Work$').Count -eq 1) -and (@($before -match '^rule PDF to Work -> Work').Count -eq 1)) (($before -match 'Work') -join ' ; ')
Shot 'd1-0.2.5'

Case 'D1 upgrade with the new setup while 0.2.5 runs'
CloseNotice | Out-Null
$script:quitBy = $null; $script:quitGone = $false
$w = Wizard "$PfExtra\new-setup.exe" 'setup-upgrade.log' { $script:quitBy = TrayPick '^Exit PecoFence'; $script:quitGone = WaitNoPf 15 }
Check 'the wizard runs to the end' ($w.exited -and $w.code -eq 0) "exit=$($w.code) pages: $($w.seen)"
Check 'it asks to close PecoFence, which quits from its tray menu' (($w.seen -match 'app running prompt') -and $quitBy -and $quitGone) "tray pick='$quitBy' gone=$quitGone menu: $LastTrayMenu"
$h = WaitWin 'PecoFence.Control' 30 -Hidden
Start-Sleep -Seconds 5
$newHash = (Get-FileHash $exe).Hash
$tools = Ipf settings get tools
Check 'Finish starts the new build' ($h -ne [IntPtr]::Zero -and $newHash -ne $oldHash -and $tools -match 'dialogJump') "hash $($oldHash.Substring(0, 8)) -> $($newHash.Substring(0, 8)); tools=$($tools.Trim())"
$after = StateText
$diff = @(Compare-Object $before $after | ForEach-Object { "$($_.SideIndicator) $($_.InputObject)" })
Check 'fences, rules, icons and settings are the same after the upgrade' ($diff.Count -eq 0) ($diff -join ' ; ')
Shot 'd1-upgraded'

Case 'D2 fresh install on a clean profile'
PfQuit
SilentUninstall "$prog\unins000.exe"
Remove-Item -Recurse -Force "$env:APPDATA\PecoFence", "$env:APPDATA\OpenFence", "$env:LOCALAPPDATA\PecoFence" -ErrorAction SilentlyContinue
if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
Check 'a clean profile' (-not (Test-Path $exe) -and -not (Test-Path "$env:APPDATA\PecoFence"))
$w = Wizard "$PfExtra\new-setup.exe" 'setup-fresh.log' { Log 'unexpected running prompt' }
Check 'the wizard runs to the end' ($w.exited -and $w.code -eq 0) "exit=$($w.code) pages: $($w.seen)"
$h = WaitWin 'PecoFence.Control' 30 -Hidden
$notice = WaitWin 'PecoFence.Notice' 20
Check 'the first run shows the welcome notice' ($notice -ne [IntPtr]::Zero) $(if ($notice -ne [IntPtr]::Zero) { [W]::Info($notice) })
Shot 'd2-welcome'
Check 'its close box closes it' (CloseNotice)
Start-Sleep -Seconds 2
$fs = @(IpfJson fence list)
$items = @(IpfJson item list)
$alpha = $items | Where-Object { $_.name -eq 'Alpha Folder' }
Check 'the first run sorts the desktop into fences' ($fs.Count -ge 3 -and $alpha.assignedBy -eq 'rule') ("fences: " + (($fs | ForEach-Object { $_.title }) -join ', ') + "; Alpha Folder in $($alpha.fenceTitle) by $($alpha.assignedBy)")
Check 'the Start menu shortcut and the startup entry are there' ((Test-Path $lnk) -and (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PecoFence) "run=$((Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PecoFence)"

Case 'prepare the day'
Ipf fence create --title Work --rect '370,290,330,250' | Out-Null
Ipf fence create --title Drag --rect '720,290,330,250' | Out-Null
Ipf item move "$PfDesk\Alpha Folder" "$PfDesk\Beta notes.txt" "$PfDesk\Delta temp.txt" "$PfDesk\Epsilon temp.txt" "$PfDesk\Gamma link.lnk" --to Work | Out-Null
Ipf item move "$PfDesk\Eta drag.txt" "$PfDesk\Zeta drag.txt" --to Drag | Out-Null
Ipf fence set --all sort name | Out-Null
Ipf fence set inbox desktopLayout false | Out-Null
$slots = @('20,20,330,250', '370,20,330,250', '720,20,330,250', '20,290,330,250', '20,560,330,250', '370,560,330,250', '720,560,330,250')
$k = 0
foreach ($f in @(IpfJson fence list)) { if ($f.title -in 'Work', 'Drag') { continue }; Ipf fence move $f.id --rect $slots[$k] | Out-Null; $k++ }
Start-Sleep -Seconds 2
$fs = @(IpfJson fence list)
Log ("fences: " + (($fs | ForEach-Object { "$($_.title) $($_.windowRect.x),$($_.windowRect.y),$($_.windowRect.w),$($_.windowRect.h) items=$($_.itemCount)" }) -join ' ; '))
$work = @(IpfJson item list --fence Work | ForEach-Object { $_.name }) -join ', '
$inbox = @(IpfJson item list --fence inbox | ForEach-Object { $_.name }) -join ', '
Check 'Work holds the five test items, the Desktop fence This PC and the Recycle Bin' (($work -split ', ').Count -eq 5 -and $inbox -match 'Recycle Bin' -and $inbox -match 'This PC') "Work: $work; Desktop: $inbox"
Shot 'day-0'

Case 'D4 right-click a folder right after start'
$pt = ItemPt 'Work' 'Alpha Folder'
$t = TrayPick '^Exit PecoFence'
Check 'Exit in the tray menu quits it' ($t -and (WaitNoPf 15)) "pick='$t' menu: $LastTrayMenu"
[G]::MoveTo($pt[0], $pt[1])
$clock = [Diagnostics.Stopwatch]::StartNew()
Start-Process explorer.exe "`"$exe`""
$shownAt = -1
for ($i = 0; $i -lt 400; $i++) { if ([W]::RootClassAt($pt[0], $pt[1]) -eq 'PecoFence.Fence') { $shownAt = $clock.ElapsedMilliseconds; break }; Start-Sleep -Milliseconds 25 }
$clickUtc = [DateTime]::UtcNow
[G]::RClick($pt[0], $pt[1]); $clickAt = $clock.ElapsedMilliseconds
$m = MenuWin 15; $menuAt = $clock.ElapsedMilliseconds
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
Shot 'd4-first-menu'
Check 'the first folder right-click opens its menu' ($m -ne [IntPtr]::Zero -and $texts -match '^Open') "fence on screen at $shownAt ms, right-click at $clickAt ms, menu at $menuAt ms after start; items: $texts"
CloseMenus
$sw = [Diagnostics.Stopwatch]::StartNew(); $st = Ipf status; $ms = $sw.ElapsedMilliseconds
Check 'the app answers at once after that menu' ($st -match 'appVersion' -and $ms -lt 5000) "status in $ms ms"
$lines = @(Get-Content (UserLog) -Encoding UTF8)
$start = $lines | Where-Object { $_ -match 'process started' } | Select-Object -First 1
$cache = $lines | Where-Object { $_ -match 'shell handler cache' } | Select-Object -First 1
$t1 = LogTime $start; $t2 = LogTime $cache
$after = if ($t1 -and $t2) { [int]($t2 - $t1).TotalMilliseconds } else { -1 }
$beforeClick = if ($t2) { [int]($clickUtc - $t2).TotalMilliseconds } else { 0 }
Check 'the log has shell handler cache prepared early after start' ($cache -match 'prepared' -and $after -ge 0 -and $after -lt 5000) "$after ms after process started, $beforeClick ms before the right-click: $cache"

Case 'D4 every item menu'
Start-Sleep -Seconds 15
$m = RClickItem 'Work' 'Alpha Folder'; $texts = MenuTexts $m
Check 'a folder, again later' ($texts -match '^Open') $texts
CloseMenus
$m = RClickItem 'Work' 'Beta notes'; $texts = MenuTexts $m
Check 'a file: Open, Send to, Copy as path, Properties' ($texts -match '^Open' -and $texts -match 'Send to' -and $texts -match 'Copy as path' -and $texts -match 'Properties') $texts
$p = MenuPoint $m '^Send to'
if ($p) { [G]::MoveTo($p[0] - 20, $p[1]); Start-Sleep -Milliseconds 300; [G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 1500 }
$menus = @([W]::Tops('#32768'))
$sub = if ($menus.Count -ge 2) { MenuTexts $menus[0] } else { '' }
Shot 'd4-send-to'
Check 'hovering Send to opens its submenu' ($menus.Count -ge 2 -and $sub -match 'Desktop|Compressed') "menus=$($menus.Count) submenu: $sub"
CloseMenus
$m = RClickItem 'Work' 'Gamma link'; $texts = MenuTexts $m; $dupes = MenuDupes $m
Check 'a shortcut: Open file location' ($texts -match '^Open' -and $texts -match 'Open file location') $texts
Check 'a shortcut: no entry twice in its menu' (-not $dupes) "twice: $dupes"
CloseMenus
$m = RClickItem 'inbox' 'This PC'; $texts = MenuTexts $m
Check 'This PC: Rename, Delete and Open file location greyed' ($texts -match '^Open ~ Enter \|' -and $texts -match 'Rename ~ F2 \(off\)' -and $texts -match 'Delete ~ Delete \(off\)' -and $texts -match 'Open file location \(off\)') $texts
CloseMenus
$m = RClickItem 'inbox' 'Recycle Bin'; $texts = MenuTexts $m
Check 'the empty Recycle Bin: Empty Recycle Bin greyed' ($texts -match '^Open ~ Enter \|' -and $texts -match 'Empty Recycle Bin \(off\)') $texts
CloseMenus
$sw = [Diagnostics.Stopwatch]::StartNew(); $st = Ipf status; $ms = $sw.ElapsedMilliseconds
Check 'the app still answers after the menus' ($st -match 'appVersion' -and $ms -lt 5000) "status in $ms ms"

Case 'D4 menu commands'
$m = RClickItem 'Work' 'Alpha Folder'; $t = MenuPick '^Open' $m
Start-Sleep -Seconds 4
$ex = Explorers
Check 'Open on a folder opens it in Explorer' ($t -and $ex -match 'Alpha Folder') "pick='$t' explorer: $ex"
CloseWindows
$m = RClickItem 'Work' 'Beta notes'; $t = MenuPick '^Properties' $m
$props = [IntPtr]::Zero
for ($i = 0; $i -lt 40 -and $props -eq [IntPtr]::Zero; $i++) { Start-Sleep -Milliseconds 200; $props = @([W]::Tops('#32770') | Where-Object { [W]::Title($_) -match 'Properties' }) | Select-Object -First 1; if (-not $props) { $props = [IntPtr]::Zero } }
Check 'Properties opens the file properties' ($props -ne [IntPtr]::Zero) "pick='$t' window: $(if ($props -ne [IntPtr]::Zero) { [W]::Info($props) })"
Shot 'd4-properties'
if ($props -ne [IntPtr]::Zero) { [W]::PostMessageW($props, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Milliseconds 600 }
Set-Clipboard -Value 'empty'
$m = RClickItem 'Work' 'Beta notes'; $t = MenuPick '^Copy as path' $m
Start-Sleep -Milliseconds 800
$clip = (Get-Clipboard -Raw)
Check 'Copy as path puts the quoted path on the clipboard' ($clip -and $clip.Trim() -eq "`"$PfDesk\Beta notes.txt`"") "pick='$t' clipboard=$clip"
$m = RClickItem 'Work' 'Gamma link'; $t = MenuPick '^Rename' $m
$ren = WaitWin 'PecoFence.Rename' 5
Start-Sleep -Milliseconds 300
$fg = Fg
[G]::Type('Gamma renamed'); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 2
Check 'Rename renames the file on disk' ((Test-Path "$PfDesk\Gamma renamed.lnk") -and -not (Test-Path "$PfDesk\Gamma link.lnk")) "pick='$t' editor=$ren fg=$fg"
$m = RClickItem 'Work' 'Delta temp'; $t = MenuPick '^Delete' $m
Start-Sleep -Seconds 1
$boxes = @(SetupBoxes); if ($boxes.Count) { Log "delete asked: $(DlgText $boxes[0])"; ClickButton $boxes[0] '^Yes$' | Out-Null }
Start-Sleep -Seconds 2
Check 'Delete moves the file to the Recycle Bin' (-not (Test-Path "$PfDesk\Delta temp.txt") -and (BinNames) -match 'Delta temp') "pick='$t' bin: $(BinNames)"
$p = ItemPt 'Work' 'Epsilon temp'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
$fg = Fg
[G]::Press(0x2E); Start-Sleep -Seconds 1
$boxes = @(SetupBoxes); if ($boxes.Count) { Log "delete asked: $(DlgText $boxes[0])"; ClickButton $boxes[0] '^Yes$' | Out-Null }
Start-Sleep -Seconds 2
Check 'the Delete key deletes the selected icon' (-not (Test-Path "$PfDesk\Epsilon temp.txt")) "fg=$fg bin: $(BinNames)"
$m = RClickItem 'inbox' 'Recycle Bin'; $texts = MenuTexts $m
Check 'with deleted files in it, the Recycle Bin offers Empty Recycle Bin' ($texts -match 'Empty Recycle Bin \|') $texts
CloseMenus
$p = ItemPt 'Work' 'Beta notes'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
Chord 0x11 0x43; Start-Sleep -Milliseconds 600
$drop = @(Get-Clipboard -Format FileDropList -ErrorAction SilentlyContinue | ForEach-Object { $_.FullName })
Chord 0x11 0x56; Start-Sleep -Seconds 3
$copyIn = ItemFence 'Beta notes - Copy'
Check 'Ctrl+C copies the file, Ctrl+V pastes a copy (filed by the rules)' (($drop -join ',') -eq "$PfDesk\Beta notes.txt" -and (Test-Path "$PfDesk\Beta notes - Copy.txt") -and $copyIn) "clipboard=$($drop -join ',') copy in '$copyIn' fg=$(Fg)"
$hang = @(Get-Content (UserLog) -Encoding UTF8 | Where-Object { $_ -match 'UI thread not responding' })
Check 'no UI thread not responding in the log, app alive' ($hang.Count -eq 0 -and (Get-Process pecofence -ErrorAction SilentlyContinue)) (($hang | Select-Object -First 2) -join ' | ')

Case 'D6 double-click the desktop'
CloseWindows
$free = @(1300, 650)
[G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 1200
$n0 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n1 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n2 = VisFences
Check 'a double-click on the empty desktop hides the fences, another shows them' ($n0 -gt 0 -and $n1 -eq 0 -and $n2 -eq $n0) "visible fences: $n0 -> $n1 -> $n2; at the point: $([W]::RootClassAt($free[0], $free[1]))"

Case 'D6 marquee: new fence here'
$c0 = @(IpfJson fence list).Count
[G]::Click(1300, 650); Start-Sleep -Milliseconds 1000
DragTo 1150 120 1450 360
# Two small buttons at the rectangle's bottom-right corner (no menu); their window title names
# both: 'Show a folder here…|New fence'. New fence is the right one.
$off = [IntPtr]::Zero
for ($i = 0; $i -lt 40 -and $off -eq [IntPtr]::Zero; $i++) { $h = [G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $off = $h }; Start-Sleep -Milliseconds 50 }
$texts = if ($off -ne [IntPtr]::Zero) { [W]::Title($off) } else { '' }
$m = MenuWin 1
Shot 'd6-marquee'
Check 'a rectangle on the empty desktop shows the New fence button at its corner, no menu' ($texts -match 'New fence' -and $m -eq [IntPtr]::Zero) "buttons: '$texts' menu=$m"
$or = Rect $off
[G]::Click($or.R - 30, [int](($or.T + $or.B) / 2))
$ren = WaitWin 'PecoFence.Rename' 5
Start-Sleep -Milliseconds 300
[G]::Type('Marquee fence'); [G]::Press(0x0D); Start-Sleep -Seconds 2
$mf = IpfJson fence get 'Marquee fence'
$r = $mf.rect
Check 'the new fence is there, titled as typed, where the rectangle was' ($mf.title -eq 'Marquee fence' -and [math]::Abs($r.x - 1150) -le 40 -and [math]::Abs($r.y - 120) -le 40 -and @(IpfJson fence list).Count -eq $c0 + 1) "rename editor=$ren rect=$($r.x),$($r.y),$($r.w),$($r.h)"

Case 'D6 Peek hotkey'
Start-Process notepad.exe; Start-Sleep -Seconds 2
$np = [G]::FindWindowW('Notepad', [NullString]::Value)
[L]::MoveWindow($np, 40, 30, 1100, 780, $true) | Out-Null; Start-Sleep -Milliseconds 500
TitleClick $np
PeekKey; Start-Sleep -Milliseconds 1500
$dim = [G]::FindWindowW('PecoFence.PeekDimmer', [NullString]::Value)
$on = ($dim -ne [IntPtr]::Zero -and [G]::IsWindowVisible($dim)); $box = [M]::BoxText()
Shot 'd6-peek'
Check 'Ctrl+Alt+Space brings the fences above Notepad' ($on -or $null -ne $box) "dimmer visible=$on search box='$box' fg=$(Fg)"
for ($i = 0; $i -lt 3 -and (([G]::IsWindowVisible($dim)) -or $null -ne [M]::BoxText()); $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 600 }
Check 'Esc ends the Peek' (-not [G]::IsWindowVisible($dim) -and $null -eq [M]::BoxText()) "fg=$(Fg)"
CloseWindows

Case 'D6 drag icons'
$p = ItemPt 'Drag' 'Eta drag'
$wk = IpfJson fence get Work
$to = @(($wk.windowRect.x + $wk.windowRect.w - 50), ($wk.windowRect.y + $wk.windowRect.h - 40))
DragTo $p[0] $p[1] $to[0] $to[1]
Start-Sleep -Seconds 1
$in = ItemFence 'Eta drag'
Check 'an icon dragged into another fence lands there' ($in -eq 'Work') "Eta drag is in '$in'"
$p = ItemPt 'Drag' 'Zeta drag'
# The fifth fence (the marquee's) brings the 「栅栏多了可以收起」 tip card to the bottom-right corner,
# right over the drop point: close it first.
CloseNotice | Out-Null
DragTo $p[0] $p[1] 1300 780
Start-Sleep -Seconds 1
$in = ItemFence 'Zeta drag'
Check 'an icon dragged onto the bare desktop goes back to the Desktop fence, the file stays' ($in -eq (IpfJson fence get inbox).title -and (Test-Path "$PfDesk\Zeta drag.txt")) "Zeta drag is in '$in'"
Shot 'd6-dragged'

Case 'D6 Desktop fence arranged like the desktop'
$ib = IpfJson fence get inbox
[G]::RClick($ib.windowRect.x + 90, $ib.windowRect.y + 18)
$m = MenuWin 5; $texts = MenuTexts $m
$t = MenuPick 'Arrange like the desktop' $m
Start-Sleep -Seconds 2
$ib2 = IpfJson fence get inbox
$wr = $ib2.windowRect
Shot 'd6-arranged'
Check 'Arrange like the desktop in the Desktop fence menu' ($t -and $ib2.desktopLayout -eq $true) "pick='$t' windowRect=$($wr.x),$($wr.y),$($wr.w),$($wr.h) menu: $texts"
[G]::Click($free[0], $free[1]); Start-Sleep -Milliseconds 1200
$n0 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n1 = VisFences
DblClick $free[0] $free[1]; Start-Sleep -Seconds 2
$n2 = VisFences
Check 'the desktop double-click still hides and shows with the arranged Desktop fence' ($n0 -gt 0 -and $n1 -eq 0 -and $n2 -eq $n0) "visible fences: $n0 -> $n1 -> $n2"

Case 'D6 tray menu'
$t = TrayPick '^Hide all fences'; Start-Sleep -Seconds 1
$n1 = VisFences
$t2 = TrayPick '^Show all fences'; Start-Sleep -Seconds 1
$n2 = VisFences
Check 'Hide all fences and Show all fences in the tray menu' ($t -and $t2 -and $n1 -eq 0 -and $n2 -gt 0) "picks '$t' / '$t2'; visible $n1 -> $n2; menu: $LastTrayMenu"

Case 'D6 settings'
$t = TrayPick '^Settings'
$sh = WaitWin 'PecoFence.Settings' 20
Check 'Settings opens from the tray menu' ($sh -ne [IntPtr]::Zero) "pick='$t'"
[L]::MoveWindow($sh, 150, 20, 1200, 840, $true) | Out-Null; Start-Sleep -Seconds 3
Shot 'd6-settings'
$s0 = IpfJson settings get
# The other theme style tile, the colour mode list (Dark), the Chameleon switch. Screen points
# for the window at 150,20 1200x840, measured on a debug build over CDP (2026-11-08).
$tile = if ($s0.themeStyle -eq 'fluent') { @(880, 404) } else { @(614, 404) }
[G]::Click($tile[0], $tile[1]); Start-Sleep -Seconds 2
$s1 = IpfJson settings get
Check 'a click on the other theme style tile switches the style' ($s1.themeStyle -ne $s0.themeStyle) "themeStyle $($s0.themeStyle) -> $($s1.themeStyle)"
Shot 'd6-settings-style'
[G]::Click(1188, 535); Start-Sleep -Milliseconds 800
[G]::Press(0x23); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Seconds 2
$s2 = IpfJson settings get
Check 'the Color mode list picks Dark' ($s2.theme -eq 'dark') "theme $($s1.theme) -> $($s2.theme)"
[G]::Click(1193, 689); Start-Sleep -Milliseconds 800
[G]::Press(0x28); Start-Sleep -Milliseconds 300; [G]::Press(0x0D); Start-Sleep -Seconds 2
$s3 = IpfJson settings get
Check 'the Title alignment list picks Center' ($s3.titleAlign -eq 'center') "titleAlign $($s2.titleAlign) -> $($s3.titleAlign)"
Shot 'd6-settings-changed'
$sr = Rect $sh
[G]::Click($sr.R - 23, $sr.T + 16); Start-Sleep -Seconds 1
Check 'its close box closes Settings' (WaitGoneWin 'PecoFence.Settings' 5) ''

Case 'D6 log and exit'
$lines = @(Get-Content (UserLog) -Encoding UTF8)
Copy-Item (UserLog) "$PfRun\day-pecofence.log" -ErrorAction SilentlyContinue
$errs = @($lines | Where-Object { $_ -match ' ERROR |panicked' })
$warns = @($lines | Where-Object { $_ -match ' WARN ' } | ForEach-Object { ($_ -replace '^\S+\s+WARN\s+', '') } | Select-Object -Unique)
Check 'the log of the day has no ERROR' ($errs.Count -eq 0) (($errs | Select-Object -First 3) -join ' | ')
Check 'the log of the day has no WARN' ($warns.Count -eq 0) (($warns | Select-Object -First 6) -join ' | ')
$t = TrayPick '^Exit PecoFence'
$gone = WaitNoPf 15
Start-Sleep -Seconds 1
Check 'Exit quits it and the Windows desktop icons come back' ($t -and $gone -and [W]::IconsVisible()) "pick='$t' gone=$gone icons=$([W]::IconsVisible())"

Case 'D2 uninstall while it runs'
$h = StartUser $exe
$style = (Ipf settings get themeStyle).Trim()
Check 'the settings changed in the day stay after a restart' ($style -eq "`"$($s1.themeStyle)`"") "themeStyle=$style"
$seen = Uninstall "$prog\unins000.exe" { $script:quitBy = TrayPick '^Exit PecoFence'; $script:quitGone = WaitNoPf 15 }
Start-Sleep -Seconds 2
Check 'the uninstaller asks to close PecoFence, then removes it' ($seen -match 'app running prompt' -and $seen -match 'removed') "seen: $seen; tray pick='$quitBy' gone=$quitGone"
Check 'no PecoFence process is left' (-not (Get-Process pecofence, pecofence-watchdog -ErrorAction SilentlyContinue))
Check 'no program folder is left' (-not (Test-Path $prog)) $(if (Test-Path $prog) { (Get-ChildItem $prog -Recurse | ForEach-Object { $_.Name }) -join ', ' })
Check 'the Start menu shortcut and the startup entry are gone' (-not (Test-Path $lnk) -and -not (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PecoFence)
Check 'the Windows desktop icons are shown' ([W]::IconsVisible())

Case 'D3 portable zip'
$pdir = "$PfRun\portable"
Expand-Archive "$PfExtra\portable.zip" $pdir -Force
$appCfg = "$env:APPDATA\PecoFence\config.json"
$cfgBefore = if (Test-Path $appCfg) { (Get-FileHash $appCfg).Hash } else { 'none' }
Start-Process "$pdir\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $pdir
$h = WaitWin 'PecoFence.Control' 30 -Hidden
Start-Sleep -Seconds 5
$script:CliExe = "$pdir\pecofence-cli.exe"
$st = IpfJson status
Check 'the zip runs from its folder with --portable, config beside the exe' ($h -ne [IntPtr]::Zero -and $st.configPath -like "$pdir\config\*" -and (Test-Path "$pdir\config\config.json")) "config=$($st.configPath) version=$($st.appVersion)"
$fs = @(IpfJson fence list)
Check 'it sorts the desktop into fences' ($fs.Count -ge 3) (($fs | ForEach-Object { $_.title }) -join ', ')
$cfgAfter = if (Test-Path $appCfg) { (Get-FileHash $appCfg).Hash } else { 'none' }
$runVals = (Get-ItemProperty $runKey -ErrorAction SilentlyContinue).PSObject.Properties | Where-Object { "$($_.Value)" -like '*pecofence*' } | ForEach-Object { "$($_.Name)=$($_.Value)" }
Check 'it writes no startup entry and leaves the installed config alone' (-not $runVals -and $cfgAfter -eq $cfgBefore) "run: $runVals; appdata config $($cfgBefore.Substring(0, 4)) -> $($cfgAfter.Substring(0, 4))"
Shot 'd3-portable'
CloseNotice | Out-Null
$t = TrayPick '^Exit PecoFence'
Check 'it quits from its tray menu' ($t -and (WaitNoPf 15) -and [W]::IconsVisible()) "pick='$t'"
$plines = @(Get-Content (UserLog) -Encoding UTF8)
Copy-Item (UserLog) "$PfRun\portable-pecofence.log" -ErrorAction SilentlyContinue
$errs = @($plines | Where-Object { $_ -match ' ERROR |panicked' })
Check 'the portable log has no ERROR' ($errs.Count -eq 0) (($errs | Select-Object -First 3) -join ' | ')

Case 'D5 restart at sign-in'
Log 'skipped: the box cannot reboot or sign in again during a job'

Case 'cleanup'
CloseWindows
foreach ($n in $testFiles) { Remove-Item -LiteralPath "$PfDesk\$n" -Recurse -Force -ErrorAction SilentlyContinue }
Get-Process msedge -ErrorAction SilentlyContinue | Stop-Process -Force
Remove-ItemProperty $hdi -Name '{20D04FE0-3AEA-1069-A2D8-08002B30309D}' -ErrorAction SilentlyContinue
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
