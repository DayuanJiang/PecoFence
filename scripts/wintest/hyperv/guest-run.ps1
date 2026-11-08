# scripts/wintest/hyperv/guest-run.ps1 -VM <name> -Steps a+b+c -Base <folder>: runs the kit's steps
# files in a Hyper-V guest of the Windows PC, the way run.sh does on an EC2 box. Host side (an
# administrator; run-hyperv.sh starts it). Files come from <Base>\<Stage> (kit, bin, extra).
# Takes a checkpoint first and restores it at the end (with the video settings), so the guest is
# left as found. Guest account pf / pecofence with auto-logon and the PFJob task (see README).
# Results: <Base>\res-<VM><Tag>\<step>.tar (C:\pf\run of each job) and jobs.txt; progress in
# run-<VM><Tag>.log; done-<VM><Tag>.txt when it ends.
param([string]$VM, [string]$Steps, [int]$Timeout = 1500, [int]$Width = 1600, [int]$Height = 900, [string]$Base = (Join-Path $env:TEMP 'pfwintest'), [string]$Stage = 'stage', [string]$Tag = '', [switch]$ClassicNotepad)
$ErrorActionPreference = 'Continue'
$base = $Base
$res = Join-Path $base "res-$VM$Tag"
$log = Join-Path $base "run-$VM$Tag.log"
$doneFile = Join-Path $base "done-$VM$Tag.txt"
Remove-Item $doneFile -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force $res -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $res | Out-Null
function L($m) { "$(Get-Date -Format HH:mm:ss) $m" | Add-Content $log -Encoding UTF8 }
$cred = New-Object PSCredential('pf', (ConvertTo-SecureString 'pecofence' -AsPlainText -Force))
$cp = 'wintest-run'
$took = $false
$s = $null
$vid = $null

function Connect() {
  for ($i = 0; $i -lt 60; $i++) {
    try { return (New-PSSession -VMName $VM -Credential $cred -ErrorAction Stop) } catch { Start-Sleep -Seconds 4 }
  }
  return $null
}
# Runs a block in the guest; reconnects once when the session broke.
function G([scriptblock]$b, $a = @()) {
  try { return (Invoke-Command -Session $script:s -ScriptBlock $b -ArgumentList $a -ErrorAction Stop) }
  catch {
    L "guest call failed: $($_.Exception.Message); reconnecting"
    if ($script:s) { Remove-PSSession $script:s -ErrorAction SilentlyContinue }
    $script:s = Connect
    if (-not $script:s) { throw 'no PowerShell Direct session after reconnect' }
    return (Invoke-Command -Session $script:s -ScriptBlock $b -ArgumentList $a)
  }
}

