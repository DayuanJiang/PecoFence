# 在哪个虚拟桌面显示, what it looks like and where it is chosen: the Task View glyph before the
# title of a bound fence (its infotip names the desktops), on every tab of a bound window, gone
# again when unbound; 「每个虚拟桌面各用一套栅栏」 from the CLI and the General page, with new
# fences made by the CLI, the fence menu, a note and a folder portal; the 「栅栏」 page's select.
# Settings page clicks use coordinates measured on this kit's shots at 1600x900 (Chinese UI).

function ParkPointer() { [G]::MoveTo(1250, 780); Start-Sleep -Milliseconds 300 }
# The title row of fence $t (screen px): the window's client origin from a dump line.
function TitleRow($t) { $d = DumpOf $t; return @(($d.l + $d.ox), ($d.t + $d.oy), $d.cw, $d) }
function TitleCrop($t, $name) { $r = TitleRow $t; return (Crop $name $r[0] $r[1] 200 36) }
# The infotip PecoFence shows now (OCR of its window), '' when none is up.
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
# Settings window helpers: open on a page, scroll it by wheel notches, close.
function SettingsScrolled($notches) {
  $h = WaitWin 'PecoFence.Settings' 20
  Start-Sleep -Seconds 4
  for ($i = 0; $i -lt $notches; $i++) { Wheel 920 426 -3; Start-Sleep -Milliseconds 700 }
  return $h
}
function CloseSettings() { $h = [G]::FindWindowW('PecoFence.Settings', [NullString]::Value); if ($h -ne [IntPtr]::Zero) { [W]::PostMessageW($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1 } }

Case 'setup'
CloseWindows
GreyWallpaper
VdTwo 'Work' 'Games'
$ids = @(VdIds)
Check 'two desktops named Work and Games' ($ids.Count -eq 2 -and (VdName $ids[1]) -eq 'Games') (VdState)
New-Item -ItemType Directory -Force "$PfRun\Portal folder" | Out-Null
Set-Content "$PfRun\Portal folder\inside.txt" 'x'
PfStart -Script (DumpScript)
Pf settings set theme light | Out-Null
Pf fence create --title Work --rect '80,80,420,240' | Out-Null
Pf fence create --title Games --rect '600,80,420,240' | Out-Null
ParkPointer
Start-Sleep -Seconds 2
$plain = TitleCrop 'Games' 'title-plain'

Case 'the glyph and its infotip'
Pf fence set Games virtualDesktops 1 | Out-Null
Start-Sleep -Seconds 1
$d = DumpOf 'Games'
Check 'dump: Games has the desktop mark naming Work' ($d.raw -match 'desktop_tip=Some\("Only on “Work”"\)') ($d.raw -replace '.*(off_desktop=.*)', '$1')
$marked = TitleCrop 'Games' 'title-marked'
$diff = [Px]::Diff($plain, $marked, 10, 40, 6, 30)
Check 'a glyph is drawn before the title' ($diff -gt 4) "diff=$([math]::Round($diff, 1))"
$moved = [Px]::Diff($plain, $marked, 40, 120, 6, 30)
Check 'the title moved one slot on' ($moved -gt 4) "diff=$([math]::Round($moved, 1))"
Shot 'ui-glyph'
$r = TitleRow 'Games'
$tip = HoverTip ($r[0] + 24) ($r[1] + 18) 'tip-glyph'
Check 'hovering the glyph shows the desktops' ($tip -match 'Only on' -and $tip -match 'Work') "tip='$tip'"
Shot 'ui-glyph-tip'
$tip = HoverTip ($r[0] + 200) ($r[1] + 140) 'tip-body'
Check 'no desktop infotip over the fence body' (-not ($tip -match 'Only on')) "tip='$tip'"
Pf fence set Games virtualDesktops '1,2' | Out-Null
Start-Sleep -Milliseconds 800
$r = TitleRow 'Games'
$tip = HoverTip ($r[0] + 24) ($r[1] + 18) 'tip-two'
Check 'both desktops named one by one count as every desktop: no glyph, no infotip' (@((FenceGet Games).virtualDesktops).Count -eq 0 -and -not ($tip -match 'Only on')) "vd=$((FenceGet Games).virtualDesktops) tip='$tip'"
Pf fence set Games virtualDesktops 1 | Out-Null
Start-Sleep -Milliseconds 800

Case 'a rename in Task View reaches the infotip'
VdRename 1 'Office'
ParkPointer
Start-Sleep -Seconds 1
$d = DumpOf 'Games'
Check 'dump: the mark names the new name' ($d.raw -match 'Only on “Office”') ($d.raw -replace '.*(desktop_tip=.*)', '$1')
VdRename 1 'Work'
Start-Sleep -Seconds 1

Case 'unbound again: no glyph'
Pf fence set Games virtualDesktops all | Out-Null
Start-Sleep -Seconds 1
$d = DumpOf 'Games'
$back = TitleCrop 'Games' 'title-back'
Check 'dump: no mark' ($d.raw -match 'desktop_tip=None') ($d.raw -replace '.*(desktop_tip=.*)', '$1')
Check 'the title row looks as before' ([Px]::Diff($plain, $back, 10, 120, 6, 30) -lt 2) "diff=$([math]::Round([Px]::Diff($plain, $back, 10, 120, 6, 30), 1))"

Case 'tabs of a bound window'
Pf fence create --title Mods --rect '600,420,300,160' | Out-Null
Start-Sleep -Milliseconds 600
Pf fence merge Mods --into Games | Out-Null
Start-Sleep -Seconds 1
$before = DumpOf 'Games'
Pf fence set Games virtualDesktops 1 | Out-Null
Start-Sleep -Seconds 1
$after = DumpOf 'Games'
$grow = @(0..1 | ForEach-Object { [math]::Round($after.rects[$_][1] - $before.rects[$_][1], 1) })
Check 'every tab pill gains the glyph slot (20 DIPs)' ($grow[0] -eq 20 -and $grow[1] -eq 20) "grew $($grow -join ', ')"
Shot 'ui-tabs-glyph'
$x = $after.l + $after.ox + $after.rects[1][0] + $after.rects[1][1] / 2
$tip = HoverTip ([int]$x) ($after.t + $after.oy + 18) 'tip-tab'
Check 'a tab infotip names the desktops' ($tip -match 'Only on' -and $tip -match 'Work') "tip='$tip'"
Pf settings set tabBar left | Out-Null
Start-Sleep -Seconds 1
Shot 'ui-rail-glyph'
$rail = DumpOf 'Games'
Check 'the left tab column still shows both tabs' ($rail.ntabs -eq 2) $rail.raw.Substring(0, [Math]::Min(160, $rail.raw.Length))
Pf settings set tabBar top | Out-Null
Pf fence detach Mods | Out-Null
Pf fence set Games virtualDesktops all | Out-Null
Pf fence set Mods virtualDesktops all | Out-Null
Start-Sleep -Seconds 1

Case 'per-desktop fences from the CLI'
Pf settings set perDesktopFences true | Out-Null
Pf fence create --title 'New CLI' --rect '80,420,300,160' | Out-Null
Start-Sleep -Milliseconds 800
$n = FenceGet 'New CLI'
Check 'switch on: a CLI fence belongs to the current desktop' (@($n.virtualDesktops).Count -eq 1 -and $n.virtualDesktops[0] -eq "{$($ids[0])}") "vd=$($n.virtualDesktops)"
VdRight
Check 'on Games it is hidden' ((FVis 'New CLI') -eq $false) (VisText @('New CLI'))
Pf fence create --tool note --title 'Games note' --rect '1000,420,260,160' | Out-Null
Pf fence create --portal "$PfRun\Portal folder" --title 'Games portal' --rect '1000,620,260,160' | Out-Null
Start-Sleep -Milliseconds 800
$note = FenceGet 'Games note'; $portal = FenceGet 'Games portal'
Check 'a note and a folder portal made on Games belong to Games' ($note.virtualDesktops[0] -eq "{$($ids[1])}" -and $portal.virtualDesktops[0] -eq "{$($ids[1])}") "note=$($note.virtualDesktops) portal=$($portal.virtualDesktops)"
$m = FenceMenu 'Games'
MenuPick '^New fence$' $m | Out-Null; Start-Sleep -Milliseconds 1200
[G]::Press(0x0D); Start-Sleep -Milliseconds 800
$made = @((PfJson fence list) | Where-Object { $_ -and $_.title -eq 'New fence' })
Check 'a fence made from the fence menu belongs to Games' ($made.Count -eq 1 -and $made[0].virtualDesktops[0] -eq "{$($ids[1])}") "found=$($made.Count) vd=$($made[0].virtualDesktops)"
$d = DumpOf 'Games portal'
Check 'the portal shows the folder glyph and the Task View glyph' ($d.raw -match 'desktop_tip=Some') ($d.raw -replace '.*(desktop_tip=.*)', '$1')
# The Task View glyph follows the folder glyph's slot (16 + 24 DIPs, 100 % here).
$tip = HoverTip ($d.l + $d.ox + 50) ($d.t + $d.oy + 18) 'tip-portal'
Check 'the portal glyph infotip names Games' ($tip -match 'Only on' -and $tip -match 'Games') "tip='$tip'"
Shot 'ui-switch-new-fences'
Pf settings set perDesktopFences false | Out-Null
Pf fence create --title 'New off' --rect '80,620,300,160' | Out-Null
Start-Sleep -Milliseconds 800
Check 'switch off: a new fence shows on every desktop' (@((FenceGet 'New off').virtualDesktops).Count -eq 0) "vd=$((FenceGet 'New off').virtualDesktops)"
foreach ($t in 'New CLI', 'Games note', 'Games portal', 'New fence', 'New off', 'Mods') { Pf fence delete $t | Out-Null }
VdLeft

Case 'the General page switch'
Pf settings set language zh-CN | Out-Null
Start-Sleep -Seconds 1
Pf settings open-ui | Out-Null
SettingsScrolled 4 | Out-Null
Shot 'ui-page-switch-before'
[G]::Click(1183, 518); Start-Sleep -Seconds 1
$on = (Pf settings get perDesktopFences).Trim()
Shot 'ui-page-switch-on'
Check 'a click on 每个虚拟桌面各用一套栅栏 switches it on' ($on -match 'true') "get=$on"
[G]::Click(1183, 518); Start-Sleep -Seconds 1
$off = (Pf settings get perDesktopFences).Trim()
Check 'a second click switches it off' ($off -match 'false') "get=$off"
CloseSettings

Case 'the 栅栏 page select'
$m = FenceMenu 'Work'
MenuPick '^栅栏选项' $m | Out-Null
SettingsScrolled 4 | Out-Null
Shot 'ui-page-select-before'
[G]::Click(1120, 380); Start-Sleep -Milliseconds 900
Shot 'ui-page-select-open'
[G]::Press(0x24); [G]::Press(0x28); [G]::Press(0x28); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 1
$w = FenceGet 'Work'
Check 'page: Work now on Games only, hidden here' (@($w.virtualDesktops).Count -eq 1 -and $w.virtualDesktops[0] -eq "{$($ids[1])}" -and (FVis 'Work') -eq $false) "vd=$($w.virtualDesktops) vis=$(FVis 'Work')"
[G]::Click(1120, 380); Start-Sleep -Milliseconds 900
[G]::Press(0x24); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Seconds 1
$w = FenceGet 'Work'
Check 'page: 所有虚拟桌面 puts it back everywhere' (@($w.virtualDesktops).Count -eq 0 -and (FVis 'Work') -eq $true) "vd=$($w.virtualDesktops) vis=$(FVis 'Work')"
CloseSettings

Case 'cleanup'
PfQuit
VdOnlyOne
