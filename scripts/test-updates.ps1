#requires -Version 5.1
# Offline tests of the production worker. All writes stay in a disposable .cache
# tree; transport and process launch boundaries are replaced with local fixtures.
[CmdletBinding()]
param([string]$TargetDir = 'target/package')
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$releaseRoot = Join-Path ([IO.Path]::GetFullPath((Join-Path $repo $TargetDir))) 'release'
. (Join-Path $PSScriptRoot 'runtime/update-worker.ps1')
Add-Type -AssemblyName System.IO.Compression
$testRoot = Join-Path $repo ('.cache/update-tests-' + [guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($testRoot)
$testVersion = [Diagnostics.FileVersionInfo]::GetVersionInfo((Join-Path $releaseRoot 'pecofence.exe')).ProductVersion
$priorVersion = '0.0.0'
$testRepository = 'Contributor/PecoFence'
$results = New-Object 'Collections.Generic.List[string]'

function Assert-Test($Condition, [string]$Message) { if (-not $Condition) { throw $Message } }
function Expect-Failure([scriptblock]$Body) {
  $failed = $false
  try { & $Body } catch { $failed = $true }
  Assert-Test $failed 'Expected a failure, but the operation succeeded.'
}
function Run-Test([string]$Name, [scriptblock]$Body) {
  & $Body
  $results.Add($Name)
  Write-Output "PASS: $Name"
}
function Write-FixtureIdentity([string]$Root, [string]$Mode, [string]$Version) {
  Write-UpdateJson (Join-Path $Root 'deployment.json') @{ schema = 1; appId = 'PecoFence'; mode = $Mode }
  Write-UpdateJson (Join-Path $Root 'release-info.json') @{ schema = 1; repository = $testRepository; version = $Version; tag = "v$Version" }
}
function New-Fixture([string]$Mode = 'portable') {
  $root = Join-Path $testRoot ('copy with spaces [' + [guid]::NewGuid().ToString('N') + ']')
  $null = [IO.Directory]::CreateDirectory($root)
  foreach ($name in $script:ManagedFiles) {
    if ($name.EndsWith('.exe')) { [IO.File]::Copy((Join-Path $releaseRoot $name), (Join-Path $root $name)) }
    elseif ($name -eq 'WebView2Loader.dll') { [IO.File]::Copy((Join-Path $repo 'third_party/webview2/WebView2Loader.x64.dll'), (Join-Path $root $name)) }
    else { [IO.File]::WriteAllText((Join-Path $root $name), "old $name") }
  }
  Write-FixtureIdentity $root $Mode $priorVersion
  foreach ($name in @('config/config.json', 'data/logs/pecofence.log', 'data/WebView2Profiles/default/user.txt', 'personal.txt')) {
    $path = Join-Path $root $name
    $null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($path))
    [IO.File]::WriteAllText($path, 'preserve this user data')
  }
  $updates = if ($Mode -eq 'portable') { Join-Path $root 'data/updates' } else { Join-Path $env:LOCALAPPDATA 'PecoFence/updates' }
  $stage = Join-Path $updates ([guid]::NewGuid().ToString('N'))
  $null = [IO.Directory]::CreateDirectory($stage)
  $suffix = if ($Mode -eq 'portable') { 'portable.zip' } else { 'setup.exe' }
  $asset = "pecofence-v$testVersion-x64-$suffix"
  $base = "https://github.com/$testRepository/releases"
  $data = [pscustomobject]@{
    schema = 1; root = $root; mode = $Mode; repository = $testRepository; currentVersion = $priorVersion; parentPid = 0
    release = [pscustomobject]@{ version = $testVersion; page = "$base/tag/v$testVersion"; asset = $asset; url = "$base/download/v$testVersion/$asset"; checksumUrl = "$base/download/v$testVersion/$asset.sha256"; bytes = 1; digest = $null }
  }
  Write-UpdateJson (Join-Path $stage 'plan.json') $data
  return [pscustomobject]@{ root = $root; stage = $stage; data = $data }
}
function New-Archive([string]$Path, [string[]]$Names = $script:ManagedFiles, [string]$Repository = $testRepository, [string]$Mode = 'portable') {
  $file = [IO.File]::Open($Path, [IO.FileMode]::CreateNew)
  $zip = New-Object IO.Compression.ZipArchive($file, [IO.Compression.ZipArchiveMode]::Create)
  try {
    foreach ($name in $Names) {
      $entry = $zip.CreateEntry($name)
      $output = $entry.Open()
      try {
        if ($name -in @('pecofence.exe', 'pecofence-watchdog.exe', 'pecofence-cli.exe', 'WebView2Loader.dll')) {
          $source = if ($name -eq 'WebView2Loader.dll') { Join-Path $repo 'third_party/webview2/WebView2Loader.x64.dll' } else { Join-Path $releaseRoot $name }
          $inputStream = [IO.File]::OpenRead($source)
          try { $inputStream.CopyTo($output) } finally { $inputStream.Dispose() }
        } else {
          $content = if ($name -eq 'deployment.json') { @{schema=1;appId='PecoFence';mode=$Mode} | ConvertTo-Json }
          elseif ($name -eq 'release-info.json') { @{schema=1;repository=$Repository;version=$testVersion;tag="v$testVersion"} | ConvertTo-Json }
          else { "new $name" }
          $bytes = $script:Utf8.GetBytes($content)
          $output.Write($bytes, 0, $bytes.Length)
        }
      } finally { $output.Dispose() }
    }
  } finally { $zip.Dispose(); $file.Dispose() }
}
$fixtureZip = Join-Path $testRoot 'valid.zip'
New-Archive $fixtureZip
$script:TransportZip = $fixtureZip
$script:BadChecksum = $false
$script:ReceivedUrls = @()
function Receive-UpdateFile([string]$Uri, [string]$Destination, [long]$Limit, [string]$Stage = '') {
  $script:ReceivedUrls += $Uri
  if ($Uri.EndsWith('.sha256')) {
    $asset = ([uri]$Uri).Segments[-1] -replace '\.sha256$', ''
    $hash = if ($script:BadChecksum) { '0' * 64 } else { Get-FileDigest $script:TransportZip }
    [IO.File]::WriteAllText($Destination, "$hash  $asset`n")
  } else { [IO.File]::Copy($script:TransportZip, $Destination) }
}
function Prepare-Download($Fixture) {
  $Fixture.data.release.bytes = (Get-Item -LiteralPath $script:TransportZip).Length
  $Fixture.data.release.digest = Get-FileDigest $script:TransportZip
  Write-UpdateJson (Join-Path $Fixture.stage 'plan.json') $Fixture.data
  Get-UpdatePackage $Fixture.stage $Fixture.data
}
function Assert-UserData($Fixture) {
  foreach ($name in @('config/config.json', 'data/logs/pecofence.log', 'data/WebView2Profiles/default/user.txt', 'personal.txt')) {
    Assert-Test ([IO.File]::ReadAllText((Join-Path $Fixture.root $name)) -ceq 'preserve this user data') "Changed user data: $name"
  }
}

Run-Test 'Portable plans stay under their executable and reject changed identities' {
  $f = New-Fixture
  $null = Read-UpdatePlan (Join-Path $f.stage 'plan.json')
  $f.data.repository = 'Other/PecoFence'
  Write-UpdateJson (Join-Path $f.stage 'plan.json') $f.data
  Expect-Failure { Read-UpdatePlan (Join-Path $f.stage 'plan.json') }
  $outside = Join-Path $testRoot 'plan.json'
  Write-UpdateJson $outside $f.data
  Expect-Failure { Read-UpdatePlan $outside }
}
Run-Test 'Download verifies size, sidecar, API digest and complete portable payload' {
  $f = New-Fixture
  Prepare-Download $f
  Assert-VerifiedDownload $f.stage $f.data
  Assert-Test ((Get-ChildItem -LiteralPath (Join-Path $f.stage 'payload') -File).Count -eq 12) 'Incomplete payload'
  Assert-Test ($script:ReceivedUrls[-1].StartsWith("https://github.com/$testRepository/releases/download/")) 'Wrong release source'
  Assert-UserData $f
  [IO.File]::AppendAllText((Join-Path $f.stage $f.data.release.asset), 'corrupt')
  Expect-Failure { Assert-VerifiedDownload $f.stage $f.data }
}
Run-Test 'Conflicting checksums and truncated packages are rejected before installation' {
  $f = New-Fixture
  $script:BadChecksum = $true
  try { Expect-Failure { Prepare-Download $f } } finally { $script:BadChecksum = $false }
  Assert-Test (-not [IO.File]::Exists((Join-Path $f.stage 'verified.json'))) 'Bad checksum marked ready'
  $f = New-Fixture
  $f.data.release.bytes = (Get-Item -LiteralPath $fixtureZip).Length + 1
  Expect-Failure { Get-UpdatePackage $f.stage $f.data }
  Assert-UserData $f
}
Run-Test 'Archives cannot write user data, traverse paths, duplicate names or change repository/mode' {
  foreach ($extra in @('../escape.txt', 'config/config.json', 'data/user.txt', 'pecofence.exe', 'PECOFENCE.EXE', 'C:\escape.txt')) {
    $archive = Join-Path $testRoot ([guid]::NewGuid().ToString('N') + '.zip')
    New-Archive $archive ($script:ManagedFiles + $extra)
    $f = New-Fixture
    Expect-Failure { Expand-UpdatePackage $archive (Join-Path $f.stage 'bad-payload') $f.data }
    Assert-Test (-not [IO.Directory]::Exists((Join-Path $f.stage 'bad-payload'))) 'Invalid archive extracted files'
    Assert-UserData $f
  }
  foreach ($change in @(@{repo='Other/PecoFence';mode='portable'}, @{repo=$testRepository;mode='installed'})) {
    $archive = Join-Path $testRoot ([guid]::NewGuid().ToString('N') + '.zip')
    New-Archive $archive $script:ManagedFiles $change.repo $change.mode
    $f = New-Fixture
    Expect-Failure { Expand-UpdatePackage $archive (Join-Path $f.stage 'bad-payload') $f.data }
  }
}
Run-Test 'ZIP extraction enforces declared size while streaming' {
  $bytes = [IO.File]::ReadAllBytes($fixtureZip)
  $changed = $false
  # Our generated fixture has no ZIP comment; start at its central directory.
  Assert-Test ([BitConverter]::ToUInt32($bytes, $bytes.Length - 22) -eq 0x06054b50) 'Unexpected ZIP footer'
  $directoryOffset = [BitConverter]::ToUInt32($bytes, $bytes.Length - 6)
  for ($offset = $directoryOffset; $offset -lt $bytes.Length - 46; $offset++) {
    if ([BitConverter]::ToUInt32($bytes, $offset) -eq 0x02014b50) {
      $length = [BitConverter]::ToUInt16($bytes, $offset + 28)
      if ($length -eq 9 -and [Text.Encoding]::UTF8.GetString($bytes, $offset + 46, $length) -eq 'README.md') {
        [BitConverter]::GetBytes([uint32]4096).CopyTo($bytes, $offset + 24)
        $changed = $true
        break
      }
    }
  }
  Assert-Test $changed 'ZIP fixture header was not located'
  $archive = Join-Path $testRoot 'incorrect-uncompressed-size.zip'
  [IO.File]::WriteAllBytes($archive, $bytes)
  $f = New-Fixture
  Expect-Failure { Expand-UpdatePackage $archive (Join-Path $f.stage 'bad-payload') $f.data }
  Assert-UserData $f
}
Run-Test 'A portable update replaces only managed files and preserves data and unrelated files' {
  $f = New-Fixture
  Prepare-Download $f
  Expand-UpdatePackage (Join-Path $f.stage $f.data.release.asset) (Join-Path $f.stage 'apply-payload') $f.data
  Install-PortableUpdate $f.stage $f.data
  Assert-PackageIdentity $f.root $f.data $testVersion
  Assert-Test ((Read-SmallJson (Join-Path $f.stage 'transaction.json')).state -eq 'complete') 'Missing completion journal'
  Assert-Test ([IO.File]::ReadAllText((Join-Path $f.root 'README.md')) -ceq 'new README.md') 'Program files did not change'
  Assert-UserData $f
}
$script:RealReplace = ${function:Replace-ManagedFile}
$script:RealRestore = ${function:Restore-PortableUpdate}
Run-Test 'A replacement failure rolls back all managed program files' {
  $f = New-Fixture
  Prepare-Download $f
  Expand-UpdatePackage (Join-Path $f.stage $f.data.release.asset) (Join-Path $f.stage 'apply-payload') $f.data
  $script:ReplaceCalls = 0
  function Replace-ManagedFile([string]$From, [string]$To, [string]$Attempt) {
    $script:ReplaceCalls++
    if ($script:ReplaceCalls -eq 8) { throw 'Injected disk-write failure' }
    & $script:RealReplace $From $To $Attempt
  }
  Expect-Failure { Install-PortableUpdate $f.stage $f.data }
  Assert-PackageIdentity $f.root $f.data $priorVersion
  Assert-Test ((Read-SmallJson (Join-Path $f.stage 'transaction.json')).state -eq 'rolledBack') 'Rollback did not complete'
  Assert-Test ([IO.File]::ReadAllText((Join-Path $f.root 'README.md')) -ceq 'old README.md') 'Rollback missed a file'
  Assert-UserData $f
}
Run-Test 'Interrupted replacement can be recovered; corrupt backups are never restored' {
  $f = New-Fixture
  Prepare-Download $f
  Expand-UpdatePackage (Join-Path $f.stage $f.data.release.asset) (Join-Path $f.stage 'apply-payload') $f.data
  $script:ReplaceCalls = 0
  function Replace-ManagedFile([string]$From, [string]$To, [string]$Attempt) {
    $script:ReplaceCalls++
    if ($script:ReplaceCalls -eq 8) { throw 'Injected interruption' }
    & $script:RealReplace $From $To $Attempt
  }
  function Restore-PortableUpdate { throw 'Simulated worker termination before rollback' }
  Expect-Failure { Install-PortableUpdate $f.stage $f.data }
  Assert-Test ((Read-SmallJson (Join-Path $f.stage 'transaction.json')).state -eq 'recoveryRequired') 'Missing persistent recovery state'
  & $script:RealRestore $f.stage $f.data
  Assert-PackageIdentity $f.root $f.data $priorVersion
  Assert-UserData $f
  $journal = Read-SmallJson (Join-Path $f.stage 'transaction.json')
  $journal.state = 'applying'
  Write-UpdateJson (Join-Path $f.stage 'transaction.json') $journal
  [IO.File]::AppendAllText((Join-Path $f.stage 'backup/README.md'), 'corrupt')
  Expect-Failure { & $script:RealRestore $f.stage $f.data }
  Assert-UserData $f
}
Run-Test 'Unregistered installed copies cannot launch setup' {
  Expect-Failure { Assert-InstalledLocation (Join-Path $testRoot 'not-an-installation') }
}

# Fresh OS processes deliberately lose every in-memory variable. The checkpoint
# override exists only in this test harness, never in the embedded production worker.
$crashHarness = Join-Path $testRoot 'crash-harness.ps1'
[IO.File]::WriteAllText($crashHarness, @'
param([string]$Worker, [string]$PlanFile, [string]$SignalFile, [string]$Phase, [int]$Checkpoint)
$ErrorActionPreference = 'Stop'
. $Worker
$stage = [IO.Path]::GetDirectoryName($PlanFile)
$data = Read-UpdatePlan $PlanFile $false
$script:OriginalReplace = ${function:Replace-ManagedFile}
$script:OriginalWrite = ${function:Write-UpdateJson}
$script:OriginalCopy = ${function:Copy-Durable}
$script:Count = 0
function Stop-AtCheckpoint {
  [IO.File]::WriteAllText($SignalFile, 'checkpoint reached')
  while ($true) { Start-Sleep -Milliseconds 200 }
}
if ($Phase -eq 'recover') {
  Restore-PortableUpdate $stage $data
} elseif ($Phase -in @('setup-crash', 'setup-recover')) {
  function Get-DesktopLocks { return @() }
  function Assert-InstalledLocation([string]$Root) {
    if ($Root -cne $data.root) { throw 'Unexpected installed target' }
  }
  function Start-UpdatedApp([string]$Root) { }
  function Start-UpdateInstaller([string]$Stage, $Data) {
    if ($Phase -eq 'setup-crash') { Stop-AtCheckpoint }
    Assert-VerifiedDownload $Stage $Data
    Write-UpdateJson (Join-Path $Stage 'setup-launched.json') @{ root=$Data.root; package=$Data.release.asset }
    return 0
  }
  $operation = if ($Phase -eq 'setup-crash') { 'Apply' } else { 'Recover' }
  Invoke-UpdateWorker $operation $PlanFile
} else {
  function Write-UpdateJson([string]$Path, $Value) {
    & $script:OriginalWrite $Path $Value
    if ($Phase -eq 'journal' -and $Path.EndsWith('transaction.json') -and $Value.state -eq 'applying') { Stop-AtCheckpoint }
  }
  function Copy-Durable([string]$From, [string]$To) {
    & $script:OriginalCopy $From $To
    if ($Phase -eq 'backup' -and $To.EndsWith('backup\release-info.json')) { Stop-AtCheckpoint }
  }
  function Replace-ManagedFile([string]$From, [string]$To, [string]$Attempt) {
    $script:Count++
    if ($Phase -eq 'rollback' -and $script:Count -eq 8) { throw 'Begin rollback' }
    & $script:OriginalReplace $From $To $Attempt
    if (($Phase -eq 'replace' -and $script:Count -eq $Checkpoint) -or ($Phase -eq 'rollback' -and $script:Count -eq 9)) { Stop-AtCheckpoint }
  }
  Install-PortableUpdate $stage $data
}
'@)

function Start-CrashHarness($Fixture, [string]$Phase, [int]$Checkpoint = 0) {
  $start = New-Object Diagnostics.ProcessStartInfo
  $start.FileName = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
  $start.Arguments = '-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "' + $crashHarness + '" -Worker "' + (Join-Path $PSScriptRoot 'runtime/update-worker.ps1') + '" -PlanFile "' + (Join-Path $Fixture.stage 'plan.json') + '" -SignalFile "' + (Join-Path $Fixture.stage 'crash-checkpoint.txt') + '" -Phase ' + $Phase + ' -Checkpoint ' + $Checkpoint
  $start.UseShellExecute = $false
  $start.CreateNoWindow = $true
  $start.RedirectStandardError = $true
  $start.RedirectStandardOutput = $true
  return [Diagnostics.Process]::Start($start)
}
function Kill-AtCheckpoint($Fixture, [string]$Phase, [int]$Checkpoint = 0) {
  $child = Start-CrashHarness $Fixture $Phase $Checkpoint
  try {
    $deadline = [DateTime]::UtcNow.AddSeconds(45)
    while (-not [IO.File]::Exists((Join-Path $Fixture.stage 'crash-checkpoint.txt'))) {
      if ($child.HasExited) { throw ('Crash fixture exited early: ' + $child.StandardError.ReadToEnd()) }
      if ([DateTime]::UtcNow -ge $deadline) { throw 'Crash checkpoint timed out' }
      Start-Sleep -Milliseconds 50
    }
    # Kill only the exact child created by this test. No app or machine shutdown.
    $child.Kill()
    Assert-Test ($child.WaitForExit(10000)) 'Fixture process did not terminate'
  } finally { if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit() }; $child.Dispose() }
}
function Complete-InFreshProcess($Fixture, [string]$Phase) {
  $child = Start-CrashHarness $Fixture $Phase
  try {
    Assert-Test ($child.WaitForExit(30000)) 'Recovery process timed out'
    Assert-Test ($child.ExitCode -eq 0) ('Recovery failed: ' + $child.StandardError.ReadToEnd())
  } finally { if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit() }; $child.Dispose() }
}

