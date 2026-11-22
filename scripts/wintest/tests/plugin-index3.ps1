# Plugin interface 3 and older PecoFence: the feed of assets\index3-feed, made by the plugin
# repository's build-index (index.json with stand-ins marked standIn for 0.5.0, index-3.json with
# every entry), four plugins of interface 3. Installed before: pf-keep 1.0.0 (the feed takes it
# down), pf-up 1.0.0 (folders readwrite; its stand-in looks like an automatic update, the whole
# entry asks for folders any), pf-big 1.0.0 (a web plugin; 1.1.0 brings a 2 MB data.json, which
# its stand-in leaves out).
# `--var Part=old` (--bin of PecoFence 0.5.0): reads index.json, keeps the stand-ins, takes pf-keep
# down, updates nothing. `Part=new` (this build): reads index-3.json first, takes pf-keep down,
# updates pf-big with its 2 MB file, leaves pf-up (it asks for more). `Part=fallback` (this build,
# no index-3.json): reads index.json, takes pf-keep down, updates nothing from a stand-in.
# Release build.
$Feed = 'C:\pf\run\pfeed3'
$FeedLog = "$PfRun\pfeed3.log"
$Commit = '1d3'.PadRight(40, 'a')

Case "setup ($Part): the feed as the store publishes it, three plugins installed"
CloseWindows
if (Test-Path $Feed) { Remove-Item -LiteralPath $Feed -Recurse -Force }
Copy-Item -Recurse 'C:\pf\kit\assets\index3-feed' $Feed
$utf8 = New-Object Text.UTF8Encoding($false)
# pf-big's data.json, byte for byte as the feed's index lists it (kept out of the repository).
[IO.File]::WriteAllText("$Feed\$Commit\plugins\pf-big\data.json", '{"pad": "' + ('a' * 2000000) + '"}', $utf8)
if ($Part -eq 'fallback') { Remove-Item "$Feed\index-3.json" }
Remove-Item $FeedLog -ErrorAction SilentlyContinue
# No index yet at the first start: it only says where the configuration is.
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8736/index.json'
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
PfQuit
function Installed($id, $api, $perms, [switch]$Web) {
  $dir = "$script:Config\plugins\$id"
  New-Item -ItemType Directory -Force "$dir\1.0.0" | Out-Null
  $m = [ordered]@{ id = $id; version = '1.0.0'; api = $api; layer = $(if ($Web) { 'web' } else { 'light' }); name = @{ en = $id }
    description = @{ en = 'A test plugin.' }; author = 'pftest'; minPecoFence = '0.4.0'; permissions = $perms }
  [IO.File]::WriteAllText("$dir\1.0.0\plugin.json", ($m | ConvertTo-Json -Depth 5), $utf8)
  if ($Web) { [IO.File]::WriteAllText("$dir\1.0.0\index.html", "<!doctype html><p>$id 1.0.0</p>`n", $utf8) }
  else { [IO.File]::WriteAllText("$dir\1.0.0\main.js", "pf.on('start', () => pf.show({ rows: [{ text: '$id 1.0.0' }] }));`n", $utf8) }
  [IO.File]::WriteAllText("$dir\1.0.0\README.md", "${id}: a test plugin.`n", $utf8)
  [IO.File]::WriteAllText("$dir\current.txt", '1.0.0', $utf8)
}
Installed 'pf-keep' 1 @{}
Installed 'pf-up' 2 @{ folders = 'readwrite' }
Installed 'pf-big' 1 @{} -Web
$feedProc = PfFeed 8736 $Feed $FeedLog
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8736/index.json'
PfStart -Lang en
Check 'the three plugins count as installed' ([bool](PfWaitLog 'plugins: read installed=3' 10)) ''

Case "the daily check reads the feed ($Part)"
$read = PfWaitLog 'plugins: index read' 90
$asked = @(Get-Content $FeedLog -ErrorAction SilentlyContinue | Where-Object { $_ -match '^GET /index' })
$left = @(PfLogAll | Where-Object { $_ -match 'plugins: index entry left out' })
Check 'the index is read, with the four plugins' ($read -match 'plugins=4') "$read"
Check 'no entry is left out' ($left.Count -eq 0) ($left -join ' / ')
$seen = $asked -join ' '
switch ($Part) {
  'old' { Check '0.5.0 asks for index.json only' ($seen -match 'GET /index\.json' -and $seen -notmatch 'index-3') ($asked -join ' / ') }
  'new' {
    Check 'this build asks for index-3.json first' ($asked.Count -ge 1 -and $asked[0] -match 'GET /index-3\.json') ($asked -join ' / ')
    Check 'and does not need index.json' ($seen -notmatch 'GET /index\.json') ($asked -join ' / ')
  }
  'fallback' { Check 'this build asks for index-3.json, then takes index.json' ($asked.Count -ge 2 -and $asked[0] -match 'index-3\.json' -and $seen -match 'GET /index\.json') ($asked -join ' / ') }
}
$saved = Get-Content "$script:Config\plugins\index.json" -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
$want = if ($Part -eq 'new') { '"any"' } else { '"readwrite"' }
Check "the index kept is the one it read (folders $want)" ("$saved" -match "`"folders`":\s*$want") ''

Case "the take-down of pf-keep reaches it ($Part)"
$down = PfWaitLog 'plugins: taken down by the index.*pf-keep' 10
Check 'pf-keep is taken down' ([bool]$down -and (Test-Path "$script:Config\plugins\pf-keep\disabled.txt")) "$down"
Check 'with the reason of the index' ((Get-Content "$script:Config\plugins\pf-keep\disabled.txt" -Raw -ErrorAction SilentlyContinue) -match 'a test take-down') ''

Case "updates ($Part): pf-up never from a stand-in, pf-big with its 2 MB file only from index-3.json"
if ($Part -eq 'new') {
  $done = PfWaitLog 'plugins: installed, its fences start again.*pf-big' 90
  $file = Get-Item "$script:Config\plugins\pf-big\1.1.0\data.json" -ErrorAction SilentlyContinue
  Check 'pf-big 1.1.0 is installed with its 2 MB data.json' ($done -and (Get-Content "$script:Config\plugins\pf-big\current.txt") -eq '1.1.0' -and $file.Length -eq 2000011) "$done size=$($file.Length)"
} else {
  Start-Sleep -Seconds 10
  Check 'pf-big stays at 1.0.0' ((Get-Content "$script:Config\plugins\pf-big\current.txt") -eq '1.0.0') ''
}
$up = @(PfLogAll | Where-Object { $_ -match 'plugins: download .*pf-up' })
Check 'nothing downloads pf-up' ($up.Count -eq 0) ($up -join ' / ')
if ($Part -eq 'fallback') {
  $check = @(PfLogAll | Where-Object { $_ -match 'plugins: update check.*pf-up' }) -join ' / '
  Check 'pf-up''s entry is a stand-in, so it counts for nothing but a take-down' ($check -match 'stand_in=true') $check
}

PfQuit
if ($feedProc) { Stop-Process -Id $feedProc.Id -ErrorAction SilentlyContinue }
Remove-Item Env:PECOFENCE_PLUGINS_URL -ErrorAction SilentlyContinue
