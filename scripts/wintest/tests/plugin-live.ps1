# The live plugin store (the published index of DayuanJiang/PecoFence-plugins, branch index), not a
# test feed: after a change of the store's tools or indexes, every released PecoFence must still read
# it and install from it. PECOFENCE_PLUGINS_URL names the live index.json (a test instance moves its
# tools only with it set); a PecoFence that knows index-3.json reads the one beside it first.
#   Part old (any build, 0.5.0 too): a note and a clock of the old tools; the next start moves them to
#                the store's notes and clock plugins, downloaded from the store.
#   Part new (a debug build of 0.6.0 or later, CDP on the settings page): the same, then 设置 › 小工具
#                lists every store plugin, and Translator (light) and Emoji Picker (web) install
#                into fences of their own.
# Manual (needs the internet and the published index):
#   run.sh --box ec2-2 --bin <release or 0.5.0> --var Part=old tests/plugin-live.ps1
#   run.sh --box ec2-2 --bin <debug build> --var Part=new tests/plugin-live.ps1
if (-not $Part) { $Part = 'new' }
$Live = 'https://raw.githubusercontent.com/DayuanJiang/PecoFence-plugins/index/index.json'
function Installed($id) { $v = Get-Content "$script:Config\plugins\$id\current.txt" -ErrorAction SilentlyContinue; if ($v -and (Test-Path "$script:Config\plugins\$id\$v\plugin.json")) { return $v }; return $null }
function Wait($test, $sec = 6) { for ($i = 0; $i -lt $sec * 4; $i++) { $v = & $test; if ($v) { return $v }; Start-Sleep -Milliseconds 250 }; return $null }
function ByTitle($t) { return @(@(PfJson fence list) | Where-Object { $_ -and $_.title -eq $t }) | Select-Object -First 1 }

Case 'setup: a note and a clock of the old tools'
CloseWindows
$env:PECOFENCE_PLUGINS_URL = $Live
if ($PfDebugBuild) { $env:PECOFENCE_WEBVIEW_ARGS = '--remote-debugging-port=9352' }
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
Pf fence create --tool note --title Note --text 'live store note' --rect '80,80,280,200' | Out-Null
Pf fence create --tool clock --title Clock --rect '400,80,260,140' | Out-Null
Start-Sleep -Seconds 3
PfQuit

Case 'the next start reads the live index and moves the tools to the store plugins'
PfStart -Lang en
$read = PfWaitLog 'plugins: index read' 60
Check 'the live index is read' ([bool]$read) "$read"
$ok = Wait { (Installed 'notes') -and (Installed 'clock') } 90
Check 'notes and clock are installed from the store' ([bool]$ok) "notes=$(Installed 'notes') clock=$(Installed 'clock')"
$moved = Wait { "$((ByTitle 'Note').script.path)" -eq 'plugin:notes' -and "$((ByTitle 'Clock').script.path)" -eq 'plugin:clock' } 30
Check 'the note and the clock are plugin fences now' ([bool]$moved) "note=$((ByTitle 'Note').script.path) clock=$((ByTitle 'Clock').script.path)"
Check 'the note keeps its text' ((Get-Content "$script:Config\config.json" -Raw -Encoding UTF8) -match 'live store note') ''
Shot 'live-migrated'

if ($Part -eq 'new') {
  Case 'the store list and two new plugins (debug build)'
  PfSettings | Out-Null
  PageClick 'nav a[data-page=tools]'; Start-Sleep -Seconds 4
  $rows = [string](Js "[...document.querySelectorAll('#pluginList .pl-row')].map(r => r.textContent).join(' / ')")
  Check 'the list shows the store plugins (Translator and Emoji Picker among them)' ($rows -match 'Translator' -and $rows -match 'Emoji Picker' -and $rows -match 'Weather') ($rows.Substring(0, [Math]::Min(400, $rows.Length)))
  foreach ($p in @(@{ name = 'Translator'; id = 'translator' }, @{ name = 'Emoji Picker'; id = 'emoji-picker' })) {
    $i = [int](Js "[...document.querySelectorAll('#pluginList .pl-row')].findIndex(r => r.textContent.startsWith('$($p.name)'))")
    PageClick '#pluginList .pl-row button' $i; Start-Sleep -Milliseconds 800
    $warn = [string](Js "document.getElementById('plBack').hidden ? '' : document.getElementById('plBack').innerText")
    Shot "live-warning-$($p.id)"
    PageClick '#plOk'
    $got = Wait { Installed $p.id } 60
    Check "$($p.name) installs from the store (its warning showed)" ([bool]$got -and $warn -ne '') "version=$(Installed $p.id) warning=$($warn.Substring(0, [Math]::Min(120, $warn.Length)) -replace "`n", ' ')"
    $fence = Wait { @(@(PfJson fence list) | Where-Object { $_ -and $_.script -and $_.script.path -eq "plugin:$($p.id)" }).Count -ge 1 } 20
    Check "$($p.name) runs in a fence of its own" ([bool]$fence) ''
  }
  Shot 'live-installed'
}
PfQuit
