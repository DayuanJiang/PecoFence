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
# An EC2 box's desktop starts every job with only the two shortcuts the box came with. What an
# earlier job left (a job that was stopped, files moved there with an older creation time, which
# the cleanup at the end of this file does not see) shows up in this job's fences (2026-11-12:
# forty left files made the desktop fence 822 px tall, and it pushed the fence under test).
# A Hyper-V guest's desktop is left as it is: only the EC2 boxes have these shortcuts.
if (Test-Path -LiteralPath (Join-Path $PfDesk 'EC2 Feedback.url')) {
  Get-ChildItem -LiteralPath $PfDesk -Force -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -ne 'desktop.ini' -and $_.Name -notlike 'EC2 *.url' } |
    ForEach-Object {
      Log "desktop: removing what an earlier job left: $($_.Name)"
      Remove-Item -LiteralPath $_.FullName -Recurse -Force -ErrorAction SilentlyContinue
    }
  # The same for Windows' app colour mode: a job that was cut off while it had the box on dark
  # left the next jobs' pictures dark (2026-11-13: a fade check read a dark search box as "no
  # fade" twice on one box). Steps that want dark set it themselves.
  $themeKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
  $apps = (Get-ItemProperty -Path $themeKey -ErrorAction SilentlyContinue).AppsUseLightTheme
  if ($null -ne $apps -and $apps -eq 0) {
    Log 'colour mode: an earlier job left Windows on dark for apps: set back to light'
    Set-ItemProperty -Path $themeKey -Name AppsUseLightTheme -Value 1 -Type DWord
  }
}
. $PfSteps
Case 'end'
if (PfAlive) { PfQuit }
# What the steps put on the desktop goes again: left there, it shows up in the next job's fences
# (2026-11-12: a file copied by an earlier job read as "Ctrl+drag makes a copy").
Get-ChildItem -LiteralPath $PfDesk -Force -ErrorAction SilentlyContinue |
  Where-Object { $_.CreationTime -ge $PfJobStart } |
  ForEach-Object { Remove-Item -LiteralPath $_.FullName -Recurse -Force -ErrorAction SilentlyContinue }
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
