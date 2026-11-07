# <pecofence.type>streamable</pecofence.type>
# PecoFence 脚本栅栏示例：一直运行，每 2 秒更新一次电脑的状态。
# 上面那一行注释让它一直运行。打印的每一行马上显示；打印一行 ~~~ 以后，下一行开始新的一屏，换掉原来的内容。
# color= 文字颜色，copy= 点了复制，open= 点了打开，run= 点了运行一个程序。
# 排除已断开的网卡，以及没有 IPv4 默认网关的虚拟或纯内网网卡。
$ip = (Get-NetIPConfiguration |
  Where-Object { $_.NetAdapter.Status -eq 'Up' -and $_.IPv4DefaultGateway } |
  Select-Object -ExpandProperty IPv4Address |
  Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
  Select-Object -First 1).IPAddress
while ($true) {
  $cpu = [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
  $os = Get-CimInstance Win32_OperatingSystem
  $mem = [int](100 - $os.FreePhysicalMemory * 100 / $os.TotalVisibleMemorySize)
  $cpuColor = if ($cpu -gt 80) { 'red' } elseif ($cpu -gt 50) { 'orange' } else { 'green' }
  "CPU $cpu% | color=$cpuColor icon=🧠"
  "内存已用 $mem% | color=$(if ($mem -gt 85) { 'red' } else { 'green' }) icon=📊"
  $disks = Get-CimInstance Win32_LogicalDisk |
    Where-Object { $_.DriveType -in 2, 3, 5, 6 -and $_.Size -gt 0 } |
    Sort-Object DeviceID
  foreach ($disk in $disks) {
    $free = [int]($disk.FreeSpace / 1GB)
    $pct = [int]($disk.FreeSpace * 100 / $disk.Size)
    "$($disk.DeviceID.TrimEnd(':')) 盘剩余 $free GB（$pct%） | open=$($disk.DeviceID)\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  }
  "---"
  "本机 IP $ip | copy=$ip icon=📋 tooltip=点一下复制"
  "打开任务管理器 | run=taskmgr.exe icon=📈"
  "~~~"
  Start-Sleep -Seconds 2
}
