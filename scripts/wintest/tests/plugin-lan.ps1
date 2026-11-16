# 插件 interface 2, row 6 of plan v6: services of this computer and of the local network through
# pf.http (assets\ollama, assets\home-assistant, assets\lan-probe, assets\local-web; stand-ins from
# assets\standin-http.ps1). Part A (an EC2 box, debug build, --var Lan=<private address of another
# box> where a Home Assistant stand-in answers on 8123 with the token below): the Ollama plugin lists
# the stand-in's models over plain http and asks it a question from its box; the details (and the
# install warning, the same lines) say 「连接这台电脑上的 Ollama（端口 11434）」 and 「连接你在设置里填写的局域网设备
# （端口 8123）」; a device address typed in the settings (normalised, kept under __devices, the note
# beside it) is the only LAN host a request reaches; this computer and public addresses are refused
# for the LAN class, other addresses for the loopback class, undeclared ports by the manifest, none of
# them connected to; the Home Assistant plugin shows the device's states and toggles a light; with a
# system proxy the requests go straight to the services; a web plugin's page reaches Ollama through
# pf.http and not with fetch; an import drops the device. Part W (a Hyper-V guest, no --var): a
# `.local` name (the guest's own) typed as the device resolves and is reached.
if (-not (Get-Variable Part -ErrorAction SilentlyContinue)) { $Part = if (Get-Variable Lan -ErrorAction SilentlyContinue) { 'A' } else { 'W' } }
$Kit = 'C:\pf\kit\assets'
$Lr = 'C:\pf\run\lan'
$Token = 'ha-token-7d2e'
$script:Standins = @()
New-Item -ItemType Directory -Force $Lr | Out-Null

