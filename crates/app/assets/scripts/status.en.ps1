# <pecofence.type>streamable</pecofence.type>
# PecoFence script fence example: keeps running and shows the computer's state every 2 seconds.
# The comment above keeps it running; each ~~~ line it prints replaces what the fence shows.
# color= colours a row, copy= copies text on a click, open= opens a file or folder, run= runs a
# program.
$ip = (Get-NetIPAddress -AddressFamily IPv4 |
  Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
  Select-Object -First 1).IPAddress
while ($true) {
  $cpu = [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
  $os = Get-CimInstance Win32_OperatingSystem
  $mem = [int](100 - $os.FreePhysicalMemory * 100 / $os.TotalVisibleMemorySize)
  $c = Get-PSDrive C
  $free = [int]($c.Free / 1GB)
  $pct = [int]($c.Free * 100 / ($c.Free + $c.Used))
  $cpuColor = if ($cpu -gt 80) { 'red' } elseif ($cpu -gt 50) { 'orange' } else { 'green' }
  "CPU $cpu% | color=$cpuColor icon=🧠"
  "Memory used $mem% | color=$(if ($mem -gt 85) { 'red' } else { 'green' }) icon=📊"
  "C: free $free GB ($pct%) | open=C:\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  "---"
  "IP address $ip | copy=$ip icon=📋 tooltip=Click to copy"
  "Open Task Manager | run=taskmgr.exe icon=📈"
  "~~~"
  Start-Sleep -Seconds 2
}
