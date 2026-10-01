# Sourced by test-updates.ps1: reuse its isolated fixtures and test runner.
. (Join-Path $PSScriptRoot 'runtime/update-cleanup.ps1')

function New-CleanupContext {
  $fixture = New-Fixture
  $stage = Join-Path ([IO.Path]::GetDirectoryName($fixture.stage)) ([guid]::NewGuid().ToString('N'))
  $null = [IO.Directory]::CreateDirectory($stage)
  $data = [pscustomobject]@{schema=1;root=$fixture.root;mode='portable';repository=$testRepository;currentVersion=$testVersion;release=$null;protectedAttempt=$null}
  Write-UpdateJson (Join-Path $stage 'plan.json') $data
  Write-FixtureIdentity $fixture.root 'portable' $testVersion
  [IO.File]::SetLastWriteTimeUtc((Join-Path $fixture.stage 'plan.json'), [DateTime]::UtcNow.AddDays(-90))
  return [pscustomobject]@{stage=$stage;data=$data;fixture=$fixture;updates=[IO.Path]::GetDirectoryName($stage)}
}

function New-CleanupAttempt($Context, [string]$Kind = 'check', [double]$AgeDays = 0) {
  $stage = Join-Path $Context.updates ([guid]::NewGuid().ToString('N'))
  $null = [IO.Directory]::CreateDirectory($stage)
  $data = $Context.fixture.data | ConvertTo-Json -Depth 10 | ConvertFrom-Json
  if ($Kind -eq 'check') { $data.release = $null }
  if ($Kind -eq 'ready') {
    $data.currentVersion = $testVersion
    $data.release.version = '999.0.0'
    $data.release.asset = 'pecofence-v999.0.0-x64-portable.zip'
    $base = "https://github.com/$testRepository/releases"
    $data.release.url = "$base/download/v999.0.0/$($data.release.asset)"
    $data.release.checksumUrl = "$($data.release.url).sha256"
    $data.release.page = "$base/tag/v999.0.0"
  }
  Write-UpdateJson (Join-Path $stage 'plan.json') $data
  [IO.File]::SetLastWriteTimeUtc((Join-Path $stage 'plan.json'), [DateTime]::UtcNow.AddDays(-$AgeDays))
  if ($Kind -ne 'check') {
    [IO.File]::WriteAllText((Join-Path $stage $data.release.asset), 'x')
    $payload = Join-Path $stage 'payload'
    $null = [IO.Directory]::CreateDirectory($payload)
    [IO.File]::WriteAllText((Join-Path $payload 'README.md'), 'extracted')
  }
  if ($Kind -eq 'ready') { Write-UpdateJson (Join-Path $stage 'verified.json') @{asset=$data.release.asset;sha256=(Get-FileDigest (Join-Path $stage $data.release.asset))} }
  if ($Kind -in @('complete', 'rolledBack', 'applying', 'recoveryRequired')) {
    $backup = Join-Path $stage 'backup'
    $null = [IO.Directory]::CreateDirectory($backup)
    foreach ($name in $script:ManagedFiles) { [IO.File]::WriteAllText((Join-Path $backup $name), "old $name") }
    Write-FixtureIdentity $backup 'portable' $priorVersion
    $files = @($script:ManagedFiles | ForEach-Object { @{name=$_;sha256=(Get-FileDigest (Join-Path $backup $_))} })
    Write-UpdateJson (Join-Path $stage 'transaction.json') @{state=$Kind;root=$data.root;files=$files}
    [IO.File]::SetLastWriteTimeUtc((Join-Path $stage 'transaction.json'), [DateTime]::UtcNow.AddDays(-$AgeDays))
  }
  return [pscustomobject]@{stage=$stage;data=$data;id=[IO.Path]::GetFileName($stage)}
}

