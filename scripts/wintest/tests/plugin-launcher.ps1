# 插件 interface 2, the launcher sample (plan v6 rows 1, 2, 5, 8): the plugin in assets\launcher lists
# what the user put into its fence. Its fence menu has 添加程序… and 添加文件… (view.menu); files,
# folders and shortcuts dropped from File Explorer become items (not PecoFence's own drags, not a
# file of its own folder); a click opens an item through the shell (a test program writes a
# marker); 显示启动器's shortcut, set in the plugin's details, shows the fence alone over a window
# like a Peek and sends the command; 移除 in the details; an import drops items and shortcuts; an
# update that adds a command asks first; an entry using launch without api 2 is left out.
# Debug build (CDP on the settings page), Chinese interface.
#   --var Part=B   the build before (with assets\launcher-before): the shots to compare
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
$Kit = if ($Part -eq 'B') { 'C:\pf\kit\assets\launcher-before' } else { 'C:\pf\kit\assets\launcher' }
$Feed = 'C:\pf\run\lfeed'
$Lr = 'C:\pf\run\launch'
$Marker = "$Lr\opened.txt"
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class LX {
  [DllImport("user32.dll")] public static extern int GetWindowLongW(IntPtr h, int i);
  public static bool Topmost(IntPtr h) { return (GetWindowLongW(h, -20) & 8) != 0; }
}
'@

function ExOpen($path, $x, $y, $w, $h) {
  Start-Process explorer.exe "`"$path`""
  for ($i = 0; $i -lt 60; $i++) {
    foreach ($win in @((New-Object -ComObject Shell.Application).Windows())) {
      $p = try { $win.Document.Folder.Self.Path } catch { '' }
      if ($p -eq $path) { $hw = [IntPtr]$win.HWND; [L]::MoveWindow($hw, $x, $y, $w, $h, $true) | Out-Null; Start-Sleep -Milliseconds 1200; return $hw }
    }
    Start-Sleep -Milliseconds 250
  }
  return [IntPtr]::Zero
}
# The point of an item of an Explorer window (by its name with or without the extension), selected.
function ExPick($hw, [string[]]$leaves) {
  $root = [Windows.Automation.AutomationElement]::FromHandle($hw)
  $isItem = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
  $first = $null
  foreach ($leaf in $leaves) {
    $el = $null
    for ($k = 0; $k -lt 20 -and -not $el; $k++) {
      foreach ($n in @($leaf, [IO.Path]::GetFileNameWithoutExtension($leaf))) {
        $by = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::NameProperty, $n)
        if (-not $el) { $el = $root.FindFirst([Windows.Automation.TreeScope]::Descendants, (New-Object Windows.Automation.AndCondition($isItem, $by))) }
      }
      if (-not $el) { Start-Sleep -Milliseconds 250 }
    }
    if (-not $el) { Log "ExPick: '$leaf' not in the window"; return $null }
    $sel = $el.GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern)
    if (-not $first) { $sel.Select(); $first = $el } else { $sel.AddToSelection() }
    Start-Sleep -Milliseconds 150
  }
  $r = $first.Current.BoundingRectangle
  return @([int]($r.Left + 40), [int](($r.Top + $r.Bottom) / 2))
}
function DragDrop($x1, $y1, $x2, $y2, $shot = '') {
  [G]::MoveTo($x1, $y1); Start-Sleep -Milliseconds 150
  [G]::mouse_event(2, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 200
  Glide $x1 $y1 ($x1 + 14) ($y1 + 14) 3 30
  Glide ($x1 + 14) ($y1 + 14) $x2 $y2 16 35
  Start-Sleep -Milliseconds 500
  [G]::MoveTo($x2 + 1, $y2 + 1); Start-Sleep -Milliseconds 500
  if ($shot) { Shot $shot }
  [G]::mouse_event(4, 0, 0, 0, [UIntPtr]::Zero); Start-Sleep -Milliseconds 1500
}
function FenceRect() { return (FenceGet $script:Fid).windowRect }
function FenceMid() { $r = FenceRect; return @([int]($r.x + $r.w / 2), [int]($r.y + $r.h / 2)) }
function Items() { $s = (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts."$script:Fid".settings; return @($s.__launch | Where-Object { $_ }) }
function Keys() { return (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts."$script:Fid".settings.__keys }
function WaitItems($n, $sec = 8) { for ($i = 0; $i -lt $sec * 2; $i++) { if (@(Items).Count -eq $n) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
function FenceHwnd($fid = $script:Fid) { $r = (FenceGet $fid).windowRect; return @(@([W]::Tops('PecoFence.Fence')) | Where-Object { (Rect $_).L -eq $r.x -and (Rect $_).T -eq $r.y })[0] }
function WaitOut($regex, $sec = 10) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((FenceOut $script:Fid) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
function FenceMenu() { $r = FenceRect; [G]::RClick($r.x + 120, $r.y + $r.h - 40); return (MenuWin 5) }
function Marks() { if (Test-Path $Marker) { return @(Get-Content $Marker | Where-Object { $_ }).Count }; return 0 }
# The plugin index of the feed: launcher at `version` with `commands`, and `others` (id = api).
function Publish([int]$n, [string]$version, $commands, $others = @{}, [string]$down = '') {
  $commit = ('{0:x}' -f $n).PadLeft(40, 'b')
  $entries = @()
  $all = @(@{ id = 'launcher'; api = 2; perms = [ordered]@{ launch = $true; drop = @('files'); commands = $commands } })
  foreach ($k in $others.Keys) { $all += @{ id = $k; api = $others[$k]; perms = [ordered]@{ launch = $true } } }
  foreach ($p in $all) {
    $dir = "$Feed\$commit\plugins\$($p.id)"; New-Item -ItemType Directory -Force $dir | Out-Null
    $m = [ordered]@{ id = $p.id; version = $version; api = $p.api; layer = 'light'; name = @{ en = $p.id; 'zh-CN' = $p.id }
      description = @{ en = 'A test plugin' }; author = 'tester'; minPecoFence = '0.5.0'; permissions = $p.perms }
    [IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 6))
    Copy-Item "$Kit\main.js", "$Kit\README.md" $dir
    $m.files = @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
    if ($down -and $p.id -eq 'launcher') { $m.disabled = $down }
    $entries += $m
  }
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = $n; commit = $commit; plugins = @($entries) } | ConvertTo-Json -Depth 8))
  Log "published launcher $version at $commit"
}
function HostRow($id) {
  Js "(() => { if (!window.pfRows) { window.pfRows = {}; window.chrome.webview.addEventListener('message', e => { if (e.data && e.data.type === 'state' && e.data.plugins) window.pfRows.v = e.data.plugins; }); } return true; })()" | Out-Null
  Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null; Start-Sleep -Seconds 3
  return (Js "(() => { const l = window.pfRows.v && window.pfRows.v.list.find(x => x.id === '$id'); return l ? { state: l.state, fences: l.fences.map(f => ({ items: f.items.length, keys: f.keys.map(k => k.id + '=' + (k.key ? k.key.label : '')) })) } : null; })()")
}

Case 'setup: test files, the plugin, its fence'
CloseWindows
New-Item -ItemType Directory -Force "$Lr\Box folder", $Feed | Out-Null
Set-Content "$Lr\notes-a.txt" 'a note' -Encoding ASCII
Add-Type -OutputAssembly "$Lr\pfmark.exe" -OutputType WindowsApplication -TypeDefinition @"
public static class PfMark { public static void Main(string[] a) { System.IO.File.AppendAllText(@"$Marker", "opened " + string.Join(" ", a) + "\r\n"); } }
"@
$lnk = (New-Object -ComObject WScript.Shell).CreateShortcut("$Lr\Linked mark.lnk"); $lnk.TargetPath = "$Lr\pfmark.exe"; $lnk.Arguments = 'via-link'; $lnk.Save()
$cmds = @([ordered]@{ id = 'show'; title = @{ en = 'Show the launcher'; 'zh-CN' = '显示启动器' } })
Publish 1 '1.0.0' $cmds
$feedProc = PfFeed 8732 $Feed
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8732/index.json'
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
$script:Config = "$PfApp\config"
PfPlacePlugin $Kit 'launcher'
if ($Part -eq 'A') {
  # A second launcher, B, whose command from its shortcut runs for ever: PecoFence stops it each time.
  PfPlacePlugin $Kit 'launcher-b'
  $bd = "$script:Config\plugins\launcher-b\1.0.0"
  [IO.File]::WriteAllText("$bd\plugin.json", ([IO.File]::ReadAllText("$bd\plugin.json") -replace '"启动器"', '"B 启动器"'))
  Add-Content "$bd\main.js" 'pf.on("command", (c) => { if (c.via === "key") for (;;) {} });'
}
PfStart -Lang zh-CN
$script:Fid = PfPluginFence '启动器' '80,140,320,360' 'launcher'
Check 'the launcher starts and shows its hint' ([bool](PfWaitLog 'plugin fence starts its main.js' 20) -and (WaitOut '拖到这里|右键')) (FenceOut $script:Fid)
Start-Sleep -Seconds 1; Shot 'fence-empty'

Case 'the fence menu: the plugin''s two entries; 添加程序… adds the program picked'
$m = FenceMenu
$texts = MenuTexts $m
Shot 'menu'
Check 'after 重新启动插件 come 添加程序… and 添加文件…' ($texts -match '重新启动插件 \| 添加程序… \| 添加文件… \|') $texts
if ($Part -eq 'A') {
  MenuPick '^添加程序' $m | Out-Null
  $dlg = WaitWin '#32770' 10
  Check 'a file dialog titled 添加程序 opens' ($dlg -ne [IntPtr]::Zero -and [L]::Text($dlg) -eq '添加程序') "dlg=$dlg title=$([L]::Text($dlg))"
  [QS]::SetFileName($dlg, "$Lr\pfmark.exe"); Start-Sleep -Milliseconds 300; [G]::Press(0x0D)
  # The picture is made after the item is saved, on a thread of its own; named after the fence.
  $pic = "$script:Config\plugins-data\launcher\$script:Fid-launch-1.png"
  for ($i = 0; $i -lt 20 -and -not (Test-Path $pic); $i++) { Start-Sleep -Milliseconds 250 }
  Check 'the program is an item of the fence, with its picture' ((WaitOut 'pfmark') -and (WaitItems 1) -and (Test-Path $pic)) "$(FenceOut $script:Fid) items=$(@(Items).Count)"
  Check 'the fence keeps the path; the plugin got none (its view has the name only)' ((Items)[0].path -eq "$Lr\pfmark.exe" -and (FenceOut $script:Fid) -notmatch 'C:\\') "$((Items)[0].path)"
} else { CloseMenus }

Case 'drop from File Explorer: a file, then a shortcut and a folder together'
$ex = ExOpen $Lr 760 140 640 480
$p = ExPick $ex @('notes-a.txt')
$to = FenceMid
DragDrop $p[0] $p[1] $to[0] $to[1] 'drop-badge'
if ($Part -eq 'A') {
  Check 'the dropped file is an item' ((WaitOut 'notes-a') -and (WaitItems 2)) "$(FenceOut $script:Fid)"
  Check 'the log names the drag over a plugin fence that fits' ([bool](PfWaitLog 'a drag over a plugin fence.*fits=true' 3)) ''
  $p = ExPick $ex @('Linked mark.lnk', 'Box folder')
  DragDrop $p[0] $p[1] $to[0] $to[1]
  Check 'a shortcut and a folder dropped together: two items more' ((WaitOut 'Box folder') -and (FenceOut $script:Fid) -match 'Linked mark' -and (WaitItems 4)) "$(FenceOut $script:Fid)"
  Shot 'fence-items'
  CloseWindows
  $ex = ExOpen "$script:Config\plugins-data\launcher" 760 140 640 480
  $p = ExPick $ex @("$script:Fid-launch-1.png")
  DragDrop $p[0] $p[1] $to[0] $to[1]
  Check 'a file of the plugin''s own data folder is refused' (@(Items).Count -eq 4 -and [bool](PfWaitLog 'launcher item refused: a file of PecoFence' 3)) "items=$(@(Items).Count)"
}
CloseWindows

Case 'a drag of a fence icon (PecoFence''s own drag) is refused'
$src = PfJson fence create --title 'Src' --rect '440,560,300,200'
$file = DeskFile 'pf-launch-own.txt'
Start-Sleep -Seconds 2
Pf item move $file --to Src | Out-Null; Start-Sleep -Seconds 1
$p = ItemPt 'Src' 'pf-launch-own'
$before = @(Items).Count
DragTo $p[0] $p[1] $to[0] $to[1]
Start-Sleep -Seconds 1
Check 'the icon is no item and stays in its fence' (@(Items).Count -eq $before -and (ItemFence 'pf-launch-own.txt') -eq 'Src' -and [bool](PfWaitLog 'a drag over a plugin fence.*internal=true.*fits=false' 3)) "items=$(@(Items).Count) fence=$(ItemFence 'pf-launch-own.txt')"

if ($Part -eq 'A') {
Case 'a click opens an item as a fence icon opens'
Remove-Item $Marker -ErrorAction SilentlyContinue
$r = FenceRect
$ok = OcrClick '^pfmark' $r.x $r.y $r.w $r.h
for ($i = 0; $i -lt 20 -and (Marks) -lt 1; $i++) { Start-Sleep -Milliseconds 500 }
Check 'a click on pfmark starts it once' ($ok -and (Marks) -eq 1 -and [bool](PfWaitLog 'launcher item opened.*ok=true' 3)) "clicked=$ok marks=$(Marks)"
Start-Sleep -Seconds 1
$ok = OcrClick '^Linked mark' $r.x $r.y $r.w $r.h
for ($i = 0; $i -lt 20 -and (Marks) -lt 2; $i++) { Start-Sleep -Milliseconds 500 }
Check 'the shortcut opens with its own argument' ($ok -and (Get-Content $Marker -Raw) -match 'opened via-link') "$(Get-Content $Marker -Raw)"
}

Case 'the plugin''s details: 显示启动器 gets a shortcut'
$ok = PfSettings
$sw = WaitWin 'PecoFence.Settings' 5
[L]::MoveWindow($sw, 760, 60, 900, 900, $true) | Out-Null
Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
$i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('启动器'))")
PageClick '#pluginList .pl-row' $i; Start-Sleep -Seconds 1
$box = "plk-$script:Fid-show"
$d = Js "(() => { const d = document.querySelector('#pluginList .pl-detail'); return d ? d.innerText : ''; })()"
Js "document.getElementById('$box') && document.getElementById('$box').scrollIntoView({ block: 'center' })" | Out-Null
Shot 'details'
Check 'the details list what it does and the command' ($d -match '打开你放进这个栅栏的文件、文件夹和程序' -and $d -match '有这些命令：显示启动器' -and $d -match '显示启动器') $d
if ($Part -eq 'A') {
  Check 'the fence''s settings list its items with 移除' (@(Js "[...document.querySelectorAll('#pluginList .pl-form button')].filter(b => b.textContent === '移除').length")[0] -eq 4) $d
  PageClick "#$box"; Start-Sleep -Milliseconds 500
  [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x4C); [G]::KeyUp(0x12); [G]::KeyUp(0x11); Start-Sleep -Seconds 2
  $keys = Js "document.getElementById('$box').innerText"
  Shot 'details-shortcut'
  Check 'Ctrl+Alt+L is recorded, saved with the fence and registered' ((Keys).show -eq 'Ctrl+Alt+L' -and $keys -match 'Ctrl' -and $keys -match 'L' -and [bool](PfWaitLog 'plugin shortcut registered.*Ctrl\+Alt\+L' 5)) "keys=$keys saved=$((Keys).show)"
  [L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 2

  Case 'the shortcut shows the fence alone over a window, then the plugin gets the command'
  $ex = ExOpen $Lr 40 100 700 560
  $mid = FenceMid
  $fw = FenceHwnd
  Check 'File Explorer covers the fence' ([W]::RootClassAt($mid[0], $mid[1]) -ne 'PecoFence.Fence') ([W]::RootClassAt($mid[0], $mid[1]))
  [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x4C); [G]::KeyUp(0x12); [G]::KeyUp(0x11); Start-Sleep -Seconds 2
  $dim = @([W]::Tops('PecoFence.PeekDimmer')).Count
  $over = [W]::RootClassAt($mid[0], $mid[1])
  $others = @(@([W]::Tops('PecoFence.Fence')) | Where-Object { $_ -ne $fw -and [LX]::Topmost($_) }).Count
  Shot 'shortcut-shown'
  Check 'the fence is over File Explorer, above the dimmer, the other fences stay below' ($over -eq 'PecoFence.Fence' -and $dim -ge 1 -and $others -eq 0) "at=$over dimmers=$dim other topmost fences=$others"
  Check 'the plugin got command {id: show, via: key}' ([bool](PfWaitLog 'plugin shortcut pressed.*command="?show' 3) -and [bool](PfWaitLog 'plugin event.*name=.?command' 3)) ''
  [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press(0x4C); [G]::KeyUp(0x12); [G]::KeyUp(0x11); Start-Sleep -Seconds 2
  Check 'pressed again, the Peek ends and File Explorer covers the fence again' ([W]::RootClassAt($mid[0], $mid[1]) -ne 'PecoFence.Fence' -and @([W]::Tops('PecoFence.PeekDimmer')).Count -eq 0) ([W]::RootClassAt($mid[0], $mid[1]))
  CloseWindows

  Case '移除 in the details takes an item out of the fence'
  $ok = PfSettings
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 2
  $i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('启动器'))")
  PageClick '#pluginList .pl-row' $i; Start-Sleep -Seconds 1
  $j = [int](Js "[...document.querySelectorAll('#pluginList .pl-form .pl-field')].findIndex(r => r.textContent.startsWith('notes-a'))")
  if ($j -ge 0) { Js "[...document.querySelectorAll('#pluginList .pl-form .pl-field')][$j].querySelector('button').click()" | Out-Null; Start-Sleep -Seconds 2 }
  Check 'notes-a is gone from the config and from the fence' ((WaitItems 3) -and (FenceOut $script:Fid) -notmatch 'notes-a' -and [bool](PfWaitLog 'launcher item removed' 3)) "$(FenceOut $script:Fid)"

  Case 'an update that adds a command asks first; an entry using launch without api 2 is left out'
  $cmds2 = $cmds + @([ordered]@{ id = 'add'; title = @{ en = 'Add'; 'zh-CN' = '添加' } })
  Publish 2 '1.1.0' $cmds2 @{ 'launcher-old' = 1 }
  $row = HostRow 'launcher'
  Check 'the row waits for the user: 有更新，需要你确认' ($row.state -eq 'confirm') ($row | ConvertTo-Json -Compress -Depth 5)
  Check 'the entry with launch and api 1 is not offered' ($null -eq (HostRow 'launcher-old')) ''
  Check 'the log names the field that needs api 2' ([bool](PfWaitLog 'launcher-old: launch need api 2' 3) -or ((PfLogAll) -join "`n") -match 'launcher-old: launch need api 2') ''
  [L]::PostMessageW((WaitWin 'PecoFence.Settings' 3), 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1

  Case 'an import keeps the plugin fence without its items and shortcut'
  $file = "$PfRun\launch-export.json"
  Pf config export $file | Out-Null
  Pf config import $file | Out-Null; Start-Sleep -Seconds 4
  Check 'no items and no shortcut after the import (the fence stays, held as the file''s scripts are)' ((WaitItems 0) -and $null -eq (Keys) -and (FenceGet $script:Fid).script.path -eq 'plugin:launcher') "items=$(@(Items).Count) keys=$(Keys) out=$(FenceOut $script:Fid)"
  Check 'its shortcut was unregistered' ([bool](PfWaitLog 'plugin shortcut unregistered' 3)) ''
}
if ($Part -eq 'A') {
  function Key([int]$vk) { [G]::KeyDown(0x11); [G]::KeyDown(0x12); [G]::Press($vk); [G]::KeyUp(0x12); [G]::KeyUp(0x11); Start-Sleep -Seconds 2 }
  # Records Ctrl+Alt+<vk> for 显示启动器 of fence `fid` in the plugin details; the help text under it.
  function SetKey($fid, [int]$vk) {
    PfSettings | Out-Null; $sw = WaitWin 'PecoFence.Settings' 5
    [L]::MoveWindow($sw, 760, 60, 900, 900, $true) | Out-Null
    Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null
    PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
    $box = "plk-$fid-show"
    Js "(() => { for (let i = 0; i < 12; i++) { const r = [...document.querySelectorAll('#pluginList .pl-row')].find(r => r.getAttribute('aria-expanded') !== 'true'); if (!r) break; r.click(); } const b = document.getElementById('$box'); if (b) b.scrollIntoView({ block: 'center' }); return !!b; })()" | Out-Null
    Start-Sleep -Milliseconds 500; PageClick "#$box"; Start-Sleep -Milliseconds 500
    Key $vk
    $help = Js "(document.getElementById('${box}Help') || {}).innerText"
    [L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 2
    return "$help"
  }
  function WaitCount($regex, [int]$n, $sec = 15) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((LogCount $regex) -ge $n) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
  $UnregL = 'plugin shortcut unregistered.*Ctrl\+Alt\+L'; $UnregK = 'plugin shortcut unregistered.*Ctrl\+Alt\+K'
  # The last log line of the shortcut: registered or unregistered.
  function Last([string]$key) { return "$(PfWaitLog "plugin shortcut (un)?registered.*Ctrl\+Alt\+$key" 1)" }

  Case 'a drop of 20 files: all of them become items, in one call'
  New-Item -ItemType Directory -Force "$Lr\many" | Out-Null
  $names = @(1..20 | ForEach-Object { 'm{0:d2}.txt' -f $_ })
  foreach ($n in $names) { Set-Content "$Lr\many\$n" $n -Encoding ASCII }
  $ex = ExOpen "$Lr\many" 760 40 700 960
  $p = ExPick $ex $names
  $to = FenceMid
  DragDrop $p[0] $p[1] $to[0] $to[1]
  Check 'twenty files are twenty items, no call refused for too many at once' ((WaitItems 20 10) -and (LogCount 'too many calls at once') -eq 0) "items=$(@(Items).Count)"
  CloseWindows

  Case 'two launchers: the shortcut of B while the launcher is up alone ends that Peek and shows B'
  $script:FidB = PfPluginFence 'B' '440,520,280,240' 'launcher-b'
  Start-Sleep -Seconds 3
  $a = SetKey $script:Fid 0x4C; $b = SetKey $script:FidB 0x4B
  Check 'Ctrl+Alt+L for the launcher, Ctrl+Alt+K for B' ((WaitCount 'plugin shortcut registered.*Ctrl\+Alt\+K' 1 5) -and (Keys).show -eq 'Ctrl+Alt+L') "help: $a | $b"
  $ex = ExOpen $Lr 40 100 1000 760
  Key 0x4C
  $aFirst = [LX]::Topmost((FenceHwnd))
  Key 0x4B
  $aThen = [LX]::Topmost((FenceHwnd)); $bThen = [LX]::Topmost((FenceHwnd $script:FidB))
  Shot 'shortcut-b-over-launcher'
  Check 'B is up, the launcher went back below' ($aFirst -and $bThen -and -not $aThen -and [bool](PfWaitLog 'the Peek of another fence alone ends first' 3)) "launcher up first=$aFirst, then launcher=$aThen B=$bThen"

  Case 'B stops a third time within an hour: its shortcut goes, the launcher keeps its own'
  # Its command from the key above stopped it once; it starts again 5 s after each stop.
  for ($n = 2; $n -le 3; $n++) { Key 0x4B; Start-Sleep -Seconds 5; Key 0x4B }
  $l = LogCount $UnregL
  Check 'three stops, then Ctrl+Alt+K is unregistered' ((WaitCount 'plugin fence stopped' 3) -and (WaitCount $UnregK 1 8) -and (Last 'K') -match 'unregistered' -and (LogCount $UnregL) -eq $l) "stops=$(LogCount 'plugin fence stopped') unregistered K=$(LogCount $UnregK)"
  [G]::Press(0x1B); Start-Sleep -Seconds 1; CloseWindows

  Case 'uninstalled: the shortcut goes'
  $r = (FenceGet $script:FidB).windowRect; [G]::RClick($r.x + 120, $r.y + $r.h - 40)
  MenuPick '^重新启动插件' (MenuWin 5) | Out-Null
  Check 'B started by hand: Ctrl+Alt+K is registered again' (WaitCount 'plugin shortcut registered.*Ctrl\+Alt\+K' 2 10) ''
  PfSettings | Out-Null
  Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginUninstall', id: 'launcher-b' })" | Out-Null
  Check 'B uninstalled: Ctrl+Alt+K is unregistered' ((WaitCount $UnregK 2 10) -and (Last 'K') -match 'unregistered' -and -not (Test-Path "$script:Config\plugins\launcher-b\1.0.0\main.js")) ''

  Case 'the Peek''s key is refused for a command, by its name'
  Pf settings set peek.doubleCtrl false | Out-Null; Start-Sleep -Seconds 1
  $help = SetKey $script:Fid 0x20
  Check 'Ctrl+Alt+Space: 已经是“速览”的快捷键, the shortcut stays Ctrl+Alt+L' ($help -match '已经是“速览”的快捷键' -and (Keys).show -eq 'Ctrl+Alt+L') "$help"

  Case 'an update without the command: the shortcut goes'
  $l = LogCount $UnregL
  Publish 3 '1.2.0' @()
  PfSettings | Out-Null; HostRow 'launcher' | Out-Null
  Check 'launcher 1.2.0 without 显示启动器 installs by itself, Ctrl+Alt+L is unregistered' ((WaitCount $UnregL ($l + 1) 20) -and (Last 'L') -match 'unregistered' -and (Get-Content "$script:Config\plugins\launcher\current.txt") -eq '1.2.0') "current=$(Get-Content "$script:Config\plugins\launcher\current.txt")"

  Case 'taken down: the shortcut goes'
  Publish 4 '1.3.0' $cmds
  HostRow 'launcher' | Out-Null
  Js "(() => { const l = window.pfRows.v.list.find(x => x.id === 'launcher'); window.chrome.webview.postMessage({ type: 'action', name: 'pluginInstall', id: 'launcher', shown: l.shown }); return true; })()" | Out-Null
  PfWaitLog 'plugins: installed, its fences start again' 20 | Out-Null; Start-Sleep -Seconds 3
  $help = SetKey $script:Fid 0x4C
  $l = LogCount $UnregL
  Check 'with 1.3.0 the command is back and Ctrl+Alt+L is set again' ((Get-Content "$script:Config\plugins\launcher\current.txt") -eq '1.3.0' -and (Last 'L') -notmatch 'unregistered') "$help"
  Publish 5 '1.3.0' $cmds @{} 'Test reason'
  PfSettings | Out-Null; HostRow 'launcher' | Out-Null
  Check 'the take-down unregisters Ctrl+Alt+L' ((WaitCount $UnregL ($l + 1) 15) -and (Last 'L') -match 'unregistered') ''
}
Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue
