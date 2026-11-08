# <pecofence.type>streamable</pecofence.type>
# PecoFence script fence example: keeps running and shows the computer's state every 2 seconds.
# The comment above keeps it running. Each line it prints shows at once; after a ~~~ line
# the next line starts a new screen in place of the old one.
# color= colours a row, copy= copies text on a click, open= opens a file or folder, run= runs a
# program.
# This PC's IPv4 address on the network: from a connected adapter with a default gateway (an
# unplugged adapter can keep an old address, virtual networks have none), Wi-Fi and Ethernet
# before VPNs, then the one Windows uses first.
function Get-LocalIP {
  Get-NetIPConfiguration |
    Where-Object { $_.NetAdapter.Status -eq 'Up' -and $_.IPv4DefaultGateway } |
    Sort-Object { -not $_.NetAdapter.ConnectorPresent }, { $_.NetIPv4Interface.InterfaceMetric } |
    Select-Object -ExpandProperty IPv4Address |
    Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
    Select-Object -First 1 -ExpandProperty IPAddress
}
$tick = 0
while ($true) {
  # The address is looked up again every 30 seconds (a network joined later, another Wi-Fi).
  if ($tick % 15 -eq 0) { $ip = Get-LocalIP }
  $tick++
  $cpu = [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
  $os = Get-CimInstance Win32_OperatingSystem
  $mem = [int](100 - $os.FreePhysicalMemory * 100 / $os.TotalVisibleMemorySize)
  # Local drives with a disk in them: removable (2), fixed (3) and RAM disks (6). Network drives
  # and CD/DVD drives are not asked.
  $disks = Get-CimInstance Win32_LogicalDisk -Filter 'DriveType = 2 OR DriveType = 3 OR DriveType = 6' |
    Where-Object { $_.Size -gt 0 } |
    Sort-Object DeviceID
  # All values are read before the first line prints, so the screen changes in one go.
  $cpuColor = if ($cpu -gt 80) { 'red' } elseif ($cpu -gt 50) { 'orange' } else { 'green' }
  "CPU $cpu% | color=$cpuColor icon=🧠"
  "Memory used $mem% | color=$(if ($mem -gt 85) { 'red' } else { 'green' }) icon=📊"
  foreach ($disk in $disks) {
    $free = [int]($disk.FreeSpace / 1GB)
    $pct = [int]($disk.FreeSpace * 100 / $disk.Size)
    "$($disk.DeviceID) free $free GB ($pct%) | open=$($disk.DeviceID)\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  }
  "---"
  if ($ip) {
    "IP address $ip | copy=$ip icon=📋 tooltip=""Click to copy"""
  } else {
    "IP address: not connected | color=gray icon=📋"
  }
  "Open Task Manager | run=taskmgr.exe icon=📈"
  "~~~"
  Start-Sleep -Seconds 2
}