Run-Test 'Cleanup confirms the running version, keeps one backup and bounds check history' {
  $c = New-CleanupContext
  $old = New-CleanupAttempt $c 'complete' 2
  $recent = New-CleanupAttempt $c 'complete' 1
  for ($i = 0; $i -lt 15; $i++) { $null = New-CleanupAttempt $c 'check' (3 + $i / 20) }
  $expired = New-CleanupAttempt $c 'check' 8
  $report = Invoke-UpdateCleanup $c.stage $c.data
  Assert-Test ($report.errors -eq 0) 'Cleanup unexpectedly failed'
  Assert-Test ([IO.File]::Exists((Join-Path $recent.stage 'healthy.json'))) 'Missing startup receipt'
  Assert-Test ([IO.Directory]::Exists((Join-Path $recent.stage 'backup'))) 'Latest backup was removed'
  Assert-Test (-not [IO.Directory]::Exists((Join-Path $old.stage 'backup'))) 'Old backup was not trimmed'
  Assert-Test (-not [IO.File]::Exists((Join-Path $recent.stage $recent.data.release.asset))) 'Completed download not removed'
  Assert-Test (-not [IO.Directory]::Exists((Join-Path $recent.stage 'payload'))) 'Extracted payload not removed'
  Assert-Test (-not [IO.Directory]::Exists($expired.stage)) 'Expired check not removed'
  $remaining = @([IO.Directory]::EnumerateDirectories($c.updates) | Where-Object { $_ -ine $c.stage })
  Assert-Test ($remaining.Count -eq 10) 'History exceeded 10 attempts'
  Assert-UserData $c.fixture
}

Run-Test 'Backup retention expires 30 days after startup acknowledgement' {
  $c = New-CleanupContext
  $a = New-CleanupAttempt $c 'rolledBack' 10
  $now = [DateTime]::UtcNow
  $null = Invoke-UpdateCleanup $c.stage $c.data $now
  Assert-Test ([IO.Directory]::Exists((Join-Path $a.stage 'backup'))) 'Backup discarded before retention period'
  $null = Invoke-UpdateCleanup $c.stage $c.data ($now.AddDays(31))
  Assert-Test (-not [IO.Directory]::Exists($a.stage)) 'Expired backup remained'
  Assert-UserData $c.fixture
}

Run-Test 'Unconfirmed new version and unresolved transactions retain all recovery files' {
  $c = New-CleanupContext
  $complete = New-CleanupAttempt $c 'complete' 90
  $pending = New-CleanupAttempt $c 'applying' 90
  $failed = New-CleanupAttempt $c 'recoveryRequired' 90
  $c.data.currentVersion = $priorVersion
  $report = Invoke-UpdateCleanup $c.stage $c.data
  foreach ($a in @($complete, $pending, $failed)) {
    Assert-Test ([IO.File]::Exists((Join-Path $a.stage $a.data.release.asset))) 'Protected download removed'
    Assert-Test ([IO.File]::Exists((Join-Path $a.stage 'backup/README.md'))) 'Protected backup removed'
    Assert-Test (-not [IO.File]::Exists((Join-Path $a.stage 'healthy.json'))) 'Old running version acknowledged newer update'
  }
  Assert-UserData $c.fixture
}

Run-Test 'Only the selected verified download survives; abandoned partial transfers are discarded' {
  $c = New-CleanupContext
  $selected = New-CleanupAttempt $c 'ready' 3
  $extra = New-CleanupAttempt $c 'ready' 2
  $partial = New-CleanupAttempt $c 'partial' 9
  $c.data.protectedAttempt = $selected.id
  $null = Invoke-UpdateCleanup $c.stage $c.data
  Assert-Test ([IO.File]::Exists((Join-Path $selected.stage $selected.data.release.asset))) 'Selected download removed'
  Assert-Test (-not [IO.Directory]::Exists($extra.stage)) 'Duplicate verified download retained'
  Assert-Test (-not [IO.Directory]::Exists($partial.stage)) 'Abandoned partial transfer retained'
  Assert-UserData $c.fixture
}