foreach ($scenario in @(@{phase='backup';at=0}, @{phase='journal';at=0}, @{phase='replace';at=1}, @{phase='replace';at=6}, @{phase='replace';at=12}, @{phase='rollback';at=0})) {
  Run-Test "Forced termination and fresh-process recovery: $($scenario.phase) $($scenario.at)" {
    $f = New-Fixture
    $originalHashes = @{}
    foreach ($name in $script:ManagedFiles) { $originalHashes[$name] = Get-FileDigest (Join-Path $f.root $name) }
    Prepare-Download $f
    Expand-UpdatePackage (Join-Path $f.stage $f.data.release.asset) (Join-Path $f.stage 'apply-payload') $f.data
    Kill-AtCheckpoint $f $scenario.phase $scenario.at
    if ($scenario.phase -ne 'backup') {
      Assert-Test ((Read-SmallJson (Join-Path $f.stage 'transaction.json')).state -eq 'applying') 'Durable journal was not retained'
      if ($scenario.phase -eq 'replace' -and $scenario.at -eq 6) {
        # Model a missing directory entry as well as a mixed old/new file set.
        [IO.File]::Delete((Join-Path $f.root 'README.md'))
      }
      Complete-InFreshProcess $f 'recover'
      Assert-Test ((Read-SmallJson (Join-Path $f.stage 'transaction.json')).state -eq 'rolledBack') 'Fresh recovery did not complete'
    } else { Assert-Test (-not [IO.File]::Exists((Join-Path $f.stage 'transaction.json'))) 'Premature mutation journal' }
    foreach ($name in $script:ManagedFiles) { Assert-Test ((Get-FileDigest (Join-Path $f.root $name)) -ceq $originalHashes[$name]) "Did not recover original bytes: $name" }
    Assert-UserData $f
  }
}

