# 插件 in the Store edition (MSIX): the same steps on the test-signed package and on the portable
# exe in one guest. A local test index (PECOFENCE_PLUGINS_URL, a named instance) with a light
# plugin that writes and reads its data and a web plugin whose page asks PecoFence for the time;
# the install warning; both fences; where the plugin's files, data and web profile land; a restart;
# an update by itself; a take-down; an uninstall (folder and profile gone); a note and a clock of
# the old tools with the index unreachable.
# Manual (a Hyper-V guest, the debug build packed and signed first):
#   run-hyperv.sh --vm PF-Win11-3 --bin <debug bin> --extra <dir> --prep <dir>/prep.ps1 tests/msix-plugins.ps1
# <dir> holds sdk\ (makeappx, signtool and their dlls, pecofence-test-signing.pfx/.cer, the PC's
# Windows SDK and C:\Users\me\Documents\pf-task-src\dist), assets\ (Assets and *.pri of a Store
# package), AppxManifest.xml (packaging/msix with the identity filled, no startup task) and
# prep.ps1 (trusts the certificate, packs C:\pf\bin, signs it as C:\pf\msix\pf.msix).
$Feed = 'C:\pf\run\pfeed'
$Inst = 'msix'
$script:Mode = 'pack'
$script:Offer = [ordered]@{}
$script:Serial = 0
$LightJs = "pf.on('start', async () => {`r`n  await pf.data.write('msix-test.txt', 'saved by VERSION');`r`n  const back = await pf.data.read('msix-test.txt');`r`n  pf.show({ rows: [{ text: 'pf-test VERSION data=' + back }] });`r`n});`r`n"
$WebJs = "(async () => { let t = 'none'; try { t = String(await window.pf.time.now()); } catch (e) { t = 'error ' + e.message; } document.body.insertAdjacentHTML('beforeend', '<p id=t>time ' + t + '</p>'); })();"
function Offer($id, $name, $version, [string]$js = '', [string]$disabled = '', [switch]$Web) {
  $script:Offer[$id] = @{ name = $name; version = $version; js = $js; disabled = $disabled; web = [bool]$Web }
}
function Publish([int]$n) {
  $commit = ('{0:x}' -f $n).PadLeft(40, 'a')
  $entries = foreach ($id in $script:Offer.Keys) {
    $p = $script:Offer[$id]
    $dir = "$Feed\$commit\plugins\$id"; New-Item -ItemType Directory -Force $dir | Out-Null
    $layer = if ($p.web) { 'web' } else { 'light' }
    $m = [ordered]@{ id = $id; version = $p.version; api = 1; layer = $layer; name = @{ en = $p.name }; description = @{ en = 'A test plugin' }
      author = 'tester'; minPecoFence = '0.4.0'; permissions = @{}; settings = @() }
    [IO.File]::WriteAllText("$dir\plugin.json", ($m | ConvertTo-Json -Depth 6))
    $js = $p.js -replace 'VERSION', $p.version
    if ($p.web) {
      [IO.File]::WriteAllText("$dir\app.js", $js)
      [IO.File]::WriteAllText("$dir\index.html", "<!doctype html><meta charset=utf-8><title>$id</title><p>$id $($p.version)</p><script src=app.js></script>")
    }
    else { [IO.File]::WriteAllText("$dir\main.js", $js) }
    [IO.File]::WriteAllText("$dir\README.md", "# $id`r`n")
    $m.files = @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
    if ($p.disabled) { $m.disabled = $p.disabled }
    $m
  }
  [IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; serial = $script:Serial; commit = $commit; plugins = @($entries) } | ConvertTo-Json -Depth 8))
  Log "published $(($script:Offer.Keys | ForEach-Object { "$_ $($script:Offer[$_].version)" }) -join ', ') at $commit"
}
function RowButton($name) { return (Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$name'))") }
function Refresh() { Js "window.chrome.webview.postMessage({ type: 'action', name: 'pluginsRefresh' })" | Out-Null; Start-Sleep -Seconds 3 }
function InstallVia($name) {
  $i = [int](RowButton $name)
  PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 600
  $d = Js "(() => ({ shown: !document.getElementById('plBack').hidden, text: document.getElementById('plBack').innerText }))()"
  Shot "$script:Mode-warning-$name"
  PageClick '#plOk'
  Start-Sleep -Seconds 4
  return $d
}
function UninstallVia($name, $id) {
  Js "(() => { const r = [...document.querySelectorAll('#pluginList .pl-row')].find(r => r.textContent.startsWith('$name')); if (r.getAttribute('aria-expanded') !== 'true') r.click(); return true; })()" | Out-Null
  Start-Sleep -Milliseconds 600
  Js "[...document.querySelectorAll('#pluginList .pl-detail[data-plugin=$id] button')].find(b => b.textContent === 'Uninstall').click()" | Out-Null
  Start-Sleep -Milliseconds 400
  Js "document.getElementById('plOk').click()" | Out-Null; Start-Sleep -Seconds 3
}
function ToolsPage() { $ok = PfSettings; PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 3; return $ok }

# ---- the packaged app: started through the Start menu entry's activation, its environment from
# HKCU\Environment, its files under the package's LocalCache when no real folder is there.
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
[ComImport, Guid("2e941141-7f97-4756-ba1d-9decde894a3d"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
public interface IAppActivation { [PreserveSig] int ActivateApplication([MarshalAs(UnmanagedType.LPWStr)] string id, [MarshalAs(UnmanagedType.LPWStr)] string args, int options, out uint pid); }
[ComImport, Guid("45BA127D-10A8-46EA-8AB7-56EA9078943C")] public class AppActivation {}
public static class Act { public static uint Start(string id) { var m = (IAppActivation)new AppActivation(); uint pid; int hr = m.ActivateApplication(id, "", 0, out pid); if (hr != 0) throw new Exception("activation hr 0x" + hr.ToString("x8")); return pid; } }
'@
$script:Pfn = ''
$script:Lc = ''
function UserEnv($name, $value) { [Environment]::SetEnvironmentVariable($name, $value, 'User'); Set-Item "Env:\$name" $value }
function PfLogPath() {
  if ($script:Mode -eq 'pack') {
    foreach ($p in "$script:Lc\Local\PecoFence\pecofence.$Inst.log", "$env:LOCALAPPDATA\PecoFence\pecofence.$Inst.log") { if (Test-Path $p) { return $p } }
    return "$script:Lc\Local\PecoFence\pecofence.$Inst.log"
  }
  return "$env:LOCALAPPDATA\PecoFence\pecofence.$PfInst.log"
}
function Start-App() {
  if ($script:Mode -eq 'pack') {
    if (PfAlive) { PfQuit }
    if (-not (Test-Path "$PfApp\pecofence-cli.exe")) { New-Item -ItemType Directory -Force $PfApp | Out-Null; Copy-Item "$PfBin\*" $PfApp -Force }
    $env:PECOFENCE_INSTANCE = $Inst
    $id = [Act]::Start("$script:Pfn!PecoFence")
    $h = WaitWin 'PecoFence.Control' 30 -Hidden
    Log "packaged start pid=$id control=$h"
    Start-Sleep -Seconds 4
    Pf settings set language en | Out-Null
  }
  else { PfStart -Lang en }
}
# Where the app's own files are on disk: the package's private copy of %APPDATA% or the real one.
function DataRoot() { if ($script:Mode -eq 'pack') { return "$script:Lc\Roaming\PecoFence" } else { return Split-Path (PfJson status).configPath } }
function Installed($id) { $c = DataRoot; $v = Get-Content "$c\plugins\$id\current.txt" -ErrorAction SilentlyContinue; if ($v -and (Test-Path "$c\plugins\$id\$v\plugin.json")) { return $v }; return $null }
function WebProfile($id) {
  $roots = if ($script:Mode -eq 'pack') { @("$script:Lc\Local\PecoFence\WebView2Profiles", "$env:LOCALAPPDATA\PecoFence\WebView2Profiles") } else { @("$env:LOCALAPPDATA\PecoFence\WebView2Profiles") }
  return @(foreach ($r in $roots) { Get-ChildItem $r -Directory -Recurse -Filter "WV2Profile_plugin-$id" -ErrorAction SilentlyContinue | ForEach-Object FullName })
}

function RunAll() {
  $m = $script:Mode
  Case "${m}: setup"
  $script:Offer = [ordered]@{}
  Offer 'pf-test' 'PF Test' '1.0.0' $LightJs
  Offer 'pf-web' 'PF Web' '1.0.0' $WebJs -Web
  Publish 1
  if ($m -eq 'pack') {
    UserEnv 'PECOFENCE_INSTANCE' $Inst
    UserEnv 'PECOFENCE_PLUGINS_URL' 'http://127.0.0.1:8731/index.json'
    UserEnv 'PECOFENCE_WEBVIEW_ARGS' '--remote-debugging-port=9352'
    UserEnv 'RUST_LOG' 'pecofence=debug'
  }
  else {
    foreach ($n in 'PECOFENCE_INSTANCE', 'PECOFENCE_PLUGINS_URL', 'PECOFENCE_WEBVIEW_ARGS', 'RUST_LOG') { [Environment]::SetEnvironmentVariable($n, $null, 'User') }
    $env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8731/index.json'; $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352'
  }
  Start-App
  $st = PfJson status
  $proc = (Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1).Path
  Log "status: $($st | ConvertTo-Json -Compress -Depth 3)"
  $loaded = PfWaitLog 'config loaded' 5
  $packed = $proc -like '*\WindowsApps\*'
  Check "${m}: the app runs$(if ($m -eq 'pack') { ' from the package, config in the store package' } else { ' as the portable exe' })" ($(if ($m -eq 'pack') { $packed -and $loaded -match 'storage="?store package' } else { -not $packed -and [bool]$st })) "exe=$proc $loaded"
  $ok = ToolsPage
  $rows = PluginRows
  Shot "$m-tools-page"
  Check "${m}: 设置 › 小工具 lists both plugins from the test index" ($ok -and $rows -match 'PF Test.*Install' -and $rows -match 'PF Web.*Install') $rows

  Case "${m}: install a light and a web plugin"
  $d = InstallVia 'PF Test'
  Check "${m}: the warning shows before the install" ($d.shown -and $d.text -match 'It will:' -and $d.text -match 'A plugin can only do what is listed here') "$($d.text)"
  $line = PfWaitLog 'plugins: fence added' 15
  Start-Sleep -Seconds 3
  $f = @(PluginFences pf-test)
  Check "${m}: the light plugin's fence runs and its data is written and read back" ((Installed 'pf-test') -eq '1.0.0' -and $f.Count -eq 1 -and "$($f[0].script.output)" -match 'pf-test 1\.0\.0 data=saved by 1\.0\.0') "fences=$($f.Count) out=$($f[0].script.output) $line"
  $root = DataRoot
  $file = @(Get-ChildItem $root -Recurse -File -Filter 'msix-test.txt' -ErrorAction SilentlyContinue | ForEach-Object FullName)
  $real = @(Get-ChildItem "$env:APPDATA\PecoFence" -Recurse -File -Filter 'msix-test.txt' -ErrorAction SilentlyContinue)
  Check "${m}: the plugin and its data are in the app's own folder$(if ($m -eq 'pack') { ' (the package LocalCache, nothing in the real AppData)' })" ((Test-Path "$root\plugins\pf-test\1.0.0\main.js") -and $file.Count -eq 1 -and $(if ($m -eq 'pack') { $real.Count -eq 0 } else { $true })) "data=$($file -join ', ') real=$($real.Count) root=$root"
  $d = InstallVia 'PF Web'
  $wl = PfWaitLog 'web plugin: page loading plugin=pf-web' 30
  Start-Sleep -Seconds 4
  $text = $null
  if (CdpOpen -Url 'pecofence-plugin://pf-web/*' -sec 20) { $text = JsWait "document.getElementById('t') && document.body.innerText" 15; $url = Js 'location.href' }
  $wf = @(PluginFences pf-web)
  Shot "$m-both-fences"
  Check "${m}: the web plugin's page loads from pecofence-plugin://pf-web/ and pf.time.now answers" ($wf.Count -eq 1 -and $url -like 'pecofence-plugin://pf-web/*' -and $text -match 'pf-web 1\.0\.0' -and $text -match 'time \d{10,}') "url=$url text=$($text -replace "`n", ' / ') $wl"
  $prof = @(WebProfile 'pf-web')
  Check "${m}: the web plugin has its own profile$(if ($m -eq 'pack') { ' under the package LocalCache' })" ($prof.Count -eq 1 -and $(if ($m -eq 'pack') { $prof[0] -like "$script:Lc*" } else { $true })) ($prof -join ', ')
  CdpOpen | Out-Null

  Case "${m}: a restart keeps both"
  PfQuit
  Start-App
  Start-Sleep -Seconds 6
  $f = @(PluginFences pf-test); $wf = @(PluginFences pf-web)
  $text = $null
  if (CdpOpen -Url 'pecofence-plugin://pf-web/*' -sec 20) { $text = JsWait "document.getElementById('t') && document.body.innerText" 15 }
  Check "${m}: after a restart both fences run again, the data is still there" ($f.Count -eq 1 -and "$($f[0].script.output)" -match 'data=saved by 1\.0\.0' -and $wf.Count -eq 1 -and $text -match 'time \d{10,}') "out=$($f[0].script.output) web=$($text -replace "`n", ' / ')"

  Case "${m}: update, take-down, uninstall"
  $ok = ToolsPage
  Offer 'pf-test' 'PF Test' '1.0.1' $LightJs; Publish 2; Refresh
  for ($i = 0; $i -lt 30 -and (Installed 'pf-test') -ne '1.0.1'; $i++) { Start-Sleep -Seconds 1 }
  Start-Sleep -Seconds 3
  $f = @(PluginFences pf-test)
  Check "${m}: an update that asks for nothing more installs by itself and runs" ((Installed 'pf-test') -eq '1.0.1' -and "$($f[0].script.output)" -match 'pf-test 1\.0\.1') "installed=$(Installed 'pf-test') out=$($f[0].script.output)"
  $script:Offer['pf-test'].disabled = 'Test reason'; Publish 3; Refresh
  Start-Sleep -Seconds 4
  $left = @(Get-ChildItem "$(DataRoot)\plugins\pf-test" -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object Name) -join ' '
  Check "${m}: a take-down stops it and deletes its code, the reason is kept" ($left -match 'disabled\.txt' -and $left -notmatch 'main\.js') "left=$left"
  UninstallVia 'PF Web' 'pf-web'
  Start-Sleep -Seconds 3
  $prof = @(WebProfile 'pf-web')
  # The browser of the settings page still holds the profile: it is deleted before the next
  # browser starts (web_plugin.rs delete_profile), so look again after a restart.
  $kept = PfWaitLog 'web plugin: (profile deleted|no profile to delete|profile in use, deleted before the browser starts next).*pf-web' 5
  Check "${m}: an uninstall removes the web plugin's folder and fence, its profile now or before the next browser" (-not (Test-Path "$(DataRoot)\plugins\pf-web") -and @(PluginFences pf-web).Count -eq 0 -and ($prof.Count -eq 0 -or $kept -match 'kept=true')) "profile=$($prof -join ', ') $kept"
  Shot "$m-after-uninstall"
  PfQuit
  Start-App
  $ok = ToolsPage
  Start-Sleep -Seconds 3
  $prof = @(WebProfile 'pf-web')
  Check "${m}: after the restart the uninstalled web plugin's profile is gone" ($prof.Count -eq 0) "profile=$($prof -join ', ')"

  Case "${m}: the old tools with the index unreachable"
  if ($m -eq 'pack') { UserEnv 'PECOFENCE_PLUGINS_URL' 'http://127.0.0.1:8739/index.json' } else { $env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8739/index.json' }
  Pf fence create --tool note --title Note --text 'msix note' --rect '80,420,280,200' | Out-Null
  Pf fence create --tool clock --title Clock --rect '400,420,260,140' | Out-Null
  PfQuit
  Start-App
  $line = PfWaitLog 'plugins: index not read' 25
  Start-Sleep -Seconds 4
  $all = @(PfJson fence list)
  $note = @($all | Where-Object { $_.title -eq 'Note' }) | Select-Object -First 1
  $clock = @($all | Where-Object { $_.title -eq 'Clock' }) | Select-Object -First 1
  Shot "$m-old-tools"
  Check "${m}: a note and a clock still show their text and time with the store unreachable" ([bool]$line -and $note.noteText -eq 'msix note' -and $clock -and @(@([L]::Tops('pecofence') -split "`n") | Where-Object { $_ -match '\| Clock \|' }).Count -eq 1) "note=$($note.noteText) clock=$($clock.tool) $line"
  PfQuit
  # The packaged app's log is under the package: keep it with the job's logs.
  if ($m -eq 'pack') { New-Item -ItemType Directory -Force "$PfRun\logs" | Out-Null; Get-ChildItem "$script:Lc\Local\PecoFence\pecofence.$Inst*.log" -ErrorAction SilentlyContinue | ForEach-Object { Copy-Item $_.FullName "$PfRun\logs\app-pack-$($_.Name)" } }
}

Case 'setup: the test-signed package'
CloseWindows
New-Item -ItemType Directory -Force $Feed | Out-Null
$script:FeedProc = PfFeed 8731 -log 'C:\pf\run\pfeed.log'
# A real %APPDATA%\PecoFence left by earlier jobs would be used by the package instead of its own copy.
Remove-Item -Recurse -Force "$env:APPDATA\PecoFence", "$env:LOCALAPPDATA\PecoFence" -ErrorAction SilentlyContinue
$add = try { Add-AppxPackage C:\pf\msix\pf.msix -ErrorAction Stop; 'ok' } catch { "failed: $($_.Exception.Message)" }
$pkg = Get-AppxPackage DayuanJiang.PecoFence
$script:Pfn = $pkg.PackageFamilyName
$script:Lc = "$env:LOCALAPPDATA\Packages\$script:Pfn\LocalCache"
Check 'the test-signed package installs' ($add -eq 'ok' -and $pkg) "add=$add pfn=$script:Pfn at $($pkg.InstallLocation)"

$script:Mode = 'pack'
RunAll
$script:Mode = 'port'
RunAll

Case 'cleanup'
foreach ($n in 'PECOFENCE_INSTANCE', 'PECOFENCE_PLUGINS_URL', 'PECOFENCE_WEBVIEW_ARGS', 'RUST_LOG') { [Environment]::SetEnvironmentVariable($n, $null, 'User') }
Get-AppxPackage DayuanJiang.PecoFence | Remove-AppxPackage -ErrorAction SilentlyContinue
Stop-Process -Id $script:FeedProc.Id -Force -ErrorAction SilentlyContinue
Check 'cleanup' $true ''
