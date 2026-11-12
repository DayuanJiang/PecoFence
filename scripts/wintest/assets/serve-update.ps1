# A local release feed for tests/update-check.ps1 and tests/owner2-update.ps1 on
# http://127.0.0.1:<Port>/: GitHub's releases/latest answer for -Tag (the test passes one patch
# above the build), that tag's Store notes for zh-CN and en-US (other languages answer 404), and
# the website's copy of the release at /update.json (its lines for en and zh-CN say "from the
# site", so a test can tell the two sources apart). Its setup file is not served (404), so an
# update stops at the download. Requests go to C:\pf\run\feed.log.
# -Hang: a server that takes connections and never answers (a network that swallows the request).
param([int]$Port = 8721, [string]$Tag = 'v0.3.3', [switch]$Hang)
$base = "http://127.0.0.1:$Port/"
$name = 'pecofence-' + $Tag + '-x64-setup.exe'
$json = '{"tag_name":"' + $Tag + '","html_url":"https://example.invalid/","body":"\u8fd9\u4e00\u7248\u4fee\u597d\u4e86\u901f\u89c8\u641c\u7d22\u91cc\u8f93\u5165\u4e2d\u6587\u65f6\u5076\u5c14\u5361\u4f4f\u7684\u95ee\u9898\uff0c\u5e76\u4e14\u4f1a\u81ea\u5df1\u68c0\u67e5\u6709\u6ca1\u6709\u65b0\u7248\u672c\u3002\n\n## \u65b0\u529f\u80fd\n- \u81ea\u52a8\u68c0\u67e5\u66f4\u65b0","assets":[{"name":"' + $name + '","browser_download_url":"' + $base + $Tag + '/' + $name + '"}]}'
$notes = @{
  'zh-CN' = '{"label":"zh-CN","releaseNotes":"\u672c\u6b21\u66f4\u65b0\uff1a\n\u00b7 \u901f\u89c8\u641c\u7d22\u91cc\u8f93\u5165\u4e2d\u6587\u4e0d\u518d\u5361\u987f\u3002\n\u00b7 PecoFence \u4f1a\u81ea\u5df1\u68c0\u67e5\u6709\u6ca1\u6709\u65b0\u7248\u672c\u3002"}'
  'en-US' = '{"label":"en-US","releaseNotes":"What''s new:\n\u00b7 Typing Chinese in the Peek search no longer stutters.\n\u00b7 PecoFence checks for new versions by itself."}'
}
# The website's update.json (scripts/build-site.py writes the real one).
$site = '{"version":"' + $Tag.TrimStart('v') + '","setup":true,"page":"https://pecofence.jiang.jp/#download","notes":{"en":"From the site: typing Chinese in the Peek search no longer stutters.","zh-CN":"\u6765\u81ea\u7f51\u7ad9\uff1a\u901f\u89c8\u641c\u7d22\u91cc\u8f93\u5165\u4e2d\u6587\u4e0d\u518d\u5361\u987f\u3002"}}'
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), $Port)
$l.Start()
"$(Get-Date -Format HH:mm:ss.fff) listening $base$(if ($Hang) { ' (never answers)' })" | Add-Content C:\pf\run\feed.log
# Connections wait in the listener's queue: the request is sent and no answer comes.
while ($Hang) { Start-Sleep -Seconds 60 }
while ($true) {
  $c = $l.AcceptTcpClient()
  try {
    $s = $c.GetStream()
    $r = New-Object System.IO.StreamReader($s)
    $line = $r.ReadLine()
    while ($true) { $h = $r.ReadLine(); if ($null -eq $h -or $h -eq '') { break } }
    $path = ($line -split ' ')[1]
    $lang = if ($path -match "^/DayuanJiang/PecoFence/$Tag/docs/store/v2-i18n/([A-Za-z-]+)\.json$") { $Matches[1] } else { $null }
    if ($path -eq '/repos/DayuanJiang/PecoFence/releases/latest') { $body = [Text.Encoding]::UTF8.GetBytes($json); $type = 'application/json'; $code = '200 OK' }
    elseif ($path -eq '/update.json') { $body = [Text.Encoding]::UTF8.GetBytes($site); $type = 'application/json'; $code = '200 OK' }
    elseif ($lang -and $notes.ContainsKey($lang)) { $body = [Text.Encoding]::UTF8.GetBytes($notes[$lang]); $type = 'text/plain; charset=utf-8'; $code = '200 OK' }
    else { $body = [Text.Encoding]::UTF8.GetBytes('not found'); $type = 'text/plain'; $code = '404 Not Found' }
    $head = [Text.Encoding]::ASCII.GetBytes("HTTP/1.1 $code`r`nContent-Type: $type`r`nContent-Length: $($body.Length)`r`nConnection: close`r`n`r`n")
    $s.Write($head, 0, $head.Length); $s.Write($body, 0, $body.Length); $s.Flush()
    "$(Get-Date -Format HH:mm:ss.fff) $line -> $code $($body.Length) bytes" | Add-Content C:\pf\run\feed.log
  } catch { "$(Get-Date -Format HH:mm:ss.fff) error $_" | Add-Content C:\pf\run\feed.log }
  finally { $c.Close() }
}
