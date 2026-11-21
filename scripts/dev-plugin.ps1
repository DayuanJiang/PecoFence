# Tries a plugin of this working copy the way a user gets one: from the Tools page of Settings
# (Install, then Add), with no publication. It builds this working copy, serves the plugin from a
# local index on http://127.0.0.1 (the only kind of address a test instance accepts, unsigned),
# and starts the build as a named test instance (`PECOFENCE_INSTANCE=dev`) with its own
# configuration in .cache\dev-instance\config, so the main PecoFence and its fences are left
# alone and can run at the same time. The script stays until that instance is closed.
#
#   .\scripts\dev-plugin.ps1                      # the Magnifier plugin, debug build
#   .\scripts\dev-plugin.ps1 -NoBuild -Port 8800  # the build as it is, another port
#   .\scripts\dev-plugin.ps1 -Plugin clock-sample
#
# The plugin is read from scripts\wintest\assets\<Plugin> again at each start. The Settings of the
# test instance open by themselves (`-NoSettings` to skip): that is the page to use, not the
# Settings of your usual PecoFence, whose Tools page lists the published plugins. In the test
# instance the local index replaces that list, so only this plugin is listed.
param(
  [string]$Plugin = 'magnifier',
  [int]$Port = 8731,
  [switch]$NoBuild,
  [switch]$Release,
  [switch]$NoSettings
)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
$source = Join-Path $root "scripts\wintest\assets\$Plugin"
if (-not (Test-Path (Join-Path $source 'plugin.json'))) { throw "No plugin.json in $source" }
$flavour = if ($Release) { 'release' } else { 'debug' }

# 1. The build: the app and its watchdog.
Push-Location $root
try {
  if (-not $NoBuild) {
    $flags = if ($Release) { @('--release') } else { @() }
    & cargo build -p pecofence -p pecofence-watchdog @flags
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
  }
  $target = (& cargo metadata --format-version 1 --no-deps | ConvertFrom-Json).target_directory
} finally { Pop-Location }
$built = Join-Path $target $flavour
if (-not (Test-Path "$built\pecofence.exe")) { throw "No pecofence.exe in $built (build without -NoBuild)" }

# 2. The instance's folder: a copy of the build, so cargo can rebuild while it runs, and the
# configuration beside it (--portable).
$instance = Join-Path $root '.cache\dev-instance'
$feed = Join-Path $root '.cache\dev-feed'
Get-Process pecofence -ErrorAction SilentlyContinue |
  Where-Object { $_.Path -and $_.Path.StartsWith($instance, [StringComparison]::OrdinalIgnoreCase) } |
  ForEach-Object { Stop-Process -Id $_.Id -Force; $_.WaitForExit(5000) | Out-Null }
New-Item -ItemType Directory -Force $instance | Out-Null
Copy-Item "$built\pecofence.exe", "$built\pecofence-watchdog.exe", "$built\pecofence-cli.exe" $instance -Force -ErrorAction SilentlyContinue
# Settings (the Tools page) needs the WebView2 loader beside the executable.
$loader = if (Test-Path "$built\WebView2Loader.dll") { "$built\WebView2Loader.dll" } else { Join-Path $root 'third_party\webview2\WebView2Loader.x64.dll' }
Copy-Item $loader (Join-Path $instance 'WebView2Loader.dll') -Force

# 3. The local index: this plugin's files, with their sizes and hashes.
$commit = ('d' * 39) + '1'
$files = Join-Path $feed "$commit\plugins\$Plugin"
Remove-Item $feed -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $files | Out-Null
Copy-Item (Join-Path $source '*') $files -Recurse
$manifest = Get-Content (Join-Path $files 'plugin.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$manifest | Add-Member files @(Get-ChildItem $files -File | ForEach-Object {
    [ordered]@{ name = $_.Name; size = $_.Length; sha256 = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower() }
  })
$index = [ordered]@{ format = 1; commit = $commit; plugins = @($manifest) }
[IO.File]::WriteAllText((Join-Path $feed 'index.json'), ($index | ConvertTo-Json -Depth 12), (New-Object Text.UTF8Encoding($false)))

# 4. A small web server for the feed, on this computer only.
$serve = Join-Path $root '.cache\dev-feed-serve.ps1'
Set-Content $serve @'
param([int]$port, [string]$dir)
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), $port); $l.Start()
while ($true) {
  $c = $l.AcceptTcpClient()
  try {
    $s = $c.GetStream(); $r = New-Object IO.StreamReader($s); $line = $r.ReadLine(); while ($r.ReadLine()) {}
    $path = [IO.Path]::GetFullPath($dir + (($line -split ' ')[1] -replace '/', '\'))
    if ($path.StartsWith($dir, [StringComparison]::OrdinalIgnoreCase) -and (Test-Path -LiteralPath $path -PathType Leaf)) {
      $b = [IO.File]::ReadAllBytes($path); $h = "HTTP/1.1 200 OK`r`nContent-Length: $($b.Length)`r`nConnection: close`r`n`r`n"
    } else { $b = @(); $h = "HTTP/1.1 404 Not Found`r`nContent-Length: 0`r`nConnection: close`r`n`r`n" }
    $hb = [Text.Encoding]::ASCII.GetBytes($h); $s.Write($hb, 0, $hb.Length); if ($b.Length) { $s.Write($b, 0, $b.Length) }; $s.Flush()
  } catch {} finally { $c.Close() }
}
'@
$server = Start-Process powershell.exe -WindowStyle Hidden -PassThru -ArgumentList @(
  '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', $serve, '-port', $Port, '-dir', $feed)

# 5. The instance, until it is closed.
try {
  $env:PECOFENCE_INSTANCE = 'dev'
  $env:PECOFENCE_PLUGINS_URL = "http://127.0.0.1:$Port/index.json"
  $env:RUST_LOG = 'pecofence=debug'
  $app = Start-Process (Join-Path $instance 'pecofence.exe') -ArgumentList '--portable', '--no-hide-icons' `
    -WorkingDirectory $instance -PassThru
  Write-Host "PecoFence (test instance 'dev') started. Its Tools page lists only '$Plugin': Install, then Add."
  Write-Host "Configuration: $instance\config   Log: pecofence-cli --instance dev log"
  Write-Host 'Close the instance (tray icon > Exit) to stop the feed.'
  if (-not $NoSettings) {
    $cli = Join-Path $instance 'pecofence-cli.exe'
    for ($i = 0; $i -lt 20; $i++) {
      & $cli --instance dev status --compact *> $null
      if ($LASTEXITCODE -eq 0) { & $cli --instance dev settings open-ui --compact *> $null; break }
      Start-Sleep -Seconds 1
    }
  }
  $app.WaitForExit()
} finally {
  Stop-Process -Id $server.Id -Force -ErrorAction SilentlyContinue
}
