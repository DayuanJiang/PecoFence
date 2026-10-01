# Download a pinned, hash-verified compiler only on a disposable GitHub Actions runner.
$ErrorActionPreference = "Stop"
if ($env:GITHUB_ACTIONS -ne "true" -or -not $env:RUNNER_TEMP -or -not $env:GITHUB_ENV) {
  throw "CI only. Locally, install Inno Setup 7 and use -Iscc or ISCC."
}
$lock = Get-Content -LiteralPath "$PSScriptRoot/../packaging/inno/toolchain.json" -Raw | ConvertFrom-Json
$download = Join-Path $env:RUNNER_TEMP "innosetup-$($lock.version).exe"
$destination = Join-Path $env:RUNNER_TEMP "pecofence-inno-$($lock.version)"
Invoke-WebRequest -Uri $lock.url -OutFile $download
if ((Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash -ine $lock.sha256) { throw "Inno Setup SHA-256 mismatch" }
$process = Start-Process -FilePath $download -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/CURRENTUSER', '/NOICONS', "/DIR=`"$destination`"") -WindowStyle Hidden -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Inno Setup installation failed: $($process.ExitCode)" }
$compiler = Join-Path $destination "ISCC.exe"
$version = & $compiler --version
if ($LASTEXITCODE -ne 0 -or "$version" -cne $lock.version) { throw "Unexpected Inno Setup compiler: $version" }
"ISCC=$compiler" | Out-File -LiteralPath $env:GITHUB_ENV -Encoding utf8 -Append
