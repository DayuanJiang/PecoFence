# 智能栅栏 and the UI thread: a smart fence whose search Everything answers with 5000 files (a
# name filter asks for that many) must not hold up the UI thread while the files are checked and
# the results named. A probe thread sends WM_NULL to the control window every few ms and keeps
# how long each one waited: idle first, then while the fence fills, then over five refreshes.
# Prints the numbers (max, 99th percentile, waits over 100 ms) and the app's own timings.
Case 'setup'
CloseWindows
Check 'Everything runs in pf session' (EvStart) "installed=$(EvInstalled)"
$perf = Join-Path $PfDocs 'smart-perf'
New-Item -ItemType Directory -Force $perf | Out-Null
for ($i = 0; $i -lt 5000; $i++) { [IO.File]::WriteAllText((Join-Path $perf ('perf {0:D4}.txt' -f $i)), 'x') }
Check 'the 5000 files are there' (@(Get-ChildItem -LiteralPath $perf -File).Count -eq 5000) ''
Add-Type @"
using System; using System.Collections.Generic; using System.Diagnostics; using System.Threading; using System.Runtime.InteropServices;
public static class UiProbe {
  [DllImport("user32.dll")] static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  static Thread th; static volatile bool run; static List<double> waits = new List<double>();
  public static void Start(IntPtr h) {
    lock (waits) waits.Clear(); run = true;
    th = new Thread(() => { var sw = new Stopwatch(); IntPtr r; while (run) { sw.Restart(); SendMessageTimeoutW(h, 0, IntPtr.Zero, IntPtr.Zero, 0, 10000, out r); double ms = sw.Elapsed.TotalMilliseconds; lock (waits) waits.Add(ms); Thread.Sleep(2); } });
    th.IsBackground = true; th.Start();
  }
  public static double[] Stop() { run = false; th.Join(); lock (waits) return waits.ToArray(); }
}
"@
# max, 99th percentile and the waits over 100 ms of one probe run.
function ProbeText($w) {
  $s = @($w | Sort-Object)
  if ($s.Count -eq 0) { return @{ max = -1; text = 'no samples' } }
  $max = [math]::Round($s[-1], 1); $p99 = [math]::Round($s[[int][math]::Floor(($s.Count - 1) * 0.99)], 1)
  $over = @($s | Where-Object { $_ -gt 100 }).Count
  return @{ max = $max; text = "samples=$($s.Count) max=$max ms p99=$p99 ms over100=$over" }
}
PfStart -Lang en
Start-Sleep -Seconds 3
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
Check 'the control window is there' ($ctl -ne [IntPtr]::Zero) "$ctl"

Case 'idle'
[UiProbe]::Start($ctl); Start-Sleep -Seconds 8
$idle = ProbeText ([UiProbe]::Stop())
Check 'idle: how long the UI thread keeps a message waiting' $true $idle.text

Case 'the fence fills'
[UiProbe]::Start($ctl)
Pf fence create --title Perf --smart-folder $perf --name-matches perf --rect '420,140,560,330' | Out-Null
$ok = WaitSmart 'Perf' { $s.source -eq 'everything' -and $s.total -ge 5000 -and $s.shown -eq 200 } 40
Start-Sleep -Seconds 2
$fill = ProbeText ([UiProbe]::Stop())
Check 'Everything answers: 5000 in all, 200 shown' $ok (SmartLine 'Perf' | ForEach-Object { $_.Substring(0, [math]::Min(200, $_.Length)) })
Check 'while it fills: how long the UI thread keeps a message waiting' $true $fill.text

Case 'five refreshes'
[UiProbe]::Start($ctl)
for ($i = 0; $i -lt 5; $i++) { Pf fence refresh Perf | Out-Null; Start-Sleep -Seconds 3 }
$again = ProbeText ([UiProbe]::Stop())
Check 'over five refreshes: how long the UI thread keeps a message waiting' $true $again.text
$log = (PfLogAll) -join "`n"
$lines = @([regex]::Matches($log, '.*smart fence: Everything answered.*') | ForEach-Object { ($_.Value -replace '^.*?smart fence: Everything answered\s*', '').Trim() })
Check 'log: the app''s own timings of each answer' ($lines.Count -ge 1) (($lines | Select-Object -Last 3) -join ' | ')
# The UI thread stays free. Before the checks moved to a worker (debug build, local disk) the
# fill kept a message waiting 352 ms and each refresh 130 ms.
Check 'the UI thread never waits 100 ms or more for a smart answer' ($fill.max -lt 100 -and $again.max -lt 100) "idle max=$($idle.max) fill max=$($fill.max) refresh max=$($again.max)"

Case 'cleanup'
Pf fence delete Perf | Out-Null
Remove-Item -LiteralPath $perf -Recurse -Force -ErrorAction SilentlyContinue
Check 'the files are gone' (-not (Test-Path -LiteralPath $perf)) ''
