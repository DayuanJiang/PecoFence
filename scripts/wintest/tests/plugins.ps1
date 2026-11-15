# 插件 against a local plugin index (PECOFENCE_PLUGINS_URL on a named instance): 设置 › 小工具
# with the plugin list and no built-in tool rows; the warning before an install; the plugin
# fence; a file whose SHA-256 is not the index's (refused, nothing left); the index signature
# (a signed index taken, a wrong signature refused); the settings form changing one fence only;
# an update that asks for nothing more (by itself, the version before kept), one whose new
# version fails to start (back to the one before), one that asks for more (waits for the OK);
# a plugin that starts programs (the stricter warning, every update asks) and the switch for
# Windows functions; a take-down; an uninstall during a download (its result dropped); an
# imported fence whose plugin is not installed; a take-down from the saved index while offline.
# The fix pass: a fence's cached view goes with it; the warning shows again when the index changes
# the plugin while it is open; a plugin taken down or an entry for a newer PecoFence is not
# installed when the page asks; a failed switch starts the fences again; a web plugin's update loads
# the new page; a plugin fence holding a note has 编辑便签 and its uninstall names the notes; an
# older index served again counts as none.
# English UI. Debug build (CDP on the settings page). The signed index is the kit's
# assets\signed-feed (index.json, index.json.sig and the plugin files, signed with the
# development key: made again with make_signed_feed.py when the release key replaces it).
$Feed = 'C:\pf\run\pfeed'
# The plugins the feed offers, by id: version, permissions, settings, main.js, a take-down reason,
# whether its main.js differs from its hash ('bad') or is sent slowly ('slow').
$script:Offer = [ordered]@{}
# The index's serial (an older one than the index kept is refused).
$script:Serial = 0
function Offer($id, $name, $version, $perms = @{}, [string]$js = '', [string]$disabled = '', [switch]$Bad, [switch]$Slow, [switch]$Web, [string]$Min = '0.4.0') {
  $settings = @(@{ key = 'label'; type = 'text'; label = @{ en = 'Label' }; default = '' })
  $script:Offer[$id] = @{ name = $name; version = $version; perms = $perms; js = $js; disabled = $disabled; bad = [bool]$Bad; slow = [bool]$Slow; settings = $settings; web = [bool]$Web; min = $Min }
}
# Writes the offered plugins at a made-up commit `n` and the index naming that commit (no
# signature: a test index needs none).
function Publish([int]$n) {
  $commit = ('{0:x}' -f $n).PadLeft(40, 'a')
  Remove-Item "$Feed\index.json.sig" -ErrorAction SilentlyContinue
  $entries = foreach ($id in $script:Offer.Keys) {
    $p = $script:Offer[$id]
    $dir = "$Feed\$commit\plugins\$id"; New-Item -ItemType Directory -Force $dir | Out-Null
    $layer = if ($p.web) { 'web' } else { 'light' }
    $m = [ordered]@{ id = $id; version = $p.version; api = 1; layer = $layer; name = @{ en = $p.name }; description = @{ en = 'A test plugin' }
      author = 'tester'; minPecoFence = $p.min; permissions = $p.perms; settings = $p.settings }
    [IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 6))
    $js = if ($p.js) { $p.js } else { "function draw() { pf.show({ rows: [{ text: '$id $($p.version) ' + (pf.settings.label || '') }] }); }`r`npf.on('start', draw); pf.on('settings', draw);`r`n" }
    if ($p.web) { [IO.File]::WriteAllText("$dir\index.html", "<!doctype html><meta charset=utf-8><title>$id</title><p>$id $($p.version)</p>") }
    else { [IO.File]::WriteAllText("$dir\main.js", $js) }
    [IO.File]::WriteAllText("$dir\README.md", "# $id`r`n")
    $files = foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } }
    if ($p.bad) { Add-Content "$dir\main.js" '// changed after the review' }
    if ($p.slow) { Set-Content "$dir\main.js.slow" 'the feed waits 8 s before it sends main.js' }
    $m.files = @($files)
    if ($p.disabled) { $m.disabled = $p.disabled }
    $m
  }
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = $script:Serial; commit = $commit; plugins = @($entries) } | ConvertTo-Json -Depth 8))
  Log "published $(($script:Offer.Keys | ForEach-Object { "$_ $($script:Offer[$_].version)" }) -join ', ') at $commit, serial $script:Serial"
}
function RowButton($name) { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$name'))") }
# The page asks for the index again (as when 设置 › 小工具 opens).
function Refresh() { Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null; Start-Sleep -Seconds 3 }
function Installed($id = 'pf-test') { $v = Get-Content "$script:Config\plugins\$id\current.txt" -ErrorAction SilentlyContinue; if ($v -and (Test-Path "$script:Config\plugins\$id\$v\plugin.json")) { return $v }; return $null }
function Versions($id = 'pf-test') { return (@(Get-ChildItem "$script:Config\plugins\$id" -Directory -ErrorAction SilentlyContinue | ForEach-Object Name) | Sort-Object) -join ' ' }
function Temps() { return @(Get-ChildItem "$script:Config\plugins" -Directory -Filter '.new-*' -ErrorAction SilentlyContinue).Count }
function Saved($id) { return (Get-Content "$script:Config\config.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts.$id.settings }
# Clicks 安装 / 更新 in the row of `name`, then the warning's OK; the warning's texts.
function InstallVia($name, [switch]$Cancel) {
  $i = [int](RowButton $name)
  PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 600
  $d = Js "(() => ({ shown: !document.getElementById('plBack').hidden, text: document.getElementById('plBack').innerText, system: !document.getElementById('plSystem').hidden, off: !document.getElementById('plSystemOff').hidden }))()"
  Shot "plugin-warning-$name"
  if ($Cancel) { PageClick '#plCancel' } else { PageClick '#plOk' }
  Start-Sleep -Seconds 4
  return $d
}
# The feed: files under C:\pf\run\pfeed; a file with a .slow marker beside it comes 8 s late.
function StartFeed() { $script:FeedProc = PfFeed 8731 -log 'C:\pf\run\pfeed.log' -Slow }
function StopFeed() { if ($script:FeedProc) { Stop-Process -Id $script:FeedProc.Id -Force -ErrorAction SilentlyContinue; $script:FeedProc = $null } }
function ToolsPage() { $ok = PfSettings; PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3; return $ok }
# The row of plugin `id` as the host last sent it (its state and the key of what its warning shows).
function HostRow($id) {
  Js "(() => { if (!window.pfRows) { window.pfRows = {}; window.chrome.webview.addEventListener('message', e => { if (e.data && e.data.type === 'state' && e.data.plugins) window.pfRows.v = e.data.plugins; }); } return true; })()" | Out-Null
  Refresh
  return (Js "(() => { const l = window.pfRows.v && window.pfRows.v.list.find(x => x.id === '$id'); return l ? { state: l.state, shown: l.shown } : null; })()")
}
# The texts of a message box.
function BoxText($dlg) {
  $t = New-Object Collections.Generic.List[string]
  [L]::EnumChildWindows($dlg, { param($h, $l) if ([L]::Cls($h) -eq 'Static') { $t.Add([L]::Text($h)) }; $true }, [IntPtr]::Zero) | Out-Null
  return (($t | Where-Object { $_ }) -join ' ')
}
# Opens the details of plugin row `name` (when closed) and clicks its 卸载; the question box.
function UninstallVia($name, $id) {
  Js "(() => { const r = [...document.querySelectorAll('#pluginList .pl-row')].find(r => r.textContent.startsWith('$name')); if (r.getAttribute('aria-expanded') !== 'true') r.click(); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 600
  Js "[...document.querySelectorAll('#pluginList .pl-detail[data-plugin=$id] button')].find(b => b.textContent === 'Uninstall').click()" | Out-Null
  return (WaitWin '#32770' 5)
}

Case 'setup'
CloseWindows
New-Item -ItemType Directory -Force $Feed | Out-Null
StartFeed
Offer 'pf-test' 'PF Test' '1.0.0'
Offer 'pf-bad' 'PF Bad' '1.0.0' -Bad
Publish 1
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8731/index.json'
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
$ok = ToolsPage
Check 'the feed runs, the settings page is reachable' ($script:FeedProc -and -not $script:FeedProc.HasExited -and $ok) "config=$script:Config"
Check 'the tools page lists both plugins with 安装' ((PluginRows) -match '^PF Test.*Install / PF Bad.*Install$') (PluginRows)

Case '设置 › 小工具: the 小工具 switch, the own script, the plugin list and the switch for Windows functions'
$p = Js "(() => ({ rows: [...document.querySelectorAll('#page-tools .card.child:not(.pl-row) .t')].map(t => t.textContent), binds: [...document.querySelectorAll('#page-tools .toggle')].map(t => t.dataset.bind), head: document.getElementById('plugins').textContent }))()"
Shot 'tools-page-after'
Check 'only Your own script is left of the tool rows; two switches; the list is headed Plugins' (($p.rows -join '|') -eq 'Your own script' -and ($p.binds -join ' ') -eq 'tools.enabled tools.allowSystem' -and $p.head -eq 'Plugins') ($p | ConvertTo-Json -Compress)

Case 'install: the warning first, then the plugin fence runs with its data folder'
$d = InstallVia 'PF Test'
Check 'the warning shows the author, the version, what it does and the promise' ($d.shown -and $d.text -match 'By tester\. The reviewed version is 1\.0\.0' -and $d.text -match 'It will:' -and $d.text -match 'Only shows things' -and $d.text -match 'A plugin can only do what is listed here' -and -not $d.system) $d.text
$line = PfWaitLog 'plugins: fence added' 15
$f = @(PluginFences pf-test)
Shot 'plugin-fence-v1'
Check 'installed into plugins\pf-test\1.0.0, one fence titled after the plugin' ((Installed) -eq '1.0.0' -and $f.Count -eq 1 -and $f[0].title -eq 'PF Test' -and $line) "fences=$($f.Count) $line"
Start-Sleep -Seconds 3; $f = @(PluginFences pf-test)
Check 'the fence runs main.js' ($f[0].script.output -match 'pf-test 1\.0\.0') "output=$($f[0].script.output)"
Check 'the row now offers nothing to install' ((PluginRows) -match '^PF Test.*On the desktop') (PluginRows)

Case 'a file whose SHA-256 is not the index''s is refused'
$d = InstallVia 'PF Bad'
$toast = PageToast
Check 'nothing of pf-bad is left, the page says why' (-not (Test-Path "$script:Config\plugins\pf-bad") -and (Temps) -eq 0 -and $toast.text -match 'differ from the reviewed') ($toast | ConvertTo-Json -Compress)
Check 'the log names the file' ([bool](PfWaitLog 'plugins: install or update failed.*refused=true.*main\.js: size or SHA-256' 5)) ''
$script:Offer.Remove('pf-bad')

Case 'the index signature: a signed index is taken, a wrong signature counts as no index'
Copy-Item 'C:\pf\kit\assets\signed-feed\*' $Feed -Recurse -Force
Refresh
$same = (Get-FileHash "$Feed\index.json").Hash -eq (Get-FileHash "$script:Config\plugins\index.json" -ErrorAction SilentlyContinue).Hash
Check 'signed with the development key: its plugin is listed, the index is saved as it came' ((PluginRows) -match 'PF Signed.*Install' -and $same) "saved=$same rows=$(PluginRows)"
Set-Content "$Feed\index.json.sig" ('0' * 128) -NoNewline
Refresh
$offline = Js "document.getElementById('pluginList').textContent.includes('reach the plugin list')"
Check 'a wrong signature: the page cannot reach the list, the log says why' ($offline -and [bool](PfWaitLog 'index not taken from this address.*signature does not match' 5)) "offline=$offline"
Publish 2; Refresh
Check 'a test index without a signature is taken again' ((PluginRows) -match '^PF Test' -and (PluginRows) -notmatch 'PF Signed') (PluginRows)

Case 'the settings form of the details changes one fence only'
$second = "$((PfJson fence create --script plugin:pf-test --title Second --rect '900,420,300,160').fence.id)"
Start-Sleep -Seconds 3
$first = (@(PluginFences pf-test) | Where-Object id -ne $second)[0].id
PageClick '#pluginList .pl-row' ([int](RowButton 'PF Test')); Start-Sleep -Milliseconds 800
$forms = Js "document.querySelectorAll('#pluginList .pl-form').length"
Js "(() => { const i = document.querySelector('[data-fence=`"$second`"] input'); i.value = 'B'; i.dispatchEvent(new Event('change')); return true; })()" | Out-Null
Start-Sleep -Seconds 3
Js "document.querySelector('[data-fence=`"$second`"]').scrollIntoView({ block: 'center' })" | Out-Null; Start-Sleep -Milliseconds 400
Shot 'plugin-details-form'
$out = @{}; foreach ($f in (PluginFences pf-test)) { $out[$f.id] = $f.script.output }
Check 'two forms; the second fence keeps B and shows it, the first is unchanged' ($forms -eq 2 -and (Saved $second).label -eq 'B' -and -not (Saved $first).label -and $out[$second] -match 'pf-test 1\.0\.0 B' -and $out[$first] -notmatch ' B') "forms=$forms first=$($out[$first]) second=$($out[$second])"

Case 'a plugin fence''s cached view goes with the fence'
$third = "$((PfJson fence create --script plugin:pf-test --title Third --rect '1200,420,300,160').fence.id)"
$cache = "$script:Config\plugins-host\$third.view.json"
$made = $false; for ($i = 0; $i -lt 10 -and -not $made; $i++) { Start-Sleep -Seconds 1; $made = Test-Path $cache }
Pf fence delete $third | Out-Null
Start-Sleep -Seconds 5
Check 'its view was cached, and 5 s after the fence is deleted the file is gone' ($made -and -not (Test-Path $cache)) "made=$made left=$(Test-Path $cache)"

Case 'an update that asks for nothing more installs by itself and keeps the version before'
Offer 'pf-test' 'PF Test' '1.1.0'; Publish 3
Refresh; PfWaitLog 'plugins: installed, its fences start again' 15 | Out-Null; Start-Sleep -Seconds 3
$out = @{}; foreach ($f in (PluginFences pf-test)) { $out[$f.id] = $f.script.output }
Check 'pf-test is 1.1.0 beside 1.0.0; both fences run it, the second still with B' ((Installed) -eq '1.1.0' -and (Versions) -eq '1.0.0 1.1.0' -and $out[$first] -match '1\.1\.0' -and $out[$second] -match '1\.1\.0 B') "versions=$(Versions) out=$($out.Values -join ' | ')"

Case 'a new version that fails to start goes back to the version before'
Offer 'pf-test' 'PF Test' '1.2.0' -js "throw new Error('broken 1.2.0');`r`n"; Publish 4
$downloads = LogCount 'plugins: download id="pf-test".*1\.2\.0'
Refresh
$back = PfWaitLog 'the new version stopped after its update.*back=true' 15; Start-Sleep -Seconds 3
$f = @(PluginFences pf-test)
Check 'back on 1.1.0, 1.2.0 deleted and remembered as failed, the fences run 1.1.0' ($back -and (Installed) -eq '1.1.0' -and (Versions) -notmatch '1\.2\.0' -and (Get-Content "$script:Config\plugins\pf-test\failed.txt") -eq '1.2.0' -and $f[0].script.output -match '1\.1\.0') "versions=$(Versions) $back"
Refresh
$again = LogCount 'plugins: download id="pf-test".*1\.2\.0'
Check 'the next check does not take 1.2.0 again' ($again -eq $downloads + 1 -and (Installed) -eq '1.1.0') "downloads=$downloads then $again"

Case 'an update that asks for more waits for the OK'
Offer 'pf-test' 'PF Test' '2.0.0' @{ network = @('example.com') }; Publish 5
Refresh
Check 'not installed by itself, the row asks' ((Installed) -eq '1.1.0' -and (PluginRows) -match 'PF Test.*Network.*An update waits for your OK') (PluginRows)
$d = InstallVia 'PF Test'
Check 'the warning lists the new host' ($d.shown -and $d.text -match 'Network: example\.com') $d.text
Start-Sleep -Seconds 2
Check 'after the OK it is 2.0.0, its fences kept' ((Installed) -eq '2.0.0' -and @(PluginFences pf-test).Count -eq 2) "installed=$(Installed)"

Case 'a plugin that starts programs: the stricter warning, and every update asks'
Offer 'pf-sys' 'PF Sys' '1.0.0' @{ system = @('ipconfig.exe'); clipboard = 'write' }; Publish 6
Refresh
Check 'its row has the tags Clipboard and Starts programs' ((PluginRows) -match 'PF Sys.*Clipboard.*Starts programs') (PluginRows)
$d = InstallVia 'PF Sys'
Check 'the warning names the program, says what it can do and that the switch is off' ($d.system -and $d.off -and $d.text -match 'Take special care with the programs it starts: ipconfig\.exe' -and $d.text -match 'Starts these programs: ipconfig\.exe' -and $d.text -match 'can do anything your account can do' -and $d.text -match 'Writes to the clipboard') $d.text
Start-Sleep -Seconds 2
Offer 'pf-sys' 'PF Sys' '1.0.1' @{ system = @('ipconfig.exe'); clipboard = 'write' }; Publish 7
Refresh
Check 'installed, and 1.0.1 asking for nothing more still waits for the OK' ((Installed 'pf-sys') -eq '1.0.0' -and (PluginRows) -match 'PF Sys.*An update waits for your OK') (PluginRows)

Case 'the switch for Windows functions'
PageClick '[data-bind="tools.allowSystem"]'; Start-Sleep -Seconds 2
$on = (Pf settings get tools.allowSystem | Out-String).Trim()
$d = InstallVia 'PF Sys'
Check 'on: the settings keep it; the warning no longer asks for it' ($on -eq 'true' -and $d.system -and -not $d.off) "on=$on off=$($d.off)"
Check 'the confirmed update is installed' ((Installed 'pf-sys') -eq '1.0.1') "installed=$(Installed 'pf-sys')"
PageClick '[data-bind="tools.allowSystem"]'; Start-Sleep -Seconds 1
Check 'turned off again' ((Pf settings get tools.allowSystem | Out-String).Trim() -eq 'false') ''

Case '新建便签 in the tray menu: the notes plugin''s row first, a fence of it once installed'
Offer 'notes' 'Notes' '1.0.0'; Publish 20; Refresh
[L]::PostMessageW((WaitWin 'PecoFence.Settings' 2), 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 1
function TrayOpen() {
  $ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
  [W]::PostMessageW($ctl, 0x8003, [IntPtr](1500 -bor (830 -shl 16)), [IntPtr](0x7B -bor (1 -shl 16))) | Out-Null
  return (MenuWin 5)
}
MenuPick '^New note' (TrayOpen) | Out-Null
$ok = (WaitWin 'PecoFence.Settings' 15) -ne [IntPtr]::Zero -and (CdpOpen) -and (JsWait "!!document.querySelector('#pluginList .pl-detail[data-plugin=notes]')" 15)
Shot 'new-note-opens-the-notes-row'
Check 'not installed: the settings page opens with the notes row open, no note fence, no editor' ($ok -and @(PluginFences 'notes').Count -eq 0 -and (WaitWin 'PecoFence.NoteEdit' 1) -eq [IntPtr]::Zero) "page=$ok"
InstallVia 'Notes' | Out-Null
MenuPick '^New note' (TrayOpen) | Out-Null; Start-Sleep -Seconds 3
$n = @(PluginFences 'notes')
Check 'installed: 新建便签 makes a second fence of the notes plugin' ((Installed 'notes') -eq '1.0.0' -and $n.Count -eq 2 -and @($n | Where-Object { $_.script.output -match 'notes 1\.0\.0' }).Count -eq 2) "fences=$($n.Count)"
$script:Offer.Remove('notes')
$ok = ToolsPage

Case 'a plugin taken down stops and says why'
$script:Offer['pf-sys'].disabled = 'Test reason'; Publish 8
Refresh
Shot 'plugin-taken-down'
$left = @(Get-ChildItem "$script:Config\plugins\pf-sys" -Recurse -File | ForEach-Object Name) -join ' '
Check 'its code deleted, plugin.json and disabled.txt with the reason left' ((($left -split ' ' | Sort-Object) -join ' ') -eq 'current.txt disabled.txt plugin.json') "left=$left"
Check 'the row says so and offers 卸载' ((PluginRows) -match 'PF Sys.*Taken down: Test reason.*Uninstall') (PluginRows)
$row = HostRow 'pf-sys'
Js "(() => { const l = window.pfRows.v.list.find(x => x.id === 'pf-sys'); window.chrome.webview.postMessage({ type: 'action', name: 'pluginInstall', id: 'pf-sys', shown: l.shown }); return true; })()" | Out-Null
Start-Sleep -Seconds 4
$refused = PfWaitLog 'plugins: download refused.*pf-sys.*taken down' 3
Check 'an install the page asks for anyway is refused: it stays down with its reason' ($row.state -eq 'disabled' -and $refused -and (Test-Path "$script:Config\plugins\pf-sys\disabled.txt") -and -not (Test-Path "$script:Config\plugins\pf-sys\1.0.1\main.js")) "state=$($row.state) $refused"

Case 'the index changes a plugin while its warning is open: the warning shows again'
Offer 'pf-ask' 'PF Ask' '1.0.0'; Publish 51; Refresh
PageClick '#pluginList .pl-row button' ([int](RowButton 'PF Ask')); Start-Sleep -Milliseconds 600
$first = Js "document.getElementById('plBack').innerText"
Offer 'pf-ask' 'PF Ask' '1.0.1' @{ clipboard = 'read' }; Publish 52; Refresh
PageClick '#plOk'; Start-Sleep -Seconds 2
$d = Js "(() => ({ shown: !document.getElementById('plBack').hidden, changed: !document.getElementById('plChanged').hidden, text: document.getElementById('plBack').innerText }))()"
Shot 'plugin-warning-again'
Check 'the OK of the old warning installs nothing; the warning shows again with 1.0.1, what it reads and why' ($first -match '1\.0\.0' -and -not (Installed 'pf-ask') -and $d.shown -and $d.changed -and $d.text -match '1\.0\.1' -and $d.text -match 'Reads the clipboard' -and $d.text -match 'just changed this plugin') $d.text
PageClick '#plOk'; Start-Sleep -Seconds 4
Check 'its OK installs 1.0.1' ((Installed 'pf-ask') -eq '1.0.1') "installed=$(Installed 'pf-ask')"
$script:Offer.Remove('pf-ask')

Case 'an entry that needs a newer PecoFence is refused, also when the page asks for it'
Offer 'pf-new' 'PF New' '1.0.0' -Min '9.0.0'; Publish 55
$row = HostRow 'pf-new'
Js "(() => { const l = window.pfRows.v.list.find(x => x.id === 'pf-new'); window.chrome.webview.postMessage({ type: 'action', name: 'pluginInstall', id: 'pf-new', shown: l.shown }); return true; })()" | Out-Null
Start-Sleep -Seconds 4
$refused = PfWaitLog 'plugins: download refused.*pf-new.*needs a newer' 3
Check 'its row says it needs a newer PecoFence; the install is refused, nothing downloaded' ($row.state -eq 'newer' -and (PluginRows) -match 'PF New.*Needs a newer PecoFence' -and -not (Installed 'pf-new') -and $refused) "state=$($row.state) $refused"
$script:Offer.Remove('pf-new')

Case 'a switch that fails starts the fences again on the version before'
Offer 'pf-ro' 'PF RO' '1.0.0'; Publish 56; Refresh
InstallVia 'PF RO' | Out-Null
Start-Sleep -Seconds 2
$starts = LogCount 'plugin fence starts its main\.js.*pf-ro'
# Held open without delete sharing: current.txt cannot be replaced.
$hold = [IO.File]::Open("$script:Config\plugins\pf-ro\current.txt", 'Open', 'Read', 'Read')
Offer 'pf-ro' 'PF RO' '1.0.1'; Publish 57; Refresh
$failed = PfWaitLog 'plugins: switch failed, the fences start again.*pf-ro' 15; Start-Sleep -Seconds 3
$again = LogCount 'plugin fence starts its main\.js.*pf-ro'
$f = @(PluginFences 'pf-ro')
Check 'the version before stays in use and its fence runs it again' ($failed -and (Installed 'pf-ro') -eq '1.0.0' -and $again -gt $starts -and $f[0].script.output -match 'pf-ro 1\.0\.0') "starts=$starts then $again output=$($f[0].script.output) $failed"
$hold.Dispose()
Refresh; PfWaitLog 'plugins: installed, its fences start again.*pf-ro' 15 | Out-Null; Start-Sleep -Seconds 2
Check 'once current.txt can be replaced, the update goes in' ((Installed 'pf-ro') -eq '1.0.1' -and @(PluginFences 'pf-ro')[0].script.output -match 'pf-ro 1\.0\.1') "installed=$(Installed 'pf-ro')"
$script:Offer.Remove('pf-ro')

Case 'a web plugin''s update loads the new version''s page'
Offer 'pf-web' 'PF Web' '1.0.0' -Web; Publish 58; Refresh
InstallVia 'PF Web' | Out-Null
$v1 = PfWaitLog 'web plugin: page loading plugin=pf-web' 30
$loads = LogCount 'web plugin: page loading plugin=pf-web'
Offer 'pf-web' 'PF Web' '1.0.1' -Web; Publish 59; Refresh
PfWaitLog 'plugins: installed, its fences start again.*pf-web' 15 | Out-Null
for ($i = 0; $i -lt 30 -and (LogCount 'web plugin: page loading plugin=pf-web') -le $loads; $i++) { Start-Sleep -Seconds 1 }
$text = ''
if (CdpOpen -Url 'pecofence-plugin://pf-web/*' -sec 15) { $text = JsWait "document.body && document.body.innerText.includes('1.0.1') && document.body.innerText" 10 }
Shot 'web-plugin-updated'
$ok = CdpOpen
Check 'after the update by itself the page loads again and shows 1.0.1' ($v1 -and (Installed 'pf-web') -eq '1.0.1' -and (LogCount 'web plugin: page loading plugin=pf-web') -gt $loads -and $text -match 'pf-web 1\.0\.1') "loads=$loads text=$text"
$script:Offer.Remove('pf-web')

Case 'a plugin fence holding a note: 编辑便签 in its menu, and its uninstall names the notes whose text goes'
Offer 'pf-note' 'PF Note' '1.0.0' @{} "pf.on('start', () => pf.show({ rows: [{ input: { id: 'note', type: 'note' } }] }));`r`n"; Publish 60; Refresh
InstallVia 'PF Note' | Out-Null
Start-Sleep -Seconds 3
$f = @(PluginFences 'pf-note')[0]
$r = $f.windowRect
[G]::RClick($r.x + 120, $r.y + 14); $menu = MenuWin 5
$texts = MenuTexts $menu
Shot 'note-plugin-menu'
MenuPick '^Edit note$' $menu | Out-Null
$editor = WaitWin 'PecoFence.NoteEdit' 5
Start-Sleep -Milliseconds 500; [G]::Type('Buy milk'); Start-Sleep -Milliseconds 500
$ok = ToolsPage; Start-Sleep -Seconds 1
Check 'its menu has Edit note and Add to-do beside the plugin entries; Edit note opens the editor, which keeps what is typed' ($texts -match 'Edit note' -and $texts -match 'Add to-do' -and $texts -match 'Plugin settings' -and $editor -ne [IntPtr]::Zero -and (FenceGet $f.id).noteText -match 'Buy milk') "menu=$texts note=$((FenceGet $f.id).noteText)"
$dlg = UninstallVia 'PF Note' 'pf-note'
$q = BoxText $dlg
Check 'the question says the text of 1 note goes too' ($dlg -ne [IntPtr]::Zero -and $q -match 'including the text of 1 note') $q
[L]::PostMessageW($dlg, 0x111, [IntPtr]6, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 2
Check 'after Yes: no fence, no plugin folder' (@(PluginFences 'pf-note').Count -eq 0 -and -not (Test-Path "$script:Config\plugins\pf-note")) ''
$script:Offer.Remove('pf-note')

Case 'an older index served again counts as none'
$script:Serial = 5
Offer 'pf-old' 'PF Old' '1.0.0' -disabled 'Old reason'; Publish 61; Refresh
$taken = PfWaitLog 'plugins: index read.*serial=5' 5
Check 'serial 5 takes PF Old down: it is not listed' ($taken -and (PluginRows) -notmatch 'PF Old') (PluginRows)
$script:Serial = 3; $script:Offer['pf-old'].disabled = ''; Publish 62; Refresh
$older = PfWaitLog 'an older index \(serial 3, the one kept has 5\)' 5
Check 'an index with serial 3 is not taken: PF Old stays unlisted' ($older -and (PluginRows) -notmatch 'PF Old') "rows=$(PluginRows) $older"
$script:Serial = 5; $script:Offer.Remove('pf-old'); $reads = LogCount 'plugins: index read.*serial=5'
Publish 63; Refresh
Check 'the same serial again is taken' ((LogCount 'plugins: index read.*serial=5') -gt $reads) "reads=$reads"

Case 'uninstall during a download: the download''s result is dropped'
$export = 'C:\pf\run\plugins-export.json'
Pf config export $export | Out-Null
$script:Offer.Remove('pf-sys')
Offer 'pf-test' 'PF Test' '2.1.0' @{ network = @('example.com') } -Slow; Publish 9
Refresh
$started = PfWaitLog 'plugins: download.*2\.1\.0' 5
# Its details are still open from the settings form.
Js "(() => { const r = [...document.querySelectorAll('#pluginList .pl-row')].find(r => r.textContent.startsWith('PF Test')); if (r.getAttribute('aria-expanded') !== 'true') r.click(); return true; })()" | Out-Null
Start-Sleep -Milliseconds 600
Js "[...document.querySelectorAll('#pluginList .pl-detail[data-plugin=pf-test] button')].find(b => b.textContent === 'Uninstall').click()" | Out-Null
$dlg = WaitWin '#32770' 5
Check 'a Yes / No question comes first' ($dlg -ne [IntPtr]::Zero -and $started) "started=$started"
[L]::PostMessageW($dlg, 0x111, [IntPtr]6, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 2
Check 'no plugin fence, no plugin folder, no data folder' (@(PluginFences pf-test).Count -eq 0 -and -not (Test-Path "$script:Config\plugins\pf-test") -and -not (Test-Path "$script:Config\plugins-data\pf-test")) ''
$late = PfWaitLog 'plugins: a cancelled download ended.*pf-test' 15; Start-Sleep -Seconds 1
Check 'the late download is dropped: nothing installed, no download folder left' ($late -and -not (Test-Path "$script:Config\plugins\pf-test") -and (Temps) -eq 0) "$late"

Case 'an imported plugin fence whose plugin is not installed'
$script:Offer['pf-test'].slow = $false; Publish 10
Pf config import $export | Out-Null; Start-Sleep -Seconds 3
$f = @(PluginFences pf-test)
Check 'the fences are back, nothing downloaded' ($f.Count -eq 2 -and -not (Test-Path "$script:Config\plugins\pf-test")) "fences=$($f.Count)"
Pf settings set tools.enabled true | Out-Null; Start-Sleep -Seconds 3
$r = (@(PluginFences pf-test))[0].windowRect
$t = OcrText $r.x $r.y $r.w $r.h 'not-installed'
Shot 'plugin-not-installed-fence'
Check 'with 小工具 on the fence says the plugin is not installed and how to install it' ($t -match "isn.t installed yet" -and $t -match 'Plugin.{1,2}details' -and -not (Test-Path "$script:Config\plugins-data\pf-test")) "ocr: $t"
$ok = ToolsPage; Refresh
InstallVia 'PF Test' | Out-Null
Start-Sleep -Seconds 3; $f = @(PluginFences pf-test)
Check 'installing it runs those fences, no third fence' ($f.Count -eq 2 -and $f[0].script.output -match '2\.1\.0') "fences=$($f.Count) output=$($f[0].script.output)"

Case 'a take-down in the saved index applies at start, offline'
PfQuit
$script:Offer['pf-test'].disabled = 'Saved reason'; Publish 11
Copy-Item "$Feed\index.json" "$script:Config\plugins\index.json" -Force
StopFeed
PfStart -Lang en; Start-Sleep -Seconds 3
$line = PfWaitLog 'plugins: taken down by the saved index' 5
$t = ''; $f = @(PluginFences pf-test)
if ($f.Count) { $r = $f[0].windowRect; $t = OcrText $r.x $r.y $r.w $r.h 'taken-down-offline' }
Check 'without the feed: main.js deleted, disabled.txt holds the reason, the fence says so' ($line -and -not (Test-Path "$script:Config\plugins\pf-test\2.1.0\main.js") -and (Get-Content "$script:Config\plugins\pf-test\disabled.txt") -eq 'Saved reason' -and $t -match 'Saved reason') "ocr: $t $line"
PfQuit
StopFeed
