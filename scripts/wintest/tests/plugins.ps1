# 社区插件 against a local plugin index (PECOFENCE_PLUGINS_URL on a named instance): the warning
# before an install, the plugin fence and its data folder, a file whose SHA-256 is not the
# index's (refused, nothing left), an update that asks for nothing more (by itself) and one that
# asks for more (waits for the OK), a plugin taken down, the uninstall, and an imported fence
# whose plugin is not installed. English UI. Debug build (CDP on the settings page).
$Feed = 'C:\pf\run\pfeed'
# Writes plugin pf-test (and pf-bad, whose main.js differs from its hash) at a made-up commit
# `n` with `version`, the network hosts `net`, and the index naming that commit.
function Publish([int]$n, [string]$version, [string[]]$net = @(), [string]$disabled = '') {
  $commit = ('{0:x}' -f $n).PadLeft(40, 'a')
  $entries = foreach ($id in 'pf-test', 'pf-bad') {
    $dir = "$Feed\$commit\plugins\$id"; New-Item -ItemType Directory -Force $dir | Out-Null
    $m = [ordered]@{ id = $id; version = $version; name = @{ en = $(if ($id -eq 'pf-test') { 'PF Test' } else { 'PF Bad' }) }; description = @{ en = 'A test plugin' }
      author = 'tester'; minPecoFence = '0.4.0'; streamable = $false; permissions = [ordered]@{ network = @($net); clipboard = 'none'; files = 'data'; run = $false } }
    [IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 5))
    [IO.File]::WriteAllText("$dir\main.js", "pf.on('start', () => pf.show({ rows: [{ text: '$id $version' }] }));`r`n")
    [IO.File]::WriteAllText("$dir\README.md", "# $id`r`n")
    $files = foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } }
    if ($id -eq 'pf-bad') { Add-Content "$dir\main.js" '// changed after the review' }
    $m.files = @($files)
    if ($disabled -and $id -eq 'pf-test') { $m.disabled = $disabled }
    $m
  }
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; commit = $commit; plugins = @($entries) } | ConvertTo-Json -Depth 8))
  Log "published $version at $commit"
}
function Rows() { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].map(r => r.textContent).join(' / ')") }
function RowButton($name) { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$name'))") }
# The page asks for the index again (as when 设置 › 小工具 opens).
function Refresh() { Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null; Start-Sleep -Seconds 3 }
function Installed() { $v = Get-Content "$script:Config\plugins\pf-test\current.txt" -ErrorAction SilentlyContinue; if ($v -and (Test-Path "$script:Config\plugins\pf-test\$v\plugin.json")) { return $v }; return $null }
function PluginFences() { return @(@((PfJson fence list)) | Where-Object { $_ -and $_.script -and $_.script.path -eq 'plugin:pf-test' }) }
# Clicks 安装 / 更新 in the row of `name`, then the warning's OK; the warning's texts.
function InstallVia($name) {
  $i = [int](RowButton $name)
  PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 600
  $d = Js "(() => ({ shown: !document.getElementById('plBack').hidden, text: document.getElementById('plBack').textContent }))()"
  Shot "plugin-warning-$name"
  PageClick '#plOk'; Start-Sleep -Seconds 4
  return $d
}

Case 'setup'
CloseWindows
New-Item -ItemType Directory -Force $Feed | Out-Null
$server = @'
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), 8731); $l.Start()
while ($true) { $c = $l.AcceptTcpClient(); try { $s = $c.GetStream(); $r = New-Object IO.StreamReader($s); $line = $r.ReadLine(); while ($r.ReadLine()) {}
  $path = 'C:\pf\run\pfeed' + (($line -split ' ')[1] -replace '/', '\'); Add-Content 'C:\pf\run\pfeed.log' "$line"
  if (Test-Path -LiteralPath $path -PathType Leaf) { $b = [IO.File]::ReadAllBytes($path); $h = "HTTP/1.1 200 OK`r`nContent-Length: $($b.Length)`r`nConnection: close`r`n`r`n" } else { $b = @(); $h = "HTTP/1.1 404 Not Found`r`nContent-Length: 0`r`nConnection: close`r`n`r`n" }
  $hb = [Text.Encoding]::ASCII.GetBytes($h); $s.Write($hb, 0, $hb.Length); if ($b.Length) { $s.Write($b, 0, $b.Length) }; $s.Flush() } catch {} finally { $c.Close() } }
'@
Set-Content 'C:\pf\run\pfeed-serve.ps1' $server
$feedProc = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'C:\pf\run\pfeed-serve.ps1' -WindowStyle Hidden -PassThru
Publish 1 '1.0.0'
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8731/index.json'
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
$ok = PfSettings
PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3
Check 'the feed runs, the settings page is reachable' ($feedProc -and -not $feedProc.HasExited -and $ok) "config=$script:Config"
Check 'the tools page lists both plugins with 安装' ((Rows) -match '^PF Test.*Install / PF Bad.*Install$') (Rows)

Case 'install: the warning first, then the plugin fence runs with its data folder'
$d = InstallVia 'PF Test'
Check 'the warning shows the author, the version and that a plugin can do anything' ($d.shown -and $d.text -match 'By tester\. The reviewed version is 1\.0\.0' -and $d.text -match 'can do anything your Windows account can') $d.text
$line = PfWaitLog 'plugins: fence added' 15
$f = @(PluginFences)
Shot 'plugin-fence-v1'
Check 'installed into plugins\pf-test, one fence titled after the plugin' ((Installed) -eq '1.0.0' -and $f.Count -eq 1 -and $f[0].title -eq 'PF Test' -and $line) "fences=$($f.Count) $line"
Start-Sleep -Seconds 3; $f = @(PluginFences)
Check 'the fence runs main.js' ($f[0].script.output -match 'pf-test 1\.0\.0') "output=$($f[0].script.output)"
Check 'the row now offers nothing to install' ((Rows) -match '^PF Test.*On the desktop') (Rows)

Case 'a file whose SHA-256 is not the index''s is refused'
$d = InstallVia 'PF Bad'
$toast = PageToast
Check 'nothing of pf-bad is left, the page says why' (-not (Test-Path "$script:Config\plugins\pf-bad") -and -not (Test-Path "$script:Config\plugins\.new-pf-bad") -and $toast.text -match 'differ from the reviewed') ($toast | ConvertTo-Json -Compress)
Check 'the log names the file' ([bool](PfWaitLog 'plugins: install or update failed.*refused=true.*main\.js: size or SHA-256' 5)) ''

Case 'an update that asks for nothing more installs by itself'
Publish 2 '1.1.0'
Refresh; PfWaitLog 'plugins: installed, its fences start again' 15 | Out-Null; Start-Sleep -Seconds 3
$f = @(PluginFences)
Check 'pf-test is 1.1.0, the same fence runs the new script' ((Installed) -eq '1.1.0' -and $f.Count -eq 1 -and $f[0].script.output -match '1\.1\.0') "installed=$(Installed) output=$($f[0].script.output)"

Case 'an update that asks for more waits for the OK'
Publish 3 '2.0.0' @('example.com')
Refresh
Check 'not installed by itself, the row asks' ((Installed) -eq '1.1.0' -and (Rows) -match 'PF Test.*Network.*An update waits for your OK') (Rows)
$d = InstallVia 'PF Test'
Check 'the warning lists the new host' ($d.shown -and $d.text -match 'Network \(example\.com\)') $d.text
Start-Sleep -Seconds 2
Check 'after the OK it is 2.0.0 with one fence' ((Installed) -eq '2.0.0' -and @(PluginFences).Count -eq 1) "installed=$(Installed)"

Case 'a plugin taken down stops and says why'
Publish 4 '2.0.0' @('example.com') 'Test reason'
Refresh
Shot 'plugin-taken-down'
Check 'main.js deleted, disabled.txt holds the reason' (-not (Test-Path "$script:Config\plugins\pf-test\2.0.0\main.js") -and (Get-Content "$script:Config\plugins\pf-test\disabled.txt") -eq 'Test reason') ''
Check 'the row says so and offers 卸载' ((Rows) -match 'PF Test.*Taken down: Test reason.*Uninstall') (Rows)

Case 'uninstall asks, then removes the fence, the plugin and its data'
$export = 'C:\pf\run\plugins-export.json'
Pf config export $export | Out-Null
PageClick '#pluginList .pl-row button' ([int](RowButton 'PF Test'))
$dlg = WaitWin '#32770' 5
Check 'a Yes / No question comes first' ($dlg -ne [IntPtr]::Zero) ''
[L]::PostMessageW($dlg, 0x111, [IntPtr]6, [IntPtr]::Zero) | Out-Null; Start-Sleep -Seconds 3
Check 'no plugin fence, no plugin folder, no data folder' (@(PluginFences).Count -eq 0 -and -not (Test-Path "$script:Config\plugins\pf-test") -and -not (Test-Path "$script:Config\plugins-data\pf-test")) ''

Case 'an imported plugin fence whose plugin is not installed'
Publish 5 '2.0.0'
Pf config import $export | Out-Null; Start-Sleep -Seconds 3
$f = @(PluginFences)
Check 'the fence is back, nothing downloaded' ($f.Count -eq 1 -and -not (Test-Path "$script:Config\plugins\pf-test")) "fences=$($f.Count)"
Pf settings set tools.enabled true | Out-Null; Start-Sleep -Seconds 3
$r = (@(PluginFences))[0].windowRect
$t = OcrText $r.x $r.y $r.w $r.h 'not-installed'
Shot 'plugin-not-installed-fence'
Check 'with 小工具 on the fence says the plugin is not installed and how to install it' ($t -match "isn.t installed yet" -and $t -match 'Plugin.{1,2}details' -and -not (Test-Path "$script:Config\plugins-data\pf-test")) "ocr: $t"
Refresh
InstallVia 'PF Test' | Out-Null
Start-Sleep -Seconds 3; $f = @(PluginFences)
Check 'installing it runs that fence, no second fence' ($f.Count -eq 1 -and $f[0].script.output -match '2\.0\.0') "fences=$($f.Count) output=$($f[0].script.output)"
PfQuit
if ($feedProc) { Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue }
