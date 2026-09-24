# Build separately from the running development app, then create a portable ZIP.
param(
  [string]$Version = "",
  [string]$TargetDir = "target/package",
  [string]$Python = "python",
  [switch]$SkipBuild
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  $env:PATH = (Join-Path $env:USERPROFILE ".cargo\bin") + ";" + $env:PATH
}
if (-not $SkipBuild) {
  cargo build --locked --release -p pecofence -p pecofence-watchdog -p pecofence-cli --target-dir $TargetDir
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
}

if (-not $Version) {
  $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value
}
if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') { throw "Invalid version: $Version" }
$distRoot = [IO.Path]::GetFullPath((Join-Path $root "dist"))
$stage = [IO.Path]::GetFullPath((Join-Path $distRoot "pecofence-$Version-x64"))
if (-not $stage.StartsWith($distRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw "Unsafe package destination" }
if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage | Out-Null

$release = Join-Path $TargetDir "release"
Copy-Item -LiteralPath (Join-Path $release "pecofence.exe") -Destination $stage
Copy-Item -LiteralPath (Join-Path $release "pecofence-watchdog.exe") -Destination $stage
Copy-Item -LiteralPath (Join-Path $release "pecofence-cli.exe") -Destination $stage
Copy-Item -LiteralPath "third_party/webview2/WebView2Loader.x64.dll" -Destination (Join-Path $stage "WebView2Loader.dll")
Copy-Item -LiteralPath "third_party/webview2/LICENSE.txt" -Destination (Join-Path $stage "LICENSE-WebView2Loader.txt")
Copy-Item -LiteralPath "LICENSE" -Destination $stage
Copy-Item -LiteralPath "docs/PORTABLE.md" -Destination (Join-Path $stage "README.md")
Copy-Item -LiteralPath "docs/UPGRADING.md" -Destination (Join-Path $stage "UPGRADING.md")
Copy-Item -LiteralPath "skills/pecofence-cli/SKILL.md" -Destination (Join-Path $stage "SKILL.md")
& $Python scripts/write-license-notices.py (Join-Path $stage "THIRD-PARTY-LICENSES.txt")
if ($LASTEXITCODE -ne 0) { throw "License notice generation failed" }

$zip = "$stage.zip"
if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip }
Compress-Archive -Path "$stage/*" -DestinationPath $zip
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash
"$hash  $(Split-Path $zip -Leaf)" | Set-Content "$zip.sha256"
Write-Output "packed $zip"
Write-Output "SHA256 $hash"
Get-ChildItem -LiteralPath $stage | Select-Object Name, @{n='KB';e={[math]::Round($_.Length/1KB)}}
