# 在哪个虚拟桌面显示 on Windows 10 (the PF-Win10 Hyper-V guest, 1600x900): Windows 10 keeps the
# current desktop under SessionInfo\<session>\VirtualDesktops. No Task View clicks here (its
# layout differs from Windows 11's), so the desktops keep their default names (Desktop N).

function ParkPointer() { [G]::MoveTo(1250, 760); Start-Sleep -Milliseconds 300 }
function TipText($name) {
  $tip = @([W]::Tops('tooltips_class32') | Where-Object { [W]::ProcOf($_) -eq 'pecofence' }) | Select-Object -First 1
  if (-not $tip) { return '' }
  $r = Rect $tip
  return (OcrText $r.L $r.T ($r.R - $r.L) ($r.B - $r.T) $name)
}
function HoverTip($x, $y, $name) {
  ParkPointer
  [G]::MoveTo($x - 3, $y); Start-Sleep -Milliseconds 120; [G]::MoveTo($x, $y)
  for ($i = 0; $i -lt 10; $i++) { Start-Sleep -Milliseconds 300; $t = TipText $name; if ($t) { return $t } }
  return ''
}

Case 'setup'
CloseWindows
GreyWallpaper
$os = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
Log "os: $($os.ProductName) $($os.DisplayVersion) build $($os.CurrentBuild).$($os.UBR)"
VdOnlyOne
VdNew
VdLeft
$ids = @(VdIds)
$main = (Get-ItemProperty $VdKey -ErrorAction SilentlyContinue).CurrentVirtualDesktop
$session = (Get-ItemProperty "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\SessionInfo\$([VD]::Session())\VirtualDesktops" -ErrorAction SilentlyContinue).CurrentVirtualDesktop
Log "registry: VirtualDesktops has current=$([bool]$main), SessionInfo\$([VD]::Session()) has current=$([bool]$session); $(VdState)"
Check 'two desktops, the first current' ($ids.Count -eq 2 -and (VdCurrent) -eq $ids[0]) (VdState)
$made = 'work-plan.txt', 'game-save.txt'
foreach ($n in $made) { Set-Content "$PfDesk\$n" 'x' }
PfStart -Script (DumpScript)
Pf settings set theme light | Out-Null
Pf fence create --title Work --rect '80,80,420,240' | Out-Null
Pf fence create --title Games --rect '600,80,420,240' | Out-Null
Pf item move "$PfDesk\work-plan.txt" --to Work | Out-Null
Pf item move "$PfDesk\game-save.txt" --to Games | Out-Null
ParkPointer
Start-Sleep -Seconds 2
$all = 'Work', 'Games'

