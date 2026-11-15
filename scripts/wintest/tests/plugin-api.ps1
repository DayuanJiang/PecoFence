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

Case 'setup: the test plugin in a plugin fence'
CloseWindows
# Placed before the start: PecoFence reads the installed plugins once as it starts.
$script:Config = "$PfApp\config"
$script:Data = "$script:Config\plugins-data\plugin-api"
New-Item -ItemType Directory -Force $script:Data | Out-Null
PfPlacePlugin $Kit 'plugin-api'
PfStart -Lang en
Check 'the portable configuration is where the plugin went' ((Split-Path (PfJson status).configPath) -eq $script:Config) (PfJson status).configPath
$script:Fid = PfPluginFence 'API' '420,120,560,620' 'plugin-api'
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

Case 'pf.http: a declared host that resolves to this computer is refused before it connects'
# localtest.me is public DNS for 127.0.0.1 and ::1: what a rebinding or a plugin's own DNS does.
$tl = [Net.Sockets.TcpListener]::new([Net.IPAddress]::IPv6Any, 18767); $tl.Server.DualMode = $true; $tl.Start()
Command 'private'
$v = WaitRow 'cmd-private' '^(ok|error)' 8
$hit = $tl.Pending(); $tl.Stop()
Check 'localtest.me (declared) is refused, naming the address' ($v -match '^error localtest.me resolves to (127\.0\.0\.1|::1), an address of this computer') $v
Check 'and nothing connected to the port it named' (-not $hit) "connected=$hit"
Check 'the log says why' ([bool](PfWaitLog 'plugin request refused.*the host resolves to this computer or the local network' 2)) ''

Case 'pf.data: names and a round trip'
Check 'a name with a slash is refused' ((Rows)['data-name-slash'] -match '^refused data.write needs a name') (Rows)['data-name-slash']
Check 'a name starting with a dot is refused' ((Rows)['data-name-dot'] -match '^refused data.write needs a name') (Rows)['data-name-dot']
$v = WaitRow 'data-roundtrip' '^(ok|error)'
Check 'write, read, list, remove' ($v -eq 'ok ["hello 你好",true,null]') $v
Check 'nothing was written outside the data folder' (-not (Test-Path "$script:Config\plugins-data\x.txt") -and -not (Test-Path "$script:Data\.hidden")) ''
Check 'a Windows device name (nul.txt) is refused' ((Rows)['data-name-device'] -match '^refused data.write needs a name') (Rows)['data-name-device']
Check 'a name ending with a dot is refused' ((Rows)['data-name-dot-end'] -match '^refused data.write needs a name') (Rows)['data-name-dot-end']

Case 'addresses a browser could read with another host'
Check 'a NUL and a user name before api.github.com: refused' ((Rows)['http-userinfo'] -match '^refused http.get needs .*permissions.network') (Rows)['http-userinfo']
Check 'a backslash and a user name before github.com: pf.open refused' ((Rows)['open-backslash'] -match '^refused open needs .*permissions.network or links') (Rows)['open-backslash']

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
Check 'undeclared program refused' ((Rows)['run-undeclared'] -match '^refused run needs whoami.exe in permissions.system') (Rows)['run-undeclared']
$v = WaitRow 'run-switch-off' '^(ok|error)'
Check 'declared ipconfig, switch off: rejected naming the switch' ($v -match '^error pf.run needs 允许插件使用 Windows 底层功能') $v
Pf settings set tools.allowSystem true | Out-Null
Command 'run'
$v = WaitRow 'cmd-run' '^(ok|error)' 25
Check 'switch on: ipconfig runs and its output comes back' ($v -eq 'ok [0,true]') $v
Check 'the log says the program ended' ([bool](PfWaitLog 'pf.run: the program ended.*program="ipconfig\.exe" code=Some\(0\)' 2)) ''
$pings = @(Get-Process ping -ErrorAction SilentlyContinue).Count
Command 'run-child'
$v = WaitRow 'cmd-run-child' '^(ok|error)' 45
Check 'a program that leaves another running: the answer comes at once' ($v -eq 'ok [0,true]') $v
Start-Sleep -Seconds 1
Check 'and what it left running was ended with it' (@(Get-Process ping -ErrorAction SilentlyContinue).Count -eq $pings) "ping before=$pings now=$(@(Get-Process ping -ErrorAction SilentlyContinue).Count)"

