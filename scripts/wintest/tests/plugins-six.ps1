# The six tool plugins of the plugin repository (notes, clock, pc-status, weather, github,
# clipboard-history) from a local test index on the build under test, each checked at what it
# does, and pictures of each next to the tool it replaces, at the same place and size, light and
# dark. Chinese UI. --extra a folder with feed\ (tools/build-index.py --out), old\ (the 0.4.0
# example scripts weather.zh.0.4.0.ps1, github.zh.0.4.0.ps1, status.zh.0.4.0.ps1) and
# clipboard-history.ps1 (#223's script fence).
#   --var Part=Old   on 0.4.0 (--bin): the note, the clock and the three example script fences
#   (default)        a debug build (CDP on the settings page): the six plugins, and #223's
#                    script fence beside the clipboard history plugin
# Pictures: shots\<old|new>-<tool>-<light|dark>.png, cut to the fence.
# manual in suite.txt: the --extra folder comes from the plugin repository (tools/build-index.py),
# made by hand for the version under test.
$Rects = [ordered]@{
  notes = '40,60,300,220'; clock = '360,60,260,150'; 'pc-status' = '640,60,320,330'
  weather = '980,60,440,170'; github = '40,420,520,360'; clipboard = '980,420,380,420'; script223 = '580,420,380,420'
}
$Note = "[ ] 买牛奶`n[x] 回邮件`n周五前交报告"
function Cut($key, $name) {
  $f = $script:Fences[$key]; if (-not $f) { Log "Cut: no fence $key"; return }
  $r = (FenceGet $f).windowRect
  Crop $name $r.x $r.y $r.w $r.h | Out-Null
}
# Light and dark on the box's own wallpaper (the fences' text follows what is under them).
function Pictures($who) {
  ParkMouse 1500 880
  foreach ($theme in 'light', 'dark') {
    Pf settings set theme $theme | Out-Null; Start-Sleep -Seconds 4
    foreach ($n in @($script:Fences.Keys)) { Cut $n "$who-$n-$theme" }
    Shot "$who-desktop-$theme"
  }
  Pf settings set theme light | Out-Null
}
# Only the inbox fence is left (it lays its icons out as the desktop does, without a box).
function Room() { ClearFences }
function IdOf($json) { return ([regex]::Match(($json | Out-String), '[0-9a-f]{8}-[0-9a-f-]{27}')).Value }

