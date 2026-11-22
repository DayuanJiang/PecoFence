# 插件 interface 2, plan v6 row 7 (change 5): folders: "readwrite". The test plugin
# assets\folders-test runs the pf.folder calls the steps write into its data folder (cmd.json).
# Picking a folder to write in: the dialog says so, a junction, the home folder, PecoFence's own
# folder and a drive root are refused with a message, a plain folder is kept. Then write (a data
# file, replaced whole, no temporary file left), mkdir, list (at most 2,000 entries, then cut),
# trash (to the Recycle Bin; the picked folder itself, a folder with something in it, a program
# file and a junction stay); refused by the text (other kinds, a colon, a trailing dot, a
# device name, a dot folder, ..) and by the real path (a junction out of the folder, one into a
# dot folder). An update from read to readwrite drops the folders picked before. The sample
# assets\daily-log writes the lines typed into today's file and trashes it from its menu; the
# launcher's 添加文件夹… (pf.launch.pick("folder")) adds a folder. Last, trash never deletes for
# good: with C:'s Recycle Bin switched off (NukeOnDelete) or too small (MaxCapacity 1 MB) it is
# refused and the file stays (PecoFence restarted for each; the settings are put back). Debug
# build (CDP on the settings page), Chinese interface.
#   --var Part=B   the build before: the sample's fence and the details, the shots to compare
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = 'A' }
$Kit = 'C:\pf\kit\assets'
$Lr = 'C:\pf\run\folders'
$Notes = "$Lr\Notes"
$Feed = 'C:\pf\run\ffeed'

# Types $path into the folder picker's Folder: box (it has the keyboard) and presses Enter.
# Enter there opens the folder; then a click on Select Folder picks it.
function PickIn($dlg, $path) {
  [G]::SetForegroundWindow($dlg) | Out-Null; Start-Sleep -Milliseconds 300
  Chord 0x11 0x41; Start-Sleep -Milliseconds 80
  [G]::Type($path); Start-Sleep -Milliseconds 200
  [G]::Press(0x0D); Start-Sleep -Milliseconds 1200
  $b = @(([W]::Buttons($dlg) -split "`n") | Where-Object { $_ -match "`tSelect Folder`t" }) | Select-Object -First 1
  Log "folder picker: $path, button $b"
  if ($b) { $r = ($b -split "`t")[3] -split ','; [G]::Click([int](([int]$r[0] + [int]$r[2]) / 2), [int](([int]$r[1] + [int]$r[3]) / 2)) }
  Start-Sleep -Milliseconds 800
}
# The rows of a test plugin's fence as key -> result.
function Rows($fid = $script:Fid) {
  $map = @{}
  foreach ($l in "$(FenceOut $fid)" -split "`n") { $k, $v = $l -split ' ', 2; if ($k) { $map[$k] = $v } }
  return $map
}
function WaitRow($k, $fid = $script:Fid, $sec = 15) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $v = (Rows $fid)[$k]; if ("$v" -match '^(ok|error)') { return $v }; Start-Sleep -Milliseconds 500 }
  return (Rows $fid)[$k]
}
# One pf.folder call of a test plugin (`id`), and its row.
function Send($k, $op, $path = '', $text = '', $id = 'folders-test') {
  $o = [ordered]@{ k = $k; op = $op; path = $path; text = $text }
  [IO.File]::WriteAllText("$script:Config\plugins-data\$id\cmd.json", '[' + ($o | ConvertTo-Json -Compress) + ']')
}
function FCall($k, $op, $path = '', $text = '') { Send $k $op $path $text; return (WaitRow $k) }
# pf.folder.pick of a test plugin, the dialog's title, `path` picked, and what a message box
# after it said ('' when none came), its OK pressed.
function PickVia($k, $path, $id = 'folders-test', $fid = $script:Fid) {
  Send $k 'pick' '' '' $id
  $dlg = WaitWin '#32770' 10
  $title = if ($dlg -ne [IntPtr]::Zero) { [L]::Text($dlg) } else { '' }
  if ($dlg -ne [IntPtr]::Zero) { PickIn $dlg $path }
  $said = ''
  $mb = WaitWin '#32770' 2
  if ($mb -ne [IntPtr]::Zero -and $mb -ne $dlg) { $said = [W]::Statics($mb); Shot "said-$k"; [G]::Press(0x0D); WaitGoneWin '#32770' 3 | Out-Null }
  elseif ($mb -ne [IntPtr]::Zero) { [L]::PostMessageW($mb, 0x111, [IntPtr]2, [IntPtr]::Zero) | Out-Null; Log 'folder dialog still open: cancelled' }
  $row = WaitRow $k $fid 8
  return @($title, $said, $row)
}
# Markdown files in pf's Recycle Bin.
function BinMd() { return @(Get-ChildItem -Force (RecycleDir) -Filter '$R*.md' -ErrorAction SilentlyContinue).Count }
# Lines of the app's own log (the shareable copy has them too) matching the regex.
function MainCount($re) { return @(Get-Content (PfLogPath) -Encoding UTF8 | Where-Object { $_ -match $re }).Count }
function Cfg() { return (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json) }
function Picked($fid = $script:Fid) { return @((Cfg).scripts."$fid".settings.__folders | Where-Object { $_ }) }
# The sample's rows by the dump (the fence titled Log): HitPt as tests/plugin-ui.ps1 has it.
function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object {
    $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] }
  })
}
function HitPt($row) {
  $w = DumpOf 'Log' 5
  $r = @(CellRects $w | Where-Object { $_.i -eq $row * 100 }) | Select-Object -First 1
  if (-not $r) { Log "no hit $row"; return $null }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  return @([int]($w.l + [double](DumpVal $w 'content_x') + ($r.l + $r.r) / 2 * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + ($r.t + $r.b) / 2 * $sc))
}
function WaitOut($fid, $regex, $sec = 10) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((FenceOut $fid) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }

Case 'setup: folders and junctions, the plugins, their fences'
CloseWindows
New-Item -ItemType Directory -Force "$Notes\sub", "$Notes\.hidden", "$Lr\Other", "$Lr\Log", "$Lr\Box", $Feed | Out-Null
cmd /c "mklink /J `"$Lr\JNotes`" `"$Notes`" 2>&1" | Out-Null
cmd /c "mklink /J `"$Notes\out`" `"$Lr\Other`" 2>&1" | Out-Null
cmd /c "mklink /J `"$Notes\j`" `"$Notes\.hidden`" 2>&1" | Out-Null
cmd /c "mklink /J `"$Notes\inner`" `"$Notes\sub`" 2>&1" | Out-Null
Check 'four junctions made' ((@(Get-ChildItem $Lr, $Notes | Where-Object { $_.Attributes -match 'ReparsePoint' })).Count -eq 4) ''
$script:Config = "$PfApp\config"
foreach ($id in 'folders-test', 'folders-up') { New-Item -ItemType Directory -Force "$script:Config\plugins-data\$id" | Out-Null }
PfPlacePlugin "$Kit\daily-log" 'daily-log'
PfPlacePlugin "$Kit\launcher" 'launcher'
if ($Part -eq 'A') {
  PfPlacePlugin "$Kit\folders-test" 'folders-test'
  # folders-up: the same test plugin installed at 1.0.0 with folders: read.
  PfPlacePlugin "$Kit\folders-test" 'folders-up'
  $j = "$script:Config\plugins\folders-up\1.0.0\plugin.json"
  [IO.File]::WriteAllText($j, ([IO.File]::ReadAllText($j) -replace '"readwrite"', '"read"'))
}
$feedProc = PfFeed 8733 $Feed
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8733/index.json'
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
PfStart -Lang zh-CN -Script (DumpScript 500 20)
$script:Log = PfPluginFence 'Log' '720,120,420,360' 'daily-log'
if ($Part -eq 'A') {
  $script:Fid = PfPluginFence 'FT' '60,120,620,520' 'folders-test'
  $script:Up = PfPluginFence 'UP' '1180,120,360,200' 'folders-up'
  $script:Launch = PfPluginFence 'Launch' '1180,400,360,300' 'launcher'
  Check 'the plugins start' ([bool](PfWaitLog 'plugin fence starts its main.js' 20) -and (WaitOut $script:Log '点这里选一个文件夹')) (FenceOut $script:Log)
} else {
  Start-Sleep -Seconds 6
  ParkMouse; Shot 'before-log'
  PfSettings | Out-Null
  $sw = WaitWin 'PecoFence.Settings' 5
  [L]::MoveWindow($sw, 60, 60, 900, 760, $true) | Out-Null
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
  Shot 'before-list'
  Check 'the build before: the sample is not installed' ((FenceOut $script:Log) -notmatch '点这里选一个文件夹') (FenceOut $script:Log)
  Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue
  return
}

