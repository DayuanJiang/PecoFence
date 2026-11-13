# <pecofence.type>streamable</pecofence.type>
# <pecofence.allow>run</pecofence.allow>
# PecoFence 脚本栅栏示例：一直运行，每 2 秒更新一次电脑的状态。
# 上面第一行注释让它一直运行。打印的每一行马上显示；打印一行 ~~~ 以后，下一行开始新的一屏，换掉原来的内容。
# color= 文字颜色，copy= 点了复制，open= 点了打开，run= 点了运行一个程序。
# 脚本要有上面第二行注释，它的行才能启动程序；这个脚本打印的都是本机自己的数值。
# 本机在网络里的 IPv4 地址：取已连接、有默认网关的网卡（拔掉网线的网卡可能还留着旧地址，虚拟网卡没有网关），
# Wi-Fi 和网线排在 VPN 前面，再按 Windows 优先使用的顺序。
function Get-LocalIP {
  Get-NetIPConfiguration |
    Where-Object { $_.NetAdapter.Status -eq 'Up' -and $_.IPv4DefaultGateway } |
    Sort-Object { -not $_.NetAdapter.ConnectorPresent }, { $_.NetIPv4Interface.InterfaceMetric } |
    Select-Object -ExpandProperty IPv4Address |
    Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } |
    Select-Object -First 1 -ExpandProperty IPAddress
}
# 第一屏马上显示，地址跟着第二屏出来（第一次查要几秒钟），之后每 30 秒重新查一次（后来才连上网络，或者换了 Wi-Fi）。
$ip = '查找中'
$tick = 0
while ($true) {
  if ($tick % 15 -eq 1) { $ip = Get-LocalIP }
  $tick++
  $cpu = [int](Get-CimInstance Win32_Processor | Measure-Object -Property LoadPercentage -Average).Average
  $os = Get-CimInstance Win32_OperatingSystem
  $mem = [int](100 - $os.FreePhysicalMemory * 100 / $os.TotalVisibleMemorySize)
  # 本机里有盘的驱动器：可移动盘（2）、本地硬盘（3）和内存盘（6）。不查网络驱动器和光驱。
  $disks = Get-CimInstance Win32_LogicalDisk -Filter 'DriveType = 2 OR DriveType = 3 OR DriveType = 6' |
    Where-Object { $_.Size -gt 0 } |
    Sort-Object DeviceID
  # 所有数值都查完以后才开始打印，这一屏一次换好。
  $cpuColor = if ($cpu -gt 80) { 'red' } elseif ($cpu -gt 50) { 'orange' } else { 'green' }
  "CPU $cpu% | color=$cpuColor icon=🧠"
  "内存已用 $mem% | color=$(if ($mem -gt 85) { 'red' } else { 'green' }) icon=📊"
  "---"
  if ($ip -eq '查找中') {
    "本机 IP：正在查找… | color=gray icon=📋"
  } elseif ($ip) {
    "本机 IP $ip | copy=$ip icon=📋 tooltip=点一下复制"
  } else {
    "本机 IP：没有连接网络 | color=gray icon=📋"
  }
  "打开任务管理器 | run=taskmgr.exe icon=📈"
  # 磁盘放在最后：栅栏放不下时，先看不到的是最后几个盘。
  "---"
  foreach ($disk in $disks) {
    $free = [int]($disk.FreeSpace / 1GB)
    $pct = [int]($disk.FreeSpace * 100 / $disk.Size)
    "$($disk.DeviceID.TrimEnd(':')) 盘剩余 $free GB（$pct%） | open=$($disk.DeviceID)\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  }
  "~~~"
  Start-Sleep -Seconds 2
}
