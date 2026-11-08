# scripts/wintest/job.ps1: what PFJob runs in pf's desktop session. run.sh puts a two-line
# C:\pf\job.ps1 on the box that sets $PfSteps and dot-sources this file.
# An error in any step line is recorded as a failed check of the current case, and the job goes
# on with the next line, so one broken step does not cost the rest of the run.
trap {
  $where = "$(Split-Path -Leaf $_.InvocationInfo.ScriptName):$($_.InvocationInfo.ScriptLineNumber)"
  if (Get-Command Check -ErrorAction SilentlyContinue) { Check 'no error' $false "$($_.Exception.Message) at $where" }
  else { "harness`tjob`tFAIL`t$($_.Exception.Message) at $where" | Add-Content C:\pf\run\results.tsv -Encoding UTF8 }
  continue
}
. C:\pf\kit\lib.ps1
. $PfSteps
Case 'end'
if (PfAlive) { PfQuit }
# Defender's machine-learning detections sometimes take an unsigned test build (or a kit script)
# away mid-run; then most checks fail for that one reason. Say so in one row.
try {
  $det = @(Get-MpThreatDetection -ErrorAction Stop | Where-Object { $_.InitialDetectionTime -ge $PfJobStart -and ($_.Resources -join ' ') -match 'C:\\pf\\' })
  $names = @($det | ForEach-Object { (Get-MpThreat -ThreatID $_.ThreatID -ErrorAction SilentlyContinue).ThreatName } | Sort-Object -Unique) -join ', '
  $what = (($det | ForEach-Object { "$($_.InitialDetectionTime.ToString('HH:mm:ss')) $($_.Resources -join '; ')" }) -join ' || ')
  Check 'Windows Defender left the test files alone' ($det.Count -eq 0) "$what $(if ($names) { "($names)" })"
} catch { Log "Defender detections unreadable: $_" }
if (@(PfLogFiles).Count -eq 0) { Log 'no app log in this job: the app log check is skipped' }
elseif (-not $PfAllowLogErrors) {
  $errs = @(PfLogErrors)
  Check 'app log has no ERROR or panic line' ($errs.Count -eq 0) ("$(@(PfLogFiles).Count) logs; " + (($errs | Select-Object -First 3) -join ' | '))
}
Set-Content C:\pf\run\done.txt 'done'
