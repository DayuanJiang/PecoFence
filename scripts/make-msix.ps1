# Build the Microsoft Store MSIX package from the same binaries as the portable ZIP.
#
#   ./scripts/make-msix.ps1                 # unsigned .msix for Partner Center upload
#   ./scripts/make-msix.ps1 -TestSign       # also a self-signed copy for local install tests
#   ./scripts/make-msix.ps1 -SkipBuild      # reuse target/package/release from make-portable.ps1
#
# Identity values come from packaging/msix/identity.json (Partner Center > Product identity).
# Requires the Windows 10/11 SDK (makeappx, makepri, signtool) and Python with Pillow.
param(
  [string]$Version = "",
  [string]$TargetDir = "target/package",
  [string]$Python = "python",
  [string]$Identity = "packaging/msix/identity.json",
  [switch]$SkipBuild,
  [switch]$TestSign,
  [string]$TestCertPassword = "pecofence-test"
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Find-SdkTool([string]$name) {
  $cmd = Get-Command $name -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }
  $kits = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
  $versions = Get-ChildItem -Path $kits -Directory -Filter "10.*" -ErrorAction SilentlyContinue |
    Sort-Object { [version]$_.Name } -Descending
  foreach ($v in $versions) {
    $candidate = Join-Path $v.FullName "x64\$name"
    if (Test-Path -LiteralPath $candidate) { return $candidate }
  }
  throw "$name not found. Install the Windows SDK (winget install Microsoft.WindowsSDK.10.0.26100)."
}
$makeappx = Find-SdkTool "makeappx.exe"
$makepri = Find-SdkTool "makepri.exe"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  $env:PATH = (Join-Path $env:USERPROFILE ".cargo\bin") + ";" + $env:PATH
}
if (-not $SkipBuild) {
  cargo build --locked --release -p pecofence -p pecofence-watchdog --target-dir $TargetDir
  if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
}

if (-not $Version) {
  $Version = (Select-String -Path "Cargo.toml" -Pattern '^version = "([^"]+)"').Matches[0].Groups[1].Value
}
if ($Version -notmatch '^(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?$') { throw "Invalid version: $Version" }
# MSIX needs four parts; the Store requires the revision (last part) to be 0.
$packageVersion = "$($Matches[1]).$($Matches[2]).$($Matches[3]).0"

$id = Get-Content -LiteralPath $Identity -Raw | ConvertFrom-Json
foreach ($field in "identityName", "publisher", "publisherDisplayName") {
  if (-not $id.$field) { throw "$Identity is missing '$field'" }
}

$distRoot = [IO.Path]::GetFullPath((Join-Path $root "dist"))
$stage = [IO.Path]::GetFullPath((Join-Path $distRoot "pecofence-$Version-msix"))
if (-not $stage.StartsWith($distRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw "Unsafe package destination" }
if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage | Out-Null

# Payload: identical to the portable ZIP minus the portable README.
$release = Join-Path $TargetDir "release"
Copy-Item -LiteralPath (Join-Path $release "pecofence.exe") -Destination $stage
Copy-Item -LiteralPath (Join-Path $release "pecofence-watchdog.exe") -Destination $stage
Copy-Item -LiteralPath "third_party/webview2/WebView2Loader.x64.dll" -Destination (Join-Path $stage "WebView2Loader.dll")
Copy-Item -LiteralPath "third_party/webview2/LICENSE.txt" -Destination (Join-Path $stage "LICENSE-WebView2Loader.txt")
Copy-Item -LiteralPath "LICENSE" -Destination (Join-Path $stage "LICENSE.txt")
& $Python scripts/write-license-notices.py (Join-Path $stage "THIRD-PARTY-LICENSES.txt")
if ($LASTEXITCODE -ne 0) { throw "License notice generation failed" }

# Tile and Store logos rendered from site/assets/mark.svg.
& $Python scripts/make-msix-assets.py (Join-Path $stage "Assets")
if ($LASTEXITCODE -ne 0) { throw "Asset generation failed" }

# Manifest with the Partner Center identity filled in.
$manifest = Get-Content -LiteralPath "packaging/msix/AppxManifest.xml" -Raw
$manifest = $manifest.Replace("__IDENTITY_NAME__", [string]$id.identityName)
$manifest = $manifest.Replace("__IDENTITY_PUBLISHER__", [string]$id.publisher)
$manifest = $manifest.Replace("__PUBLISHER_DISPLAY_NAME__", [string]$id.publisherDisplayName)
$manifest = $manifest.Replace("__VERSION__", $packageVersion)
if ($manifest -match "__[A-Z_]+__") { throw "Unreplaced manifest token: $($Matches[0])" }
$manifestPath = Join-Path $stage "AppxManifest.xml"
[IO.File]::WriteAllText($manifestPath, $manifest, (New-Object Text.UTF8Encoding $false))

# Resource index so scale-qualified assets resolve.
$priConfig = Join-Path $distRoot "pecofence-$Version-priconfig.xml"
if (Test-Path -LiteralPath $priConfig) { Remove-Item -LiteralPath $priConfig }
& $makepri createconfig /cf $priConfig /dq en-US /pv 10.0.0 /o
if ($LASTEXITCODE -ne 0) { throw "makepri createconfig failed" }
& $makepri new /pr $stage /cf $priConfig /of (Join-Path $stage "resources.pri") /mn $manifestPath /o
if ($LASTEXITCODE -ne 0) { throw "makepri new failed" }
Remove-Item -LiteralPath $priConfig

$msix = Join-Path $distRoot "pecofence-$Version-x64.msix"
if (Test-Path -LiteralPath $msix) { Remove-Item -LiteralPath $msix }
& $makeappx pack /d $stage /p $msix /o
if ($LASTEXITCODE -ne 0) { throw "makeappx pack failed" }
$hash = (Get-FileHash $msix -Algorithm SHA256).Hash
"$hash  $(Split-Path $msix -Leaf)" | Set-Content "$msix.sha256"
Write-Output "packed $msix (unsigned; the Store signs it after certification)"
Write-Output "SHA256 $hash"

if ($TestSign) {
  # Self-signed certificate whose subject equals the package Publisher, for local
  # Add-AppxPackage tests only. Install the .cer into Trusted People first.
  $signtool = Find-SdkTool "signtool.exe"
  $cert = New-SelfSignedCertificate -Type Custom -Subject ([string]$id.publisher) `
    -KeyUsage DigitalSignature -FriendlyName "PecoFence MSIX test signing" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
  $pfx = Join-Path $distRoot "pecofence-test-signing.pfx"
  $cer = Join-Path $distRoot "pecofence-test-signing.cer"
  $secure = ConvertTo-SecureString -String $TestCertPassword -Force -AsPlainText
  Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $secure | Out-Null
  Export-Certificate -Cert $cert -FilePath $cer | Out-Null
  Remove-Item -LiteralPath ("Cert:\CurrentUser\My\" + $cert.Thumbprint)
  $signed = $msix -replace '\.msix$', '-testsigned.msix'
  Copy-Item -LiteralPath $msix -Destination $signed -Force
  & $signtool sign /fd SHA256 /f $pfx /p $TestCertPassword $signed
  if ($LASTEXITCODE -ne 0) { throw "signtool failed" }
  Write-Output "test-signed $signed"
  Write-Output "install the certificate once: Import-Certificate -FilePath `"$cer`" -CertStoreLocation Cert:\LocalMachine\TrustedPeople (admin)"
}