Run-Test 'Unknown files, different sources, malformed journals and junctions are retained' {
  $c = New-CleanupContext
  $unknown = New-CleanupAttempt $c 'check' 90
  [IO.File]::WriteAllText((Join-Path $unknown.stage 'user-note.txt'), 'keep')
  $foreign = New-CleanupAttempt $c 'check' 90
  $foreign.data.repository = 'Other/PecoFence'
  Write-UpdateJson (Join-Path $foreign.stage 'plan.json') $foreign.data
  $broken = New-CleanupAttempt $c 'complete' 90
  [IO.File]::WriteAllText((Join-Path $broken.stage 'transaction.json'), '{broken')
  $linked = New-CleanupAttempt $c 'check' 90
  $outside = Join-Path $testRoot ('unrelated-' + [guid]::NewGuid().ToString('N'))
  $null = [IO.Directory]::CreateDirectory($outside)
  [IO.File]::WriteAllText((Join-Path $outside 'README.md'), 'keep')
  $null = New-Item -ItemType Junction -Path (Join-Path $linked.stage 'payload') -Target $outside
  $null = Invoke-UpdateCleanup $c.stage $c.data
  foreach ($a in @($unknown, $foreign, $broken, $linked)) { Assert-Test ([IO.Directory]::Exists($a.stage)) 'Unrecognized data was removed' }
  Assert-Test ([IO.File]::ReadAllText((Join-Path $outside 'README.md')) -eq 'keep') 'Cleanup followed a junction'
  Assert-UserData $c.fixture
}

Run-Test 'Cleanup keeps the newest reusable download and discards corrupt cache bytes' {
  $c = New-CleanupContext
  $older = New-CleanupAttempt $c 'ready' 3
  $latest = New-CleanupAttempt $c 'ready' 2
  $corrupt = New-CleanupAttempt $c 'ready' 1
  [IO.File]::WriteAllText((Join-Path $corrupt.stage $corrupt.data.release.asset), 'z')
  $null = Invoke-UpdateCleanup $c.stage $c.data
  Assert-Test ([IO.File]::Exists((Join-Path $latest.stage $latest.data.release.asset))) 'Newest valid download discarded'
  Assert-Test (-not [IO.File]::Exists((Join-Path $older.stage $older.data.release.asset))) 'Duplicate download retained'
  Assert-Test (-not [IO.File]::Exists((Join-Path $corrupt.stage $corrupt.data.release.asset))) 'Corrupt download retained'
}

Run-Test 'A corrupt recent backup cannot evict the last verified backup' {
  $c = New-CleanupContext
  $valid = New-CleanupAttempt $c 'complete' 2
  $corrupt = New-CleanupAttempt $c 'complete' 1
  [IO.File]::AppendAllText((Join-Path $corrupt.stage 'backup/README.md'), 'corrupt')
  $null = Invoke-UpdateCleanup $c.stage $c.data
  Assert-Test ([IO.File]::Exists((Join-Path $valid.stage 'backup/README.md'))) 'Valid backup evicted by corrupt one'
  Assert-Test ([IO.File]::Exists((Join-Path $corrupt.stage $corrupt.data.release.asset))) 'Damaged attempt changed instead of being retained'
}

Run-Test 'Installed cleanup isolates installation roots and preserves interrupted setup' {
  $savedLocalAppData = $env:LOCALAPPDATA
  try {
    $env:LOCALAPPDATA = Join-Path $testRoot ('cleanup-profile-' + [guid]::NewGuid().ToString('N'))
    $f = New-Fixture 'installed'
    $other = New-Fixture 'installed'
    $stage = Join-Path ([IO.Path]::GetDirectoryName($f.stage)) ([guid]::NewGuid().ToString('N'))
    $null = [IO.Directory]::CreateDirectory($stage)
    $current = [pscustomobject]@{schema=1;root=$f.root;mode='installed';repository=$testRepository;currentVersion=$testVersion;release=$null;protectedAttempt=$null}
    foreach ($a in @($f, $other)) {
      [IO.File]::WriteAllText((Join-Path $a.stage $a.data.release.asset), 'setup')
      Write-UpdateJson (Join-Path $a.stage 'installer.json') @{state='interrupted';root=$a.root}
    }
    $null = Invoke-UpdateCleanup $stage $current
    Assert-Test ([IO.File]::Exists((Join-Path $f.stage $f.data.release.asset))) 'Pending setup deleted'
    Write-UpdateJson (Join-Path $f.stage 'installer.json') @{state='complete';root=$f.root}
    $null = Invoke-UpdateCleanup $stage $current
    Assert-Test (-not [IO.File]::Exists((Join-Path $f.stage $f.data.release.asset))) 'Completed setup retained after startup'
    Assert-Test ([IO.File]::Exists((Join-Path $other.stage $other.data.release.asset))) 'Another installation was modified'
    Assert-UserData $f
    Assert-UserData $other
  } finally { $env:LOCALAPPDATA = $savedLocalAppData }
}

