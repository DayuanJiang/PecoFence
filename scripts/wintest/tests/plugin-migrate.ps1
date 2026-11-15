# 升级为插件: an old configuration with a note (to-dos), a clock with seconds, an unchanged weather
# example, an unchanged PC status example of 0.3.0, an edited GitHub example and a script of the
# user's own. With the plugin index unreachable nothing changes and all still work; with a local
# test index holding stand-ins of notes, clock, weather, github and pc-status
# (lib.ps1's PfStandIn) the four move to their plugins once installed, the note keeps its
# text, the edited copy and the own script stay; a restart moves nothing more; an import of the
# old configuration moves them again; a backup restore gives the running plugins its settings
# and starts the plugin a fence names now; an import keeps a plugin fence's settings without the
# folders picked elsewhere and brings a new note text; the snapshot 升级为插件之前 brings the old
# fences back, and the note keeps the text it has by then.
$Feed = 'C:\pf\run\pfeed'; $Assets = 'C:\pf\kit\assets'; $Ex = 'C:\pf\run\ex'
$Titles = 'Note', 'Clock', 'Weather', 'Status', 'GitHub', 'Mine'
function ByTitle($t) { return (@(@((PfJson fence list)) | Where-Object { $_ -and $_.title -eq $t }) | Select-Object -First 1) }
# What each fence is: its tool, or the script or plugin it runs.
function State() { return ($Titles | ForEach-Object { $f = ByTitle $_; $p = "$($f.script.path)"; "$_=" + $(if ($f.tool -ne 'script') { $f.tool } elseif ($p -like 'plugin:*') { $p } else { [IO.Path]::GetFileName($p) }) }) -join ' ' }
function Now() { Pf config export "$PfRun\now.json" | Out-Null; return (Get-Content "$PfRun\now.json" -Raw -Encoding UTF8 | ConvertFrom-Json) }
function Moved() { return @(PfLogAll | Where-Object { $_ -match 'upgrade to plugins: fence moved.*plugin="' } | Sort-Object -Unique).Count }
function Before() { return ,@(Snaps | Where-Object { $_.name -eq 'Before the switch to plugins' }) }
$Old = 'Note=note Clock=clock Weather=weather.ps1 Status=status.ps1 GitHub=github.ps1 Mine=mine.ps1'
$New = 'Note=plugin:notes Clock=plugin:clock Weather=plugin:weather Status=plugin:pc-status GitHub=github.ps1 Mine=mine.ps1'
$Text = "[ ] milk`n[x] rent`nplain line"

Case 'setup: the tools of an earlier version'
CloseWindows
New-Item -ItemType Directory -Force $Ex, $Feed | Out-Null
# As PecoFence wrote an example: a BOM and CRLF.
function Written($from, $to, $more = '') {
  $t = (Get-Content "$Assets\old-examples\$from" -Raw -Encoding UTF8) + $more
  [IO.File]::WriteAllText("$Ex\$to", ($t -replace "`r?`n", "`r`n"), (New-Object Text.UTF8Encoding $true))
}
Written 'weather.en.0.4.0.ps1' 'weather.ps1'
Written 'status.en.0.3.0.ps1' 'status.ps1'
Written 'github.en.0.4.0.ps1' 'github.ps1' "# my repository`n"
Set-Content "$Ex\mine.ps1" 'Write-Output "mine runs"' -Encoding UTF8
# No test index yet: a test instance moves nothing then.
Remove-Item Env:\PECOFENCE_PLUGINS_URL -ErrorAction SilentlyContinue
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
Pf fence create --tool note --title Note --text $Text --rect '80,80,280,200' | Out-Null
Pf fence create --tool clock --title Clock --rect '400,80,260,140' | Out-Null
Pf settings set tools.clockSeconds true | Out-Null
$i = 0
foreach ($t in 'Weather', 'Status', 'GitHub', 'Mine') { Pf fence create --script "$Ex\$($t.ToLower()).ps1" --title $t --rect "$(80 + 320 * $i),400,300,220" | Out-Null; $i++ }
Start-Sleep -Seconds 4
Check 'the old tools are there' ((State) -eq $Old -and (ByTitle 'Note').noteText -eq $Text) "$(State) note=$((ByTitle 'Note').noteText -replace "`n", ' / ')"
Pf config export "$PfRun\old.json" | Out-Null
Shot 'm01-old-tools'
PfQuit

Case 'index unreachable: nothing changes, all still work'
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8739/index.json'
PfStart -Lang en
$line = PfWaitLog 'plugins: index not read' 25
Start-Sleep -Seconds 4
Check 'the index was asked for and not read' ([bool]$line -and [bool](PfWaitLog 'upgrade to plugins: tool fences of an earlier version.*missing' 1)) "$line"
Check 'every fence as it was' ((State) -eq $Old) (State)
Check 'the note shows its text, the clock its time' ((ByTitle 'Note').noteText -eq $Text -and @(@([L]::Tops('pecofence') -split "`n") | Where-Object { $_ -match '\| Clock \|' }).Count -eq 1) ''
$st = (ByTitle 'Status').script; $mine = (ByTitle 'Mine').script
Check 'the PC status example and the own script run' ("$($st.output)" -match 'CPU' -and "$($mine.output)" -match 'mine runs') "status=$((("$($st.output)") -split "`n")[0]) mine=$($mine.output)"
Check 'no plugin installed, no snapshot taken' (-not (Test-Path "$script:Config\plugins\notes") -and (Before).Count -eq 0) ''
PfQuit

Case 'index reachable: the four move once their plugins are installed'
$commit = 'e19'.PadLeft(40, 'a')
$entries = foreach ($id in 'notes', 'clock', 'weather', 'github', 'pc-status') {
  $dir = "$Feed\$commit\plugins\$id"; New-Item -ItemType Directory -Force $dir | Out-Null
  Copy-Item "$(PfStandIn $id)\*" $dir
  # Here they show their settings again when they change.
  [IO.File]::WriteAllText("$dir\main.js", "const show = () => pf.show({ rows: [{ text: `"$id stand-in, settings `" + JSON.stringify(pf.settings) }] });`npf.on(`"start`", show);`npf.on(`"settings`", show);`n")
  $m = Get-Content "$dir\plugin.json" -Raw -Encoding UTF8 | ConvertFrom-Json
  $m | Add-Member files @(foreach ($f in Get-ChildItem $dir -File) { [ordered]@{ name = $f.Name; size = $f.Length; sha256 = (Get-FileHash $f.FullName -Algorithm SHA256).Hash.ToLower() } })
  $m
}
[IO.File]::WriteAllText("$Feed\index.json", ([ordered]@{ format = 1; commit = $commit; plugins = @($entries) } | ConvertTo-Json -Depth 8))
$feedProc = PfFeed 8733
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8733/index.json'
PfStart -Lang en
for ($w = 0; $w -lt 60 -and (Moved) -lt 4; $w++) { Start-Sleep -Milliseconds 500 }
Start-Sleep -Seconds 3
$c = Now
$note = (ByTitle 'Note').id; $clock = (ByTitle 'Clock').id
Check 'the note, the clock, the weather and the PC status are plugin fences; the edited copy and the own script stay' ((State) -eq $New) (State)
Check 'four moves logged, the four plugins installed (github is not needed)' ((Moved) -eq 4 -and @(Get-ChildItem "$script:Config\plugins\*\current.txt").Count -eq 4 -and -not (Test-Path "$script:Config\plugins\github")) "moved=$(Moved)"
Check 'the note keeps its text' ($c.notes.$note -eq $Text) ($c.notes.$note -replace "`n", ' / ')
Check 'the clock took Show seconds along' ($c.scripts.$clock.settings.seconds -eq $true) ($c.scripts.$clock | ConvertTo-Json -Compress)
Check 'the plugins run in their fences' ("$((ByTitle 'Note').script.output)" -match 'notes stand-in' -and "$((ByTitle 'Clock').script.output)" -match 'clock stand-in.*"seconds":true') "note=$((ByTitle 'Note').script.output) clock=$((ByTitle 'Clock').script.output)"
Check 'the snapshot 升级为插件之前 was saved' ((Before).Count -eq 1) ((Snaps | ForEach-Object { $_.name }) -join ', ')
Shot 'm02-plugin-fences'

Case 'a restart moves nothing more'
PfQuit
PfStart -Lang en
Start-Sleep -Seconds 6
Check 'the same fences, no new move, one snapshot' ((State) -eq $New -and (Moved) -eq 4 -and (Before).Count -eq 1) "$(State) moved=$(Moved)"

Case 'an import of the old configuration moves them again'
Pf config import "$PfRun\old.json" | Out-Null
Start-Sleep -Seconds 5
$c = Now
Check 'moved again at once (the plugins are there)' ((State) -eq $New -and (Moved) -eq 8) "$(State) moved=$(Moved)"
Check 'the note keeps its text, still one 升级为插件之前' ($c.notes.$note -eq $Text -and (Before).Count -eq 1) ($c.notes.$note -replace "`n", ' / ')