Case 'a folder to write in: a junction, the home folder, PecoFence''s own folder and a drive root are refused'
$r = PickVia 'p1-junction' "$Lr\JNotes"
Check 'the dialog says the plugin may read and write' ($r[0] -eq '选择插件可以读写的文件夹') "title=$($r[0])"
Check 'a junction: a message says why, the plugin gets null' ($r[1] -match '插件不能写入这个文件夹' -and $r[1] -match '目录连接点' -and $r[2] -eq 'ok null') "said=$($r[1]) row=$($r[2])"
$r = PickVia 'p2-home' $env:USERPROFILE
Check 'the home folder itself: refused' ($r[1] -match '插件不能写入这个文件夹' -and $r[2] -eq 'ok null') "said=$($r[1]) row=$($r[2])"
$r = PickVia 'p3-own' "$script:Config"
Check 'PecoFence''s own folder: refused' ($r[1] -match '插件不能写入这个文件夹' -and $r[2] -eq 'ok null') "said=$($r[1]) row=$($r[2])"
$r = PickVia 'p4-root' 'C:\'
Check 'a drive root: refused' ($r[1] -match '插件不能写入这个文件夹' -and $r[2] -eq 'ok null') "said=$($r[1]) row=$($r[2])"
$why = 'plugin folder to write in refused.*why="'
Check 'nothing was kept, the log says why each time' (@(Picked).Count -eq 0 -and (MainCount $why) -eq 4) "picked=$(@(Picked) -join ',') refused=$(MainCount $why)"
$r = PickVia 'p5-notes' $Notes
Check 'a plain folder: no message, the plugin gets its path, the fence keeps it' ($r[1] -eq '' -and $r[2] -eq ('ok ' + (ConvertTo-Json $Notes)) -and @(Picked) -contains $Notes) "said=$($r[1]) row=$($r[2]) picked=$(@(Picked) -join ',')"

Case 'write, replace, mkdir, list, trash inside the picked folder'
$v = FCall 'w1-mkdir' 'mkdir' "$Notes\2026"
Check 'mkdir makes the folder' ($v -eq 'ok null' -and (Test-Path "$Notes\2026" -PathType Container)) $v
$v = FCall 'w2-mkdir-again' 'mkdir' "$Notes\2026"
Check 'mkdir of a folder there already: fine' ($v -eq 'ok null') $v
$v = FCall 'w3-write' 'write' "$Notes\2026\a.md" 'hello 你好'
Check 'write: the file has the text in UTF-8' ($v -eq 'ok null' -and [IO.File]::ReadAllText("$Notes\2026\a.md") -eq 'hello 你好') "$v"
$v = FCall 'w4-replace' 'write' "$Notes\2026\a.md" 'second'
$left = @(Get-ChildItem -Force "$Notes\2026" | ForEach-Object { $_.Name })
Check 'a second write replaces it whole; no temporary file is left' ($v -eq 'ok null' -and [IO.File]::ReadAllText("$Notes\2026\a.md") -eq 'second' -and $left.Count -eq 1) "$v files=$($left -join ',')"
$v = FCall 'w5-list' 'list' "$Notes\2026"
Check 'list names it' ($v -eq 'ok ["a.md"]') $v
New-Item -ItemType Directory -Force "$Notes\many" | Out-Null
foreach ($i in 1..2005) { [IO.File]::WriteAllText("$Notes\many\f$i.txt", '') }
$v = FCall 'w5b-list-many' 'list' "$Notes\many"
Check 'a folder of 2,005 files: the list has 2,000 and says it was cut' ($v -eq 'ok "cut after 2000"') $v
$v = FCall 'w6-read' 'read' "$Notes\2026\a.md"
Check 'read gives the text' ($v -eq 'ok "second"') $v
$bin = BinMd
$v = FCall 'w7-trash' 'trash' "$Notes\2026\a.md"
Check 'trash: the file goes to the Recycle Bin' ($v -eq 'ok true' -and -not (Test-Path "$Notes\2026\a.md") -and (BinMd) -eq $bin + 1) "$v bin=$bin then $(BinMd)"
$v = FCall 'w8-trash-folder' 'trash' "$Notes\2026"
Check 'trash of a folder in it: the folder goes too' ($v -eq 'ok true' -and -not (Test-Path "$Notes\2026")) $v
$v = FCall 'w9-trash-picked' 'trash' $Notes
Check 'the picked folder itself stays' ($v -match '^error folder.trash: the picked folder itself stays' -and (Test-Path $Notes)) $v
New-Item -ItemType Directory -Force "$Notes\full" | Out-Null
[IO.File]::WriteAllText("$Notes\full\keep.md", 'keep')
$v = FCall 'wa-trash-full' 'trash' "$Notes\full"
Check 'a folder with something in it stays (only empty folders go)' ($v -match '^error folder.trash: only an empty folder' -and (Test-Path "$Notes\full\keep.md")) $v
[IO.File]::WriteAllText("$Notes\tool.exe", 'x')
$v = FCall 'wb-trash-exe' 'trash' "$Notes\tool.exe"
Check 'a program file stays (only data files go)' ($v -match '^error folder.trash: only these kinds of files' -and (Test-Path "$Notes\tool.exe")) $v
Remove-Item "$Notes\tool.exe"
$v = FCall 'wc-trash-junction' 'trash' "$Notes\inner"
Check 'a junction stays, and so does the folder it leads to' ($v -match '^error folder.trash: a link or junction' -and (Test-Path "$Notes\inner") -and (Test-Path "$Notes\sub")) $v
$n = @('plugin wrote a file into a picked folder', 'plugin made a folder in a picked folder', 'plugin moved a file or folder of a picked folder to the Recycle Bin') | ForEach-Object { MainCount $_ }
Check 'the log has each write, folder and trash' (($n -join ',') -eq '2,2,2') "counts=$($n -join ',')"

Case 'refused by the path''s text'
$bad = [ordered]@{
  'x1-exe' = @("$Notes\a.exe", 'permissions.folders any'); 'x2-hta' = @("$Notes\a.hta", 'permissions.folders any')
  'x3-lnk' = @("$Notes\a.lnk", 'permissions.folders any'); 'x4-stream' = @("$Notes\a.txt:hidden", 'colon')
  'x5-dot-end' = @("$Notes\a.txt.", 'ends in a dot or a space'); 'x6-space-end' = @("$Notes\a.txt ", 'ends in a dot or a space')
  'x7-device' = @("$Notes\nul.txt", 'device name'); 'x8-dot-folder' = @("$Notes\.hidden\a.txt", 'starts with a dot')
  'x9-up' = @("$Notes\..\Other\a.txt", 'starts with a dot'); 'xa-outside' = @("$Lr\Other\a.txt", 'not inside a folder the user picked')
}
$wrong = @()
foreach ($k in $bad.Keys) { $v = FCall $k 'write' $bad[$k][0] 'x'; if ($v -notmatch "^error folder.write[: ].*$([regex]::Escape($bad[$k][1]))") { $wrong += "$k=$v" } }
Check 'other kinds, a colon, a trailing dot or space, a device, a dot folder, .., outside: each refused with its reason' ($wrong.Count -eq 0) ($wrong -join ' | ')
$v = FCall 'xb-mkdir-dot' 'mkdir' "$Notes\.git"
Check 'mkdir of a dot folder is refused too' ($v -match '^error folder.mkdir: .*starts with a dot' -and -not (Test-Path "$Notes\.git")) $v
$made = @(Get-ChildItem -Force $Notes, "$Lr\Other" | Where-Object { -not $_.PSIsContainer }).Count
Check 'no file was made by any of them' ($made -eq 0 -and -not (Test-Path "$Notes\a.txt")) "files=$made"

Case 'links: the real path is checked'
$v = FCall 'l1-out' 'write' "$Notes\out\pf-folders.txt" 'x'
Check 'a junction out of the folder: refused, nothing written where it leads' ($v -match '^error folder.write: .*not inside a folder the user picked' -and -not (Test-Path "$Lr\Other\pf-folders.txt")) $v
$v = FCall 'l2-dot' 'write' "$Notes\j\a.txt" 'x'
Check 'a junction into a dot folder: refused by its real path' ($v -match '^error folder.write: .*starts with a dot' -and -not (Test-Path "$Notes\.hidden\a.txt")) $v
$v = FCall 'l3-inner' 'write' "$Notes\inner\b.txt" 'inner'
Check 'a junction to a folder inside the picked one: written there' ($v -eq 'ok null' -and [IO.File]::ReadAllText("$Notes\sub\b.txt") -eq 'inner') $v
Shot 'test-plugin'

Case 'an update from read to readwrite: the folders picked before go'
$r = PickVia 'u1-pick' "$Lr\Other" 'folders-up' $script:Up
Check 'folders: read: the dialog says read, the folder is kept' ($r[0] -eq '选择插件可以读取的文件夹' -and @(Picked $script:Up) -contains "$Lr\Other") "title=$($r[0]) row=$($r[2])"
Send 'u2-write' 'write' "$Lr\Other\a.txt" 'x' 'folders-up'
$v = WaitRow 'u2-write' $script:Up
Check 'read only: a write is refused by the manifest' ($v -match 'permissions.folders readwrite') $v
# The index has folders-up 1.1.0 with folders: readwrite: an update that asks first.
$commit = 'c' * 40
$dir = "$Feed\$commit\plugins\folders-up"; New-Item -ItemType Directory -Force $dir | Out-Null
$m = Get-Content "$Kit\folders-test\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
$m.id = 'folders-up'; $m.version = '1.1.0'
[IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 6))
Copy-Item "$Kit\folders-test\main.js", "$Kit\folders-test\README.md" $dir
$m | Add-Member -NotePropertyName files -NotePropertyValue @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
[IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = 1; commit = $commit; plugins = @($m) } | ConvertTo-Json -Depth 8))
PfSettings | Out-Null
$sw = WaitWin 'PecoFence.Settings' 5
[L]::MoveWindow($sw, 60, 60, 900, 760, $true) | Out-Null
Js "(() => { window.pfRows = {}; window.chrome.webview.addEventListener('message', e => { if (e.data && e.data.type === 'state' && e.data.plugins) window.pfRows.v = e.data.plugins; }); return true; })()" | Out-Null
Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null
$state = ''
for ($i = 0; $i -lt 20 -and $state -ne 'confirm'; $i++) { Start-Sleep -Milliseconds 500; $state = Js "(() => { const l = window.pfRows.v && window.pfRows.v.list.find(x => x.id === 'folders-up'); return l ? l.state : ''; })()" }
Check 'the update waits for the user (folders grew)' ($state -eq 'confirm') "state=$state"
Js "(() => { const l = window.pfRows.v.list.find(x => x.id === 'folders-up'); window.chrome.webview.postMessage({ type: 'action', name: 'pluginInstall', id: 'folders-up', shown: l.shown }); return true; })()" | Out-Null
$line = PfWaitLog 'folders read became readwrite, the folders picked before are dropped' 20
Start-Sleep -Seconds 3
Check 'after the update: 1.1.0, the folder picked for reading is gone' ([bool]$line -and (Get-Content "$script:Config\plugins\folders-up\current.txt") -eq '1.1.0' -and @(Picked $script:Up).Count -eq 0) "picked=$(@(Picked $script:Up) -join ',')"
Send 'u3-write' 'write' "$Lr\Other\a.txt" 'x' 'folders-up'
$v = WaitRow 'u3-write' $script:Up
Check 'its writes there are refused until the user picks again' ($v -match 'not inside a folder the user picked' -and -not (Test-Path "$Lr\Other\a.txt")) $v

