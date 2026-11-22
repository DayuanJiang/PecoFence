# Web plugins' browser permissions (plugin interface 3): the test plugin assets/webperms asks for the
# screen (getDisplayMedia), the camera and microphone (fake devices), the location, a notification
# and a WebRTC connection, and runs inline code. Debug build (CDP on the page).
#   Part U: nothing declared: each is refused at once (no screen picker), inline code of the
#           page's file runs, code made at run time from text does not.
#   Part D: screen, camera, microphone, location, webrtc and notify declared: each works (the
#           screen picker shows, or with --auto-select-desktop-capture-source the stream comes).
#   Part S: a decision is not kept in the plugin's browser profile: camera refused, then declared
#           (allowed), then not declared again (refused), one profile across three starts.
#   Part O: the upgrade: 0.5.0 (its debug build in --extra) refuses the camera and keeps that in
#           the profile; the build under test, the camera declared, still gets the stream.
# run.sh --bin target/wintest-bin-debug --var Part=U|D|S [--var Api=3] tests/plugin-web-perms.ps1
# run.sh --bin target/wintest-bin-debug --extra <0.5.0 debug bin> --var Part=O --var Api=3 ...
if (-not $Part) { $Part = 'U' }
$Demo = 'C:\pf\kit\assets\webperms'
$Declared = '"network": ["example.org"], "screen": true, "camera": true, "microphone": true, "location": true, "webrtc": true, "notify": true'