Case 'the CLI and the menu'
$l = @(PfJson virtual-desktop list)
Check 'virtual-desktop list: Desktop 1 (current), Desktop 2' ($l.Count -eq 2 -and $l[0].name -eq 'Desktop 1' -and $l[0].current -and $l[1].name -eq 'Desktop 2') (($l | ConvertTo-Json -Compress))
Pf fence set Games virtualDesktops 2 | Out-Null
Start-Sleep -Seconds 1
Check 'Games bound to Desktop 2 fades out' ((FVis Games) -eq $false -and (FVis Work) -eq $true) (VisText $all)
VdRight
$sw = @(PfLogAll | Where-Object { $_ -match 'virtual desktop switched from=' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'Win+Ctrl+Right: the switch is followed, Games shows' ((FVis Games) -eq $true -and (FVis Work) -eq $true) "$(VisText $all); $sw"
$sub = VdSubmenu 'Work'
$st = MenuTexts $sub
Check 'the submenu lists Desktop 1 and Desktop 2 (current)' ($st -match 'Desktop 1' -and $st -match 'Desktop 2 \(current\)') $st
MenuPick '^Desktop 1$' $sub | Out-Null
Start-Sleep -Seconds 1
Check 'menu: Work only on Desktop 1, gone here' ((FVis Work) -eq $false -and (FVis Games) -eq $true) (VisText $all)
Shot 'win10-d2'
for ($k = 0; $k -lt 2; $k++) {
  VdLeft; $a = VisText $all
  VdRight; $b = VisText $all
  Check "round ${k}: Desktop 1 Work only, Desktop 2 Games only" ($a -eq 'Work=True Games=False' -and $b -eq 'Work=False Games=True') "d1: $a; d2: $b"
}

Case 'the glyph and its infotip'
$d = DumpOf 'Games'
Check 'dump: Games names Desktop 2' ($d.raw -match 'desktop_tip=Some\("Only on “Desktop 2”"\)') ($d.raw -replace '.*(desktop_tip=.*)', '$1')
$tip = HoverTip ($d.l + $d.ox + 24) ($d.t + $d.oy + 18) 'tip-win10'
Check 'hovering the glyph shows Desktop 2' ($tip -match 'Only on' -and $tip -match 'Desktop 2') "tip='$tip'"
Shot 'win10-glyph-tip'

Case 'quick hide right after a switch'
# Windows 10 reports no foreground change on a switch, and the desktop comes to the front only
# with the first click on it: PecoFence's foreground poll sees the switch and follows the
# desktop's clicks from then on, so that click counts.
for ($k = 0; $k -lt 2; $k++) {
  if ($k -eq 0) { VdLeft } else { VdRight; VdLeft }
  $fg = Fg
  DoubleClick 1100 700; Start-Sleep -Seconds 1
  Check "round ${k}: the first double-click after the switch hides Work" ((FVis Work) -eq $false) "Work=$(FVis Work) fg before=$fg after=$(Fg)"
  DoubleClick 1100 700; Start-Sleep -Seconds 1
  Check "round ${k}: and the next one shows it" ((FVis Work) -eq $true) "Work=$(FVis Work)"
}
$seen = @(PfLogAll | Where-Object { $_ -match 'virtual desktop switch seen by the foreground poll' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Log "poll: $seen"
# Not clicked within 10 s: the desktop's clicks are no longer followed (every mouse move reaches
# PecoFence while they are).
VdRight
[G]::MoveTo(900, 500); Start-Sleep -Seconds 11
$end = @(PfLogAll | Where-Object { $_ -match 'its clicks are no longer followed' -and $_ -notmatch 'omitted' }) | Select-Object -Last 1
Check 'ten seconds after a switch without a click on the desktop, its clicks are let go' ([bool]$end) "$end"
VdLeft

Case 'new fences with the switch on'
Pf settings set perDesktopFences true | Out-Null
Pf fence create --title 'New one' --rect '80,420,300,160' | Out-Null
Start-Sleep -Milliseconds 800
$n = FenceGet 'New one'
Check 'a new fence belongs to Desktop 1' (@($n.virtualDesktops).Count -eq 1 -and $n.virtualDesktops[0] -eq "{$($ids[0])}") "vd=$($n.virtualDesktops)"
Pf settings set perDesktopFences false | Out-Null
Pf fence delete 'New one' | Out-Null

Case 'restart on Desktop 2, then the poll alone'
VdRight
PfQuit
PfStart -Script @('vd-poll-only', 'sleep 3600000')
ParkPointer
Start-Sleep -Seconds 2
Check 'started on Desktop 2: Games shows, Work stays hidden' ((FVis Games) -eq $true -and (FVis Work) -eq $false) (VisText $all)
$sw = [Diagnostics.Stopwatch]::StartNew()
[G]::KeyDown(0x5B); [G]::KeyDown(0x11); [G]::Press(0x25); [G]::KeyUp(0x11); [G]::KeyUp(0x5B)
while ((FVis Work) -ne $true -and $sw.ElapsedMilliseconds -lt 3000) { Start-Sleep -Milliseconds 10 }
$ms = $sw.ElapsedMilliseconds
Start-Sleep -Seconds 2
Check 'the poll alone follows Win+Ctrl+Left within a second' ((FVis Work) -eq $true -and (FVis Games) -eq $false -and $ms -lt 1000) "after ${ms} ms; $(VisText $all)"

Case 'Desktop 2 closed'
VdRight
VdClose
Start-Sleep -Seconds 1
$g = FenceGet Games
Check 'Games moves to Desktop 1 with the windows' ((VdCurrent) -eq $ids[0] -and $g.virtualDesktops[0] -eq "{$($ids[0])}" -and (FVis Games) -eq $true) ("vd=$($g.virtualDesktops) " + (VdState))

Case 'cleanup'
PfQuit
VdOnlyOne
ClearPfNotices
foreach ($n in $made) { Remove-Item "$PfDesk\$n" -ErrorAction SilentlyContinue }
