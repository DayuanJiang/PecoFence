# Plugin interface 3 and older PecoFence: the feed of assets\index3-feed, made by the plugin
# repository's build-index (index.json for 0.5.0 with stand-ins, index-3.json with every entry),
# both plugins of interface 3 (folders "any", which 0.5.0 cannot read). pf-keep 1.0.0 (interface 1)
# is installed before; the feed takes pf-keep down. `--var Part=old` runs it with PecoFence 0.5.0
# (--bin of 0.5.0): it reads index.json, keeps both stand-ins (none left out) and takes pf-keep
# down. `--var Part=new` (this build): it reads index-3.json first, keeps both whole entries and
# takes pf-keep down. Release build.
$Feed = 'C:\pf\run\pfeed3'
$FeedLog = "$PfRun\pfeed3.log"

Case "setup ($Part): the feed as the store publishes it, pf-keep 1.0.0 installed"
CloseWindows
if (Test-Path $Feed) { Remove-Item -LiteralPath $Feed -Recurse -Force }
Copy-Item -Recurse 'C:\pf\kit\assets\index3-feed' $Feed
Remove-Item $FeedLog -ErrorAction SilentlyContinue
# No index yet at the first start: it only says where the configuration is.
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8736/index.json'
PfStart -Lang en
$script:Config = Split-Path (PfJson status).configPath
PfQuit
$dir = "$script:Config\plugins\pf-keep"
New-Item -ItemType Directory -Force "$dir\1.0.0" | Out-Null
$m = [ordered]@{ id = 'pf-keep'; version = '1.0.0'; api = 1; layer = 'light'; name = @{ en = 'pf-keep' }
  description = @{ en = 'A test plugin.' }; author = 'pftest'; minPecoFence = '0.4.0'; permissions = @{} }
$utf8 = New-Object Text.UTF8Encoding($false)
[IO.File]::WriteAllText("$dir\1.0.0\plugin.json", ($m | ConvertTo-Json -Depth 5), $utf8)
[IO.File]::WriteAllText("$dir\1.0.0\main.js", "pf.on('start', () => pf.show({ rows: [{ text: 'pf-keep 1.0.0' }] }));`n", $utf8)
[IO.File]::WriteAllText("$dir\1.0.0\README.md", "pf-keep: a test plugin.`n", $utf8)
[IO.File]::WriteAllText("$dir\current.txt", '1.0.0', $utf8)
$feedProc = PfFeed 8736 $Feed $FeedLog
$env:PECOFENCE_PLUGINS_URL = 'http://127.0.0.1:8736/index.json'
PfStart -Lang en
Check 'pf-keep 1.0.0 counts as installed' ([bool](PfWaitLog 'plugins: read installed=1' 10)) ''

Case "the daily check reads the feed ($Part)"
$read = PfWaitLog 'plugins: index read' 90
$asked = @(Get-Content $FeedLog -ErrorAction SilentlyContinue | Where-Object { $_ -match '^GET /index' })
$left = @(PfLogAll | Where-Object { $_ -match 'plugins: index entry left out' })
Check 'the index is read, with both plugins' ($read -match 'plugins=2') "$read"
Check 'no entry is left out' ($left.Count -eq 0) ($left -join ' / ')
if ($Part -eq 'old') {
  Check '0.5.0 asks for index.json only' (($asked -join ' ') -match 'GET /index\.json' -and ($asked -join ' ') -notmatch 'index-3') ($asked -join ' / ')
} else {
  Check 'this build asks for index-3.json first' ($asked.Count -ge 1 -and $asked[0] -match 'GET /index-3\.json') ($asked -join ' / ')
  Check 'and does not need index.json' (($asked -join ' ') -notmatch 'GET /index\.json') ($asked -join ' / ')
}
$saved = Get-Content "$script:Config\plugins\index.json" -Raw -Encoding UTF8 -ErrorAction SilentlyContinue
$want = if ($Part -eq 'old') { '"readwrite"' } else { '"any"' }
Check "the index kept is the one it read (folders $want)" ("$saved" -match "`"folders`":\s*$want") ''

Case "the take-down of pf-keep reaches it ($Part)"
$down = PfWaitLog 'plugins: taken down by the index.*pf-keep' 10
Check 'pf-keep is taken down' ([bool]$down -and (Test-Path "$dir\disabled.txt")) "$down"
Check 'with the reason of the index' ((Get-Content "$dir\disabled.txt" -Raw -ErrorAction SilentlyContinue) -match 'a test take-down') ''

PfQuit
if ($feedProc) { Stop-Process -Id $feedProc.Id -ErrorAction SilentlyContinue }
Remove-Item Env:PECOFENCE_PLUGINS_URL -ErrorAction SilentlyContinue