if ($Part -eq 'Old') {
  Case '0.4.0: the tools the plugins replace'
  CloseWindows
  PfStart -Lang zh-CN
  Room
  Pf settings set tools.clockSeconds true | Out-Null
  $script:Fences = [ordered]@{}
  $script:Fences.notes = IdOf (Pf fence create --tool note --title 便签 --text $Note --rect $Rects.notes)
  $script:Fences.clock = IdOf (Pf fence create --tool clock --title 时钟 --rect $Rects.clock)
  $script:Fences.'pc-status' = IdOf (Pf fence create --script "$PfExtra\old\status.zh.0.4.0.ps1" --title 电脑状态 --rect $Rects.'pc-status')
  $script:Fences.weather = IdOf (Pf fence create --script "$PfExtra\old\weather.zh.0.4.0.ps1" --title 天气 --rect $Rects.weather)
  $script:Fences.github = IdOf (Pf fence create --script "$PfExtra\old\github.zh.0.4.0.ps1" --title 'GitHub 仓库动态' --rect $Rects.github)
  Start-Sleep -Seconds 25
  foreach ($n in $script:Fences.Keys) { Log "$n $($script:Fences[$n]): $((FenceGet $script:Fences[$n]).script.output -replace "`n", ' | ')" }
  Check 'every tool fence is there' (@($script:Fences.Values | Where-Object { $_ }).Count -eq 5) ($script:Fences | Out-String)
  Pictures 'old'
  PfQuit
  return
}

$Feed = 'C:\pf\run\pfeed'
$script:Fences = [ordered]@{}
function Plugin($id) { return @(@((PfJson fence list)) | Where-Object { $_ -and $_.script -and $_.script.path -eq "plugin:$id" })[0] }
function Shows($id) { return [string](FenceGet (Plugin $id).id).script.output }
function Rows() { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].map(r => r.textContent).join(' / ')") }
function Config() { return Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json }
# A click on row `row` (cell `cell`) of plugin `id`'s fence, read from the fence's dump.
function ClickRow($id, $row, $cell = -1, [switch]$Double) {
  $title = (Plugin $id).title
  $w = DumpOf $title 5
  $field = if ($cell -ge 0) { 'script_cells' } else { 'script_rows' }
  $want = if ($cell -ge 0) { $row * 100 + $cell } else { $row }
  $v = DumpVal $w $field
  $hit = @($v -split ';' | Where-Object { $_ -match "^${want}:" })[0]
  if (-not $hit) { Log "ClickRow: no $field $want in $title ($v)"; return $false }
  $n = ($hit -split ':')[1] -split ','
  $sc = [double](DumpVal $w 'content_y') / 36.0
  $x = [int]($w.l + [double](DumpVal $w 'content_x') + ([double]$n[0] + [double]$n[2]) / 2 * $sc)
  $y = [int]($w.t + [double](DumpVal $w 'content_y') + ([double]$n[1] + [double]$n[3]) / 2 * $sc)
  if ($Double) { [G]::Click($x, $y); Start-Sleep -Milliseconds 90; [G]::Click($x, $y) } else { [G]::Click($x, $y) }
  Start-Sleep -Milliseconds 900; ParkMouse 1500 880
  return $true
}
# Puts something on the clipboard from an STA PowerShell: text, files, a picture, or text marked
# private as password managers mark it.
function Clip($kind, $value) {
  $code = switch ($kind) {
    'text' { "Add-Type -A System.Windows.Forms; [Windows.Forms.Clipboard]::SetText('$value')" }
    'files' { "Add-Type -A System.Windows.Forms; `$c = New-Object Collections.Specialized.StringCollection; '$value'.Split('|') | % { [void]`$c.Add(`$_) }; [Windows.Forms.Clipboard]::SetFileDropList(`$c)" }
    'image' { "Add-Type -A System.Windows.Forms, System.Drawing; `$b = New-Object Drawing.Bitmap 320, 180; `$g = [Drawing.Graphics]::FromImage(`$b); `$g.Clear([Drawing.Color]::SteelBlue); `$g.FillEllipse([Drawing.Brushes]::Gold, 110, 40, 100, 100); [Windows.Forms.Clipboard]::SetImage(`$b)" }
    'private' { "Add-Type -A System.Windows.Forms; `$d = New-Object Windows.Forms.DataObject; `$d.SetData('UnicodeText', '$value'); `$d.SetData('ExcludeClipboardContentFromMonitorProcessing', (New-Object IO.MemoryStream (,[byte[]](0)))); [Windows.Forms.Clipboard]::SetDataObject(`$d, `$true)" }
  }
  & powershell.exe -NoProfile -STA -Command $code
  Start-Sleep -Milliseconds 1200
}

