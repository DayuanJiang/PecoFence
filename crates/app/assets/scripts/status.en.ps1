# <pecofence.type>streamable</pecofence.type>
# PecoFence script fence example: keeps running and shows the computer's state every 2 seconds.
# The comment above keeps it running. Each line it prints shows at once; after a ~~~ line
# the next line starts a new screen in place of the old one.
# color= colours a row, copy= copies text on a click, open= opens a file or folder, run= runs a
# program.
# Ignore disconnected adapters and virtual/local-only networks without an IPv4 gateway.
$ip = (Get-NetIPConfiguration |
  Where-Object { $_.NetAdapter.Status -eq 'Up' -and $_.IPv4DefaultGateway } |
  Select-Object -ExpandProperty IPv4Address |
  Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
  Select-Object -First 1).IPAddress
while ($true) {
  $cpu = [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
  $os = Get-CimInstance Win32_OperatingSystem
  $mem = [int](100 - $os.FreePhysicalMemory * 100 / $os.TotalVisibleMemorySize)
  # Finish data queries before output so a slow disk query cannot leave a partial frame.
  $disks = Get-CimInstance Win32_LogicalDisk |
    Where-Object { $_.DriveType -in 2, 3, 5, 6 -and $_.Size -gt 0 } |
    Sort-Object DeviceID
  $cpuColor = if ($cpu -gt 80) { 'red' } elseif ($cpu -gt 50) { 'orange' } else { 'green' }
  "CPU $cpu% | color=$cpuColor icon=🧠"
  "Memory used $mem% | color=$(if ($mem -gt 85) { 'red' } else { 'green' }) icon=📊"
  foreach ($disk in $disks) {
    $free = [int]($disk.FreeSpace / 1GB)
    $pct = [int]($disk.FreeSpace * 100 / $disk.Size)
    "$($disk.DeviceID) free $free GB ($pct%) | open=$($disk.DeviceID)\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  }
  "---"
  "IP address $ip | copy=$ip icon=📋 tooltip=Click to copy"
  "Open Task Manager | run=taskmgr.exe icon=📈"
  "~~~"
  Start-Sleep -Seconds 2
}
