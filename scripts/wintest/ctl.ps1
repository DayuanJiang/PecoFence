# scripts/wintest/ctl.ps1: box-side commands of run.sh, run over ssh as Administrator.
#   reset    end test processes, empty C:\pf\run, remove pf's PecoFence data
#   check    parse every .ps1 in C:\pf\kit and its steps, and find functions shadowed by aliases
#   start    start the PFJob task (after an earlier run of it has ended)
#   status   done / running / ended (ended = the task stopped without writing done.txt)
#   abort    stop the task and the test processes
#   collect  copy the app logs into C:\pf\run\logs and pack C:\pf\run into C:\pf\run.tar
param([string]$Cmd)
$ErrorActionPreference = 'Continue'
$pfLocal = 'C:\Users\pf\AppData\Local'
$pfRoaming = 'C:\Users\pf\AppData\Roaming'

function StopTestProcesses() {
  Get-Process pecofence, pecofence-watchdog, pecofence-cli, notepad -ErrorAction SilentlyContinue | Stop-Process -Force
}

switch ($Cmd) {
  'reset' {
    StopTestProcesses
    Start-Sleep -Milliseconds 500
    Remove-Item -Recurse -Force C:\pf\run, C:\pf\run.tar, "$pfLocal\PecoFence", "$pfRoaming\PecoFence", "$pfRoaming\OpenFence" -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force C:\pf\run | Out-Null
    'reset ok'
  }
  'check' {
    $bad = 0
    $files = @(Get-ChildItem C:\pf\kit\*.ps1, C:\pf\kit\steps\*.ps1)
    foreach ($f in $files) {
      $tokens = $null; $errs = $null
      $ast = [System.Management.Automation.Language.Parser]::ParseFile($f.FullName, [ref]$tokens, [ref]$errs)
      foreach ($e in $errs) { "PARSE $($f.Name):$($e.Extent.StartLineNumber) $($e.Message)"; $bad++ }
      if (-not $ast) { continue }
      $defs = $ast.FindAll({ $args[0] -is [System.Management.Automation.Language.FunctionDefinitionAst] }, $true)
      foreach ($fn in $defs) {
        $a = Get-Alias -Name $fn.Name -ErrorAction SilentlyContinue
        if ($a) { "ALIAS $($f.Name):$($fn.Extent.StartLineNumber) function $($fn.Name) is shadowed by the alias $($a.Name) -> $($a.Definition)"; $bad++ }
      }
    }
    "checked $($files.Count) files: $bad problems"
  }
  'start' {
    for ($i = 0; $i -lt 60 -and (Get-ScheduledTask PFJob).State -eq 'Running'; $i++) { Start-Sleep -Seconds 1 }
    if ((Get-ScheduledTask PFJob).State -eq 'Running') { 'busy: PFJob is still running another job'; break }
    Start-ScheduledTask -TaskName PFJob
    for ($i = 0; $i -lt 40; $i++) {
      if ((Get-ScheduledTask PFJob).State -eq 'Running' -or (Test-Path C:\pf\run\done.txt)) { 'started'; break }
      Start-Sleep -Milliseconds 250
    }
    if ($i -ge 40) { 'not started: PFJob did not run (is pf signed in?)' }
  }
  'status' {
    # The task state first: the job writes done.txt before it exits, so a stopped task without
    # done.txt really ended early (the other order races with a job that just finished).
    $state = (Get-ScheduledTask PFJob).State
    if (Test-Path C:\pf\run\done.txt) { 'done' }
    elseif ($state -eq 'Running') { 'running' }
    else { 'ended' }
  }
  'abort' {
    Stop-ScheduledTask -TaskName PFJob -ErrorAction SilentlyContinue
    StopTestProcesses
    'aborted'
  }
  'collect' {
    # Nothing may still write into C:\pf\run while it is packed (a half-written file breaks
    # the archive), so the job's programs end first.
    StopTestProcesses
    Start-Sleep -Milliseconds 500
    New-Item -ItemType Directory -Force C:\pf\run\logs | Out-Null
    Copy-Item "$pfLocal\PecoFence\*.log" C:\pf\run\logs -ErrorAction SilentlyContinue
    Remove-Item C:\pf\run.tar -ErrorAction SilentlyContinue
    # The files at the top of C:\pf\run plus logs\ and shots\; other folders are the steps'
    # working data (the portable app, demo folders with Chinese names, ...).
    $items = @(Get-ChildItem C:\pf\run -File | ForEach-Object { "run\$($_.Name)" }) + @('run\logs', 'run\shots')
    $err = & tar.exe -cf C:\pf\run.tar -C C:\pf @items 2>&1 | Out-String
    'collected ' + (Get-Item C:\pf\run.tar).Length + ' ' + $err.Trim()
  }
  default { "unknown command '$Cmd'" }
}
