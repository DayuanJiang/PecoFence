# Fence polish round 2026-11-12: PecoFence quits while the name box of a fence is open (a panic
# at exit before: the box was destroyed while its thread-local was torn down).
trap { PfTrap $_; continue }
Case 'quit with a fence name box open'
CloseWindows
PfStart -Lang 'zh-CN'
Start-Sleep -Seconds 3
ClearPfNotices
$r = FreeRect 320 240 1040 420
[G]::MoveTo($r[0], $r[1]); Start-Sleep -Milliseconds 200; MDown; Start-Sleep -Milliseconds 200
Glide $r[0] $r[1] $r[2] $r[3] 14 30
Start-Sleep -Milliseconds 300; MUp; Start-Sleep -Milliseconds 700
$or = Rect ([G]::FindWindowW('PecoFence.MarqueeOffer', [NullString]::Value))
[G]::Click($or.R - 40, [int](($or.T + $or.B) / 2))
$box = WaitWin 'PecoFence.Rename' 6
Check 'the name box is open' ($box -ne [IntPtr]::Zero) "box=$box fg=$(Fg)"
PfQuit
Start-Sleep -Seconds 2
$p = @(PfLogAll | Where-Object { $_ -match 'PANIC' })
Check 'no panic when PecoFence quits with the name box open' ($p.Count -eq 0) "panic lines: $($p.Count); $(if ($p.Count) { $p[0].Substring(0, [math]::Min(300, $p[0].Length)) })"
foreach ($f in @(Get-ChildItem "$env:LOCALAPPDATA\CrashDumps" -Filter 'pecofence*' -ErrorAction SilentlyContinue)) { Log "crash dump: $($f.Name) $($f.LastWriteTime)" }