# One call on the page: the Promise is awaited, with a user gesture (getDisplayMedia needs one).
function Ask([string]$expr) {
  $script:CdpN++
  $id = $script:CdpN
  $body = @{ id = $id; method = 'Runtime.evaluate'; params = @{ expression = "(async () => JSON.stringify({ v: await ($expr) }))()"; awaitPromise = $true; returnByValue = $true; userGesture = $true } } | ConvertTo-Json -Depth 5 -Compress
  $bytes = [Text.Encoding]::UTF8.GetBytes($body)
  $none = [Threading.CancellationToken]::None
  $script:Cdp.SendAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $bytes)), [Net.WebSockets.WebSocketMessageType]::Text, $true, $none).Wait()
  while ($true) {
    $ms = New-Object IO.MemoryStream
    $buf = New-Object byte[] 65536
    do { $r = $script:Cdp.ReceiveAsync((New-Object 'ArraySegment[byte]' -ArgumentList (, $buf)), $none).Result; $ms.Write($buf, 0, $r.Count) } while (-not $r.EndOfMessage)
    $resp = [Text.Encoding]::UTF8.GetString($ms.ToArray()) | ConvertFrom-Json
    if ($resp.id -eq $id) { break }
  }
  if ($resp.result.exceptionDetails) { return "exception: $($resp.result.exceptionDetails.exception.description)" }
  $v = ($resp.result.result.value | ConvertFrom-Json).v
  Log "ask $expr -> $($v | ConvertTo-Json -Compress -Depth 4)"
  return $v
}
# Whether WebView2's screen picker shows (it is drawn in the page: read the screen).
function PickerShows() { return [bool](OcrFind 'Choose what to share') }
function Manifest([string]$perms) {
  $j = "$script:Config\plugins\webperms\1.0.0\plugin.json"
  $api = if ($Api) { "`"api`": $Api, " } else { '' }
  (Get-Content "$Demo\plugin.json" -Raw) -replace '"permissions": \{[^}]*\}', "$api`"permissions`": { $perms }" | Set-Content $j -Encoding UTF8
  Log "manifest: $(Get-Content $j -Raw)"
}
function OpenPage() {
  $fid = PfPluginFence 'Perms' '420,160,380,420' 'webperms'
  $loaded = PfWaitLog 'web plugin: page loading' 30
  $reached = CdpOpen -Url 'pecofence-plugin://webperms/*' -sec 30
  $ready = JsWait "document.getElementById('out').textContent === 'app.js 已运行'" 15
  Check 'the test page is up' ($loaded -and $reached -and $ready) "loaded=$([bool]$loaded) cdp=$reached ready=$ready"
  return $fid
}

Case 'setup'
CloseWindows
if (-not $PfDebugBuild) { Check 'a debug build (CDP on the page)' $false $PfBin; return }
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352 --use-fake-device-for-media-stream'
if ($Part -eq 'D') { $env:PECOFENCE_WEBVIEW_ARGS += ' --auto-select-desktop-capture-source="Entire screen"' }
$script:Config = "$PfApp\config"
PfPlacePlugin $Demo 'webperms'
if ($Part -eq 'D') { Manifest $Declared }
if ($Part -eq 'O') { Copy-Item "$PfExtra\*" $PfApp -Force; Log "first start: the build in $PfExtra" }
PfStart -Lang en

if ($Part -eq 'U') {
  Case 'nothing declared'
  OpenPage | Out-Null
  $s = Ask 'window.t.screen()'
  $picker = PickerShows
  Shot 'U-after-getDisplayMedia'
  Check 'getDisplayMedia is refused at once' ("$s" -like 'NotAllowedError*') "$s"
  Check 'no screen picker shows' (-not $picker) "picker=$picker"
  $c = Ask 'window.t.camera()'
  Check 'the camera is refused' ("$c" -like 'NotAllowedError*') "$c"
  $m = Ask 'window.t.microphone()'
  Check 'the microphone is refused' ("$m" -like 'NotAllowedError*') "$m"
  $l = Ask 'window.t.location()'
  Check 'the location is refused (no position)' ("$l" -like 'error 1:*' -or "$l" -eq 'pending') "$l"
  $n = Ask 'window.t.notification()'
  Check 'notifications are refused' ("$n" -eq 'permission denied') "$n"
  $w = Ask 'window.t.webrtc()'
  Check 'there is no WebRTC' ("$w" -like 'no RTCPeerConnection*') "$w"
  Case 'inline code'
  $i = Ask 'window.t.inline()'
  Check 'an inline script of the page file runs' ($i.inlineScript -eq $true) ($i | ConvertTo-Json -Compress)
  Check 'an inline style element applies' ($i.inlineStyle -eq 'rgb(1, 2, 3)') "$($i.inlineStyle)"
  Check 'a style attribute applies' ($i.styleAttribute -eq '7px') "$($i.styleAttribute)"
  Check 'an inline event handler runs' ($i.handler -eq $true) "$($i.handler)"
  Check 'a data: picture loads' ($i.dataImage -eq $true) "$($i.dataImage)"
  Check 'a blob: picture loads' ($i.blobImage -eq $true) "$($i.blobImage)"
  Check 'a script made at run time from text does not run' ($i.runtimeScript -eq $false) "$($i.runtimeScript)"
  Check 'eval does not run' ($i.evaluated -eq $false) "$($i.evaluated)"
  Check 'a worker from a blob: address does not start' ($i.blobWorker -eq $false) "$($i.blobWorker)"
  $refused = @(PfLogAll | Where-Object { $_ -match 'web plugin: (permission refused|screen capture refused)' })
  Log "refusals in the log: $($refused.Count)"
}

if ($Part -eq 'D') {
  Case 'all declared'
  OpenPage | Out-Null
  $s = Ask 'window.t.screen()'
  $picker = PickerShows
  Shot 'D-after-getDisplayMedia'
  Check 'getDisplayMedia goes on (a stream, or the picker shows)' ("$s" -like 'stream video:*' -or ("$s" -eq 'pending' -and $picker)) "$s / picker=$picker"
  $c = Ask 'window.t.camera()'
  Check 'the camera gives a stream' ("$c" -like 'stream video:*') "$c"
  $m = Ask 'window.t.microphone()'
  Check 'the microphone gives a stream' ("$m" -like 'stream audio:*') "$m"
  $l = Ask 'window.t.location()'
  Check 'the location is not refused (a position, or none known on this box)' ("$l" -eq 'position' -or "$l" -like 'error 2:*' -or "$l" -like 'error 3:*' -or "$l" -eq 'pending') "$l"
  $n = Ask 'window.t.notification()'
  $card = PfWaitLog 'web plugin: notification' 5
  Shot 'D-notification-card'
  Check 'a notification shows as PecoFence''s card' ("$n" -eq 'shown' -and $card) "$n / $card"
  $w = Ask 'window.t.webrtc()'
  Check 'two peer connections exchange a message' ("$w" -eq 'message hi') "$w"
  $i = Ask 'window.t.inline()'
  Check 'code made at run time still does not run' ($i.runtimeScript -eq $false -and $i.evaluated -eq $false -and $i.blobWorker -eq $false) ($i | ConvertTo-Json -Compress)
}

if ($Part -eq 'S') {
  Case 'camera undeclared'
  OpenPage | Out-Null
  $c1 = Ask 'window.t.camera()'
  Check 'first start: the camera is refused' ("$c1" -like 'NotAllowedError*') "$c1"
  Case 'camera declared'
  PfQuit
  Manifest '"network": ["example.org"], "camera": true'
  PfStart -Lang en
  $script:Cdp = $null
  $reached = CdpOpen -Url 'pecofence-plugin://webperms/*' -sec 40
  JsWait "document.getElementById('out').textContent === 'app.js 已运行'" 15 | Out-Null
  $c2 = Ask 'window.t.camera()'
  Check 'second start, declared: the camera gives a stream (the refusal was not kept)' ("$c2" -like 'stream video:*') "$c2 cdp=$reached"
  Case 'camera undeclared again'
  PfQuit
  Manifest '"network": ["example.org"]'
  PfStart -Lang en
  $script:Cdp = $null
  $reached = CdpOpen -Url 'pecofence-plugin://webperms/*' -sec 40
  JsWait "document.getElementById('out').textContent === 'app.js 已运行'" 15 | Out-Null
  $c3 = Ask 'window.t.camera()'
  Check 'third start, undeclared: the camera is refused (the grant was not kept)' ("$c3" -like 'NotAllowedError*') "$c3 cdp=$reached"
}

if ($Part -eq 'O') {
  Case 'the camera refused by 0.5.0'
  OpenPage | Out-Null
  $c1 = Ask 'window.t.camera()'
  Check 'first start (0.5.0): the camera is refused' ("$c1" -like 'NotAllowedError*') "$c1"
  Case 'the build under test, camera declared'
  PfQuit
  Copy-Item "$PfBin\*" $PfApp -Force
  Manifest '"network": ["example.org"], "camera": true'
  PfStart -Lang en
  $script:Cdp = $null
  $reached = CdpOpen -Url 'pecofence-plugin://webperms/*' -sec 40
  JsWait "document.getElementById('out').textContent === 'app.js 已运行'" 15 | Out-Null
  $c2 = Ask 'window.t.camera()'
  $asked = PfWaitLog 'web plugin: permission allowed, the manifest declares it.*Camera' 5
  Check 'second start: PermissionRequested comes and the camera gives a stream (0.5.0''s kept refusal does not answer)' ("$c2" -like 'stream video:*' -and $asked) "$c2 / asked=$([bool]$asked) cdp=$reached"
}