# A stand-in service of standin-http.ps1 on address:port; its log (one line per request).
function Standin([string]$kind, [string]$address, [int]$port) {
  $log = "$Lr\$kind-$port.log"
  $p = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "$Kit\standin-http.ps1", '-Kind', $kind, '-Address', $address, '-Port', $port, '-Log', $log, '-Token', $Token -WindowStyle Hidden -PassThru
  for ($i = 0; $i -lt 40 -and -not (Test-Path $log); $i++) { Start-Sleep -Milliseconds 250 }
  $script:Standins += $p
  Log "stand-in $kind on ${address}:$port, pid $($p.Id)"
  return $log
}
function Requests([string]$log) { return @(Get-Content $log -Encoding UTF8 -ErrorAction SilentlyContinue | Where-Object { $_ -and $_ -notmatch '^listening' }) }
function WaitOut([string]$fid, [string]$regex, $sec = 15) { for ($i = 0; $i -lt $sec * 2; $i++) { if ((FenceOut $fid) -match $regex) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
# The fence settings of fence `fid` as the configuration file keeps them.
function Saved([string]$fid) { return (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts."$fid".settings }
function WaitSaved([string]$fid, [scriptblock]$ok, $sec = 10) { for ($i = 0; $i -lt $sec * 2; $i++) { if (& $ok (Saved $fid)) { return $true }; Start-Sleep -Milliseconds 500 }; return $false }
# Opens plugin `id`'s row (named `name`) in 设置 › 小工具 (CDP on the settings page); the text of
# its details.
function OpenPlugin([string]$id, [string]$name) {
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 2
  $i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$name'))")
  if ((Js "document.querySelectorAll('#pluginList .pl-row')[$i].getAttribute('aria-expanded')") -ne 'true') { PageClick '#pluginList .pl-row' $i; Start-Sleep -Seconds 1 }
  return (Js "(() => { const d = document.querySelector('#pluginList .pl-detail[data-plugin=`"$id`"]'); return d ? d.innerText : ''; })()")
}
# The selector of fence `fid`'s form, and `what` in it.
function InForm([string]$fid, [string]$what) { return "#pluginList .pl-form[data-fence=`"$fid`"] $what" }
# Sets a box of fence `fid`'s form as if typed, then leaves it (its change event).
function SetBox([string]$fid, [string]$what, [string]$value) {
  $q = (InForm $fid $what) | ConvertTo-Json; $v = $value | ConvertTo-Json
  return (Js "(() => { const i = document.querySelector($q); if (!i) return 'none'; i.value = $v; i.dispatchEvent(new Event('change')); return 'ok'; })()")
}
function BoxValue([string]$fid, [string]$what) { $q = (InForm $fid $what) | ConvertTo-Json; return (Js "(() => { const i = document.querySelector($q); return i ? i.value : 'none'; })()") }
# The plugin's view as fence `title` draws it, for the Ollama box: rows and cells (row * 100 + cell).
function CellRects($w) {
  $v = DumpVal $w 'script_cells'
  if (-not $v -or $v -eq '-') { return @() }
  return @($v -split ';' | ForEach-Object { $pair = $_ -split ':'; $n = $pair[1] -split ','
    [pscustomobject]@{ i = [int]$pair[0]; l = [double]$n[0]; t = [double]$n[1]; r = [double]$n[2]; b = [double]$n[3] } })
}
function ClickCell([string]$title, $row, $cell = 0) {
  $w = DumpOf $title 5; $r = @(CellRects $w | Where-Object { $_.i -eq $row * 100 + $cell }) | Select-Object -First 1
  if (-not $r) { Log "no cell $row/$cell in $title"; return $false }
  $sc = [double](DumpVal $w 'content_y') / 36.0
  [G]::Click([int]($w.l + [double](DumpVal $w 'content_x') + ($r.l + $r.r) / 2 * $sc), [int]($w.t + [double](DumpVal $w 'content_y') + ($r.t + $r.b) / 2 * $sc))
  return $true
}
function FenceShot([string]$fid, [string]$name) { $r = (FenceGet $fid).windowRect; ParkMouse; Start-Sleep -Milliseconds 400; (Crop $name ($r.x - 8) ($r.y - 8) ($r.w + 16) ($r.h + 16)).Dispose() }

if ($Part -eq 'W') {
  Case 'a .local name typed as the device (this computer''s own name, its address on the LAN)'
  CloseWindows
  $name = "$env:COMPUTERNAME.local".ToLower()
  $v4 = @(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -notmatch '^(127|169\.254)\.' -and $_.PrefixOrigin -ne 'WellKnown' } | Select-Object -ExpandProperty IPAddress) | Select-Object -First 1
  $resolved = try { @([Net.Dns]::GetHostAddresses($name) | ForEach-Object { $_.ToString() }) -join ' ' } catch { "error $($_.Exception.Message)" }
  Log "name $name, LAN address $v4, resolves to: $resolved"
  Check "Windows resolves $name (mDNS)" ($resolved -and $resolved -notmatch '^error') "$name -> $resolved; LAN address $v4"
  New-NetFirewallRule -DisplayName 'pf lan test 8123' -Direction Inbound -Protocol TCP -LocalPort 8123 -Action Allow -ErrorAction SilentlyContinue | Out-Null
  $Ha = Standin 'ha' $v4 8123
  $script:Config = "$PfApp\config"
  PfPlacePlugin "$Kit\lan-probe" 'lan-probe'
  PfStart -Lang en
  $Pr = PfPluginFence 'Probe' '80,120,560,160' 'lan-probe'
  Start-Sleep -Seconds 2
  PfQuit
  # The device and the address as the page would keep them (the page is tested in part A).
  $raw = Get-Content "$script:Config\config.json" -Raw -Encoding UTF8
  $j = $raw | ConvertFrom-Json
  $j.scripts."$Pr".settings | Add-Member -NotePropertyName '__devices' -NotePropertyValue ([pscustomobject]@{ host = $name }) -Force
  $j.scripts."$Pr".settings | Add-Member -NotePropertyName 'url' -NotePropertyValue "http://${name}:8123/api/states" -Force
  [IO.File]::WriteAllText("$script:Config\config.json", ($j | ConvertTo-Json -Depth 30), (New-Object Text.UTF8Encoding($false)))
  PfStart -Lang en
  $reached = WaitOut $Pr '-> 401 401: Unauthorized' 20
  $line = (PfLogAll | Where-Object { $_ -match 'plugin local request: the name resolved' } | Select-Object -Last 1)
  Check "the probe reaches the stand-in at $name" $reached "$(FenceOut $Pr) | $line"
  Check 'the stand-in heard it from the LAN address, Host the .local name' (@(Requests $Ha | Where-Object { $_ -match "host=$([regex]::Escape($name)):8123 hosts=1" }).Count -ge 1) ((Requests $Ha) -join ' | ')
  Shot 'local-name'
  foreach ($p in $script:Standins) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
  return
}

Case 'setup: stand-ins of Ollama and of the ports that must stay untouched, the plugins, their fences'
CloseWindows
$Ollama = Standin 'ollama' '127.0.0.1' 11434
# Whatever answers here must never hear of a request: a lan request to this computer, and a port
# the plugin did not declare.
$Trap8123 = Standin 'ha' '127.0.0.1' 8123
$Trap11435 = Standin 'ollama' '127.0.0.1' 11435
Check 'the stand-ins listen' ((Get-Content $Ollama -ErrorAction SilentlyContinue) -match 'listening' -and (Get-Content $Trap8123 -ErrorAction SilentlyContinue) -match 'listening') ''
$env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
$script:Config = "$PfApp\config"
foreach ($p in 'ollama', 'home-assistant', 'lan-probe', 'local-web') { PfPlacePlugin "$Kit\$p" $p }
PfStart -Lang zh-CN -Script (DumpScript 400 25 3000)
# Left of the settings window (x 700 on), whose page the steps use meanwhile.
$Of = PfPluginFence 'Ollama' '80,60,420,400' 'ollama'
$Hf = PfPluginFence 'HA' '80,500,420,300' 'home-assistant'
$Pr = PfPluginFence 'Probe' '520,60,170,300' 'lan-probe'

Case 'Ollama on this computer: its models over plain http, a question from the box, the answer'
Check 'the fence lists the two models of the stand-in' (WaitOut $Of 'llama3\.2:3b' 20) (FenceOut $Of)
Check 'and the second' ((FenceOut $Of) -match 'qwen2\.5:7b') (FenceOut $Of)
$got = Requests $Ollama
Check 'the stand-in got GET /api/tags from 127.0.0.1 with the Host the plugin named, once' (@($got | Where-Object { $_ -match '^GET /api/tags host=127\.0\.0\.1:11434 hosts=1 auth=no from=127\.0\.0\.1' }).Count -ge 1) ($got -join ' | ')
Check 'the log says how the request went: resolved, the address chosen, answered' ([bool](PfWaitLog 'plugin local request: the name resolved' 5) -and [bool](PfWaitLog 'plugin request answered.*local="?loopback' 5)) ''
Start-Sleep -Seconds 2
$clicked = ClickCell 'Ollama' 2
$box = WaitWin 'PecoFence.Rename' 5
Check 'a click on the box opens it' ($clicked -and $box -ne [IntPtr]::Zero) "box=$box"
[G]::Type('Why is the sky blue?'); Start-Sleep -Milliseconds 300
Chord 0x11 0x0D; Start-Sleep -Milliseconds 800
Check 'Ctrl+Enter asks: the answer of the model shows in the fence' (WaitOut $Of 'Stand-in answer to: Why is the sky blue\?' 15) (FenceOut $Of)
$post = @(Requests $Ollama | Where-Object { $_ -match '^POST /api/generate ' }) | Select-Object -Last 1
Check 'the stand-in got POST /api/generate with the model, the prompt and stream false' ($post -match '"model":"llama3\.2:3b"' -and $post -match '"prompt":"Why is the sky blue\?"' -and $post -match '"stream":false' -and $post -match 'hosts=1') "$post"
FenceShot $Of 'ollama-fence'

Case 'the details say which services it connects to (the install warning has the same lines)'
$ok = PfSettings
$sw = WaitWin 'PecoFence.Settings' 5
[L]::MoveWindow($sw, 700, 40, 900, 840, $true) | Out-Null
$d = OpenPlugin 'ollama' 'Ollama'
Check 'Ollama: 连接这台电脑上的 Ollama（端口 11434）' ($ok -and $d -match '连接这台电脑上的 Ollama（端口 11434）') $d
Js "document.querySelector('#pluginList .pl-detail[data-plugin=`"ollama`"]').scrollIntoView({ block: 'center' })" | Out-Null
Start-Sleep -Milliseconds 400; $r = Rect $sw; ParkMouse; (Crop 'ollama-details' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T)).Dispose()
$d = OpenPlugin 'home-assistant' 'Home Assistant'
Check 'Home Assistant: 连接你在设置里填写的局域网设备（端口 8123）' ($d -match '连接你在设置里填写的局域网设备（端口 8123）') $d
$d = OpenPlugin 'lan-probe' '局域网测试'
Check 'a plugin with both: both lines' ($d -match '连接这台电脑上的 Ollama（端口 11434）' -and $d -match '连接你在设置里填写的局域网设备（端口 8123）') $d
$note = Js ("(() => {{ const i = document.querySelector({0}); return i ? i.placeholder + ' | ' + i.closest('.pl-field').innerText : 'none'; }})()" -f ((InForm $Pr 'input.pl-device') | ConvertTo-Json))
Check 'the device box: an example address, and the note beside it' ($note -match '^192\.168\.1\.20 \|' -and $note -match '这个插件会向这个地址发送请求。') $note

Case 'a LAN request goes only to the device the user typed'
SetBox $Pr 'input[type=text]:not(.pl-device)' "http://${Lan}:8123/api/p1-nodevice" | Out-Null
Check 'no device typed: refused' (WaitOut $Pr "p1-nodevice -> error .*is not the address of a device typed in this fence's settings") (FenceOut $Pr)
# (The LAN box's own log shows that p1-nodevice never came: the address goes before a device.)
SetBox $Pr 'input[type=text]:not(.pl-device)' '' | Out-Null; Start-Sleep -Milliseconds 500
PageType (InForm $Pr 'input.pl-device') "http://${Lan}:8123/lovelace"
[G]::Press(0x09); Start-Sleep -Milliseconds 800
# The list is drawn again once the focus leaves it (a box being typed keeps its text).
Js "document.activeElement.blur()" | Out-Null; Start-Sleep -Milliseconds 800
Js ("document.querySelector({0}).scrollIntoView({{ block: 'center' }})" -f ((InForm $Pr 'input.pl-device') | ConvertTo-Json)) | Out-Null
Start-Sleep -Milliseconds 400
$r = Rect $sw; ParkMouse; (Crop 'device-setting' $r.L $r.T ($r.R - $r.L) ($r.B - $r.T)).Dispose()
Check 'the address as typed from a browser is kept as its host, under __devices, no plain value' ((WaitSaved $Pr { param($s) $s.__devices.host -eq $Lan -and -not $s.host }) -and (BoxValue $Pr 'input.pl-device') -eq $Lan) "box=$(BoxValue $Pr 'input.pl-device') saved=$((Saved $Pr) | ConvertTo-Json -Compress)"
Check 'the plugin sees it in pf.settings' (WaitOut $Pr "device=$([regex]::Escape($Lan))") (FenceOut $Pr)
SetBox $Pr 'input[type=text]:not(.pl-device)' "http://${Lan}:8123/api/p2-device" | Out-Null
Check 'the device: reached over plain http (the stand-in wants a token: 401)' (WaitOut $Pr "p2-device -> 401 401: Unauthorized") (FenceOut $Pr)
Check 'the log names the class and the address' ([bool](PfWaitLog 'plugin lan request: the host against the device addresses typed.*found=true' 5)) ''
SetBox $Pr 'input[type=text]:not(.pl-device)' "http://10.99.99.99:8123/api/p3-other" | Out-Null
Check 'another address of the local network: refused' (WaitOut $Pr "p3-other -> error 10\.99\.99\.99 is not the address of a device typed") (FenceOut $Pr)
SetBox $Pr 'input[type=text]:not(.pl-device)' "http://${Lan}:11434/" | Out-Null
Check 'the device on a port only `loopback` declares: refused by the manifest' (WaitOut $Pr ":11434/ -> error http\.get needs .*permissions\.local") (FenceOut $Pr)

Case 'this computer is no LAN device, a public address neither; loopback only to this computer'
SetBox $Pr 'input.pl-device' '127.0.0.1' | Out-Null; Start-Sleep -Milliseconds 800
SetBox $Pr 'input[type=text]:not(.pl-device)' 'http://127.0.0.1:8123/api/p4-self' | Out-Null
Check '127.0.0.1 typed as the device: refused, not an address of the local network' (WaitOut $Pr "p4-self -> error 127\.0\.0\.1 resolves to 127\.0\.0\.1, not an address of the local network") (FenceOut $Pr)
SetBox $Pr 'input.pl-device' 'example.com' | Out-Null; Start-Sleep -Milliseconds 800
SetBox $Pr 'input[type=text]:not(.pl-device)' 'http://example.com:8123/p5-public' | Out-Null
Check 'a public name typed as the device: refused before connecting' (WaitOut $Pr "p5-public -> error example\.com resolves to .*, not an address of the local network") (FenceOut $Pr)
SetBox $Pr 'input[type=text]:not(.pl-device)' 'http://localhost:11434/api/tags' | Out-Null
Check 'localhost on the loopback port: reached (::1 first, the stand-in listens on 127.0.0.1)' (WaitOut $Pr 'localhost:11434/api/tags -> 200 \{"models"') (FenceOut $Pr)
$tags = @(Requests $Ollama | Where-Object { $_ -match '^GET /api/tags host=localhost:11434 hosts=1 ' })
Check 'with the Host header localhost:11434, once' ($tags.Count -ge 1) ((Requests $Ollama) -join ' | ')
SetBox $Pr 'input[type=text]:not(.pl-device)' 'http://127.0.0.1:11435/p6-port' | Out-Null
Check 'a port of this computer not declared: refused by the manifest' (WaitOut $Pr "p6-port -> error http\.get needs .*permissions\.local") (FenceOut $Pr)
Check 'nothing reached the services that must stay untouched (127.0.0.1:8123 and :11435)' ((Requests $Trap8123).Count -eq 0 -and (Requests $Trap11435).Count -eq 0) "8123: $((Requests $Trap8123) -join ' | ') 11435: $((Requests $Trap11435) -join ' | ')"
SetBox $Pr 'input.pl-device' 'my box' | Out-Null; Start-Sleep -Milliseconds 800
$toast = PageToast
Check 'a device text that is no address: refused with a word on the page, the address before stays' ($toast.error -and $toast.text -match '请填写设备的地址' -and (Saved $Pr).__devices.host -eq 'example.com') "$($toast.text) saved=$((Saved $Pr).__devices.host)"
SetBox $Pr 'input.pl-device' '' | Out-Null
Check 'an empty box removes the device' ((WaitSaved $Pr { param($s) -not $s.__devices.host }) -and (WaitOut $Pr 'device=none')) (FenceOut $Pr)

Case 'Home Assistant on the LAN device: the typed address and token, its states, a light toggled'
$d = OpenPlugin 'home-assistant' 'Home Assistant'
Check 'before an address: the plugin asks for it' (WaitOut $Hf 'Home Assistant 的地址') (FenceOut $Hf)
PageType (InForm $Hf 'input.pl-device') $Lan; [G]::Press(0x09); Start-Sleep -Milliseconds 800
PageType (InForm $Hf '.pl-secret input') $Token; [G]::Press(0x09); Start-Sleep -Milliseconds 800
Check 'the states of the device show' (WaitOut $Hf 'Living room temperature' 15) (FenceOut $Hf)
Check 'a sensor with its unit, a light that is on' ((FenceOut $Hf) -match '21\.5 °C' -and (FenceOut $Hf) -match 'Kitchen light') (FenceOut $Hf)
$before = FenceOut $Hf
FenceShot $Hf 'ha-fence'
Check 'a click on the light''s button turns it off' ((ClickCell 'HA' 2 1) -and (WaitOut $Hf 'Kitchen light\W+关' 10)) "before: $before | after: $(FenceOut $Hf)"
ClickCell 'HA' 2 1 | Out-Null
Check 'and on again' (WaitOut $Hf 'Kitchen light\W+开' 10) (FenceOut $Hf)

Case 'with a system proxy (Clash and the like): local requests go straight to the services'
$script:Cdp.Dispose(); $script:Cdp = $null
PfQuit
[MiniProxy]::Start(18768); SystemProxy '127.0.0.1:18768'
$n0 = (Requests $Ollama).Count
PfStart -Lang zh-CN
Check 'Ollama answers again' (WaitOut $Of 'llama3\.2:3b' 20) (FenceOut $Of)
Check 'the device answers again' (WaitOut $Hf 'Living room temperature' 20) (FenceOut $Hf)
$direct = @(Requests $Ollama | Select-Object -Skip $n0 | Where-Object { $_ -match 'from=127\.0\.0\.1' }).Count
# A program that follows the system proxy sends a plain http request to the device through it.
$viaProxy = try { (Invoke-WebRequest "http://${Lan}:8123/api/p7-proxy" -UseBasicParsing -TimeoutSec 10).StatusCode } catch { "$($_.Exception.Message)" }
Start-Sleep -Milliseconds 800
$seen = @([MiniProxy]::Seen) -join ' | '
SystemProxy ''; [MiniProxy]::Stop()
Check 'PecoFence''s requests did not go to the proxy; Ollama heard them itself' ($seen -notmatch '/api/states' -and $seen -notmatch '11434' -and $direct -ge 1) "proxy saw: $seen; direct=$direct"
Check 'while another program''s request to the device did go to the proxy (it is in effect)' ($seen -match "GET http://$([regex]::Escape($Lan)):8123/api/p7-proxy") "proxy saw: $seen ($viaProxy)"

Case 'the web layer: a page reaches Ollama through pf.http, not with fetch'
$Wf = PfPluginFence 'LocalWeb' '520,400,170,160' 'local-web'
Check 'CDP reaches the page' ((CdpOpen -Url 'pecofence-plugin://local-web/*' -sec 30) -and (JsWait "document.readyState === 'complete'" 15)) ''
$n0 = (Requests $Ollama).Count
Js "(window.r = undefined, pf.http.get('http://127.0.0.1:11434/api/tags').then(a => window.r = a.status + ' ' + a.body.slice(0, 10), e => window.r = 'refused: ' + e.message), 1)" | Out-Null
$v = JsWait 'window.r' 10
Check 'pf.http from the page: the models' ("$v" -match '^200 \{"models"') "$v"
Js "(window.f = undefined, fetch('http://127.0.0.1:11434/api/tags').then(() => window.f = 'loaded', e => window.f = 'refused: ' + e.name), 1)" | Out-Null
$f = JsWait 'window.f' 10
Start-Sleep -Milliseconds 800
$more = @(Requests $Ollama | Select-Object -Skip $n0)
Check 'fetch from the page: refused, Ollama heard only the pf.http request' ("$f" -like 'refused*' -and $more.Count -eq 1) "fetch=$f; heard: $($more -join ' | ')"
$script:Cdp.Dispose(); $script:Cdp = $null
Pf fence delete $Wf | Out-Null

Case 'an import drops the device; another plugin in the fence gets none'
PfSettings | Out-Null
$d = OpenPlugin 'lan-probe' '局域网测试'
PageType (InForm $Pr 'input.pl-device') $Lan; [G]::Press(0x09)
Check 'the device is set again' (WaitSaved $Pr { param($s) $s.__devices.host -eq $Lan }) ''
$file = "$Lr\export.json"
Pf config export $file | Out-Null
Check 'an export keeps it (the user''s own computer)' ((Get-Content $file -Raw -Encoding UTF8) -match '"__devices"') ''
Pf config import $file | Out-Null; Start-Sleep -Seconds 3
# An imported file's scripts wait for the user's turn of 小工具.
Pf settings set tools.enabled true | Out-Null
Check 'an import of that file: no __devices left, the plugin sees no device' ((WaitSaved $Pr { param($s) -not $s.__devices }) -and (WaitOut $Pr 'device=none' 20)) "$((Saved $Pr) | ConvertTo-Json -Compress) | $(FenceOut $Pr)"
Check 'no errors in the log' (@(PfLogErrors).Count -eq 0) ((PfLogErrors) -join ' | ')

foreach ($p in $script:Standins) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue }
