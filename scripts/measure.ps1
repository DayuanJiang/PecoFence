# Samples PecoFence's resource usage (plan §10.3 task 25 / §8 budgets).
#
#   scripts/measure.ps1                 # sample the running pecofence.exe for 30 s
#   scripts/measure.ps1 -Seconds 120 -Interval 2 -Csv target/measure.csv
#   scripts/measure.ps1 -Launch         # start target/release/pecofence.exe first, then sample
#
# Reports Private WS / Working Set / Commit (Private Bytes), CPU % (process, all cores), GDI and
# USER handle counts, thread count, plus the watchdog's footprint. Budgets from the plan:
# idle Private WS < 40 MB, idle CPU < 0.5 %.
param(
    [int]$Seconds = 30,
    [double]$Interval = 1.0,
    [string]$Csv = "",
    [switch]$Launch
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

Add-Type -Namespace OF -Name Native -MemberDefinition @'
[DllImport("user32.dll")] public static extern uint GetGuiResources(IntPtr hProcess, uint uiFlags);
'@

if ($Launch) {
    $exe = Join-Path $root "target/release/pecofence.exe"
    if (-not (Test-Path $exe)) { throw "build first: cargo build --release" }
    Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe)
    Start-Sleep -Seconds 3
}

function PrivateWorkingSet($procId) {
    # "Working Set - Private" is what Task Manager's Memory column shows; Process.PrivateMemorySize64
    # is commit (Private Bytes), which runs 2-4x higher and is not the plan's budget metric.
    $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$procId" -ErrorAction SilentlyContinue
    if ($perf) { [math]::Round($perf.WorkingSetPrivate / 1MB, 1) } else { $null }
}

function Sample($proc) {
    $proc.Refresh()
    [pscustomobject]@{
        PrivateWS_MB = PrivateWorkingSet $proc.Id
        Commit_MB = [math]::Round($proc.PrivateMemorySize64 / 1MB, 1)
        WorkingSet_MB = [math]::Round($proc.WorkingSet64 / 1MB, 1)
        Threads = $proc.Threads.Count
        Handles = $proc.HandleCount
        GDI = [OF.Native]::GetGuiResources($proc.Handle, 0)
        USER = [OF.Native]::GetGuiResources($proc.Handle, 1)
        CpuSeconds = $proc.TotalProcessorTime.TotalSeconds
    }
}

$main = Get-Process -Name pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $main) { throw "pecofence.exe is not running (use -Launch)" }
$dog = Get-Process -Name pecofence-watchdog -ErrorAction SilentlyContinue | Select-Object -First 1
$cores = [Environment]::ProcessorCount

$rows = @()
$prev = Sample $main
$t0 = Get-Date
Write-Host ("sampling PID {0} for {1}s every {2}s ({3} cores)" -f $main.Id, $Seconds, $Interval, $cores)
while (((Get-Date) - $t0).TotalSeconds -lt $Seconds) {
    Start-Sleep -Milliseconds ([int]($Interval * 1000))
    if ($main.HasExited) { Write-Warning "process exited"; break }
    $cur = Sample $main
    $cpuPct = [math]::Round(100 * ($cur.CpuSeconds - $prev.CpuSeconds) / $Interval / $cores, 2)
    $row = [pscustomobject]@{
        T = [math]::Round(((Get-Date) - $t0).TotalSeconds, 1)
        PrivateWS_MB = $cur.PrivateWS_MB
        WorkingSet_MB = $cur.WorkingSet_MB
        CPU_Pct = $cpuPct
        Threads = $cur.Threads
        Handles = $cur.Handles
        GDI = $cur.GDI
        USER = $cur.USER
    }
    $rows += $row
    $prev = $cur
}

$rows | Format-Table -AutoSize | Out-String | Write-Host
$summary = [pscustomobject]@{
    Samples = $rows.Count
    PrivateWS_MB_Avg = [math]::Round(($rows | Measure-Object PrivateWS_MB -Average).Average, 1)
    PrivateWS_MB_Max = ($rows | Measure-Object PrivateWS_MB -Maximum).Maximum
    CPU_Pct_Avg = [math]::Round(($rows | Measure-Object CPU_Pct -Average).Average, 2)
    CPU_Pct_Max = ($rows | Measure-Object CPU_Pct -Maximum).Maximum
    GDI_Max = ($rows | Measure-Object GDI -Maximum).Maximum
    USER_Max = ($rows | Measure-Object USER -Maximum).Maximum
    Watchdog_PrivateWS_MB = if ($dog) { [math]::Round($dog.PrivateMemorySize64 / 1MB, 1) } else { $null }
    Exe_KB = [math]::Round((Get-Item $main.Path).Length / 1KB)
}
Write-Host "summary:"
$summary | Format-List | Out-String | Write-Host
$budgetWs = 40; $budgetCpu = 0.5
if ($summary.PrivateWS_MB_Avg -gt $budgetWs) { Write-Warning ("Private WS {0} MB exceeds the {1} MB idle budget" -f $summary.PrivateWS_MB_Avg, $budgetWs) }
if ($summary.CPU_Pct_Avg -gt $budgetCpu) { Write-Warning ("CPU {0}% exceeds the {1}% idle budget" -f $summary.CPU_Pct_Avg, $budgetCpu) }

if ($Csv) {
    $rows | Export-Csv -NoTypeInformation -Path $Csv
    Write-Host "wrote $Csv"
}