Case 'the details say it reads and writes the folder you pick'
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
$i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('每日记录'))")
PageClick '#pluginList .pl-row' $i; Start-Sleep -Seconds 1
$d = Js "(() => { const d = document.querySelector('#pluginList .pl-detail'); return d ? d.innerText : ''; })()"
Shot 'details'
Check 'the details: 读写你选的文件夹（装好后由你来选）, without the read-only line' ($d -match '读写你选的文件夹（装好后由你来选）' -and $d -notmatch '只读不写') $d
[L]::PostMessageW($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; WaitGoneWin 'PecoFence.Settings' 5 | Out-Null

Case 'the daily log sample: a picked folder, two lines, today''s file to the Recycle Bin'
$p = HitPt 0
[G]::Click($p[0], $p[1])
$dlg = WaitWin '#32770' 10
Check 'its first row opens the dialog to pick a folder to write in' ($dlg -ne [IntPtr]::Zero -and [L]::Text($dlg) -eq '选择插件可以读写的文件夹') "dlg=$dlg"
PickIn $dlg "$Lr\Log"
Check 'the fence shows the folder and the box' ((WaitOut $script:Log '^Log') -and (WaitOut $script:Log '今天还没有记录')) (FenceOut $script:Log)
$today = Get-Date
$file = "$Lr\Log\$($today.ToString('yyyy-MM'))\$($today.ToString('yyyy-MM-dd')).md"
foreach ($line in 'first entry', 'second entry') {
  $p = HitPt 1
  [G]::Click($p[0], $p[1])
  $box = WaitWin 'PecoFence.Rename' 5
  [G]::Type($line); Start-Sleep -Milliseconds 300
  Chord 0x11 0x0D; Start-Sleep -Milliseconds 1500
}
$text = if (Test-Path $file) { [IO.File]::ReadAllText($file) } else { '' }
Check 'Ctrl+Enter wrote each line, with the time, into today''s file in a month folder' ($text -match "(?m)^- \d.* first entry$" -and $text -match "(?m)^- \d.* second entry$" -and $text -match '^# ') "file=$file text=$text"
Check 'the fence shows both lines' ((WaitOut $script:Log 'first entry') -and (FenceOut $script:Log) -match 'second entry') (FenceOut $script:Log)
ParkMouse; Shot 'after-log'
$r = (FenceGet $script:Log).windowRect
[G]::RClick($r.x + 200, $r.y + $r.h - 30)
$m = MenuWin 5
$texts = MenuTexts $m
Check 'its menu has 选择文件夹… and 把今天的记录移到回收站' ($texts -match '选择文件夹… \| 把今天的记录移到回收站') $texts
$bin = BinMd
MenuPick '^把今天的记录移到回收站' $m | Out-Null
Start-Sleep -Seconds 2
Check 'today''s file is in the Recycle Bin, the fence says nothing is logged' (-not (Test-Path $file) -and (BinMd) -eq $bin + 1 -and (WaitOut $script:Log '今天还没有记录')) "bin=$bin then $(BinMd)"

Case 'the launcher: 添加文件夹… adds the folder picked'
$r = (FenceGet $script:Launch).windowRect
[G]::RClick($r.x + 120, $r.y + $r.h - 40)
$m = MenuWin 5
Check 'the launcher''s menu has 添加文件夹…' ((MenuTexts $m) -match '添加文件… \| 添加文件夹…') (MenuTexts $m)
MenuPick '^添加文件夹' $m | Out-Null
$dlg = WaitWin '#32770' 10
Check 'a folder dialog titled 添加文件夹' ($dlg -ne [IntPtr]::Zero -and [L]::Text($dlg) -eq '添加文件夹') "title=$([L]::Text($dlg))"
if ($dlg -ne [IntPtr]::Zero) { PickIn $dlg "$Lr\Box" }
$items = @()
for ($i = 0; $i -lt 20 -and $items.Count -eq 0; $i++) { Start-Sleep -Milliseconds 500; $items = @((Cfg).scripts."$script:Launch".settings.__launch | Where-Object { $_ }) }
Check 'the folder is an item; the plugin has its name only' ((WaitOut $script:Launch 'Box') -and $items.Count -eq 1 -and $items[0].path -eq "$Lr\Box" -and (FenceOut $script:Launch) -notmatch 'C:\\') "items=$($items.Count) $(FenceOut $script:Launch)"

# C:'s Recycle Bin settings of pf (one key per volume), switched for a case and put back.
$BinKeys = @(Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\BitBucket\Volume' -ErrorAction SilentlyContinue)
$BinSaved = @{}
foreach ($k in $BinKeys) { $p = Get-ItemProperty $k.PSPath; $BinSaved[$k.PSPath] = @($p.NukeOnDelete, $p.MaxCapacity) }
Log "recycle bin keys: $(($BinKeys | ForEach-Object { $_.PSChildName }) -join ',') saved $(($BinSaved.Values | ForEach-Object { $_ -join '/' }) -join ',')"
function BinSet($nuke, $max) { foreach ($k in $BinKeys) { Set-ItemProperty $k.PSPath NukeOnDelete $nuke -Type DWord; if ($max) { Set-ItemProperty $k.PSPath MaxCapacity $max -Type DWord } } }
function BinBack() {
  foreach ($k in $BinKeys) {
    $o = $BinSaved[$k.PSPath]
    foreach ($pair in @(@('NukeOnDelete', $o[0]), @('MaxCapacity', $o[1]))) {
      if ($null -eq $pair[1]) { Remove-ItemProperty $k.PSPath $pair[0] -ErrorAction SilentlyContinue } else { Set-ItemProperty $k.PSPath $pair[0] $pair[1] -Type DWord }
    }
  }
  Log 'recycle bin settings put back'
}
# PecoFence started again (the shell reads these settings once per process), then a trash of
# `file` that Windows would delete for good.
function NukeCase($k, $what, $file) {
  trap { PfTrap $_; continue }
  # The test plugin runs cmd.json again as it starts: nothing.
  [IO.File]::WriteAllText("$script:Config\plugins-data\folders-test\cmd.json", '[]')
  PfStart -Lang zh-CN -Script (DumpScript 500 20)
  PfWaitLog 'plugin fence starts its main.js' 20 | Out-Null
  Start-Sleep -Seconds 2
  $bin = @(Get-ChildItem -Force (RecycleDir) -Filter '$R*' -ErrorAction SilentlyContinue).Count
  $v = FCall $k 'trash' $file
  $after = @(Get-ChildItem -Force (RecycleDir) -Filter '$R*' -ErrorAction SilentlyContinue).Count
  Check "$($what): refused with the reason, the file stays, nothing went to the bin" ($v -match '^error folder.trash: .*cannot go to the Recycle Bin' -and (Test-Path $file) -and $after -eq $bin) "$v bin=$bin then $after"
}

Case 'never deleted for good: what cannot go to the Recycle Bin stays where it is'
Check 'C: has its Recycle Bin settings key' ($BinKeys.Count -ge 1) "keys=$($BinKeys.Count)"
[IO.File]::WriteAllText("$Notes\nuke.md", 'keep me')
BinSet 1 0
NukeCase 'n1-off' 'the Recycle Bin of C: switched off' "$Notes\nuke.md"
$big = [IO.File]::Create("$Notes\big.log"); $big.SetLength(3MB); $big.Close()
BinSet 0 1
NukeCase 'n2-small' 'a 3 MB file and a Recycle Bin of 1 MB' "$Notes\big.log"
BinBack
Check 'the Recycle Bin settings are back' (@($BinKeys | Where-Object { $o = $BinSaved[$_.PSPath]; $p = Get-ItemProperty $_.PSPath; "$($p.NukeOnDelete)/$($p.MaxCapacity)" -ne "$($o[0])/$($o[1])" }).Count -eq 0) ''

Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue
cmd /c "rmdir `"$Lr\JNotes`" `"$Notes\out`" `"$Notes\j`" `"$Notes\inner`" 2>&1" | Out-Null
