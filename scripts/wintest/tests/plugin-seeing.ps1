# 插件 interface 3, plan v2 "Pointer, focus, shortcuts" (L2, L12, L14): the plugin in assets\seeing may know
# where the pointer is (pf.pointer, the `pointer` event, at most 30 a second while it moves) and where the
# keyboard focus and the text caret are (pf.focus, pf.caret, the `focus` event): GetGUIThreadInfo and MSAA
# in Notepad; in Edge the MSAA caret and, as MSAA's focus stops at the document there, UI Automation's
# focused element (a text area, an input, a button reached by Tab). A copy without the permissions is
# refused and gets no events; the reads stop while no fence of such a plugin shows; PecoFence's own window
# in front keeps what was seen. Edge's processor time and memory with a focus plugin shown against none.
# Its command brings Ctrl+Shift+F9: 添加到桌面 gives it to the new fence, a press sends the command, 清除
# keeps it cleared (null), a fence whose shortcut another fence holds gets none.
# Debug build (CDP on the settings page), Chinese interface.
$Kit = 'C:\pf\kit\assets\seeing'
$Sr = 'C:\pf\run\seeing'
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class SX {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  public static int[] Rect(IntPtr h) { RECT r; GetWindowRect(h, out r); return new int[] { r.L, r.T, r.R, r.B }; }
}
'@
function Out($fid) { return FenceOut $fid }
# "x,y,w,h" after `word` in the plugin's rows, as four numbers (none: $null).
function RectOf($fid, $word) { if ((Out $fid) -match "$word (-?\d+),(-?\d+),(\d+),(\d+)") { return @([int]$Matches[1], [int]$Matches[2], [int]$Matches[3], [int]$Matches[4]) }; return $null }
function Count($fid, $word) { if ((Out $fid) -match "$word [^\n]*events (\d+)") { return [int]$Matches[1] }; return -1 }
function PointerAt($fid) { if ((Out $fid) -match 'pointer (-?\d+),(-?\d+) monitor') { return @([int]$Matches[1], [int]$Matches[2]) }; return $null }
# Rectangle `r` (x, y, w, h) inside `win` (left, top, right, bottom), 2 px of slack.
function Inside($r, $win) { return $r -and $r[0] -ge $win[0] - 2 -and $r[1] -ge $win[1] - 2 -and $r[0] + $r[2] -le $win[2] + 2 -and $r[1] + $r[3] -le $win[3] + 2 }
# Whether rectangle `a` (x, y, w, h) holds the top-left point of `b`.
function Holds($a, $b) { return $a -and $b -and $b[0] -ge $a[0] - 2 -and $b[1] -ge $a[1] - 2 -and $b[0] -le $a[0] + $a[2] + 2 -and $b[1] -le $a[1] + $a[3] + 2 }
function Keys($fid) { return (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts."$fid".settings.__keys }
function Wait($test, $sec = 6) { for ($i = 0; $i -lt $sec * 4; $i++) { if (& $test) { return $true }; Start-Sleep -Milliseconds 250 }; return $false }
function CtrlShift([int]$vk) { [G]::KeyDown(0x11); [G]::KeyDown(0x10); Start-Sleep -Milliseconds 40; [G]::Press($vk); Start-Sleep -Milliseconds 40; [G]::KeyUp(0x10); [G]::KeyUp(0x11) }
# The newest fence of plugin `id` that is not in `old`.
function NewFence($id, $old) { return @(PluginFences $id | Where-Object { $old -notcontains $_.id })[0].id }
$EdgeExe = @("${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe", "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe") | Where-Object { Test-Path $_ } | Select-Object -First 1
# An Edge window of its own profile on page `page`, at 600,200 800x600: [hwnd, the msedge pids that
# ran before it].
function EdgeOpen($page, $profile) {
  $before = @(Get-Process msedge -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
  Start-Process $EdgeExe -ArgumentList '--new-window', '--no-first-run', "--user-data-dir=$profile", "file:///$($page -replace '\\', '/')"
  $ew = $null
  for ($i = 0; $i -lt 40 -and -not $ew; $i++) { $ew = @(@([W]::Tops('Chrome_WidgetWin_1')) | Where-Object { [L]::Text($_) -match 'seeing' })[0]; if (-not $ew) { Start-Sleep -Milliseconds 500 } }
  [L]::MoveWindow($ew, 600, 200, 800, 600, $true) | Out-Null; Start-Sleep -Seconds 2
  return @($ew, $before)
}
# Processor time (ms) and private memory (MB) over `sec` seconds of Edge's processes that are not in
# `before` (its other processes: started before this Edge), counted per process alive at both ends.
function Use($before, $sec) {
  $mine = { @(Get-Process msedge -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id }) }
  $t0 = @{}; foreach ($p in & $mine) { $t0[$p.Id] = $p.TotalProcessorTime.TotalMilliseconds }
  Start-Sleep -Seconds $sec
  $now = & $mine
  $ms = 0; foreach ($p in $now) { if ($t0.ContainsKey($p.Id)) { $ms += $p.TotalProcessorTime.TotalMilliseconds - $t0[$p.Id] } }
  $mb = [math]::Round(($now | Measure-Object -Property PrivateMemorySize64 -Sum).Sum / 1MB, 1)
  return @([math]::Round($ms), $mb, $now.Count)
}

Case 'setup: the plugin with the permissions and a copy without them'
CloseWindows
New-Item -ItemType Directory -Force $Sr | Out-Null
$script:Config = "$PfApp\config"
PfPlacePlugin $Kit 'seeing'
PfPlacePlugin $Kit 'blind'
$bd = "$script:Config\plugins\blind\1.0.0"
$m = Get-Content "$bd\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$m.permissions = [ordered]@{}; $m.name.'zh-CN' = '看不见'
[IO.File]::WriteAllText("$bd\plugin.json", ($m | ConvertTo-Json -Depth 6))
# A copy that runs on while hidden (`background`): hidden, it is told nothing all the same.
PfPlacePlugin $Kit 'seeing-bg'
$gd = "$script:Config\plugins\seeing-bg\1.0.0"
$m = Get-Content "$gd\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$m.permissions | Add-Member -NotePropertyName background -NotePropertyValue $true
$m.permissions.commands = @(); $m.name.'zh-CN' = '后台看位置'
[IO.File]::WriteAllText("$gd\plugin.json", ($m | ConvertTo-Json -Depth 6))
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang zh-CN
$script:See = PfPluginFence '看位置' '40,60,520,170' 'seeing'
$script:Blind = PfPluginFence '看不见' '40,260,520,170' 'blind'
$script:Bg = PfPluginFence '后台看位置' '40,460,520,170' 'seeing-bg'
Check 'both plugins start' ((Wait { (Out $script:See) -match 'pull pointer' -and (Out $script:Blind) -match 'pull refused' } 20)) "$(Out $script:See) | $(Out $script:Blind)"
Check 'the pointer and focus reads start with the plugin that may know of them' ([bool](PfWaitLog 'one may know where the pointer is' 5) -and [bool](PfWaitLog 'one may know where the focus and the caret are' 5)) ''
Shot 'fences'

Case 'pointer: the event follows the pointer, at most 30 a second, none while it rests'
[G]::MoveTo(800, 500); Start-Sleep -Milliseconds 600
$p = PointerAt $script:See
Check 'the event says where the pointer is (800, 500) and its monitor' ($p -and [Math]::Abs($p[0] - 800) -le 2 -and [Math]::Abs($p[1] - 500) -le 2 -and (Out $script:See) -match 'monitor 0,0,\d+,\d+') "$(Out $script:See)"
$n0 = Count $script:See 'pointer'
Glide 700 400 1100 700 100 25
Start-Sleep -Milliseconds 400
$n1 = Count $script:See 'pointer'
Check 'a 2.5 s move gives between 20 and 80 pointer events' (($n1 - $n0) -ge 20 -and ($n1 - $n0) -le 80) "events $n0 -> $n1"
Start-Sleep -Seconds 2
$n2 = Count $script:See 'pointer'
Check 'no pointer event while it rests' ($n2 -eq $n1) "events $n1 -> $n2"
Check 'pf.pointer answers too' ((Out $script:See) -match 'pull pointer \d+,\d+') "$(Out $script:See)"
Check 'without the permissions: pf.pointer is refused and no event comes' ((Out $script:Blind) -match 'refused pointer needs permissions.pointer' -and (Count $script:Blind 'pointer') -eq 0 -and (Count $script:Blind 'focus') -eq 0) "$(Out $script:Blind)"

Case 'focus and caret in Notepad (GetGUIThreadInfo, MSAA)'
Start-Process notepad.exe
$np = WaitWin 'Notepad' 10
[L]::MoveWindow($np, 640, 280, 700, 420, $true) | Out-Null; Start-Sleep -Milliseconds 800
[G]::Click(980, 480); Start-Sleep -Milliseconds 500
[G]::Type('hello focus'); Start-Sleep -Milliseconds 1500
$win = [SX]::Rect($np)
$caret = RectOf $script:See 'caret'; $focus = RectOf $script:See 'focus'
Check 'the caret is a narrow box inside Notepad' ((Inside $caret $win) -and $caret[2] -le 8 -and $caret[3] -ge 8) "caret=$($caret -join ',') win=$($win -join ',') $(Out $script:See)"
Check 'the focus box is inside Notepad and holds the caret' ((Inside $focus $win) -and (Holds $focus $caret)) "focus=$($focus -join ',') win=$($win -join ',')"
Log "notepad: $(PfWaitLog 'plugins: the focus or the caret moved' 1)"
[G]::Press(0x24); Start-Sleep -Milliseconds 1200
$atHome = RectOf $script:See 'caret'
Check 'Home moves the caret left; the event follows' ($atHome -and $atHome[0] -lt $caret[0] - 20) "before=$($caret -join ',') after=$($atHome -join ',')"
Check 'pf.caret answers the same place' ((Wait { (Out $script:See) -match "pull pointer \d+,\d+ focus \S+ caret $($atHome[0]),$($atHome[1])," } 4)) "$(Out $script:See)"
Shot 'notepad'

Case 'PecoFence''s own window in front keeps what was seen'
$before = RectOf $script:See 'caret'
PfSettings | Out-Null; $sw = WaitWin 'PecoFence.Settings' 5; Start-Sleep -Seconds 2
$after = RectOf $script:See 'caret'
Check 'with the settings window in front the caret stays where Notepad had it' ($after -and $after[0] -eq $before[0] -and $after[1] -eq $before[1]) "before=$($before -join ',') after=$($after -join ',')"
[L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1
CloseWindows

Case 'focus and caret in Edge: a text area, an input, a button reached by Tab'
$page = "$Sr\page.html"
# Fixed places in the page, from its top-left corner: the text area, then the input and the button.
[IO.File]::WriteAllText($page, '<!doctype html><title>seeing</title><body style="margin:0;font:16px sans-serif"><textarea id="t" style="position:absolute;left:40px;top:40px;width:500px;height:160px"></textarea><input id="i" style="position:absolute;left:40px;top:240px;width:200px;height:24px"><button id="b" style="position:absolute;left:300px;top:240px;width:120px;height:28px">A button</button></body>')
if ($EdgeExe) {
  $e = EdgeOpen $page "$Sr\edge"; $ew = $e[0]
  $win = [SX]::Rect($ew)
  # A click on the bare page: the page itself has the focus, its box gives the page's corner.
  [G]::Click($win[0] + 700, $win[1] + 520); Start-Sleep -Milliseconds 1200
  $doc = RectOf $script:See 'focus'
  $ox = if ($doc) { $doc[0] } else { $win[0] + 12 }; $oy = if ($doc) { $doc[1] } else { $win[1] + 80 }
  Log "edge page corner $ox,$oy (page focus $($doc -join ','))"
  [G]::Click($ox + 290, $oy + 120); Start-Sleep -Milliseconds 500
  [G]::Type('hello edge'); Start-Sleep -Seconds 2
  $caret = RectOf $script:See 'caret'; $ta = RectOf $script:See 'focus'
  $line = "$(PfWaitLog 'plugins: the focus or the caret moved' 1)"
  Log "edge text area: $line"
  Check 'Edge text area: the caret is inside the window (MSAA)' ((Inside $caret $win) -and $line -match 'caret_by="msaa"') "caret=$($caret -join ',') win=$($win -join ',') $line"
  Check 'Edge text area: the focus is the text area (MSAA, or UI Automation while Edge''s tree is not ready), about 500 x 160 at its place, holding the caret' ($ta -and [Math]::Abs($ta[0] - ($ox + 40)) -le 6 -and [Math]::Abs($ta[1] - ($oy + 40)) -le 6 -and [Math]::Abs($ta[2] - 500) -le 12 -and [Math]::Abs($ta[3] - 160) -le 12 -and (Holds $ta $caret) -and $line -match 'focus_by="(msaa|uia)"') "focus=$($ta -join ',') corner=$ox,$oy $line"
  $uiaLines = @(PfLogAll | Where-Object { $_ -match 'focus or the caret moved.*focus_by="uia"' }).Count
  Log "edge: reads whose focus came from UI Automation: $uiaLines"
  [G]::Click($ox + 140, $oy + 252); Start-Sleep -Milliseconds 500
  [G]::Type('in'); Start-Sleep -Seconds 1
  $ir = RectOf $script:See 'focus'; $ic = RectOf $script:See 'caret'
  Check 'Edge input: the focus is the input box (200 x 24 at its place), holding the caret' ($ir -and [Math]::Abs($ir[0] - ($ox + 40)) -le 6 -and [Math]::Abs($ir[1] - ($oy + 240)) -le 6 -and $ir[3] -le 40 -and (Holds $ir $ic)) "input=$($ir -join ',') caret=$($ic -join ',') corner=$ox,$oy"
  [G]::Press(0x09); Start-Sleep -Milliseconds 1500
  $br = RectOf $script:See 'focus'
  Check 'Tab to the button: the focus is the button (120 x 28 right of the input), no caret' ($br -and [Math]::Abs($br[0] - ($ox + 300)) -le 6 -and [Math]::Abs($br[1] - ($oy + 240)) -le 6 -and (Out $script:See) -match 'caret none') "button=$($br -join ',') $(Out $script:See)"
  Shot 'edge'
  CloseWindows
} else { Log 'Edge is not installed: the Edge checks are left out' }

Case 'Edge''s processor time and memory: a focus plugin shown against none'
if ($EdgeExe) {
  # A profile of its own: a browser process no reader has asked before.
  Pf fence hide-all | Out-Null
  PfWaitLog 'none knows where the focus and the caret are now' 4 | Out-Null
  $e = EdgeOpen $page "$Sr\edge2"; $ew = $e[0]; $others = $e[1]
  $win = [SX]::Rect($ew)
  [G]::Click($win[0] + 300, $win[1] + 200); Start-Sleep -Milliseconds 300; [G]::Type('quiet')
  # Edge's start and the page's first paint settle first.
  Start-Sleep -Seconds 10
  $rows = @()
  $pf = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
  foreach ($round in 1, 2) {
    $off = Use $others 10
    Pf fence show-all | Out-Null; Start-Sleep -Seconds 1
    [G]::Click($win[0] + 300, $win[1] + 200); Start-Sleep -Seconds 3
    $pf.Refresh(); $p0 = $pf.TotalProcessorTime.TotalMilliseconds
    $on = Use $others 10
    $pf.Refresh(); $p1 = $pf.TotalProcessorTime.TotalMilliseconds
    Pf fence hide-all | Out-Null; Start-Sleep -Seconds 2
    $rows += "round $round`: none shown $($off[0]) ms, $($off[1]) MB ($($off[2]) processes); a focus plugin shown $($on[0]) ms, $($on[1]) MB ($($on[2]) processes); pecofence $([math]::Round($p1 - $p0)) ms"
  }
  Pf fence show-all | Out-Null
  Log "edge use per 10 s: $($rows -join ' | ')"
  Check 'measured: Edge with and without a focus plugin shown (see the log line)' ($rows.Count -eq 2) ($rows -join ' | ')
  CloseWindows
}

Case 'the reads stop while no fence of such a plugin shows'
$bg0 = Count $script:Bg 'pointer'
Pf fence hide-all | Out-Null
Check 'all fences hidden: the pointer and focus reads stop' ([bool](PfWaitLog 'none knows where the pointer is now' 4) -and [bool](PfWaitLog 'none knows where the focus and the caret are now' 4)) ''
Glide 500 500 900 600 30 25
Start-Sleep -Seconds 2
Check 'a hidden fence of a background plugin gets no pointer event' ((Count $script:Bg 'pointer') -eq $bg0) "events $bg0 -> $(Count $script:Bg 'pointer')"
Check 'and its calls answer null while hidden (its timer runs on)' ((Wait { (Out $script:Bg) -match 'pull pointer none focus none caret none' } 4)) "$(Out $script:Bg)"
Pf fence show-all | Out-Null; Start-Sleep -Seconds 1
Check 'shown again: the reads start again' ((LogCount 'one may know where the pointer is') -ge 2) "count=$(LogCount 'one may know where the pointer is')"

Case 'the command''s own shortcut: given to the new fence, a press sends the command'
PfSettings | Out-Null; $sw = WaitWin 'PecoFence.Settings' 5
$old = @(PluginFences 'seeing' | ForEach-Object { $_.id })
Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginAdd', id: 'seeing' })" | Out-Null
Wait { @(PluginFences 'seeing').Count -gt $old.Count } 10 | Out-Null
$script:New = NewFence 'seeing' $old
Check 'the new fence has Ctrl+Shift+F9 for 显示, and it is registered' ($script:New -and (Wait { (Keys $script:New).show -eq 'Ctrl+Shift+F9' } 8) -and [bool](PfWaitLog "plugin's own shortcut given to its fence.*Ctrl\+Shift\+F9" 5) -and [bool](PfWaitLog 'plugin shortcut registered.*Ctrl\+Shift\+F9' 5)) "new=$script:New keys=$(Keys $script:New | ConvertTo-Json -Compress)"
Check 'the fence made before it got none (only an install gives one)' ($null -eq (Keys $script:See)) "$(Keys $script:See | ConvertTo-Json -Compress)"
[L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1
Start-Process notepad.exe; $np = WaitWin 'Notepad' 10; Start-Sleep -Milliseconds 800
CtrlShift 0x78
Check 'Ctrl+Shift+F9 sends 显示 to the new fence' ((Wait { (Out $script:New) -match 'commands show/key' } 5)) "$(Out $script:New)"
CloseWindows

Case '清除 keeps it cleared; another fence then may have it, a third not'
PfSettings | Out-Null; $sw = WaitWin 'PecoFence.Settings' 5
Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginKey', id: 'seeing', fence: '$script:New', command: 'show', box: 'x' })" | Out-Null
$cleared = { $k = Keys $script:New; $k -and ($k.PSObject.Properties.Name -contains 'show') -and $null -eq $k.show }
Check 'cleared: the fence keeps show = null, the shortcut is unregistered' ((Wait $cleared 8) -and [bool](PfWaitLog 'plugin shortcut unregistered.*Ctrl\+Shift\+F9' 4)) "$(Keys $script:New | ConvertTo-Json -Compress)"
$old = @(PluginFences 'seeing' | ForEach-Object { $_.id })
Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginAdd', id: 'seeing' })" | Out-Null
Wait { @(PluginFences 'seeing').Count -gt $old.Count } 10 | Out-Null
$third = NewFence 'seeing' $old
Check 'a fence added after the clear gets Ctrl+Shift+F9' ($third -and (Wait { (Keys $third).show -eq 'Ctrl+Shift+F9' } 8)) "third=$third $(Keys $third | ConvertTo-Json -Compress)"
$old = @(PluginFences 'seeing' | ForEach-Object { $_.id })
Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginAdd', id: 'seeing' })" | Out-Null
Wait { @(PluginFences 'seeing').Count -gt $old.Count } 10 | Out-Null
$fourth = NewFence 'seeing' $old
Check 'one more fence gets none: another fence holds it' ($fourth -and [bool](PfWaitLog "plugin's own shortcut not given.*held=true" 4) -and $null -eq (Keys $fourth)) "fourth=$fourth $(Keys $fourth | ConvertTo-Json -Compress)"
Check 'the cleared fence is still cleared' (& $cleared) "$(Keys $script:New | ConvertTo-Json -Compress)"
[L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
