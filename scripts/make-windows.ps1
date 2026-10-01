#requires -Version 5.1
# Build once, then package identical binaries with distribution-specific markers.
[CmdletBinding()]
param(
  [ValidateSet("All", "Portable", "Installer")][string]$Format = "All",
  [string]$Version = "",
  [string]$TargetDir = "target/package",
  [string]$Python = "python",
  [string]$Repository = "",
  [string]$Iscc = "",
  [switch]$SkipBuild
)
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$utf8 = New-Object System.Text.UTF8Encoding($false)

function Reset-Stage([string]$Path) {
  $resolved = [IO.Path]::GetFullPath($Path)
  if (-not $resolved.StartsWith($stagingRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw "Unsafe staging destination: $resolved"
  }
  if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
  New-Item -ItemType Directory -Path $resolved -Force | Out-Null
}

function Write-Checksum([string]$Path) {
  $hash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
  [IO.File]::WriteAllText("$Path.sha256", "$hash  $(Split-Path $Path -Leaf)`n", $utf8)
  Write-Host "Packed $Path"
}

Push-Location -LiteralPath $root
try {
  $workspaceVersion = (Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value
  if (-not $Version) { $Version = $workspaceVersion }
  if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$' -or $Version -cne $workspaceVersion) {
    throw "Package version must match Cargo.toml ($workspaceVersion)"
  }
  if (-not $Repository) { $Repository = $env:GITHUB_REPOSITORY }
  if (-not $Repository) { $Repository = "DayuanJiang/PecoFence" }
  if ($Repository -notmatch '^[A-Za-z0-9][A-Za-z0-9-]*/[A-Za-z0-9_.-]+$' -or $Repository.Split('/')[1] -in @('.', '..')) {
    throw "Repository must be a GitHub owner/repository name"
  }

  if ($Format -ne "Portable") {
    . "$PSScriptRoot/find-iscc.ps1"
    $Iscc = Find-Iscc $Iscc
  }
  if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $env:PATH = (Join-Path $env:USERPROFILE ".cargo/bin") + ";" + $env:PATH
  }
  if (-not $SkipBuild) {
    & cargo build --locked --release -p pecofence -p pecofence-watchdog --target-dir $TargetDir
    if ($LASTEXITCODE -ne 0) { throw "App/watchdog build failed" }
    # Keep the CLI's schemars feature out of the GUI executable.
    & cargo build --locked --release -p pecofence-cli --target-dir $TargetDir
    if ($LASTEXITCODE -ne 0) { throw "CLI build failed" }
  }

  $release = [IO.Path]::GetFullPath((Join-Path $TargetDir "release"))
  foreach ($exe in @("pecofence.exe", "pecofence-watchdog.exe", "pecofence-cli.exe")) {
    if (-not (Test-Path -LiteralPath (Join-Path $release $exe) -PathType Leaf)) {
      throw "Missing $exe in $release"
    }
  }
  # Catch stale --SkipBuild output before assigning it the current release name.
  $builtVersion = (Get-Item -LiteralPath (Join-Path $release "pecofence.exe")).VersionInfo.ProductVersion
  if ($builtVersion -cne $Version) { throw "Built app version '$builtVersion' does not match $Version; rebuild with the Windows SDK installed" }

  $distRoot = Join-Path $root "dist"
  $stagingRoot = [IO.Path]::GetFullPath((Join-Path $distRoot ".staging"))
  $baseName = "pecofence-v$Version-x64"
  $common = Join-Path $stagingRoot "$baseName-common"
  Reset-Stage $common
  foreach ($exe in @("pecofence.exe", "pecofence-watchdog.exe", "pecofence-cli.exe")) {
    Copy-Item -LiteralPath (Join-Path $release $exe) -Destination $common
  }
  Copy-Item -LiteralPath "third_party/webview2/WebView2Loader.x64.dll" -Destination (Join-Path $common "WebView2Loader.dll")
  Copy-Item -LiteralPath "third_party/webview2/LICENSE.txt" -Destination (Join-Path $common "LICENSE-WebView2Loader.txt")
  Copy-Item -LiteralPath "LICENSE" -Destination $common
  Copy-Item -LiteralPath "docs/UPGRADING.md" -Destination (Join-Path $common "UPGRADING.md")
  Copy-Item -LiteralPath "skills/pecofence-cli/SKILL.md" -Destination (Join-Path $common "SKILL.md")
  & $Python scripts/write-license-notices.py (Join-Path $common "THIRD-PARTY-LICENSES.txt")
  if ($LASTEXITCODE -ne 0) { throw "License notice generation failed" }
  # Package provenance and the repository used by the manual in-app updater.
  $info = [ordered]@{ schema = 1; repository = $Repository; version = $Version; tag = "v$Version" }
  [IO.File]::WriteAllText((Join-Path $common "release-info.json"), ($info | ConvertTo-Json) + "`n", $utf8)

  $modes = switch ($Format) { "All" { "portable"; "installed" }; "Portable" { "portable" }; "Installer" { "installed" } }
  foreach ($mode in $modes) {
    $stage = Join-Path $stagingRoot "$baseName-$mode"
    Reset-Stage $stage
    Get-ChildItem -LiteralPath $common -File | Copy-Item -Destination $stage
    $guide = if ($mode -eq "portable") { "docs/PORTABLE.md" } else { "docs/INSTALLER.md" }
    Copy-Item -LiteralPath $guide -Destination (Join-Path $stage "README.md")
    $marker = '{"schema":1,"appId":"PecoFence","mode":"' + $mode + '"}'
    [IO.File]::WriteAllText((Join-Path $stage "deployment.json"), $marker + "`n", $utf8)
    if ($mode -eq "portable") {
      $zip = Join-Path $distRoot "$baseName-portable.zip"
      Compress-Archive -Path "$stage/*" -DestinationPath $zip -Force
      Write-Checksum $zip
    } else {
      & $Iscc --quiet "--define=PayloadDir=$stage" "--define=AppVersion=$Version" "--define=Repository=$Repository" "--output-dir=$distRoot" "--output-filename=$baseName-setup" packaging/inno/pecofence.iss
      if ($LASTEXITCODE -ne 0) { throw "Inno Setup compilation failed" }
      Write-Checksum (Join-Path $distRoot "$baseName-setup.exe")
    }
  }
} finally {
  Pop-Location
}
