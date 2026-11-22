# Issues round 2026-11-22, two monitors.
#   A  Feedback #207: 「桌面」 arranged like the desktop stands on the first monitor only. A file
#      made from the desktop menu (新建) after a right-click on the empty desktop of the second
#      monitor went into it there; now it goes into the plain fence nearest the click on the
#      second monitor (a folder fence takes nothing), and the menu's 新建 › 文件夹 opens its name
#      box there. With no fence on that monitor it still goes to 「桌面」. On the first monitor
#      it takes the cell under the click, as before (#238).
#   B  Feedback #203: icons dragged from a fence onto 「桌面」 arranged like the desktop and
#      back, one and several at once, recorded (rec.ps1) so the frames can be looked at.
# Default: both parts.
#
# Needs a Hyper-V guest with a second monitor (see monitor-sets-move.ps1):
#   scripts/wintest/hyperv/run-hyperv.sh --vm PF-Win11-2 --prep scripts/wintest/hyperv/vdd-prep.ps1 \
#     --extra <folder> scripts/wintest/tests/issues1122-monitors.ps1
if (-not $Part) { $Part = 'AB' }
trap { PfTrap $_; continue }
. C:\pf\kit\mon2.ps1
. C:\pf\kit\rec.ps1

# Windows 11's desktop menu (a XAML window) closes on Esc once it has the keyboard.
# Closes Windows' desktop menu: Esc twice, then clicks on bare desktop of the first monitor, until
# the point `$pt` (where the menu was opened) is bare desktop again and the menu's window is not in
# front. Esc does not always reach a menu opened on the second monitor.
function CloseDeskMenu($pt) {
  $desk = @('Progman', 'WorkerW')
  for ($i = 0; $i -lt 8; $i++) {
    $fg = [W]::Cls([L]::GetForegroundWindow())
    $root = if ($pt) { [W]::RootClassAt($pt[0], $pt[1]) } else { 'Progman' }
    if ($fg -notlike 'Xaml*' -and $fg -ne '#32768' -and $desk -contains $root) { break }
    if ($i -lt 2) { [G]::Press(0x1B) } else {
      $p = FreeSpot -NearX 300 -NearY 900
      if ($p) { [Mon2]::Click($p[0], $p[1], $false) }
    }
    Start-Sleep -Milliseconds 600
  }
  CloseMenus
  Log "desk menu closed after $i tries; in front: $([W]::Cls([L]::GetForegroundWindow())); at the click: $(if ($pt) { [W]::RootClassAt($pt[0], $pt[1]) })"
}
function M2Fmt($r) { if (-not $r) { return 'none' }; return "($($r.x),$($r.y) $($r.w)x$($r.h))" }
function M2In($r, $x, $y) { return $r -and $x -ge $r.x -and $x -lt $r.x + $r.w -and $y -ge $r.y -and $y -lt $r.y + $r.h }
# A spot of bare desktop near the middle of work area `wa` ({x, y, w, h}): Windows' desktop at
# the point and 40 px around it.
function M2Bare($wa) {
  $desk = @('Progman', 'WorkerW')
  $cx = $wa.x + [int]($wa.w / 2); $cy = $wa.y + [int]($wa.h / 2)
  foreach ($d in 0..12) {
    foreach ($p in @(@(($cx + 40 * $d), $cy), @(($cx - 40 * $d), $cy), @($cx, ($cy + 40 * $d)), @($cx, ($cy - 40 * $d)))) {
      if (-not (M2In $wa $p[0] $p[1])) { continue }
      $ok = $true
      foreach ($o in @(@(0, 0), @(-40, -40), @(40, -40), @(-40, 40), @(40, 40))) {
        if ($desk -notcontains [W]::RootClassAt($p[0] + $o[0], $p[1] + $o[1])) { $ok = $false; break }
      }
      if ($ok) { return $p }
    }
  }
  return $null
}
# Where the item named `$file` shows now: its fence, and for 「桌面」 arranged like the desktop the
# middle of its cell (screen px); for another fence the middle of that fence.
function M2Where($file) {
  $it = @(PfJson item list) | Where-Object { $_.fileName -eq $file } | Select-Object -First 1
  if (-not $it) { return $null }
  $f = FenceGet $it.fence
  $r = $f.windowRect
  $pt = $null
  if ($f.desktopLayout -and $f.desktopCells -and $f.desktopCells."$($it.id)") {
    $c = $f.desktopCells."$($it.id)"
    $s = $script:M2Scale
    $pt = @([int]($r.x + [math]::Round(4 * $s) + ($c[0] + 0.5) * [math]::Round(80 * $s)), [int]($r.y + [math]::Round(4 * $s) + ($c[1] + 0.5) * [math]::Round(96 * $s)))
  } elseif ($r) {
    $pt = @([int]($r.x + $r.w / 2), [int]($r.y + $r.h / 2))
  }
  return [pscustomobject]@{ fence = $f.title; kind = $f.kind; desktop = [bool]$f.desktopLayout; rect = $r; pt = $pt; cell = $(if ($f.desktopCells) { $f.desktopCells."$($it.id)" -join ',' }) }
}
function M2WaitWhere($file, $sec = 8) {
  for ($i = 0; $i -lt $sec * 4; $i++) { $w = M2Where $file; if ($w -and $w.pt) { return $w }; Start-Sleep -Milliseconds 250 }
  return (M2Where $file)
}
# Clicks the item named `$name` of the open desktop menu (Windows 11's XAML menu, or the
# classic one), found by UI Automation; false when it is not there.
function M2MenuClick($name) {
  $cond = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $name)
  for ($i = 0; $i -lt 10; $i++) {
    foreach ($h in @([W]::Tops('Microsoft.UI.Content.PopupWindowSiteBridge')) + @([W]::Tops('XamlExplorerHostIslandWindow_WASDK')) + @([W]::Tops('#32768'))) {
      try {
        $e = [Windows.Automation.AutomationElement]::FromHandle($h).FindFirst([Windows.Automation.TreeScope]::Descendants, $cond)
      } catch { $e = $null }
      if ($e) {
        $r = $e.Current.BoundingRectangle
        if ($r.Width -gt 0) {
          [Mon2]::Click([int]($r.X + $r.Width / 2), [int]($r.Y + $r.Height / 2), $false)
          Log "menu item '$name' clicked at $([int]($r.X + $r.Width / 2)),$([int]($r.Y + $r.Height / 2))"
          return $true
        }
      }
    }
    Start-Sleep -Milliseconds 300
  }
  Log "menu item '$name' not found"
  return $false
}
function M2Fmtw($w) { if (-not $w) { return 'not found' }; return "fence '$($w.fence)' ($($w.kind), desktop=$($w.desktop)) $(M2Fmt $w.rect) cell=$($w.cell) at $($w.pt -join ',')" }