try {
  $v = Get-VM -Name $VM -ErrorAction Stop
  $snaps = @(Get-VMSnapshot -VMName $VM -ErrorAction SilentlyContinue)
  if ($v.State -ne 'Off' -or $snaps.Count -gt 0) { throw "VM $VM is in use: state $($v.State), checkpoints $($snaps.Count)" }
  $vid = Get-VMVideo -VMName $VM
  L "video before: $($vid.ResolutionType) $($vid.HorizontalResolution)x$($vid.VerticalResolution)"
  Checkpoint-VM -Name $VM -SnapshotName $cp -ErrorAction Stop
  $took = $true
  L "checkpoint $cp taken"
  try { Set-VMVideo -VMName $VM -ResolutionType Single -HorizontalResolution $Width -VerticalResolution $Height -ErrorAction Stop; L "video set to ${Width}x${Height}" }
  catch { L "Set-VMVideo failed: $($_.Exception.Message)" }
  Start-VM -Name $VM -ErrorAction Stop
  $t0 = Get-Date
  $s = Connect
  if (-not $s) { throw 'no PowerShell Direct session' }
  for ($i = 0; $i -lt 60; $i++) {
    $n = G { @(Get-Process explorer -ErrorAction SilentlyContinue).Count }
    if ($n -gt 0) { break }
    Start-Sleep -Seconds 2
  }
  L "guest desktop up after $([int]((Get-Date) - $t0).TotalSeconds) s"
  Start-Sleep -Seconds 20

  $prep = G {
    New-Item -ItemType Directory -Force C:\pf | Out-Null
    Remove-Item -Recurse -Force C:\pf\kit, C:\pf\bin, C:\pf\extra, C:\pf\run, C:\pf\run.tar -ErrorAction SilentlyContinue
    # The job runs hidden (no console window on the desktop), at normal priority, as on the EC2 boxes.
    Set-Content -Path C:\pf\run-job.vbs -Encoding ASCII -Value 'CreateObject("WScript.Shell").Run "powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\pf\job.ps1", 0, True'
    $st = (Get-ScheduledTask PFJob).Settings
    $st.Priority = 4
    $st.ExecutionTimeLimit = 'PT3H'
    Set-ScheduledTask -TaskName PFJob -Action (New-ScheduledTaskAction -Execute 'wscript.exe' -Argument 'C:\pf\run-job.vbs') -Settings $st | Out-Null
    # Spotlight off, so the wallpaper stays the same.
    New-Item -Force 'HKCU:\Software\Microsoft\Windows\CurrentVersion\DesktopSpotlight\Settings' | Out-Null
    Set-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\DesktopSpotlight\Settings' EnabledState 0 -Type DWord
    New-Item -Force 'HKCU:\Software\Policies\Microsoft\Windows\CloudContent' | Out-Null
    Set-ItemProperty 'HKCU:\Software\Policies\Microsoft\Windows\CloudContent' DisableSpotlightCollectionOnDesktop 1 -Type DWord
    Set-ItemProperty 'HKCU:\Software\Policies\Microsoft\Windows\CloudContent' DisableWindowsSpotlightFeatures 1 -Type DWord
    $t = Get-ScheduledTask PFJob
    "action=$($t.Actions[0].Execute) $($t.Actions[0].Arguments) priority=$($t.Settings.Priority) user=$($t.Principal.UserId) $($t.Principal.LogonType) $($t.Principal.RunLevel)"
  }
  L "prep: $prep"
  if ($ClassicNotepad) {
    # The kit's steps expect the classic Notepad (as on Server 2025): the Store Notepad restores
    # the tabs of earlier jobs and shows "Cannot find ..." boxes over them.
    $np = G { Get-AppxPackage *WindowsNotepad* | Remove-AppxPackage -ErrorAction Continue 2>&1 | Out-String; 'left: ' + @(Get-AppxPackage *WindowsNotepad*).Count }
    L "store notepad removed: $np"
  }
  foreach ($d in 'kit', 'bin', 'extra') { Copy-Item -ToSession $s -Path (Join-Path $base "$Stage\$d") -Destination 'C:\pf\' -Recurse -Force }
  L ('copied: ' + (G { (Get-ChildItem -Recurse -File C:\pf\kit, C:\pf\bin, C:\pf\extra | Measure-Object).Count }) + ' files')

  foreach ($name in $Steps.Split('+')) {
    L "== $name"
    $st = G {
      param($name)
      # The kit's own reset when it has one (newer kits also end Edge and left setup programs).
      if (Test-Path C:\pf\kit\ctl.ps1) { & powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\pf\kit\ctl.ps1 reset | Out-Null }
      Get-Process pecofence, pecofence-watchdog, pecofence-cli, notepad, msedge -ErrorAction SilentlyContinue | Stop-Process -Force
      Start-Sleep -Milliseconds 500
      $lo = 'C:\Users\pf\AppData\Local'; $ro = 'C:\Users\pf\AppData\Roaming'
      Remove-Item -Recurse -Force C:\pf\run, C:\pf\run.tar, "$lo\PecoFence", "$ro\PecoFence", "$ro\OpenFence" -ErrorAction SilentlyContinue
      New-Item -ItemType Directory -Force C:\pf\run | Out-Null
      $j = "`$PfSteps = 'C:\pf\kit\steps\$name.ps1'`r`n. C:\pf\kit\job.ps1`r`n"
      [IO.File]::WriteAllText('C:\pf\job.ps1', $j, (New-Object Text.UTF8Encoding($true)))
      for ($i = 0; $i -lt 90 -and (Get-ScheduledTask PFJob).State -eq 'Running'; $i++) { Start-Sleep -Seconds 1 }
      if ((Get-ScheduledTask PFJob).State -eq 'Running') { return 'busy' }
      Start-ScheduledTask -TaskName PFJob
      for ($i = 0; $i -lt 40; $i++) {
        if ((Get-ScheduledTask PFJob).State -eq 'Running' -or (Test-Path C:\pf\run\done.txt)) { return 'started' }
        Start-Sleep -Milliseconds 250
      }
      return 'not started'
    } @($name)
    $t = Get-Date
    $status = $st
    if ($st -eq 'started') {
      while ($true) {
        Start-Sleep -Seconds 5
        $status = G { $state = (Get-ScheduledTask PFJob).State; if (Test-Path C:\pf\run\done.txt) { 'done' } elseif ($state -eq 'Running') { 'running' } else { 'ended' } }
        if ($status -ne 'running') { break }
        if (((Get-Date) - $t).TotalSeconds -gt $Timeout) { G { Stop-ScheduledTask -TaskName PFJob } | Out-Null; $status = "timeout after ${Timeout}s"; break }
      }
    }
    $secs = [int]((Get-Date) - $t).TotalSeconds
    "$name $secs $status" | Add-Content (Join-Path $res 'jobs.txt') -Encoding ASCII
    L "$name $status in $secs s"
    $col = G {
      Get-Process pecofence, pecofence-watchdog, pecofence-cli, notepad -ErrorAction SilentlyContinue | Stop-Process -Force
      Start-Sleep -Milliseconds 500
      New-Item -ItemType Directory -Force C:\pf\run\logs, C:\pf\run\shots | Out-Null
      Copy-Item 'C:\Users\pf\AppData\Local\PecoFence\*.log' C:\pf\run\logs -ErrorAction SilentlyContinue
      $items = @(Get-ChildItem C:\pf\run -File | ForEach-Object { "run\$($_.Name)" }) + @('run\logs', 'run\shots')
      $err = & tar.exe -cf C:\pf\run.tar -C C:\pf @items 2>&1 | Out-String
      'collected ' + (Get-Item C:\pf\run.tar).Length + ' ' + $err.Trim()
    }
    L $col
    Copy-Item -FromSession $s -Path C:\pf\run.tar -Destination (Join-Path $res "$name.tar") -Force
  }
} catch {
  L "ERROR $($_.Exception.Message)"
} finally {
  if ($s) { Remove-PSSession $s -ErrorAction SilentlyContinue }
  if ($took) {
    Stop-VM -Name $VM -TurnOff -Force -ErrorAction SilentlyContinue
    Restore-VMSnapshot -VMName $VM -Name $cp -Confirm:$false -ErrorAction SilentlyContinue
    Remove-VMSnapshot -VMName $VM -Name $cp -ErrorAction SilentlyContinue
    $v2 = Get-VMVideo -VMName $VM
    if ($vid -and ($v2.ResolutionType -ne $vid.ResolutionType -or $v2.HorizontalResolution -ne $vid.HorizontalResolution -or $v2.VerticalResolution -ne $vid.VerticalResolution)) {
      Set-VMVideo -VMName $VM -ResolutionType $vid.ResolutionType -HorizontalResolution $vid.HorizontalResolution -VerticalResolution $vid.VerticalResolution
      $v2 = Get-VMVideo -VMName $VM
    }
    L "restored: state $((Get-VM $VM).State), checkpoints $(@(Get-VMSnapshot -VMName $VM).Count), video $($v2.ResolutionType) $($v2.HorizontalResolution)x$($v2.VerticalResolution)"
  }
  'done' | Set-Content $doneFile
}
