# Owner round 2026-11-12, third batch: what icons do when they appear, move, change and leave.
#   run.sh --var Part=A tests/owner3-ux3-icons.ps1     loose icons (自由摆放): a move by hand, an icon that leaves
#   run.sh --var Part=B ...                            fences made or renamed while the fences hide
#   run.sh --var Part=C ...                            the drag label over the desktop
#   run.sh --var Part=D ...                            tooltips fade
#   --var Rec=1 also records the clips of the report (rec-*).
# Real mouse and keys, zh-CN, a new configuration (「桌面」 arranged like the desktop, 自由摆放).
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
if (-not (Get-Variable Rec -ErrorAction SilentlyContinue)) { $Rec = '' }
. C:\pf\kit\rec.ps1
function UxItems() { return @(@(PfJson item list --fence inbox) | Where-Object { $_ -and $_.id }) }
function UxCells() {
  $h = @{}; $g = FenceGet inbox
  if ($g.desktopCells) { foreach ($p in $g.desktopCells.PSObject.Properties) { $h[$p.Name] = "$($p.Value[0]),$($p.Value[1])" } }
  return $h
}
# The cell of a loose icon by file name or shown name, its top-left corner on screen and the
# point to press it at (100 %: cells of 80 x 96 px, 4 px in from the work area's corner).
function UxCell($name) { $c = UxCells; $it = UxItems | Where-Object { $_.fileName -eq $name -or $_.name -eq $name } | Select-Object -First 1; if ($it) { return $c["$($it.id)"] }; return $null }
function UxCorner($cell) { $p = $cell -split ','; return @([int](4 + [int]$p[0] * 80), [int](4 + [int]$p[1] * 96)) }
function UxPt($name) { $v = UxCell $name; if (-not $v) { return $null }; $c = UxCorner $v; return @(($c[0] + 40), ($c[1] + 30)) }
function UxAll() { $c = UxCells; return ((UxItems | Sort-Object name | ForEach-Object { $n = if ($_.fileName) { $_.fileName } else { $_.name }; "$n=$($c["$($_.id)"])" }) -join ' ; ') }
function UxWait([scriptblock]$ok, $sec = 8) { for ($k = 0; $k -lt $sec * 5; $k++) { if (& $ok) { return $true }; Start-Sleep -Milliseconds 200 }; return $false }
$UxNames = @('季度预算.fpx', '图.fpx', '一个名字特别长的客户往来文件最终版.fpx', 'b.fpx')
function UxClean() { foreach ($n in $UxNames) { Remove-Item -LiteralPath (Join-Path $PfDesk $n) -Force -ErrorAction SilentlyContinue } }
function UxStart() {
  CloseWindows
  UxClean
  foreach ($n in $UxNames) { DeskFile $n | Out-Null }
  [G]::SystemParametersInfoW(0x25, 1, [NullString]::Value, 3) | Out-Null
  $script:UxAnimWas = [Anim]::Get(); [Anim]::Set($true)
  PfStart -Lang 'zh-CN'
  Start-Sleep -Seconds 4
  CloseNotice | Out-Null
}
function UxEnd() { [Anim]::Set($script:UxAnimWas); UxClean }