Case 'two monitors: A 1920x1080 at 100 % (primary), B at 125 % to its right'
$GdiA, $GdiB = Mon2Pair 1920 1080 100 125
PfStart
Start-Sleep -Seconds 4
$mons = @(PfJson monitor list)
$MA = @($mons | Where-Object { $_.primary })[0]; $MB = @($mons | Where-Object { -not $_.primary })[0]
Check 'setup: A is the primary monitor at 0,0 at 100 %, B to its right at 125 %' ($MA -and $MB -and $MA.rect.x -eq 0 -and $MA.dpi -eq 96 -and $MB.dpi -eq 120 -and $MB.rect.x -ge $MA.rect.w) (($mons | ForEach-Object { "$($_.label) $(M2Fmt $_.rect) work $(M2Fmt $_.workArea) dpi $($_.dpi) primary $($_.primary)" }) -join '; ')
$script:M2Scale = 1.0
$inbox = @(PfJson fence list) | Where-Object { $_.kind -eq 'inbox' } | Select-Object -First 1
$fi = FenceGet $inbox.id
Check 'setup: 「桌面」 is arranged like the desktop with 自由摆放, on A' ($fi.desktopLayout -and $fi.freePlacement -and $fi.windowRect.x -lt $MA.rect.w) "desktopLayout=$($fi.desktopLayout) freePlacement=$($fi.freePlacement) window $(M2Fmt $fi.windowRect)"
foreach ($f in @(PfJson fence list)) { $g = FenceGet $f.id; Log "fence '$($g.title)' kind=$($g.kind) $(M2Fmt $g.windowRect)" }
ShotAll 'start'

