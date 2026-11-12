# Owner round 2026-11-12, three small things users asked for. One part per call:
#   --var Part=HT   the hover tab switch waits as long as 「展开快慢」 says (linux.do 2998063 #146)
#   --var Rec=before|after with Part=HT records the switch with 快 for the before/after animation.
#   --var Part=DD   what a drag does with no key, Ctrl, Shift, Alt and Ctrl+Shift: File Explorer
#                   against PecoFence, on one volume and across volumes (feedback #153)
#   --var Part=DDT  the same plain drags with Windows' DefaultDropEffect set to move, then to copy
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
  Pf settings set rollUp.hoverSpeed fast | Out-Null; Start-Sleep -Seconds 1
  ParkMouse 1560 820; Start-Sleep -Milliseconds 600
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