Run-Test 'Cleanup cannot run alongside a locked update or without a matching running app' {
  $c = New-CleanupContext
  $a = New-CleanupAttempt $c 'complete' 90
  $lock = [IO.File]::Open((Join-Path $c.updates 'update.lock'), [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
  try { Expect-Failure { Invoke-UpdateWorker 'Cleanup' (Join-Path $c.stage 'plan.json') } }
  finally { $lock.Dispose() }
  Expect-Failure { Invoke-UpdateWorker 'Cleanup' (Join-Path $c.stage 'plan.json') $PID }
  Assert-Test (-not [IO.File]::Exists((Join-Path $a.stage 'healthy.json'))) 'Unauthorized process confirmed startup'
  Assert-Test ([IO.Directory]::Exists((Join-Path $a.stage 'backup'))) 'Locked backup deleted'
}

Run-Test 'The staged worker loads cleanup functions and checks a running versioned host' {
  $c = New-CleanupContext
  $a = New-CleanupAttempt $c 'complete' 90
  # A tiny versioned fixture process exercises the real worker entry point without
  # opening PecoFence windows or touching the user's installed/default instance.
  $hostExe = Join-Path $testRoot 'cleanup-host.exe'
  $numericVersion = ($testVersion -split '[-+]', 2)[0]
  $source = @"
using System.Reflection;
[assembly: AssemblyFileVersion("$numericVersion")]
[assembly: AssemblyInformationalVersion("$testVersion")]
public static class CleanupFixtureHost {
  public static void Main() { System.Threading.Thread.Sleep(120000); }
}
"@
  Add-Type -TypeDefinition $source -OutputAssembly $hostExe -OutputType ConsoleApplication
  [IO.File]::Copy($hostExe, (Join-Path $c.data.root 'pecofence.exe'), $true)
  foreach ($name in @('update-worker.ps1', 'update-cleanup.ps1')) {
    Copy-Durable (Join-Path $PSScriptRoot ('runtime/' + $name)) (Join-Path $c.stage $name)
  }
  $hostStart = New-Object Diagnostics.ProcessStartInfo
  $hostStart.FileName = Join-Path $c.data.root 'pecofence.exe'
  $hostStart.UseShellExecute = $false; $hostStart.CreateNoWindow = $true
  $hostProcess = [Diagnostics.Process]::Start($hostStart)
  $worker = $null
  try {
    # Early startup must fail without acknowledging or deleting an update.
    Expect-Failure { Invoke-UpdateWorker 'Cleanup' (Join-Path $c.stage 'plan.json') $hostProcess.Id }
    Assert-Test (-not [IO.File]::Exists((Join-Path $a.stage 'healthy.json'))) 'Early startup acknowledged an update'
    $deadline = [DateTime]::UtcNow.AddSeconds(31)
    while ([DateTime]::UtcNow -lt $deadline) {
      Assert-Test (-not $hostProcess.HasExited) 'Fixture host exited early'
      Start-Sleep -Milliseconds 100
    }
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $start.Arguments = '-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "' + (Join-Path $c.stage 'update-worker.ps1') + '" -Action Cleanup -Plan "' + (Join-Path $c.stage 'plan.json') + '" -ParentPid ' + $hostProcess.Id
    $start.UseShellExecute = $false; $start.CreateNoWindow = $true
    $start.RedirectStandardError = $true; $start.RedirectStandardOutput = $true
    $worker = [Diagnostics.Process]::Start($start)
    Assert-Test ($worker.WaitForExit(30000)) 'Staged cleanup worker timed out'
    Assert-Test ($worker.ExitCode -eq 0) ('Staged worker failed: ' + $worker.StandardError.ReadToEnd())
    Assert-Test ((Read-SmallJson (Join-Path $c.stage 'result.json')).status -eq 'ok') 'Worker did not report completion'
    Assert-Test ([IO.File]::Exists((Join-Path $a.stage 'healthy.json'))) 'Running host was not acknowledged'
    Assert-Test (-not [IO.File]::Exists((Join-Path $a.stage $a.data.release.asset))) 'Verified completed download not cleaned'
    Assert-UserData $c.fixture
  } finally {
    if ($null -ne $worker) { if (-not $worker.HasExited) { $worker.Kill(); $worker.WaitForExit() }; $worker.Dispose() }
    if (-not $hostProcess.HasExited) { $hostProcess.Kill(); $hostProcess.WaitForExit() }; $hostProcess.Dispose()
  }
}

# Real process termination during deletion. Signals are outside managed attempts
# so the production ownership scan sees exactly what a shipped app would write.
$cleanupHarness = Join-Path $testRoot 'cleanup-crash.ps1'
[IO.File]::WriteAllText($cleanupHarness, @'
param([string]$Worker, [string]$PlanFile, [string]$Signal, [string]$Phase)
$ErrorActionPreference = 'Stop'
. $Worker
. (Join-Path ([IO.Path]::GetDirectoryName($Worker)) 'update-cleanup.ps1')
$current = Read-SmallJson $PlanFile
$stage = [IO.Path]::GetDirectoryName($PlanFile)
$script:DeleteFile = ${function:Remove-CleanupFile}
function Remove-CleanupFile([string]$Path) {
  & $script:DeleteFile $Path
  if (($Phase -eq 'backup' -and $Path.Contains('\backup\')) -or
      ($Phase -eq 'tombstone' -and $Path.Contains('\.cleanup-')) -or
      ($Phase -eq 'owner' -and $Path.Contains('\.cleanup-') -and $Path.EndsWith('\plan.json'))) {
    [IO.File]::WriteAllText($Signal, 'checkpoint')
    while ($true) { Start-Sleep -Milliseconds 100 }
  }
}
$null = Invoke-UpdateCleanup $stage $current
'@)

foreach ($phase in @('backup', 'tombstone', 'owner')) {
  Run-Test "Forced cleanup termination resumes in a fresh process: $phase" {
    $c = New-CleanupContext
    $old = New-CleanupAttempt $c 'complete' 90
    $keep = New-CleanupAttempt $c 'complete' 1
    $signal = Join-Path $testRoot ('cleanup-signal-' + [guid]::NewGuid().ToString('N'))
    $start = New-Object Diagnostics.ProcessStartInfo
    $start.FileName = Join-Path $env:SystemRoot 'System32/WindowsPowerShell/v1.0/powershell.exe'
    $argsBase = '-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "' + $cleanupHarness + '" -Worker "' + (Join-Path $PSScriptRoot 'runtime/update-worker.ps1') + '" -PlanFile "' + (Join-Path $c.stage 'plan.json') + '" -Signal "' + $signal + '" -Phase '
    $start.Arguments = $argsBase + $phase
    $start.UseShellExecute = $false; $start.CreateNoWindow = $true
    $start.RedirectStandardError = $true; $start.RedirectStandardOutput = $true
    $child = [Diagnostics.Process]::Start($start)
    try {
      $deadline = [DateTime]::UtcNow.AddSeconds(30)
      while (-not [IO.File]::Exists($signal)) {
        if ($child.HasExited) { throw ('Cleanup exited before checkpoint: ' + $child.StandardError.ReadToEnd() + $child.StandardOutput.ReadToEnd()) }
        if ([DateTime]::UtcNow -gt $deadline) { throw 'Cleanup checkpoint timed out' }
        Start-Sleep -Milliseconds 50
      }
      $child.Kill(); $child.WaitForExit()
    } finally { if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit() }; $child.Dispose() }
    $start.Arguments = $argsBase + 'resume'
    $child = [Diagnostics.Process]::Start($start)
    try {
      Assert-Test ($child.WaitForExit(30000)) 'Fresh cleanup timed out'
      Assert-Test ($child.ExitCode -eq 0) ('Fresh cleanup failed: ' + $child.StandardError.ReadToEnd())
    } finally { if (-not $child.HasExited) { $child.Kill(); $child.WaitForExit() }; $child.Dispose() }
    Assert-Test (-not [IO.Directory]::Exists($old.stage)) 'Old cleanup did not finish'
    Assert-Test (@([IO.Directory]::EnumerateDirectories($c.updates, '.cleanup-*')).Count -eq 0) 'Deletion tombstone remained'
    Assert-Test ([IO.File]::Exists((Join-Path $keep.stage 'backup/README.md'))) 'Last backup was damaged'
    Assert-UserData $c.fixture
  }
}
