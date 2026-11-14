# <pecofence.type>streamable</pecofence.type>
# <pecofence.allow>run</pecofence.allow>
# PecoFence script fence example: keeps running and shows the computer's state every 2 seconds.
# The first comment above keeps it running. Each line it prints shows at once; after a ~~~ line
# the next line starts a new screen in place of the old one.
# color= colours a row, copy= copies text on a click, open= opens a file or folder, run= runs a
# program. Rows start programs only in a script that allows it with the second comment above;
# this one prints nothing but this computer's own values.
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
# The first screen shows at once and the address comes with the second (the first lookup takes a
# few seconds); then it is looked up every 30 seconds (a network joined later, another Wi-Fi).
$ip = 'looking'
$tick = 0
while ($true) {
  if ($tick % 15 -eq 1) { $ip = Get-LocalIP }
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
  "---"
  if ($ip -eq 'looking') {
    "IP address: looking it up… | color=gray icon=📋"
  } elseif ($ip) {
    "IP address $ip | copy=$ip icon=📋 tooltip=""Click to copy"""
  } else {
    "IP address: not connected | color=gray icon=📋"
  }
  "Open Task Manager | run=taskmgr.exe icon=📈"
  # The drives come last: when the fence is too short, the last drives are what goes out of view.
  "---"
  foreach ($disk in $disks) {
    $free = [int]($disk.FreeSpace / 1GB)
    $pct = [int]($disk.FreeSpace * 100 / $disk.Size)
    "$($disk.DeviceID) free $free GB ($pct%) | open=$($disk.DeviceID)\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  }
  "~~~"
  Start-Sleep -Seconds 2
}
