# The release memory check on the home PC (pc-memory.sh copies this and starts it in the
# signed-in session at normal priority). Runs each build in turn as a portable test instance
# with a copy of the user's configuration (their fences, Liquid Glass, the 电脑状态 fence)
# plus a clock fence with seconds, and records every 30 s what Task Manager shows for the
# PecoFence group (the app and every process it started), the private commit and the GPU
# memory. Never touches the user's own PecoFence.
param(
  [string]$Root = 'C:\Users\me\pfmem-check',
  [string]$Order = 'a,b,a,b',
  [int]$Minutes = 30
)
$ErrorActionPreference = 'Continue'
# The CLI prints UTF-8: read in the console code page, Chinese fence titles break its JSON.
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$Out = "$Root\out"
New-Item -ItemType Directory -Force $Out | Out-Null
function Note($m) { "$(Get-Date -Format HH:mm:ss.fff) $m" | Add-Content "$Out\log.txt" -Encoding UTF8 }
$env:PECOFENCE_INSTANCE = 'memcheck'
$env:PECOFENCE_USAGE_URL = 'http://127.0.0.1:9'

Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public class PfWin {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeoutW(IntPtr h, uint m, IntPtr w, IntPtr l, uint f, uint t, out IntPtr r);
  public static IntPtr Control(uint want) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => { var sb = new StringBuilder(64); GetClassNameW(h, sb, 64); uint p; GetWindowThreadProcessId(h, out p);
      if (sb.ToString() == "PecoFence.Control" && p == want) { found = h; return false; } return true; }, IntPtr.Zero);
    return found;
  }
}
'@

# The process and everything it started, as Task Manager groups them under the app.
function PfGroup([int]$id) {
  $all = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name)
  $ids = @($id); $i = 0
  while ($i -lt $ids.Count) { $ids += @($all | Where-Object { $_.ParentProcessId -eq $ids[$i] } | ForEach-Object { [int]$_.ProcessId }); $i++ }
  return $ids
}

function PrivateWs([int[]]$ids) {
  $s = (Get-Counter '\Process(*)\ID Process', '\Process(*)\Working Set - Private' -ErrorAction SilentlyContinue).CounterSamples
  $byInst = @{}
  foreach ($x in $s) {
    $inst = $x.Path.Split('(')[1].Split(')')[0]
    if (-not $byInst[$inst]) { $byInst[$inst] = @{} }
    if ($x.Path -like '*\id process') { $byInst[$inst].id = [int]$x.CookedValue } else { $byInst[$inst].ws = $x.CookedValue }
  }
  $r = @{}
  foreach ($v in $byInst.Values) { if ($ids -contains $v.id) { $r[$v.id] = $v.ws } }
  return $r
}

function GpuMb([int]$id) {
  $d = 0; $sh = 0
  $c = (Get-Counter "\GPU Process Memory(pid_$($id)_*)\Dedicated Usage", "\GPU Process Memory(pid_$($id)_*)\Shared Usage" -ErrorAction SilentlyContinue).CounterSamples
  foreach ($x in $c) { if ($x.Path -like '*dedicated usage') { $d += $x.CookedValue } else { $sh += $x.CookedValue } }
  return @([math]::Round($d / 1MB, 1), [math]::Round($sh / 1MB, 1))
}

'build,run,second,group_ws_mb,app_ws_mb,app_commit_mb,children,children_ws_mb,gpu_dedicated_mb,gpu_shared_mb,threads,handles,cpu_s' | Set-Content "$Out\samples.csv" -Encoding UTF8
$run = 0
foreach ($b in $Order.Split(',')) {
  $run++
  $dir = "$Root\$b"
  $cli = "$dir\pecofence-cli.exe"
  Remove-Item "$dir\config" -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force "$dir\config" | Out-Null
  Copy-Item "$Root\config.json" "$dir\config\config.json"
  $p = Start-Process "$dir\pecofence.exe" -ArgumentList '--portable', '--no-hide-icons' -WorkingDirectory $dir -PassThru
  $t0 = Get-Date
  Note "run $run build $b pid $($p.Id) priority $($p.PriorityClass)"
  Start-Sleep 20
  $o = & $cli fence create --tool clock --rect '60,60,260,120' 2>&1
  Note "clock fence: $(($o | Out-String).Trim().Length) chars of answer"
  $list = (& $cli fence list 2>$null | Out-String) | ConvertFrom-Json
  $rollId = (@($list) | Where-Object { $_.kind -ne 'inbox' -and -not $_.tool } | Select-Object -First 1).id
  Note "fence rolled every five minutes: $rollId"
  $next = 30
  $rolls = 0
  while (((Get-Date) - $t0).TotalMinutes -lt $Minutes) {
    $wait = $next - ((Get-Date) - $t0).TotalSeconds
    if ($wait -gt 0) { Start-Sleep -Milliseconds ([int]($wait * 1000)) }
    $sec = [int]((Get-Date) - $t0).TotalSeconds
    $p.Refresh()
    if ($p.HasExited) { Note "run $run exited early"; break }
    $ids = PfGroup $p.Id
    $ws = PrivateWs $ids
    $appWs = $ws[$p.Id]
    $kids = @($ids | Where-Object { $_ -ne $p.Id })
    $kidWs = 0; foreach ($k in $kids) { if ($ws[$k]) { $kidWs += $ws[$k] } }
    $g = GpuMb $p.Id
    $mb = { param($v) [math]::Round($v / 1MB, 1) }
    "$b,$run,$sec,$(& $mb ($appWs + $kidWs)),$(& $mb $appWs),$(& $mb $p.PrivateMemorySize64),$($kids.Count),$(& $mb $kidWs),$($g[0]),$($g[1]),$($p.Threads.Count),$($p.HandleCount),$([math]::Round($p.TotalProcessorTime.TotalSeconds, 2))" | Add-Content "$Out\samples.csv" -Encoding UTF8
    # Some use every five minutes: a fence rolls up and opens again.
    if ($sec -ge 300 * ($rolls + 1)) {
      $rolls++
      if ($rollId) { & $cli fence roll $rollId 2>&1 | Out-Null; Start-Sleep 2; & $cli fence unroll $rollId 2>&1 | Out-Null }
    }
    $next += 30
  }
  $h = [PfWin]::Control([uint32]$p.Id)
  $r = [IntPtr]::Zero
  [PfWin]::SendMessageTimeoutW($h, 0x11, [IntPtr]0, [IntPtr]1, 2, 5000, [ref]$r) | Out-Null
  [PfWin]::SendMessageTimeoutW($h, 0x16, [IntPtr]1, [IntPtr]1, 2, 5000, [ref]$r) | Out-Null
  Start-Sleep 5
  if (-not $p.HasExited) { Note "run $run did not quit; ending it"; Stop-Process -Id $p.Id -Force }
  Copy-Item "$env:LOCALAPPDATA\PecoFence\pecofence.memcheck.log" "$Out\app-$run-$b.log" -ErrorAction SilentlyContinue
  Start-Sleep 10
}
Note 'done'
