# 插件的轻量层: every pf function of a light plugin on the desktop, with the test plugin in
# assets/plugin-api (installed as an install leaves it: plugins\plugin-api\1.0.0 and current.txt).
# The plugin shows one row per check ("<name> <result>"); `fence get` reads them. Requests go to
# api.github.com (declared) and example.com (not declared); the tarball address of the repository
# redirects to codeload.github.com (not declared). Either build.
$Kit = 'C:\pf\kit\assets\plugin-api'
$script:Fid = $null
function Rows() {
  $out = (FenceGet $script:Fid).script.output
  $map = @{}
  foreach ($l in "$out" -split "`n") { $k, $v = $l -split ' ', 2; if ($k) { $map[$k] = $v } }
  return $map
}
function WaitRow($key, $regex, $sec = 20) {
  for ($i = 0; $i -lt $sec * 2; $i++) { $v = (Rows)[$key]; if ("$v" -match $regex) { return $v }; Start-Sleep -Milliseconds 500 }
  return (Rows)[$key]
}
function Command($cmd) { [IO.File]::WriteAllText("$script:Data\cmd.txt", $cmd) }
function Output() { return "$((FenceGet $script:Fid).script.output)" }

Case 'setup: the test plugin in a plugin fence'
CloseWindows
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
$script:Data = "$script:Config\plugins-data\plugin-api"
New-Item -ItemType Directory -Force "$script:Config\plugins\plugin-api\1.0.0", $script:Data | Out-Null
Copy-Item "$Kit\*" "$script:Config\plugins\plugin-api\1.0.0" -Force
Set-Content "$script:Config\plugins\plugin-api\current.txt" '1.0.0' -Encoding ASCII -NoNewline
Set-Content 'C:\pf\run\idle.cmd' '@echo off' -Encoding ASCII
$f = PfJson fence create --title 'API' --script 'C:\pf\run\idle.cmd' --rect '420,120,560,620'
$script:Fid = if ($f.fence) { $f.fence.id } else { $f.id }
Pf fence set $script:Fid scriptPath 'plugin:plugin-api' | Out-Null
$line = PfWaitLog 'plugin fence starts its main.js' 20
Check 'the plugin starts' ([bool]$line) "$line"
$v = WaitRow 'sys' '^ok'
Shot 'started'

Case 'pf.http: a declared host, an undeclared one, a redirect to an undeclared one'
$v = WaitRow 'http-declared' '^(ok|error)'
Check 'api.github.com (declared) answers 200' ($v -eq 'ok 200') $v
$v = (Rows)['http-undeclared']
Check 'example.com (not declared) is refused at once, naming permissions.network' ($v -match '^refused http.get needs .*example.com in permissions.network') $v
$v = WaitRow 'http-redirect' '^(ok|error)'
Check 'the redirect to codeload.github.com is refused' ($v -match '^error redirect refused: .*codeload.github.com in permissions.network') $v
Check 'the log names the refused redirect' ([bool](PfWaitLog 'plugin request: redirect refused' 2)) ''
Check 'the request carried the plugin''s User-Agent (log of the answer)' ([bool](PfWaitLog 'plugin request answered.*api.github.com.*status=200' 2)) ''

Case 'pf.data: names and a round trip'
Check 'a name with a slash is refused' ((Rows)['data-name-slash'] -match '^refused data.write needs a name') (Rows)['data-name-slash']
Check 'a name starting with a dot is refused' ((Rows)['data-name-dot'] -match '^refused data.write needs a name') (Rows)['data-name-dot']
$v = WaitRow 'data-roundtrip' '^(ok|error)'
Check 'write, read, list, remove' ($v -eq 'ok ["hello 你好",true,null]') $v
Check 'nothing was written outside the data folder' (-not (Test-Path "$script:Config\plugins-data\x.txt") -and -not (Test-Path "$script:Data\.hidden")) ''

