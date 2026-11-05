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
if (-not $PfAllowLogErrors) {
  $errs = @(PfLogErrors)
  Check 'app log has no ERROR or panic line' ($errs.Count -eq 0) (($errs | Select-Object -First 3) -join ' | ')
}
Set-Content C:\pf\run\done.txt 'done'
