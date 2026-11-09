# A local release feed for the update samples: GitHub's releases/latest answer for v0.3.3 on
# http://127.0.0.1:<Port>/ (no setup file: the samples never install). Requests go to
# C:\pf\run\feed.log.
param([int]$Port = 8721)
$Tag = 'v0.3.3'
$base = "http://127.0.0.1:$Port/"
$name = 'pecofence-' + $Tag + '-x64-setup.exe'
$json = '{"tag_name":"' + $Tag + '","html_url":"https://example.invalid/","body":"\u8fd9\u4e00\u7248\u4fee\u597d\u4e86\u901f\u89c8\u641c\u7d22\u91cc\u8f93\u5165\u4e2d\u6587\u65f6\u5076\u5c14\u5361\u4f4f\u7684\u95ee\u9898\uff0c\u5e76\u4e14\u4f1a\u81ea\u5df1\u68c0\u67e5\u6709\u6ca1\u6709\u65b0\u7248\u672c\u3002\n\n## \u65b0\u529f\u80fd\n- \u81ea\u52a8\u68c0\u67e5\u66f4\u65b0","assets":[{"name":"' + $name + '","browser_download_url":"' + $base + $Tag + '/' + $name + '"}]}'
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), $Port)
$l.Start()
"$(Get-Date -Format HH:mm:ss.fff) listening $base" | Add-Content C:\pf\run\feed.log
while ($true) {
  $c = $l.AcceptTcpClient()
  try {
    $s = $c.GetStream()
    $r = New-Object System.IO.StreamReader($s)
    $line = $r.ReadLine()
    while ($true) { $h = $r.ReadLine(); if ($null -eq $h -or $h -eq '') { break } }
    $path = ($line -split ' ')[1]
    if ($path -eq '/repos/DayuanJiang/PecoFence/releases/latest') { $body = [Text.Encoding]::UTF8.GetBytes($json); $type = 'application/json'; $code = '200 OK' }
    else { $body = [Text.Encoding]::UTF8.GetBytes('not found'); $type = 'text/plain'; $code = '404 Not Found' }
    $head = [Text.Encoding]::ASCII.GetBytes("HTTP/1.1 $code`r`nContent-Type: $type`r`nContent-Length: $($body.Length)`r`nConnection: close`r`n`r`n")
    $s.Write($head, 0, $head.Length); $s.Write($body, 0, $body.Length); $s.Flush()
    "$(Get-Date -Format HH:mm:ss.fff) $line -> $code $($body.Length) bytes" | Add-Content C:\pf\run\feed.log
  } catch { "$(Get-Date -Format HH:mm:ss.fff) error $_" | Add-Content C:\pf\run\feed.log }
  finally { $c.Close() }
}