Case 'a backup restore: the running plugins get its settings, a fence naming another plugin starts that one'
# The import held its scripts back with 小工具 off: on again, the plugins run.
Pf settings set tools.enabled true | Out-Null
function PlugOut($t) { return "$((ByTitle $t).script.output)" }
for ($w = 0; $w -lt 20 -and (PlugOut 'Clock') -notmatch '"seconds":true'; $w++) { Start-Sleep -Milliseconds 500 }
Pf config export "$PfRun\running.json" | Out-Null
# The backup has 显示秒 off for the clock, and the weather plugin for the PC status fence.
$json = Get-Content "$PfRun\running.json" -Raw -Encoding UTF8
$json = $json -replace "(`"$clock`": \{[^}]*?`"settings`": \{\s*`"seconds`": )true", '${1}false' -replace '"plugin:pc-status"', '"plugin:weather"'
New-Item -ItemType Directory -Force "$script:Config\backups" | Out-Null
$backup = "$script:Config\backups\2000-01-01.json"
[IO.File]::WriteAllText($backup, $json)
$was = "clock=$(PlugOut 'Clock') status=$(PlugOut 'Status')"
Pf backup restore $backup | Out-Null
for ($w = 0; $w -lt 20 -and (PlugOut 'Clock') -notmatch '"seconds":false'; $w++) { Start-Sleep -Milliseconds 500 }
Start-Sleep -Seconds 2
Check 'the clock plugin shows the restored setting' ($was -match '"seconds":true' -and (PlugOut 'Clock') -match '"seconds":false') "before: $was; after: $(PlugOut 'Clock')"
Check 'the PC status fence runs the weather plugin it names now' ((PlugOut 'Status') -match 'weather stand-in' -and (State) -match 'Status=plugin:weather') "$(PlugOut 'Status'); $(State)"

Case 'an import keeps a plugin fence''s settings, without the folders picked on the other computer'
# The file also has another text for the note (a plugin fence holds no note for the CLI to set).
$Edited = "[ ] milk`n[x] rent`nedited after the upgrade"
$json = Get-Content $backup -Raw -Encoding UTF8
$json = $json -replace "(`"$clock`": \{[^}]*?`"settings`": \{)", '${1} "__folders": ["C:\\Windows"],' -replace 'plain line', 'edited after the upgrade'
[IO.File]::WriteAllText("$PfRun\folders.json", $json)
$asked = ((Get-Content "$PfRun\folders.json" -Raw -Encoding UTF8 | ConvertFrom-Json).scripts.$clock.settings.__folders)
Pf config import "$PfRun\folders.json" | Out-Null
Start-Sleep -Seconds 3
$s = (Now).scripts.$clock.settings
Check 'the clock keeps its settings, the folder list is gone' ("$asked" -eq 'C:\Windows' -and $s.seconds -eq $false -and $null -eq $s.__folders) "file=$asked now=$($s | ConvertTo-Json -Compress)"
Check 'the note has the text of the file' ((Now).notes.$note -eq $Edited) ((Now).notes.$note -replace "`n", ' / ')

Case 'the snapshot 升级为插件之前 brings the old fences back'
Pf snapshot restore (Before)[0].id | Out-Null
Start-Sleep -Seconds 4
Check 'the old fences are back, and stay' ((State) -eq $Old -and (Moved) -eq 8) "$(State) moved=$(Moved)"
Check 'the note has the text it had, not the one of the upgrade day' ((ByTitle 'Note').noteText -eq $Edited) ((ByTitle 'Note').noteText -replace "`n", ' / ')
Check 'the log says why they stay' ([bool](PfWaitLog 'restored, its fences stay as they are' 2)) ''
# The import held its scripts back with 小工具 off: on again, the old tools show and work.
Pf settings set tools.enabled true | Out-Null
Start-Sleep -Seconds 6
Check 'on again: the clock and the note show, the PC status example runs' (@(@([L]::Tops('pecofence') -split "`n") | Where-Object { $_ -match '\| (Clock|Note) \|' }).Count -eq 2 -and "$((ByTitle 'Status').script.output)" -match 'CPU' -and (Moved) -eq 8) "status=$((("$((ByTitle 'Status').script.output)") -split "`n")[0]) moved=$(Moved)"
Shot 'm03-restored'
Check 'no errors logged' ((PfLogErrors).Count -eq 0) ((PfLogErrors) -join ' / ')

Case 'cleanup'
PfQuit
if ($feedProc) { Stop-Process -Id $feedProc.Id -Force -ErrorAction SilentlyContinue }
