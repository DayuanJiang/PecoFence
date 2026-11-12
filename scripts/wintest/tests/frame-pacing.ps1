# Fence fix round 2026-11-07, area render-perf: frame pacing and memory on a PC without a GPU
# (the EC2 boxes and the Hyper-V guests draw with WARP). Every check states the fixed behaviour;
# on 0.3.1 (run.sh --bin <0.3.1 build>) a FAIL is the finding showing. About 5 minutes.
#   P3  a missed compositor tick costs one frame, not two (frame clock timeout)
#   P4  a push animation does not re-arrange the Desktop fence (desktop layout) every frame
#   P6  a Liquid Glass title drag moves the window on every mouse move
#   P8  the end of a Peek redraws no opaque fence
#   S15 (memory) is in software-render-memory.ps1
$PfAllowLogErrors = $true
[Anim]::Set($true)

# "animation run" lines after each test-script command `$cmd` (frames, mean gap, max gap, UI work).
function RunsAfter([string[]]$lines, [string]$cmd) {
  $out = @(); $last = ''
  foreach ($l in $lines) {
    if ($l -match 'pecofence::test: >> (.*)$') { $c = $Matches[1].Trim(); if ($c -notlike 'sleep*' -and $c -notlike 'dump*') { $last = $c }; continue }
    if ($last -eq $cmd -and $l -match 'animation run( stuttered)? frames=(\d+) total_ms=([\d.]+) max_gap_ms=([\d.]+) max_frame_ms=([\d.]+)') {
      $f = [int]$Matches[2]
      if ($f -ge 3) { $out += [pscustomobject]@{ frames = $f; mean = [double]$Matches[3] / ($f - 1); gap = [double]$Matches[4]; work = [double]$Matches[5] } }
    }
  }
  return $out
}
function Med($v) { $s = @($v | Sort-Object); if ($s.Count -eq 0) { return -1 }; return $s[[int][math]::Floor($s.Count / 2)] }

Case 'P3 and P8: roll, unroll and Peek from test-script lines, ten fences on screen'
# A plain grey wallpaper: a Fluent fence in light mode that took light text from a dark part of
# the wallpaper shows dark text for the length of a Peek, and is redrawn when it ends (2026-11-12).
# On grey every fence keeps the mode's own text, so P8 still means: no fence is redrawn.
GreyWallpaper
# Ten fences on screen. The guests' compositor skipped beats only with the review's heavier layout
# (thumbnails, 2000-item folders; notes/fix/render-perf): here 0.3.1 also draws 60 frames a second,
# so the two frame checks guard against a slower clock, and timer_frames shows the new timeout.
$FxData = 'C:\pf\fxdata'
foreach ($d in 'pa', 'pb') { New-Item -ItemType Directory -Force "$FxData\$d" | Out-Null; for ($i = 1; $i -le 300; $i++) { $f = "$FxData\$d\file {0:D4}.txt" -f $i; if (-not (Test-Path $f)) { [IO.File]::WriteAllText($f, "fx $i") } } }
$S = @('sleep 12000', 'dump start')
for ($i = 0; $i -lt 6; $i++) { $S += @('roll RollMe', 'sleep 1200', 'unroll RollMe', 'sleep 1200') }
for ($i = 0; $i -lt 6; $i++) { $S += @('peek', 'sleep 1500', 'end-peek', 'sleep 1500') }
$S += @('dump end', 'sleep 3600000')
PfStart -Script $S
Pf fence create --title RollMe --rect '20,40,370,380' | Out-Null
Pf fence create --portal "$FxData\pa" --rect '410,40,370,380' | Out-Null
Pf fence create --portal "$FxData\pb" --rect '800,40,370,380' | Out-Null
for ($i = 0; $i -lt 4; $i++) { Pf fence create --title "Load$i" --rect "$(20 + 390 * $i),440,370,380" | Out-Null }
PfWaitLog '>> dump end' 150 | Out-Null
Start-Sleep -Seconds 1
$L = @(PfLogAll)
$roll = @(RunsAfter $L 'roll RollMe'); $unroll = @(RunsAfter $L 'unroll RollMe'); $endPeek = @(RunsAfter $L 'end-peek')
$d = "roll frames $(Med ($roll | ForEach-Object frames)), mean gap $([math]::Round((Med ($roll | ForEach-Object mean)), 1)) ms, max gap $(Med ($roll | ForEach-Object gap)) ms; unroll frames $(Med ($unroll | ForEach-Object frames)), mean gap $([math]::Round((Med ($unroll | ForEach-Object mean)), 1)) ms; runs $($roll.Count)/$($unroll.Count)"
Log "P3 $d"
Check 'P3: the 167 ms roll-up draws at least 10 frames (7 to 9 on 0.3.1 with the review layout)' ((Med ($roll | ForEach-Object frames)) -ge 10) $d
Check 'P3: the 333 ms expand has a mean frame gap under 19 ms (22 to 24 ms on 0.3.1 with the review layout)' ($unroll.Count -ge 5 -and (Med ($unroll | ForEach-Object mean)) -lt 19) $d
$tf = @($L | Where-Object { $_ -match 'animation run.*timer_frames=' }).Count
Check 'P3: the animation run lines count the frames the timeout started (timer_frames)' ($tf -gt 0) "lines with timer_frames: $tf"
$ended = @($L | Where-Object { $_ -match 'fences redrawn for the Peek floating=false redrawn=(\d+)' } | ForEach-Object { [int]($_ -replace '.*redrawn=(\d+).*', '$1') })
Check 'P8: ending the Peek redraws no opaque fence (it redrew all of them)' ($ended.Count -ge 6 -and @($ended | Where-Object { $_ -ne 0 }).Count -eq 0) "ends logged: $($ended.Count), redrawn: $($ended -join ',')"
Log ("P8 last Peek frame work ms: " + (($endPeek | ForEach-Object work) -join ', ') + " | " + ((@($L | Where-Object { $_ -match 'peek ended' }) | Select-Object -First 2 | ForEach-Object { $_ -replace '^.*peek ended', 'peek ended' }) -join ' | '))
PfQuit
Remove-Item -Recurse -Force $FxData -ErrorAction SilentlyContinue

Case 'P4: an expand that pushes two fences arranges the Desktop fence (desktop layout) once, after the animation'
# Titles a test-script line finds by `contains`: 'P' alone would roll Programs.
$S = @('sleep 9000', 'roll PushTop', 'sleep 4000', 'dump push')
for ($i = 0; $i -lt 4; $i++) { $S += @('unroll PushTop', 'sleep 1500', 'roll PushTop', 'sleep 1500') }
$S += @('dump end', 'sleep 3600000')
PfFresh
PfStart -Script $S
Pf fence create --title PushTop --rect '520,80,440,300' | Out-Null
Pf fence create --title PushMid --rect '520,400,440,200' | Out-Null
Pf fence create --title PushLow --rect '520,620,440,200' | Out-Null
PfWaitLog '>> roll PushTop' 40 | Out-Null
Start-Sleep -Seconds 1
Pf fence move PushMid --y 128 | Out-Null
Pf fence move PushLow --y 340 | Out-Null
PfWaitLog '>> dump end' 90 | Out-Null
Start-Sleep -Seconds 1
$L = @(PfLogAll)
$push = @(RunsAfter $L 'unroll PushTop') + @(RunsAfter $L 'roll PushTop')
$arr = @($L | Where-Object { $_ -match 'desktop fence arranged after the push animation' }).Count
$desk = @(@(PfJson fence list) | Where-Object { $_ -and $_.desktopLayout })
Check 'P4: the Desktop fence (desktop layout) is arranged like the desktop (else this case tests nothing)' ($desk.Count -ge 1) "desktopLayout fences: $($desk.Count)"
Check 'P4: each push animation arranged the Desktop fence (desktop layout) once, after it ended' ($arr -ge 6) "arranged after the run: $arr times; push runs $($push.Count)"
$q = FenceGet PushMid; $s2 = FenceGet PushLow
Check 'P4: PushMid and PushLow rest under the rolled PushTop afterwards' ($q.windowRect.y -eq 128 -and $s2.windowRect.y -eq 340) "PushMid y=$($q.windowRect.y) PushLow y=$($s2.windowRect.y)"
Log ("P4 UI work per push run ms: " + (($push | ForEach-Object work) -join ', ') + " | max gaps: " + (($push | ForEach-Object gap) -join ', '))
PfQuit

Case 'P6: a Liquid Glass title drag follows every mouse move'
PfFresh
PfStart
Pf settings set themeStyle liquidGlass | Out-Null
# Snapping off: an alignment with another fence's edge moves the window off the pointer path
# by a few pixels on purpose (it is not lag).
Pf settings set snapping.enabled false | Out-Null
Pf fence create --title G --rect '1000,60,300,180' | Out-Null
Start-Sleep -Seconds 3
$h = [G]::FindWindowW('PecoFence.Fence', 'G')
$r0 = Rect $h
$x = $r0.L + 150; $y = $r0.T + 16
[G]::MoveTo($x, $y); Start-Sleep -Milliseconds 300
MDown; Start-Sleep -Milliseconds 200
[G]::MoveTo($x, $y + 10); Start-Sleep -Milliseconds 150
$base = (Rect $h).T
$hit = 0; $seen = @()
for ($k = 1; $k -le 12; $k++) {
  [G]::MoveTo($x, $y + 10 + 15 * $k)
  Start-Sleep -Milliseconds 4
  $dy = (Rect $h).T - $base
  $seen += $dy
  if ([math]::Abs($dy - 15 * $k) -le 1) { $hit++ }
  Start-Sleep -Milliseconds 40
}
Start-Sleep -Milliseconds 200
MUp; Start-Sleep -Milliseconds 800
$r1 = Rect $h
Check 'P6: 4 ms after each mouse move the window is already where the pointer put it (11 of 12 or more)' ($hit -ge 11) "matched $hit of 12; moved by $($seen -join ',') (want 15,30,...,180)"
Check 'P6: the drop lands where the pointer let go' ([math]::Abs(($r1.T - $base) - 180) -le 2 -and [math]::Abs($r1.L - $r0.L) -le 1) "window top $($r0.T) -> $($r1.T), want $($base + 180)"
PfQuit