Case 'pf.sys, pf.time, pf.open, pf.openSystem, pf.folder, pf.notify'
$v = (Rows)['sys']
Check 'cpu, memory, drives (C:), ip, network are there; no battery on the box' ($v -match '"cpu":"number"' -and $v -match '"memory":"number"' -and $v -match '"c":true' -and $v -match '"ip":"string"' -and $v -match '"network":[1-9]' -and $v -match '"battery":null') $v
Check 'pf.time.now and format' ((Rows)['time'] -eq 'ok ["number",true]') (Rows)['time']
Check 'pf.open outside a click is refused' ((Rows)['open-outside-click'] -match '^refused pf.open works only in a click') (Rows)['open-outside-click']
Check 'pf.openSystem of another name is refused' ((Rows)['open-system-other'] -match '^refused openSystem needs the name taskmgr') (Rows)['open-system-other']
Check 'pf.folder.pick without the folders permission is refused' ((Rows)['folder-pick'] -match '^refused folder.pick needs permissions.folders') (Rows)['folder-pick']
$card = WaitWin 'PecoFence.Notice' 10
Check 'pf.notify shows the corner card' ($card -ne [IntPtr]::Zero -and (WaitRow 'notify' '^ok') -eq 'ok null') "card=$card row=$((Rows)['notify'])"
Shot 'notice'
if ($card -ne [IntPtr]::Zero) { $r = Rect $card; [G]::Click($r.R - 60, $r.B - 30); Start-Sleep -Milliseconds 800 }

Case 'pf.run: refused with the system switch off, works with it on'
Check 'undeclared program refused' ((Rows)['run-undeclared'] -match '^refused run needs cmd.exe in permissions.system') (Rows)['run-undeclared']
$v = WaitRow 'run-switch-off' '^(ok|error)'
Check 'declared ipconfig, switch off: rejected naming the switch' ($v -match '^error pf.run needs 允许插件使用 Windows 底层功能') $v
Pf settings set tools.allowSystem true | Out-Null
Command 'run'
$v = WaitRow 'cmd-run' '^(ok|error)' 25
Check 'switch on: ipconfig runs and its output comes back' ($v -eq 'ok [0,true]') $v
Check 'the log says the program ended' ([bool](PfWaitLog 'pf.run: the program ended.*program="ipconfig" code=Some\(0\)' 2)) ''

Case 'pf.clipboard: text, a private copy, files, a picture there and back, setText'
function LastClip() { return (Rows)["clipboard-$((Rows)['clipboard-count'])"] }
Set-Clipboard -Value 'pf clipboard test 1'
Start-Sleep -Seconds 2
Check 'a copied text: one clipboard event, and text() reads it' ((Rows)['clipboard-count'] -eq '1' -and (LastClip) -eq '{"text":{"text":"pf clipboard test 1","truncated":false},"files":null}') "count=$((Rows)['clipboard-count']) $(LastClip)"
function PrivateCopy($marker, [byte[]]$value) {
  $d = New-Object System.Windows.Forms.DataObject
  $d.SetData([System.Windows.Forms.DataFormats]::UnicodeText, 'secret password 123')
  $d.SetData($marker, (New-Object IO.MemoryStream(, $value)))
  [System.Windows.Forms.Clipboard]::SetDataObject($d, $true)
}
$before = (Rows)['clipboard-count']
PrivateCopy 'ExcludeClipboardContentFromMonitorProcessing' ([byte[]](1, 0, 0, 0))
Start-Sleep -Seconds 2
PrivateCopy 'CanIncludeInClipboardHistory' ([byte[]](0, 0, 0, 0))
Start-Sleep -Seconds 2
Check 'private copies (two kinds of marker): no clipboard event' ((Rows)['clipboard-count'] -eq $before -and (Output) -notmatch 'secret') "count=$((Rows)['clipboard-count']) before=$before"
Command 'read-text'
$v = WaitRow 'cmd-read-text' '^(ok|error)'
Check 'a private copy: text() reads nothing' ($v -eq 'ok null') $v
Check 'the log says the change was private' ([bool](PfWaitLog 'clipboard changed, marked private' 2)) ''
Set-Content 'C:\pf\run\clipfile.txt' 'x'
Set-Clipboard -Path 'C:\pf\run\clipfile.txt'
Start-Sleep -Seconds 2
$v = LastClip
Check 'copied files: files() lists the path' ($v -match '"files":\["C:\\\\pf\\\\run\\\\clipfile.txt"\]') $v
$bmp = New-Object System.Drawing.Bitmap 64, 48
$g = [System.Drawing.Graphics]::FromImage($bmp); $g.Clear([System.Drawing.Color]::FromArgb(255, 200, 30, 60)); $g.Dispose()
[System.Windows.Forms.Clipboard]::SetImage($bmp)
$v = WaitRow 'image' '^(ok|error)'
$png = if (Test-Path "$script:Data\picture.png") { [IO.File]::ReadAllBytes("$script:Data\picture.png") } else { @(0, 0, 0, 0) }
Check 'image() saved a PNG in the data folder' ($v -eq 'ok "picture.png"' -and $png[1] -eq 0x50 -and $png[2] -eq 0x4E -and $png[3] -eq 0x47) "$v bytes=$($png.Length)"
Set-Clipboard -Value 'something else'
Start-Sleep -Seconds 1
Command 'set-image'
$v = WaitRow 'cmd-set-image' '^(ok|error)'
Start-Sleep -Milliseconds 500
$back = [System.Windows.Forms.Clipboard]::GetImage()
$px = if ($back) { $back.GetPixel(10, 10) } else { $null }
Check 'setImage() puts the saved picture back on the clipboard' ($v -eq 'ok null' -and $back -and $back.Width -eq 64 -and $back.Height -eq 48 -and $px.R -eq 200 -and $px.G -eq 30 -and $px.B -eq 60) "$v size=$($back.Width)x$($back.Height) px=$px"
Command 'set-text'
$v = WaitRow 'cmd-set-text' '^(ok|error)'
Start-Sleep -Milliseconds 500
Check 'setText() puts text on the clipboard' ($v -eq 'ok null' -and (Get-Clipboard -Raw) -eq 'from the plugin 插件') "$v clip=$(Get-Clipboard -Raw)"