Run-Test 'Interrupted setup reuses the verified installer in a fresh process, without copying installed files' {
  $savedLocalAppData = $env:LOCALAPPDATA
  try {
    $env:LOCALAPPDATA = Join-Path $testRoot 'fake-profile'
    $f = New-Fixture 'installed'
    $package = Join-Path $f.stage $f.data.release.asset
    [IO.File]::Copy((Join-Path $releaseRoot 'pecofence.exe'), $package)
    $f.data.release.bytes = (Get-Item -LiteralPath $package).Length
    $f.data.release.digest = Get-FileDigest $package
    Write-UpdateJson (Join-Path $f.stage 'plan.json') $f.data
    Write-UpdateJson (Join-Path $f.stage 'verified.json') @{ asset=$f.data.release.asset; sha256=$f.data.release.digest }
    Kill-AtCheckpoint $f 'setup-crash'
    Assert-Test ((Read-SmallJson (Join-Path $f.stage 'installer.json')).state -eq 'installing') 'Missing setup journal'
    Complete-InFreshProcess $f 'setup-recover'
    Assert-Test ((Read-SmallJson (Join-Path $f.stage 'installer.json')).state -eq 'complete') 'Setup recovery did not finish'
    Assert-Test ((Read-SmallJson (Join-Path $f.stage 'setup-launched.json')).root -ceq $f.root) 'Recovery chose a different installation'
    Assert-Test (-not [IO.Directory]::Exists((Join-Path $f.stage 'backup'))) 'Installer copy was treated as portable'
    Assert-UserData $f
  } finally { $env:LOCALAPPDATA = $savedLocalAppData }
}
Run-Test 'Redirects through a directory junction are rejected before writing' {
  $f = New-Fixture
  $other = Join-Path $testRoot 'junction-target'
  $null = [IO.Directory]::CreateDirectory($other)
  $link = Join-Path $f.root 'linked'
  $null = New-Item -ItemType Junction -Path $link -Target $other
  Expect-Failure { Assert-PlainPath (Join-Path $link 'file.json') }
  Assert-Test (-not [IO.File]::Exists((Join-Path $other 'file.json'))) 'A write followed the junction'
}

. (Join-Path $PSScriptRoot 'test-update-cleanup.ps1')

Write-UpdateJson (Join-Path $testRoot 'report.json') @{ passed=$results.Count; tests=@($results); network='offline fixtures'; actualInstallerLaunched=$false }
Write-Output "$($results.Count) updater tests passed. Report: $testRoot\report.json"