Case 'setup: a test index with the six plugins'
CloseWindows
Remove-Item $Feed -Recurse -Force -ErrorAction SilentlyContinue
Copy-Item "$PfExtra\feed" $Feed -Recurse
$server = @'
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), 8731); $l.Start()
while ($true) { $c = $l.AcceptTcpClient(); try { $s = $c.GetStream(); $r = New-Object IO.StreamReader($s); $line = $r.ReadLine(); while ($r.ReadLine()) {}
  $path = 'C:\pf\run\pfeed' + (($line -split ' ')[1] -replace '/', '\')
  if (Test-Path -LiteralPath $path -PathType Leaf) { $b = [IO.File]::ReadAllBytes($path); $h = "HTTP/1.1 200 OK`r`nContent-Length: $($b.Length)`r`nConnection: close`r`n`r`n" } else { $b = @(); $h = "HTTP/1.1 404 Not Found`r`nContent-Length: 0`r`nConnection: close`r`n`r`n" }
  $hb = [Text.Encoding]::ASCII.GetBytes($h); $s.Write($hb, 0, $hb.Length); if ($b.Length) { $s.Write($b, 0, $b.Length) }; $s.Flush() } catch {} finally { $c.Close() } }
'@
Set-Content 'C:\pf\run\pfeed-serve.ps1' $server
$feedProc = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\run\pfeed-serve.ps1' -WindowStyle Hidden -PassThru
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8731/index.json'
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Script (DumpScript) -Lang zh-CN
$script:Config = Split-Path (PfJson status).configPath
Room
$ok = PfSettings
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
$names = [ordered]@{ notes = '便签'; clock = '时钟'; 'pc-status' = '电脑状态'; weather = '天气'; github = 'GitHub 仓库动态'; 'clipboard-history' = '剪贴板历史' }
Check 'the tools page lists the six plugins' ($ok -and @($names.Values | Where-Object { (Rows) -notmatch [regex]::Escape($_) }).Count -eq 0) (Rows)

Case 'install: each plugin from its row, each in a fence'
foreach ($id in $names.Keys) {
  $i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$($names[$id])'))")
  PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 700
  if ($id -eq 'clipboard-history') { Shot 'install-warning-clipboard-history' }
  PageClick '#plOk'; Start-Sleep -Seconds 4
}
[L]::PostMessageW((WaitWin 'PecoFence.Settings' 2), 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
$missing = @($names.Keys | Where-Object { -not (Plugin $_) })
Check 'six plugin fences' ($missing.Count -eq 0) "missing: $($missing -join ', ')"
$script:Fences = [ordered]@{ notes = (Plugin notes).id; clock = (Plugin clock).id; 'pc-status' = (Plugin pc-status).id; weather = (Plugin weather).id; github = (Plugin github).id; clipboard = (Plugin clipboard-history).id }
foreach ($n in $script:Fences.Keys) { if ($script:Fences[$n]) { Pf fence move $script:Fences[$n] --rect $Rects[$n] | Out-Null } }

Case 'the note text, the city and the seconds kept with each fence, through a restart'
PfQuit
$c = Config
function SetProp($o, $name, $value) { if ($o.PSObject.Properties[$name]) { $o.$name = $value } else { $o | Add-Member -NotePropertyName $name -NotePropertyValue $value } }
if (-not $c.notes) { SetProp $c 'notes' ([pscustomobject]@{}) }
SetProp $c.notes $script:Fences.notes $Note
foreach ($k in 'weather', 'clock') { $s = $c.scripts.($script:Fences[$k]); if (-not $s.settings) { SetProp $s 'settings' ([pscustomobject]@{}) } }
SetProp $c.scripts.($script:Fences.weather).settings 'city' 'Tokyo'
SetProp $c.scripts.($script:Fences.clock).settings 'seconds' $true
[IO.File]::WriteAllText("$script:Config\config.json", ($c | ConvertTo-Json -Depth 30))
PfStart -Script (DumpScript) -Lang zh-CN
Start-Sleep -Seconds 12
Check 'notes: the note shows its text' ((DumpVal (DumpOf (Plugin notes).title) 'note_boxes') -match '^0:') "boxes=$(DumpVal (DumpOf (Plugin notes).title) 'note_boxes') config=$((Config).notes.($script:Fences.notes))"
Check 'clock: the time with seconds and the date' ((Shows clock) -match '^\d{1,2}:\d{2}:\d{2}') (Shows clock)

Case 'clock: a narrow fence shows hours and minutes without AM or PM, as the native clock'
Pf fence resize $script:Fences.clock --w 70 | Out-Null; Start-Sleep -Seconds 2
$r = (FenceGet $script:Fences.clock).windowRect
# OCR misses the thin digits now and then: up to three reads.
$t = ''; for ($i = 0; $i -lt 3 -and $t -notmatch '\d'; $i++) { $t = OcrText $r.x ($r.y + 30) $r.w 70 'narrow-clock'; Start-Sleep -Milliseconds 500 }
Check 'the narrow clock reads like 9:31' ($t -match '\d{1,2}.?\d{2}' -and $t -notmatch '[AP]\.?M|\d\s*[AP]\b') "ocr=$t"
Pf fence move $script:Fences.clock --rect $Rects.clock | Out-Null; Start-Sleep -Seconds 1

Case 'notes: edited in place, a to-do ticked without the editor'
$w = DumpOf (Plugin notes).title
$b = @((DumpVal $w 'note_boxes') -split ';' | Where-Object { $_ -match '^0:' })[0]
$sc = [double](DumpVal $w 'content_y') / 36.0
if ($b) {
  $n = ($b -split ':')[1] -split ','
  [G]::Click([int]($w.l + [double](DumpVal $w 'content_x') + ([double]$n[0] + [double]$n[2]) / 2 * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + ([double]$n[1] + [double]$n[3]) / 2 * $sc))
  Start-Sleep -Seconds 1
}
$ed = WaitWin 'PecoFence.NoteEdit' 1
Check 'a click on the first box ticks it without the editor: [x] 买牛奶' ((Config).notes.($script:Fences.notes) -match '^\[x\] 买牛奶' -and $ed -eq [IntPtr]::Zero) "box=$b editor=$ed text=$((Config).notes.($script:Fences.notes))"
$nr = (FenceGet $script:Fences.notes).windowRect
[G]::Click($nr.x + 150, $nr.y + 110)
$ed = WaitWin 'PecoFence.NoteEdit' 6
[G]::Press(0x23); [G]::Type('，再买面包'); [G]::Press(0x1B); Start-Sleep -Seconds 1
$text = (Config).notes.($script:Fences.notes)
Check 'a click on the text opens the editor; what is typed is kept with the fence' ($ed -ne [IntPtr]::Zero -and $text -match '周五前交报告，再买面包' -and $text -match '回邮件') "editor=$ed text=$text"

Case 'pc-status: values, the IP copied, Task Manager'
$s = Shows pc-status
Check 'CPU, memory, a drive' ($s -match 'CPU \d+%' -and $s -match '内存已用 \d+%' -and $s -match 'C 盘剩余 \d+ GB') ($s -replace "`n", ' | ')
$ip = [regex]::Match($s, '本机 IP (\d+\.\d+\.\d+\.\d+)').Groups[1].Value
ClickRow pc-status 3 | Out-Null
Check 'a click on the IP row copies it' ((Get-Clipboard) -eq $ip -and $ip) "ip=$ip clipboard=$(Get-Clipboard) shows=$((Shows pc-status) -replace "`n", ' | ')"
$before = @(Get-Process Taskmgr -ErrorAction SilentlyContinue).Id
ClickRow pc-status 4 | Out-Null; Start-Sleep -Seconds 3
$tm = @(Get-Process Taskmgr -ErrorAction SilentlyContinue | Where-Object { $before -notcontains $_.Id })
Check '打开任务管理器 opens Task Manager' ($tm.Count -ge 1) ''
$tm | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }

Case 'weather: Tokyo from wttr.in'
$s = Shows weather
Check 'now, feels like, today and tomorrow, the time it updated' ($s -match '^Tokyo  -?\d+°C' -and $s -match '体感 -?\d+°C' -and $s -match '今天 -?\d+ 到 -?\d+°C，明天' -and $s -match '更新，点一下刷新') ($s -replace "`n", ' | ')

Case 'github: the newest issues and PRs, a click opens one'
$s = Shows github
Check 'the repository, items with numbers, refresh' ($s -match '^DayuanJiang/PecoFence' -and $s -match '#\d+  ' -and $s -match '更新，点一下刷新') ($s -replace "`n", ' | ')
$edge = @(Get-Process msedge -ErrorAction SilentlyContinue).Id
ClickRow github 2 | Out-Null; Start-Sleep -Seconds 3
Check 'a click on an item opens it in the browser' ([bool](PfWaitLog 'plugin opened a page' 3)) ''
Get-Process msedge -ErrorAction SilentlyContinue | Where-Object { $edge -notcontains $_.Id } | Stop-Process -Force -ErrorAction SilentlyContinue

Case 'clipboard history: texts, files, a picture, private content left out'
# What the clipboard holds now, read in an STA PowerShell: image, files:<paths> or text:<text>.
function ClipNow() { return (& powershell.exe -NoProfile -STA -Command "Add-Type -A System.Windows.Forms; `$c = [Windows.Forms.Clipboard]; if (`$c::ContainsImage()) { 'image' } elseif (`$c::ContainsFileDropList()) { 'files:' + ((`$c::GetFileDropList()) -join '|') } else { 'text:' + `$c::GetText() }") }
# The rows of the clipboard history fence that are entries (they have a pin cell).
function Entries() { $v = DumpVal (DumpOf (Plugin clipboard-history).title) 'script_cells'; return @($v -split ';' | ForEach-Object { [int]($_ -split ':')[0] } | Where-Object { $_ -ge 300 -and $_ % 100 -eq 2 } | ForEach-Object { [math]::Floor($_ / 100) }) }
Clip text 'PecoFence 剪贴板 1'
Clip text 'second copy'
Clip files 'C:\Windows\win.ini|C:\Windows\system.ini'
Clip image
Clip private 'secret password'
Start-Sleep -Seconds 1
# The IP address pc-status copied above is an entry too.
Check 'five entries (with the IP address pc-status copied): the private copy is left out' ((Entries).Count -eq 5) "entry rows=$((Entries) -join ',')"
$pngs = @(Get-ChildItem "$script:Config\plugins-data\clipboard-history" -Filter '*.png' -ErrorAction SilentlyContinue)
Check 'the picture is a file in the plugin''s data folder' ($pngs.Count -eq 1) ($pngs.Name -join ', ')
Shot 'clipboard-history-entries'
$got = @(foreach ($row in 3..7) { ClickRow clipboard-history $row 0 | Out-Null; ClipNow })
Check 'newest first, and a click copies each again: the picture, the files, both texts' ($got[0] -eq 'image' -and $got[1] -eq 'files:C:\Windows\win.ini|C:\Windows\system.ini' -and $got[2] -eq 'text:second copy' -and $got[3] -eq 'text:PecoFence 剪贴板 1' -and $got[4] -eq "text:$ip" -and ($got -notmatch 'secret')) ($got -join ' / ')
Check 'copying entries back makes no new entries' ((Entries).Count -eq 5) "entry rows=$((Entries) -join ',')"

Case 'clipboard history: pin, Show entries'
ClickRow clipboard-history 6 2 | Out-Null
$pin = Get-Content "$script:Config\plugins-data\clipboard-history\c-$($script:Fences.clipboard)-pinned.json" -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
ClickRow clipboard-history 3 0 | Out-Null
Check 'the pinned text is written down and goes to the top' ($pin -match 'PecoFence 剪贴板 1' -and (ClipNow) -eq 'text:PecoFence 剪贴板 1') "pinned=$pin rows=$((Entries) -join ',')"
ClickRow clipboard-history 0 0 | Out-Null
Shot 'clipboard-history-blurred'
$cfg = Get-Content "$script:Config\plugins-data\clipboard-history\c-$($script:Fences.clipboard)-settings.json" -Raw -ErrorAction SilentlyContinue
Check 'Show entries off is kept' ($cfg -match '"show":false') $cfg
ClickRow clipboard-history 0 0 | Out-Null

Case '#223''s script fence beside the plugin, then pictures'
$script:Fences.script223 = IdOf (Pf fence create --script "$PfExtra\clipboard-history.ps1" --title '剪贴板历史（脚本）' --rect $Rects.script223)
Start-Sleep -Seconds 6
Clip text 'second copy'
Clip files 'C:\Windows\win.ini|C:\Windows\system.ini'
Clip image
Start-Sleep -Seconds 2
Pictures 'new'
ClickRow clipboard-history 1 0 | Out-Null
Check 'Delete all keeps the pinned entry only' ((Entries).Count -eq 1 -and @(Get-ChildItem "$script:Config\plugins-data\clipboard-history" -Filter '*.png').Count -eq 0) "entry rows=$((Entries) -join ',')"

Case 'quit'
PfQuit
$cache = Get-Content "$script:Config\plugins-host\$($script:Fences.clipboard).view.json" -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
Log "clipboard history's last view: $cache"
Check 'the plugin thread ended' ([bool](PfWaitLog 'plugin thread ended' 5)) ''
if ($feedProc) { Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue }
