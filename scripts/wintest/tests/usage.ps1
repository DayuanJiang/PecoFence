# 发送使用统计 (docs/USAGE_STATS.md). A test run records only when PECOFENCE_USAGE_URL names a
# local listener, so the steps start one. They check: an earlier day's report waiting in
# usage.json is POSTed about a minute after the start with x-amz-content-sha256 = the SHA-256
# of the body, and forgotten after the 2xx answer; a few operations appear in usage.json and
# nothing typed does; switching the setting off deletes the file, and switching it on again
# makes a new install number; without the variable nothing is recorded.
function UsageFile() { return "$PfApp\config\usage.json" }
function Reports() {
  $t = Get-Content (UsageFile) -Raw -ErrorAction SilentlyContinue
  if (-not $t) { return @() }
  $r = $t | ConvertFrom-Json
  return @($r | ForEach-Object { $_ })
}
# Today's events as "name key".
function Events() { $r = @(Reports); if (-not $r.Count) { return @() }; return @($r[-1].events | ForEach-Object { "$($_[1]) $($_[2])".Trim() }) }

Case 'setup: a listener, and an earlier day waiting in usage.json'
CloseWindows
# Answers every request with 204 and keeps its head and body in C:\pf\run.
$listener = @'
param([int]$Port)
$l = New-Object System.Net.Sockets.TcpListener([System.Net.IPAddress]::Parse('127.0.0.1'), $Port)
$l.Start()
$n = 0
while ($true) {
  $c = $l.AcceptTcpClient()
  try {
    $s = $c.GetStream(); $s.ReadTimeout = 5000
    $buf = New-Object byte[] 65536; $all = New-Object IO.MemoryStream; $end = -1; $need = 0; $head = ''
    while ($true) {
      $got = $s.Read($buf, 0, $buf.Length); if ($got -le 0) { break }
      $all.Write($buf, 0, $got)
      if ($end -lt 0) {
        $text = [Text.Encoding]::GetEncoding(28591).GetString($all.ToArray()); $i = $text.IndexOf("`r`n`r`n")
        if ($i -ge 0) { $end = $i + 4; $head = $text.Substring(0, $i); if ($head -match '(?im)^Content-Length:\s*(\d+)') { $need = [int]$Matches[1] } }
      }
      if ($end -ge 0 -and $all.Length - $end -ge $need) { break }
    }
    if ($end -lt 0) { continue }
    $n++
    $bytes = $all.ToArray(); $body = New-Object byte[] ($bytes.Length - $end); [Array]::Copy($bytes, $end, $body, 0, $body.Length)
    [IO.File]::WriteAllBytes("C:\pf\run\usage-body-$n.json", $body)
    [IO.File]::WriteAllText("C:\pf\run\usage-head-$n.txt", $head)
    $answer = [Text.Encoding]::ASCII.GetBytes("HTTP/1.1 204 No Content`r`nContent-Length: 0`r`nConnection: close`r`n`r`n")
    $s.Write($answer, 0, $answer.Length); $s.Flush()
  } catch { Add-Content C:\pf\run\usage-listener.err "$_" } finally { $c.Close() }
}
'@
[IO.File]::WriteAllText("$PfRun\serve-usage.ps1", $listener)
$script:Srv = Start-Process powershell.exe -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "$PfRun\serve-usage.ps1", '-Port', '8733' -WindowStyle Hidden -PassThru
$script:OldId = '0123456789abcdef0123456789abcdef'
New-Item -ItemType Directory -Force "$PfApp\config" | Out-Null
[IO.File]::WriteAllText((UsageFile), '[{"v":1,"id":"' + $script:OldId + '","day":"2020-01-01","app":"0.0.1","events":[[60,"start","by_hand"]]}]')
Set-Content "$PfDesk\zqplan notes.txt" 'sample'
$env:PECOFENCE_USAGE_URL = 'http://127.0.0.1:8733'
PfStart
Check 'the listener runs and PecoFence starts' ($script:Srv -and -not $script:Srv.HasExited -and (PfAlive)) ''

Case 'a few operations: the CLI, the Peek shortcut, a search that opens a file'
Pf fence list | Out-Null
DeskClick 1500 820 | Out-Null
PeekKey; Start-Sleep -Milliseconds 800
[G]::Type('zqplan'); Start-Sleep -Milliseconds 900
$box = [M]::BoxText()
[G]::Press(0x0D); Start-Sleep -Seconds 3
Check 'the search took the typed text and opened the file' ($box -eq 'zqplan' -and (NotepadTitles) -like '*zqplan*') "box='$box' notepad=$(NotepadTitles)"
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
for ($i = 0; $i -lt 3 -and [M]::BoxText() -ne $null; $i++) { [G]::Press(0x1B); Start-Sleep -Milliseconds 400 }

Case 'the earlier day is sent about a minute after the start'
for ($i = 0; $i -lt 100 -and -not (Test-Path "$PfRun\usage-head-1.txt"); $i++) { Start-Sleep -Seconds 1 }
Start-Sleep -Seconds 2
$head = Get-Content "$PfRun\usage-head-1.txt" -Raw -ErrorAction SilentlyContinue
$sent = Get-Content "$PfRun\usage-body-1.json" -Raw -ErrorAction SilentlyContinue
$hash = if ($sent) { (Get-FileHash "$PfRun\usage-body-1.json" -Algorithm SHA256).Hash.ToLower() } else { '' }
$said = if ($head -match '(?im)^x-amz-content-sha256:\s*([0-9a-f]{64})\s*$') { $Matches[1] } else { '' }
Check 'one POST /usage with a JSON body' ($head -match '^POST /usage HTTP/1\.1' -and $head -match '(?im)^Content-Type:\s*application/json\s*$' -and -not (Test-Path "$PfRun\usage-head-2.txt")) "$head"
Check 'x-amz-content-sha256 is the SHA-256 of the body' ($hash -and $said -eq $hash) "header=$said body=$hash"
$report = if ($sent) { $sent | ConvertFrom-Json } else { $null }
Check 'the body is the report of the earlier day' ($report.id -eq $script:OldId -and $report.day -eq '2020-01-01' -and $report.v -eq 1) "$sent"

Case 'usage.json after the run'
PfQuit
$text = Get-Content (UsageFile) -Raw -ErrorAction SilentlyContinue
$days = @(Reports)
$ev = @(Events)
Check 'the day the listener took is gone; today''s report goes on with the same number' ($days.Count -eq 1 -and $days[0].id -eq $script:OldId -and $days[0].day -eq (Get-Date -Format 'yyyy-MM-dd')) "days=$($days.Count) $($days | ForEach-Object { $_.day })"
Check 'the operations are there in order' ((($ev -join ',') -match 'start after_update,.*cli fences\.list,.*hotkey peek,.*peek_search_open,.*exit$')) ($ev -join ',')
Check 'nothing typed, no file name, no per-character event' ($text -and $text -notmatch 'zqplan' -and $text -notmatch 'peek_search_type|peek_search_changed') ($ev -join ',')
Check 'the report says the version, the settings and the counts' ($days[0].app -match '^\d+\.\d+\.\d+' -and $days[0].settings.hideRealIcons -is [bool] -and $days[0].state.fences -ge 1 -and $days[0].health.startupMs -gt 0) "app=$($days[0].app) state=$($days[0].state | ConvertTo-Json -Compress) health=$($days[0].health | ConvertTo-Json -Compress)"

Case 'switching it off deletes the file; on again, a new number'
PfStart
Pf settings set usageStats false | Out-Null
Start-Sleep -Seconds 1
$gone = -not (Test-Path (UsageFile))
Pf fence list | Out-Null
PfQuit
Check 'off: usage.json is deleted at once, and the run that follows writes none' ($gone -and -not (Test-Path (UsageFile))) "gone=$gone"
PfStart
Pf settings set usageStats true | Out-Null
PfQuit
$days = @(Reports)
Check 'on again: one report with a new number' ($days.Count -eq 1 -and $days[0].id -match '^[0-9a-f]{32}$' -and $days[0].id -ne $script:OldId) "days=$($days.Count) new=$($days.Count -and $days[0].id -ne $script:OldId)"

Case 'a test run without the variable records nothing'
Remove-Item (UsageFile) -ErrorAction SilentlyContinue
Remove-Item Env:PECOFENCE_USAGE_URL
PfStart
Pf fence list | Out-Null
PfQuit
Check 'no usage.json, and the log says so' (-not (Test-Path (UsageFile)) -and @(PfLogAll | Where-Object { $_ -match 'usage statistics: not recorded in this run' }).Count -ge 1) ''
Check 'no request but the first' (-not (Test-Path "$PfRun\usage-head-2.txt")) ''
if ($script:Srv -and -not $script:Srv.HasExited) { $script:Srv | Stop-Process -Force }
