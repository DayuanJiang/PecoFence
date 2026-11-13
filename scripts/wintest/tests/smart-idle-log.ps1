# 智能栅栏, the log while nothing happens: the once-a-minute run of a smart fence wrote two lines
# per fence and minute at the default level (about 26 KB an hour for each smart fence), all an
# idle PecoFence logged. Those lines are debug lines now; a run somebody asked for and a change
# the minute run finds still show at info. With or without Everything (the folder is read
# without it). About 1 minute.
Case 'setup'
$dir = Join-Path $PfDocs 'smart-idle-log'
New-Item -ItemType Directory -Force $dir | Out-Null
1..5 | ForEach-Object { [IO.File]::WriteAllText((Join-Path $dir "report $_.pdf"), 'x') }
# The default level for everything, debug for the smart fences only (so the minute runs can be
# seen at all); the script runs the minute's housekeeping three times.
PfStart -Lang en -LogLevel 'info,pecofence::app::smart=debug' -Script @('sleep 12000', 'housekeeping', 'sleep 4000', 'housekeeping', 'sleep 4000', 'housekeeping', 'sleep 4000', 'dump minutes', 'sleep 3600000')
Pf fence create --title Reports --smart-folder $dir --ext pdf --rect '420,140,420,300' | Out-Null
Check 'the smart fence shows the five files' ((PfWaitLog 'smart fence results changed.*count=5' 20) -ne $null) ''

Case 'the minute run is quiet at the default level'
PfWaitLog '>> dump minutes' 40 | Out-Null
$lines = @(PfLogAll | Where-Object { $_ -match 'smart fence' -and $_ -match 'why="minute"' })
$info = @($lines | Where-Object { $_ -cmatch ' INFO ' })
$debug = @($lines | Where-Object { $_ -cmatch ' DEBUG ' })
Check 'the three minute runs ran (debug lines)' ($debug.Count -ge 3) "debug lines: $($debug.Count)"
Check 'no minute run wrote a line at the default level' ($info.Count -eq 0) "info lines: $($info.Count); $($info | Select-Object -First 1)"
$answers = @(PfLogAll | Where-Object { $_ -match 'smart fence: (folder read without Everything|Everything answered)' })
Check 'their answers are debug lines too' (@($answers | Where-Object { $_ -cmatch ' DEBUG ' }).Count -ge 3) "answers: $($answers.Count), debug: $(@($answers | Where-Object { $_ -cmatch ' DEBUG ' }).Count)"

Case 'what somebody asked for still shows'
Pf fence refresh Reports | Out-Null
Check 'a refresh from the CLI logs its run at info' ((PfWaitLog ' INFO .*smart fence.*why="cli"' 15) -ne $null) ''
[IO.File]::WriteAllText((Join-Path $dir 'report 6.pdf'), 'x')
Check 'a new file is logged as a change' ((PfWaitLog ' INFO .*smart fence results changed.*count=6' 20) -ne $null) ''
PfQuit
Remove-Item -Recurse -Force $dir -ErrorAction SilentlyContinue