if ($Part -eq 'A') {
  # 自由摆放: a loose icon moved by hand stood in its cell in the next frame, up to half a cell
  # from where it was let go. It now settles into the cell from there (167 ms), as an icon
  # dropped into a fence does.
  Case 'A loose icon moved by hand settles into its cell'
  UxStart
  $p = UxPt '图.fpx'; DragTo $p[0] $p[1] 1164 322 20; Start-Sleep -Seconds 1
  ParkMouse 1100 700
  $occupied = @((UxCells).Values | Where-Object { $_ -eq '10,3' -or $_ -eq '11,3' -or $_ -eq '10,4' })
  Check 'setup: the loose icon stands at 14,3 and the cells around 10,3 are empty' ((UxCell '图.fpx') -eq '14,3' -and $occupied.Count -eq 0) "图=$(UxCell '图.fpx'); $(UxAll)"
  # Let go with the icon's middle 30 px right of and 30 px below the middle of cell 10,3
  # (x 804 to 884, y 292 to 388; the press is 18 px above the icon's middle).
  if ($Rec) { RecStart 'rec-move' -X 760 -Y 250 -W 520 -H 200 -Fps 30; RecMark '把桌面上的一个图标拖到旁边，松开' }
  $from = UxPt '图.fpx'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  Glide $from[0] $from[1] ($from[0] - 14) ($from[1] + 6) 3 30
  Glide ($from[0] - 14) ($from[1] + 6) 874 352 20 30
  Start-Sleep -Milliseconds 600
  MUp
  # The cell, read as often as the screen can be read for 400 ms and compared with the icon
  # at rest in it (the pointer stays where it is: the drop goes to where it is when the app
  # handles the release): an icon on its way differs for the first 100 ms or so; one that
  # stands in its cell at once does not (the first read may still show the drag picture).
  $sw = [Diagnostics.Stopwatch]::StartNew(); $grabs = @()
  while ($sw.ElapsedMilliseconds -lt 400) { $t = $sw.ElapsedMilliseconds; $grabs += , @($t, [Px]::Grab(804, 292, 80, 96)) }
  Start-Sleep -Milliseconds 900
  $rest = [Px]::Grab(804, 292, 80, 96)
  $reads = @($grabs | ForEach-Object { , @($_[0], [Px]::Diff($rest, $_[1], 0, 79, 0, 95)) })
  $onWay = @($reads | Where-Object { $_[0] -ge 40 -and $_[0] -le 130 -and $_[1] -ge 3.0 })
  if ($Rec) { RecStop }
  ParkMouse 1100 700
  $line = @(PfLogAll | Where-Object { $_ -match 'icons moved by hand settle into their cells' }) | Select-Object -Last 1
  Check 'the icon stands in the cell it was let go over' ((UxCell '图.fpx') -eq '10,3') "cell $(UxCell '图.fpx')"
  Check 'the log says the moved icon glides from where it was let go' ($line -match 'gliding=1') "$line"
  Check 'the icon is seen on its way to the cell between 40 and 130 ms after the release' ($onWay.Count -ge 1) "the cell against the icon at rest (ms:mean brightness change): $(($reads | ForEach-Object { '{0}:{1:N1}' -f $_[0], $_[1] }) -join ' ')"
  # At rest the window is cut to the cells again: the way of the glide and the cell that was
  # left are bare desktop, the new cell is the icon's.
  Start-Sleep -Milliseconds 500
  $way = [W]::RootClassAt(900, 380); $left = [W]::RootClassAt(1164, 322); $now = [W]::RootClassAt(844, 322)
  Check 'after the glide the window covers the new cell alone' ($now -eq 'PecoFence.Fence' -and $way -ne 'PecoFence.Fence' -and $left -ne 'PecoFence.Fence') "new cell: $now; beside it, where the glide passed: $way; the cell that was left: $left"

  # A loose icon whose file is deleted was cut off in one frame (the window was cut to the
  # cells that stay at once); icons inside a fence fade out over 83 ms.
  Case 'A loose icon that leaves fades out'
  $c = UxCorner (UxCell '图.fpx')
  $rest = [Px]::Grab($c[0], $c[1], 80, 96)
  if ($Rec) { RecStart 'rec-leave' -X 760 -Y 250 -W 520 -H 200 -Fps 30; RecMark '删除桌面上的一个文件' }
  Remove-Item -LiteralPath (Join-Path $PfDesk '图.fpx') -Force
  $sw = [Diagnostics.Stopwatch]::StartNew(); $vals = @()
  while ($sw.ElapsedMilliseconds -lt 2500) {
    $t = $sw.ElapsedMilliseconds
    $g = [Px]::Grab($c[0], $c[1], 80, 96); $d = [Px]::Diff($rest, $g, 0, 79, 0, 95); $g.Dispose()
    $vals += , @($t, $d)
  }
  if ($Rec) { RecStop }
  $final = $vals[-1][1]
  $mid = @($vals | Where-Object { $final -gt 3 -and $_[1] -gt 0.15 * $final -and $_[1] -lt 0.85 * $final })
  $firstGone = @($vals | Where-Object { $final -gt 3 -and $_[1] -ge 0.85 * $final }) | Select-Object -First 1
  $around = @($vals | Where-Object { $firstGone -and $_[0] -ge $firstGone[0] - 200 -and $_[0] -le $firstGone[0] + 60 } | ForEach-Object { "{0}:{1:N1}" -f $_[0], $_[1] })
  Check 'setup: the icon is gone from its cell' ($final -gt 3 -and -not (UxCell '图.fpx')) "change of the cell against the icon at rest: $('{0:N1}' -f $final)"
  Check 'the icon is seen at part strength on its way out' ($mid.Count -ge 1) "$($mid.Count) reads between 15 % and 85 % gone; reads around the change (ms:change): $($around -join ' ')"
  Start-Sleep -Milliseconds 600
  $left = [W]::RootClassAt($c[0] + 40, $c[1] + 30)
  Check 'afterwards the cell of the icon that left is bare desktop' ($left -ne 'PecoFence.Fence') "window at the cell: $left"
  UxEnd
}

