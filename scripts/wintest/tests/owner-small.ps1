# Owner round 2026-11-12, three small things users asked for. One part per call:
#   --var Part=HT   the hover tab switch waits as long as 「展开快慢」 says (linux.do 2998063 #146)
#   --var Rec=before|after with Part=HT records the switch with 快 for the before/after animation.
#   --var Part=DD   what a drag does with no key, Ctrl, Shift, Alt and Ctrl+Shift: File Explorer
#                   against PecoFence, on one volume and across volumes (feedback #153)
#   --var Part=DDT  the same plain drags with Windows' DefaultDropEffect set to move, then to copy
#   --var Part=RD   an icon dragged out of a fence with the right mouse button (feedback #121);
#                   --var Rec=before|after records the drag into a File Explorer window
# "Another volume" in DD and DDT is a share of the box itself, \\localhost\pfdd, made once as
# Administrator (the box has one drive): New-Item -ItemType Directory C:\pfdd-share;
# icacls C:\pfdd-share /grant "pf:(OI)(CI)M"; New-SmbShare -Name pfdd -Path C:\pfdd-share
# -FullAccess Everyone. Without it the rows across volumes fail with "no share".
trap { PfTrap $_; continue }
if (-not $Part) { $Part = 'HT' }

# ---- File Explorer windows and drags (as in fb-file-drop-program.ps1)
function ExOpen($path, $x, $y, $w, $h) {
  Start-Process explorer.exe "`"$path`""
  for ($i = 0; $i -lt 60; $i++) {
    foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) {
      $p = try { $win.Document.Folder.Self.Path } catch { '' }
      if ($p -eq $path) {
        $hw = [IntPtr]$win.HWND
        [L]::MoveWindow($hw, $x, $y, $w, $h, $true) | Out-Null
        Start-Sleep -Milliseconds 1200
        return $hw
      }
    }
    Start-Sleep -Milliseconds 250
  }
  Log "ExOpen: no window for $path"
  return [IntPtr]::Zero
}
# An item of an Explorer window by file name (with or without its extension shown).
function ExItem($hw, $leaf) {
  $root = [Windows.Automation.AutomationElement]::FromHandle($hw)
  $isItem = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
  for ($k = 0; $k -lt 20; $k++) {
    foreach ($n in @($leaf, [IO.Path]::GetFileNameWithoutExtension($leaf))) {
      $byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $n)
      $el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, (New-Object Windows.Automation.AndCondition($isItem, $byName)))
      if ($el) { return $el }
    }
    Start-Sleep -Milliseconds 250
  }
  Log "ExItem: '$leaf' not in the window"
  return $null
}
# The point to press on an Explorer item (its icon and name, at the left of the row).
function ExPt($el) { $r = $el.Current.BoundingRectangle; return @([int]($r.Left + 40), [int](($r.Top + $r.Bottom) / 2)) }
# A drag from one point to another with the left (or right) button. $keys: keys held from the
# first move to the release.
function DragDrop($x1, $y1, $x2, $y2, $keys = @(), [switch]$Right) {
  $down = 2; $up = 4
  if ($Right) { $down = 8; $up = 0x10 }
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 150
  [G]::mouse_event($down, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  Glide $x1 $y1 ($x1 + 14) ($y1 + 14) 3 30
  foreach ($k in $keys) { [G]::KeyDown($k); Start-Sleep -Milliseconds 40 }
  Glide ($x1 + 14) ($y1 + 14) $x2 $y2 16 30
  Start-Sleep -Milliseconds 400
  [G]::MoveTo($x2 + 1, $y2 + 1); Start-Sleep -Milliseconds 350
  [G]::mouse_event($up, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
  foreach ($k in $keys) { [G]::KeyUp($k); Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 500
}
$Mods = [ordered]@{ 'plain' = @(); 'Ctrl' = @(0x11); 'Shift' = @(0x10); 'Alt' = @(0x12); 'Ctrl+Shift' = @(0x11, 0x10) }
# Fresh test files: Chinese names of mixed length, a new one for every drag.
$script:TfN = 0
$script:TfNames = '报告', '季度销售数据汇总表', '会议纪要 2026', '图', '项目计划书（最终版）', '发票', '给客户的报价单 第二版'
function NewLeaf() { $script:TfN++; return "$($script:TfNames[$script:TfN % $script:TfNames.Count]) $($script:TfN).txt" }
# What became of $src after a drop into $destDir: move, copy, link (a shortcut there) or none.
function DropOutcome($src, $destDir, $sec = 4) {
  $leaf = Split-Path $src -Leaf; $base = [IO.Path]::GetFileNameWithoutExtension($leaf)
  $there = $false; $lnk = $false
  for ($k = 0; $k -lt $sec * 4; $k++) {
    $there = Test-Path -LiteralPath (Join-Path $destDir $leaf)
    $lnk = @(Get-ChildItem -LiteralPath $destDir -Filter '*.lnk' -Force -ErrorAction SilentlyContinue | Where-Object { $_.Name -like "*$base*" }).Count -gt 0
    if ($there -or $lnk) { break }
    Start-Sleep -Milliseconds 250
  }
  Start-Sleep -Milliseconds 500
  $here = Test-Path -LiteralPath $src
  if ($lnk) { return 'link' }
  if ($there -and $here) { return 'copy' }
  if ($there) { return 'move' }
  return 'none'
}
# The point of a desktop file's icon in the fence $fence, after putting it there.
function FenceFilePt($fence, $path) {
  $leaf = Split-Path $path -Leaf
  for ($k = 0; $k -lt 20 -and -not (ItemFence $leaf); $k++) { Start-Sleep -Milliseconds 300 }
  if ((ItemFence $leaf) -ne $fence) { Pf item move $path --to $fence | Out-Null; $null = WaitItemFence $leaf $fence 6 }
  $it = @(PfJson item list --fence $fence) | Where-Object { $_ -and $_.fileName -eq $leaf } | Select-Object -First 1
  if (-not $it) { Log "FenceFilePt: '$leaf' not in $fence"; return $null }
  Start-Sleep -Milliseconds 500
  return (ItemPt $fence $it.name)
}
# The empty lower part of a fence.
function FenceFreePt($fence) { $r = (FenceGet $fence).windowRect; return @([int]($r.x + $r.w / 2), [int]($r.y + $r.h - 45)) }

# ---------------------------------------------------------------------------------------------
# HT: with 「鼠标悬停时切换标签页」 on, the tab under a resting pointer shows after the delay of
# 「展开快慢」: 快 150 ms, 默认 Windows' hover time (400 ms), 慢 1 s. It was the Windows hover time
# whatever the setting said. `fence get` reports `fit` only for the tab that is shown, which tells
# when the switch happened.
function HtX($d, $i) { return [int]($d.l + $d.ox + $d.rects[$i][0] + $d.rects[$i][1] / 2) }
function HtY($d) { return [int]($d.t + $d.oy + 18) }
function HtShown($t) { return ($null -ne (FenceGet $t).fit) }
# Rests the pointer on the tab $title and returns the ms until that tab shows (-1: not in $max ms).
function HtHoverMs($d, $title, $max = 3000) {
  $i = [array]::IndexOf(@($d.tabs), $title)
  $x = HtX $d $i; $y = HtY $d
  [G]::MoveTo($x - 6, $y + 2); Start-Sleep -Milliseconds 30
  $sw = [Diagnostics.Stopwatch]::StartNew()
  [G]::MoveTo($x, $y)
  while ($sw.ElapsedMilliseconds -lt $max) { if (HtShown $title) { return [int]$sw.ElapsedMilliseconds } }
  return -1
}
function PartHT {
  trap { PfTrap $_; continue }
  Case 'HT setup: one fence with two tabs'
  CloseWindows
  PfStart -Script (DumpScript 250 10)
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title '常用' --rect '300,200,460,260' | Out-Null
  Pf fence create --title '项目资料与归档' --rect '300,520,300,200' | Out-Null
  Start-Sleep -Seconds 1
  Pf fence merge '项目资料与归档' --into '常用' | Out-Null
  Pf settings set rollUp.switchTabOnHover true | Out-Null
  Start-Sleep -Seconds 2
  ParkMouse 1560 820
  $d = DumpOf '常用'
  Log "dump: $($d.raw)"
  $one = '常用'; $two = '项目资料与归档'
  # How long one `fence get` takes here (the resolution of the times below).
  $sw = [Diagnostics.Stopwatch]::StartNew(); 1..5 | ForEach-Object { $null = HtShown $one }; $per = [int]($sw.ElapsedMilliseconds / 5)
  # Right after the merge both tabs still report a `fit`; one switch each way settles it.
  $null = HtHoverMs $d $two; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
  $null = HtHoverMs $d $one; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
  $null = HtHoverMs $d $two; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
  Check 'HT two tabs, the second one shown after resting on it' ($d.ntabs -eq 2 -and (HtShown $two) -and -not (HtShown $one)) "tabs=$($d.tabs -join ' | '); one fence get takes about $per ms"

  Case 'HT the hover switch waits as long as 展开快慢 says'
  $times = @{}
  foreach ($speed in 'fast', 'normal', 'slow') {
    Pf settings set rollUp.hoverSpeed $speed | Out-Null
    Start-Sleep -Seconds 1
    $ms = @()
    foreach ($k in 1..3) {
      ParkMouse 1560 820; Start-Sleep -Milliseconds 800
      $ms += HtHoverMs $d $one
      ParkMouse 1560 820; Start-Sleep -Milliseconds 800
      $ms += HtHoverMs $d $two
    }
    $times[$speed] = $ms
    Log "hover switch with hoverSpeed=${speed}: $($ms -join ', ') ms"
  }
  $f = $times['fast']; $n = $times['normal']; $s = $times['slow']
  $all = "快: $($f -join ', ') ms; 默认: $($n -join ', ') ms; 慢: $($s -join ', ') ms (one fence get about $per ms)"
  Check 'HT 快: the tab shows 150 ms after the pointer rests on it (100 to 280 ms measured)' (@($f | Where-Object { $_ -lt 100 -or $_ -gt 280 }).Count -eq 0) $all
  Check 'HT 默认: the Windows hover time, 400 ms (300 to 520 ms measured)' (@($n | Where-Object { $_ -lt 300 -or $_ -gt 520 }).Count -eq 0) $all
  Check 'HT 慢: 1 s (900 to 1250 ms measured)' (@($s | Where-Object { $_ -lt 900 -or $_ -gt 1250 }).Count -eq 0) $all

  Case 'HT what must not change'
  # Passing over a tab without resting there switches nothing, also with 快.
  # The pointer stays off the tabs for more than a second first: for a second after a hover
  # switch the next tab waits only 50 ms (owner4-latency.ps1 part NEXT).
  Pf settings set rollUp.hoverSpeed fast | Out-Null; Start-Sleep -Seconds 1
  ParkMouse 1560 820; Start-Sleep -Milliseconds 1400
  $x = HtX $d 0; $y = HtY $d
  [G]::MoveTo($x - 6, $y + 2); Start-Sleep -Milliseconds 30; [G]::MoveTo($x, $y); Start-Sleep -Milliseconds 60
  ParkMouse 1560 820; Start-Sleep -Milliseconds 700
  Check 'HT 快: a pointer that leaves the tab after 60 ms switches nothing' ((HtShown $two) -and -not (HtShown $one)) "shown: one=$(HtShown $one) two=$(HtShown $two)"
  # With the switch off a resting pointer switches nothing.
  Pf settings set rollUp.switchTabOnHover false | Out-Null; Start-Sleep -Seconds 1
  $off = HtHoverMs $d $one 1500
  Check 'HT 「鼠标悬停时切换标签页」 off: resting on a tab for 1.5 s switches nothing' ($off -eq -1 -and (HtShown $two)) "ms=$off"
  ParkMouse 1560 820
  # A click still switches at once.
  [G]::Click((HtX $d 0), (HtY $d)); Start-Sleep -Milliseconds 600
  Check 'HT a click on the tab still switches' (HtShown $one) "shown: one=$(HtShown $one)"
  ParkMouse 1560 820

  if ($Rec) {
    Case "HT recording ($Rec)"
    . C:\pf\kit\rec.ps1
    $animWas = [Anim]::Get(); [Anim]::Set($true)
    Pf settings set rollUp.switchTabOnHover true | Out-Null
    Pf settings set rollUp.hoverSpeed fast | Out-Null; Start-Sleep -Seconds 1
    $null = HtHoverMs $d $two; ParkMouse 1560 820; Start-Sleep -Milliseconds 800
    RecStart "HT-$Rec" -X 260 -Y 170 -W 540 -H 330 -Fps 30
    RecMark '「展开快慢」选了「快」。鼠标停到另一个标签上'
    Start-Sleep -Milliseconds 400
    $ms = HtHoverMs $d $one
    RecMetric $ms
    RecMark "标签页切换过来了，等了 $ms 毫秒"
    Start-Sleep -Milliseconds 1200
    RecStop
    [Anim]::Set($animWas)
    Check "HT recorded the switch with 快 ($Rec)" ($ms -gt 0) "ms=$ms"
    ParkMouse 1560 820
  }
  Pf settings set rollUp.hoverSpeed normal | Out-Null
  Pf settings set rollUp.switchTabOnHover false | Out-Null
}
# ---------------------------------------------------------------------------------------------
# DD, DDT: the effect of a file drag. Sources: a File Explorer window, an icon in a fence (a
# desktop file). Targets: a File Explorer window, a fence (its files land on the desktop), a
# folder fence. One volume = all on C:, another volume = one side on the share.
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class DdShell { [DllImport("shell32.dll")] public static extern void SHChangeNotify(int e, uint f, IntPtr a, IntPtr b); }
'@
# Windows' DefaultDropEffect for files and folders of this user (0 removes it; 1 copy, 2 move).
function DdSetDefault($value) {
  $classes = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Software\Classes', $true)
  foreach ($name in '*', 'AllFilesystemObjects') {
    if ($value) {
      $k = $classes.CreateSubKey($name); $k.SetValue('DefaultDropEffect', [int]$value, 'DWord'); $k.Close()
    } else {
      $k = $classes.OpenSubKey($name, $true)
      if ($k) {
        $k.DeleteValue('DefaultDropEffect', $false)
        $empty = ($k.ValueCount -eq 0 -and $k.SubKeyCount -eq 0); $k.Close()
        if ($empty) { $classes.DeleteSubKey($name, $false) }
      }
    }
  }
  $classes.Close()
  [DdShell]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)
  Start-Sleep -Seconds 2
}
function DdSetup {
  trap { PfTrap $_; continue }
  CloseWindows
  $script:DdShare = '\\localhost\pfdd'
  $script:DdHaveShare = Test-Path $script:DdShare
  $script:DdDirs = @{ srcC = "$PfRun\dd-src"; dstC = "$PfRun\dd-dst"; portC = "$PfRun\dd-portal" }
  if ($script:DdHaveShare) { $script:DdDirs.srcS = "$script:DdShare\dd-src"; $script:DdDirs.dstS = "$script:DdShare\dd-dst"; $script:DdDirs.portS = "$script:DdShare\dd-portal" }
  foreach ($d in $script:DdDirs.Values) { Remove-Item -Recurse -Force $d -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force $d | Out-Null }
  PfStart
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title 'Src' --rect '1140,40,220,240' | Out-Null
  Pf fence create --title 'Dst' --rect '1370,40,220,240' | Out-Null
  Pf fence create --portal $script:DdDirs.portC --title 'PortalC' --rect '1140,300,220,240' | Out-Null
  if ($script:DdHaveShare) { Pf fence create --portal $script:DdDirs.portS --title 'PortalS' --rect '1370,300,220,240' | Out-Null }
  Start-Sleep -Seconds 2
  Pf fence set Src sort name | Out-Null
  $E = @{}
  $E.exSrcC = @{ kind = 'ex'; dir = $script:DdDirs.srcC; hw = (ExOpen $script:DdDirs.srcC 0 0 560 400) }
  $E.exDstC = @{ kind = 'ex'; dir = $script:DdDirs.dstC; hw = (ExOpen $script:DdDirs.dstC 0 410 560 400); pt = @(420, 720) }
  if ($script:DdHaveShare) {
    $E.exSrcS = @{ kind = 'ex'; dir = $script:DdDirs.srcS; hw = (ExOpen $script:DdDirs.srcS 565 0 560 400) }
    $E.exDstS = @{ kind = 'ex'; dir = $script:DdDirs.dstS; hw = (ExOpen $script:DdDirs.dstS 565 410 560 400); pt = @(985, 720) }
    $E.portS = @{ kind = 'fence'; dir = $script:DdDirs.portS; pt = (FenceFreePt 'PortalS') }
  }
  # The list view: a long name scrolls the details view sideways and takes the name out of reach.
  foreach ($w in @((New-Object -ComObject Shell.Application).Windows())) { try { $w.Document.CurrentViewMode = 3 } catch { Log "view mode: $_" } }
  Start-Sleep -Milliseconds 800
  $E.fenceSrc = @{ kind = 'fence'; dir = $PfDesk; fence = 'Src' }
  $E.fenceDst = @{ kind = 'fence'; dir = $PfDesk; pt = (FenceFreePt 'Dst') }
  $E.portC = @{ kind = 'fence'; dir = $script:DdDirs.portC; pt = (FenceFreePt 'PortalC') }
  $script:DdEnds = $E
  ParkMouse 1300 700
  Shot 'dd-setup'
  $wins = @($E.Values | Where-Object { $_.kind -eq 'ex' -and $_.hw -ne [IntPtr]::Zero }).Count
  Check 'DD setup: the folders, their File Explorer windows and four fences' ($wins -eq $(if ($script:DdHaveShare) { 4 } else { 2 }) -and (FenceGet 'PortalC').title -eq 'PortalC') "share=$script:DdHaveShare windows=$wins fences=$((@(PfJson fence list) | Where-Object { $_ -and $_.title } | ForEach-Object { $_.title }) -join ', ')"
}
function DdCleanup {
  trap { PfTrap $_; continue }
  CloseWindows
  foreach ($d in $script:DdDirs.Values) { Remove-Item -Recurse -Force $d -ErrorAction SilentlyContinue }
  Get-ChildItem -LiteralPath $PfDesk -Force | Where-Object { $_.Name -match ' \d+\.txt( - Shortcut)?(\.lnk)?$' } | Remove-Item -Force -ErrorAction SilentlyContinue
}
# One drag of a fresh file from the end $from to the end $to with $keys held.
function DdOne($fromName, $toName, $keys) {
  trap { PfTrap $_; continue }
  $from = $script:DdEnds[$fromName]; $to = $script:DdEnds[$toName]
  if (-not $from -or -not $to) { return 'no share' }
  $leaf = NewLeaf; $base = [IO.Path]::GetFileNameWithoutExtension($leaf)
  $src = Join-Path $from.dir $leaf
  Set-Content -LiteralPath $src -Value 'dd' -Encoding ASCII
  $p = $null
  if ($from.kind -eq 'ex') { $el = ExItem $from.hw $leaf; if ($el) { $p = ExPt $el } } else { $p = FenceFilePt $from.fence $src }
  $r = 'no source'
  if ($p) {
    DragDrop $p[0] $p[1] $to.pt[0] $to.pt[1] $keys
    $r = DropOutcome $src $to.dir
  }
  Log "dd $fromName -> $toName keys=[$($keys -join ',')] '$leaf': $r"
  # Nothing of this drag stays for the next one.
  Get-ChildItem -LiteralPath $to.dir -Force | Where-Object { $_.Name -like "*$base*" } | Remove-Item -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $src -Force -ErrorAction SilentlyContinue
  [G]::Press(0x1B); Start-Sleep -Milliseconds 200
  return $r
}
function DdRow($fromName, $toName, $modNames) {
  $cells = @()
  foreach ($m in $modNames) { $cells += "$m=$(DdOne $fromName $toName $Mods[$m])" }
  return ($cells -join ' ')
}
$script:DdRowsOne = @(
  @('File Explorer to File Explorer', 'exSrcC', 'exDstC'),
  @('fence to File Explorer', 'fenceSrc', 'exDstC'),
  @('File Explorer to fence', 'exSrcC', 'fenceDst'),
  @('fence to folder fence', 'fenceSrc', 'portC'))
$script:DdRowsOther = @(
  @('File Explorer to File Explorer', 'exSrcC', 'exDstS'),
  @('fence to File Explorer', 'fenceSrc', 'exDstS'),
  @('File Explorer to fence', 'exSrcS', 'fenceDst'),
  @('fence to folder fence', 'fenceSrc', 'portS'))
# Runs the rows; the first row is File Explorer's own answer, which the others must give too.
function DdGroup($title, $rows, $modNames, $want) {
  trap { PfTrap $_; continue }
  $ref = DdRow $rows[0][1] $rows[0][2] $modNames
  Check "DD $title, $($rows[0][0]): $want" ($ref -eq $want) $ref
  foreach ($row in $rows[1..3]) {
    $got = DdRow $row[1] $row[2] $modNames
    Check "DD $title, $($row[0]): as File Explorer does" ($got -eq $ref) "$got (File Explorer: $ref)"
  }
}
function PartDD {
  trap { PfTrap $_; continue }
  Case 'DD setup'
  DdSetDefault 0
  DdSetup
  $all = @($Mods.Keys)
  Case 'DD Windows as installed, one volume'
  DdGroup 'one volume' $script:DdRowsOne $all 'plain=move Ctrl=copy Shift=move Alt=link Ctrl+Shift=link'
  Case 'DD Windows as installed, another volume'
  DdGroup 'another volume' $script:DdRowsOther $all 'plain=copy Ctrl=copy Shift=move Alt=link Ctrl+Shift=link'
  DdCleanup
}
function PartDDT {
  trap { PfTrap $_; continue }
  Case 'DDT setup'
  DdSetDefault 0
  DdSetup
  Case 'DDT DefaultDropEffect = 2 (move)'
  DdSetDefault 2
  DdGroup 'move by default, another volume' $script:DdRowsOther @('plain', 'Ctrl', 'Alt') 'plain=move Ctrl=copy Alt=link'
  DdGroup 'move by default, one volume' $script:DdRowsOne @('plain') 'plain=move'
  Case 'DDT DefaultDropEffect = 1 (copy)'
  DdSetDefault 1
  DdGroup 'copy by default, one volume' $script:DdRowsOne @('plain', 'Shift', 'Alt') 'plain=copy Shift=move Alt=link'
  DdGroup 'copy by default, another volume' $script:DdRowsOther @('plain') 'plain=copy'
  Case 'DDT cleanup'
  DdSetDefault 0
  $left = (Get-ItemProperty -LiteralPath 'Registry::HKEY_CURRENT_USER\Software\Classes\AllFilesystemObjects' -Name DefaultDropEffect -ErrorAction SilentlyContinue).DefaultDropEffect
  Check 'DDT DefaultDropEffect is removed again' ($null -eq $left) "left=$left"
  DdCleanup
}
if ($Part -eq 'HT') { PartHT }
if ($Part -eq 'DD') { PartDD }
if ($Part -eq 'DDT') { PartDDT }
# DDR: one plain drag from the other volume into a fence with DefaultDropEffect = 2, recorded
# for the before/after animation (--var Rec=before|after).
function PartDDR {
  trap { PfTrap $_; continue }
  Case 'DDR setup'
  DdSetDefault 0
  DdSetup
  DdSetDefault 2
  . C:\pf\kit\rec.ps1
  $leaf = '季度销售数据汇总表.txt'; $src = Join-Path $script:DdDirs.srcS $leaf
  Set-Content -LiteralPath $src -Value 'dd' -Encoding ASCII
  $el = ExItem $script:DdEnds.exSrcS.hw $leaf; $p = ExPt $el; $q = $script:DdEnds.fenceDst.pt
  Case "DDR recording ($Rec)"
  RecStart "DD-$Rec" -X 565 -Y 0 -W 1035 -H 420 -Fps 15
  RecMark '另一个盘的文件夹里有一个文件'
  Start-Sleep -Milliseconds 900
  RecMark '不按任何键，把它拖进栅栏'
  DragDrop $p[0] $p[1] $q[0] $q[1]
  $r = DropOutcome $src $PfDesk
  RecMark '松开之后'
  ParkMouse 1300 700
  Start-Sleep -Seconds 2
  RecStop
  Check "DDR a plain drag from another volume into a fence, DefaultDropEffect = 2: the file moves ($Rec)" ($r -eq 'move') "outcome=$r"
  DdSetDefault 0
  Remove-Item -LiteralPath (Join-Path $PfDesk $leaf) -Force -ErrorAction SilentlyContinue
  DdCleanup
}
if ($Part -eq 'DDR') { PartDDR }

# ---------------------------------------------------------------------------------------------
# RD: an icon dragged out of a fence with the right mouse button (feedback #121). It did nothing;
# now it is the drag the left button starts, and the target shows its menu at the drop.
# The menu on screen as text, its default entry in [ ]; 'no menu' when none opened.
function RdMenu($sec = 3) {
  $m = MenuWin $sec
  if ($m -eq [IntPtr]::Zero) { return @{ h = $m; text = 'no menu' } }
  $text = (@(MenuRows $m) | Where-Object { $_.text } | ForEach-Object { if ($_.state -band 0x1000) { "[$($_.text)]" } else { $_.text } }) -join ' | '
  return @{ h = $m; text = $text }
}
# A fresh desktop file in the fence 'Src'; returns its path and the point of its icon.
function RdFile() {
  $src = Join-Path $PfDesk (NewLeaf)
  Set-Content -LiteralPath $src -Value 'rd' -Encoding ASCII
  return @{ path = $src; leaf = (Split-Path $src -Leaf); pt = (FenceFilePt 'Src' $src) }
}
# A right-button drag of a fresh file from 'Src' to the point $q; returns the file and the menu.
function RdDrag($q) {
  $f = RdFile
  if (-not $f.pt) { $f.menu = @{ h = [IntPtr]::Zero; text = 'no source' }; return $f }
  DragDrop $f.pt[0] $f.pt[1] $q[0] $q[1] -Right
  $f.menu = RdMenu 3
  return $f
}
# Picks the entry matching $regex (or Esc without one) and waits for the file work.
function RdPick($menu, $regex) {
  if ($menu.h -eq [IntPtr]::Zero) { return }
  if ($regex) { $null = MenuPick $regex $menu.h } else { [G]::Press(0x1B) }
  Start-Sleep -Milliseconds 900
}
function PartRD {
  trap { PfTrap $_; continue }
  Case 'RD setup'
  CloseWindows
  $dir = "$PfRun\rd-target"; $out = "$PfRun\rd-outside"
  foreach ($d in $dir, $out) { Remove-Item -Recurse -Force $d -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force $d | Out-Null }
  $deskFolder = '桌面上的资料夹'; $fenceFolder = '归档'
  foreach ($n in $deskFolder, $fenceFolder) { Remove-Item -Recurse -Force (Join-Path $PfDesk $n) -ErrorAction SilentlyContinue }
  PfStart
  Start-Sleep -Seconds 3
  ClearFences
  Pf fence create --title 'Src' --rect '1140,40,220,300' | Out-Null
  Pf fence create --title 'Dst' --rect '1370,40,220,300' | Out-Null
  Start-Sleep -Seconds 1
  Pf fence set Src sort name | Out-Null; Pf fence set Dst sort name | Out-Null
  foreach ($n in $deskFolder, $fenceFolder) { New-Item -ItemType Directory -Force (Join-Path $PfDesk $n) | Out-Null }
  for ($k = 0; $k -lt 20 -and -not ((ItemFence $deskFolder) -and (ItemFence $fenceFolder)); $k++) { Start-Sleep -Milliseconds 300 }
  Pf item move (Join-Path $PfDesk $fenceFolder) --to Dst | Out-Null
  $null = WaitItemFence $fenceFolder 'Dst' 6
  $hw = ExOpen $dir 560 380 560 420
  foreach ($w in @((New-Object -ComObject Shell.Application).Windows())) { try { $w.Document.CurrentViewMode = 3 } catch { Log "view mode: $_" } }
  Start-Sleep -Milliseconds 800
  $exPt = @(980, 700)
  ParkMouse 1300 700
  $inbox = ItemFence $deskFolder
  Shot 'rd-setup'
  Check 'RD setup: two fences, a folder in 「桌面」 and one in a fence, a File Explorer window' ($hw -ne [IntPtr]::Zero -and $inbox -and $inbox -ne 'Dst' -and (ItemFence $fenceFolder) -eq 'Dst') "explorer=$hw desktop folder in '$inbox', fence folder in '$(ItemFence $fenceFolder)'"

  Case 'RD from a fence to a File Explorer window'
  $f = RdDrag $exPt
  Shot 'rd-explorer-menu'
  RdPick $f.menu '^Move here$'
  $r = DropOutcome $f.path $dir
  Check 'RD File Explorer shows its menu (Copy here, Move here, Create shortcuts here, Cancel) and Move here moves the file' ($f.menu.text -match 'Copy here' -and $f.menu.text -match 'Move here' -and $f.menu.text -match 'Create shortcuts here' -and $f.menu.text -match 'Cancel' -and $r -eq 'move') "menu: $($f.menu.text); outcome=$r"
  $f = RdDrag $exPt
  RdPick $f.menu '^Cancel$'
  $r = DropOutcome $f.path $dir 2
  Check 'RD Cancel in that menu: the file stays in its fence' ($f.menu.text -match 'Cancel' -and $r -eq 'none' -and (ItemFence $f.leaf) -eq 'Src') "menu: $($f.menu.text); outcome=$r fence='$(ItemFence $f.leaf)'"
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue

  Case 'RD from a fence to a folder icon'
  $f = RdDrag (ItemPt 'inbox' $deskFolder)
  Shot 'rd-desktop-folder-menu'
  RdPick $f.menu 'Copy here'
  $r = DropOutcome $f.path (Join-Path $PfDesk $deskFolder)
  Check 'RD a folder in 「桌面」: the menu, and Copy here copies the file into it' ($f.menu.text -match 'Move here' -and $f.menu.text -match 'Copy here' -and $f.menu.text -match 'Cancel' -and $r -eq 'copy' -and (ItemFence $f.leaf) -eq 'Src') "menu: $($f.menu.text); outcome=$r fence='$(ItemFence $f.leaf)'"
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue
  $f = RdDrag (ItemPt 'Dst' $fenceFolder)
  Shot 'rd-fence-folder-menu'
  RdPick $f.menu 'Move here'
  $r = DropOutcome $f.path (Join-Path $PfDesk $fenceFolder)
  Check 'RD a folder in another fence: the menu, and Move here moves the file into it' ($f.menu.text -match 'Move here' -and $f.menu.text -match 'Copy here' -and $f.menu.text -match 'Cancel' -and $r -eq 'move') "menu: $($f.menu.text); outcome=$r"
  $f = RdDrag (ItemPt 'Dst' $fenceFolder)
  RdPick $f.menu '^Cancel$'
  $r = DropOutcome $f.path (Join-Path $PfDesk $fenceFolder) 2
  Check 'RD Cancel on a folder in a fence: the file stays in its fence' ($f.menu.text -match 'Cancel' -and $r -eq 'none' -and (ItemFence $f.leaf) -eq 'Src') "menu: $($f.menu.text); outcome=$r fence='$(ItemFence $f.leaf)'"
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue

  # The report's own case: onto a shortcut that points to a folder.
  $lnk = Join-Path $PfDesk 'rd target link.lnk'
  $sc = (New-Object -ComObject WScript.Shell).CreateShortcut($lnk); $sc.TargetPath = $dir; $sc.Save()
  for ($k = 0; $k -lt 20 -and -not (ItemFence 'rd target link.lnk'); $k++) { Start-Sleep -Milliseconds 300 }
  Pf item move $lnk --to Dst | Out-Null
  $null = WaitItemFence 'rd target link.lnk' 'Dst' 6
  Start-Sleep -Milliseconds 600
  $f = RdDrag (ItemPt 'Dst' 'rd target link')
  Shot 'rd-shortcut-menu'
  RdPick $f.menu 'Move here'
  $r = DropOutcome $f.path $dir
  Check 'RD a shortcut to a folder in another fence: the menu, and Move here moves the file into the folder it points to' ($f.menu.text -match 'Move here' -and $f.menu.text -match 'Cancel' -and $r -eq 'move' -and (ItemFence 'rd target link.lnk') -eq 'Dst') "menu: $($f.menu.text); outcome=$r; the shortcut is in '$(ItemFence 'rd target link.lnk')'"
  Remove-Item -LiteralPath $lnk -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 1

  Case 'RD from a fence to a fence'
  $f = RdDrag (FenceFreePt 'Dst')
  $to = WaitItemFence $f.leaf 'Dst' 5
  Check 'RD another fence: the icon moves there, as with the left button, without a menu' ($f.menu.text -eq 'no menu' -and $to -eq 'Dst' -and (Test-Path -LiteralPath $f.path)) "menu: $($f.menu.text); fence='$to'"
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue
  $f = RdDrag (FenceFreePt 'Src')
  Start-Sleep -Milliseconds 800
  Check 'RD its own fence: nothing changes and no menu opens' ($f.menu.text -eq 'no menu' -and (ItemFence $f.leaf) -eq 'Src' -and (Test-Path -LiteralPath $f.path)) "menu: $($f.menu.text); fence='$(ItemFence $f.leaf)'"
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue

  Case 'RD from a fence to the Recycle Bin and to the bare desktop'
  $f = RdDrag (ItemPt 'inbox' 'Recycle Bin')
  Shot 'rd-bin-menu'
  RdPick $f.menu 'Move here'
  Start-Sleep -Seconds 2
  $base = [IO.Path]::GetFileNameWithoutExtension($f.leaf)
  Check 'RD the Recycle Bin: Move here and Cancel, and Move here recycles the file' ($f.menu.text -match 'Move here' -and $f.menu.text -notmatch 'Copy here' -and $f.menu.text -match 'Cancel' -and -not (Test-Path -LiteralPath $f.path) -and (BinNames) -match [regex]::Escape($base)) "menu: $($f.menu.text); on the desktop=$(Test-Path -LiteralPath $f.path); in the bin=$((BinNames) -match [regex]::Escape($base))"
  $spot = FreeSpot 1250 600
  $f = RdDrag $spot
  $to = WaitItemFence $f.leaf $inbox 5
  Check 'RD the bare desktop: the icon goes to 「桌面」, as with the left button' ($f.menu.text -eq 'no menu' -and $to -eq $inbox -and (Test-Path -LiteralPath $f.path)) "menu: $($f.menu.text); fence='$to' spot=$($spot -join ',')"
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue

  Case 'RD Esc ends a right-button drag'
  $f = RdFile
  [G]::MoveTo($f.pt[0], $f.pt[1]); Start-Sleep -Milliseconds 150
  [G]::mouse_event(8, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  Glide $f.pt[0] $f.pt[1] $exPt[0] $exPt[1] 16 30
  [G]::Press(0x1B); Start-Sleep -Milliseconds 400
  [G]::mouse_event(0x10, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 800
  $m = RdMenu 2
  $r = DropOutcome $f.path $dir 1
  Check 'RD Esc during the drag: no menu, nothing moves' ($m.text -eq 'no menu' -and $r -eq 'none' -and (ItemFence $f.leaf) -eq 'Src') "menu: $($m.text); outcome=$r fence='$(ItemFence $f.leaf)'"
  CloseMenus
  # The left button pressed during the drag ends it too (as the right one ends a left drag).
  [G]::MoveTo($f.pt[0], $f.pt[1]); Start-Sleep -Milliseconds 150
  [G]::mouse_event(8, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  Glide $f.pt[0] $f.pt[1] $exPt[0] $exPt[1] 16 30
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
  [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  [G]::mouse_event(0x10, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 800
  $m = RdMenu 2
  $r = DropOutcome $f.path $dir 1
  Check 'RD the left button pressed during the drag: no menu, nothing moves' ($m.text -eq 'no menu' -and $r -eq 'none' -and (ItemFence $f.leaf) -eq 'Src') "menu: $($m.text); outcome=$r fence='$(ItemFence $f.leaf)'"
  CloseMenus; [G]::Press(0x1B); Start-Sleep -Milliseconds 300

  Case 'RD what must not change'
  # Rename by a slow second click on the name of the selected icon (the left button).
  [G]::Click($f.pt[0], $f.pt[1]); Start-Sleep -Milliseconds 900
  [G]::Click($f.pt[0], $f.pt[1] + 38)
  $box = [IntPtr]::Zero
  for ($k = 0; $k -lt 60 -and $box -eq [IntPtr]::Zero; $k++) { Start-Sleep -Milliseconds 50; $h = [G]::FindWindowW('PecoFence.Rename', [NullString]::Value); if ($h -ne [IntPtr]::Zero -and [G]::IsWindowVisible($h)) { $box = $h } }
  Shot 'rd-slow-click-rename'
  Check 'RD a slow second click on the selected icon opens the rename box' ($box -ne [IntPtr]::Zero) "rename box=$box"
  [G]::Press(0x1B); Start-Sleep -Milliseconds 500
  # A plain right-click: the icon's menu. With a few pixels of movement inside the drag
  # threshold too.
  [G]::RClick($f.pt[0], $f.pt[1])
  $m = RdMenu 6
  Shot 'rd-item-menu'
  Check 'RD a right-click on an icon opens its menu' ($m.text -match 'Rename' -and $m.text -match 'Properties') "menu: $($m.text)"
  CloseMenus; Start-Sleep -Milliseconds 400
  [G]::MoveTo($f.pt[0], $f.pt[1]); Start-Sleep -Milliseconds 150
  [G]::mouse_event(8, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 120
  [G]::MoveTo($f.pt[0] + 2, $f.pt[1] + 1); Start-Sleep -Milliseconds 120
  [G]::mouse_event(0x10, 0, 0, 0, [UIntPtr]::Zero)
  $m = RdMenu 6
  Check 'RD a right-click that moves 2 px still opens the menu' ($m.text -match 'Rename' -and $m.text -match 'Properties') "menu: $($m.text)"
  CloseMenus; Start-Sleep -Milliseconds 400
  # On the empty part of a fence: the fence menu; a right-button drag there selects nothing
  # (a fence has no marquee with the right button) and opens nothing.
  $free = FenceFreePt 'Src'
  [G]::RClick($free[0], $free[1])
  $m = RdMenu 6
  Check 'RD a right-click on the empty part of a fence opens the fence menu' ($m.h -ne [IntPtr]::Zero -and $m.text -notmatch 'Properties') "menu: $($m.text)"
  CloseMenus; Start-Sleep -Milliseconds 400
  DragDrop $free[0] $free[1] ($free[0] - 60) ($free[1] - 60) -Right
  $m = RdMenu 2
  $sel = @(@(PfJson item list --fence Src) | Where-Object { $_ -and $_.selected }).Count
  Check 'RD a right-button drag that starts on empty space: no menu, nothing selected' ($m.text -eq 'no menu' -and $sel -eq 0) "menu: $($m.text); selected=$sel"
  CloseMenus
  Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue

  Case 'RD from File Explorer into a fence with the right button (as before)'
  $leaf = NewLeaf; $src = Join-Path $out $leaf
  Set-Content -LiteralPath $src -Value 'rd' -Encoding ASCII
  $hw2 = ExOpen $out 560 380 560 420
  foreach ($w in @((New-Object -ComObject Shell.Application).Windows())) { try { $w.Document.CurrentViewMode = 3 } catch { Log "view mode: $_" } }
  Start-Sleep -Milliseconds 800
  $el = ExItem $hw2 $leaf
  $p = ExPt $el; $q = FenceFreePt 'Dst'
  DragDrop $p[0] $p[1] $q[0] $q[1] -Right
  $m = RdMenu 3
  RdPick $m 'Move here'
  $r = DropOutcome $src $PfDesk
  $to = WaitItemFence $leaf 'Dst' 5
  Check 'RD a file from File Explorer: the menu, and Move here moves it into the fence' ($m.text -match 'Move here' -and $m.text -match 'Copy here' -and $r -eq 'move' -and $to -eq 'Dst') "menu: $($m.text); outcome=$r fence='$to'"
  Remove-Item -LiteralPath (Join-Path $PfDesk $leaf) -Force -ErrorAction SilentlyContinue

  if ($Rec) {
    Case "RD recording ($Rec)"
    . C:\pf\kit\rec.ps1
    CloseWindows
    $hw = ExOpen $dir 560 380 560 420
    foreach ($w in @((New-Object -ComObject Shell.Application).Windows())) { try { $w.Document.CurrentViewMode = 3 } catch { Log "view mode: $_" } }
    Start-Sleep -Milliseconds 800
    $f = RdFile
    ParkMouse 1300 700
    RecStart "RD-$Rec" -X 560 -Y 30 -W 810 -H 780 -Fps 15
    RecMark '栅栏里有一个文件'
    Start-Sleep -Milliseconds 900
    RecMark '按住鼠标右键，把它拖到资源管理器的窗口里，松开'
    DragDrop $f.pt[0] $f.pt[1] $exPt[0] $exPt[1] -Right
    $m = RdMenu 3
    Start-Sleep -Milliseconds 1200
    RecMark '松开之后'
    RdPick $m '^Move here$'
    $r = DropOutcome $f.path $dir 2
    Start-Sleep -Seconds 2
    RecStop
    Check "RD recorded ($Rec)" $true "menu: $($m.text); outcome=$r"
    Remove-Item -LiteralPath $f.path -Force -ErrorAction SilentlyContinue
  }

  Case 'RD cleanup'
  CloseMenus
  CloseWindows
  foreach ($d in $dir, $out, (Join-Path $PfDesk $deskFolder), (Join-Path $PfDesk $fenceFolder)) { Remove-Item -Recurse -Force $d -ErrorAction SilentlyContinue }
  Get-ChildItem -LiteralPath $PfDesk -Force | Where-Object { $_.Name -match ' \d+\.txt( - Shortcut)?(\.lnk)?$' } | Remove-Item -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath (Join-Path $PfDesk 'rd target link.lnk') -Force -ErrorAction SilentlyContinue
  Clear-RecycleBin -Force -ErrorAction SilentlyContinue
}
if ($Part -eq 'RD') { PartRD }
