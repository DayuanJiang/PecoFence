# Hold-out test 2: the bug fixes of PecoFence 0.2.4 (the `## 0.2.4` section of CHANGELOG.md),
# each retested the way the reporter hit it, with the real mouse and keyboard where a user would
# click or drag. Every check states the fixed behaviour: on 0.2.3 (run.sh --bin <0.2.3 folder>)
# a FAIL is the bug showing, on main every check should pass.
# Three parts, each under 9 minutes: run.sh --extra <folder holding part.txt with A, B or C>
# (all three when there is no part.txt).

# ---- lib candidates

# Which parts run: the letters in C:\pf\extra\part.txt. Each part is a function with a trap of
# its own: an error inside a block (if, function) skips the rest of that block, so the steps of a
# part must be flat lines of a function, not of an if block.
$H2Part = 'ABC'
if (Test-Path "$PfExtra\part.txt") { $H2Part = (Get-Content "$PfExtra\part.txt" -Raw).Trim() }
function InPart($p) { return $H2Part.Contains($p) }
Add-Type -AssemblyName WindowsBase, Microsoft.VisualBasic

# The kit's end check matches ' ERROR ' case-insensitively, so a logged text with " error " in it
# (the 反馈 log excerpt's "warning, error and key event lines") fails it: check case-sensitively.
$PfAllowLogErrors = $true
function LogErrorsCs() { return @(PfLogAll | Where-Object { $_ -cmatch ' ERROR |panicked' -and $_ -notmatch 'feedback-info \{' }) }

# (The other helpers this test needed are in the kit since 2026-11-06.)

# ---- the parts

function H2PartA {
trap { PfTrap $_; continue }

Case '12 status liveWallpapers, snapshot restore --dry-run'
PfFresh
$st = PfJson status
$lw = @($st.liveWallpapers)
Check 'status lists liveWallpapers, one entry per monitor' ($lw.Count -eq 1 -and $lw[0].state -in @('none', 'captured', 'notCapturable', 'pending')) "liveWallpapers=$(ConvertTo-Json @($st.liveWallpapers) -Compress -Depth 4)"
Log ("first run fences: " + ((@(PfJson fence list) | ForEach-Object { "$($_.title) [$($_.kind)] $($_.rect.x),$($_.rect.y),$($_.rect.w),$($_.rect.h)" }) -join ' ; '))
Log ("first run rules: " + ((@((PfJson rule list).list) | ForEach-Object { "$($_.name) -> $($_.targetTitle)" }) -join ' ; '))
Pf fence create --title Twelve --rect '900,120,300,260' | Out-Null
Start-Sleep -Seconds 1
Pf snapshot save h2-twelve | Out-Null
Pf fence roll Twelve | Out-Null
Start-Sleep -Seconds 1
Pf fence move Twelve --x 1000 | Out-Null
Start-Sleep -Seconds 1
$dry = (Pf snapshot restore h2-twelve --dry-run).Trim()
$f = FenceGet Twelve
Check '--dry-run lists the fence the restore would move and unroll' ($dry -match '"dryRun":\s*true' -and $dry -match 'Twelve' -and $dry -match 'rolledUp') $dry
Check 'and restores nothing' ($f.rolledUp -and $f.rect.x -eq 1000) "rolledUp=$($f.rolledUp) x=$($f.rect.x)"
# Seen once (run 2): `fence move` 25 ms after `fence roll` left the fence open. Ten tries.
$open = @()
foreach ($k in 1..10) {
  Pf fence roll Twelve | Out-Null
  Pf fence move Twelve --x (1000 + $k * 10) | Out-Null
  Start-Sleep -Milliseconds 900
  $open += [bool](-not (FenceGet Twelve).rolledUp)
  Pf fence unroll Twelve | Out-Null
  Start-Sleep -Milliseconds 600
}
Check 'extra: a fence moved right after `fence roll` stays rolled up (10 tries)' (@($open | Where-Object { $_ }).Count -eq 0) "open afterwards: $($open -join ', ')"

Case '9 a restored snapshot gives each fence its roll state and height'
$AnimWas = [Anim]::Get(); [Anim]::Set($true)
PfFresh (DumpScript)
Pf fence create --title Roller --rect '420,140,300,320' | Out-Null
Pf fence create --title Opener --rect '780,140,300,320' | Out-Null
Start-Sleep -Seconds 2
Pf snapshot save h2-rolls | Out-Null
# Opener: taller since the snapshot, then rolled up.
Pf fence resize Opener --h 440 | Out-Null
Start-Sleep -Milliseconds 800
Pf fence roll Opener | Out-Null
Start-Sleep -Seconds 2
# Roller: rolled up just before the restore (a script running `fence roll`, then `snapshot restore`).
Pf fence roll Roller | Out-Null
Pf snapshot restore h2-rolls | Out-Null
Start-Sleep -Seconds 3
$r = FenceGet Roller; $o = FenceGet Opener
$dr = DumpOf Roller; $do = DumpOf Opener
Check 'a fence rolled up just before the restore is open, as in the snapshot' (-not $r.rolledUp -and $dr -and -not $dr.rolled -and ($dr.b - $dr.t) -ge 300) "rolledUp=$($r.rolledUp) window=$($dr.l),$($dr.t),$($dr.r),$($dr.b) rect.h=$($r.rect.h)"
Check 'a fence rolled up since the snapshot opens to the snapshot height (320), not its last (440)' (-not $o.rolledUp -and $do -and [math]::Abs(($do.b - $do.t) - 320) -le 4) "rolledUp=$($o.rolledUp) window h=$($do.b - $do.t) rect.h=$($o.rect.h)"
PfStart (DumpScript)
$r = FenceGet Roller; $o = FenceGet Opener
Check 'after a restart both are still open at 320' (-not $r.rolledUp -and -not $o.rolledUp -and $o.rect.h -eq 320 -and $r.rect.h -eq 320) "Roller rolledUp=$($r.rolledUp) h=$($r.rect.h); Opener rolledUp=$($o.rolledUp) h=$($o.rect.h)"
[Anim]::Set($AnimWas)

Case '49 saving a snapshot when 20 exist and some were saved automatically'
PfFresh
Pf fence create --title Snappy --rect '420,140,300,260' | Out-Null
$ids = @()
foreach ($i in 1..12) { $s = PfJson snapshot save "mine $i"; $ids += "$($s.snapshot.id)" }
Check 'twelve snapshots of my own' ((Snaps).Count -eq 12 -and $ids[0] -match '^[0-9a-f-]{36}$') "count=$((Snaps).Count) first id=$($ids[0])"
# Settings > 快照 > 恢复, eight times: every restore first saves a 恢复快照前 snapshot. Then the
# page's 保存快照. The page's own messages, sent by the test script.
$lines = @('sleep 4000')
foreach ($k in 1..8) { $lines += "message {`"type`":`"action`",`"name`":`"restoreSnapshot`",`"id`":`"$($ids[0])`"}"; $lines += 'sleep 1200' }
$lines += @('message {"type":"action","name":"saveSnapshot","snapshotName":"page save"}', 'sleep 1500', 'dump done49', 'sleep 3600000')
PfStart $lines
$done = PfWaitLog '>> dump done49' 40
$s = Snaps
$mine = @($s | Where-Object { $_.name -like 'mine *' })
Log ("snapshots: " + (($s | ForEach-Object { $_.name }) -join ' | '))
Check 'the page saves a snapshot when 20 exist, 8 of them automatic' ($done -and @($s | Where-Object { $_.name -eq 'page save' }).Count -eq 1 -and $s.Count -eq 20 -and $mine.Count -eq 12) "count=$($s.Count) mine=$($mine.Count) names=$(($s | ForEach-Object { $_.name }) -join ', ')"
$cli = (Pf snapshot save 'cli save').Trim()
$s = Snaps
$mine = @($s | Where-Object { $_.name -like 'mine *' })
Check 'snapshot save in the CLI too, replacing the oldest automatic one' ($cli -notmatch 'limit' -and @($s | Where-Object { $_.name -eq 'cli save' }).Count -eq 1 -and $s.Count -eq 20 -and $mine.Count -eq 12) "reply=$cli count=$($s.Count) mine=$($mine.Count)"

Case '11 the notice after a crash inside ExplorerPatcher names it'
PfQuit
# A crashed run's shareable log as crashlog leaves it: the CRASH line names ExplorerPatcher's
# DLL as the faulting module (the line the next start reads, feedback.rs crash_module).
$t = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ss.ffffffZ')
Add-Content (ShareLog) "$t ERROR pecofence_platform::crashlog: CRASH (unhandled exception) code=0xc0000005 at=ExplorerPatcher.amd64.dll+0xc0c1 module_owner=other module_version=26100.8457.70.3 access=write stack=  ExplorerPatcher.amd64.dll+0xc0c1" -Encoding UTF8
PfStart
$toast = ToastOcr 'PecoFence (crashed|closed)' 8
Shot '11-notice'
$seen = PfWaitLog 'previous run crashed' 5
$ep = PfWaitLog 'the crash was inside ExplorerPatcher' 2
$aumid = @(Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings' -ErrorAction SilentlyContinue | Where-Object { $_.PSChildName -like 'NotifyIconGeneratedAumid*' } | ForEach-Object { "$($_.PSChildName)=$((Get-ItemProperty $_.PSPath).LastNotificationAddedTime)" }) -join '; '
Log "balloon records: $aumid"
Check 'the start finds the crash of the last run' ([bool]$seen) "$seen"
Check 'the start sees it was inside ExplorerPatcher' ([bool]$ep) "log: $ep"
Check 'the notice on screen says so (OCR)' ($toast -match 'ExplorerPatcher' -and $toast -match 'Register as shell extension') "toast: '$toast'"

Case '30 42 the 反馈 form: system facts and the log excerpt'
PfQuit
# The run before was a long one: 3000 lines (240 KB) of shareable log, ended normally, with a
# warning and a display change in the middle (lines an excerpt keeps).
$lines = New-Object Collections.Generic.List[string]
$t0 = (Get-Date).ToUniversalTime().AddHours(-5)
foreach ($n in 1..3000) {
  $ts = $t0.AddSeconds($n * 5).ToString('yyyy-MM-ddTHH:mm:ss.ffffffZ')
  if ($n -eq 1200) { $lines.Add("$ts  WARN pecofence::anchor: fences covered by another window hwnd=0x1a2b3c count=4") }
  elseif ($n -eq 1500) { $lines.Add("$ts  INFO pecofence::app: display changed monitors=2 layout=extended") }
  else { $lines.Add("$ts  INFO pecofence::app::visuals: item icons loaded n=$n count=12 ms=4 cached=true") }
}
$lines.Add("$($t0.AddHours(5).ToString('yyyy-MM-ddTHH:mm:ss.ffffffZ'))  INFO pecofence: process exiting")
[IO.File]::WriteAllLines((ShareLog), $lines, (New-Object Text.UTF8Encoding($false)))
PfStart @('sleep 6000', 'feedback-info', 'sleep 3600000')
$fi = FeedbackInfo 30
$sys = $fi.system
Log ("system facts: " + ($sys | ConvertTo-Json -Compress))
Check 'state: how many fences sit above other windows (feedback #92)' ("$($sys.state)" -match '\d+ above other windows') "$($sys.state)"
Check 'displays: size, position and scale' ("$($sys.displays)" -match '1600x900 @0,0 100%') "$($sys.displays)"
Check 'app: installer, ZIP or Store' ("$($sys.app)" -match '\((installer|ZIP|Microsoft Store)') "$($sys.app)"
Check 'startup: the drives of the program and its settings' ("$($sys.startup)" -match 'app on C:' -and "$($sys.startup)" -match 'config on C:') "$($sys.startup)"
Check 'session: Remote Desktop, live wallpaper, last display change' ("$($sys.session)" -match 'remote desktop yes' -and "$($sys.session)" -match 'live wallpaper' -and "$($sys.session)" -match 'display change') "$($sys.session)"
$log = "$($fi.log)"
$bytes = [Text.Encoding]::UTF8.GetByteCount($log)
Log ("log excerpt: $bytes bytes; markers: " + (($log -split "`n" | Where-Object { $_ -match '^---' }) -join ' / '))
Check 'the log goes up to 50 KB (24 KB before)' ($bytes -gt 26 * 1024 -and $bytes -le 50 * 1024 + 256) "$bytes bytes"
Check 'the run before goes along, since this one just started' ($log -match '--- previous run ---' -and $log -match '--- this run ---') (($log -split "`n" | Where-Object { $_ -match '^---' }) -join ' / ')
Check 'its first lines, and the warning and the display change from the stretch left out' ($log -match ' n=1 count' -and $log -match 'fences covered by another window' -and $log -match 'display changed monitors=2' -and $log -match 'lines left out') "first=$($log -match ' n=1 count') warn=$($log -match 'fences covered') display=$($log -match 'display changed monitors=2')"
# The run before crashed (its last line a CRASH line): both versions send its log; how much?
PfQuit
$lines.Add("$($t0.AddHours(5).ToString('yyyy-MM-ddTHH:mm:ss.ffffffZ')) ERROR pecofence_platform::crashlog: CRASH (unhandled exception) code=0xc0000005 at=pecofence.exe+0x1f368 module_owner=pecofence access=write stack=  pecofence.exe+0x1f368")
[IO.File]::WriteAllLines((ShareLog), $lines, (New-Object Text.UTF8Encoding($false)))
PfStart @('sleep 6000', 'feedback-info', 'sleep 3600000')
$fi = FeedbackInfo 30
$log = "$($fi.log)"
$bytes = [Text.Encoding]::UTF8.GetByteCount($log)
Log ("log excerpt after a crash: $bytes bytes; markers: " + (($log -split "`n" | Where-Object { $_ -match '^---' }) -join ' / '))
Check 'after a crashed run the report carries over 24 KB of log, at most 50 KB' ($fi.crashed -and $bytes -gt 26 * 1024 -and $bytes -le 50 * 1024 + 256) "crashed=$($fi.crashed) $bytes bytes"

Case '43 hang watch: the UI thread stops answering for 7 s'
PfFresh
$ctl = [G]::FindWindowW('PecoFence.Control', [NullString]::Value)
$tid = [Th]::UiThread($ctl)
$froze = [Th]::Freeze($tid, 7000)
Start-Sleep -Seconds 2
$nr = PfWaitLog 'UI thread not responding' 3
$stack = PfWaitLog 'UI thread stack' 2
$again = PfWaitLog 'UI thread responding again' 3
Check 'the log says the UI thread stopped answering, with the command running' ($froze -and $nr -match 'command=') "froze=$froze tid=$tid | $nr"
Check 'and where it was stuck' ($stack -match '\+0x') "$(if ($stack) { $stack.Substring(0, [math]::Min(200, $stack.Length)) })"
Check 'and how long it took to answer again' ($again -match 'ms=\d+') "$again"
# Ended while it does not answer (Windows' End task, Task Manager): the next 反馈 sends that run.
$h = [Th]::Suspend($tid)
Start-Sleep -Seconds 1
Get-Process pecofence -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
PfStart @('sleep 5000', 'feedback-info', 'sleep 3600000')
$fi = FeedbackInfo 30
$log = "$($fi.log)"
Check 'after that, the 反馈 form sends the run that did not end normally too' ($log -match '--- previous run \(did not end normally\) ---') (($log -split "`n" | Where-Object { $_ -match '^---' }) -join ' / ')

Case '54 55 after a crash PecoFence starts again by itself'
PfQuit
# A normal start (no test script: the watchdog does not restart test runs), the instance's own data.
$env:PECOFENCE_INSTANCE = $PfInst
Start-Process "$PfApp\pecofence.exe" -ArgumentList '--portable' -WorkingDirectory $PfApp
$h = WaitWin 'PecoFence.Control' 20 -Hidden
Start-Sleep -Seconds 4
$p1 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
$wd = @(Get-Process pecofence-watchdog -ErrorAction SilentlyContinue).Count
$hidden1 = -not [W]::IconsVisible()
Check 'a normal start hides the Windows icons and starts the watchdog' ($p1 -and $hidden1 -and $wd -ge 1) "pid=$($p1.Id) watchdogs=$wd icons hidden=$hidden1 fences=$(VisFences)"
# A crash within the first minute is not restarted (it would crash again): wait past it.
Start-Sleep -Seconds 58
$ended = [Th]::Terminate($p1.Id, [uint32]3221225477)
# The watchdog shows the Windows icons, then starts PecoFence again (which hides them again).
$shownAt = -1; $p2 = $null; $t = [Diagnostics.Stopwatch]::StartNew()
while ($t.ElapsedMilliseconds -lt 15000 -and -not $p2) {
  if ($shownAt -lt 0 -and [W]::IconsVisible()) { $shownAt = $t.ElapsedMilliseconds }
  $p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Where-Object { $_.Id -ne $p1.Id } | Select-Object -First 1
  Start-Sleep -Milliseconds 100
}
$startedAt = if ($p2) { $t.ElapsedMilliseconds } else { -1 }
Start-Sleep -Seconds 5
$fences = VisFences
$items = @(PfJson item list).Count
$restartLine = (Get-Content (PfLogPath) -TotalCount 40 -Encoding UTF8 | Where-Object { $_ -match 'restarted_after_crash' }) | Select-Object -First 1
Shot '54-restarted'
Check 'after the crash (exit code 0xC0000005, run over a minute) the watchdog shows the Windows icons' ($ended -eq 1 -and $shownAt -ge 0) "ended=$ended icons shown after $shownAt ms"
Check 'and starts PecoFence again, which shows its fences and icons and hides the Windows icons again' ($p2 -and $fences -ge 1 -and $items -ge 1 -and -not [W]::IconsVisible() -and $restartLine -match 'restarted_after_crash=0xC0000005') "new pid=$($p2.Id) after $startedAt ms, fences=$fences items=$items | $(if ($restartLine) { $restartLine.Substring([math]::Max(0, $restartLine.IndexOf('restarted_after_crash'))) })"
# Ended from Task Manager (exit code 1) after a minute: not started again.
Start-Sleep -Seconds 52
$p2 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
$ended = if ($p2) { [Th]::Terminate($p2.Id, 1) } else { -1 }
Start-Sleep -Seconds 6
$p3 = Get-Process pecofence -ErrorAction SilentlyContinue | Select-Object -First 1
Check 'ended from Task Manager it stays closed, the icons shown' ($ended -eq 1 -and -not $p3 -and [W]::IconsVisible()) "ended=$ended running=$([bool]$p3) icons visible=$([W]::IconsVisible())"
Get-Process pecofence -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
if (-not [W]::IconsVisible()) { [W]::ToggleIcons() }
}

function H2PartB {
trap { PfTrap $_; continue }

Case 'B setup'
[Th]::SetSingleClick($false)
$made = @()
foreach ($n in 'h2-sel-a.txt', 'h2-sel-b.txt', 'h2-d1.txt', 'h2-d2.txt', 'h2-one.txt', 'H2README') { $made += DeskFile $n }
New-Item -ItemType Directory -Force "$PfDesk\h2same" | Out-Null; $made += "$PfDesk\h2same"
$made += DeskLink 'h2same.lnk'
$made += DeskLink 'h2-out.lnk'
Set-Content "$PfDesk\H2README.bat" "@echo ran>>$PfRun\h2bat.txt" -Encoding ASCII; $made += "$PfDesk\H2README.bat"
Remove-Item "$PfRun\h2bat.txt" -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$PfRun\h2portal\sub" | Out-Null
Set-Content "$PfRun\h2portal\top.txt" 'top'; Set-Content "$PfRun\h2portal\sub\inner.txt" 'inner'
PfFresh (DumpScript)
Pf fence create --title Sel1 --rect '300,100,320,230' | Out-Null
Pf fence create --title Sel2 --rect '660,100,320,230' | Out-Null
Pf fence create --title Same --rect '300,360,340,230' | Out-Null
Pf fence create --title Progs2 --rect '680,360,300,230' | Out-Null
Start-Sleep -Seconds 2
Pf item move "$PfDesk\h2-sel-a.txt" "$PfDesk\h2-d1.txt" "$PfDesk\h2-d2.txt" --to Sel1 | Out-Null
Pf item move "$PfDesk\h2-sel-b.txt" "$PfDesk\h2-one.txt" --to Sel2 | Out-Null
Pf item move "$PfDesk\h2same" "$PfDesk\h2same.lnk" "$PfDesk\H2README" "$PfDesk\H2README.bat" --to Same | Out-Null
Pf item move "$PfDesk\h2-out.lnk" --to Progs2 | Out-Null
Start-Sleep -Seconds 2
$where = @('h2-sel-a.txt', 'h2-d1.txt', 'h2-sel-b.txt', 'h2-one.txt', 'h2same', 'h2same.lnk', 'H2README', 'H2README.bat', 'h2-out.lnk' | ForEach-Object { "$_=$(ItemFence $_)" })
Check 'the test icons are in their fences' (($where -join ' ') -match 'h2-sel-a.txt=Sel1' -and ($where -join ' ') -match 'H2README.bat=Same' -and ($where -join ' ') -match 'h2-out.lnk=Progs2') ($where -join ', ')
$empty = @(700, 780)

Case '1 a double-click on the empty desktop still hides and shows the fences'
# The desktop comes to the front from another window, as after any other program (0.2.3 follows
# desktop clicks only after that).
Start-Process notepad.exe; Start-Sleep -Seconds 2
[G]::Click($empty[0], $empty[1]); Start-Sleep -Milliseconds 600
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 900
$n0 = VisFences
DblClick $empty[0] $empty[1]; Start-Sleep -Milliseconds 1200
$n1 = VisFences
DblClick $empty[0] $empty[1]; Start-Sleep -Milliseconds 1200
$n2 = VisFences
Check 'hides them, and a second double-click shows them' ($n0 -ge 5 -and $n1 -eq 0 -and $n2 -eq $n0) "visible fences $n0 -> $n1 -> $n2"
if ($n2 -ne $n0) { Pf fence show-all | Out-Null; Start-Sleep -Seconds 1 }

Case '1 刷新 (Refresh) in the desktop menu, double-clicked, leaves the fences shown'
# The menu opens down and right from the pointer: from here its Refresh lies over the bare
# desktop (run 13 opened it over a fence, so the second click hit that fence).
$menuAt = @(1010, 130)
$n0 = VisFences
# Where Refresh shows, with the menu closed again: the bare desktop.
[G]::RClick($menuAt[0], $menuAt[1])
$p = ShellMenuPoint 'Refresh' 5
Shot '1-menu'
[G]::Press(0x1B); Start-Sleep -Milliseconds 600
$bare = if ($p) { [W]::RootClassAt($p[0], $p[1]) } else { '' }
[G]::RClick($menuAt[0], $menuAt[1])
$p = ShellMenuPoint 'Refresh' 5
if ($p) { DblClick $p[0] $p[1] } else { CloseMenus }
Start-Sleep -Milliseconds 1500
$n1 = VisFences
Check 'the desktop menu has Refresh, over the bare desktop' ($null -ne $p -and $bare -match '^(Progman|WorkerW)$') "at $p; there without the menu: $bare"
Check 'double-clicking Refresh hides no fence' ($p -and $n1 -eq $n0) "visible fences $n0 -> $n1"
if ($n1 -ne $n0) { Pf fence show-all | Out-Null; Start-Sleep -Seconds 1 }
# Refresh, then one more click there at once (a slower double-click), three times.
$hid = @()
foreach ($k in 1..3) {
  $n0 = VisFences
  [G]::RClick($menuAt[0], $menuAt[1])
  $p = ShellMenuPoint 'Refresh' 5
  if ($p) { [G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 150; [G]::Click($p[0], $p[1]) } else { CloseMenus }
  Start-Sleep -Milliseconds 1500
  $n1 = VisFences
  $hid += "$n0->$n1"
  if ($n1 -ne $n0) { Pf fence show-all | Out-Null; Start-Sleep -Seconds 1 }
}
Check 'Refresh, then another click there at once, hides no fence (3 tries)' (@($hid | Where-Object { $_ -match '->0$' }).Count -eq 0) "visible fences per try: $($hid -join ', ')"

Case '1 one click on the empty desktop right after using a fence leaves them shown'
$hid = @()
$s1 = FenceGet Sel1
foreach ($k in 1..3) {
  $n0 = VisFences
  [G]::Click($s1.windowRect.x + 160, $s1.windowRect.y + 200); Start-Sleep -Milliseconds 700
  [G]::Click($empty[0], $empty[1]); Start-Sleep -Milliseconds 1200
  $n1 = VisFences
  $hid += "$n0->$n1"
  if ($n1 -ne $n0) { Pf fence show-all | Out-Null; Start-Sleep -Seconds 1 }
}
Check 'a click in a fence, then one click on the desktop, hides no fence (3 tries)' (@($hid | Where-Object { $_ -match '->0$' }).Count -eq 0) "visible fences per try: $($hid -join ', ')"

Case '34 a click elsewhere takes the selection off the icons of the other fences'
$pa = ItemPt Sel1 'h2-sel-a'; $pb = ItemPt Sel2 'h2-sel-b'
[G]::Click($pa[0], $pa[1]); Start-Sleep -Milliseconds 600
$a1 = DumpVal (DumpOf Sel1) 'selected'
[G]::Click($pb[0], $pb[1]); Start-Sleep -Milliseconds 600
$a2 = DumpVal (DumpOf Sel1) 'selected'; $b2 = DumpVal (DumpOf Sel2) 'selected'
Check 'a click on an icon of another fence clears the first fence' ($a1 -eq '1' -and $a2 -eq '0' -and $b2 -eq '1') "Sel1 selected $a1 -> $a2, Sel2 $b2"
[G]::Click($empty[0], $empty[1]); Start-Sleep -Milliseconds 700
$b3 = DumpVal (DumpOf Sel2) 'selected'
Check 'a click on the desktop clears it too' ($b3 -eq '0') "Sel2 selected $b3"
Shot '34-after'
# Switching to another program keeps the selection (grey), as in Explorer.
[G]::Click($pa[0], $pa[1]); Start-Sleep -Milliseconds 600
Start-Process notepad.exe; Start-Sleep -Seconds 2
$a4 = DumpVal (DumpOf Sel1) 'selected'
Check 'switching to another program keeps it' ($a4 -eq '1') "Sel1 selected $a4, front $(Fg)"
Get-Process notepad -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 600

Case '48 Ctrl+D deletes the selected icon to the Recycle Bin'
$p = ItemPt Sel1 'h2-d1'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
Chord 0x11 0x44; Start-Sleep -Seconds 2
$gone = -not (Test-Path "$PfDesk\h2-d1.txt")
Check 'Ctrl+D sends it to the Recycle Bin' ($gone -and (BinNames) -match 'h2-d1') "on desktop=$(-not $gone) bin: $(BinNames)"
$p = ItemPt Sel1 'h2-d2'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
[G]::KeyDown(0x11); [G]::KeyDown(0x10); [G]::Press(0x44); [G]::KeyUp(0x10); [G]::KeyUp(0x11); Start-Sleep -Seconds 2
Check 'Ctrl+Shift+D does nothing' (Test-Path "$PfDesk\h2-d2.txt") "on desktop=$(Test-Path "$PfDesk\h2-d2.txt") bin: $(BinNames)"
CloseWindows

Case '37 a folder and a file without extension open themselves beside a same-name shortcut and .bat'
$opened = @()
foreach ($i in 0..3) {
  $p = CellPt Same $i
  DblClick $p[0] $p[1]; Start-Sleep -Seconds 3
  $ex = @((New-Object -ComObject Shell.Application).Windows() | ForEach-Object { try { $_.Document.Folder.Self.Path } catch { '?' } })
  $np = @(Get-Process notepad -ErrorAction SilentlyContinue).Count
  $ow = @(Get-Process OpenWith -ErrorAction SilentlyContinue).Count
  $bat = if (Test-Path "$PfRun\h2bat.txt") { @(Get-Content "$PfRun\h2bat.txt").Count } else { 0 }
  $opened += "cell $i -> explorer=[$($ex -join ',')] notepad=$np openwith=$ow bat=$bat"
  CloseWindows
  Get-Process OpenWith -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Milliseconds 500
}
Log ($opened -join ' ; ')
$all = $opened -join ' ; '
$folderOpened = @($opened | Where-Object { $_ -match 'explorer=\[[^\]]*h2same' }).Count
$notepads = @($opened | Where-Object { $_ -match 'notepad=1' }).Count
Check 'the folder opens in Explorer, the shortcut runs its program once' ($folderOpened -eq 1 -and $notepads -eq 1) $all
$batRuns = if (Test-Path "$PfRun\h2bat.txt") { @(Get-Content "$PfRun\h2bat.txt").Count } else { 0 }
Check 'the file without extension does not run the .bat beside it (the .bat runs once, by itself)' ($batRuns -eq 1) "bat ran $batRuns times; $all"

Case '35 an icon dragged out of a fence onto the empty desktop stays in Desktop'
$inbox = (@(PfJson fence list) | Where-Object { $_.kind -eq 'inbox' } | Select-Object -First 1).title
$p = ItemPt Progs2 'h2-out'
DragTo $p[0] $p[1] 560 700 20
Start-Sleep -Seconds 4
$now = ItemFence 'h2-out.lnk'
Check "a shortcut dragged onto the desktop stays in '$inbox' (the rules do not send it back)" ($now -eq $inbox) "h2-out.lnk in '$now'"
Shot '35-after'

Case '38 the push pass moves a locked fence below an expanding one'
# Left of this part's fences and clear of the first-run column (since U15 it fills x 1088 to 1584 at
# the top): a fence pushed under it could not clear Upper above the taskbar (no room, see
# push.rs a_push_the_work_area_stops_is_marked).
Pf fence create --title Upper --rect '20,420,260,250' | Out-Null
Pf fence create --title Lower --rect '20,500,260,150' | Out-Null
Start-Sleep -Seconds 1
Pf fence set Lower locked true | Out-Null
Pf fence roll Upper | Out-Null
Start-Sleep -Seconds 2
$lo0 = FenceGet Lower; $dl0 = DumpOf Lower
# Rest the pointer on the rolled fence: it expands (hover peek).
$u = FenceGet Upper
[G]::MoveTo($u.windowRect.x + 150, $u.windowRect.y + 18); Start-Sleep -Milliseconds 300
[G]::MoveTo($u.windowRect.x + 152, $u.windowRect.y + 18); Start-Sleep -Milliseconds 1800
$du = DumpOf Upper; $dl1 = DumpOf Lower
Shot '38-expanded'
Check 'the locked fence below moves out of the way while the upper one is expanded' ($du -and -not $du.rolled -and $dl1 -and $dl1.t -ge $du.b) "Upper window $($du.t)..$($du.b) rolled=$($du.rolled); Lower top $($dl0.t) -> $($dl1.t)"
# Off both fences: over Lower, which Upper pushed, the hover peek stays open.
[G]::MoveTo(700, 780); Start-Sleep -Seconds 2
$dl2 = DumpOf Lower; $lo2 = FenceGet Lower
Check 'and goes back to its place, its saved place unchanged' ($dl2 -and [math]::Abs($dl2.t - $dl0.t) -le 2 -and $lo2.rect.y -eq $lo0.rect.y) "Lower top $($dl2.t), saved y $($lo0.rect.y) -> $($lo2.rect.y)"
# Saved expanded, then a restart: the locked fence keeps its saved place.
Pf fence unroll Upper | Out-Null
Start-Sleep -Seconds 2
PfStart (DumpScript)
Start-Sleep -Seconds 2
$lo3 = FenceGet Lower; $dl3 = DumpOf Lower; $du3 = DumpOf Upper
Check 'after a restart with the upper fence saved expanded the locked one is pushed, its saved place kept' ($lo3.rect.y -eq $lo0.rect.y -and $dl3 -and $du3 -and $dl3.t -ge $du3.b) "Lower saved y $($lo0.rect.y) -> $($lo3.rect.y), window top $($dl3.t), Upper bottom $($du3.b)"
Pf fence delete Upper | Out-Null; Pf fence delete Lower | Out-Null

Case '51 a folder portal tab gets its up button in a subfolder'
Pf fence create --portal "$PfRun\h2portal" --rect '1000,420,330,240' | Out-Null
Pf fence create --title Host --rect '1000,120,330,240' | Out-Null
Start-Sleep -Seconds 2
Pf fence merge h2portal --into Host | Out-Null
Start-Sleep -Seconds 2
$d = DumpOf Host
$ti = [array]::IndexOf(@($d.tabs), 'h2portal')
Check 'the portal is the second tab of Host' ($ti -ge 0 -and @($d.tabs).Count -eq 2) "tabs=$($d.tabs -join ',') rects=$((@($d.rects) | ForEach-Object { "$($_[0])+$($_[1])" }) -join ' ')"
$tx = [int]($d.l + $d.ox + $d.rects[$ti][0]); $tw = [int]$d.rects[$ti][1]; $ty = [int]($d.t + $d.oy)
[G]::Click($tx + [int]($tw / 2), $ty + 18); Start-Sleep -Milliseconds 900
[G]::MoveTo(200, 780); Start-Sleep -Milliseconds 400
$g0 = Crop '51-tab-root' $tx $ty $tw 36
# A hosted tab has no window rect of its own: its items show in Host's window ('sub' before 'top').
$p = CellPt Host 0
DblClick $p[0] $p[1]; Start-Sleep -Seconds 2
$cur = (FenceGet h2portal).portal.current
Check 'a double-click on its subfolder goes into it' ("$cur" -match 'sub$') "current=$cur"
[G]::MoveTo(200, 780); Start-Sleep -Milliseconds 500
$g1 = Crop '51-tab-sub' $tx $ty $tw 36
$first = [Px]::FirstInk($g1, 0, $tw - 1, 8, 28, 60)
$diff = [Px]::Diff($g0, $g1, [math]::Max(0, $first), [math]::Max(0, $first + 14), 4, 32)
Check 'the tab shows an up arrow instead of its folder icon' ($first -ge 0 -and $diff -gt 8) "first ink at $first, change at the icon $([math]::Round($diff, 1))"
if ($first -ge 0) { [G]::Click($tx + $first + 8, $ty + 18); Start-Sleep -Seconds 2 }
$cur = (FenceGet h2portal).portal.current
Check 'a click on it goes one folder up' ($first -ge 0 -and -not $cur) "current=$cur clicked at $($tx + $first + 8),$($ty + 18)"
Pf fence delete Host | Out-Null; Pf fence delete h2portal | Out-Null

Case '28 after Win+D, Settings and Properties open on top of the fences'
CloseWindows
Chord 0x5B 0x44; Start-Sleep -Seconds 2
Pf settings open-ui | Out-Null
$hs = WaitWin 'PecoFence.Settings' 25
Start-Sleep -Seconds 3
$s1 = FenceGet Sel1
[L]::MoveWindow($hs, $s1.windowRect.x - 100, $s1.windowRect.y - 40, 900, 600, $true) | Out-Null
Start-Sleep -Milliseconds 800
$at = [W]::RootClassAt($s1.windowRect.x + 160, $s1.windowRect.y + 120)
Shot '28-settings'
Check 'Settings shows above a fence' ($hs -ne [IntPtr]::Zero -and $at -eq 'PecoFence.Settings') "at the fence's middle: $at"
if ($hs -ne [IntPtr]::Zero) { [L]::PostMessageW($hs, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
Start-Sleep -Seconds 1
Chord 0x5B 0x44; Start-Sleep -Seconds 2
$m = RClickItem Sel2 'h2-sel-b'
$picked = MenuPick '^Properties'
$pr = $null
for ($k = 0; $k -lt 20 -and -not $pr; $k++) { $pr = ProcTops '#32770' 'pecofence' | Select-Object -First 1; Start-Sleep -Milliseconds 300 }
Start-Sleep -Milliseconds 800
$s2 = FenceGet Sel2
if ($pr) { [L]::MoveWindow($pr, $s2.windowRect.x + 40, $s2.windowRect.y + 20, 380, 480, $true) | Out-Null }
Start-Sleep -Milliseconds 800
$at = [W]::RootClassAt($s2.windowRect.x + 160, $s2.windowRect.y + 120)
Shot '28-properties'
Check 'the Properties window of an icon shows above the fences' ($pr -and $at -eq '#32770') "picked '$picked'; at the fence's middle: $at; $(if ($pr) { [W]::Info($pr) })"
if ($pr) { [L]::PostMessageW($pr, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
Start-Sleep -Milliseconds 600
[G]::Click($empty[0], $empty[1]); Start-Sleep -Milliseconds 600

Case '50 single-click open, when Explorer is set to it'
[Th]::SetSingleClick($true)
Start-Sleep -Seconds 2
$seen = PfWaitLog 'click to open setting changed' 4
$p = ItemPt Sel2 'h2-one'
[G]::KeyDown(0x11); [G]::Click($p[0], $p[1]); [G]::KeyUp(0x11); Start-Sleep -Seconds 2
$np0 = NotepadTitles
[G]::Click($empty[0], $empty[1]); Start-Sleep -Milliseconds 600
[G]::Click($p[0], $p[1]); Start-Sleep -Seconds 3
$np1 = NotepadTitles
Check 'PecoFence follows the changed setting' ([bool]$seen) "$seen"
Check 'Ctrl+click only selects' ($np0 -notmatch 'h2-one') "notepad after Ctrl+click: '$np0'"
Check 'a click opens the icon' ($np1 -match 'h2-one') "notepad after a click: '$np1'"
[Th]::SetSingleClick($false)
CloseWindows

Case 'B cleanup'
PfQuit
foreach ($p in $made) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue }
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
}

function H2PartC {
trap { PfTrap $_; continue }

Case 'C setup'
Add-Type -AssemblyName Microsoft.VisualBasic
$made = @()
foreach ($n in 'h2-bin-back.txt', 'h2-empty.txt', 'h2-peek.txt', 'h2-peek2.txt') { $made += DeskFile $n }
New-Item -ItemType Directory -Force "$PfDesk\h2-full-dir" | Out-Null; Set-Content "$PfDesk\h2-full-dir\a.txt" 'a'; $made += "$PfDesk\h2-full-dir"
$made += DeskLink 'h2-own.lnk'
$pub = @(Get-ChildItem 'C:\Users\Public\Desktop' -Filter *.lnk -ErrorAction SilentlyContinue)
Log ("Public Desktop: " + (($pub | ForEach-Object { $_.Name }) -join ', '))
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$PfRun\h2drop" | Out-Null
foreach ($n in 'red', 'blue') {
  $b = New-Object Drawing.Bitmap 200, 200; $g = [Drawing.Graphics]::FromImage($b)
  $g.Clear($(if ($n -eq 'red') { [Drawing.Color]::FromArgb(220, 30, 30) } else { [Drawing.Color]::FromArgb(30, 60, 220) })); $g.Dispose()
  $b.Save("$PfDesk\h2-$n.jfif", [Drawing.Imaging.ImageFormat]::Jpeg); $b.Dispose(); $made += "$PfDesk\h2-$n.jfif"
}
foreach ($f in 'arial.ttf', 'wingding.ttf') { Copy-Item "C:\Windows\Fonts\$f" "$PfDesk\h2-$f" -ErrorAction SilentlyContinue; $made += "$PfDesk\h2-$f" }
$curs = @(Get-ChildItem 'C:\Windows\Cursors' -Filter *.ani -ErrorAction SilentlyContinue | Select-Object -First 2)
foreach ($c in $curs) { Copy-Item $c.FullName "$PfDesk\h2-$($c.Name)"; $made += "$PfDesk\h2-$($c.Name)" }
# The same files in a folder, for Explorer's own view of them.
New-Item -ItemType Directory -Force "$PfRun\h2thumbs" | Out-Null
Copy-Item "$PfDesk\h2-red.jfif", "$PfDesk\h2-blue.jfif", "$PfDesk\h2-arial.ttf", "$PfDesk\h2-wingding.ttf" "$PfRun\h2thumbs\"
# File Explorer Options > 始终显示图标，从不显示缩略图: a Windows Server account starts with it on.
$adv = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced'
$IconsOnlyWas = (Get-ItemProperty $adv -ErrorAction SilentlyContinue).IconsOnly
Log "IconsOnly was '$IconsOnlyWas'; .jfif: $((cmd /c assoc .jfif 2>&1) -join ' ')"
Set-ItemProperty $adv -Name IconsOnly -Value 0 -Type DWord
[Th]::SetSingleClick($false)
# A plain grey wallpaper: the glass behind two icons is then the same, and only the icons differ.
GreyWallpaper
PfFresh (DumpScript)
Pf fence create --title Bin --rect '700,100,300,220' | Out-Null
Pf fence create --title Mixed --rect '700,360,300,220' | Out-Null
Pf fence create --title Pics --rect '300,100,240,170' | Out-Null
Pf fence create --title Fonts --rect '300,300,240,170' | Out-Null
Pf fence create --title Curs --rect '300,500,240,170' | Out-Null
Pf fence create --title Peeky --rect '1040,560,300,200' | Out-Null
Start-Sleep -Seconds 2
Pf item move 'Recycle Bin' "$PfDesk\h2-full-dir" --to Bin | Out-Null
Pf item move "$PfDesk\h2-own.lnk" --to Mixed | Out-Null
if ($pub.Count) { Pf item move $pub[0].FullName --to Mixed | Out-Null }
Pf item move "$PfDesk\h2-red.jfif" "$PfDesk\h2-blue.jfif" --to Pics | Out-Null
Pf item move "$PfDesk\h2-arial.ttf" "$PfDesk\h2-wingding.ttf" --to Fonts | Out-Null
if ($curs.Count -eq 2) { Pf item move "$PfDesk\h2-$($curs[0].Name)" "$PfDesk\h2-$($curs[1].Name)" --to Curs | Out-Null }
Pf item move "$PfDesk\h2-peek.txt" "$PfDesk\h2-peek2.txt" --to Peeky | Out-Null
Start-Sleep -Seconds 3
Log ("items: " + ((@(PfJson item list) | ForEach-Object { "$($_.name)@$($_.fenceTitle)" }) -join ', '))
$empty = @(700, 780)

Case '45 files show their own thumbnail or icon, as in Explorer'
[G]::MoveTo(200, 780); Start-Sleep -Seconds 4
$red = IconCrop Pics 1 '45-red'; $blue = IconCrop Pics 0 '45-blue'
$cr = MeanRgb $red; $cb = MeanRgb $blue
$f0 = IconCrop Fonts 0 '45-font0'; $f1 = IconCrop Fonts 1 '45-font1'
$df = [Px]::Diff($f0, $f1, 0, 47, 0, 47)
$ex = ExplorerIconRgb "$PfRun\h2thumbs" @('h2-red', 'h2-blue')
$er = $ex['h2-red']; $eb = $ex['h2-blue']
Shot '45-explorer'
CloseWindows
Check 'Explorer shows the .jfif thumbnails here (the reference)' ($er -and $eb -and $er[0] -gt $er[2] + 60 -and $eb[2] -gt $eb[0] + 60) "Explorer: red file rgb $($er -join ' '), blue file rgb $($eb -join ' ')"
Check '.jfif pictures show their own picture in the fence' ($cr[0] -gt $cr[2] + 60 -and $cb[2] -gt $cb[0] + 60) "fence: red file rgb $($cr -join ' '), blue file rgb $($cb -join ' ')"
Check 'two fonts show two different previews' ($df -gt 6) "difference $([math]::Round($df, 1))"
$c0 = IconCrop Curs 0 '45-cur0'; $c1 = IconCrop Curs 1 '45-cur1'
$dc = [Px]::Diff($c0, $c1, 0, 47, 0, 47)
Check 'two animated cursors show their own icons' ($curs.Count -eq 2 -and $dc -gt 3) "$(($curs | ForEach-Object { $_.Name }) -join ', '): difference $([math]::Round($dc, 1))"

Case '36 a file dragged from the Recycle Bin window into a fence gets its name back'
[Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile("$PfDesk\h2-bin-back.txt", 'OnlyErrorDialogs', 'SendToRecycleBin')
Start-Process explorer.exe 'shell:RecycleBinFolder'
$rb = $null
for ($k = 0; $k -lt 30 -and -not $rb; $k++) { Start-Sleep -Milliseconds 300; $rb = @([W]::Tops('CabinetWClass')) | Select-Object -First 1 }
[L]::MoveWindow($rb, 20, 380, 620, 420, $true) | Out-Null
Start-Sleep -Seconds 2
$byName = New-Object Windows.Automation.PropertyCondition([Windows.Automation.AutomationElement]::ControlTypeProperty, [Windows.Automation.ControlType]::ListItem)
$rows = @([Windows.Automation.AutomationElement]::FromHandle($rb).FindAll([Windows.Automation.TreeScope]::Descendants, $byName))
$row = $rows | Where-Object { $_.Current.Name -like 'h2-bin-back*' } | Select-Object -First 1
$b = FenceGet Bin
if ($row) { $r = $row.Current.BoundingRectangle; DragTo ([int]($r.X + 50)) ([int]($r.Y + $r.Height / 2)) ($b.windowRect.x + 200) ($b.windowRect.y + 160) 20 }
Start-Sleep -Seconds 3
$back = Test-Path "$PfDesk\h2-bin-back.txt"
$odd = @(Get-ChildItem $PfDesk -Filter '$R*' -ErrorAction SilentlyContinue | ForEach-Object { $_.Name })
Check 'the row is in the Recycle Bin window' ($null -ne $row) "rows: $(($rows | ForEach-Object { $_.Current.Name }) -join ', ')"
Check 'the file is back on the desktop under its own name, in the fence' ($back -and $odd.Count -eq 0 -and (ItemFence 'h2-bin-back.txt') -eq 'Bin') "own name=$back, `$R names: $($odd -join ', '), fence '$(ItemFence 'h2-bin-back.txt')'"
foreach ($o in $odd) { Remove-Item -LiteralPath "$PfDesk\$o" -Force -ErrorAction SilentlyContinue }
CloseWindows

Case '41 清空回收站 (Empty Recycle Bin) after deleting in another program'
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
[Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile("$PfDesk\h2-empty.txt", 'OnlyErrorDialogs', 'SendToRecycleBin')
Start-Sleep -Seconds 2
$p = ItemAt Bin 'Recycle Bin'
[G]::RClick($p[0], $p[1])
$m = MenuWin 8
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
$picked = MenuPick '^Empty Recycle Bin'
$yes = $null
for ($k = 0; $k -lt 12 -and -not $yes; $k++) {
  Start-Sleep -Milliseconds 400
  foreach ($w in @([W]::Tops('#32770'))) { $t = ClickButton $w '^Yes$'; if ($t) { $yes = "$([W]::ProcOf($w)) '$([W]::Title($w))'"; break } }
}
Start-Sleep -Seconds 3
Check 'the Recycle Bin menu has Empty Recycle Bin, not greyed' ($texts -match 'Empty Recycle Bin' -and $texts -notmatch 'Empty Recycle Bin \(off\)') $texts
Check 'and it empties the bin' ($picked -and -not (BinNames)) "picked '$picked', confirm $yes, bin now: '$(BinNames)'"

Case '39 Delete, Ctrl+X, Ctrl+C on own icons plus Public Desktop icons'
$pubName = if ($pub.Count) { [IO.Path]::GetFileNameWithoutExtension($pub[0].Name) } else { '' }
$po = ItemPt Mixed 'h2-own'; $pp = ItemPt Mixed $pubName
[G]::Click($po[0], $po[1]); Start-Sleep -Milliseconds 400
[G]::KeyDown(0x11); [G]::Click($pp[0], $pp[1]); [G]::KeyUp(0x11); Start-Sleep -Milliseconds 500
$sel = DumpVal (DumpOf Mixed) 'selected'
[G]::RClick($pp[0], $pp[1])
$m = MenuWin 6
$texts = if ($m -ne [IntPtr]::Zero) { MenuTexts $m } else { '' }
CloseMenus
Check "two icons selected (mine and '$pubName' of the Public Desktop)" ($pub.Count -gt 0 -and $sel -eq '2') "selected=$sel"
Check "the icon menu has Explorer's commands for the pair" ($texts -match 'Create shortcut|Send to|Copy as path') $texts
[Windows.Forms.Clipboard]::Clear()
Chord 0x11 0x43; Start-Sleep -Seconds 1
$c = ClipFiles; $e = ClipEffect
Check 'Ctrl+C copies both' ($c.Count -eq 2) "clipboard: $($c -join ', ') effect=$e"
[Windows.Forms.Clipboard]::Clear()
Chord 0x11 0x58; Start-Sleep -Seconds 1
$c = ClipFiles; $e = ClipEffect
Check 'Ctrl+X cuts both' ($c.Count -eq 2 -and $e -eq 2) "clipboard: $($c -join ', ') effect=$e"
[G]::Press(0x2E); Start-Sleep -Seconds 3
$dlg = @([W]::Tops('#32770') + [W]::Tops('OperationStatusWindow')) | Where-Object { $_ } | Select-Object -First 1
$dlgText = if ($dlg) { "$([W]::ProcOf($dlg)) '$([W]::Title($dlg))' $(DlgText $dlg)" } else { 'none' }
Shot '39-delete'
Check 'Delete sends my icon to the Recycle Bin' (-not (Test-Path "$PfDesk\h2-own.lnk") -and (BinNames) -match 'h2-own') "own on desktop=$(Test-Path "$PfDesk\h2-own.lnk"), bin: $(BinNames)"
Check 'and asks for administrator permission for the Public Desktop one (pf is a standard user)' ($dlg -and $pub.Count -and (Test-Path $pub[0].FullName)) "dialog: $dlgText; public still there=$(if ($pub.Count) { Test-Path $pub[0].FullName })"
if ($dlg) { $t = ClickButton $dlg '^(Cancel|Skip)$'; Start-Sleep -Seconds 1; if (-not $t) { [G]::Press(0x1B) } }
Start-Sleep -Seconds 1
CloseWindows

Case '10 while a file is dragged out of a fence during Peek the dimming fades away'
Start-Process explorer.exe "$PfRun\h2drop"
$fw = $null
for ($k = 0; $k -lt 30 -and -not $fw; $k++) { Start-Sleep -Milliseconds 300; $fw = @([W]::Tops('CabinetWClass')) | Select-Object -First 1 }
# Below Mixed and right of Curs, left of Peeky: no fence covers it during Peek.
[L]::MoveWindow($fw, 560, 600, 470, 245, $true) | Out-Null
Start-Sleep -Seconds 2
$drop = @(800, 790)
[G]::Click(800, 615); Start-Sleep -Milliseconds 500
PeekKey; Start-Sleep -Seconds 1
$dim = @([W]::Tops('PecoFence.PeekDimmer')) | Select-Object -First 1
$a0 = if ($dim) { [Th]::Alpha($dim) } else { -2 }
$p = ItemAt Peeky 'h2-peek'
[G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 200
MDown; Start-Sleep -Milliseconds 150
Glide $p[0] $p[1] $drop[0] $drop[1] 20 25
Start-Sleep -Milliseconds 700
$a1 = if ($dim) { [Th]::Alpha($dim) } else { -2 }
Shot '10-dragging'
MUp; Start-Sleep -Seconds 2
$dimNow = @([W]::Tops('PecoFence.PeekDimmer')).Count
$dropped = Test-Path "$PfRun\h2drop\h2-peek.txt"
Check 'Peek dims the desktop' ($dim -and $a0 -ge 60) "dimmer alpha $a0"
Check 'the dimming fades away while the file is dragged' ($a1 -ge 0 -and $a1 -le 10) "alpha $a0 -> $a1 while dragging"
Check 'dropped on the folder window, the file lands there and the Peek ends' ($dropped -and $dimNow -eq 0) "in folder=$dropped dimmers left=$dimNow"
# Esc while dragging: the dimming comes back and the Peek goes on.
if ($dimNow) { Pf peek end | Out-Null; Start-Sleep -Seconds 1 }
[G]::Click(800, 615); Start-Sleep -Milliseconds 500
PeekKey; Start-Sleep -Seconds 1
$dim = @([W]::Tops('PecoFence.PeekDimmer')) | Select-Object -First 1
$p = ItemAt Peeky 'h2-peek2'
[G]::MoveTo($p[0], $p[1]); Start-Sleep -Milliseconds 200
MDown; Start-Sleep -Milliseconds 150
Glide $p[0] $p[1] $drop[0] $drop[1] 20 25
Start-Sleep -Milliseconds 700
$a1 = if ($dim) { [Th]::Alpha($dim) } else { -2 }
[G]::Press(0x1B); Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Seconds 1
$a2 = if ($dim) { [Th]::Alpha($dim) } else { -2 }
$still = @([W]::Tops('PecoFence.PeekDimmer')).Count
Check 'Esc during the drag brings the dimming back and the Peek goes on' ($a1 -ge 0 -and $a1 -le 10 -and $a2 -ge 60 -and $still -ge 1 -and (Test-Path "$PfDesk\h2-peek2.txt")) "alpha while dragging $a1, after Esc $a2, dimmers $still"
Pf peek end | Out-Null
CloseWindows

Case '40 deleting with a full Recycle Bin does not freeze the fences'
$dir = RecycleDir
$t = [Diagnostics.Stopwatch]::StartNew()
$n = [Th]::FillBin($dir, $PfDesk, 10000)
Log "filled the Recycle Bin with $n items in $($t.ElapsedMilliseconds) ms"
$cnt = (New-Object -ComObject Shell.Application).NameSpace(10).Items().Count
# A folder: deleting one asked the bin three times in a row.
$p = ItemAt Bin 'h2-full-dir'
[G]::Click($p[0], $p[1]); Start-Sleep -Milliseconds 500
$fw = [W]::Tops('PecoFence.Fence') | Where-Object { $r = Rect $_; $r.L -le $p[0] -and $r.R -ge $p[0] -and $r.T -le $p[1] -and $r.B -ge $p[1] } | Select-Object -First 1
[G]::Press(0x2E)
$max = if ($fw) { [Th]::MaxPing($fw, 8000) } else { -1 }
Shot '40-after'
$gone = -not (Test-Path "$PfDesk\h2-full-dir")
Check 'the bin holds thousands of items' ($cnt -ge 10000) "items in the bin: $cnt"
Check 'Delete with a full bin: the fence answers within 300 ms throughout' ($gone -and $max -ge 0 -and $max -lt 300) "deleted=$gone, longest wait for an answer $max ms"
$t = [Diagnostics.Stopwatch]::StartNew()
$n = [Th]::EmptyFill($dir)
Log "removed $n filler files in $($t.ElapsedMilliseconds) ms"

Case 'C cleanup'
PfQuit
foreach ($p in $made) { Remove-Item -LiteralPath $p -Recurse -Force -ErrorAction SilentlyContinue }
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
if ($null -ne $IconsOnlyWas) { Set-ItemProperty $adv -Name IconsOnly -Value $IconsOnlyWas -Type DWord } else { Remove-ItemProperty $adv -Name IconsOnly -ErrorAction SilentlyContinue }
}

# ---- steps

Case 'setup'
CloseWindows
Clear-RecycleBin -Force -ErrorAction SilentlyContinue
Log "parts $H2Part, bin $PfBin, debug build $PfDebugBuild"
if (InPart 'A') { H2PartA }
if (InPart 'B') { H2PartB }
if (InPart 'C') { H2PartC }
Case 'log check'
$errs = LogErrorsCs
Check 'app log has no ERROR or panic line (case-sensitive)' ($errs.Count -eq 0) (($errs | Select-Object -First 3) -join ' | ')