if ($Part -eq 'B') {
  # The fences are hidden (隐藏所有栅栏, or a double-click on the desktop). A fence made then
  # showed alone, with the tray menu still offering 「显示所有栅栏」.
  Case 'A fence made while the fences are hidden brings them all back'
  UxStart
  $all = @(@(PfJson fence list) | Where-Object { $_ -and $_.id })
  Check 'setup: every fence shows' ((VisFences) -eq $all.Count) "fences $(VisFences) of $($all.Count)"
  Pf fence hide-all | Out-Null
  $hid = UxWait { (VisFences) -eq 0 } 5
  Pf fence create --title '新来的栅栏' --rect '600,300,300,200' | Out-Null
  UxWait { (VisFences) -ge 1 } 4 | Out-Null
  Start-Sleep -Milliseconds 800
  Shot 'B-made-while-hidden'
  $vis = @([W]::Tops('PecoFence.Fence') | ForEach-Object { [W]::Title($_) }) -join ', '
  Check 'after a fence is made while the fences are hidden, every fence shows' ($hid -and (VisFences) -eq $all.Count + 1) "hidden before=$hid; visible now $(VisFences) of $($all.Count + 1): [$vis]"

  # A fence that hides when it is empty gets its first file while the fences are hidden: its
  # window is made then, and it waits with the others.
  Case 'A fence that comes by itself while the fences are hidden stays hidden'
  Pf fence set '新来的栅栏' hideWhenEmpty true | Out-Null
  $gone = UxWait { (VisFences) -eq $all.Count } 5
  Pf fence hide-all | Out-Null
  $hid = UxWait { (VisFences) -eq 0 } 5
  Pf item move 'b.fpx' --to '新来的栅栏' | Out-Null
  Start-Sleep -Milliseconds 1500
  $vis = @([W]::Tops('PecoFence.Fence') | ForEach-Object { [W]::Title($_) }) -join ', '
  Check 'the fence that got a file stays hidden with the others' ($gone -and $hid -and (VisFences) -eq 0) "empty fence without a window=$gone hidden=$hid; visible now: [$vis]"
  Pf fence show-all | Out-Null
  $back = UxWait { (VisFences) -eq $all.Count + 1 } 5
  Check 'shown again, it is there with its file' ($back) "visible $(VisFences) of $($all.Count + 1)"

  # F2 on an icon, then the fences hide without a click: the name box stayed alone on the
  # wallpaper and still renamed the file on Enter.
  Case 'The name box goes when the fences hide'
  $p = UxPt '季度预算.fpx'
  [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 700
  [G]::Press(0x71)
  $box = WaitWin 'PecoFence.Rename' 5
  Pf fence hide-all | Out-Null
  $hid = UxWait { (VisFences) -eq 0 } 5
  $closed = WaitGoneWin 'PecoFence.Rename' 3
  Shot 'B-name-box-after-hide'
  Check 'the open name box closes when the fences hide' ($box -ne [IntPtr]::Zero -and $hid -and $closed) "box opened=$($box -ne [IntPtr]::Zero) fences hidden=$hid box gone=$closed"
  Check 'the file kept its name' (Test-Path -LiteralPath (Join-Path $PfDesk '季度预算.fpx')) ''
  Pf fence show-all | Out-Null
  UxEnd
}

if ($Part -eq 'C') {
  # A loose icon dragged over another loose icon (or its own place) carried 「移动到 桌面」,
  # on the desktop it stands on already; a reorder inside any other fence shows no text.
  Case 'A loose icon dragged over the icons of its own desktop carries no label'
  UxStart
  $p = UxPt '图.fpx'; DragTo $p[0] $p[1] 644 322 20; Start-Sleep -Seconds 1
  ParkMouse 1100 700
  $to = UxPt '季度预算.fpx'
  $x = $to[0]; $y = $to[1]
  # Where the label shows: right of the pointer, clear of the dragged icon's own picture and
  # of the first column.
  $lab = @(($x + 52), ($y - 10), 170, 80)
  $bare = [Px]::Grab($lab[0], $lab[1], $lab[2], $lab[3])
  $from = UxPt '图.fpx'
  [G]::MoveTo($from[0], $from[1]); Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  Glide $from[0] $from[1] ($from[0] - 14) ($from[1] + 6) 3 30
  Glide ($from[0] - 14) ($from[1] + 6) $x $y 20 30
  Start-Sleep -Milliseconds 900
  $g = [Px]::Grab($lab[0], $lab[1], $lab[2], $lab[3]); $d = [Px]::Diff($bare, $g, 0, $lab[2] - 1, 0, $lab[3] - 1); $g.Dispose()
  Crop 'C-loose-over-loose' ([math]::Max(0, $x - 44)) ($y - 80) 320 190 | Out-Null
  # Back over the bare desktop before the release, so the icon keeps a cell of its own.
  Glide $x $y 644 322 12 30
  Start-Sleep -Milliseconds 300
  MUp
  Start-Sleep -Milliseconds 900
  ParkMouse 1100 700
  Check 'no label beside the pointer while a loose icon is over another loose icon' ($d -lt 1.0) "mean brightness change right of the pointer: $('{0:N2}' -f $d) (the label 「移动到 桌面」 gives 5 or more)"
  Check 'the icon is still a loose icon with a cell of its own' ((UxCell '图.fpx') -ne $null) "cell $(UxCell '图.fpx'); $(UxAll)"
  UxEnd
}

# The tip region of a loose icon in the first column: right of its cell, below the pointer.
function UxTipReads($region, $bare, $ms) {
  $sw = [Diagnostics.Stopwatch]::StartNew(); $grabs = @()
  while ($sw.ElapsedMilliseconds -lt $ms) { $t = $sw.ElapsedMilliseconds; $grabs += , @($t, [Px]::Grab($region[0], $region[1], $region[2], $region[3])) }
  return @($grabs | ForEach-Object { , @($_[0], [Px]::Diff($bare, $_[1], 0, $region[2] - 1, 0, $region[3] - 1)) })
}
function UxReadsText($reads, $from, $to) { return (($reads | Where-Object { $_[0] -ge $from -and $_[0] -le $to } | ForEach-Object { '{0}:{1:N1}' -f $_[0], $_[1] }) -join ' ') }

if ($Part -eq 'D') {
  # The tip of an icon was there from one frame to the next and gone the same way. It now
  # fades in and out over 83 ms; with Windows' animations off it still comes and goes at once.
  Case 'The tip of an icon fades in and out'
  UxStart
  $p = UxPt '季度预算.fpx'
  $region = @(86, ($p[1] + 22), 150, 56)
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 600
  $bare = [Px]::Grab($region[0], $region[1], $region[2], $region[3])
  if ($Rec) { RecStart 'rec-tip' -X 0 -Y 250 -W 400 -H 200 -Fps 30; RecMark '把指针停在一个图标上，再移开' }
  [G]::MoveTo($p[0], $p[1])
  $in = UxTipReads $region $bare 1500
  $full = $in[-1][1]
  $first = @($in | Where-Object { $_[1] -gt 0.15 * $full }) | Select-Object -First 1
  $partIn = @($in | Where-Object { $full -gt 3 -and $_[1] -gt 0.15 * $full -and $_[1] -lt 0.85 * $full })
  Check 'setup: the tip shows beside the icon' ($full -gt 3 -and $first -and $first[0] -ge 250) "change of the tip's place: $('{0:N1}' -f $full); first seen after $($first[0]) ms"
  Check 'the tip is seen at part strength while it comes' ($partIn.Count -ge 1) "$($partIn.Count) reads between 15 % and 85 %; reads around its arrival (ms:change): $(UxReadsText $in ($first[0] - 70) ($first[0] + 160))"
  [G]::MoveTo(700, 600)
  $out = UxTipReads $region $bare 700
  $partOut = @($out | Where-Object { $_[1] -gt 0.15 * $full -and $_[1] -lt 0.85 * $full })
  if ($Rec) { RecStop }
  Check 'the tip is seen at part strength while it leaves' ($out[-1][1] -lt 1.0 -and $partOut.Count -ge 1) "$($partOut.Count) reads between 15 % and 85 %; first reads after the pointer left (ms:change): $(UxReadsText $out 0 260)"

  Case 'With Windows'' animations off the tip comes and goes at once'
  [Anim]::Set($false); Start-Sleep -Milliseconds 800
  # Not the very point of the first rest: a move back to the same point counts as no move.
  [G]::MoveTo(($p[0] + 3), ($p[1] + 2))
  $in = UxTipReads $region $bare 1500
  $full = $in[-1][1]
  $first = @($in | Where-Object { $_[1] -gt 0.15 * $full }) | Select-Object -First 1
  $partIn = @($in | Where-Object { $full -gt 3 -and $_[1] -gt 0.15 * $full -and $_[1] -lt 0.85 * $full })
  [G]::MoveTo(700, 600)
  $out = UxTipReads $region $bare 700
  $partOut = @($out | Where-Object { $_[1] -gt 0.15 * $full -and $_[1] -lt 0.85 * $full })
  Check 'no read shows the tip at part strength' ($full -gt 3 -and $partIn.Count -eq 0 -and $partOut.Count -eq 0 -and $out[-1][1] -lt 1.0) "in: $($partIn.Count) part reads, $(UxReadsText $in ($first[0] - 70) ($first[0] + 130)); out: $($partOut.Count) part reads, $(UxReadsText $out 0 200)"
  [Anim]::Set($true)
  UxEnd
}

if ($Part -eq 'P') {
  # For comparison, not a check of PecoFence: how the tip of one of Windows' own desktop icons
  # comes and goes on this box (before PecoFence starts), with Windows' tooltip fade on.
  Case 'Windows'' own desktop tip, recorded'
  Add-Type @'
using System; using System.Runtime.InteropServices;
public static class UxSpi {
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, ref int v, uint f);
  [DllImport("user32.dll")] static extern bool SystemParametersInfoW(uint a, uint b, IntPtr v, uint f);
  public static bool Get(uint a) { int v = 0; SystemParametersInfoW(a, 0, ref v, 0); return v != 0; }
  public static void Set(uint a, bool on) { SystemParametersInfoW(a, 0, (IntPtr)(on ? 1 : 0), 3); }
}
'@
  CloseWindows
  $was = "client area animation=$([Anim]::Get()) tooltip animation=$([UxSpi]::Get(0x1016)) tooltip fade=$([UxSpi]::Get(0x1018))"
  [Anim]::Set($true); [UxSpi]::Set(0x1017, $true); [UxSpi]::Set(0x1019, $true)
  if (-not [W]::IconsVisible()) { [W]::ToggleIcons(); Start-Sleep -Seconds 1 }
  # The Recycle Bin of Windows' own desktop stands at about 36,575 on the boxes.
  $region = @(90, 598, 180, 40)
  [G]::MoveTo(700, 600); Start-Sleep -Milliseconds 800
  $bare = [Px]::Grab($region[0], $region[1], $region[2], $region[3])
  RecStart 'rec-explorer-tip' -X 0 -Y 0 -W 400 -H 200 -Fps 30
  [G]::MoveTo(36, 574)
  $in = UxTipReads $region $bare 2500
  $full = $in[-1][1]
  $first = @($in | Where-Object { $_[1] -gt 0.15 * $full }) | Select-Object -First 1
  $partIn = @($in | Where-Object { $full -gt 3 -and $_[1] -gt 0.15 * $full -and $_[1] -lt 0.85 * $full })
  [G]::MoveTo(700, 600)
  $out = UxTipReads $region $bare 900
  $partOut = @($out | Where-Object { $_[1] -gt 0.15 * $full -and $_[1] -lt 0.85 * $full })
  RecStop
  Shot 'P-desktop'
  Check 'Windows'' tip showed (settings before: see detail)' ($full -gt 3) "$was; change $('{0:N1}' -f $full), first seen after $($first[0]) ms; coming: $($partIn.Count) part reads, $(UxReadsText $in ($first[0] - 70) ($first[0] + 400)); leaving: $($partOut.Count) part reads, $(UxReadsText $out 0 500)"
}