if ($Part -match 'A') {
  # As the reporter has it: 「新项目出现在桌面时自动归类」 off, so the new files are no rule's (a
  # folder made by 新建 would go to 「文件夹」 by the first-run rules otherwise).
  $rulesWere = Pf rule list
  [IO.File]::WriteAllText("$PfRun\pk207-rules-off.json", ($rulesWere -replace '"keepUpdated":\s*true', '"keepUpdated": false'), (New-Object Text.UTF8Encoding $false))
  Pf rule import "$PfRun\pk207-rules-off.json" | Out-Null
  $at = M2Bare $MB.workArea
  Check 'setup: bare desktop found on B' ($at -ne $null) "at=$($at -join ',') B work $(M2Fmt $MB.workArea)"
  if ($at) {
    # No fence on B: the new file stays in 「桌面」 (on A), as before.
    Case 'A: no fence on B: a new file after a right-click on B goes to 「桌面」'
    [Mon2]::Click($at[0], $at[1], $true); Start-Sleep -Milliseconds 900
    ShotAll 'a-menu-on-b'
    CloseDeskMenu $at
    # The menu's 新建 writes the file at once; the test writes it itself. A type no rule takes.
    $n0 = "pk207 none $(Get-Date -Format HHmmss).pk207"
    Set-Content -LiteralPath (Join-Path $PfDesk $n0) 'x'
    $w0 = M2WaitWhere $n0
    ShotAll 'a-no-fence-on-b'
    Check 'with no fence on B the new file goes to 「桌面」' ($w0 -and $w0.desktop) (M2Fmtw $w0)
    Remove-Item -LiteralPath (Join-Path $PfDesk $n0) -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    # Fences on B around the click: a folder fence nearest (it shows its folder, so it takes
    # nothing), the plain fence Near a little farther, Far in a corner.
    Case 'A: fences on B: a new file after a right-click on B goes into the plain fence nearest the click'
    $dir = Join-Path $PfRun 'pk207dir'
    New-Item -ItemType Directory -Force $dir | Out-Null
    Pf fence create --portal $dir --rect "$($at[0] + 70),$($at[1] - 120),300,250" | Out-Null
    Pf fence create --title Near --rect "$($at[0] - 470),$($at[1] - 120),380,250" | Out-Null
    Pf fence create --title Far --rect "$($MB.workArea.x + 30),$($MB.workArea.y + $MB.workArea.h - 230),300,200" | Out-Null
    Start-Sleep -Seconds 2
    foreach ($t in 'pk207dir', 'Near', 'Far') { $g = FenceGet $t; Log "fence '$t' $(M2Fmt $g.windowRect)" }
    $bare = @('Progman', 'WorkerW') -contains [W]::RootClassAt($at[0], $at[1])
    Check 'setup: the click spot on B is still bare desktop' $bare "at=$($at -join ',') root=$([W]::RootClassAt($at[0], $at[1]))"
    [Mon2]::Click($at[0], $at[1], $true); Start-Sleep -Milliseconds 900
    CloseDeskMenu $at
    $n1 = "pk207 b $(Get-Date -Format HHmmss).pk207"
    Set-Content -LiteralPath (Join-Path $PfDesk $n1) 'x'
    $w1 = M2WaitWhere $n1
    Start-Sleep -Milliseconds 600
    ShotAll 'a-fence-on-b'
    Log "new file after the right-click on B ($($at -join ',')): $(M2Fmtw $w1)"
    Check 'the new file goes into Near, the plain fence on B nearest the click' ($w1 -and $w1.fence -eq 'Near') (M2Fmtw $w1)
    $ren = @([W]::Tops('PecoFence.Rename')).Count
    Check 'a file with content gets no name box' ($ren -eq 0) "rename windows=$ren"
    Remove-Item -LiteralPath (Join-Path $PfDesk $n1) -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    # The real menu: 新建 › 文件夹 (English guest: New › Folder), clicked by UI Automation. (新建 ›
    # 文本文档 comes with Notepad, which the Hyper-V runner removes.)
    Case 'A: the desktop menu 新建 › 文件夹 on B opens the name box in Near'
    $before = @(Get-ChildItem -LiteralPath $PfDesk -Directory | ForEach-Object Name)
    [Mon2]::Click($at[0], $at[1], $true); Start-Sleep -Milliseconds 1200
    $okNew = M2MenuClick 'New'
    Start-Sleep -Milliseconds 900
    ShotAll 'a-menu-new-b'
    $okFolder = M2MenuClick 'Folder'
    Check 'setup: the desktop menu took 新建 › 文件夹' ($okNew -and $okFolder) "new=$okNew folder=$okFolder"
    Start-Sleep -Milliseconds 1500
    $made = @(Get-ChildItem -LiteralPath $PfDesk -Directory | ForEach-Object Name | Where-Object { $before -notcontains $_ })
    Log "folders the menu made: $($made -join ', ')"
    if ($made.Count) {
      $w3 = M2WaitWhere $made[0]
      Start-Sleep -Milliseconds 600
      ShotAll 'a-menu-new-folder-b'
      Check 'the menu''s new folder goes into Near' ($w3 -and $w3.fence -eq 'Near') (M2Fmtw $w3)
      $ren = @([W]::Tops('PecoFence.Rename')).Count
      Check 'its name box opens' ($ren -ge 1) "rename windows=$ren; in front: $([W]::Cls([L]::GetForegroundWindow()))"
      if ($ren -ge 1) {
        [G]::Type('pk207 named'); Start-Sleep -Milliseconds 200; [G]::Press(0x0D); Start-Sleep -Milliseconds 1500
        $named = Test-Path -LiteralPath (Join-Path $PfDesk 'pk207 named')
        Check 'a name typed there renames the folder' $named "folders: $((Get-ChildItem -LiteralPath $PfDesk -Directory | ForEach-Object Name) -join ', ')"
        ShotAll 'a-menu-new-folder-b-named'
      }
      CloseDeskMenu $at
      foreach ($m in @($made) + 'pk207 named') { Remove-Item -LiteralPath (Join-Path $PfDesk $m) -Recurse -ErrorAction SilentlyContinue }
    } else {
      CloseDeskMenu $at
    }
    Start-Sleep -Seconds 1

    # The same on A: 「桌面」 takes it under the click (0.5.0, #238).
    Case 'A: a new file after a right-click on the empty desktop of A (as before)'
    $atA = M2Bare $MA.workArea
    [Mon2]::Click($atA[0], $atA[1], $true); Start-Sleep -Milliseconds 900
    CloseDeskMenu $atA
    $n2 = "pk207 a $(Get-Date -Format HHmmss).pk207"
    Set-Content -LiteralPath (Join-Path $PfDesk $n2) 'x'
    $w2 = M2WaitWhere $n2
    $want = "$([int][Math]::Floor(($atA[0] - $fi.windowRect.x - 4) / 80)),$([int][Math]::Floor(($atA[1] - $fi.windowRect.y - 4) / 96))"
    Check 'on A the new file takes the cell under the click in 「桌面」' ($w2 -and $w2.desktop -and $w2.cell -eq $want) "$(M2Fmtw $w2) want cell $want"
    Remove-Item -LiteralPath (Join-Path $PfDesk $n2) -ErrorAction SilentlyContinue
    foreach ($t in 'pk207dir', 'Near', 'Far') { Pf fence delete $t | Out-Null }
    Start-Sleep -Seconds 1
  }
  [IO.File]::WriteAllText("$PfRun\pk207-rules.json", $rulesWere, (New-Object Text.UTF8Encoding $false))
  Pf rule import "$PfRun\pk207-rules.json" | Out-Null
}