Case 'a data file over 20 MB, an error in a handler, a row with fields of the wrong type'
[IO.File]::WriteAllBytes("$script:Data\big.txt", (New-Object byte[] (21MB)))
Command 'read-big'
$v = WaitRow 'cmd-read-big' '^(ok|error)'
Check 'pf.data.read of a 21 MB file is rejected' ($v -match '^error .*larger than 20 MB') $v
Remove-Item "$script:Data\big.txt"
Command 'throw'
$line = PfWaitLog 'plugin error: it escaped a handler' 10
Check 'an error in a handler is logged with its message' ("$line" -match 'thrown by the throw command') "$line"
Command 'bad-row'
$v = WaitRow 'bad-row' '^shown'
Check 'the plugin goes on, and a row with an id of the wrong type still shows' ($v -eq 'shown' -and (FenceOut $script:Fid) -notmatch 'stopped') "$v $(FenceOut $script:Fid)"
Check 'the log names the field left out' ([bool](PfWaitLog 'a field of the wrong type is left out.*field=.?id' 5)) ''

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
Check 'private copies (two kinds of marker): no clipboard event' ((Rows)['clipboard-count'] -eq $before -and (FenceOut $script:Fid) -notmatch 'secret') "count=$((Rows)['clipboard-count']) before=$before"
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
for ($i = 0; $i -lt 20 -and -not $busy; $i++) { Start-Sleep -Milliseconds 500; $busy = (FenceOut $script:Fid) -match 'longer than 200 ms within one second' }
Check 'too much time in a second: the fence says so and that it starts again in 5 s' ($busy -and (FenceOut $script:Fid) -match 'starts again by itself in 5 seconds') (FenceOut $script:Fid)
$again = PfWaitLog 'plugin starts again by itself' 12
Check 'it starts again by itself' ([bool]$again) "$again"
Start-Sleep -Seconds 3
Command 'hang'
$final = $false
for ($i = 0; $i -lt 60 -and -not $final; $i++) { Start-Sleep -Milliseconds 500; $final = (FenceOut $script:Fid) -match 'three times within an hour' }
Check 'the third stop within an hour: it stays stopped' $final (FenceOut $script:Fid)
Check 'it stopped for the 50 ms limit' ((FenceOut $script:Fid) -match 'one run took longer than 50 ms') (FenceOut $script:Fid)
Shot 'stopped-three-times'
Start-Sleep -Seconds 7
Check 'and does not start again by itself' ((FenceOut $script:Fid) -match 'three times within an hour') (FenceOut $script:Fid)
Remove-Item "$script:Data\cmd.txt"
$r = (FenceGet $script:Fid).windowRect
$clicked = OcrClick '^Restart$' $r.x $r.y $r.w $r.h
Start-Sleep -Seconds 3
Check 'Restart in the fence starts it again' ($clicked -and (WaitRow 'sys' '^ok' 10) -match '^ok') "clicked=$clicked out=$(FenceOut $script:Fid)"

Case 'the last view is cached; quit stops the plugin thread'
PfQuit
$cache = "$script:Config\plugins-host\$($script:Fid).view.json"
$text = if (Test-Path $cache) { Get-Content $cache -Raw -Encoding UTF8 } else { '' }
Check 'the view cache file has the last rows' ($text -match '"rows"' -and $text -match 'http-declared') $cache
Check 'the log says every plugin stopped and the thread ended' ([bool](PfWaitLog 'plugin thread: every plugin stops' 5) -and [bool](PfWaitLog 'plugin thread ended' 5)) ''

Case 'with a system proxy (Clash and the like on 127.0.0.1) requests go through it'
[MiniProxy]::Start(18768); SystemProxy '127.0.0.1:18768'
PfStart -Lang en
Command 'proxied'
$v = WaitRow 'cmd-proxied' '^(ok|error)' 30
$seen = @([MiniProxy]::Seen) -join ' | '
SystemProxy ''; [MiniProxy]::Stop()
Check 'api.github.com answers 200 through the proxy' ($v -eq 'ok 200' -and $seen -match 'CONNECT api\.github\.com:443') "$v; proxy saw: $seen"
Check 'the log says the proxy''s address is not checked' ([bool](PfWaitLog 'plugin request goes through a proxy' 2)) ''
PfQuit
Set-Clipboard -Value ' '