Case 'limits: 200 ms a second, then two endless loops: restarts after 5 s, the third stop waits for Restart'
Command 'busy'
$busy = $false
for ($i = 0; $i -lt 20 -and -not $busy; $i++) { Start-Sleep -Milliseconds 500; $busy = (Output) -match 'longer than 200 ms within one second' }
Check 'too much time in a second: the fence says so and that it starts again in 5 s' ($busy -and (Output) -match 'starts again by itself in 5 seconds') (Output)
$again = PfWaitLog 'plugin starts again by itself' 12
Check 'it starts again by itself' ([bool]$again) "$again"
Start-Sleep -Seconds 3
Command 'hang'
$final = $false
for ($i = 0; $i -lt 60 -and -not $final; $i++) { Start-Sleep -Milliseconds 500; $final = (Output) -match 'three times within an hour' }
Check 'the third stop within an hour: it stays stopped' $final (Output)
Check 'it stopped for the 50 ms limit' ((Output) -match 'one run took longer than 50 ms') (Output)
Shot 'stopped-three-times'
Start-Sleep -Seconds 7
Check 'and does not start again by itself' ((Output) -match 'three times within an hour') (Output)
Remove-Item "$script:Data\cmd.txt"
$r = (FenceGet $script:Fid).windowRect
$clicked = OcrClick '^Restart$' $r.x $r.y $r.w $r.h
Start-Sleep -Seconds 3
Check 'Restart in the fence starts it again' ($clicked -and (WaitRow 'sys' '^ok' 10) -match '^ok') "clicked=$clicked out=$(Output)"

Case 'the last view is cached; quit stops the plugin thread'
PfQuit
$cache = "$script:Config\plugins-host\$($script:Fid).view.json"
$text = if (Test-Path $cache) { Get-Content $cache -Raw -Encoding UTF8 } else { '' }
Check 'the view cache file has the last rows' ($text -match '"rows"' -and $text -match 'http-declared') $cache
Check 'the log says every plugin stopped and the thread ended' ([bool](PfWaitLog 'plugin thread: every plugin stops' 5) -and [bool](PfWaitLog 'plugin thread ended' 5)) ''
Set-Clipboard -Value ' '