if ($Part -match 'B') {
  Case 'B: icons between a fence and 「桌面」 arranged like the desktop (A, recorded)'
  $names = 1..4 | ForEach-Object { "pk203 file $_.pk203" }
  foreach ($n in $names) { Set-Content -LiteralPath (Join-Path $PfDesk $n) 'x' }
  Start-Sleep -Seconds 2
  $wa = $MA.workArea
  Pf fence create --title Src203 --rect "$($wa.x + $wa.w - 520),$($wa.y + 80),400,300" | Out-Null
  Start-Sleep -Seconds 1
  Pf item move --glob 'pk203 file *' --to Src203 | Out-Null
  Start-Sleep -Seconds 2
  $src = FenceGet Src203
  Log "Src203 $(M2Fmt $src.windowRect); items $((@(PfJson item list --fence Src203) | ForEach-Object name) -join ', ')"
  ShotAll 'b-start'
  $rx = [math]::Max(0, $src.windowRect.x - 900)
  RecStart 'b-drags' -X $rx -Y 0 -W 1300 -H 700 -Fps 20
  # One icon out onto 「桌面」.
  RecMark 'one icon from the fence onto 「桌面」'
  $p = ItemPt Src203 'pk203 file 1.pk203'
  $drop = FreeSpot -NearX ($src.windowRect.x - 300) -NearY 300
  if ($p -and $drop) { DragTo $p[0] $p[1] $drop[0] $drop[1] 20 }
  Start-Sleep -Milliseconds 800
  $w = M2Where 'pk203 file 1.pk203'
  $want = "$([int][Math]::Floor(($drop[0] - $fi.windowRect.x - 4) / 80)),$([int][Math]::Floor(($drop[1] - $fi.windowRect.y - 4) / 96))"
  Check 'B: the icon dropped on 「桌面」 stands in the cell under the drop' ($w -and $w.desktop -and $w.cell -eq $want) "$(M2Fmtw $w) want $want"
  # And back into the fence.
  RecMark 'the same icon back into the fence'
  if ($w -and $w.pt) { DragTo $w.pt[0] $w.pt[1] ($src.windowRect.x + 200) ($src.windowRect.y + 200) 20 }
  Start-Sleep -Milliseconds 800
  $w = M2Where 'pk203 file 1.pk203'
  Check 'B: dragged back, the icon is in the fence again' ($w -and $w.fence -eq 'Src203') (M2Fmtw $w)
  # Two icons at once (Ctrl-click the second).
  RecMark 'two icons at once onto 「桌面」'
  $p1 = ItemPt Src203 'pk203 file 2.pk203'; $p2 = ItemPt Src203 'pk203 file 3.pk203'
  [G]::Click($p1[0], $p1[1]); Start-Sleep -Milliseconds 300
  [G]::KeyDown(0x11); [G]::Click($p2[0], $p2[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 300
  $drop2 = FreeSpot -NearX ($src.windowRect.x - 500) -NearY 500
  DragTo $p1[0] $p1[1] $drop2[0] $drop2[1] 20
  Start-Sleep -Milliseconds 800
  $wa2 = M2Where 'pk203 file 2.pk203'; $wb2 = M2Where 'pk203 file 3.pk203'
  Check 'B: both icons of the selection stand on 「桌面」' ($wa2 -and $wa2.desktop -and $wb2 -and $wb2.desktop) "$(M2Fmtw $wa2) | $(M2Fmtw $wb2)"
  # Both back at once.
  RecMark 'both back into the fence'
  if ($wa2.pt -and $wb2.pt) {
    DeskClick 300 900 | Out-Null
    [G]::Click($wa2.pt[0], $wa2.pt[1]); Start-Sleep -Milliseconds 300
    [G]::KeyDown(0x11); [G]::Click($wb2.pt[0], $wb2.pt[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 300
    DragTo $wa2.pt[0] $wa2.pt[1] ($src.windowRect.x + 150) ($src.windowRect.y + 150) 20
  }
  Start-Sleep -Milliseconds 1000
  $wa3 = M2Where 'pk203 file 2.pk203'; $wb3 = M2Where 'pk203 file 3.pk203'
  Check 'B: both are in the fence again' ($wa3 -and $wa3.fence -eq 'Src203' -and $wb3 -and $wb3.fence -eq 'Src203') "$(M2Fmtw $wa3) | $(M2Fmtw $wb3)"
  # One icon moved inside 「桌面」 to another cell.
  RecMark 'one icon from the fence onto 「桌面」, then to another cell there'
  $p = ItemPt Src203 'pk203 file 4.pk203'
  $drop3 = FreeSpot -NearX ($src.windowRect.x - 700) -NearY 250
  DragTo $p[0] $p[1] $drop3[0] $drop3[1] 20
  Start-Sleep -Milliseconds 800
  $w4 = M2Where 'pk203 file 4.pk203'
  if ($w4 -and $w4.pt) { DragTo $w4.pt[0] $w4.pt[1] ($w4.pt[0] + 160) ($w4.pt[1] + 192) 20 }
  Start-Sleep -Milliseconds 800
  $w5 = M2Where 'pk203 file 4.pk203'
  Check 'B: moved inside 「桌面」, the icon takes the new cell' ($w4 -and $w5 -and $w5.desktop -and $w5.cell -ne $w4.cell) "$(M2Fmtw $w4) -> $(M2Fmtw $w5)"
  Start-Sleep -Milliseconds 800
  RecStop
  ShotAll 'b-end'
  foreach ($n in $names) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -ErrorAction SilentlyContinue }
}
