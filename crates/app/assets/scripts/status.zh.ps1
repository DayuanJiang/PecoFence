# <pecofence.type>streamable</pecofence.type>
# PecoFence 脚本栅栏示例：一直运行，每 2 秒更新一次电脑的状态。
# 上面那一行注释让它一直运行；每打印一行 ~~~，栅栏就换成新内容。
# color= 文字颜色，copy= 点了复制，open= 点了打开，run= 点了运行一个程序。
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
  "内存已用 $mem% | color=$(if ($mem -gt 85) { 'red' } else { 'green' }) icon=📊"
  "C 盘剩余 $free GB（$pct%） | open=C:\ color=$(if ($pct -lt 10) { 'red' } else { 'green' }) icon=💽"
  "---"
  "本机 IP $ip | copy=$ip icon=📋 tooltip=点一下复制"
  "打开任务管理器 | run=taskmgr.exe icon=📈"
  "~~~"
  Start-Sleep -Seconds 2
}
