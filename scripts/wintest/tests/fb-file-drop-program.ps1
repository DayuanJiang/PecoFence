# #198 point 2: files dragged onto a program in a fence open with it, as on the Windows desktop.
# Every source x every target x every item mix, with new files for every drop.
#   run.sh --var Part=X tests/fb-file-drop-program.ps1   what File Explorer itself does (no PecoFence)
#   --var Part=A   from another fence            --var Part=B   from 「桌面」 arranged like the desktop
#   --var Part=C   from a File Explorer window    --var Part=D   Ctrl / Shift / Alt / right-button drags
#   --var Part=N   Notepad opens a .txt dropped on its shortcut, from every source
#   --var Part=R   the before/after recording
#   --var Build=old  the build before #198: files onto a program go into the fence (one folder opens)
# A copy of this file named fb-file-drop-program-<Part>.ps1 runs that part, so one run.sh call
# can run several parts.
# The test program records what it got: an .exe compiled here (its shortcut keeps a saved argument
# and a start folder) and a .bat; the dragged items must stay where they were.
if ($PfSteps -cmatch '-([A-Z])\.ps1$') { $Part = $Matches[1] }
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
if (-not (Get-Variable Build -ErrorAction SilentlyContinue)) { $Build = 'new' }
$New = $Build -ne 'old'
$Fd = "$PfRun\fdp"
$Recv = "$Fd\recv.txt"
$BatRecv = "$Fd\bat.txt"
$Into = "$Fd\into"
$SrcDir = "$Fd\src"
$Wd = "$Fd\wd"
$Cap = "$Fd\bin\cap.exe"
$made = New-Object System.Collections.ArrayList
function Made($p) { [void]$made.Add($p); return $p }
$script:Tag = 0

Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public static class LP {
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern uint GetLongPathNameW(string s, StringBuilder l, uint n);
  public static string Long(string p) { var sb = new StringBuilder(1024); return GetLongPathNameW(p, sb, 1024) > 0 ? sb.ToString() : p; }
}
'@
function LongP($p) { return ([LP]::Long("$p")).TrimEnd('\').ToLowerInvariant() }

# ---- the test programs and targets

function MakeCap($path) {
  $cs = @"
using System; using System.IO; using System.Text;
public static class FdpCap {
  public static void Main(string[] args) {
    var sb = new StringBuilder();
    sb.Append("exe=" + System.Reflection.Assembly.GetExecutingAssembly().Location + "\n");
    sb.Append("cwd=" + Environment.CurrentDirectory + "\n");
    foreach (var a in args) sb.Append("arg=" + a + "\n");
    File.AppendAllText(@"$Recv", sb.ToString() + "--\n", new UTF8Encoding(false));
  }
}
"@
  Add-Type -TypeDefinition $cs -OutputAssembly $path -OutputType WindowsApplication
}
function MakeBat($path) {
  $q = '"'
  $text = "@echo off`r`n>>$q$BatRecv$q echo exe=%~f0`r`n>>$q$BatRecv$q echo cwd=%CD%`r`n>>$q$BatRecv$q echo args=%*`r`n>>$q$BatRecv$q echo --`r`n"
  [IO.File]::WriteAllText($path, $text, [Text.Encoding]::ASCII)
}
function NewLink($path, $target, $arguments = '', $wdir = '') {
  $s = (New-Object -ComObject WScript.Shell).CreateShortcut($path)
  $s.TargetPath = $target
  if ($arguments) { $s.Arguments = $arguments }
  if ($wdir) { $s.WorkingDirectory = $wdir }
  $s.Save()
}
function ClearRecv() { Remove-Item -LiteralPath $Recv, $BatRecv -Force -ErrorAction SilentlyContinue }

# What the test program got in its last runs: @{ exe; cwd; args; runs } or $null when nothing
# ran within $sec (exe and its shortcut: one arg= line each; .bat: %* on one line).
function Received($which, $sec = 8) {
  $f = if ($which -eq 'bat') { $BatRecv } else { $Recv }
  for ($i = 0; $i -lt $sec * 5; $i++) {
    if (Test-Path -LiteralPath $f) {
      Start-Sleep -Milliseconds 500
      $lines = @(Get-Content -LiteralPath $f -Encoding UTF8)
      $r = @{ exe = ''; cwd = ''; args = @(); runs = @($lines | Where-Object { $_ -eq '--' }).Count }
      foreach ($l in $lines) {
        if ($l -like 'exe=*') { $r.exe = $l.Substring(4) }
        elseif ($l -like 'cwd=*') { $r.cwd = $l.Substring(4) }
        elseif ($l -like 'arg=*') { $r.args += $l.Substring(4) }
        elseif ($l -like 'args=*') {
          foreach ($m in [regex]::Matches($l.Substring(5), '"([^"]*)"|(\S+)')) { if ($m.Groups[1].Success) { $r.args += $m.Groups[1].Value } else { $r.args += $m.Groups[2].Value } }
        }
      }
      return $r
    }
    Start-Sleep -Milliseconds 200
  }
  return $null
}
function RecvText($r) { if (-not $r) { return 'nothing ran' }; return "runs $($r.runs), exe '$(Split-Path -Leaf $r.exe)', cwd '$($r.cwd)', args [$($r.args -join ' | ')]" }

# ---- fences and items

function ClearFences() {
  foreach ($f in @(PfJson fence list)) { if ($f -and $f.title -and $f.kind -ne 'inbox') { Pf fence delete "$($f.id)" | Out-Null } }
  foreach ($r in @(Rules)) { if ($r -and $r.id) { Pf rule remove "$($r.id)" | Out-Null } }
  Start-Sleep -Seconds 1
}
# The name a fence shows for a desktop file, and where its icon is.
function NameOf($file) { foreach ($it in @(PfJson item list)) { if ($it -and $it.fileName -eq $file) { return $it.name } }; return $null }
function FencePt($fence, $file) { return (ItemPt $fence (NameOf $file)) }
# 「桌面」 arranged like the desktop with 自由摆放: its icons stand in 80 x 96 cells from the top left.
function InboxPt($file) {
  $id = ''; foreach ($it in @(PfJson item list --fence inbox)) { if ($it -and $it.fileName -eq $file) { $id = "$($it.id)" } }
  $g = FenceGet inbox
  $c = if ($id -and $g.desktopCells) { $g.desktopCells.PSObject.Properties | Where-Object { $_.Name -eq $id } | Select-Object -First 1 }
  # Without 自由摆放 the icons stand by name, as ItemPt counts them.
  if (-not $c) { Log "InboxPt: no cell for '$file' ($id)"; return (ItemPt inbox (NameOf $file)) }
  return @([int](4 + $c.Value[0] * 80 + 40), [int](4 + $c.Value[1] * 96 + 30))
}

# ---- File Explorer windows

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
# Selects the items in an Explorer window; returns the point of the first one.
function ExSelect($hw, $leaves) {
  $first = $null
  foreach ($leaf in $leaves) {
    $el = ExItem $hw $leaf
    if (-not $el) { return $null }
    $sel = $el.GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern)
    if (-not $first) { $sel.Select(); $first = $el } else { $sel.AddToSelection() }
    Start-Sleep -Milliseconds 150
  }
  return (ExPt $first)
}

# ---- input

# Selects the icons at the points: a click on the first, Ctrl+click on the others.
function SelectPts($pts) {
  [G]::Click($pts[0][0], $pts[0][1]); Start-Sleep -Milliseconds 350
  for ($i = 1; $i -lt $pts.Count; $i++) {
    [G]::KeyDown(0x11); Start-Sleep -Milliseconds 60
    [G]::Click($pts[$i][0], $pts[$i][1]); Start-Sleep -Milliseconds 60
    [G]::KeyUp(0x11); Start-Sleep -Milliseconds 350
  }
  Start-Sleep -Milliseconds 600
}
# A drag from one point to another with the left (or right) button. $keys: keys held from the
# first move to the release. $ocr names a crop of the badge beside the pointer before the
# release; its text is returned.
function DragDrop($x1, $y1, $x2, $y2, $keys = @(), [switch]$Right, $ocr = '') {
  $down = 2; $up = 4
  if ($Right) { $down = 8; $up = 0x10 }
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 150
  [G]::mouse_event($down, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  Glide $x1 $y1 ($x1 + 14) ($y1 + 14) 3 30
  foreach ($k in $keys) { [G]::KeyDown($k); Start-Sleep -Milliseconds 40 }
  Glide ($x1 + 14) ($y1 + 14) $x2 $y2 16 35
  Start-Sleep -Milliseconds 500
  [G]::MoveTo($x2 + 1, $y2 + 1); Start-Sleep -Milliseconds 400
  $badge = ''
  if ($ocr) { $badge = ((OcrText ($x2 - 30) ($y2 - 10) 420 150 $ocr) -replace '\s+', ' ').Trim() }
  [G]::mouse_event($up, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
  foreach ($k in $keys) { [G]::KeyUp($k); Start-Sleep -Milliseconds 30 }
  Start-Sleep -Milliseconds 700
  return $badge
}
$Mods = [ordered]@{ 'plain' = @(); 'Ctrl' = @(0x11); 'Shift' = @(0x10); 'Alt' = @(0x12); 'Ctrl+Shift' = @(0x11, 0x10) }
# What File Explorer does on a program, its shortcut or a .bat (Part X, 2026-11-08): every key
# gives the badge "Open with <name>" and opens all items with it, none moves; a right-button
# drag shows the menu "Open with" (default) | "Cancel".
$script:ModWant = @{ 'plain' = 'open'; 'Ctrl' = 'open'; 'Shift' = 'open'; 'Alt' = 'open'; 'Ctrl+Shift' = 'open'; 'right' = 'open'; 'right, Cancel' = 'none' }
$script:RightMenu = '[Open with] | Cancel'

# ---- the dragged items: new ones for every drop

$Mixes = @('one file', 'several files', 'files and folders', 'one folder')
# Makes the items of a mix in $dir: `zNN a.txt`, `zNN b.txt`, folder `zNN dir` (with a file in it).
function NewItems($dir, $mix) {
  $script:Tag++
  $t = 'z{0:D2}' -f $script:Tag
  $files = switch ($mix) {
    'one file' { @("$t a.txt") }
    'several files' { @("$t a.txt", "$t b.txt") }
    'files and folders' { @("$t a.txt", "$t b.txt", "$t dir") }
    'one folder' { @("$t dir") }
  }
  foreach ($leaf in $files) {
    $p = Join-Path $dir $leaf
    if ($leaf -like '* dir') { New-Item -ItemType Directory -Force $p | Out-Null; Set-Content -LiteralPath "$p\keep.txt" 'keep' }
    else { Set-Content -LiteralPath $p "drop test $leaf" }
  }
  return $files
}
function RemoveItems($leaves) {
  foreach ($leaf in $leaves) {
    foreach ($d in @($PfDesk, $Into, $SrcDir)) { Remove-Item -LiteralPath (Join-Path $d $leaf) -Recurse -Force -ErrorAction SilentlyContinue }
  }
}

# ---- the targets in fence Apps (desktop files, sorted by name)

$Targets = @(
  @{ file = 't1 app.lnk'; kind = 'program'; recv = 'exe'; what = 'a shortcut to a program' },
  @{ file = 't2 exe.exe'; kind = 'program'; recv = 'exe'; what = 'the program file' },
  @{ file = 't3 bat.bat'; kind = 'program'; recv = 'bat'; what = 'a .bat file' },
  @{ file = 't4 doc.lnk'; kind = 'fence'; recv = 'exe'; what = 'a shortcut to a document' },
  @{ file = 't5 folder.lnk'; kind = 'folder'; recv = 'exe'; what = 'a shortcut to a folder' },
  @{ file = 't6 broken.lnk'; kind = 'fence'; recv = 'exe'; what = 'a broken shortcut' }
)
function MakeTargets() {
  New-Item -ItemType Directory -Force "$Fd\bin", $Into, $SrcDir, $Wd | Out-Null
  if (-not (Test-Path $Cap)) { MakeCap $Cap }
  Set-Content -LiteralPath "$Fd\doc.txt" 'a document'
  NewLink (Made "$PfDesk\t1 app.lnk") $Cap '"saved arg"' $Wd
  Copy-Item $Cap (Made "$PfDesk\t2 exe.exe")
  MakeBat (Made "$PfDesk\t3 bat.bat")
  NewLink (Made "$PfDesk\t4 doc.lnk") "$Fd\doc.txt"
  NewLink (Made "$PfDesk\t5 folder.lnk") $Into
  NewLink (Made "$PfDesk\t6 broken.lnk") "$Fd\gone\missing.exe"
  NewLink (Made "$PfDesk\t7 notepad.lnk") "$env:WINDIR\notepad.exe"
}

# What should happen: 'open' (the program gets every item, nothing moves), 'folder' (into the
# folder the shortcut points to), 'fence' (into fence Apps).
function Expect($t, $mix) {
  if ($t.kind -eq 'program') { if ($New -or $mix -eq 'one folder') { return 'open' }; return 'fence' }
  return $t.kind
}

# One drop of a mix onto a target, from a source ('fence', 'inbox', 'explorer'), and its check.
function OneDrop($src, $t, $mix) {
  ClearRecv
  $dir = if ($src -eq 'explorer') { $SrcDir } else { $PfDesk }
  $leaves = NewItems $dir $mix
  foreach ($leaf in $leaves) { [void]$made.Add((Join-Path $dir $leaf)) }
  $home0 = ''
  if ($src -eq 'explorer') {
    Start-Sleep -Milliseconds 800
    $p0 = ExSelect $script:ExSrc $leaves
    if (-not $p0) { Check "$mix onto $($t.what)" $false 'the items did not show in Explorer'; RemoveItems $leaves; return }
  } else {
    Start-Sleep -Seconds 2
    $fenceRef = if ($src -eq 'fence') { 'Src' } else { 'inbox' }
    Pf item move @($leaves | ForEach-Object { Join-Path $PfDesk $_ }) --to $fenceRef | Out-Null
    Start-Sleep -Milliseconds 1200
    $home0 = if ($src -eq 'fence') { 'Src' } else { $script:InboxTitle }
    $pts = @(foreach ($leaf in $leaves) { if ($src -eq 'fence') { , (FencePt 'Src' $leaf) } else { , (InboxPt $leaf) } })
    if (@($pts | Where-Object { -not $_ }).Count) { Check "$mix onto $($t.what)" $false 'no icon point for the items'; RemoveItems $leaves; return }
    SelectPts $pts
    $p0 = $pts[0]
  }
  $cells0 = if ($src -eq 'inbox') { (@($leaves | ForEach-Object { (InboxPt $_) -join ',' })) -join ' ' } else { '' }
  $q = FencePt 'Apps' $t.file
  DragDrop $p0[0] $p0[1] $q[0] $q[1] | Out-Null
  $want = Expect $t $mix
  $r = Received $t.recv $(if ($want -eq 'open') { 8 } else { 3 })
  # Where every item is now.
  $rows = @(); $stayed = 0; $intoFolder = 0; $inApps = 0
  foreach ($leaf in $leaves) {
    $at = Join-Path $dir $leaf
    $there = Test-Path -LiteralPath $at
    $inF = Test-Path -LiteralPath (Join-Path $Into $leaf)
    $onDesk = Test-Path -LiteralPath (Join-Path $PfDesk $leaf)
    $fence = if ($onDesk) { ItemFence $leaf } else { '' }
    if ($there -and ($src -eq 'explorer' -or $fence -eq $home0)) { $stayed++ }
    if ($inF -and -not $there) { $intoFolder++ }
    if ($onDesk -and $fence -eq 'Apps') { $inApps++ }
    $rows += "$leaf (where it was $there, in the folder $inF, fence '$fence')"
  }
  $n = $leaves.Count
  $ok = switch ($want) {
    'open' {
      $got = @($r.args | Where-Object { $_ -ne 'saved arg' } | ForEach-Object { LongP $_ }) | Sort-Object
      $exp = @($leaves | ForEach-Object { LongP (Join-Path $dir $_) }) | Sort-Object
      $saved = ($t.file -ne 't1 app.lnk') -or ($r.args.Count -gt 0 -and $r.args[0] -eq 'saved arg' -and (LongP $r.cwd) -eq (LongP $Wd))
      # 「桌面」 with 自由摆放: the icons also keep their cells.
      $cells = if ($src -eq 'inbox') { (@($leaves | ForEach-Object { (InboxPt $_) -join ',' })) -join ' ' } else { '' }
      ($r -and $r.runs -eq 1 -and (($got -join '|') -eq ($exp -join '|')) -and $saved -and $stayed -eq $n -and $cells -eq $cells0)
    }
    'folder' { (-not $r) -and $intoFolder -eq $n }
    'fence' { (-not $r) -and $inApps -eq $n }
  }
  $say = @{ open = 'opens with it, the items stay'; folder = 'go into its folder'; fence = 'go into the fence' }[$want]
  $cellText = if ($src -eq 'inbox') { "; cells before [$cells0] after [$cells]" } else { '' }
  Check "$mix onto $($t.what): $say" $ok "$(RecvText $r); $($rows -join '; ')$cellText"
  RemoveItems $leaves
  ClearRecv
  Start-Sleep -Milliseconds 800
}

function Setup() {
  CloseWindows
  Remove-Item -Recurse -Force $Fd -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force $Fd | Out-Null
  foreach ($t in $Targets) { Remove-Item -LiteralPath (Join-Path $PfDesk $t.file) -Force -ErrorAction SilentlyContinue }
  MakeTargets
  PfStart
  ClearFences
  Pf fence create --title Apps --rect '900,90,660,300' | Out-Null
  Pf fence create --title Src --rect '900,470,660,330' | Out-Null
  Start-Sleep -Seconds 1
  Pf item move @(foreach ($t in $Targets) { Join-Path $PfDesk $t.file }) "$PfDesk\t7 notepad.lnk" --to Apps | Out-Null
  Pf fence set Apps sort name | Out-Null
  Pf fence set Src sort name | Out-Null
  $script:InboxTitle = (FenceGet inbox).title
  Start-Sleep -Seconds 2
  Log "Apps: $((@(PfJson item list --fence Apps) | ForEach-Object { $_.name }) -join ', '); inbox '$script:InboxTitle' layout $((FenceGet inbox).desktopLayout) free $((FenceGet inbox).freePlacement)"
  [G]::MoveTo(1580, 870)
}

function Matrix($src) {
  trap { PfTrap $_; continue }
  foreach ($t in $Targets) {
    Case "$src | onto $($t.what)"
    foreach ($mix in $Mixes) { OneDrop $src $t $mix }
  }
}

# ---- File Explorer's own drops (Part X)

function ExplorerOwn() {
  trap { PfTrap $_; continue }
  Case 'X setup'
  CloseWindows
  Remove-Item -Recurse -Force $Fd -ErrorAction SilentlyContinue
  $apps = "$Fd\apps"
  New-Item -ItemType Directory -Force "$Fd\bin", $SrcDir, $apps, $Wd | Out-Null
  MakeCap $Cap
  NewLink "$apps\x1 app.lnk" $Cap '"saved arg"' $Wd
  Copy-Item $Cap "$apps\x2 exe.exe"
  MakeBat "$apps\x3 bat.bat"
  $script:ExSrc = ExOpen $SrcDir 10 60 760 520
  $exApps = ExOpen $apps 800 60 780 520
  Check 'two Explorer windows' ($script:ExSrc -ne [IntPtr]::Zero -and $exApps -ne [IntPtr]::Zero) (Explorers)
  $xt = @(@{ file = 'x1 app.lnk'; recv = 'exe' }, @{ file = 'x2 exe.exe'; recv = 'exe' }, @{ file = 'x3 bat.bat'; recv = 'bat' })
  foreach ($t in $xt) {
    Case "X Explorer onto $($t.file)"
    $cases = @($Mods.Keys) + @('right', 'files and folders')
    foreach ($how in $cases) {
      ClearRecv
      $mix = if ($how -eq 'files and folders') { 'files and folders' } else { 'several files' }
      $leaves = NewItems $SrcDir $mix
      Start-Sleep -Milliseconds 900
      $p = ExSelect $script:ExSrc $leaves
      $tg = ExItem $exApps $t.file
      if (-not $p -or -not $tg) { Check "$how" $false 'items not found'; continue }
      $q = ExPt $tg
      $before = @(Get-ChildItem -LiteralPath $apps -Name)
      $shot = "x-$([IO.Path]::GetFileNameWithoutExtension($t.file))-$($how -replace '\W', '')"
      $menu = ''
      if ($how -eq 'right') {
        $badge = DragDrop $p[0] $p[1] $q[0] $q[1] -Right -ocr $shot
        $m = MenuWin 3
        if ($m -ne [IntPtr]::Zero) {
          $menu = (@(MenuRows $m) | Where-Object { $_.text } | ForEach-Object { if ($_.state -band 0x1000) { "[$($_.text)]" } else { $_.text } }) -join ' | '
          Shot "$shot-menu"
          $def = @(MenuRows $m) | Where-Object { $_.text -and ($_.state -band 0x1000) } | Select-Object -First 1
          if ($def) { [G]::Click([int](($def.L + $def.R) / 2), [int](($def.T + $def.B) / 2)) } else { [G]::Press(0x1B) }
          Start-Sleep -Milliseconds 800
        } else { $menu = 'no menu' }
      } else {
        # (PowerShell names ignore case: a local $mods would hide $Mods.)
        $held = if ($Mods.Contains($how)) { $Mods[$how] } else { @() }
        $badge = DragDrop $p[0] $p[1] $q[0] $q[1] $held -ocr $shot
      }
      $r = Received $t.recv 6
      $left = @($leaves | Where-Object { Test-Path -LiteralPath (Join-Path $SrcDir $_) }).Count
      $newIn = @(Get-ChildItem -LiteralPath $apps -Name | Where-Object { $before -notcontains $_ })
      $menuText = if ($menu) { "; menu: $menu" } else { '' }
      Check "$how ($mix)" $true "badge '$badge'$menuText; $(RecvText $r); left in the source $left of $($leaves.Count); new in the target folder [$($newIn -join ', ')]"
      foreach ($x in $newIn) { Remove-Item -LiteralPath (Join-Path $apps $x) -Recurse -Force -ErrorAction SilentlyContinue }
      RemoveItems $leaves
    }
  }
  CloseWindows
}

# ---- modifiers and the right button on the program targets (Part D)

function ModDrops($src) {
  trap { PfTrap $_; continue }
  foreach ($file in @('t1 app.lnk', 't2 exe.exe')) {
    $t = $Targets | Where-Object { $_.file -eq $file } | Select-Object -First 1
    Case "D $src | keys onto $($t.what)"
    # A fence starts no drag with the right button (its icons get their menu), so the right
    # button is tried from File Explorer only.
    $hows = @($Mods.Keys)
    if ($src -eq 'explorer') { $hows += @('right', 'right, Cancel') }
    foreach ($how in $hows) {
      ClearRecv
      $dir = if ($src -eq 'explorer') { $SrcDir } else { $PfDesk }
      $leaves = NewItems $dir 'several files'
      foreach ($leaf in $leaves) { [void]$made.Add((Join-Path $dir $leaf)) }
      $home0 = ''
      if ($src -eq 'explorer') {
        Start-Sleep -Milliseconds 800
        $p0 = ExSelect $script:ExSrc $leaves
      } else {
        Start-Sleep -Seconds 2
        $fenceRef = if ($src -eq 'fence') { 'Src' } else { 'inbox' }
        Pf item move @($leaves | ForEach-Object { Join-Path $PfDesk $_ }) --to $fenceRef | Out-Null
        Start-Sleep -Milliseconds 1200
        $home0 = if ($src -eq 'fence') { 'Src' } else { $script:InboxTitle }
        $pts = @(foreach ($leaf in $leaves) { if ($src -eq 'fence') { , (FencePt 'Src' $leaf) } else { , (InboxPt $leaf) } })
        SelectPts $pts
        $p0 = $pts[0]
      }
      $q = FencePt 'Apps' $t.file
      $shot = "d-$src-$([IO.Path]::GetFileNameWithoutExtension($file))-$($how -replace '\W', '')"
      $menu = ''
      if ($how -like 'right*') {
        $badge = DragDrop $p0[0] $p0[1] $q[0] $q[1] -Right -ocr $shot
        $m = MenuWin 3
        if ($m -ne [IntPtr]::Zero) {
          $menu = (@(MenuRows $m) | Where-Object { $_.text } | ForEach-Object { if ($_.state -band 0x1000) { "[$($_.text)]" } else { $_.text } }) -join ' | '
          Shot "$shot-menu"
          $def = @(MenuRows $m) | Where-Object { $_.text -and ($_.state -band 0x1000) } | Select-Object -First 1
          if ($def -and $how -eq 'right') { [G]::Click([int](($def.L + $def.R) / 2), [int](($def.T + $def.B) / 2)) } else { [G]::Press(0x1B) }
          Start-Sleep -Milliseconds 800
        } else { $menu = 'no menu' }
      } else {
        $badge = DragDrop $p0[0] $p0[1] $q[0] $q[1] $Mods[$how] -ocr $shot
      }
      $r = Received $t.recv 8
      $stayed = 0
      foreach ($leaf in $leaves) {
        $there = Test-Path -LiteralPath (Join-Path $dir $leaf)
        if ($there -and ($src -eq 'explorer' -or (ItemFence $leaf) -eq $home0)) { $stayed++ }
      }
      $got = @($r.args | Where-Object { $_ -ne 'saved arg' } | ForEach-Object { LongP $_ }) | Sort-Object
      $exp = @($leaves | ForEach-Object { LongP (Join-Path $dir $_) }) | Sort-Object
      $opened = $r -and $r.runs -eq 1 -and (($got -join '|') -eq ($exp -join '|'))
      $want = $script:ModWant[$how]
      $ok = if ($want -eq 'open') { $opened -and $stayed -eq $leaves.Count } else { -not $r -and $stayed -eq $leaves.Count }
      # The badge names the shortcut, as 「使用 %1 打开」 says in English (a drag that starts in a
      # fence shows no badge text at all, on any target); the right button gets File Explorer's
      # own menu.
      $ok = $ok -and ($src -ne 'explorer' -or $badge -match 'Open with') -and ($how -notlike 'right*' -or $menu -eq $script:RightMenu)
      $menuText = if ($menu) { "; menu: $menu" } else { '' }
      Check "$how : $want" $ok "badge '$badge'$menuText; $(RecvText $r); stayed $stayed of $($leaves.Count)"
      RemoveItems $leaves
      ClearRecv
      Start-Sleep -Milliseconds 800
    }
  }
}

# ---- Notepad opens a .txt dropped on its shortcut (Part N)

function NotepadDrops() {
  trap { PfTrap $_; continue }
  foreach ($src in @('fence', 'inbox', 'explorer')) {
    Case "N $src | a .txt onto a shortcut to Notepad"
    Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
    $dir = if ($src -eq 'explorer') { $SrcDir } else { $PfDesk }
    $script:Tag++
    $leaf = 'z{0:D2} note.txt' -f $script:Tag
    Set-Content -LiteralPath (Made (Join-Path $dir $leaf)) 'notepad drop test'
    if ($src -eq 'explorer') {
      Start-Sleep -Milliseconds 900
      $p0 = ExSelect $script:ExSrc @($leaf)
      $home0 = ''
    } else {
      Start-Sleep -Seconds 2
      $fenceRef = if ($src -eq 'fence') { 'Src' } else { 'inbox' }
      Pf item move (Join-Path $PfDesk $leaf) --to $fenceRef | Out-Null
      Start-Sleep -Milliseconds 1200
      $home0 = if ($src -eq 'fence') { 'Src' } else { $script:InboxTitle }
      $p0 = if ($src -eq 'fence') { FencePt 'Src' $leaf } else { InboxPt $leaf }
    }
    $q = FencePt 'Apps' 't7 notepad.lnk'
    DragDrop $p0[0] $p0[1] $q[0] $q[1] | Out-Null
    $title = ''
    for ($i = 0; $i -lt 30; $i++) { $title = NotepadTitles; if ($title -match [regex]::Escape([IO.Path]::GetFileNameWithoutExtension($leaf))) { break }; Start-Sleep -Milliseconds 300 }
    Shot "n-$src-notepad"
    $there = Test-Path -LiteralPath (Join-Path $dir $leaf)
    $fence = if ($src -eq 'explorer') { '' } else { ItemFence $leaf }
    $want = $New
    $ok = if ($want) { ($title -match [regex]::Escape([IO.Path]::GetFileNameWithoutExtension($leaf))) -and $there -and ($src -eq 'explorer' -or $fence -eq $home0) } else { -not $title -and ((ItemFence $leaf) -eq 'Apps') }
    $say = if ($want) { 'Notepad opens it, the file stays' } else { 'it goes into the fence' }
    Check $say $ok "notepad '$title'; where it was $there; fence '$fence'"
    Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
    RemoveItems @($leaf)
    Start-Sleep -Milliseconds 800
  }
}

# ---- the recording (Part R): a .txt from one fence onto the Notepad shortcut in another

function Recording() {
  trap { PfTrap $_; continue }
  . C:\pf\kit\rec.ps1
  Case 'R setup'
  CloseWindows
  $leaf = '会议记录.txt'
  Set-Content -LiteralPath (Made (Join-Path $PfDesk $leaf)) "周五 10 点开会`r`n带上上周的数据" -Encoding UTF8
  # WScript.Shell cannot save a shortcut with a Chinese name: save it, then rename it.
  NewLink "$PfDesk\fdp-notepad.lnk" "$env:WINDIR\notepad.exe"
  Rename-Item -LiteralPath "$PfDesk\fdp-notepad.lnk" '记事本.lnk'
  [void]$made.Add("$PfDesk\记事本.lnk")
  PfStart -Lang 'zh-CN'
  ClearFences
  # Low on the screen: Notepad opens over the top part (where it was last), the fences stay in view.
  Pf fence create --title '文档' --rect '160,672,420,170' | Out-Null
  Pf fence create --title '常用软件' --rect '700,672,420,170' | Out-Null
  Start-Sleep -Seconds 1
  Pf item move (Join-Path $PfDesk $leaf) --to '文档' | Out-Null
  Pf item move "$PfDesk\记事本.lnk" --to '常用软件' | Out-Null
  Start-Sleep -Seconds 2
  $p = FencePt '文档' $leaf
  $q = FencePt '常用软件' '记事本.lnk'
  [G]::MoveTo(640, 560); Start-Sleep -Milliseconds 600
  $name = if ($New) { 'R-after' } else { 'R-before' }
  RecStart $name -X 0 -Y 0 -W 1600 -H 900 -Fps 12
  Start-Sleep -Milliseconds 800
  RecMark '把「会议记录.txt」拖到「记事本」的快捷方式上'
  Glide 640 560 $p[0] $p[1] 14 40
  Start-Sleep -Milliseconds 300
  MDown; Start-Sleep -Milliseconds 250
  Glide $p[0] $p[1] $q[0] $q[1] 30 45
  Start-Sleep -Milliseconds 900
  RecMark '松开鼠标'
  MUp
  Start-Sleep -Milliseconds 300
  Glide $q[0] $q[1] ($q[0] + 260) ($q[1] - 40) 10 40
  Start-Sleep -Seconds 4
  RecMark '结果'
  Start-Sleep -Seconds 3
  RecStop
  $title = NotepadTitles
  $fence = ItemFence $leaf
  Log "R: notepad '$title', the file's fence '$fence'"
  Check 'recorded' $true "notepad '$title'; fence '$fence'"
  Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
}

# ---- run the part

if ($Part -eq 'X') { ExplorerOwn }
if ($Part -in @('A', 'B', 'C', 'D', 'N')) { Case 'setup'; Setup }
# Right of the first four columns of 「桌面」, where its new icons go.
if ($Part -eq 'C' -or $Part -eq 'D' -or $Part -eq 'N') { $script:ExSrc = ExOpen $SrcDir 340 420 540 450 }
if ($Part -eq 'A') { Matrix 'fence' }
if ($Part -eq 'B') { Matrix 'inbox' }
if ($Part -eq 'C') { Matrix 'explorer' }
if ($Part -eq 'D') { foreach ($s in @('fence', 'inbox', 'explorer')) { ModDrops $s } }
if ($Part -eq 'N') { NotepadDrops }
if ($Part -eq 'R') { Recording }

Case 'cleanup'
CloseWindows
if (PfAlive) { PfQuit }
foreach ($p in $made) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue }
foreach ($t in $Targets) { Remove-Item -LiteralPath (Join-Path $PfDesk $t.file) -Force -ErrorAction SilentlyContinue }
Remove-Item -LiteralPath "$PfDesk\t7 notepad.lnk" -Force -ErrorAction SilentlyContinue
