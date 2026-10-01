# Loaded only by the embedded worker's Cleanup action, under update.lock.
# Never infer ownership from a directory name alone. Unknown files or links keep
# an attempt intact; deletion never recurses through an arbitrary directory tree.

function Assert-CleanupDirectory([string]$Path, [string]$Updates) {
  $full = Assert-PlainPath $Path
  $parent = Assert-PlainPath $Updates
  if ([IO.Path]::GetDirectoryName($full) -ine $parent -or [IO.Path]::GetFileName($full) -cnotmatch '\A(?:\.cleanup-)?[0-9a-f]{32}\z') { throw 'Cleanup destination is outside the updates directory.' }
  return $full
}

function Assert-CleanupContents([string]$Stage, [string]$Asset = '') {
  $null = Assert-PlainPath $Stage
  $files = @('plan.json', 'plan.json.tmp', 'release.json', 'checksum.txt',
    'verified.json', 'verified.json.tmp', 'progress.json', 'progress.json.tmp',
    'handoff.json', 'handoff.json.tmp', 'result.json', 'result.json.tmp',
    'transaction.json', 'transaction.json.tmp', 'installer.json', 'installer.json.tmp',
    'healthy.json', 'healthy.json.tmp', 'cleanup.json', 'cleanup.json.tmp',
    'update-worker.ps1', 'update-worker.json.tmp', 'update-cleanup.ps1', 'update-cleanup.json.tmp',
    'Check.log', 'Download.log', 'Apply.log', 'Recover.log', 'Cleanup.log', 'setup.log')
  foreach ($entry in [IO.Directory]::EnumerateFileSystemEntries($Stage)) {
    $null = Assert-PlainPath $entry
    $name = [IO.Path]::GetFileName($entry)
    if ([IO.Directory]::Exists($entry)) {
      if ($name -cnotin @('payload', 'apply-payload', 'backup')) { throw 'Unknown update directory; retained.' }
      foreach ($child in [IO.Directory]::EnumerateFileSystemEntries($entry)) {
        $null = Assert-PlainPath $child
        if ([IO.Directory]::Exists($child) -or [IO.Path]::GetFileName($child) -cnotin $script:ManagedFiles) { throw 'Unknown file in update payload; retained.' }
      }
    } elseif ($name -cnotin $files -and (-not $Asset -or $name -cne $Asset)) { throw 'Unknown update file; retained.' }
  }
}

function Remove-CleanupFile([string]$Path) {
  $null = Assert-PlainPath $Path
  [IO.File]::Delete($Path)
}

function Remove-CleanupPayload([string]$Stage, [string]$Name) {
  if ($Name -cnotin @('payload', 'apply-payload', 'backup')) { throw 'Invalid payload directory.' }
  $path = Join-Path $Stage $Name
  $null = Assert-PlainPath $path
  if (-not [IO.Directory]::Exists($path)) { return }
  foreach ($file in [IO.Directory]::EnumerateFileSystemEntries($path)) {
    $null = Assert-PlainPath $file
    if ([IO.Directory]::Exists($file) -or [IO.Path]::GetFileName($file) -cnotin $script:ManagedFiles) { throw 'Unknown payload content; retained.' }
    Remove-CleanupFile $file
  }
  [IO.Directory]::Delete($path, $false)
}

function Test-CleanupVersion([string]$Running, [string]$Expected) {
  if ($Running -cnotmatch '\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)([-+][A-Za-z0-9.+-]+)?\z' -or $Expected -cnotmatch '\A(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)([-+][A-Za-z0-9.+-]+)?\z') { return $false }
  if ($Running -ceq $Expected) { return $true }
  $runningNumber = [version]($Running -split '[-+]', 2)[0]
  $expectedNumber = [version]($Expected -split '[-+]', 2)[0]
  return $runningNumber -gt $expectedNumber -or ($runningNumber -eq $expectedNumber -and -not $Running.Contains('-'))
}

function Get-CleanupCandidate([string]$Stage, $Current, [datetime]$Now) {
  $data = Read-SmallJson (Join-Path $Stage 'plan.json')
  if ($data.schema -ne 1 -or $data.root -ine $Current.root -or $data.mode -cne $Current.mode -or $data.repository -cne $Current.repository) { throw 'Attempt belongs to a different copy or source; retained.' }
  $asset = ''
  if ($null -ne $data.release) { Assert-ReleasePlan $data; $asset = $data.release.asset }
  Assert-CleanupContents $Stage $asset
  $completed = $false
  $backup = $false
  $journalPath = $null
  $expectedVersion = $null
  $transaction = Join-Path $Stage 'transaction.json'
  $installer = Join-Path $Stage 'installer.json'
  if ([IO.File]::Exists($transaction) -and [IO.File]::Exists($installer)) { throw 'Conflicting journals; retained.' }
  if ([IO.File]::Exists($transaction)) {
    $journal = Read-SmallJson $transaction
    if ($data.mode -cne 'portable' -or $journal.root -ine $data.root -or $journal.state -cnotin @('complete', 'rolledBack')) { return $null }
    $journalPath = $transaction
    $expectedVersion = if ($journal.state -ceq 'complete') { $data.release.version } else { $data.currentVersion }
    $backup = [IO.Directory]::Exists((Join-Path $Stage 'backup'))
  } elseif ([IO.File]::Exists($installer)) {
    $journal = Read-SmallJson $installer
    if ($data.mode -cne 'installed' -or $journal.root -ine $data.root -or $journal.state -cne 'complete') { return $null }
    $journalPath = $installer
    $expectedVersion = $data.release.version
  }
  $time = [IO.File]::GetLastWriteTimeUtc((Join-Path $Stage 'plan.json'))
  $healthyAt = $Now
  if ($journalPath) {
    # A completed file transaction alone does not authorize deleting recovery data.
    # The caller is a matching app that has reached its delayed message-loop timer.
    if (-not (Test-CleanupVersion $Current.currentVersion $expectedVersion)) { return $null }
    $time = [IO.File]::GetLastWriteTimeUtc($journalPath)
    $healthPath = Join-Path $Stage 'healthy.json'
    if (-not [IO.File]::Exists($healthPath)) {
      Write-UpdateJson $healthPath @{schema=1;root=$data.root;repository=$data.repository;version=$expectedVersion;observedVersion=$Current.currentVersion;observedUtc=$Now.ToString('o')}
    }
    $health = Read-SmallJson $healthPath
    if ($health.schema -ne 1 -or $health.root -ine $data.root -or $health.repository -cne $data.repository -or $health.version -cne $expectedVersion -or -not (Test-CleanupVersion $health.observedVersion $expectedVersion)) { throw 'Invalid startup receipt; retained.' }
    $healthyAt = [DateTimeOffset]::Parse($health.observedUtc).UtcDateTime
    if ($healthyAt -gt $Now) { throw 'Startup receipt is in the future; retained.' }
    $completed = $true
    if ($backup) {
      $discardingBackup = $false
      $cleanupPath = Join-Path $Stage 'cleanup.json'
      if ([IO.File]::Exists($cleanupPath)) {
        $cleanup = Read-SmallJson $cleanupPath
        $discardingBackup = $cleanup.schema -eq 1 -and $cleanup.state -ceq 'compact' -and $cleanup.root -ieq $Current.root -and $cleanup.mode -ceq $Current.mode -and $cleanup.repository -ceq $Current.repository -and $cleanup.attemptId -ceq [IO.Path]::GetFileName($Stage) -and $cleanup.removeBackup -eq $true
      }
      if ($discardingBackup) { $backup = $false }
      else {
        $seen = New-Object 'Collections.Generic.HashSet[string]' ([StringComparer]::OrdinalIgnoreCase)
        if (@($journal.files).Count -ne $script:ManagedFiles.Count) { throw 'Invalid backup inventory; retained.' }
        foreach ($item in $journal.files) {
          if ($item.name -cnotin $script:ManagedFiles -or -not $seen.Add($item.name) -or $item.sha256 -cnotmatch '\A[0-9a-f]{64}\z' -or (Get-FileDigest (Join-Path $Stage ('backup\' + $item.name))) -cne $item.sha256) { throw 'Damaged backup; retained for inspection.' }
        }
        Assert-PackageIdentity (Join-Path $Stage 'backup') $data $data.currentVersion
      }
    }
  }
  $ready = $false
  if (-not $journalPath -and $asset -and [IO.File]::Exists((Join-Path $Stage 'verified.json')) -and -not [IO.File]::Exists((Join-Path $Stage 'handoff.json')) -and $data.currentVersion -ceq $Current.currentVersion) {
    try { Assert-VerifiedDownload $Stage $data; $ready = $true }
    catch { $ready = $false } # A corrupt cache is not a reusable download.
  }
  return [pscustomobject]@{stage=$Stage;id=[IO.Path]::GetFileName($Stage);asset=$asset;time=$time;healthyAt=$healthyAt;completed=$completed;backup=$backup;ready=$ready}
}

function Remove-CleanupTombstone([string]$Stage, [string]$Updates, $Current) {
  $full = Assert-CleanupDirectory $Stage $Updates
  $id = [IO.Path]::GetFileName($full)
  if ($id -cnotmatch '\A\.cleanup-([0-9a-f]{32})\z') { throw 'Invalid cleanup tombstone.' }
  $attemptId = $Matches[1]
  if (@([IO.Directory]::EnumerateFileSystemEntries($full)).Count -eq 0) { [IO.Directory]::Delete($full, $false); return }
  $record = Read-SmallJson (Join-Path $full 'cleanup.json')
  if ($record.schema -ne 1 -or $record.state -cne 'discard' -or $record.attemptId -cne $attemptId -or $record.root -ine $Current.root -or $record.mode -cne $Current.mode -or $record.repository -cne $Current.repository) { throw 'Invalid cleanup ownership record; retained.' }
  # The asset filename was already validated before the atomic rename. Recheck its
  # syntax here, including after plan.json was removed during an interrupted sweep.
  if ($record.asset -and $record.asset -cnotmatch '\Apecofence-v[0-9]+\.[0-9]+\.[0-9]+-x64-(portable\.zip|setup\.exe)\z') { throw 'Invalid cleanup asset.' }
  Assert-CleanupContents $full $record.asset
  foreach ($name in @('transaction.json', 'installer.json')) {
    $path = Join-Path $full $name
    if ([IO.File]::Exists($path)) {
      $journal = Read-SmallJson $path
      if ($journal.root -ine $Current.root -or $journal.state -cnotin @('complete', 'rolledBack')) { throw 'Unresolved journal in cleanup directory; retained.' }
    }
  }
  foreach ($name in @('payload', 'apply-payload', 'backup')) { Remove-CleanupPayload $full $name }
  foreach ($file in [IO.Directory]::EnumerateFiles($full)) {
    if ([IO.Path]::GetFileName($file) -cnotin @('plan.json', 'cleanup.json')) { Remove-CleanupFile $file }
  }
  # Keep ownership until all payload and diagnostic files have been removed.
  Remove-CleanupFile (Join-Path $full 'plan.json')
  Remove-CleanupFile (Join-Path $full 'cleanup.json')
  [IO.Directory]::Delete($full, $false)
}

function Discard-CleanupAttempt($Candidate, [string]$Updates, $Current) {
  $from = Assert-CleanupDirectory $Candidate.stage $Updates
  $to = Assert-CleanupDirectory (Join-Path $Updates ('.cleanup-' + $Candidate.id)) $Updates
  Assert-CleanupContents $from $Candidate.asset
  Write-UpdateJson (Join-Path $from 'cleanup.json') @{schema=1;state='discard';attemptId=$Candidate.id;root=$Current.root;mode=$Current.mode;repository=$Current.repository;asset=$Candidate.asset}
  # Both resolved paths are direct children of this copy's update directory.
  # Once renamed, startup cannot mistake a partially deleted cache for an update.
  [IO.Directory]::Move($from, $to)
  Remove-CleanupTombstone $to $Updates $Current
}

function Invoke-UpdateCleanup([string]$Stage, $Current, [datetime]$Now = [DateTime]::UtcNow) {
  $updates = Assert-PlainPath ([IO.Path]::GetDirectoryName($Stage))
  $null = Assert-CleanupDirectory $Stage $updates
  $protectedId = ''
  if ($Current.PSObject.Properties['protectedAttempt']) { $protectedId = [string]$Current.protectedAttempt }
  if ($protectedId -and $protectedId -cnotmatch '\A[0-9a-f]{32}\z') { throw 'Invalid protected attempt.' }
  $candidates = New-Object 'Collections.Generic.List[object]'
  $report = @{removed=0;compacted=0;retained=0;errors=0;notes=(New-Object 'Collections.Generic.List[string]')}
  foreach ($directory in [IO.Directory]::EnumerateDirectories($updates)) {
    $name = [IO.Path]::GetFileName($directory)
    if ($directory -ieq $Stage -or $name -ceq $protectedId) { continue }
    try {
      if ($name -cmatch '\A\.cleanup-[0-9a-f]{32}\z') {
        Remove-CleanupTombstone $directory $updates $Current
        $report.removed++
      } elseif ($name -cmatch '\A[0-9a-f]{32}\z') {
        $null = Assert-CleanupDirectory $directory $updates
        $candidate = Get-CleanupCandidate $directory $Current $Now
        if ($null -ne $candidate) { $candidates.Add($candidate) } else { $report.retained++ }
      }
    } catch {
      $report.errors++
      $message = $_.Exception.Message
      if ($report.notes.Count -lt 10) { $report.notes.Add($message.Substring(0, [Math]::Min(256, $message.Length))) }
    }
  }
  $ordered = @($candidates | Sort-Object -Property @{Expression='time';Descending=$true}, @{Expression='id';Descending=$true})
  # The app's selected download is always protected. Otherwise keep just the most
  # recent verified download so a restart can resume it without another transfer.
  $ready = @($ordered | Where-Object { $_.ready -and -not $protectedId } | Select-Object -First 1)
  $backup = @($ordered | Where-Object { $_.backup -and ($Now - $_.healthyAt).TotalDays -lt 30 } | Select-Object -First 1)
  $logs = @($ordered | Where-Object { -not $_.ready -and ($Now - $_.time).TotalDays -lt 7 } | Select-Object -First 10)
  foreach ($candidate in $ordered) {
    try {
      if ($ready.Count -and $candidate.id -ceq $ready[0].id) { $report.retained++; continue }
      $keepBackup = $backup.Count -and $candidate.id -ceq $backup[0].id
      # Only attempts whose live replacement is finished and boot-confirmed, or
      # which never wrote a live program file, reach this loop.
      foreach ($name in @('payload', 'apply-payload')) { Remove-CleanupPayload $candidate.stage $name }
      if ($candidate.asset) { Remove-CleanupFile (Join-Path $candidate.stage $candidate.asset) }
      if (-not $keepBackup) {
        if ([IO.Directory]::Exists((Join-Path $candidate.stage 'backup'))) {
          Write-UpdateJson (Join-Path $candidate.stage 'cleanup.json') @{schema=1;state='compact';attemptId=$candidate.id;root=$Current.root;mode=$Current.mode;repository=$Current.repository;removeBackup=$true}
        }
        Remove-CleanupPayload $candidate.stage 'backup'
      }
      if ($candidate.completed) {
        foreach ($name in $script:ManagedFiles) { Remove-CleanupFile (Join-Path $Current.root ($name + '.update-' + $candidate.id)) }
      }
      $report.compacted++
      if (-not $keepBackup -and $candidate.id -cnotin @($logs | ForEach-Object { $_.id })) {
        Discard-CleanupAttempt $candidate $updates $Current
        $report.removed++
      }
    } catch {
      $report.errors++
      $message = $_.Exception.Message
      if ($report.notes.Count -lt 10) { $report.notes.Add($message.Substring(0, [Math]::Min(256, $message.Length))) }
    }
  }
  return $report
}
