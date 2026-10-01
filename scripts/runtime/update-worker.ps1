# Embedded in pecofence.exe. Windows PowerShell 5.1 / .NET Framework only.
# Inputs are JSON data, never interpolated into executable PowerShell expressions.
[CmdletBinding()]
param(
  [ValidateSet('Check', 'Download', 'Apply', 'Recover', 'Cleanup')][string]$Action,
  [string]$Plan,
  [int]$ParentPid = 0
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:ManagedFiles = @(
  'pecofence.exe', 'pecofence-watchdog.exe', 'pecofence-cli.exe', 'WebView2Loader.dll',
  'deployment.json', 'release-info.json', 'README.md', 'UPGRADING.md', 'SKILL.md',
  'LICENSE', 'LICENSE-WebView2Loader.txt', 'THIRD-PARTY-LICENSES.txt'
)
$script:Utf8 = New-Object System.Text.UTF8Encoding($false)

function Assert-PlainPath([string]$Path) {
  if (-not [IO.Path]::IsPathRooted($Path) -or $Path.Contains('..\') -or $Path.Contains('../')) { throw 'An absolute path without parent traversal is required.' }
  $full = [IO.Path]::GetFullPath($Path)
  $part = $full
  while ($part) {
    if ([IO.File]::Exists($part) -or [IO.Directory]::Exists($part)) {
      if (([IO.File]::GetAttributes($part) -band [IO.FileAttributes]::ReparsePoint) -ne 0) { throw "Update paths cannot contain links or junctions: $part" }
    }
    $parent = [IO.Path]::GetDirectoryName($part)
    if ($parent -eq $part) { break }
    $part = $parent
  }
  return $full.TrimEnd('\', '/')
}

function Read-SmallJson([string]$Path, [long]$Limit = 16384) {
  $null = Assert-PlainPath $Path
  if ((Get-Item -LiteralPath $Path).Length -gt $Limit) { throw "JSON file is too large: $Path" }
  return ([IO.File]::ReadAllText($Path) | ConvertFrom-Json)
}

function Write-UpdateJson([string]$Path, $Value) {
  $null = Assert-PlainPath $Path
  $temp = "$Path.tmp"
  $null = Assert-PlainPath $temp
  $bytes = $script:Utf8.GetBytes(($Value | ConvertTo-Json -Depth 12 -Compress))
  $file = [IO.File]::Open($temp, [IO.FileMode]::Create, [IO.FileAccess]::Write, [IO.FileShare]::None)
  try { $file.Write($bytes, 0, $bytes.Length); $file.Flush($true) } finally { $file.Dispose() }
  if ([IO.File]::Exists($Path)) { [IO.File]::Replace($temp, $Path, [NullString]::Value) }
  else { [IO.File]::Move($temp, $Path) }
}

function Get-FileDigest([string]$Path) {
  $null = Assert-PlainPath $Path
  $hash = [Security.Cryptography.SHA256]::Create()
  $file = [IO.File]::OpenRead($Path)
  try { return ([BitConverter]::ToString($hash.ComputeHash($file))).Replace('-', '').ToLowerInvariant() }
  finally { $file.Dispose(); $hash.Dispose() }
}

function Assert-Repository([string]$Repository) {
  if ($Repository -cnotmatch '\A[A-Za-z0-9_.-]{1,100}/[A-Za-z0-9_.-]{1,100}\z' -or @($Repository.Split('/') | Where-Object { $_ -eq '.' -or $_ -eq '..' }).Count) { throw 'Invalid package repository.' }
}

function Assert-PackageIdentity([string]$Root, $Data, [string]$Version) {
  $marker = Read-SmallJson (Join-Path $Root 'deployment.json')
  $info = Read-SmallJson (Join-Path $Root 'release-info.json')
  if ($marker.schema -ne 1 -or $marker.appId -cne 'PecoFence' -or $marker.mode -cne $Data.mode -or @($marker.PSObject.Properties).Count -ne 3) { throw 'The deployment marker changed or belongs to another distribution.' }
  if ($info.schema -ne 1 -or $info.repository -cne $Data.repository -or $info.version -cne $Version -or $info.tag -cne "v$Version" -or @($info.PSObject.Properties).Count -ne 4) { throw 'The package repository/version changed. Check for updates again.' }
}

function Read-UpdatePlan([string]$Path, [bool]$RequireCurrent = $true) {
  $pathFull = Assert-PlainPath $Path
  $data = Read-SmallJson $pathFull
  if ($data.schema -ne 1 -or $data.mode -cnotin @('portable', 'installed')) { throw 'Unsupported update plan.' }
  Assert-Repository $data.repository
  if ($data.currentVersion -cnotmatch '\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)([-+][A-Za-z0-9.+-]+)?\z') { throw 'Invalid current version.' }
  $root = Assert-PlainPath $data.root
  if ($root -cne $data.root) { throw 'The application root is not canonical.' }
  $updates = if ($data.mode -eq 'portable') { Join-Path $root 'data\updates' } else { Join-Path $env:LOCALAPPDATA 'PecoFence\updates' }
  $updates = Assert-PlainPath $updates
  $stage = [IO.Path]::GetDirectoryName($pathFull)
  if ([IO.Path]::GetDirectoryName($stage) -ine $updates -or [IO.Path]::GetFileName($stage) -cnotmatch '\A[0-9a-f]{32}\z' -or [IO.Path]::GetFileName($pathFull) -cne 'plan.json') { throw 'The update plan is outside its application data directory.' }
  if ($RequireCurrent) { Assert-PackageIdentity $root $data $data.currentVersion }
  return $data
}

function Assert-ReleasePlan($Data) {
  $release = $Data.release
  if ($release.version -cnotmatch '\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\z') { throw 'Only stable releases can be installed.' }
  $current = [version]($Data.currentVersion -split '[-+]', 2)[0]
  $remote = [version]$release.version
  if ($remote -lt $current -or ($remote -eq $current -and -not $Data.currentVersion.Contains('-'))) { throw 'Updates cannot install the same or an older version.' }
  $suffix = if ($Data.mode -eq 'portable') { 'portable.zip' } else { 'setup.exe' }
  $asset = "pecofence-v$($release.version)-x64-$suffix"
  $base = "https://github.com/$($Data.repository)/releases"
  if ($release.asset -cne $asset -or $release.url -cne "$base/download/v$($release.version)/$asset" -or $release.checksumUrl -cne "$base/download/v$($release.version)/$asset.sha256" -or $release.page -cne "$base/tag/v$($release.version)" -or $release.bytes -le 0 -or $release.bytes -gt 128MB) { throw 'Invalid update artifact identity, URL or size.' }
  if ($null -ne $release.digest -and $release.digest -cnotmatch '\A[0-9a-f]{64}\z') { throw 'Invalid GitHub digest.' }
}

function Write-UpdateProgress([string]$Stage, [string]$Phase, [long]$Received, [long]$Total) {
  Write-UpdateJson (Join-Path $Stage 'progress.json') @{ phase = $Phase; received = $Received; total = $Total }
}

# Follow only HTTPS redirects to GitHub's release asset hosts. No credentials,
# cookies, release-provided commands, server filenames or custom endpoints are used.
function Receive-UpdateFile([string]$Uri, [string]$Destination, [long]$Limit, [string]$Stage = '') {
  $uriValue = [uri]$Uri
  $started = [Diagnostics.Stopwatch]::StartNew()
  for ($redirect = 0; $redirect -le 5; $redirect++) {
    if ($uriValue.Scheme -ne 'https' -or -not $uriValue.IsDefaultPort -or $uriValue.UserInfo -or $uriValue.Fragment -or $uriValue.Host -notin @('api.github.com', 'github.com', 'release-assets.githubusercontent.com', 'objects.githubusercontent.com')) { throw 'Untrusted update redirect.' }
    $request = [Net.HttpWebRequest]::Create($uriValue)
    $request.AllowAutoRedirect = $false
    $request.Timeout = 15000
    $request.ReadWriteTimeout = 15000
    $request.UserAgent = 'PecoFence-Updater/1'
    $request.Accept = if ($uriValue.Host -eq 'api.github.com') { 'application/vnd.github+json' } else { 'application/octet-stream' }
    if ($uriValue.Host -eq 'api.github.com') { $request.Headers.Add('X-GitHub-Api-Version', '2022-11-28') }
    $response = $null
    try {
      $response = $request.GetResponse()
      $status = [int]$response.StatusCode
      if ($status -in @(301, 302, 303, 307, 308)) {
        $uriValue = New-Object Uri($uriValue, $response.Headers['Location'])
        continue
      }
      if ($status -ne 200 -or $response.ContentLength -gt $Limit) { throw 'Unexpected HTTP status or oversized response.' }
      $null = Assert-PlainPath $Destination
      $stream = $response.GetResponseStream()
      $file = [IO.File]::Open($Destination, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
      try {
        $buffer = New-Object byte[] 65536
        $received = 0L
        $lastProgress = 0L
        while (($count = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) {
          $received += $count
          if ($received -gt $Limit -or $started.Elapsed.TotalMinutes -gt 5) { throw 'Update transfer exceeded its size or time limit.' }
          $file.Write($buffer, 0, $count)
          if ($Stage -and ($started.ElapsedMilliseconds - $lastProgress -ge 200)) {
            Write-UpdateProgress $Stage 'downloading' $received $Limit
            $lastProgress = $started.ElapsedMilliseconds
          }
        }
        $file.Flush($true)
      } finally { $file.Dispose(); $stream.Dispose() }
      return
    } finally { if ($null -ne $response) { $response.Dispose() }; $request.Abort() }
  }
  throw 'Too many update redirects.'
}

function Assert-ExecutableVersion([string]$Path, [string]$Version) {
  $found = [Diagnostics.FileVersionInfo]::GetVersionInfo($Path).ProductVersion
  if ($found -cne $Version) { throw "Executable version mismatch: $Path" }
}

function Expand-UpdatePackage([string]$Archive, [string]$Destination, $Data) {
  Add-Type -AssemblyName System.IO.Compression
  $null = Assert-PlainPath $Destination
  if ([IO.Directory]::Exists($Destination)) { throw 'The extraction directory already exists.' }
  $file = [IO.File]::OpenRead($Archive)
  $zip = New-Object IO.Compression.ZipArchive($file, [IO.Compression.ZipArchiveMode]::Read)
  try {
    $seen = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
    $total = 0L
    foreach ($entry in $zip.Entries) {
      $name = $entry.FullName
      $kind = (($entry.ExternalAttributes -shr 16) -band 0xF000)
      if ($name -cnotin $script:ManagedFiles -or -not $seen.Add($name) -or $entry.Length -le 0 -or $entry.Length -gt 128MB -or $kind -notin @(0, 0x8000) -or ($entry.ExternalAttributes -band 0x400) -ne 0) { throw "Unexpected, duplicate, linked or invalid ZIP entry: $name" }
      $total += $entry.Length
      if ($total -gt 256MB) { throw 'Uncompressed package is too large.' }
    }
    if ($seen.Count -ne $script:ManagedFiles.Count) { throw 'The portable package is incomplete.' }
    $null = [IO.Directory]::CreateDirectory($Destination)
    foreach ($entry in $zip.Entries) {
      $output = [IO.File]::Open((Join-Path $Destination $entry.FullName), [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
      $inputStream = $entry.Open()
      try {
        $buffer = New-Object byte[] 65536
        $written = 0L
        while (($count = $inputStream.Read($buffer, 0, $buffer.Length)) -gt 0) {
          $written += $count
          if ($written -gt $entry.Length) { throw 'ZIP content exceeded its declared size.' }
          $output.Write($buffer, 0, $count)
        }
        if ($written -ne $entry.Length) { throw 'ZIP content was truncated.' }
        $output.Flush($true)
      }
      finally { $output.Dispose(); $inputStream.Dispose() }
    }
  } finally { $zip.Dispose(); $file.Dispose() }
  Assert-PackageIdentity $Destination $Data $Data.release.version
  foreach ($name in @('pecofence.exe', 'pecofence-watchdog.exe', 'pecofence-cli.exe')) { Assert-ExecutableVersion (Join-Path $Destination $name) $Data.release.version }
}

function Assert-VerifiedDownload([string]$Stage, $Data) {
  Assert-ReleasePlan $Data
  $verified = Read-SmallJson (Join-Path $Stage 'verified.json')
  $package = Join-Path $Stage $Data.release.asset
  if ($verified.sha256 -cnotmatch '\A[0-9a-f]{64}\z' -or $verified.asset -cne $Data.release.asset -or (Get-Item -LiteralPath $package).Length -ne $Data.release.bytes -or (Get-FileDigest $package) -cne $verified.sha256 -or ($null -ne $Data.release.digest -and $Data.release.digest -cne $verified.sha256)) { throw 'The downloaded package changed or failed verification.' }
}

function Get-UpdatePackage([string]$Stage, $Data) {
  Assert-ReleasePlan $Data
  Write-UpdateProgress $Stage 'downloading' 0 $Data.release.bytes
  $hashPath = Join-Path $Stage 'checksum.txt'
  Receive-UpdateFile $Data.release.checksumUrl $hashPath 4096
  $text = [IO.File]::ReadAllText($hashPath).Trim()
  $match = [regex]::Match($text, '\A([a-fA-F0-9]{64})[ \t]+\*?' + [regex]::Escape($Data.release.asset) + '\z')
  if (-not $match.Success) { throw 'The SHA-256 file does not identify this package.' }
  $expected = $match.Groups[1].Value.ToLowerInvariant()
  if ($null -ne $Data.release.digest -and $Data.release.digest -cne $expected) { throw 'GitHub and the checksum file disagree.' }
  $package = Join-Path $Stage $Data.release.asset
  Receive-UpdateFile $Data.release.url $package $Data.release.bytes $Stage
  Write-UpdateProgress $Stage 'verifying' $Data.release.bytes $Data.release.bytes
  if ((Get-Item -LiteralPath $package).Length -ne $Data.release.bytes -or (Get-FileDigest $package) -cne $expected) { throw 'The package size or SHA-256 did not match.' }
  if ($Data.mode -eq 'portable') {
    Write-UpdateProgress $Stage 'extracting' $Data.release.bytes $Data.release.bytes
    Expand-UpdatePackage $package (Join-Path $Stage 'payload') $Data
  } else { Assert-ExecutableVersion $package $Data.release.version }
  Write-UpdateJson (Join-Path $Stage 'verified.json') @{ asset = $Data.release.asset; sha256 = $expected }
}

function Get-DesktopLocks {
  $locks = New-Object 'Collections.Generic.List[Threading.Mutex]'
  try {
    foreach ($name in @('Local\PecoFence.SingleInstance', 'Local\openFence.SingleInstance')) {
      $created = $false
      $mutex = New-Object Threading.Mutex($false, $name, [ref]$created)
      $locks.Add($mutex)
      if (-not $created) { throw 'Another PecoFence copy is running. Close it before updating or recovering.' }
    }
    return ,$locks
  } catch { foreach ($mutex in $locks) { $mutex.Dispose() }; throw }
}

function Assert-InstalledLocation([string]$Root) {
  $registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::CurrentUser, [Microsoft.Win32.RegistryView]::Registry64)
  $key = $null
  try {
    $key = $registry.OpenSubKey('Software\Microsoft\Windows\CurrentVersion\Uninstall\DayuanJiang.PecoFence_is1')
    if ($null -eq $key -or ([string]$key.GetValue('InstallLocation', '')).TrimEnd('\') -ine $Root) { throw 'This directory is not the registered current-user installation.' }
  } finally { if ($null -ne $key) { $key.Dispose() }; $registry.Dispose() }
}

function Copy-Durable([string]$From, [string]$To) {
  $null = Assert-PlainPath $From
  $null = Assert-PlainPath $To
  $inputStream = [IO.File]::OpenRead($From)
  $output = $null
  try {
    $output = [IO.File]::Open($To, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    $inputStream.CopyTo($output); $output.Flush($true)
  } finally { $inputStream.Dispose(); if ($null -ne $output) { $output.Dispose() } }
}

function Replace-ManagedFile([string]$From, [string]$To, [string]$Attempt) {
  $temporary = "$To.update-$Attempt"
  $null = Assert-PlainPath $temporary
  if ([IO.File]::Exists($temporary)) { [IO.File]::Delete($temporary) }
  try {
    Copy-Durable $From $temporary
    if ([IO.File]::Exists($To)) { [IO.File]::Replace($temporary, $To, [NullString]::Value) }
    else { [IO.File]::Move($temporary, $To) }
  }
  finally { if ([IO.File]::Exists($temporary)) { [IO.File]::Delete($temporary) } }
}

function Restore-PortableUpdate([string]$Stage, $Data) {
  if ($Data.mode -cne 'portable') { throw 'File recovery is only available for portable copies.' }
  $journalPath = Join-Path $Stage 'transaction.json'
  $journal = Read-SmallJson $journalPath
  if ($journal.state -cnotin @('applying', 'recoveryRequired') -or $journal.root -cne $Data.root -or @($journal.files).Count -ne $script:ManagedFiles.Count) { throw 'Invalid recovery journal.' }
  $seen = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
  # Verify every backup before changing any destination; never trust paths from JSON.
  foreach ($item in $journal.files) {
    if ($item.name -cnotin $script:ManagedFiles -or -not $seen.Add($item.name) -or $item.sha256 -cnotmatch '\A[0-9a-f]{64}\z') { throw 'Invalid recovery file.' }
    if ((Get-FileDigest (Join-Path $Stage "backup\$($item.name)")) -cne $item.sha256) { throw 'A recovery backup has changed. Files have been retained for manual recovery.' }
  }
  Assert-PackageIdentity (Join-Path $Stage 'backup') $Data $Data.currentVersion
  foreach ($item in $journal.files) {
    $target = Join-Path $Data.root $item.name
    if (-not [IO.File]::Exists($target) -or (Get-FileDigest $target) -cne $item.sha256) { Replace-ManagedFile (Join-Path $Stage "backup\$($item.name)") $target ([IO.Path]::GetFileName($Stage)) }
  }
  $journal.state = 'rolledBack'
  Write-UpdateJson $journalPath $journal
}

function Install-PortableUpdate([string]$Stage, $Data) {
  $backup = Join-Path $Stage 'backup'
  if ([IO.Directory]::Exists($backup)) { throw 'An existing backup must be recovered before retrying.' }
  $null = [IO.Directory]::CreateDirectory($backup)
  $files = @()
  foreach ($name in $script:ManagedFiles) {
    $target = Join-Path $Data.root $name
    $null = Assert-PlainPath $target
    # Includes helpers still leaving after the application exits. Never terminate them.
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while ($true) {
      try { $probe = [IO.File]::Open($target, [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None); $probe.Dispose(); break }
      catch { if ([DateTime]::UtcNow -ge $deadline) { throw }; Start-Sleep -Milliseconds 200 }
    }
    Copy-Durable $target (Join-Path $backup $name)
    $files += @{ name = $name; sha256 = (Get-FileDigest (Join-Path $backup $name)) }
  }
  $journalPath = Join-Path $Stage 'transaction.json'
  Write-UpdateJson $journalPath @{ state = 'applying'; root = $Data.root; files = $files }
  try {
    foreach ($name in $script:ManagedFiles) { Replace-ManagedFile (Join-Path $Stage "apply-payload\$name") (Join-Path $Data.root $name) ([IO.Path]::GetFileName($Stage)) }
    Assert-PackageIdentity $Data.root $Data $Data.release.version
    Write-UpdateJson $journalPath @{ state = 'complete'; root = $Data.root; files = $files }
  } catch {
    $originalError = $_
    try { Restore-PortableUpdate $Stage $Data }
    catch { Write-UpdateJson $journalPath @{ state = 'recoveryRequired'; root = $Data.root; files = $files }; throw "Update and recovery failed. Keep $Stage for recovery. $($_.Exception.Message)" }
    throw "The update failed and the original program files were restored. $($originalError.Exception.Message)"
  }
}

function Start-UpdatedApp([string]$Root) {
  $start = New-Object Diagnostics.ProcessStartInfo
  $start.FileName = Join-Path $Root 'pecofence.exe'
  $start.Arguments = '--open-settings'
  $start.WorkingDirectory = $Root
  $start.UseShellExecute = $true
  $process = [Diagnostics.Process]::Start($start)
  $process.Dispose()
}

function Start-UpdateInstaller([string]$Stage, $Data) {
  Assert-InstalledLocation $Data.root
  Assert-ExecutableVersion (Join-Path $Stage $Data.release.asset) $Data.release.version
  $start = New-Object Diagnostics.ProcessStartInfo
  $start.FileName = Join-Path $Stage $Data.release.asset
  $start.Arguments = '/NORESTART /DIR="' + $Data.root + '" /LOG="' + (Join-Path $Stage 'setup.log') + '"'
  $start.WorkingDirectory = $Stage
  $start.UseShellExecute = $true
  $process = [Diagnostics.Process]::Start($start)
  try { $process.WaitForExit(); return $process.ExitCode } finally { $process.Dispose() }
}

function Invoke-UpdateWorker([string]$Operation, [string]$PlanPath, [int]$WaitForPid = 0) {
  $stage = [IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($PlanPath))
  $data = Read-UpdatePlan $PlanPath ($Operation -ne 'Recover')
  $lockPath = Assert-PlainPath (Join-Path ([IO.Path]::GetDirectoryName($stage)) 'update.lock')
  $lock = [IO.File]::Open($lockPath, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
  $desktopLocks = @()
  $parentProcess = $null
  try {
    if ($Operation -eq 'Cleanup') {
      if ($WaitForPid -le 0) { throw 'Cleanup requires a running application.' }
      $parentProcess = [Diagnostics.Process]::GetProcessById($WaitForPid)
      if ($parentProcess.MainModule.FileName -ine (Join-Path $data.root 'pecofence.exe') -or ([DateTime]::UtcNow - $parentProcess.StartTime.ToUniversalTime()).TotalSeconds -lt 30) { throw 'Cleanup requires a matching application that has started successfully.' }
      Assert-ExecutableVersion $parentProcess.MainModule.FileName $data.currentVersion
      . (Join-Path $PSScriptRoot 'update-cleanup.ps1')
      $report = Invoke-UpdateCleanup $stage $data
      Write-UpdateJson (Join-Path $stage 'result.json') @{status='ok';cleanup=$report}
      return
    } elseif ($Operation -eq 'Check') {
      try { Receive-UpdateFile "https://api.github.com/repos/$($data.repository)/releases/latest" (Join-Path $stage 'release.json') 2MB }
      catch {
        # PowerShell wraps .NET method failures in MethodInvocationException.
        $failure = $_.Exception
        while ($null -ne $failure.InnerException) { $failure = $failure.InnerException }
        if ($failure -is [Net.WebException] -and $null -ne $failure.Response -and [int]$failure.Response.StatusCode -eq 404) {
          $failure.Response.Dispose()
          Write-UpdateJson (Join-Path $stage 'result.json') @{ status = 'noRelease' }; return
        }
        throw
      }
    } elseif ($Operation -eq 'Download') {
      Get-UpdatePackage $stage $data
    } else {
      if ($Operation -eq 'Apply' -or $data.mode -eq 'installed') {
        Assert-VerifiedDownload $stage $data
        if ($data.mode -eq 'portable') { Expand-UpdatePackage (Join-Path $stage $data.release.asset) (Join-Path $stage 'apply-payload') $data }
        else { Assert-InstalledLocation $data.root }
      }
      # Hold this process handle across the handoff (PID reuse cannot change it).
      if ($WaitForPid -gt 0) {
        $parentProcess = [Diagnostics.Process]::GetProcessById($WaitForPid)
        if ($parentProcess.MainModule.FileName -ine (Join-Path $data.root 'pecofence.exe')) { throw 'The parent process is not this PecoFence copy.' }
      }
      Write-UpdateJson (Join-Path $stage 'handoff.json') @{ status = 'ready' }
      if ($null -ne $parentProcess -and -not $parentProcess.WaitForExit(120000)) { throw 'PecoFence did not exit. No program files were changed.' }
      $desktopLocks = Get-DesktopLocks
      if ($Operation -eq 'Recover' -and $data.mode -eq 'portable') { Restore-PortableUpdate $stage $data }
      elseif ($data.mode -eq 'portable') {
        Assert-PackageIdentity $data.root $data $data.currentVersion
        Install-PortableUpdate $stage $data
      } else {
        foreach ($mutex in $desktopLocks) { $mutex.Dispose() }; $desktopLocks = @()
        # A killed worker/OS leaves this durable record. Recovery re-runs the
        # verified setup for this registered location; it never copies files over
        # an installer-managed product or rewrites its uninstall registration.
        Write-UpdateJson (Join-Path $stage 'installer.json') @{ state = 'installing'; root = $data.root }
        $exitCode = Start-UpdateInstaller $stage $data
        if ($exitCode -ne 0) {
          Write-UpdateJson (Join-Path $stage 'installer.json') @{ state = 'interrupted'; root = $data.root }
          throw "Setup was cancelled or failed (exit code $exitCode)."
        }
        Write-UpdateJson (Join-Path $stage 'installer.json') @{ state = 'complete'; root = $data.root }
      }
      foreach ($mutex in $desktopLocks) { $mutex.Dispose() }; $desktopLocks = @()
      Start-UpdatedApp $data.root
    }
    Write-UpdateJson (Join-Path $stage 'result.json') @{ status = 'ok' }
  } finally {
    foreach ($mutex in $desktopLocks) { $mutex.Dispose() }
    if ($null -ne $parentProcess) { $parentProcess.Dispose() }
    $lock.Dispose()
  }
}

# Dot sourcing exposes the real functions for offline fixtures without changing
# production endpoints, application identity, or introducing a test bypass.
if ($MyInvocation.InvocationName -ne '.') {
  try { Invoke-UpdateWorker $Action $Plan $ParentPid }
  catch {
    $message = $_.Exception.Message
    try {
      $null = Read-UpdatePlan $Plan $false
      $stage = [IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($Plan))
      Write-UpdateJson (Join-Path $stage 'result.json') @{ status = 'error'; message = $message }
      if ($Action -in @('Apply', 'Recover') -and [IO.File]::Exists((Join-Path $stage 'handoff.json'))) {
        Add-Type -AssemblyName System.Windows.Forms
        $null = [Windows.Forms.MessageBox]::Show("$message`n`nUpdate files: $stage", 'PecoFence update', 'OK', 'Error')
      }
    } catch { }
    [Console]::Error.WriteLine($message)
    exit 1
  }
}
